//! Row-level API.
//!
//! This module corresponds to `rows.go` in the Go implementation.

use std::collections::BTreeMap;

use crate::adjust::AdjustDirection;
use crate::constants::{DEFAULT_ROW_HEIGHT, MAX_CELL_STYLES, MAX_ROW_HEIGHT, TOTAL_ROWS};
use crate::errors::Result;
use crate::errors::{ErrMaxRowHeight, ErrMaxRows, ErrSheetNotExist};
use crate::file::File;
use crate::lib_util::{
    cell_name_to_coordinates, coordinates_to_cell_name, coordinates_to_range_ref,
    range_ref_to_coordinates, sort_coordinates,
};
use crate::options::Options;
use crate::xml::worksheet::{XlsxConditionalFormatting, XlsxDataValidation, XlsxWorksheet};

impl File {
    /// Set the height of a row.
    pub fn set_row_height(&self, sheet: &str, row: i32, height: f64) -> Result<()> {
        if height > MAX_ROW_HEIGHT as f64 {
            return Err(Box::new(ErrMaxRowHeight));
        }
        let path = self
            .get_sheet_xml_path(sheet)
            .ok_or_else(|| ErrSheetNotExist {
                sheet_name: sheet.to_string(),
            })?;
        let mut ws = self.work_sheet_reader(sheet)?;
        let r = get_or_make_row(&mut ws, row);
        r.ht = Some(height);
        r.custom_height = Some(true);
        self.sheet.insert(path, ws);
        Ok(())
    }

    /// Get the height of a row.
    pub fn get_row_height(&self, sheet: &str, row: i32) -> Result<f64> {
        let ws = self.work_sheet_reader(sheet)?;
        for r in &ws.sheet_data.row {
            if r.r == Some(row as i64) {
                return Ok(r.ht.unwrap_or(DEFAULT_ROW_HEIGHT));
            }
        }
        if let Some(fmt) = &ws.sheet_format_pr {
            return Ok(fmt.default_row_height);
        }
        Ok(DEFAULT_ROW_HEIGHT)
    }

    /// Set the visibility of a row.
    pub fn set_row_visible(&self, sheet: &str, row: i32, visible: bool) -> Result<()> {
        let path = self
            .get_sheet_xml_path(sheet)
            .ok_or_else(|| ErrSheetNotExist {
                sheet_name: sheet.to_string(),
            })?;
        let mut ws = self.work_sheet_reader(sheet)?;
        let r = get_or_make_row(&mut ws, row);
        r.hidden = Some(!visible);
        self.sheet.insert(path, ws);
        Ok(())
    }

    /// Get the visibility of a row.
    pub fn get_row_visible(&self, sheet: &str, row: i32) -> Result<bool> {
        let ws = self.work_sheet_reader(sheet)?;
        for r in &ws.sheet_data.row {
            if r.r == Some(row as i64) {
                return Ok(!r.hidden.unwrap_or(false));
            }
        }
        Ok(true)
    }

    /// Set the outline level of a row.
    pub fn set_row_outline_level(&self, sheet: &str, row: i32, level: u8) -> Result<()> {
        if level == 0 || level > 7 {
            return Err(Box::new(crate::errors::ErrOutlineLevel));
        }
        let path = self
            .get_sheet_xml_path(sheet)
            .ok_or_else(|| ErrSheetNotExist {
                sheet_name: sheet.to_string(),
            })?;
        let mut ws = self.work_sheet_reader(sheet)?;
        let r = get_or_make_row(&mut ws, row);
        r.outline_level = Some(level);
        self.sheet.insert(path, ws);
        Ok(())
    }

    /// Get the outline level of a row.
    pub fn get_row_outline_level(&self, sheet: &str, row: i32) -> Result<u8> {
        let ws = self.work_sheet_reader(sheet)?;
        for r in &ws.sheet_data.row {
            if r.r == Some(row as i64) {
                return Ok(r.outline_level.unwrap_or(0));
            }
        }
        Ok(0)
    }

