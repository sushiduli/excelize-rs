//! Theme part (`xl/theme/themeN.xml`).
//!
//! Ported from Go `xmlTheme.go`.

use serde::{Deserialize, Serialize};

use super::common::{AttrValString, XlsxExtLst, XlsxInnerXml};

// ------------------------------------------------------------------
// Serialization types (with explicit `a:` namespace prefix)
// ------------------------------------------------------------------

/// Directly maps the theme element in the namespace
/// http://schemas.openxmlformats.org/drawingml/2006/main.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "a:theme")]
pub struct XlsxTheme {
    #[serde(rename = "@xmlns:a", default)]
    pub xmlns_a: Option<String>,
    #[serde(rename = "@xmlns:r", default)]
    pub xmlns_r: Option<String>,
    #[serde(rename = "@name", default)]
    pub name: String,
    #[serde(rename = "a:themeElements", default)]
    pub theme_elements: XlsxBaseStyles,
    #[serde(rename = "a:objectDefaults", default)]
    pub object_defaults: XlsxInnerXml,
    #[serde(rename = "a:extraClrSchemeLst", default)]
    pub extra_clr_scheme_lst: XlsxInnerXml,
    #[serde(rename = "a:custClrLst", default)]
    pub cust_clr_lst: Option<XlsxInnerXml>,
    #[serde(rename = "a:extLst", default)]
    pub ext_lst: Option<XlsxExtLst>,
}

/// Defines the theme elements for a theme, and is the workhorse of the theme.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "a:themeElements")]
pub struct XlsxBaseStyles {
    #[serde(rename = "a:clrScheme", default)]
    pub clr_scheme: XlsxColorScheme,
    #[serde(rename = "a:fontScheme", default)]
    pub font_scheme: XlsxFontScheme,
    #[serde(rename = "a:fmtScheme", default)]
    pub fmt_scheme: XlsxStyleMatrix,
    #[serde(rename = "a:extLst", default)]
    pub ext_lst: Option<XlsxExtLst>,
}

/// Holds the actual color values that are to be applied to a given diagram and
/// how those colors are to be applied.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct XlsxCtColor {
    #[serde(rename = "a:scrgbClr", default)]
    pub scrgb_clr: Option<XlsxInnerXml>,
    #[serde(rename = "a:srgbClr", default)]
    pub srgb_clr: Option<AttrValString>,
    #[serde(rename = "a:hslClr", default)]
    pub hsl_clr: Option<XlsxInnerXml>,
    #[serde(rename = "a:sysClr", default)]
    pub sys_clr: Option<XlsxSysClr>,
    #[serde(rename = "a:schemeClr", default)]
    pub scheme_clr: Option<XlsxInnerXml>,
    #[serde(rename = "a:prstClr", default)]
    pub prst_clr: Option<XlsxInnerXml>,
}

/// Defines a set of colors for the theme.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "a:clrScheme")]
pub struct XlsxColorScheme {
    #[serde(rename = "@name", default)]
    pub name: String,
    #[serde(rename = "a:dk1", default)]
    pub dk1: XlsxCtColor,
    #[serde(rename = "a:lt1", default)]
    pub lt1: XlsxCtColor,
    #[serde(rename = "a:dk2", default)]
    pub dk2: XlsxCtColor,
    #[serde(rename = "a:lt2", default)]
    pub lt2: XlsxCtColor,
    #[serde(rename = "a:accent1", default)]
    pub accent1: XlsxCtColor,
    #[serde(rename = "a:accent2", default)]
    pub accent2: XlsxCtColor,
    #[serde(rename = "a:accent3", default)]
    pub accent3: XlsxCtColor,
    #[serde(rename = "a:accent4", default)]
    pub accent4: XlsxCtColor,
    #[serde(rename = "a:accent5", default)]
    pub accent5: XlsxCtColor,
    #[serde(rename = "a:accent6", default)]
    pub accent6: XlsxCtColor,
    #[serde(rename = "a:hlink", default)]
    pub hlink: XlsxCtColor,
    #[serde(rename = "a:folHlink", default)]
    pub fol_hlink: XlsxCtColor,
    #[serde(rename = "a:extLst", default)]
    pub ext_lst: Option<XlsxExtLst>,
}

