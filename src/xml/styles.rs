//! Styles part (`xl/styles.xml`).
//!
//! Ported from Go `xmlStyles.go`.

use serde::{Deserialize, Serialize};

use super::common::{
    AttrValBool, AttrValFloat, AttrValInt, AttrValString, XlsxColor, XlsxExtLst, XlsxInnerXml,
};
use super::drawing::XlsxPositiveSize2D;

/// Directly maps the root element of the Styles part.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "styleSheet")]
pub struct XlsxStyleSheet {
    #[serde(rename = "@xmlns", default, skip_serializing_if = "Option::is_none")]
    pub xmlns: Option<String>,
    #[serde(rename = "numFmts", default, skip_serializing_if = "Option::is_none")]
    pub num_fmts: Option<XlsxNumFmts>,
    #[serde(rename = "fonts", default, skip_serializing_if = "Option::is_none")]
    pub fonts: Option<XlsxFonts>,
    #[serde(rename = "fills", default, skip_serializing_if = "Option::is_none")]
    pub fills: Option<XlsxFills>,
    #[serde(rename = "borders", default, skip_serializing_if = "Option::is_none")]
    pub borders: Option<XlsxBorders>,
    #[serde(rename = "cellStyleXfs", default, skip_serializing_if = "Option::is_none")]
    pub cell_style_xfs: Option<XlsxCellStyleXfs>,
    #[serde(rename = "cellXfs", default, skip_serializing_if = "Option::is_none")]
    pub cell_xfs: Option<XlsxCellXfs>,
    #[serde(rename = "cellStyles", default, skip_serializing_if = "Option::is_none")]
    pub cell_styles: Option<XlsxCellStyles>,
    #[serde(rename = "dxfs", default, skip_serializing_if = "Option::is_none")]
    pub dxfs: Option<XlsxDxfs>,
    #[serde(rename = "tableStyles", default, skip_serializing_if = "Option::is_none")]
    pub table_styles: Option<XlsxTableStyles>,
    #[serde(rename = "colors", default, skip_serializing_if = "Option::is_none")]
    pub colors: Option<XlsxStyleColors>,
    #[serde(rename = "extLst", default, skip_serializing_if = "Option::is_none")]
    pub ext_lst: Option<XlsxExtLst>,
}

/// Formatting information pertaining to text alignment in cells.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct XlsxAlignment {
    #[serde(rename = "@horizontal", default, skip_serializing_if = "Option::is_none")]
    pub horizontal: Option<String>,
    #[serde(rename = "@indent", default, skip_serializing_if = "Option::is_none")]
    pub indent: Option<i64>,
    #[serde(rename = "@justifyLastLine", default, skip_serializing_if = "Option::is_none")]
    pub justify_last_line: Option<bool>,
    #[serde(rename = "@readingOrder", default, skip_serializing_if = "Option::is_none")]
    pub reading_order: Option<u64>,
    #[serde(rename = "@relativeIndent", default, skip_serializing_if = "Option::is_none")]
    pub relative_indent: Option<i64>,
    #[serde(rename = "@shrinkToFit", default, skip_serializing_if = "Option::is_none")]
    pub shrink_to_fit: Option<bool>,
    #[serde(rename = "@textRotation", default, skip_serializing_if = "Option::is_none")]
    pub text_rotation: Option<i64>,
    #[serde(rename = "@vertical", default, skip_serializing_if = "Option::is_none")]
    pub vertical: Option<String>,
    #[serde(rename = "@wrapText", default, skip_serializing_if = "Option::is_none")]
    pub wrap_text: Option<bool>,
}

/// Protection properties associated with a cell.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XlsxProtection {
    #[serde(rename = "@hidden", default, skip_serializing_if = "Option::is_none")]
    pub hidden: Option<bool>,
    #[serde(rename = "@locked", default, skip_serializing_if = "Option::is_none")]
    pub locked: Option<bool>,
}

/// A single set of cell border.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct XlsxLine {
    #[serde(rename = "@style", default, skip_serializing_if = "Option::is_none")]
    pub style: Option<String>,
    #[serde(rename = "color", default, skip_serializing_if = "Option::is_none")]
    pub color: Option<XlsxColor>,
}

/// Directly maps the fonts element.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "fonts")]
pub struct XlsxFonts {
    #[serde(rename = "@count", default)]
    pub count: i64,
    #[serde(rename = "font", default)]
    pub font: Vec<XlsxFont>,
}

