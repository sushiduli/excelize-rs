//! Constants and default values used across the crate.
//!
//! These values are ported from the Go `excelize` package, primarily from
//! `templates.go`, `errors.go` and other global constant blocks.

/// XML header used when serializing package parts.
pub const XML_HEADER: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#;

// ------------------------------------------------------------------
// Excel specifications and limits
// ------------------------------------------------------------------

/// EMU (English Metric Units) per pixel/point conversion factor.
pub const EMU: i32 = 9525;
pub const MAX_CELL_STYLES: i32 = 65430;
pub const MAX_COLUMNS: i32 = 16384;
pub const MAX_COLUMN_WIDTH: i32 = 255;
pub const MAX_FIELD_LENGTH: usize = 255;
pub const MAX_FILE_PATH_LENGTH: usize = 207;
pub const MAX_FORM_CONTROL_VALUE: i32 = 30000;
pub const MAX_FONT_FAMILY_LENGTH: usize = 31;
pub const MAX_GRAPHIC_ALT_TEXT_LENGTH: usize = 65535;
pub const MAX_GRAPHIC_NAME_LENGTH: usize = 254;
pub const MAX_FONT_SIZE: i32 = 409;
pub const MAX_ROW_HEIGHT: i32 = 409;
pub const MAX_SHEET_NAME_LENGTH: usize = 31;
pub const MIN_COLUMNS: i32 = 1;
pub const MIN_FONT_SIZE: i32 = 1;
pub const STREAM_CHUNK_SIZE: i64 = 1 << 24;
pub const TOTAL_CELL_CHARS: usize = 32767;
pub const TOTAL_ROWS: i32 = 1048576;
pub const TOTAL_SHEET_HYPERLINKS: i32 = 65529;
pub const UNZIP_SIZE_LIMIT: i64 = 1000 << 24;

// ------------------------------------------------------------------
// Default path constants for package parts
// ------------------------------------------------------------------

pub const DEFAULT_XML_PATH_RELS: &str = "_rels/.rels";
pub const DEFAULT_XML_PATH_CONTENT_TYPES: &str = "[Content_Types].xml";
pub const DEFAULT_XML_PATH_DOC_PROPS_APP: &str = "docProps/app.xml";
pub const DEFAULT_XML_PATH_DOC_PROPS_CORE: &str = "docProps/core.xml";
pub const DEFAULT_XML_PATH_DOC_PROPS_CUSTOM: &str = "docProps/custom.xml";
pub const DEFAULT_XML_PATH_WORKBOOK: &str = "xl/workbook.xml";
pub const DEFAULT_XML_PATH_WORKBOOK_RELS: &str = "xl/_rels/workbook.xml.rels";
pub const DEFAULT_XML_PATH_SHEET: &str = "xl/worksheets/sheet1.xml";
pub const DEFAULT_XML_PATH_STYLES: &str = "xl/styles.xml";
pub const DEFAULT_XML_PATH_THEME: &str = "xl/theme/theme1.xml";
pub const DEFAULT_XML_PATH_SHARED_STRINGS: &str = "xl/sharedStrings.xml";
pub const DEFAULT_XML_PATH_CALC_CHAIN: &str = "xl/calcChain.xml";
pub const DEFAULT_XML_PATH_VOLATILE_DEPS: &str = "xl/volatileDeps.xml";
pub const DEFAULT_XML_PATH_METADATA: &str = "xl/metadata.xml";
pub const DEFAULT_XML_PATH_RD_RICH_VALUE: &str = "xl/richData/rdrichvalue.xml";
pub const DEFAULT_XML_PATH_RD_RICH_VALUE_REL: &str = "xl/richData/richValueRel.xml";
pub const DEFAULT_XML_PATH_RD_RICH_VALUE_STRUCTURE: &str = "xl/richData/rdrichvaluestructure.xml";
/// Legacy part name used before 0.1.8; kept for reading older files.
pub const DEFAULT_XML_PATH_RD_RICH_VALUE_STRUCTURE_LEGACY: &str =
    "xl/richData/rdRichValueStructure.xml";
pub const DEFAULT_XML_PATH_RD_RICH_VALUE_WEB_IMAGE: &str = "xl/richData/rdRichValueWebImage.xml";
pub const DEFAULT_XML_PATH_RD_RICH_VALUE_REL_RELS: &str = "xl/richData/_rels/richValueRel.xml.rels";
pub const DEFAULT_XML_PATH_RD_RICH_VALUE_WEB_IMAGE_RELS: &str =
    "xl/richData/_rels/rdRichValueWebImage.xml.rels";
pub const DEFAULT_XML_PATH_CELL_IMAGES: &str = "xl/cellimages.xml";
pub const DEFAULT_XML_PATH_CELL_IMAGES_RELS: &str = "xl/_rels/cellimages.xml.rels";

// ------------------------------------------------------------------
// OLE / Compound document identifier
// ------------------------------------------------------------------

pub const OLE_IDENTIFIER: &[u8] = &[0xd0, 0xcf, 0x11, 0xe0, 0xa1, 0xb1, 0x1a, 0xe1];

// ------------------------------------------------------------------
// Namespaces
// ------------------------------------------------------------------

pub const NAMESPACE_SPREADSHEET: &str = "http://schemas.openxmlformats.org/spreadsheetml/2006/main";
pub const NAMESPACE_RICH_DATA: &str = "http://schemas.microsoft.com/office/spreadsheetml/2017/richdata";
pub const NAMESPACE_RICH_DATA_2: &str =
    "http://schemas.microsoft.com/office/spreadsheetml/2017/richdata2";
pub const NAMESPACE_RICH_VALUE_REL: &str =
    "http://schemas.microsoft.com/office/spreadsheetml/2022/richvaluerel";
