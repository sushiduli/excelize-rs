//! Error types and messages for the `excelize` crate.
//!
//! These errors are ported from the Go `excelize` package's `errors.go`.

use std::fmt;

use thiserror::Error;

use crate::constants::{
    MAX_CELL_STYLES, MAX_COLUMN_WIDTH, MAX_COLUMNS, MAX_FIELD_LENGTH, MAX_FILE_PATH_LENGTH,
    MAX_FONT_SIZE, MAX_FORM_CONTROL_VALUE, MAX_GRAPHIC_ALT_TEXT_LENGTH, MAX_GRAPHIC_NAME_LENGTH,
    MAX_ROW_HEIGHT, MAX_SHEET_NAME_LENGTH, MIN_COLUMNS, MIN_FONT_SIZE, TOTAL_CELL_CHARS,
};

// ------------------------------------------------------------------
// Static errors
// ------------------------------------------------------------------

#[derive(Debug, Error)]
#[error("unsupported VBA project")]
pub struct ErrAddVBAProject;

#[derive(Debug, Error)]
#[error("unexpected child of attrValBool")]
pub struct ErrAttrValBool;

#[derive(Debug, Error)]
#[error("cell value must be 0-{0} characters", TOTAL_CELL_CHARS)]
pub struct ErrCellCharsLength;

#[derive(Debug, Error)]
#[error("the cell styles exceeds the {MAX_CELL_STYLES} limit")]
pub struct ErrCellStyles;

#[derive(Debug, Error)]
#[error("cannot set both 'Formula' and 'Paragraph' for chart title")]
pub struct ErrChartTitle;

#[derive(Debug, Error)]
#[error(
    "the column number must be greater than or equal to {MIN_COLUMNS} and less than or equal to {MAX_COLUMNS}"
)]
pub struct ErrColumnNumber;

#[derive(Debug, Error)]
#[error("the width of the column must be less than or equal to {MAX_COLUMN_WIDTH} characters")]
pub struct ErrColumnWidth;

#[derive(Debug, Error)]
#[error("coordinates length must be 4")]
pub struct ErrCoordinates;

#[derive(Debug, Error)]
#[error("custom number format can not be empty")]
pub struct ErrCustomNumFmt;

#[derive(Debug, Error)]
#[error("data validation must be 0-{MAX_FIELD_LENGTH} characters")]
pub struct ErrDataValidationFormulaLength;

#[derive(Debug, Error)]
#[error("data validation range exceeds limit")]
pub struct ErrDataValidationRange;

#[derive(Debug, Error)]
#[error("the same name already exists on the scope")]
pub struct ErrDefinedNameDuplicate;

#[derive(Debug, Error)]
#[error("no defined name on the scope")]
pub struct ErrDefinedNameScope;

#[derive(Debug, Error)]
#[error("the same name sheet already exists")]
pub struct ErrExistsSheet;

#[derive(Debug, Error)]
#[error("the same name table already exists")]
pub struct ErrExistsTableName;

#[derive(Debug, Error)]
#[error("fill type value must be one of 'gradient' or 'pattern'")]
pub struct ErrFillType;

#[derive(Debug, Error)]
#[error("fill color value must be an array of two colors for 'gradient' type")]
pub struct ErrFillGradientColor;

#[derive(Debug, Error)]
#[error("fill shading value must be between 0 and 16 for 'gradient' type")]
pub struct ErrFillGradientShading;

#[derive(Debug, Error)]
#[error("fill color value must be empty or an array of one color for 'pattern' type")]
pub struct ErrFillPatternColor;

#[derive(Debug, Error)]
#[error("fill pattern value must be between 0 and 18")]
pub struct ErrFillPattern;

#[derive(Debug, Error)]
#[error("the length of the font family name must be less than or equal to 31")]
pub struct ErrFontLength;

#[derive(Debug, Error)]
#[error("font size must be an integer from {MIN_FONT_SIZE} to {MAX_FONT_SIZE} points")]
pub struct ErrFontSize;

