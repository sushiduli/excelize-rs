//! Pivot cache definition part (`xl/pivotCache/pivotCacheDefinitionN.xml`).
//!
//! Ported from Go `xmlPivotCache.go`.

use serde::{Deserialize, Serialize};

use super::common::{AttrValInt, XlsxExtLst};

/// Directly maps the `pivotCacheDefinition` element.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "pivotCacheDefinition")]
pub struct XlsxPivotCacheDefinition {
    #[serde(rename = "@xmlns", default, skip_serializing_if = "Option::is_none")]
    pub xmlns: Option<String>,
    #[serde(rename = "@xmlns:r", default, skip_serializing_if = "Option::is_none")]
    pub xmlns_r: Option<String>,
    #[serde(rename = "@r:id", default, skip_serializing_if = "Option::is_none")]
    pub rid: Option<String>,
    #[serde(rename = "@invalid", default, skip_serializing_if = "Option::is_none")]
    pub invalid: Option<bool>,
    #[serde(rename = "@saveData", default)]
    pub save_data: bool,
    #[serde(
        rename = "@refreshOnLoad",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub refresh_on_load: Option<bool>,
    #[serde(
        rename = "@optimizeMemory",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub optimize_memory: Option<bool>,
    #[serde(
        rename = "@enableRefresh",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub enable_refresh: Option<bool>,
    #[serde(
        rename = "@refreshedBy",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub refreshed_by: Option<String>,
    #[serde(
        rename = "@refreshedDate",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub refreshed_date: Option<f64>,
    #[serde(
        rename = "@refreshedDateIso",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub refreshed_date_iso: Option<f64>,
    #[serde(rename = "@backgroundQuery", default)]
    pub background_query: bool,
    #[serde(
        rename = "@missingItemsLimit",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub missing_items_limit: Option<i32>,
    #[serde(
        rename = "@createdVersion",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub created_version: Option<i32>,
    #[serde(
        rename = "@refreshedVersion",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub refreshed_version: Option<i32>,
    #[serde(
        rename = "@minRefreshableVersion",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub min_refreshable_version: Option<i32>,
    #[serde(
        rename = "@recordCount",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub record_count: Option<i32>,
    #[serde(
        rename = "@upgradeOnRefresh",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub upgrade_on_refresh: Option<bool>,
    #[serde(
        rename = "@tupleCache",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub tuple_cache_attr: Option<bool>,
    #[serde(
        rename = "@supportSubquery",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub support_subquery: Option<bool>,
    #[serde(
        rename = "@supportAdvancedDrill",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub support_advanced_drill: Option<bool>,
    #[serde(
        rename = "cacheSource",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub cache_source: Option<XlsxCacheSource>,
    #[serde(
        rename = "cacheFields",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub cache_fields: Option<XlsxCacheFields>,
    #[serde(
        rename = "cacheHierarchies",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub cache_hierarchies: Option<XlsxCacheHierarchies>,
    #[serde(rename = "kpis", default, skip_serializing_if = "Option::is_none")]
    pub kpis: Option<XlsxKpis>,
    #[serde(
        rename = "tupleCache",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub tuple_cache: Option<XlsxTupleCache>,
    #[serde(
        rename = "calculatedItems",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub calculated_items: Option<XlsxCalculatedItems>,
    #[serde(
        rename = "calculatedMembers",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub calculated_members: Option<XlsxCalculatedMembers>,
    #[serde(
        rename = "dimensions",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub dimensions: Option<XlsxDimensions>,
    #[serde(
        rename = "measureGroups",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub measure_groups: Option<XlsxMeasureGroups>,
    #[serde(rename = "maps", default, skip_serializing_if = "Option::is_none")]
    pub maps: Option<XlsxMaps>,
    #[serde(rename = "extLst", default, skip_serializing_if = "Option::is_none")]
    pub ext_lst: Option<XlsxExtLst>,
}