pub const NAMESPACE_WPS_ET_CUSTOM_DATA: &str = "http://www.wps.cn/officeDocument/2017/etCustomData";
pub const NAMESPACE_DRAWING_ML_MAIN: &str = "http://schemas.openxmlformats.org/drawingml/2006/main";
pub const NAMESPACE_EXTENDED_PROPERTIES: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/extended-properties";
pub const NAMESPACE_DOCUMENT_PROPERTIES_VARIANT_TYPES: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/docPropsVTypes";
pub const NAMESPACE_DRAWING_2016_SVG: &str =
    "http://schemas.microsoft.com/office/drawing/2016/SVG/main";
pub const NAMESPACE_DRAWING_ML_A14: &str = "http://schemas.microsoft.com/office/drawing/2010/main";
pub const NAMESPACE_DRAWING_ML_CHART: &str =
    "http://schemas.openxmlformats.org/drawingml/2006/chart";
pub const NAMESPACE_DRAWING_ML_SLICER: &str =
    "http://schemas.microsoft.com/office/drawing/2010/slicer";
pub const NAMESPACE_DRAWING_ML_SLICER_X15: &str =
    "http://schemas.microsoft.com/office/drawing/2012/slicer";
pub const NAMESPACE_DRAWING_ML_SPREADSHEET: &str =
    "http://schemas.openxmlformats.org/drawingml/2006/spreadsheetDrawing";
pub const NAMESPACE_MAC_EXCEL_2008_MAIN: &str =
    "http://schemas.microsoft.com/office/mac/excel/2008/main";
pub const NAMESPACE_SPREADSHEET_EXCEL_2006_MAIN: &str =
    "http://schemas.microsoft.com/office/excel/2006/main";
pub const NAMESPACE_SPREADSHEET_X14: &str =
    "http://schemas.microsoft.com/office/spreadsheetml/2009/9/main";
pub const NAMESPACE_SPREADSHEET_X15: &str =
    "http://schemas.microsoft.com/office/spreadsheetml/2010/11/main";
pub const NAMESPACE_SPREADSHEET_XR10: &str =
    "http://schemas.microsoft.com/office/spreadsheetml/2016/revision10";
pub const NAMESPACE_DUBLIN_CORE: &str = "http://purl.org/dc/elements/1.1/";
pub const NAMESPACE_DUBLIN_CORE_TERMS: &str = "http://purl.org/dc/terms/";
pub const NAMESPACE_DUBLIN_CORE_METADATA_INITIATIVE: &str = "http://purl.org/dc/dcmitype/";
pub const NAMESPACE_XML: &str = "http://www.w3.org/XML/1998/namespace";
pub const NAMESPACE_XML_SCHEMA_INSTANCE: &str = "http://www.w3.org/2001/XMLSchema-instance";

// ------------------------------------------------------------------
// Relationship type URIs
// ------------------------------------------------------------------

pub const SOURCE_RELATIONSHIP: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships";
pub const SOURCE_RELATIONSHIP_CHART: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/chart";
pub const SOURCE_RELATIONSHIP_CHARTSHEET: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/chartsheet";
pub const SOURCE_RELATIONSHIP_COMMENTS: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/comments";
pub const SOURCE_RELATIONSHIP_CUSTOM_PROPERTIES: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/custom-properties";
pub const SOURCE_RELATIONSHIP_DIALOGSHEET: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/dialogsheet";
pub const SOURCE_RELATIONSHIP_DRAWING_ML: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/drawing";
pub const SOURCE_RELATIONSHIP_DRAWING_VML: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/vmlDrawing";
pub const SOURCE_RELATIONSHIP_EXTEND_PROPERTIES: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/extended-properties";
pub const SOURCE_RELATIONSHIP_HYPER_LINK: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/hyperlink";
pub const SOURCE_RELATIONSHIP_IMAGE: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/image";
pub const SOURCE_RELATIONSHIP_OFFICE_DOCUMENT: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument";
pub const SOURCE_RELATIONSHIP_PIVOT_CACHE: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/pivotCacheDefinition";
pub const SOURCE_RELATIONSHIP_PIVOT_TABLE: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/pivotTable";
pub const SOURCE_RELATIONSHIP_SHARED_STRINGS: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/sharedStrings";
pub const SOURCE_RELATIONSHIP_SLICER: &str =
    "http://schemas.microsoft.com/office/2007/relationships/slicer";
pub const SOURCE_RELATIONSHIP_SLICER_CACHE: &str =
    "http://schemas.microsoft.com/office/2007/relationships/slicerCache";
pub const SOURCE_RELATIONSHIP_TABLE: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/table";
pub const SOURCE_RELATIONSHIP_VBA_PROJECT: &str =
    "http://schemas.microsoft.com/office/2006/relationships/vbaProject";
pub const SOURCE_RELATIONSHIP_WORKSHEET: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet";
pub const SOURCE_RELATIONSHIP_SHEET_METADATA: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/sheetMetadata";
pub const SOURCE_RELATIONSHIP_CELL_IMAGES: &str =
    "http://www.wps.cn/officeDocument/2020/cellImage";
pub const SOURCE_RELATIONSHIP_RD_RICH_VALUE: &str =
    "http://schemas.microsoft.com/office/2017/06/relationships/rdRichValue";
pub const SOURCE_RELATIONSHIP_RD_RICH_VALUE_STRUCTURE: &str =
    "http://schemas.microsoft.com/office/2017/06/relationships/rdRichValueStructure";
pub const SOURCE_RELATIONSHIP_RICH_VALUE_REL: &str =
    "http://schemas.microsoft.com/office/2022/10/relationships/richValueRel";

// ------------------------------------------------------------------
// Strict namespaces (for Strict Open XML → Transitional conversion)
// ------------------------------------------------------------------

pub const STRICT_NAMESPACE_DOCUMENT_PROPERTIES_VARIANT_TYPES: &str =
    "http://purl.oclc.org/ooxml/officeDocument/docPropsVTypes";
pub const STRICT_NAMESPACE_DRAWING_ML_MAIN: &str = "http://purl.oclc.org/ooxml/drawingml/main";
pub const STRICT_NAMESPACE_EXTENDED_PROPERTIES: &str =
    "http://purl.oclc.org/ooxml/officeDocument/extendedProperties";
