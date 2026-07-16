//! Cell-level read/write API.
//!
//! This module corresponds to `cell.go` in the Go implementation.

use std::time::Duration;

use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

use crate::constants::{
    SOURCE_RELATIONSHIP, SOURCE_RELATIONSHIP_HYPER_LINK, TOTAL_SHEET_HYPERLINKS,
};
use crate::date;
use crate::errors::Result;
use crate::errors::{
    ErrCellCharsLength, ErrCoordinates, ErrInvalidFormula, ErrParameterInvalid,
    ErrTotalSheetHyperlinks,
};
use crate::file::File;
use crate::lib_util::{
    cell_name_to_coordinates, coordinates_to_cell_name, range_ref_to_coordinates, sort_coordinates,
    split_cell_name,
};
use crate::numfmt;
use crate::xml::common::{RichTextRun, XlsxR, XlsxT};
use crate::xml::shared_strings::XlsxSi;
use crate::xml::styles::{XlsxNumFmt, XlsxXf};
use crate::xml::worksheet::{XlsxC, XlsxF, XlsxHyperlink, XlsxHyperlinks, XlsxWorksheet};

// ------------------------------------------------------------------
// Public cell value types
// ------------------------------------------------------------------

/// Value that can be written to a worksheet cell.
#[derive(Debug, Clone, PartialEq)]
pub enum CellValue {
    /// Plain text or string value.
    String(String),
    /// Integer value.
    Int(i64),
    /// Floating point value.
    Float(f64),
    /// Boolean value.
    Bool(bool),
    /// Formula string (without leading `=`).
    Formula(String),
    /// Date/time value.
    DateTime(NaiveDateTime),
    /// Date value.
    Date(NaiveDate),
    /// Time value.
    Time(NaiveTime),
    /// Time duration stored as a fraction of a day.
    Duration(Duration),
    /// Rich text runs.
    RichText(Vec<RichTextRun>),
}

impl From<&str> for CellValue {
    fn from(v: &str) -> Self {
        CellValue::String(v.to_string())
    }
}

impl From<String> for CellValue {
    fn from(v: String) -> Self {
        CellValue::String(v)
    }
}

impl From<i64> for CellValue {
    fn from(v: i64) -> Self {
        CellValue::Int(v)
    }
}

impl From<i32> for CellValue {
    fn from(v: i32) -> Self {
        CellValue::Int(v as i64)
    }
}

impl From<u64> for CellValue {
    fn from(v: u64) -> Self {
        CellValue::Int(v as i64)
    }
}

impl From<f64> for CellValue {
    fn from(v: f64) -> Self {
        CellValue::Float(v)
    }
}

impl From<bool> for CellValue {
    fn from(v: bool) -> Self {
        CellValue::Bool(v)
    }
}

impl From<NaiveDateTime> for CellValue {
    fn from(v: NaiveDateTime) -> Self {
        CellValue::DateTime(v)
    }
}

impl From<NaiveDate> for CellValue {
    fn from(v: NaiveDate) -> Self {
        CellValue::Date(v)
    }
}

impl From<NaiveTime> for CellValue {
    fn from(v: NaiveTime) -> Self {
        CellValue::Time(v)
    }
}

impl From<Duration> for CellValue {
    fn from(v: Duration) -> Self {
        CellValue::Duration(v)
    }
}

impl From<Vec<RichTextRun>> for CellValue {
    fn from(v: Vec<RichTextRun>) -> Self {
        CellValue::RichText(v)
    }
}

// ------------------------------------------------------------------
// Cell data type
// ------------------------------------------------------------------

/// Cell data type enumeration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellType {
    /// Unset / blank.
    Unset,
    /// Boolean.
    Bool,
    /// ISO 8601 date.
    Date,
    /// Error.
    Error,
    /// Formula string.
    Formula,
    /// Inline string.
    InlineString,
    /// Number.
    Number,
    /// Shared string.
    SharedString,
}

impl Default for CellType {
    fn default() -> Self {
        CellType::Unset
    }
}

impl CellType {
    fn from_type_attr(t: Option<&str>) -> Self {
        match t {
            Some("b") => CellType::Bool,
            Some("d") => CellType::Date,
            Some("e") => CellType::Error,
            Some("str") => CellType::Formula,
            Some("inlineStr") => CellType::InlineString,
            Some("s") => CellType::SharedString,
            Some(_) => CellType::Number,
            None => CellType::Unset,
        }
    }
}

// ------------------------------------------------------------------
// Formula / hyperlink options
// ------------------------------------------------------------------

/// Optional settings for [`File::set_cell_formula_with_opts`].
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct FormulaOpts {
    /// Formula type (`array`, `shared`, `dataTable`, ...).
    pub r#type: Option<String>,
    /// Reference range for array/shared formulas.
    pub r#ref: Option<String>,
}

/// Optional settings for [`File::set_cell_hyperlink`].
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct HyperlinkOpts {
    /// Display text for the hyperlink.
    pub display: Option<String>,
    /// Tooltip text for the hyperlink.
    pub tooltip: Option<String>,
}

// ------------------------------------------------------------------
// Public API
// ------------------------------------------------------------------

impl File {
    /// Set the value of a cell.
    pub fn set_cell_value(
        &self,
        sheet: &str,
        cell: &str,
        value: impl Into<CellValue>,
    ) -> Result<()> {
        let value = value.into();
        self.set_cell_value_internal(sheet, cell, value)
    }

    /// Get the raw string value of a cell.
    pub fn get_cell_value(&self, sheet: &str, cell: &str) -> Result<String> {
        let raw = self.options.lock().unwrap().raw_cell_value;
        self.get_cell_value_with_options(sheet, cell, raw)
    }

    /// Get the formatted value of a cell, optionally bypassing number formatting.
    pub fn get_cell_value_with_options(
        &self,
        sheet: &str,
        cell: &str,
        raw: bool,
    ) -> Result<String> {
        let ws = self.work_sheet_reader(sheet)?;
        let cell = merge_cells_parser(&ws, cell);
        let Some(c) = find_cell(&ws, &cell) else {
            return Ok(String::new());
        };
        Ok(read_cell_value(self, c, raw))
    }

    /// Get the data type of a cell.
    pub fn get_cell_type(&self, sheet: &str, cell: &str) -> Result<CellType> {
        let ws = self.work_sheet_reader(sheet)?;
        let cell = merge_cells_parser(&ws, cell);
        let Some(c) = find_cell(&ws, &cell) else {
            return Ok(CellType::Unset);
        };
        if c.f.is_some() {
            return Ok(CellType::Formula);
        }
        Ok(CellType::from_type_attr(c.t.as_deref()))
    }

    /// Set a cell to a string value.
    pub fn set_cell_str(&self, sheet: &str, cell: &str, value: &str) -> Result<()> {
        self.set_cell_value(sheet, cell, CellValue::String(value.to_string()))
    }

