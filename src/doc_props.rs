//! Document properties API (`docProps.go`).
//!
//! Provides read/write access to the application, core and custom document
//! properties stored in `docProps/app.xml`, `docProps/core.xml` and
//! `docProps/custom.xml`.

use std::collections::HashMap;

use chrono::{DateTime, NaiveDateTime};
use quick_xml::de::from_reader as xml_from_reader;
use quick_xml::se::to_string as xml_to_string;

use crate::constants::{
    CONTENT_TYPE_CUSTOM_PROPERTIES, DEFAULT_XML_PATH_DOC_PROPS_APP,
    DEFAULT_XML_PATH_DOC_PROPS_CORE, DEFAULT_XML_PATH_DOC_PROPS_CUSTOM, DEFAULT_XML_PATH_RELS,
    EXT_URI_CUSTOM_PROPERTY_FMT_ID, NAMESPACE_DOCUMENT_PROPERTIES_VARIANT_TYPES,
    NAMESPACE_DUBLIN_CORE, NAMESPACE_DUBLIN_CORE_METADATA_INITIATIVE, NAMESPACE_DUBLIN_CORE_TERMS,
    NAMESPACE_XML_SCHEMA_INSTANCE, SOURCE_RELATIONSHIP_CUSTOM_PROPERTIES,
};
use crate::errors::{ErrParameterInvalid, Result};
use crate::file::File;
use crate::xml::app::{AppProperties, XlsxProperties};
use crate::xml::content_types::{XlsxContentTypeEntry, XlsxOverride};
use crate::xml::core::{DecodeCoreProperties, DocProperties, XlsxCoreProperties, XlsxDcTerms};
use crate::xml::custom::{
    CustomProperty, CustomPropertyValue, DecodeCustomProperties, DecodeProperty,
    XlsxCustomProperties, XlsxProperty,
};

const NAMESPACE_CUSTOM_PROPERTIES: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/custom-properties";

impl File {
    // ------------------------------------------------------------------
    // Public API
    // ------------------------------------------------------------------

    /// Set the document application properties (`docProps/app.xml`).
    pub fn set_app_props(&mut self, props: &AppProperties) -> Result<()> {
        let mut app = self.doc_props_app_reader()?;
        app.application = non_empty(props.application.clone());
        app.scale_crop = Some(props.scale_crop);
        app.doc_security = Some(props.doc_security as i64);
        app.company = non_empty(props.company.clone());
        app.links_up_to_date = Some(props.links_up_to_date);
        app.hyperlinks_changed = Some(props.hyperlinks_changed);
        app.app_version = non_empty(props.app_version.clone());
        self.doc_props_app_writer(&app);
        Ok(())
    }

    /// Get the document application properties.
    pub fn get_app_props(&self) -> Result<AppProperties> {
        let app = self.doc_props_app_reader()?;
        Ok(AppProperties {
            application: app.application.unwrap_or_default(),
            scale_crop: app.scale_crop.unwrap_or(false),
            doc_security: app.doc_security.unwrap_or(0) as i32,
            company: app.company.unwrap_or_default(),
            links_up_to_date: app.links_up_to_date.unwrap_or(false),
            hyperlinks_changed: app.hyperlinks_changed.unwrap_or(false),
            app_version: app.app_version.unwrap_or_default(),
        })
    }

