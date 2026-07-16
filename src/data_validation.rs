//! Data validation support.
//!
//! Ported from Go `datavalidation.go`.

use std::collections::HashMap;

use quick_xml::de::from_reader as xml_from_reader;
use quick_xml::se::to_string as xml_to_string;
use serde::{Deserialize, Serialize};

use crate::File;
use crate::constants::{
    EXT_URI_DATA_VALIDATIONS, MAX_FIELD_LENGTH, NAMESPACE_SPREADSHEET_EXCEL_2006_MAIN,
    NAMESPACE_SPREADSHEET_X14,
};
use crate::errors::{
    ErrDataValidationFormulaLength, ErrDataValidationRange, ErrParameterInvalid, ErrSheetNotExist,
    Result,
};
use crate::lib_util::{
    coordinates_to_cell_name, coordinates_to_range_ref, count_utf16_string, flat_sqref,
    in_coordinates,
};
use crate::xml::common::XlsxInnerXml;
use crate::xml::worksheet::{
    DataValidation, XlsxDataValidation, XlsxDataValidations, XlsxWorksheet, XlsxX14DataValidation,
    XlsxX14DataValidations,
};

/// Data validation type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DataValidationType(pub u8);

impl DataValidationType {
    pub const NONE: DataValidationType = DataValidationType(0);
    pub const CUSTOM: DataValidationType = DataValidationType(1);
    pub const DATE: DataValidationType = DataValidationType(2);
    pub const DECIMAL: DataValidationType = DataValidationType(3);
    pub const LIST: DataValidationType = DataValidationType(4);
    pub const TEXT_LENGTH: DataValidationType = DataValidationType(5);
    pub const TIME: DataValidationType = DataValidationType(6);
    pub const WHOLE: DataValidationType = DataValidationType(7);
}

/// Data validation error style.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DataValidationErrorStyle(pub u8);

impl DataValidationErrorStyle {
    pub const STOP: DataValidationErrorStyle = DataValidationErrorStyle(1);
    pub const WARNING: DataValidationErrorStyle = DataValidationErrorStyle(2);
    pub const INFORMATION: DataValidationErrorStyle = DataValidationErrorStyle(3);
}

/// Data validation operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DataValidationOperator(pub u8);

impl DataValidationOperator {
    pub const BETWEEN: DataValidationOperator = DataValidationOperator(1);
    pub const EQUAL: DataValidationOperator = DataValidationOperator(2);
    pub const GREATER_THAN: DataValidationOperator = DataValidationOperator(3);
    pub const GREATER_THAN_OR_EQUAL: DataValidationOperator = DataValidationOperator(4);
    pub const LESS_THAN: DataValidationOperator = DataValidationOperator(5);
    pub const LESS_THAN_OR_EQUAL: DataValidationOperator = DataValidationOperator(6);
    pub const NOT_BETWEEN: DataValidationOperator = DataValidationOperator(7);
    pub const NOT_EQUAL: DataValidationOperator = DataValidationOperator(8);
}

/// Value accepted by [`DataValidation::set_range`].
#[derive(Debug, Clone, PartialEq)]
pub enum DataValidationValue {
    /// No value (corresponds to Go `nil`).
    None,
    /// Integer formula value.
    Integer(i64),
    /// Floating point formula value.
    Float(f64),
    /// Literal formula text.
    Text(String),
}

impl From<i64> for DataValidationValue {
    fn from(v: i64) -> Self {
        DataValidationValue::Integer(v)
    }
}

impl From<f64> for DataValidationValue {
    fn from(v: f64) -> Self {
        DataValidationValue::Float(v)
    }
}

impl From<&str> for DataValidationValue {
    fn from(v: &str) -> Self {
        DataValidationValue::Text(v.to_string())
    }
}

impl From<String> for DataValidationValue {
    fn from(v: String) -> Self {
        DataValidationValue::Text(v)
    }
}

fn dv_type_str(t: DataValidationType) -> &'static str {
    match t {
        DataValidationType::NONE => "none",
        DataValidationType::CUSTOM => "custom",
        DataValidationType::DATE => "date",
        DataValidationType::DECIMAL => "decimal",
        DataValidationType::LIST => "list",
        DataValidationType::TEXT_LENGTH => "textLength",
        DataValidationType::TIME => "time",
        DataValidationType::WHOLE => "whole",
        _ => "",
    }
}