pub const STRICT_NAMESPACE_SPREADSHEET: &str = "http://purl.oclc.org/ooxml/spreadsheetml/main";
pub const STRICT_SOURCE_RELATIONSHIP: &str =
    "http://purl.oclc.org/ooxml/officeDocument/relationships";
pub const STRICT_SOURCE_RELATIONSHIP_CHART: &str =
    "http://purl.oclc.org/ooxml/officeDocument/relationships/chart";
pub const STRICT_SOURCE_RELATIONSHIP_COMMENTS: &str =
    "http://purl.oclc.org/ooxml/officeDocument/relationships/comments";
pub const STRICT_SOURCE_RELATIONSHIP_EXTEND_PROPERTIES: &str =
    "http://purl.oclc.org/ooxml/officeDocument/relationships/extendedProperties";
pub const STRICT_SOURCE_RELATIONSHIP_IMAGE: &str =
    "http://purl.oclc.org/ooxml/officeDocument/relationships/image";
pub const STRICT_SOURCE_RELATIONSHIP_OFFICE_DOCUMENT: &str =
    "http://purl.oclc.org/ooxml/officeDocument/relationships/officeDocument";

// ------------------------------------------------------------------
// Content types
// ------------------------------------------------------------------

pub const CONTENT_TYPE_ADDIN_MACRO: &str = "application/vnd.ms-excel.addin.macroEnabled.main+xml";
pub const CONTENT_TYPE_CUSTOM_PROPERTIES: &str =
    "application/vnd.openxmlformats-officedocument.custom-properties+xml";
pub const CONTENT_TYPE_DRAWING: &str = "application/vnd.openxmlformats-officedocument.drawing+xml";
pub const CONTENT_TYPE_DRAWING_ML: &str =
    "application/vnd.openxmlformats-officedocument.drawingml.chart+xml";
pub const CONTENT_TYPE_MACRO: &str = "application/vnd.ms-excel.sheet.macroEnabled.main+xml";
pub const CONTENT_TYPE_RELATIONSHIPS: &str =
    "application/vnd.openxmlformats-package.relationships+xml";
pub const CONTENT_TYPE_SHEET_ML: &str =
    "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml";
pub const CONTENT_TYPE_SLICER: &str = "application/vnd.ms-excel.slicer+xml";
pub const CONTENT_TYPE_SLICER_CACHE: &str = "application/vnd.ms-excel.slicerCache+xml";
pub const CONTENT_TYPE_SPREADSHEET_ML_CHARTSHEET: &str =
    "application/vnd.openxmlformats-officedocument.spreadsheetml.chartsheet+xml";
pub const CONTENT_TYPE_SPREADSHEET_ML_COMMENTS: &str =
    "application/vnd.openxmlformats-officedocument.spreadsheetml.comments+xml";
pub const CONTENT_TYPE_SPREADSHEET_ML_PIVOT_CACHE_DEFINITION: &str =
    "application/vnd.openxmlformats-officedocument.spreadsheetml.pivotCacheDefinition+xml";
pub const CONTENT_TYPE_SPREADSHEET_ML_PIVOT_TABLE: &str =
    "application/vnd.openxmlformats-officedocument.spreadsheetml.pivotTable+xml";
pub const CONTENT_TYPE_SPREADSHEET_ML_SHARED_STRINGS: &str =
    "application/vnd.openxmlformats-officedocument.spreadsheetml.sharedStrings+xml";
pub const CONTENT_TYPE_SPREADSHEET_ML_TABLE: &str =
    "application/vnd.openxmlformats-officedocument.spreadsheetml.table+xml";
pub const CONTENT_TYPE_SPREADSHEET_ML_WORKSHEET: &str =
    "application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml";
pub const CONTENT_TYPE_TEMPLATE: &str =
    "application/vnd.openxmlformats-officedocument.spreadsheetml.template.main+xml";
pub const CONTENT_TYPE_TEMPLATE_MACRO: &str =
    "application/vnd.ms-excel.template.macroEnabled.main+xml";
pub const CONTENT_TYPE_VBA: &str = "application/vnd.ms-office.vbaProject";
pub const CONTENT_TYPE_VML: &str = "application/vnd.openxmlformats-officedocument.vmlDrawing";
pub const CONTENT_TYPE_SHEET_METADATA: &str =
    "application/vnd.openxmlformats-officedocument.spreadsheetml.sheetMetadata+xml";
pub const CONTENT_TYPE_WPS_CELL_IMAGES: &str = "application/vnd.wps-officedocument.cellimage+xml";
pub const CONTENT_TYPE_RD_RICH_VALUE: &str = "application/vnd.ms-excel.rdrichvalue+xml";
pub const CONTENT_TYPE_RD_RICH_VALUE_STRUCTURE: &str =
    "application/vnd.ms-excel.rdrichvaluestructure+xml";
pub const CONTENT_TYPE_RICH_VALUE_REL: &str = "application/vnd.ms-excel.richvaluerel+xml";

// ------------------------------------------------------------------
// Extension list URIs
// ------------------------------------------------------------------

