//! Decode-only chart types.
//!
//! The serialize-oriented types in `crate::xml::chart` use namespace-prefixed
//! element names (`c:barChart`, `a:solidFill`, etc.). `quick_xml`'s serde
//! deserializer strips namespace prefixes from child elements, so these
//! decode-only mirrors use unprefixed names for reading chart XML back.

use serde::de::{Deserializer, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Serialize, Serializer};
use std::fmt;

use super::common::{AttrValBool, AttrValFloat, AttrValInt, AttrValString};

/// Consumes any XML element/attribute content during deserialization.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct DecodeIgnore;

impl<'de> serde::de::Deserialize<'de> for DecodeIgnore {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct IgnoreVisitor;

        impl<'de> Visitor<'de> for IgnoreVisitor {
            type Value = DecodeIgnore;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("any XML content")
            }

            fn visit_bool<E>(self, _: bool) -> Result<Self::Value, E> {
                Ok(DecodeIgnore)
            }

            fn visit_i64<E>(self, _: i64) -> Result<Self::Value, E> {
                Ok(DecodeIgnore)
            }

            fn visit_u64<E>(self, _: u64) -> Result<Self::Value, E> {
                Ok(DecodeIgnore)
            }

            fn visit_f64<E>(self, _: f64) -> Result<Self::Value, E> {
                Ok(DecodeIgnore)
            }

            fn visit_str<E>(self, _: &str) -> Result<Self::Value, E> {
                Ok(DecodeIgnore)
            }

            fn visit_string<E>(self, _: String) -> Result<Self::Value, E> {
                Ok(DecodeIgnore)
            }

            fn visit_unit<E>(self) -> Result<Self::Value, E> {
                Ok(DecodeIgnore)
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                while seq.next_element::<DecodeIgnore>()?.is_some() {}
                Ok(DecodeIgnore)
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                while map.next_entry::<DecodeIgnore, DecodeIgnore>()?.is_some() {}
                Ok(DecodeIgnore)
            }

            fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: Deserializer<'de>,
            {
                DecodeIgnore::deserialize(deserializer)
            }
        }

        deserializer.deserialize_any(IgnoreVisitor)
    }
}

impl Serialize for DecodeIgnore {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_unit_struct("DecodeIgnore")
    }
}

/// Mirrors `XlsxChartSpace` for deserialization.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "c:chartSpace")]
pub struct DecodeChartSpace {
    #[serde(rename = "chart", default)]
    pub chart: DecodeCChart,
    #[serde(rename = "spPr", default)]
    pub sp_pr: Option<DecodeCSpPr>,
}

/// Mirrors `CChart` for deserialization.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecodeCChart {
    #[serde(rename = "title", default)]
    pub title: Option<DecodeCTitle>,
    #[serde(rename = "plotArea", default)]
    pub plot_area: Option<DecodeCPlotArea>,
    #[serde(rename = "legend", default)]
    pub legend: Option<DecodeCLegend>,
    #[serde(rename = "dispBlanksAs", default)]
    pub disp_blanks_as: Option<AttrValString>,
}