    /// Insert `n` rows before `row`.
    pub fn insert_rows(&self, sheet: &str, row: i32, n: usize) -> Result<()> {
        if n == 0 {
            return Ok(());
        }
        let path = self
            .get_sheet_xml_path(sheet)
            .ok_or_else(|| ErrSheetNotExist {
                sheet_name: sheet.to_string(),
            })?;
        let mut ws = self.work_sheet_reader(sheet)?;
        shift_rows_down(&mut ws, row as i64, n as i64);
        self.sheet.insert(path, ws);
        Ok(())
    }

    /// Remove a row.
    pub fn remove_row(&self, sheet: &str, row: i32) -> Result<()> {
        if row < 1 {
            return Ok(());
        }
        let path = self
            .get_sheet_xml_path(sheet)
            .ok_or_else(|| ErrSheetNotExist {
                sheet_name: sheet.to_string(),
            })?;
        let mut ws = self.work_sheet_reader(sheet)?;

        // Remove formulas from every cell in the deleted row so that the calc
        // chain and shared-formula siblings stay consistent.
        let indices: Vec<(usize, usize)> = ws
            .sheet_data
            .row
            .iter()
            .enumerate()
            .filter(|(_, r)| r.r == Some(row as i64))
            .flat_map(|(ri, r)| r.c.iter().enumerate().map(move |(ci, _)| (ri, ci)))
            .collect();
        for (ri, ci) in indices {
            let mut cell = std::mem::take(&mut ws.sheet_data.row[ri].c[ci]);
            self.remove_formula(&mut cell, &mut ws, sheet)?;
            ws.sheet_data.row[ri].c[ci] = cell;
        }

        ws.sheet_data.row.retain(|r| r.r != Some(row as i64));
        self.sheet.insert(path, ws);
        self.adjust_helper(sheet, AdjustDirection::Rows, row, -1)?;
        Ok(())
    }

    /// Duplicate a row.
    pub fn duplicate_row(&self, sheet: &str, row: i32) -> Result<()> {
        self.duplicate_row_to(sheet, row, row + 1)
    }

    /// Duplicate a row to a target position, shifting existing rows down.
    ///
    /// Equivalent to Go `DuplicateRowTo`.
    pub fn duplicate_row_to(&self, sheet: &str, row: i32, row2: i32) -> Result<()> {
        if row < 1 || row2 < 1 || row == row2 {
            return Ok(());
        }
        let path = self
            .get_sheet_xml_path(sheet)
            .ok_or_else(|| ErrSheetNotExist {
                sheet_name: sheet.to_string(),
            })?;
        let ws = self.work_sheet_reader(sheet)?;
        let source = ws
            .sheet_data
            .row
            .iter()
            .find(|r| r.r == Some(row as i64))
            .cloned();

        // Shift existing rows and adjust references (formulas, merged cells,
        // conditional formatting, data validations, calc chain, ...).
        self.sheet.insert(path.clone(), ws);
        self.adjust_helper(sheet, AdjustDirection::Rows, row2, 1)?;

        let mut ws = self.work_sheet_reader(sheet)?;
        if let Some(mut source) = source {
            source.r = Some(row2 as i64);
            let offset = row2 - row;
            for c in &mut source.c {
                if let Some(name) = &c.r {
                    if let Ok((col, old_row)) = cell_name_to_coordinates(name) {
                        if let Ok(new_name) = coordinates_to_cell_name(col, old_row + offset, false)
                        {
                            c.r = Some(new_name);
                        }
                    }
                }
            }
            self.adjust_single_row_formulas(sheet, sheet, &mut source, row, offset, true)?;

            if let Some(idx) = ws
                .sheet_data
                .row
                .iter()
                .position(|r| r.r == Some(row2 as i64))
            {
                ws.sheet_data.row[idx] = source;
            } else {
                ws.sheet_data.row.push(source);
            }
            ws.sheet_data.row.sort_by_key(|r| r.r.unwrap_or(0));

            // Copy worksheet decorations that belong to the source row.
            self.duplicate_row_conditional_formats(&mut ws, row, row2)?;
            self.duplicate_row_data_validations(&mut ws, row, row2)?;
            self.duplicate_row_merge_cells(&mut ws, row, row2)?;
        }
        self.sheet.insert(path, ws);
        Ok(())
    }

