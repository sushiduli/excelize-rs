//! Decode-only drawing part helpers (`xl/drawings/drawingN.xml`).
//!
//! Ported from Go `xmlDecodeDrawing.go`.

use serde::{Deserialize, Serialize};

use super::common::XlsxInnerXml;

/// Directly maps the `oneCellAnchor` (One Cell Anchor Shape Size) and
/// `twoCellAnchor` (Two Cell Anchor Shape Size). This element specifies a two
/// cell anchor placeholder for a group, a shape, or a drawing element. It moves
/// with cells and its extents are in EMU units.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecodeCellAnchor {
    #[serde(rename = "@editAs", default)]
    pub edit_as: Option<String>,
    #[serde(rename = "from", default)]
    pub from: Option<DecodeFrom>,
    #[serde(rename = "to", default)]
    pub to: Option<DecodeTo>,
    #[serde(rename = "ext", default)]
    pub ext: Option<DecodePositiveSize2D>,
    #[serde(rename = "sp", default)]
    pub sp: Option<DecodeSp>,
    #[serde(rename = "pic", default)]
    pub pic: Option<DecodePic>,
    #[serde(rename = "graphicFrame", default)]
    pub graphic_frame: Option<super::drawing::XlsxGraphicFrame>,
    #[serde(rename = "clientData", default)]
    pub client_data: Option<DecodeClientData>,
    #[serde(rename = "AlternateContent", default)]
    pub alternate_content: Vec<XlsxAlternateContent>,
}

/// Defines the structure used to deserialize the cell anchor for adjust drawing
/// object on inserting/deleting column/rows.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecodeCellAnchorPos {
    #[serde(rename = "@editAs", default)]
    pub edit_as: Option<String>,
    #[serde(rename = "from", default)]
    pub from: Option<XlsxFrom>,
    #[serde(rename = "to", default)]
    pub to: Option<XlsxTo>,
    #[serde(rename = "pos", default)]
    pub pos: Option<XlsxInnerXml>,
    #[serde(rename = "ext", default)]
    pub ext: Option<XlsxPositiveSize2D>,
    #[serde(rename = "sp", default)]
    pub sp: Option<XlsxSp>,
    #[serde(rename = "grpSp", default)]
    pub grp_sp: Option<XlsxInnerXml>,
    #[serde(rename = "graphicFrame", default)]
    pub graphic_frame: Option<XlsxInnerXml>,
    #[serde(rename = "cxnSp", default)]
    pub cxn_sp: Option<XlsxInnerXml>,
    #[serde(rename = "pic", default)]
    pub pic: Option<XlsxInnerXml>,
    #[serde(rename = "contentPart", default)]
    pub content_part: Option<XlsxInnerXml>,
    #[serde(rename = "AlternateContent", default)]
    pub alternate_content: Vec<XlsxAlternateContent>,
    #[serde(rename = "clientData", default)]
    pub client_data: Option<XlsxInnerXml>,
}

/// Defines the structure used to deserialize the `mc:Choice` element.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "Choice")]
pub struct DecodeChoice {
    #[serde(rename = "@a14", default)]
    pub xmlns_a14: Option<String>,
    #[serde(rename = "@sle15", default)]
    pub xmlns_sle15: Option<String>,
    #[serde(rename = "@Requires", default)]
    pub requires: Option<String>,
    #[serde(rename = "graphicFrame", default)]
    pub graphic_frame: DecodeGraphicFrame,
}

/// Defines the structure used to deserialize the `xdr:graphicFrame` element.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "graphicFrame")]
pub struct DecodeGraphicFrame {
    #[serde(rename = "@macro", default)]
    pub macro_name: String,
    #[serde(rename = "nvGraphicFramePr", default)]
    pub nv_graphic_frame_pr: DecodeNvGraphicFramePr,
}

/// Defines the structure used to deserialize the `xdr:nvGraphicFramePr` element.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "nvGraphicFramePr")]
pub struct DecodeNvGraphicFramePr {
    #[serde(rename = "cNvPr", default)]
    pub c_nv_pr: DecodeCNvPr,
}

