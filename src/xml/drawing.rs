//! DrawingML / SpreadsheetDrawing XML types.
//!
//! Corresponds to the Go source file `xmlDrawing.go`.

use serde::{Deserialize, Serialize};

use super::common::{AttrValInt, AttrValString, RichTextRun, XlsxInnerXml};

// ------------------------------------------------------------------
// Non-visual drawing properties
// ------------------------------------------------------------------

/// Directly maps the `cNvPr` (Non-Visual Drawing Properties) element.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XlsxCNvPr {
    #[serde(rename = "@id")]
    pub id: i64,
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@descr")]
    pub descr: String,
    #[serde(rename = "@title", default)]
    pub title: Option<String>,
    #[serde(rename = "a:hlinkClick", default, alias = "hlinkClick")]
    pub hlink_click: Option<XlsxHlinkClick>,
}

/// Click Hyperlink.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XlsxHlinkClick {
    #[serde(rename = "@xmlns:r", default)]
    pub xmlns_r: Option<String>,
    #[serde(rename = "@r:id", default)]
    pub r_id: Option<String>,
    #[serde(rename = "@invalidUrl", default)]
    pub invalid_url: Option<String>,
    #[serde(rename = "@action", default)]
    pub action: Option<String>,
    #[serde(rename = "@tgtFrame", default)]
    pub tgt_frame: Option<String>,
    #[serde(rename = "@tooltip", default)]
    pub tooltip: Option<String>,
    #[serde(rename = "@history", default)]
    pub history: Option<bool>,
    #[serde(rename = "@highlightClick", default)]
    pub highlight_click: Option<bool>,
    #[serde(rename = "@endSnd", default)]
    pub end_snd: Option<bool>,
}

/// Picture Locks.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XlsxPicLocks {
    #[serde(
        rename = "@noAdjustHandles",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub no_adjust_handles: Option<bool>,
    #[serde(
        rename = "@noChangeArrowheads",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub no_change_arrowheads: Option<bool>,
    #[serde(rename = "@noChangeAspect")]
    pub no_change_aspect: bool,
    #[serde(
        rename = "@noChangeShapeType",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub no_change_shape_type: Option<bool>,
    #[serde(rename = "@noCrop", default, skip_serializing_if = "Option::is_none")]
    pub no_crop: Option<bool>,
    #[serde(
        rename = "@noEditPoints",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub no_edit_points: Option<bool>,
    #[serde(rename = "@noGrp", default, skip_serializing_if = "Option::is_none")]
    pub no_grp: Option<bool>,
    #[serde(rename = "@noMove", default, skip_serializing_if = "Option::is_none")]
    pub no_move: Option<bool>,
    #[serde(rename = "@noResize", default, skip_serializing_if = "Option::is_none")]
    pub no_resize: Option<bool>,
    #[serde(rename = "@noRot", default, skip_serializing_if = "Option::is_none")]
    pub no_rot: Option<bool>,
    #[serde(rename = "@noSelect", default, skip_serializing_if = "Option::is_none")]
    pub no_select: Option<bool>,
}

// ------------------------------------------------------------------
// Picture fill / blip
// ------------------------------------------------------------------

/// Specifies the existence of an image and contains a reference to the image data.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XlsxBlip {
    #[serde(rename = "@r:embed")]
    pub embed: String,
    #[serde(rename = "@cstate", default)]
    pub cstate: Option<String>,
    #[serde(rename = "@xmlns:r")]
    pub xmlns_r: String,
    #[serde(rename = "a:extLst", default)]
    pub ext_list: Option<XlsxEGOfficeArtExtensionList>,
}

/// Stretch fill.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XlsxStretch {
    #[serde(rename = "a:fillRect", default)]
    pub fill_rect: String,
}

/// SVG blip.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XlsxCTSVGBlip {
    #[serde(rename = "@xmlns:asvg")]
    pub xmlns_asvg: String,
    #[serde(rename = "@r:embed")]
    pub embed: String,
    #[serde(rename = "@r:link", default)]
    pub link: Option<String>,
}