pub const EXT_URI_CALC_FEATURES: &str = "{B58B0392-4F1F-4190-BB64-5DF3571DCE5F}";
pub const EXT_URI_CONDITIONAL_FORMATTING_RULE_ID: &str = "{B025F937-C7B1-47D3-B67F-A62EFF666E3E}";
pub const EXT_URI_CONDITIONAL_FORMATTINGS: &str = "{78C0D931-6437-407d-A8EE-F0AAD7539E65}";
pub const EXT_URI_CUSTOM_PROPERTY_FMT_ID: &str = "{D5CDD505-2E9C-101B-9397-08002B2CF9AE}";
pub const EXT_URI_DATA_FIELD: &str = "{E15A36E0-9728-4E99-A89B-3F7291B0FE68}";
pub const EXT_URI_DATA_MODEL: &str = "{FCE2AD5D-F65C-4FA6-A056-5C36A1767C68}";
pub const EXT_URI_DATA_VALIDATIONS: &str = "{CCE6A557-97BC-4b89-ADB6-D9C93CAAB3DF}";
pub const EXT_URI_DRAWING_BLIP: &str = "{28A0092B-C50C-407E-A947-70E740481C1C}";
pub const EXT_URI_EXTERNAL_LINK_PR: &str = "{FCE6A71B-6B00-49CD-AB44-F6B1AE7CDE65}";
pub const EXT_URI_IGNORED_ERRORS: &str = "{01252117-D84E-4E92-8308-4BE1C098FCBB}";
pub const EXT_URI_MAC_EXCEL_MX: &str = "{64002731-A6B0-56B0-2670-7721B7C09600}";
pub const EXT_URI_MODEL_TIME_GROUPINGS: &str = "{9835A34E-60A6-4A7C-AAB8-D5F71C897F49}";
pub const EXT_URI_PIVOT_CACHE_DEFINITION: &str = "{725AE2AE-9491-48be-B2B4-4EB974FC3084}";
pub const EXT_URI_PIVOT_CACHES_X14: &str = "{876F7934-8845-4945-9796-88D515C7AA90}";
pub const EXT_URI_PIVOT_CACHES_X15: &str = "{841E416B-1EF1-43b6-AB56-02D37102CBD5}";
pub const EXT_URI_PIVOT_DATA_FIELD: &str = "{E15A36E0-9728-4e99-A89B-3F7291B0FE68}";
pub const EXT_URI_PIVOT_FIELD: &str = "{2946ED86-A175-432a-8AC1-64E0C546D7DE}";
pub const EXT_URI_PIVOT_FILTER: &str = "{0605FD5F-26C8-4aeb-8148-2DB25E43C511}";
pub const EXT_URI_PIVOT_HIERARCHY: &str = "{F1805F06-0CD304483-9156-8803C3D141DF}";
pub const EXT_URI_PIVOT_TABLE_REFERENCES: &str = "{983426D0-5260-488c-9760-48F4B6AC55F4}";
pub const EXT_URI_PROTECTED_RANGES: &str = "{FC87AEE6-9EDD-4A0A-B7FB-166176984837}";
pub const EXT_URI_SLICER_CACHE_DEFINITION: &str = "{2F2917AC-EB37-4324-AD4E-5DD8C200BD13}";
pub const EXT_URI_SLICER_CACHE_HIDE_ITEMS_WITH_NO_DATA: &str =
    "{470722E0-AACD-4C17-9CDC-17EF765DBC7E}";
pub const EXT_URI_SLICER_CACHES_X14: &str = "{BBE1A952-AA13-448e-AADC-164F8A28A991}";
pub const EXT_URI_SLICER_CACHES_X15: &str = "{46BE6895-7355-4a93-B00E-2C351335B9C9}";
pub const EXT_URI_SLICER_LIST_X14: &str = "{A8765BA9-456A-4dab-B4F3-ACF838C121DE}";
pub const EXT_URI_SLICER_LIST_X15: &str = "{3A4CF648-6AED-40f4-86FF-DC5316D8AED3}";
pub const EXT_URI_SPARKLINE_GROUPS: &str = "{05C60535-1F16-4fd2-B633-F4F36F0B64E0}";
pub const EXT_URI_SVG: &str = "{96DAC541-7B7A-43D3-8B79-37D633B846F1}";
pub const EXT_URI_TIMELINE_CACHE_PIVOT_CACHES: &str = "{A2CB5862-8E78-49c6-8D9D-AF26E26ADB89}";
pub const EXT_URI_TIMELINE_CACHE_REFS: &str = "{D0CA8CA8-9F24-4464-BF8E-62219DCF47F9}";
pub const EXT_URI_TIMELINE_REFS: &str = "{7E03D99C-DC04-49d9-9315-930204A7B6E9}";
pub const EXT_URI_WEB_EXTENSIONS: &str = "{F7C9EE02-42E1-4005-9D12-6889AFFD525C}";
pub const EXT_URI_WORKBOOK_PR_X14: &str = "{79F54976-1DA5-4618-B147-ACDE4B953A38}";
pub const EXT_URI_WORKBOOK_PR_X15: &str = "{140A7094-0E35-4892-8432-C4D2E57EDEB5}";

// ------------------------------------------------------------------
// Extension URI priority lists
// ------------------------------------------------------------------

pub const WORKBOOK_EXT_URI_PRIORITY: &[&str] = &[
    EXT_URI_PIVOT_CACHES_X14,
    EXT_URI_SLICER_CACHES_X14,
    EXT_URI_SLICER_CACHES_X15,
    EXT_URI_WORKBOOK_PR_X14,
    EXT_URI_PIVOT_CACHES_X15,
    EXT_URI_PIVOT_TABLE_REFERENCES,
    EXT_URI_TIMELINE_CACHE_PIVOT_CACHES,
    EXT_URI_TIMELINE_CACHE_REFS,
    EXT_URI_WORKBOOK_PR_X15,
    EXT_URI_DATA_MODEL,
    EXT_URI_CALC_FEATURES,
    EXT_URI_EXTERNAL_LINK_PR,
    EXT_URI_MODEL_TIME_GROUPINGS,
];

pub const WORKSHEET_EXT_URI_PRIORITY: &[&str] = &[
    EXT_URI_CONDITIONAL_FORMATTINGS,
    EXT_URI_DATA_VALIDATIONS,
    EXT_URI_SPARKLINE_GROUPS,
    EXT_URI_SLICER_LIST_X14,
    EXT_URI_PROTECTED_RANGES,
    EXT_URI_IGNORED_ERRORS,
    EXT_URI_WEB_EXTENSIONS,
    EXT_URI_SLICER_LIST_X15,
    EXT_URI_TIMELINE_REFS,
    EXT_URI_EXTERNAL_LINK_PR,
];

