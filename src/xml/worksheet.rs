//! Worksheet part (`xl/worksheets/sheet*.xml`).
//!
//! Ported from Go `xmlWorksheet.go`.

use serde::{Deserialize, Serialize};

use super::common::{XlsxColor, XlsxExtLst, XlsxInnerXml, XlsxPhoneticPr};
use super::shared_strings::XlsxSi;
use super::table::XlsxAutoFilter;
use super::workbook::XlsxAlternateContent;

/// Returns `true` when an optional boolean is `None` or `Some(false)`, so that
/// serialized x14 rules omit attributes that default to false.
fn is_false(v: &Option<bool>) -> bool {
    !v.unwrap_or(false)
}

/// Worksheet root element.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "worksheet")]
pub struct XlsxWorksheet {
    #[serde(rename = "sheetPr", default, skip_serializing_if = "Option::is_none")]
    pub sheet_pr: Option<XlsxSheetPr>,
    #[serde(rename = "dimension", default, skip_serializing_if = "Option::is_none")]
    pub dimension: Option<XlsxDimension>,
    #[serde(
        rename = "sheetViews",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub sheet_views: Option<XlsxSheetViews>,
    #[serde(
        rename = "sheetFormatPr",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub sheet_format_pr: Option<XlsxSheetFormatPr>,
    #[serde(rename = "cols", default, skip_serializing_if = "Option::is_none")]
    pub cols: Option<XlsxCols>,
    #[serde(rename = "sheetData", default)]
    pub sheet_data: XlsxSheetData,
    #[serde(
        rename = "sheetCalcPr",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub sheet_calc_pr: Option<XlsxInnerXml>,
    #[serde(
        rename = "sheetProtection",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub sheet_protection: Option<XlsxSheetProtection>,
    #[serde(
        rename = "protectedRanges",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub protected_ranges: Option<XlsxInnerXml>,
    #[serde(rename = "scenarios", default, skip_serializing_if = "Option::is_none")]
    pub scenarios: Option<XlsxInnerXml>,
    #[serde(
        rename = "autoFilter",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub auto_filter: Option<XlsxAutoFilter>,
    #[serde(rename = "sortState", default, skip_serializing_if = "Option::is_none")]
    pub sort_state: Option<XlsxSortState>,
    #[serde(
        rename = "dataConsolidate",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub data_consolidate: Option<XlsxInnerXml>,
    #[serde(
        rename = "customSheetViews",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub custom_sheet_views: Option<XlsxCustomSheetViews>,
    #[serde(
        rename = "mergeCells",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub merge_cells: Option<XlsxMergeCells>,
    #[serde(
        rename = "phoneticPr",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub phonetic_pr: Option<XlsxPhoneticPr>,
    #[serde(rename = "conditionalFormatting", default)]
    pub conditional_formatting: Vec<XlsxConditionalFormatting>,
    #[serde(
        rename = "dataValidations",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub data_validations: Option<XlsxDataValidations>,
    #[serde(
        rename = "hyperlinks",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub hyperlinks: Option<XlsxHyperlinks>,
    #[serde(
        rename = "printOptions",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub print_options: Option<XlsxPrintOptions>,
    #[serde(
        rename = "pageMargins",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub page_margins: Option<XlsxPageMargins>,
    #[serde(rename = "pageSetup", default, skip_serializing_if = "Option::is_none")]
    pub page_setup: Option<XlsxPageSetUp>,
    #[serde(
        rename = "headerFooter",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub header_footer: Option<XlsxHeaderFooter>,
    #[serde(rename = "rowBreaks", default, skip_serializing_if = "Option::is_none")]
    pub row_breaks: Option<XlsxRowBreaks>,
    #[serde(rename = "colBreaks", default, skip_serializing_if = "Option::is_none")]
    pub col_breaks: Option<XlsxColBreaks>,
    #[serde(
        rename = "customProperties",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub custom_properties: Option<XlsxInnerXml>,
    #[serde(
        rename = "cellWatches",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub cell_watches: Option<XlsxInnerXml>,
    #[serde(
        rename = "ignoredErrors",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub ignored_errors: Option<XlsxIgnoredErrors>,
    #[serde(rename = "smartTags", default, skip_serializing_if = "Option::is_none")]
    pub smart_tags: Option<XlsxInnerXml>,
    #[serde(rename = "drawing", default, skip_serializing_if = "Option::is_none")]
    pub drawing: Option<XlsxDrawing>,
    #[serde(
        rename = "legacyDrawing",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub legacy_drawing: Option<XlsxLegacyDrawing>,
    #[serde(
        rename = "legacyDrawingHF",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub legacy_drawing_hf: Option<XlsxLegacyDrawingHF>,
    #[serde(rename = "drawingHF", default, skip_serializing_if = "Option::is_none")]
    pub drawing_hf: Option<XlsxDrawingHF>,
    #[serde(rename = "picture", default, skip_serializing_if = "Option::is_none")]
    pub picture: Option<XlsxPicture>,
    #[serde(
        rename = "oleObjects",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub ole_objects: Option<XlsxInnerXml>,
    #[serde(rename = "controls", default, skip_serializing_if = "Option::is_none")]
    pub controls: Option<XlsxInnerXml>,
    #[serde(
        rename = "webPublishItems",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub web_publish_items: Option<XlsxInnerXml>,
    #[serde(
        rename = "mc:AlternateContent",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub alternate_content: Option<XlsxAlternateContent>,
    #[serde(
        rename = "tableParts",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub table_parts: Option<XlsxTableParts>,
    #[serde(rename = "extLst", default, skip_serializing_if = "Option::is_none")]
    pub ext_lst: Option<XlsxExtLst>,
    #[serde(
        rename = "http://schemas.openxmlformats.org/markup-compatibility/2006 AlternateContent",
        default,
        skip_serializing
    )]
    pub decode_alternate_content: Option<XlsxInnerXml>,
}

/// Drawing reference on a worksheet.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "drawing")]
pub struct XlsxDrawing {
    #[serde(
        rename = "@r:id",
        alias = "@id",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub rid: Option<String>,
}

/// Header and footer settings.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "headerFooter")]
pub struct XlsxHeaderFooter {
    #[serde(
        rename = "@differentOddEven",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub different_odd_even: Option<bool>,
    #[serde(
        rename = "@differentFirst",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub different_first: Option<bool>,
    #[serde(
        rename = "@scaleWithDoc",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub scale_with_doc: Option<bool>,
    #[serde(
        rename = "@alignWithMargins",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub align_with_margins: Option<bool>,
    #[serde(rename = "oddHeader", default, skip_serializing_if = "Option::is_none")]
    pub odd_header: Option<String>,
    #[serde(rename = "oddFooter", default, skip_serializing_if = "Option::is_none")]
    pub odd_footer: Option<String>,
    #[serde(
        rename = "evenHeader",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub even_header: Option<String>,
    #[serde(
        rename = "evenFooter",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub even_footer: Option<String>,
    #[serde(
        rename = "firstHeader",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub first_header: Option<String>,
    #[serde(
        rename = "firstFooter",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub first_footer: Option<String>,
}

/// Drawing reference in header/footer.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XlsxDrawingHF {
    #[serde(rename = "$value", default)]
    pub content: String,
}

