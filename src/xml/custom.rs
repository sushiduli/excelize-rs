//! Custom file properties part (`docProps/custom.xml`).
//!
//! Ported from Go `xmlCustom.go`.

use serde::{Deserialize, Serialize};

/// Directly maps the element for the custom file properties part, that
/// represents additional information. The information can be used as metadata
/// for XML.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "Properties")]
pub struct XlsxCustomProperties {
    #[serde(rename = "@xmlns", default)]
    pub xmlns: Option<String>,
    #[serde(rename = "@xmlns:vt", default)]
    pub vt: Option<String>,
    #[serde(rename = "property", default)]
    pub property: Vec<XlsxProperty>,
}

/// Directly maps the element specifies a single custom file property. Custom
/// file property type is defined through child elements in the File Properties
/// Variant Type namespace. Custom file property value can be set by setting the
/// appropriate Variant Type child element value.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct XlsxProperty {
    #[serde(rename = "@fmtid", default)]
    pub fmt_id: String,
    #[serde(rename = "@pid", default)]
    pub pid: i64,
    #[serde(rename = "@name", default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(
        rename = "@linkTarget",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub link_target: Option<String>,
    #[serde(rename = "vt:vector", default, skip_serializing_if = "Option::is_none")]
    pub vector: Option<String>,
    #[serde(rename = "vt:array", default, skip_serializing_if = "Option::is_none")]
    pub array: Option<String>,
    #[serde(rename = "vt:blob", default, skip_serializing_if = "Option::is_none")]
    pub blob: Option<String>,
    #[serde(rename = "vt:oblob", default, skip_serializing_if = "Option::is_none")]
    pub oblob: Option<String>,
    #[serde(rename = "vt:empty", default, skip_serializing_if = "Option::is_none")]
    pub empty: Option<String>,
    #[serde(rename = "vt:null", default, skip_serializing_if = "Option::is_none")]
    pub null: Option<String>,
    #[serde(rename = "vt:i1", default, skip_serializing_if = "Option::is_none")]
    pub i1: Option<i8>,
    #[serde(rename = "vt:i2", default, skip_serializing_if = "Option::is_none")]
    pub i2: Option<i16>,
    #[serde(rename = "vt:i4", default, skip_serializing_if = "Option::is_none")]
    pub i4: Option<i32>,
    #[serde(rename = "vt:i8", default, skip_serializing_if = "Option::is_none")]
    pub i8: Option<i64>,
    #[serde(rename = "vt:int", default, skip_serializing_if = "Option::is_none")]
    pub int: Option<i64>,
    #[serde(rename = "vt:ui1", default, skip_serializing_if = "Option::is_none")]
    pub ui1: Option<u8>,
    #[serde(rename = "vt:ui2", default, skip_serializing_if = "Option::is_none")]
    pub ui2: Option<u16>,
    #[serde(rename = "vt:ui4", default, skip_serializing_if = "Option::is_none")]
    pub ui4: Option<u32>,
    #[serde(rename = "vt:ui8", default, skip_serializing_if = "Option::is_none")]
    pub ui8: Option<u64>,
    #[serde(rename = "vt:uint", default, skip_serializing_if = "Option::is_none")]
    pub uint: Option<u64>,
    #[serde(rename = "vt:r4", default, skip_serializing_if = "Option::is_none")]
    pub r4: Option<f32>,
    #[serde(rename = "vt:r8", default, skip_serializing_if = "Option::is_none")]
    pub r8: Option<f64>,
    #[serde(
        rename = "vt:decimal",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub decimal: Option<String>,
    #[serde(rename = "vt:lpstr", default, skip_serializing_if = "Option::is_none")]
    pub lpstr: Option<String>,
    #[serde(rename = "vt:lpwstr", default, skip_serializing_if = "Option::is_none")]
    pub lpwstr: Option<String>,
    #[serde(rename = "vt:bstr", default, skip_serializing_if = "Option::is_none")]
    pub bstr: Option<String>,
    #[serde(rename = "vt:date", default, skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
    #[serde(
        rename = "vt:filetime",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub file_time: Option<String>,
    #[serde(rename = "vt:bool", default, skip_serializing_if = "Option::is_none")]
    pub r#bool: Option<bool>,
    #[serde(rename = "vt:cy", default, skip_serializing_if = "Option::is_none")]
    pub cy: Option<String>,
    #[serde(rename = "vt:error", default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(rename = "vt:stream", default, skip_serializing_if = "Option::is_none")]
    pub stream: Option<String>,
    #[serde(
        rename = "vt:ostream",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub ostream: Option<String>,
    #[serde(
        rename = "vt:storage",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub storage: Option<String>,
    #[serde(
        rename = "vt:ostorage",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub ostorage: Option<String>,
    #[serde(
        rename = "vt:vstream",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub vstream: Option<String>,
    #[serde(rename = "vt:clsid", default, skip_serializing_if = "Option::is_none")]
    pub cls_id: Option<String>,
}