// ------------------------------------------------------------------
// Default numeric values
// ------------------------------------------------------------------

pub const PIVOT_TABLE_VERSION: i32 = 3;
pub const PIVOT_TABLE_REFRESHED_VERSION: i32 = 8;
pub const DEFAULT_DRAWING_SCALE: f64 = 1.0;
pub const DEFAULT_CHART_DIMENSION_WIDTH: i32 = 480;
pub const DEFAULT_CHART_DIMENSION_HEIGHT: i32 = 260;
pub const DEFAULT_SLICER_WIDTH: i32 = 200;
pub const DEFAULT_SLICER_HEIGHT: i32 = 200;
pub const DEFAULT_CHART_LEGEND_POSITION: &str = "bottom";
pub const DEFAULT_CHART_SHOW_BLANKS_AS: &str = "gap";
pub const DEFAULT_SHAPE_SIZE: i32 = 160;
pub const DEFAULT_LINE_WIDTH: i32 = 1;
pub const DEFAULT_COL_WIDTH: f64 = 9.140625;
pub const DEFAULT_COL_WIDTH_PIXELS: f64 = 64.0;
pub const DEFAULT_ROW_HEIGHT: f64 = 15.0;
pub const DEFAULT_ROW_HEIGHT_PIXELS: f64 = 20.0;
pub const DEFAULT_FONT_SIZE: f64 = 11.0;

// ------------------------------------------------------------------
// Defined name character code ranges
// ------------------------------------------------------------------

