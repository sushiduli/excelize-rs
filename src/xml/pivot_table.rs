//! PivotTable definition part (`xl/pivotTables/pivotTableN.xml`).
//!
//! Ported from Go `xmlPivotTable.go`.

use serde::{Deserialize, Serialize};

use super::common::XlsxExtLst;

/// Directly maps the `pivotTableDefinition` root element.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "pivotTableDefinition")]
pub struct XlsxPivotTableDefinition {
    #[serde(rename = "@xmlns", default, skip_serializing_if = "Option::is_none")]
    pub xmlns: Option<String>,
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@cacheId")]
    pub cache_id: i64,
    #[serde(
        rename = "@applyNumberFormats",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub apply_number_formats: Option<bool>,
    #[serde(
        rename = "@applyBorderFormats",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub apply_border_formats: Option<bool>,
    #[serde(
        rename = "@applyFontFormats",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub apply_font_formats: Option<bool>,
    #[serde(
        rename = "@applyPatternFormats",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub apply_pattern_formats: Option<bool>,
    #[serde(
        rename = "@applyAlignmentFormats",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub apply_alignment_formats: Option<bool>,
    #[serde(
        rename = "@applyWidthHeightFormats",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub apply_width_height_formats: Option<bool>,
    #[serde(
        rename = "@dataOnRows",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub data_on_rows: Option<bool>,
    #[serde(
        rename = "@dataPosition",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub data_position: Option<i64>,
    #[serde(rename = "@dataCaption")]
    pub data_caption: String,
    #[serde(
        rename = "@grandTotalCaption",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub grand_total_caption: Option<String>,
    #[serde(
        rename = "@errorCaption",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub error_caption: Option<String>,
    #[serde(
        rename = "@showError",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub show_error: Option<bool>,
    #[serde(
        rename = "@missingCaption",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub missing_caption: Option<String>,
    #[serde(
        rename = "@showMissing",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub show_missing: Option<bool>,
    #[serde(
        rename = "@pageStyle",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub page_style: Option<String>,
    #[serde(
        rename = "@pivotTableStyle",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub pivot_table_style: Option<String>,
    #[serde(
        rename = "@vacatedStyle",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub vacated_style: Option<String>,
    #[serde(rename = "@tag", default, skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    #[serde(
        rename = "@updatedVersion",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub updated_version: Option<i64>,
    #[serde(
        rename = "@minRefreshableVersion",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub min_refreshable_version: Option<i64>,
    #[serde(
        rename = "@asteriskTotals",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub asterisk_totals: Option<bool>,
    #[serde(
        rename = "@showItems",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub show_items: Option<bool>,
    #[serde(rename = "@editData", default, skip_serializing_if = "Option::is_none")]
    pub edit_data: Option<bool>,
    #[serde(
        rename = "@disableFieldList",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub disable_field_list: Option<bool>,
    #[serde(
        rename = "@showCalcMbrs",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub show_calc_mbrs: Option<bool>,
    #[serde(
        rename = "@visualTotals",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub visual_totals: Option<bool>,
    #[serde(
        rename = "@showMultipleLabel",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub show_multiple_label: Option<bool>,
    #[serde(
        rename = "@showDataDropDown",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub show_data_drop_down: Option<bool>,
    #[serde(
        rename = "@showDrill",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub show_drill: Option<bool>,
    #[serde(
        rename = "@printDrill",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub print_drill: Option<bool>,
    #[serde(
        rename = "@showMemberPropertyTips",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub show_member_property_tips: Option<bool>,
    #[serde(
        rename = "@showDataTips",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub show_data_tips: Option<bool>,
    #[serde(
        rename = "@enableWizard",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub enable_wizard: Option<bool>,
    #[serde(
        rename = "@enableDrill",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub enable_drill: Option<bool>,
    #[serde(
        rename = "@enableFieldProperties",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub enable_field_properties: Option<bool>,
    #[serde(
        rename = "@preserveFormatting",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub preserve_formatting: Option<bool>,
    #[serde(
        rename = "@useAutoFormatting",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub use_auto_formatting: Option<bool>,
    #[serde(rename = "@pageWrap", default, skip_serializing_if = "Option::is_none")]
    pub page_wrap: Option<i64>,
    #[serde(
        rename = "@pageOverThenDown",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub page_over_then_down: Option<bool>,
    #[serde(
        rename = "@subtotalHiddenItems",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub subtotal_hidden_items: Option<bool>,
    #[serde(
        rename = "@rowGrandTotals",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub row_grand_totals: Option<bool>,
    #[serde(
        rename = "@colGrandTotals",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub col_grand_totals: Option<bool>,
    #[serde(
        rename = "@fieldPrintTitles",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub field_print_titles: Option<bool>,
    #[serde(
        rename = "@itemPrintTitles",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub item_print_titles: Option<bool>,
    #[serde(
        rename = "@mergeItem",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub merge_item: Option<bool>,
    #[serde(
        rename = "@showDropZones",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub show_drop_zones: Option<bool>,
    #[serde(
        rename = "@createdVersion",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub created_version: Option<i64>,
    #[serde(rename = "@indent", default, skip_serializing_if = "Option::is_none")]
    pub indent: Option<i64>,
    #[serde(
        rename = "@showEmptyRow",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub show_empty_row: Option<bool>,
    #[serde(
        rename = "@showEmptyCol",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub show_empty_col: Option<bool>,
    #[serde(
        rename = "@showHeaders",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub show_headers: Option<bool>,
    #[serde(rename = "@compact", default, skip_serializing_if = "Option::is_none")]
    pub compact: Option<bool>,
    #[serde(rename = "@outline", default, skip_serializing_if = "Option::is_none")]
    pub outline: Option<bool>,
    #[serde(
        rename = "@outlineData",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub outline_data: Option<bool>,
    #[serde(
        rename = "@compactData",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub compact_data: Option<bool>,
    #[serde(
        rename = "@published",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub published: Option<bool>,
    #[serde(
        rename = "@gridDropZones",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub grid_drop_zones: Option<bool>,
    #[serde(
        rename = "@immersive",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub immersive: Option<bool>,
    #[serde(
        rename = "@multipleFieldFilters",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub multiple_field_filters: Option<bool>,
    #[serde(
        rename = "@chartFormat",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub chart_format: Option<i64>,
    #[serde(
        rename = "@rowHeaderCaption",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub row_header_caption: Option<String>,
    #[serde(
        rename = "@colHeaderCaption",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub col_header_caption: Option<String>,
    #[serde(
        rename = "@fieldListSortAscending",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub field_list_sort_ascending: Option<bool>,
    #[serde(
        rename = "@mdxSubqueries",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub mdx_subqueries: Option<bool>,
    #[serde(
        rename = "@customListSort",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub custom_list_sort: Option<bool>,
    #[serde(rename = "location", default, skip_serializing_if = "Option::is_none")]
    pub location: Option<XlsxLocation>,
    #[serde(
        rename = "pivotFields",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub pivot_fields: Option<XlsxPivotFields>,
    #[serde(rename = "rowFields", default, skip_serializing_if = "Option::is_none")]
    pub row_fields: Option<XlsxRowFields>,
    #[serde(rename = "rowItems", default, skip_serializing_if = "Option::is_none")]
    pub row_items: Option<XlsxRowItems>,
    #[serde(rename = "colFields", default, skip_serializing_if = "Option::is_none")]
    pub col_fields: Option<XlsxColFields>,
    #[serde(rename = "colItems", default, skip_serializing_if = "Option::is_none")]
    pub col_items: Option<XlsxColItems>,
    #[serde(
        rename = "pageFields",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub page_fields: Option<XlsxPageFields>,
    #[serde(
        rename = "dataFields",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub data_fields: Option<XlsxDataFields>,
    #[serde(
        rename = "conditionalFormats",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub conditional_formats: Option<XlsxConditionalFormats>,
    #[serde(
        rename = "pivotTableStyleInfo",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub pivot_table_style_info: Option<XlsxPivotTableStyleInfo>,
}