#[derive(Debug, Error)]
#[error("scroll value must be an integer from 0 to {MAX_FORM_CONTROL_VALUE}")]
pub struct ErrFormControlValue;

#[derive(Debug, Error)]
#[error("group worksheet must contain an active worksheet")]
pub struct ErrGroupSheets;

#[derive(Debug, Error)]
#[error("unsupported image extension")]
pub struct ErrImgExt;

#[derive(Debug, Error)]
#[error("image decode failed")]
pub struct ErrImgLoad;

#[derive(Debug, Error)]
#[error("formula not valid")]
pub struct ErrInvalidFormula;

#[derive(Debug, Error)]
#[error("invalid date value")]
pub struct ErrInvalidDate;

#[derive(Debug, Error)]
#[error("file path length exceeds maximum limit {MAX_FILE_PATH_LENGTH} characters")]
pub struct ErrMaxFilePathLength;

#[derive(Debug, Error)]
#[error("the height of the row must be less than or equal to {MAX_ROW_HEIGHT} points")]
pub struct ErrMaxRowHeight;

#[derive(Debug, Error)]
#[error("row number exceeds maximum limit")]
pub struct ErrMaxRows;

#[derive(Debug, Error)]
#[error("the name length exceeds the {MAX_FIELD_LENGTH} characters limit")]
pub struct ErrNameLength;

#[derive(Debug, Error)]
#[error("the alt text length exceeds the {MAX_GRAPHIC_ALT_TEXT_LENGTH} characters limit")]
pub struct ErrMaxGraphicAltTextLength;

#[derive(Debug, Error)]
#[error("the name length exceeds the {MAX_GRAPHIC_NAME_LENGTH} characters limit")]
pub struct ErrMaxGraphicNameLength;

#[derive(Debug, Error)]
#[error("the value of UnzipSizeLimit should be greater than or equal to UnzipXMLSizeLimit")]
pub struct ErrOptionsUnzipSizeLimit;

#[derive(Debug, Error)]
#[error("invalid outline level")]
pub struct ErrOutlineLevel;

#[derive(Debug, Error)]
#[error("adjust to value must be an integer from 0 to 400")]
pub struct ErrPageSetupAdjustTo;

#[derive(Debug, Error)]
#[error("parameter is invalid")]
pub struct ErrParameterInvalid;

#[derive(Debug, Error)]
#[error("parameter is required")]
pub struct ErrParameterRequired;

#[derive(Debug, Error)]
#[error("password length invalid")]
pub struct ErrPasswordLengthInvalid;

#[derive(Debug, Error)]
#[error("this kind of show value as type requires a base field")]
pub struct ErrPivotTableShowValuesAsBaseField;

#[derive(Debug, Error)]
#[error("this kind of show value as type and base field requires a base item")]
pub struct ErrPivotTableShowValuesAsBaseItem;

#[derive(Debug, Error)]
#[error("cannot enable ClassicLayout and CompactData in the same time")]
pub struct ErrPivotTableClassicLayout;

#[derive(Debug, Error)]
#[error("no path defined for file, consider File.write_to or File.write")]
pub struct ErrSave;

#[derive(Debug, Error)]
#[error("invalid worksheet index")]
pub struct ErrSheetIdx;

#[derive(Debug, Error)]
#[error("the sheet name can not be blank")]
pub struct ErrSheetNameBlank;

#[derive(Debug, Error)]
#[error("the sheet can not contain any of the characters :\\/?*[or]")]
pub struct ErrSheetNameInvalid;

#[derive(Debug, Error)]
#[error("the sheet name length exceeds the {MAX_SHEET_NAME_LENGTH} characters limit")]
pub struct ErrSheetNameLength;

#[derive(Debug, Error)]
#[error("the first or last character of the sheet name can not be a single quote")]
pub struct ErrSheetNameSingleQuote;

