//! Shared OpenXML data structures used across multiple package parts.
//!
//! These small types are referenced by worksheets, shared strings, comments,
//! styles and many other parts. Keeping them in one module avoids circular
//! dependencies between the XML model modules.

use std::io::Cursor;

use quick_xml::events::Event;
use quick_xml::{Reader, Writer};
use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Visitor};
use std::fmt;

// ------------------------------------------------------------------
// Attribute-value wrappers
// ------------------------------------------------------------------

/// Wrapper for a string value carried by a child element `attrValString`.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttrValString {
    #[serde(rename = "@val", default)]
    pub val: Option<String>,
}

/// Wrapper for an integer value carried by a child element `attrValInt`.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttrValInt {
    #[serde(rename = "@val", default)]
    pub val: Option<i64>,
}

/// Wrapper for a floating point value carried by a child element `attrValFloat`.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct AttrValFloat {
    #[serde(rename = "@val", default)]
    pub val: Option<f64>,
}

/// Wrapper for a boolean value carried by a child element `attrValBool`.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttrValBool {
    #[serde(
        rename = "@val",
        default,
        serialize_with = "serialize_attr_val_bool",
        deserialize_with = "deserialize_attr_val_bool"
    )]
    pub val: Option<bool>,
}

impl AttrValBool {
    /// Return the boolean value or `false` when missing.
    pub fn value(&self) -> bool {
        self.val.unwrap_or(false)
    }
}

fn serialize_attr_val_bool<S>(value: &Option<bool>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match value {
        Some(true) => serializer.serialize_str("1"),
        _ => serializer.serialize_str("0"),
    }
}

fn deserialize_attr_val_bool<'de, D>(deserializer: D) -> Result<Option<bool>, D::Error>
where
    D: Deserializer<'de>,
{
    struct AttrValBoolVisitor;

    impl<'de> Visitor<'de> for AttrValBoolVisitor {
        type Value = Option<bool>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a boolean attribute value (0, 1, true, false, or empty)")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            match value {
                "" | "1" | "true" => Ok(Some(true)),
                "0" | "false" => Ok(Some(false)),
                _ => Err(E::custom(format!(
                    "invalid boolean attribute value: {}",
                    value
                ))),
            }
        }

        fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(Some(value))
        }
    }

    deserializer.deserialize_any(AttrValBoolVisitor)
}

impl AttrValString {
    /// Return the string value or an empty string when missing.
    pub fn value(&self) -> String {
        self.val.clone().unwrap_or_default()
    }
}

impl AttrValInt {
    /// Return the integer value or `0` when missing.
    pub fn value(&self) -> i64 {
        self.val.unwrap_or(0)
    }
}

impl AttrValFloat {
    /// Return the float value or `0.0` when missing.
    pub fn value(&self) -> f64 {
        self.val.unwrap_or(0.0)
    }
}

// ------------------------------------------------------------------
// Color
// ------------------------------------------------------------------

/// Directly maps the color element.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct XlsxColor {
    #[serde(rename = "@auto", default, skip_serializing_if = "Option::is_none")]
    pub auto: Option<bool>,
    #[serde(rename = "@indexed", default, skip_serializing_if = "Option::is_none")]
    pub indexed: Option<i64>,
    #[serde(rename = "@rgb", default, skip_serializing_if = "Option::is_none")]
    pub rgb: Option<String>,
    #[serde(rename = "@theme", default, skip_serializing_if = "Option::is_none")]
    pub theme: Option<i64>,
    #[serde(rename = "@tint", default, skip_serializing_if = "Option::is_none")]
    pub tint: Option<f64>,
}

// ------------------------------------------------------------------
// Rich text / phonetic primitives
// ------------------------------------------------------------------

/// Directly maps the `t` element in a run or shared string item.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XlsxT {
    #[serde(rename = "@xml:space", default)]
    pub space: Option<String>,
    #[serde(rename = "$value", default)]
    pub val: String,
}

/// Represents a run of rich text.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct XlsxR {
    #[serde(rename = "rPr", default)]
    pub r_pr: Option<XlsxRPr>,
    #[serde(rename = "t", default)]
    pub t: Option<XlsxT>,
}