/// Description of the data source whose data is stored in the pivot cache.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct XlsxCacheSource {
    #[serde(rename = "@type", default)]
    pub r#type: String,
    #[serde(
        rename = "@connectionId",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub connection_id: Option<i32>,
    #[serde(
        rename = "worksheetSource",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub worksheet_source: Option<XlsxWorksheetSource>,
    #[serde(
        rename = "consolidation",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub consolidation: Option<XlsxConsolidation>,
    #[serde(rename = "extLst", default, skip_serializing_if = "Option::is_none")]
    pub ext_lst: Option<XlsxExtLst>,
}

/// Location of the source of the data stored in the cache.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XlsxWorksheetSource {
    #[serde(rename = "@r:id", default, skip_serializing_if = "Option::is_none")]
    pub rid: Option<String>,
    #[serde(rename = "@ref", default, skip_serializing_if = "Option::is_none")]
    pub r#ref: Option<String>,
    #[serde(rename = "@name", default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(rename = "@sheet", default, skip_serializing_if = "Option::is_none")]
    pub sheet: Option<String>,
}

/// Description of the PivotCache source using multiple consolidation ranges.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XlsxConsolidation {}

/// Collection of field definitions in the source data.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct XlsxCacheFields {
    #[serde(rename = "@count", default)]
    pub count: i32,
    #[serde(rename = "cacheField", default)]
    pub cache_field: Vec<XlsxCacheField>,
}

/// Single field in the PivotCache.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct XlsxCacheField {
    #[serde(rename = "@name", default)]
    pub name: String,
    #[serde(rename = "@caption", default, skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
    #[serde(
        rename = "@propertyName",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub property_name: Option<String>,
    #[serde(
        rename = "@serverField",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub server_field: Option<bool>,
    #[serde(
        rename = "@uniqueList",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub unique_list: Option<bool>,
    #[serde(rename = "@numFmtId", default)]
    pub num_fmt_id: i32,
    #[serde(rename = "@formula", default, skip_serializing_if = "Option::is_none")]
    pub formula: Option<String>,
    #[serde(rename = "@sqlType", default, skip_serializing_if = "Option::is_none")]
    pub sql_type: Option<i32>,
    #[serde(
        rename = "@hierarchy",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub hierarchy: Option<i32>,
    #[serde(rename = "@level", default, skip_serializing_if = "Option::is_none")]
    pub level: Option<i32>,
    #[serde(
        rename = "@databaseField",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub database_field: Option<bool>,
    #[serde(
        rename = "@mappingCount",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub mapping_count: Option<i32>,
    #[serde(
        rename = "@memberPropertyField",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub member_property_field: Option<bool>,
    #[serde(
        rename = "sharedItems",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub shared_items: Option<XlsxSharedItems>,
    #[serde(
        rename = "fieldGroup",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub field_group: Option<XlsxFieldGroup>,
    #[serde(rename = "mpMap", default, skip_serializing_if = "Option::is_none")]
    pub mp_map: Option<XlsxX>,
    #[serde(rename = "extLst", default, skip_serializing_if = "Option::is_none")]
    pub ext_lst: Option<XlsxExtLst>,
}

/// Collection of unique items for a field in the PivotCacheDefinition.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct XlsxSharedItems {
    #[serde(
        rename = "@containsSemiMixedTypes",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub contains_semi_mixed_types: Option<bool>,
    #[serde(
        rename = "@containsNonDate",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub contains_non_date: Option<bool>,
    #[serde(
        rename = "@containsDate",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub contains_date: Option<bool>,
    #[serde(
        rename = "@containsString",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub contains_string: Option<bool>,
    #[serde(
        rename = "@containsBlank",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub contains_blank: Option<bool>,
    #[serde(
        rename = "@containsMixedTypes",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub contains_mixed_types: Option<bool>,
    #[serde(
        rename = "@containsNumber",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub contains_number: Option<bool>,
    #[serde(
        rename = "@containsInteger",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub contains_integer: Option<bool>,
    #[serde(rename = "@minValue", default, skip_serializing_if = "Option::is_none")]
    pub min_value: Option<f64>,
    #[serde(rename = "@maxValue", default, skip_serializing_if = "Option::is_none")]
    pub max_value: Option<f64>,
    #[serde(rename = "@minDate", default, skip_serializing_if = "Option::is_none")]
    pub min_date: Option<String>,
    #[serde(rename = "@maxDate", default, skip_serializing_if = "Option::is_none")]
    pub max_date: Option<String>,
    #[serde(rename = "@count", default)]
    pub count: i32,
    #[serde(rename = "@longText", default, skip_serializing_if = "Option::is_none")]
    pub long_text: Option<bool>,
    #[serde(rename = "$value", default)]
    pub items: Vec<XlsxSharedItem>,
}