pub const SUPPORTED_DEFINED_NAME_AT_START_CHAR_CODE_RANGE: &[(u32, u32)] = &[
    (65, 90),
    (92, 92),
    (95, 95),
    (97, 122),
    (161, 161),
    (164, 164),
    (167, 168),
    (170, 170),
    (173, 173),
    (175, 186),
    (188, 696),
    (699, 705),
    (711, 711),
    (713, 715),
    (717, 717),
    (720, 721),
    (728, 731),
    (733, 733),
    (736, 740),
    (750, 750),
    (880, 883),
    (886, 887),
    (890, 893),
    (902, 902),
    (904, 906),
    (908, 908),
    (910, 929),
    (931, 1013),
    (1015, 1153),
    (1162, 1315),
    (1329, 1366),
    (1369, 1369),
    (1377, 1415),
    (1488, 1514),
    (1520, 1522),
    (1569, 1610),
    (1646, 1647),
    (1649, 1747),
    (1749, 1749),
    (1765, 1766),
    (1774, 1775),
    (1786, 1788),
    (1791, 1791),
    (1808, 1808),
    (1810, 1839),
    (1869, 1957),
    (1969, 1969),
    (1994, 2026),
    (2036, 2037),
    (2042, 2042),
    (2308, 2361),
    (2365, 2365),
    (2384, 2384),
    (2392, 2401),
    (2417, 2418),
    (2427, 2431),
    (2437, 2444),
    (2447, 2448),
    (2451, 2472),
    (2474, 2480),
    (2482, 2482),
    (2486, 2489),
    (2493, 2493),
    (2510, 2510),
    (2524, 2525),
    (2527, 2529),
    (2544, 2545),
    (2565, 2570),
    (2575, 2576),
    (2579, 2600),
    (2602, 2608),
    (2610, 2611),
    (2613, 2614),
    (2616, 2617),
    (2649, 2652),
    (2654, 2654),
    (2674, 2676),
    (2693, 2701),
    (2703, 2705),
    (2707, 2728),
    (2730, 2736),
    (2738, 2739),
    (2741, 2745),
    (2749, 2749),
    (2768, 2768),
    (2784, 2785),
    (2821, 2828),
    (2831, 2832),
    (2835, 2856),
    (2858, 2864),
    (2866, 2867),
    (2869, 2873),
    (2877, 2877),
    (2908, 2909),
    (2911, 2913),
    (2929, 2929),
    (2947, 2947),
    (2949, 2954),
    (2958, 2960),
    (2962, 2965),
    (2969, 2970),
    (2972, 2972),
    (2974, 2975),
    (2979, 2980),
    (2984, 2986),
    (2990, 3001),
    (3024, 3024),
    (3077, 3084),
    (3086, 3088),
    (3090, 3112),
    (3114, 3123),
    (3125, 3129),
    (3133, 3133),
    (3160, 3161),
    (3168, 3169),
    (3205, 3212),
    (3214, 3216),
    (3218, 3240),
    (3242, 3251),
    (3253, 3257),
    (3261, 3261),
    (3294, 3294),
    (3296, 3297),
    (3333, 3340),
    (3342, 3344),
    (3346, 3368),
    (3370, 3385),
    (3389, 3389),
    (3424, 3425),
    (3450, 3455),
    (3461, 3478),
    (3482, 3505),
    (3507, 3515),
    (3517, 3517),
    (3520, 3526),
    (3585, 3642),
    (3648, 3662),
    (3713, 3714),
    (3716, 3716),
    (3719, 3720),
    (3722, 3722),
    (3725, 3725),
    (3732, 3735),
    (3737, 3743),
    (3745, 3747),
    (3749, 3749),
    (3751, 3751),
    (3754, 3755),
    (3757, 3760),
    (3762, 3763),
    (3773, 3773),
    (3776, 3780),
    (3782, 3782),
    (3804, 3805),
    (3840, 3840),
    (3904, 3911),
    (3913, 3948),
    (3976, 3979),
    (4096, 4138),
    (4159, 4159),
    (4176, 4181),
    (4186, 4189),
    (4193, 4193),
    (4197, 4198),
    (4206, 4208),
    (4213, 4225),
    (4238, 4238),
    (4256, 4293),
    (4304, 4346),
    (4348, 4348),
    (4352, 4441),
    (4447, 4514),
    (4520, 4601),
    (4608, 4680),
    (4682, 4685),
    (4688, 4694),
    (4696, 4696),
    (4698, 4701),
    (4704, 4744),
    (4746, 4749),
    (4752, 4784),
    (4786, 4789),
    (4792, 4798),
    (4800, 4800),
    (4802, 4805),
    (4808, 4822),
    (4824, 4880),
    (4882, 4885),
    (4888, 4954),
    (4992, 5007),
    (5024, 5108),
    (5121, 5740),
    (5743, 5750),
    (5761, 5786),
    (5792, 5866),
    (5870, 5872),
    (5888, 5900),
    (5902, 5905),
    (5920, 5937),
    (5952, 5969),
    (5984, 5996),
    (5998, 6000),
    (6016, 6067),
    (6103, 6103),
    (6108, 6108),
    (6176, 6263),
    (6272, 6312),
    (6314, 6314),
    (6400, 6428),
    (6480, 6509),
    (6512, 6516),
    (6528, 6569),
    (6593, 6599),
    (6656, 6678),
    (6917, 6963),
    (6981, 6987),
    (7043, 7072),
    (7086, 7087),
    (7168, 7203),
    (7245, 7247),
    (7258, 7293),
    (7424, 7615),
    (7680, 7957),
    (7960, 7965),
    (7968, 8005),
    (8008, 8013),
    (8016, 8023),
    (8025, 8025),
    (8027, 8027),
    (8029, 8029),
    (8031, 8061),
    (8064, 8116),
    (8118, 8124),
    (8126, 8126),
    (8130, 8132),
    (8134, 8140),
    (8144, 8147),
    (8150, 8155),
    (8160, 8172),
    (8178, 8180),
    (8182, 8188),
    (8208, 8208),
    (8211, 8214),
    (8216, 8216),
    (8220, 8221),
    (8224, 8225),
    (8229, 8231),
    (8240, 8240),
    (8242, 8243),
    (8245, 8245),
    (8251, 8251),
    (8305, 8305),
    (8308, 8308),
    (8319, 8319),
    (8321, 8324),
    (8336, 8340),
    (8450, 8451),
    (8453, 8453),
    (8455, 8455),
    (8457, 8467),
    (8469, 8470),
    (8473, 8477),
    (8481, 8482),
    (8484, 8484),
    (8486, 8486),
    (8488, 8488),
    (8490, 8493),
    (8495, 8505),
    (8508, 8511),
    (8517, 8521),
    (8526, 8526),
    (8531, 8532),
    (8539, 8542),
    (8544, 8584),
    (8592, 8601),
    (8658, 8658),
    (8660, 8660),
    (8704, 8704),
    (8706, 8707),
    (8711, 8712),
    (8715, 8715),
    (8719, 8719),
    (8721, 8721),
    (8725, 8725),
    (8730, 8730),
    (8733, 8736),
    (8739, 8739),
    (8741, 8741),
    (8743, 8748),
    (8750, 8750),
    (8756, 8759),
    (8764, 8765),
    (8776, 8776),
    (8780, 8780),
    (8786, 8786),
    (8800, 8801),
    (8804, 8807),
    (8810, 8811),
    (8814, 8815),
    (8834, 8835),
    (8838, 8839),
    (8853, 8853),
    (8857, 8857),
    (8869, 8869),
    (8895, 8895),
    (8978, 8978),
    (9312, 9397),
    (9424, 9449),
    (9472, 9547),
    (9552, 9588),
    (9601, 9615),
    (9618, 9621),
    (9632, 9633),
    (9635, 9641),
    (9650, 9651),
    (9654, 9655),
    (9660, 9661),
    (9664, 9665),
    (9670, 9672),
    (9675, 9675),
    (9678, 9681),
    (9698, 9701),
    (9711, 9711),
    (9733, 9734),
    (9737, 9737),
    (9742, 9743),
    (9756, 9756),
    (9758, 9758),
    (9792, 9792),
    (9794, 9794),
    (9824, 9825),
    (9827, 9829),
    (9831, 9834),
    (9836, 9837),
    (9839, 9839),
    (11264, 11310),
    (11312, 11358),
    (11360, 11375),
    (11377, 11389),
    (11392, 11492),
    (11520, 11557),
    (11568, 11621),
    (11631, 11631),
    (11648, 11670),
    (11680, 11686),
    (11688, 11694),
    (11696, 11702),
    (11704, 11710),
    (11712, 11718),
    (11720, 11726),
    (11728, 11734),
    (11736, 11742),
    (12288, 12291),
    (12293, 12311),
    (12317, 12319),
    (12321, 12329),
    (12337, 12341),
    (12344, 12348),
    (12353, 12438),
    (12443, 12447),
    (12449, 12543),
    (12549, 12589),
    (12593, 12686),
    (12704, 12727),
    (12784, 12828),
    (12832, 12841),
    (12849, 12850),
    (12857, 12857),
    (12896, 12923),
    (12927, 12927),
    (12963, 12968),
    (13059, 13059),
    (13069, 13069),
    (13076, 13076),
    (13080, 13080),
    (13090, 13091),
    (13094, 13095),
    (13099, 13099),
    (13110, 13110),
    (13115, 13115),
    (13129, 13130),
    (13133, 13133),
    (13137, 13137),
    (13143, 13143),
    (13179, 13182),
    (13184, 13188),
    (13192, 13258),
    (13261, 13267),
    (13269, 13270),
    (13272, 13272),
    (13275, 13277),
    (13312, 19893),
    (19968, 40899),
    (40960, 42124),
    (42240, 42508),
    (42512, 42527),
    (42538, 42539),
    (42560, 42591),
    (42594, 42606),
    (42624, 42647),
    (42786, 42887),
    (42891, 42892),
    (43003, 43009),
    (43011, 43013),
    (43015, 43018),
    (43020, 43042),
    (43072, 43123),
    (43138, 43187),
    (43274, 43301),
    (43312, 43334),
    (43520, 43560),
    (43584, 43586),
    (43588, 43595),
    (44032, 55203),
    (57344, 63560),
    (63744, 64045),
    (64048, 64106),
    (64112, 64217),
    (64256, 64262),
    (64275, 64279),
    (64285, 64285),
    (64287, 64296),
    (64298, 64310),
    (64312, 64316),
    (64318, 64318),
    (64320, 64321),
    (64323, 64324),
    (64326, 64433),
    (64467, 64829),
    (64848, 64911),
    (64914, 64967),
    (65008, 65019),
    (65072, 65073),
    (65075, 65092),
    (65097, 65106),
    (65108, 65111),
    (65113, 65126),
    (65128, 65131),
    (65136, 65140),
    (65142, 65276),
    (65281, 65374),
    (65377, 65470),
    (65474, 65479),
    (65482, 65487),
    (65490, 65495),
    (65498, 65500),
    (65504, 65510),
];