/// Run properties.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct XlsxRPr {
    #[serde(rename = "rFont", default)]
    pub r_font: Option<AttrValString>,
    #[serde(rename = "charset", default)]
    pub charset: Option<AttrValInt>,
    #[serde(rename = "family", default)]
    pub family: Option<AttrValInt>,
    #[serde(rename = "b", default)]
    pub b: Option<AttrValBool>,
    #[serde(rename = "i", default)]
    pub i: Option<AttrValBool>,
    #[serde(rename = "strike", default)]
    pub strike: Option<AttrValBool>,
    #[serde(rename = "outline", default)]
    pub outline: Option<AttrValBool>,
    #[serde(rename = "shadow", default)]
    pub shadow: Option<AttrValBool>,
    #[serde(rename = "condense", default)]
    pub condense: Option<AttrValBool>,
    #[serde(rename = "extend", default)]
    pub extend: Option<AttrValBool>,
    #[serde(rename = "color", default)]
    pub color: Option<XlsxColor>,
    #[serde(rename = "sz", default)]
    pub sz: Option<AttrValFloat>,
    #[serde(rename = "u", default)]
    pub u: Option<AttrValString>,
    #[serde(rename = "vertAlign", default)]
    pub vert_align: Option<AttrValString>,
    #[serde(rename = "scheme", default)]
    pub scheme: Option<AttrValString>,
}

/// A phonetic run.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XlsxPhoneticRun {
    #[serde(rename = "@sb", default)]
    pub sb: u32,
    #[serde(rename = "@eb", default)]
    pub eb: u32,
    #[serde(rename = "t", default)]
    pub t: String,
}

/// Phonetic properties.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct XlsxPhoneticPr {
    #[serde(rename = "@fontId", default)]
    pub font_id: Option<i64>,
    #[serde(rename = "@type", default)]
    pub r#type: Option<String>,
    #[serde(rename = "@alignment", default)]
    pub alignment: Option<String>,
}

/// Rich text run used in the public API.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct RichTextRun {
    #[serde(default)]
    pub font: Option<crate::styles::Font>,
    #[serde(default)]
    pub text: String,
}

// ------------------------------------------------------------------
// Extension list
// ------------------------------------------------------------------

/// Directly maps the extLst element.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XlsxExtLst {
    #[serde(rename = "ext", default)]
    pub ext: Vec<XlsxExt>,
}

/// Directly maps an ext element used in extension lists.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XlsxExt {
    #[serde(rename = "@uri", default)]
    pub uri: Option<String>,
    #[serde(
        rename = "@xmlns:x14",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub xmlns_x14: Option<String>,
    #[serde(rename = "@xmlns:xm", default, skip_serializing_if = "Option::is_none")]
    pub xmlns_xm: Option<String>,
    #[serde(rename = "$value", default)]
    pub content: String,
}