/// Defines the structure used to deserialize the `sp` element.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "sp")]
pub struct DecodeSp {
    #[serde(rename = "@macro", default)]
    pub macro_name: Option<String>,
    #[serde(rename = "@textlink", default)]
    pub text_link: Option<String>,
    #[serde(rename = "@fLocksText", default)]
    pub f_locks_text: bool,
    #[serde(rename = "@fPublished", default)]
    pub f_published: Option<bool>,
    #[serde(rename = "nvSpPr", default)]
    pub nv_sp_pr: Option<DecodeNvSpPr>,
    #[serde(rename = "spPr", default)]
    pub sp_pr: Option<DecodeSpPr>,
}

/// Non-Visual Properties for a Shape. This element specifies all non-visual
/// properties for a shape.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "nvSpPr")]
pub struct DecodeNvSpPr {
    #[serde(rename = "cNvPr", default)]
    pub c_nv_pr: Option<DecodeCNvPr>,
    #[serde(rename = "extLst", default)]
    pub ext_lst: Option<DecodePositiveSize2D>,
    #[serde(rename = "cNvSpPr", default)]
    pub c_nv_sp_pr: Option<DecodeCNvSpPr>,
}

/// Connection Non-Visual Shape Properties. This element specifies the set of
/// non-visual properties for a connection shape.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "cNvSpPr")]
pub struct DecodeCNvSpPr {
    #[serde(rename = "@txBox", default)]
    pub tx_box: bool,
}

/// Directly maps the root element for a part of this content type shall `wsDr`.
/// In order to solve the problem that the label structure is changed after
/// serialization and deserialization, two different structures are defined.
/// `DecodeWsDr` is just for deserialization.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "xdr:wsDr")]
pub struct DecodeWsDr {
    #[serde(rename = "@xmlns:a", default)]
    pub a: String,
    #[serde(rename = "@xmlns:xdr", default)]
    pub xdr: String,
    #[serde(rename = "@xmlns:r", default)]
    pub r: String,
    #[serde(rename = "AlternateContent", default)]
    pub alternate_content: Vec<XlsxInnerXml>,
    #[serde(rename = "oneCellAnchor", default)]
    pub one_cell_anchor: Vec<DecodeCellAnchor>,
    #[serde(rename = "twoCellAnchor", default)]
    pub two_cell_anchor: Vec<DecodeCellAnchor>,
}

/// Directly maps the `cNvPr` (Non-Visual Drawing Properties). This element
/// specifies non-visual canvas properties.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "cNvPr")]
pub struct DecodeCNvPr {
    #[serde(rename = "@id", default)]
    pub id: i32,
    #[serde(rename = "@name", default)]
    pub name: String,
    #[serde(rename = "@descr", default)]
    pub descr: String,
    #[serde(rename = "@title", default)]
    pub title: Option<String>,
    #[serde(rename = "hlinkClick", default)]
    pub hlink_click: Option<DecodeHlinkClick>,
}

/// Directly maps the `hlinkClick` (Hyperlink Click). This element specifies the
/// on-click hyperlink information to be applied to a run of text. When the
/// hyperlink text is clicked the link is fetched.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "hlinkClick")]
pub struct DecodeHlinkClick {
    #[serde(rename = "@id", default)]
    pub id: Option<String>,
    #[serde(rename = "@invalidUrl", default)]
    pub invalid_url: Option<String>,
    #[serde(rename = "@action", default)]
    pub action: Option<String>,
    #[serde(rename = "@tgtFrame", default)]
    pub tgt_frame: Option<String>,
    #[serde(rename = "@tooltip", default)]
    pub tooltip: Option<String>,
    #[serde(rename = "@history", default)]
    pub history: bool,
    #[serde(rename = "@highlightClick", default)]
    pub highlight_click: bool,
    #[serde(rename = "@endSnd", default)]
    pub end_snd: bool,
}