    /// Set the document core properties (`docProps/core.xml`).
    pub fn set_doc_props(&mut self, props: &DocProperties) -> Result<()> {
        let core = self.doc_props_core_reader()?;
        let mut new_props = XlsxCoreProperties {
            dc: Some(NAMESPACE_DUBLIN_CORE.to_string()),
            dcterms: Some(NAMESPACE_DUBLIN_CORE_TERMS.to_string()),
            dcmitype: Some(NAMESPACE_DUBLIN_CORE_METADATA_INITIATIVE.to_string()),
            xsi: Some(NAMESPACE_XML_SCHEMA_INSTANCE.to_string()),
            title: core.title.clone(),
            subject: core.subject.clone(),
            creator: core.creator.clone(),
            keywords: core.keywords.clone(),
            description: core.description.clone(),
            last_modified_by: core.last_modified_by.clone(),
            language: core.language.clone(),
            identifier: core.identifier.clone(),
            revision: core.revision.clone(),
            created: core.created.as_ref().map(|d| XlsxDcTerms {
                text: d.text.clone(),
                r#type: d.r#type.clone(),
            }),
            modified: core.modified.as_ref().map(|d| XlsxDcTerms {
                text: d.text.clone(),
                r#type: d.r#type.clone(),
            }),
            content_status: core.content_status.clone(),
            category: core.category.clone(),
            version: core.version.clone(),
        };

        if !props.category.is_empty() {
            new_props.category = Some(props.category.clone());
        } else {
            new_props.category = None;
        }
        if !props.content_status.is_empty() {
            new_props.content_status = Some(props.content_status.clone());
        } else {
            new_props.content_status = None;
        }
        if !props.creator.is_empty() {
            new_props.creator = Some(props.creator.clone());
        } else {
            new_props.creator = None;
        }
        if !props.description.is_empty() {
            new_props.description = Some(props.description.clone());
        } else {
            new_props.description = None;
        }
        if !props.identifier.is_empty() {
            new_props.identifier = Some(props.identifier.clone());
        } else {
            new_props.identifier = None;
        }
        if !props.keywords.is_empty() {
            new_props.keywords = Some(props.keywords.clone());
        } else {
            new_props.keywords = None;
        }
        if !props.last_modified_by.is_empty() {
            new_props.last_modified_by = Some(props.last_modified_by.clone());
        } else {
            new_props.last_modified_by = None;
        }
        if !props.revision.is_empty() {
            new_props.revision = Some(props.revision.clone());
        } else {
            new_props.revision = None;
        }
        if !props.subject.is_empty() {
            new_props.subject = Some(props.subject.clone());
        } else {
            new_props.subject = None;
        }
        if !props.title.is_empty() {
            new_props.title = Some(props.title.clone());
        } else {
            new_props.title = None;
        }
        if !props.language.is_empty() {
            new_props.language = Some(props.language.clone());
        } else {
            new_props.language = None;
        }
        if !props.version.is_empty() {
            new_props.version = Some(props.version.clone());
        } else {
            new_props.version = None;
        }

        if !props.created.is_empty() {
            new_props.created = Some(XlsxDcTerms {
                text: props.created.clone(),
                r#type: Some("dcterms:W3CDTF".to_string()),
            });
        }
        if !props.modified.is_empty() {
            new_props.modified = Some(XlsxDcTerms {
                text: props.modified.clone(),
                r#type: Some("dcterms:W3CDTF".to_string()),
            });
        }

        self.doc_props_core_writer(&new_props);
        Ok(())
    }

    /// Get the document core properties.
    pub fn get_doc_props(&self) -> Result<DocProperties> {
        let core = self.doc_props_core_reader()?;
        let mut ret = DocProperties {
            category: core.category.unwrap_or_default(),
            content_status: core.content_status.unwrap_or_default(),
            created: String::new(),
            creator: core.creator.unwrap_or_default(),
            description: core.description.unwrap_or_default(),
            identifier: core.identifier.unwrap_or_default(),
            keywords: core.keywords.unwrap_or_default(),
            last_modified_by: core.last_modified_by.unwrap_or_default(),
            modified: String::new(),
            revision: core.revision.unwrap_or_default(),
            subject: core.subject.unwrap_or_default(),
            title: core.title.unwrap_or_default(),
            language: core.language.unwrap_or_default(),
            version: core.version.unwrap_or_default(),
        };
        if let Some(c) = core.created {
            ret.created = c.text;
        }
        if let Some(m) = core.modified {
            ret.modified = m.text;
        }
        Ok(ret)
    }