#[derive(Debug, Error)]
#[error("must have the same number of 'Location' and 'Range' parameters")]
pub struct ErrSparkline;

#[derive(Debug, Error)]
#[error("parameter 'Location' is required")]
pub struct ErrSparklineLocation;

#[derive(Debug, Error)]
#[error("parameter 'Range' is required")]
pub struct ErrSparklineRange;

#[derive(Debug, Error)]
#[error("parameter 'Style' value must be an integer from 0 to 35")]
pub struct ErrSparklineStyle;

#[derive(Debug, Error)]
#[error("parameter 'Type' value must be one of 'line', 'column' or 'win_loss'")]
pub struct ErrSparklineType;

#[derive(Debug, Error)]
#[error("over maximum limit hyperlinks in a worksheet")]
pub struct ErrTotalSheetHyperlinks;

#[derive(Debug, Error)]
#[error("transparency value must be an integer from 0 to 100")]
pub struct ErrTransparency;

#[derive(Debug, Error)]
#[error("unknown encryption mechanism")]
pub struct ErrUnknownEncryptMechanism;

#[derive(Debug, Error)]
#[error("worksheet has set no protect")]
pub struct ErrUnprotectSheet;

#[derive(Debug, Error)]
#[error("worksheet protect password not match")]
pub struct ErrUnprotectSheetPassword;

#[derive(Debug, Error)]
#[error("workbook has set no protect")]
pub struct ErrUnprotectWorkbook;

#[derive(Debug, Error)]
#[error("workbook protect password not match")]
pub struct ErrUnprotectWorkbookPassword;

#[derive(Debug, Error)]
#[error("unsupported encryption mechanism")]
pub struct ErrUnsupportedEncryptMechanism;

#[derive(Debug, Error)]
#[error("unsupported hash algorithm")]
pub struct ErrUnsupportedHashAlgorithm;

#[derive(Debug, Error)]
#[error("unsupported number format token")]
pub struct ErrUnsupportedNumberFormat;

#[derive(Debug, Error)]
#[error("unsupported pivot table show value as type")]
pub struct ErrUnsupportedPivotTableShowValuesAsType;

#[derive(Debug, Error)]
#[error("unsupported workbook file format")]
pub struct ErrWorkbookFileFormat;

#[derive(Debug, Error)]
#[error("the supplied open workbook password is not correct")]
pub struct ErrWorkbookPassword;

#[derive(Debug, Error)]
#[error("workbook must contain at least one worksheet")]
pub struct ErrWorkbook;

// ------------------------------------------------------------------
// Parameterized errors
// ------------------------------------------------------------------

/// Error returned when a worksheet does not exist.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrSheetNotExist {
    pub sheet_name: String,
}

impl fmt::Display for ErrSheetNotExist {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "sheet {} does not exist", self.sheet_name)
    }
}

impl std::error::Error for ErrSheetNotExist {}

/// A convenience alias for a `Result` using the dynamic `Error` type.
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

// ------------------------------------------------------------------
// Helper constructors (ported from `errors.go`)
// ------------------------------------------------------------------

pub fn new_add_comment_error(cell: &str) -> String {
    format!("comment already exist on cell {cell}")
}

pub fn new_cell_name_to_coordinates_error(cell: &str, err: impl std::error::Error) -> String {
    format!("cannot convert cell {cell:?} to coordinates: {err}")
}

pub fn new_chart_title_error(name: &str) -> String {
    format!("chart title field {name} value must be an integer from 0 to 100")
}

pub fn new_coordinates_to_cell_name_error(col: i32, row: i32) -> String {
    format!("invalid cell reference [{col}, {row}]")
}

pub fn new_field_length_error(name: &str) -> String {
    format!("field {name} must be less than or equal to 255 characters")
}

pub fn new_invalid_auto_filter_column_error(col: &str) -> String {
    format!("incorrect index of column {col:?}")
}