/// Mirrors `CPlotArea` for deserialization.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecodeCPlotArea {
    #[serde(rename = "areaChart", default)]
    pub area_chart: Vec<DecodeCCharts>,
    #[serde(rename = "area3DChart", default)]
    pub area_3d_chart: Vec<DecodeCCharts>,
    #[serde(rename = "barChart", default)]
    pub bar_chart: Vec<DecodeCCharts>,
    #[serde(rename = "bar3DChart", default)]
    pub bar_3d_chart: Vec<DecodeCCharts>,
    #[serde(rename = "bubbleChart", default)]
    pub bubble_chart: Vec<DecodeCCharts>,
    #[serde(rename = "doughnutChart", default)]
    pub doughnut_chart: Vec<DecodeCCharts>,
    #[serde(rename = "lineChart", default)]
    pub line_chart: Vec<DecodeCCharts>,
    #[serde(rename = "line3DChart", default)]
    pub line_3d_chart: Vec<DecodeCCharts>,
    #[serde(rename = "ofPieChart", default)]
    pub of_pie_chart: Vec<DecodeCCharts>,
    #[serde(rename = "pieChart", default)]
    pub pie_chart: Vec<DecodeCCharts>,
    #[serde(rename = "pie3DChart", default)]
    pub pie_3d_chart: Vec<DecodeCCharts>,
    #[serde(rename = "radarChart", default)]
    pub radar_chart: Vec<DecodeCCharts>,
    #[serde(rename = "scatterChart", default)]
    pub scatter_chart: Vec<DecodeCCharts>,
    #[serde(rename = "stockChart", default)]
    pub stock_chart: Vec<DecodeCCharts>,
    #[serde(rename = "surfaceChart", default)]
    pub surface_chart: Vec<DecodeCCharts>,
    #[serde(rename = "surface3DChart", default)]
    pub surface_3d_chart: Vec<DecodeCCharts>,
    #[serde(rename = "catAx", default)]
    pub cat_ax: Vec<DecodeCAxis>,
    #[serde(rename = "valAx", default)]
    pub val_ax: Vec<DecodeCAxis>,
    #[serde(rename = "dateAx", default)]
    pub date_ax: Vec<DecodeCAxis>,
    #[serde(rename = "serAx", default)]
    pub ser_ax: Vec<DecodeCAxis>,
    #[serde(rename = "dTable", default)]
    pub d_table: Option<DecodeCDTable>,
    #[serde(rename = "layout", default)]
    pub layout: Option<DecodeIgnore>,
    #[serde(rename = "spPr", default)]
    pub sp_pr: Option<DecodeCSpPr>,
}

/// Mirrors `CDTable` for deserialization.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecodeCDTable {
    #[serde(rename = "showHorzBorder", default)]
    pub show_horz_border: Option<AttrValBool>,
    #[serde(rename = "showVertBorder", default)]
    pub show_vert_border: Option<AttrValBool>,
    #[serde(rename = "showOutline", default)]
    pub show_outline: Option<AttrValBool>,
    #[serde(rename = "showKeys", default)]
    pub show_keys: Option<AttrValBool>,
}

/// Mirrors `CCharts` for deserialization.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecodeCCharts {
    #[serde(rename = "barDir", default)]
    pub bar_dir: Option<AttrValString>,
    #[serde(rename = "grouping", default)]
    pub grouping: Option<AttrValString>,
    #[serde(rename = "scatterStyle", default)]
    pub scatter_style: Option<AttrValString>,
    #[serde(rename = "shape", default)]
    pub shape: Option<AttrValString>,
    #[serde(rename = "ofPieType", default)]
    pub of_pie_type: Option<AttrValString>,
    #[serde(rename = "wireframe", default)]
    pub wireframe: Option<AttrValBool>,
    #[serde(rename = "upDownBars", default)]
    pub up_down_bars: Option<DecodeCUpDownBars>,
    #[serde(rename = "varyColors", default)]
    pub vary_colors: Option<AttrValBool>,
    #[serde(rename = "ser", default)]
    pub ser: Vec<DecodeCSer>,
    #[serde(rename = "axId", default)]
    pub ax_id: Vec<DecodeCAxId>,
    #[serde(rename = "dLbls", default)]
    pub d_lbls: Option<DecodeCDLbls>,
    #[serde(rename = "gapWidth", default)]
    pub gap_width: Option<AttrValInt>,
    #[serde(rename = "holeSize", default)]
    pub hole_size: Option<AttrValInt>,
    #[serde(rename = "overlap", default)]
    pub overlap: Option<AttrValInt>,
    #[serde(rename = "splitPos", default)]
    pub split_pos: Option<AttrValInt>,
    #[serde(rename = "bubbleScale", default)]
    pub bubble_scale: Option<AttrValFloat>,
    #[serde(rename = "smooth", default)]
    pub smooth: Option<AttrValBool>,
}