/// Represents location information for the PivotTable.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "location")]
pub struct XlsxLocation {
    #[serde(rename = "@ref")]
    pub r#ref: String,
    #[serde(rename = "@firstHeaderRow")]
    pub first_header_row: i64,
    #[serde(rename = "@firstDataRow")]
    pub first_data_row: i64,
    #[serde(rename = "@firstDataCol")]
    pub first_data_col: i64,
    #[serde(
        rename = "@rowPageCount",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub row_page_count: Option<i64>,
    #[serde(
        rename = "@colPageCount",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub col_page_count: Option<i64>,
}

/// Represents the collection of fields that appear on the PivotTable.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "pivotFields")]
pub struct XlsxPivotFields {
    #[serde(rename = "@count")]
    pub count: i64,
    #[serde(rename = "pivotField", default)]
    pub pivot_field: Vec<XlsxPivotField>,
}

/// Represents a single field in the PivotTable.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "pivotField")]
pub struct XlsxPivotField {
    #[serde(rename = "@name", default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(rename = "@axis", default, skip_serializing_if = "Option::is_none")]
    pub axis: Option<String>,
    #[serde(
        rename = "@dataField",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub data_field: Option<bool>,
    #[serde(
        rename = "@subtotalCaption",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub subtotal_caption: Option<String>,
    #[serde(
        rename = "@showDropDowns",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub show_drop_downs: Option<bool>,
    #[serde(
        rename = "@hiddenLevel",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub hidden_level: Option<bool>,
    #[serde(
        rename = "@uniqueMemberProperty",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub unique_member_property: Option<String>,
    #[serde(rename = "@compact", default, skip_serializing_if = "Option::is_none")]
    pub compact: Option<bool>,
    #[serde(
        rename = "@allDrilled",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub all_drilled: Option<bool>,
    #[serde(rename = "@numFmtId", default, skip_serializing_if = "Option::is_none")]
    pub num_fmt_id: Option<String>,
    #[serde(rename = "@outline", default, skip_serializing_if = "Option::is_none")]
    pub outline: Option<bool>,
    #[serde(
        rename = "@subtotalTop",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub subtotal_top: Option<bool>,
    #[serde(
        rename = "@dragToRow",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub drag_to_row: Option<bool>,
    #[serde(
        rename = "@dragToCol",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub drag_to_col: Option<bool>,
    #[serde(
        rename = "@multipleItemSelectionAllowed",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub multiple_item_selection_allowed: Option<bool>,
    #[serde(
        rename = "@dragToPage",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub drag_to_page: Option<bool>,
    #[serde(
        rename = "@dragToData",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub drag_to_data: Option<bool>,
    #[serde(rename = "@dragOff", default, skip_serializing_if = "Option::is_none")]
    pub drag_off: Option<bool>,
    #[serde(rename = "@showAll", default, skip_serializing_if = "Option::is_none")]
    pub show_all: Option<bool>,
    #[serde(
        rename = "@insertBlankRow",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub insert_blank_row: Option<bool>,
    #[serde(
        rename = "@serverField",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub server_field: Option<bool>,
    #[serde(
        rename = "@insertPageBreak",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub insert_page_break: Option<bool>,
    #[serde(rename = "@autoShow", default, skip_serializing_if = "Option::is_none")]
    pub auto_show: Option<bool>,
    #[serde(
        rename = "@topAutoShow",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub top_auto_show: Option<bool>,
    #[serde(
        rename = "@hideNewItems",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub hide_new_items: Option<bool>,
    #[serde(
        rename = "@measureFilter",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub measure_filter: Option<bool>,
    #[serde(
        rename = "@includeNewItemsInFilter",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub include_new_items_in_filter: Option<bool>,
    #[serde(
        rename = "@itemPageCount",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub item_page_count: Option<i64>,
    #[serde(rename = "@sortType", default, skip_serializing_if = "Option::is_none")]
    pub sort_type: Option<String>,
    #[serde(
        rename = "@dataSourceSort",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub data_source_sort: Option<bool>,
    #[serde(
        rename = "@nonAutoSortDefault",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub non_auto_sort_default: Option<bool>,
    #[serde(rename = "@rankBy", default, skip_serializing_if = "Option::is_none")]
    pub rank_by: Option<i64>,
    #[serde(
        rename = "@defaultSubtotal",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub default_subtotal: Option<bool>,
    #[serde(
        rename = "@sumSubtotal",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub sum_subtotal: Option<bool>,
    #[serde(
        rename = "@countASubtotal",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub count_a_subtotal: Option<bool>,
    #[serde(
        rename = "@avgSubtotal",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub avg_subtotal: Option<bool>,
    #[serde(
        rename = "@maxSubtotal",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub max_subtotal: Option<bool>,
    #[serde(
        rename = "@minSubtotal",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub min_subtotal: Option<bool>,
    #[serde(
        rename = "@productSubtotal",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub product_subtotal: Option<bool>,
    #[serde(
        rename = "@countSubtotal",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub count_subtotal: Option<bool>,
    #[serde(
        rename = "@stdDevSubtotal",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub std_dev_subtotal: Option<bool>,
    #[serde(
        rename = "@stdDevPSubtotal",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub std_dev_p_subtotal: Option<bool>,
    #[serde(
        rename = "@varSubtotal",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub var_subtotal: Option<bool>,
    #[serde(
        rename = "@varPSubtotal",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub var_p_subtotal: Option<bool>,
    #[serde(
        rename = "@showPropCell",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub show_prop_cell: Option<bool>,
    #[serde(
        rename = "@showPropTip",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub show_prop_tip: Option<bool>,
    #[serde(
        rename = "@showPropAsCaption",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub show_prop_as_caption: Option<bool>,
    #[serde(
        rename = "@defaultAttributeDrillState",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub default_attribute_drill_state: Option<bool>,
    #[serde(rename = "items", default, skip_serializing_if = "Option::is_none")]
    pub items: Option<XlsxItems>,
    #[serde(
        rename = "autoSortScope",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub auto_sort_scope: Option<XlsxAutoSortScope>,
    #[serde(rename = "extLst", default, skip_serializing_if = "Option::is_none")]
    pub ext_lst: Option<XlsxExtLst>,
}