    /// Set a cell to an integer value.
    pub fn set_cell_int(&self, sheet: &str, cell: &str, value: i64) -> Result<()> {
        self.set_cell_value(sheet, cell, CellValue::Int(value))
    }

    /// Set a cell to an unsigned integer value.
    pub fn set_cell_uint(&self, sheet: &str, cell: &str, value: u64) -> Result<()> {
        self.set_cell_value(sheet, cell, CellValue::Int(value as i64))
    }

    /// Set a cell to a floating point value.
    pub fn set_cell_float(&self, sheet: &str, cell: &str, value: f64) -> Result<()> {
        self.set_cell_float_with_precision(sheet, cell, value, -1, 64)
    }

    /// Set a cell to a floating point value with explicit precision and bit size.
    ///
    /// `precision` of `-1` uses as many decimal places as necessary. `bit_size`
    /// should be `32` for `f32` values and `64` for `f64` values.
    pub fn set_cell_float_with_precision(
        &self,
        sheet: &str,
        cell: &str,
        value: f64,
        precision: i32,
        bit_size: i32,
    ) -> Result<()> {
        if value.is_nan() || value.is_infinite() {
            return self.set_cell_str(sheet, cell, &value.to_string());
        }
        let path =
            self.get_sheet_xml_path(sheet)
                .ok_or_else(|| crate::errors::ErrSheetNotExist {
                    sheet_name: sheet.to_string(),
                })?;
        let mut ws = self.work_sheet_reader(sheet)?;
        let c = get_or_make_cell(&mut ws, cell);
        let v = if precision < 0 {
            if bit_size <= 32 {
                format!("{}", value as f32)
            } else {
                format!("{}", value)
            }
        } else if bit_size <= 32 {
            format!("{:.*}", precision as usize, value as f32)
        } else {
            format!("{:.*}", precision as usize, value)
        };
        c.t = None;
        c.v = Some(v);
        c.f = None;
        c.is = None;
        update_dimension(&mut ws)?;
        self.sheet.insert(path, ws);
        Ok(())
    }

    /// Set a cell to a boolean value.
    pub fn set_cell_bool(&self, sheet: &str, cell: &str, value: bool) -> Result<()> {
        self.set_cell_value(sheet, cell, CellValue::Bool(value))
    }

    /// Set a cell to a string value without escaping it as a shared string.
    pub fn set_cell_default(&self, sheet: &str, cell: &str, value: &str) -> Result<()> {
        let path =
            self.get_sheet_xml_path(sheet)
                .ok_or_else(|| crate::errors::ErrSheetNotExist {
                    sheet_name: sheet.to_string(),
                })?;
        let mut ws = self.work_sheet_reader(sheet)?;
        let c = get_or_make_cell(&mut ws, cell);
        set_cell_default_value(c, value);
        update_dimension(&mut ws)?;
        self.sheet.insert(path, ws);
        Ok(())
    }

    /// Set a cell formula.
    pub fn set_cell_formula(&self, sheet: &str, cell: &str, formula: &str) -> Result<()> {
        self.set_cell_formula_with_opts(sheet, cell, formula, &[])
    }