    /// Copy conditional formatting rules that apply only to `row` so that they
    /// also apply to `row2`.
    fn duplicate_row_conditional_formats(
        &self,
        ws: &mut XlsxWorksheet,
        row: i32,
        row2: i32,
    ) -> Result<()> {
        let mut new_cfs: Vec<XlsxConditionalFormatting> = Vec::new();
        for cf in &ws.conditional_formatting {
            let mut sqrefs = Vec::new();
            if let Some(sqref) = &cf.sqref {
                for r#ref in sqref.split_whitespace() {
                    if let Some(new_ref) = duplicate_sq_ref_helper(row, row2, r#ref)? {
                        sqrefs.push(new_ref);
                    }
                }
            }
            if !sqrefs.is_empty() {
                let mut cf_copy = cf.clone();
                cf_copy.sqref = Some(sqrefs.join(" "));
                new_cfs.push(cf_copy);
            }
        }
        ws.conditional_formatting.extend(new_cfs);
        Ok(())
    }

    /// Copy data validation rules that apply only to `row` so that they also
    /// apply to `row2`.
    fn duplicate_row_data_validations(
        &self,
        ws: &mut XlsxWorksheet,
        row: i32,
        row2: i32,
    ) -> Result<()> {
        let dvs = match ws.data_validations.as_mut() {
            Some(dvs) => dvs,
            None => return Ok(()),
        };
        let mut new_dvs: Vec<XlsxDataValidation> = Vec::new();
        for dv in &dvs.data_validation {
            let mut sqrefs = Vec::new();
            for r#ref in dv.sqref.split_whitespace() {
                if let Some(new_ref) = duplicate_sq_ref_helper(row, row2, r#ref)? {
                    sqrefs.push(new_ref);
                }
            }
            if !sqrefs.is_empty() {
                let mut dv_copy = dv.clone();
                dv_copy.sqref = sqrefs.join(" ");
                new_dvs.push(dv_copy);
            }
        }
        dvs.data_validation.extend(new_dvs);
        dvs.count = Some(dvs.data_validation.len() as i64);
        Ok(())
    }

    /// Copy single-row merged cells from `row` to `row2`.
    fn duplicate_row_merge_cells(&self, ws: &mut XlsxWorksheet, row: i32, row2: i32) -> Result<()> {
        if ws.merge_cells.is_none() {
            return Ok(());
        }
        let mut source_row = row;
        if source_row > row2 {
            source_row += 1;
        }

        // If the target row sits inside an existing multi-row merge, abort.
        if let Some(merges) = ws.merge_cells.as_ref() {
            for m in &merges.cells {
                if let Some(ref_) = &m.r#ref {
                    if let Ok(coords) = range_ref_to_coordinates(ref_) {
                        if coords[1] < row2 && row2 < coords[3] {
                            return Ok(());
                        }
                    }
                }
            }
        }

        let refs: Vec<Option<String>> = ws
            .merge_cells
            .as_ref()
            .unwrap()
            .cells
            .iter()
            .map(|m| m.r#ref.clone())
            .collect();
        for ref_ in refs.into_iter().flatten() {
            let mut coords = range_ref_to_coordinates(&ref_)?;
            sort_coordinates(&mut coords)?;
            let (x1, y1, x2, y2) = (coords[0], coords[1], coords[2], coords[3]);
            if y1 == y2 && y1 == source_row {
                let from = coordinates_to_cell_name(x1, row2, false)?;
                let to = coordinates_to_cell_name(x2, row2, false)?;
                crate::merge::add_merge(ws, &from, &to)?;
            }
        }
        Ok(())
    }

    /// Apply a style index to every cell in a range of rows.
    ///
    /// This overwrites existing row and cell styles in the range; it does not
    /// merge with existing styles. The `start` and `end` row numbers are
    /// inclusive and will be swapped if `end` is less than `start`.
    pub fn set_row_style(
        &self,
        sheet: &str,
        mut start: i32,
        mut end: i32,
        style_id: i32,
    ) -> Result<()> {
        if end < start {
            std::mem::swap(&mut start, &mut end);
        }
        if start < 1 {
            return Err(crate::errors::new_invalid_row_number_error(start).into());
        }
        if end > crate::constants::TOTAL_ROWS {
            return Err(Box::new(crate::errors::ErrMaxRows));
        }
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
        let path = self
            .get_sheet_xml_path(sheet)
            .ok_or_else(|| ErrSheetNotExist {
                sheet_name: sheet.to_string(),
            })?;
        let mut ws = self.work_sheet_reader(sheet)?;
        for row in start..=end {
            let _ = get_or_make_row(&mut ws, row);
        }
        for r in &mut ws.sheet_data.row {
            if let Some(row_num) = r.r {
                if row_num >= start as i64 && row_num <= end as i64 {
                    r.s = Some(style_id as i64);
                    r.custom_format = Some(true);
                    for c in &mut r.c {
                        if let Some(ref cell_name) = c.r {
                            if let Ok((_, cell_row)) =
                                crate::lib_util::cell_name_to_coordinates(cell_name)
                            {
                                if cell_row == row_num as i32 {
                                    c.s = Some(style_id as i64);
                                }
                            }
                        } else {
                            c.s = Some(style_id as i64);
                        }
                    }
                }
            }
        }
        self.sheet.insert(path, ws);
        Ok(())
    }
}