/// Office art extension.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "a:ext")]
pub struct XlsxCTOfficeArtExtension {
    #[serde(rename = "@uri")]
    pub uri: String,
    #[serde(rename = "asvg:svgBlip")]
    pub svg_blip: XlsxCTSVGBlip,
}

/// Office art extension list.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XlsxEGOfficeArtExtensionList {
    #[serde(rename = "a:ext", default)]
    pub ext: Vec<XlsxCTOfficeArtExtension>,
}

/// Picture fill.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XlsxBlipFill {
    #[serde(rename = "a:blip")]
    pub blip: XlsxBlip,
    #[serde(rename = "a:stretch")]
    pub stretch: XlsxStretch,
}

// ------------------------------------------------------------------
// Geometry / transform
// ------------------------------------------------------------------

/// Column/row offset.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XlsxOff {
    #[serde(rename = "@x")]
    pub x: i64,
    #[serde(rename = "@y")]
    pub y: i64,
}

/// Positive 2D size.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XlsxPositiveSize2D {
    #[serde(rename = "@cx")]
    pub cx: i64,
    #[serde(rename = "@cy")]
    pub cy: i64,
}

/// Preset geometry.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XlsxPrstGeom {
    #[serde(rename = "@prst")]
    pub prst: String,
}

/// 2D transform.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XlsxXfrm {
    #[serde(rename = "a:off", alias = "off")]
    pub off: XlsxOff,
    #[serde(rename = "a:ext", alias = "ext")]
    pub ext: XlsxPositiveSize2D,
}

// ------------------------------------------------------------------
// Non-visual picture properties
// ------------------------------------------------------------------

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XlsxCNvPicPr {
    #[serde(rename = "a:picLocks")]
    pub pic_locks: XlsxPicLocks,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XlsxNvPicPr {
    #[serde(rename = "xdr:cNvPr")]
    pub c_nv_pr: XlsxCNvPr,
    #[serde(rename = "xdr:cNvPicPr")]
    pub c_nv_pic_pr: XlsxCNvPicPr,
}

// ------------------------------------------------------------------
// Color helpers used by shape properties
// ------------------------------------------------------------------

/// RGB color model - percentage variant.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct AScrgbClr {
    #[serde(rename = "@r")]
    pub r: f64,
    #[serde(rename = "@g")]
    pub g: f64,
    #[serde(rename = "@b")]
    pub b: f64,
}

/// Scheme color.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ASchemeClr {
    #[serde(rename = "@val", default)]
    pub val: Option<String>,
    #[serde(rename = "a:lumMod", default)]
    pub lum_mod: Option<AttrValInt>,
    #[serde(rename = "a:lumOff", default)]
    pub lum_off: Option<AttrValInt>,
}

