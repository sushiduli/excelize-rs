//! Chart part (`xl/charts/chartN.xml`).
//!
//! Ported from Go `xmlChart.go`.

use serde::{Deserialize, Serialize};

use super::common::{
    AttrValBool, AttrValFloat, AttrValInt, AttrValString, RichTextRun, XlsxExtLst,
};
use super::drawing::{ABodyPr, ALn, AP, ASchemeClr, ASolidFill, Fill, GraphicOptions, LineOptions};
use crate::styles::{Alignment, Font};

// ------------------------------------------------------------------
// Root chart-space element
// ------------------------------------------------------------------

/// Directly maps the chartSpace element. The chart namespace in DrawingML is
/// for representing visualizations of numeric data with column charts, pie
/// charts, scatter charts, or other types of charts.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "c:chartSpace")]
pub struct XlsxChartSpace {
    #[serde(rename = "@xmlns:c", default)]
    pub xmlns_c: Option<String>,
    #[serde(rename = "@xmlns:a", default)]
    pub xmlns_a: Option<String>,
    #[serde(rename = "c:date1904", default)]
    pub date_1904: Option<AttrValBool>,
    #[serde(rename = "c:lang", default)]
    pub lang: Option<AttrValString>,
    #[serde(rename = "c:roundedCorners", default)]
    pub rounded_corners: Option<AttrValBool>,
    #[serde(rename = "c:chart", default)]
    pub chart: CChart,
    #[serde(rename = "c:spPr", default)]
    pub sp_pr: Option<CSpPr>,
    #[serde(rename = "c:txPr", default)]
    pub tx_pr: Option<CTxPr>,
    #[serde(rename = "c:printSettings", default)]
    pub print_settings: Option<CPrintSettings>,
}

// ------------------------------------------------------------------
// Chart container
// ------------------------------------------------------------------

/// Directly maps the element that specifies the thickness of the walls or
/// floor as a percentage of the largest dimension of the plot volume and
/// SpPr element.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct CThicknessSpPr {
    #[serde(rename = "c:thickness", default)]
    pub thickness: Option<AttrValInt>,
    #[serde(rename = "c:spPr", default)]
    pub sp_pr: Option<CSpPr>,
}

/// Directly maps the chart element. This element specifies a title.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct CChart {
    #[serde(rename = "c:title", default)]
    pub title: Option<CTitle>,
    #[serde(rename = "c:autoTitleDeleted", default)]
    pub auto_title_deleted: Option<CAutoTitleDeleted>,
    #[serde(rename = "c:view3D", default)]
    pub view_3d: Option<CView3D>,
    #[serde(rename = "c:floor", default)]
    pub floor: Option<CThicknessSpPr>,
    #[serde(rename = "c:sideWall", default)]
    pub side_wall: Option<CThicknessSpPr>,
    #[serde(rename = "c:backWall", default)]
    pub back_wall: Option<CThicknessSpPr>,
    #[serde(rename = "c:plotArea", default)]
    pub plot_area: Option<CPlotArea>,
    #[serde(rename = "c:legend", default)]
    pub legend: Option<CLegend>,
    #[serde(rename = "c:plotVisOnly", default)]
    pub plot_vis_only: Option<AttrValBool>,
    #[serde(rename = "c:dispBlanksAs", default)]
    pub disp_blanks_as: Option<AttrValString>,
    #[serde(rename = "c:showDLblsOverMax", default)]
    pub show_d_lbls_over_max: Option<AttrValBool>,
}

/// Directly maps the title element. This element specifies a title.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct CTitle {
    #[serde(rename = "c:tx", default)]
    pub tx: Option<CTx>,
    #[serde(rename = "c:layout", default)]
    pub layout: Option<CLayout>,
    #[serde(rename = "c:overlay", default)]
    pub overlay: Option<AttrValBool>,
    #[serde(rename = "c:spPr", default)]
    pub sp_pr: Option<CSpPr>,
    #[serde(rename = "c:txPr", default)]
    pub tx_pr: Option<CTxPr>,
    #[serde(rename = "c:extLst", default)]
    pub ext_lst: Option<XlsxExtLst>,
}

/// Directly maps the layout element. This element specifies how the chart
/// element is placed on the chart.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct CLayout {
    #[serde(rename = "c:manualLayout", default)]
    pub manual_layout: Option<CManualLayout>,
    #[serde(rename = "c:extLst", default)]
    pub ext_lst: Option<XlsxExtLst>,
}