/// Page setup settings for the worksheet.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "pageSetup")]
pub struct XlsxPageSetUp {
    #[serde(
        rename = "@blackAndWhite",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub black_and_white: Option<bool>,
    #[serde(
        rename = "@cellComments",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub cell_comments: Option<String>,
    #[serde(rename = "@copies", default, skip_serializing_if = "Option::is_none")]
    pub copies: Option<i64>,
    #[serde(rename = "@draft", default, skip_serializing_if = "Option::is_none")]
    pub draft: Option<bool>,
    #[serde(rename = "@errors", default, skip_serializing_if = "Option::is_none")]
    pub errors: Option<String>,
    #[serde(
        rename = "@firstPageNumber",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub first_page_number: Option<String>,
    #[serde(
        rename = "@fitToHeight",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub fit_to_height: Option<i64>,
    #[serde(
        rename = "@fitToWidth",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub fit_to_width: Option<i64>,
    #[serde(
        rename = "@horizontalDpi",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub horizontal_dpi: Option<String>,
    #[serde(rename = "@r:id", default, skip_serializing_if = "Option::is_none")]
    pub rid: Option<String>,
    #[serde(
        rename = "@orientation",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub orientation: Option<String>,
    #[serde(
        rename = "@pageOrder",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub page_order: Option<String>,
    #[serde(
        rename = "@paperHeight",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub paper_height: Option<String>,
    #[serde(
        rename = "@paperSize",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub paper_size: Option<i64>,
    #[serde(
        rename = "@paperWidth",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub paper_width: Option<String>,
    #[serde(rename = "@scale", default, skip_serializing_if = "Option::is_none")]
    pub scale: Option<i64>,
    #[serde(
        rename = "@useFirstPageNumber",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub use_first_page_number: Option<bool>,
    #[serde(
        rename = "@usePrinterDefaults",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub use_printer_defaults: Option<bool>,
    #[serde(
        rename = "@verticalDpi",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub vertical_dpi: Option<String>,
}

/// Print options for the sheet.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "printOptions")]
pub struct XlsxPrintOptions {
    #[serde(
        rename = "@gridLines",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub grid_lines: Option<bool>,
    #[serde(
        rename = "@gridLinesSet",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub grid_lines_set: Option<bool>,
    #[serde(rename = "@headings", default, skip_serializing_if = "Option::is_none")]
    pub headings: Option<bool>,
    #[serde(
        rename = "@horizontalCentered",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub horizontal_centered: Option<bool>,
    #[serde(
        rename = "@verticalCentered",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub vertical_centered: Option<bool>,
}

/// Page margins for a sheet.
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

/// Sheet formatting properties.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "sheetFormatPr")]
pub struct XlsxSheetFormatPr {
    #[serde(
        rename = "@baseColWidth",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub base_col_width: Option<u8>,
    #[serde(
        rename = "@defaultColWidth",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub default_col_width: Option<f64>,
    #[serde(rename = "@defaultRowHeight")]
    pub default_row_height: f64,
    #[serde(
        rename = "@customHeight",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub custom_height: Option<bool>,
    #[serde(
        rename = "@zeroHeight",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub zero_height: Option<bool>,
    #[serde(rename = "@thickTop", default, skip_serializing_if = "Option::is_none")]
    pub thick_top: Option<bool>,
    #[serde(
        rename = "@thickBottom",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub thick_bottom: Option<bool>,
    #[serde(
        rename = "@outlineLevelRow",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub outline_level_row: Option<u8>,
    #[serde(
        rename = "@outlineLevelCol",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub outline_level_col: Option<u8>,
}

/// Worksheet views collection.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "sheetViews")]
pub struct XlsxSheetViews {
    #[serde(rename = "sheetView", default)]
    pub sheet_view: Vec<XlsxSheetView>,
}

/// A single sheet view definition.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "sheetView")]
pub struct XlsxSheetView {
    #[serde(
        rename = "@windowProtection",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub window_protection: Option<bool>,
    #[serde(
        rename = "@showFormulas",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub show_formulas: Option<bool>,
    #[serde(
        rename = "@showGridLines",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub show_grid_lines: Option<bool>,
    #[serde(
        rename = "@showRowColHeaders",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub show_row_col_headers: Option<bool>,
    #[serde(
        rename = "@showZeros",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub show_zeros: Option<bool>,
    #[serde(
        rename = "@rightToLeft",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub right_to_left: Option<bool>,
    #[serde(
        rename = "@tabSelected",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub tab_selected: Option<bool>,
    #[serde(
        rename = "@showRuler",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub show_ruler: Option<bool>,
    #[serde(
        rename = "@showWhiteSpace",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub show_white_space: Option<bool>,
    #[serde(
        rename = "@showOutlineSymbols",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub show_outline_symbols: Option<bool>,
    #[serde(
        rename = "@defaultGridColor",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub default_grid_color: Option<bool>,
    #[serde(rename = "@view", default, skip_serializing_if = "Option::is_none")]
    pub view: Option<String>,
    #[serde(
        rename = "@topLeftCell",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub top_left_cell: Option<String>,
    #[serde(rename = "@colorId", default, skip_serializing_if = "Option::is_none")]
    pub color_id: Option<i64>,
    #[serde(
        rename = "@zoomScale",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub zoom_scale: Option<f64>,
    #[serde(
        rename = "@zoomScaleNormal",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub zoom_scale_normal: Option<f64>,
    #[serde(
        rename = "@zoomScalePageLayoutView",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub zoom_scale_page_layout_view: Option<f64>,
    #[serde(
        rename = "@zoomScaleSheetLayoutView",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub zoom_scale_sheet_layout_view: Option<f64>,
    #[serde(rename = "@workbookViewId")]
    pub workbook_view_id: i64,
    #[serde(rename = "pane", default, skip_serializing_if = "Option::is_none")]
    pub pane: Option<XlsxPane>,
    #[serde(rename = "selection", default)]
    pub selection: Vec<XlsxSelection>,
}

/// Worksheet view selection.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "selection")]
pub struct XlsxSelection {
    #[serde(
        rename = "@activeCell",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub active_cell: Option<String>,
    #[serde(
        rename = "@activeCellId",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub active_cell_id: Option<i64>,
    #[serde(rename = "@pane", default, skip_serializing_if = "Option::is_none")]
    pub pane: Option<String>,
    #[serde(rename = "@sqref", default, skip_serializing_if = "Option::is_none")]
    pub sqref: Option<String>,
}

/// Worksheet view pane.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "pane")]
pub struct XlsxPane {
    #[serde(
        rename = "@activePane",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub active_pane: Option<String>,
    #[serde(rename = "@state", default, skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    #[serde(
        rename = "@topLeftCell",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub top_left_cell: Option<String>,
    #[serde(rename = "@xSplit", default, skip_serializing_if = "Option::is_none")]
    pub x_split: Option<f64>,
    #[serde(rename = "@ySplit", default, skip_serializing_if = "Option::is_none")]
    pub y_split: Option<f64>,
}

/// Sheet-level properties.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "sheetPr")]
pub struct XlsxSheetPr {
    #[serde(
        rename = "@syncHorizontal",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub sync_horizontal: Option<bool>,
    #[serde(
        rename = "@syncVertical",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub sync_vertical: Option<bool>,
    #[serde(rename = "@syncRef", default, skip_serializing_if = "Option::is_none")]
    pub sync_ref: Option<String>,
    #[serde(
        rename = "@transitionEvaluation",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub transition_evaluation: Option<bool>,
    #[serde(
        rename = "@transitionEntry",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub transition_entry: Option<bool>,
    #[serde(
        rename = "@published",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub published: Option<bool>,
    #[serde(rename = "@codeName", default, skip_serializing_if = "Option::is_none")]
    pub code_name: Option<String>,
    #[serde(
        rename = "@filterMode",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub filter_mode: Option<bool>,
    #[serde(
        rename = "@enableFormatConditionsCalculation",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub enable_format_conditions_calculation: Option<bool>,
    #[serde(rename = "tabColor", default, skip_serializing_if = "Option::is_none")]
    pub tab_color: Option<XlsxColor>,
    #[serde(rename = "outlinePr", default, skip_serializing_if = "Option::is_none")]
    pub outline_pr: Option<XlsxOutlinePr>,
    #[serde(
        rename = "pageSetUpPr",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub page_set_up_pr: Option<XlsxPageSetUpPr>,
}