/// Directly maps the font element.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct XlsxFont {
    #[serde(rename = "name", default, skip_serializing_if = "Option::is_none")]
    pub name: Option<AttrValString>,
    #[serde(rename = "charset", default, skip_serializing_if = "Option::is_none")]
    pub charset: Option<AttrValInt>,
    #[serde(rename = "family", default, skip_serializing_if = "Option::is_none")]
    pub family: Option<AttrValInt>,
    #[serde(rename = "b", default, skip_serializing_if = "Option::is_none")]
    pub b: Option<AttrValBool>,
    #[serde(rename = "i", default, skip_serializing_if = "Option::is_none")]
    pub i: Option<AttrValBool>,
    #[serde(rename = "strike", default, skip_serializing_if = "Option::is_none")]
    pub strike: Option<AttrValBool>,
    #[serde(rename = "outline", default, skip_serializing_if = "Option::is_none")]
    pub outline: Option<AttrValBool>,
    #[serde(rename = "shadow", default, skip_serializing_if = "Option::is_none")]
    pub shadow: Option<AttrValBool>,
    #[serde(rename = "condense", default, skip_serializing_if = "Option::is_none")]
    pub condense: Option<AttrValBool>,
    #[serde(rename = "extend", default, skip_serializing_if = "Option::is_none")]
    pub extend: Option<AttrValBool>,
    #[serde(rename = "color", default, skip_serializing_if = "Option::is_none")]
    pub color: Option<XlsxColor>,
    #[serde(rename = "sz", default, skip_serializing_if = "Option::is_none")]
    pub sz: Option<AttrValFloat>,
    #[serde(rename = "u", default, skip_serializing_if = "Option::is_none")]
    pub u: Option<AttrValString>,
    #[serde(rename = "vertAlign", default, skip_serializing_if = "Option::is_none")]
    pub vert_align: Option<AttrValString>,
    #[serde(rename = "scheme", default, skip_serializing_if = "Option::is_none")]
    pub scheme: Option<AttrValString>,
}

/// Directly maps the fills element.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "fills")]
pub struct XlsxFills {
    #[serde(rename = "@count", default)]
    pub count: i64,
    #[serde(rename = "fill", default)]
    pub fill: Vec<XlsxFill>,
}

/// Directly maps the fill element.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct XlsxFill {
    #[serde(rename = "patternFill", default, skip_serializing_if = "Option::is_none")]
    pub pattern_fill: Option<XlsxPatternFill>,
    #[serde(rename = "gradientFill", default, skip_serializing_if = "Option::is_none")]
    pub gradient_fill: Option<XlsxGradientFill>,
}

/// Cell fill information for pattern and solid color cell fills.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct XlsxPatternFill {
    #[serde(rename = "@patternType", default, skip_serializing_if = "Option::is_none")]
    pub pattern_type: Option<String>,
    #[serde(rename = "fgColor", default, skip_serializing_if = "Option::is_none")]
    pub fg_color: Option<XlsxColor>,
    #[serde(rename = "bgColor", default, skip_serializing_if = "Option::is_none")]
    pub bg_color: Option<XlsxColor>,
}

/// A gradient-style cell fill.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct XlsxGradientFill {
    #[serde(rename = "@bottom", default, skip_serializing_if = "Option::is_none")]
    pub bottom: Option<f64>,
    #[serde(rename = "@degree", default, skip_serializing_if = "Option::is_none")]
    pub degree: Option<f64>,
    #[serde(rename = "@left", default, skip_serializing_if = "Option::is_none")]
    pub left: Option<f64>,
    #[serde(rename = "@right", default, skip_serializing_if = "Option::is_none")]
    pub right: Option<f64>,
    #[serde(rename = "@top", default, skip_serializing_if = "Option::is_none")]
    pub top: Option<f64>,
    #[serde(rename = "@type", default, skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(rename = "stop", default)]
    pub stop: Vec<XlsxGradientFillStop>,
}

/// A single gradient stop.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct XlsxGradientFillStop {
    #[serde(rename = "@position")]
    pub position: f64,
    #[serde(rename = "color", default, skip_serializing_if = "Option::is_none")]
    pub color: Option<XlsxColor>,
}