/// Directly maps the manualLayout element. This element specifies the exact
/// position of a chart element.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct CManualLayout {
    #[serde(rename = "c:layoutTarget", default)]
    pub layout_target: Option<AttrValString>,
    #[serde(rename = "c:xMode", default)]
    pub x_mode: Option<AttrValString>,
    #[serde(rename = "c:yMode", default)]
    pub y_mode: Option<AttrValString>,
    #[serde(rename = "c:wMode", default)]
    pub w_mode: Option<AttrValString>,
    #[serde(rename = "c:hMode", default)]
    pub h_mode: Option<AttrValString>,
    #[serde(rename = "c:x", default)]
    pub x: Option<AttrValFloat>,
    #[serde(rename = "c:y", default)]
    pub y: Option<AttrValFloat>,
    #[serde(rename = "c:w", default)]
    pub w: Option<AttrValFloat>,
    #[serde(rename = "c:h", default)]
    pub h: Option<AttrValFloat>,
    #[serde(rename = "c:extLst", default)]
    pub ext_lst: Option<XlsxExtLst>,
}

/// Directly maps the tx element. This element specifies text to use on a
/// chart, including rich text formatting.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct CTx {
    #[serde(rename = "c:strRef", default)]
    pub str_ref: Option<CStrRef>,
    #[serde(rename = "c:rich", default)]
    pub rich: Option<CRich>,
}

/// Directly maps the rich element. This element contains a string with rich
/// text formatting.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct CRich {
    #[serde(rename = "a:bodyPr", default)]
    pub body_pr: Option<ABodyPr>,
    #[serde(rename = "a:lstStyle", default)]
    pub lst_style: Option<String>,
    #[serde(rename = "a:p", default)]
    pub p: Vec<AP>,
}

// ------------------------------------------------------------------
// Shape / text properties (chart variants)
// ------------------------------------------------------------------

/// Directly maps the dTable element.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct CDTable {
    #[serde(rename = "c:showHorzBorder", default)]
    pub show_horz_border: Option<AttrValBool>,
    #[serde(rename = "c:showVertBorder", default)]
    pub show_vert_border: Option<AttrValBool>,
    #[serde(rename = "c:showOutline", default)]
    pub show_outline: Option<AttrValBool>,
    #[serde(rename = "c:showKeys", default)]
    pub show_keys: Option<AttrValBool>,
    #[serde(rename = "c:spPr", default)]
    pub sp_pr: Option<CSpPr>,
    #[serde(rename = "c:txPr", default)]
    pub tx_pr: Option<CTxPr>,
    #[serde(rename = "c:extLst", default)]
    pub ext_lst: Option<XlsxExtLst>,
}

/// Directly maps the spPr element. This element specifies the visual shape
/// properties that can be applied to a shape. These properties include the
/// shape fill, outline, geometry, effects, and 3D orientation.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct CSpPr {
    #[serde(rename = "a:noFill", default)]
    pub no_fill: Option<String>,
    #[serde(rename = "a:solidFill", default)]
    pub solid_fill: Option<ASolidFill>,
    #[serde(rename = "a:ln", default)]
    pub ln: Option<ALn>,
    #[serde(rename = "a:sp3d", default)]
    pub sp_3d: Option<ASp3D>,
    #[serde(rename = "a:effectLst", default)]
    pub effect_lst: Option<String>,
}

/// Directly maps the a:sp3d element. This element defines the 3D properties
/// associated with a particular shape in DrawingML.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ASp3D {
    #[serde(rename = "@contourW", default)]
    pub contour_w: i64,
    #[serde(rename = "a:contourClr", default)]
    pub contour_clr: Option<AContourClr>,
}

/// Directly maps the a:contourClr element. This element defines the color
/// for the contour on a shape.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AContourClr {
    #[serde(rename = "a:schemeClr", default)]
    pub scheme_clr: Option<ASchemeClr>,
}

/// Directly maps the txPr element. This element specifies text formatting.
/// The lstStyle element is not supported.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct CTxPr {
    #[serde(rename = "a:bodyPr", default)]
    pub body_pr: Option<ABodyPr>,
    #[serde(rename = "a:lstStyle", default)]
    pub lst_style: Option<String>,
    #[serde(rename = "a:p", default)]
    pub p: Option<AP>,
}

// ------------------------------------------------------------------
// View / plot area
// ------------------------------------------------------------------

/// Directly maps the autoTitleDeleted element. This element specifies the
/// title shall not be shown for this chart.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CAutoTitleDeleted {
    #[serde(rename = "@val", default)]
    pub val: bool,
}

/// Directly maps the view3D element. This element specifies the 3-D view of
/// the chart.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CView3D {
    #[serde(rename = "c:rotX", default)]
    pub rot_x: Option<AttrValInt>,
    #[serde(rename = "c:rotY", default)]
    pub rot_y: Option<AttrValInt>,
    #[serde(rename = "c:rAngAx", default)]
    pub r_ang_ax: Option<AttrValInt>,
    #[serde(rename = "c:depthPercent", default)]
    pub depth_percent: Option<AttrValInt>,
    #[serde(rename = "c:perspective", default)]
    pub perspective: Option<AttrValInt>,
    #[serde(rename = "c:extLst", default)]
    pub ext_lst: Option<XlsxExtLst>,
}