/// Mirrors `CUpDownBars` for deserialization.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecodeCUpDownBars {
    #[serde(rename = "gapWidth", default)]
    pub gap_width: Option<DecodeIgnore>,
    #[serde(rename = "upBars", default)]
    pub up_bars: Option<DecodeIgnore>,
    #[serde(rename = "downBars", default)]
    pub down_bars: Option<DecodeIgnore>,
    #[serde(rename = "extLst", default)]
    pub ext_lst: Option<DecodeIgnore>,
}

/// Mirrors `CSer` for deserialization.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecodeCSer {
    #[serde(rename = "idx", default)]
    pub idx: Option<AttrValInt>,
    #[serde(rename = "order", default)]
    pub order: Option<AttrValInt>,
    #[serde(rename = "tx", default)]
    pub tx: Option<DecodeCTx>,
    #[serde(rename = "spPr", default)]
    pub sp_pr: Option<DecodeCSpPr>,
    #[serde(rename = "dPt", default)]
    pub d_pt: Vec<DecodeCDPt>,
    #[serde(rename = "dLbls", default)]
    pub d_lbls: Option<DecodeCDLbls>,
    #[serde(rename = "marker", default)]
    pub marker: Option<DecodeCMarker>,
    #[serde(rename = "cat", default)]
    pub cat: Option<DecodeCCat>,
    #[serde(rename = "val", default)]
    pub val: Option<DecodeCVal>,
    #[serde(rename = "xVal", default)]
    pub x_val: Option<DecodeCCat>,
    #[serde(rename = "yVal", default)]
    pub y_val: Option<DecodeCVal>,
    #[serde(rename = "bubbleSize", default)]
    pub bubble_size: Option<DecodeCVal>,
    #[serde(rename = "smooth", default)]
    pub smooth: Option<AttrValBool>,
}

/// Mirrors `CTx` for deserialization.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecodeCTx {
    #[serde(rename = "strRef", default)]
    pub str_ref: Option<DecodeCStrRef>,
}

/// Mirrors `CStrRef` for deserialization.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecodeCStrRef {
    #[serde(rename = "f", default)]
    pub f: String,
}

/// Mirrors `CCat` for deserialization.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecodeCCat {
    #[serde(rename = "strRef", default)]
    pub str_ref: Option<DecodeCStrRef>,
}

/// Mirrors `CVal` for deserialization.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecodeCVal {
    #[serde(rename = "numRef", default)]
    pub num_ref: Option<DecodeCNumRef>,
}

/// Mirrors `CNumRef` for deserialization.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecodeCNumRef {
    #[serde(rename = "f", default)]
    pub f: String,
}

/// Mirrors `CMarker` for deserialization.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecodeCMarker {
    #[serde(rename = "symbol", default)]
    pub symbol: Option<AttrValString>,
    #[serde(rename = "size", default)]
    pub size: Option<AttrValInt>,
    #[serde(rename = "spPr", default)]
    pub sp_pr: Option<DecodeCSpPr>,
}

/// Mirrors `CDPt` for deserialization.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecodeCDPt {
    #[serde(rename = "idx", default)]
    pub idx: Option<AttrValInt>,
    #[serde(rename = "spPr", default)]
    pub sp_pr: Option<DecodeCSpPr>,
}

/// Mirrors `CDLbls` for deserialization.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecodeCDLbls {
    #[serde(rename = "dLblPos", default)]
    pub d_lbl_pos: Option<AttrValString>,
    #[serde(rename = "numFmt", default)]
    pub num_fmt: Option<DecodeCNumFmt>,
    #[serde(rename = "spPr", default)]
    pub sp_pr: Option<DecodeCSpPr>,
    #[serde(rename = "txPr", default)]
    pub tx_pr: Option<DecodeCTxPr>,
    #[serde(rename = "showLegendKey", default)]
    pub show_legend_key: Option<AttrValBool>,
    #[serde(rename = "showVal", default)]
    pub show_val: Option<AttrValBool>,
    #[serde(rename = "showCatName", default)]
    pub show_cat_name: Option<AttrValBool>,
    #[serde(rename = "showSerName", default)]
    pub show_ser_name: Option<AttrValBool>,
    #[serde(rename = "showPercent", default)]
    pub show_percent: Option<AttrValBool>,
    #[serde(rename = "showBubbleSize", default)]
    pub show_bubble_size: Option<AttrValBool>,
    #[serde(rename = "showLeaderLines", default)]
    pub show_leader_lines: Option<AttrValBool>,
}