    /// Set a cell formula with optional formula type and reference range.
    pub fn set_cell_formula_with_opts(
        &self,
        sheet: &str,
        cell: &str,
        formula: &str,
        opts: &[FormulaOpts],
    ) -> Result<()> {
        let path =
            self.get_sheet_xml_path(sheet)
                .ok_or_else(|| crate::errors::ErrSheetNotExist {
                    sheet_name: sheet.to_string(),
                })?;
        let mut ws = self.work_sheet_reader(sheet)?;
        self.clear_calc_cache();

        if formula.is_empty() {
            if let Some(c) = find_cell_mut(&mut ws, cell) {
                c.f = None;
            }
            self.delete_calc_chain(self.get_sheet_id(sheet), cell)?;
            update_dimension(&mut ws)?;
            self.sheet.insert(path, ws);
            return Ok(());
        }

        if formula.starts_with('=') {
            return Err(Box::new(ErrInvalidFormula));
        }

        let c = get_or_make_cell(&mut ws, cell);
        let mut f = XlsxF {
            content: formula.to_string(),
            ..Default::default()
        };
        for opt in opts {
            if let Some(t) = &opt.r#type {
                f.t = Some(t.clone());
            }
            if let Some(r#ref) = &opt.r#ref {
                f.r#ref = Some(r#ref.clone());
            }
        }
        c.f = Some(f);
        c.is = None;
        update_dimension(&mut ws)?;
        self.sheet.insert(path, ws);
        Ok(())
    }

    /// Get the formula from a cell.
    pub fn get_cell_formula(&self, sheet: &str, cell: &str) -> Result<String> {
        let ws = self.work_sheet_reader(sheet)?;
        let cell = merge_cells_parser(&ws, cell);
        let Some(c) = find_cell(&ws, &cell) else {
            return Ok(String::new());
        };
        let Some(f) = &c.f else {
            return Ok(String::new());
        };
        if f.t.as_deref() == Some("shared") {
            if let Some(si) = f.si {
                if let Some(master) = find_shared_formula_master(&ws, si) {
                    return Ok(master.content.clone());
                }
            }
        }
        Ok(f.content.clone())
    }

    /// Get the style index applied to a cell.
    ///
    /// If the cell has no explicit style, the row style and then the column
    /// style are returned, matching Excel's inheritance behavior.
    pub fn get_cell_style(&self, sheet: &str, cell: &str) -> Result<i32> {
        let ws = self.work_sheet_reader(sheet)?;
        let (col, row) =
            cell_name_to_coordinates(cell).map_err(|_| Box::new(ErrParameterInvalid))?;
        let mut style = 0_i64;
        if let Some(c) = find_cell(&ws, cell) {
            style = c.s.unwrap_or(0);
        }
        if style == 0 {
            if let Some(r) = ws.sheet_data.row.iter().find(|r| r.r == Some(row as i64)) {
                style = r.s.unwrap_or(0);
            }
        }
        if style == 0 {
            if let Some(cols) = &ws.cols {
                for c in &cols.col {
                    if c.min <= col as i64 && col as i64 <= c.max {
                        style = c.style.unwrap_or(0);
                        if style != 0 {
                            break;
                        }
                    }
                }
            }
        }
        Ok(style as i32)
    }

    /// Apply a style index to a range of cells.
    ///
    /// The range is defined by `top_left_cell` and `bottom_right_cell`. The
    /// corners are normalized automatically, so `B3:A1` is equivalent to
    /// `A1:B3`. Passing the same cell for both corners styles a single cell.
    pub fn set_cell_style(
        &self,
        sheet: &str,
        top_left_cell: &str,
        bottom_right_cell: &str,
        style_id: i32,
    ) -> Result<()> {
        if style_id < 0 {
            return Err(crate::errors::new_invalid_style_id_error(style_id).into());
        }
        let style_count = self
            .styles_reader()?
            .cell_xfs
            .as_ref()
            .map(|x| x.xf.len())
            .unwrap_or(1) as i32;
        if style_id >= style_count {
            return Err(crate::errors::new_invalid_style_id_error(style_id).into());
        }
        let (mut h_col, mut h_row) =
            cell_name_to_coordinates(top_left_cell).map_err(|_| Box::new(ErrParameterInvalid))?;
        let (mut v_col, mut v_row) = cell_name_to_coordinates(bottom_right_cell)
            .map_err(|_| Box::new(ErrParameterInvalid))?;
        if v_col < h_col {
            std::mem::swap(&mut v_col, &mut h_col);
        }
        if v_row < h_row {
            std::mem::swap(&mut v_row, &mut h_row);
        }
        let path =
            self.get_sheet_xml_path(sheet)
                .ok_or_else(|| crate::errors::ErrSheetNotExist {
                    sheet_name: sheet.to_string(),
                })?;
        let mut ws = self.work_sheet_reader(sheet)?;
        for row in h_row..=v_row {
            for col in h_col..=v_col {
                let cell = coordinates_to_cell_name(col, row, false)
                    .map_err(|_| Box::new(ErrParameterInvalid))?;
                let c = get_or_make_cell(&mut ws, &cell);
                c.s = Some(style_id as i64);
            }
        }
        self.sheet.insert(path, ws);
        Ok(())
    }

    /// Set rich text runs to a cell.
    pub fn set_cell_rich_text(
        &self,
        sheet: &str,
        cell: &str,
        runs: Vec<RichTextRun>,
    ) -> Result<()> {
        if runs.is_empty() {
            return self.set_cell_str(sheet, cell, "");
        }
        let path =
            self.get_sheet_xml_path(sheet)
                .ok_or_else(|| crate::errors::ErrSheetNotExist {
                    sheet_name: sheet.to_string(),
                })?;
        let mut ws = self.work_sheet_reader(sheet)?;
        let (col, row) = cell_name_to_coordinates(cell).unwrap_or((1, 1));
        let current_s = {
            let c = get_or_make_cell(&mut ws, cell);
            c.s.unwrap_or(0)
        };
        let style_id = prepare_cell_style(&ws, col as i64, row as i64, current_s);
        let c = get_or_make_cell(&mut ws, cell);
        c.s = Some(style_id);

        let si = runs_to_xlsx_si(&runs);
        let mut sst = self.shared_strings_reader().unwrap_or_default();
        let idx = sst
            .si
            .iter()
            .position(|existing| existing == &si)
            .unwrap_or_else(|| {
                sst.si.push(si);
                sst.unique_count += 1;
                sst.si.len() - 1
            });
        sst.count += 1;
        *self.shared_strings.lock().unwrap() = Some(sst);

        c.t = Some("s".to_string());
        c.v = Some(idx.to_string());
        c.f = None;
        c.is = None;
        update_dimension(&mut ws)?;
        self.sheet.insert(path, ws);
        Ok(())
    }

    /// Get rich text runs from a cell.
    pub fn get_cell_rich_text(&self, sheet: &str, cell: &str) -> Result<Vec<RichTextRun>> {
        let ws = self.work_sheet_reader(sheet)?;
        let cell = merge_cells_parser(&ws, cell);
        let Some(c) = find_cell(&ws, &cell) else {
            return Ok(Vec::new());
        };
        if c.t.as_deref() == Some("inlineStr") {
            if let Some(is) = &c.is {
                return Ok(runs_from_xlsx_si(is));
            }
        }
        if c.t.as_deref() == Some("s") {
            if let Some(v) = &c.v {
                if let Ok(idx) = v.parse::<usize>() {
                    let sst = self.shared_strings_reader()?;
                    if let Some(si) = sst.si.get(idx) {
                        return Ok(runs_from_xlsx_si(si));
                    }
                }
            }
        }
        Ok(Vec::new())
    }

    /// Get a cell hyperlink.
    ///
    /// Returns `(true, link)` if the cell has a hyperlink, otherwise
    /// `(false, "")`.
    pub fn get_cell_hyperlink(&self, sheet: &str, cell: &str) -> Result<(bool, String)> {
        split_cell_name(cell).map_err(|_| Box::new(ErrParameterInvalid))?;
        let ws = self.work_sheet_reader(sheet)?;
        let cell = merge_cells_parser(&ws, cell);
        if let Some(links) = &ws.hyperlinks {
            for link in &links.hyperlink {
                if link.r#ref == cell {
                    if let Some(rid) = &link.rid {
                        return Ok((true, self.get_sheet_relationships_target_by_id(sheet, rid)));
                    }
                    return Ok((true, link.location.clone().unwrap_or_default()));
                }
                let ok = check_cell_in_range_ref(&cell, &link.r#ref).unwrap_or(false);
                if ok {
                    if let Some(rid) = &link.rid {
                        return Ok((true, self.get_sheet_relationships_target_by_id(sheet, rid)));
                    }
                    return Ok((true, link.location.clone().unwrap_or_default()));
                }
            }
        }
        Ok((false, String::new()))
    }

    /// Get all cell references which contain hyperlinks in a worksheet.
    ///
    /// `link_type` may be `"External"`, `"Location"`, `"None"` or empty.
    pub fn get_hyperlink_cells(&self, sheet: &str, link_type: &str) -> Result<Vec<String>> {
        let ws = self.work_sheet_reader(sheet)?;
        let mut out = Vec::new();
        if let Some(links) = &ws.hyperlinks {
            for link in &links.hyperlink {
                match link_type {
                    "External" => {
                        if link.rid.is_some() {
                            out.push(link.r#ref.clone());
                        }
                    }
                    "Location" => {
                        if link.location.as_deref().unwrap_or("").is_empty() {
                            continue;
                        }
                        out.push(link.r#ref.clone());
                    }
                    "None" => return Ok(out),
                    "" => out.push(link.r#ref.clone()),
                    _ => return Err(crate::errors::new_invalid_link_type_error(link_type).into()),
                }
            }
        }
        Ok(out)
    }

    /// Set a cell hyperlink.
    pub fn set_cell_hyperlink(
        &self,
        sheet: &str,
        cell: &str,
        link: &str,
        link_type: &str,
        opts: &[HyperlinkOpts],
    ) -> Result<()> {
        split_cell_name(cell).map_err(|_| Box::new(ErrParameterInvalid))?;
        let path =
            self.get_sheet_xml_path(sheet)
                .ok_or_else(|| crate::errors::ErrSheetNotExist {
                    sheet_name: sheet.to_string(),
                })?;
        let mut ws = self.work_sheet_reader(sheet)?;
        let cell = merge_cells_parser(&ws, cell);

        if link_type == "None" {
            remove_hyperlink(self, &mut ws, sheet, &cell)?;
            update_dimension(&mut ws)?;
            self.sheet.insert(path, ws);
            return Ok(());
        }

        if ws.hyperlinks.is_none() {
            ws.hyperlinks = Some(XlsxHyperlinks::default());
        }
        let links = ws.hyperlinks.as_mut().unwrap();

        let existing_idx = links.hyperlink.iter().position(|h| h.r#ref == cell);
        let mut link_data = existing_idx
            .and_then(|i| links.hyperlink.get(i).cloned())
            .unwrap_or_default();
        link_data.r#ref = cell.clone();

        if links.hyperlink.len() as i32 > TOTAL_SHEET_HYPERLINKS {
            return Err(Box::new(ErrTotalSheetHyperlinks));
        }

        match link_type {
            "External" => {
                let sheet_path = self.get_sheet_xml_path(sheet).unwrap_or_default();
                let sheet_rels = format!(
                    "xl/worksheets/_rels/{}.rels",
                    sheet_path.trim_start_matches("xl/worksheets/")
                );
                let rid_num = self.set_rels(
                    link_data.rid.as_deref().unwrap_or(""),
                    &sheet_rels,
                    SOURCE_RELATIONSHIP_HYPER_LINK,
                    link,
                    "External",
                );
                link_data = XlsxHyperlink {
                    r#ref: cell,
                    rid: Some(format!("rId{rid_num}")),
                    ..Default::default()
                };
                self.add_sheet_name_space(sheet, SOURCE_RELATIONSHIP);
            }
            "Location" => {
                link_data.location = Some(link.to_string());
                link_data.rid = None;
            }
            _ => return Err(crate::errors::new_invalid_link_type_error(link_type).into()),
        }

        for opt in opts {
            if let Some(display) = &opt.display {
                link_data.display = Some(display.clone());
            }
            if let Some(tooltip) = &opt.tooltip {
                link_data.tooltip = Some(tooltip.clone());
            }
        }

        if let Some(idx) = existing_idx {
            links.hyperlink[idx] = link_data;
        } else {
            links.hyperlink.push(link_data);
        }
        update_dimension(&mut ws)?;
        self.sheet.insert(path, ws);
        Ok(())
    }

    /// Write a row of values starting at the given cell.
    pub fn set_sheet_row(&self, sheet: &str, cell: &str, values: &[CellValue]) -> Result<()> {
        let (col, row) =
            cell_name_to_coordinates(cell).map_err(|_| Box::new(ErrParameterInvalid))?;
        for (i, value) in values.iter().enumerate() {
            let cell_name = coordinates_to_cell_name(col + i as i32, row, false)
                .map_err(|_| Box::new(ErrParameterInvalid))?;
            self.set_cell_value(sheet, &cell_name, value.clone())?;
        }
        Ok(())
    }

    /// Write a column of values starting at the given cell.
    pub fn set_sheet_col(&self, sheet: &str, cell: &str, values: &[CellValue]) -> Result<()> {
        let (col, row) =
            cell_name_to_coordinates(cell).map_err(|_| Box::new(ErrParameterInvalid))?;
        for (i, value) in values.iter().enumerate() {
            let cell_name = coordinates_to_cell_name(col, row + i as i32, false)
                .map_err(|_| Box::new(ErrParameterInvalid))?;
            self.set_cell_value(sheet, &cell_name, value.clone())?;
        }
        Ok(())
    }
}

// ------------------------------------------------------------------
// Internal helpers
// ------------------------------------------------------------------

impl File {
    fn set_cell_value_internal(&self, sheet: &str, cell: &str, value: CellValue) -> Result<()> {
        let path =
            self.get_sheet_xml_path(sheet)
                .ok_or_else(|| crate::errors::ErrSheetNotExist {
                    sheet_name: sheet.to_string(),
                })?;
        let mut ws = self.work_sheet_reader(sheet)?;

        match value {
            CellValue::String(s) => self.set_string_cell_value(&mut ws, cell, &s),
            CellValue::Int(n) => {
                let c = get_or_make_cell(&mut ws, cell);
                c.t = None;
                c.v = Some(n.to_string());
                c.f = None;
                c.is = None;
            }
            CellValue::Float(n) => {
                let c = get_or_make_cell(&mut ws, cell);
                c.t = None;
                c.v = Some(n.to_string());
                c.f = None;
                c.is = None;
            }
            CellValue::Bool(b) => {
                let c = get_or_make_cell(&mut ws, cell);
                c.t = Some("b".to_string());
                c.v = Some(if b { "1".to_string() } else { "0".to_string() });
                c.f = None;
                c.is = None;
            }
            CellValue::Formula(f) => {
                if f.starts_with('=') {
                    return Err(Box::new(ErrInvalidFormula));
                }
                let c = get_or_make_cell(&mut ws, cell);
                c.t = None;
                c.v = None;
                c.f = Some(XlsxF {
                    content: f,
                    ..Default::default()
                });
                c.is = None;
            }
            CellValue::DateTime(dt) => {
                self.set_default_date_time_style(&mut ws, cell, dt.date(), true)?;
                let c = get_or_make_cell(&mut ws, cell);
                c.v = Some(date::datetime_to_excel_serial(dt, self.date_1904()?).to_string());
                c.t = None;
                c.f = None;
                c.is = None;
            }
            CellValue::Date(d) => {
                self.set_default_date_time_style(&mut ws, cell, d, false)?;
                let c = get_or_make_cell(&mut ws, cell);
                c.v = Some(date::date_to_excel_serial(d, self.date_1904()?).to_string());
                c.t = None;
                c.f = None;
                c.is = None;
            }
            CellValue::Time(t) => {
                self.set_default_time_style(&mut ws, cell)?;
                let c = get_or_make_cell(&mut ws, cell);
                c.v = Some(date::time_to_excel_serial(t).to_string());
                c.t = None;
                c.f = None;
                c.is = None;
            }
            CellValue::Duration(d) => {
                self.set_default_time_style(&mut ws, cell)?;
                let c = get_or_make_cell(&mut ws, cell);
                let days = d.as_secs_f64() / 86400.0;
                c.v = Some(days.to_string());
                c.t = None;
                c.f = None;
                c.is = None;
            }
            CellValue::RichText(runs) => {
                let c = get_or_make_cell(&mut ws, cell);
                c.t = Some("inlineStr".to_string());
                c.v = None;
                c.f = None;
                c.is = Some(runs_to_xlsx_si(&runs));
            }
        }

        update_dimension(&mut ws)?;
        self.sheet.insert(path, ws);
        Ok(())
    }

    /// Remove the formula from a cell while preserving its cached value, if any.
    ///
    /// Equivalent to Go `removeFormula`. Shared-formula masters are expanded so
    /// that every sibling formula cell is also cleared.
    pub(crate) fn remove_formula(
        &self,
        c: &mut XlsxC,
        ws: &mut XlsxWorksheet,
        sheet: &str,
    ) -> Result<()> {
        self.clear_calc_cache();
        if c.f.is_some() && c.vm.is_none() {
            let sheet_id = self.get_sheet_id(sheet);
            if let Some(ref cell_ref) = c.r {
                self.delete_calc_chain(sheet_id, cell_ref)?;
            }
            let f = c.f.as_ref().unwrap();
            if f.t.as_deref() == Some("shared")
                && f.r#ref.as_deref().map_or(false, |s| !s.is_empty())
            {
                if let Some(si_val) = f.si {
                    for row in &mut ws.sheet_data.row {
                        for cell in &mut row.c {
                            if cell.f.as_ref().and_then(|ff| ff.si) == Some(si_val) {
                                if let Some(ref r) = cell.r {
                                    let _ = self.delete_calc_chain(sheet_id, r);
                                }
                                cell.f = None;
                            }
                        }
                    }
                }
            }
            c.f = None;
        }
        Ok(())
    }

    fn set_string_cell_value(&self, ws: &mut XlsxWorksheet, cell: &str, value: &str) {
        if value.is_empty() {
            return;
        }
        if value.len() > crate::constants::TOTAL_CELL_CHARS {
            let _ = Err::<(), _>(Box::new(ErrCellCharsLength));
            return;
        }
        let idx = self.shared_string_index(value);
        let c = get_or_make_cell(ws, cell);
        c.t = Some("s".to_string());
        c.v = Some(idx.to_string());
        c.f = None;
        c.is = None;
    }

    fn set_default_time_style(&self, ws: &mut XlsxWorksheet, cell: &str) -> Result<()> {
        // Time-only values use the built-in time format 21 (hh:mm:ss).
        self.set_default_date_time_style(ws, cell, NaiveDate::default(), true)
    }

    fn set_default_date_time_style(
        &self,
        ws: &mut XlsxWorksheet,
        cell: &str,
        date: NaiveDate,
        has_time: bool,
    ) -> Result<()> {
        let c = get_or_make_cell(ws, cell);
        if c.s.is_some() {
            return Ok(());
        }
        let num_fmt_id = if has_time && date != NaiveDate::default() {
            22 // m/d/yy hh:mm
        } else if has_time {
            21 // hh:mm:ss
        } else {
            14 // mm-dd-yy
        };
        let style_id = self.find_or_create_style(num_fmt_id, None)?;
        c.s = Some(style_id);
        Ok(())
    }

    /// Find an existing cell style with `num_fmt_id`, or create one.
    fn find_or_create_style(&self, num_fmt_id: i32, format_code: Option<&str>) -> Result<i64> {
        let mut styles = self.styles_reader()?;
        let cell_xfs = styles.cell_xfs.get_or_insert_with(Default::default);

        for (idx, xf) in cell_xfs.xf.iter().enumerate() {
            if xf.num_fmt_id == Some(num_fmt_id as i64) {
                return Ok(idx as i64);
            }
        }

        let new_idx = cell_xfs.xf.len() as i64;
        cell_xfs.xf.push(XlsxXf {
            num_fmt_id: Some(num_fmt_id as i64),
            apply_number_format: Some(true),
            ..Default::default()
        });
        cell_xfs.count = cell_xfs.xf.len() as i64;

        // For custom formats (ID >= 164) register the format code in numFmts.
        if num_fmt_id >= 164 {
            if let Some(code) = format_code {
                let num_fmts = styles.num_fmts.get_or_insert_with(Default::default);
                let custom_id = num_fmt_id;
                if !num_fmts
                    .num_fmt
                    .iter()
                    .any(|n| n.num_fmt_id == custom_id as i64)
                {
                    num_fmts.num_fmt.push(XlsxNumFmt {
                        num_fmt_id: custom_id as i64,
                        format_code: code.to_string(),
                        format_code_16: None,
                    });
                    num_fmts.count = num_fmts.num_fmt.len() as i64;
                }
            }
        }

        *self.styles.lock().unwrap() = Some(styles);
        Ok(new_idx)
    }

    fn date_1904(&self) -> Result<bool> {
        let wb = self.workbook_reader()?;
        Ok(wb
            .workbook_pr
            .as_ref()
            .and_then(|p| p.date1904)
            .unwrap_or(false))
    }

    /// Return the number format ID applied to a cell style, if any.
    fn get_cell_num_fmt_id(&self, c: &XlsxC) -> Option<i32> {
        let style_id = c.s.unwrap_or(0) as usize;
        let styles = self.styles_reader().ok()?;
        let cell_xfs = styles.cell_xfs?;
        let xf = cell_xfs.xf.get(style_id)?;
        xf.num_fmt_id.map(|id| id as i32)
    }

    /// Return the explicit format code for a number format ID, if registered.
    fn get_num_fmt_code(&self, num_fmt_id: i32) -> Option<String> {
        let styles = self.styles_reader().ok()?;
        styles
            .num_fmts?
            .num_fmt
            .into_iter()
            .find(|n| n.num_fmt_id == num_fmt_id as i64)
            .map(|n| n.format_code)
    }

    /// Return the shared-string index for `text`, creating it if necessary.
    fn shared_string_index(&self, text: &str) -> i32 {
        {
            let map = self.shared_strings_map.lock().unwrap();
            if let Some(&idx) = map.get(text) {
                return idx;
            }
        }
        let mut sst = self.shared_strings_reader().unwrap_or_default();
        let mut map = self.shared_strings_map.lock().unwrap();
        if let Some(&idx) = map.get(text) {
            return idx;
        }
        let idx = map.len() as i32;
        map.insert(text.to_string(), idx);
        sst.si.push(XlsxSi {
            t: Some(XlsxT {
                space: None,
                val: text.to_string(),
            }),
            ..Default::default()
        });
        sst.unique_count += 1;
        sst.count += 1;
        *self.shared_strings.lock().unwrap() = Some(sst);
        idx
    }
}

// ------------------------------------------------------------------
// Worksheet cell helpers
// ------------------------------------------------------------------

pub(crate) fn find_cell<'a>(ws: &'a XlsxWorksheet, cell: &str) -> Option<&'a XlsxC> {
    for row in &ws.sheet_data.row {
        for c in &row.c {
            if c.r.as_deref() == Some(cell) {
                return Some(c);
            }
        }
    }
    None
}

pub(crate) fn find_cell_mut<'a>(ws: &'a mut XlsxWorksheet, cell: &str) -> Option<&'a mut XlsxC> {
    for row in &mut ws.sheet_data.row {
        for c in &mut row.c {
            if c.r.as_deref() == Some(cell) {
                return Some(c);
            }
        }
    }
    None
}