/// RGB color model - hex variant.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ASrgbClr {
    #[serde(rename = "@val", default)]
    pub val: Option<String>,
    #[serde(rename = "a:tint", default)]
    pub tint: Option<AttrValInt>,
    #[serde(rename = "a:shade", default)]
    pub shade: Option<AttrValInt>,
    #[serde(rename = "a:comp", default)]
    pub comp: Option<AttrValInt>,
    #[serde(rename = "a:inv", default)]
    pub inv: Option<AttrValInt>,
    #[serde(rename = "a:gray", default)]
    pub gray: Option<AttrValInt>,
    #[serde(rename = "a:alpha", default)]
    pub alpha: Option<AttrValInt>,
    #[serde(rename = "a:alphaOff", default)]
    pub alpha_off: Option<AttrValInt>,
    #[serde(rename = "a:alphaMod", default)]
    pub alpha_mod: Option<AttrValInt>,
    #[serde(rename = "a:hue", default)]
    pub hue: Option<AttrValInt>,
    #[serde(rename = "a:hueOff", default)]
    pub hue_off: Option<AttrValInt>,
    #[serde(rename = "a:hueMod", default)]
    pub hue_mod: Option<AttrValInt>,
    #[serde(rename = "a:sat", default)]
    pub sat: Option<AttrValInt>,
    #[serde(rename = "a:satOff", default)]
    pub sat_off: Option<AttrValInt>,
    #[serde(rename = "a:satMod", default)]
    pub sat_mod: Option<AttrValInt>,
    #[serde(rename = "a:lum", default)]
    pub lum: Option<AttrValInt>,
    #[serde(rename = "a:lumOff", default)]
    pub lum_off: Option<AttrValInt>,
    #[serde(rename = "a:lumMod", default)]
    pub lum_mod: Option<AttrValInt>,
    #[serde(rename = "a:red", default)]
    pub red: Option<AttrValInt>,
    #[serde(rename = "a:redOff", default)]
    pub red_off: Option<AttrValInt>,
    #[serde(rename = "a:redMod", default)]
    pub red_mod: Option<AttrValInt>,
    #[serde(rename = "a:green", default)]
    pub green: Option<AttrValInt>,
    #[serde(rename = "a:greenOff", default)]
    pub green_off: Option<AttrValInt>,
    #[serde(rename = "a:greenMod", default)]
    pub green_mod: Option<AttrValInt>,
    #[serde(rename = "a:blue", default)]
    pub blue: Option<AttrValInt>,
    #[serde(rename = "a:blueOff", default)]
    pub blue_off: Option<AttrValInt>,
    #[serde(rename = "a:blueMod", default)]
    pub blue_mod: Option<AttrValInt>,
    #[serde(rename = "a:gamma", default)]
    pub gamma: Option<AttrValInt>,
    #[serde(rename = "a:invGamma", default)]
    pub inv_gamma: Option<AttrValInt>,
}

/// Solid fill.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ASolidFill {
    #[serde(rename = "a:schemeClr", default)]
    pub scheme_clr: Option<ASchemeClr>,
    #[serde(rename = "a:srgbClr", default)]
    pub srgb_clr: Option<ASrgbClr>,
    #[serde(rename = "a:prstClr", default)]
    pub prst_clr: Option<AttrValString>,
}

/// Outline.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ALn {
    #[serde(rename = "@algn", default)]
    pub algn: Option<String>,
    #[serde(rename = "@cap", default)]
    pub cap: Option<String>,
    #[serde(rename = "@cmpd", default)]
    pub cmpd: Option<String>,
    #[serde(rename = "@w", default)]
    pub w: Option<i64>,
    #[serde(rename = "a:noFill", default)]
    pub no_fill: Option<AttrValString>,
    #[serde(rename = "a:round", default)]
    pub round: Option<String>,
    #[serde(rename = "a:solidFill", default)]
    pub solid_fill: Option<ASolidFill>,
    #[serde(rename = "a:prstDash", default)]
    pub prst_dash: Option<AttrValString>,
    #[serde(rename = "a:prstClr", default)]
    pub prst_clr: Option<XlsxInnerXml>,
}

// ------------------------------------------------------------------
// Shape properties and picture
// ------------------------------------------------------------------

/// Shape properties.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XlsxSpPr {
    #[serde(rename = "a:xfrm")]
    pub xfrm: XlsxXfrm,
    #[serde(rename = "a:prstGeom")]
    pub prst_geom: XlsxPrstGeom,
    #[serde(rename = "a:solidFill", default)]
    pub solid_fill: Option<ASolidFill>,
    #[serde(rename = "a:ln", default)]
    pub ln: Option<ALn>,
}

/// Picture element.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XlsxPic {
    #[serde(rename = "xdr:nvPicPr")]
    pub nv_pic_pr: XlsxNvPicPr,
    #[serde(rename = "xdr:blipFill")]
    pub blip_fill: XlsxBlipFill,
    #[serde(rename = "xdr:spPr")]
    pub sp_pr: XlsxSpPr,
}