/// Directly maps the borders element.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "borders")]
pub struct XlsxBorders {
    #[serde(rename = "@count", default)]
    pub count: i64,
    #[serde(rename = "border", default)]
    pub border: Vec<XlsxBorder>,
}

/// A single set of cell border formats.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct XlsxBorder {
    #[serde(rename = "@diagonalDown", default, skip_serializing_if = "Option::is_none")]
    pub diagonal_down: Option<bool>,
    #[serde(rename = "@diagonalUp", default, skip_serializing_if = "Option::is_none")]
    pub diagonal_up: Option<bool>,
    #[serde(rename = "@outline", default, skip_serializing_if = "Option::is_none")]
    pub outline: Option<bool>,
    #[serde(rename = "left", default, skip_serializing_if = "Option::is_none")]
    pub left: Option<XlsxLine>,
    #[serde(rename = "right", default, skip_serializing_if = "Option::is_none")]
    pub right: Option<XlsxLine>,
    #[serde(rename = "top", default, skip_serializing_if = "Option::is_none")]
    pub top: Option<XlsxLine>,
    #[serde(rename = "bottom", default, skip_serializing_if = "Option::is_none")]
    pub bottom: Option<XlsxLine>,
    #[serde(rename = "diagonal", default, skip_serializing_if = "Option::is_none")]
    pub diagonal: Option<XlsxLine>,
    #[serde(rename = "vertical", default, skip_serializing_if = "Option::is_none")]
    pub vertical: Option<XlsxLine>,
    #[serde(rename = "horizontal", default, skip_serializing_if = "Option::is_none")]
    pub horizontal: Option<XlsxLine>,
}

/// Directly maps the cellStyles element.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "cellStyles")]
pub struct XlsxCellStyles {
    #[serde(rename = "@count", default)]
    pub count: i64,
    #[serde(rename = "cellStyle", default)]
    pub cell_style: Vec<XlsxCellStyle>,
}

/// A named cell style.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "cellStyle")]
pub struct XlsxCellStyle {
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@xfId")]
    pub xf_id: i64,
    #[serde(rename = "@builtinId", default, skip_serializing_if = "Option::is_none")]
    pub built_in_id: Option<i64>,
    #[serde(rename = "@iLevel", default, skip_serializing_if = "Option::is_none")]
    pub i_level: Option<i64>,
    #[serde(rename = "@hidden", default, skip_serializing_if = "Option::is_none")]
    pub hidden: Option<bool>,
    #[serde(rename = "@customBuiltin", default, skip_serializing_if = "Option::is_none")]
    pub custom_built_in: Option<bool>,
}

/// Directly maps the cellStyleXfs element.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "cellStyleXfs")]
pub struct XlsxCellStyleXfs {
    #[serde(rename = "@count", default)]
    pub count: i64,
    #[serde(rename = "xf", default)]
    pub xf: Vec<XlsxXf>,
}

/// A single xf element describing all formatting for a cell.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct XlsxXf {
    #[serde(rename = "@numFmtId", default, skip_serializing_if = "Option::is_none")]
    pub num_fmt_id: Option<i64>,
    #[serde(rename = "@fontId", default, skip_serializing_if = "Option::is_none")]
    pub font_id: Option<i64>,
    #[serde(rename = "@fillId", default, skip_serializing_if = "Option::is_none")]
    pub fill_id: Option<i64>,
    #[serde(rename = "@borderId", default, skip_serializing_if = "Option::is_none")]
    pub border_id: Option<i64>,
    #[serde(rename = "@xfId", default, skip_serializing_if = "Option::is_none")]
    pub xf_id: Option<i64>,
    #[serde(rename = "@quotePrefix", default, skip_serializing_if = "Option::is_none")]
    pub quote_prefix: Option<bool>,
    #[serde(rename = "@pivotButton", default, skip_serializing_if = "Option::is_none")]
    pub pivot_button: Option<bool>,
    #[serde(rename = "@applyNumberFormat", default, skip_serializing_if = "Option::is_none")]
    pub apply_number_format: Option<bool>,
    #[serde(rename = "@applyFont", default, skip_serializing_if = "Option::is_none")]
    pub apply_font: Option<bool>,
    #[serde(rename = "@applyFill", default, skip_serializing_if = "Option::is_none")]
    pub apply_fill: Option<bool>,
    #[serde(rename = "@applyBorder", default, skip_serializing_if = "Option::is_none")]
    pub apply_border: Option<bool>,
    #[serde(rename = "@applyAlignment", default, skip_serializing_if = "Option::is_none")]
    pub apply_alignment: Option<bool>,
    #[serde(rename = "@applyProtection", default, skip_serializing_if = "Option::is_none")]
    pub apply_protection: Option<bool>,
    #[serde(rename = "alignment", default, skip_serializing_if = "Option::is_none")]
    pub alignment: Option<XlsxAlignment>,
    #[serde(rename = "protection", default, skip_serializing_if = "Option::is_none")]
    pub protection: Option<XlsxProtection>,
}