fn dv_error_style_str(s: DataValidationErrorStyle) -> &'static str {
    match s {
        DataValidationErrorStyle::WARNING => "warning",
        DataValidationErrorStyle::INFORMATION => "information",
        _ => "stop",
    }
}

fn dv_operator_str(o: DataValidationOperator) -> &'static str {
    match o {
        DataValidationOperator::BETWEEN => "between",
        DataValidationOperator::EQUAL => "equal",
        DataValidationOperator::GREATER_THAN => "greaterThan",
        DataValidationOperator::GREATER_THAN_OR_EQUAL => "greaterThanOrEqual",
        DataValidationOperator::LESS_THAN => "lessThan",
        DataValidationOperator::LESS_THAN_OR_EQUAL => "lessThanOrEqual",
        DataValidationOperator::NOT_BETWEEN => "notBetween",
        DataValidationOperator::NOT_EQUAL => "notEqual",
        _ => "",
    }
}

fn escape_formula(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn unescape_formula(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
}

fn unescape_data_validation_formula(val: &str) -> String {
    let s = unescape_formula(val);
    if s.starts_with('"') {
        s.replace("\"\"", "\"")
    } else {
        s
    }
}

/// Create a new data validation rule.
pub fn new_data_validation(allow_blank: bool) -> DataValidation {
    DataValidation {
        allow_blank,
        ..Default::default()
    }
}

impl DataValidation {
    /// Set the error alert for this data validation rule.
    pub fn set_error(&mut self, style: DataValidationErrorStyle, title: &str, msg: &str) {
        self.error = Some(msg.to_string());
        self.error_title = Some(title.to_string());
        self.show_error_message = true;
        self.error_style = Some(dv_error_style_str(style).to_string());
    }

    /// Set the input prompt for this data validation rule.
    pub fn set_input(&mut self, title: &str, msg: &str) {
        self.show_input_message = true;
        self.prompt_title = Some(title.to_string());
        self.prompt = Some(msg.to_string());
    }

    /// Set a fixed drop-down list.
    pub fn set_drop_list(&mut self, keys: &[&str]) -> Result<()> {
        let formula = keys.join(",");
        if MAX_FIELD_LENGTH < count_utf16_string(&formula) {
            return Err(Box::new(ErrDataValidationFormulaLength));
        }
        self.r#type = dv_type_str(DataValidationType::LIST).to_string();
        if formula.starts_with('=') {
            self.formula1 = escape_formula(&formula);
        } else {
            let escaped = escape_formula(&formula).replace('"', "\"\"");
            self.formula1 = format!("\"{escaped}\"");
        }
        Ok(())
    }

    /// Set a numeric range validation.
    pub fn set_range(
        &mut self,
        f1: DataValidationValue,
        f2: DataValidationValue,
        t: DataValidationType,
        o: DataValidationOperator,
    ) -> Result<()> {
        fn gen_formula(val: &DataValidationValue) -> Result<String> {
            match val {
                DataValidationValue::Integer(v) => Ok(format!("{}", v)),
                DataValidationValue::Float(v) => {
                    if v.abs() > f32::MAX as f64 {
                        return Err(Box::new(ErrDataValidationRange));
                    }
                    Ok(format!("{}", v))
                }
                DataValidationValue::Text(s) => Ok(s.clone()),
                DataValidationValue::None => Err(Box::new(ErrParameterInvalid)),
            }
        }
        self.formula1 = gen_formula(&f1)?;
        self.formula2 = gen_formula(&f2)?;
        self.r#type = dv_type_str(t).to_string();
        self.operator = dv_operator_str(o).to_string();
        Ok(())
    }

    /// Set a drop-down list backed by a worksheet range reference.
    pub fn set_sqref_drop_list(&mut self, sqref: &str) {
        self.formula1 = sqref.to_string();
        self.r#type = dv_type_str(DataValidationType::LIST).to_string();
    }

    /// Append a cell reference sequence to this rule's sqref.
    pub fn set_sqref(&mut self, sqref: &str) {
        if self.sqref.is_empty() {
            self.sqref = sqref.to_string();
        } else {
            self.sqref = format!("{} {}", self.sqref, sqref);
        }
    }
}

impl File {
    /// Add a data validation rule to a worksheet.
    pub fn add_data_validation(&self, sheet: &str, dv: &DataValidation) -> Result<()> {
        let path = self
            .get_sheet_xml_path(sheet)
            .ok_or_else(|| ErrSheetNotExist {
                sheet_name: sheet.to_string(),
            })?;
        let mut ws = self.work_sheet_reader(sheet)?;
        if ws.data_validations.is_none() {
            ws.data_validations = Some(XlsxDataValidations::default());
        }
        let dvs = ws.data_validations.as_mut().unwrap();
        let xdv = XlsxDataValidation {
            allow_blank: Some(dv.allow_blank),
            error: dv.error.clone(),
            error_style: dv.error_style.clone(),
            error_title: dv.error_title.clone(),
            operator: Some(dv.operator.clone()).filter(|s| !s.is_empty()),
            prompt: dv.prompt.clone(),
            prompt_title: dv.prompt_title.clone(),
            show_drop_down: Some(dv.show_drop_down),
            show_error_message: Some(dv.show_error_message),
            show_input_message: Some(dv.show_input_message),
            sqref: dv.sqref.clone(),
            r#type: Some(dv.r#type.clone()).filter(|s| !s.is_empty()),
            formula1: if dv.formula1.is_empty() {
                None
            } else {
                Some(XlsxInnerXml {
                    content: dv.formula1.clone(),
                })
            },
            formula2: if dv.formula2.is_empty() {
                None
            } else {
                Some(XlsxInnerXml {
                    content: dv.formula2.clone(),
                })
            },
            xm_sqref: None,
        };
        dvs.data_validation.push(xdv);
        dvs.count = Some(dvs.data_validation.len() as i64);
        self.sheet.insert(path, ws);
        Ok(())
    }

    /// Get the data validation rules for a worksheet.
    pub fn get_data_validations(&self, sheet: &str) -> Result<Vec<DataValidation>> {
        let ws = self.work_sheet_reader(sheet)?;
        let mut out = Vec::new();
        if let Some(dvs) = &ws.data_validations {
            out.extend(get_data_validations(dvs));
        }
        if let Some(ext_lst) = &ws.ext_lst {
            for ext in &ext_lst.ext {
                if ext.uri.as_deref() != Some(EXT_URI_DATA_VALIDATIONS) {
                    continue;
                }
                let dvs = parse_x14_data_validations(&ext.content)?;
                for dv in &dvs.data_validation {
                    out.push(x14_dv_to_public(dv));
                }
            }
        }
        Ok(out)
    }

    /// Delete data validation rules from a worksheet.
    pub fn delete_data_validation(&self, sheet: &str, sqref: &[&str]) -> Result<()> {
        let path = self
            .get_sheet_xml_path(sheet)
            .ok_or_else(|| ErrSheetNotExist {
                sheet_name: sheet.to_string(),
            })?;
        let mut ws = self.work_sheet_reader(sheet)?;
        if ws.data_validations.is_none() && ws.ext_lst.is_none() {
            return Ok(());
        }
        if sqref.is_empty() {
            ws.data_validations = None;
            self.sheet.insert(path, ws);
            return Ok(());
        }
        let del_cells = flat_sqref(&sqref.join(" "))?;
        if let Some(dvs) = ws.data_validations.as_mut() {
            delete_data_validation(dvs, &del_cells)?;
            if dvs.data_validation.is_empty() {
                ws.data_validations = None;
            }
        }
        if ws.ext_lst.is_some() {
            self.delete_x14_data_validation(&mut ws, &del_cells)?;
        }
        self.sheet.insert(path, ws);
        Ok(())
    }
}

fn xlsx_dv_to_public(dv: &XlsxDataValidation) -> DataValidation {
    let mut formula1 = dv
        .formula1
        .as_ref()
        .map(|f| unescape_data_validation_formula(&f.content))
        .unwrap_or_default();
    let mut formula2 = dv
        .formula2
        .as_ref()
        .map(|f| unescape_data_validation_formula(&f.content))
        .unwrap_or_default();
    let mut sqref = dv.sqref.clone();
    if let Some(xm) = &dv.xm_sqref {
        sqref = xm.clone();
        formula1 = formula1
            .trim_start_matches("<xm:f>")
            .trim_end_matches("</xm:f>")
            .to_string();
        formula2 = formula2
            .trim_start_matches("<xm:f>")
            .trim_end_matches("</xm:f>")
            .to_string();
    }
    DataValidation {
        allow_blank: dv.allow_blank.unwrap_or(false),
        error: dv.error.clone(),
        error_style: dv.error_style.clone(),
        error_title: dv.error_title.clone(),
        operator: dv.operator.clone().unwrap_or_default(),
        prompt: dv.prompt.clone(),
        prompt_title: dv.prompt_title.clone(),
        show_drop_down: dv.show_drop_down.unwrap_or(false),
        show_error_message: dv.show_error_message.unwrap_or(false),
        show_input_message: dv.show_input_message.unwrap_or(false),
        sqref,
        r#type: dv.r#type.clone().unwrap_or_default(),
        formula1,
        formula2,
    }
}

fn get_data_validations(dvs: &XlsxDataValidations) -> Vec<DataValidation> {
    dvs.data_validation.iter().map(xlsx_dv_to_public).collect()
}

fn delete_data_validation(
    dvs: &mut XlsxDataValidations,
    del_cells: &HashMap<i32, Vec<Vec<i32>>>,
) -> Result<()> {
    let mut i = 0;
    while i < dvs.data_validation.len() {
        let new_sqref = delete_cells_from_sqref(&dvs.data_validation[i].sqref, del_cells)?;
        dvs.data_validation[i].sqref = new_sqref;
        if dvs.data_validation[i].sqref.is_empty() {
            dvs.data_validation.remove(i);
        } else {
            i += 1;
        }
    }
    dvs.count = Some(dvs.data_validation.len() as i64);
    Ok(())
}

impl File {
    fn delete_x14_data_validation(
        &self,
        ws: &mut XlsxWorksheet,
        del_cells: &HashMap<i32, Vec<Vec<i32>>>,
    ) -> Result<()> {
        let mut x14 = crate::xml::common::XlsxExtLst::default();
        if let Some(ext_lst) = ws.ext_lst.take() {
            for ext in ext_lst.ext {
                if ext.uri.as_deref() != Some(EXT_URI_DATA_VALIDATIONS) {
                    x14.ext.push(ext);
                    continue;
                }
                let dvs = parse_x14_data_validations(&ext.content)?;
                let mut new_dvs = XlsxX14DataValidations {
                    xmlns_xm: Some(NAMESPACE_SPREADSHEET_EXCEL_2006_MAIN.to_string()),
                    count: None,
                    disable_prompts: dvs.disable_prompts,
                    x_window: dvs.x_window,
                    y_window: dvs.y_window,
                    data_validation: Vec::new(),
                };
                for dv in &dvs.data_validation {
                    let sqref_to_delete = dv.xm_sqref.as_deref().unwrap_or(&dv.sqref);
                    let new_sqref = delete_cells_from_sqref(sqref_to_delete, del_cells)?;
                    if new_sqref.is_empty() {
                        continue;
                    }
                    new_dvs.data_validation.push(build_x14_dv(dv, &new_sqref));
                }
                new_dvs.count = Some(new_dvs.data_validation.len() as i64);
                if new_dvs.data_validation.is_empty() {
                    continue;
                }
                let content = xml_to_string(&new_dvs)?;
                x14.ext.push(crate::xml::common::XlsxExt {
                    uri: Some(EXT_URI_DATA_VALIDATIONS.to_string()),
                    xmlns_x14: Some(NAMESPACE_SPREADSHEET_X14.to_string()),
                    xmlns_xm: None,
                    content,
                });
            }
        }
        if x14.ext.is_empty() {
            ws.ext_lst = None;
        } else {
            ws.ext_lst = Some(x14);
        }
        Ok(())
    }
}

pub(crate) fn squash_sqref(cells: &[Vec<i32>]) -> Vec<String> {
    if cells.is_empty() {
        return Vec::new();
    }
    if cells.len() == 1 {
        if let Ok(cell) = coordinates_to_cell_name(cells[0][0], cells[0][1], false) {
            return vec![cell];
        }
        return Vec::new();
    }
    let mut refs = Vec::new();
    let mut l = 0usize;
    let mut r = 0usize;
    for i in 1..cells.len() {
        if cells[i][0] == cells[r][0] && cells[i][1] - cells[r][1] > 1 {
            let mut coords = cells[l].clone();
            coords.extend_from_slice(&cells[r]);
            let ref_str = if l == r {
                coordinates_to_cell_name(cells[l][0], cells[l][1], false).unwrap_or_default()
            } else {
                coordinates_to_range_ref(&coords, false).unwrap_or_default()
            };
            if !ref_str.is_empty() {
                refs.push(ref_str);
            }
            l = i;
            r = i;
        } else {
            r += 1;
        }
    }
    let mut coords = cells[l].clone();
    coords.extend_from_slice(&cells[r]);
    let ref_str = if l == r {
        coordinates_to_cell_name(cells[l][0], cells[l][1], false).unwrap_or_default()
    } else {
        coordinates_to_range_ref(&coords, false).unwrap_or_default()
    };
    if !ref_str.is_empty() {
        refs.push(ref_str);
    }
    refs
}

pub(crate) fn delete_cells_from_sqref(
    sqref: &str,
    del_cells: &HashMap<i32, Vec<Vec<i32>>>,
) -> Result<String> {
    let mut col_cells = flat_sqref(sqref)?;
    for (col, cells) in del_cells {
        for cell in cells {
            if let Some(col_vec) = col_cells.get_mut(col) {
                let idx = in_coordinates(col_vec, cell);
                if idx != -1 {
                    col_vec.remove(idx as usize);
                }
            }
        }
    }
    let mut apply_sqref = Vec::new();
    for col in col_cells.values() {
        apply_sqref.extend(squash_sqref(col));
    }
    Ok(apply_sqref.join(" "))
}

// ------------------------------------------------------------------
// x14 extension list data validations
// ------------------------------------------------------------------

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "dataValidations")]
struct DecodeX14DataValidations {
    #[serde(rename = "@count", default)]
    count: Option<i64>,
    #[serde(rename = "@disablePrompts", default)]
    disable_prompts: Option<bool>,
    #[serde(rename = "@xWindow", default)]
    x_window: Option<i64>,
    #[serde(rename = "@yWindow", default)]
    y_window: Option<i64>,
    #[serde(rename = "dataValidation", default)]
    data_validation: Vec<DecodeX14DataValidation>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "dataValidation")]