/// Mirrors `CSpPr` for deserialization.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecodeCSpPr {
    #[serde(rename = "noFill", default)]
    pub no_fill: Option<String>,
    #[serde(rename = "solidFill", default)]
    pub solid_fill: Option<DecodeASolidFill>,
    #[serde(rename = "gradFill", default)]
    pub grad_fill: Option<DecodeIgnore>,
    #[serde(rename = "pattFill", default)]
    pub patt_fill: Option<DecodeIgnore>,
    #[serde(rename = "ln", default)]
    pub ln: Option<DecodeALn>,
    #[serde(rename = "sp3d", default)]
    pub sp3d: Option<DecodeIgnore>,
    #[serde(rename = "effectLst", default)]
    pub effect_lst: Option<DecodeIgnore>,
    #[serde(rename = "extLst", default)]
    pub ext_lst: Option<DecodeIgnore>,
}

/// Mirrors `ASolidFill` for deserialization.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecodeASolidFill {
    #[serde(rename = "srgbClr", default)]
    pub srgb_clr: Option<DecodeASrgbClr>,
    #[serde(rename = "schemeClr", default)]
    pub scheme_clr: Option<DecodeASchemeClr>,
    #[serde(rename = "prstClr", default)]
    pub prst_clr: Option<DecodeIgnore>,
    #[serde(rename = "sysClr", default)]
    pub sys_clr: Option<DecodeIgnore>,
    #[serde(rename = "hslClr", default)]
    pub hsl_clr: Option<DecodeIgnore>,
    #[serde(rename = "scrgbClr", default)]
    pub scrgb_clr: Option<DecodeIgnore>,
}

/// Mirrors `ASrgbClr` for deserialization.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecodeASrgbClr {
    #[serde(rename = "@val", default)]
    pub val: Option<String>,
    #[serde(rename = "lumMod", default)]
    pub lum_mod: Option<AttrValInt>,
    #[serde(rename = "lumOff", default)]
    pub lum_off: Option<AttrValInt>,
}

/// Mirrors `ASchemeClr` for deserialization.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecodeASchemeClr {
    #[serde(rename = "@val", default)]
    pub val: Option<String>,
    #[serde(rename = "lumMod", default)]
    pub lum_mod: Option<AttrValInt>,
    #[serde(rename = "lumOff", default)]
    pub lum_off: Option<AttrValInt>,
}

/// Mirrors `ALn` for deserialization.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecodeALn {
    #[serde(rename = "@w", default)]
    pub w: Option<String>,
    #[serde(rename = "noFill", default)]
    pub no_fill: Option<String>,
    #[serde(rename = "solidFill", default)]
    pub solid_fill: Option<DecodeASolidFill>,
    #[serde(rename = "gradFill", default)]
    pub grad_fill: Option<DecodeIgnore>,
    #[serde(rename = "pattFill", default)]
    pub patt_fill: Option<DecodeIgnore>,
    #[serde(rename = "round", default)]
    pub round: Option<DecodeIgnore>,
    #[serde(rename = "bevel", default)]
    pub bevel: Option<DecodeIgnore>,
    #[serde(rename = "miter", default)]
    pub miter: Option<DecodeIgnore>,
    #[serde(rename = "prstDash", default)]
    pub prst_dash: Option<AttrValString>,
    #[serde(rename = "custDash", default)]
    pub cust_dash: Option<DecodeIgnore>,
    #[serde(rename = "headEnd", default)]
    pub head_end: Option<DecodeIgnore>,
    #[serde(rename = "tailEnd", default)]
    pub tail_end: Option<DecodeIgnore>,
    #[serde(rename = "extLst", default)]
    pub ext_lst: Option<DecodeIgnore>,
}