/// Directly maps the plotArea element. This element specifies the plot area
/// of the chart.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct CPlotArea {
    #[serde(rename = "c:layout", default)]
    pub layout: Option<String>,
    #[serde(rename = "c:areaChart", default)]
    pub area_chart: Vec<CCharts>,
    #[serde(rename = "c:area3DChart", default)]
    pub area_3d_chart: Vec<CCharts>,
    #[serde(rename = "c:barChart", default)]
    pub bar_chart: Vec<CCharts>,
    #[serde(rename = "c:bar3DChart", default)]
    pub bar_3d_chart: Vec<CCharts>,
    #[serde(rename = "c:bubbleChart", default)]
    pub bubble_chart: Vec<CCharts>,
    #[serde(rename = "c:doughnutChart", default)]
    pub doughnut_chart: Vec<CCharts>,
    #[serde(rename = "c:lineChart", default)]
    pub line_chart: Vec<CCharts>,
    #[serde(rename = "c:line3DChart", default)]
    pub line_3d_chart: Vec<CCharts>,
    #[serde(rename = "c:stockChart", default)]
    pub stock_chart: Vec<CCharts>,
    #[serde(rename = "c:pieChart", default)]
    pub pie_chart: Vec<CCharts>,
    #[serde(rename = "c:pie3DChart", default)]
    pub pie_3d_chart: Vec<CCharts>,
    #[serde(rename = "c:ofPieChart", default)]
    pub of_pie_chart: Vec<CCharts>,
    #[serde(rename = "c:radarChart", default)]
    pub radar_chart: Vec<CCharts>,
    #[serde(rename = "c:scatterChart", default)]
    pub scatter_chart: Vec<CCharts>,
    #[serde(rename = "c:surface3DChart", default)]
    pub surface_3d_chart: Vec<CCharts>,
    #[serde(rename = "c:surfaceChart", default)]
    pub surface_chart: Vec<CCharts>,
    #[serde(rename = "c:catAx", default)]
    pub cat_ax: Vec<CAxs>,
    #[serde(rename = "c:valAx", default)]
    pub val_ax: Vec<CAxs>,
    #[serde(rename = "c:dateAx", default)]
    pub date_ax: Vec<CAxs>,
    #[serde(rename = "c:serAx", default)]
    pub ser_ax: Vec<CAxs>,
    #[serde(rename = "c:dTable", default)]
    pub d_table: Option<CDTable>,
    #[serde(rename = "c:spPr", default)]
    pub sp_pr: Option<CSpPr>,
}

/// Specifies the common element of the chart.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct CCharts {
    #[serde(rename = "c:barDir", default)]
    pub bar_dir: Option<AttrValString>,
    #[serde(rename = "c:bubbleScale", default)]
    pub bubble_scale: Option<AttrValFloat>,
    #[serde(rename = "c:grouping", default)]
    pub grouping: Option<AttrValString>,
    #[serde(rename = "c:radarStyle", default)]
    pub radar_style: Option<AttrValString>,
    #[serde(rename = "c:scatterStyle", default)]
    pub scatter_style: Option<AttrValString>,
    #[serde(rename = "c:ofPieType", default)]
    pub of_pie_type: Option<AttrValString>,
    #[serde(rename = "c:varyColors", default)]
    pub vary_colors: Option<AttrValBool>,
    #[serde(rename = "c:wireframe", default)]
    pub wireframe: Option<AttrValBool>,
    #[serde(rename = "c:ser", default)]
    pub ser: Option<Vec<CSer>>,
    #[serde(rename = "c:splitPos", default)]
    pub split_pos: Option<AttrValInt>,
    #[serde(rename = "c:serLines", default)]
    pub ser_lines: Option<AttrValString>,
    #[serde(rename = "c:dLbls", default)]
    pub d_lbls: Option<CDLbls>,
    #[serde(rename = "c:dropLines", default)]
    pub drop_lines: Option<CLines>,
    #[serde(rename = "c:hiLowLines", default)]
    pub hi_low_lines: Option<CLines>,
    #[serde(rename = "c:upDownBars", default)]
    pub up_down_bars: Option<CUpDownBars>,
    #[serde(rename = "c:gapWidth", default)]
    pub gap_width: Option<AttrValInt>,
    #[serde(rename = "c:shape", default)]
    pub shape: Option<AttrValString>,
    #[serde(rename = "c:holeSize", default)]
    pub hole_size: Option<AttrValInt>,
    #[serde(rename = "c:smooth", default)]
    pub smooth: Option<AttrValBool>,
    #[serde(rename = "c:overlap", default)]
    pub overlap: Option<AttrValInt>,
    #[serde(rename = "c:axId", default)]
    pub ax_id: Vec<AttrValInt>,
}

