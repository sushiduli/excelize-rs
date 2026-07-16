//! Column-level API.
//!
//! This module corresponds to `col.go` in the Go implementation.

use std::collections::HashMap;

use crate::constants::{
    DEFAULT_COL_WIDTH, DEFAULT_COL_WIDTH_PIXELS, DEFAULT_FONT_SIZE, MAX_COLUMN_WIDTH, TOTAL_ROWS,
};
use crate::errors::Result;
use crate::errors::{ErrColumnNumber, ErrColumnWidth, ErrMaxRows, ErrSheetNotExist};
use crate::file::File;
use crate::lib_util::{cell_name_to_coordinates, column_name_to_number};
use crate::options::Options;
use crate::row::cell_value_string;
use crate::styles::Font;
use crate::xml::common::RichTextRun;
use crate::xml::worksheet::{XlsxCol, XlsxCols, XlsxWorksheet};

impl File {
    /// Set the width of one or more columns.
    pub fn set_col_width(
        &self,
        sheet: &str,
        start_col: &str,
        end_col: &str,
        width: f64,
    ) -> Result<()> {
        if width > MAX_COLUMN_WIDTH as f64 {
            return Err(Box::new(ErrColumnWidth));
        }
        let start = column_name_to_number(start_col)?;
        let end = column_name_to_number(end_col)?;
        if start > end {
            return Err(ErrColumnNumber.to_string().into());
        }
        let path = self
            .get_sheet_xml_path(sheet)
            .ok_or_else(|| ErrSheetNotExist {
                sheet_name: sheet.to_string(),
            })?;
        let mut ws = self.work_sheet_reader(sheet)?;
        ensure_cols(&mut ws);
        let cols = ws.cols.as_mut().unwrap();
        merge_col_range(&mut cols.col, start as i64, end as i64, |col| {
            col.width = Some(width);
            col.custom_width = Some(true);
            col.best_fit = Some(true);
        });
        self.sheet.insert(path, ws);
        Ok(())
    }

    /// Get the width of a column.
    pub fn get_col_width(&self, sheet: &str, col: &str) -> Result<f64> {
        let col_num = column_name_to_number(col)?;
        let ws = self.work_sheet_reader(sheet)?;
        if let Some(cols) = &ws.cols {
            for c in &cols.col {
                if col_num as i64 >= c.min && col_num as i64 <= c.max {
                    return Ok(c.width.unwrap_or(DEFAULT_COL_WIDTH));
                }
            }
        }
        if let Some(fmt) = &ws.sheet_format_pr {
            if let Some(w) = fmt.default_col_width {
                return Ok(w);
            }
        }
        Ok(DEFAULT_COL_WIDTH)
    }

    /// Set the visibility of one or more columns.
    pub fn set_col_visible(
        &self,
        sheet: &str,
        start_col: &str,
        end_col: &str,
        visible: bool,
    ) -> Result<()> {
        let start = column_name_to_number(start_col)?;
        let end = column_name_to_number(end_col)?;
        let path = self
            .get_sheet_xml_path(sheet)
            .ok_or_else(|| ErrSheetNotExist {
                sheet_name: sheet.to_string(),
            })?;
        let mut ws = self.work_sheet_reader(sheet)?;
        ensure_cols(&mut ws);
        let cols = ws.cols.as_mut().unwrap();
        merge_col_range(&mut cols.col, start as i64, end as i64, |col| {
            col.hidden = Some(!visible);
        });
        self.sheet.insert(path, ws);
        Ok(())
    }

    /// Get the visibility of a column.
    pub fn get_col_visible(&self, sheet: &str, col: &str) -> Result<bool> {
        let col_num = column_name_to_number(col)?;
        let ws = self.work_sheet_reader(sheet)?;
        if let Some(cols) = &ws.cols {
            for c in &cols.col {
                if col_num as i64 >= c.min && col_num as i64 <= c.max {
                    return Ok(!c.hidden.unwrap_or(false));
                }
            }
        }
        Ok(true)
    }

