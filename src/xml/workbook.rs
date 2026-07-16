//! Workbook part (`xl/workbook.xml`).
//!
//! Ported from Go `xmlWorkbook.go`.

use serde::{Deserialize, Serialize};

use super::common::{XlsxExtLst, XlsxInnerXml};

/// Relationships part (`_rels/*.rels`).
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "Relationships")]
pub struct XlsxRelationships {
    #[serde(rename = "Relationship", default)]
    pub relationships: Vec<XlsxRelationship>,
}

/// A single relationship.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "Relationship")]
pub struct XlsxRelationship {
    #[serde(rename = "@Id", default)]
    pub id: String,
    #[serde(rename = "@Target", default)]
    pub target: String,
    #[serde(rename = "@Type", default)]
    pub r#type: String,
    #[serde(
        rename = "@TargetMode",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub target_mode: Option<String>,
}

/// Workbook root element.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "workbook", rename_all = "PascalCase")]
pub struct XlsxWorkbook {
    #[serde(
        rename = "@conformance",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub conformance: Option<String>,
    #[serde(
        rename = "fileVersion",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub file_version: Option<XlsxFileVersion>,
    #[serde(
        rename = "fileSharing",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub file_sharing: Option<XlsxExtLst>,
    #[serde(
        rename = "workbookPr",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub workbook_pr: Option<XlsxWorkbookPr>,
    #[serde(
        rename = "mc:AlternateContent",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub alternate_content: Option<XlsxAlternateContent>,
    #[serde(
        rename = "http://schemas.openxmlformats.org/markup-compatibility/2006 AlternateContent",
        default,
        skip_serializing
    )]
    pub decode_alternate_content: Option<XlsxInnerXml>,
    #[serde(
        rename = "workbookProtection",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub workbook_protection: Option<XlsxWorkbookProtection>,
    #[serde(rename = "bookViews", default, skip_serializing_if = "Option::is_none")]
    pub book_views: Option<XlsxBookViews>,
    #[serde(rename = "sheets", default)]
    pub sheets: XlsxSheets,
    #[serde(
        rename = "functionGroups",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub function_groups: Option<XlsxFunctionGroups>,
    #[serde(
        rename = "externalReferences",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub external_references: Option<XlsxExternalReferences>,
    #[serde(
        rename = "definedNames",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub defined_names: Option<XlsxDefinedNames>,
    #[serde(rename = "calcPr", default, skip_serializing_if = "Option::is_none")]
    pub calc_pr: Option<XlsxCalcPr>,
    #[serde(rename = "oleSize", default, skip_serializing_if = "Option::is_none")]
    pub ole_size: Option<XlsxExtLst>,
    #[serde(
        rename = "customWorkbookViews",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub custom_workbook_views: Option<XlsxCustomWorkbookViews>,
    #[serde(
        rename = "pivotCaches",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub pivot_caches: Option<XlsxPivotCaches>,
    #[serde(
        rename = "smartTagPr",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub smart_tag_pr: Option<XlsxExtLst>,
    #[serde(
        rename = "smartTagTypes",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub smart_tag_types: Option<XlsxExtLst>,
    #[serde(
        rename = "webPublishing",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub web_publishing: Option<XlsxExtLst>,
    #[serde(
        rename = "fileRecoveryPr",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub file_recovery_pr: Option<XlsxFileRecoveryPr>,
    #[serde(
        rename = "webPublishObjects",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub web_publish_objects: Option<XlsxExtLst>,
    #[serde(rename = "extLst", default, skip_serializing_if = "Option::is_none")]
    pub ext_lst: Option<XlsxExtLst>,
}

/// Sheet recovery information.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XlsxFileRecoveryPr {
    #[serde(
        rename = "@autoRecover",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub auto_recover: Option<bool>,
    #[serde(
        rename = "@crashSave",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub crash_save: Option<bool>,
    #[serde(
        rename = "@dataExtractLoad",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub data_extract_load: Option<bool>,
    #[serde(
        rename = "@repairLoad",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub repair_load: Option<bool>,
}

