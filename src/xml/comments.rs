//! Comments part (`xl/commentsN.xml`).
//!
//! Ported from Go `xmlComments.go`.

use serde::{Deserialize, Serialize};

use super::common::{RichTextRun, XlsxPhoneticPr, XlsxPhoneticRun, XlsxR};

/// Directly maps the comments element.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "comments", rename_all = "PascalCase")]
pub struct XlsxComments {
    #[serde(rename = "authors", default)]
    pub authors: XlsxAuthor,
    #[serde(rename = "commentList", default)]
    pub comment_list: XlsxCommentList,
    #[serde(skip)]
    pub cells: Vec<String>,
}

/// Holds the list of authors.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XlsxAuthor {
    #[serde(rename = "author", default)]
    pub author: Vec<String>,
}

/// Container for the list of comments.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct XlsxCommentList {
    #[serde(rename = "comment", default)]
    pub comment: Vec<XlsxComment>,
}

/// A single comment.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct XlsxComment {
    #[serde(rename = "@ref", default)]
    pub r#ref: String,
    #[serde(rename = "@authorId", default)]
    pub author_id: i32,
    #[serde(rename = "text", default)]
    pub text: XlsxText,
}

/// Rich text content of a comment.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct XlsxText {
    #[serde(rename = "t", default)]
    pub t: Option<String>,
    #[serde(rename = "r", default)]
    pub r: Vec<XlsxR>,
    #[serde(rename = "rPh", default)]
    pub r_ph: Option<XlsxPhoneticRun>,
    #[serde(rename = "phoneticPr", default)]
    pub phonetic_pr: Option<XlsxPhoneticPr>,
}

/// Comment information used in the public API.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct Comment {
    #[serde(default)]
    pub author: String,
    #[serde(rename = "AuthorID", default)]
    pub author_id: i32,
    #[serde(default)]
    pub cell: String,
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub width: u32,
    #[serde(default)]
    pub height: u32,
    #[serde(default)]
    pub paragraph: Vec<RichTextRun>,
}