    /// Set the outline level of one or more columns.
    pub fn set_col_outline_level(
        &self,
        sheet: &str,
        start_col: &str,
        end_col: &str,
        level: u8,
    ) -> Result<()> {
        if level == 0 || level > 7 {
            return Err(Box::new(crate::errors::ErrOutlineLevel));
        }
        let start = column_name_to_number(start_col)?;
        let end = column_name_to_number(end_col)?;
        let path = self
            .get_sheet_xml_path(sheet)
            .ok_or_else(|| ErrSheetNotExist {
                sheet_name: sheet.to_string(),
            })?;
        let mut ws = self.work_sheet_reader(sheet)?;
        ensure_cols(&mut ws);
        let cols = ws.cols.as_mut().unwrap();
        merge_col_range(&mut cols.col, start as i64, end as i64, |col| {
            col.outline_level = Some(level);
        });
        self.sheet.insert(path, ws);
        Ok(())
    }

    /// Get the outline level of a column.
    pub fn get_col_outline_level(&self, sheet: &str, col: &str) -> Result<u8> {
        let col_num = column_name_to_number(col)?;
        let ws = self.work_sheet_reader(sheet)?;
        if let Some(cols) = &ws.cols {
            for c in &cols.col {
                if col_num as i64 >= c.min && col_num as i64 <= c.max {
                    return Ok(c.outline_level.unwrap_or(0));
                }
            }
        }
        Ok(0)
    }

    /// Insert `n` columns before `col`.
    pub fn insert_cols(&self, sheet: &str, col: &str, n: usize) -> Result<()> {
        if n == 0 {
            return Ok(());
        }
        let col_num = column_name_to_number(col)?;
        let path = self
            .get_sheet_xml_path(sheet)
            .ok_or_else(|| ErrSheetNotExist {
                sheet_name: sheet.to_string(),
            })?;
        let mut ws = self.work_sheet_reader(sheet)?;
        shift_cols_right(&mut ws, col_num as i64, n as i64);
        self.sheet.insert(path, ws);
        Ok(())
    }

    /// Remove a column.
    pub fn remove_col(&self, sheet: &str, col: &str) -> Result<()> {
        let col_num = column_name_to_number(col)?;
        let path = self
            .get_sheet_xml_path(sheet)
            .ok_or_else(|| ErrSheetNotExist {
                sheet_name: sheet.to_string(),
            })?;
        let mut ws = self.work_sheet_reader(sheet)?;
        // Remove cells in the column and shift others left.
        for row in &mut ws.sheet_data.row {
            row.c.retain(|c| {
                if let Some(name) = &c.r {
                    if let Ok((cnum, _)) = crate::lib_util::cell_name_to_coordinates(name) {
                        return cnum != col_num;
                    }
                }
                true
            });
            for cell in &mut row.c {
                if let Some(name) = &cell.r {
                    if let Ok((cnum, row_num)) = crate::lib_util::cell_name_to_coordinates(name) {
                        if cnum > col_num {
                            if let Ok(new_name) =
                                crate::lib_util::coordinates_to_cell_name(cnum - 1, row_num, false)
                            {
                                cell.r = Some(new_name);
                            }
                        }
                    }
                }
            }
        }
        // Update cols definitions.
        if let Some(cols) = ws.cols.as_mut() {
            let mut new_cols = Vec::new();
            for c in &cols.col {
                if c.max < col_num as i64 {
                    new_cols.push(c.clone());
                } else if c.min > col_num as i64 {
                    let mut nc = c.clone();
                    nc.min -= 1;
                    nc.max -= 1;
                    new_cols.push(nc);
                } else {
                    if c.min < col_num as i64 {
                        let mut before = c.clone();
                        before.max = col_num as i64 - 1;
                        new_cols.push(before);
                    }
                    if c.max > col_num as i64 {
                        let mut after = c.clone();
                        after.min = col_num as i64;
                        after.max -= 1;
                        new_cols.push(after);
                    }
                }
            }
            cols.col = new_cols;
        }
        self.sheet.insert(path, ws);
        Ok(())
    }