/// Outline properties.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "outlinePr")]
pub struct XlsxOutlinePr {
    #[serde(
        rename = "@applyStyles",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub apply_styles: Option<bool>,
    #[serde(
        rename = "@summaryBelow",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub summary_below: Option<bool>,
    #[serde(
        rename = "@summaryRight",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub summary_right: Option<bool>,
    #[serde(
        rename = "@showOutlineSymbols",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub show_outline_symbols: Option<bool>,
}

/// Page setup properties of the worksheet.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "pageSetUpPr")]
pub struct XlsxPageSetUpPr {
    #[serde(
        rename = "@autoPageBreaks",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub auto_page_breaks: Option<bool>,
    #[serde(
        rename = "@fitToPage",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub fit_to_page: Option<bool>,
}

/// Column width and formatting definitions.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "cols")]
pub struct XlsxCols {
    #[serde(rename = "col", default)]
    pub col: Vec<XlsxCol>,
}

/// Column width and formatting for one or more columns.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "col")]
pub struct XlsxCol {
    #[serde(rename = "@bestFit", default, skip_serializing_if = "Option::is_none")]
    pub best_fit: Option<bool>,
    #[serde(
        rename = "@collapsed",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub collapsed: Option<bool>,
    #[serde(
        rename = "@customWidth",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub custom_width: Option<bool>,
    #[serde(rename = "@hidden", default, skip_serializing_if = "Option::is_none")]
    pub hidden: Option<bool>,
    #[serde(rename = "@max")]
    pub max: i64,
    #[serde(rename = "@min")]
    pub min: i64,
    #[serde(
        rename = "@outlineLevel",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub outline_level: Option<u8>,
    #[serde(rename = "@phonetic", default, skip_serializing_if = "Option::is_none")]
    pub phonetic: Option<bool>,
    #[serde(rename = "@style", default, skip_serializing_if = "Option::is_none")]
    pub style: Option<i64>,
    #[serde(rename = "@width", default, skip_serializing_if = "Option::is_none")]
    pub width: Option<f64>,
}

/// Used range of the worksheet.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "dimension")]
pub struct XlsxDimension {
    #[serde(rename = "@ref")]
    pub r#ref: String,
}

/// Cell table itself, grouped by rows.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "sheetData")]
pub struct XlsxSheetData {
    #[serde(rename = "row", default)]
    pub row: Vec<XlsxRow>,
}

/// A single row in the worksheet.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "row")]
pub struct XlsxRow {
    #[serde(rename = "c", default)]
    pub c: Vec<XlsxC>,
    #[serde(rename = "@r", default, skip_serializing_if = "Option::is_none")]
    pub r: Option<i64>,
    #[serde(rename = "@spans", default, skip_serializing_if = "Option::is_none")]
    pub spans: Option<String>,
    #[serde(rename = "@s", default, skip_serializing_if = "Option::is_none")]
    pub s: Option<i64>,
    #[serde(
        rename = "@customFormat",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub custom_format: Option<bool>,
    #[serde(rename = "@ht", default, skip_serializing_if = "Option::is_none")]
    pub ht: Option<f64>,
    #[serde(rename = "@hidden", default, skip_serializing_if = "Option::is_none")]
    pub hidden: Option<bool>,
    #[serde(
        rename = "@customHeight",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub custom_height: Option<bool>,
    #[serde(
        rename = "@outlineLevel",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub outline_level: Option<u8>,
    #[serde(
        rename = "@collapsed",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub collapsed: Option<bool>,
    #[serde(rename = "@thickTop", default, skip_serializing_if = "Option::is_none")]
    pub thick_top: Option<bool>,
    #[serde(rename = "@thickBot", default, skip_serializing_if = "Option::is_none")]
    pub thick_bot: Option<bool>,
    #[serde(rename = "@ph", default, skip_serializing_if = "Option::is_none")]
    pub ph: Option<bool>,
}

/// A single cell in the worksheet.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "c")]
pub struct XlsxC {
    #[serde(
        rename = "@xml:space",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub xml_space: Option<String>,
    #[serde(rename = "@r", default, skip_serializing_if = "Option::is_none")]
    pub r: Option<String>,
    #[serde(rename = "@s", default, skip_serializing_if = "Option::is_none")]
    pub s: Option<i64>,
    #[serde(rename = "@t", default, skip_serializing_if = "Option::is_none")]
    pub t: Option<String>,
    #[serde(rename = "@cm", default, skip_serializing_if = "Option::is_none")]
    pub cm: Option<u64>,
    #[serde(rename = "@vm", default, skip_serializing_if = "Option::is_none")]
    pub vm: Option<u64>,
    #[serde(rename = "@ph", default, skip_serializing_if = "Option::is_none")]
    pub ph: Option<bool>,
    #[serde(rename = "f", default, skip_serializing_if = "Option::is_none")]
    pub f: Option<XlsxF>,
    #[serde(rename = "v", default, skip_serializing_if = "Option::is_none")]
    pub v: Option<String>,
    #[serde(rename = "is", default, skip_serializing_if = "Option::is_none")]
    pub is: Option<XlsxSi>,
}

/// Formula for a cell.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "f")]
pub struct XlsxF {
    #[serde(rename = "$value", default)]
    pub content: String,
    #[serde(rename = "@t", default, skip_serializing_if = "Option::is_none")]
    pub t: Option<String>,
    #[serde(rename = "@aca", default, skip_serializing_if = "Option::is_none")]
    pub aca: Option<bool>,
    #[serde(rename = "@ref", default, skip_serializing_if = "Option::is_none")]
    pub r#ref: Option<String>,
    #[serde(rename = "@dt2D", default, skip_serializing_if = "Option::is_none")]
    pub dt2_d: Option<bool>,
    #[serde(rename = "@dtr", default, skip_serializing_if = "Option::is_none")]
    pub dtr: Option<bool>,
    #[serde(rename = "@del1", default, skip_serializing_if = "Option::is_none")]
    pub del1: Option<bool>,
    #[serde(rename = "@del2", default, skip_serializing_if = "Option::is_none")]
    pub del2: Option<bool>,
    #[serde(rename = "@r1", default, skip_serializing_if = "Option::is_none")]
    pub r1: Option<String>,
    #[serde(rename = "@r2", default, skip_serializing_if = "Option::is_none")]
    pub r2: Option<String>,
    #[serde(rename = "@ca", default, skip_serializing_if = "Option::is_none")]
    pub ca: Option<bool>,
    #[serde(rename = "@si", default, skip_serializing_if = "Option::is_none")]
    pub si: Option<i64>,
    #[serde(rename = "@bx", default, skip_serializing_if = "Option::is_none")]
    pub bx: Option<bool>,
}

/// Sheet protection options.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "sheetProtection")]
pub struct XlsxSheetProtection {
    #[serde(
        rename = "@algorithmName",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub algorithm_name: Option<String>,
    #[serde(rename = "@password", default, skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(
        rename = "@hashValue",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub hash_value: Option<String>,
    #[serde(
        rename = "@saltValue",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub salt_value: Option<String>,
    #[serde(
        rename = "@spinCount",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub spin_count: Option<i64>,
    #[serde(rename = "@sheet")]
    pub sheet: bool,
    #[serde(rename = "@objects")]
    pub objects: bool,
    #[serde(rename = "@scenarios")]
    pub scenarios: bool,
    #[serde(rename = "@formatCells")]
    pub format_cells: bool,
    #[serde(rename = "@formatColumns")]
    pub format_columns: bool,
    #[serde(rename = "@formatRows")]
    pub format_rows: bool,
    #[serde(rename = "@insertColumns")]
    pub insert_columns: bool,
    #[serde(rename = "@insertRows")]
    pub insert_rows: bool,
    #[serde(rename = "@insertHyperlinks")]
    pub insert_hyperlinks: bool,
    #[serde(rename = "@deleteColumns")]
    pub delete_columns: bool,
    #[serde(rename = "@deleteRows")]
    pub delete_rows: bool,
    #[serde(rename = "@selectLockedCells")]
    pub select_locked_cells: bool,
    #[serde(rename = "@sort")]
    pub sort: bool,
    #[serde(rename = "@autoFilter")]
    pub auto_filter: bool,
    #[serde(rename = "@pivotTables")]
    pub pivot_tables: bool,
    #[serde(rename = "@selectUnlockedCells")]
    pub select_unlocked_cells: bool,
}