fn get_or_make_cell<'a>(ws: &'a mut XlsxWorksheet, cell: &str) -> &'a mut XlsxC {
    let (_col, row_num) = cell_name_to_coordinates(cell).unwrap_or((1, 1));
    let row_idx = ws
        .sheet_data
        .row
        .iter()
        .position(|r| r.r == Some(row_num as i64));
    if let Some(idx) = row_idx {
        if let Some(pos) = ws.sheet_data.row[idx]
            .c
            .iter()
            .position(|c| c.r.as_deref() == Some(cell))
        {
            return &mut ws.sheet_data.row[idx].c[pos];
        }
        let c = XlsxC {
            r: Some(cell.to_string()),
            ..Default::default()
        };
        ws.sheet_data.row[idx].c.push(c);
        let last = ws.sheet_data.row[idx].c.len() - 1;
        return &mut ws.sheet_data.row[idx].c[last];
    }
    let mut row = crate::xml::worksheet::XlsxRow {
        r: Some(row_num as i64),
        ..Default::default()
    };
    row.c.push(XlsxC {
        r: Some(cell.to_string()),
        ..Default::default()
    });
    ws.sheet_data.row.push(row);
    let last_row = ws.sheet_data.row.len() - 1;
    let last_cell = ws.sheet_data.row[last_row].c.len() - 1;
    &mut ws.sheet_data.row[last_row].c[last_cell]
}