    /// Apply a style index to one or more columns.
    pub fn set_col_style(
        &self,
        sheet: &str,
        start_col: &str,
        end_col: &str,
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
        let start = column_name_to_number(start_col)?;
        let end = column_name_to_number(end_col)?;
        let path = self
            .get_sheet_xml_path(sheet)
            .ok_or_else(|| ErrSheetNotExist {
                sheet_name: sheet.to_string(),
            })?;
        let mut ws = self.work_sheet_reader(sheet)?;
        ensure_cols(&mut ws);
        let cols = ws.cols.as_mut().unwrap();
        merge_col_range(&mut cols.col, start as i64, end as i64, |col| {
            col.style = Some(style_id as i64);
        });
        self.sheet.insert(path, ws);
        Ok(())
    }

    /// Get the style ID applied to a column.
    pub fn get_col_style(&self, sheet: &str, col: &str) -> Result<i32> {
        let col_num = column_name_to_number(col)?;
        let ws = self.work_sheet_reader(sheet)?;
        if let Some(cols) = &ws.cols {
            for c in &cols.col {
                if col_num as i64 >= c.min && col_num as i64 <= c.max {
                    return Ok(c.style.unwrap_or(0) as i32);
                }
            }
        }
        Ok(0)
    }

    /// Auto fit column width according to the text content.
    ///
    /// Equivalent to Go `AutoFitColWidth`. The width is calculated approximately
    /// based on the default font format.
    pub fn auto_fit_col_width(&self, sheet: &str, columns: &str) -> Result<()> {
        let (min_col, max_col) = parse_col_range(columns)?;
        let rows = self.get_rows(sheet, Options::default())?;
        let default_fnt = read_default_font(self);
        let path = self
            .get_sheet_xml_path(sheet)
            .ok_or_else(|| ErrSheetNotExist {
                sheet_name: sheet.to_string(),
            })?;
        let mut ws = self.work_sheet_reader(sheet)?;
        for col in min_col..=max_col {
            let mut width = 0.0;
            for (row_idx, _) in rows.iter().enumerate() {
                let cell =
                    crate::lib_util::coordinates_to_cell_name(col, row_idx as i32 + 1, false)?;
                let val = match self.calc_cell_value(sheet, &cell) {
                    Ok(v) => v,
                    Err(_) => continue,
                };
                if val.is_empty() {
                    continue;
                }
                let style_id = self.get_cell_style(sheet, &cell).unwrap_or(0);
                let mut fnt = default_fnt.clone();
                if let Ok(style) = self.get_style(style_id) {
                    if let Some(font) = style.font {
                        fnt = font;
                    }
                }
                if let Ok(cell_type) = self.get_cell_type(sheet, &cell) {
                    if cell_type == crate::cell::CellType::InlineString
                        || cell_type == crate::cell::CellType::SharedString
                    {
                        let runs = self.get_cell_rich_text(sheet, &cell).unwrap_or_default();
                        let w = calc_rich_text_width(&fnt, &runs);
                        if w > width {
                            width = w;
                        }
                        continue;
                    }
                }
                let w = calc_text_width(&fnt, &val);
                if w > width {
                    width = w;
                }
            }
            if width > 0.0 {
                width += 2.0;
                if width > MAX_COLUMN_WIDTH as f64 {
                    width = MAX_COLUMN_WIDTH as f64;
                }
                ensure_cols(&mut ws);
                let cols = ws.cols.as_mut().unwrap();
                merge_col_range(&mut cols.col, col as i64, col as i64, |c| {
                    c.hidden = Some(false);
                    c.width = Some(width);
                    c.custom_width = Some(true);
                });
            }
        }
        self.sheet.insert(path, ws);
        Ok(())
    }
}

// ------------------------------------------------------------------
// Column iterator
// ------------------------------------------------------------------

/// Streaming column iterator.
#[derive(Debug)]
pub struct Cols<'a> {
    file: &'a File,
    ws: XlsxWorksheet,
    cur_col: i32,
    total_cols: i32,
    total_rows: i32,
    err: Option<String>,
}