// ------------------------------------------------------------------
// Anchors
// ------------------------------------------------------------------

/// Starting anchor.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XlsxFrom {
    #[serde(rename = "xdr:col")]
    pub col: i64,
    #[serde(rename = "xdr:colOff")]
    pub col_off: i64,
    #[serde(rename = "xdr:row")]
    pub row: i64,
    #[serde(rename = "xdr:rowOff")]
    pub row_off: i64,
}

/// Ending anchor.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XlsxTo {
    #[serde(rename = "xdr:col")]
    pub col: i64,
    #[serde(rename = "xdr:colOff")]
    pub col_off: i64,
    #[serde(rename = "xdr:row")]
    pub row: i64,
    #[serde(rename = "xdr:rowOff")]
    pub row_off: i64,
}

/// Client data for a drawing object.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XdrClientData {
    #[serde(rename = "@fLocksWithSheet")]
    pub f_locks_with_sheet: bool,
    #[serde(rename = "@fPrintsWithSheet")]
    pub f_prints_with_sheet: bool,
}

/// Alternate content container.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "mc:AlternateContent")]
pub struct XlsxAlternateContent {
    #[serde(rename = "@xmlns:mc", default)]
    pub xmlns_mc: Option<String>,
    #[serde(rename = "$value", default)]
    pub content: XlsxInnerXml,
}

/// One/two cell anchor placeholder.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct XdrCellAnchor {
    #[serde(rename = "@editAs", default, skip_serializing_if = "Option::is_none")]
    pub edit_as: Option<String>,
    #[serde(rename = "xdr:pos", default, skip_serializing_if = "Option::is_none")]
    pub pos: Option<XlsxPoint2D>,
    #[serde(rename = "xdr:from", default, skip_serializing_if = "Option::is_none")]
    pub from: Option<XlsxFrom>,
    #[serde(rename = "xdr:to", default, skip_serializing_if = "Option::is_none")]
    pub to: Option<XlsxTo>,
    #[serde(rename = "xdr:ext", default, skip_serializing_if = "Option::is_none")]
    pub ext: Option<XlsxPositiveSize2D>,
    #[serde(rename = "xdr:sp", default, skip_serializing_if = "Option::is_none")]
    pub sp: Option<XdrSp>,
    #[serde(rename = "xdr:pic", default, skip_serializing_if = "Option::is_none")]
    pub pic: Option<XlsxPic>,
    #[serde(
        rename = "xdr:graphicFrame",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub graphic_frame: Option<XlsxGraphicFrame>,
    #[serde(rename = "mc:AlternateContent", default)]
    pub alternate_content: Vec<XlsxAlternateContent>,
    #[serde(
        rename = "xdr:clientData",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub client_data: Option<XdrClientData>,
}

/// Position used when serializing the cell anchor.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "xdr:pos")]
pub struct XlsxPoint2D {
    #[serde(rename = "@x")]
    pub x: i64,
    #[serde(rename = "@y")]
    pub y: i64,
}

/// Cell anchor position helper.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XlsxCellAnchorPos {
    #[serde(rename = "@editAs", default)]
    pub edit_as: Option<String>,
    #[serde(rename = "xdr:from", default)]
    pub from: Option<XlsxFrom>,
    #[serde(rename = "xdr:to", default)]
    pub to: Option<XlsxTo>,
    #[serde(rename = "xdr:pos", default)]
    pub pos: Option<XlsxInnerXml>,
    #[serde(rename = "xdr:ext", default)]
    pub ext: Option<XlsxPositiveSize2D>,
    #[serde(rename = "xdr:sp", default)]
    pub sp: Option<XlsxSp>,
    #[serde(rename = "xdr:grpSp", default)]
    pub grp_sp: Option<XlsxInnerXml>,
    #[serde(rename = "xdr:graphicFrame", default)]
    pub graphic_frame: Option<XlsxInnerXml>,
    #[serde(rename = "xdr:cxnSp", default)]
    pub cxn_sp: Option<XlsxInnerXml>,
    #[serde(rename = "xdr:pic", default)]
    pub pic: Option<XlsxInnerXml>,
    #[serde(rename = "xdr:contentPart", default)]
    pub content_part: Option<XlsxInnerXml>,
    #[serde(rename = "mc:AlternateContent", default)]
    pub alternate_content: Vec<XlsxAlternateContent>,
    #[serde(rename = "xdr:clientData", default)]
    pub client_data: Option<XlsxInnerXml>,
}