fn update_dimension(ws: &mut XlsxWorksheet) -> Result<()> {
    if ws.sheet_data.row.is_empty() {
        ws.dimension = Some(crate::xml::worksheet::XlsxDimension {
            r#ref: "A1".to_string(),
        });
        return Ok(());
    }
    let mut min_col = i32::MAX;
    let mut min_row = i32::MAX;
    let mut max_col = 1;
    let mut max_row = 1;
    for row in &ws.sheet_data.row {
        let row_num = row.r.unwrap_or(1) as i32;
        min_row = min_row.min(row_num);
        max_row = max_row.max(row_num);
        for c in &row.c {
            if let Some(name) = &c.r {
                if let Ok((col, _)) = cell_name_to_coordinates(name) {
                    min_col = min_col.min(col);
                    max_col = max_col.max(col);
                }
            }
        }
    }
    if min_col == i32::MAX {
        min_col = 1;
    }
    let first = coordinates_to_cell_name(min_col, min_row, false)?;
    let last = coordinates_to_cell_name(max_col, max_row, false)?;
    ws.dimension = Some(crate::xml::worksheet::XlsxDimension {
        r#ref: format!("{first}:{last}"),
    });
    Ok(())
}

fn set_cell_default_value(c: &mut XlsxC, value: &str) {
    c.f = None;
    c.is = None;
    if value.parse::<f64>().is_ok() {
        c.t = None;
        c.v = Some(value.to_string());
        return;
    }
    if !value.is_empty() {
        c.t = Some("inlineStr".to_string());
        c.v = None;
        c.is = Some(XlsxSi {
            t: Some(XlsxT {
                space: None,
                val: value.to_string(),
            }),
            ..Default::default()
        });
        return;
    }
    c.t = Some(value.to_string());
    c.v = Some(value.to_string());
}