impl<'a> Cols<'a> {
    /// Advance to the next column. Returns `false` when there are no more columns.
    pub fn next(&mut self) -> bool {
        self.cur_col += 1;
        self.cur_col <= self.total_cols
    }

    /// Return the first error that occurred during iteration, if any.
    pub fn error(&self) -> Option<&str> {
        self.err.as_deref()
    }

    /// Return the values of the current column's cells as strings.
    pub fn rows(&self, opts: Options) -> Result<Vec<String>> {
        if self.cur_col < 1 || self.cur_col > self.total_cols || self.total_rows == 0 {
            return Ok(Vec::new());
        }
        let mut cells = vec![String::new(); self.total_rows as usize];
        for row in &self.ws.sheet_data.row {
            let row_num = row.r.unwrap_or(0) as i32;
            if row_num < 1 || row_num > self.total_rows {
                continue;
            }
            let mut cell_col: i32 = 0;
            for c in &row.c {
                cell_col += 1;
                if let Some(name) = &c.r {
                    cell_col = cell_name_to_coordinates(name)?.0;
                }
                if cell_col == self.cur_col {
                    cells[(row_num - 1) as usize] =
                        cell_value_string(self.file, c, opts.raw_cell_value);
                }
            }
        }
        Ok(cells)
    }
}

impl File {
    /// Return a streaming iterator over the columns of a worksheet.
    pub fn cols(&self, sheet: &str) -> Result<Cols<'_>> {
        let ws = self.work_sheet_reader(sheet)?;
        let (total_rows, total_cols) = scan_dimensions(&ws)?;
        Ok(Cols {
            file: self,
            ws,
            cur_col: 0,
            total_cols,
            total_rows,
            err: None,
        })
    }

    /// Return all columns in a worksheet as a two-dimensional vector of strings.
    pub fn get_cols(&self, sheet: &str, opts: Options) -> Result<Vec<Vec<String>>> {
        let mut cols = self.cols(sheet)?;
        let mut results = Vec::new();
        while cols.next() {
            results.push(cols.rows(opts.clone())?);
        }
        Ok(results)
    }
}

fn scan_dimensions(ws: &XlsxWorksheet) -> Result<(i32, i32)> {
    let mut total_rows = 0;
    let mut total_cols = 0;
    let mut row_num = 0;
    for row in &ws.sheet_data.row {
        row_num += 1;
        if let Some(r) = row.r {
            row_num = r as i32;
        }
        if row_num > TOTAL_ROWS {
            return Err(Box::new(ErrMaxRows));
        }
        total_rows = total_rows.max(row_num);
        let mut cell_col = 0;
        for c in &row.c {
            cell_col += 1;
            if let Some(name) = &c.r {
                cell_col = cell_name_to_coordinates(name)?.0;
            }
            total_cols = total_cols.max(cell_col);
        }
    }
    Ok((total_rows, total_cols))
}

// ------------------------------------------------------------------
// Internal helpers
// ------------------------------------------------------------------

fn ensure_cols(ws: &mut crate::xml::worksheet::XlsxWorksheet) {
    if ws.cols.is_none() {
        ws.cols = Some(XlsxCols::default());
    }
}

/// Apply a mutation to every column record covering `[min, max]`, splitting
/// existing records when needed so the range is exactly represented.
fn merge_col_range(cols: &mut Vec<XlsxCol>, min: i64, max: i64, mut f: impl FnMut(&mut XlsxCol)) {
    let mut new_cols = Vec::new();
    let mut covered = false;
    for col in cols.drain(..) {
        if col.max < min || col.min > max {
            new_cols.push(col);
            continue;
        }
        covered = true;
        if col.min < min {
            let mut before = col.clone();
            before.max = min - 1;
            new_cols.push(before);
        }
        let mut target = col.clone();
        target.min = target.min.max(min);
        target.max = target.max.min(max);
        f(&mut target);
        new_cols.push(target);
        if col.max > max {
            let mut after = col;
            after.min = max + 1;
            new_cols.push(after);
        }
    }
    if !covered {
        let mut target = XlsxCol {
            min,
            max,
            ..Default::default()
        };
        f(&mut target);
        new_cols.push(target);
    }
    // Merge adjacent identical definitions.
    new_cols.sort_by_key(|c| c.min);
    let mut merged: Vec<XlsxCol> = Vec::new();
    for col in new_cols {
        if let Some(last) = merged.last_mut() {
            if last.width == col.width
                && last.hidden == col.hidden
                && last.outline_level == col.outline_level
                && last.style == col.style
                && last.custom_width == col.custom_width
                && last.best_fit == col.best_fit
                && last.max + 1 == col.min
            {
                last.max = col.max;
                continue;
            }
        }
        merged.push(col);
    }
    *cols = merged;
}

