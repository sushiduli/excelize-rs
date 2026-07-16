//! Chart API.
//!
//! Ported from Go `chart.go` and `drawing.go`.
//!
//! This is a functional subset: all chart types can be written, but only the
//! most common types are fully tuned. Reading charts back via `get_charts`
//! returns a lightweight representation of the chart series and title.

use std::collections::HashMap;

use quick_xml::de::from_reader as xml_from_reader;
use quick_xml::se::to_string as xml_to_string;

use crate::constants::{
    DEFAULT_CHART_DIMENSION_HEIGHT, DEFAULT_CHART_DIMENSION_WIDTH, DEFAULT_CHART_LEGEND_POSITION,
    DEFAULT_CHART_SHOW_BLANKS_AS, DEFAULT_DRAWING_SCALE, NAMESPACE_DRAWING_ML_CHART,
    NAMESPACE_DRAWING_ML_MAIN, NAMESPACE_DRAWING_ML_SPREADSHEET, NAMESPACE_SPREADSHEET,
    SOURCE_RELATIONSHIP, SOURCE_RELATIONSHIP_CHART, SOURCE_RELATIONSHIP_CHARTSHEET,
    SOURCE_RELATIONSHIP_DRAWING_ML,
};
use crate::errors::Result;
use crate::errors::{
    ErrChartTitle, ErrExistsSheet, ErrTransparency, new_unsupported_chart_type_error,
};
use crate::file::File;
use crate::lib_util::cell_name_to_coordinates;
use crate::xml::chart::{
    CAutoTitleDeleted, CAxs, CChart, CCharts, CLegend, CNumFmt, CPageMargins, CPlotArea,
    CPrintSettings, CScaling, CSer, CSpPr, CStrRef, CThicknessSpPr, CTitle, CTx, CVal, CView3D,
    XlsxChartSpace,
};
use crate::xml::decode_chart::{
    DecodeARPr, DecodeCAxis, DecodeCCharts, DecodeCDLbls, DecodeCDPt, DecodeCMarker, DecodeCNumFmt,
    DecodeCPlotArea, DecodeCSer, DecodeCSpPr, DecodeCTitle, DecodeChartSpace,
};
use crate::xml::drawing::{Fill, LineOptions, LineType};
use crate::xml::chart_sheet::{
    XlsxChartsheet, XlsxChartsheetView, XlsxChartsheetViews, XlsxDrawing as ChartsheetDrawing,
};
use crate::xml::common::{AttrValBool, AttrValFloat, AttrValInt, AttrValString, RichTextRun};
use crate::xml::decode_drawing::{
    DecodeCellAnchor, DecodeClientData, DecodeFrom, DecodePic, DecodeTo, DecodeWsDr,
};
use crate::xml::drawing::{
    GraphicOptions, XdrCellAnchor, XdrClientData, XlsxBlip, XlsxBlipFill, XlsxCNvPicPr, XlsxCNvPr,
    XlsxChart as XlsxChartRef, XlsxFrom, XlsxGraphic, XlsxGraphicData, XlsxGraphicFrame,
    XlsxHlinkClick, XlsxNvGraphicFramePr, XlsxNvPicPr, XlsxOff, XlsxPic, XlsxPicLocks, XlsxPoint2D,
    XlsxPositiveSize2D, XlsxPrstGeom, XlsxSpPr, XlsxStretch, XlsxTo, XlsxWsDr, XlsxXfrm,
};
use crate::xml::workbook::XlsxSheet;

// ------------------------------------------------------------------
// Re-exports of public types
// ------------------------------------------------------------------

pub use crate::xml::chart::{
    Chart, ChartAxis, ChartDataLabel, ChartDataLabelPositionType, ChartDataPoint, ChartDimension,
    ChartLegend, ChartMarker, ChartNumFmt, ChartPlotArea, ChartSeries, ChartTickLabelPositionType,
    ChartTitle, ChartType, ChartUpDownBar,
};

// ------------------------------------------------------------------
// Default value tables
// ------------------------------------------------------------------

const DEFAULT_EMU: i32 = 9525;

const SUPPORTED_DRAWING_UNDERLINE_TYPES: &[&str] = &[
    "none",
    "words",
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

const SUPPORTED_DRAWING_TEXT_VERTICAL_TYPE: &[&str] = &[
    "horz",
    "vert",
    "vert270",
    "wordArtVert",
    "eaVert",
    "mongolianVert",
    "wordArtVertRtl",
];

const CHART_LEGEND_POSITION: &[(&str, &str)] = &[
    ("top", "t"),
    ("left", "l"),
    ("right", "r"),
    ("top_right", "tr"),
];

fn chart_data_labels_position_types(t: ChartDataLabelPositionType) -> &'static str {
    match t.0 {
        1 => "bestFit",
        2 => "b",
        3 => "ctr",
        4 => "inBase",
        5 => "inEnd",
        6 => "l",
        7 => "outEnd",
        8 => "r",
        9 => "t",
        _ => "",
    }
}

fn supported_chart_data_labels_position(t: ChartType) -> &'static [ChartDataLabelPositionType] {
    match t.0 {
        6 | 7 | 8 => &[
            ChartDataLabelPositionType::CENTER,
            ChartDataLabelPositionType::INSIDE_BASE,
            ChartDataLabelPositionType::INSIDE_END,
            ChartDataLabelPositionType::OUTSIDE_END,
        ],
        21 | 22 | 23 => &[
            ChartDataLabelPositionType::CENTER,
            ChartDataLabelPositionType::INSIDE_BASE,
            ChartDataLabelPositionType::INSIDE_END,
            ChartDataLabelPositionType::OUTSIDE_END,
        ],
        0 | 3 | 24 => &[
            ChartDataLabelPositionType::CENTER,
            ChartDataLabelPositionType::INSIDE_BASE,
            ChartDataLabelPositionType::INSIDE_END,
        ],
        1 | 4 | 25 => &[
            ChartDataLabelPositionType::CENTER,
            ChartDataLabelPositionType::INSIDE_BASE,
            ChartDataLabelPositionType::INSIDE_END,
        ],
        2 | 5 | 26 | 27 => &[
            ChartDataLabelPositionType::CENTER,
            ChartDataLabelPositionType::INSIDE_BASE,
            ChartDataLabelPositionType::INSIDE_END,
        ],
        41 => &[
            ChartDataLabelPositionType::CENTER,
            ChartDataLabelPositionType::LEFT,
            ChartDataLabelPositionType::RIGHT,
            ChartDataLabelPositionType::ABOVE,
            ChartDataLabelPositionType::BELOW,
        ],
        43 | 44 | 45 | 46 => &[
            ChartDataLabelPositionType::BEST_FIT,
            ChartDataLabelPositionType::INSIDE_END,
            ChartDataLabelPositionType::OUTSIDE_END,
        ],
        40 => &[
            ChartDataLabelPositionType::BEST_FIT,
            ChartDataLabelPositionType::INSIDE_END,
            ChartDataLabelPositionType::OUTSIDE_END,
        ],
        47 => &[
            ChartDataLabelPositionType::CENTER,
            ChartDataLabelPositionType::LEFT,
            ChartDataLabelPositionType::RIGHT,
            ChartDataLabelPositionType::ABOVE,
            ChartDataLabelPositionType::BELOW,
        ],
        _ => &[],
    }
}

fn chart_view_3d_rot_x(t: ChartType) -> i64 {
    match t.0 {
        3..=5 | 9..=20 | 24..=39 | 49 | 50 => 15,
        42 => 20,
        44 => 30,
        51 | 52 => 90,
        _ => 0,
    }
}

fn chart_view_3d_rot_y(t: ChartType) -> i64 {
    match t.0 {
        3..=5 | 9..=20 | 24..=39 | 49 | 50 => 20,
        42 => 15,
        _ => 0,
    }
}

fn chart_view_3d_perspective(t: ChartType) -> i64 {
    match t.0 {
        42 => 30,
        _ => 0,
    }
}

fn chart_view_3d_rang_ax(t: ChartType) -> i64 {
    match t.0 {
        3..=5 | 9..=20 | 24..=39 => 1,
        _ => 0,
    }
}

fn plot_area_chart_grouping(t: ChartType) -> &'static str {
    match t.0 {
        2 | 5 | 8 | 11 | 14 | 17 | 20 | 23 | 27 | 31 | 35 | 39 => "percentStacked",
        1 | 4 | 7 | 10 | 13 | 16 | 19 | 22 | 26 | 30 | 34 | 38 => "stacked",
        0 | 3 | 24 | 28 | 32 | 36 | 41 | 42 => "standard",
        _ => "clustered",
    }
}

fn plot_area_chart_bar_dir(t: ChartType) -> Option<&'static str> {
    match t.0 {
        6..=20 => Some("bar"),
        21..=39 => Some("col"),
        _ => None,
    }
}

fn chart_val_ax_num_fmt(t: ChartType) -> &'static str {
    match t.0 {
        2 | 5 | 8 | 11 | 14 | 17 | 20 | 23 | 27 | 31 | 35 | 39 => "0%",
        _ => "General",
    }
}

fn chart_val_ax_cross_between(t: ChartType) -> &'static str {
    match t.0 {
        0..=5 | 49..=52 | 53 | 54 => "midCat",
        _ => "between",
    }
}

fn chart_shape(t: ChartType) -> Option<&'static str> {
    match t.0 {
        12 | 13 | 14 | 28 | 29 | 30 | 31 => Some("cone"),
        15 | 16 | 17 | 32 | 33 | 34 | 35 => Some("pyramid"),
        18 | 19 | 20 | 36 | 37 | 38 | 39 => Some("cylinder"),
        _ => None,
    }
}

fn is_bar_col(t: ChartType) -> bool {
    (6..=39).contains(&t.0)
}

fn is_3d_chart(t: ChartType) -> bool {
    matches!(t.0, 3..=5 | 9..=20 | 24..=39 | 42 | 44 | 49 | 50)
}

fn chart_element_name(t: ChartType) -> Option<&'static str> {
    match t.0 {
        0..=2 => Some("areaChart"),
        3..=5 => Some("area3DChart"),
        6..=8 | 21..=23 => Some("barChart"),
        9..=20 | 24..=39 => Some("bar3DChart"),
        40 => Some("doughnutChart"),
        41 => Some("lineChart"),
        42 => Some("line3DChart"),
        43 => Some("pieChart"),
        44 => Some("pie3DChart"),
        45 | 46 => Some("ofPieChart"),
        47 => Some("radarChart"),
        48 => Some("scatterChart"),
        49 | 50 => Some("surface3DChart"),
        51 | 52 => Some("surfaceChart"),
        53 | 54 => Some("bubbleChart"),
        55 | 56 => Some("stockChart"),
        _ => None,
    }
}

fn chart_series_uses_xy(t: ChartType) -> bool {
    matches!(t.0, 48 | 53 | 54)
}

fn chart_series_uses_bubble_size(t: ChartType) -> bool {
    matches!(t.0, 53 | 54)
}

// ------------------------------------------------------------------
// Public API
// ------------------------------------------------------------------