pub fn new_invalid_auto_filter_exp_error(exp: &str) -> String {
    format!("incorrect number of tokens in criteria {exp:?}")
}

pub fn new_invalid_auto_filter_operator_error(op: &str, exp: &str) -> String {
    format!(
        "the operator {op:?} in expression {exp:?} is not valid in relation to Blanks/NonBlanks"
    )
}

pub fn new_invalid_cell_name_error(cell: &str) -> String {
    format!("invalid cell name {cell:?}")
}

pub fn new_invalid_column_name_error(col: &str) -> String {
    format!("invalid column name {col:?}")
}

pub fn new_invalid_excel_date_error(date_value: f64) -> String {
    format!("invalid date value {date_value}, negative values are not supported")
}

pub fn new_invalid_link_type_error(link_type: &str) -> String {
    format!("invalid link type {link_type:?}")
}

pub fn new_invalid_name_error(name: &str) -> String {
    format!(
        "invalid name {name:?}, the name should be starts with a letter or underscore, can not include a space or character, and can not conflict with an existing name in the workbook"
    )
}

pub fn new_invalid_optional_value(name: &str, value: &str, values: &[&str]) -> String {
    format!(
        "invalid {name} value {value:?}, acceptable value should be one of {}",
        values.join(", ")
    )
}

pub fn new_invalid_row_number_error(row: i32) -> String {
    format!("invalid row number {row}")
}

pub fn new_invalid_shared_string_index_error(idx: i32) -> String {
    format!("invalid shared string index {idx}")
}

pub fn new_invalid_slicer_name_error(name: &str) -> String {
    format!("invalid slicer name {name:?}")
}

pub fn new_invalid_style_id_error(style_id: i32) -> String {
    format!("invalid style ID {style_id}")
}

pub fn new_no_exist_slicer_error(name: &str) -> String {
    format!("slicer {name} does not exist")
}

pub fn new_no_exist_table_error(name: &str) -> String {
    format!("table {name} does not exist")
}

pub fn new_not_worksheet_error(name: &str) -> String {
    format!("sheet {name} is not a worksheet")
}

pub fn new_pivot_table_col_fields_error(data: &[String]) -> String {
    format!(
        "data fields {} appear both in the pivot table column fields and filter fields",
        data.join(", ")
    )
}

pub fn new_pivot_table_row_fields_error(data: &[String]) -> String {
    format!(
        "data fields {} appear both in the pivot table row fields and filter fields",
        data.join(", ")
    )
}

pub fn new_pivot_table_data_range_error(msg: &str) -> String {
    format!("parameter 'DataRange' parsing error: {msg}")
}

pub fn new_pivot_table_selected_item_error(item: &str, field: &str) -> String {
    format!("selected item {item} does not exist in pivot table field {field}")
}

pub fn new_pivot_table_range_error(msg: &str) -> String {
    format!("parameter 'PivotTableRange' parsing error: {msg}")
}

pub fn new_pivot_table_show_values_as_base_field_error(field: &str) -> String {
    format!("base field {field} does not exist in shared items")
}

pub fn new_stream_set_row_error(row: i32) -> String {
    format!("row {row} has already been written")
}

pub fn new_stream_set_row_order_error(name: &str) -> String {
    format!("must call the {name} function before the SetRow function")
}

pub fn new_unknown_filter_token_error(token: &str) -> String {
    format!("unknown operator: {token}")
}

pub fn new_unsupported_chart_type_error(chart_type: i32) -> String {
    format!("unsupported chart type {chart_type}")
}

pub fn new_unsupported_pivot_cache_source_type_error(source_type: &str) -> String {
    format!("unsupported pivot table cache source type: {source_type}")
}

pub fn new_unzip_size_limit_error(unzip_size_limit: i64) -> String {
    format!("unzip size exceeds the {unzip_size_limit} bytes limit")
}

pub fn new_view_idx_error(view_index: i32) -> String {
    format!("view index {view_index} out of range")
}