struct DecodeX14DataValidation {
    #[serde(rename = "@allowBlank", default)]
    allow_blank: Option<bool>,
    #[serde(rename = "@error", default)]
    error: Option<String>,
    #[serde(rename = "@errorStyle", default)]
    error_style: Option<String>,
    #[serde(rename = "@errorTitle", default)]
    error_title: Option<String>,
    #[serde(rename = "@operator", default)]
    operator: Option<String>,
    #[serde(rename = "@prompt", default)]
    prompt: Option<String>,
    #[serde(rename = "@promptTitle", default)]
    prompt_title: Option<String>,
    #[serde(rename = "@showDropDown", default)]
    show_drop_down: Option<bool>,
    #[serde(rename = "@showErrorMessage", default)]
    show_error_message: Option<bool>,
    #[serde(rename = "@showInputMessage", default)]
    show_input_message: Option<bool>,
    #[serde(rename = "@sqref", default)]
    sqref: String,
    #[serde(rename = "@type", default)]
    r#type: Option<String>,
    #[serde(rename = "formula1", default)]
    formula1: Option<DecodeX14Formula>,
    #[serde(rename = "formula2", default)]
    formula2: Option<DecodeX14Formula>,
    #[serde(rename = "sqref", default)]
    xm_sqref: Option<String>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
struct DecodeX14Formula {
    #[serde(rename = "f", default)]
    f: Option<String>,
}

fn parse_x14_data_validations(content: &str) -> Result<DecodeX14DataValidations> {
    let stripped = content.replace("<x14:", "<").replace("</x14:", "</");
    Ok(xml_from_reader(stripped.as_bytes())?)
}

fn x14_dv_to_public(dv: &DecodeX14DataValidation) -> DataValidation {
    DataValidation {
        allow_blank: dv.allow_blank.unwrap_or(false),
        error: dv.error.clone(),
        error_style: dv.error_style.clone(),
        error_title: dv.error_title.clone(),
        operator: dv.operator.clone().unwrap_or_default(),
        prompt: dv.prompt.clone(),
        prompt_title: dv.prompt_title.clone(),
        show_drop_down: dv.show_drop_down.unwrap_or(false),
        show_error_message: dv.show_error_message.unwrap_or(false),
        show_input_message: dv.show_input_message.unwrap_or(false),
        sqref: dv.xm_sqref.clone().unwrap_or_else(|| dv.sqref.clone()),
        r#type: dv.r#type.clone().unwrap_or_default(),
        formula1: dv
            .formula1
            .as_ref()
            .and_then(|f| f.f.as_ref())
            .map(|s| unescape_data_validation_formula(s))
            .unwrap_or_default(),
        formula2: dv
            .formula2
            .as_ref()
            .and_then(|f| f.f.as_ref())
            .map(|s| unescape_data_validation_formula(s))
            .unwrap_or_default(),
    }
}

fn build_x14_dv(dv: &DecodeX14DataValidation, new_sqref: &str) -> XlsxX14DataValidation {
    XlsxX14DataValidation {
        allow_blank: dv.allow_blank,
        error: dv.error.clone(),
        error_style: dv.error_style.clone(),
        error_title: dv.error_title.clone(),
        operator: dv.operator.clone(),
        prompt: dv.prompt.clone(),
        prompt_title: dv.prompt_title.clone(),
        show_drop_down: dv.show_drop_down,
        show_error_message: dv.show_error_message,
        show_input_message: dv.show_input_message,
        sqref: dv.sqref.clone(),
        r#type: dv.r#type.clone(),
        formula1: dv
            .formula1
            .as_ref()
            .and_then(|f| f.f.as_ref())
            .map(|formula| XlsxInnerXml {
                content: format!("<xm:f>{formula}</xm:f>"),
            }),
        formula2: dv
            .formula2
            .as_ref()
            .and_then(|f| f.f.as_ref())
            .map(|formula| XlsxInnerXml {
                content: format!("<xm:f>{formula}</xm:f>"),
            }),
        xm_sqref: Some(new_sqref.to_string()),
    }
}

// ------------------------------------------------------------------
// Tests
// ------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::xml::common::XlsxExt;