fn prepare_cell_style(ws: &XlsxWorksheet, col: i64, row: i64, style: i64) -> i64 {
    if style != 0 {
        return style;
    }
    if row > 0 && row as usize <= ws.sheet_data.row.len() {
        if let Some(style_id) = ws.sheet_data.row[row as usize - 1].s {
            if style_id != 0 {
                return style_id;
            }
        }
    }
    if let Some(cols) = &ws.cols {
        for c in &cols.col {
            if c.min <= col && col <= c.max {
                if let Some(style_id) = c.style {
                    if style_id != 0 {
                        return style_id;
                    }
                }
            }
        }
    }
    style
}

// ------------------------------------------------------------------
// Merge cell helpers
// ------------------------------------------------------------------

fn merge_cells_parser(ws: &XlsxWorksheet, cell: &str) -> String {
    let upper = cell.to_uppercase();
    let Ok((col, row)) = cell_name_to_coordinates(&upper) else {
        return upper;
    };
    if let Some(merges) = &ws.merge_cells {
        for mc in &merges.cells {
            let mut r#ref = mc.r#ref.clone().unwrap_or_default();
            if r#ref.is_empty() {
                continue;
            }
            if !r#ref.contains(':') {
                r#ref = format!("{ref}:{ref}", ref = r#ref);
            }
            let Ok(mut coords) = range_ref_to_coordinates(&r#ref) else {
                continue;
            };
            let _ = sort_coordinates(&mut coords);
            if cell_in_range(&[col, row], &coords) {
                return r#ref.split(':').next().unwrap_or(&upper).to_string();
            }
        }
    }
    upper
}

fn check_cell_in_range_ref(cell: &str, range_ref: &str) -> Result<bool> {
    let (col, row) = cell_name_to_coordinates(cell).map_err(|_| Box::new(ErrParameterInvalid))?;
    if !range_ref.contains(':') {
        return Ok(false);
    }
    let mut coords =
        range_ref_to_coordinates(range_ref).map_err(|_| Box::new(ErrParameterInvalid))?;
    sort_coordinates(&mut coords).map_err(|_| Box::new(ErrCoordinates))?;
    Ok(cell_in_range(&[col, row], &coords))
}

fn cell_in_range(cell: &[i32], rect: &[i32]) -> bool {
    cell.len() >= 2
        && rect.len() >= 4
        && cell[0] >= rect[0]
        && cell[0] <= rect[2]
        && cell[1] >= rect[1]
        && cell[1] <= rect[3]
}

// ------------------------------------------------------------------
// Shared formula helpers
// ------------------------------------------------------------------

fn find_shared_formula_master<'a>(ws: &'a XlsxWorksheet, si: i64) -> Option<&'a XlsxF> {
    for row in &ws.sheet_data.row {
        for c in &row.c {
            if let Some(f) = &c.f {
                if f.t.as_deref() == Some("shared")
                    && f.si == Some(si)
                    && !f.r#ref.as_deref().unwrap_or("").is_empty()
                {
                    return Some(f);
                }
            }
        }
    }
    None
}

// ------------------------------------------------------------------
// Hyperlink helpers
// ------------------------------------------------------------------

fn remove_hyperlink(file: &File, ws: &mut XlsxWorksheet, sheet: &str, cell: &str) -> Result<()> {
    if ws.hyperlinks.is_none() {
        return Ok(());
    }
    let links = ws.hyperlinks.as_mut().unwrap();
    let mut i = 0;
    while i < links.hyperlink.len() {
        let link = &links.hyperlink[i];
        let remove =
            link.r#ref == cell || check_cell_in_range_ref(cell, &link.r#ref).unwrap_or(false);
        if remove {
            if let Some(rid) = &link.rid {
                file.delete_sheet_relationships(sheet, rid);
            }
            links.hyperlink.remove(i);
        } else {
            i += 1;
        }
    }
    if links.hyperlink.is_empty() {
        ws.hyperlinks = None;
    }
    Ok(())
}

// ------------------------------------------------------------------
// Value reading helpers
// ------------------------------------------------------------------