/// Directly maps the catAx and valAx element.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct CAxs {
    #[serde(rename = "c:axId", default)]
    pub ax_id: Option<AttrValInt>,
    #[serde(rename = "c:scaling", default)]
    pub scaling: Option<CScaling>,
    #[serde(rename = "c:delete", default)]
    pub delete: Option<AttrValBool>,
    #[serde(rename = "c:axPos", default)]
    pub ax_pos: Option<AttrValString>,
    #[serde(rename = "c:majorGridlines", default)]
    pub major_gridlines: Option<CLines>,
    #[serde(rename = "c:minorGridlines", default)]
    pub minor_gridlines: Option<CLines>,
    #[serde(rename = "c:title", default)]
    pub title: Option<CTitle>,
    #[serde(rename = "c:numFmt", default)]
    pub num_fmt: Option<CNumFmt>,
    #[serde(rename = "c:majorTickMark", default)]
    pub major_tick_mark: Option<AttrValString>,
    #[serde(rename = "c:minorTickMark", default)]
    pub minor_tick_mark: Option<AttrValString>,
    #[serde(rename = "c:tickLblPos", default)]
    pub tick_lbl_pos: Option<AttrValString>,
    #[serde(rename = "c:spPr", default)]
    pub sp_pr: Option<CSpPr>,
    #[serde(rename = "c:txPr", default)]
    pub tx_pr: Option<CTxPr>,
    #[serde(rename = "c:crossAx", default)]
    pub cross_ax: Option<AttrValInt>,
    #[serde(rename = "c:crosses", default)]
    pub crosses: Option<AttrValString>,
    #[serde(rename = "c:crossBetween", default)]
    pub cross_between: Option<AttrValString>,
    #[serde(rename = "c:majorUnit", default)]
    pub major_unit: Option<AttrValFloat>,
    #[serde(rename = "c:minorUnit", default)]
    pub minor_unit: Option<AttrValFloat>,
    #[serde(rename = "c:auto", default)]
    pub auto: Option<AttrValBool>,
    #[serde(rename = "c:lblAlgn", default)]
    pub lbl_algn: Option<AttrValString>,
    #[serde(rename = "c:lblOffset", default)]
    pub lbl_offset: Option<AttrValInt>,
    #[serde(rename = "c:tickLblSkip", default)]
    pub tick_lbl_skip: Option<AttrValInt>,
    #[serde(rename = "c:tickMarkSkip", default)]
    pub tick_mark_skip: Option<AttrValInt>,
    #[serde(rename = "c:noMultiLvlLbl", default)]
    pub no_multi_lvl_lbl: Option<AttrValBool>,
}

/// Directly maps the upDownBars element. This element specifies the up and
/// down bars.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct CUpDownBars {
    #[serde(rename = "c:gapWidth", default)]
    pub gap_width: Option<AttrValString>,
    #[serde(rename = "c:upBars", default)]
    pub up_bars: Option<CLines>,
    #[serde(rename = "c:downBars", default)]
    pub down_bars: Option<CLines>,
    #[serde(rename = "c:extLst", default)]
    pub ext_lst: Option<XlsxExtLst>,
}

/// Directly maps the chart lines content model.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct CLines {
    #[serde(rename = "c:spPr", default)]
    pub sp_pr: Option<CSpPr>,
}

/// Directly maps the scaling element. This element contains additional axis
/// settings.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct CScaling {
    #[serde(rename = "c:logBase", default)]
    pub log_base: Option<AttrValFloat>,
    #[serde(rename = "c:orientation", default)]
    pub orientation: Option<AttrValString>,
    #[serde(rename = "c:max", default)]
    pub max: Option<AttrValFloat>,
    #[serde(rename = "c:min", default)]
    pub min: Option<AttrValFloat>,
}

/// Directly maps the numFmt element. This element specifies number formatting
/// for the parent element.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CNumFmt {
    #[serde(rename = "@formatCode", default)]
    pub format_code: String,
    #[serde(rename = "@sourceLinked", default)]
    pub source_linked: bool,
}

// ------------------------------------------------------------------
// Series / data
// ------------------------------------------------------------------