/// Shape placeholder used for raw inner XML capture.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XlsxSp {
    #[serde(rename = "@macro", default)]
    pub macro_name: Option<String>,
    #[serde(rename = "@textlink", default)]
    pub text_link: Option<String>,
    #[serde(rename = "@fLocksText", default)]
    pub f_locks_text: Option<bool>,
    #[serde(rename = "@fPublished", default)]
    pub f_published: Option<bool>,
    #[serde(rename = "$value", default)]
    pub content: XlsxInnerXml,
}

// ------------------------------------------------------------------
// Root worksheet drawing element
// ------------------------------------------------------------------

/// Root element for a worksheet drawing part.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "xdr:wsDr")]
pub struct XlsxWsDr {
    #[serde(rename = "@xmlns", default)]
    pub ns: Option<String>,
    #[serde(rename = "@xmlns:a", default)]
    pub xmlns_a: Option<String>,
    #[serde(rename = "@xmlns:xdr", default)]
    pub xmlns_xdr: Option<String>,
    #[serde(rename = "@xmlns:r", default)]
    pub xmlns_r: Option<String>,
    #[serde(rename = "mc:AlternateContent", default)]
    pub alternate_content: Vec<XlsxAlternateContent>,
    #[serde(rename = "xdr:absoluteAnchor", default)]
    pub absolute_anchor: Vec<XdrCellAnchor>,
    #[serde(rename = "xdr:oneCellAnchor", default)]
    pub one_cell_anchor: Vec<XdrCellAnchor>,
    #[serde(rename = "xdr:twoCellAnchor", default)]
    pub two_cell_anchor: Vec<XdrCellAnchor>,
}

// ------------------------------------------------------------------
// Graphic frame
// ------------------------------------------------------------------

/// Graphic frame element.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "xdr:graphicFrame")]
pub struct XlsxGraphicFrame {
    #[serde(rename = "@macro")]
    pub macro_name: String,
    #[serde(rename = "xdr:nvGraphicFramePr", alias = "nvGraphicFramePr")]
    pub nv_graphic_frame_pr: XlsxNvGraphicFramePr,
    #[serde(rename = "xdr:xfrm", alias = "xfrm")]
    pub xfrm: XlsxXfrm,
    #[serde(rename = "a:graphic", default, alias = "graphic")]
    pub graphic: Option<XlsxGraphic>,
}

/// Non-visual properties for a graphic frame.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XlsxNvGraphicFramePr {
    #[serde(rename = "xdr:cNvPr", default, alias = "cNvPr")]
    pub c_nv_pr: Option<XlsxCNvPr>,
    #[serde(rename = "xdr:cNvGraphicFramePr", alias = "cNvGraphicFramePr")]
    pub c_nv_graphic_frame_pr: String,
}

/// Graphic object.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XlsxGraphic {
    #[serde(rename = "a:graphicData", default, alias = "graphicData")]
    pub graphic_data: Option<XlsxGraphicData>,
}

/// Graphic object data.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XlsxGraphicData {
    #[serde(rename = "@uri")]
    pub uri: String,
    #[serde(rename = "c:chart", default, alias = "chart")]
    pub chart: Option<XlsxChart>,
    #[serde(
        rename = "sle:slicer",
        default,
        alias = "slicer",
        skip_serializing_if = "Option::is_none"
    )]
    pub sle: Option<XlsxSle>,
}