pub(crate) fn read_cell_value(file: &File, c: &XlsxC, raw: bool) -> String {
    let raw_value = match c.t.as_deref() {
        Some("s") => {
            if let Some(v) = &c.v {
                if let Ok(idx) = v.parse::<i32>() {
                    return read_shared_string(file, idx);
                }
            }
            return String::new();
        }
        Some("inlineStr") => {
            return c
                .is
                .as_ref()
                .map(|is| inline_string_text(is))
                .unwrap_or_default();
        }
        Some("b") => {
            return c
                .v
                .as_deref()
                .map(|v| {
                    if v == "1" {
                        "TRUE".to_string()
                    } else {
                        "FALSE".to_string()
                    }
                })
                .unwrap_or_default();
        }
        Some("str") => return c.v.clone().unwrap_or_default(),
        _ => c.v.clone().unwrap_or_default(),
    };

    // For formula cells without cached value, return the formula.
    if c.f.is_some() && c.v.is_none() {
        return format!("={}", c.f.as_ref().unwrap().content);
    }

    // Apply number formatting unless the caller requested raw values.
    if raw {
        return raw_value;
    }

    if let Some(num_fmt_id) = file.get_cell_num_fmt_id(c) {
        if let Ok(value) = raw_value.parse::<f64>() {
            let format_code = file.get_num_fmt_code(num_fmt_id);
            let date1904 = file.date_1904().unwrap_or(false);
            return numfmt::apply_number_format(
                value,
                num_fmt_id,
                format_code.as_deref(),
                date1904,
            );
        }
    }

    raw_value
}

fn read_shared_string(file: &File, idx: i32) -> String {
    let sst = file.shared_strings_reader().unwrap_or_default();
    sst.si
        .get(idx as usize)
        .map(|si| {
            if let Some(t) = &si.t {
                t.val.clone()
            } else {
                si.r.iter()
                    .filter_map(|r| r.t.as_ref().map(|t| t.val.clone()))
                    .collect()
            }
        })
        .unwrap_or_default()
}

fn inline_string_text(si: &XlsxSi) -> String {
    if let Some(t) = &si.t {
        return t.val.clone();
    }
    si.r.iter()
        .filter_map(|r| r.t.as_ref().map(|t| t.val.clone()))
        .collect()
}

// ------------------------------------------------------------------
// Rich text conversion
// ------------------------------------------------------------------

pub(crate) fn runs_to_xlsx_si(runs: &[RichTextRun]) -> XlsxSi {
    let mut r = Vec::new();
    for run in runs {
        let mut xrun = XlsxR::default();
        if !run.text.is_empty() {
            let (text, space) = trim_cell_value(&run.text);
            xrun.t = Some(XlsxT { space, val: text });
        }
        if let Some(font) = &run.font {
            let mut rpr = crate::xml::common::XlsxRPr::default();
            if let Some(name) = &font.name {
                rpr.r_font = Some(crate::xml::common::AttrValString {
                    val: Some(name.clone()),
                });
            }
            if let Some(size) = font.size {
                rpr.sz = Some(crate::xml::common::AttrValFloat { val: Some(size) });
            }
            if let Some(family) = font.family {
                rpr.family = Some(crate::xml::common::AttrValInt { val: Some(family) });
            }
            if let Some(charset) = font.charset {
                rpr.charset = Some(crate::xml::common::AttrValInt { val: Some(charset) });
            }
            if let Some(bold) = font.bold {
                rpr.b = Some(crate::xml::common::AttrValBool { val: Some(bold) });
            }
            if let Some(italic) = font.italic {
                rpr.i = Some(crate::xml::common::AttrValBool { val: Some(italic) });
            }
            if let Some(strike) = font.strike {
                rpr.strike = Some(crate::xml::common::AttrValBool { val: Some(strike) });
            }
            if let Some(underline) = &font.underline {
                rpr.u = Some(crate::xml::common::AttrValString {
                    val: Some(underline.clone()),
                });
            }
            if let Some(color) = &font.color {
                rpr.color = Some(crate::xml::common::XlsxColor {
                    rgb: Some(color.clone()),
                    ..Default::default()
                });
            }
            if let Some(vert_align) = &font.vert_align {
                rpr.vert_align = Some(crate::xml::common::AttrValString {
                    val: Some(vert_align.clone()),
                });
            }
            xrun.r_pr = Some(rpr);
        }
        r.push(xrun);
    }
    XlsxSi {
        t: None,
        r,
        ..Default::default()
    }
}

fn runs_from_xlsx_si(si: &XlsxSi) -> Vec<RichTextRun> {
    let mut runs = Vec::new();
    if let Some(t) = &si.t {
        runs.push(RichTextRun {
            text: t.val.clone(),
            ..Default::default()
        });
    }
    for xrun in &si.r {
        let mut run = RichTextRun::default();
        if let Some(t) = &xrun.t {
            run.text = t.val.clone();
        }
        if let Some(rpr) = &xrun.r_pr {
            let mut font = crate::styles::Font::default();
            font.name = rpr.r_font.as_ref().and_then(|f| f.val.clone());
            font.size = rpr.sz.as_ref().and_then(|s| s.val);
            font.family = rpr.family.as_ref().and_then(|f| f.val);
            font.charset = rpr.charset.as_ref().and_then(|f| f.val);
            font.bold = rpr.b.as_ref().and_then(|b| b.val);
            font.italic = rpr.i.as_ref().and_then(|i| i.val);
            font.strike = rpr.strike.as_ref().and_then(|s| s.val);
            font.underline = rpr.u.as_ref().and_then(|u| u.val.clone());
            font.color = rpr.color.as_ref().and_then(|c| c.rgb.clone());
            font.vert_align = rpr.vert_align.as_ref().and_then(|v| v.val.clone());
            run.font = Some(font);
        }
        runs.push(run);
    }
    runs
}