// ------------------------------------------------------------------
// Row iterator
// ------------------------------------------------------------------

/// Row formatting attributes returned by `Rows::get_row_opts`.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct RowOpts {
    /// Row height in points.
    pub height: f64,
    /// Whether the row is hidden.
    pub hidden: bool,
    /// Default style index applied to the row.
    pub style_id: i32,
}

/// Streaming row iterator.
#[derive(Debug)]
pub struct Rows<'a> {
    file: &'a File,
    ws: XlsxWorksheet,
    cur_row: i32,
    seek_row: i32,
    max_row: i32,
    err: Option<String>,
    row_opts: RowOpts,
}

impl<'a> Rows<'a> {
    /// Advance to the next row. Returns `false` when there are no more rows.
    pub fn next(&mut self) -> bool {
        self.seek_row += 1;
        if self.cur_row >= self.seek_row {
            return self.cur_row > 0 && self.cur_row <= self.max_row;
        }
        if self.seek_row > self.max_row {
            return false;
        }
        self.cur_row = self.seek_row;
        self.row_opts = self.extract_row_opts();
        true
    }

    /// Return the formatting options of the current row.
    pub fn get_row_opts(&self) -> RowOpts {
        self.row_opts.clone()
    }

    /// Return the first error that occurred during iteration, if any.
    pub fn error(&self) -> Option<&str> {
        self.err.as_deref()
    }

    /// Close the iterator. This is a no-op for the in-memory iterator, but it
    /// matches the Go API.
    pub fn close(&self) -> Result<()> {
        if let Some(e) = &self.err {
            return Err(e.clone().into());
        }
        Ok(())
    }