/// Mirrors `CAxId` for deserialization.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecodeCAxId {
    #[serde(rename = "@val", default)]
    pub val: i64,
}

/// Mirrors a generic chart axis (`c:catAx`, `c:valAx`, `c:dateAx`, `c:serAx`).
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecodeCAxis {
    #[serde(rename = "axId", default)]
    pub ax_id: Option<AttrValInt>,
    #[serde(rename = "scaling", default)]
    pub scaling: Option<DecodeCScaling>,
    #[serde(rename = "delete", default)]
    pub delete: Option<AttrValBool>,
    #[serde(rename = "axPos", default)]
    pub ax_pos: Option<AttrValString>,
    #[serde(rename = "majorGridlines", default)]
    pub major_grid_lines: Option<DecodeIgnore>,
    #[serde(rename = "minorGridlines", default)]
    pub minor_grid_lines: Option<DecodeIgnore>,
    #[serde(rename = "title", default)]
    pub title: Option<DecodeCTitle>,
    #[serde(rename = "numFmt", default)]
    pub num_fmt: Option<DecodeCNumFmt>,
    #[serde(rename = "majorTickMark", default)]
    pub major_tick_mark: Option<AttrValString>,
    #[serde(rename = "minorTickMark", default)]
    pub minor_tick_mark: Option<AttrValString>,
    #[serde(rename = "tickLblPos", default)]
    pub tick_lbl_pos: Option<AttrValString>,
    #[serde(rename = "tickLblSkip", default)]
    pub tick_lbl_skip: Option<AttrValInt>,
    #[serde(rename = "crossAx", default)]
    pub cross_ax: Option<AttrValInt>,
    #[serde(rename = "crosses", default)]
    pub crosses: Option<AttrValString>,
    #[serde(rename = "crossBetween", default)]
    pub cross_between: Option<AttrValString>,
    #[serde(rename = "auto", default)]
    pub auto: Option<AttrValBool>,
    #[serde(rename = "lblOffset", default)]
    pub lbl_offset: Option<AttrValInt>,
    #[serde(rename = "majorUnit", default)]
    pub major_unit: Option<AttrValFloat>,
    #[serde(rename = "minorUnit", default)]
    pub minor_unit: Option<AttrValFloat>,
    #[serde(rename = "spPr", default)]
    pub sp_pr: Option<DecodeIgnore>,
    #[serde(rename = "txPr", default)]
    pub tx_pr: Option<DecodeIgnore>,
    #[serde(rename = "extLst", default)]
    pub ext_lst: Option<DecodeIgnore>,
}

/// Mirrors `CScaling` for deserialization.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecodeCScaling {
    #[serde(rename = "orientation", default)]
    pub orientation: Option<AttrValString>,
    #[serde(rename = "max", default)]
    pub max: Option<AttrValFloat>,
    #[serde(rename = "min", default)]
    pub min: Option<AttrValFloat>,
    #[serde(rename = "logBase", default)]
    pub log_base: Option<AttrValFloat>,
}

/// Mirrors `CNumFmt` for deserialization.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecodeCNumFmt {
    #[serde(rename = "@formatCode", default)]
    pub format_code: String,
    #[serde(rename = "@sourceLinked", default)]
    pub source_linked: bool,
}

/// Mirrors `CTitle` for deserialization.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecodeCTitle {
    #[serde(rename = "tx", default)]
    pub tx: Option<DecodeCTitleTx>,
    #[serde(rename = "overlay", default)]
    pub overlay: Option<AttrValBool>,
    #[serde(rename = "layout", default)]
    pub layout: Option<DecodeIgnore>,
    #[serde(rename = "spPr", default)]
    pub sp_pr: Option<DecodeCSpPr>,
    #[serde(rename = "txPr", default)]
    pub tx_pr: Option<DecodeCTxPr>,
    #[serde(rename = "extLst", default)]
    pub ext_lst: Option<DecodeIgnore>,
}