/// Sort state collection.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "sortState")]
pub struct XlsxSortState {
    #[serde(
        rename = "@columnSort",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub column_sort: Option<bool>,
    #[serde(
        rename = "@caseSensitive",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub case_sensitive: Option<bool>,
    #[serde(
        rename = "@sortMethod",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub sort_method: Option<String>,
    #[serde(rename = "@ref")]
    pub r#ref: String,
    #[serde(rename = "$value", default)]
    pub content: String,
}

/// Custom sheet views collection.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "customSheetViews")]
pub struct XlsxCustomSheetViews {
    #[serde(rename = "customSheetView", default)]
    pub custom_sheet_view: Vec<XlsxCustomSheetView>,
}

/// Row or column break for pagination.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "brk")]
pub struct XlsxBrk {
    #[serde(rename = "@id", default, skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    #[serde(rename = "@min", default, skip_serializing_if = "Option::is_none")]
    pub min: Option<i64>,
    #[serde(rename = "@max", default, skip_serializing_if = "Option::is_none")]
    pub max: Option<i64>,
    #[serde(rename = "@man", default, skip_serializing_if = "Option::is_none")]
    pub man: Option<bool>,
    #[serde(rename = "@pt", default, skip_serializing_if = "Option::is_none")]
    pub pt: Option<bool>,
}

/// Collection of row or column breaks.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XlsxBreaks {
    #[serde(rename = "brk", default)]
    pub brk: Vec<XlsxBrk>,
    #[serde(rename = "@count", default, skip_serializing_if = "Option::is_none")]
    pub count: Option<i64>,
    #[serde(
        rename = "@manualBreakCount",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub manual_break_count: Option<i64>,
}

/// Collection of row breaks.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "rowBreaks")]
pub struct XlsxRowBreaks {
    #[serde(flatten)]
    pub breaks: XlsxBreaks,
}

/// Collection of column breaks.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "colBreaks")]
pub struct XlsxColBreaks {
    #[serde(flatten)]
    pub breaks: XlsxBreaks,
}

/// A single custom sheet view.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "customSheetView")]
pub struct XlsxCustomSheetView {
    #[serde(rename = "pane", default, skip_serializing_if = "Option::is_none")]
    pub pane: Option<XlsxPane>,
    #[serde(rename = "selection", default, skip_serializing_if = "Option::is_none")]
    pub selection: Option<XlsxSelection>,
    #[serde(rename = "rowBreaks", default, skip_serializing_if = "Option::is_none")]
    pub row_breaks: Option<XlsxBreaks>,
    #[serde(rename = "colBreaks", default, skip_serializing_if = "Option::is_none")]
    pub col_breaks: Option<XlsxBreaks>,
    #[serde(
        rename = "pageMargins",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub page_margins: Option<XlsxPageMargins>,
    #[serde(
        rename = "printOptions",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub print_options: Option<XlsxPrintOptions>,
    #[serde(rename = "pageSetup", default, skip_serializing_if = "Option::is_none")]
    pub page_setup: Option<XlsxPageSetUp>,
    #[serde(
        rename = "headerFooter",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub header_footer: Option<XlsxHeaderFooter>,
    #[serde(
        rename = "autoFilter",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub auto_filter: Option<XlsxAutoFilter>,
    #[serde(rename = "extLst", default, skip_serializing_if = "Option::is_none")]
    pub ext_lst: Option<XlsxExtLst>,
    #[serde(rename = "@guid")]
    pub guid: String,
    #[serde(rename = "@scale", default, skip_serializing_if = "Option::is_none")]
    pub scale: Option<i64>,
    #[serde(rename = "@colorId", default, skip_serializing_if = "Option::is_none")]
    pub color_id: Option<i64>,
    #[serde(
        rename = "@showPageBreaks",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub show_page_breaks: Option<bool>,
    #[serde(
        rename = "@showFormulas",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub show_formulas: Option<bool>,
    #[serde(
        rename = "@showGridLines",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub show_grid_lines: Option<bool>,
    #[serde(
        rename = "@showRowCol",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub show_row_col: Option<bool>,
    #[serde(
        rename = "@outlineSymbols",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub outline_symbols: Option<bool>,
    #[serde(
        rename = "@zeroValues",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub zero_values: Option<bool>,
    #[serde(
        rename = "@fitToPage",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub fit_to_page: Option<bool>,
    #[serde(
        rename = "@printArea",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub print_area: Option<bool>,
    #[serde(rename = "@filter", default, skip_serializing_if = "Option::is_none")]
    pub filter: Option<bool>,
    #[serde(
        rename = "@showAutoFilter",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub show_auto_filter: Option<bool>,
    #[serde(
        rename = "@hiddenRows",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub hidden_rows: Option<bool>,
    #[serde(
        rename = "@hiddenColumns",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub hidden_columns: Option<bool>,
    #[serde(rename = "@state", default, skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    #[serde(
        rename = "@filterUnique",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub filter_unique: Option<bool>,
    #[serde(rename = "@view", default, skip_serializing_if = "Option::is_none")]
    pub view: Option<String>,
    #[serde(
        rename = "@showRuler",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub show_ruler: Option<bool>,
    #[serde(
        rename = "@topLeftCell",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub top_left_cell: Option<String>,
}

/// A single merged cell.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "mergeCell")]
pub struct XlsxMergeCell {
    #[serde(rename = "@ref", default, skip_serializing_if = "Option::is_none")]
    pub r#ref: Option<String>,
}

/// All merged cells in the sheet.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "mergeCells")]
pub struct XlsxMergeCells {
    #[serde(rename = "@count", default, skip_serializing_if = "Option::is_none")]
    pub count: Option<i64>,
    #[serde(rename = "mergeCell", default)]
    pub cells: Vec<XlsxMergeCell>,
}

/// Data validations collection.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "dataValidations")]
pub struct XlsxDataValidations {
    #[serde(rename = "@count", default, skip_serializing_if = "Option::is_none")]
    pub count: Option<i64>,
    #[serde(
        rename = "@disablePrompts",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub disable_prompts: Option<bool>,
    #[serde(rename = "@xWindow", default, skip_serializing_if = "Option::is_none")]
    pub x_window: Option<i64>,
    #[serde(rename = "@yWindow", default, skip_serializing_if = "Option::is_none")]
    pub y_window: Option<i64>,
    #[serde(rename = "dataValidation", default)]
    pub data_validation: Vec<XlsxDataValidation>,
}

/// Single data validation rule.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "dataValidation")]
pub struct XlsxDataValidation {
    #[serde(
        rename = "@allowBlank",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub allow_blank: Option<bool>,
    #[serde(rename = "@error", default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(
        rename = "@errorStyle",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub error_style: Option<String>,
    #[serde(
        rename = "@errorTitle",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub error_title: Option<String>,
    #[serde(rename = "@operator", default, skip_serializing_if = "Option::is_none")]
    pub operator: Option<String>,
    #[serde(rename = "@prompt", default, skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    #[serde(
        rename = "@promptTitle",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub prompt_title: Option<String>,
    #[serde(rename = "@showDropDown", default, skip_serializing_if = "is_false")]
    pub show_drop_down: Option<bool>,
    #[serde(
        rename = "@showErrorMessage",
        default,
        skip_serializing_if = "is_false"
    )]
    pub show_error_message: Option<bool>,
    #[serde(
        rename = "@showInputMessage",
        default,
        skip_serializing_if = "is_false"
    )]
    pub show_input_message: Option<bool>,
    #[serde(rename = "@sqref")]
    pub sqref: String,
    #[serde(rename = "@type", default, skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(rename = "formula1", default, skip_serializing_if = "Option::is_none")]
    pub formula1: Option<XlsxInnerXml>,
    #[serde(rename = "formula2", default, skip_serializing_if = "Option::is_none")]
    pub formula2: Option<XlsxInnerXml>,
    #[serde(rename = "sqref", default, skip_serializing_if = "Option::is_none")]
    pub xm_sqref: Option<String>,
}