    /// Set a custom document property by name and value. A property with a
    /// `None` value is deleted.
    pub fn set_custom_props(&mut self, prop: &CustomProperty) -> Result<()> {
        if prop.name.is_empty() {
            return Err(Box::new(ErrParameterInvalid));
        }

        let custom = self.custom_properties_reader()?;

        let mut by_name: HashMap<String, XlsxProperty> = HashMap::new();
        let mut max_pid: i64 = 1;
        for p in &custom.property {
            let name = p.name.clone().unwrap_or_default();
            if !name.is_empty() {
                max_pid = max_pid.max(p.pid);
                by_name.insert(name, decode_property_to_xlsx(p));
            }
        }

        match &prop.value {
            None => {
                by_name.remove(&prop.name);
            }
            Some(value) => {
                let pid = if let Some(existing) = by_name.get(&prop.name) {
                    existing.pid
                } else {
                    max_pid += 1;
                    max_pid
                };
                by_name.insert(
                    prop.name.clone(),
                    XlsxProperty {
                        fmt_id: EXT_URI_CUSTOM_PROPERTY_FMT_ID.to_string(),
                        pid,
                        name: Some(prop.name.clone()),
                        link_target: None,
                        ..custom_value_to_xlsx(value)?
                    },
                );
            }
        }

        self.ensure_custom_properties_relationship();
        self.add_custom_properties_content_type()?;

        let mut properties: Vec<XlsxProperty> = by_name.into_values().collect();
        properties.sort_by_key(|p| p.pid);
        self.custom_properties_writer(&XlsxCustomProperties {
            xmlns: Some(NAMESPACE_CUSTOM_PROPERTIES.to_string()),
            vt: Some(NAMESPACE_DOCUMENT_PROPERTIES_VARIANT_TYPES.to_string()),
            property: properties,
        });
        Ok(())
    }

    /// Get all custom document properties.
    pub fn get_custom_props(&self) -> Result<Vec<CustomProperty>> {
        let custom = self.custom_properties_reader()?;
        let mut props = Vec::new();
        for p in &custom.property {
            props.push(CustomProperty {
                name: p.name.clone().unwrap_or_default(),
                value: decode_property_value(p),
            });
        }
        Ok(props)
    }

    /// Delete a custom document property by name.
    pub fn delete_custom_props(&mut self, name: &str) -> Result<()> {
        if name.is_empty() {
            return Err(Box::new(ErrParameterInvalid));
        }
        self.set_custom_props(&CustomProperty {
            name: name.to_string(),
            value: None,
        })
    }

    // ------------------------------------------------------------------
    // Internal readers / writers
    // ------------------------------------------------------------------

    fn doc_props_app_reader(&self) -> Result<XlsxProperties> {
        let data = crate::file::namespace_strict_to_transitional(
            &self.read_xml(DEFAULT_XML_PATH_DOC_PROPS_APP),
        );
        if data.is_empty() {
            return Ok(XlsxProperties::default());
        }
        if !self.xml_attr.contains_key(DEFAULT_XML_PATH_DOC_PROPS_APP) {
            if let Some(attrs) = crate::file::extract_root_namespace_attributes(&data) {
                self.xml_attr
                    .insert(DEFAULT_XML_PATH_DOC_PROPS_APP.to_string(), attrs);
            }
        }
        Ok(xml_from_reader(data.as_slice())?)
    }

    fn doc_props_core_reader(&self) -> Result<DecodeCoreProperties> {
        let data = crate::file::namespace_strict_to_transitional(
            &self.read_xml(DEFAULT_XML_PATH_DOC_PROPS_CORE),
        );
        if data.is_empty() {
            return Ok(DecodeCoreProperties::default());
        }
        if !self.xml_attr.contains_key(DEFAULT_XML_PATH_DOC_PROPS_CORE) {
            if let Some(attrs) = crate::file::extract_root_namespace_attributes(&data) {
                self.xml_attr
                    .insert(DEFAULT_XML_PATH_DOC_PROPS_CORE.to_string(), attrs);
            }
        }
        Ok(xml_from_reader(data.as_slice())?)
    }