    #[test]
    fn test_new_data_validation() {
        let dv = new_data_validation(true);
        assert!(dv.allow_blank);
        assert!(!dv.show_error_message);
        assert!(!dv.show_input_message);
    }

    #[test]
    fn test_set_error_styles() {
        let mut dv = new_data_validation(true);
        dv.set_error(DataValidationErrorStyle::STOP, "title", "msg");
        assert_eq!(dv.error_style, Some("stop".to_string()));
        dv.set_error(DataValidationErrorStyle::WARNING, "title", "msg");
        assert_eq!(dv.error_style, Some("warning".to_string()));
        dv.set_error(DataValidationErrorStyle::INFORMATION, "title", "msg");
        assert_eq!(dv.error_style, Some("information".to_string()));
    }

    #[test]
    fn test_set_drop_list() {
        let mut dv = new_data_validation(true);
        dv.set_drop_list(&["1", "2", "3"]).unwrap();
        assert_eq!(dv.formula1, "\"1,2,3\"");

        dv.set_drop_list(&["=A1"]).unwrap();
        assert_eq!(dv.formula1, "=A1");

        dv.set_drop_list(&["A<", "B>", "C\"", "D\t", "E'", "F"])
            .unwrap();
        assert_eq!(dv.formula1, "\"A&lt;,B&gt;,C\"\",D\t,E',F\"");

        let long = vec!["s"; 256];
        assert!(
            dv.set_drop_list(&long.iter().map(|s| *s).collect::<Vec<_>>())
                .is_err()
        );
    }