fn shift_cols_right(ws: &mut crate::xml::worksheet::XlsxWorksheet, start_col: i64, n: i64) {
    // Shift cell references.
    for row in &mut ws.sheet_data.row {
        for cell in &mut row.c {
            if let Some(name) = &cell.r {
                if let Ok((col, row_num)) = crate::lib_util::cell_name_to_coordinates(name) {
                    if col >= start_col as i32 {
                        let new_col = (col as i64 + n) as i32;
                        if let Ok(new_name) =
                            crate::lib_util::coordinates_to_cell_name(new_col, row_num, false)
                        {
                            cell.r = Some(new_name);
                        }
                    }
                }
            }
        }
    }
    // Shift col definitions.
    if let Some(cols) = ws.cols.as_mut() {
        for col in &mut cols.col {
            if col.min >= start_col {
                col.min += n;
                col.max += n;
            } else if col.max >= start_col {
                col.max += n;
            }
        }
    }
}

fn parse_col_range(columns: &str) -> Result<(i32, i32)> {
    let parts: Vec<&str> = columns.split(':').collect();
    let min = column_name_to_number(parts[0])?;
    let max = if parts.len() == 2 {
        column_name_to_number(parts[1])?
    } else {
        min
    };
    if min <= max {
        Ok((min, max))
    } else {
        Ok((max, min))
    }
}

fn read_default_font(file: &File) -> Font {
    if let Ok(styles) = file.styles_reader() {
        if let Some(fonts) = styles.fonts {
            if let Some(f) = fonts.font.first() {
                return Font {
                    size: f.sz.as_ref().and_then(|s| s.val),
                    name: f.name.as_ref().and_then(|n| n.val.clone()),
                    family: f.family.as_ref().and_then(|v| v.val),
                    bold: f.b.as_ref().and_then(|b| b.val),
                    italic: f.i.as_ref().and_then(|i| i.val),
                    strike: f.strike.as_ref().and_then(|s| s.val),
                    ..Default::default()
                };
            }
        }
    }
    Font {
        size: Some(DEFAULT_FONT_SIZE),
        name: Some("Calibri".to_string()),
        ..Default::default()
    }
}

fn font_width_factors(name: Option<&str>) -> (f64, f64, f64) {
    let map: HashMap<String, (f64, f64, f64)> = [
        ("calibri", (0.97, 1.30, 1.00)),
        ("aptos", (1.03, 1.37, 1.00)),
        ("arial", (1.07, 1.42, 1.00)),
        ("arial narrow", (0.88, 1.20, 1.00)),
        ("calibri light", (0.98, 1.19, 1.00)),
        ("cambria", (1.07, 1.28, 1.00)),
        ("consolas", (1.21, 1.21, 1.00)),
        ("courier new", (1.45, 1.45, 1.00)),
        ("times new roman", (1.07, 1.42, 1.00)),
        ("verdana", (1.10, 1.45, 1.00)),
    ]
    .iter()
    .map(|(k, v)| (k.to_string(), *v))
    .collect();
    name.and_then(|n| map.get(&n.to_lowercase()))
        .copied()
        .unwrap_or((1.0, 1.0, 1.0))
}

fn is_wide_rune(r: char) -> bool {
    matches!(
        r,
        '\u{1100}'..='\u{115F}'
            | '\u{2E80}'..='\u{9FFF}'
            | '\u{A960}'..='\u{A97F}'
            | '\u{AC00}'..='\u{D7AF}'
            | '\u{F900}'..='\u{FAFF}'
            | '\u{FF01}'..='\u{FF60}'
            | '\u{FFE0}'..='\u{FFE6}'
    )
}