/// Parse the inner XML of an `<extLst>` element and preserve namespace
/// declarations on each `<ext>` child.
pub fn parse_ext_lst_content(xml: &str) -> Result<XlsxExtLst, quick_xml::Error> {
    let wrapped = format!("<extLst>{xml}</extLst>");
    let mut reader = Reader::from_str(&wrapped);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    let mut ext_lst = XlsxExtLst::default();
    let mut current: Option<XlsxExt> = None;
    let mut depth = 0;
    let mut content_buf: Vec<u8> = Vec::new();
    let mut writer: Option<Writer<Cursor<&mut Vec<u8>>>> = None;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                if e.name().as_ref() == b"ext" && depth == 0 {
                    let mut ext = XlsxExt::default();
                    for attr in e.attributes() {
                        let attr = attr?;
                        let value = String::from_utf8_lossy(&attr.value).to_string();
                        match attr.key.as_ref() {
                            b"uri" => ext.uri = Some(value),
                            b"xmlns:x14" => ext.xmlns_x14 = Some(value),
                            b"xmlns:xm" => ext.xmlns_xm = Some(value),
                            _ => {}
                        }
                    }
                    current = Some(ext);
                    depth = 1;
                    content_buf.clear();
                    writer = Some(Writer::new(Cursor::new(&mut content_buf)));
                } else if depth > 0 {
                    depth += 1;
                    if let Some(ref mut w) = writer {
                        w.write_event(Event::Start(e))?;
                    }
                }
            }
            Ok(Event::End(e)) => {
                if e.name().as_ref() == b"ext" && depth == 1 {
                    if let Some(mut ext) = current.take() {
                        writer = None;
                        ext.content = String::from_utf8_lossy(&content_buf).to_string();
                        ext_lst.ext.push(ext);
                    }
                    depth = 0;
                } else if depth > 0 {
                    depth -= 1;
                    if let Some(ref mut w) = writer {
                        w.write_event(Event::End(e))?;
                    }
                }
            }
            Ok(Event::Empty(e)) => {
                if depth > 0 {
                    if let Some(ref mut w) = writer {
                        w.write_event(Event::Empty(e))?;
                    }
                }
            }
            Ok(Event::Text(e)) => {
                if depth > 0 {
                    if let Some(ref mut w) = writer {
                        w.write_event(Event::Text(e))?;
                    }
                }
            }
            Ok(Event::CData(e)) => {
                if depth > 0 {
                    if let Some(ref mut w) = writer {
                        w.write_event(Event::CData(e))?;
                    }
                }
            }
            Ok(Event::Comment(e)) => {
                if depth > 0 {
                    if let Some(ref mut w) = writer {
                        w.write_event(Event::Comment(e))?;
                    }
                }
            }
            Ok(Event::PI(e)) => {
                if depth > 0 {
                    if let Some(ref mut w) = writer {
                        w.write_event(Event::PI(e))?;
                    }
                }
            }
            Ok(Event::DocType(e)) => {
                if depth > 0 {
                    if let Some(ref mut w) = writer {
                        w.write_event(Event::DocType(e))?;
                    }
                }
            }
            Ok(Event::Decl(_)) => {}
            Ok(Event::Eof) => break,
            Err(e) => return Err(e),
        }
        buf.clear();
    }
    Ok(ext_lst)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
    struct TestBool {
        #[serde(rename = "b", default)]
        b: Option<AttrValBool>,
    }

    #[test]
    fn test_attr_val_bool_serialization() {
        let t = TestBool {
            b: Some(AttrValBool { val: Some(true) }),
        };
        let xml = quick_xml::se::to_string(&t).unwrap();
        assert!(xml.contains(r#"<b val="1"/>"#), "{}", xml);

        let f = TestBool {
            b: Some(AttrValBool { val: Some(false) }),
        };
        let xml = quick_xml::se::to_string(&f).unwrap();
        assert!(xml.contains(r#"<b val="0"/>"#), "{}", xml);
    }

    #[test]
    fn test_attr_val_bool_deserialization() {
        let xml = r#"<TestBool><b val="1"/></TestBool>"#;
        let t: TestBool = quick_xml::de::from_str(xml).unwrap();
        assert_eq!(t.b.unwrap().val, Some(true));

        let xml = r#"<TestBool><b val="0"/></TestBool>"#;
        let t: TestBool = quick_xml::de::from_str(xml).unwrap();
        assert_eq!(t.b.unwrap().val, Some(false));

        let xml = r#"<TestBool/>"#;
        let t: TestBool = quick_xml::de::from_str(xml).unwrap();
        assert_eq!(t.b, None);
    }
}

/// Serialize an `<extLst>` element to the inner XML that should be placed
/// between the `<extLst>` and `</extLst>` tags.
pub fn serialize_ext_lst(ext_lst: &XlsxExtLst) -> String {
    let mut s = String::new();
    for ext in &ext_lst.ext {
        s.push_str("<ext");
        if let Some(uri) = &ext.uri {
            s.push_str(&format!(r#" uri="{}""#, uri));
        }
        if let Some(ns) = &ext.xmlns_x14 {
            s.push_str(&format!(r#" xmlns:x14="{}""#, ns));
        }
        if let Some(ns) = &ext.xmlns_xm {
            s.push_str(&format!(r#" xmlns:xm="{}""#, ns));
        }
        s.push('>');
        s.push_str(&ext.content);
        s.push_str("</ext>");
    }
    s
}

// ------------------------------------------------------------------
// Inner XML / opaque content
// ------------------------------------------------------------------

/// Container used to preserve raw XML content that is not fully modeled.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XlsxInnerXml {
    #[serde(rename = "$value", default)]
    pub content: String,
}