/// Directly maps the `picLocks` (Picture Locks). This element specifies all
/// locking properties for a graphic frame.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "picLocks")]
pub struct DecodePicLocks {
    #[serde(rename = "@noAdjustHandles", default)]
    pub no_adjust_handles: bool,
    #[serde(rename = "@noChangeArrowheads", default)]
    pub no_change_arrowheads: bool,
    #[serde(rename = "@noChangeAspect", default)]
    pub no_change_aspect: bool,
    #[serde(rename = "@noChangeShapeType", default)]
    pub no_change_shape_type: bool,
    #[serde(rename = "@noCrop", default)]
    pub no_crop: bool,
    #[serde(rename = "@noEditPoints", default)]
    pub no_edit_points: bool,
    #[serde(rename = "@noGrp", default)]
    pub no_grp: bool,
    #[serde(rename = "@noMove", default)]
    pub no_move: bool,
    #[serde(rename = "@noResize", default)]
    pub no_resize: bool,
    #[serde(rename = "@noRot", default)]
    pub no_rot: bool,
    #[serde(rename = "@noSelect", default)]
    pub no_select: bool,
}

/// Specifies the existence of an image (binary large image or picture) and
/// contains a reference to the image data.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "blip")]
pub struct DecodeBlip {
    #[serde(rename = "@embed", default)]
    pub embed: String,
    #[serde(rename = "@cstate", default)]
    pub cstate: Option<String>,
    #[serde(rename = "@r", default)]
    pub r: String,
}

/// Directly maps the `stretch` element. This element specifies that a BLIP
/// should be stretched to fill the target rectangle.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "stretch")]
pub struct DecodeStretch {
    #[serde(rename = "fillRect", default)]
    pub fill_rect: String,
}

/// Directly maps the `colOff` and `rowOff` element. This element is used to
/// specify the column offset within a cell.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "off")]
pub struct DecodeOff {
    #[serde(rename = "@x", default)]
    pub x: i64,
    #[serde(rename = "@y", default)]
    pub y: i64,
}

/// Directly maps the `a:ext` element.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "ext")]
pub struct DecodePositiveSize2D {
    #[serde(rename = "@cx", default)]
    pub cx: i64,
    #[serde(rename = "@cy", default)]
    pub cy: i64,
}

/// Directly maps the `prstGeom` (Preset geometry). This element specifies when
/// a preset geometric shape should be used instead of a custom geometric shape.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "prstGeom")]
pub struct DecodePrstGeom {
    #[serde(rename = "@prst", default)]
    pub prst: String,
}

/// Directly maps the `xfrm` (2D Transform for Graphic Frame). This element
/// specifies the transform to be applied to the corresponding graphic frame.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "xfrm")]
pub struct DecodeXfrm {
    #[serde(rename = "off", default)]
    pub off: DecodeOff,
    #[serde(rename = "ext", default)]
    pub ext: DecodePositiveSize2D,
}

/// Directly maps the `cNvPicPr` (Non-Visual Picture Drawing Properties). This
/// element specifies the non-visual properties for the picture canvas.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "cNvPicPr")]
pub struct DecodeCNvPicPr {
    #[serde(rename = "picLocks", default)]
    pub pic_locks: DecodePicLocks,
}

/// Directly maps the `nvPicPr` (Non-Visual Properties for a Picture). This
/// element specifies all non-visual properties for a picture.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "nvPicPr")]
pub struct DecodeNvPicPr {
    #[serde(rename = "cNvPr", default)]
    pub c_nv_pr: DecodeCNvPr,
    #[serde(rename = "cNvPicPr", default)]
    pub c_nv_pic_pr: DecodeCNvPicPr,
}

/// Directly maps the `blipFill` (Picture Fill). This element specifies the kind
/// of picture fill that the picture object has.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "blipFill")]
pub struct DecodeBlipFill {
    #[serde(rename = "blip", default)]
    pub blip: DecodeBlip,
    #[serde(rename = "stretch", default)]
    pub stretch: DecodeStretch,
}

/// Directly maps the `spPr` (Shape Properties). This element specifies the
/// visual shape properties that can be applied to a picture.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "spPr")]
pub struct DecodeSpPr {
    #[serde(rename = "xfrm", default)]
    pub xfrm: DecodeXfrm,
    #[serde(rename = "prstGeom", default)]
    pub prst_geom: DecodePrstGeom,
}

