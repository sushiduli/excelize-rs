//! Table part (`xl/tables/table*.xml`).
//!
//! Ported from Go `xmlTable.go`.

use serde::{Deserialize, Serialize};

use super::common::XlsxInnerXml;

/// Directly maps the table element. A table helps organize and provide structure
/// to list of information in a worksheet.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "table")]
pub struct XlsxTable {
    #[serde(rename = "@xmlns", default)]
    pub xmlns: Option<String>,
    #[serde(rename = "@id")]
    pub id: i64,
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@displayName", default)]
    pub display_name: Option<String>,
    #[serde(rename = "@comment", default)]
    pub comment: Option<String>,
    #[serde(rename = "@ref")]
    pub r#ref: String,
    #[serde(rename = "@tableType", default)]
    pub table_type: Option<String>,
    #[serde(rename = "@headerRowCount", default)]
    pub header_row_count: Option<i64>,
    #[serde(rename = "@insertRow", default)]
    pub insert_row: bool,
    #[serde(rename = "@insertRowShift", default)]
    pub insert_row_shift: bool,
    #[serde(rename = "@totalsRowCount", default)]
    pub totals_row_count: i64,
    #[serde(rename = "@totalsRowShown", default)]
    pub totals_row_shown: Option<bool>,
    #[serde(rename = "@published", default)]
    pub published: bool,
    #[serde(rename = "@headerRowDxfId", default)]
    pub header_row_dxf_id: i64,
    #[serde(rename = "@dataDxfId", default)]
    pub data_dxf_id: i64,
    #[serde(rename = "@totalsRowDxfId", default)]
    pub totals_row_dxf_id: i64,
    #[serde(rename = "@headerRowBorderDxfId", default)]
    pub header_row_border_dxf_id: i64,
    #[serde(rename = "@tableBorderDxfId", default)]
    pub table_border_dxf_id: i64,
    #[serde(rename = "@totalsRowBorderDxfId", default)]
    pub totals_row_border_dxf_id: i64,
    #[serde(rename = "@headerRowCellStyle", default)]
    pub header_row_cell_style: Option<String>,
    #[serde(rename = "@dataCellStyle", default)]
    pub data_cell_style: Option<String>,
    #[serde(rename = "@totalsRowCellStyle", default)]
    pub totals_row_cell_style: Option<String>,
    #[serde(rename = "@connectionId", default)]
    pub connection_id: i64,
    #[serde(rename = "autoFilter", default)]
    pub auto_filter: Option<XlsxAutoFilter>,
    #[serde(rename = "tableColumns", default)]
    pub table_columns: Option<XlsxTableColumns>,
    #[serde(rename = "tableStyleInfo", default)]
    pub table_style_info: Option<XlsxTableStyleInfo>,
}

/// Temporarily hides rows based on a filter criteria, which is applied column by
/// column to a table of data in the worksheet.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "autoFilter")]
pub struct XlsxAutoFilter {
    #[serde(rename = "@ref")]
    pub r#ref: String,
    #[serde(rename = "filterColumn", default)]
    pub filter_column: Vec<XlsxFilterColumn>,
}

/// Identifies a particular column in the AutoFilter range and specifies filter
/// information that has been applied to this column.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "filterColumn")]
pub struct XlsxFilterColumn {
    #[serde(rename = "@colId")]
    pub col_id: i64,
    #[serde(rename = "@hiddenButton", default)]
    pub hidden_button: bool,
    #[serde(rename = "@showButton", default)]
    pub show_button: bool,
    #[serde(rename = "customFilters", default)]
    pub custom_filters: Option<XlsxCustomFilters>,
    #[serde(rename = "filters", default)]
    pub filters: Option<XlsxFilters>,
    #[serde(rename = "colorFilter", default)]
    pub color_filter: Option<XlsxColorFilter>,
    #[serde(rename = "dynamicFilter", default)]
    pub dynamic_filter: Option<XlsxDynamicFilter>,
    #[serde(rename = "iconFilter", default)]
    pub icon_filter: Option<XlsxIconFilter>,
    #[serde(rename = "top10", default)]
    pub top10: Option<XlsxTop10>,
}

/// Groups custom filter elements together when there is more than one custom
/// filter criteria to apply.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "customFilters")]
pub struct XlsxCustomFilters {
    #[serde(rename = "@and", default)]
    pub and: bool,
    #[serde(rename = "customFilter", default)]
    pub custom_filter: Vec<XlsxCustomFilter>,
}

/// A custom AutoFilter specifies an operator and a value.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "customFilter")]
pub struct XlsxCustomFilter {
    #[serde(rename = "@operator", default)]
    pub operator: Option<String>,
    #[serde(rename = "@val", default)]
    pub val: Option<String>,
}

/// Groups filter criteria together when multiple values are chosen to filter by,
/// or when a group of date values are chosen to filter by.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "filters")]
pub struct XlsxFilters {
    #[serde(rename = "@blank", default)]
    pub blank: bool,
    #[serde(rename = "@calendarType", default)]
    pub calendar_type: Option<String>,
    #[serde(rename = "filter", default)]
    pub filter: Vec<XlsxFilter>,
    #[serde(rename = "dateGroupItem", default)]
    pub date_group_item: Vec<XlsxDateGroupItem>,
}

/// Expresses a filter criteria value.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "filter")]
pub struct XlsxFilter {
    #[serde(rename = "@val", default)]
    pub val: Option<String>,
}

/// Specifies the color to filter by and whether to use the cell's fill or font
/// color in the filter criteria.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "colorFilter")]
pub struct XlsxColorFilter {
    #[serde(rename = "@cellColor")]
    pub cell_color: bool,
    #[serde(rename = "@dxfId")]
    pub dxf_id: i64,
}