/// Represents the collection of items in a PivotTable field.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "items")]
pub struct XlsxItems {
    #[serde(rename = "@count")]
    pub count: i64,
    #[serde(rename = "item", default)]
    pub item: Vec<XlsxItem>,
}

/// Represents a single item in a PivotTable field.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "item")]
pub struct XlsxItem {
    #[serde(rename = "@n", default, skip_serializing_if = "Option::is_none")]
    pub n: Option<String>,
    #[serde(rename = "@t", default, skip_serializing_if = "Option::is_none")]
    pub t: Option<String>,
    #[serde(rename = "@h", default, skip_serializing_if = "Option::is_none")]
    pub h: Option<bool>,
    #[serde(rename = "@s", default, skip_serializing_if = "Option::is_none")]
    pub s: Option<bool>,
    #[serde(rename = "@sd", default, skip_serializing_if = "Option::is_none")]
    pub sd: Option<bool>,
    #[serde(rename = "@f", default, skip_serializing_if = "Option::is_none")]
    pub f: Option<bool>,
    #[serde(rename = "@m", default, skip_serializing_if = "Option::is_none")]
    pub m: Option<bool>,
    #[serde(rename = "@c", default, skip_serializing_if = "Option::is_none")]
    pub c: Option<bool>,
    #[serde(rename = "@x", default, skip_serializing_if = "Option::is_none")]
    pub x: Option<i64>,
    #[serde(rename = "@d", default, skip_serializing_if = "Option::is_none")]
    pub d: Option<bool>,
    #[serde(rename = "@e", default, skip_serializing_if = "Option::is_none")]
    pub e: Option<bool>,
}