/// Encompass the definition of pictures within the DrawingML framework. While
/// pictures are in many ways very similar to shapes they have specific
/// properties that are unique in order to optimize for picture-specific
/// scenarios.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "pic")]
pub struct DecodePic {
    #[serde(rename = "nvPicPr", default)]
    pub nv_pic_pr: DecodeNvPicPr,
    #[serde(rename = "blipFill", default)]
    pub blip_fill: DecodeBlipFill,
    #[serde(rename = "spPr", default)]
    pub sp_pr: DecodeSpPr,
}

/// Specifies the starting anchor.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "from")]
pub struct DecodeFrom {
    #[serde(rename = "col", default)]
    pub col: i32,
    #[serde(rename = "colOff", default)]
    pub col_off: i64,
    #[serde(rename = "row", default)]
    pub row: i32,
    #[serde(rename = "rowOff", default)]
    pub row_off: i64,
}

/// Directly specifies the ending anchor.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "to")]
pub struct DecodeTo {
    #[serde(rename = "col", default)]
    pub col: i32,
    #[serde(rename = "colOff", default)]
    pub col_off: i64,
    #[serde(rename = "row", default)]
    pub row: i32,
    #[serde(rename = "rowOff", default)]
    pub row_off: i64,
}

/// Directly maps the `clientData` element. An empty element which specifies
/// (via attributes) certain properties related to printing and selection of the
/// drawing object.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "clientData")]
pub struct DecodeClientData {
    #[serde(rename = "@fLocksWithSheet", default)]
    pub f_locks_with_sheet: bool,
    #[serde(rename = "@fPrintsWithSheet", default)]
    pub f_prints_with_sheet: bool,
}

/// Directly maps the Kingsoft WPS Office embedded cell images.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "cellImages")]
pub struct DecodeCellImages {
    #[serde(rename = "cellImage", default)]
    pub cell_image: Vec<DecodeCellImage>,
}

/// Defines the structure used to deserialize the Kingsoft WPS Office embedded
/// cell images.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "cellImage")]
pub struct DecodeCellImage {
    #[serde(rename = "pic", default)]
    pub pic: DecodePic,
}

// ------------------------------------------------------------------
// Types referenced from `xmlDrawing.go` / `xmlWorkbook.go` that are used by
// `DecodeCellAnchorPos`. They are duplicated here so this module stays
// self-contained until the corresponding modules are ported.
// ------------------------------------------------------------------

/// Container for a sequence of multiple representations of a given piece of
/// content. The program reading the file should only process one of these, and
/// the one chosen should be based on which conditions match.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "AlternateContent")]
pub struct XlsxAlternateContent {
    #[serde(rename = "@xmlns:mc", default)]
    pub xmlns_mc: Option<String>,
    #[serde(rename = "$value", default)]
    pub content: XlsxInnerXml,
}

/// Specifies the starting anchor (drawing variant).
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "from")]
pub struct XlsxFrom {
    #[serde(rename = "col", default)]
    pub col: i32,
    #[serde(rename = "colOff", default)]
    pub col_off: i64,
    #[serde(rename = "row", default)]
    pub row: i32,
    #[serde(rename = "rowOff", default)]
    pub row_off: i64,
}

/// Directly specifies the ending anchor (drawing variant).
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "to")]
pub struct XlsxTo {
    #[serde(rename = "col", default)]
    pub col: i32,
    #[serde(rename = "colOff", default)]
    pub col_off: i64,
    #[serde(rename = "row", default)]
    pub row: i32,
    #[serde(rename = "rowOff", default)]
    pub row_off: i64,
}

/// Directly maps the `a:ext` element (drawing variant).
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "ext")]
pub struct XlsxPositiveSize2D {
    #[serde(rename = "@cx", default)]
    pub cx: i64,
    #[serde(rename = "@cy", default)]
    pub cy: i64,
}

/// Shape. This element specifies the existence of a single shape.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "sp")]
pub struct XlsxSp {
    #[serde(rename = "@macro", default)]
    pub macro_name: Option<String>,
    #[serde(rename = "@textlink", default)]
    pub text_link: Option<String>,
    #[serde(rename = "@fLocksText", default)]
    pub f_locks_text: bool,
    #[serde(rename = "@fPublished", default)]
    pub f_published: Option<bool>,
    #[serde(rename = "$value", default)]
    pub content: XlsxInnerXml,
}
