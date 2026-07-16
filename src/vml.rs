//! VML-based features: comments, form controls, and header/footer images.
//!
//! Ported from Go `vml.go`, `vmlDrawing.go`, and `picture.go`.

use std::borrow::Cow;
use std::collections::HashMap;

use quick_xml::Reader;
use quick_xml::events::Event;

use crate::constants::{
    MAX_FIELD_LENGTH, MAX_FORM_CONTROL_VALUE, SOURCE_RELATIONSHIP, SOURCE_RELATIONSHIP_COMMENTS,
    SOURCE_RELATIONSHIP_DRAWING_VML, SOURCE_RELATIONSHIP_IMAGE, TOTAL_CELL_CHARS,
};
use crate::errors::{ErrParameterInvalid, new_add_comment_error, new_invalid_optional_value};
use crate::file::File;
use crate::lib_util::{
    cell_name_to_coordinates, coordinates_to_cell_name, count_utf16_string, in_str_slice,
    truncate_utf16_units,
};
use crate::xml::common::{
    AttrValBool, AttrValFloat, AttrValInt, AttrValString, RichTextRun, XlsxColor, XlsxR, XlsxRPr,
    XlsxT,
};
use crate::xml::drawing::GraphicOptions;
use crate::xml::vml::{
    VmlDrawing, VmlFormula, VmlFormulas, VmlIdmap, VmlImageData, VmlLock, VmlPath, VmlShape,
    VmlShapeLayout, VmlShapeType, VmlStroke,
};

// ------------------------------------------------------------------
// Public types
// ------------------------------------------------------------------

pub use crate::xml::comments::Comment;

/// Supported form control types.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum FormControlType {
    /// Comment indicator (note).
    #[default]
    Note,
    /// Button.
    Button,
    /// Option button / radio button.
    OptionButton,
    /// Spin button / spinner.
    SpinButton,
    /// Check box.
    CheckBox,
    /// Group box.
    GroupBox,
    /// Label.
    Label,
    /// Scroll bar.
    ScrollBar,
}

/// Header/footer image horizontal position.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum HeaderFooterImagePositionType {
    /// Left section.
    #[default]
    Left,
    /// Center section.
    Center,
    /// Right section.
    Right,
}

/// Form control options.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct FormControl {
    pub cell: String,
    pub macro_name: String,
    pub width: u32,
    pub height: u32,
    pub checked: bool,
    pub current_val: u32,
    pub min_val: u32,
    pub max_val: u32,
    pub inc_change: u32,
    pub page_change: u32,
    pub horizontally: bool,
    pub cell_link: String,
    pub text: String,
    pub paragraph: Vec<RichTextRun>,
    pub r#type: FormControlType,
    pub format: GraphicOptions,
}

/// Header/footer image options.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct HeaderFooterImageOptions {
    pub position: HeaderFooterImagePositionType,
    pub file: Vec<u8>,
    pub is_footer: bool,
    pub first_page: bool,
    pub extension: String,
    pub width: String,
    pub height: String,
}

// ------------------------------------------------------------------
// Public API
// ------------------------------------------------------------------

impl File {
    /// Retrieve all comments in a worksheet.
    pub fn get_comments(&self, sheet: &str) -> crate::errors::Result<Vec<Comment>> {
        let mut comments = Vec::new();
        let sheet_xml_path = self.get_sheet_xml_path(sheet).ok_or_else(|| {
            Box::new(crate::errors::ErrSheetNotExist {
                sheet_name: sheet.to_string(),
            }) as Box<dyn std::error::Error + Send + Sync>
        })?;
        let mut comments_xml = self.get_sheet_comments(&sheet_xml_path);
        if !comments_xml.starts_with('/') {
            comments_xml = format!("xl{}", comments_xml.trim_start_matches(".."));
        }
        let comments_xml = comments_xml.trim_start_matches('/').to_string();
        let cmts = self.comments_reader(&comments_xml)?;
        if let Some(cmts) = cmts {
            for cmt in &cmts.comment_list.comment {
                let mut comment = Comment::default();
                if (cmt.author_id as usize) < cmts.authors.author.len() {
                    comment.author = cmts.authors.author[cmt.author_id as usize].clone();
                }
                comment.cell = cmt.r#ref.clone();
                comment.author_id = cmt.author_id;
                if let Some(t) = &cmt.text.t {
                    comment.text.push_str(t);
                }
                for run in &cmt.text.r {
                    if let Some(t) = &run.t {
                        let mut rtr = RichTextRun {
                            text: t.val.clone(),
                            ..Default::default()
                        };
                        if let Some(rpr) = &run.r_pr {
                            rtr.font = Some(rpr_to_font(rpr));
                        }
                        comment.paragraph.push(rtr);
                    }
                }
                comments.push(comment);
            }
        }
        Ok(comments)
    }

    /// Add a comment to a worksheet.
    pub fn add_comment(&self, sheet: &str, opts: Comment) -> crate::errors::Result<()> {
        self.add_vml_object(VmlOptions {
            sheet: sheet.to_string(),
            comment: Some(opts.clone()),
            form_control: Some(FormControl {
                cell: opts.cell.clone(),
                text: opts.text.clone(),
                paragraph: opts.paragraph.clone(),
                width: opts.width,
                height: opts.height,
                r#type: FormControlType::Note,
                ..Default::default()
            }),
            ..Default::default()
        })
    }

    /// Delete a comment in a worksheet by cell reference.
    pub fn delete_comment(&self, sheet: &str, cell: &str) -> crate::errors::Result<()> {
        let ws = self.work_sheet_reader(sheet)?;
        if ws.legacy_drawing.is_none() {
            return Ok(());
        }
        let sheet_xml_path = self.get_sheet_xml_path(sheet).unwrap_or_default();
        let mut comments_xml = self.get_sheet_comments(&sheet_xml_path);
        if !comments_xml.starts_with('/') {
            comments_xml = format!("xl{}", comments_xml.trim_start_matches(".."));
        }
        let comments_xml = comments_xml.trim_start_matches('/').to_string();
        if let Some(cmts) = self.comments_reader(&comments_xml)? {
            let mut cmts = cmts;
            cmts.comment_list.comment.retain(|c| c.r#ref != cell);
            cmts.cells.retain(|c| c != cell);
            self.comments.insert(comments_xml, cmts);
        }
        let rid = ws
            .legacy_drawing
            .as_ref()
            .and_then(|d| d.rid.clone())
            .unwrap_or_default();
        let sheet_relationships_drawing_vml =
            self.get_sheet_relationships_target_by_id(sheet, &rid);
        self.delete_vml_shape(&sheet_relationships_drawing_vml, cell, true)
    }

    /// Add a form control to a worksheet.
    pub fn add_form_control(&self, sheet: &str, opts: FormControl) -> crate::errors::Result<()> {
        self.add_vml_object(VmlOptions {
            sheet: sheet.to_string(),
            form_control: Some(opts),
            ..Default::default()
        })
    }

    /// Delete a form control in a worksheet by cell reference.
    pub fn delete_form_control(&self, sheet: &str, cell: &str) -> crate::errors::Result<()> {
        let ws = self.work_sheet_reader(sheet)?;
        if ws.legacy_drawing.is_none() {
            return Ok(());
        }
        let rid = ws
            .legacy_drawing
            .as_ref()
            .and_then(|d| d.rid.clone())
            .unwrap_or_default();
        let sheet_relationships_drawing_vml =
            self.get_sheet_relationships_target_by_id(sheet, &rid);
        self.delete_vml_shape(&sheet_relationships_drawing_vml, cell, false)
    }

    /// Retrieve all form controls in a worksheet.
    pub fn get_form_controls(&self, sheet: &str) -> crate::errors::Result<Vec<FormControl>> {
        let mut form_controls = Vec::new();
        let ws = self.work_sheet_reader(sheet)?;
        if ws.legacy_drawing.is_none() {
            return Ok(form_controls);
        }
        let rid = ws
            .legacy_drawing
            .as_ref()
            .and_then(|d| d.rid.clone())
            .unwrap_or_default();
        let target = self.get_sheet_relationships_target_by_id(sheet, &rid);
        let drawing_vml = target.replace("..", "xl");
        let vml = self.vml_drawing_reader(&drawing_vml)?.unwrap_or_default();
        for sp in &vml.shape {
            if sp.shape_type != "#_x0000_t201" {
                continue;
            }
            let fc = extract_form_control(&sp.inner_xml)?;
            if fc.r#type == FormControlType::Note || fc.cell.is_empty() {
                continue;
            }
            form_controls.push(fc);
        }
        Ok(form_controls)
    }