/// Directly maps the ser element. This element specifies a series on a chart.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct CSer {
    #[serde(rename = "c:idx", default)]
    pub idx: Option<AttrValInt>,
    #[serde(rename = "c:order", default)]
    pub order: Option<AttrValInt>,
    #[serde(rename = "c:tx", default)]
    pub tx: Option<CTx>,
    #[serde(rename = "c:spPr", default)]
    pub sp_pr: Option<CSpPr>,
    #[serde(rename = "c:dPt", default)]
    pub d_pt: Vec<CDPt>,
    #[serde(rename = "c:dLbls", default)]
    pub d_lbls: Option<CDLbls>,
    #[serde(rename = "c:marker", default)]
    pub marker: Option<CMarker>,
    #[serde(rename = "c:invertIfNegative", default)]
    pub invert_if_negative: Option<AttrValBool>,
    #[serde(rename = "c:cat", default)]
    pub cat: Option<CCat>,
    #[serde(rename = "c:val", default)]
    pub val: Option<CVal>,
    #[serde(rename = "c:xVal", default)]
    pub x_val: Option<CCat>,
    #[serde(rename = "c:yVal", default)]
    pub y_val: Option<CVal>,
    #[serde(rename = "c:smooth", default)]
    pub smooth: Option<AttrValBool>,
    #[serde(rename = "c:bubbleSize", default)]
    pub bubble_size: Option<CVal>,
    #[serde(rename = "c:bubble3D", default)]
    pub bubble_3d: Option<AttrValBool>,
}

/// Directly maps the marker element. This element specifies a data marker.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct CMarker {
    #[serde(rename = "c:symbol", default)]
    pub symbol: Option<AttrValString>,
    #[serde(rename = "c:size", default)]
    pub size: Option<AttrValInt>,
    #[serde(rename = "c:spPr", default)]
    pub sp_pr: Option<CSpPr>,
}

/// Directly maps the dPt element. This element specifies a single data point.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct CDPt {
    #[serde(rename = "c:idx", default)]
    pub idx: Option<AttrValInt>,
    #[serde(rename = "c:bubble3D", default)]
    pub bubble_3d: Option<AttrValBool>,
    #[serde(rename = "c:spPr", default)]
    pub sp_pr: Option<CSpPr>,
}

/// Directly maps the cat element. This element specifies the data used for
/// the category axis.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct CCat {
    #[serde(rename = "c:strRef", default)]
    pub str_ref: Option<CStrRef>,
}

/// Directly maps the strRef element. This element specifies a reference to
/// data for a single data label or title with a cache of the last values used.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CStrRef {
    #[serde(rename = "c:f", default)]
    pub f: String,
    #[serde(rename = "c:strCache", default)]
    pub str_cache: Option<CStrCache>,
}

/// Directly maps the strCache element. This element specifies the last string
/// data used for a chart.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CStrCache {
    #[serde(rename = "c:pt", default)]
    pub pt: Vec<CPt>,
    #[serde(rename = "c:ptCount", default)]
    pub pt_count: Option<AttrValInt>,
}

/// Directly maps the pt element. This element specifies data for a particular
/// data point.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CPt {
    #[serde(rename = "@idx", default)]
    pub idx: i64,
    #[serde(rename = "c:v", default)]
    pub v: Option<String>,
}

/// Directly maps the val element. This element specifies the data values
/// which shall be used to define the location of data markers on a chart.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct CVal {
    #[serde(rename = "c:numRef", default)]
    pub num_ref: Option<CNumRef>,
}

/// Directly maps the numRef element. This element specifies a reference to
/// numeric data with a cache of the last values used.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CNumRef {
    #[serde(rename = "c:f", default)]
    pub f: String,
    #[serde(rename = "c:numCache", default)]
    pub num_cache: Option<CNumCache>,
}

/// Directly maps the numCache element. This element specifies the last data
/// shown on the chart for a series.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CNumCache {
    #[serde(rename = "c:formatCode", default)]
    pub format_code: String,
    #[serde(rename = "c:pt", default)]
    pub pt: Vec<CPt>,
    #[serde(rename = "c:ptCount", default)]
    pub pt_count: Option<AttrValInt>,
}

/// Directly maps the dLbls element. This element serves as a root element
/// that specifies the settings for the data labels for an entire series or
/// the entire chart.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct CDLbls {
    #[serde(rename = "c:numFmt", default)]
    pub num_fmt: Option<CNumFmt>,
    #[serde(rename = "c:spPr", default)]
    pub sp_pr: Option<CSpPr>,
    #[serde(rename = "c:txPr", default)]
    pub tx_pr: Option<CTxPr>,
    #[serde(rename = "c:dLblPos", default)]
    pub d_lbl_pos: Option<AttrValString>,
    #[serde(rename = "c:showLegendKey", default)]
    pub show_legend_key: Option<AttrValBool>,
    #[serde(rename = "c:showVal", default)]
    pub show_val: Option<AttrValBool>,
    #[serde(rename = "c:showCatName", default)]
    pub show_cat_name: Option<AttrValBool>,
    #[serde(rename = "c:showSerName", default)]
    pub show_ser_name: Option<AttrValBool>,
    #[serde(rename = "c:showPercent", default)]
    pub show_percent: Option<AttrValBool>,
    #[serde(rename = "c:showBubbleSize", default)]
    pub show_bubble_size: Option<AttrValBool>,
    #[serde(rename = "c:showLeaderLines", default)]
    pub show_leader_lines: Option<AttrValBool>,
}