/// A shared item in the pivot table cache field.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum XlsxSharedItem {
    M(XlsxSharedItemData),
    N(XlsxSharedItemData),
    B(XlsxSharedItemData),
    E(XlsxSharedItemData),
    S(XlsxSharedItemData),
    D(XlsxSharedItemData),
}

/// Attributes and child elements shared by all shared item variants.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct XlsxSharedItemData {
    #[serde(rename = "@v", default, skip_serializing_if = "Option::is_none")]
    pub v: Option<String>,
    #[serde(rename = "@xmlns", default, skip_serializing_if = "Option::is_none")]
    pub xmlns: Option<String>,
    #[serde(rename = "@u", default, skip_serializing_if = "Option::is_none")]
    pub u: Option<bool>,
    #[serde(rename = "@f", default, skip_serializing_if = "Option::is_none")]
    pub f: Option<bool>,
    #[serde(rename = "@c", default, skip_serializing_if = "Option::is_none")]
    pub c: Option<String>,
    #[serde(rename = "@cp", default, skip_serializing_if = "Option::is_none")]
    pub cp: Option<i32>,
    #[serde(rename = "@in", default, skip_serializing_if = "Option::is_none")]
    pub r#in: Option<i32>,
    #[serde(rename = "@bc", default, skip_serializing_if = "Option::is_none")]
    pub bc: Option<String>,
    #[serde(rename = "@fc", default, skip_serializing_if = "Option::is_none")]
    pub fc: Option<String>,
    #[serde(rename = "@i", default, skip_serializing_if = "Option::is_none")]
    pub i: Option<bool>,
    #[serde(rename = "@un", default, skip_serializing_if = "Option::is_none")]
    pub un: Option<bool>,
    #[serde(rename = "@st", default, skip_serializing_if = "Option::is_none")]
    pub st: Option<bool>,
    #[serde(rename = "@b", default, skip_serializing_if = "Option::is_none")]
    pub b: Option<bool>,
    #[serde(rename = "tpls", default, skip_serializing_if = "Option::is_none")]
    pub tpls: Option<XlsxTuples>,
    #[serde(rename = "x", default, skip_serializing_if = "Option::is_none")]
    pub x: Option<AttrValInt>,
}

/// Members for the OLAP sheet data entry, also known as a tuple.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XlsxTuples {}

/// Collection of properties for a field group.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XlsxFieldGroup {}

/// Collection of OLAP hierarchies in the PivotCache.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XlsxCacheHierarchies {}

/// Collection of Key Performance Indicators defined on the OLAP server.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XlsxKpis {}

/// Cache of OLAP sheet data members, or tuples.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XlsxTupleCache {}

/// Collection of calculated items.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XlsxCalculatedItems {}

/// Collection of calculated members in an OLAP PivotTable.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XlsxCalculatedMembers {}

/// Collection of PivotTable OLAP dimensions.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XlsxDimensions {}

/// Collection of PivotTable OLAP measure groups.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XlsxMeasureGroups {}

/// PivotTable OLAP measure group - Dimension maps.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XlsxMaps {}

/// Index reference used by `mpMap` and pivot table items.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XlsxX {}

/// Extended properties of a pivot table cache definition (`x14:pivotCacheDefinition`).
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "x14:pivotCacheDefinition")]
pub struct XlsxX14PivotCacheDefinition {
    #[serde(rename = "@xmlns", default, skip_serializing_if = "Option::is_none")]
    pub xmlns: Option<String>,
    #[serde(rename = "@pivotCacheId", default)]
    pub pivot_cache_id: i32,
}

/// Structure used to parse the `x14:pivotCacheDefinition` element without the prefix.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "pivotCacheDefinition")]
pub struct DecodeX14PivotCacheDefinition {
    #[serde(rename = "@pivotCacheId", default)]
    pub pivot_cache_id: i32,
}