/// Defines an additional font that is used for language specific fonts in
/// themes.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XlsxCtSupplementalFont {
    #[serde(rename = "@script", default)]
    pub script: String,
    #[serde(rename = "@typeface", default)]
    pub typeface: String,
}

/// Defines a major and minor font which is used in the font scheme.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct XlsxFontCollection {
    #[serde(rename = "a:latin", default)]
    pub latin: Option<XlsxCtTextFont>,
    #[serde(rename = "a:ea", default)]
    pub ea: Option<XlsxCtTextFont>,
    #[serde(rename = "a:cs", default)]
    pub cs: Option<XlsxCtTextFont>,
    #[serde(rename = "a:font", default)]
    pub font: Vec<XlsxCtSupplementalFont>,
    #[serde(rename = "a:extLst", default)]
    pub ext_lst: Option<XlsxExtLst>,
}

/// Defines the font scheme within the theme.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "a:fontScheme")]
pub struct XlsxFontScheme {
    #[serde(rename = "@name", default)]
    pub name: String,
    #[serde(rename = "a:majorFont", default)]
    pub major_font: XlsxFontCollection,
    #[serde(rename = "a:minorFont", default)]
    pub minor_font: XlsxFontCollection,
    #[serde(rename = "a:extLst", default)]
    pub ext_lst: Option<XlsxExtLst>,
}

/// Defines a set of formatting options, which can be referenced by documents
/// that apply a certain style to a given part of an object.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "a:fmtScheme")]
pub struct XlsxStyleMatrix {
    #[serde(rename = "@name", default)]
    pub name: Option<String>,
    #[serde(rename = "a:fillStyleLst", default)]
    pub fill_style_lst: XlsxInnerXml,
    #[serde(rename = "a:lnStyleLst", default)]
    pub ln_style_lst: XlsxInnerXml,
    #[serde(rename = "a:effectStyleLst", default)]
    pub effect_style_lst: XlsxInnerXml,
    #[serde(rename = "a:bgFillStyleLst", default)]
    pub bg_fill_style_lst: XlsxInnerXml,
}

/// Specifies a color bound to predefined operating system elements.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "a:sysClr")]
pub struct XlsxSysClr {
    #[serde(rename = "@val", default)]
    pub val: String,
    #[serde(rename = "@lastClr", default)]
    pub last_clr: String,
}

/// Defines a text font properties used by theme font collections.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XlsxCtTextFont {
    #[serde(rename = "@typeface", default)]
    pub typeface: String,
    #[serde(rename = "@panose", default)]
    pub panose: Option<String>,
    #[serde(rename = "@pitchFamily", default)]
    pub pitch_family: Option<String>,
    #[serde(rename = "@Charset", default)]
    pub charset: Option<String>,
}

// ------------------------------------------------------------------
// Deserialization types (no namespace prefix on child elements)
// ------------------------------------------------------------------

/// Deserialization-only representation of the `theme` element.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "theme")]
pub struct DecodeTheme {
    #[serde(rename = "@name", default)]
    pub name: String,
    #[serde(rename = "themeElements", default)]
    pub theme_elements: DecodeBaseStyles,
    #[serde(rename = "objectDefaults", default)]
    pub object_defaults: XlsxInnerXml,
    #[serde(rename = "extraClrSchemeLst", default)]
    pub extra_clr_scheme_lst: XlsxInnerXml,
    #[serde(rename = "custClrLst", default)]
    pub cust_clr_lst: Option<XlsxInnerXml>,
    #[serde(rename = "extLst", default)]
    pub ext_lst: Option<XlsxExtLst>,
}

/// Deserialization-only representation of the theme elements.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "themeElements")]
pub struct DecodeBaseStyles {
    #[serde(rename = "clrScheme", default)]
    pub clr_scheme: DecodeColorScheme,
    #[serde(rename = "fontScheme", default)]
    pub font_scheme: DecodeFontScheme,
    #[serde(rename = "fmtScheme", default)]
    pub fmt_scheme: DecodeStyleMatrix,
    #[serde(rename = "extLst", default)]
    pub ext_lst: Option<XlsxExtLst>,
}

