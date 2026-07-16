//! Shared string table part (`xl/sharedStrings.xml`).
//!
//! Ported from Go `xmlSharedStrings.go`.

use serde::{Deserialize, Serialize};

use super::common::{XlsxPhoneticPr, XlsxPhoneticRun, XlsxR, XlsxT};

/// Directly maps the `sst` element.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "sst", rename_all = "PascalCase")]
pub struct XlsxSst {
    #[serde(rename = "@count", default)]
    pub count: i64,
    #[serde(rename = "@uniqueCount", default)]
    pub unique_count: i64,
    #[serde(rename = "si", default)]
    pub si: Vec<XlsxSi>,
}

/// String item in the shared string table.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct XlsxSi {
    #[serde(rename = "t", default)]
    pub t: Option<XlsxT>,
    #[serde(rename = "r", default)]
    pub r: Vec<XlsxR>,
    #[serde(rename = "rPh", default)]
    pub r_ph: Vec<XlsxPhoneticRun>,
    #[serde(rename = "phoneticPr", default)]
    pub phonetic_pr: Option<XlsxPhoneticPr>,
}