// ------------------------------------------------------------------
// Legend / print settings
// ------------------------------------------------------------------

/// Directly maps the legendEntry element. This element specifies the legend
/// entry.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct CLegendEntry {
    #[serde(rename = "c:idx", default)]
    pub idx: Option<AttrValInt>,
    #[serde(rename = "c:txPr", default)]
    pub tx_pr: Option<CTxPr>,
}

/// Directly maps the legend element. This element specifies the legend.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct CLegend {
    #[serde(rename = "c:layout", default)]
    pub layout: Option<String>,
    #[serde(rename = "c:legendPos", default)]
    pub legend_pos: Option<AttrValString>,
    #[serde(rename = "c:legendEntry", default)]
    pub legend_entry: Vec<CLegendEntry>,
    #[serde(rename = "c:overlay", default)]
    pub overlay: Option<AttrValBool>,
    #[serde(rename = "c:spPr", default)]
    pub sp_pr: Option<CSpPr>,
    #[serde(rename = "c:txPr", default)]
    pub tx_pr: Option<CTxPr>,
}

/// Directly maps the printSettings element. This element specifies the print
/// settings for the chart.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct CPrintSettings {
    #[serde(rename = "c:headerFooter", default)]
    pub header_footer: Option<String>,
    #[serde(rename = "c:pageMargins", default)]
    pub page_margins: Option<CPageMargins>,
    #[serde(rename = "c:pageSetup", default)]
    pub page_setup: Option<String>,
}

/// Directly maps the pageMargins element. This element specifies the page
/// margins for a chart.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct CPageMargins {
    #[serde(rename = "@b", default)]
    pub b: f64,
    #[serde(rename = "@footer", default)]
    pub footer: f64,
    #[serde(rename = "@header", default)]
    pub header: f64,
    #[serde(rename = "@l", default)]
    pub l: f64,
    #[serde(rename = "@r", default)]
    pub r: f64,
    #[serde(rename = "@t", default)]
    pub t: f64,
}

// ------------------------------------------------------------------
// Public API types
// ------------------------------------------------------------------

/// Type of supported chart types.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChartType(pub u8);