/// Workbook protection settings.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XlsxWorkbookProtection {
    #[serde(
        rename = "@lockRevision",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub lock_revision: Option<bool>,
    #[serde(
        rename = "@lockStructure",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub lock_structure: Option<bool>,
    #[serde(
        rename = "@lockWindows",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub lock_windows: Option<bool>,
    #[serde(
        rename = "@revisionsAlgorithmName",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub revisions_algorithm_name: Option<String>,
    #[serde(
        rename = "@revisionsHashValue",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub revisions_hash_value: Option<String>,
    #[serde(
        rename = "@revisionsSaltValue",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub revisions_salt_value: Option<String>,
    #[serde(
        rename = "@revisionsSpinCount",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub revisions_spin_count: Option<i64>,
    #[serde(
        rename = "@workbookAlgorithmName",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub workbook_algorithm_name: Option<String>,
    #[serde(
        rename = "@workbookHashValue",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub workbook_hash_value: Option<String>,
    #[serde(
        rename = "@workbookSaltValue",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub workbook_salt_value: Option<String>,
    #[serde(
        rename = "@workbookSpinCount",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub workbook_spin_count: Option<i64>,
}

/// File version tracking.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XlsxFileVersion {
    #[serde(rename = "@appName", default, skip_serializing_if = "Option::is_none")]
    pub app_name: Option<String>,
    #[serde(rename = "@codeName", default, skip_serializing_if = "Option::is_none")]
    pub code_name: Option<String>,
    #[serde(
        rename = "@lastEdited",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub last_edited: Option<String>,
    #[serde(
        rename = "@lowestEdited",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub lowest_edited: Option<String>,
    #[serde(rename = "@rupBuild", default, skip_serializing_if = "Option::is_none")]
    pub rup_build: Option<String>,
}

/// Workbook properties.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct XlsxWorkbookPr {
    #[serde(rename = "@date1904", default, skip_serializing_if = "Option::is_none")]
    pub date1904: Option<bool>,
    #[serde(
        rename = "@showObjects",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub show_objects: Option<String>,
    #[serde(
        rename = "@showBorderUnselectedTables",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub show_border_unselected_tables: Option<bool>,
    #[serde(
        rename = "@filterPrivacy",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub filter_privacy: Option<bool>,
    #[serde(
        rename = "@promptedSolutions",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub prompted_solutions: Option<bool>,
    #[serde(
        rename = "@showInkAnnotation",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub show_ink_annotation: Option<bool>,
    #[serde(
        rename = "@backupFile",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub backup_file: Option<bool>,
    #[serde(
        rename = "@saveExternalLinkValues",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub save_external_link_values: Option<bool>,
    #[serde(
        rename = "@updateLinks",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub update_links: Option<String>,
    #[serde(rename = "@codeName", default, skip_serializing_if = "Option::is_none")]
    pub code_name: Option<String>,
    #[serde(
        rename = "@hidePivotFieldList",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub hide_pivot_field_list: Option<bool>,
    #[serde(
        rename = "@showPivotChartFilter",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub show_pivot_chart_filter: Option<bool>,
    #[serde(
        rename = "@allowRefreshQuery",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub allow_refresh_query: Option<bool>,
    #[serde(
        rename = "@publishItems",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub publish_items: Option<bool>,
    #[serde(
        rename = "@checkCompatibility",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub check_compatibility: Option<bool>,
    #[serde(
        rename = "@autoCompressPictures",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub auto_compress_pictures: Option<bool>,
    #[serde(
        rename = "@refreshAllConnections",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub refresh_all_connections: Option<bool>,
    #[serde(
        rename = "@defaultThemeVersion",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub default_theme_version: Option<String>,
}

/// Collection of workbook views.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct XlsxBookViews {
    #[serde(rename = "workbookView", default)]
    pub workbook_view: Vec<XlsxWorkBookView>,
}

