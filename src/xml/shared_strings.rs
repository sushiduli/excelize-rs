//! Shared string table part (`xl/sharedStrings.xml`).
//!
//! Ported from Go `xmlSharedStrings.go`.

use serde::{Deserialize, Serialize};

use super::common::{XlsxPhoneticPr, XlsxPhoneticRun, XlsxR, XlsxT};

fn default_xmlns() -> String {
    crate::constants::NAMESPACE_SPREADSHEET.to_string()
}

/// Directly maps the `sst` element.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "sst", rename_all = "PascalCase")]
pub struct XlsxSst {
    #[serde(rename = "@xmlns", default = "default_xmlns")]
    pub xmlns: String,
    #[serde(rename = "@count", default)]
    pub count: i64,
    #[serde(rename = "@uniqueCount", default)]
    pub unique_count: i64,
    #[serde(rename = "si", default)]
    pub si: Vec<XlsxSi>,
}

impl Default for XlsxSst {
    fn default() -> Self {
        Self {
            xmlns: default_xmlns(),
            count: 0,
            unique_count: 0,
            si: Vec::new(),
        }
    }
}

/// String item in the shared string table.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct XlsxSi {
    #[serde(rename = "t", default, skip_serializing_if = "Option::is_none")]
    pub t: Option<XlsxT>,
    #[serde(rename = "r", default)]
    pub r: Vec<XlsxR>,
    #[serde(rename = "rPh", default)]
    pub r_ph: Vec<XlsxPhoneticRun>,
    #[serde(rename = "phoneticPr", default, skip_serializing_if = "Option::is_none")]
    pub phonetic_pr: Option<XlsxPhoneticPr>,
}
