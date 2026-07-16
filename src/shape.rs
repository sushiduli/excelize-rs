//! Shape support.
//!
//! Ported from Go `shape.go`.

use crate::File;
use crate::constants::{
    DEFAULT_LINE_WIDTH, DEFAULT_SHAPE_SIZE, EMU, SOURCE_RELATIONSHIP_DRAWING_ML,
};
use crate::errors::{ErrParameterInvalid, ErrTransparency, Result};
use crate::lib_util::{cell_name_to_coordinates, column_number_to_name};
use crate::styles::Font;
use crate::xml::common::{AttrValInt, AttrValString, RichTextRun};
use crate::xml::drawing::{
    ABodyPr, AEndParaRPr, AFontRef, AP, AR, ARPr, ARef, ASolidFill, ASrgbClr, GraphicOptions,
    LineOptions, LineType, Shape, XdrCNvSpPr, XdrCellAnchor, XdrClientData, XdrNvSpPr, XdrSp,
    XdrStyle, XdrTxBody, XlsxCNvPr, XlsxCTTextFont, XlsxFrom, XlsxOff, XlsxPositiveSize2D,
    XlsxPrstGeom, XlsxSpPr, XlsxTo, XlsxXfrm,
};

const SUPPORTED_DRAWING_UNDERLINE_TYPES: &[&str] = &[
    "none",
    "sng",
    "dbl",
    "heavy",
    "dotted",
    "dottedHeavy",
    "dash",
    "dashHeavy",
    "dashLong",
    "dashLongHeavy",
    "dotDash",
    "dotDashHeavy",
    "dotDotDash",
    "dotDotDashHeavy",
    "wavy",
    "wavyHeavy",
    "wavyDbl",
];

impl File {
    /// Add a shape to a worksheet.
    pub fn add_shape(&self, sheet: &str, opts: &Shape) -> Result<()> {
        let mut options = opts.clone();
        parse_shape_options(&mut options)?;

        let mut ws = self.work_sheet_reader(sheet)?;

        // Create or reuse the worksheet drawing relationship.
        let drawing_id = self.count_drawings() + 1;
        let mut drawing_xml = format!("xl/drawings/drawing{drawing_id}.xml");
        let mut sheet_rels_drawing_xml = format!("../drawings/drawing{drawing_id}.xml");

        if let Some(ref drawing) = ws.drawing {
            sheet_rels_drawing_xml = self
                .get_sheet_relationships_target_by_id(sheet, drawing.rid.as_deref().unwrap_or(""));
            drawing_xml = sheet_rels_drawing_xml.replace("..", "xl");
        } else {
            let sheet_xml_path = self.get_sheet_xml_path(sheet).unwrap_or_default();
            let sheet_rels = format!(
                "xl/worksheets/_rels/{}{}.rels",
                sheet_xml_path.strip_prefix("xl/worksheets/").unwrap_or(""),
                ""
            );
            let r_id = self.add_rels(
                &sheet_rels,
                SOURCE_RELATIONSHIP_DRAWING_ML,
                &sheet_rels_drawing_xml,
                "",
            );
            self.add_sheet_drawing(sheet, r_id)?;
            self.add_sheet_name_space(sheet, "r");
        }

        self.add_drawing_shape(sheet, &drawing_xml, &opts.cell, &options)?;
        self.add_content_type_part(drawing_id, "drawings")?;
        Ok(())
    }