/// Deserialization-only representation of a set of colors for the theme.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "clrScheme")]
pub struct DecodeColorScheme {
    #[serde(rename = "@name", default)]
    pub name: String,
    #[serde(rename = "dk1", default)]
    pub dk1: DecodeCtColor,
    #[serde(rename = "lt1", default)]
    pub lt1: DecodeCtColor,
    #[serde(rename = "dk2", default)]
    pub dk2: DecodeCtColor,
    #[serde(rename = "lt2", default)]
    pub lt2: DecodeCtColor,
    #[serde(rename = "accent1", default)]
    pub accent1: DecodeCtColor,
    #[serde(rename = "accent2", default)]
    pub accent2: DecodeCtColor,
    #[serde(rename = "accent3", default)]
    pub accent3: DecodeCtColor,
    #[serde(rename = "accent4", default)]
    pub accent4: DecodeCtColor,
    #[serde(rename = "accent5", default)]
    pub accent5: DecodeCtColor,
    #[serde(rename = "accent6", default)]
    pub accent6: DecodeCtColor,
    #[serde(rename = "hlink", default)]
    pub hlink: DecodeCtColor,
    #[serde(rename = "folHlink", default)]
    pub fol_hlink: DecodeCtColor,
    #[serde(rename = "extLst", default)]
    pub ext_lst: Option<XlsxExtLst>,
}

/// Deserialization-only representation of the actual color values.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecodeCtColor {
    #[serde(rename = "scrgbClr", default)]
    pub scrgb_clr: Option<XlsxInnerXml>,
    #[serde(rename = "srgbClr", default)]
    pub srgb_clr: Option<AttrValString>,
    #[serde(rename = "hslClr", default)]
    pub hsl_clr: Option<XlsxInnerXml>,
    #[serde(rename = "sysClr", default)]
    pub sys_clr: Option<XlsxSysClr>,
    #[serde(rename = "schemeClr", default)]
    pub scheme_clr: Option<XlsxInnerXml>,
    #[serde(rename = "prstClr", default)]
    pub prst_clr: Option<XlsxInnerXml>,
}

/// Deserialization-only representation of the font scheme within the theme.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "fontScheme")]
pub struct DecodeFontScheme {
    #[serde(rename = "@name", default)]
    pub name: String,
    #[serde(rename = "majorFont", default)]
    pub major_font: DecodeFontCollection,
    #[serde(rename = "minorFont", default)]
    pub minor_font: DecodeFontCollection,
    #[serde(rename = "extLst", default)]
    pub ext_lst: Option<XlsxExtLst>,
}

/// Deserialization-only representation of a major and minor font collection.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecodeFontCollection {
    #[serde(rename = "latin", default)]
    pub latin: Option<XlsxCtTextFont>,
    #[serde(rename = "ea", default)]
    pub ea: Option<XlsxCtTextFont>,
    #[serde(rename = "cs", default)]
    pub cs: Option<XlsxCtTextFont>,
    #[serde(rename = "font", default)]
    pub font: Vec<XlsxCtSupplementalFont>,
    #[serde(rename = "extLst", default)]
    pub ext_lst: Option<XlsxExtLst>,
}

/// Deserialization-only representation of a set of formatting options.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "fmtScheme")]
pub struct DecodeStyleMatrix {
    #[serde(rename = "@name", default)]
    pub name: Option<String>,
    #[serde(rename = "fillStyleLst", default)]
    pub fill_style_lst: XlsxInnerXml,
    #[serde(rename = "lnStyleLst", default)]
    pub ln_style_lst: XlsxInnerXml,
    #[serde(rename = "effectStyleLst", default)]
    pub effect_style_lst: XlsxInnerXml,
    #[serde(rename = "bgFillStyleLst", default)]
    pub bg_fill_style_lst: XlsxInnerXml,
}