/// Slicer reference.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XlsxSle {
    #[serde(rename = "@xmlns:sle")]
    pub xmlns_sle: String,
    #[serde(rename = "@name")]
    pub name: String,
}

/// Chart reference.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XlsxChart {
    #[serde(rename = "@xmlns:c")]
    pub xmlns_c: String,
    #[serde(rename = "@r:id", alias = "@id")]
    pub r_id: String,
    #[serde(rename = "@xmlns:r")]
    pub xmlns_r: String,
}

// ------------------------------------------------------------------
// Shape (xdr:sp)
// ------------------------------------------------------------------

/// Shape element.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "xdr:sp")]
pub struct XdrSp {
    #[serde(rename = "@macro")]
    pub macro_name: String,
    #[serde(rename = "@textlink")]
    pub text_link: String,
    #[serde(rename = "xdr:nvSpPr", default)]
    pub nv_sp_pr: Option<XdrNvSpPr>,
    #[serde(rename = "xdr:spPr", default)]
    pub sp_pr: Option<XlsxSpPr>,
    #[serde(rename = "xdr:style", default)]
    pub style: Option<XdrStyle>,
    #[serde(rename = "xdr:txBody", default)]
    pub tx_body: Option<XdrTxBody>,
}

/// Non-visual properties for a shape.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XdrNvSpPr {
    #[serde(rename = "xdr:cNvPr", default)]
    pub c_nv_pr: Option<XlsxCNvPr>,
    #[serde(rename = "xdr:cNvSpPr", default)]
    pub c_nv_sp_pr: Option<XdrCNvSpPr>,
}

/// Connection non-visual shape properties.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XdrCNvSpPr {
    #[serde(rename = "@txBox")]
    pub tx_box: bool,
}

/// Shape style.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct XdrStyle {
    #[serde(rename = "a:lnRef", default)]
    pub ln_ref: Option<ARef>,
    #[serde(rename = "a:fillRef", default)]
    pub fill_ref: Option<ARef>,
    #[serde(rename = "a:effectRef", default)]
    pub effect_ref: Option<ARef>,
    #[serde(rename = "a:fontRef", default)]
    pub font_ref: Option<AFontRef>,
}

/// Reference for line/fill/effect.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct ARef {
    #[serde(rename = "@idx")]
    pub idx: i64,
    #[serde(rename = "a:scrgbClr", default)]
    pub scrgb_clr: Option<AScrgbClr>,
    #[serde(rename = "a:schemeClr", default)]
    pub scheme_clr: Option<AttrValString>,
    #[serde(rename = "a:srgbClr", default)]
    pub srgb_clr: Option<AttrValString>,
}

/// Font reference.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AFontRef {
    #[serde(rename = "@idx")]
    pub idx: String,
    #[serde(rename = "a:schemeClr", default)]
    pub scheme_clr: Option<AttrValString>,
}

// ------------------------------------------------------------------
// Shape text body
// ------------------------------------------------------------------

/// Shape text body.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct XdrTxBody {
    #[serde(rename = "a:bodyPr", default)]
    pub body_pr: Option<ABodyPr>,
    #[serde(rename = "a:p", default)]
    pub p: Vec<AP>,
}