    #[test]
    fn test_set_range() {
        let mut dv = new_data_validation(true);
        dv.set_range(
            10.into(),
            20.into(),
            DataValidationType::WHOLE,
            DataValidationOperator::BETWEEN,
        )
        .unwrap();
        assert_eq!(dv.formula1, "10");
        assert_eq!(dv.formula2, "20");
        assert_eq!(dv.r#type, "whole");
        assert_eq!(dv.operator, "between");

        dv.set_range(
            (-f32::MAX as f64).into(),
            (f32::MAX as f64).into(),
            DataValidationType::WHOLE,
            DataValidationOperator::GREATER_THAN,
        )
        .unwrap();

        assert!(
            dv.set_range(
                (-f64::MAX).into(),
                20.into(),
                DataValidationType::WHOLE,
                DataValidationOperator::GREATER_THAN,
            )
            .is_err()
        );
        assert!(
            dv.set_range(
                DataValidationValue::None,
                20.into(),
                DataValidationType::WHOLE,
                DataValidationOperator::BETWEEN,
            )
            .is_err()
        );
    }

    #[test]
    fn test_set_sqref() {
        let mut dv = new_data_validation(true);
        dv.set_sqref("A1:B2");
        assert_eq!(dv.sqref, "A1:B2");
        dv.set_sqref("A1:B2");
        assert_eq!(dv.sqref, "A1:B2 A1:B2");
    }

    #[test]
    fn test_add_get_delete_data_validation() {
        let f = File::new_with_options(crate::options::Options::default());

        let mut dv = new_data_validation(true);
        dv.sqref = "A1:B2".to_string();
        dv.set_range(
            10.into(),
            20.into(),
            DataValidationType::WHOLE,
            DataValidationOperator::BETWEEN,
        )
        .unwrap();
        dv.set_error(DataValidationErrorStyle::STOP, "error title", "error body");
        f.add_data_validation("Sheet1", &dv).unwrap();

        let dvs = f.get_data_validations("Sheet1").unwrap();
        assert_eq!(dvs.len(), 1);
        assert_eq!(dvs[0].sqref, "A1:B2");
        assert_eq!(dvs[0].r#type, "whole");

        f.delete_data_validation("Sheet1", &["A1:B2"]).unwrap();
        let dvs = f.get_data_validations("Sheet1").unwrap();
        assert!(dvs.is_empty());
    }

    #[test]
    fn test_delete_partial_sqref() {
        let f = File::new_with_options(crate::options::Options::default());

        let mut dv = new_data_validation(true);
        dv.sqref = "C2:C5".to_string();
        dv.set_range(
            10.into(),
            20.into(),
            DataValidationType::WHOLE,
            DataValidationOperator::BETWEEN,
        )
        .unwrap();
        f.add_data_validation("Sheet1", &dv).unwrap();

        f.delete_data_validation("Sheet1", &["C4"]).unwrap();
        let dvs = f.get_data_validations("Sheet1").unwrap();
        assert_eq!(dvs.len(), 1);
        assert_eq!(dvs[0].sqref, "C2:C3 C5");
    }

    #[test]
    fn test_delete_unordered_sqref() {
        let f = File::new_with_options(crate::options::Options::default());
        let path = f.get_sheet_xml_path("Sheet1").unwrap();
        let mut ws = f.work_sheet_reader("Sheet1").unwrap();
        ws.data_validations = Some(XlsxDataValidations {
            count: Some(1),
            data_validation: vec![XlsxDataValidation {
                allow_blank: Some(true),
                show_input_message: Some(true),
                show_error_message: Some(true),
                r#type: Some("whole".to_string()),
                operator: Some("between".to_string()),
                sqref: "A5:A10 A15:A20 A3:A4".to_string(),
                formula1: Some(XlsxInnerXml {
                    content: "1".to_string(),
                }),
                formula2: Some(XlsxInnerXml {
                    content: "100".to_string(),
                }),
                ..Default::default()
            }],
            ..Default::default()
        });
        f.sheet.insert(path, ws);

        f.delete_data_validation("Sheet1", &["A7"]).unwrap();
        let dvs = f.get_data_validations("Sheet1").unwrap();
        assert_eq!(dvs.len(), 1);
        assert_eq!(dvs[0].sqref, "A3:A6 A8:A10 A15:A20");
    }

    #[test]
    fn test_get_data_validations_ext_lst() {
        let f = File::new_with_options(crate::options::Options::default());
        let path = f.get_sheet_xml_path("Sheet1").unwrap();
        let mut ws = f.work_sheet_reader("Sheet1").unwrap();
        ws.ext_lst = Some(crate::xml::common::XlsxExtLst {
            ext: vec![XlsxExt {
                uri: Some(EXT_URI_DATA_VALIDATIONS.to_string()),
                xmlns_x14: Some(NAMESPACE_SPREADSHEET_X14.to_string()),
                xmlns_xm: Some(NAMESPACE_SPREADSHEET_EXCEL_2006_MAIN.to_string()),
                content: r#"<x14:dataValidations xmlns:xm="http://schemas.microsoft.com/office/excel/2006/main"><x14:dataValidation type="list" allowBlank="1"><x14:formula1><xm:f>Sheet1!$B$1:$B$5</xm:f></x14:formula1><xm:sqref>A7:B8</xm:sqref></x14:dataValidation></x14:dataValidations>"#.to_string(),
            }],
        });
        f.sheet.insert(path, ws);

        let dvs = f.get_data_validations("Sheet1").unwrap();
        assert_eq!(dvs.len(), 1);
        assert_eq!(dvs[0].allow_blank, true);
        assert_eq!(dvs[0].r#type, "list");
        assert_eq!(dvs[0].formula1, "Sheet1!$B$1:$B$5");
        assert_eq!(dvs[0].sqref, "A7:B8");
    }

    #[test]
    fn test_delete_data_validation_ext_lst() {
        let f = File::new_with_options(crate::options::Options::default());
        let path = f.get_sheet_xml_path("Sheet1").unwrap();
        let mut ws = f.work_sheet_reader("Sheet1").unwrap();
        ws.ext_lst = Some(crate::xml::common::XlsxExtLst {
            ext: vec![XlsxExt {
                uri: Some(EXT_URI_DATA_VALIDATIONS.to_string()),
                xmlns_x14: Some(NAMESPACE_SPREADSHEET_X14.to_string()),
                xmlns_xm: Some(NAMESPACE_SPREADSHEET_EXCEL_2006_MAIN.to_string()),
                content: format!(
                    r#"<x14:dataValidations xmlns:xm="{}"><x14:dataValidation allowBlank="true" showErrorMessage="true" showInputMessage="true" sqref="" type="list"><xm:sqref>A1:A2</xm:sqref><x14:formula1><xm:f>Sheet1!$A$2:$A$4</xm:f></x14:formula1></x14:dataValidation><x14:dataValidation allowBlank="true" showErrorMessage="true" showInputMessage="true" sqref="" type="list"><xm:sqref>B1:B2</xm:sqref><x14:formula1><xm:f>Sheet1!$B$2:$B$3</xm:f></x14:formula1></x14:dataValidation></x14:dataValidations>"#,
                    NAMESPACE_SPREADSHEET_EXCEL_2006_MAIN
                ),
            }],
        });
        f.sheet.insert(path, ws);

        f.delete_data_validation("Sheet1", &["A1:A2"]).unwrap();
        let dvs = f.get_data_validations("Sheet1").unwrap();
        assert_eq!(dvs.len(), 1);
        assert_eq!(dvs[0].sqref, "B1:B2");

        f.delete_data_validation("Sheet1", &["B1:B2"]).unwrap();
        let dvs = f.get_data_validations("Sheet1").unwrap();
        assert!(dvs.is_empty());
    }

    #[test]
    fn test_squash_sqref() {
        let cells = vec![vec![1, 1], vec![1, 2], vec![1, 4], vec![1, 5], vec![1, 6]];
        let refs = squash_sqref(&cells);
        assert_eq!(refs, vec!["A1:A2", "A4:A6"]);
    }

    #[test]
    fn test_data_validation_round_trip() {
        use std::fs;

        let tmp = std::env::temp_dir().join("excelize_rust_data_validation.xlsx");
        let _ = fs::remove_file(&tmp);

        let mut f = File::new_with_options(crate::options::Options::default());
        let mut dv = new_data_validation(true);
        dv.sqref = "A1:B2".to_string();
        dv.set_range(
            10.into(),
            20.into(),
            DataValidationType::WHOLE,
            DataValidationOperator::BETWEEN,
        )
        .unwrap();
        dv.set_error(DataValidationErrorStyle::STOP, "error title", "error body");
        f.add_data_validation("Sheet1", &dv).unwrap();

        let mut dv2 = new_data_validation(true);
        dv2.sqref = "A3:B4".to_string();
        dv2.set_drop_list(&["1", "2", "3"]).unwrap();
        dv2.set_input("input title", "input body");
        f.add_data_validation("Sheet1", &dv2).unwrap();

        f.save_as(tmp.to_str().unwrap()).unwrap();

        let f2 =
            File::open_file(tmp.to_str().unwrap(), crate::options::Options::default()).unwrap();
        let dvs = f2.get_data_validations("Sheet1").unwrap();
        assert_eq!(dvs.len(), 2);
        assert_eq!(dvs[0].sqref, "A1:B2");
        assert_eq!(dvs[1].sqref, "A3:B4");

        let _ = fs::remove_file(&tmp);
    }
}
