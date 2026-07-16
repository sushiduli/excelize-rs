//! Metadata and rich value parts (`xl/metadata.xml`, `xl/richData/`).
//!
//! Ported from Go `xmlMetaData.go`.

use serde::{Deserialize, Serialize};

use super::common::XlsxInnerXml;

/// Directly maps the metadata element. A cell in a spreadsheet application can
/// have metadata associated with it. Metadata is just a set of additional
/// properties about the particular cell, and this metadata is stored in the
/// metadata xml part. There are two types of metadata: cell metadata and value
/// metadata.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "metadata")]
pub struct XlsxMetadata {
    #[serde(rename = "metadataTypes", default, skip_serializing_if = "Option::is_none")]
    pub metadata_types: Option<XlsxInnerXml>,
    #[serde(rename = "metadataStrings", default, skip_serializing_if = "Option::is_none")]
    pub metadata_strings: Option<XlsxInnerXml>,
    #[serde(rename = "mdxMetadata", default, skip_serializing_if = "Option::is_none")]
    pub mdx_metadata: Option<XlsxInnerXml>,
    #[serde(rename = "futureMetadata", default)]
    pub future_metadata: Vec<XlsxFutureMetadata>,
    #[serde(rename = "cellMetadata", default, skip_serializing_if = "Option::is_none")]
    pub cell_metadata: Option<XlsxMetadataBlocks>,
    #[serde(rename = "valueMetadata", default, skip_serializing_if = "Option::is_none")]
    pub value_metadata: Option<XlsxMetadataBlocks>,
    #[serde(rename = "extLst", default, skip_serializing_if = "Option::is_none")]
    pub ext_lst: Option<XlsxInnerXml>,
}

/// Directly maps the futureMetadata element. This element represents future
/// metadata information.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XlsxFutureMetadata {
    #[serde(rename = "bk", default)]
    pub bk: Vec<XlsxFutureMetadataBlock>,
    #[serde(rename = "extLst", default, skip_serializing_if = "Option::is_none")]
    pub ext_lst: Option<XlsxInnerXml>,
}

/// Directly maps the bk element. This element represents a block of future
/// metadata information. This is a location for storing feature extension
/// information.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XlsxFutureMetadataBlock {
    #[serde(rename = "extLst", default, skip_serializing_if = "Option::is_none")]
    pub ext_lst: Option<XlsxInnerXml>,
}

/// Directly maps the metadata element. This element represents cell metadata
/// information. Cell metadata is information metadata about a specific cell,
/// and it stays tied to that cell position.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XlsxMetadataBlocks {
    #[serde(rename = "@count", default, skip_serializing_if = "Option::is_none")]
    pub count: Option<i64>,
    #[serde(rename = "bk", default)]
    pub bk: Vec<XlsxMetadataBlock>,
}

/// Directly maps the bk element. This element represents a block of metadata
/// records.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XlsxMetadataBlock {
    #[serde(rename = "rc", default)]
    pub rc: Vec<XlsxMetadataRecord>,
}

/// Directly maps the rc element. This element represents a reference to a
/// specific metadata record.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XlsxMetadataRecord {
    #[serde(rename = "@t")]
    pub t: i64,
    #[serde(rename = "@v")]
    pub v: i64,
}

/// Directly maps the rvData element that specifies rich value data.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "rvData")]
pub struct XlsxRichValueData {
    #[serde(rename = "@count", default, skip_serializing_if = "Option::is_none")]
    pub count: Option<i64>,
    #[serde(rename = "rv", default)]
    pub rv: Vec<XlsxRichValue>,
    #[serde(rename = "extLst", default, skip_serializing_if = "Option::is_none")]
    pub ext_lst: Option<XlsxInnerXml>,
}

/// Directly maps the rv element that specifies rich value data information for
/// a single rich value.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XlsxRichValue {
    #[serde(rename = "@s")]
    pub s: i64,
    #[serde(rename = "v", default)]
    pub v: Vec<String>,
    #[serde(rename = "fb", default, skip_serializing_if = "Option::is_none")]
    pub fb: Option<XlsxInnerXml>,
}

/// Directly maps the richValueRels element. This element specifies a list of
/// rich value relationships.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "richValueRels")]
pub struct XlsxRichValueRels {
    #[serde(rename = "rel", default)]
    pub rels: Vec<XlsxRichValueRelRelationship>,
    #[serde(rename = "extLst", default, skip_serializing_if = "Option::is_none")]
    pub ext_lst: Option<XlsxInnerXml>,
}

/// Directly maps the rel element. This element specifies a relationship for a
/// rich value property.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XlsxRichValueRelRelationship {
    #[serde(rename = "@id")]
    pub id: String,
}

/// Directly maps the rvStructures element. This element specifies rich value
/// structures, which contain lists of rich value keys and the data types for
/// the corresponding rich value data.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "rvStructures")]
pub struct XlsxRichValueStructures {
    #[serde(rename = "@count", default, skip_serializing_if = "Option::is_none")]
    pub count: Option<i64>,
    #[serde(rename = "s", default)]
    pub s: Vec<XlsxRichValueStructure>,
    #[serde(rename = "extLst", default, skip_serializing_if = "Option::is_none")]
    pub ext_lst: Option<XlsxInnerXml>,
}

/// Directly maps the s element. This element specifies the list of rich value
/// structures.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XlsxRichValueStructure {
    #[serde(rename = "@t")]
    pub t: String,
    #[serde(rename = "k", default)]
    pub k: Vec<XlsxRichValueKey>,
}

/// Directly maps the k element. This element specifies the rich value key.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XlsxRichValueKey {
    #[serde(rename = "@n")]
    pub n: String,
    #[serde(rename = "@t", default, skip_serializing_if = "Option::is_none")]
    pub t: Option<String>,
}

/// Directly maps the webImagesSrd element. This element specifies a list of
/// sets of properties associated with web image rich values.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "webImagesSrd")]
pub struct XlsxWebImagesSupportingRichData {
    #[serde(rename = "webImageSrd", default)]
    pub web_image_srd: Vec<XlsxWebImageSupportingRichData>,
    #[serde(rename = "extLst", default, skip_serializing_if = "Option::is_none")]
    pub ext_lst: Option<XlsxInnerXml>,
}

/// Directly maps the webImageSrd element. This element specifies a set of
/// properties for a web image rich value.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct XlsxWebImageSupportingRichData {
    #[serde(rename = "address", default)]
    pub address: XlsxExternalReference,
    #[serde(rename = "moreImagesAddress", default)]
    pub more_images_address: XlsxExternalReference,
    #[serde(rename = "blip", default)]
    pub blip: XlsxExternalReference,
}

/// Directly maps the externalReference element of the external workbook
/// references part.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XlsxExternalReference {
    #[serde(rename = "@r:id", default, skip_serializing_if = "Option::is_none")]
    pub r_id: Option<String>,
}