/// Single x14 data validation rule.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "x14:dataValidation")]
pub struct XlsxX14DataValidation {
    #[serde(
        rename = "@allowBlank",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub allow_blank: Option<bool>,
    #[serde(rename = "@error", default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(
        rename = "@errorStyle",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub error_style: Option<String>,
    #[serde(
        rename = "@errorTitle",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub error_title: Option<String>,
    #[serde(rename = "@operator", default, skip_serializing_if = "Option::is_none")]
    pub operator: Option<String>,
    #[serde(rename = "@prompt", default, skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    #[serde(
        rename = "@promptTitle",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub prompt_title: Option<String>,
    #[serde(rename = "@showDropDown", default, skip_serializing_if = "is_false")]
    pub show_drop_down: Option<bool>,
    #[serde(
        rename = "@showErrorMessage",
        default,
        skip_serializing_if = "is_false"
    )]
    pub show_error_message: Option<bool>,
    #[serde(
        rename = "@showInputMessage",
        default,
        skip_serializing_if = "is_false"
    )]
    pub show_input_message: Option<bool>,
    #[serde(rename = "@sqref")]
    pub sqref: String,
    #[serde(rename = "@type", default, skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(
        rename = "x14:formula1",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub formula1: Option<XlsxInnerXml>,
    #[serde(
        rename = "x14:formula2",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub formula2: Option<XlsxInnerXml>,
    #[serde(rename = "xm:sqref", default, skip_serializing_if = "Option::is_none")]
    pub xm_sqref: Option<String>,
}

/// x14 data validations collection.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "x14:dataValidations")]
pub struct XlsxX14DataValidations {
    #[serde(rename = "@xmlns:xm", default, skip_serializing_if = "Option::is_none")]
    pub xmlns_xm: Option<String>,
    #[serde(rename = "@count", default, skip_serializing_if = "Option::is_none")]
    pub count: Option<i64>,
    #[serde(
        rename = "@disablePrompts",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub disable_prompts: Option<bool>,
    #[serde(rename = "@xWindow", default, skip_serializing_if = "Option::is_none")]
    pub x_window: Option<i64>,
    #[serde(rename = "@yWindow", default, skip_serializing_if = "Option::is_none")]
    pub y_window: Option<i64>,
    #[serde(rename = "x14:dataValidation", default)]
    pub data_validation: Vec<XlsxX14DataValidation>,
}

/// Conditional formatting rules applied to a range.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "conditionalFormatting")]
pub struct XlsxConditionalFormatting {
    #[serde(rename = "@pivot", default, skip_serializing_if = "Option::is_none")]
    pub pivot: Option<bool>,
    #[serde(rename = "@sqref", default, skip_serializing_if = "Option::is_none")]
    pub sqref: Option<String>,
    #[serde(rename = "cfRule", default)]
    pub cf_rule: Vec<XlsxCfRule>,
}

/// Conditional formatting rule.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "cfRule")]
pub struct XlsxCfRule {
    #[serde(rename = "@type", default, skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(rename = "@dxfId", default, skip_serializing_if = "Option::is_none")]
    pub dxf_id: Option<i64>,
    #[serde(rename = "@priority", default, skip_serializing_if = "Option::is_none")]
    pub priority: Option<i64>,
    #[serde(
        rename = "@stopIfTrue",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub stop_if_true: Option<bool>,
    #[serde(
        rename = "@aboveAverage",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub above_average: Option<bool>,
    #[serde(rename = "@percent", default, skip_serializing_if = "Option::is_none")]
    pub percent: Option<bool>,
    #[serde(rename = "@bottom", default, skip_serializing_if = "Option::is_none")]
    pub bottom: Option<bool>,
    #[serde(rename = "@operator", default, skip_serializing_if = "Option::is_none")]
    pub operator: Option<String>,
    #[serde(rename = "@text", default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(
        rename = "@timePeriod",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub time_period: Option<String>,
    #[serde(rename = "@rank", default, skip_serializing_if = "Option::is_none")]
    pub rank: Option<i64>,
    #[serde(rename = "@stdDev", default, skip_serializing_if = "Option::is_none")]
    pub std_dev: Option<i64>,
    #[serde(
        rename = "@equalAverage",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub equal_average: Option<bool>,
    #[serde(rename = "formula", default)]
    pub formula: Vec<String>,
    #[serde(
        rename = "colorScale",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub color_scale: Option<XlsxColorScale>,
    #[serde(rename = "dataBar", default, skip_serializing_if = "Option::is_none")]
    pub data_bar: Option<XlsxDataBar>,
    #[serde(rename = "iconSet", default, skip_serializing_if = "Option::is_none")]
    pub icon_set: Option<XlsxIconSet>,
    #[serde(rename = "extLst", default, skip_serializing_if = "Option::is_none")]
    pub ext_lst: Option<XlsxExtLst>,
}

/// Gradated color scale in a conditional formatting rule.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "colorScale")]
pub struct XlsxColorScale {
    #[serde(rename = "cfvo", default)]
    pub cfvo: Vec<XlsxCfvo>,
    #[serde(rename = "color", default)]
    pub color: Vec<XlsxColor>,
}

/// Data bar conditional formatting rule.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "dataBar")]
pub struct XlsxDataBar {
    #[serde(
        rename = "@maxLength",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub max_length: Option<i64>,
    #[serde(
        rename = "@minLength",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub min_length: Option<i64>,
    #[serde(
        rename = "@showValue",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub show_value: Option<bool>,
    #[serde(rename = "cfvo", default)]
    pub cfvo: Vec<XlsxCfvo>,
    #[serde(rename = "color", default)]
    pub color: Vec<XlsxColor>,
}

/// Icon set conditional formatting rule.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "iconSet")]
pub struct XlsxIconSet {
    #[serde(rename = "cfvo", default)]
    pub cfvo: Vec<XlsxCfvo>,
    #[serde(rename = "@iconSet", default, skip_serializing_if = "Option::is_none")]
    pub icon_set: Option<String>,
    #[serde(
        rename = "@showValue",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub show_value: Option<bool>,
    #[serde(rename = "@percent", default, skip_serializing_if = "Option::is_none")]
    pub percent: Option<bool>,
    #[serde(rename = "@reverse", default, skip_serializing_if = "Option::is_none")]
    pub reverse: Option<bool>,
}

/// Conditional format value object.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "cfvo")]
pub struct XlsxCfvo {
    #[serde(rename = "@gte", default, skip_serializing_if = "Option::is_none")]
    pub gte: Option<bool>,
    #[serde(rename = "@type", default, skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(rename = "@val", default, skip_serializing_if = "Option::is_none")]
    pub val: Option<String>,
    #[serde(rename = "extLst", default, skip_serializing_if = "Option::is_none")]
    pub ext_lst: Option<XlsxExtLst>,
}

/// Hyperlinks collection.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "hyperlinks")]
pub struct XlsxHyperlinks {
    #[serde(rename = "hyperlink", default)]
    pub hyperlink: Vec<XlsxHyperlink>,
}

/// A single hyperlink.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "hyperlink")]
pub struct XlsxHyperlink {
    #[serde(rename = "@ref")]
    pub r#ref: String,
    #[serde(rename = "@location", default, skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    #[serde(rename = "@display", default, skip_serializing_if = "Option::is_none")]
    pub display: Option<String>,
    #[serde(rename = "@tooltip", default, skip_serializing_if = "Option::is_none")]
    pub tooltip: Option<String>,
    #[serde(rename = "@r:id", default, skip_serializing_if = "Option::is_none")]
    pub rid: Option<String>,
}