    /// Add an image that can be referenced from the worksheet header/footer.
    pub fn add_header_footer_image(
        &self,
        sheet: &str,
        opts: &HeaderFooterImageOptions,
    ) -> crate::errors::Result<()> {
        let ws = self.work_sheet_reader(sheet)?;
        let ext = opts.extension.to_lowercase();
        let image_types = supported_image_types();
        if !image_types.contains_key(&ext) {
            return Err(Box::new(crate::errors::ErrImgExt));
        }
        let sheet_id = self.get_sheet_id(sheet);
        let vml_id = self.count_vml_drawing() + 1;
        let drawing_vml = format!("xl/drawings/vmlDrawing{vml_id}.vml");
        let sheet_relationships_drawing_vml = format!("../drawings/vmlDrawing{vml_id}.vml");
        let sheet_xml_path = self.get_sheet_xml_path(sheet).unwrap_or_default();
        let sheet_rels = format!(
            "xl/worksheets/_rels/{}.rels",
            sheet_xml_path.trim_start_matches("xl/worksheets/")
        );

        let (vml_id, drawing_vml, _sheet_relationships_drawing_vml) =
            if let Some(ld) = ws.legacy_drawing_hf.as_ref() {
                let target = self
                    .get_sheet_relationships_target_by_id(sheet, ld.rid.as_deref().unwrap_or(""));
                let id: i32 = target
                    .trim_start_matches("../drawings/vmlDrawing")
                    .trim_end_matches(".vml")
                    .parse()
                    .unwrap_or(vml_id);
                (id, target.replace("..", "xl"), target)
            } else {
                let r_id = self.add_rels(
                    &sheet_rels,
                    SOURCE_RELATIONSHIP_DRAWING_VML,
                    &sheet_relationships_drawing_vml,
                    "",
                );
                self.add_sheet_name_space(sheet, SOURCE_RELATIONSHIP);
                self.add_sheet_legacy_drawing_hf(sheet, r_id)?;
                (vml_id, drawing_vml, sheet_relationships_drawing_vml)
            };

        let mut vml = self
            .vml_drawing_reader(&drawing_vml)?
            .unwrap_or_else(|| default_header_footer_vml_drawing(sheet_id));

        let shape_id = format!(
            "{}{}{}",
            match opts.position {
                HeaderFooterImagePositionType::Left => "L",
                HeaderFooterImagePositionType::Center => "C",
                HeaderFooterImagePositionType::Right => "R",
            },
            if opts.is_footer { "F" } else { "H" },
            if opts.first_page { "FIRST" } else { "" }
        );

        vml.shape.retain(|s| s.id != shape_id);

        let style = format!(
            "position:absolute;margin-left:0;margin-top:0;width:{};height:{};z-index:1",
            opts.width, opts.height
        );
        let drawing_vml_rels = format!("xl/drawings/_rels/vmlDrawing{vml_id}.vml.rels");
        let media = self.add_media(&opts.file, &ext);
        let media_str = format!("..{}", media.trim_start_matches("xl"));
        let image_id = self.add_rels(&drawing_vml_rels, SOURCE_RELATIONSHIP_IMAGE, &media_str, "");

        let sp = EncodeShape {
            image_data: Some(VmlImageData {
                rel_id: Some(format!("rId{image_id}")),
                ..Default::default()
            }),
            lock: Some(VmlLock {
                ext: "edit".to_string(),
                rotation: Some("t".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };
        let inner = build_shape_inner_xml(&sp, &FormCtrlPreset::default(), &VmlOptions::default());
        vml.shape.push(VmlShape {
            id: shape_id,
            spid: Some("_x0000_s1025".to_string()),
            shape_type: "#_x0000_t75".to_string(),
            style,
            inner_xml: inner,
            ..Default::default()
        });
        self.vml_drawing.insert(drawing_vml, vml);

        crate::sheet::set_content_type_part_image_extensions(self)?;
        self.set_content_type_part_vml_extensions()
    }
}

// ------------------------------------------------------------------
// Internal options bundle
// ------------------------------------------------------------------

#[derive(Debug, Default, Clone)]
struct VmlOptions {
    sheet: String,
    comment: Option<Comment>,
    form_control: Option<FormControl>,
}

impl VmlOptions {
    fn is_form_control(&self) -> bool {
        self.form_control
            .as_ref()
            .map(|f| f.r#type != FormControlType::Note)
            .unwrap_or(false)
    }

    fn form_control(&self) -> &FormControl {
        self.form_control.as_ref().unwrap()
    }

    fn comment(&self) -> &Comment {
        self.comment.as_ref().unwrap()
    }
}

// ------------------------------------------------------------------
// Internal helpers
// ------------------------------------------------------------------

impl File {
    fn get_sheet_comments(&self, sheet_xml_path: &str) -> String {
        let sheet_file = std::path::Path::new(sheet_xml_path)
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        let rels = self
            .rels_reader(&format!("xl/worksheets/_rels/{sheet_file}.rels"))
            .unwrap_or_default();
        if let Some(rels) = rels {
            for rel in &rels.relationships {
                if rel.r#type == SOURCE_RELATIONSHIP_COMMENTS {
                    return rel.target.clone();
                }
            }
        }
        String::new()
    }

    fn delete_vml_shape(
        &self,
        sheet_relationships_drawing_vml: &str,
        cell: &str,
        is_comment: bool,
    ) -> crate::errors::Result<()> {
        let (col, row) = cell_name_to_coordinates(cell)
            .map_err(|e| Box::<dyn std::error::Error + Send + Sync>::from(e))?;
        let vml_id: i32 = sheet_relationships_drawing_vml
            .trim_start_matches("../drawings/vmlDrawing")
            .trim_end_matches(".vml")
            .parse()
            .unwrap_or(0);
        let drawing_vml = sheet_relationships_drawing_vml.replace("..", "xl");
        let mut vml = self
            .vml_drawing_reader(&drawing_vml)?
            .unwrap_or_else(|| VmlDrawing {
                xmlns_v: "urn:schemas-microsoft-com:vml".to_string(),
                xmlns_o: "urn:schemas-microsoft-com:office:office".to_string(),
                xmlns_x: "urn:schemas-microsoft-com:office:excel".to_string(),
                xmlns_mv: Some("http://macVmlSchemaUri".to_string()),
                shape_layout: Some(VmlShapeLayout {
                    ext: "edit".to_string(),
                    idmap: Some(VmlIdmap {
                        ext: "edit".to_string(),
                        data: vml_id,
                    }),
                }),
                shape_type: Some(VmlShapeType {
                    stroke: Some(VmlStroke {
                        join_style: "miter".to_string(),
                    }),
                    v_path: Some(VmlPath {
                        gradient_shape_ok: Some("t".to_string()),
                        connect_type: "rect".to_string(),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            });

        let cond = |object_type: &str| {
            if is_comment {
                object_type == "Note"
            } else {
                object_type != "Note"
            }
        };
        let mut removed = false;
        vml.shape.retain(|sp| {
            if removed {
                return true;
            }
            let object_type = extract_object_type(&sp.inner_xml);
            let anchor = extract_anchor(&sp.inner_xml);
            if cond(&object_type) && !anchor.is_empty() {
                if let Ok((left_col, top_row)) = extract_anchor_cell(&anchor) {
                    if left_col == col - 1 && top_row == row - 1 {
                        removed = true;
                        return false;
                    }
                }
            }
            true
        });
        self.vml_drawing.insert(drawing_vml, vml);
        Ok(())
    }

    fn add_vml_object(&self, opts: VmlOptions) -> crate::errors::Result<()> {
        let ws = self.work_sheet_reader(&opts.sheet)?;
        let mut vml_id = self.count_comments() + 1;
        if opts.is_form_control() {
            if let Some(fc) = &opts.form_control {
                if fc.r#type as u8 > FormControlType::ScrollBar as u8 {
                    return Err(Box::new(ErrParameterInvalid));
                }
            }
            vml_id = self.count_vml_drawing() + 1;
        }
        let sheet_id = self.get_sheet_id(&opts.sheet);
        let drawing_vml = format!("xl/drawings/vmlDrawing{vml_id}.vml");
        let sheet_relationships_drawing_vml = format!("../drawings/vmlDrawing{vml_id}.vml");
        let sheet_xml_path = self.get_sheet_xml_path(&opts.sheet).unwrap_or_default();
        let sheet_rels = format!(
            "xl/worksheets/_rels/{}.rels",
            sheet_xml_path.trim_start_matches("xl/worksheets/")
        );

        let (vml_id, drawing_vml, _sheet_relationships_drawing_vml) = if let Some(ld) =
            ws.legacy_drawing.as_ref()
        {
            let target = self
                .get_sheet_relationships_target_by_id(&opts.sheet, ld.rid.as_deref().unwrap_or(""));
            let id: i32 = target
                .trim_start_matches("../drawings/vmlDrawing")
                .trim_end_matches(".vml")
                .parse()
                .unwrap_or(vml_id);
            (id, target.replace("..", "xl"), target)
        } else {
            let r_id = self.add_rels(
                &sheet_rels,
                SOURCE_RELATIONSHIP_DRAWING_VML,
                &sheet_relationships_drawing_vml,
                "",
            );
            self.add_sheet_name_space(&opts.sheet, SOURCE_RELATIONSHIP);
            self.add_sheet_legacy_drawing(&opts.sheet, r_id)?;
            (vml_id, drawing_vml, sheet_relationships_drawing_vml)
        };

        let opts = prepare_form_ctrl_options(opts);
        self.add_drawing_vml(sheet_id, &drawing_vml, &opts)?;
        if !opts.is_form_control() {
            let comments_xml = format!("xl/comments{vml_id}.xml");
            self.add_comment_internal(&comments_xml, &opts)?;
            if self.get_sheet_comments(&sheet_xml_path).is_empty() {
                let sheet_relationships_comments = format!("../comments{vml_id}.xml");
                self.add_rels(
                    &sheet_rels,
                    SOURCE_RELATIONSHIP_COMMENTS,
                    &sheet_relationships_comments,
                    "",
                );
            }
        }
        self.add_content_type_part(vml_id, "comments")
    }

    fn add_comment_internal(
        &self,
        comments_xml: &str,
        opts: &VmlOptions,
    ) -> crate::errors::Result<()> {
        let mut author = opts.comment().author.clone();
        if author.is_empty() {
            author = "Author".to_string();
        }
        if count_utf16_string(&author) > MAX_FIELD_LENGTH {
            author = truncate_utf16_units(&author, MAX_FIELD_LENGTH);
        }
        let mut cmts = self.comments_reader(comments_xml)?.unwrap_or_else(|| {
            let mut c = crate::xml::comments::XlsxComments::default();
            c.authors.author.push(author.clone());
            c
        });
        if in_str_slice(&cmts.cells, &opts.comment().cell, true) != -1 {
            return Err(new_add_comment_error(&opts.comment().cell).into());
        }
        let mut author_id = in_str_slice(&cmts.authors.author, &author, true);
        if author_id == -1 {
            cmts.authors.author.push(author.clone());
            author_id = cmts.authors.author.len() as i32 - 1;
        }
        let default_font = self.get_default_font()?;
        let mut chars = 0usize;
        let mut cmt = crate::xml::comments::XlsxComment {
            r#ref: opts.comment().cell.clone(),
            author_id,
            text: crate::xml::comments::XlsxText {
                r: Vec::new(),
                ..Default::default()
            },
        };
        if !opts.comment().text.is_empty() {
            let mut text = opts.comment().text.clone();
            if count_utf16_string(&text) > TOTAL_CELL_CHARS {
                text = truncate_utf16_units(&text, TOTAL_CELL_CHARS);
            }
            cmt.text.t = Some(text);
            chars += count_utf16_string(&opts.comment().text);
        }
        for run in &opts.comment().paragraph {
            if chars == TOTAL_CELL_CHARS {
                break;
            }
            let mut text = run.text.clone();
            if chars + count_utf16_string(&text) > TOTAL_CELL_CHARS {
                text = truncate_utf16_units(&text, TOTAL_CELL_CHARS - chars);
            }
            chars += count_utf16_string(&text);
            let mut r = XlsxR {
                r_pr: Some(XlsxRPr {
                    sz: Some(AttrValFloat { val: Some(9.0) }),
                    color: Some(XlsxColor {
                        indexed: Some(81),
                        ..Default::default()
                    }),
                    r_font: Some(AttrValString {
                        val: Some(default_font.clone()),
                    }),
                    family: Some(AttrValInt { val: Some(2) }),
                    ..Default::default()
                }),
                t: Some(XlsxT {
                    space: Some("preserve".to_string()),
                    val: text,
                }),
            };
            if let Some(font) = &run.font {
                r.r_pr = Some(font_to_rpr(font));
            }
            cmt.text.r.push(r);
        }
        cmts.comment_list.comment.push(cmt);
        cmts.cells.push(opts.comment().cell.clone());
        self.comments.insert(comments_xml.to_string(), cmts);
        Ok(())
    }

    fn add_drawing_vml(
        &self,
        sheet_id: i32,
        drawing_vml: &str,
        opts: &VmlOptions,
    ) -> crate::errors::Result<()> {
        let cell = &opts.form_control().cell;
        let (col, row) = cell_name_to_coordinates(cell)
            .map_err(|e| Box::<dyn std::error::Error + Send + Sync>::from(e))?;
        let (left_offset, vml_id) = if opts.is_form_control() {
            (0, 201)
        } else {
            (23, 202)
        };
        let style = if opts.is_form_control() {
            "position:absolute;73.5pt;width:108pt;height:59.25pt;z-index:1;mso-wrap-style:tight"
                .to_string()
        } else {
            "position:absolute;73.5pt;width:108pt;height:59.25pt;z-index:1;visibility:hidden"
                .to_string()
        };
        let fc = opts.form_control();
        let format = &fc.format;
        let (col_start, row_start, col_end, row_end, _x1, _y1, x2, y2) = self
            .position_object_pixels(
                &opts.sheet,
                col,
                row,
                fc.width as i32,
                fc.height as i32,
                format,
            )?;
        let anchor = format!(
            "{}, {}, {}, 0, {}, {}, {}, {}",
            col_start, left_offset, row_start, col_end, x2, row_end, y2
        );

        let mut vml = self
            .vml_drawing_reader(drawing_vml)?
            .unwrap_or_else(|| default_vml_drawing(sheet_id, vml_id));

        let preset = form_ctrl_preset(fc.r#type);
        let sp = self.add_form_ctrl_shape(&preset, col, row, &anchor, opts)?;
        let inner_xml = build_shape_inner_xml(&sp, &preset, opts);

        vml.shape.push(VmlShape {
            id: "_x0000_s1025".to_string(),
            shape_type: format!("#_x0000_t{vml_id}"),
            style,
            button: preset.stroke_button.clone(),
            filled: preset.filled.clone(),
            fill_color: preset.fill_color.clone(),
            stroked: preset.stroked.clone(),
            stroke_color: preset.stroke_color.clone(),
            inner_xml,
            ..Default::default()
        });
        self.vml_drawing.insert(drawing_vml.to_string(), vml);
        Ok(())
    }

    fn add_form_ctrl_shape(
        &self,
        preset: &FormCtrlPreset,
        col: i32,
        row: i32,
        anchor: &str,
        opts: &VmlOptions,
    ) -> crate::errors::Result<EncodeShape> {
        let fc = opts.form_control();
        let mut sp = EncodeShape {
            fill: preset.fill.clone(),
            shadow: preset.shadow.clone(),
            path: Some(VmlPath {
                connect_type: "none".to_string(),
                ..Default::default()
            }),
            text_box: Some(VmlTextBox {
                style: "mso-direction-alt:auto".to_string(),
                div_style: "text-align:left".to_string(),
                font: Vec::new(),
            }),
            image_data: None,
            client_data: VmlClientData {
                object_type: preset.object_type.clone(),
                anchor: anchor.to_string(),
                auto_fill: preset.auto_fill.clone(),
                row: Some(row - 1),
                column: Some(col - 1),
                text_h_align: preset.text_h_align.clone(),
                text_v_align: preset.text_v_align.clone(),
                no_three_d: preset.no_three_d.clone(),
                first_button: preset.first_button.clone(),
                ..Default::default()
            },
            lock: None,
        };
        if let Some(po) = fc.format.print_object {
            if !po {
                sp.client_data.print_object = Some("False".to_string());
            }
        }
        if !fc.format.positioning.is_empty() {
            let supported = ["absolute", "oneCell", "twoCell"];
            let idx = in_str_slice(&supported, &fc.format.positioning, true);
            if idx == -1 {
                return Err(new_invalid_optional_value(
                    "Positioning",
                    &fc.format.positioning,
                    &["absolute", "oneCell", "twoCell"],
                )
                .into());
            }
            let idx = idx as usize;
            sp.client_data.move_with_cells = [Some("".to_string()), None, None][idx].clone();
            sp.client_data.size_with_cells =
                [Some("".to_string()), Some("".to_string()), None][idx].clone();
        }
        if fc.r#type == FormControlType::Note {
            sp.client_data.move_with_cells = Some("".to_string());
            sp.client_data.size_with_cells = Some("".to_string());
        }
        if !opts.is_form_control() {
            return Ok(sp);
        }
        if let Some(tb) = sp.text_box.as_mut() {
            tb.font = form_ctrl_text(fc);
        }
        sp.client_data.fmla_macro.clone_from(&fc.macro_name);
        if (fc.r#type == FormControlType::CheckBox || fc.r#type == FormControlType::OptionButton)
            && fc.checked
        {
            sp.client_data.checked = 1;
        }
        if fc.r#type == FormControlType::CheckBox {
            sp.client_data.fmla_link.clone_from(&fc.cell_link);
        }
        self.add_form_ctrl_values(&mut sp, fc)?;
        Ok(sp)
    }

    fn add_form_ctrl_values(
        &self,
        sp: &mut EncodeShape,
        fc: &FormControl,
    ) -> crate::errors::Result<()> {
        if fc.r#type != FormControlType::ScrollBar && fc.r#type != FormControlType::SpinButton {
            return Ok(());
        }
        if fc.current_val > MAX_FORM_CONTROL_VALUE as u32
            || fc.min_val > MAX_FORM_CONTROL_VALUE as u32
            || fc.max_val > MAX_FORM_CONTROL_VALUE as u32
            || fc.inc_change > MAX_FORM_CONTROL_VALUE as u32
            || fc.page_change > MAX_FORM_CONTROL_VALUE as u32
        {
            return Err(Box::new(crate::errors::ErrFormControlValue));
        }
        if !fc.cell_link.is_empty() {
            cell_name_to_coordinates(&fc.cell_link)
                .map_err(|e| Box::<dyn std::error::Error + Send + Sync>::from(e))?;
        }
        sp.client_data.fmla_link.clone_from(&fc.cell_link);
        sp.client_data.val = fc.current_val;
        sp.client_data.min = fc.min_val;
        sp.client_data.max = fc.max_val;
        sp.client_data.inc = fc.inc_change;
        sp.client_data.page = fc.page_change;
        if fc.r#type == FormControlType::ScrollBar {
            if fc.horizontally {
                sp.client_data.horiz = Some("".to_string());
            }
            sp.client_data.dx = 15;
        }
        Ok(())
    }
}

// ------------------------------------------------------------------
// VML shape inner XML builders
// ------------------------------------------------------------------

#[derive(Debug, Default, Clone, PartialEq)]
struct EncodeShape {
    fill: Option<VmlFill>,
    shadow: Option<VmlShadow>,
    path: Option<VmlPath>,
    text_box: Option<VmlTextBox>,
    image_data: Option<VmlImageData>,
    client_data: VmlClientData,
    lock: Option<VmlLock>,
}

#[derive(Debug, Default, Clone, PartialEq)]
struct VmlFill {
    angle: i32,
    color2: String,
    r#type: String,
    fill: Option<VmlOFill>,
}

#[derive(Debug, Default, Clone, PartialEq)]
struct VmlOFill {
    ext: String,
    r#type: String,
}

#[derive(Debug, Default, Clone, PartialEq)]
struct VmlShadow {
    on: String,
    color: String,
    obscured: String,
}

#[derive(Debug, Default, Clone, PartialEq)]
struct VmlTextBox {
    style: String,
    div_style: String,
    font: Vec<VmlFont>,
}

#[derive(Debug, Default, Clone, PartialEq)]
struct VmlFont {
    face: Option<String>,
    size: Option<u32>,
    color: Option<String>,
    content: String,
}

#[derive(Debug, Default, Clone, PartialEq)]
struct VmlClientData {
    object_type: String,
    anchor: String,
    move_with_cells: Option<String>,
    size_with_cells: Option<String>,
    locked: Option<String>,
    print_object: Option<String>,
    auto_fill: String,
    fmla_macro: String,
    text_h_align: String,
    text_v_align: String,
    row: Option<i32>,
    column: Option<i32>,
    checked: u32,
    fmla_link: String,
    no_three_d: Option<String>,
    first_button: Option<String>,
    val: u32,
    min: u32,
    max: u32,
    inc: u32,
    page: u32,
    horiz: Option<String>,
    dx: u32,
}

fn build_shape_inner_xml(sp: &EncodeShape, preset: &FormCtrlPreset, opts: &VmlOptions) -> String {
    let mut parts = Vec::new();
    if let Some(fill) = &sp.fill {
        let mut s = format!(
            "<v:fill angle=\"{}\" color2=\"{}\"",
            fill.angle, fill.color2
        );
        if !fill.r#type.is_empty() {
            s.push_str(&format!(" type=\"{}\"", fill.r#type));
        }
        if let Some(of) = &fill.fill {
            s.push_str(&format!(
                ">\n        <o:fill ext=\"{}\" type=\"{}\"/>\n      </v:fill>",
                of.ext, of.r#type
            ));
        } else {
            s.push_str("/>");
        }
        parts.push(s);
    }
    if let Some(shadow) = &sp.shadow {
        parts.push(format!(
            "<v:shadow on=\"{}\" color=\"{}\" obscured=\"{}\"/>",
            shadow.on, shadow.color, shadow.obscured
        ));
    }
    if let Some(path) = &sp.path {
        parts.push(format!(
            "<v:path o:connecttype=\"{}\"{}/>",
            path.connect_type,
            path.gradient_shape_ok
                .as_ref()
                .map(|v| format!(" gradientshapeok=\"{v}\""))
                .unwrap_or_default()
        ));
    }
    if let Some(tb) = &sp.text_box {
        let mut font_xml = String::new();
        for font in &tb.font {
            let mut start = String::from("<font");
            if let Some(face) = &font.face {
                start.push_str(&format!(" face=\"{}\"", html_escape(face)));
            }
            if let Some(size) = font.size {
                start.push_str(&format!(" size=\"{}\"", size));
            }
            if let Some(color) = &font.color {
                start.push_str(&format!(" color=\"{}\"", html_escape(color)));
            }
            start.push('>');
            font_xml.push_str(&start);
            font_xml.push_str(&font.content);
            font_xml.push_str("</font>");
        }
        parts.push(format!(
            "<v:textbox style=\"{}\">\n        <div style=\"{}\">{}</div>\n      </v:textbox>",
            tb.style, tb.div_style, font_xml
        ));
    }
    if let Some(image_data) = &sp.image_data {
        parts.push(image_data.to_xml_string());
    }
    parts.push(build_client_data_xml(&sp.client_data, preset, opts));
    if let Some(lock) = &sp.lock {
        parts.push(lock.to_xml_string());
    }
    parts.join("\n      ")
}

fn build_client_data_xml(
    cd: &VmlClientData,
    _preset: &FormCtrlPreset,
    _opts: &VmlOptions,
) -> String {
    let mut s = format!("<x:ClientData ObjectType=\"{}\"", cd.object_type);
    s.push('>');
    if let Some(v) = &cd.move_with_cells {
        s.push_str(&format!(
            "\n        <x:MoveWithCells>{}</x:MoveWithCells>",
            v
        ));
    }
    if let Some(v) = &cd.size_with_cells {
        s.push_str(&format!(
            "\n        <x:SizeWithCells>{}</x:SizeWithCells>",
            v
        ));
    }
    s.push_str(&format!("\n        <x:Anchor>{}</x:Anchor>", cd.anchor));
    if let Some(v) = &cd.locked {
        s.push_str(&format!("\n        <x:Locked>{}</x:Locked>", v));
    }
    if let Some(v) = &cd.print_object {
        s.push_str(&format!("\n        <x:PrintObject>{}</x:PrintObject>", v));
    }
    if !cd.auto_fill.is_empty() {
        s.push_str(&format!(
            "\n        <x:AutoFill>{}</x:AutoFill>",
            cd.auto_fill
        ));
    }
    if !cd.fmla_macro.is_empty() {
        s.push_str(&format!(
            "\n        <x:FmlaMacro>{}</x:FmlaMacro>",
            cd.fmla_macro
        ));
    }
    if !cd.text_h_align.is_empty() {
        s.push_str(&format!(
            "\n        <x:TextHAlign>{}</x:TextHAlign>",
            cd.text_h_align
        ));
    }
    if !cd.text_v_align.is_empty() {
        s.push_str(&format!(
            "\n        <x:TextVAlign>{}</x:TextVAlign>",
            cd.text_v_align
        ));
    }
    if let Some(v) = cd.row {
        s.push_str(&format!("\n        <x:Row>{}</x:Row>", v));
    }
    if let Some(v) = cd.column {
        s.push_str(&format!("\n        <x:Column>{}</x:Column>", v));
    }
    if cd.checked != 0 {
        s.push_str(&format!("\n        <x:Checked>{}</x:Checked>", cd.checked));
    }
    if !cd.fmla_link.is_empty() {
        s.push_str(&format!(
            "\n        <x:FmlaLink>{}</x:FmlaLink>",
            cd.fmla_link
        ));
    }
    if let Some(v) = &cd.no_three_d {
        s.push_str(&format!("\n        <x:NoThreeD>{}</x:NoThreeD>", v));
    }
    if let Some(v) = &cd.first_button {
        s.push_str(&format!("\n        <x:FirstButton>{}</x:FirstButton>", v));
    }
    if cd.val != 0 {
        s.push_str(&format!("\n        <x:Val>{}</x:Val>", cd.val));
    }
    if cd.min != 0 {
        s.push_str(&format!("\n        <x:Min>{}</x:Min>", cd.min));
    }
    if cd.max != 0 {
        s.push_str(&format!("\n        <x:Max>{}</x:Max>", cd.max));
    }
    if cd.inc != 0 {
        s.push_str(&format!("\n        <x:Inc>{}</x:Inc>", cd.inc));
    }
    if cd.page != 0 {
        s.push_str(&format!("\n        <x:Page>{}</x:Page>", cd.page));
    }
    if let Some(v) = &cd.horiz {
        s.push_str(&format!("\n        <x:Horiz>{}</x:Horiz>", v));
    }
    if cd.dx != 0 {
        s.push_str(&format!("\n        <x:Dx>{}</x:Dx>", cd.dx));
    }
    s.push_str("\n      </x:ClientData>");
    s
}

fn default_vml_drawing(sheet_id: i32, vml_id: i32) -> VmlDrawing {
    VmlDrawing {
        xmlns_v: "urn:schemas-microsoft-com:vml".to_string(),
        xmlns_o: "urn:schemas-microsoft-com:office:office".to_string(),
        xmlns_x: "urn:schemas-microsoft-com:office:excel".to_string(),
        xmlns_mv: Some("http://macVmlSchemaUri".to_string()),
        shape_layout: Some(VmlShapeLayout {
            ext: "edit".to_string(),
            idmap: Some(VmlIdmap {
                ext: "edit".to_string(),
                data: sheet_id,
            }),
        }),
        shape_type: Some(VmlShapeType {
            id: format!("_x0000_t{vml_id}"),
            coord_size: "21600,21600".to_string(),
            spt: 202,
            path: "m0,0l0,21600,21600,21600,21600,0xe".to_string(),
            stroke: Some(VmlStroke {
                join_style: "miter".to_string(),
            }),
            v_path: Some(VmlPath {
                gradient_shape_ok: Some("t".to_string()),
                connect_type: "rect".to_string(),
                ..Default::default()
            }),
            ..Default::default()
        }),
        ..Default::default()
    }
}

fn default_header_footer_vml_drawing(sheet_id: i32) -> VmlDrawing {
    VmlDrawing {
        xmlns_v: "urn:schemas-microsoft-com:vml".to_string(),
        xmlns_o: "urn:schemas-microsoft-com:office:office".to_string(),
        xmlns_x: "urn:schemas-microsoft-com:office:excel".to_string(),
        shape_layout: Some(VmlShapeLayout {
            ext: "edit".to_string(),
            idmap: Some(VmlIdmap {
                ext: "edit".to_string(),
                data: sheet_id,
            }),
        }),
        shape_type: Some(VmlShapeType {
            id: "_x0000_t75".to_string(),
            coord_size: "21600,21600".to_string(),
            spt: 75,
            prefer_relative: Some("t".to_string()),
            path: "m@4@5l@4@11@9@11@9@5xe".to_string(),
            filled: Some("f".to_string()),
            stroked: Some("f".to_string()),
            stroke: Some(VmlStroke {
                join_style: "miter".to_string(),
            }),
            formulas: Some(VmlFormulas {
                formula: vec![
                    "if lineDrawn pixelLineWidth 0",
                    "sum @0 1 0",
                    "sum 0 0 @1",
                    "prod @2 1 2",
                    "prod @3 21600 pixelWidth",
                    "prod @3 21600 pixelHeight",
                    "sum @0 0 1",
                    "prod @6 1 2",
                    "prod @7 21600 pixelWidth",
                    "sum @8 21600 0",
                    "prod @7 21600 pixelHeight",
                    "sum @10 21600 0",
                ]
                .into_iter()
                .map(|e| VmlFormula {
                    equation: e.to_string(),
                })
                .collect(),
            }),
            v_path: Some(VmlPath {
                extrusion_ok: Some("f".to_string()),
                gradient_shape_ok: Some("t".to_string()),
                connect_type: "rect".to_string(),
            }),
            lock: Some(VmlLock {
                ext: "edit".to_string(),
                aspect_ratio: Some("t".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        }),
        ..Default::default()
    }
}

// ------------------------------------------------------------------
// Form control presets
// ------------------------------------------------------------------

#[derive(Debug, Default, Clone, PartialEq)]
struct FormCtrlPreset {
    object_type: String,
    auto_fill: String,
    filled: Option<String>,
    fill_color: Option<String>,
    stroked: Option<String>,
    stroke_color: Option<String>,
    stroke_button: Option<String>,
    fill: Option<VmlFill>,
    shadow: Option<VmlShadow>,
    text_h_align: String,
    text_v_align: String,
    no_three_d: Option<String>,
    first_button: Option<String>,
}

fn form_ctrl_preset(t: FormControlType) -> FormCtrlPreset {
    let none = FormCtrlPreset {
        object_type: String::new(),
        auto_fill: String::new(),
        filled: None,
        fill_color: None,
        stroked: None,
        stroke_color: None,
        stroke_button: None,
        fill: None,
        shadow: None,
        text_h_align: String::new(),
        text_v_align: String::new(),
        no_three_d: None,
        first_button: None,
    };
    match t {
        FormControlType::Note => FormCtrlPreset {
            object_type: "Note".to_string(),
            auto_fill: "True".to_string(),
            fill_color: Some("#FBF6D6".to_string()),
            stroke_color: Some("#EDEAA1".to_string()),
            fill: Some(VmlFill {
                angle: -180,
                color2: "#FBFE82".to_string(),
                r#type: "gradient".to_string(),
                fill: Some(VmlOFill {
                    ext: "view".to_string(),
                    r#type: "gradientUnscaled".to_string(),
                }),
            }),
            shadow: Some(VmlShadow {
                on: "t".to_string(),
                color: "black".to_string(),
                obscured: "t".to_string(),
            }),
            ..none
        },
        FormControlType::Button => FormCtrlPreset {
            object_type: "Button".to_string(),
            auto_fill: "True".to_string(),
            fill_color: Some("buttonFace [67]".to_string()),
            stroke_color: Some("windowText [64]".to_string()),
            stroke_button: Some("t".to_string()),
            fill: Some(VmlFill {
                angle: -180,
                color2: "buttonFace [67]".to_string(),
                r#type: "gradient".to_string(),
                fill: Some(VmlOFill {
                    ext: "view".to_string(),
                    r#type: "gradientUnscaled".to_string(),
                }),
            }),
            text_h_align: "Center".to_string(),
            text_v_align: "Center".to_string(),
            ..none
        },
        FormControlType::CheckBox => FormCtrlPreset {
            object_type: "Checkbox".to_string(),
            auto_fill: "True".to_string(),
            filled: Some("f".to_string()),
            fill_color: Some("window [65]".to_string()),
            stroked: Some("f".to_string()),
            stroke_color: Some("windowText [64]".to_string()),
            no_three_d: Some("".to_string()),
            text_v_align: "Center".to_string(),
            ..none
        },
        FormControlType::GroupBox => FormCtrlPreset {
            object_type: "GBox".to_string(),
            auto_fill: "False".to_string(),
            filled: Some("f".to_string()),
            stroked: Some("f".to_string()),
            stroke_color: Some("windowText [64]".to_string()),
            no_three_d: Some("".to_string()),
            ..none
        },
        FormControlType::Label => FormCtrlPreset {
            object_type: "Label".to_string(),
            auto_fill: "False".to_string(),
            filled: Some("f".to_string()),
            fill_color: Some("window [65]".to_string()),
            stroked: Some("f".to_string()),
            stroke_color: Some("windowText [64]".to_string()),
            ..none
        },
        FormControlType::OptionButton => FormCtrlPreset {
            object_type: "Radio".to_string(),
            auto_fill: "False".to_string(),
            filled: Some("f".to_string()),
            fill_color: Some("window [65]".to_string()),
            stroked: Some("f".to_string()),
            stroke_color: Some("windowText [64]".to_string()),
            no_three_d: Some("".to_string()),
            first_button: Some("".to_string()),
            text_v_align: "Center".to_string(),
            ..none
        },
        FormControlType::ScrollBar => FormCtrlPreset {
            object_type: "Scroll".to_string(),
            stroked: Some("f".to_string()),
            stroke_color: Some("windowText [64]".to_string()),
            ..none
        },
        FormControlType::SpinButton => FormCtrlPreset {
            object_type: "Spin".to_string(),
            auto_fill: "False".to_string(),
            stroked: Some("f".to_string()),
            stroke_color: Some("windowText [64]".to_string()),
            ..none
        },
    }
}

fn prepare_form_ctrl_options(mut opts: VmlOptions) -> VmlOptions {
    if let Some(fc) = opts.form_control.as_mut() {
        if fc.format.scale_x == 0.0 {
            fc.format.scale_x = 1.0;
        }
        if fc.format.scale_y == 0.0 {
            fc.format.scale_y = 1.0;
        }
        if fc.width == 0 {
            fc.width = 140;
        }
        if fc.height == 0 {
            fc.height = 60;
        }
    }
    opts
}

fn form_ctrl_text(fc: &FormControl) -> Vec<VmlFont> {
    let mut fonts = Vec::new();
    if !fc.text.is_empty() {
        fonts.push(VmlFont {
            content: fc.text.clone(),
            ..Default::default()
        });
    }
    for run in &fc.paragraph {
        let mut content = format!("{}<br></br>\r\n", run.text);
        let mut face = None;
        let mut size = None;
        let mut color = None;
        if let Some(font) = &run.font {
            face.clone_from(&font.name);
            if let Some(sz) = font.size {
                size = Some((sz * 20.0) as u32);
            }
            color.clone_from(&font.color);
            let mut color_str = color.clone().unwrap_or_default();
            if !color_str.starts_with('#') && !color_str.is_empty() {
                color_str = format!("#{color_str}");
            }
            color = Some(color_str);
            if font.underline == Some("single".to_string()) {
                content = format!("<u>{content}</u>");
            } else if font.underline == Some("double".to_string()) {
                content = format!("<u class=\"font1\">{content}</u>");
            }
            if font.italic == Some(true) {
                content = format!("<i>{content}</i>");
            }
            if font.bold == Some(true) {
                content = format!("<b>{content}</b>");
            }
        }
        fonts.push(VmlFont {
            face,
            size,
            color,
            content,
        });
    }
    fonts
}

// ------------------------------------------------------------------
// Extraction helpers
// ------------------------------------------------------------------

/// Wrap a `<v:shape>` inner XML fragment in a synthetic root so that it can be
/// parsed with quick-xml even though the namespace declarations live on the
/// drawing root.
fn wrap_vml_inner_xml(inner_xml: &str) -> String {
    format!(
        r#"<shape xmlns:v="urn:schemas-microsoft-com:vml" xmlns:o="urn:schemas-microsoft-com:office:office" xmlns:x="urn:schemas-microsoft-com:office:excel">{}</shape>"#,
        inner_xml
    )
}

fn local_name(name: &[u8]) -> &[u8] {
    if let Some(pos) = name.iter().rposition(|&b| b == b':') {
        &name[pos + 1..]
    } else {
        name
    }
}

#[derive(Debug, Default)]
struct ParsedClientData {
    object_type: String,
    anchor: String,
    fmla_macro: String,
    checked: u32,
    fmla_link: String,
    val: u32,
    min: u32,
    max: u32,
    inc: u32,
    page: u32,
    horiz: bool,
}

fn parse_client_data(inner_xml: &str) -> Option<ParsedClientData> {
    let wrapped = wrap_vml_inner_xml(inner_xml);
    let mut reader = Reader::from_str(&wrapped);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    let mut cd = ParsedClientData::default();
    let mut in_client_data = false;
    let mut current_tag = Vec::new();
    let mut current_text = String::new();
    let mut found = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let name = e.name();
                let local = local_name(name.as_ref());
                if local == b"ClientData" {
                    in_client_data = true;
                    found = true;
                    for attr in e.attributes().flatten() {
                        let key = String::from_utf8_lossy(attr.key.as_ref());
                        if key == "ObjectType" {
                            cd.object_type = String::from_utf8_lossy(&attr.value).to_string();
                        }
                    }
                } else if in_client_data {
                    current_tag = local.to_vec();
                    current_text.clear();
                }
            }
            Ok(Event::Empty(e)) => {
                let name = e.name();
                let local = local_name(name.as_ref());
                if local == b"ClientData" {
                    found = true;
                    for attr in e.attributes().flatten() {
                        let key = String::from_utf8_lossy(attr.key.as_ref());
                        if key == "ObjectType" {
                            cd.object_type = String::from_utf8_lossy(&attr.value).to_string();
                        }
                    }
                } else if in_client_data && local == b"Horiz" {
                    cd.horiz = true;
                }
            }
            Ok(Event::Text(e)) => {
                if in_client_data && !current_tag.is_empty() {
                    let t = e.unescape().unwrap_or(Cow::Borrowed(""));
                    current_text.push_str(&t);
                }
            }
            Ok(Event::End(e)) => {
                let name = e.name();
                let local = local_name(name.as_ref());
                if local == b"ClientData" {
                    in_client_data = false;
                } else if in_client_data {
                    let text = current_text.trim();
                    match local {
                        b"Anchor" => cd.anchor = text.to_string(),
                        b"FmlaMacro" => cd.fmla_macro = text.to_string(),
                        b"Checked" => cd.checked = text.parse().unwrap_or(0),
                        b"FmlaLink" => cd.fmla_link = text.to_string(),
                        b"Val" => cd.val = text.parse().unwrap_or(0),
                        b"Min" => cd.min = text.parse().unwrap_or(0),
                        b"Max" => cd.max = text.parse().unwrap_or(0),
                        b"Inc" => cd.inc = text.parse().unwrap_or(0),
                        b"Page" => cd.page = text.parse().unwrap_or(0),
                        b"Horiz" => cd.horiz = true,
                        _ => {}
                    }
                    current_tag.clear();
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }

    if found { Some(cd) } else { None }
}

fn extract_object_type(inner_xml: &str) -> String {
    parse_client_data(inner_xml)
        .map(|cd| cd.object_type)
        .unwrap_or_default()
}

fn extract_anchor(inner_xml: &str) -> String {
    parse_client_data(inner_xml)
        .map(|cd| cd.anchor)
        .unwrap_or_default()
}

fn extract_anchor_cell(
    anchor: &str,
) -> Result<(i32, i32), Box<dyn std::error::Error + Send + Sync>> {
    let parts: Vec<&str> = anchor.split(',').collect();
    if parts.len() != 8 {
        return Err(Box::new(ErrParameterInvalid));
    }
    let left_col = parts[0].trim().parse::<i32>()?;
    let top_row = parts[2].trim().parse::<i32>()?;
    Ok((left_col, top_row))
}

fn extract_form_control(inner_xml: &str) -> crate::errors::Result<FormControl> {
    let mut fc = FormControl::default();
    let cd = parse_client_data(inner_xml).unwrap_or_default();
    for (t, preset) in [
        (
            FormControlType::Note,
            form_ctrl_preset(FormControlType::Note),
        ),
        (
            FormControlType::Button,
            form_ctrl_preset(FormControlType::Button),
        ),
        (
            FormControlType::CheckBox,
            form_ctrl_preset(FormControlType::CheckBox),
        ),
        (
            FormControlType::GroupBox,
            form_ctrl_preset(FormControlType::GroupBox),
        ),
        (
            FormControlType::Label,
            form_ctrl_preset(FormControlType::Label),
        ),
        (
            FormControlType::OptionButton,
            form_ctrl_preset(FormControlType::OptionButton),
        ),
        (
            FormControlType::ScrollBar,
            form_ctrl_preset(FormControlType::ScrollBar),
        ),
        (
            FormControlType::SpinButton,
            form_ctrl_preset(FormControlType::SpinButton),
        ),
    ] {
        if preset.object_type == cd.object_type && !cd.anchor.is_empty() {
            fc.r#type = t;
            break;
        }
    }
    if fc.r#type == FormControlType::Note {
        return Ok(fc);
    }
    fc.paragraph = extract_vml_fonts(inner_xml);
    if !fc.paragraph.is_empty() && fc.paragraph[0].font.is_none() {
        fc.text.clone_from(&fc.paragraph[0].text);
        fc.paragraph.remove(0);
    }
    let (col, row) = extract_anchor_cell(&cd.anchor)?;
    fc.cell = coordinates_to_cell_name(col + 1, row + 1, false)?;
    fc.macro_name = cd.fmla_macro;
    fc.checked = cd.checked != 0;
    fc.cell_link = cd.fmla_link;
    fc.current_val = cd.val;
    fc.min_val = cd.min;
    fc.max_val = cd.max;
    fc.inc_change = cd.inc;
    fc.page_change = cd.page;
    fc.horizontally = cd.horiz;
    Ok(fc)
}

fn extract_vml_fonts(inner_xml: &str) -> Vec<RichTextRun> {
    parse_vml_textbox(inner_xml)
}

#[derive(Debug, Default)]
struct FontParseState {
    text: String,
    face: Option<String>,
    size: Option<u32>,
    color: Option<String>,
    bold: bool,
    italic: bool,
    underline: Option<String>,
}

fn parse_vml_textbox(inner_xml: &str) -> Vec<RichTextRun> {
    let wrapped = wrap_vml_inner_xml(inner_xml);
    let mut reader = Reader::from_str(&wrapped);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    let mut runs = Vec::new();
    let mut depth = 0usize;
    let mut current: Option<FontParseState> = None;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let name = e.name();
                let local = local_name(name.as_ref());
                match local {
                    b"textbox" if depth == 0 => depth = 1,
                    b"div" if depth == 1 => depth = 2,
                    b"font" if depth == 2 => {
                        depth = 3;
                        let mut state = FontParseState::default();
                        for attr in e.attributes().flatten() {
                            let key = String::from_utf8_lossy(attr.key.as_ref());
                            let value = String::from_utf8_lossy(&attr.value).to_string();
                            if key == "face" || key.ends_with(":face") {
                                state.face = Some(value);
                            } else if key == "size" || key.ends_with(":size") {
                                state.size = value.parse().ok();
                            } else if key == "color" || key.ends_with(":color") {
                                state.color = Some(value);
                            }
                        }
                        current = Some(state);
                    }
                    _ if depth >= 3 => {
                        match local {
                            b"b" | b"strong" => {
                                if let Some(c) = current.as_mut() {
                                    c.bold = true;
                                }
                            }
                            b"i" | b"em" => {
                                if let Some(c) = current.as_mut() {
                                    c.italic = true;
                                }
                            }
                            b"u" => {
                                let class = e
                                    .attributes()
                                    .flatten()
                                    .find(|a| {
                                        let k = String::from_utf8_lossy(a.key.as_ref());
                                        k == "class" || k.ends_with(":class")
                                    })
                                    .map(|a| String::from_utf8_lossy(&a.value).to_string());
                                let kind = if class.as_deref() == Some("font1") {
                                    "double"
                                } else {
                                    "single"
                                };
                                if let Some(c) = current.as_mut() {
                                    c.underline = Some(kind.to_string());
                                }
                            }
                            _ => {}
                        }
                        depth += 1;
                    }
                    _ => {}
                }
            }
            Ok(Event::Empty(e)) => {
                let name = e.name();
                let local = local_name(name.as_ref());
                if depth >= 3 && local == b"br" {
                    // Drop <br> elements; surrounding text is preserved.
                }
            }
            Ok(Event::Text(e)) => {
                if depth >= 3 {
                    if let Some(c) = current.as_mut() {
                        let t = e.unescape().unwrap_or(Cow::Borrowed(""));
                        c.text.push_str(&t);
                    }
                }
            }
            Ok(Event::End(e)) => {
                let name = e.name();
                let local = local_name(name.as_ref());
                match local {
                    b"textbox" if depth == 1 => depth = 0,
                    b"div" if depth == 2 => depth = 1,
                    b"font" if depth == 3 => {
                        depth = 2;
                        if let Some(state) = current.take() {
                            let mut run = RichTextRun::default();
                            run.text = state.text;
                            let mut font = crate::styles::Font::default();
                            if let Some(face) = state.face {
                                font.name = Some(face);
                            }
                            if let Some(size) = state.size {
                                font.size = Some(size as f64 / 20.0);
                            }
                            if let Some(color) = state.color {
                                font.color = Some(color.trim_start_matches('#').to_string());
                            }
                            if state.bold {
                                font.bold = Some(true);
                            }
                            if state.italic {
                                font.italic = Some(true);
                            }
                            if let Some(u) = state.underline {
                                font.underline = Some(u);
                            }
                            if font.name.is_some()
                                || font.size.is_some()
                                || font.color.is_some()
                                || font.bold == Some(true)
                                || font.italic == Some(true)
                                || font.underline.is_some()
                            {
                                run.font = Some(font);
                            }
                            runs.push(run);
                        }
                    }
                    _ if depth > 3 => depth -= 1,
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Ok(_) => {}
            Err(_) => break,
        }
        buf.clear();
    }

    runs
}

// ------------------------------------------------------------------
// Font / run property conversion
// ------------------------------------------------------------------

fn font_to_rpr(font: &crate::styles::Font) -> XlsxRPr {
    let mut rpr = XlsxRPr::default();
    rpr.family = Some(AttrValInt { val: Some(2) });
    if let Some(name) = &font.name {
        rpr.r_font = Some(AttrValString {
            val: Some(name.clone()),
        });
    }
    if let Some(size) = font.size {
        rpr.sz = Some(AttrValFloat { val: Some(size) });
    }
    if font.bold == Some(true) {
        rpr.b = Some(AttrValBool { val: Some(true) });
    }
    if font.italic == Some(true) {
        rpr.i = Some(AttrValBool { val: Some(true) });
    }
    if font.strike == Some(true) {
        rpr.strike = Some(AttrValBool { val: Some(true) });
    }
    if let Some(u) = &font.underline {
        rpr.u = Some(AttrValString {
            val: Some(u.clone()),
        });
    }
    if let Some(color) = &font.color {
        let mut c = color.clone();
        if !c.starts_with("FF") && c.len() == 6 {
            c = format!("FF{c}");
        }
        rpr.color = Some(XlsxColor {
            rgb: Some(c),
            ..Default::default()
        });
    }
    rpr
}

fn rpr_to_font(rpr: &XlsxRPr) -> crate::styles::Font {
    let mut font = crate::styles::Font::default();
    if let Some(name) = rpr.r_font.as_ref().and_then(|a| a.val.clone()) {
        font.name = Some(name);
    }
    if let Some(sz) = rpr.sz.as_ref().and_then(|a| a.val) {
        font.size = Some(sz);
    }
    if rpr.b.as_ref().and_then(|a| a.val).unwrap_or(false) {
        font.bold = Some(true);
    }
    if rpr.i.as_ref().and_then(|a| a.val).unwrap_or(false) {
        font.italic = Some(true);
    }
    if rpr.strike.as_ref().and_then(|a| a.val).unwrap_or(false) {
        font.strike = Some(true);
    }
    if let Some(u) = rpr.u.as_ref().and_then(|a| a.val.clone()) {
        font.underline = Some(u);
    }
    if let Some(color) = rpr.color.as_ref().and_then(|c| c.rgb.clone()) {
        font.color = Some(color.trim_start_matches("FF").to_string());
    }
    font
}

fn supported_image_types() -> HashMap<String, String> {
    [
        (".bmp".to_string(), ".bmp".to_string()),
        (".emf".to_string(), ".emf".to_string()),
        (".emz".to_string(), ".emz".to_string()),
        (".gif".to_string(), ".gif".to_string()),
        (".ico".to_string(), ".ico".to_string()),
        (".jpeg".to_string(), ".jpeg".to_string()),
        (".jpg".to_string(), ".jpeg".to_string()),
        (".png".to_string(), ".png".to_string()),
        (".svg".to_string(), ".svg".to_string()),
        (".tif".to_string(), ".tiff".to_string()),
        (".tiff".to_string(), ".tiff".to_string()),
        (".wmf".to_string(), ".wmf".to_string()),
        (".wmz".to_string(), ".wmz".to_string()),
    ]
    .into_iter()
    .collect()
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

// ------------------------------------------------------------------
// Tests
// ------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::xml::common::RichTextRun;
    use crate::{File, Options};

    #[test]
    fn comment_round_trip() {
        let mut f = File::new_with_options(Options::default());
        f.add_comment(
            "Sheet1",
            Comment {
                cell: "A1".to_string(),
                author: "Excelize".to_string(),
                text: "Hello comment".to_string(),
                ..Default::default()
            },
        )
        .unwrap();

        let comments = f.get_comments("Sheet1").unwrap();
        assert_eq!(comments.len(), 1);
        assert_eq!(comments[0].cell, "A1");
        assert_eq!(comments[0].author, "Excelize");
        assert_eq!(comments[0].text, "Hello comment");

        let tmp = std::env::temp_dir().join("excelize_rust_comment.xlsx");
        let _ = std::fs::remove_file(&tmp);
        f.save_as(tmp.to_str().unwrap()).unwrap();

        let f2 = File::open_file(tmp.to_str().unwrap(), Options::default()).unwrap();
        let comments2 = f2.get_comments("Sheet1").unwrap();
        assert_eq!(comments2.len(), 1);
        assert_eq!(comments2[0].cell, "A1");
        assert_eq!(comments2[0].text, "Hello comment");
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn rich_text_comment_round_trip() {
        let f = File::new_with_options(Options::default());
        f.add_comment(
            "Sheet1",
            Comment {
                cell: "B2".to_string(),
                author: "Author".to_string(),
                paragraph: vec![
                    RichTextRun {
                        text: "Bold: ".to_string(),
                        font: Some(crate::styles::Font {
                            bold: Some(true),
                            ..Default::default()
                        }),
                    },
                    RichTextRun {
                        text: "plain text".to_string(),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
        )
        .unwrap();

        let comments = f.get_comments("Sheet1").unwrap();
        assert_eq!(comments.len(), 1);
        assert_eq!(comments[0].paragraph.len(), 2);
        assert_eq!(comments[0].paragraph[0].text, "Bold: ");
        assert_eq!(
            comments[0].paragraph[0].font.as_ref().unwrap().bold,
            Some(true)
        );
    }

    #[test]
    fn form_control_button_round_trip() {
        let f = File::new_with_options(Options::default());
        f.add_form_control(
            "Sheet1",
            FormControl {
                cell: "C3".to_string(),
                r#type: FormControlType::Button,
                text: "Click me".to_string(),
                ..Default::default()
            },
        )
        .unwrap();

        let controls = f.get_form_controls("Sheet1").unwrap();
        assert_eq!(controls.len(), 1);
        assert_eq!(controls[0].cell, "C3");
        assert_eq!(controls[0].r#type, FormControlType::Button);
        assert_eq!(controls[0].text, "Click me");
    }

    #[test]
    fn delete_comment() {
        let f = File::new_with_options(Options::default());
        f.add_comment(
            "Sheet1",
            Comment {
                cell: "D4".to_string(),
                text: "to delete".to_string(),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(f.get_comments("Sheet1").unwrap().len(), 1);
        f.delete_comment("Sheet1", "D4").unwrap();
        assert_eq!(f.get_comments("Sheet1").unwrap().len(), 0);
    }

    #[test]
    fn header_footer_image_adds_vml_part() {
        let f = File::new_with_options(Options::default());
        // 1x1 red PNG
        let png = vec![
            0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48,
            0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00,
            0x00, 0x90, 0x77, 0x53, 0xde, 0x00, 0x00, 0x00, 0x0c, 0x49, 0x44, 0x41, 0x54, 0x08,
            0x99, 0x63, 0xf8, 0x0f, 0x00, 0x00, 0x01, 0x01, 0x00, 0x05, 0x18, 0xd8, 0x4e, 0x00,
            0x00, 0x00, 0x00, 0x49, 0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
        ];
        f.add_header_footer_image(
            "Sheet1",
            &HeaderFooterImageOptions {
                position: HeaderFooterImagePositionType::Center,
                file: png,
                extension: ".png".to_string(),
                width: "100pt".to_string(),
                height: "50pt".to_string(),
                ..Default::default()
            },
        )
        .unwrap();

        let ws = f.work_sheet_reader("Sheet1").unwrap();
        assert!(ws.legacy_drawing_hf.is_some());
    }

    #[test]
    fn vml_image_data_serializes_fields() {
        let image_data = VmlImageData {
            rel_id: Some("rId1".to_string()),
            title: Some("logo".to_string()),
            crop_top: Some("1px".to_string()),
            grayscale: Some("t".to_string()),
            ..Default::default()
        };
        let xml = image_data.to_xml_string();
        assert!(xml.contains("<v:imagedata"));
        assert!(xml.contains("o:relid=\"rId1\""));
        assert!(xml.contains("o:title=\"logo\""));
        assert!(xml.contains("croptop=\"1px\""));
        assert!(xml.contains("grayscale=\"t\""));
    }

    #[test]
    fn encode_shape_emits_image_data_and_lock() {
        let sp = EncodeShape {
            image_data: Some(VmlImageData {
                rel_id: Some("rId2".to_string()),
                ..Default::default()
            }),
            lock: Some(VmlLock {
                ext: "edit".to_string(),
                rotation: Some("t".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };
        let inner = build_shape_inner_xml(&sp, &FormCtrlPreset::default(), &VmlOptions::default());
        assert!(inner.contains("<v:imagedata o:relid=\"rId2\"/>"));
        assert!(inner.contains("<o:lock v:ext=\"edit\" rotation=\"t\"/>"));
    }
}
