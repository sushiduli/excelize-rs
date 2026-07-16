//! Core document properties (`docProps/core.xml`).
//!
//! Ported from Go `xmlCore.go`.

use serde::{Deserialize, Serialize};

/// Directly maps the document core properties.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocProperties {
    #[serde(default)]
    pub category: String,
    #[serde(rename = "ContentStatus", default)]
    pub content_status: String,
    #[serde(default)]
    pub created: String,
    #[serde(default)]
    pub creator: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub identifier: String,
    #[serde(default)]
    pub keywords: String,
    #[serde(rename = "LastModifiedBy", default)]
    pub last_modified_by: String,
    #[serde(default)]
    pub modified: String,
    #[serde(default)]
    pub revision: String,
    #[serde(default)]
    pub subject: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub language: String,
    #[serde(default)]
    pub version: String,
}

/// DCMI metadata terms for the coreProperties.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecodeDcTerms {
    #[serde(rename = "$value", default)]
    pub text: String,
    #[serde(rename = "@xsi:type", default)]
    pub r#type: Option<String>,
}

/// Deserialization-only representation of `coreProperties`.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "coreProperties", rename_all = "PascalCase")]
pub struct DecodeCoreProperties {
    #[serde(rename = "title", default)]
    pub title: Option<String>,
    #[serde(rename = "subject", default)]
    pub subject: Option<String>,
    #[serde(rename = "creator", default)]
    pub creator: Option<String>,
    #[serde(default)]
    pub keywords: Option<String>,
    #[serde(rename = "description", default)]
    pub description: Option<String>,
    #[serde(rename = "lastModifiedBy", default)]
    pub last_modified_by: Option<String>,
    #[serde(rename = "language", default)]
    pub language: Option<String>,
    #[serde(rename = "identifier", default)]
    pub identifier: Option<String>,
    #[serde(default)]
    pub revision: Option<String>,
    #[serde(rename = "created", default)]
    pub created: Option<DecodeDcTerms>,
    #[serde(rename = "modified", default)]
    pub modified: Option<DecodeDcTerms>,
    #[serde(rename = "contentStatus", default)]
    pub content_status: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
}

/// DCMI metadata terms for the coreProperties.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XlsxDcTerms {
    #[serde(rename = "$value", default)]
    pub text: String,
    #[serde(rename = "@xsi:type", default, skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
}

/// Serialization representation of `coreProperties`.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "coreProperties", rename_all = "PascalCase")]
pub struct XlsxCoreProperties {
    #[serde(rename = "@xmlns:dc", default, skip_serializing_if = "Option::is_none")]
    pub dc: Option<String>,
    #[serde(
        rename = "@xmlns:dcterms",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub dcterms: Option<String>,
    #[serde(
        rename = "@xmlns:dcmitype",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub dcmitype: Option<String>,
    #[serde(
        rename = "@xmlns:xsi",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub xsi: Option<String>,
    #[serde(rename = "dc:title", default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(
        rename = "dc:subject",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub subject: Option<String>,
    #[serde(
        rename = "dc:creator",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub creator: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub keywords: Option<String>,
    #[serde(
        rename = "dc:description",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub description: Option<String>,
    #[serde(
        rename = "lastModifiedBy",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub last_modified_by: Option<String>,
    #[serde(
        rename = "dc:language",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub language: Option<String>,
    #[serde(
        rename = "dc:identifier",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub identifier: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub revision: Option<String>,
    #[serde(
        rename = "dcterms:created",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub created: Option<XlsxDcTerms>,
    #[serde(
        rename = "dcterms:modified",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub modified: Option<XlsxDcTerms>,
    #[serde(
        rename = "contentStatus",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub content_status: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use quick_xml::de::from_str;

    #[test]
    fn deserialize_created() {
        let s = r#"<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties" xmlns:dcterms="http://purl.org/dc/terms/" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"><dcterms:created xsi:type="dcterms:W3CDTF">2019-06-04T22:00:10Z</dcterms:created></cp:coreProperties>"#;
        let core: DecodeCoreProperties = from_str(s).unwrap();
        assert_eq!(core.created.as_ref().unwrap().text, "2019-06-04T22:00:10Z");
    }

    #[test]
    fn deserialize_created_simple() {
        let s = r#"<coreProperties xmlns:dcterms="http://purl.org/dc/terms/"><dcterms:created>2019-06-04T22:00:10Z</dcterms:created></coreProperties>"#;
        let core: DecodeCoreProperties = from_str(s).unwrap();
        assert_eq!(core.created.as_ref().unwrap().text, "2019-06-04T22:00:10Z");
    }
}