/// Table part references in the worksheet.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "tableParts")]
pub struct XlsxTableParts {
    #[serde(rename = "@count", default, skip_serializing_if = "Option::is_none")]
    pub count: Option<i64>,
    #[serde(rename = "tablePart", default)]
    pub table_part: Vec<XlsxTablePart>,
}

/// Single table part reference.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "tablePart")]
pub struct XlsxTablePart {
    #[serde(rename = "@r:id", default, skip_serializing_if = "Option::is_none")]
    pub rid: Option<String>,
}

/// Background sheet image reference.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "picture")]
pub struct XlsxPicture {
    #[serde(rename = "@r:id", default, skip_serializing_if = "Option::is_none")]
    pub rid: Option<String>,
}

/// Single ignored error for a range of cells.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "ignoredError")]
pub struct XlsxIgnoredError {
    #[serde(rename = "@sqref")]
    pub sqref: String,
    #[serde(
        rename = "@evalError",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub eval_error: Option<bool>,
    #[serde(
        rename = "@twoDigitTextYear",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub two_digit_text_year: Option<bool>,
    #[serde(
        rename = "@numberStoredAsText",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub number_stored_as_text: Option<bool>,
    #[serde(rename = "@formula", default, skip_serializing_if = "Option::is_none")]
    pub formula: Option<bool>,
    #[serde(
        rename = "@formulaRange",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub formula_range: Option<bool>,
    #[serde(
        rename = "@unlockedFormula",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub unlocked_formula: Option<bool>,
    #[serde(
        rename = "@emptyCellReference",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub empty_cell_reference: Option<bool>,
    #[serde(
        rename = "@listDataValidation",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub list_data_validation: Option<bool>,
    #[serde(
        rename = "@calculatedColumn",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub calculated_column: Option<bool>,
}

/// Collection of ignored errors.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "ignoredErrors")]
pub struct XlsxIgnoredErrors {
    #[serde(rename = "ignoredError", default)]
    pub ignored_error: Vec<XlsxIgnoredError>,
    #[serde(rename = "extLst", default, skip_serializing_if = "Option::is_none")]
    pub ext_lst: Option<XlsxExtLst>,
}

/// Legacy drawing reference.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "legacyDrawing")]
pub struct XlsxLegacyDrawing {
    #[serde(rename = "@r:id", default, skip_serializing_if = "Option::is_none")]
    pub rid: Option<String>,
}

/// Legacy drawing reference in header/footer.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "legacyDrawingHF")]
pub struct XlsxLegacyDrawingHF {
    #[serde(rename = "@r:id", default, skip_serializing_if = "Option::is_none")]
    pub rid: Option<String>,
}

// ------------------------------------------------------------------
// x14 / decode conditional formatting and sparkline types
// ------------------------------------------------------------------

/// Decode x14 sparkline groups.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "sparklineGroups")]
pub struct DecodeX14SparklineGroups {
    #[serde(rename = "@xmlns:xm", default, skip_serializing_if = "Option::is_none")]
    pub xmlns_xm: Option<String>,
    #[serde(rename = "$value", default)]
    pub content: String,
}

