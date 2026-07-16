//! Slicer and timeline parts.
//!
//! Ported from Go `xmlSlicers.go`.

use serde::{Deserialize, Serialize};

use super::common::{XlsxExtLst, XlsxInnerXml};

/// Directly maps the `slicers` element that specifies a slicer view on the
/// worksheet.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "slicers", rename_all = "camelCase")]
pub struct XlsxSlicers {
    #[serde(rename = "@xmlns:mc", default)]
    pub xmlns_mc: Option<String>,
    #[serde(rename = "@xmlns:x", default)]
    pub xmlns_x: Option<String>,
    #[serde(rename = "@xmlns:xr10", default)]
    pub xmlns_xr10: Option<String>,
    #[serde(default)]
    pub slicer: Vec<XlsxSlicer>,
}

/// A complex type that specifies a slicer view.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct XlsxSlicer {
    #[serde(rename = "@name", default)]
    pub name: String,
    #[serde(rename = "@xr10:uid", default)]
    pub xr10_uid: Option<String>,
    #[serde(rename = "@cache", default)]
    pub cache: String,
    #[serde(rename = "@caption", default)]
    pub caption: Option<String>,
    #[serde(rename = "@startItem", default)]
    pub start_item: Option<i64>,
    #[serde(rename = "@columnCount", default)]
    pub column_count: Option<i64>,
    #[serde(rename = "@showCaption", default)]
    pub show_caption: Option<bool>,
    #[serde(rename = "@level", default)]
    pub level: Option<i64>,
    #[serde(rename = "@style", default)]
    pub style: Option<String>,
    #[serde(rename = "@lockedPosition", default)]
    pub locked_position: Option<bool>,
    #[serde(rename = "@rowHeight", default)]
    pub row_height: i64,
}

/// Directly maps the `slicerCacheDefinition` element that specifies a slicer
/// cache.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "slicerCacheDefinition", rename_all = "camelCase")]
pub struct XlsxSlicerCacheDefinition {
    #[serde(rename = "@xmlns:mc", default)]
    pub xmlns_mc: Option<String>,
    #[serde(rename = "@xmlns:x", default)]
    pub xmlns_x: Option<String>,
    #[serde(rename = "@xmlns:x15", default)]
    pub xmlns_x15: Option<String>,
    #[serde(rename = "@xmlns:xr10", default)]
    pub xmlns_xr10: Option<String>,
    #[serde(rename = "@name", default)]
    pub name: String,
    #[serde(rename = "@xr10:uid", default)]
    pub xr10_uid: Option<String>,
    #[serde(rename = "@sourceName", default)]
    pub source_name: String,
    #[serde(default)]
    pub pivot_tables: Option<XlsxSlicerCachePivotTables>,
    #[serde(default)]
    pub data: Option<XlsxSlicerCacheData>,
    #[serde(default)]
    pub ext_lst: Option<XlsxExtLst>,
}

/// A complex type that specifies a group of pivotTable elements that specify
/// the PivotTable views that are filtered by the slicer cache.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "pivotTables", rename_all = "camelCase")]
pub struct XlsxSlicerCachePivotTables {
    #[serde(default)]
    pub pivot_table: Vec<XlsxSlicerCachePivotTable>,
}

/// A complex type that specifies a PivotTable view filtered by a slicer cache.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct XlsxSlicerCachePivotTable {
    #[serde(rename = "@tabId", default)]
    pub tab_id: i64,
    #[serde(rename = "@name", default)]
    pub name: String,
}

/// A complex type that specifies a data source for the slicer cache.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct XlsxSlicerCacheData {
    #[serde(default)]
    pub olap: Option<XlsxInnerXml>,
    #[serde(default)]
    pub tabular: Option<XlsxTabularSlicerCache>,
}

/// A complex type that specifies non-OLAP slicer items that are cached within
/// this slicer cache and properties of the slicer cache specific to non-OLAP
/// slicer items.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct XlsxTabularSlicerCache {
    #[serde(rename = "@pivotCacheId", default)]
    pub pivot_cache_id: i64,
    #[serde(rename = "@sortOrder", default)]
    pub sort_order: Option<String>,
    #[serde(rename = "@customListSort", default)]
    pub custom_list_sort: Option<bool>,
    #[serde(rename = "@showMissing", default)]
    pub show_missing: Option<bool>,
    #[serde(rename = "@crossFilter", default)]
    pub cross_filter: Option<String>,
    #[serde(default)]
    pub items: Option<XlsxTabularSlicerCacheItems>,
    #[serde(default)]
    pub ext_lst: Option<XlsxExtLst>,
}

/// A complex type that specifies non-OLAP slicer items that are cached within
/// this slicer cache.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "items", rename_all = "camelCase")]
pub struct XlsxTabularSlicerCacheItems {
    #[serde(rename = "@count", default)]
    pub count: Option<i64>,
    #[serde(default)]
    pub i: Vec<XlsxTabularSlicerCacheItem>,
}