/// A single workbook view.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct XlsxWorkBookView {
    #[serde(
        rename = "@visibility",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub visibility: Option<String>,
    #[serde(
        rename = "@minimized",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub minimized: Option<bool>,
    #[serde(
        rename = "@showHorizontalScroll",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub show_horizontal_scroll: Option<bool>,
    #[serde(
        rename = "@showVerticalScroll",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub show_vertical_scroll: Option<bool>,
    #[serde(
        rename = "@showSheetTabs",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub show_sheet_tabs: Option<bool>,
    #[serde(rename = "@xWindow", default, skip_serializing_if = "Option::is_none")]
    pub x_window: Option<String>,
    #[serde(rename = "@yWindow", default, skip_serializing_if = "Option::is_none")]
    pub y_window: Option<String>,
    #[serde(
        rename = "@windowWidth",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub window_width: Option<i64>,
    #[serde(
        rename = "@windowHeight",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub window_height: Option<i64>,
    #[serde(rename = "@tabRatio", default, skip_serializing_if = "Option::is_none")]
    pub tab_ratio: Option<f64>,
    #[serde(
        rename = "@firstSheet",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub first_sheet: Option<i64>,
    #[serde(
        rename = "@activeTab",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub active_tab: Option<i64>,
    #[serde(
        rename = "@autoFilterDateGrouping",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub auto_filter_date_grouping: Option<bool>,
}

/// Collection of sheets in the workbook.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XlsxSheets {
    #[serde(rename = "sheet", default)]
    pub sheet: Vec<XlsxSheet>,
}

/// A single sheet reference.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XlsxSheet {
    #[serde(rename = "@name", default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(rename = "@sheetId", default, skip_serializing_if = "Option::is_none")]
    pub sheet_id: Option<i64>,
    #[serde(rename = "@r:id", default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Fallback for namespaced `r:id` when only the local name survives parsing.
    #[serde(rename = "@id", default, skip_serializing_if = "Option::is_none")]
    pub plain_id: Option<String>,
    #[serde(rename = "@state", default, skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
}

/// A function group.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XlsxFunctionGroup {
    #[serde(rename = "@name", default)]
    pub name: String,
}

/// Collection of function groups.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XlsxFunctionGroups {
    #[serde(
        rename = "@builtInGroupCount",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub built_in_group_count: Option<i64>,
    #[serde(rename = "functionGroup", default)]
    pub function_group: Vec<XlsxFunctionGroup>,
}

/// External workbook references.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XlsxExternalReferences {
    #[serde(rename = "externalReference", default)]
    pub external_reference: Vec<XlsxExternalReference>,
}

/// A single external reference.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XlsxExternalReference {
    #[serde(rename = "@r:id", default, skip_serializing_if = "Option::is_none")]
    pub rid: Option<String>,
}

/// Pivot cache references.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XlsxPivotCaches {
    #[serde(rename = "pivotCache", default)]
    pub pivot_cache: Vec<XlsxPivotCache>,
}

/// A single pivot cache reference.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XlsxPivotCache {
    #[serde(rename = "@cacheId", default)]
    pub cache_id: i64,
    #[serde(rename = "@r:id", default, skip_serializing_if = "Option::is_none")]
    pub rid: Option<String>,
}

/// Markup compatibility alternate content container.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "mc:AlternateContent")]
pub struct XlsxAlternateContent {
    #[serde(rename = "@xmlns:mc", default, skip_serializing_if = "Option::is_none")]
    pub xmlns_mc: Option<String>,
    #[serde(rename = "$value", default)]
    pub content: String,
}

/// Choice element inside alternate content.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "mc:Choice")]
pub struct XlsxChoice {
    #[serde(
        rename = "@xmlns:a14",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub xmlns_a14: Option<String>,
    #[serde(
        rename = "@xmlns:sle15",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub xmlns_sle15: Option<String>,
    #[serde(rename = "@Requires", default, skip_serializing_if = "Option::is_none")]
    pub requires: Option<String>,
    #[serde(rename = "$value", default)]
    pub content: String,
}