/// Decode x14 conditional formatting ext element.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "ext")]
pub struct DecodeX14ConditionalFormattingExt {
    #[serde(rename = "@id", default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

/// Decode x14 conditional formattings placeholder.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "conditionalFormattings")]
pub struct DecodeX14ConditionalFormattings {
    #[serde(rename = "@xmlns:xm", default, skip_serializing_if = "Option::is_none")]
    pub xmlns_xm: Option<String>,
    #[serde(rename = "$value", default)]
    pub content: String,
}

/// Decode x14 conditional formatting rules container.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "conditionalFormattings")]
pub struct DecodeX14ConditionalFormattingRules {
    #[serde(rename = "@xmlns:xm", default, skip_serializing_if = "Option::is_none")]
    pub xmlns_xm: Option<String>,
    #[serde(rename = "conditionalFormatting", default)]
    pub cond_fmt: Vec<DecodeX14ConditionalFormatting>,
}

/// Decode x14 conditional formatting.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "conditionalFormatting")]
pub struct DecodeX14ConditionalFormatting {
    #[serde(rename = "@pivot", default, skip_serializing_if = "Option::is_none")]
    pub pivot: Option<bool>,
    #[serde(rename = "cfRule", default)]
    pub cf_rule: Vec<DecodeX14CfRule>,
    #[serde(rename = "sqref", default, skip_serializing_if = "Option::is_none")]
    pub sqref: Option<String>,
    #[serde(
        rename = "x14:extLst",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub ext_lst: Option<XlsxExtLst>,
}

/// Decode x14 cfRule.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "cfRule")]
pub struct DecodeX14CfRule {
    #[serde(rename = "@type", default, skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(rename = "@priority", default, skip_serializing_if = "Option::is_none")]
    pub priority: Option<i64>,
    #[serde(
        rename = "@stopIfTrue",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub stop_if_true: Option<bool>,
    #[serde(
        rename = "@aboveAverage",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub above_average: Option<bool>,
    #[serde(rename = "@percent", default, skip_serializing_if = "Option::is_none")]
    pub percent: Option<bool>,
    #[serde(rename = "@bottom", default, skip_serializing_if = "Option::is_none")]
    pub bottom: Option<bool>,
    #[serde(rename = "@operator", default, skip_serializing_if = "Option::is_none")]
    pub operator: Option<String>,
    #[serde(rename = "@text", default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(
        rename = "@timePeriod",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub time_period: Option<String>,
    #[serde(rename = "@rank", default, skip_serializing_if = "Option::is_none")]
    pub rank: Option<i64>,
    #[serde(rename = "@stdDev", default, skip_serializing_if = "Option::is_none")]
    pub std_dev: Option<i64>,
    #[serde(
        rename = "@equalAverage",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub equal_average: Option<bool>,
    #[serde(
        rename = "@activePresent",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub active_present: Option<bool>,
    #[serde(rename = "@id", default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(rename = "f", default)]
    pub f: Vec<String>,
    #[serde(
        rename = "colorScale",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub color_scale: Option<XlsxInnerXml>,
    #[serde(rename = "dataBar", default, skip_serializing_if = "Option::is_none")]
    pub data_bar: Option<DecodeX14DataBar>,
    #[serde(rename = "iconSet", default, skip_serializing_if = "Option::is_none")]
    pub icon_set: Option<DecodeX14IconSet>,
    #[serde(rename = "dxf", default, skip_serializing_if = "Option::is_none")]
    pub dxf: Option<XlsxInnerXml>,
    #[serde(rename = "extLst", default, skip_serializing_if = "Option::is_none")]
    pub ext_lst: Option<XlsxExtLst>,
}

/// Decode x14 dataBar.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "dataBar")]
pub struct DecodeX14DataBar {
    #[serde(
        rename = "@maxLength",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub max_length: Option<i64>,
    #[serde(
        rename = "@minLength",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub min_length: Option<i64>,
    #[serde(rename = "@border", default, skip_serializing_if = "Option::is_none")]
    pub border: Option<bool>,
    #[serde(rename = "@gradient", default, skip_serializing_if = "Option::is_none")]
    pub gradient: Option<bool>,
    #[serde(
        rename = "@showValue",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub show_value: Option<bool>,
    #[serde(
        rename = "@direction",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub direction: Option<String>,
    #[serde(rename = "cfvo", default)]
    pub cfvo: Vec<DecodeX14Cfvo>,
    #[serde(
        rename = "borderColor",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub border_color: Option<XlsxColor>,
    #[serde(
        rename = "negativeFillColor",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub negative_fill_color: Option<XlsxColor>,
    #[serde(rename = "axisColor", default, skip_serializing_if = "Option::is_none")]
    pub axis_color: Option<XlsxColor>,
}

/// Decode x14 iconSet.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "iconSet")]
pub struct DecodeX14IconSet {
    #[serde(rename = "@iconSet", default, skip_serializing_if = "Option::is_none")]
    pub icon_set: Option<String>,
    #[serde(
        rename = "@showValue",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub show_value: Option<bool>,
    #[serde(rename = "@percent", default, skip_serializing_if = "Option::is_none")]
    pub percent: Option<bool>,
    #[serde(rename = "@reverse", default, skip_serializing_if = "Option::is_none")]
    pub reverse: Option<bool>,
    #[serde(rename = "@custom", default, skip_serializing_if = "Option::is_none")]
    pub custom: Option<bool>,
    #[serde(rename = "cfvo", default)]
    pub cfvo: Vec<DecodeX14Cfvo>,
    #[serde(rename = "cfIcon", default)]
    pub cf_icon: Vec<XlsxInnerXml>,
}

/// Decode x14 cfvo.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "cfvo")]
pub struct DecodeX14Cfvo {
    #[serde(rename = "@type", default, skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(rename = "@gte", default, skip_serializing_if = "Option::is_none")]
    pub gte: Option<bool>,
    #[serde(rename = "f", default, skip_serializing_if = "Option::is_none")]
    pub f: Option<String>,
    #[serde(rename = "extLst", default, skip_serializing_if = "Option::is_none")]
    pub ext_lst: Option<XlsxExtLst>,
}

/// x14 conditional formattings placeholder.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "x14:conditionalFormattings")]
pub struct XlsxX14ConditionalFormattings {
    #[serde(rename = "$value", default)]
    pub content: String,
}

/// x14 conditional formatting.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "x14:conditionalFormatting")]
pub struct XlsxX14ConditionalFormatting {
    #[serde(rename = "@xmlns:xm", default, skip_serializing_if = "Option::is_none")]
    pub xmlns_xm: Option<String>,
    #[serde(rename = "@pivot", default, skip_serializing_if = "Option::is_none")]
    pub pivot: Option<bool>,
    #[serde(rename = "x14:cfRule", default)]
    pub cf_rule: Vec<XlsxX14CfRule>,
    #[serde(rename = "xm:sqref", default, skip_serializing_if = "Option::is_none")]
    pub sqref: Option<String>,
    #[serde(
        rename = "x14:extLst",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub ext_lst: Option<XlsxExtLst>,
}

/// x14 cfRule.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "x14:cfRule")]
pub struct XlsxX14CfRule {
    #[serde(rename = "@type", default, skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(rename = "@priority", default, skip_serializing_if = "Option::is_none")]
    pub priority: Option<i64>,
    #[serde(rename = "@stopIfTrue", default, skip_serializing_if = "is_false")]
    pub stop_if_true: Option<bool>,
    #[serde(rename = "@aboveAverage", default, skip_serializing_if = "is_false")]
    pub above_average: Option<bool>,
    #[serde(rename = "@percent", default, skip_serializing_if = "is_false")]
    pub percent: Option<bool>,
    #[serde(rename = "@bottom", default, skip_serializing_if = "is_false")]
    pub bottom: Option<bool>,
    #[serde(rename = "@operator", default, skip_serializing_if = "Option::is_none")]
    pub operator: Option<String>,
    #[serde(rename = "@text", default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(
        rename = "@timePeriod",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub time_period: Option<String>,
    #[serde(rename = "@rank", default, skip_serializing_if = "Option::is_none")]
    pub rank: Option<i64>,
    #[serde(rename = "@stdDev", default, skip_serializing_if = "Option::is_none")]
    pub std_dev: Option<i64>,
    #[serde(rename = "@equalAverage", default, skip_serializing_if = "is_false")]
    pub equal_average: Option<bool>,
    #[serde(rename = "@activePresent", default, skip_serializing_if = "is_false")]
    pub active_present: Option<bool>,
    #[serde(rename = "@id", default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(rename = "xm:f", default)]
    pub f: Vec<String>,
    #[serde(
        rename = "x14:colorScale",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub color_scale: Option<XlsxInnerXml>,
    #[serde(
        rename = "x14:dataBar",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub data_bar: Option<Xlsx14DataBar>,
    #[serde(
        rename = "x14:iconSet",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub icon_set: Option<Xlsx14IconSet>,
    #[serde(rename = "x14:dxf", default, skip_serializing_if = "Option::is_none")]
    pub dxf: Option<XlsxInnerXml>,
    #[serde(
        rename = "x14:extLst",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub ext_lst: Option<XlsxExtLst>,
}

/// x14 dataBar.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "x14:dataBar")]
pub struct Xlsx14DataBar {
    #[serde(
        rename = "@maxLength",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub max_length: Option<i64>,
    #[serde(
        rename = "@minLength",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub min_length: Option<i64>,
    #[serde(rename = "@border", default, skip_serializing_if = "Option::is_none")]
    pub border: Option<bool>,
    #[serde(rename = "@gradient", default, skip_serializing_if = "Option::is_none")]
    pub gradient: Option<bool>,
    #[serde(
        rename = "@showValue",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub show_value: Option<bool>,
    #[serde(
        rename = "@direction",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub direction: Option<String>,
    #[serde(rename = "x14:cfvo", default)]
    pub cfvo: Vec<Xlsx14Cfvo>,
    #[serde(
        rename = "x14:borderColor",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub border_color: Option<XlsxColor>,
    #[serde(
        rename = "x14:negativeFillColor",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub negative_fill_color: Option<XlsxColor>,
    #[serde(
        rename = "x14:axisColor",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub axis_color: Option<XlsxColor>,
}

/// x14 iconSet.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "x14:iconSet")]
pub struct Xlsx14IconSet {
    #[serde(rename = "@iconSet", default, skip_serializing_if = "Option::is_none")]
    pub icon_set: Option<String>,
    #[serde(
        rename = "@showValue",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub show_value: Option<bool>,
    #[serde(rename = "@percent", default, skip_serializing_if = "Option::is_none")]
    pub percent: Option<bool>,
    #[serde(rename = "@reverse", default, skip_serializing_if = "Option::is_none")]
    pub reverse: Option<bool>,
    #[serde(rename = "@custom", default, skip_serializing_if = "Option::is_none")]
    pub custom: Option<bool>,
    #[serde(rename = "x14:cfvo", default)]
    pub cfvo: Vec<Xlsx14Cfvo>,
    #[serde(rename = "x14:cfIcon", default)]
    pub cf_icon: Vec<XlsxInnerXml>,
}

/// x14 cfvo.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "x14:cfvo")]
pub struct Xlsx14Cfvo {
    #[serde(rename = "@type", default, skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(rename = "@gte", default, skip_serializing_if = "Option::is_none")]
    pub gte: Option<bool>,
    #[serde(rename = "xm:f", default, skip_serializing_if = "Option::is_none")]
    pub f: Option<String>,
    #[serde(
        rename = "x14:extLst",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub ext_lst: Option<XlsxExtLst>,
}

/// x14 sparkline groups.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "x14:sparklineGroups")]
pub struct XlsxX14SparklineGroups {
    #[serde(rename = "@xmlns:xm", default, skip_serializing_if = "Option::is_none")]
    pub xmlns_xm: Option<String>,
    #[serde(rename = "x14:sparklineGroup", default)]
    pub sparkline_groups: Vec<XlsxX14SparklineGroup>,
    #[serde(rename = "$value", default)]
    pub content: String,
}

/// x14 sparkline group.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "x14:sparklineGroup")]
pub struct XlsxX14SparklineGroup {
    #[serde(
        rename = "@manualMax",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub manual_max: Option<i64>,
    #[serde(
        rename = "@manualMin",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub manual_min: Option<i64>,
    #[serde(
        rename = "@lineWeight",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub line_weight: Option<f64>,
    #[serde(rename = "@type", default, skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(rename = "@dateAxis", default, skip_serializing_if = "Option::is_none")]
    pub date_axis: Option<bool>,
    #[serde(
        rename = "@displayEmptyCellsAs",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub display_empty_cells_as: Option<String>,
    #[serde(rename = "@markers", default, skip_serializing_if = "Option::is_none")]
    pub markers: Option<bool>,
    #[serde(rename = "@high", default, skip_serializing_if = "Option::is_none")]
    pub high: Option<bool>,
    #[serde(rename = "@low", default, skip_serializing_if = "Option::is_none")]
    pub low: Option<bool>,
    #[serde(rename = "@first", default, skip_serializing_if = "Option::is_none")]
    pub first: Option<bool>,
    #[serde(rename = "@last", default, skip_serializing_if = "Option::is_none")]
    pub last: Option<bool>,
    #[serde(rename = "@negative", default, skip_serializing_if = "Option::is_none")]
    pub negative: Option<bool>,
    #[serde(
        rename = "@displayXAxis",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub display_x_axis: Option<bool>,
    #[serde(
        rename = "@displayHidden",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub display_hidden: Option<bool>,
    #[serde(
        rename = "@minAxisType",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub min_axis_type: Option<String>,
    #[serde(
        rename = "@maxAxisType",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub max_axis_type: Option<String>,
    #[serde(
        rename = "@rightToLeft",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub right_to_left: Option<bool>,
    #[serde(
        rename = "x14:colorSeries",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub color_series: Option<XlsxColor>,
    #[serde(
        rename = "x14:colorNegative",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub color_negative: Option<XlsxColor>,
    #[serde(
        rename = "x14:colorAxis",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub color_axis: Option<XlsxColor>,
    #[serde(
        rename = "x14:colorMarkers",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub color_markers: Option<XlsxColor>,
    #[serde(
        rename = "x14:colorFirst",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub color_first: Option<XlsxColor>,
    #[serde(
        rename = "x14:colorLast",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub color_last: Option<XlsxColor>,
    #[serde(
        rename = "x14:colorHigh",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub color_high: Option<XlsxColor>,
    #[serde(
        rename = "x14:colorLow",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub color_low: Option<XlsxColor>,
    #[serde(rename = "x14:sparklines", default)]
    pub sparklines: XlsxX14Sparklines,
}

/// x14 sparklines container.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "x14:sparklines")]
pub struct XlsxX14Sparklines {
    #[serde(rename = "x14:sparkline", default)]
    pub sparkline: Vec<XlsxX14Sparkline>,
}

/// x14 sparkline.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "x14:sparkline")]
pub struct XlsxX14Sparkline {
    #[serde(rename = "xm:f", default)]
    pub f: String,
    #[serde(rename = "xm:sqref", default)]
    pub sqref: String,
}

// ------------------------------------------------------------------
// Public API types
// ------------------------------------------------------------------

/// Data validation rule settings.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DataValidation {
    pub allow_blank: bool,
    pub error: Option<String>,
    pub error_style: Option<String>,
    pub error_title: Option<String>,
    pub operator: String,
    pub prompt: Option<String>,
    pub prompt_title: Option<String>,
    pub show_drop_down: bool,
    pub show_error_message: bool,
    pub show_input_message: bool,
    pub sqref: String,
    pub r#type: String,
    pub formula1: String,
    pub formula2: String,
}