impl ChartType {
    pub const AREA: ChartType = ChartType(0);
    pub const AREA_STACKED: ChartType = ChartType(1);
    pub const AREA_PERCENT_STACKED: ChartType = ChartType(2);
    pub const AREA_3D: ChartType = ChartType(3);
    pub const AREA_3D_STACKED: ChartType = ChartType(4);
    pub const AREA_3D_PERCENT_STACKED: ChartType = ChartType(5);
    pub const BAR: ChartType = ChartType(6);
    pub const BAR_STACKED: ChartType = ChartType(7);
    pub const BAR_PERCENT_STACKED: ChartType = ChartType(8);
    pub const BAR_3D_CLUSTERED: ChartType = ChartType(9);
    pub const BAR_3D_STACKED: ChartType = ChartType(10);
    pub const BAR_3D_PERCENT_STACKED: ChartType = ChartType(11);
    pub const BAR_3D_CONE_CLUSTERED: ChartType = ChartType(12);
    pub const BAR_3D_CONE_STACKED: ChartType = ChartType(13);
    pub const BAR_3D_CONE_PERCENT_STACKED: ChartType = ChartType(14);
    pub const BAR_3D_PYRAMID_CLUSTERED: ChartType = ChartType(15);
    pub const BAR_3D_PYRAMID_STACKED: ChartType = ChartType(16);
    pub const BAR_3D_PYRAMID_PERCENT_STACKED: ChartType = ChartType(17);
    pub const BAR_3D_CYLINDER_CLUSTERED: ChartType = ChartType(18);
    pub const BAR_3D_CYLINDER_STACKED: ChartType = ChartType(19);
    pub const BAR_3D_CYLINDER_PERCENT_STACKED: ChartType = ChartType(20);
    pub const COL: ChartType = ChartType(21);
    pub const COL_STACKED: ChartType = ChartType(22);
    pub const COL_PERCENT_STACKED: ChartType = ChartType(23);
    pub const COL_3D: ChartType = ChartType(24);
    pub const COL_3D_CLUSTERED: ChartType = ChartType(25);
    pub const COL_3D_STACKED: ChartType = ChartType(26);
    pub const COL_3D_PERCENT_STACKED: ChartType = ChartType(27);
    pub const COL_3D_CONE: ChartType = ChartType(28);
    pub const COL_3D_CONE_CLUSTERED: ChartType = ChartType(29);
    pub const COL_3D_CONE_STACKED: ChartType = ChartType(30);
    pub const COL_3D_CONE_PERCENT_STACKED: ChartType = ChartType(31);
    pub const COL_3D_PYRAMID: ChartType = ChartType(32);
    pub const COL_3D_PYRAMID_CLUSTERED: ChartType = ChartType(33);
    pub const COL_3D_PYRAMID_STACKED: ChartType = ChartType(34);
    pub const COL_3D_PYRAMID_PERCENT_STACKED: ChartType = ChartType(35);
    pub const COL_3D_CYLINDER: ChartType = ChartType(36);
    pub const COL_3D_CYLINDER_CLUSTERED: ChartType = ChartType(37);
    pub const COL_3D_CYLINDER_STACKED: ChartType = ChartType(38);
    pub const COL_3D_CYLINDER_PERCENT_STACKED: ChartType = ChartType(39);
    pub const DOUGHNUT: ChartType = ChartType(40);
    pub const LINE: ChartType = ChartType(41);
    pub const LINE_3D: ChartType = ChartType(42);
    pub const PIE: ChartType = ChartType(43);
    pub const PIE_3D: ChartType = ChartType(44);
    pub const PIE_OF_PIE: ChartType = ChartType(45);
    pub const BAR_OF_PIE: ChartType = ChartType(46);
    pub const RADAR: ChartType = ChartType(47);
    pub const SCATTER: ChartType = ChartType(48);
    pub const SURFACE_3D: ChartType = ChartType(49);
    pub const WIREFRAME_SURFACE_3D: ChartType = ChartType(50);
    pub const CONTOUR: ChartType = ChartType(51);
    pub const WIREFRAME_CONTOUR: ChartType = ChartType(52);
    pub const BUBBLE: ChartType = ChartType(53);
    pub const BUBBLE_3D: ChartType = ChartType(54);
    pub const STOCK_HIGH_LOW_CLOSE: ChartType = ChartType(55);
    pub const STOCK_OPEN_HIGH_LOW_CLOSE: ChartType = ChartType(56);
}

/// Type of supported chart tick label position types.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChartTickLabelPositionType(pub u8);

impl ChartTickLabelPositionType {
    pub const NEXT_TO_AXIS: ChartTickLabelPositionType = ChartTickLabelPositionType(0);
    pub const HIGH: ChartTickLabelPositionType = ChartTickLabelPositionType(1);
    pub const LOW: ChartTickLabelPositionType = ChartTickLabelPositionType(2);
    pub const NONE: ChartTickLabelPositionType = ChartTickLabelPositionType(3);
}

/// Type of chart data labels position.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChartDataLabelPositionType(pub u8);

impl ChartDataLabelPositionType {
    pub const UNSET: ChartDataLabelPositionType = ChartDataLabelPositionType(0);
    pub const BEST_FIT: ChartDataLabelPositionType = ChartDataLabelPositionType(1);
    pub const BELOW: ChartDataLabelPositionType = ChartDataLabelPositionType(2);
    pub const CENTER: ChartDataLabelPositionType = ChartDataLabelPositionType(3);
    pub const INSIDE_BASE: ChartDataLabelPositionType = ChartDataLabelPositionType(4);
    pub const INSIDE_END: ChartDataLabelPositionType = ChartDataLabelPositionType(5);
    pub const LEFT: ChartDataLabelPositionType = ChartDataLabelPositionType(6);
    pub const OUTSIDE_END: ChartDataLabelPositionType = ChartDataLabelPositionType(7);
    pub const RIGHT: ChartDataLabelPositionType = ChartDataLabelPositionType(8);
    pub const ABOVE: ChartDataLabelPositionType = ChartDataLabelPositionType(9);
}

/// Directly maps the number format settings of the chart.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChartNumFmt {
    #[serde(default)]
    pub custom_num_fmt: String,
    #[serde(default)]
    pub source_linked: bool,
}