    /// Return the values of the current row's cells as strings.
    pub fn columns(&self, opts: Options) -> Result<Vec<String>> {
        if self.cur_row < 1 || self.cur_row > self.max_row {
            return Ok(Vec::new());
        }
        let row = &self.ws.sheet_data.row[(self.cur_row - 1) as usize];
        let raw = opts.raw_cell_value;
        let mut cells: Vec<String> = Vec::new();
        let mut cell_col: i32 = 0;
        for c in &row.c {
            cell_col += 1;
            if let Some(name) = &c.r {
                cell_col = cell_name_to_coordinates(name)?.0;
            }
            let val = cell_value_string(self.file, c, raw);
            if !val.is_empty() || c.f.is_some() {
                let blank = cell_col - cells.len() as i32;
                for _ in 1..blank {
                    cells.push(String::new());
                }
                cells.push(val);
            }
        }
        Ok(cells)
    }

    fn extract_row_opts(&self) -> RowOpts {
        if let Some(row) = self.ws.sheet_data.row.get((self.cur_row - 1) as usize) {
            RowOpts {
                height: row.ht.unwrap_or(DEFAULT_ROW_HEIGHT),
                hidden: row.hidden.unwrap_or(false),
                style_id: row
                    .s
                    .filter(|&s| s > 0 && s < MAX_CELL_STYLES as i64)
                    .unwrap_or(0) as i32,
            }
        } else {
            RowOpts::default()
        }
    }
}

impl File {
    /// Return a streaming iterator over the rows of a worksheet.
    pub fn rows(&self, sheet: &str) -> Result<Rows<'_>> {
        let mut ws = self.work_sheet_reader(sheet)?;
        prepare_rows(&mut ws);
        if ws
            .sheet_data
            .row
            .iter()
            .any(|r| r.r.unwrap_or(0) > TOTAL_ROWS as i64)
        {
            return Err(Box::new(ErrMaxRows));
        }
        let max_row = ws
            .sheet_data
            .row
            .last()
            .map(|r| r.r.unwrap_or(0) as i32)
            .unwrap_or(0);
        Ok(Rows {
            file: self,
            ws,
            cur_row: 0,
            seek_row: 0,
            max_row,
            err: None,
            row_opts: RowOpts::default(),
        })
    }

    /// Return all rows in a worksheet as a two-dimensional vector of strings.
    pub fn get_rows(&self, sheet: &str, opts: Options) -> Result<Vec<Vec<String>>> {
        let mut rows = self.rows(sheet)?;
        let mut results = Vec::new();
        while rows.next() {
            results.push(rows.columns(opts.clone())?);
        }
        Ok(results)
    }
}

// ------------------------------------------------------------------
// Internal helpers
// ------------------------------------------------------------------

fn get_or_make_row(
    ws: &mut crate::xml::worksheet::XlsxWorksheet,
    row: i32,
) -> &mut crate::xml::worksheet::XlsxRow {
    if let Some(idx) = ws
        .sheet_data
        .row
        .iter()
        .position(|r| r.r == Some(row as i64))
    {
        return &mut ws.sheet_data.row[idx];
    }
    ws.sheet_data.row.push(crate::xml::worksheet::XlsxRow {
        r: Some(row as i64),
        ..Default::default()
    });
    let last = ws.sheet_data.row.len() - 1;
    &mut ws.sheet_data.row[last]
}