    /// Create the drawing shape element and append it to the drawing part.
    fn add_drawing_shape(
        &self,
        sheet: &str,
        drawing_xml: &str,
        cell: &str,
        opts: &Shape,
    ) -> Result<()> {
        let (mut ws_dr, mut anchor, c_nv_pr_id) = self.shape_cell_anchor(
            sheet,
            drawing_xml,
            cell,
            opts.width as i32,
            opts.height as i32,
            &opts.format,
        )?;

        let shape_color = opts
            .fill
            .color
            .first()
            .map(|c| c.trim_start_matches('#').to_uppercase())
            .unwrap_or_default();
        let line_color = opts
            .line
            .fill
            .color
            .first()
            .map(|c| c.trim_start_matches('#').to_uppercase())
            .unwrap_or_default();

        let mut shape = XdrSp {
            macro_name: opts.macro_name.clone(),
            text_link: String::new(),
            nv_sp_pr: Some(XdrNvSpPr {
                c_nv_pr: Some(XlsxCNvPr {
                    id: c_nv_pr_id,
                    name: if opts.format.name.is_empty() {
                        format!("Shape {c_nv_pr_id}")
                    } else {
                        opts.format.name.clone()
                    },
                    descr: opts.format.alt_text.clone(),
                    title: None,
                    hlink_click: None,
                }),
                c_nv_sp_pr: Some(XdrCNvSpPr { tx_box: true }),
            }),
            sp_pr: Some(XlsxSpPr {
                xfrm: XlsxXfrm {
                    off: XlsxOff { x: 0, y: 0 },
                    ext: XlsxPositiveSize2D {
                        cx: opts.width as i64 * EMU as i64,
                        cy: opts.height as i64 * EMU as i64,
                    },
                },
                prst_geom: XlsxPrstGeom {
                    prst: opts.r#type.clone(),
                },
                ln: crate::chart::draw_chart_ln(&opts.line),
                ..Default::default()
            }),
            style: Some(XdrStyle {
                ln_ref: Some(shape_ref(&line_color, 2)),
                fill_ref: Some(shape_ref(&shape_color, 1)),
                effect_ref: Some(shape_ref("", 0)),
                font_ref: Some(AFontRef {
                    idx: "minor".to_string(),
                    scheme_clr: Some(AttrValString {
                        val: Some("tx1".to_string()),
                    }),
                }),
            }),
            tx_body: Some(XdrTxBody {
                body_pr: Some(ABodyPr {
                    vert_overflow: Some("clip".to_string()),
                    horz_overflow: Some("clip".to_string()),
                    wrap: Some("none".to_string()),
                    rtl_col: Some(false),
                    anchor: Some("t".to_string()),
                    ..Default::default()
                }),
                p: Vec::new(),
            }),
        };

        if opts.fill.transparency > 0 {
            let val = (100 - opts.fill.transparency) * 1000;
            if let Some(ref mut sp_pr) = shape.sp_pr {
                sp_pr.solid_fill = Some(ASolidFill {
                    srgb_clr: Some(ASrgbClr {
                        val: Some(shape_color.clone()),
                        alpha: Some(AttrValInt {
                            val: Some(val as i64),
                        }),
                        ..Default::default()
                    }),
                    ..Default::default()
                });
            }
        }

        let default_font = self
            .get_default_font()
            .unwrap_or_else(|_| "Calibri".to_string());
        let paragraphs = if opts.paragraph.is_empty() {
            vec![RichTextRun {
                font: Some(Font {
                    bold: Some(false),
                    italic: Some(false),
                    underline: Some("none".to_string()),
                    name: Some(default_font.clone()),
                    size: Some(11.0),
                    color: Some("000000".to_string()),
                    ..Default::default()
                }),
                text: " ".to_string(),
            }]
        } else {
            opts.paragraph.clone()
        };

        if let Some(ref mut tx_body) = shape.tx_body {
            for p in paragraphs {
                let font = p.font.unwrap_or_default();
                let u = if SUPPORTED_DRAWING_UNDERLINE_TYPES
                    .iter()
                    .any(|&t| t == font.underline.as_deref().unwrap_or("none"))
                {
                    font.underline.unwrap_or_else(|| "none".to_string())
                } else {
                    "none".to_string()
                };
                let text = if p.text.is_empty() {
                    " ".to_string()
                } else {
                    p.text
                };
                let srgb_clr = font
                    .color
                    .as_deref()
                    .unwrap_or("")
                    .trim_start_matches('#')
                    .to_uppercase();
                let mut r_pr = ARPr {
                    i: font.italic.unwrap_or(false),
                    b: font.bold.unwrap_or(false),
                    lang: Some("en-US".to_string()),
                    alt_lang: Some("en-US".to_string()),
                    u: Some(u),
                    sz: font.size.map(|s| s * 100.0),
                    latin: Some(XlsxCTTextFont {
                        typeface: font.name.clone().unwrap_or_else(|| default_font.clone()),
                        charset: None,
                        panose: None,
                        pitch_family: None,
                    }),
                    ea: Some(XlsxCTTextFont {
                        typeface: font.name.clone().unwrap_or_else(|| default_font.clone()),
                        charset: None,
                        panose: None,
                        pitch_family: None,
                    }),
                    cs: Some(XlsxCTTextFont {
                        typeface: font.name.clone().unwrap_or_else(|| default_font.clone()),
                        charset: None,
                        panose: None,
                        pitch_family: None,
                    }),
                    ..Default::default()
                };
                if srgb_clr.len() == 6 {
                    r_pr.solid_fill = Some(ASolidFill {
                        srgb_clr: Some(ASrgbClr {
                            val: Some(srgb_clr),
                            ..Default::default()
                        }),
                        ..Default::default()
                    });
                }
                tx_body.p.push(AP {
                    r: Some(AR {
                        r_pr: Some(r_pr),
                        t: Some(text),
                    }),
                    end_para_r_pr: Some(AEndParaRPr {
                        lang: "en-US".to_string(),
                        ..Default::default()
                    }),
                    ..Default::default()
                });
            }
        }

        anchor.sp = Some(shape);
        anchor.client_data = Some(XdrClientData {
            f_locks_with_sheet: opts.format.locked.unwrap_or(true),
            f_prints_with_sheet: opts.format.print_object.unwrap_or(true),
        });

        if opts.format.positioning == "oneCell" {
            ws_dr.one_cell_anchor.push(anchor);
        } else {
            ws_dr.two_cell_anchor.push(anchor);
        }
        self.drawings.insert(drawing_xml.to_string(), ws_dr);
        Ok(())
    }