/// Directly maps the format settings of the chart axis.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChartAxis {
    #[serde(default)]
    pub none: bool,
    #[serde(default)]
    pub drop_lines: bool,
    #[serde(default)]
    pub high_low_lines: bool,
    #[serde(default)]
    pub major_grid_lines: bool,
    #[serde(default)]
    pub minor_grid_lines: bool,
    #[serde(default)]
    pub major_unit: f64,
    #[serde(default)]
    pub tick_label_position: ChartTickLabelPositionType,
    #[serde(default)]
    pub tick_label_skip: i64,
    #[serde(default)]
    pub reverse_order: bool,
    #[serde(default)]
    pub secondary: bool,
    #[serde(default)]
    pub maximum: Option<f64>,
    #[serde(default)]
    pub minimum: Option<f64>,
    #[serde(default)]
    pub alignment: Alignment,
    #[serde(default)]
    pub font: Font,
    #[serde(default)]
    pub log_base: f64,
    #[serde(default)]
    pub num_fmt: ChartNumFmt,
    #[serde(default)]
    pub title: ChartTitle,
    #[serde(rename = "axID", default)]
    pub ax_id: i64,
}

/// Directly maps the dimension of the chart.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChartDimension {
    pub width: u64,
    pub height: u64,
}

/// Directly maps the format settings of the stock chart up bars and down
/// bars.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChartUpDownBar {
    pub fill: Fill,
    pub border: LineOptions,
}

/// Directly maps the format settings of the plot area.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChartPlotArea {
    #[serde(rename = "secondPlotValues", default)]
    pub second_plot_values: i64,
    #[serde(default)]
    pub show_bubble_size: bool,
    #[serde(default)]
    pub show_cat_name: bool,
    #[serde(default)]
    pub show_data_table: bool,
    #[serde(default)]
    pub show_data_table_keys: bool,
    #[serde(default)]
    pub show_leader_lines: bool,
    #[serde(default)]
    pub show_percent: bool,
    #[serde(default)]
    pub show_ser_name: bool,
    #[serde(default)]
    pub show_val: bool,
    pub fill: Fill,
    pub up_bars: ChartUpDownBar,
    pub down_bars: ChartUpDownBar,
    pub num_fmt: ChartNumFmt,
}

/// Directly maps the format settings of the chart.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct Chart {
    pub r#type: ChartType,
    #[serde(default)]
    pub series: Vec<ChartSeries>,
    pub format: GraphicOptions,
    pub dimension: ChartDimension,
    pub legend: ChartLegend,
    pub title: ChartTitle,
    #[serde(default)]
    pub vary_colors: Option<bool>,
    pub x_axis: ChartAxis,
    pub y_axis: ChartAxis,
    pub plot_area: ChartPlotArea,
    pub fill: Fill,
    pub border: LineOptions,
    #[serde(default)]
    pub show_blanks_as: String,
    #[serde(default)]
    pub bubble_size: i64,
    #[serde(default)]
    pub hole_size: i64,
    #[serde(default)]
    pub gap_width: Option<u64>,
    #[serde(default)]
    pub overlap: Option<i64>,
    #[serde(rename = "order", default)]
    pub order: i64,
    #[serde(default, skip_serializing)]
    pub combo: Vec<Chart>,
}

/// Directly maps the format settings of the chart title.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChartTitle {
    pub fill: Fill,
    pub border: LineOptions,
    #[serde(default)]
    pub paragraph: Vec<RichTextRun>,
    #[serde(default)]
    pub font: Option<Font>,
    #[serde(default)]
    pub formula: String,
    #[serde(default)]
    pub offset_x: i64,
    #[serde(default)]
    pub offset_y: i64,
    #[serde(default)]
    pub width: i64,
    #[serde(default)]
    pub height: i64,
    #[serde(default)]
    pub overlay: bool,
}

/// Directly maps the format settings of the chart legend.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChartLegend {
    #[serde(default)]
    pub position: String,
    #[serde(default)]
    pub show_legend_key: bool,
    #[serde(default)]
    pub font: Option<Font>,
}

/// Directly maps the format settings of the chart marker.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChartMarker {
    pub border: LineOptions,
    pub fill: Fill,
    #[serde(default)]
    pub symbol: String,
    #[serde(default)]
    pub size: i64,
}

/// Directly maps the format settings of the chart labels.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChartDataLabel {
    pub alignment: Alignment,
    pub font: Font,
    pub fill: Fill,
}

/// Directly maps the format settings of the chart data point for doughnut,
/// pie and 3D pie charts.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChartDataPoint {
    pub index: i64,
    pub fill: Fill,
}

/// Directly maps the format settings of the chart series.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChartSeries {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub categories: String,
    #[serde(default)]
    pub values: String,
    #[serde(default)]
    pub sizes: String,
    pub fill: Fill,
    pub legend: ChartLegend,
    pub line: LineOptions,
    pub marker: ChartMarker,
    pub data_label: ChartDataLabel,
    pub data_label_position: ChartDataLabelPositionType,
    #[serde(default)]
    pub data_point: Vec<ChartDataPoint>,
}