/// Fallback element inside alternate content.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "mc:Fallback")]
pub struct XlsxFallback {
    #[serde(rename = "$value", default)]
    pub content: String,
}

/// Defined names container.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XlsxDefinedNames {
    #[serde(rename = "definedName", default)]
    pub defined_name: Vec<XlsxDefinedName>,
}

/// A single defined name.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XlsxDefinedName {
    #[serde(rename = "@comment", default, skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(
        rename = "@customMenu",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub custom_menu: Option<String>,
    #[serde(
        rename = "@description",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub description: Option<String>,
    #[serde(rename = "@function", default, skip_serializing_if = "Option::is_none")]
    pub function: Option<bool>,
    #[serde(
        rename = "@functionGroupId",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub function_group_id: Option<i64>,
    #[serde(rename = "@help", default, skip_serializing_if = "Option::is_none")]
    pub help: Option<String>,
    #[serde(rename = "@hidden", default, skip_serializing_if = "Option::is_none")]
    pub hidden: Option<bool>,
    #[serde(
        rename = "@localSheetId",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub local_sheet_id: Option<i64>,
    #[serde(rename = "@name", default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(
        rename = "@publishToServer",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub publish_to_server: Option<bool>,
    #[serde(
        rename = "@shortcutKey",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub shortcut_key: Option<String>,
    #[serde(
        rename = "@statusBar",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub status_bar: Option<String>,
    #[serde(
        rename = "@vbProcedure",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub vb_procedure: Option<bool>,
    #[serde(
        rename = "@workbookParameter",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub workbook_parameter: Option<bool>,
    #[serde(rename = "@xml", default, skip_serializing_if = "Option::is_none")]
    pub xlm: Option<bool>,
    #[serde(rename = "$value", default)]
    pub data: String,
}

/// Calculation properties.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct XlsxCalcPr {
    #[serde(
        rename = "@calcCompleted",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub calc_completed: Option<bool>,
    #[serde(rename = "@calcId", default, skip_serializing_if = "Option::is_none")]
    pub calc_id: Option<i64>,
    #[serde(rename = "@calcMode", default, skip_serializing_if = "Option::is_none")]
    pub calc_mode: Option<String>,
    #[serde(
        rename = "@calcOnSave",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub calc_on_save: Option<bool>,
    #[serde(
        rename = "@concurrentCalc",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub concurrent_calc: Option<bool>,
    #[serde(
        rename = "@concurrentManualCount",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub concurrent_manual_count: Option<i64>,
    #[serde(
        rename = "@forceFullCalc",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub force_full_calc: Option<bool>,
    #[serde(
        rename = "@fullCalcOnLoad",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub full_calc_on_load: Option<bool>,
    #[serde(
        rename = "@fullPrecision",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub full_precision: Option<bool>,
    #[serde(rename = "@iterate", default, skip_serializing_if = "Option::is_none")]
    pub iterate: Option<bool>,
    #[serde(
        rename = "@iterateCount",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub iterate_count: Option<i64>,
    #[serde(
        rename = "@iterateDelta",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub iterate_delta: Option<f64>,
    #[serde(rename = "@refMode", default, skip_serializing_if = "Option::is_none")]
    pub ref_mode: Option<String>,
}

/// Custom workbook views container.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct XlsxCustomWorkbookViews {
    #[serde(rename = "customWorkbookView", default)]
    pub custom_workbook_view: Vec<XlsxCustomWorkbookView>,
}