/// Specifies dynamic filter criteria. These criteria can change either with the
/// data itself or with the current system date.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "dynamicFilter")]
pub struct XlsxDynamicFilter {
    #[serde(rename = "@maxValIso", default)]
    pub max_val_iso: Option<String>,
    #[serde(rename = "@type", default)]
    pub r#type: Option<String>,
    #[serde(rename = "@val", default)]
    pub val: Option<f64>,
    #[serde(rename = "@valIso", default)]
    pub val_iso: Option<String>,
}

/// Specifies the icon set and particular icon within that set to filter by.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "iconFilter")]
pub struct XlsxIconFilter {
    #[serde(rename = "@iconId")]
    pub icon_id: i64,
    #[serde(rename = "@iconSet", default)]
    pub icon_set: Option<String>,
}

/// Specifies the top N (percent or number of items) to filter by.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "top10")]
pub struct XlsxTop10 {
    #[serde(rename = "@filterVal", default)]
    pub filter_val: Option<f64>,
    #[serde(rename = "@percent", default)]
    pub percent: bool,
    #[serde(rename = "@top")]
    pub top: bool,
    #[serde(rename = "@val", default)]
    pub val: Option<f64>,
}

/// Expresses a group of dates or times which are used in an AutoFilter criteria.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "dateGroupItem")]
pub struct XlsxDateGroupItem {
    #[serde(rename = "@dateTimeGrouping", default)]
    pub date_time_grouping: Option<String>,
    #[serde(rename = "@day", default)]
    pub day: i64,
    #[serde(rename = "@hour", default)]
    pub hour: i64,
    #[serde(rename = "@minute", default)]
    pub minute: i64,
    #[serde(rename = "@month", default)]
    pub month: i64,
    #[serde(rename = "@second", default)]
    pub second: i64,
    #[serde(rename = "@year", default)]
    pub year: i64,
}

/// Represents the collection of all table columns for this table.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "tableColumns")]
pub struct XlsxTableColumns {
    #[serde(rename = "@count")]
    pub count: i64,
    #[serde(rename = "tableColumn", default)]
    pub table_column: Vec<XlsxTableColumn>,
}

/// Represents a single column for this table.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "tableColumn")]
pub struct XlsxTableColumn {
    #[serde(rename = "@id")]
    pub id: i64,
    #[serde(rename = "@uniqueName", default)]
    pub unique_name: Option<String>,
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@totalsRowFunction", default)]
    pub totals_row_function: Option<String>,
    #[serde(rename = "@totalsRowLabel", default)]
    pub totals_row_label: Option<String>,
    #[serde(rename = "@queryTableFieldId", default)]
    pub query_table_field_id: i64,
    #[serde(rename = "@headerRowDxfId", default)]
    pub header_row_dxf_id: i64,
    #[serde(rename = "@dataDxfId", default)]
    pub data_dxf_id: i64,
    #[serde(rename = "@totalsRowDxfId", default)]
    pub totals_row_dxf_id: i64,
    #[serde(rename = "@headerRowCellStyle", default)]
    pub header_row_cell_style: Option<String>,
    #[serde(rename = "@dataCellStyle", default)]
    pub data_cell_style: Option<String>,
    #[serde(rename = "@totalsRowCellStyle", default)]
    pub totals_row_cell_style: Option<String>,
}

/// Describes which style is used to display this table, and specifies which
/// portions of the table have the style applied.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "tableStyleInfo")]
pub struct XlsxTableStyleInfo {
    #[serde(rename = "@name", default)]
    pub name: Option<String>,
    #[serde(rename = "@showFirstColumn")]
    pub show_first_column: bool,
    #[serde(rename = "@showLastColumn")]
    pub show_last_column: bool,
    #[serde(rename = "@showRowStripes")]
    pub show_row_stripes: bool,
    #[serde(rename = "@showColumnStripes")]
    pub show_column_stripes: bool,
}

/// A single cell table generated from an XML mapping.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "singleXmlCells")]
pub struct XlsxSingleXmlCells {
    #[serde(rename = "singleXmlCell", default)]
    pub single_xml_cell: Vec<XlsxSingleXmlCell>,
}

/// Represents the table properties for a single cell XML table.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "singleXmlCell")]
pub struct XlsxSingleXmlCell {
    #[serde(rename = "@id")]
    pub id: i64,
    #[serde(rename = "@r")]
    pub r: String,
    #[serde(rename = "@connectionId")]
    pub connection_id: i64,
    #[serde(rename = "xmlCellPr")]
    pub xml_cell_pr: XlsxXmlCellPr,
    #[serde(rename = "extLst", default)]
    pub ext_lst: Option<XlsxInnerXml>,
}

/// Stores the XML properties for the cell of a single cell xml table.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "xmlCellPr")]
pub struct XlsxXmlCellPr {
    #[serde(rename = "@id")]
    pub id: i64,
    #[serde(rename = "@uniqueName", default)]
    pub unique_name: Option<String>,
    #[serde(rename = "xmlPr", default)]
    pub xml_pr: Option<XlsxInnerXml>,
    #[serde(rename = "extLst", default)]
    pub ext_lst: Option<XlsxInnerXml>,
}

/// Directly maps the format settings of the table.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Table {
    pub t_id: i64,
    pub r_id: String,
    pub table_xml: String,
    pub range: String,
    pub name: String,
    pub style_name: String,
    pub show_column_stripes: bool,
    pub show_first_column: bool,
    pub show_header_row: Option<bool>,
    pub show_last_column: bool,
    pub show_row_stripes: Option<bool>,
}

/// Directly maps the auto filter settings.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutoFilterOptions {
    pub column: String,
    pub expression: String,
}
