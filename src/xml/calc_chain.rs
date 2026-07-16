//! Calculation chain and volatile dependencies parts.
//!
//! Ported from Go `xmlCalcChain.go`.

use serde::{Deserialize, Serialize};

use super::common::XlsxExtLst;

/// Directly maps the calcChain element.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "calcChain", rename_all = "PascalCase")]
pub struct XlsxCalcChain {
    #[serde(rename = "c", default)]
    pub c: Vec<XlsxCalcChainC>,
}

/// Directly maps the `c` element inside the calcChain.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XlsxCalcChainC {
    #[serde(rename = "@r", default)]
    pub r: String,
    #[serde(rename = "@i", default)]
    pub i: i32,
    #[serde(rename = "@l", default)]
    pub l: bool,
    #[serde(rename = "@s", default)]
    pub s: bool,
    #[serde(rename = "@t", default)]
    pub t: bool,
    #[serde(rename = "@a", default)]
    pub a: bool,
}

/// Volatile dependencies part.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "volTypes", rename_all = "PascalCase")]
pub struct XlsxVolTypes {
    #[serde(rename = "volType", default)]
    pub vol_type: Vec<XlsxVolType>,
    #[serde(rename = "extLst", default)]
    pub ext_lst: Option<XlsxExtLst>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XlsxVolType {
    #[serde(rename = "@type", default)]
    pub r#type: String,
    #[serde(rename = "main", default)]
    pub main: Vec<XlsxVolMain>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XlsxVolMain {
    #[serde(rename = "@first", default)]
    pub first: String,
    #[serde(rename = "tp", default)]
    pub tp: Vec<XlsxVolTopic>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XlsxVolTopic {
    #[serde(rename = "@t", default)]
    pub t: Option<String>,
    #[serde(default)]
    pub v: String,
    #[serde(rename = "stp", default)]
    pub stp: Vec<String>,
    #[serde(rename = "tr", default)]
    pub tr: Vec<XlsxVolTopicRef>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XlsxVolTopicRef {
    #[serde(rename = "@r", default)]
    pub r: String,
    #[serde(rename = "@s", default)]
    pub s: i32,
}