    fn custom_properties_reader(&self) -> Result<DecodeCustomProperties> {
        let data = crate::file::namespace_strict_to_transitional(
            &self.read_xml(DEFAULT_XML_PATH_DOC_PROPS_CUSTOM),
        );
        if data.is_empty() {
            return Ok(DecodeCustomProperties::default());
        }
        Ok(xml_from_reader(data.as_slice())?)
    }

    fn doc_props_app_writer(&self, props: &XlsxProperties) {
        if let Ok(mut output) = xml_to_string(props).map(|s| s.into_bytes()) {
            self.replace_namespace_bytes_if_needed(DEFAULT_XML_PATH_DOC_PROPS_APP, &mut output);
            self.save_file_list(DEFAULT_XML_PATH_DOC_PROPS_APP, &output);
        }
    }

    fn doc_props_core_writer(&self, props: &XlsxCoreProperties) {
        if let Ok(mut output) = xml_to_string(props).map(|s| s.into_bytes()) {
            self.replace_namespace_bytes_if_needed(DEFAULT_XML_PATH_DOC_PROPS_CORE, &mut output);
            // The serialization struct uses the local name `coreProperties`;
            // OOXML requires the `cp` prefix on this element.
            if let Ok(s) = std::str::from_utf8(&output) {
                let s = s
                    .replacen("<coreProperties", "<cp:coreProperties", 1)
                    .replacen("</coreProperties>", "</cp:coreProperties>", 1);
                output = s.into_bytes();
            }
            self.save_file_list(DEFAULT_XML_PATH_DOC_PROPS_CORE, &output);
        }
    }

    fn custom_properties_writer(&self, props: &XlsxCustomProperties) {
        if let Ok(output) = xml_to_string(props).map(|s| s.into_bytes()) {
            self.save_file_list(DEFAULT_XML_PATH_DOC_PROPS_CUSTOM, &output);
        }
    }

    // ------------------------------------------------------------------
    // Helpers
    // ------------------------------------------------------------------

    fn ensure_custom_properties_relationship(&self) {
        if let Ok(Some(rels)) = self.rels_reader(DEFAULT_XML_PATH_RELS) {
            if rels.relationships.iter().any(|r| {
                r.r#type == SOURCE_RELATIONSHIP_CUSTOM_PROPERTIES
                    && r.target == DEFAULT_XML_PATH_DOC_PROPS_CUSTOM
            }) {
                return;
            }
        }
        self.add_rels(
            DEFAULT_XML_PATH_RELS,
            SOURCE_RELATIONSHIP_CUSTOM_PROPERTIES,
            DEFAULT_XML_PATH_DOC_PROPS_CUSTOM,
            "",
        );
    }

    fn add_custom_properties_content_type(&self) -> Result<()> {
        let mut ct = self.content_types_reader()?;
        let exists = ct.entries.iter().any(|e| match e {
            XlsxContentTypeEntry::Override(o) => {
                o.part_name == "/docProps/custom.xml"
                    && o.content_type == CONTENT_TYPE_CUSTOM_PROPERTIES
            }
            _ => false,
        });
        if !exists {
            ct.entries
                .push(XlsxContentTypeEntry::Override(XlsxOverride {
                    part_name: "/docProps/custom.xml".to_string(),
                    content_type: CONTENT_TYPE_CUSTOM_PROPERTIES.to_string(),
                }));
            *self.content_types.lock().unwrap() = Some(ct);
        }
        Ok(())
    }
}

fn non_empty(s: String) -> Option<String> {
    if s.is_empty() { None } else { Some(s) }
}

