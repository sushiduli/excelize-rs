//! Content types part (`[Content_Types].xml`).
//!
//! Ported from Go `xmlContentTypes.go`.

use serde::{Deserialize, Serialize};

// ------------------------------------------------------------------
// Content types container
// ------------------------------------------------------------------

/// Directly maps the `Types` element of the content types part.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "Types")]
pub struct XlsxTypes {
    #[serde(rename = "@xmlns", default)]
    pub xmlns: Option<String>,
    /// Mixed `Default` and `Override` entries under the root `Types` element.
    #[serde(rename = "$value", default)]
    pub entries: Vec<XlsxContentTypeEntry>,
}

impl XlsxTypes {
    /// Return all `Default` entries.
    pub fn defaults(&self) -> Vec<XlsxDefault> {
        self.entries
            .iter()
            .filter_map(|e| match e {
                XlsxContentTypeEntry::Default(d) => Some(d.clone()),
                _ => None,
            })
            .collect()
    }

    /// Return all `Override` entries.
    pub fn overrides(&self) -> Vec<XlsxOverride> {
        self.entries
            .iter()
            .filter_map(|e| match e {
                XlsxContentTypeEntry::Override(o) => Some(o.clone()),
                _ => None,
            })
            .collect()
    }

    /// Iterate over entries mutably.
    pub fn entries_mut(&mut self) -> &mut Vec<XlsxContentTypeEntry> {
        &mut self.entries
    }
}

/// A single child of the `Types` element.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum XlsxContentTypeEntry {
    #[serde(rename = "Default")]
    Default(XlsxDefault),
    #[serde(rename = "Override")]
    Override(XlsxOverride),
}

/// Maps the `Override` element.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XlsxOverride {
    #[serde(rename = "@PartName")]
    pub part_name: String,
    #[serde(rename = "@ContentType")]
    pub content_type: String,
}

/// Maps the `Default` element.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XlsxDefault {
    #[serde(rename = "@Extension")]
    pub extension: String,
    #[serde(rename = "@ContentType")]
    pub content_type: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use quick_xml::de::from_str;

    #[test]
    fn deserialize_template() {
        let s = crate::constants::XML_HEADER.to_string() + crate::templates::TEMPLATE_CONTENT_TYPES;
        let ct: XlsxTypes = from_str(&s).unwrap();
        assert_eq!(ct.defaults().len(), 2);
        assert_eq!(ct.overrides().len(), 6);
    }
}