/// Mirrors `CLegend` for deserialization.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecodeCLegend {
    #[serde(rename = "legendPos", default)]
    pub legend_pos: Option<AttrValString>,
    #[serde(rename = "overlay", default)]
    pub overlay: Option<AttrValBool>,
    #[serde(rename = "layout", default)]
    pub layout: Option<DecodeIgnore>,
    #[serde(rename = "txPr", default)]
    pub tx_pr: Option<DecodeCTxPr>,
    #[serde(rename = "spPr", default)]
    pub sp_pr: Option<DecodeIgnore>,
    #[serde(rename = "legendEntry", default)]
    pub legend_entry: Vec<DecodeIgnore>,
    #[serde(rename = "extLst", default)]
    pub ext_lst: Option<DecodeIgnore>,
}

/// Mirrors `CTxPr` for deserialization.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecodeCTxPr {
    #[serde(rename = "bodyPr", default)]
    pub body_pr: Option<DecodeIgnore>,
    #[serde(rename = "lstStyle", default)]
    pub lst_style: Option<DecodeIgnore>,
    #[serde(rename = "p", default)]
    pub p: Option<DecodeAP>,
}

/// Mirrors `AP` for deserialization.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecodeAP {
    #[serde(rename = "pPr", default)]
    pub p_pr: Option<DecodeAPPr>,
    #[serde(rename = "r", default)]
    pub r: Vec<DecodeCR>,
    #[serde(rename = "t", default)]
    pub t: Option<String>,
    #[serde(rename = "endParaRPr", default)]
    pub end_para_r_pr: Option<DecodeIgnore>,
}

/// Mirrors `APPr` for deserialization.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecodeAPPr {
    #[serde(rename = "defRPr", default)]
    pub def_r_pr: Option<DecodeARPr>,
}

/// Mirrors `ARPr` for deserialization.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecodeARPr {
    #[serde(rename = "@sz", default)]
    pub sz: Option<String>,
    #[serde(rename = "@b", default)]
    pub b: Option<String>,
    #[serde(rename = "@i", default)]
    pub i: Option<String>,
    #[serde(rename = "@strike", default)]
    pub strike: Option<String>,
    #[serde(rename = "@u", default)]
    pub u: Option<String>,
    #[serde(rename = "@vertAlign", default)]
    pub vert_align: Option<String>,
    #[serde(rename = "solidFill", default)]
    pub solid_fill: Option<DecodeASolidFill>,
    #[serde(rename = "latin", default)]
    pub latin: Option<DecodeALatin>,
    #[serde(rename = "ea", default)]
    pub ea: Option<DecodeALatin>,
    #[serde(rename = "cs", default)]
    pub cs: Option<DecodeALatin>,
}

/// Mirrors `ALatin` for deserialization.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecodeALatin {
    #[serde(rename = "@typeface", default)]
    pub typeface: Option<String>,
}

/// Mirrors the `c:tx` child of a chart title for deserialization.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecodeCTitleTx {
    #[serde(rename = "strRef", default)]
    pub str_ref: Option<DecodeCStrRef>,
    #[serde(rename = "rich", default)]
    pub rich: Option<DecodeCRich>,
}

/// Mirrors `CRich` for deserialization.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecodeCRich {
    #[serde(rename = "bodyPr", default)]
    pub body_pr: Option<DecodeIgnore>,
    #[serde(rename = "lstStyle", default)]
    pub lst_style: Option<DecodeIgnore>,
    #[serde(rename = "p", default)]
    pub p: Vec<DecodeAP>,
}

/// Mirrors a rich-text run for deserialization.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecodeCR {
    #[serde(rename = "rPr", default)]
    pub r_pr: Option<DecodeARPr>,
    #[serde(rename = "t", default)]
    pub t: Option<String>,
}