fn decode_property_to_xlsx(p: &DecodeProperty) -> XlsxProperty {
    XlsxProperty {
        fmt_id: p.fmt_id.clone(),
        pid: p.pid,
        name: p.name.clone(),
        link_target: p.link_target.clone(),
        vector: p.vector.clone(),
        array: p.array.clone(),
        blob: p.blob.clone(),
        oblob: p.oblob.clone(),
        empty: p.empty.clone(),
        null: p.null.clone(),
        i1: p.i1,
        i2: p.i2,
        i4: p.i4,
        i8: p.i8,
        int: p.int,
        ui1: p.ui1,
        ui2: p.ui2,
        ui4: p.ui4,
        ui8: p.ui8,
        uint: p.uint,
        r4: p.r4,
        r8: p.r8,
        decimal: p.decimal.clone(),
        lpstr: p.lpstr.clone(),
        lpwstr: p.lpwstr.clone(),
        bstr: p.bstr.clone(),
        date: p.date.clone(),
        file_time: p.file_time.clone(),
        r#bool: p.r#bool,
        cy: p.cy.clone(),
        error: p.error.clone(),
        stream: p.stream.clone(),
        ostream: p.ostream.clone(),
        storage: p.storage.clone(),
        ostorage: p.ostorage.clone(),
        vstream: p.vstream.clone(),
        cls_id: p.cls_id.clone(),
    }
}

fn custom_value_to_xlsx(value: &CustomPropertyValue) -> Result<XlsxProperty> {
    let mut p = XlsxProperty::default();
    match value {
        CustomPropertyValue::Int(v) => p.i4 = Some(*v),
        CustomPropertyValue::Float(v) => p.r8 = Some(*v),
        CustomPropertyValue::Bool(v) => p.r#bool = Some(*v),
        CustomPropertyValue::String(v) => p.lpwstr = Some(v.clone()),
        CustomPropertyValue::Date(v) => {
            validate_date_string(v)?;
            p.file_time = Some(v.clone());
        }
    }
    Ok(p)
}

fn validate_date_string(s: &str) -> Result<()> {
    if DateTime::parse_from_rfc3339(s).is_ok() {
        return Ok(());
    }
    if NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%.f").is_ok() {
        return Ok(());
    }
    Err(Box::new(ErrParameterInvalid))
}

fn decode_property_value(p: &DecodeProperty) -> Option<CustomPropertyValue> {
    if let Some(v) = p.i1 {
        return Some(CustomPropertyValue::Int(v as i32));
    }
    if let Some(v) = p.i2 {
        return Some(CustomPropertyValue::Int(v as i32));
    }
    if let Some(v) = p.i4 {
        return Some(CustomPropertyValue::Int(v));
    }
    if let Some(v) = p.i8 {
        return Some(CustomPropertyValue::Int(v as i32));
    }
    if let Some(v) = p.int {
        return Some(CustomPropertyValue::Int(v as i32));
    }
    if let Some(v) = p.ui1 {
        return Some(CustomPropertyValue::Int(v as i32));
    }
    if let Some(v) = p.ui2 {
        return Some(CustomPropertyValue::Int(v as i32));
    }
    if let Some(v) = p.ui4 {
        return Some(CustomPropertyValue::Int(v as i32));
    }
    if let Some(v) = p.ui8 {
        return Some(CustomPropertyValue::Int(v as i32));
    }
    if let Some(v) = p.uint {
        return Some(CustomPropertyValue::Int(v as i32));
    }
    if let Some(v) = p.r4 {
        return Some(CustomPropertyValue::Float(v as f64));
    }
    if let Some(v) = p.r8 {
        return Some(CustomPropertyValue::Float(v));
    }
    if let Some(v) = p.r#bool {
        return Some(CustomPropertyValue::Bool(v));
    }
    if let Some(v) = &p.lpwstr {
        return Some(CustomPropertyValue::String(v.clone()));
    }
    if let Some(v) = &p.file_time {
        return Some(CustomPropertyValue::Date(v.clone()));
    }
    if let Some(v) = &p.lpstr {
        return Some(CustomPropertyValue::String(v.clone()));
    }
    None
}
