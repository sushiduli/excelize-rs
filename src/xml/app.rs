//! Extended document properties (`docProps/app.xml`).
//!
//! Ported from Go `xmlApp.go`.

use serde::{Deserialize, Serialize};

use super::common::XlsxInnerXml;

/// Directly maps the document application properties.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppProperties {
    #[serde(default)]
    pub application: String,
    #[serde(default)]
    pub scale_crop: bool,
    #[serde(default)]
    pub doc_security: i32,
    #[serde(default)]
    pub company: String,
    #[serde(default)]
    pub links_up_to_date: bool,
    #[serde(default)]
    pub hyperlinks_changed: bool,
    #[serde(default)]
    pub app_version: String,
}

/// OOXML document properties such as the template used, the number of pages and
/// words, and the application name and version.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "Properties", rename_all = "PascalCase")]
pub struct XlsxProperties {
    #[serde(rename = "@xmlns:vt", default, skip_serializing_if = "Option::is_none")]
    pub vt: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub template: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub manager: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub company: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pages: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub words: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub characters: Option<i64>,
    #[serde(
        rename = "PresentationFormat",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub presentation_format: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lines: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub paragraphs: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub slides: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<i64>,
    #[serde(rename = "TotalTime", default, skip_serializing_if = "Option::is_none")]
    pub total_time: Option<i64>,
    #[serde(
        rename = "HiddenSlides",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub hidden_slides: Option<i64>,
    #[serde(rename = "MMClips", default, skip_serializing_if = "Option::is_none")]
    pub mm_clips: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scale_crop: Option<bool>,
    #[serde(
        rename = "HeadingPairs",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub heading_pairs: Option<XlsxVectorVariant>,
    #[serde(
        rename = "TitlesOfParts",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub titles_of_parts: Option<XlsxVectorLpstr>,
    #[serde(
        rename = "LinksUpToDate",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub links_up_to_date: Option<bool>,
    #[serde(
        rename = "CharactersWithSpaces",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub characters_with_spaces: Option<i64>,
    #[serde(rename = "SharedDoc", default, skip_serializing_if = "Option::is_none")]
    pub shared_doc: Option<bool>,
    #[serde(
        rename = "HyperlinkBase",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub hyperlink_base: Option<String>,
    #[serde(rename = "HLinks", default, skip_serializing_if = "Option::is_none")]
    pub h_links: Option<XlsxVectorVariant>,
    #[serde(
        rename = "HyperlinksChanged",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub hyperlinks_changed: Option<bool>,
    #[serde(rename = "DigSig", default, skip_serializing_if = "Option::is_none")]
    pub dig_sig: Option<XlsxDigSig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub application: Option<String>,
    #[serde(
        rename = "AppVersion",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub app_version: Option<String>,
    #[serde(
        rename = "DocSecurity",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub doc_security: Option<i64>,
}

/// Specifies the set of hyperlinks that were in the document when last saved.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XlsxVectorVariant {
    #[serde(rename = "$value", default)]
    pub content: String,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XlsxVectorLpstr {
    #[serde(rename = "$value", default)]
    pub content: String,
}

/// Contains the signature of a digitally signed document.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XlsxDigSig {
    #[serde(flatten, default)]
    pub inner: XlsxInnerXml,
}