/// Represents the sorting scope for the PivotTable.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "autoSortScope")]
pub struct XlsxAutoSortScope {}

/// Represents the collection of row fields for the PivotTable.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "rowFields")]
pub struct XlsxRowFields {
    #[serde(rename = "@count")]
    pub count: i64,
    #[serde(rename = "field", default)]
    pub field: Vec<XlsxField>,
}

/// Represents a generic field that can appear on the column or row region.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "field")]
pub struct XlsxField {
    #[serde(rename = "@x")]
    pub x: i64,
}

/// Represents the collection of items in the row axis of the PivotTable.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "rowItems")]
pub struct XlsxRowItems {
    #[serde(rename = "@count")]
    pub count: i64,
    #[serde(rename = "i", default)]
    pub i: Vec<XlsxI>,
}

/// Represents the collection of items in the row region of the PivotTable.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "i")]
pub struct XlsxI {
    #[serde(rename = "x", default)]
    pub x: Vec<XlsxX>,
}

/// Represents an array of indexes to cached shared item values.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "x")]
pub struct XlsxX {}

/// Represents the collection of fields on the column axis of the PivotTable.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "colFields")]
pub struct XlsxColFields {
    #[serde(rename = "@count")]
    pub count: i64,
    #[serde(rename = "field", default)]
    pub field: Vec<XlsxField>,
}