/// A complex type that specifies a non-OLAP slicer item that is cached within
/// this slicer cache.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "i", rename_all = "camelCase")]
pub struct XlsxTabularSlicerCacheItem {
    #[serde(rename = "@x", default)]
    pub x: i64,
    #[serde(rename = "@s", default)]
    pub s: Option<bool>,
    #[serde(rename = "@nd", default)]
    pub nd: Option<bool>,
}

/// Specifies a table data source for the slicer cache.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "x15:tableSlicerCache", rename_all = "camelCase")]
pub struct XlsxTableSlicerCache {
    #[serde(rename = "@tableId", default)]
    pub table_id: i64,
    #[serde(rename = "@column", default)]
    pub column: i64,
    #[serde(rename = "@sortOrder", default)]
    pub sort_order: Option<String>,
    #[serde(rename = "@customListSort", default)]
    pub custom_list_sort: Option<bool>,
    #[serde(rename = "@crossFilter", default)]
    pub cross_filter: Option<String>,
    #[serde(default)]
    pub ext_lst: Option<XlsxExtLst>,
}

/// Specifies a list of slicer.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "x14:slicerList", rename_all = "camelCase")]
pub struct XlsxX14SlicerList {
    #[serde(default)]
    pub slicer: Vec<XlsxX14Slicer>,
}

/// Specifies a slicer view.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "x14:slicer")]
pub struct XlsxX14Slicer {
    #[serde(rename = "@r:id", default)]
    pub rid: String,
}

/// Directly maps the `x14:slicerCaches` element.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "x14:slicerCaches")]
pub struct XlsxX14SlicerCaches {
    #[serde(rename = "@xmlns:x14", default)]
    pub xmlns: String,
    #[serde(rename = "$value", default)]
    pub content: String,
}

/// Directly maps the `x14:slicerCache` element.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "x14:slicerCache")]
pub struct XlsxX14SlicerCache {
    #[serde(rename = "@r:id", default)]
    pub rid: String,
}

/// Directly maps the `x15:slicerCaches` element.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "x15:slicerCaches")]
pub struct XlsxX15SlicerCaches {
    #[serde(rename = "@xmlns:x14", default)]
    pub xmlns: String,
    #[serde(rename = "$value", default)]
    pub content: String,
}

/// Defines the structure used to parse the `x15:tableSlicerCache` element of
/// the table slicer cache.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "tableSlicerCache", rename_all = "camelCase")]
pub struct DecodeTableSlicerCache {
    #[serde(rename = "@tableId", default)]
    pub table_id: i64,
    #[serde(rename = "@column", default)]
    pub column: i64,
    #[serde(rename = "@sortOrder", default)]
    pub sort_order: String,
}

/// Defines the structure used to parse the `x14:slicerList` element of a list
/// of slicer.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "slicerList", rename_all = "camelCase")]
pub struct DecodeSlicerList {
    #[serde(default)]
    pub slicer: Vec<DecodeSlicer>,
}

/// Defines the structure used to parse the `x14:slicer` element of a slicer.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "slicer")]
pub struct DecodeSlicer {
    #[serde(rename = "@id", default)]
    pub rid: String,
}

/// Defines the structure used to parse the `x14:slicerCaches` and
/// `x15:slicerCaches` element of a slicer cache.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "slicerCaches")]
pub struct DecodeSlicerCaches {
    #[serde(rename = "$value", default)]
    pub content: String,
}

/// A mechanism for filtering data in pivot table views, cube functions and
/// charts based on non-worksheet pivot tables. In the case of using OLAP
/// Timeline source data, a Timeline is based on a key attribute of an OLAP
/// hierarchy. In the case of using native Timeline source data, a Timeline is
/// based on a data table column.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "timelines", rename_all = "camelCase")]
pub struct XlsxTimelines {
    #[serde(rename = "@xmlns:mc", default)]
    pub xmlns_mc: Option<String>,
    #[serde(rename = "@xmlns:x", default)]
    pub xmlns_x: Option<String>,
    #[serde(rename = "@xmlns:xr10", default)]
    pub xmlns_xr10: Option<String>,
    #[serde(default)]
    pub timeline: Vec<XlsxTimeline>,
}

/// Timeline view specifies the display of a timeline on a worksheet.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct XlsxTimeline {
    #[serde(rename = "@name", default)]
    pub name: String,
    #[serde(rename = "@xr10:uid", default)]
    pub xr10_uid: Option<String>,
    #[serde(rename = "@cache", default)]
    pub cache: String,
    #[serde(rename = "@caption", default)]
    pub caption: Option<String>,
    #[serde(rename = "@showHeader", default)]
    pub show_header: Option<bool>,
    #[serde(rename = "@showSelectionLabel", default)]
    pub show_selection_label: Option<bool>,
    #[serde(rename = "@showTimeLevel", default)]
    pub show_time_level: Option<bool>,
    #[serde(rename = "@showHorizontalScrollbar", default)]
    pub show_horizontal_scrollbar: Option<bool>,
    #[serde(rename = "@level", default)]
    pub level: i64,
    #[serde(rename = "@selectionLevel", default)]
    pub selection_level: i64,
    #[serde(rename = "@scrollPosition", default)]
    pub scroll_position: Option<String>,
    #[serde(rename = "@style", default)]
    pub style: Option<String>,
}