/// Body properties.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct ABodyPr {
    #[serde(rename = "@anchor", default)]
    pub anchor: Option<String>,
    #[serde(rename = "@anchorCtr")]
    pub anchor_ctr: bool,
    #[serde(rename = "@rot")]
    pub rot: i64,
    #[serde(rename = "@bIns", default)]
    pub b_ins: Option<f64>,
    #[serde(rename = "@compatLnSpc", default)]
    pub compat_ln_spc: Option<bool>,
    #[serde(rename = "@forceAA", default)]
    pub force_aa: Option<bool>,
    #[serde(rename = "@fromWordArt", default)]
    pub from_word_art: Option<bool>,
    #[serde(rename = "@horzOverflow", default)]
    pub horz_overflow: Option<String>,
    #[serde(rename = "@lIns", default)]
    pub l_ins: Option<f64>,
    #[serde(rename = "@numCol", default)]
    pub num_col: Option<i64>,
    #[serde(rename = "@rIns", default)]
    pub r_ins: Option<f64>,
    #[serde(rename = "@rtlCol", default)]
    pub rtl_col: Option<bool>,
    #[serde(rename = "@spcCol", default)]
    pub spc_col: Option<i64>,
    #[serde(rename = "@spcFirstLastPara")]
    pub spc_first_last_para: bool,
    #[serde(rename = "@tIns", default)]
    pub t_ins: Option<f64>,
    #[serde(rename = "@upright", default)]
    pub upright: Option<bool>,
    #[serde(rename = "@vert", default)]
    pub vert: Option<String>,
    #[serde(rename = "@vertOverflow", default)]
    pub vert_overflow: Option<String>,
    #[serde(rename = "@wrap", default)]
    pub wrap: Option<String>,
}

/// Paragraph.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct AP {
    #[serde(rename = "a:pPr", default)]
    pub p_pr: Option<APPr>,
    #[serde(rename = "a:r", default)]
    pub r: Option<AR>,
    #[serde(rename = "a:endParaRPr", default)]
    pub end_para_r_pr: Option<AEndParaRPr>,
}

/// Paragraph properties.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct APPr {
    #[serde(rename = "a:defRPr")]
    pub def_r_pr: ARPr,
}

/// Run.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct AR {
    #[serde(rename = "a:rPr", default)]
    pub r_pr: Option<ARPr>,
    #[serde(rename = "a:t", default)]
    pub t: Option<String>,
}

/// Run properties.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct ARPr {
    #[serde(rename = "@altLang", default)]
    pub alt_lang: Option<String>,
    #[serde(rename = "@b")]
    pub b: bool,
    #[serde(rename = "@baseline")]
    pub baseline: i64,
    #[serde(rename = "@bmk", default)]
    pub bmk: Option<String>,
    #[serde(rename = "@cap", default)]
    pub cap: Option<String>,
    #[serde(rename = "@dirty", default)]
    pub dirty: Option<bool>,
    #[serde(rename = "@err", default)]
    pub err: Option<bool>,
    #[serde(rename = "@i")]
    pub i: bool,
    #[serde(rename = "@kern")]
    pub kern: i64,
    #[serde(rename = "@kumimoji", default)]
    pub kumimoji: Option<bool>,
    #[serde(rename = "@lang", default)]
    pub lang: Option<String>,
    #[serde(rename = "@noProof", default)]
    pub no_proof: Option<bool>,
    #[serde(rename = "@normalizeH", default)]
    pub normalize_h: Option<bool>,
    #[serde(rename = "@smtClean", default)]
    pub smt_clean: Option<bool>,
    #[serde(rename = "@smtId", default)]
    pub smt_id: Option<u64>,
    #[serde(rename = "@spc")]
    pub spc: i64,
    #[serde(rename = "@strike", default)]
    pub strike: Option<String>,
    #[serde(rename = "@sz", default)]
    pub sz: Option<f64>,
    #[serde(rename = "@u", default)]
    pub u: Option<String>,
    #[serde(rename = "a:solidFill", default)]
    pub solid_fill: Option<ASolidFill>,
    #[serde(rename = "a:latin", default)]
    pub latin: Option<XlsxCTTextFont>,
    #[serde(rename = "a:ea", default)]
    pub ea: Option<XlsxCTTextFont>,
    #[serde(rename = "a:cs", default)]
    pub cs: Option<XlsxCTTextFont>,
}

/// End paragraph run properties.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AEndParaRPr {
    #[serde(rename = "@lang")]
    pub lang: String,
    #[serde(rename = "@altLang", default)]
    pub alt_lang: Option<String>,
    #[serde(rename = "@sz", default)]
    pub sz: Option<i64>,
}