/// Rewrite a single-row reference that points to `row` so that it points to
/// `row2`, preserving absolute references.
fn duplicate_sq_ref_helper(row: i32, row2: i32, r#ref: &str) -> Result<Option<String>> {
    let abs = r#ref.contains('$');
    let mut expanded = r#ref.to_string();
    if !expanded.contains(':') {
        expanded.push(':');
        expanded.push_str(r#ref);
    }
    let coords = range_ref_to_coordinates(&expanded)?;
    let (x1, y1, x2, y2) = (coords[0], coords[1], coords[2], coords[3]);
    if y1 == y2 && y1 == row {
        Ok(Some(coordinates_to_range_ref(&[x1, row2, x2, row2], abs)?))
    } else {
        Ok(None)
    }
}

fn shift_rows_down(ws: &mut crate::xml::worksheet::XlsxWorksheet, start_row: i64, n: i64) {
    for row in &mut ws.sheet_data.row {
        if let Some(row_num) = row.r {
            if row_num >= start_row {
                row.r = Some(row_num + n);
                for c in &mut row.c {
                    if let Some(name) = &c.r {
                        if let Ok((col, old_row)) = crate::lib_util::cell_name_to_coordinates(name)
                        {
                            if let Ok(new_name) = crate::lib_util::coordinates_to_cell_name(
                                col,
                                old_row + n as i32,
                                false,
                            ) {
                                c.r = Some(new_name);
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Ensure worksheet rows are sorted by row number and contiguous from 1 to the
/// maximum row number, matching Go's `prepareSheetXML` behavior for iteration.
pub(crate) fn prepare_rows(ws: &mut XlsxWorksheet) {
    if ws.sheet_data.row.is_empty() {
        return;
    }
    let mut map: BTreeMap<i64, crate::xml::worksheet::XlsxRow> = BTreeMap::new();
    let mut max_row = 0_i64;
    let mut next = 1_i64;
    for row in ws.sheet_data.row.drain(..) {
        let r = row.r.unwrap_or(next);
        max_row = max_row.max(r);
        if !map.contains_key(&r) {
            map.insert(r, row);
        }
        next = r + 1;
    }
    let mut rows = Vec::with_capacity(max_row as usize);
    for r in 1..=max_row {
        let mut row = map.remove(&r).unwrap_or_default();
        row.r = Some(r);
        rows.push(row);
    }
    ws.sheet_data.row = rows;
}

/// Convert a worksheet cell to its string representation.
pub(crate) fn cell_value_string(
    file: &File,
    c: &crate::xml::worksheet::XlsxC,
    raw: bool,
) -> String {
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
            return c.is.as_ref().map(inline_string_text).unwrap_or_default();
        }
        Some("b") => {
            return c.v.as_deref().map_or(String::new(), |v| {
                if v == "1" {
                    "TRUE".to_string()
                } else {
                    "FALSE".to_string()
                }
            });
        }
        Some("str") => return c.v.clone().unwrap_or_default(),
        _ => c.v.clone().unwrap_or_default(),
    };

    // For formula cells without a cached value, return the formula text.
    if c.f.is_some() && c.v.is_none() {
        return format!("={}", c.f.as_ref().unwrap().content);
    }

    if raw {
        return raw_value;
    }

    if let Some(num_fmt_id) = get_cell_num_fmt_id(file, c) {
        if let Ok(value) = raw_value.parse::<f64>() {
            let format_code = get_num_fmt_code(file, num_fmt_id);
            let date1904 = file
                .workbook_reader()
                .ok()
                .and_then(|wb| wb.workbook_pr.as_ref().and_then(|p| p.date1904))
                .unwrap_or(false);
            return crate::numfmt::apply_number_format(
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
    sst.si.get(idx as usize).map_or(String::new(), |si| {
        if let Some(t) = &si.t {
            t.val.clone()
        } else {
            si.r.iter()
                .filter_map(|r| r.t.as_ref().map(|t| t.val.clone()))
                .collect()
        }
    })
}

fn inline_string_text(si: &crate::xml::shared_strings::XlsxSi) -> String {
    if let Some(t) = &si.t {
        return t.val.clone();
    }
    si.r.iter()
        .filter_map(|r| r.t.as_ref().map(|t| t.val.clone()))
        .collect()
}

fn get_cell_num_fmt_id(file: &File, c: &crate::xml::worksheet::XlsxC) -> Option<i32> {
    let style_id = c.s.unwrap_or(0) as usize;
    let styles = file.styles_reader().ok()?;
    let cell_xfs = styles.cell_xfs?;
    let xf = cell_xfs.xf.get(style_id)?;
    xf.num_fmt_id.map(|id| id as i32)
}

fn get_num_fmt_code(file: &File, num_fmt_id: i32) -> Option<String> {
    let styles = file.styles_reader().ok()?;
    styles
        .num_fmts?
        .num_fmt
        .into_iter()
        .find(|n| n.num_fmt_id == num_fmt_id as i64)
        .map(|n| n.format_code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn row_height_round_trip() {
        let f = File::new_with_options(crate::options::Options::default());
        f.set_row_height("Sheet1", 2, 30.0).unwrap();
        assert_eq!(f.get_row_height("Sheet1", 2).unwrap(), 30.0);
        assert_eq!(f.get_row_height("Sheet1", 3).unwrap(), DEFAULT_ROW_HEIGHT);
    }

    #[test]
    fn row_visible_round_trip() {
        let f = File::new_with_options(crate::options::Options::default());
        f.set_row_visible("Sheet1", 2, false).unwrap();
        assert!(!f.get_row_visible("Sheet1", 2).unwrap());
        assert!(f.get_row_visible("Sheet1", 1).unwrap());
    }

    #[test]
    fn set_row_style_range() {
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
        f.set_cell_str("Sheet1", "B2", "x").unwrap();
        f.set_cell_str("Sheet1", "C1", "y").unwrap();

        // Range is applied and out-of-range cells are untouched.
        f.set_row_style("Sheet1", 2, 3, style).unwrap();
        assert_eq!(f.get_cell_style("Sheet1", "B2").unwrap(), style);
        assert_eq!(f.get_cell_style("Sheet1", "A3").unwrap(), style);
        assert_eq!(f.get_cell_style("Sheet1", "C1").unwrap(), 0);

        // End/start are swapped when necessary.
        f.set_cell_str("Sheet1", "A5", "z").unwrap();
        f.set_row_style("Sheet1", 5, 4, style).unwrap();
        assert_eq!(f.get_cell_style("Sheet1", "A5").unwrap(), style);

        // Error cases match Go behavior.
        assert!(f.set_row_style("Sheet1", 0, 1, style).is_err());
        assert!(f.set_row_style("Sheet1", 1, TOTAL_ROWS + 1, style).is_err());
        assert!(f.set_row_style("Sheet1", 1, 1, -1).is_err());
        assert!(f.set_row_style("Sheet1", 1, 1, 999).is_err());
    }

    #[test]
    fn rows_iterator_and_get_rows() {
        let f = File::new_with_options(crate::options::Options::default());
        f.set_cell_str("Sheet1", "A1", "a").unwrap();
        f.set_cell_str("Sheet1", "C1", "c").unwrap();
        f.set_cell_str("Sheet1", "A3", "x").unwrap();

        let mut rows = f.rows("Sheet1").unwrap();
        assert!(rows.next());
        assert_eq!(
            rows.columns(Options::default()).unwrap(),
            vec!["a", "", "c"]
        );
        assert!(rows.next());
        assert_eq!(
            rows.columns(Options::default()).unwrap(),
            Vec::<String>::new()
        );
        assert!(rows.next());
        assert_eq!(rows.columns(Options::default()).unwrap(), vec!["x"]);
        assert!(!rows.next());
        rows.close().unwrap();

        let all = f.get_rows("Sheet1", Options::default()).unwrap();
        assert_eq!(all, vec![vec!["a", "", "c"], vec![], vec!["x"]]);
    }

    #[test]
    fn rows_iterator_opts() {
        let mut opts = Options::default();
        opts.raw_cell_value = true;
        let f = File::new_with_options(opts.clone());
        f.set_cell_value(
            "Sheet1",
            "A1",
            crate::cell::CellValue::Date(chrono::NaiveDate::from_ymd_opt(2024, 7, 13).unwrap()),
        )
        .unwrap();
        let mut rows = f.rows("Sheet1").unwrap();
        assert!(rows.next());
        assert_eq!(rows.columns(opts).unwrap(), vec!["45486"]);
    }

    #[test]
    fn duplicate_row_to_basic() {
        let f = File::new_with_options(crate::options::Options::default());
        f.set_cell_str("Sheet1", "A2", "x").unwrap();
        f.set_cell_str("Sheet1", "B2", "y").unwrap();
        f.duplicate_row_to("Sheet1", 2, 5).unwrap();
        assert_eq!(f.get_cell_value("Sheet1", "A2").unwrap(), "x");
        assert_eq!(f.get_cell_value("Sheet1", "A5").unwrap(), "x");
        assert_eq!(f.get_cell_value("Sheet1", "B5").unwrap(), "y");
    }

    #[test]
    fn remove_row_shifts_formulas_and_values() {
        let f = File::new_with_options(crate::options::Options::default());
        f.set_cell_str("Sheet1", "A1", "a").unwrap();
        f.set_cell_formula("Sheet1", "A3", "A1+1").unwrap();
        f.set_cell_formula("Sheet1", "A4", "A3+1").unwrap();
        f.remove_row("Sheet1", 2).unwrap();

        assert_eq!(f.get_cell_formula("Sheet1", "A2").unwrap(), "A1+1");
        assert_eq!(f.get_cell_formula("Sheet1", "A3").unwrap(), "A2+1");
    }

    #[test]
    fn remove_row_clears_shared_formula_siblings() {
        let f = File::new_with_options(crate::options::Options::default());
        let path = f.get_sheet_xml_path("Sheet1").unwrap();
        let mut ws = f.work_sheet_reader("Sheet1").unwrap();

        ws.sheet_data.row.push(crate::xml::worksheet::XlsxRow {
            r: Some(1),
            c: vec![crate::xml::worksheet::XlsxC {
                r: Some("A1".to_string()),
                f: Some(crate::xml::worksheet::XlsxF {
                    content: "A2*2".to_string(),
                    t: Some("shared".to_string()),
                    r#ref: Some("A1:A3".to_string()),
                    si: Some(0),
                    ..Default::default()
                }),
                ..Default::default()
            }],
            ..Default::default()
        });
        ws.sheet_data.row.push(crate::xml::worksheet::XlsxRow {
            r: Some(2),
            c: vec![crate::xml::worksheet::XlsxC {
                r: Some("A2".to_string()),
                f: Some(crate::xml::worksheet::XlsxF {
                    t: Some("shared".to_string()),
                    si: Some(0),
                    ..Default::default()
                }),
                ..Default::default()
            }],
            ..Default::default()
        });
        ws.sheet_data.row.push(crate::xml::worksheet::XlsxRow {
            r: Some(3),
            c: vec![crate::xml::worksheet::XlsxC {
                r: Some("A3".to_string()),
                f: Some(crate::xml::worksheet::XlsxF {
                    t: Some("shared".to_string()),
                    si: Some(0),
                    ..Default::default()
                }),
                ..Default::default()
            }],
            ..Default::default()
        });
        f.sheet.insert(path, ws);

        f.remove_row("Sheet1", 1).unwrap();
        let ws = f.work_sheet_reader("Sheet1").unwrap();
        assert!(
            ws.sheet_data
                .row
                .iter()
                .flat_map(|r| &r.c)
                .all(|c| c.f.is_none())
        );
    }

    #[test]
    fn duplicate_row_to_copies_single_row_merge() {
        let f = File::new_with_options(crate::options::Options::default());
        f.set_cell_str("Sheet1", "A1", "merged").unwrap();
        f.merge_cell("Sheet1", "A1", "B1").unwrap();
        f.duplicate_row_to("Sheet1", 1, 3).unwrap();

        let merges = f.get_merge_cells("Sheet1").unwrap();
        assert!(merges.contains(&"A1:B1".to_string()));
        assert!(merges.contains(&"A3:B3".to_string()));
        assert_eq!(f.get_cell_value("Sheet1", "A3").unwrap(), "merged");
    }
}
