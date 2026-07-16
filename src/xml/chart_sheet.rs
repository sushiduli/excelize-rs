//! Chartsheet part (`chartsheets/sheet*.xml`).
//!
//! Ported from Go `xmlChartSheet.go`.

use serde::{Deserialize, Serialize};

use super::common::{XlsxColor, XlsxExtLst, XlsxInnerXml};

/// Directly maps the chartsheet element of Chartsheet Parts in a
/// SpreadsheetML document.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "chartsheet")]
pub struct XlsxChartsheet {
    #[serde(rename = "@xmlns", default)]
    pub xmlns: Option<String>,
    #[serde(rename = "sheetPr", default)]
    pub sheet_pr: Option<XlsxChartsheetPr>,
    #[serde(rename = "sheetViews", default)]
    pub sheet_views: Option<XlsxChartsheetViews>,
    #[serde(rename = "sheetProtection", default)]
    pub sheet_protection: Option<XlsxChartsheetProtection>,
    #[serde(rename = "customSheetViews", default)]
    pub custom_sheet_views: Option<XlsxCustomChartsheetViews>,
    #[serde(rename = "pageMargins", default)]
    pub page_margins: Option<XlsxPageMargins>,
    #[serde(rename = "pageSetup", default)]
    pub page_setup: Option<XlsxPageSetUp>,
    #[serde(rename = "headerFooter", default)]
    pub header_footer: Option<XlsxHeaderFooter>,
    #[serde(rename = "drawing", default)]
    pub drawing: Option<XlsxDrawing>,
    #[serde(rename = "drawingHF", default)]
    pub drawing_hf: Option<XlsxDrawingHF>,
    #[serde(rename = "picture", default)]
    pub picture: Option<XlsxPicture>,
    #[serde(rename = "webPublishItems", default)]
    pub web_publish_items: Option<XlsxInnerXml>,
    #[serde(rename = "extLst", default)]
    pub ext_lst: Option<XlsxExtLst>,
}

/// Specifies chart sheet properties.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "sheetPr")]
pub struct XlsxChartsheetPr {
    #[serde(rename = "@published", default)]
    pub published: Option<bool>,
    #[serde(rename = "@codeName", default)]
    pub code_name: Option<String>,
    #[serde(rename = "tabColor", default)]
    pub tab_color: Option<XlsxColor>,
}

/// Specifies chart sheet views.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "sheetViews")]
pub struct XlsxChartsheetViews {
    #[serde(rename = "sheetView", default)]
    pub sheet_view: Vec<XlsxChartsheetView>,
    #[serde(rename = "extLst", default)]
    pub ext_lst: Vec<XlsxExtLst>,
}

/// Defines custom view properties for chart sheets.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "sheetView")]
pub struct XlsxChartsheetView {
    #[serde(rename = "@tabSelected", default)]
    pub tab_selected: Option<bool>,
    #[serde(rename = "@zoomScale", default)]
    pub zoom_scale: Option<u32>,
    #[serde(rename = "@workbookViewId")]
    pub workbook_view_id: u32,
    #[serde(rename = "@zoomToFit", default)]
    pub zoom_to_fit: Option<bool>,
    #[serde(rename = "extLst", default)]
    pub ext_lst: Vec<XlsxExtLst>,
}

/// Expresses the chart sheet protection options to enforce when the chart
/// sheet is protected.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "sheetProtection")]
pub struct XlsxChartsheetProtection {
    #[serde(rename = "@algorithmName", default)]
    pub algorithm_name: Option<String>,
    #[serde(rename = "@hashValue", default)]
    pub hash_value: Option<String>,
    #[serde(rename = "@saltValue", default)]
    pub salt_value: Option<String>,
    #[serde(rename = "@spinCount", default)]
    pub spin_count: Option<u32>,
    #[serde(rename = "@content", default)]
    pub content: Option<bool>,
    #[serde(rename = "@objects", default)]
    pub objects: Option<bool>,
}

/// Collection of custom Chart Sheet View information.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "customSheetViews")]
pub struct XlsxCustomChartsheetViews {
    #[serde(rename = "customSheetView", default)]
    pub custom_sheet_view: Vec<XlsxCustomChartsheetView>,
}