/// Directly maps the cellXfs element.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "cellXfs")]
pub struct XlsxCellXfs {
    #[serde(rename = "@count", default)]
    pub count: i64,
    #[serde(rename = "xf", default)]
    pub xf: Vec<XlsxXf>,
}

/// Directly maps the dxfs element.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "dxfs")]
pub struct XlsxDxfs {
    #[serde(rename = "@count", default)]
    pub count: i64,
    #[serde(rename = "dxf", default)]
    pub dxfs: Vec<XlsxDxf>,
}

/// A single dxf record expressing incremental formatting.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct XlsxDxf {
    #[serde(rename = "font", default, skip_serializing_if = "Option::is_none")]
    pub font: Option<XlsxFont>,
    #[serde(rename = "numFmt", default, skip_serializing_if = "Option::is_none")]
    pub num_fmt: Option<XlsxNumFmt>,
    #[serde(rename = "fill", default, skip_serializing_if = "Option::is_none")]
    pub fill: Option<XlsxFill>,
    #[serde(rename = "alignment", default, skip_serializing_if = "Option::is_none")]
    pub alignment: Option<XlsxAlignment>,
    #[serde(rename = "border", default, skip_serializing_if = "Option::is_none")]
    pub border: Option<XlsxBorder>,
    #[serde(rename = "protection", default, skip_serializing_if = "Option::is_none")]
    pub protection: Option<XlsxProtection>,
    #[serde(rename = "extLst", default, skip_serializing_if = "Option::is_none")]
    pub ext_lst: Option<XlsxPositiveSize2D>,
}

/// Directly maps the tableStyles element.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "tableStyles")]
pub struct XlsxTableStyles {
    #[serde(rename = "@count", default)]
    pub count: i64,
    #[serde(rename = "@defaultPivotStyle")]
    pub default_pivot_style: String,
    #[serde(rename = "@defaultTableStyle")]
    pub default_table_style: String,
    #[serde(rename = "tableStyle", default)]
    pub table_styles: Vec<XlsxTableStyle>,
}

/// A single table style definition.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "tableStyle")]
pub struct XlsxTableStyle {
    #[serde(rename = "@name", default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(rename = "@pivot", default)]
    pub pivot: i64,
    #[serde(rename = "@count", default)]
    pub count: i64,
    #[serde(rename = "@table", default, skip_serializing_if = "Option::is_none")]
    pub table: Option<bool>,
    #[serde(rename = "$value", default)]
    pub table_style_element: String,
}

/// Directly maps the numFmts element.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "numFmts")]
pub struct XlsxNumFmts {
    #[serde(rename = "@count", default)]
    pub count: i64,
    #[serde(rename = "numFmt", default)]
    pub num_fmt: Vec<XlsxNumFmt>,
}

/// Directly maps the numFmt element.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XlsxNumFmt {
    #[serde(rename = "@numFmtId")]
    pub num_fmt_id: i64,
    #[serde(rename = "@formatCode")]
    pub format_code: String,
    #[serde(
        rename = "http://schemas.microsoft.com/office/spreadsheetml/2015/02/main formatCode16",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub format_code_16: Option<String>,
}

/// A single ARGB entry for the corresponding color index.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct XlsxIndexedColors {
    #[serde(rename = "rgbColor", default)]
    pub rgb_color: Vec<XlsxColor>,
}

/// Color information associated with the style sheet.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "colors")]
pub struct XlsxStyleColors {
    #[serde(rename = "indexedColors", default, skip_serializing_if = "Option::is_none")]
    pub indexed_colors: Option<XlsxIndexedColors>,
    #[serde(rename = "mruColors", default, skip_serializing_if = "Option::is_none")]
    pub mru_colors: Option<XlsxInnerXml>,
}