/// Text font.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XlsxCTTextFont {
    #[serde(rename = "@typeface")]
    pub typeface: String,
    #[serde(rename = "@panose", default)]
    pub panose: Option<String>,
    #[serde(rename = "@pitchFamily", default)]
    pub pitch_family: Option<String>,
    #[serde(rename = "@Charset", default)]
    pub charset: Option<String>,
}

// ------------------------------------------------------------------
// Public API types
// ------------------------------------------------------------------

/// Picture insert type.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PictureInsertType(pub u8);

impl PictureInsertType {
    pub const PLACE_OVER_CELLS: PictureInsertType = PictureInsertType(0);
    pub const PLACE_IN_CELL: PictureInsertType = PictureInsertType(1);
    pub const IMAGE: PictureInsertType = PictureInsertType(2);
    pub const DISPIMG: PictureInsertType = PictureInsertType(3);
}

/// Line type.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LineType(pub u8);

impl LineType {
    pub const UNSET: LineType = LineType(0);
    pub const SOLID: LineType = LineType(1);
    pub const NONE: LineType = LineType(2);
    pub const AUTOMATIC: LineType = LineType(3);
}

/// Line dash type.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LineDashType(pub u8);

impl LineDashType {
    pub const UNSET: LineDashType = LineDashType(0);
    pub const SOLID: LineDashType = LineDashType(1);
    pub const DOT: LineDashType = LineDashType(2);
    pub const DASH: LineDashType = LineDashType(3);
    pub const LG_DASH: LineDashType = LineDashType(4);
    pub const SASH_DOT: LineDashType = LineDashType(5);
    pub const LG_DASH_DOT: LineDashType = LineDashType(6);
    pub const LG_DASH_DOT_DOT: LineDashType = LineDashType(7);
    pub const SYS_DASH: LineDashType = LineDashType(8);
    pub const SYS_DOT: LineDashType = LineDashType(9);
    pub const SYS_DASH_DOT: LineDashType = LineDashType(10);
    pub const SYS_DASH_DOT_DOT: LineDashType = LineDashType(11);
}

/// Fill settings.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fill {
    #[serde(default)]
    pub r#type: String,
    pub pattern: i64,
    #[serde(default)]
    pub color: Vec<String>,
    pub shading: i64,
    pub transparency: i64,
}

/// Line options.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct LineOptions {
    pub r#type: LineType,
    pub dash: LineDashType,
    pub fill: Fill,
    pub smooth: bool,
    pub width: f64,
}

/// Picture format settings.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct Picture {
    #[serde(default)]
    pub extension: String,
    #[serde(default)]
    pub file: Vec<u8>,
    #[serde(default)]
    pub format: Option<GraphicOptions>,
    pub insert_type: PictureInsertType,
}

/// Graphic options.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraphicOptions {
    #[serde(default)]
    pub alt_text: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub print_object: Option<bool>,
    #[serde(default)]
    pub locked: Option<bool>,
    pub lock_aspect_ratio: bool,
    pub auto_fit: bool,
    pub auto_fit_ignore_aspect: bool,
    pub offset_x: i64,
    pub offset_y: i64,
    pub scale_x: f64,
    pub scale_y: f64,
    #[serde(default)]
    pub hyperlink: String,
    #[serde(default)]
    pub hyperlink_type: String,
    #[serde(default)]
    pub positioning: String,
}

/// Shape format settings.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct Shape {
    #[serde(default)]
    pub cell: String,
    #[serde(default)]
    pub r#type: String,
    #[serde(default)]
    pub macro_name: String,
    pub width: u64,
    pub height: u64,
    pub format: GraphicOptions,
    pub fill: Fill,
    pub line: LineOptions,
    #[serde(default)]
    pub paragraph: Vec<RichTextRun>,
}