fn trim_cell_value(value: &str) -> (String, Option<String>) {
    let mut space = None;
    if !value.is_empty() {
        let prefix = value.as_bytes()[0];
        let suffix = value.as_bytes()[value.len() - 1];
        for &ascii in &[9u8, 10, 13, 32] {
            if prefix == ascii || suffix == ascii {
                space = Some("preserve".to_string());
                break;
            }
        }
    }
    (value.to_string(), space)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{NaiveDate, NaiveTime};

    #[test]
    fn set_and_get_string() {
        let f = File::new_with_options(crate::options::Options::default());
        f.set_cell_str("Sheet1", "A1", "hello").unwrap();
        assert_eq!(f.get_cell_value("Sheet1", "A1").unwrap(), "hello");
    }

    #[test]
    fn set_cell_style_range() {
        let f = File::new_with_options(crate::options::Options::default());
        let style = f
            .new_style(&crate::styles::Style {
                fill: crate::styles::Fill {
                    r#type: "pattern".to_string(),
                    pattern: 1,
                    color: vec!["63BE7B".to_string()],
                    ..Default::default()
                },
                ..Default::default()
            })
            .unwrap();

        f.set_cell_style("Sheet1", "B2", "C3", style).unwrap();
        assert_eq!(f.get_cell_style("Sheet1", "B2").unwrap(), style);
        assert_eq!(f.get_cell_style("Sheet1", "C3").unwrap(), style);
        assert_eq!(f.get_cell_style("Sheet1", "A1").unwrap(), 0);
        assert_eq!(f.get_cell_style("Sheet1", "D4").unwrap(), 0);

        // Reversed corners are normalized.
        f.set_cell_style("Sheet1", "E5", "D4", style).unwrap();
        assert_eq!(f.get_cell_style("Sheet1", "D4").unwrap(), style);
        assert_eq!(f.get_cell_style("Sheet1", "E5").unwrap(), style);

        // Single cell still works.
        f.set_cell_style("Sheet1", "A1", "A1", style).unwrap();
        assert_eq!(f.get_cell_style("Sheet1", "A1").unwrap(), style);
    }

    #[test]
    fn set_and_get_int() {
        let f = File::new_with_options(crate::options::Options::default());
        f.set_cell_int("Sheet1", "B2", 42).unwrap();
        assert_eq!(f.get_cell_value("Sheet1", "B2").unwrap(), "42");
    }

    #[test]
    fn set_and_get_bool() {
        let f = File::new_with_options(crate::options::Options::default());
        f.set_cell_bool("Sheet1", "C3", true).unwrap();
        assert_eq!(f.get_cell_value("Sheet1", "C3").unwrap(), "TRUE");
    }

    #[test]
    fn set_and_get_formula() {
        let f = File::new_with_options(crate::options::Options::default());
        f.set_cell_formula("Sheet1", "D4", "SUM(A1:A3)").unwrap();
        assert_eq!(f.get_cell_value("Sheet1", "D4").unwrap(), "=SUM(A1:A3)");
        assert_eq!(f.get_cell_formula("Sheet1", "D4").unwrap(), "SUM(A1:A3)");
    }

    #[test]
    fn set_and_get_date() {
        let f = File::new_with_options(crate::options::Options::default());
        let d = NaiveDate::from_ymd_opt(2024, 7, 13).unwrap();
        f.set_cell_value("Sheet1", "E5", CellValue::Date(d))
            .unwrap();
        assert_eq!(f.get_cell_value("Sheet1", "E5").unwrap(), "07-13-24");
    }

    #[test]
    fn set_and_get_datetime() {
        let f = File::new_with_options(crate::options::Options::default());
        let dt = NaiveDate::from_ymd_opt(2024, 7, 13)
            .unwrap()
            .and_hms_opt(12, 30, 0)
            .unwrap();
        f.set_cell_value("Sheet1", "F6", CellValue::DateTime(dt))
            .unwrap();
        assert_eq!(f.get_cell_value("Sheet1", "F6").unwrap(), "7/13/24 12:30");
    }

    #[test]
    fn set_and_get_time() {
        let f = File::new_with_options(crate::options::Options::default());
        let t = NaiveTime::from_hms_opt(14, 30, 0).unwrap();
        f.set_cell_value("Sheet1", "G7", CellValue::Time(t))
            .unwrap();
        assert_eq!(f.get_cell_value("Sheet1", "G7").unwrap(), "14:30:00");
    }

    #[test]
    fn raw_cell_value_respects_option() {
        let mut opts = crate::options::Options::default();
        opts.raw_cell_value = true;
        let f = File::new_with_options(opts);
        let d = NaiveDate::from_ymd_opt(2024, 7, 13).unwrap();
        f.set_cell_value("Sheet1", "H8", CellValue::Date(d))
            .unwrap();
        assert_eq!(f.get_cell_value("Sheet1", "H8").unwrap(), "45486");
    }

    #[test]
    fn set_and_get_default() {
        let f = File::new_with_options(crate::options::Options::default());
        f.set_cell_default("Sheet1", "A1", "123").unwrap();
        assert_eq!(f.get_cell_value("Sheet1", "A1").unwrap(), "123");
        f.set_cell_default("Sheet1", "A2", "text").unwrap();
        assert_eq!(f.get_cell_value("Sheet1", "A2").unwrap(), "text");
    }

    #[test]
    fn set_and_get_uint() {
        let f = File::new_with_options(crate::options::Options::default());
        f.set_cell_uint("Sheet1", "A1", 42).unwrap();
        assert_eq!(f.get_cell_value("Sheet1", "A1").unwrap(), "42");
    }

    #[test]
    fn set_and_get_float_precision() {
        let f = File::new_with_options(crate::options::Options::default());
        f.set_cell_float_with_precision("Sheet1", "A1", 1.325, 2, 32)
            .unwrap();
        assert_eq!(f.get_cell_value("Sheet1", "A1").unwrap(), "1.33");
    }

    #[test]
    fn set_and_get_sheet_row_and_col() {
        let f = File::new_with_options(crate::options::Options::default());
        f.set_sheet_row("Sheet1", "A1", &["a".into(), "b".into(), 1.into()])
            .unwrap();
        f.set_sheet_col("Sheet1", "B5", &["x".into(), "y".into()])
            .unwrap();
        assert_eq!(f.get_cell_value("Sheet1", "A1").unwrap(), "a");
        assert_eq!(f.get_cell_value("Sheet1", "C1").unwrap(), "1");
        assert_eq!(f.get_cell_value("Sheet1", "B6").unwrap(), "y");
    }

    #[test]
    fn hyperlink_round_trip() {
        let f = File::new_with_options(crate::options::Options::default());
        f.set_cell_hyperlink("Sheet1", "A1", "https://example.com", "External", &[])
            .unwrap();
        let (has, link) = f.get_cell_hyperlink("Sheet1", "A1").unwrap();
        assert!(has);
        assert_eq!(link, "https://example.com");

        let cells = f.get_hyperlink_cells("Sheet1", "").unwrap();
        assert_eq!(cells, vec!["A1"]);

        f.set_cell_hyperlink("Sheet1", "A1", "", "None", &[])
            .unwrap();
        let (has, _) = f.get_cell_hyperlink("Sheet1", "A1").unwrap();
        assert!(!has);
    }

    #[test]
    fn rich_text_round_trip_shared_string() {
        let f = File::new_with_options(crate::options::Options::default());
        let runs = vec![
            RichTextRun {
                text: "bold".to_string(),
                font: Some(crate::styles::Font {
                    bold: Some(true),
                    ..Default::default()
                }),
            },
            RichTextRun {
                text: " plain".to_string(),
                ..Default::default()
            },
        ];
        f.set_cell_rich_text("Sheet1", "A1", runs.clone()).unwrap();
        let got = f.get_cell_rich_text("Sheet1", "A1").unwrap();
        assert_eq!(got, runs);
    }
}