/// Deserialization-only representation of the custom file properties part.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "Properties")]
pub struct DecodeCustomProperties {
    #[serde(rename = "@xmlns", default)]
    pub xmlns: Option<String>,
    #[serde(rename = "@xmlns:vt", default)]
    pub vt: Option<String>,
    #[serde(rename = "property", default)]
    pub property: Vec<DecodeProperty>,
}

/// Deserialization-only representation of a single custom file property.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecodeProperty {
    #[serde(rename = "@fmtid", default)]
    pub fmt_id: String,
    #[serde(rename = "@pid", default)]
    pub pid: i64,
    #[serde(rename = "@name", default)]
    pub name: Option<String>,
    #[serde(rename = "@linkTarget", default)]
    pub link_target: Option<String>,
    #[serde(rename = "vector", default)]
    pub vector: Option<String>,
    #[serde(rename = "array", default)]
    pub array: Option<String>,
    #[serde(rename = "blob", default)]
    pub blob: Option<String>,
    #[serde(rename = "oblob", default)]
    pub oblob: Option<String>,
    #[serde(rename = "empty", default)]
    pub empty: Option<String>,
    #[serde(rename = "null", default)]
    pub null: Option<String>,
    #[serde(rename = "i1", default)]
    pub i1: Option<i8>,
    #[serde(rename = "i2", default)]
    pub i2: Option<i16>,
    #[serde(rename = "i4", default)]
    pub i4: Option<i32>,
    #[serde(rename = "i8", default)]
    pub i8: Option<i64>,
    #[serde(rename = "int", default)]
    pub int: Option<i64>,
    #[serde(rename = "ui1", default)]
    pub ui1: Option<u8>,
    #[serde(rename = "ui2", default)]
    pub ui2: Option<u16>,
    #[serde(rename = "ui4", default)]
    pub ui4: Option<u32>,
    #[serde(rename = "ui8", default)]
    pub ui8: Option<u64>,
    #[serde(rename = "uint", default)]
    pub uint: Option<u64>,
    #[serde(rename = "r4", default)]
    pub r4: Option<f32>,
    #[serde(rename = "r8", default)]
    pub r8: Option<f64>,
    #[serde(rename = "decimal", default)]
    pub decimal: Option<String>,
    #[serde(rename = "lpstr", default)]
    pub lpstr: Option<String>,
    #[serde(rename = "lpwstr", default)]
    pub lpwstr: Option<String>,
    #[serde(rename = "bstr", default)]
    pub bstr: Option<String>,
    #[serde(rename = "date", default)]
    pub date: Option<String>,
    #[serde(rename = "filetime", default)]
    pub file_time: Option<String>,
    #[serde(rename = "bool", default)]
    pub r#bool: Option<bool>,
    #[serde(rename = "cy", default)]
    pub cy: Option<String>,
    #[serde(rename = "error", default)]
    pub error: Option<String>,
    #[serde(rename = "stream", default)]
    pub stream: Option<String>,
    #[serde(rename = "ostream", default)]
    pub ostream: Option<String>,
    #[serde(rename = "storage", default)]
    pub storage: Option<String>,
    #[serde(rename = "ostorage", default)]
    pub ostorage: Option<String>,
    #[serde(rename = "vstream", default)]
    pub vstream: Option<String>,
    #[serde(rename = "clsid", default)]
    pub cls_id: Option<String>,
}

/// Directly maps the custom property of the workbook. The value date type may be
/// one of the following: int32, float64, string, bool, time.Time, or nil.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct CustomProperty {
    pub name: String,
    pub value: Option<CustomPropertyValue>,
}

/// Value types that can be stored in a custom property.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CustomPropertyValue {
    Int(i32),
    Float(f64),
    Bool(bool),
    Date(String),
    String(String),
}