/// A single custom workbook view.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct XlsxCustomWorkbookView {
    #[serde(
        rename = "@activeSheetId",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub active_sheet_id: Option<i64>,
    #[serde(
        rename = "@autoUpdate",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub auto_update: Option<bool>,
    #[serde(
        rename = "@changesSavedWin",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub changes_saved_win: Option<bool>,
    #[serde(rename = "@guid", default, skip_serializing_if = "Option::is_none")]
    pub guid: Option<String>,
    #[serde(
        rename = "@includeHiddenRowCol",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub include_hidden_row_col: Option<bool>,
    #[serde(
        rename = "@includePrintSettings",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub include_print_settings: Option<bool>,
    #[serde(
        rename = "@maximized",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub maximized: Option<bool>,
    #[serde(rename = "@mergeInterval", default)]
    pub merge_interval: i64,
    #[serde(
        rename = "@minimized",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub minimized: Option<bool>,
    #[serde(rename = "@name", default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(rename = "@onlySync", default, skip_serializing_if = "Option::is_none")]
    pub only_sync: Option<bool>,
    #[serde(
        rename = "@personalView",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub personal_view: Option<bool>,
    #[serde(
        rename = "@showComments",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub show_comments: Option<String>,
    #[serde(
        rename = "@showFormulaBar",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub show_formula_bar: Option<bool>,
    #[serde(
        rename = "@showHorizontalScroll",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub show_horizontal_scroll: Option<bool>,
    #[serde(
        rename = "@showObjects",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub show_objects: Option<String>,
    #[serde(
        rename = "@showSheetTabs",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub show_sheet_tabs: Option<bool>,
    #[serde(
        rename = "@showStatusbar",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub show_statusbar: Option<bool>,
    #[serde(
        rename = "@showVerticalScroll",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub show_vertical_scroll: Option<bool>,
    #[serde(rename = "@tabRatio", default, skip_serializing_if = "Option::is_none")]
    pub tab_ratio: Option<f64>,
    #[serde(
        rename = "@windowHeight",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub window_height: Option<i64>,
    #[serde(
        rename = "@windowWidth",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub window_width: Option<i64>,
    #[serde(rename = "@xWindow", default, skip_serializing_if = "Option::is_none")]
    pub x_window: Option<i64>,
    #[serde(rename = "@yWindow", default, skip_serializing_if = "Option::is_none")]
    pub y_window: Option<i64>,
}

// ------------------------------------------------------------------
// Public API types
// ------------------------------------------------------------------

/// Defined name used in the public API.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DefinedName {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub comment: String,
    #[serde(rename = "RefersTo", default)]
    pub refers_to: String,
    #[serde(default)]
    pub scope: String,
}

/// Calculation properties options.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct CalcPropsOptions {
    #[serde(default)]
    pub calc_id: Option<u64>,
    #[serde(default)]
    pub calc_mode: Option<String>,
    #[serde(
        rename = "FullCalcOnLoad",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub full_calc_on_load: Option<bool>,
    #[serde(default)]
    pub ref_mode: Option<String>,
    #[serde(default)]
    pub iterate: Option<bool>,
    #[serde(default)]
    pub iterate_count: Option<u64>,
    #[serde(default)]
    pub iterate_delta: Option<f64>,
    #[serde(
        rename = "FullPrecision",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub full_precision: Option<bool>,
    #[serde(
        rename = "CalcCompleted",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub calc_completed: Option<bool>,
    #[serde(
        rename = "CalcOnSave",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub calc_on_save: Option<bool>,
    #[serde(
        rename = "ConcurrentCalc",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub concurrent_calc: Option<bool>,
    #[serde(
        rename = "ConcurrentManualCount",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub concurrent_manual_count: Option<u64>,
    #[serde(
        rename = "ForceFullCalc",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub force_full_calc: Option<bool>,
}

/// Workbook properties options.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkbookPropsOptions {
    #[serde(default)]
    pub date1904: Option<bool>,
    #[serde(
        rename = "FilterPrivacy",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub filter_privacy: Option<bool>,
    #[serde(default)]
    pub code_name: Option<String>,
}

/// Workbook protection options.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkbookProtectionOptions {
    #[serde(rename = "AlgorithmName", default)]
    pub algorithm_name: String,
    #[serde(default)]
    pub password: String,
    #[serde(rename = "LockStructure", default)]
    pub lock_structure: bool,
    #[serde(rename = "LockWindows", default)]
    pub lock_windows: bool,
}