/// Represents the collection of column items of the PivotTable.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "colItems")]
pub struct XlsxColItems {
    #[serde(rename = "@count")]
    pub count: i64,
    #[serde(rename = "i", default)]
    pub i: Vec<XlsxI>,
}

/// Represents the collection of items in the page or report filter region.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "pageFields")]
pub struct XlsxPageFields {
    #[serde(rename = "@count")]
    pub count: i64,
    #[serde(rename = "pageField", default)]
    pub page_field: Vec<XlsxPageField>,
}

/// Represents a field on the page or report filter of the PivotTable.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "pageField")]
pub struct XlsxPageField {
    #[serde(rename = "@fld")]
    pub fld: i64,
    #[serde(rename = "@item", default, skip_serializing_if = "Option::is_none")]
    pub item: Option<i64>,
    #[serde(rename = "@hier", default, skip_serializing_if = "Option::is_none")]
    pub hier: Option<i64>,
    #[serde(rename = "@name", default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(rename = "@cap", default, skip_serializing_if = "Option::is_none")]
    pub cap: Option<String>,
    #[serde(rename = "extLst", default, skip_serializing_if = "Option::is_none")]
    pub ext_lst: Option<XlsxExtLst>,
}

/// Represents the collection of items in the data region of the PivotTable.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "dataFields")]
pub struct XlsxDataFields {
    #[serde(rename = "@count")]
    pub count: i64,
    #[serde(rename = "dataField", default)]
    pub data_field: Vec<XlsxDataField>,
}

/// Represents a field that contains data summarized in a PivotTable.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "dataField")]
pub struct XlsxDataField {
    #[serde(rename = "@name", default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(rename = "@fld")]
    pub fld: i64,
    #[serde(rename = "@subtotal", default, skip_serializing_if = "Option::is_none")]
    pub subtotal: Option<String>,
    #[serde(
        rename = "@showDataAs",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub show_data_as: Option<String>,
    #[serde(
        rename = "@baseField",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub base_field: Option<i64>,
    #[serde(rename = "@baseItem", default, skip_serializing_if = "Option::is_none")]
    pub base_item: Option<i64>,
    #[serde(rename = "@numFmtId", default, skip_serializing_if = "Option::is_none")]
    pub num_fmt_id: Option<i64>,
    #[serde(rename = "extLst", default, skip_serializing_if = "Option::is_none")]
    pub ext_lst: Option<XlsxExtLst>,
}

/// Specifies extended information about a data field in the pivot table.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "x14:dataField")]
pub struct XlsxX14DataField {
    #[serde(
        rename = "@xmlns:x14",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub xmlns_x14: Option<String>,
    #[serde(
        rename = "@pivotShowAs",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub pivot_show_as: Option<String>,
    #[serde(
        rename = "@sourceField",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub source_field: Option<i64>,
    #[serde(
        rename = "@uniqueName",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub unique_name: Option<String>,
}

/// Represents the collection of conditional formats applied to a PivotTable.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "conditionalFormats")]
pub struct XlsxConditionalFormats {}

/// Represents information on style applied to the PivotTable.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "pivotTableStyleInfo")]
pub struct XlsxPivotTableStyleInfo {
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@showRowHeaders")]
    pub show_row_headers: bool,
    #[serde(rename = "@showColHeaders")]
    pub show_col_headers: bool,
    #[serde(
        rename = "@showRowStripes",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub show_row_stripes: Option<bool>,
    #[serde(
        rename = "@showColStripes",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub show_col_stripes: Option<bool>,
    #[serde(
        rename = "@showLastColumn",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub show_last_column: Option<bool>,
}

/// Defines the structure used to parse the `x14:dataField` element of the pivot
/// table data field.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "dataField")]
pub struct DecodeX14DataField {
    #[serde(
        rename = "@pivotShowAs",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub pivot_show_as: Option<String>,
    #[serde(
        rename = "@sourceField",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub source_field: Option<i64>,
    #[serde(
        rename = "@uniqueName",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub unique_name: Option<String>,
}