/// Defines custom view properties for chart sheets.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "customSheetView")]
pub struct XlsxCustomChartsheetView {
    #[serde(rename = "@guid")]
    pub guid: String,
    #[serde(rename = "@scale", default)]
    pub scale: Option<u32>,
    #[serde(rename = "@state", default)]
    pub state: Option<String>,
    #[serde(rename = "@zoomToFit", default)]
    pub zoom_to_fit: Option<bool>,
    #[serde(rename = "pageMargins", default)]
    pub page_margins: Vec<XlsxPageMargins>,
    #[serde(rename = "pageSetup", default)]
    pub page_setup: Vec<XlsxPageSetUp>,
    #[serde(rename = "headerFooter", default)]
    pub header_footer: Vec<XlsxHeaderFooter>,
}

/// Page margins for a sheet or a custom sheet view.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "pageMargins")]
pub struct XlsxPageMargins {
    #[serde(rename = "@left")]
    pub left: f64,
    #[serde(rename = "@right")]
    pub right: f64,
    #[serde(rename = "@top")]
    pub top: f64,
    #[serde(rename = "@bottom")]
    pub bottom: f64,
    #[serde(rename = "@header")]
    pub header: f64,
    #[serde(rename = "@footer")]
    pub footer: f64,
}

/// Page setup settings for the worksheet.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "pageSetup")]
pub struct XlsxPageSetUp {
    #[serde(rename = "@blackAndWhite", default)]
    pub black_and_white: Option<bool>,
    #[serde(rename = "@cellComments", default)]
    pub cell_comments: Option<String>,
    #[serde(rename = "@copies", default)]
    pub copies: Option<i64>,
    #[serde(rename = "@draft", default)]
    pub draft: Option<bool>,
    #[serde(rename = "@errors", default)]
    pub errors: Option<String>,
    #[serde(rename = "@firstPageNumber", default)]
    pub first_page_number: Option<String>,
    #[serde(rename = "@fitToHeight", default)]
    pub fit_to_height: Option<i64>,
    #[serde(rename = "@fitToWidth", default)]
    pub fit_to_width: Option<i64>,
    #[serde(rename = "@horizontalDpi", default)]
    pub horizontal_dpi: Option<String>,
    #[serde(rename = "@r:id", default)]
    pub r_id: Option<String>,
    #[serde(rename = "@orientation", default)]
    pub orientation: Option<String>,
    #[serde(rename = "@pageOrder", default)]
    pub page_order: Option<String>,
    #[serde(rename = "@paperHeight", default)]
    pub paper_height: Option<String>,
    #[serde(rename = "@paperSize", default)]
    pub paper_size: Option<i64>,
    #[serde(rename = "@paperWidth", default)]
    pub paper_width: Option<String>,
    #[serde(rename = "@scale", default)]
    pub scale: Option<i64>,
    #[serde(rename = "@useFirstPageNumber", default)]
    pub use_first_page_number: Option<bool>,
    #[serde(rename = "@usePrinterDefaults", default)]
    pub use_printer_defaults: Option<bool>,
    #[serde(rename = "@verticalDpi", default)]
    pub vertical_dpi: Option<String>,
}

/// When printed or viewed in page layout view, each page of a worksheet can
/// have a page header, a page footer, or both.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "headerFooter")]
pub struct XlsxHeaderFooter {
    #[serde(rename = "@differentOddEven", default)]
    pub different_odd_even: Option<bool>,
    #[serde(rename = "@differentFirst", default)]
    pub different_first: Option<bool>,
    #[serde(rename = "@scaleWithDoc", default)]
    pub scale_with_doc: Option<bool>,
    #[serde(rename = "@alignWithMargins", default)]
    pub align_with_margins: Option<bool>,
    #[serde(rename = "oddHeader", default)]
    pub odd_header: Option<String>,
    #[serde(rename = "oddFooter", default)]
    pub odd_footer: Option<String>,
    #[serde(rename = "evenHeader", default)]
    pub even_header: Option<String>,
    #[serde(rename = "evenFooter", default)]
    pub even_footer: Option<String>,
    #[serde(rename = "firstHeader", default)]
    pub first_header: Option<String>,
    #[serde(rename = "firstFooter", default)]
    pub first_footer: Option<String>,
}

/// Drawing reference for a chartsheet.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "drawing")]
pub struct XlsxDrawing {
    #[serde(rename = "@r:id", default)]
    pub r_id: Option<String>,
}

/// Drawing reference in header/footer.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "drawingHF")]
pub struct XlsxDrawingHF {
    #[serde(flatten, default)]
    pub inner: XlsxInnerXml,
}

/// Picture reference for a chartsheet.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "picture")]
pub struct XlsxPicture {
    #[serde(rename = "@r:id", default)]
    pub r_id: Option<String>,
}