fn calc_text_width(font: &Font, text: &str) -> f64 {
    let mut lower = 0.0;
    let mut upper = 0.0;
    let mut wide = 0.0;
    for r in text.chars() {
        if r == '\n' || r == '\r' {
            continue;
        }
        if is_wide_rune(r) {
            wide += 2.0;
        } else if r.is_uppercase() {
            upper += 1.0;
        } else {
            lower += 1.0;
        }
    }
    let size = font.size.unwrap_or(DEFAULT_FONT_SIZE);
    let (lf, uf, wf) = font_width_factors(font.name.as_deref());
    let mut w = lower * lf + upper * uf + wide * wf;
    w *= size / DEFAULT_FONT_SIZE;
    if font.bold.unwrap_or(false) {
        w *= 1.05;
    }
    if font.italic.unwrap_or(false) {
        w *= 1.05;
    }
    if font.vert_align.as_deref().unwrap_or("baseline") != "baseline" {
        w *= 0.6;
    }
    w
}

fn calc_rich_text_width(default_font: &Font, runs: &[RichTextRun]) -> f64 {
    let mut w = 0.0;
    let mut width = 0.0;
    for run in runs {
        let mut fnt = default_font.clone();
        if let Some(font) = &run.font {
            fnt = font.clone();
        }
        if let Some(i) = run.text.find(['\n', '\r'].as_ref()) {
            let first = &run.text[..i];
            let rest = &run.text[i + 1..];
            w += calc_text_width(&fnt, first);
            if w > width {
                width = w;
            }
            w = calc_text_width(&fnt, rest);
        } else {
            w += calc_text_width(&fnt, &run.text);
        }
    }
    if w > width {
        width = w;
    }
    width
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn col_width_round_trip() {
        let f = File::new_with_options(crate::options::Options::default());
        f.set_col_width("Sheet1", "A", "C", 20.0).unwrap();
        assert_eq!(f.get_col_width("Sheet1", "B").unwrap(), 20.0);
        assert_eq!(f.get_col_width("Sheet1", "D").unwrap(), DEFAULT_COL_WIDTH);
    }

    #[test]
    fn col_visible_round_trip() {
        let f = File::new_with_options(crate::options::Options::default());
        f.set_col_visible("Sheet1", "B", "B", false).unwrap();
        assert!(!f.get_col_visible("Sheet1", "B").unwrap());
        assert!(f.get_col_visible("Sheet1", "A").unwrap());
    }

    #[test]
    fn cols_iterator_and_get_cols() {
        let f = File::new_with_options(crate::options::Options::default());
        f.set_cell_str("Sheet1", "A1", "a").unwrap();
        f.set_cell_str("Sheet1", "B2", "b").unwrap();

        let mut cols = f.cols("Sheet1").unwrap();
        assert!(cols.next());
        assert_eq!(cols.rows(Options::default()).unwrap(), vec!["a", ""]);
        assert!(cols.next());
        assert_eq!(cols.rows(Options::default()).unwrap(), vec!["", "b"]);
        assert!(!cols.next());

        let all = f.get_cols("Sheet1", Options::default()).unwrap();
        assert_eq!(all, vec![vec!["a", ""], vec!["", "b"]]);
    }

    #[test]
    fn col_style_round_trip() {
        let f = File::new_with_options(crate::options::Options::default());
        f.set_col_style("Sheet1", "B", "D", 0).unwrap();
        assert_eq!(f.get_col_style("Sheet1", "C").unwrap(), 0);
    }

    #[test]
    fn auto_fit_col_width_basic() {
        let f = File::new_with_options(crate::options::Options::default());
        f.set_cell_str("Sheet1", "A1", "Hello, World!").unwrap();
        let before = f.get_col_width("Sheet1", "A").unwrap();
        f.auto_fit_col_width("Sheet1", "A").unwrap();
        let after = f.get_col_width("Sheet1", "A").unwrap();
        assert!(after > before);
    }
}