    /// Compute the cell anchor for a shape.
    fn shape_cell_anchor(
        &self,
        sheet: &str,
        drawing_xml: &str,
        cell: &str,
        width: i32,
        height: i32,
        format: &GraphicOptions,
    ) -> Result<(crate::xml::drawing::XlsxWsDr, XdrCellAnchor, i64)> {
        let (from_col, from_row) = cell_name_to_coordinates(cell)?;
        let w = (width as f64 * format.scale_x) as i32;
        let h = (height as f64 * format.scale_y) as i32;
        let (col_start, row_start, col_end, row_end, x1, y1, x2, y2) =
            self.position_object_pixels(sheet, from_col, from_row, w, h, format)?;

        let (ws_dr, c_nv_pr_id) = self.drawing_parser(drawing_xml)?;
        let mut anchor = XdrCellAnchor::default();
        anchor.from = Some(XlsxFrom {
            col: col_start as i64,
            col_off: x1 as i64 * EMU as i64,
            row: row_start as i64,
            row_off: y1 as i64 * EMU as i64,
        });

        if format.positioning != "oneCell" {
            anchor.to = Some(XlsxTo {
                col: col_end as i64,
                col_off: x2 as i64 * EMU as i64,
                row: row_end as i64,
                row_off: y2 as i64 * EMU as i64,
            });
            anchor.edit_as = Some(format.positioning.clone());
        } else {
            anchor.ext = Some(XlsxPositiveSize2D {
                cx: x2 as i64 * EMU as i64,
                cy: y2 as i64 * EMU as i64,
            });
        }

        Ok((ws_dr, anchor, c_nv_pr_id))
    }
}

fn parse_shape_options(opts: &mut Shape) -> Result<()> {
    if opts.r#type.is_empty() {
        return Err(Box::new(ErrParameterInvalid));
    }
    if opts.width == 0 {
        opts.width = DEFAULT_SHAPE_SIZE as u64;
    }
    if opts.height == 0 {
        opts.height = DEFAULT_SHAPE_SIZE as u64;
    }
    if opts.line.width == 0.0 {
        opts.line.width = DEFAULT_LINE_WIDTH as f64;
    }
    if !(0..=100).contains(&opts.fill.transparency) {
        return Err(Box::new(ErrTransparency));
    }
    if opts.format.scale_x == 0.0 {
        opts.format.scale_x = 1.0;
    }
    if opts.format.scale_y == 0.0 {
        opts.format.scale_y = 1.0;
    }
    Ok(())
}

fn shape_ref(color: &str, idx: i64) -> ARef {
    if color.is_empty() {
        ARef {
            idx: 0,
            scrgb_clr: Some(crate::xml::drawing::AScrgbClr {
                r: 0.0,
                g: 0.0,
                b: 0.0,
            }),
            ..Default::default()
        }
    } else {
        ARef {
            idx,
            srgb_clr: Some(AttrValString {
                val: Some(color.to_string()),
            }),
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::options::Options;
    use crate::xml::drawing::Fill;

    #[test]
    fn add_shape_creates_drawing_part() {
        let f = File::new_with_options(Options::default());
        f.add_shape(
            "Sheet1",
            &Shape {
                cell: "G6".to_string(),
                r#type: "rect".to_string(),
                width: 180,
                height: 40,
                fill: Fill {
                    r#type: "pattern".to_string(),
                    pattern: 1,
                    color: vec!["8EB9FF".to_string()],
                    ..Default::default()
                },
                line: LineOptions {
                    width: 1.2,
                    fill: Fill {
                        r#type: "pattern".to_string(),
                        pattern: 1,
                        color: vec!["4286F4".to_string()],
                        ..Default::default()
                    },
                    ..Default::default()
                },
                paragraph: vec![RichTextRun {
                    text: "Rectangle Shape".to_string(),
                    font: Some(Font {
                        bold: Some(true),
                        italic: Some(true),
                        name: Some("Times New Roman".to_string()),
                        size: Some(18.0),
                        color: Some("777777".to_string()),
                        underline: Some("sng".to_string()),
                        ..Default::default()
                    }),
                }],
                ..Default::default()
            },
        )
        .unwrap();

        let ws = f.work_sheet_reader("Sheet1").unwrap();
        assert!(ws.drawing.is_some());

        let target = f.get_sheet_relationships_target_by_id(
            "Sheet1",
            ws.drawing.as_ref().unwrap().rid.as_deref().unwrap(),
        );
        let drawing_xml = target.replace("..", "xl");
        let (ws_dr, _) = f.drawing_parser(&drawing_xml).unwrap();
        assert_eq!(ws_dr.two_cell_anchor.len(), 1);

        let anchor = &ws_dr.two_cell_anchor[0];
        assert_eq!(anchor.from.as_ref().unwrap().col, 6); // G = 7th col, 0-based = 6
        assert_eq!(anchor.from.as_ref().unwrap().row, 5); // row 6, 0-based = 5
        let sp = anchor.sp.as_ref().unwrap();
        assert_eq!(sp.sp_pr.as_ref().unwrap().prst_geom.prst, "rect");
    }
}