/// Sparkline settings.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct SparklineOptions {
    pub location: Vec<String>,
    pub range: Vec<String>,
    pub max: i64,
    pub cust_max: i64,
    pub min: i64,
    pub cust_min: i64,
    pub r#type: String,
    pub weight: f64,
    pub date_axis: bool,
    pub markers: bool,
    pub high: bool,
    pub low: bool,
    pub first: bool,
    pub last: bool,
    pub negative: bool,
    pub axis: bool,
    pub hidden: bool,
    pub reverse: bool,
    pub style: i64,
    pub series_color: String,
    pub negative_color: String,
    pub markers_color: String,
    pub first_color: String,
    pub last_color: String,
    pub hight_color: String,
    pub low_color: String,
    pub empty_cells: String,
}

/// Worksheet selection.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Selection {
    pub sqref: String,
    pub active_cell: String,
    pub pane: String,
}

/// Pane settings.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Panes {
    pub freeze: bool,
    pub split: bool,
    pub x_split: i64,
    pub y_split: i64,
    pub top_left_cell: String,
    pub active_pane: String,
    pub selection: Vec<Selection>,
}

/// Conditional format settings.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConditionalFormatOptions {
    pub r#type: String,
    pub above_average: bool,
    pub percent: bool,
    pub format: Option<i64>,
    pub criteria: String,
    pub value: String,
    pub min_type: String,
    pub mid_type: String,
    pub max_type: String,
    pub min_value: String,
    pub mid_value: String,
    pub max_value: String,
    pub min_color: String,
    pub mid_color: String,
    pub max_color: String,
    pub bar_color: String,
    pub bar_border_color: String,
    pub bar_direction: String,
    pub bar_only: bool,
    pub bar_solid: bool,
    pub icon_style: String,
    pub reverse_icons: bool,
    pub icons_only: bool,
    pub stop_if_true: bool,
}

/// Worksheet protection settings.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SheetProtectionOptions {
    pub algorithm_name: String,
    pub auto_filter: bool,
    pub delete_columns: bool,
    pub delete_rows: bool,
    pub edit_objects: bool,
    pub edit_scenarios: bool,
    pub format_cells: bool,
    pub format_columns: bool,
    pub format_rows: bool,
    pub insert_columns: bool,
    pub insert_hyperlinks: bool,
    pub insert_rows: bool,
    pub password: String,
    pub pivot_tables: bool,
    pub select_locked_cells: bool,
    pub select_unlocked_cells: bool,
    pub sort: bool,
}

/// Header and footer settings.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeaderFooterOptions {
    pub align_with_margins: Option<bool>,
    pub different_first: bool,
    pub different_odd_even: bool,
    pub scale_with_doc: Option<bool>,
    pub odd_header: String,
    pub odd_footer: String,
    pub even_header: String,
    pub even_footer: String,
    pub first_header: String,
    pub first_footer: String,
}

/// Page layout margin settings.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct PageLayoutMarginsOptions {
    pub bottom: Option<f64>,
    pub footer: Option<f64>,
    pub header: Option<f64>,
    pub left: Option<f64>,
    pub right: Option<f64>,
    pub top: Option<f64>,
    pub horizontally: Option<bool>,
    pub vertically: Option<bool>,
}

/// Page layout settings.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct PageLayoutOptions {
    pub size: Option<i64>,
    pub orientation: Option<String>,
    pub first_page_number: Option<u64>,
    pub adjust_to: Option<u64>,
    pub fit_to_height: Option<i64>,
    pub fit_to_width: Option<i64>,
    pub black_and_white: Option<bool>,
    pub page_order: Option<String>,
}

/// Sheet view settings.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct ViewOptions {
    pub default_grid_color: Option<bool>,
    pub right_to_left: Option<bool>,
    pub show_formulas: Option<bool>,
    pub show_grid_lines: Option<bool>,
    pub show_row_col_headers: Option<bool>,
    pub show_ruler: Option<bool>,
    pub show_zeros: Option<bool>,
    pub top_left_cell: Option<String>,
    pub view: Option<String>,
    pub zoom_scale: Option<f64>,
}

/// Worksheet properties options.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct SheetPropsOptions {
    pub code_name: Option<String>,
    pub enable_format_conditions_calculation: Option<bool>,
    pub published: Option<bool>,
    pub auto_page_breaks: Option<bool>,
    pub fit_to_page: Option<bool>,
    pub tab_color_indexed: Option<i64>,
    pub tab_color_rgb: Option<String>,
    pub tab_color_theme: Option<i64>,
    pub tab_color_tint: Option<f64>,
    pub outline_summary_below: Option<bool>,
    pub outline_summary_right: Option<bool>,
    pub base_col_width: Option<u8>,
    pub default_col_width: Option<f64>,
    pub default_row_height: Option<f64>,
    pub custom_height: Option<bool>,
    pub zero_height: Option<bool>,
    pub thick_top: Option<bool>,
    pub thick_bottom: Option<bool>,
}