pub const SUPPORTED_DEFINED_NAME_AFTER_START_CHAR_CODE_RANGE: &[(u32, u32)] = &[
    (46, 46),
    (48, 57),
    (63, 63),
    (65, 90),
    (92, 92),
    (95, 95),
    (97, 122),
    (161, 161),
    (164, 164),
    (167, 168),
    (170, 170),
    (173, 173),
    (175, 186),
    (188, 887),
    (890, 893),
    (900, 902),
    (904, 906),
    (908, 908),
    (910, 929),
    (931, 1315),
    (1329, 1366),
    (1369, 1369),
    (1377, 1415),
    (1425, 1469),
    (1471, 1471),
    (1473, 1474),
    (1476, 1477),
    (1479, 1479),
    (1488, 1514),
    (1520, 1522),
    (1536, 1539),
    (1542, 1544),
    (1547, 1547),
    (1550, 1562),
    (1567, 1567),
    (1569, 1630),
    (1632, 1641),
    (1646, 1747),
    (1749, 1791),
    (1807, 1866),
    (1869, 1969),
    (1984, 2038),
    (2042, 2042),
    (2305, 2361),
    (2364, 2381),
    (2384, 2388),
    (2392, 2403),
    (2406, 2415),
    (2417, 2418),
    (2427, 2431),
    (2433, 2435),
    (2437, 2444),
    (2447, 2448),
    (2451, 2472),
    (2474, 2480),
    (2482, 2482),
    (2486, 2489),
    (2492, 2500),
    (2503, 2504),
    (2507, 2510),
    (2519, 2519),
    (2524, 2525),
    (2527, 2531),
    (2534, 2554),
    (2561, 2563),
    (2565, 2570),
    (2575, 2576),
    (2579, 2600),
    (2602, 2608),
    (2610, 2611),
    (2613, 2614),
    (2616, 2617),
    (2620, 2620),
    (2622, 2626),
    (2631, 2632),
    (2635, 2637),
    (2641, 2641),
    (2649, 2652),
    (2654, 2654),
    (2662, 2677),
    (2689, 2691),
    (2693, 2701),
    (2703, 2705),
    (2707, 2728),
    (2730, 2736),
    (2738, 2739),
    (2741, 2745),
    (2748, 2757),
    (2759, 2761),
    (2763, 2765),
    (2768, 2768),
    (2784, 2787),
    (2790, 2799),
    (2801, 2801),
    (2817, 2819),
    (2821, 2828),
    (2831, 2832),
    (2835, 2856),
    (2858, 2864),
    (2866, 2867),
    (2869, 2873),
    (2876, 2884),
    (2887, 2888),
    (2891, 2893),
    (2902, 2903),
    (2908, 2909),
    (2911, 2915),
    (2918, 2929),
    (2946, 2947),
    (2949, 2954),
    (2958, 2960),
    (2962, 2965),
    (2969, 2970),
    (2972, 2972),
    (2974, 2975),
    (2979, 2980),
    (2984, 2986),
    (2990, 3001),
    (3006, 3010),
    (3014, 3016),
    (3018, 3021),
    (3024, 3024),
    (3031, 3031),
    (3046, 3066),
    (3073, 3075),
    (3077, 3084),
    (3086, 3088),
    (3090, 3112),
    (3114, 3123),
    (3125, 3129),
    (3133, 3140),
    (3142, 3144),
    (3146, 3149),
    (3157, 3158),
    (3160, 3161),
    (3168, 3171),
    (3174, 3183),
    (3192, 3199),
    (3202, 3203),
    (3205, 3212),
    (3214, 3216),
    (3218, 3240),
    (3242, 3251),
    (3253, 3257),
    (3260, 3268),
    (3270, 3272),
    (3274, 3277),
    (3285, 3286),
    (3294, 3294),
    (3296, 3299),
    (3302, 3311),
    (3313, 3314),
    (3330, 3331),
    (3333, 3340),
    (3342, 3344),
    (3346, 3368),
    (3370, 3385),
    (3389, 3396),
    (3398, 3400),
    (3402, 3405),
    (3415, 3415),
    (3424, 3427),
    (3430, 3445),
    (3449, 3455),
    (3458, 3459),
    (3461, 3478),
    (3482, 3505),
    (3507, 3515),
    (3517, 3517),
    (3520, 3526),
    (3530, 3530),
    (3535, 3540),
    (3542, 3542),
    (3544, 3551),
    (3570, 3571),
    (3585, 3642),
    (3647, 3662),
    (3664, 3673),
    (3713, 3714),
    (3716, 3716),
    (3719, 3720),
    (3722, 3722),
    (3725, 3725),
    (3732, 3735),
    (3737, 3743),
    (3745, 3747),
    (3749, 3749),
    (3751, 3751),
    (3754, 3755),
    (3757, 3769),
    (3771, 3773),
    (3776, 3780),
    (3782, 3782),
    (3784, 3789),
    (3792, 3801),
    (3804, 3805),
    (3840, 3843),
    (3859, 3897),
    (3902, 3911),
    (3913, 3948),
    (3953, 3972),
    (3974, 3979),
    (3984, 3991),
    (3993, 4028),
    (4030, 4044),
    (4046, 4047),
    (4096, 4169),
    (4176, 4249),
    (4254, 4293),
    (4304, 4346),
    (4348, 4348),
    (4352, 4441),
    (4447, 4514),
    (4520, 4601),
    (4608, 4680),
    (4682, 4685),
    (4688, 4694),
    (4696, 4696),
    (4698, 4701),
    (4704, 4744),
    (4746, 4749),
    (4752, 4784),
    (4786, 4789),
    (4792, 4798),
    (4800, 4800),
    (4802, 4805),
    (4808, 4822),
    (4824, 4880),
    (4882, 4885),
    (4888, 4954),
    (4959, 4960),
    (4969, 4988),
    (4992, 5017),
    (5024, 5108),
    (5121, 5740),
    (5743, 5750),
    (5760, 5786),
    (5792, 5866),
    (5870, 5872),
    (5888, 5900),
    (5902, 5908),
    (5920, 5940),
    (5952, 5971),
    (5984, 5996),
    (5998, 6000),
    (6002, 6003),
    (6016, 6099),
    (6103, 6103),
    (6107, 6109),
    (6112, 6121),
    (6128, 6137),
    (6155, 6158),
    (6160, 6169),
    (6176, 6263),
    (6272, 6314),
    (6400, 6428),
    (6432, 6443),
    (6448, 6459),
    (6464, 6464),
    (6470, 6509),
    (6512, 6516),
    (6528, 6569),
    (6576, 6601),
    (6608, 6617),
    (6624, 6683),
    (6912, 6987),
    (6992, 7001),
    (7009, 7036),
    (7040, 7082),
    (7086, 7097),
    (7168, 7223),
    (7232, 7241),
    (7245, 7293),
    (7424, 7654),
    (7678, 7957),
    (7960, 7965),
    (7968, 8005),
    (8008, 8013),
    (8016, 8023),
    (8025, 8025),
    (8027, 8027),
    (8029, 8029),
    (8031, 8061),
    (8064, 8116),
    (8118, 8132),
    (8134, 8147),
    (8150, 8155),
    (8157, 8175),
    (8178, 8180),
    (8182, 8190),
    (8192, 8208),
    (8211, 8214),
    (8216, 8216),
    (8220, 8221),
    (8224, 8225),
    (8229, 8240),
    (8242, 8243),
    (8245, 8245),
    (8251, 8251),
    (8260, 8260),
    (8274, 8274),
    (8287, 8292),
    (8298, 8305),
    (8308, 8316),
    (8319, 8332),
    (8336, 8340),
    (8352, 8373),
    (8400, 8432),
    (8448, 8527),
    (8531, 8584),
    (8592, 9000),
    (9003, 9191),
    (9216, 9254),
    (9280, 9290),
    (9312, 9885),
    (9888, 9916),
    (9920, 9923),
    (9985, 9988),
    (9990, 9993),
    (9996, 10023),
    (10025, 10059),
    (10061, 10061),
    (10063, 10066),
    (10070, 10070),
    (10072, 10078),
    (10081, 10087),
    (10102, 10132),
    (10136, 10159),
    (10161, 10174),
    (10176, 10180),
    (10183, 10186),
    (10188, 10188),
    (10192, 10213),
    (10224, 10626),
    (10649, 10711),
    (10716, 10747),
    (10750, 11084),
    (11088, 11092),
    (11264, 11310),
    (11312, 11358),
    (11360, 11375),
    (11377, 11389),
    (11392, 11498),
    (11517, 11517),
    (11520, 11557),
    (11568, 11621),
    (11631, 11631),
    (11648, 11670),
    (11680, 11686),
    (11688, 11694),
    (11696, 11702),
    (11704, 11710),
    (11712, 11718),
    (11720, 11726),
    (11728, 11734),
    (11736, 11742),
    (11744, 11775),
    (11823, 11823),
    (11904, 11929),
    (11931, 12019),
    (12032, 12245),
    (12272, 12283),
    (12288, 12311),
    (12317, 12335),
    (12337, 12348),
    (12350, 12351),
    (12353, 12438),
    (12441, 12447),
    (12449, 12543),
    (12549, 12589),
    (12593, 12686),
    (12688, 12727),
    (12736, 12771),
    (12784, 12830),
    (12832, 12867),
    (12880, 13054),
    (13056, 19893),
    (19904, 40899),
    (40960, 42124),
    (42128, 42182),
    (42240, 42508),
    (42512, 42539),
    (42560, 42591),
    (42594, 42610),
    (42620, 42621),
    (42623, 42647),
    (42752, 42892),
    (43003, 43051),
    (43072, 43123),
    (43136, 43204),
    (43216, 43225),
    (43264, 43310),
    (43312, 43347),
    (43520, 43574),
    (43584, 43597),
    (43600, 43609),
    (44032, 55203),
    (55296, 64045),
    (64048, 64106),
    (64112, 64217),
    (64256, 64262),
    (64275, 64279),
    (64285, 64310),
    (64312, 64316),
    (64318, 64318),
    (64320, 64321),
    (64323, 64324),
    (64326, 64433),
    (64467, 64829),
    (64848, 64911),
    (64914, 64967),
    (65008, 65021),
    (65024, 65039),
    (65056, 65062),
    (65072, 65073),
    (65075, 65092),
    (65097, 65106),
    (65108, 65111),
    (65113, 65126),
    (65128, 65131),
    (65136, 65140),
    (65142, 65276),
    (65279, 65279),
    (65281, 65374),
    (65377, 65470),
    (65474, 65479),
    (65482, 65487),
    (65490, 65495),
    (65498, 65500),
    (65504, 65510),
    (65512, 65518),
    (65529, 65533),
];

/// Built-in defined name for the auto-filter database.
pub const BUILT_IN_DEFINED_NAME_FILTER_DATABASE: &str = "_xlnm._FilterDatabase";