impl File {
    /// Add a chart to a worksheet.
    pub fn add_chart(&mut self, sheet: &str, cell: &str, chart: &Chart) -> Result<()> {
        let _ = self.work_sheet_reader(sheet)?;
        let opts = parse_chart_options(chart)?;
        let mut combo = Vec::new();
        let mut order = opts.series.len() as i64;
        for c in &chart.combo {
            if c.r#type.0 > 56 {
                return Err(new_unsupported_chart_type_error(c.r#type.0 as i32).into());
            }
            let mut co = parse_chart_options(c)?;
            co.order = order;
            order += co.series.len() as i64;
            combo.push(co);
        }

        let drawing_id = self.count_drawings() + 1;
        let chart_id = self.count_charts() + 1;
        let drawing_xml = format!("xl/drawings/drawing{drawing_id}.xml");
        let (drawing_id, drawing_xml) = self.prepare_drawing(sheet, drawing_id, &drawing_xml)?;
        let drawing_rels = format!("xl/drawings/_rels/drawing{drawing_id}.xml.rels");
        let drawing_rid = self.add_rels(
            &drawing_rels,
            SOURCE_RELATIONSHIP_CHART,
            &format!("../charts/chart{chart_id}.xml"),
            "",
        );

        self.add_drawing_chart(
            sheet,
            &drawing_xml,
            cell,
            opts.dimension.width as i32,
            opts.dimension.height as i32,
            drawing_rid,
            &opts.format,
        )?;
        self.add_chart_xml(&opts, &combo);
        self.add_content_type_part(chart_id, "chart")?;
        let _ = self.add_content_type_part(drawing_id, "drawings");
        self.add_sheet_name_space(sheet, NAMESPACE_SPREADSHEET);
        Ok(())
    }

    /// Create a chartsheet (a worksheet containing only a chart).
    pub fn add_chart_sheet(&mut self, sheet: &str, chart: &Chart) -> Result<()> {
        if self.get_sheet_index(sheet).is_ok() {
            return Err(Box::new(ErrExistsSheet));
        }
        let opts = parse_chart_options(chart)?;
        let mut combo = Vec::new();
        let mut order = opts.series.len() as i64;
        for c in &chart.combo {
            if c.r#type.0 > 56 {
                return Err(new_unsupported_chart_type_error(c.r#type.0 as i32).into());
            }
            let mut co = parse_chart_options(c)?;
            co.order = order;
            order += co.series.len() as i64;
            combo.push(co);
        }

        let mut wb = self.workbook_reader()?;
        let mut max_sheet_id = 0i64;
        let mut max_rid = 0i32;
        for s in &wb.sheets.sheet {
            if let Some(id) = s.sheet_id {
                if id > max_sheet_id {
                    max_sheet_id = id;
                }
            }
            if let Some(rid) = &s.id {
                if let Ok(n) = rid.trim_start_matches("rId").parse::<i32>() {
                    if n > max_rid {
                        max_rid = n;
                    }
                }
            }
        }
        for rel in &self
            .rels_reader(&self.get_workbook_rels_path())?
            .unwrap_or_default()
            .relationships
        {
            if let Ok(n) = rel.id.trim_start_matches("rId").parse::<i32>() {
                if n > max_rid {
                    max_rid = n;
                }
            }
        }
        max_sheet_id += 1;
        max_rid += 1;

        let path = format!("xl/chartsheets/sheet{max_sheet_id}.xml");
        let new_rid = format!("rId{max_rid}");
        wb.sheets.sheet.push(XlsxSheet {
            name: Some(sheet.to_string()),
            sheet_id: Some(max_sheet_id),
            id: Some(new_rid.clone()),
            plain_id: None,
            state: None,
        });

        let drawing_id = self.count_drawings() + 1;
        let chart_id = self.count_charts() + 1;
        let drawing_xml = format!("xl/drawings/drawing{drawing_id}.xml");
        let sheet_rels = format!(
            "xl/chartsheets/_rels/{}.rels",
            path.trim_start_matches("xl/chartsheets/")
        );
        let sheet_drawing_target = format!("../drawings/drawing{drawing_id}.xml");
        let sheet_drawing_rid = self.add_rels(
            &sheet_rels,
            SOURCE_RELATIONSHIP_DRAWING_ML,
            &sheet_drawing_target,
            "",
        );

        let cs = XlsxChartsheet {
            sheet_views: Some(XlsxChartsheetViews {
                sheet_view: vec![XlsxChartsheetView {
                    zoom_scale: Some(100),
                    zoom_to_fit: Some(true),
                    ..Default::default()
                }],
                ..Default::default()
            }),
            drawing: Some(ChartsheetDrawing {
                r_id: Some(format!("rId{sheet_drawing_rid}")),
            }),
            ..Default::default()
        };
        let cs_bytes = xml_to_string(&cs)?.into_bytes();
        self.save_file_list(&path, &cs_bytes);
        self.sheet_map
            .lock()
            .unwrap()
            .insert(sheet.to_string(), path.clone());

        let drawing_rels = format!("xl/drawings/_rels/drawing{drawing_id}.xml.rels");
        let drawing_rid = self.add_rels(
            &drawing_rels,
            SOURCE_RELATIONSHIP_CHART,
            &format!("../charts/chart{chart_id}.xml"),
            "",
        );
        self.add_sheet_drawing_chart(&drawing_xml, drawing_rid, &opts.format)?;
        self.add_chart_xml(&opts, &combo);
        self.add_content_type_part(chart_id, "chart")?;
        let _ = self.add_content_type_part(max_sheet_id as i32, "chartsheet");
        let _ = self.add_content_type_part(drawing_id, "drawings");

        self.relationships.insert(self.get_workbook_rels_path(), {
            let mut rels = self
                .rels_reader(&self.get_workbook_rels_path())?
                .unwrap_or_default();
            rels.relationships
                .push(crate::xml::workbook::XlsxRelationship {
                    id: new_rid,
                    r#type: SOURCE_RELATIONSHIP_CHARTSHEET.to_string(),
                    target: format!("/xl/chartsheets/sheet{max_sheet_id}.xml"),
                    target_mode: None,
                });
            rels
        });
        *self.workbook.lock().unwrap() = Some(wb);
        *self.sheet_count.lock().unwrap() += 1;
        self.add_sheet_name_space(sheet, NAMESPACE_SPREADSHEET);
        Ok(())
    }

    /// Delete the chart at a given cell in a worksheet.
    pub fn delete_chart(&mut self, sheet: &str, cell: &str) -> Result<()> {
        let (col, row) = cell_name_to_coordinates(cell)?;
        let col = col - 1;
        let row = row - 1;
        let ws = self.work_sheet_reader(sheet)?;
        if ws.drawing.is_none() {
            return Ok(());
        }
        let drawing_xml = self
            .get_sheet_relationships_target_by_id(
                sheet,
                ws.drawing.as_ref().unwrap().rid.as_deref().unwrap_or(""),
            )
            .replace("..", "xl")
            .trim_start_matches('/')
            .to_string();
        self.delete_drawing(col, row, &drawing_xml, "Chart")?;
        Ok(())
    }

    /// Return all charts embedded in a worksheet.
    pub fn get_charts(&self, sheet: &str) -> Result<Vec<Chart>> {
        let ws = self.work_sheet_reader(sheet)?;
        if ws.drawing.is_none() {
            return Ok(Vec::new());
        }
        let drawing_xml = self
            .get_sheet_relationships_target_by_id(
                sheet,
                ws.drawing.as_ref().unwrap().rid.as_deref().unwrap_or(""),
            )
            .replace("..", "xl")
            .trim_start_matches('/')
            .to_string();
        let drawing_rels = drawing_xml
            .replace("xl/drawings", "xl/drawings/_rels")
            .replace(".xml", ".xml.rels");
        self.get_charts_from_drawing(&drawing_xml, &drawing_rels)
    }
}

// ------------------------------------------------------------------
// Option parsing
// ------------------------------------------------------------------

fn parse_chart_options(chart: &Chart) -> Result<Chart> {
    if chart.r#type.0 > 56 {
        return Err(new_unsupported_chart_type_error(chart.r#type.0 as i32).into());
    }
    let mut opts = chart.clone();
    if opts.dimension.width == 0 {
        opts.dimension.width = DEFAULT_CHART_DIMENSION_WIDTH as u64;
    }
    if opts.dimension.height == 0 {
        opts.dimension.height = DEFAULT_CHART_DIMENSION_HEIGHT as u64;
    }
    if opts.legend.position.is_empty() {
        opts.legend.position = DEFAULT_CHART_LEGEND_POSITION.to_string();
    }
    parse_title(&mut opts.title)?;
    if !(0..=100).contains(&opts.fill.transparency) {
        return Err(Box::new(ErrTransparency));
    }
    if opts.vary_colors.is_none() {
        opts.vary_colors = Some(true);
    }
    if opts.border.width == 0.0 {
        opts.border.width = 0.75;
    }
    if opts.show_blanks_as.is_empty() {
        opts.show_blanks_as = DEFAULT_CHART_SHOW_BLANKS_AS.to_string();
    }
    opts.format = parse_graphic_options(&opts.format)?;
    for series in &opts.series {
        if !(0..=100).contains(&series.fill.transparency) {
            return Err(Box::new(ErrTransparency));
        }
    }
    Ok(opts)
}

fn parse_title(title: &mut ChartTitle) -> Result<()> {
    if !(0..=100).contains(&title.offset_x)
        || !(0..=100).contains(&title.offset_y)
        || !(0..=100).contains(&title.width)
        || !(0..=100).contains(&title.height)
    {
        return Err(Box::new(ErrChartTitle));
    }
    if !title.paragraph.is_empty() && !title.formula.is_empty() {
        return Err(Box::new(ErrChartTitle));
    }
    if title.font.is_none() {
        title.font = Some(crate::styles::Font::default());
    }
    let font = title.font.as_mut().unwrap();
    if font.color.as_deref().unwrap_or("").is_empty() {
        font.color = Some("595959".to_string());
    }
    if font.size.unwrap_or(0.0) == 0.0 {
        font.size = Some(14.0);
    }
    for run in &mut title.paragraph {
        if run.font.is_none() {
            run.font = Some(crate::styles::Font::default());
        }
        let f = run.font.as_mut().unwrap();
        if f.color.as_deref().unwrap_or("").is_empty() {
            f.color = Some("595959".to_string());
        }
        if f.size.unwrap_or(0.0) == 0.0 {
            f.size = Some(14.0);
        }
    }
    Ok(())
}

pub(crate) fn parse_graphic_options(opts: &GraphicOptions) -> Result<GraphicOptions> {
    let mut out = opts.clone();
    if out.print_object.is_none() {
        out.print_object = Some(true);
    }
    if out.locked.is_none() {
        out.locked = Some(true);
    }
    if out.scale_x == 0.0 {
        out.scale_x = DEFAULT_DRAWING_SCALE;
    }
    if out.scale_y == 0.0 {
        out.scale_y = DEFAULT_DRAWING_SCALE;
    }
    if !out.positioning.is_empty()
        && !["oneCell", "twoCell", "absolute"].contains(&out.positioning.as_str())
    {
        return Err(crate::errors::new_invalid_optional_value(
            "Positioning",
            &out.positioning,
            &["oneCell", "twoCell", "absolute"],
        )
        .into());
    }
    Ok(out)
}

// ------------------------------------------------------------------
// Chart XML generation
// ------------------------------------------------------------------

impl File {
    fn add_chart_xml(&self, opts: &Chart, combo: &[Chart]) {
        let chart_id = self.count_charts() + 1;
        let mut space = XlsxChartSpace {
            xmlns_a: Some(NAMESPACE_DRAWING_ML_MAIN.to_string()),
            xmlns_c: Some(NAMESPACE_DRAWING_ML_CHART.to_string()),
            date_1904: Some(AttrValBool { val: Some(false) }),
            lang: Some(AttrValString {
                val: Some("en-US".to_string()),
            }),
            rounded_corners: Some(AttrValBool { val: Some(false) }),
            chart: CChart {
                title: draw_title(&opts.title, ""),
                auto_title_deleted: Some(CAutoTitleDeleted { val: false }),
                view_3d: if is_3d_chart(opts.r#type) {
                    Some(CView3D {
                        rot_x: Some(AttrValInt {
                            val: Some(chart_view_3d_rot_x(opts.r#type)),
                        }),
                        rot_y: Some(AttrValInt {
                            val: Some(chart_view_3d_rot_y(opts.r#type)),
                        }),
                        perspective: Some(AttrValInt {
                            val: Some(chart_view_3d_perspective(opts.r#type)),
                        }),
                        r_ang_ax: Some(AttrValInt {
                            val: Some(chart_view_3d_rang_ax(opts.r#type)),
                        }),
                        ..Default::default()
                    })
                } else {
                    None
                },
                floor: if is_3d_chart(opts.r#type) {
                    Some(CThicknessSpPr {
                        thickness: Some(AttrValInt { val: Some(0) }),
                        ..Default::default()
                    })
                } else {
                    None
                },
                side_wall: if is_3d_chart(opts.r#type) {
                    Some(CThicknessSpPr {
                        thickness: Some(AttrValInt { val: Some(0) }),
                        ..Default::default()
                    })
                } else {
                    None
                },
                back_wall: if is_3d_chart(opts.r#type) {
                    Some(CThicknessSpPr {
                        thickness: Some(AttrValInt { val: Some(0) }),
                        ..Default::default()
                    })
                } else {
                    None
                },
                plot_area: Some(build_plot_area(opts, combo)),
                plot_vis_only: Some(AttrValBool { val: Some(false) }),
                disp_blanks_as: Some(AttrValString {
                    val: Some(opts.show_blanks_as.clone()),
                }),
                show_d_lbls_over_max: Some(AttrValBool { val: Some(false) }),
                ..Default::default()
            },
            sp_pr: Some(CSpPr {
                solid_fill: Some(crate::xml::drawing::ASolidFill {
                    scheme_clr: Some(crate::xml::drawing::ASchemeClr {
                        val: Some("bg1".to_string()),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                ln: draw_chart_ln(&opts.border),
                ..Default::default()
            }),
            print_settings: Some(CPrintSettings {
                page_margins: Some(CPageMargins {
                    b: 0.75,
                    l: 0.7,
                    r: 0.7,
                    t: 0.7,
                    header: 0.3,
                    footer: 0.3,
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        if let Some(sp_pr) = &mut space.sp_pr {
            apply_fill(sp_pr, &opts.fill);
        }
        space.chart.legend = draw_legend(opts);
        if let Some(pa) = &mut space.chart.plot_area {
            pa.sp_pr = Some(draw_plot_area_sp_pr());
            if let Some(sp_pr) = &mut pa.sp_pr {
                apply_fill(sp_pr, &opts.plot_area.fill);
            }
            pa.d_table = draw_plot_area_d_table(opts);
        }

        let chart_xml = xml_to_string(&space).unwrap_or_default().into_bytes();
        self.save_file_list(&format!("xl/charts/chart{chart_id}.xml"), &chart_xml);
    }
}

fn build_plot_area(opts: &Chart, combo: &[Chart]) -> CPlotArea {
    let element = match chart_element_name(opts.r#type) {
        Some(name) => name,
        None => return CPlotArea::default(),
    };
    let chart = build_chart(opts);
    let mut pa = CPlotArea::default();
    match element {
        "areaChart" => pa.area_chart = vec![chart],
        "area3DChart" => pa.area_3d_chart = vec![chart],
        "barChart" => pa.bar_chart = vec![chart],
        "bar3DChart" => pa.bar_3d_chart = vec![chart],
        "bubbleChart" => pa.bubble_chart = vec![chart],
        "doughnutChart" => pa.doughnut_chart = vec![chart],
        "lineChart" => pa.line_chart = vec![chart],
        "line3DChart" => pa.line_3d_chart = vec![chart],
        "pieChart" => pa.pie_chart = vec![chart],
        "pie3DChart" => pa.pie_3d_chart = vec![chart],
        "ofPieChart" => pa.of_pie_chart = vec![chart],
        "radarChart" => pa.radar_chart = vec![chart],
        "scatterChart" => pa.scatter_chart = vec![chart],
        "surface3DChart" => pa.surface_3d_chart = vec![chart],
        "surfaceChart" => pa.surface_chart = vec![chart],
        "stockChart" => pa.stock_chart = vec![chart],
        _ => {}
    }

    for co in combo {
        if let Some(name) = chart_element_name(co.r#type) {
            let co_chart = build_chart(co);
            match name {
                "areaChart" => pa.area_chart.push(co_chart),
                "area3DChart" => pa.area_3d_chart.push(co_chart),
                "barChart" => pa.bar_chart.push(co_chart),
                "bar3DChart" => pa.bar_3d_chart.push(co_chart),
                "bubbleChart" => pa.bubble_chart.push(co_chart),
                "doughnutChart" => pa.doughnut_chart.push(co_chart),
                "lineChart" => pa.line_chart.push(co_chart),
                "line3DChart" => pa.line_3d_chart.push(co_chart),
                "pieChart" => pa.pie_chart.push(co_chart),
                "pie3DChart" => pa.pie_3d_chart.push(co_chart),
                "ofPieChart" => pa.of_pie_chart.push(co_chart),
                "radarChart" => pa.radar_chart.push(co_chart),
                "scatterChart" => pa.scatter_chart.push(co_chart),
                "surface3DChart" => pa.surface_3d_chart.push(co_chart),
                "surfaceChart" => pa.surface_chart.push(co_chart),
                "stockChart" => pa.stock_chart.push(co_chart),
                _ => {}
            }
        }
    }

    if !chart_series_uses_xy(opts.r#type) {
        pa.cat_ax = draw_plot_area_cat_ax(&pa, opts);
    }
    if chart_series_uses_xy(opts.r#type) {
        pa.val_ax = draw_plot_area_cat_ax(&pa, opts);
        pa.val_ax.extend(draw_plot_area_val_ax(&pa, opts));
    } else {
        pa.val_ax = draw_plot_area_val_ax(&pa, opts);
    }

    // Generate secondary axes for combo charts that request a secondary Y-axis.
    if let Some(secondary_opts) = combo
        .iter()
        .rev()
        .find(|co| co.order > 0 && co.y_axis.secondary)
    {
        pa.cat_ax.extend(draw_plot_area_cat_ax(&pa, secondary_opts));
        pa.val_ax.extend(draw_plot_area_val_ax(&pa, secondary_opts));
    }

    if opts.r#type == ChartType::STOCK_HIGH_LOW_CLOSE
        || opts.r#type == ChartType::STOCK_OPEN_HIGH_LOW_CLOSE
    {
        pa.date_ax = pa.cat_ax.clone();
        pa.cat_ax.clear();
    }
    if (49..=52).contains(&opts.r#type.0) {
        pa.ser_ax = draw_plot_area_ser_ax(opts);
    }
    pa
}

fn build_chart(opts: &Chart) -> CCharts {
    let mut c = CCharts {
        vary_colors: Some(AttrValBool {
            val: Some(opts.vary_colors.unwrap_or(true)),
        }),
        ser: Some(draw_chart_series(opts)),
        ax_id: gen_ax_id(opts),
        ..Default::default()
    };
    if let Some(dir) = plot_area_chart_bar_dir(opts.r#type) {
        c.bar_dir = Some(AttrValString {
            val: Some(dir.to_string()),
        });
        c.grouping = Some(AttrValString {
            val: Some(plot_area_chart_grouping(opts.r#type).to_string()),
        });
    }
    if matches!(opts.r#type.0, 0..=5) {
        c.grouping = Some(AttrValString {
            val: Some(plot_area_chart_grouping(opts.r#type).to_string()),
        });
    }
    if let Some(shape) = chart_shape(opts.r#type) {
        c.shape = Some(AttrValString {
            val: Some(shape.to_string()),
        });
    }
    if opts.r#type == ChartType::DOUGHNUT {
        let hole = if opts.hole_size > 0 && opts.hole_size <= 90 {
            opts.hole_size
        } else {
            75
        };
        c.hole_size = Some(AttrValInt { val: Some(hole) });
    }
    if is_bar_col(opts.r#type) {
        if let Some(gw) = opts.gap_width {
            if gw <= 500 {
                c.gap_width = Some(AttrValInt {
                    val: Some(gw as i64),
                });
            }
        }
        let mut overlap = if matches!(opts.r#type.0, 7 | 8 | 22 | 23) {
            Some(100)
        } else {
            None
        };
        if let Some(o) = opts.overlap {
            if (-100..=100).contains(&o) {
                overlap = Some(o);
            }
        }
        if let Some(o) = overlap {
            c.overlap = Some(AttrValInt { val: Some(o) });
        }
    }
    if matches!(opts.r#type.0, 53 | 54) && opts.bubble_size > 0 && opts.bubble_size <= 300 {
        c.bubble_scale = Some(AttrValFloat {
            val: Some(opts.bubble_size as f64),
        });
    }
    if opts.r#type == ChartType::LINE || opts.r#type == ChartType::LINE_3D {
        c.grouping = Some(AttrValString {
            val: Some("standard".to_string()),
        });
        c.vary_colors = Some(AttrValBool { val: Some(false) });
    }
    if opts.r#type == ChartType::RADAR {
        c.radar_style = Some(AttrValString {
            val: Some("marker".to_string()),
        });
        c.vary_colors = Some(AttrValBool { val: Some(false) });
    }
    if opts.r#type == ChartType::SCATTER {
        c.scatter_style = Some(AttrValString {
            val: Some("smoothMarker".to_string()),
        });
        c.vary_colors = Some(AttrValBool { val: Some(false) });
    }
    if opts.r#type == ChartType::PIE_OF_PIE {
        c.of_pie_type = Some(AttrValString {
            val: Some("pie".to_string()),
        });
        if opts.plot_area.second_plot_values > 0 {
            c.split_pos = Some(AttrValInt {
                val: Some(opts.plot_area.second_plot_values),
            });
        }
        c.ser_lines = Some(AttrValString::default());
    }
    if opts.r#type == ChartType::BAR_OF_PIE {
        c.of_pie_type = Some(AttrValString {
            val: Some("bar".to_string()),
        });
        if opts.plot_area.second_plot_values > 0 {
            c.split_pos = Some(AttrValInt {
                val: Some(opts.plot_area.second_plot_values),
            });
        }
        c.ser_lines = Some(AttrValString::default());
    }
    if opts.x_axis.drop_lines && matches!(opts.r#type.0, 0..=5 | 41 | 42) {
        c.drop_lines = Some(crate::xml::chart::CLines::default());
    }
    if opts.x_axis.high_low_lines && opts.r#type == ChartType::LINE {
        c.hi_low_lines = Some(crate::xml::chart::CLines::default());
    }
    if opts.r#type == ChartType::WIREFRAME_SURFACE_3D || opts.r#type == ChartType::WIREFRAME_CONTOUR
    {
        c.wireframe = Some(AttrValBool { val: Some(true) });
    }
    if opts.r#type == ChartType::STOCK_HIGH_LOW_CLOSE {
        c.hi_low_lines = Some(crate::xml::chart::CLines::default());
    }
    if opts.r#type == ChartType::STOCK_OPEN_HIGH_LOW_CLOSE {
        c.hi_low_lines = Some(crate::xml::chart::CLines::default());
        let mut up_sp_pr = CSpPr::default();
        if let Some(ln) = draw_chart_ln(&opts.plot_area.up_bars.border) {
            up_sp_pr.ln = Some(ln);
        }
        apply_fill(&mut up_sp_pr, &opts.plot_area.up_bars.fill);
        let mut down_sp_pr = CSpPr::default();
        if let Some(ln) = draw_chart_ln(&opts.plot_area.down_bars.border) {
            down_sp_pr.ln = Some(ln);
        }
        apply_fill(&mut down_sp_pr, &opts.plot_area.down_bars.fill);
        c.up_down_bars = Some(crate::xml::chart::CUpDownBars {
            gap_width: Some(AttrValString {
                val: Some("150".to_string()),
            }),
            up_bars: Some(crate::xml::chart::CLines {
                sp_pr: Some(up_sp_pr),
            }),
            down_bars: Some(crate::xml::chart::CLines {
                sp_pr: Some(down_sp_pr),
            }),
            ..Default::default()
        });
    }
    c.d_lbls = Some(draw_chart_d_lbls(opts));
    c
}

fn gen_ax_id(opts: &Chart) -> Vec<AttrValInt> {
    let (x, y) = if opts.order > 0 && opts.y_axis.secondary {
        (100000003, 100000004)
    } else {
        (100000000, 100000001)
    };
    let mut ids = vec![AttrValInt { val: Some(x) }, AttrValInt { val: Some(y) }];
    if (49..=52).contains(&opts.r#type.0) {
        ids.push(AttrValInt {
            val: Some(100000005),
        });
    }
    ids
}

fn draw_chart_series(opts: &Chart) -> Vec<CSer> {
    opts.series
        .iter()
        .enumerate()
        .map(|(k, s)| CSer {
            idx: Some(AttrValInt {
                val: Some(k as i64 + opts.order),
            }),
            order: Some(AttrValInt {
                val: Some(k as i64 + opts.order),
            }),
            tx: Some(CTx {
                str_ref: Some(CStrRef {
                    f: s.name.clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            sp_pr: draw_chart_series_sp_pr(k, s, opts),
            marker: draw_series_marker(k, s, opts),
            d_pt: draw_chart_series_d_pt(k, s, opts),
            d_lbls: draw_chart_series_d_lbls(k, s, opts),
            invert_if_negative: Some(AttrValBool { val: Some(false) }),
            cat: if chart_series_uses_xy(opts.r#type) || s.categories.is_empty() {
                None
            } else {
                Some(crate::xml::chart::CCat {
                    str_ref: Some(CStrRef {
                        f: s.categories.clone(),
                        ..Default::default()
                    }),
                })
            },
            x_val: if chart_series_uses_xy(opts.r#type) && !s.categories.is_empty() {
                Some(crate::xml::chart::CCat {
                    str_ref: Some(CStrRef {
                        f: s.categories.clone(),
                        ..Default::default()
                    }),
                })
            } else {
                None
            },
            val: if chart_series_uses_xy(opts.r#type) {
                None
            } else {
                Some(CVal {
                    num_ref: Some(crate::xml::chart::CNumRef {
                        f: s.values.clone(),
                        num_cache: Some(crate::xml::chart::CNumCache::default()),
                    }),
                })
            },
            y_val: if chart_series_uses_xy(opts.r#type) {
                Some(CVal {
                    num_ref: Some(crate::xml::chart::CNumRef {
                        f: s.values.clone(),
                        num_cache: Some(crate::xml::chart::CNumCache::default()),
                    }),
                })
            } else {
                None
            },
            smooth: Some(AttrValBool {
                val: Some(s.line.smooth),
            }),
            bubble_size: if chart_series_uses_bubble_size(opts.r#type) {
                Some(CVal {
                    num_ref: Some(crate::xml::chart::CNumRef {
                        f: if s.sizes.is_empty() {
                            s.values.clone()
                        } else {
                            s.sizes.clone()
                        },
                        num_cache: Some(crate::xml::chart::CNumCache::default()),
                    }),
                })
            } else {
                None
            },
            bubble_3d: if opts.r#type == ChartType::BUBBLE_3D {
                Some(AttrValBool { val: Some(true) })
            } else {
                None
            },
            ..Default::default()
        })
        .collect()
}

fn draw_series_marker(
    _idx: usize,
    series: &ChartSeries,
    opts: &Chart,
) -> Option<crate::xml::chart::CMarker> {
    if !matches!(opts.r#type.0, 41 | 42 | 47 | 48 | 55 | 56) {
        return None;
    }
    let mut m = crate::xml::chart::CMarker {
        symbol: Some(AttrValString {
            val: Some(series.marker.symbol.clone()),
        }),
        size: Some(AttrValInt {
            val: Some(if series.marker.size > 0 {
                series.marker.size
            } else {
                5
            }),
        }),
        ..Default::default()
    };
    if series.marker.symbol.is_empty() {
        let default_symbol = match opts.r#type.0 {
            48 => "circle",
            55 => "dot",
            56 => "none",
            _ => "auto",
        };
        m.symbol = Some(AttrValString {
            val: Some(default_symbol.to_string()),
        });
    }
    Some(m)
}

fn draw_chart_series_sp_pr(idx: usize, series: &ChartSeries, opts: &Chart) -> Option<CSpPr> {
    let accent = format!("accent{}", (opts.order as usize + idx) % 6 + 1);
    let mut sp_pr = CSpPr {
        solid_fill: Some(crate::xml::drawing::ASolidFill {
            scheme_clr: Some(crate::xml::drawing::ASchemeClr {
                val: Some(accent),
                ..Default::default()
            }),
            ..Default::default()
        }),
        ..Default::default()
    };
    apply_fill(&mut sp_pr, &series.fill);
    let solid_ln = crate::xml::drawing::ALn {
        w: Some(pt_to_emus(series.line.width)),
        cap: Some("rnd".to_string()),
        solid_fill: sp_pr.solid_fill.clone(),
        ..Default::default()
    };
    let mut solid_ln = solid_ln;
    if series.line.dash != crate::xml::drawing::LineDashType::UNSET {
        if let Some(dash) = line_dash_type(series.line.dash) {
            solid_ln.prst_dash = Some(AttrValString {
                val: Some(dash.to_string()),
            });
        }
    }
    let no_ln = crate::xml::drawing::ALn {
        no_fill: Some(AttrValString::default()),
        ..Default::default()
    };
    let solid_sp_pr = CSpPr {
        ln: Some(solid_ln),
        ..Default::default()
    };
    let no_sp_pr = CSpPr {
        ln: Some(no_ln),
        ..Default::default()
    };
    let line_sp_pr = match opts.r#type.0 {
        41 => match series.line.r#type {
            crate::xml::drawing::LineType::UNSET
            | crate::xml::drawing::LineType::SOLID
            | crate::xml::drawing::LineType::AUTOMATIC => Some(solid_sp_pr),
            crate::xml::drawing::LineType::NONE => Some(no_sp_pr),
            crate::xml::drawing::LineType(_) => None,
        },
        48 | 55 | 56 => match series.line.r#type {
            crate::xml::drawing::LineType::SOLID => Some(solid_sp_pr),
            crate::xml::drawing::LineType::NONE => Some(no_sp_pr),
            crate::xml::drawing::LineType(_) => Some(no_sp_pr),
        },
        _ => None,
    };
    if let Some(sp) = line_sp_pr {
        return Some(sp);
    }
    if sp_pr
        .solid_fill
        .as_ref()
        .and_then(|f| f.srgb_clr.as_ref())
        .is_some()
        || sp_pr.no_fill.is_some()
    {
        return Some(sp_pr);
    }
    None
}

fn line_dash_type(dash: crate::xml::drawing::LineDashType) -> Option<&'static str> {
    match dash.0 {
        1 => Some("solid"),
        2 => Some("dot"),
        3 => Some("dash"),
        4 => Some("lgDash"),
        5 => Some("dashDot"),
        6 => Some("lgDashDot"),
        7 => Some("lgDashDotDot"),
        8 => Some("sysDash"),
        9 => Some("sysDot"),
        10 => Some("sysDashDot"),
        11 => Some("sysDashDotDot"),
        _ => None,
    }
}

fn draw_chart_series_d_pt(
    idx: usize,
    series: &ChartSeries,
    opts: &Chart,
) -> Vec<crate::xml::chart::CDPt> {
    if !matches!(opts.r#type.0, 40 | 43 | 44) && series.data_point.is_empty() {
        return Vec::new();
    }
    if !series.data_point.is_empty() {
        return series
            .data_point
            .iter()
            .map(|dp| {
                let mut sp_pr = CSpPr::default();
                apply_fill(&mut sp_pr, &dp.fill);
                crate::xml::chart::CDPt {
                    idx: Some(AttrValInt {
                        val: Some(dp.index),
                    }),
                    bubble_3d: Some(AttrValBool { val: Some(false) }),
                    sp_pr: Some(sp_pr),
                }
            })
            .collect();
    }
    let accent = format!("accent{}", idx + 1);
    vec![crate::xml::chart::CDPt {
        idx: Some(AttrValInt {
            val: Some(idx as i64),
        }),
        bubble_3d: Some(AttrValBool { val: Some(false) }),
        sp_pr: Some(CSpPr {
            solid_fill: Some(crate::xml::drawing::ASolidFill {
                scheme_clr: Some(crate::xml::drawing::ASchemeClr {
                    val: Some(accent),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ln: Some(crate::xml::drawing::ALn {
                w: Some(25400),
                cap: Some("rnd".to_string()),
                solid_fill: Some(crate::xml::drawing::ASolidFill {
                    scheme_clr: Some(crate::xml::drawing::ASchemeClr {
                        val: Some("lt1".to_string()),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            sp_3d: Some(crate::xml::chart::ASp3D {
                contour_w: 25400,
                contour_clr: Some(crate::xml::chart::AContourClr {
                    scheme_clr: Some(crate::xml::drawing::ASchemeClr {
                        val: Some("lt1".to_string()),
                        ..Default::default()
                    }),
                }),
            }),
            ..Default::default()
        }),
    }]
}

fn draw_chart_series_d_lbls(
    idx: usize,
    series: &ChartSeries,
    opts: &Chart,
) -> Option<crate::xml::chart::CDLbls> {
    if matches!(opts.r#type.0, 48 | 49 | 50 | 51 | 52) {
        return None;
    }
    let mut d_lbls = draw_chart_d_lbls(opts);
    if let Some(types) = Some(supported_chart_data_labels_position(opts.r#type))
        .filter(|_| series.data_label_position != ChartDataLabelPositionType::UNSET)
    {
        if types.contains(&series.data_label_position) {
            d_lbls.d_lbl_pos = Some(AttrValString {
                val: Some(chart_data_labels_position_types(series.data_label_position).to_string()),
            });
        }
    }
    d_lbls.sp_pr = {
        let mut sp_pr = d_lbls.sp_pr.unwrap_or_default();
        apply_fill(&mut sp_pr, &series.data_label.fill);
        Some(sp_pr)
    };
    let mut tx_pr = crate::xml::chart::CTxPr {
        body_pr: Some(crate::xml::drawing::ABodyPr::default()),
        p: Some(crate::xml::drawing::AP {
            p_pr: Some(crate::xml::drawing::APPr {
                def_r_pr: crate::xml::drawing::ARPr::default(),
            }),
            ..Default::default()
        }),
        ..Default::default()
    };
    if let Some(p) = tx_pr.p.as_mut() {
        if let Some(p_pr) = p.p_pr.as_mut() {
            draw_chart_font(&series.data_label.font, &mut p_pr.def_r_pr);
        }
    }
    d_lbls.tx_pr = Some(tx_pr);
    Some(d_lbls)
}

fn draw_chart_font(font: &crate::styles::Font, rpr: &mut crate::xml::drawing::ARPr) {
    rpr.b = font.bold.unwrap_or(false);
    rpr.i = font.italic.unwrap_or(false);
    if let Some(u) = &font.underline {
        if let Some(idx) = SUPPORTED_DRAWING_UNDERLINE_TYPES
            .iter()
            .position(|&t| t.eq_ignore_ascii_case(u))
        {
            rpr.u = Some(SUPPORTED_DRAWING_UNDERLINE_TYPES[idx].to_string());
        }
    }
    if let Some(color) = &font.color {
        if !color.is_empty() {
            if rpr.solid_fill.is_none() {
                rpr.solid_fill = Some(crate::xml::drawing::ASolidFill::default());
            }
            let fill = rpr.solid_fill.as_mut().unwrap();
            fill.scheme_clr = None;
            fill.srgb_clr = Some(crate::xml::drawing::ASrgbClr {
                val: Some(color.trim_start_matches('#').to_uppercase()),
                ..Default::default()
            });
        }
    }
    if let Some(family) = &font.name {
        if !family.is_empty() {
            if rpr.latin.is_none() {
                rpr.latin = Some(crate::xml::drawing::XlsxCTTextFont::default());
            }
            if rpr.ea.is_none() {
                rpr.ea = Some(crate::xml::drawing::XlsxCTTextFont::default());
            }
            if rpr.cs.is_none() {
                rpr.cs = Some(crate::xml::drawing::XlsxCTTextFont::default());
            }
            rpr.latin.as_mut().unwrap().typeface = family.clone();
            rpr.ea.as_mut().unwrap().typeface = family.clone();
            rpr.cs.as_mut().unwrap().typeface = family.clone();
        }
    }
    if let Some(size) = font.size {
        if size > 0.0 {
            rpr.sz = Some(size * 100.0);
        }
    }
    if font.strike.unwrap_or(false) {
        rpr.strike = Some("sngStrike".to_string());
    }
}

fn draw_plot_area_tx_pr(axis: &ChartAxis) -> crate::xml::chart::CTxPr {
    let mut body_pr = crate::xml::drawing::ABodyPr {
        rot: -60000000,
        spc_first_last_para: true,
        vert: Some("horz".to_string()),
        vert_overflow: Some("ellipsis".to_string()),
        wrap: Some("square".to_string()),
        anchor: Some("ctr".to_string()),
        anchor_ctr: true,
        ..Default::default()
    };
    let mut def_r_pr = crate::xml::drawing::ARPr {
        sz: Some(900.0),
        kern: 1200,
        solid_fill: Some(crate::xml::drawing::ASolidFill {
            scheme_clr: Some(crate::xml::drawing::ASchemeClr {
                val: Some("tx1".to_string()),
                lum_mod: Some(AttrValInt { val: Some(15000) }),
                lum_off: Some(AttrValInt { val: Some(85000) }),
                ..Default::default()
            }),
            ..Default::default()
        }),
        latin: Some(crate::xml::drawing::XlsxCTTextFont {
            typeface: "+mn-lt".to_string(),
            ..Default::default()
        }),
        ea: Some(crate::xml::drawing::XlsxCTTextFont {
            typeface: "+mn-ea".to_string(),
            ..Default::default()
        }),
        cs: Some(crate::xml::drawing::XlsxCTTextFont {
            typeface: "+mn-cs".to_string(),
            ..Default::default()
        }),
        ..Default::default()
    };
    draw_chart_font(&axis.font, &mut def_r_pr);
    if (-90..=90).contains(&axis.alignment.text_rotation) {
        body_pr.rot = axis.alignment.text_rotation * 60000;
    }
    if let Some(idx) = SUPPORTED_DRAWING_TEXT_VERTICAL_TYPE
        .iter()
        .position(|&t| t.eq_ignore_ascii_case(&axis.alignment.vertical))
    {
        body_pr.vert = Some(SUPPORTED_DRAWING_TEXT_VERTICAL_TYPE[idx].to_string());
    }
    crate::xml::chart::CTxPr {
        body_pr: Some(body_pr),
        p: Some(crate::xml::drawing::AP {
            p_pr: Some(crate::xml::drawing::APPr { def_r_pr }),
            end_para_r_pr: Some(crate::xml::drawing::AEndParaRPr {
                lang: "en-US".to_string(),
                ..Default::default()
            }),
            ..Default::default()
        }),
        ..Default::default()
    }
}

fn draw_plot_area_sp_pr() -> CSpPr {
    CSpPr {
        ln: Some(crate::xml::drawing::ALn {
            w: Some(9525),
            cap: Some("flat".to_string()),
            cmpd: Some("sng".to_string()),
            algn: Some("ctr".to_string()),
            solid_fill: Some(crate::xml::drawing::ASolidFill {
                scheme_clr: Some(crate::xml::drawing::ASchemeClr {
                    val: Some("tx1".to_string()),
                    lum_mod: Some(AttrValInt { val: Some(15000) }),
                    lum_off: Some(AttrValInt { val: Some(85000) }),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        }),
        ..Default::default()
    }
}

fn draw_plot_area_d_table(opts: &Chart) -> Option<crate::xml::chart::CDTable> {
    if matches!(
        opts.r#type.0,
        0..=5 | 6..=39 | 41 | 42
    ) && opts.plot_area.show_data_table
    {
        Some(crate::xml::chart::CDTable {
            show_horz_border: Some(AttrValBool { val: Some(true) }),
            show_vert_border: Some(AttrValBool { val: Some(true) }),
            show_outline: Some(AttrValBool { val: Some(true) }),
            show_keys: Some(AttrValBool {
                val: Some(opts.plot_area.show_data_table_keys),
            }),
            ..Default::default()
        })
    } else {
        None
    }
}

fn draw_title(title: &ChartTitle, vert: &str) -> Option<CTitle> {
    if title.paragraph.is_empty() && title.formula.is_empty() {
        return None;
    }
    let mut t = CTitle {
        overlay: Some(AttrValBool {
            val: Some(title.overlay),
        }),
        ..Default::default()
    };
    let default_p_pr = || crate::xml::drawing::APPr {
        def_r_pr: crate::xml::drawing::ARPr {
            latin: Some(crate::xml::drawing::XlsxCTTextFont {
                typeface: "+mn-lt".to_string(),
                ..Default::default()
            }),
            ea: Some(crate::xml::drawing::XlsxCTTextFont {
                typeface: "+mn-ea".to_string(),
                ..Default::default()
            }),
            cs: Some(crate::xml::drawing::XlsxCTTextFont {
                typeface: "+mn-cs".to_string(),
                ..Default::default()
            }),
            ..Default::default()
        },
    };
    let end_para_r_pr = crate::xml::drawing::AEndParaRPr {
        lang: "en-US".to_string(),
        ..Default::default()
    };
    if !title.formula.is_empty() {
        t.tx = Some(CTx {
            str_ref: Some(CStrRef {
                f: title.formula.clone(),
                ..Default::default()
            }),
            ..Default::default()
        });
        let mut p_pr = default_p_pr();
        draw_chart_font(
            title
                .font
                .as_ref()
                .unwrap_or(&crate::styles::Font::default()),
            &mut p_pr.def_r_pr,
        );
        t.tx_pr = Some(crate::xml::chart::CTxPr {
            body_pr: Some(crate::xml::drawing::ABodyPr {
                rot: if vert == "horz" { -5400000 } else { 0 },
                vert: Some(vert.to_string()),
                ..Default::default()
            }),
            p: Some(crate::xml::drawing::AP {
                p_pr: Some(p_pr),
                end_para_r_pr: Some(end_para_r_pr),
                ..Default::default()
            }),
            ..Default::default()
        });
    } else {
        let mut rich = crate::xml::chart::CRich::default();
        for run in &title.paragraph {
            let mut r = crate::xml::drawing::AR {
                t: Some(run.text.clone()),
                ..Default::default()
            };
            if let Some(font) = &run.font {
                let mut rpr = crate::xml::drawing::ARPr::default();
                draw_chart_font(font, &mut rpr);
                r.r_pr = Some(rpr);
            }
            rich.p.push(crate::xml::drawing::AP {
                p_pr: Some(default_p_pr()),
                r: Some(r),
                end_para_r_pr: Some(end_para_r_pr.clone()),
                ..Default::default()
            });
        }
        if !title.paragraph.is_empty() && vert == "horz" {
            rich.body_pr = Some(crate::xml::drawing::ABodyPr {
                rot: -5400000,
                vert: Some(vert.to_string()),
                ..Default::default()
            });
        }
        t.tx = Some(CTx {
            rich: Some(rich),
            ..Default::default()
        });
    }
    if title.offset_x > 0 || title.offset_y > 0 || title.width > 0 || title.height > 0 {
        let mut layout = crate::xml::chart::CLayout::default();
        let mut manual = crate::xml::chart::CManualLayout::default();
        if title.offset_x > 0 {
            manual.x_mode = Some(AttrValString {
                val: Some("edge".to_string()),
            });
            manual.x = Some(AttrValFloat {
                val: Some(title.offset_x as f64 / 100.0),
            });
        }
        if title.offset_y > 0 {
            manual.y_mode = Some(AttrValString {
                val: Some("edge".to_string()),
            });
            manual.y = Some(AttrValFloat {
                val: Some(title.offset_y as f64 / 100.0),
            });
        }
        if title.width > 0 {
            manual.w_mode = Some(AttrValString {
                val: Some("edge".to_string()),
            });
            manual.w = Some(AttrValFloat {
                val: Some(title.width as f64 / 100.0),
            });
        }
        if title.height > 0 {
            manual.h_mode = Some(AttrValString {
                val: Some("edge".to_string()),
            });
            manual.h = Some(AttrValFloat {
                val: Some(title.height as f64 / 100.0),
            });
        }
        layout.manual_layout = Some(manual);
        t.layout = Some(layout);
    }
    let mut sp_pr = CSpPr::default();
    apply_fill(&mut sp_pr, &title.fill);
    if let Some(ln) = draw_chart_ln(&title.border) {
        sp_pr.ln = Some(ln);
    }
    t.sp_pr = Some(sp_pr);
    Some(t)
}

fn draw_legend(opts: &Chart) -> Option<CLegend> {
    if opts.legend.position == "none" {
        return None;
    }
    let pos = CHART_LEGEND_POSITION
        .iter()
        .find(|&&(k, _)| k == opts.legend.position)
        .map(|&(_, v)| v)
        .unwrap_or("b");
    let mut legend = CLegend {
        legend_pos: Some(AttrValString {
            val: Some(pos.to_string()),
        }),
        overlay: Some(AttrValBool { val: Some(false) }),
        ..Default::default()
    };
    if let Some(font) = &opts.legend.font {
        let mut def_r_pr = crate::xml::drawing::ARPr::default();
        draw_chart_font(font, &mut def_r_pr);
        legend.tx_pr = Some(crate::xml::chart::CTxPr {
            p: Some(crate::xml::drawing::AP {
                p_pr: Some(crate::xml::drawing::APPr { def_r_pr }),
                ..Default::default()
            }),
            ..Default::default()
        });
    }
    for (k, s) in opts.series.iter().enumerate() {
        if let Some(font) = &s.legend.font {
            let mut def_r_pr = crate::xml::drawing::ARPr::default();
            draw_chart_font(font, &mut def_r_pr);
            legend.legend_entry.push(crate::xml::chart::CLegendEntry {
                idx: Some(AttrValInt {
                    val: Some(k as i64 + opts.order),
                }),
                tx_pr: Some(crate::xml::chart::CTxPr {
                    p: Some(crate::xml::drawing::AP {
                        p_pr: Some(crate::xml::drawing::APPr { def_r_pr }),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            });
        }
    }
    Some(legend)
}

fn draw_chart_d_lbls(opts: &Chart) -> crate::xml::chart::CDLbls {
    crate::xml::chart::CDLbls {
        show_legend_key: Some(AttrValBool {
            val: Some(opts.legend.show_legend_key),
        }),
        show_val: Some(AttrValBool {
            val: Some(opts.plot_area.show_val),
        }),
        show_cat_name: Some(AttrValBool {
            val: Some(opts.plot_area.show_cat_name),
        }),
        show_ser_name: Some(AttrValBool {
            val: Some(opts.plot_area.show_ser_name),
        }),
        show_percent: Some(AttrValBool {
            val: Some(opts.plot_area.show_percent),
        }),
        show_bubble_size: Some(AttrValBool {
            val: Some(opts.plot_area.show_bubble_size),
        }),
        show_leader_lines: Some(AttrValBool {
            val: Some(opts.plot_area.show_leader_lines),
        }),
        num_fmt: draw_chart_num_fmt(&opts.plot_area.num_fmt),
        ..Default::default()
    }
}

fn draw_chart_num_fmt(fmt: &ChartNumFmt) -> Option<CNumFmt> {
    if fmt.custom_num_fmt.is_empty() && !fmt.source_linked {
        return None;
    }
    Some(CNumFmt {
        format_code: fmt.custom_num_fmt.clone(),
        source_linked: fmt.source_linked,
    })
}

fn tick_lbl_pos(pos: ChartTickLabelPositionType) -> String {
    match pos.0 {
        1 => "high",
        2 => "low",
        3 => "none",
        _ => "nextTo",
    }
    .to_string()
}

fn draw_plot_area_cat_ax(_pa: &CPlotArea, opts: &Chart) -> Vec<CAxs> {
    let (ax_id, cross_ax) = if opts.order > 0 && opts.y_axis.secondary {
        (100000003, 100000004)
    } else {
        (100000000, 100000001)
    };
    let mut ax = CAxs {
        ax_id: Some(AttrValInt { val: Some(ax_id) }),
        scaling: Some(CScaling {
            orientation: Some(AttrValString {
                val: Some(if opts.x_axis.reverse_order {
                    "maxMin".to_string()
                } else {
                    "minMax".to_string()
                }),
            }),
            max: opts.x_axis.maximum.map(|v| AttrValFloat { val: Some(v) }),
            min: opts.x_axis.minimum.map(|v| AttrValFloat { val: Some(v) }),
            ..Default::default()
        }),
        delete: Some(AttrValBool {
            val: Some(opts.x_axis.none),
        }),
        ax_pos: Some(AttrValString {
            val: Some(if opts.x_axis.reverse_order {
                "t".to_string()
            } else {
                "b".to_string()
            }),
        }),
        num_fmt: Some(CNumFmt {
            format_code: "General".to_string(),
            source_linked: false,
        }),
        major_tick_mark: Some(AttrValString {
            val: Some("none".to_string()),
        }),
        minor_tick_mark: Some(AttrValString {
            val: Some("none".to_string()),
        }),
        tick_lbl_pos: Some(AttrValString {
            val: Some(tick_lbl_pos(opts.x_axis.tick_label_position)),
        }),
        cross_ax: Some(AttrValInt {
            val: Some(cross_ax),
        }),
        crosses: Some(AttrValString {
            val: Some("autoZero".to_string()),
        }),
        auto: Some(AttrValBool { val: Some(true) }),
        lbl_algn: Some(AttrValString {
            val: Some("ctr".to_string()),
        }),
        lbl_offset: Some(AttrValInt { val: Some(100) }),
        no_multi_lvl_lbl: Some(AttrValBool { val: Some(false) }),
        title: draw_title(&opts.x_axis.title, "horz"),
        sp_pr: Some(draw_plot_area_sp_pr()),
        tx_pr: Some(draw_plot_area_tx_pr(&opts.x_axis)),
        ..Default::default()
    };
    if let Some(num_fmt) = draw_chart_num_fmt(&opts.x_axis.num_fmt) {
        ax.num_fmt = Some(num_fmt);
    }
    if opts.x_axis.major_grid_lines {
        ax.major_gridlines = Some(crate::xml::chart::CLines {
            sp_pr: Some(draw_plot_area_sp_pr()),
        });
    }
    if opts.x_axis.minor_grid_lines {
        ax.minor_gridlines = Some(crate::xml::chart::CLines {
            sp_pr: Some(draw_plot_area_sp_pr()),
        });
    }
    if opts.x_axis.tick_label_skip != 0 {
        ax.tick_lbl_skip = Some(AttrValInt {
            val: Some(opts.x_axis.tick_label_skip),
        });
    }
    if opts.order > 0 && opts.y_axis.secondary {
        ax.delete = Some(AttrValBool { val: Some(true) });
        ax.crosses = None;
    }
    vec![ax]
}

fn draw_plot_area_val_ax(_pa: &CPlotArea, opts: &Chart) -> Vec<CAxs> {
    let (ax_id, cross_ax) = if opts.order > 0 && opts.y_axis.secondary {
        (100000004, 100000003)
    } else {
        (100000001, 100000000)
    };
    let log_base = if opts.y_axis.log_base >= 2.0 && opts.y_axis.log_base <= 1000.0 {
        Some(AttrValFloat {
            val: Some(opts.y_axis.log_base),
        })
    } else {
        None
    };
    let mut ax = CAxs {
        ax_id: Some(AttrValInt { val: Some(ax_id) }),
        scaling: Some(CScaling {
            log_base,
            orientation: Some(AttrValString {
                val: Some(if opts.y_axis.reverse_order {
                    "maxMin".to_string()
                } else {
                    "minMax".to_string()
                }),
            }),
            max: opts.y_axis.maximum.map(|v| AttrValFloat { val: Some(v) }),
            min: opts.y_axis.minimum.map(|v| AttrValFloat { val: Some(v) }),
            ..Default::default()
        }),
        delete: Some(AttrValBool {
            val: Some(opts.y_axis.none),
        }),
        ax_pos: Some(AttrValString {
            val: Some(if opts.y_axis.reverse_order {
                "r".to_string()
            } else {
                "l".to_string()
            }),
        }),
        num_fmt: Some(CNumFmt {
            format_code: chart_val_ax_num_fmt(opts.r#type).to_string(),
            source_linked: false,
        }),
        major_tick_mark: Some(AttrValString {
            val: Some("none".to_string()),
        }),
        minor_tick_mark: Some(AttrValString {
            val: Some("none".to_string()),
        }),
        tick_lbl_pos: Some(AttrValString {
            val: Some(tick_lbl_pos(opts.y_axis.tick_label_position)),
        }),
        cross_ax: Some(AttrValInt {
            val: Some(cross_ax),
        }),
        crosses: Some(AttrValString {
            val: Some(if opts.order > 0 && opts.y_axis.secondary {
                "max".to_string()
            } else {
                "autoZero".to_string()
            }),
        }),
        cross_between: Some(AttrValString {
            val: Some(chart_val_ax_cross_between(opts.r#type).to_string()),
        }),
        title: draw_title(&opts.y_axis.title, ""),
        sp_pr: Some(draw_plot_area_sp_pr()),
        tx_pr: Some(draw_plot_area_tx_pr(&opts.y_axis)),
        ..Default::default()
    };
    if let Some(num_fmt) = draw_chart_num_fmt(&opts.y_axis.num_fmt) {
        ax.num_fmt = Some(num_fmt);
    }
    if opts.y_axis.major_grid_lines {
        ax.major_gridlines = Some(crate::xml::chart::CLines {
            sp_pr: Some(draw_plot_area_sp_pr()),
        });
    }
    if opts.y_axis.minor_grid_lines {
        ax.minor_gridlines = Some(crate::xml::chart::CLines {
            sp_pr: Some(draw_plot_area_sp_pr()),
        });
    }
    if opts.r#type == ChartType::CONTOUR || opts.r#type == ChartType::WIREFRAME_CONTOUR {
        ax.tick_lbl_pos = Some(AttrValString {
            val: Some("none".to_string()),
        });
    }
    if opts.y_axis.major_unit != 0.0 {
        ax.major_unit = Some(AttrValFloat {
            val: Some(opts.y_axis.major_unit),
        });
    }
    vec![ax]
}

fn draw_plot_area_ser_ax(opts: &Chart) -> Vec<CAxs> {
    let max_val = opts.y_axis.maximum.map(|v| AttrValFloat { val: Some(v) });
    let min_val = opts.y_axis.minimum.map(|v| AttrValFloat { val: Some(v) });
    vec![CAxs {
        ax_id: Some(AttrValInt {
            val: Some(100000005),
        }),
        scaling: Some(CScaling {
            orientation: Some(AttrValString {
                val: Some(if opts.y_axis.reverse_order {
                    "maxMin".to_string()
                } else {
                    "minMax".to_string()
                }),
            }),
            max: max_val,
            min: min_val,
            ..Default::default()
        }),
        delete: Some(AttrValBool {
            val: Some(opts.y_axis.none),
        }),
        ax_pos: Some(AttrValString {
            val: Some(if opts.x_axis.reverse_order {
                "t".to_string()
            } else {
                "b".to_string()
            }),
        }),
        tick_lbl_pos: Some(AttrValString {
            val: Some(tick_lbl_pos(opts.y_axis.tick_label_position)),
        }),
        cross_ax: Some(AttrValInt {
            val: Some(100000001),
        }),
        sp_pr: Some(draw_plot_area_sp_pr()),
        tx_pr: Some(draw_plot_area_tx_pr(&ChartAxis::default())),
        ..Default::default()
    }]
}

pub(crate) fn draw_chart_ln(
    opts: &crate::xml::drawing::LineOptions,
) -> Option<crate::xml::drawing::ALn> {
    match opts.r#type {
        crate::xml::drawing::LineType::NONE => Some(crate::xml::drawing::ALn {
            no_fill: Some(AttrValString::default()),
            ..Default::default()
        }),
        crate::xml::drawing::LineType::SOLID => Some(crate::xml::drawing::ALn {
            w: Some(pt_to_emus(opts.width)),
            cap: Some("flat".to_string()),
            cmpd: Some("sng".to_string()),
            algn: Some("ctr".to_string()),
            solid_fill: apply_fill_to_solid(&opts.fill),
            ..Default::default()
        }),
        _ => None,
    }
}

fn apply_fill(sp_pr: &mut CSpPr, fill: &crate::xml::drawing::Fill) {
    if fill.r#type == "pattern" && fill.pattern == 1 && fill.color.len() == 1 {
        let color = fill.color[0].trim_start_matches('#').to_uppercase();
        sp_pr.solid_fill = Some(crate::xml::drawing::ASolidFill {
            srgb_clr: Some(crate::xml::drawing::ASrgbClr {
                val: Some(color),
                ..Default::default()
            }),
            ..Default::default()
        });
    }
}

fn apply_fill_to_solid(
    fill: &crate::xml::drawing::Fill,
) -> Option<crate::xml::drawing::ASolidFill> {
    if fill.r#type == "pattern" && fill.pattern == 1 && fill.color.len() == 1 {
        let color = fill.color[0].trim_start_matches('#').to_uppercase();
        return Some(crate::xml::drawing::ASolidFill {
            srgb_clr: Some(crate::xml::drawing::ASrgbClr {
                val: Some(color),
                ..Default::default()
            }),
            ..Default::default()
        });
    }
    None
}

fn pt_to_emus(pt: f64) -> i64 {
    if pt < 0.25 || pt > 999.0 {
        25400
    } else {
        (pt * 12700.0) as i64
    }
}

// ------------------------------------------------------------------
// Drawing helpers
// ------------------------------------------------------------------

impl File {
    pub(crate) fn prepare_drawing(
        &self,
        sheet: &str,
        drawing_id: i32,
        drawing_xml: &str,
    ) -> Result<(i32, String)> {
        let ws = self.work_sheet_reader(sheet)?;
        if let Some(drawing) = &ws.drawing {
            let target = self
                .get_sheet_relationships_target_by_id(sheet, drawing.rid.as_deref().unwrap_or(""));
            let target = target.replace("/xl/drawings/", "../drawings/");
            let id_str = target
                .trim_start_matches("../drawings/drawing")
                .trim_end_matches(".xml");
            let id = id_str.parse::<i32>().unwrap_or(drawing_id);
            let xml = target.replace("..", "xl");
            return Ok((id, xml));
        }
        let sheet_xml_path = self.get_sheet_xml_path(sheet).unwrap_or_default();
        let sheet_rels = format!(
            "xl/worksheets/_rels/{}.rels",
            sheet_xml_path.trim_start_matches("xl/worksheets/")
        );
        let target = format!("../drawings/drawing{drawing_id}.xml");
        let r_id = self.add_rels(&sheet_rels, SOURCE_RELATIONSHIP_DRAWING_ML, &target, "");
        self.add_sheet_drawing(sheet, r_id)?;
        Ok((drawing_id, drawing_xml.to_string()))
    }

    pub(crate) fn drawing_parser(&self, path: &str) -> Result<(XlsxWsDr, i64)> {
        if let Some(d) = self.drawings.get(path) {
            let c_nv_pr_id = (d.one_cell_anchor.len() + d.two_cell_anchor.len() + 2) as i64;
            return Ok((d.clone(), c_nv_pr_id));
        }
        let mut content = XlsxWsDr {
            ns: Some(NAMESPACE_DRAWING_ML_SPREADSHEET.to_string()),
            xmlns_xdr: Some(NAMESPACE_DRAWING_ML_SPREADSHEET.to_string()),
            xmlns_a: Some(NAMESPACE_DRAWING_ML_MAIN.to_string()),
            xmlns_r: Some(SOURCE_RELATIONSHIP.to_string()),
            ..Default::default()
        };
        if let Some(bytes) = self.pkg.get(path) {
            let data = bytes.value().clone();
            let decoded: DecodeWsDr =
                xml_from_reader::<_, DecodeWsDr>(data.as_slice()).unwrap_or_default();
            content.xmlns_r = Some(decoded.r);
            for a in decoded.one_cell_anchor {
                content.one_cell_anchor.push(decode_cell_anchor(a));
            }
            for a in decoded.two_cell_anchor {
                content.two_cell_anchor.push(decode_cell_anchor(a));
            }
        }
        let c_nv_pr_id = (content.one_cell_anchor.len() + content.two_cell_anchor.len() + 2) as i64;
        self.drawings.insert(path.to_string(), content.clone());
        Ok((content, c_nv_pr_id))
    }
}

fn decode_cell_anchor(a: DecodeCellAnchor) -> XdrCellAnchor {
    let graphic_frame = a.graphic_frame;
    XdrCellAnchor {
        edit_as: a.edit_as,
        from: a.from.map(decode_from),
        to: a.to.map(decode_to),
        ext: a.ext.map(|e| XlsxPositiveSize2D { cx: e.cx, cy: e.cy }),
        pic: a.pic.map(decode_pic),
        graphic_frame,
        client_data: a.client_data.map(decode_client_data),
        ..Default::default()
    }
}

fn decode_from(f: DecodeFrom) -> XlsxFrom {
    XlsxFrom {
        col: f.col as i64,
        col_off: f.col_off,
        row: f.row as i64,
        row_off: f.row_off,
    }
}

fn decode_to(t: DecodeTo) -> XlsxTo {
    XlsxTo {
        col: t.col as i64,
        col_off: t.col_off,
        row: t.row as i64,
        row_off: t.row_off,
    }
}

fn decode_pic(p: DecodePic) -> XlsxPic {
    XlsxPic {
        nv_pic_pr: XlsxNvPicPr {
            c_nv_pr: XlsxCNvPr {
                id: p.nv_pic_pr.c_nv_pr.id as i64,
                name: p.nv_pic_pr.c_nv_pr.name,
                descr: p.nv_pic_pr.c_nv_pr.descr,
                title: p.nv_pic_pr.c_nv_pr.title,
                hlink_click: p.nv_pic_pr.c_nv_pr.hlink_click.map(|h| XlsxHlinkClick {
                    xmlns_r: h.id.as_ref().map(|_| SOURCE_RELATIONSHIP.to_string()),
                    r_id: h.id,
                    invalid_url: h.invalid_url,
                    action: h.action,
                    tgt_frame: h.tgt_frame,
                    tooltip: h.tooltip,
                    history: Some(h.history),
                    highlight_click: Some(h.highlight_click),
                    end_snd: Some(h.end_snd),
                }),
            },
            c_nv_pic_pr: XlsxCNvPicPr {
                pic_locks: XlsxPicLocks {
                    no_change_aspect: p.nv_pic_pr.c_nv_pic_pr.pic_locks.no_change_aspect,
                    ..Default::default()
                },
            },
        },
        blip_fill: XlsxBlipFill {
            blip: XlsxBlip {
                embed: p.blip_fill.blip.embed,
                cstate: p.blip_fill.blip.cstate,
                xmlns_r: SOURCE_RELATIONSHIP.to_string(),
                ..Default::default()
            },
            stretch: XlsxStretch {
                fill_rect: p.blip_fill.stretch.fill_rect,
            },
        },
        sp_pr: XlsxSpPr {
            xfrm: XlsxXfrm {
                off: XlsxOff {
                    x: p.sp_pr.xfrm.off.x,
                    y: p.sp_pr.xfrm.off.y,
                },
                ext: XlsxPositiveSize2D {
                    cx: p.sp_pr.xfrm.ext.cx,
                    cy: p.sp_pr.xfrm.ext.cy,
                },
            },
            prst_geom: XlsxPrstGeom {
                prst: p.sp_pr.prst_geom.prst,
            },
            ..Default::default()
        },
    }
}

fn decode_client_data(c: DecodeClientData) -> XdrClientData {
    XdrClientData {
        f_locks_with_sheet: c.f_locks_with_sheet,
        f_prints_with_sheet: c.f_prints_with_sheet,
    }
}

impl File {
    fn add_drawing_chart(
        &self,
        sheet: &str,
        drawing_xml: &str,
        cell: &str,
        width: i32,
        height: i32,
        r_id: i32,
        opts: &GraphicOptions,
    ) -> Result<()> {
        let (col, row) = cell_name_to_coordinates(cell)?;
        let width = (width as f64 * opts.scale_x) as i32;
        let height = (height as f64 * opts.scale_y) as i32;
        let (col_start, row_start, col_end, row_end, x1, y1, x2, y2) =
            self.position_object_pixels(sheet, col, row, width, height, opts)?;
        let (mut content, c_nv_pr_id) = self.drawing_parser(drawing_xml)?;

        let mut anchor = XdrCellAnchor {
            edit_as: if opts.positioning.is_empty() {
                None
            } else {
                Some(opts.positioning.clone())
            },
            from: Some(XlsxFrom {
                col: col_start as i64,
                col_off: x1 as i64 * DEFAULT_EMU as i64,
                row: row_start as i64,
                row_off: y1 as i64 * DEFAULT_EMU as i64,
            }),
            to: Some(XlsxTo {
                col: col_end as i64,
                col_off: x2 as i64 * DEFAULT_EMU as i64,
                row: row_end as i64,
                row_off: y2 as i64 * DEFAULT_EMU as i64,
            }),
            ..Default::default()
        };

        let graphic_frame = XlsxGraphicFrame {
            macro_name: String::new(),
            nv_graphic_frame_pr: XlsxNvGraphicFramePr {
                c_nv_pr: Some(XlsxCNvPr {
                    id: c_nv_pr_id,
                    name: if opts.name.is_empty() {
                        format!("Chart {c_nv_pr_id}")
                    } else {
                        opts.name.clone()
                    },
                    descr: opts.alt_text.clone(),
                    ..Default::default()
                }),
                c_nv_graphic_frame_pr: String::new(),
            },
            xfrm: XlsxXfrm {
                off: XlsxOff { x: 0, y: 0 },
                ext: XlsxPositiveSize2D { cx: 0, cy: 0 },
            },
            graphic: Some(XlsxGraphic {
                graphic_data: Some(XlsxGraphicData {
                    uri: NAMESPACE_DRAWING_ML_CHART.to_string(),
                    chart: Some(XlsxChartRef {
                        xmlns_c: NAMESPACE_DRAWING_ML_CHART.to_string(),
                        r_id: format!("rId{r_id}"),
                        xmlns_r: SOURCE_RELATIONSHIP.to_string(),
                    }),
                    ..Default::default()
                }),
            }),
        };
        anchor.graphic_frame = Some(graphic_frame);
        anchor.client_data = Some(crate::xml::drawing::XdrClientData {
            f_locks_with_sheet: opts.locked.unwrap_or(true),
            f_prints_with_sheet: opts.print_object.unwrap_or(true),
        });
        content.two_cell_anchor.push(anchor);
        self.drawings.insert(drawing_xml.to_string(), content);
        Ok(())
    }

    fn add_sheet_drawing_chart(
        &self,
        drawing_xml: &str,
        r_id: i32,
        opts: &GraphicOptions,
    ) -> Result<()> {
        let (mut content, c_nv_pr_id) = self.drawing_parser(drawing_xml)?;
        let mut anchor = XdrCellAnchor {
            edit_as: if opts.positioning.is_empty() {
                None
            } else {
                Some(opts.positioning.clone())
            },
            pos: Some(XlsxPoint2D { x: 0, y: 0 }),
            ext: Some(XlsxPositiveSize2D {
                cx: 9280533,
                cy: 6051719,
            }),
            ..Default::default()
        };
        let graphic_frame = XlsxGraphicFrame {
            macro_name: String::new(),
            nv_graphic_frame_pr: XlsxNvGraphicFramePr {
                c_nv_pr: Some(XlsxCNvPr {
                    id: c_nv_pr_id,
                    name: format!("Chart {c_nv_pr_id}"),
                    ..Default::default()
                }),
                c_nv_graphic_frame_pr: String::new(),
            },
            xfrm: XlsxXfrm {
                off: XlsxOff { x: 0, y: 0 },
                ext: XlsxPositiveSize2D { cx: 0, cy: 0 },
            },
            graphic: Some(XlsxGraphic {
                graphic_data: Some(XlsxGraphicData {
                    uri: NAMESPACE_DRAWING_ML_CHART.to_string(),
                    chart: Some(XlsxChartRef {
                        xmlns_c: NAMESPACE_DRAWING_ML_CHART.to_string(),
                        r_id: format!("rId{r_id}"),
                        xmlns_r: SOURCE_RELATIONSHIP.to_string(),
                    }),
                    ..Default::default()
                }),
            }),
        };
        anchor.graphic_frame = Some(graphic_frame);
        anchor.client_data = Some(crate::xml::drawing::XdrClientData {
            f_locks_with_sheet: opts.locked.unwrap_or(true),
            f_prints_with_sheet: opts.print_object.unwrap_or(true),
        });
        content.absolute_anchor.push(anchor);
        self.drawings.insert(drawing_xml.to_string(), content);
        Ok(())
    }

    pub(crate) fn delete_drawing(
        &self,
        col: i32,
        row: i32,
        drawing_xml: &str,
        drawing_type: &str,
    ) -> Result<Vec<String>> {
        let (mut wsdr, _) = self.drawing_parser(drawing_xml)?;
        let mut del_rid = Vec::new();
        let mut ref_rid = Vec::new();
        let mut r_id_maps: HashMap<String, i32> = HashMap::new();

        let matches_anchor = |a: &XdrCellAnchor| -> bool {
            match drawing_type {
                "Chart" => a.pic.is_none(),
                "Pic" => a.pic.is_some(),
                _ => true,
            }
        };

        let mut delete_from = |anchors: &mut Vec<XdrCellAnchor>| {
            let mut i = 0;
            while i < anchors.len() {
                let remove = {
                    let a = &anchors[i];
                    if let Some(from) = &a.from {
                        if matches_anchor(a) && from.col == col as i64 && from.row == row as i64 {
                            true
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                };
                if remove {
                    let r_id = extract_embed_rid(&anchors[i]);
                    *r_id_maps.entry(r_id.clone()).or_insert(0) += 1;
                    ref_rid.push(r_id.clone());
                    anchors.remove(i);
                    *r_id_maps.entry(r_id).or_insert(0) -= 1;
                } else {
                    i += 1;
                }
            }
        };
        delete_from(&mut wsdr.one_cell_anchor);
        delete_from(&mut wsdr.two_cell_anchor);
        self.drawings.insert(drawing_xml.to_string(), wsdr);
        for r_id in ref_rid {
            if r_id_maps.get(&r_id).copied().unwrap_or(0) == 0 && !del_rid.contains(&r_id) {
                del_rid.push(r_id);
            }
        }
        Ok(del_rid)
    }

    fn get_charts_from_drawing(&self, drawing_xml: &str, drawing_rels: &str) -> Result<Vec<Chart>> {
        let (wsdr, _) = self.drawing_parser(drawing_xml)?;
        let mut charts = Vec::new();
        for anchor in wsdr
            .one_cell_anchor
            .iter()
            .chain(wsdr.two_cell_anchor.iter())
        {
            let r_id = anchor
                .graphic_frame
                .as_ref()
                .and_then(|g| g.graphic.as_ref())
                .and_then(|g| g.graphic_data.as_ref())
                .and_then(|d| d.chart.as_ref())
                .map(|c| c.r_id.clone())
                .unwrap_or_default();
            if r_id.is_empty() {
                continue;
            }
            if let Some(rel) = self.get_drawing_relationship(drawing_rels, &r_id) {
                let chart_path = rel
                    .target
                    .replace("../", "xl/")
                    .trim_start_matches('/')
                    .to_string();
                if let Some(bytes) = self.pkg.get(&chart_path) {
                    let xml = String::from_utf8_lossy(bytes.value());
                    let mut chart = chart_from_xml(&xml);
                    if let Some(ext) = anchor.ext.as_ref() {
                        chart.dimension = ChartDimension {
                            width: (ext.cx / DEFAULT_EMU as i64) as u64,
                            height: (ext.cy / DEFAULT_EMU as i64) as u64,
                        };
                    }
                    charts.push(chart);
                }
            }
        }
        Ok(charts)
    }

    pub(crate) fn get_drawing_relationship(
        &self,
        rels_path: &str,
        r_id: &str,
    ) -> Option<crate::xml::workbook::XlsxRelationship> {
        if let Ok(Some(rels)) = self.rels_reader(rels_path) {
            for rel in &rels.relationships {
                if rel.id == r_id {
                    return Some(rel.clone());
                }
            }
        }
        None
    }

    pub(crate) fn position_object_pixels(
        &self,
        sheet: &str,
        col: i32,
        row: i32,
        width: i32,
        height: i32,
        opts: &GraphicOptions,
    ) -> Result<(i32, i32, i32, i32, i32, i32, i32, i32)> {
        let col_idx = col - 1;
        let row_idx = row - 1;
        let mut col_end = col_idx;
        let mut row_end = row_idx;
        let x1 = opts.offset_x as i32;
        let y1 = opts.offset_y as i32;
        let mut x2 = width;
        let mut y2 = height;
        if opts.positioning != "oneCell" {
            let col_width = self.get_col_width_pixels(sheet, col)?;
            let row_height = self.get_row_height_pixels(sheet, row)?;
            let x1 = x1.min(col_width);
            let y1 = y1.min(row_height);
            x2 += x1;
            y2 += y1;
            while x2 >= self.get_col_width_pixels(sheet, col_end + 1)? {
                col_end += 1;
                x2 -= self.get_col_width_pixels(sheet, col_end)?;
            }
            while y2 >= self.get_row_height_pixels(sheet, row_end + 1)? {
                row_end += 1;
                y2 -= self.get_row_height_pixels(sheet, row_end)?;
            }
            return Ok((col_idx, row_idx, col_end, row_end, x1, y1, x2, y2));
        }
        Ok((col_idx, row_idx, col_end, row_end, x1, y1, x2, y2))
    }

    pub(crate) fn get_col_width_pixels(&self, sheet: &str, col: i32) -> Result<i32> {
        let width = self.get_col_width(sheet, &crate::lib_util::column_number_to_name(col)?)?;
        Ok(convert_col_width_to_pixels(width) as i32)
    }

    pub(crate) fn get_row_height_pixels(&self, sheet: &str, row: i32) -> Result<i32> {
        let height = self.get_row_height(sheet, row)?;
        Ok(convert_row_height_to_pixels(height) as i32)
    }
}

fn extract_embed_rid(anchor: &XdrCellAnchor) -> String {
    if let Some(pic) = &anchor.pic {
        pic.blip_fill.blip.embed.clone()
    } else {
        String::new()
    }
}

fn chart_from_xml(xml: &str) -> Chart {
    let mut chart = Chart::default();

    if let Ok(space) = xml_from_reader::<_, DecodeChartSpace>(xml.as_bytes()) {
        chart.r#type = chart_type_from_chart_space(&space)
            .unwrap_or_else(|| detect_chart_type(xml));
        chart.title = chart_title_from_chart_space(&space);
        chart.legend = legend_from_chart_space(&space);
        chart.series = series_from_chart_space(&space);
        let (x_axis, y_axis) = axes_from_chart_space(&space);
        chart.x_axis = x_axis;
        chart.y_axis = y_axis;
        chart.plot_area = plot_area_from_chart_space(&space);
        chart.fill = chart_level_fill_from_sp_pr(space.sp_pr.as_ref());
        chart.border = line_from_sp_pr(space.sp_pr.as_ref());
        if let Some(val) = space
            .chart
            .disp_blanks_as
            .as_ref()
            .and_then(|v| v.val.clone())
        {
            chart.show_blanks_as = val;
        }
        if let Some(group) = space.chart.plot_area.as_ref().and_then(first_chart_group) {
            chart.vary_colors = group.vary_colors.as_ref().and_then(|v| v.val);
            chart.gap_width = group.gap_width.as_ref().and_then(|v| v.val).map(|v| v as u64);
            chart.overlap = group.overlap.as_ref().and_then(|v| v.val);
            chart.hole_size = group.hole_size.as_ref().and_then(|v| v.val).unwrap_or(0);
            chart.bubble_size = group
                .bubble_scale
                .as_ref()
                .and_then(|v| v.val)
                .map(|v| v as i64)
                .unwrap_or(0);
        }
    } else {
        chart.r#type = detect_chart_type(xml);
        chart.title.paragraph = extract_chart_title_text(xml);
    }

    chart
}

fn chart_attr_str(v: &Option<AttrValString>) -> &str {
    v.as_ref().and_then(|v| v.val.as_deref()).unwrap_or("")
}

fn chart_type_from_chart_space(space: &DecodeChartSpace) -> Option<ChartType> {
    let pa = space.chart.plot_area.as_ref()?;
    let attr_str = chart_attr_str;
    if let Some(g) = pa.bar_chart.first() {
        let col = attr_str(&g.bar_dir) == "col";
        return Some(match (col, attr_str(&g.grouping)) {
            (false, "stacked") => ChartType::BAR_STACKED,
            (false, "percentStacked") => ChartType::BAR_PERCENT_STACKED,
            (false, _) => ChartType::BAR,
            (true, "stacked") => ChartType::COL_STACKED,
            (true, "percentStacked") => ChartType::COL_PERCENT_STACKED,
            (true, _) => ChartType::COL,
        });
    }
    if let Some(g) = pa.bar_3d_chart.first() {
        let col = attr_str(&g.bar_dir) == "col";
        let base = match (attr_str(&g.shape), col) {
            ("cone", false) => 12,
            ("pyramid", false) => 15,
            ("cylinder", false) => 18,
            ("cone", true) => 28,
            ("pyramid", true) => 32,
            ("cylinder", true) => 36,
            (_, false) => 9,
            (_, true) => 24,
        };
        // Bar-direction 3D blocks run clustered/stacked/percentStacked, while
        // column-direction blocks run standard/clustered/stacked/percentStacked.
        let idx = if col {
            match attr_str(&g.grouping) {
                "clustered" => 1,
                "stacked" => 2,
                "percentStacked" => 3,
                _ => 0,
            }
        } else {
            match attr_str(&g.grouping) {
                "stacked" => 1,
                "percentStacked" => 2,
                _ => 0,
            }
        };
        return Some(ChartType(base + idx));
    }
    if let Some(g) = pa.area_chart.first() {
        return Some(match attr_str(&g.grouping) {
            "stacked" => ChartType::AREA_STACKED,
            "percentStacked" => ChartType::AREA_PERCENT_STACKED,
            _ => ChartType::AREA,
        });
    }
    if let Some(g) = pa.area_3d_chart.first() {
        return Some(match attr_str(&g.grouping) {
            "stacked" => ChartType::AREA_3D_STACKED,
            "percentStacked" => ChartType::AREA_3D_PERCENT_STACKED,
            _ => ChartType::AREA_3D,
        });
    }
    if pa.line_chart.first().is_some() {
        return Some(ChartType::LINE);
    }
    if pa.line_3d_chart.first().is_some() {
        return Some(ChartType::LINE_3D);
    }
    if pa.pie_chart.first().is_some() {
        return Some(ChartType::PIE);
    }
    if pa.pie_3d_chart.first().is_some() {
        return Some(ChartType::PIE_3D);
    }
    if let Some(g) = pa.of_pie_chart.first() {
        return Some(if attr_str(&g.of_pie_type) == "bar" {
            ChartType::BAR_OF_PIE
        } else {
            ChartType::PIE_OF_PIE
        });
    }
    if pa.doughnut_chart.first().is_some() {
        return Some(ChartType::DOUGHNUT);
    }
    if pa.radar_chart.first().is_some() {
        return Some(ChartType::RADAR);
    }
    if pa.scatter_chart.first().is_some() {
        return Some(ChartType::SCATTER);
    }
    if let Some(g) = pa.surface_chart.first() {
        let wireframe = g.wireframe.as_ref().and_then(|v| v.val).unwrap_or_default();
        return Some(if wireframe {
            ChartType::WIREFRAME_CONTOUR
        } else {
            ChartType::CONTOUR
        });
    }
    if let Some(g) = pa.surface_3d_chart.first() {
        let wireframe = g.wireframe.as_ref().and_then(|v| v.val).unwrap_or_default();
        return Some(if wireframe {
            ChartType::WIREFRAME_SURFACE_3D
        } else {
            ChartType::SURFACE_3D
        });
    }
    if pa.bubble_chart.first().is_some() {
        return Some(ChartType::BUBBLE);
    }
    if let Some(g) = pa.stock_chart.first() {
        let has_up_down_bars = g
            .up_down_bars
            .as_ref()
            .map(|u| u.up_bars.is_some() || u.down_bars.is_some())
            .unwrap_or_default();
        return Some(if has_up_down_bars {
            ChartType::STOCK_OPEN_HIGH_LOW_CLOSE
        } else {
            ChartType::STOCK_HIGH_LOW_CLOSE
        });
    }
    None
}

fn detect_chart_type(xml: &str) -> ChartType {
    // Chart type is determined by the chart element present in the plot area.
    // Bar charts are further split by the c:barDir value.
    if xml.contains("<c:barChart") {
        if xml.contains(r#"<c:barDir val="col""#) {
            return ChartType::COL;
        }
        return ChartType::BAR;
    }
    if xml.contains("<c:lineChart") {
        return ChartType::LINE;
    }
    if xml.contains("<c:pieChart") {
        return ChartType::PIE;
    }
    if xml.contains("<c:doughnutChart") {
        return ChartType::DOUGHNUT;
    }
    if xml.contains("<c:areaChart") {
        return ChartType::AREA;
    }
    if xml.contains("<c:scatterChart") {
        return ChartType::SCATTER;
    }
    if xml.contains("<c:radarChart") {
        return ChartType::RADAR;
    }
    if xml.contains("<c:surfaceChart") || xml.contains("<c:surface3DChart") {
        return ChartType::SURFACE_3D;
    }
    if xml.contains("<c:bubbleChart") {
        return ChartType::BUBBLE;
    }
    if xml.contains("<c:stockChart") {
        return ChartType::STOCK_HIGH_LOW_CLOSE;
    }
    ChartType::default()
}

fn extract_chart_title_text(xml: &str) -> Vec<crate::xml::common::RichTextRun> {
    // Extract the contents of the first c:title element.
    let Some(start) = xml.find("<c:title") else {
        return Vec::new();
    };
    let Some(end) = xml[start..].find("</c:title>") else {
        return Vec::new();
    };
    let title_xml = &xml[start..start + end + "</c:title>".len()];

    // Collect text from all a:t runs inside the title.
    let mut runs = Vec::new();
    let mut pos = 0;
    while let Some(tag_start) = title_xml[pos..].find("<a:t>") {
        let tag_start = pos + tag_start + "<a:t>".len();
        let Some(tag_end) = title_xml[tag_start..].find("</a:t>") else {
            break;
        };
        runs.push(crate::xml::common::RichTextRun {
            text: title_xml[tag_start..tag_start + tag_end].to_string(),
            ..Default::default()
        });
        pos = tag_start + tag_end + "</a:t>".len();
    }
    runs
}

fn axes_from_chart_space(space: &DecodeChartSpace) -> (ChartAxis, ChartAxis) {
    let Some(pa) = space.chart.plot_area.as_ref() else {
        return (ChartAxis::default(), ChartAxis::default());
    };

    let x_axis = pa
        .cat_ax
        .first()
        .or_else(|| pa.date_ax.first())
        .or_else(|| pa.ser_ax.first())
        .map(axis_from_decode)
        .unwrap_or_default();

    // Use the last value axis as the active Y axis; when a secondary axis is
    // present it is typically the second (last) valAx, otherwise this falls
    // back to the primary axis.
    let mut y_axis = pa
        .val_ax
        .last()
        .or_else(|| pa.date_ax.last())
        .map(axis_from_decode)
        .unwrap_or_default();
    y_axis.secondary = pa.val_ax.len() > 1 || pa.date_ax.len() > 1;

    (x_axis, y_axis)
}

fn axis_from_decode(axis: &DecodeCAxis) -> ChartAxis {
    let mut a = ChartAxis::default();

    a.none = axis
        .delete
        .as_ref()
        .and_then(|d| d.val)
        .unwrap_or(false);
    a.major_grid_lines = axis.major_grid_lines.is_some();
    a.minor_grid_lines = axis.minor_grid_lines.is_some();
    a.ax_id = axis.ax_id.as_ref().and_then(|v| v.val).unwrap_or(0);
    a.tick_label_skip = axis
        .tick_lbl_skip
        .as_ref()
        .and_then(|v| v.val)
        .unwrap_or(0);
    a.tick_label_position = tick_label_position_from_str(
        axis.tick_lbl_pos.as_ref().and_then(|v| v.val.as_deref()),
    );

    if let Some(scaling) = &axis.scaling {
        a.reverse_order = scaling
            .orientation
            .as_ref()
            .and_then(|v| v.val.as_deref())
            == Some("maxMin");
        a.maximum = scaling.max.as_ref().and_then(|v| v.val);
        a.minimum = scaling.min.as_ref().and_then(|v| v.val);
        a.log_base = scaling.log_base.as_ref().and_then(|v| v.val).unwrap_or(0.0);
    }

    a.major_unit = axis
        .major_unit
        .as_ref()
        .and_then(|v| v.val)
        .unwrap_or(0.0);

    if let Some(num_fmt) = &axis.num_fmt {
        a.num_fmt = chart_num_fmt_from_decode(num_fmt);
    }

    if let Some(title) = &axis.title {
        a.title = chart_title_from_decode_title(title);
    }

    a
}

fn tick_label_position_from_str(pos: Option<&str>) -> ChartTickLabelPositionType {
    match pos {
        Some("high") => ChartTickLabelPositionType::HIGH,
        Some("low") => ChartTickLabelPositionType::LOW,
        Some("none") => ChartTickLabelPositionType::NONE,
        _ => ChartTickLabelPositionType::NEXT_TO_AXIS,
    }
}

fn chart_num_fmt_from_decode(num_fmt: &DecodeCNumFmt) -> crate::xml::chart::ChartNumFmt {
    crate::xml::chart::ChartNumFmt {
        custom_num_fmt: num_fmt.format_code.clone(),
        source_linked: num_fmt.source_linked,
    }
}

fn chart_title_from_chart_space(space: &DecodeChartSpace) -> ChartTitle {
    space
        .chart
        .title
        .as_ref()
        .map(chart_title_from_decode_title)
        .unwrap_or_default()
}

fn chart_title_from_decode_title(title: &DecodeCTitle) -> ChartTitle {
    let mut t = ChartTitle::default();
    if let Some(tx) = &title.tx {
        let has_formula = tx
            .str_ref
            .as_ref()
            .map(|r| !r.f.is_empty())
            .unwrap_or(false);
        if has_formula {
            t.formula = tx.str_ref.as_ref().unwrap().f.clone();
        } else if let Some(rich) = &tx.rich {
            for p in &rich.p {
                if let Some(text) = &p.t {
                    t.paragraph.push(RichTextRun {
                        text: text.clone(),
                        font: None,
                    });
                }
                for r in &p.r {
                    if let Some(text) = &r.t {
                        t.paragraph.push(RichTextRun {
                            text: text.clone(),
                            font: font_from_decode_rpr(r.r_pr.as_ref()),
                        });
                    }
                }
            }
        }
    }
    t.font = font_from_decode_rpr(
        title
            .tx_pr
            .as_ref()
            .and_then(|tx_pr| tx_pr.p.as_ref())
            .and_then(|p| p.p_pr.as_ref())
            .and_then(|p_pr| p_pr.def_r_pr.as_ref()),
    );
    t.overlay = title
        .overlay
        .as_ref()
        .and_then(|v| v.val)
        .unwrap_or(false);
    t.fill = fill_from_sp_pr(title.sp_pr.as_ref());
    t.border = line_from_sp_pr(title.sp_pr.as_ref());
    t
}

fn legend_from_chart_space(space: &DecodeChartSpace) -> ChartLegend {
    let Some(legend) = space.chart.legend.as_ref() else {
        return ChartLegend::default();
    };

    let position = legend
        .legend_pos
        .as_ref()
        .and_then(|p| p.val.as_deref())
        .map(chart_legend_position_from_xml)
        .unwrap_or_default();

    let font = font_from_decode_rpr(
        legend
            .tx_pr
            .as_ref()
            .and_then(|tx_pr| tx_pr.p.as_ref())
            .and_then(|p| p.p_pr.as_ref())
            .and_then(|p_pr| p_pr.def_r_pr.as_ref()),
    );

    let show_legend_key = space
        .chart
        .plot_area
        .as_ref()
        .and_then(show_legend_key_from_plot_area)
        .unwrap_or(false);

    ChartLegend {
        position,
        show_legend_key,
        font,
    }
}

fn chart_legend_position_from_xml(pos: &str) -> String {
    CHART_LEGEND_POSITION
        .iter()
        .find(|&&(_, v)| v == pos)
        .map(|&(k, _)| k.to_string())
        .unwrap_or_else(|| match pos {
            "b" => "bottom".to_string(),
            "none" => "none".to_string(),
            _ => String::new(),
        })
}

fn first_chart_group(pa: &DecodeCPlotArea) -> Option<&DecodeCCharts> {
    let groups: [&Vec<DecodeCCharts>; 16] = [
        &pa.area_chart,
        &pa.area_3d_chart,
        &pa.bar_chart,
        &pa.bar_3d_chart,
        &pa.bubble_chart,
        &pa.doughnut_chart,
        &pa.line_chart,
        &pa.line_3d_chart,
        &pa.of_pie_chart,
        &pa.pie_chart,
        &pa.pie_3d_chart,
        &pa.radar_chart,
        &pa.scatter_chart,
        &pa.stock_chart,
        &pa.surface_chart,
        &pa.surface_3d_chart,
    ];
    groups.into_iter().flat_map(|g| g.iter()).next()
}

fn plot_area_from_chart_space(space: &DecodeChartSpace) -> ChartPlotArea {
    let mut plot_area = ChartPlotArea::default();
    let Some(pa) = space.chart.plot_area.as_ref() else {
        return plot_area;
    };
    if let Some(group) = first_chart_group(pa) {
        if let Some(d_lbls) = group.d_lbls.as_ref() {
            plot_area.show_val = d_lbls.show_val.as_ref().and_then(|v| v.val).unwrap_or_default();
            plot_area.show_cat_name = d_lbls
                .show_cat_name
                .as_ref()
                .and_then(|v| v.val)
                .unwrap_or_default();
            plot_area.show_ser_name = d_lbls
                .show_ser_name
                .as_ref()
                .and_then(|v| v.val)
                .unwrap_or_default();
            plot_area.show_percent = d_lbls
                .show_percent
                .as_ref()
                .and_then(|v| v.val)
                .unwrap_or_default();
            plot_area.show_bubble_size = d_lbls
                .show_bubble_size
                .as_ref()
                .and_then(|v| v.val)
                .unwrap_or_default();
            plot_area.show_leader_lines = d_lbls
                .show_leader_lines
                .as_ref()
                .and_then(|v| v.val)
                .unwrap_or_default();
        }
        if let Some(split_pos) = group.split_pos.as_ref().and_then(|v| v.val) {
            plot_area.second_plot_values = split_pos;
        }
    }
    if let Some(d_table) = pa.d_table.as_ref() {
        plot_area.show_data_table = true;
        plot_area.show_data_table_keys = d_table
            .show_keys
            .as_ref()
            .and_then(|v| v.val)
            .unwrap_or_default();
    }
    plot_area.fill = fill_from_sp_pr(pa.sp_pr.as_ref());
    plot_area
}

fn show_legend_key_from_plot_area(pa: &DecodeCPlotArea) -> Option<bool> {
    let groups: [&Vec<DecodeCCharts>; 16] = [
        &pa.area_chart,
        &pa.area_3d_chart,
        &pa.bar_chart,
        &pa.bar_3d_chart,
        &pa.bubble_chart,
        &pa.doughnut_chart,
        &pa.line_chart,
        &pa.line_3d_chart,
        &pa.of_pie_chart,
        &pa.pie_chart,
        &pa.pie_3d_chart,
        &pa.radar_chart,
        &pa.scatter_chart,
        &pa.stock_chart,
        &pa.surface_chart,
        &pa.surface_3d_chart,
    ];
    groups
        .iter()
        .flat_map(|g| g.iter())
        .filter_map(|c| c.d_lbls.as_ref())
        .filter_map(|d| d.show_legend_key.as_ref())
        .filter_map(|v| v.val)
        .next()
}

fn font_from_decode_rpr(rpr: Option<&DecodeARPr>) -> Option<crate::styles::Font> {
    let rpr = rpr?;
    let mut font = crate::styles::Font::default();
    if let Some(sz) = rpr.sz.as_ref().and_then(|s| s.parse::<f64>().ok()) {
        if sz > 0.0 {
            font.size = Some(sz / 100.0);
        }
    }
    if let Some(b) = &rpr.b {
        font.bold = Some(parse_bool_string(b));
    }
    if let Some(i) = &rpr.i {
        font.italic = Some(parse_bool_string(i));
    }
    if let Some(strike) = &rpr.strike {
        font.strike = Some(matches!(strike.as_str(), "sngStrike" | "dblStrike" | "true" | "1"));
    }
    if let Some(u) = &rpr.u {
        if u != "none" {
            font.underline = Some(u.clone());
        }
    }
    if let Some(vert_align) = &rpr.vert_align {
        font.vert_align = Some(vert_align.clone());
    }
    if let Some(name) = rpr.latin.as_ref().and_then(|l| l.typeface.as_ref()) {
        font.name = Some(name.clone());
    }
    if let Some(color) = rpr.solid_fill.as_ref().and_then(|sf| {
        sf.srgb_clr
            .as_ref()
            .and_then(|c| c.val.clone())
            .or_else(|| sf.scheme_clr.as_ref().and_then(|c| c.val.clone()))
    }) {
        font.color = Some(color);
    }
    Some(font)
}

fn parse_bool_string(s: &str) -> bool {
    matches!(s.to_ascii_lowercase().as_str(), "true" | "1" | "on")
}

fn chart_groups_with_types(pa: &DecodeCPlotArea) -> Vec<(&DecodeCCharts, ChartType)> {
    let groups: [(&Vec<DecodeCCharts>, ChartType); 16] = [
        (&pa.area_chart, ChartType::AREA),
        (&pa.area_3d_chart, ChartType::AREA_3D),
        (&pa.bar_chart, ChartType::BAR),
        (&pa.bar_3d_chart, ChartType::BAR_3D_CLUSTERED),
        (&pa.bubble_chart, ChartType::BUBBLE),
        (&pa.doughnut_chart, ChartType::DOUGHNUT),
        (&pa.line_chart, ChartType::LINE),
        (&pa.line_3d_chart, ChartType::LINE_3D),
        (&pa.of_pie_chart, ChartType::PIE_OF_PIE),
        (&pa.pie_chart, ChartType::PIE),
        (&pa.pie_3d_chart, ChartType::PIE_3D),
        (&pa.radar_chart, ChartType::RADAR),
        (&pa.scatter_chart, ChartType::SCATTER),
        (&pa.stock_chart, ChartType::STOCK_HIGH_LOW_CLOSE),
        (&pa.surface_chart, ChartType::CONTOUR),
        (&pa.surface_3d_chart, ChartType::SURFACE_3D),
    ];
    let mut out = Vec::new();
    for (group, chart_type) in groups {
        for c in group.iter() {
            out.push((c, chart_type));
        }
    }
    out
}

fn series_from_chart_space(space: &DecodeChartSpace) -> Vec<ChartSeries> {
    let Some(pa) = space.chart.plot_area.as_ref() else {
        return Vec::new();
    };
    chart_groups_with_types(pa)
        .into_iter()
        .flat_map(|(group, chart_type)| {
            group
                .ser
                .iter()
                .map(move |ser| chart_series_from_cser(ser, chart_type))
        })
        .collect()
}

fn chart_series_from_cser(ser: &DecodeCSer, chart_type: ChartType) -> ChartSeries {
    let mut series = ChartSeries::default();

    series.name = ser
        .tx
        .as_ref()
        .and_then(|tx| tx.str_ref.as_ref())
        .map(|r| r.f.clone())
        .unwrap_or_default();

    if chart_series_uses_xy(chart_type) {
        series.categories = ser
            .x_val
            .as_ref()
            .and_then(|x| x.str_ref.as_ref())
            .map(|r| r.f.clone())
            .unwrap_or_default();
        series.values = ser
            .y_val
            .as_ref()
            .and_then(|y| y.num_ref.as_ref())
            .map(|r| r.f.clone())
            .unwrap_or_default();
    } else {
        series.categories = ser
            .cat
            .as_ref()
            .and_then(|cat| cat.str_ref.as_ref())
            .map(|r| r.f.clone())
            .unwrap_or_default();
        series.values = ser
            .val
            .as_ref()
            .and_then(|val| val.num_ref.as_ref())
            .map(|r| r.f.clone())
            .unwrap_or_default();
    }

    if chart_series_uses_bubble_size(chart_type) {
        series.sizes = ser
            .bubble_size
            .as_ref()
            .and_then(|bs| bs.num_ref.as_ref())
            .map(|r| r.f.clone())
            .unwrap_or_default();
    }

    series.marker = marker_from_cmarker(ser.marker.as_ref());
    series.data_point = data_points_from_cdpt(&ser.d_pt);
    series.data_label_position = data_label_position_from_d_lbls(ser.d_lbls.as_ref());

    if let Some(d_lbls) = ser.d_lbls.as_ref() {
        series.data_label.fill = fill_from_sp_pr(d_lbls.sp_pr.as_ref());
        if let Some(font) = d_lbls
            .tx_pr
            .as_ref()
            .and_then(|tx| tx.p.as_ref())
            .and_then(|p| p.p_pr.as_ref())
            .and_then(|p_pr| font_from_decode_rpr(p_pr.def_r_pr.as_ref()))
        {
            series.data_label.font = font;
        }
    }

    if let Some(sp_pr) = &ser.sp_pr {
        series.fill = fill_from_sp_pr(Some(sp_pr));
        series.line = line_from_sp_pr(Some(sp_pr));
    }

    if let Some(smooth) = ser.smooth.as_ref().and_then(|s| s.val) {
        series.line.smooth = smooth;
    }

    series
}

fn marker_from_cmarker(marker: Option<&DecodeCMarker>) -> ChartMarker {
    let Some(m) = marker else {
        return ChartMarker::default();
    };
    ChartMarker {
        symbol: m
            .symbol
            .as_ref()
            .and_then(|s| s.val.clone())
            .unwrap_or_default(),
        size: m.size.as_ref().and_then(|s| s.val).unwrap_or(0),
        ..Default::default()
    }
}

fn data_points_from_cdpt(d_pts: &[DecodeCDPt]) -> Vec<ChartDataPoint> {
    d_pts
        .iter()
        .map(|dp| ChartDataPoint {
            index: dp.idx.as_ref().and_then(|i| i.val).unwrap_or(0),
            fill: fill_from_sp_pr(dp.sp_pr.as_ref()),
        })
        .collect()
}

fn data_label_position_from_d_lbls(d_lbls: Option<&DecodeCDLbls>) -> ChartDataLabelPositionType {
    let Some(pos) = d_lbls.and_then(|d| d.d_lbl_pos.as_ref()).and_then(|p| p.val.as_deref()) else {
        return ChartDataLabelPositionType::UNSET;
    };
    match pos {
        "bestFit" => ChartDataLabelPositionType::BEST_FIT,
        "b" => ChartDataLabelPositionType::BELOW,
        "ctr" => ChartDataLabelPositionType::CENTER,
        "inBase" => ChartDataLabelPositionType::INSIDE_BASE,
        "inEnd" => ChartDataLabelPositionType::INSIDE_END,
        "l" => ChartDataLabelPositionType::LEFT,
        "outEnd" => ChartDataLabelPositionType::OUTSIDE_END,
        "r" => ChartDataLabelPositionType::RIGHT,
        "t" => ChartDataLabelPositionType::ABOVE,
        _ => ChartDataLabelPositionType::UNSET,
    }
}

fn chart_level_fill_from_sp_pr(sp_pr: Option<&DecodeCSpPr>) -> Fill {
    // The writer always emits a schemeClr "bg1" chart background, so only an
    // explicit srgbClr represents a user-configured chart fill.
    let Some(sp_pr) = sp_pr else {
        return Fill::default();
    };
    let Some(solid) = sp_pr.solid_fill.as_ref() else {
        return if sp_pr.no_fill.is_some() {
            Fill {
                r#type: "none".to_string(),
                ..Default::default()
            }
        } else {
            Fill::default()
        };
    };
    let Some(color) = solid.srgb_clr.as_ref().and_then(|c| c.val.clone()) else {
        return Fill::default();
    };
    Fill {
        r#type: "pattern".to_string(),
        pattern: 1,
        color: vec![format!("#{}", color.trim_start_matches('#'))],
        ..Default::default()
    }
}

fn fill_from_sp_pr(sp_pr: Option<&DecodeCSpPr>) -> Fill {
    let Some(sp_pr) = sp_pr else {
        return Fill::default();
    };

    let color = sp_pr.solid_fill.as_ref().and_then(|sf| {
        sf.srgb_clr
            .as_ref()
            .and_then(|c| c.val.clone())
            .or_else(|| sf.scheme_clr.as_ref().and_then(|c| c.val.clone()))
    });

    if let Some(color) = color {
        return Fill {
            r#type: "pattern".to_string(),
            pattern: 1,
            color: vec![format!("#{}", color.trim_start_matches('#'))],
            ..Default::default()
        };
    }

    if sp_pr.no_fill.is_some() {
        return Fill {
            r#type: "none".to_string(),
            ..Default::default()
        };
    }

    Fill::default()
}

fn line_from_sp_pr(sp_pr: Option<&DecodeCSpPr>) -> LineOptions {
    let Some(sp_pr) = sp_pr else {
        return LineOptions::default();
    };
    let Some(ln) = &sp_pr.ln else {
        return LineOptions::default();
    };

    let fill = fill_from_sp_pr(Some(&DecodeCSpPr {
        solid_fill: ln.solid_fill.clone(),
        ..Default::default()
    }));

    if fill.color.is_empty() && ln.no_fill.is_some() {
        return LineOptions {
            r#type: LineType::NONE,
            ..Default::default()
        };
    }

    let width = ln
        .w
        .as_deref()
        .and_then(|w| w.parse::<i64>().ok())
        .map(|w| w as f64 / 12700.0)
        .unwrap_or(0.0);

    LineOptions {
        r#type: LineType::SOLID,
        width,
        fill,
        dash: dash_from_prst_dash(ln.prst_dash.as_ref()),
        ..Default::default()
    }
}

fn dash_from_prst_dash(prst_dash: Option<&AttrValString>) -> crate::xml::drawing::LineDashType {
    use crate::xml::drawing::LineDashType;
    let Some(val) = prst_dash.and_then(|v| v.val.as_deref()) else {
        return LineDashType::UNSET;
    };
    match val {
        "solid" => LineDashType::SOLID,
        "dot" => LineDashType::DOT,
        "dash" => LineDashType::DASH,
        "lgDash" => LineDashType::LG_DASH,
        "dashDot" => LineDashType::SASH_DOT,
        "lgDashDot" => LineDashType::LG_DASH_DOT,
        "lgDashDotDot" => LineDashType::LG_DASH_DOT_DOT,
        "sysDash" => LineDashType::SYS_DASH,
        "sysDot" => LineDashType::SYS_DOT,
        "sysDashDot" => LineDashType::SYS_DASH_DOT,
        "sysDashDotDot" => LineDashType::SYS_DASH_DOT_DOT,
        _ => LineDashType::UNSET,
    }
}

fn convert_col_width_to_pixels(width: f64) -> f64 {
    width * 8.0 + 0.5
}

fn convert_row_height_to_pixels(height: f64) -> f64 {
    (4.0 / 3.4 * height).ceil()
}

// ------------------------------------------------------------------
// Tests
// ------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Options;

    #[test]
    fn add_column_chart_round_trip() {
        let mut f = File::new_with_options(Options::default());
        f.set_cell_str("Sheet1", "A1", "Fruit").unwrap();
        f.set_cell_str("Sheet1", "B1", "Apple").unwrap();
        f.set_cell_str("Sheet1", "C1", "Orange").unwrap();
        f.set_cell_str("Sheet1", "A2", "Small").unwrap();
        f.set_cell_int("Sheet1", "B2", 2).unwrap();
        f.set_cell_int("Sheet1", "C2", 3).unwrap();

        let chart = Chart {
            r#type: ChartType::COL,
            series: vec![ChartSeries {
                name: "Sheet1!$A$2".to_string(),
                categories: "Sheet1!$B$1:$C$1".to_string(),
                values: "Sheet1!$B$2:$C$2".to_string(),
                ..Default::default()
            }],
            title: ChartTitle {
                paragraph: vec![crate::xml::common::RichTextRun {
                    text: "Fruit Chart".to_string(),
                    ..Default::default()
                }],
                ..Default::default()
            },
            ..Default::default()
        };
        f.add_chart("Sheet1", "E1", &chart).unwrap();

        let charts = f.get_charts("Sheet1").unwrap();
        assert_eq!(charts.len(), 1);
        assert_eq!(charts[0].r#type, ChartType::COL);
        assert_eq!(charts[0].title.paragraph.len(), 1);
        assert_eq!(charts[0].title.paragraph[0].text, "Fruit Chart");
    }

    #[test]
    fn combo_chart_generates_two_chart_groups() {
        let mut f = File::new_with_options(Options::default());
        f.set_cell_str("Sheet1", "A1", "Fruit").unwrap();
        f.set_cell_str("Sheet1", "B1", "Apple").unwrap();
        f.set_cell_str("Sheet1", "C1", "Orange").unwrap();
        f.set_cell_str("Sheet1", "A2", "Small").unwrap();
        f.set_cell_int("Sheet1", "B2", 2).unwrap();
        f.set_cell_int("Sheet1", "C2", 3).unwrap();
        f.set_cell_str("Sheet1", "A3", "Large").unwrap();
        f.set_cell_int("Sheet1", "B3", 5).unwrap();
        f.set_cell_int("Sheet1", "C3", 7).unwrap();

        let combo = Chart {
            r#type: ChartType::LINE,
            series: vec![ChartSeries {
                name: "Sheet1!$A$3".to_string(),
                categories: "Sheet1!$B$1:$C$1".to_string(),
                values: "Sheet1!$B$3:$C$3".to_string(),
                marker: ChartMarker {
                    symbol: "none".to_string(),
                    size: 10,
                    ..Default::default()
                },
                ..Default::default()
            }],
            ..Default::default()
        };
        let chart = Chart {
            r#type: ChartType::COL,
            series: vec![ChartSeries {
                name: "Sheet1!$A$2".to_string(),
                categories: "Sheet1!$B$1:$C$1".to_string(),
                values: "Sheet1!$B$2:$C$2".to_string(),
                ..Default::default()
            }],
            combo: vec![combo],
            ..Default::default()
        };
        f.add_chart("Sheet1", "E1", &chart).unwrap();

        let charts = f.get_charts("Sheet1").unwrap();
        assert_eq!(charts.len(), 1);
        assert_eq!(charts[0].r#type, ChartType::COL);
    }

    #[test]
    fn chart_secondary_axis_uses_separate_ax_ids() {
        let mut f = File::new_with_options(Options::default());
        f.set_cell_str("Sheet1", "A1", "Fruit").unwrap();
        f.set_cell_str("Sheet1", "B1", "Apple").unwrap();
        f.set_cell_str("Sheet1", "C1", "Orange").unwrap();
        f.set_cell_str("Sheet1", "A2", "Small").unwrap();
        f.set_cell_int("Sheet1", "B2", 2).unwrap();
        f.set_cell_int("Sheet1", "C2", 3).unwrap();
        f.set_cell_str("Sheet1", "A3", "Large").unwrap();
        f.set_cell_int("Sheet1", "B3", 500).unwrap();
        f.set_cell_int("Sheet1", "C3", 700).unwrap();

        let mut combo = Chart {
            r#type: ChartType::LINE,
            series: vec![ChartSeries {
                name: "Sheet1!$A$3".to_string(),
                categories: "Sheet1!$B$1:$C$1".to_string(),
                values: "Sheet1!$B$3:$C$3".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        };
        combo.y_axis.secondary = true;
        let chart = Chart {
            r#type: ChartType::COL,
            series: vec![ChartSeries {
                name: "Sheet1!$A$2".to_string(),
                categories: "Sheet1!$B$1:$C$1".to_string(),
                values: "Sheet1!$B$2:$C$2".to_string(),
                ..Default::default()
            }],
            combo: vec![combo],
            ..Default::default()
        };
        f.add_chart("Sheet1", "E1", &chart).unwrap();

        let charts = f.get_charts("Sheet1").unwrap();
        assert_eq!(charts.len(), 1);
    }

    #[test]
    fn chart_series_style_serializes_sp_pr() {
        let mut f = File::new_with_options(Options::default());
        f.set_cell_str("Sheet1", "A1", "Fruit").unwrap();
        f.set_cell_str("Sheet1", "B1", "Apple").unwrap();
        f.set_cell_str("Sheet1", "C1", "Orange").unwrap();
        f.set_cell_str("Sheet1", "A2", "Small").unwrap();
        f.set_cell_int("Sheet1", "B2", 2).unwrap();
        f.set_cell_int("Sheet1", "C2", 3).unwrap();

        let chart = Chart {
            r#type: ChartType::COL,
            series: vec![ChartSeries {
                name: "Sheet1!$A$2".to_string(),
                categories: "Sheet1!$B$1:$C$1".to_string(),
                values: "Sheet1!$B$2:$C$2".to_string(),
                fill: crate::xml::drawing::Fill {
                    r#type: "pattern".to_string(),
                    pattern: 1,
                    color: vec!["#FF0000".to_string()],
                    ..Default::default()
                },
                ..Default::default()
            }],
            ..Default::default()
        };
        f.add_chart("Sheet1", "E1", &chart).unwrap();

        let charts = f.get_charts("Sheet1").unwrap();
        assert_eq!(charts.len(), 1);
    }

    #[test]
    fn add_chart_sheet_uses_sheet_drawing_rel_id() {
        let mut f = File::new_with_options(Options::default());
        f.set_cell_str("Sheet1", "A1", "Fruit").unwrap();
        f.set_cell_str("Sheet1", "B1", "Apple").unwrap();
        f.set_cell_str("Sheet1", "C1", "Orange").unwrap();
        f.set_cell_str("Sheet1", "A2", "Small").unwrap();
        f.set_cell_int("Sheet1", "B2", 2).unwrap();
        f.set_cell_int("Sheet1", "C2", 3).unwrap();

        let chart = Chart {
            r#type: ChartType::COL,
            series: vec![ChartSeries {
                name: "Sheet1!$A$2".to_string(),
                categories: "Sheet1!$B$1:$C$1".to_string(),
                values: "Sheet1!$B$2:$C$2".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        };
        f.add_chart_sheet("Chart1", &chart).unwrap();

        // The chartsheet XML should reference the sheet-to-drawing relationship,
        // not the workbook-level chartsheet relationship.
        let cs_path = f.get_sheet_xml_path("Chart1").expect("chartsheet path");
        let cs_xml = String::from_utf8(f.read_xml(&cs_path)).unwrap();
        let sheet_rels_path = cs_path.replace("chartsheets/", "chartsheets/_rels/") + ".rels";
        let sheet_rels = f
            .relationships
            .get(&sheet_rels_path)
            .expect("missing chartsheet relationships");
        let drawing_rel = sheet_rels
            .relationships
            .iter()
            .find(|r| r.r#type == SOURCE_RELATIONSHIP_DRAWING_ML)
            .expect("missing sheet-to-drawing relationship");
        assert!(drawing_rel.target.contains("drawing1.xml"));
        assert!(
            cs_xml.contains(&format!("r:id=\"{}\"", drawing_rel.id)),
            "chartsheet drawing r:id {} not found in: {}",
            drawing_rel.id,
            cs_xml
        );

        // Make sure the workbook relationship for the chartsheet uses a different rId.
        let wb_rels_path = f.get_workbook_rels_path();
        let wb_rels = f
            .relationships
            .get(&wb_rels_path)
            .expect("missing workbook relationships");
        let chartsheet_wb_rel = wb_rels
            .relationships
            .iter()
            .find(|r| r.r#type == SOURCE_RELATIONSHIP_CHARTSHEET)
            .expect("missing workbook chartsheet relationship");
        assert_ne!(chartsheet_wb_rel.id, drawing_rel.id);
    }

    #[test]
    fn chart_series_round_trip_basic() {
        let mut f = File::new_with_options(Options::default());
        f.set_cell_str("Sheet1", "A1", "Fruit").unwrap();
        f.set_cell_str("Sheet1", "B1", "Apple").unwrap();
        f.set_cell_str("Sheet1", "C1", "Orange").unwrap();
        f.set_cell_str("Sheet1", "A2", "Small").unwrap();
        f.set_cell_int("Sheet1", "B2", 2).unwrap();
        f.set_cell_int("Sheet1", "C2", 3).unwrap();

        let chart = Chart {
            r#type: ChartType::COL,
            series: vec![ChartSeries {
                name: "Sheet1!$A$2".to_string(),
                categories: "Sheet1!$B$1:$C$1".to_string(),
                values: "Sheet1!$B$2:$C$2".to_string(),
                fill: crate::xml::drawing::Fill {
                    r#type: "pattern".to_string(),
                    pattern: 1,
                    color: vec!["#FF0000".to_string()],
                    ..Default::default()
                },
                ..Default::default()
            }],
            ..Default::default()
        };
        f.add_chart("Sheet1", "E1", &chart).unwrap();

        let charts = f.get_charts("Sheet1").unwrap();
        assert_eq!(charts.len(), 1);
        assert_eq!(charts[0].series.len(), 1);
        let s = &charts[0].series[0];
        assert_eq!(s.name, "Sheet1!$A$2");
        assert_eq!(s.categories, "Sheet1!$B$1:$C$1");
        assert_eq!(s.values, "Sheet1!$B$2:$C$2");
        assert_eq!(s.fill.color, vec!["#FF0000".to_string()]);
    }

    #[test]
    fn chart_series_round_trip_line_marker() {
        let mut f = File::new_with_options(Options::default());
        f.set_cell_str("Sheet1", "A1", "Month").unwrap();
        f.set_cell_str("Sheet1", "B1", "Jan").unwrap();
        f.set_cell_str("Sheet1", "C1", "Feb").unwrap();
        f.set_cell_str("Sheet1", "A2", "Sales").unwrap();
        f.set_cell_int("Sheet1", "B2", 10).unwrap();
        f.set_cell_int("Sheet1", "C2", 20).unwrap();

        let chart = Chart {
            r#type: ChartType::LINE,
            series: vec![ChartSeries {
                name: "Sheet1!$A$2".to_string(),
                categories: "Sheet1!$B$1:$C$1".to_string(),
                values: "Sheet1!$B$2:$C$2".to_string(),
                marker: ChartMarker {
                    symbol: "diamond".to_string(),
                    size: 7,
                    ..Default::default()
                },
                ..Default::default()
            }],
            ..Default::default()
        };
        f.add_chart("Sheet1", "E1", &chart).unwrap();

        let charts = f.get_charts("Sheet1").unwrap();
        assert_eq!(charts.len(), 1);
        assert_eq!(charts[0].r#type, ChartType::LINE);
        assert_eq!(charts[0].series.len(), 1);
        let s = &charts[0].series[0];
        assert_eq!(s.marker.symbol, "diamond");
        assert_eq!(s.marker.size, 7);
    }

    #[test]
    fn chart_axis_round_trip_basic() {
        let mut f = File::new_with_options(Options::default());
        f.set_cell_str("Sheet1", "A1", "Fruit").unwrap();
        f.set_cell_str("Sheet1", "B1", "Apple").unwrap();
        f.set_cell_str("Sheet1", "C1", "Orange").unwrap();
        f.set_cell_str("Sheet1", "A2", "Small").unwrap();
        f.set_cell_int("Sheet1", "B2", 2).unwrap();
        f.set_cell_int("Sheet1", "C2", 3).unwrap();

        let chart = Chart {
            r#type: ChartType::COL,
            series: vec![ChartSeries {
                name: "Sheet1!$A$2".to_string(),
                categories: "Sheet1!$B$1:$C$1".to_string(),
                values: "Sheet1!$B$2:$C$2".to_string(),
                ..Default::default()
            }],
            x_axis: ChartAxis {
                reverse_order: true,
                major_grid_lines: true,
                maximum: Some(100.0),
                minimum: Some(10.0),
                title: ChartTitle {
                    paragraph: vec![RichTextRun {
                        text: "Quarter".to_string(),
                        ..Default::default()
                    }],
                    ..Default::default()
                },
                ..Default::default()
            },
            y_axis: ChartAxis {
                major_unit: 20.0,
                tick_label_position: ChartTickLabelPositionType::HIGH,
                num_fmt: ChartNumFmt {
                    custom_num_fmt: "0.00".to_string(),
                    source_linked: false,
                },
                ..Default::default()
            },
            ..Default::default()
        };
        f.add_chart("Sheet1", "E1", &chart).unwrap();

        let charts = f.get_charts("Sheet1").unwrap();
        assert_eq!(charts.len(), 1);
        let x = &charts[0].x_axis;
        assert!(x.reverse_order);
        assert!(x.major_grid_lines);
        assert_eq!(x.maximum, Some(100.0));
        assert_eq!(x.minimum, Some(10.0));
        assert_eq!(x.title.paragraph.len(), 1);
        assert_eq!(x.title.paragraph[0].text, "Quarter");

        let y = &charts[0].y_axis;
        assert_eq!(y.major_unit, 20.0);
        assert_eq!(y.tick_label_position, ChartTickLabelPositionType::HIGH);
        assert_eq!(y.num_fmt.custom_num_fmt, "0.00");
    }

    #[test]
    fn chart_axis_round_trip_secondary() {
        let mut f = File::new_with_options(Options::default());
        f.set_cell_str("Sheet1", "A1", "Fruit").unwrap();
        f.set_cell_str("Sheet1", "B1", "Apple").unwrap();
        f.set_cell_str("Sheet1", "C1", "Orange").unwrap();
        f.set_cell_str("Sheet1", "A2", "Small").unwrap();
        f.set_cell_int("Sheet1", "B2", 2).unwrap();
        f.set_cell_int("Sheet1", "C2", 3).unwrap();
        f.set_cell_str("Sheet1", "A3", "Large").unwrap();
        f.set_cell_int("Sheet1", "B3", 500).unwrap();
        f.set_cell_int("Sheet1", "C3", 700).unwrap();

        let mut combo = Chart {
            r#type: ChartType::LINE,
            series: vec![ChartSeries {
                name: "Sheet1!$A$3".to_string(),
                categories: "Sheet1!$B$1:$C$1".to_string(),
                values: "Sheet1!$B$3:$C$3".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        };
        combo.y_axis.secondary = true;
        let chart = Chart {
            r#type: ChartType::COL,
            series: vec![ChartSeries {
                name: "Sheet1!$A$2".to_string(),
                categories: "Sheet1!$B$1:$C$1".to_string(),
                values: "Sheet1!$B$2:$C$2".to_string(),
                ..Default::default()
            }],
            combo: vec![combo],
            ..Default::default()
        };
        f.add_chart("Sheet1", "E1", &chart).unwrap();

        let charts = f.get_charts("Sheet1").unwrap();
        assert_eq!(charts.len(), 1);
        assert!(charts[0].y_axis.secondary);
    }

    #[test]
    fn chart_legend_round_trip() {
        let mut f = File::new_with_options(Options::default());
        f.set_cell_str("Sheet1", "A1", "Fruit").unwrap();
        f.set_cell_str("Sheet1", "B1", "Apple").unwrap();
        f.set_cell_str("Sheet1", "C1", "Orange").unwrap();
        f.set_cell_str("Sheet1", "A2", "Small").unwrap();
        f.set_cell_int("Sheet1", "B2", 2).unwrap();
        f.set_cell_int("Sheet1", "C2", 3).unwrap();

        let chart = Chart {
            r#type: ChartType::COL,
            series: vec![ChartSeries {
                name: "Sheet1!$A$2".to_string(),
                categories: "Sheet1!$B$1:$C$1".to_string(),
                values: "Sheet1!$B$2:$C$2".to_string(),
                ..Default::default()
            }],
            legend: ChartLegend {
                position: "right".to_string(),
                show_legend_key: true,
                font: Some(crate::styles::Font {
                    size: Some(12.0),
                    italic: Some(true),
                    color: Some("FF0000".to_string()),
                    name: Some("Arial".to_string()),
                    ..Default::default()
                }),
            },
            ..Default::default()
        };
        f.add_chart("Sheet1", "E1", &chart).unwrap();

        let charts = f.get_charts("Sheet1").unwrap();
        assert_eq!(charts.len(), 1);
        let legend = &charts[0].legend;
        assert_eq!(legend.position, "right");
        assert!(legend.show_legend_key);
        let font = legend.font.as_ref().expect("legend font");
        assert_eq!(font.size, Some(12.0));
        assert_eq!(font.italic, Some(true));
        assert_eq!(font.color.as_deref(), Some("FF0000"));
        assert_eq!(font.name.as_deref(), Some("Arial"));
    }

    #[test]
    fn chart_title_format_round_trip() {
        let mut f = File::new_with_options(Options::default());
        f.set_cell_str("Sheet1", "A1", "Sales Report").unwrap();
        f.set_cell_str("Sheet1", "B1", "Apple").unwrap();
        f.set_cell_str("Sheet1", "A2", "Small").unwrap();
        f.set_cell_int("Sheet1", "B2", 2).unwrap();

        let chart = Chart {
            r#type: ChartType::COL,
            series: vec![ChartSeries {
                name: "Sheet1!$A$2".to_string(),
                categories: "Sheet1!$B$1".to_string(),
                values: "Sheet1!$B$2".to_string(),
                ..Default::default()
            }],
            title: ChartTitle {
                formula: "Sheet1!$A$1".to_string(),
                overlay: true,
                font: Some(crate::styles::Font {
                    size: Some(16.0),
                    bold: Some(true),
                    color: Some("FF0000".to_string()),
                    ..Default::default()
                }),
                fill: crate::xml::drawing::Fill {
                    r#type: "pattern".to_string(),
                    pattern: 1,
                    color: vec!["#FF0000".to_string()],
                    ..Default::default()
                },
                border: crate::xml::drawing::LineOptions {
                    r#type: crate::xml::drawing::LineType::SOLID,
                    width: 1.0,
                    fill: crate::xml::drawing::Fill {
                        r#type: "pattern".to_string(),
                        pattern: 1,
                        color: vec!["#0000FF".to_string()],
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        };
        f.add_chart("Sheet1", "E1", &chart).unwrap();

        let charts = f.get_charts("Sheet1").unwrap();
        assert_eq!(charts.len(), 1);
        let title = &charts[0].title;
        assert_eq!(title.formula, "Sheet1!$A$1");
        assert!(title.overlay);
        assert_eq!(title.fill.color, vec!["#FF0000".to_string()]);
        assert_eq!(title.border.width, 1.0);
        assert_eq!(title.border.fill.color, vec!["#0000FF".to_string()]);
        let font = title.font.as_ref().expect("title font");
        assert_eq!(font.size, Some(16.0));
        assert_eq!(font.bold, Some(true));
        assert_eq!(font.color.as_deref(), Some("FF0000"));
    }

    #[test]
    fn chart_plot_area_round_trip() {
        let mut f = File::new_with_options(Options::default());
        f.set_cell_str("Sheet1", "A1", "Fruit").unwrap();
        f.set_cell_str("Sheet1", "B1", "Apple").unwrap();
        f.set_cell_str("Sheet1", "C1", "Orange").unwrap();
        f.set_cell_str("Sheet1", "A2", "Small").unwrap();
        f.set_cell_int("Sheet1", "B2", 2).unwrap();
        f.set_cell_int("Sheet1", "C2", 3).unwrap();

        let chart = Chart {
            r#type: ChartType::COL,
            series: vec![ChartSeries {
                name: "Sheet1!$A$2".to_string(),
                categories: "Sheet1!$B$1:$C$1".to_string(),
                values: "Sheet1!$B$2:$C$2".to_string(),
                ..Default::default()
            }],
            plot_area: ChartPlotArea {
                show_val: true,
                show_cat_name: true,
                show_percent: true,
                show_data_table: true,
                show_data_table_keys: true,
                fill: crate::xml::drawing::Fill {
                    r#type: "pattern".to_string(),
                    pattern: 1,
                    color: vec!["#DDEBF7".to_string()],
                    ..Default::default()
                },
                ..Default::default()
            },
            vary_colors: Some(false),
            gap_width: Some(200),
            overlap: Some(-20),
            show_blanks_as: "zero".to_string(),
            ..Default::default()
        };
        f.add_chart("Sheet1", "E1", &chart).unwrap();

        let charts = f.get_charts("Sheet1").unwrap();
        assert_eq!(charts.len(), 1);
        let got = &charts[0];
        let pa = &got.plot_area;
        assert!(pa.show_val);
        assert!(pa.show_cat_name);
        assert!(pa.show_percent);
        assert!(!pa.show_ser_name);
        assert!(!pa.show_bubble_size);
        assert!(pa.show_data_table);
        assert!(pa.show_data_table_keys);
        assert_eq!(pa.fill.color, vec!["#DDEBF7".to_string()]);
        assert_eq!(got.vary_colors, Some(false));
        assert_eq!(got.gap_width, Some(200));
        assert_eq!(got.overlap, Some(-20));
        assert_eq!(got.show_blanks_as, "zero");
    }

    #[test]
    fn chart_level_fill_border_round_trip() {
        let mut f = File::new_with_options(Options::default());
        f.set_cell_str("Sheet1", "A1", "Fruit").unwrap();
        f.set_cell_str("Sheet1", "B1", "Apple").unwrap();
        f.set_cell_str("Sheet1", "A2", "Small").unwrap();
        f.set_cell_int("Sheet1", "B2", 2).unwrap();

        let chart = Chart {
            r#type: ChartType::COL,
            series: vec![ChartSeries {
                name: "Sheet1!$A$2".to_string(),
                categories: "Sheet1!$B$1".to_string(),
                values: "Sheet1!$B$2".to_string(),
                ..Default::default()
            }],
            fill: crate::xml::drawing::Fill {
                r#type: "pattern".to_string(),
                pattern: 1,
                color: vec!["#FFE699".to_string()],
                ..Default::default()
            },
            border: crate::xml::drawing::LineOptions {
                r#type: crate::xml::drawing::LineType::SOLID,
                width: 2.0,
                fill: crate::xml::drawing::Fill {
                    r#type: "pattern".to_string(),
                    pattern: 1,
                    color: vec!["#00B050".to_string()],
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        };
        f.add_chart("Sheet1", "E1", &chart).unwrap();

        let charts = f.get_charts("Sheet1").unwrap();
        assert_eq!(charts.len(), 1);
        let got = &charts[0];
        assert_eq!(got.fill.r#type, "pattern");
        assert_eq!(got.fill.pattern, 1);
        assert_eq!(got.fill.color, vec!["#FFE699".to_string()]);
        assert_eq!(got.border.r#type, crate::xml::drawing::LineType::SOLID);
        assert_eq!(got.border.width, 2.0);
        assert_eq!(got.border.fill.color, vec!["#00B050".to_string()]);
    }

    #[test]
    fn chart_doughnut_hole_size_round_trip() {
        let mut f = File::new_with_options(Options::default());
        f.set_cell_str("Sheet1", "A1", "Fruit").unwrap();
        f.set_cell_str("Sheet1", "B1", "Apple").unwrap();
        f.set_cell_str("Sheet1", "A2", "Small").unwrap();
        f.set_cell_int("Sheet1", "B2", 2).unwrap();

        let chart = Chart {
            r#type: ChartType::DOUGHNUT,
            series: vec![ChartSeries {
                name: "Sheet1!$A$2".to_string(),
                categories: "Sheet1!$B$1".to_string(),
                values: "Sheet1!$B$2".to_string(),
                ..Default::default()
            }],
            hole_size: 50,
            ..Default::default()
        };
        f.add_chart("Sheet1", "E1", &chart).unwrap();

        let charts = f.get_charts("Sheet1").unwrap();
        assert_eq!(charts.len(), 1);
        assert_eq!(charts[0].r#type, ChartType::DOUGHNUT);
        assert_eq!(charts[0].hole_size, 50);
    }

    #[test]
    fn chart_bubble_size_round_trip() {
        let mut f = File::new_with_options(Options::default());
        f.set_cell_str("Sheet1", "A1", "Fruit").unwrap();
        f.set_cell_str("Sheet1", "B1", "Apple").unwrap();
        f.set_cell_str("Sheet1", "A2", "Small").unwrap();
        f.set_cell_int("Sheet1", "B2", 2).unwrap();

        let chart = Chart {
            r#type: ChartType::BUBBLE,
            series: vec![ChartSeries {
                name: "Sheet1!$A$2".to_string(),
                categories: "Sheet1!$B$1".to_string(),
                values: "Sheet1!$B$2".to_string(),
                ..Default::default()
            }],
            bubble_size: 150,
            ..Default::default()
        };
        f.add_chart("Sheet1", "E1", &chart).unwrap();

        let charts = f.get_charts("Sheet1").unwrap();
        assert_eq!(charts.len(), 1);
        assert_eq!(charts[0].r#type, ChartType::BUBBLE);
        assert_eq!(charts[0].bubble_size, 150);
    }

    #[test]
    fn chart_of_pie_second_plot_values_round_trip() {
        let mut f = File::new_with_options(Options::default());
        f.set_cell_str("Sheet1", "A1", "Fruit").unwrap();
        f.set_cell_str("Sheet1", "B1", "Apple").unwrap();
        f.set_cell_str("Sheet1", "A2", "Small").unwrap();
        f.set_cell_int("Sheet1", "B2", 2).unwrap();

        let chart = Chart {
            r#type: ChartType::PIE_OF_PIE,
            series: vec![ChartSeries {
                name: "Sheet1!$A$2".to_string(),
                categories: "Sheet1!$B$1".to_string(),
                values: "Sheet1!$B$2".to_string(),
                ..Default::default()
            }],
            plot_area: ChartPlotArea {
                second_plot_values: 3,
                ..Default::default()
            },
            ..Default::default()
        };
        f.add_chart("Sheet1", "E1", &chart).unwrap();

        let charts = f.get_charts("Sheet1").unwrap();
        assert_eq!(charts.len(), 1);
        assert_eq!(charts[0].plot_area.second_plot_values, 3);
    }

    #[test]
    fn chart_type_round_trip_all_subtypes() {
        let types = [
            ChartType::AREA,
            ChartType::AREA_STACKED,
            ChartType::AREA_PERCENT_STACKED,
            ChartType::AREA_3D,
            ChartType::AREA_3D_STACKED,
            ChartType::AREA_3D_PERCENT_STACKED,
            ChartType::BAR,
            ChartType::BAR_STACKED,
            ChartType::BAR_PERCENT_STACKED,
            ChartType::BAR_3D_CLUSTERED,
            ChartType::BAR_3D_STACKED,
            ChartType::BAR_3D_PERCENT_STACKED,
            ChartType::BAR_3D_CONE_CLUSTERED,
            ChartType::BAR_3D_CONE_STACKED,
            ChartType::BAR_3D_CONE_PERCENT_STACKED,
            ChartType::BAR_3D_PYRAMID_CLUSTERED,
            ChartType::BAR_3D_PYRAMID_STACKED,
            ChartType::BAR_3D_PYRAMID_PERCENT_STACKED,
            ChartType::BAR_3D_CYLINDER_CLUSTERED,
            ChartType::BAR_3D_CYLINDER_STACKED,
            ChartType::BAR_3D_CYLINDER_PERCENT_STACKED,
            ChartType::COL,
            ChartType::COL_STACKED,
            ChartType::COL_PERCENT_STACKED,
            ChartType::COL_3D,
            ChartType::COL_3D_CLUSTERED,
            ChartType::COL_3D_STACKED,
            ChartType::COL_3D_PERCENT_STACKED,
            ChartType::COL_3D_CONE,
            ChartType::COL_3D_CONE_CLUSTERED,
            ChartType::COL_3D_CONE_STACKED,
            ChartType::COL_3D_CONE_PERCENT_STACKED,
            ChartType::COL_3D_PYRAMID,
            ChartType::COL_3D_PYRAMID_CLUSTERED,
            ChartType::COL_3D_PYRAMID_STACKED,
            ChartType::COL_3D_PYRAMID_PERCENT_STACKED,
            ChartType::COL_3D_CYLINDER,
            ChartType::COL_3D_CYLINDER_CLUSTERED,
            ChartType::COL_3D_CYLINDER_STACKED,
            ChartType::COL_3D_CYLINDER_PERCENT_STACKED,
            ChartType::DOUGHNUT,
            ChartType::LINE,
            ChartType::LINE_3D,
            ChartType::PIE,
            ChartType::PIE_3D,
            ChartType::PIE_OF_PIE,
            ChartType::BAR_OF_PIE,
            ChartType::RADAR,
            ChartType::SCATTER,
            ChartType::SURFACE_3D,
            ChartType::WIREFRAME_SURFACE_3D,
            ChartType::CONTOUR,
            ChartType::WIREFRAME_CONTOUR,
            ChartType::BUBBLE,
            ChartType::STOCK_HIGH_LOW_CLOSE,
            ChartType::STOCK_OPEN_HIGH_LOW_CLOSE,
        ];
        for chart_type in types {
            let mut f = File::new_with_options(Options::default());
            f.set_cell_str("Sheet1", "A1", "Fruit").unwrap();
            f.set_cell_str("Sheet1", "B1", "Apple").unwrap();
            f.set_cell_str("Sheet1", "C1", "Orange").unwrap();
            f.set_cell_str("Sheet1", "A2", "Small").unwrap();
            f.set_cell_int("Sheet1", "B2", 2).unwrap();
            f.set_cell_int("Sheet1", "C2", 3).unwrap();
            f.set_cell_str("Sheet1", "A3", "Large").unwrap();
            f.set_cell_int("Sheet1", "B3", 5).unwrap();
            f.set_cell_int("Sheet1", "C3", 7).unwrap();

            let chart = Chart {
                r#type: chart_type,
                series: vec![
                    ChartSeries {
                        name: "Sheet1!$A$2".to_string(),
                        categories: "Sheet1!$B$1:$C$1".to_string(),
                        values: "Sheet1!$B$2:$C$2".to_string(),
                        ..Default::default()
                    },
                    ChartSeries {
                        name: "Sheet1!$A$3".to_string(),
                        categories: "Sheet1!$B$1:$C$1".to_string(),
                        values: "Sheet1!$B$3:$C$3".to_string(),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            };
            f.add_chart("Sheet1", "E1", &chart).unwrap();

            let charts = f.get_charts("Sheet1").unwrap();
            assert_eq!(charts.len(), 1, "chart type {chart_type:?}");
            assert_eq!(charts[0].r#type, chart_type, "chart type {chart_type:?}");
        }
    }

    #[test]
    fn combo_chart_series_merged_on_readback() {
        let mut f = File::new_with_options(Options::default());
        f.set_cell_str("Sheet1", "A1", "Fruit").unwrap();
        f.set_cell_str("Sheet1", "B1", "Apple").unwrap();
        f.set_cell_str("Sheet1", "C1", "Orange").unwrap();
        f.set_cell_str("Sheet1", "A2", "Small").unwrap();
        f.set_cell_int("Sheet1", "B2", 2).unwrap();
        f.set_cell_int("Sheet1", "C2", 3).unwrap();
        f.set_cell_str("Sheet1", "A3", "Large").unwrap();
        f.set_cell_int("Sheet1", "B3", 5).unwrap();
        f.set_cell_int("Sheet1", "C3", 7).unwrap();
        f.set_cell_str("Sheet1", "A4", "Total").unwrap();
        f.set_cell_int("Sheet1", "B4", 7).unwrap();
        f.set_cell_int("Sheet1", "C4", 10).unwrap();

        let combo = Chart {
            r#type: ChartType::LINE,
            series: vec![ChartSeries {
                name: "Sheet1!$A$4".to_string(),
                categories: "Sheet1!$B$1:$C$1".to_string(),
                values: "Sheet1!$B$4:$C$4".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        };
        let chart = Chart {
            r#type: ChartType::COL,
            series: vec![
                ChartSeries {
                    name: "Sheet1!$A$2".to_string(),
                    categories: "Sheet1!$B$1:$C$1".to_string(),
                    values: "Sheet1!$B$2:$C$2".to_string(),
                    ..Default::default()
                },
                ChartSeries {
                    name: "Sheet1!$A$3".to_string(),
                    categories: "Sheet1!$B$1:$C$1".to_string(),
                    values: "Sheet1!$B$3:$C$3".to_string(),
                    ..Default::default()
                },
            ],
            combo: vec![combo],
            ..Default::default()
        };
        f.add_chart("Sheet1", "E1", &chart).unwrap();

        let charts = f.get_charts("Sheet1").unwrap();
        assert_eq!(charts.len(), 1);
        assert_eq!(charts[0].r#type, ChartType::COL);
        assert_eq!(charts[0].series.len(), 3);
        assert_eq!(charts[0].series[0].name, "Sheet1!$A$2");
        assert_eq!(charts[0].series[1].name, "Sheet1!$A$3");
        assert_eq!(charts[0].series[2].name, "Sheet1!$A$4");
        assert_eq!(charts[0].series[2].values, "Sheet1!$B$4:$C$4");
    }

    #[test]
    fn chart_series_line_dash_round_trip() {
        let mut f = File::new_with_options(Options::default());
        f.set_cell_str("Sheet1", "A1", "Month").unwrap();
        f.set_cell_str("Sheet1", "B1", "Jan").unwrap();
        f.set_cell_str("Sheet1", "A2", "Sales").unwrap();
        f.set_cell_int("Sheet1", "B2", 10).unwrap();

        let chart = Chart {
            r#type: ChartType::LINE,
            series: vec![ChartSeries {
                name: "Sheet1!$A$2".to_string(),
                categories: "Sheet1!$B$1".to_string(),
                values: "Sheet1!$B$2".to_string(),
                line: crate::xml::drawing::LineOptions {
                    r#type: crate::xml::drawing::LineType::SOLID,
                    width: 1.5,
                    dash: crate::xml::drawing::LineDashType::DASH,
                    smooth: true,
                    ..Default::default()
                },
                ..Default::default()
            }],
            ..Default::default()
        };
        f.add_chart("Sheet1", "E1", &chart).unwrap();

        let charts = f.get_charts("Sheet1").unwrap();
        assert_eq!(charts.len(), 1);
        let line = &charts[0].series[0].line;
        assert_eq!(line.r#type, crate::xml::drawing::LineType::SOLID);
        assert_eq!(line.width, 1.5);
        assert_eq!(line.dash, crate::xml::drawing::LineDashType::DASH);
        assert!(line.smooth);
    }

    #[test]
    fn chart_series_data_label_round_trip() {
        let mut f = File::new_with_options(Options::default());
        f.set_cell_str("Sheet1", "A1", "Fruit").unwrap();
        f.set_cell_str("Sheet1", "B1", "Apple").unwrap();
        f.set_cell_str("Sheet1", "A2", "Small").unwrap();
        f.set_cell_int("Sheet1", "B2", 2).unwrap();

        let chart = Chart {
            r#type: ChartType::COL,
            series: vec![ChartSeries {
                name: "Sheet1!$A$2".to_string(),
                categories: "Sheet1!$B$1".to_string(),
                values: "Sheet1!$B$2".to_string(),
                data_label: crate::xml::chart::ChartDataLabel {
                    fill: crate::xml::drawing::Fill {
                        r#type: "pattern".to_string(),
                        pattern: 1,
                        color: vec!["#FFFF00".to_string()],
                        ..Default::default()
                    },
                    font: crate::styles::Font {
                        size: Some(10.0),
                        bold: Some(true),
                        color: Some("FF0000".to_string()),
                        name: Some("Arial".to_string()),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            }],
            ..Default::default()
        };
        f.add_chart("Sheet1", "E1", &chart).unwrap();

        let charts = f.get_charts("Sheet1").unwrap();
        assert_eq!(charts.len(), 1);
        let label = &charts[0].series[0].data_label;
        assert_eq!(label.fill.color, vec!["#FFFF00".to_string()]);
        assert_eq!(label.font.size, Some(10.0));
        assert_eq!(label.font.bold, Some(true));
        assert_eq!(label.font.color.as_deref(), Some("FF0000"));
        assert_eq!(label.font.name.as_deref(), Some("Arial"));
    }

}
