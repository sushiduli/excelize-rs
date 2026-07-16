//! Streaming writer API.
//!
//! Ported from Go `stream.go`.

use std::collections::HashMap;
use std::fs;
use std::fs::File as FsFile;
use std::io::{self, BufReader, Cursor, Read, Write};
use std::path::PathBuf;
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::{NaiveDate, NaiveDateTime};
use quick_xml::Reader;
use quick_xml::escape::escape;
use quick_xml::events::Event;
use quick_xml::se::to_string as xml_to_string;
use std::time::Duration;

use crate::cell::CellValue;
use crate::constants::{
    DEFAULT_COL_WIDTH, MAX_COLUMN_WIDTH, MAX_COLUMNS, MAX_ROW_HEIGHT, MIN_COLUMNS,
    NAMESPACE_SPREADSHEET, STREAM_CHUNK_SIZE, TOTAL_ROWS, XML_HEADER,
};
use crate::date;
use crate::errors::{
    ErrColumnNumber, ErrColumnWidth, ErrMaxRowHeight, ErrOutlineLevel, ErrSheetNotExist, Result,
};
use crate::file::File;
use crate::lib_util::{
    cell_name_to_coordinates, cell_refs_to_coordinates, coordinates_to_cell_name,
    coordinates_to_range_ref, range_ref_to_coordinates, sort_coordinates,
};
use crate::styles::Style;
use crate::xml::common::{RichTextRun, XlsxT};
use crate::xml::shared_strings::XlsxSi;
use crate::xml::table::{
    Table as TableOptions, XlsxAutoFilter, XlsxTable, XlsxTableColumn, XlsxTableColumns,
    XlsxTableStyleInfo,
};
use crate::xml::worksheet::{
    Panes, XlsxBreaks, XlsxBrk, XlsxC, XlsxCol, XlsxColBreaks, XlsxCols, XlsxF, XlsxPane,
    XlsxRowBreaks, XlsxSelection, XlsxSheetViews, XlsxWorksheet,
};

/// Cell value used by the streaming writer to carry a per-cell style or formula.
///
/// Ported from Go `stream.go` `Cell`. The `value` field is optional to match
/// Go's `interface{}` value which may be `nil`.
#[derive(Debug, Clone, PartialEq)]
pub struct Cell {
    pub style_id: i32,
    pub formula: String,
    pub value: Option<CellValue>,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            style_id: 0,
            formula: String::new(),
            value: None,
        }
    }
}

/// Row formatting options used by `StreamWriter::set_row`.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct RowOpts {
    pub height: f64,
    pub hidden: bool,
    pub style_id: i32,
    pub outline_level: i32,
}

/// Internal representation of a value to be written by the streaming writer.
pub struct StreamCell {
    style_id: i32,
    formula: String,
    value: Option<CellValue>,
}

/// Values that can be passed to `StreamWriter::set_row`.
pub trait StreamCellValue {
    fn to_stream(&self) -> StreamCell;
}

impl StreamCellValue for CellValue {
    fn to_stream(&self) -> StreamCell {
        StreamCell {
            style_id: 0,
            formula: String::new(),
            value: Some(self.clone()),
        }
    }
}

impl StreamCellValue for Cell {
    fn to_stream(&self) -> StreamCell {
        StreamCell {
            style_id: self.style_id,
            formula: self.formula.clone(),
            value: self.value.clone(),
        }
    }
}

impl StreamCellValue for &Cell {
    fn to_stream(&self) -> StreamCell {
        (*self).to_stream()
    }
}

impl<T: StreamCellValue> StreamCellValue for Option<T> {
    fn to_stream(&self) -> StreamCell {
        match self {
            Some(v) => v.to_stream(),
            None => StreamCell {
                style_id: 0,
                formula: String::new(),
                value: None,
            },
        }
    }
}

macro_rules! impl_stream_cell_value_for_int {
    ($t:ty) => {
        impl StreamCellValue for $t {
            fn to_stream(&self) -> StreamCell {
                CellValue::Int(*self as i64).to_stream()
            }
        }
    };
}

impl_stream_cell_value_for_int!(i8);
impl_stream_cell_value_for_int!(i16);
impl_stream_cell_value_for_int!(i32);
impl_stream_cell_value_for_int!(i64);
impl_stream_cell_value_for_int!(isize);
impl_stream_cell_value_for_int!(u8);
impl_stream_cell_value_for_int!(u16);
impl_stream_cell_value_for_int!(u32);
impl_stream_cell_value_for_int!(u64);
impl_stream_cell_value_for_int!(usize);

impl StreamCellValue for f32 {
    fn to_stream(&self) -> StreamCell {
        CellValue::Float(*self as f64).to_stream()
    }
}

impl StreamCellValue for f64 {
    fn to_stream(&self) -> StreamCell {
        CellValue::Float(*self).to_stream()
    }
}

impl StreamCellValue for bool {
    fn to_stream(&self) -> StreamCell {
        CellValue::Bool(*self).to_stream()
    }
}

impl StreamCellValue for &str {
    fn to_stream(&self) -> StreamCell {
        CellValue::String(self.to_string()).to_stream()
    }
}

impl StreamCellValue for String {
    fn to_stream(&self) -> StreamCell {
        CellValue::String(self.clone()).to_stream()
    }
}

impl StreamCellValue for NaiveDateTime {
    fn to_stream(&self) -> StreamCell {
        CellValue::DateTime(*self).to_stream()
    }
}

impl StreamCellValue for NaiveDate {
    fn to_stream(&self) -> StreamCell {
        CellValue::Date(*self).to_stream()
    }
}

impl StreamCellValue for chrono::NaiveTime {
    fn to_stream(&self) -> StreamCell {
        CellValue::Time(*self).to_stream()
    }
}

impl StreamCellValue for Duration {
    fn to_stream(&self) -> StreamCell {
        CellValue::Float(self.as_nanos() as f64 / 86_400_000_000_000.0).to_stream()
    }
}

impl StreamCellValue for Vec<RichTextRun> {
    fn to_stream(&self) -> StreamCell {
        CellValue::RichText(self.clone()).to_stream()
    }
}

impl StreamCellValue for num_complex::Complex32 {
    fn to_stream(&self) -> StreamCell {
        CellValue::String(self.to_string()).to_stream()
    }
}

impl StreamCellValue for num_complex::Complex64 {
    fn to_stream(&self) -> StreamCell {
        CellValue::String(self.to_string()).to_stream()
    }
}

/// Streaming writer for worksheets with large amounts of data.
#[derive(Debug)]
pub struct StreamWriter<'a> {
    file: &'a File,
    sheet: String,
    #[allow(dead_code)]
    sheet_id: i32,
    sheet_written: bool,
    worksheet: XlsxWorksheet,
    raw_data: BufferedWriter,
    rows: i32,
    merge_cells_count: i32,
    merge_cells: String,
    table_parts: String,
    flushed: bool,
}

impl<'a> StreamWriter<'a> {
    /// Create a new stream writer for the given worksheet.
    pub fn new(file: &'a File, sheet: &str) -> Result<Self> {
        crate::excelize::check_sheet_name(sheet)?;
        let sheet_id = file.get_sheet_id(sheet);
        if sheet_id == -1 {
            return Err(Box::new(ErrSheetNotExist {
                sheet_name: sheet.to_string(),
            }));
        }

        let worksheet = file.work_sheet_reader(sheet)?;
        let tmp_dir = {
            let opts = file.options.lock().unwrap();
            opts.tmp_dir.clone()
        };

        let mut raw_data = BufferedWriter::new(tmp_dir);
        raw_data.write_str(&format!(
            "{XML_HEADER}<worksheet{TEMPLATE_NAMESPACE_ID_MAP}"
        ))?;
        bulk_append_fields(&mut raw_data, &worksheet, 3, 4)?;

        Ok(Self {
            file,
            sheet: sheet.to_string(),
            sheet_id,
            sheet_written: false,
            worksheet,
            raw_data,
            rows: 0,
            merge_cells_count: 0,
            merge_cells: String::new(),
            table_parts: String::new(),
            flushed: false,
        })
    }

    /// Add a table within the streamed sheet.
    pub fn add_table(&mut self, table: &TableOptions) -> Result<()> {
        let options = parse_table_options(table)?;
        let mut coordinates = range_ref_to_coordinates(&options.range).map_err(|e| io_err(e))?;
        sort_coordinates(&mut coordinates).map_err(|e| io_err(e))?;

        // Correct the minimum number of rows, the table at least two lines.
        let mut coordinates = coordinates;
        if coordinates[1] == coordinates[3] {
            coordinates[3] += 1;
        }

        let ref_str = coordinates_to_range_ref(&coordinates, false).map_err(|e| io_err(e))?;
        let table_headers = self.get_row_values(coordinates[1], coordinates[0], coordinates[2])?;
        let table_columns: Vec<XlsxTableColumn> = table_headers
            .into_iter()
            .enumerate()
            .map(|(i, name)| XlsxTableColumn {
                id: (i + 1) as i64,
                name,
                ..Default::default()
            })
            .collect();

        let table_id = self.file.count_tables() + 1;
        let name = if options.name.is_empty() {
            format!("Table{table_id}")
        } else {
            options.name.clone()
        };

        let tbl = XlsxTable {
            xmlns: Some(NAMESPACE_SPREADSHEET.to_string()),
            id: table_id as i64,
            name: name.clone(),
            display_name: Some(name),
            r#ref: ref_str.clone(),
            auto_filter: Some(XlsxAutoFilter {
                r#ref: ref_str,
                ..Default::default()
            }),
            table_columns: Some(XlsxTableColumns {
                count: table_columns.len() as i64,
                table_column: table_columns,
            }),
            table_style_info: Some(XlsxTableStyleInfo {
                name: Some(options.style_name.clone()),
                show_first_column: options.show_first_column,
                show_last_column: options.show_last_column,
                show_row_stripes: options.show_row_stripes.unwrap_or(true),
                show_column_stripes: options.show_column_stripes,
            }),
            ..Default::default()
        };

        let sheet_relationships_table_xml = format!("../tables/table{table_id}.xml");
        let table_xml = sheet_relationships_table_xml.replace("..", "xl");
        let sheet_xml_path = self
            .file
            .get_sheet_xml_path(&self.sheet)
            .unwrap_or_default();
        let sheet_rels = format!(
            "xl/worksheets/_rels/{}.rels",
            sheet_xml_path.trim_start_matches("xl/worksheets/")
        );
        let r_id = self.file.add_rels(
            &sheet_rels,
            crate::constants::SOURCE_RELATIONSHIP_TABLE,
            &sheet_relationships_table_xml,
            "",
        );

        self.table_parts = format!(
            r#"<tableParts count="1"><tablePart r:id="rId{r_id}"></tablePart></tableParts>"#
        );

        self.file.add_content_type_part(table_id, "table")?;
        let mut body = xml_to_string(&tbl)?.into_bytes();
        crate::file::strip_empty_attributes(&mut body);
        self.file.save_file_list(&table_xml, &body);
        Ok(())
    }

    /// Write a row of values starting at `cell`.
    ///
    /// Values may be any type implementing [`StreamCellValue`]. Mixed types can
    /// be passed as `&[&dyn StreamCellValue]` to match Go's `[]interface{}`.
    /// Row options are optional and default to [`RowOpts::default`] when `None`.
    pub fn set_row(
        &mut self,
        cell: &str,
        values: &[&dyn StreamCellValue],
        opts: Option<RowOpts>,
    ) -> Result<()> {
        if self.flushed {
            return Err(crate::errors::new_stream_set_row_order_error("SetRow").into());
        }
        let (col, row) = cell_name_to_coordinates(cell).map_err(|e| io_err(e))?;
        if row <= self.rows {
            return Err(crate::errors::new_stream_set_row_error(row).into());
        }
        self.rows = row;
        self.write_sheet_data()?;

        let opts = opts.unwrap_or_default();
        let attrs = opts.marshal_attrs()?;
        self.raw_data
            .write_str(&format!(r#"<row r="{row}"{attrs}>"#))?;

        for (i, val) in values.iter().enumerate() {
            let val = val.to_stream();
            // Match Go behavior: skip only values that are nil (None) and have
            // no per-cell style or formula. Empty strings are written as inline
            // strings, and styled/formula cells are written even without a value.
            if val.value.is_none() && val.formula.is_empty() && val.style_id == 0 {
                continue;
            }
            let ref_str =
                coordinates_to_cell_name(col + i as i32, row, false).map_err(|e| io_err(e))?;
            let mut c = XlsxC {
                r: Some(ref_str),
                s: Some(prepare_cell_style(
                    &self.worksheet,
                    (col + i as i32) as i64,
                    row as i64,
                    opts.style_id as i64,
                )),
                ..Default::default()
            };

            if !val.formula.is_empty() {
                set_cell_formula(&mut c, &val.formula);
            }
            if val.style_id > 0 {
                c.s = Some(val.style_id as i64);
            }
            if let Some(value) = val.value {
                if let Err(e) = self.set_cell_val_func(&mut c, value) {
                    self.raw_data.write_str("</row>")?;
                    return Err(e);
                }
            }
            write_cell(&mut self.raw_data, &c)?;
        }
        self.raw_data.write_str("</row>")?;
        Ok(self.raw_data.sync()?)
    }

    /// Set column visibility for a range of columns.
    pub fn set_col_visible(&mut self, min_val: i32, max_val: i32, visible: bool) -> Result<()> {
        if self.sheet_written {
            return Err(crate::errors::new_stream_set_row_order_error("SetColVisible").into());
        }
        if min_val < MIN_COLUMNS
            || min_val > MAX_COLUMNS
            || max_val < MIN_COLUMNS
            || max_val > MAX_COLUMNS
        {
            return Err(Box::new(ErrColumnNumber));
        }
        let (min_val, max_val) = if min_val > max_val {
            (max_val, min_val)
        } else {
            (min_val, max_val)
        };
        set_col_visible_ws(&mut self.worksheet, min_val, max_val, visible);
        Ok(())
    }

    /// Set the outline level for a column.
    pub fn set_col_outline_level(&mut self, col: i32, level: u8) -> Result<()> {
        if self.sheet_written {
            return Err(crate::errors::new_stream_set_row_order_error("SetColOutlineLevel").into());
        }
        if col < MIN_COLUMNS || col > MAX_COLUMNS {
            return Err(Box::new(ErrColumnNumber));
        }
        if level == 0 || level > 7 {
            return Err(Box::new(ErrOutlineLevel));
        }
        set_col_outline_level_ws(&mut self.worksheet, col, level);
        Ok(())
    }

    /// Set the style for a range of columns.
    pub fn set_col_style(&mut self, min_val: i32, max_val: i32, style_id: i32) -> Result<()> {
        if self.sheet_written {
            return Err(crate::errors::new_stream_set_row_order_error("SetColStyle").into());
        }
        if min_val < MIN_COLUMNS
            || min_val > MAX_COLUMNS
            || max_val < MIN_COLUMNS
            || max_val > MAX_COLUMNS
        {
            return Err(Box::new(ErrColumnNumber));
        }
        let (min_val, max_val) = if max_val < min_val {
            (max_val, min_val)
        } else {
            (min_val, max_val)
        };
        let styles = self.file.styles_reader()?;
        let style_count = styles.cell_xfs.as_ref().map(|x| x.xf.len()).unwrap_or(1) as i32;
        if style_id < 0 || style_id >= style_count {
            return Err(crate::errors::new_invalid_style_id_error(style_id).into());
        }
        set_col_style_ws(&mut self.worksheet, min_val, max_val, style_id);
        Ok(())
    }

    /// Set the width for a range of columns.
    pub fn set_col_width(&mut self, min_val: i32, max_val: i32, width: f64) -> Result<()> {
        if self.sheet_written {
            return Err(crate::errors::new_stream_set_row_order_error("SetColWidth").into());
        }
        if min_val < MIN_COLUMNS
            || min_val > MAX_COLUMNS
            || max_val < MIN_COLUMNS
            || max_val > MAX_COLUMNS
        {
            return Err(Box::new(ErrColumnNumber));
        }
        if width > MAX_COLUMN_WIDTH as f64 {
            return Err(Box::new(ErrColumnWidth));
        }
        let (min_val, max_val) = if min_val > max_val {
            (max_val, min_val)
        } else {
            (min_val, max_val)
        };
        set_col_width_ws(&mut self.worksheet, min_val, max_val, width);
        Ok(())
    }

    /// Insert a page break at the given cell.
    pub fn insert_page_break(&mut self, cell: &str) -> Result<()> {
        insert_page_break_ws(&mut self.worksheet, cell)
    }

    /// Set panes for the streamed sheet.
    pub fn set_panes(&mut self, panes: &Panes) -> Result<()> {
        if self.sheet_written {
            return Err(crate::errors::new_stream_set_row_order_error("SetPanes").into());
        }
        set_panes_ws(&mut self.worksheet, panes)
    }

    /// Merge cells in the streamed sheet.
    pub fn merge_cell(&mut self, top_left_cell: &str, bottom_right_cell: &str) -> Result<()> {
        cell_refs_to_coordinates(top_left_cell, bottom_right_cell).map_err(|e| io_err(e))?;
        self.merge_cells_count += 1;
        self.merge_cells.push_str(&format!(
            r#"<mergeCell ref="{}:{}"/>"#,
            top_left_cell, bottom_right_cell
        ));
        Ok(())
    }

    /// Flush the streaming writer.
    pub fn flush(&mut self) -> Result<()> {
        if self.flushed {
            return Ok(());
        }
        self.write_sheet_data()?;
        self.raw_data.write_str("</sheetData>")?;
        bulk_append_fields(&mut self.raw_data, &self.worksheet, 9, 16)?;

        if self.merge_cells_count > 0 {
            self.raw_data.write_str(&format!(
                r#"<mergeCells count="{}">{}</mergeCells>"#,
                self.merge_cells_count, self.merge_cells
            ))?;
        }

        bulk_append_fields(&mut self.raw_data, &self.worksheet, 18, 39)?;
        self.raw_data.write_str(&self.table_parts)?;
        bulk_append_fields(&mut self.raw_data, &self.worksheet, 41, 41)?;
        self.raw_data.write_str("</worksheet>")?;
        self.raw_data.flush()?;

        let tmp_path = self.raw_data.into_temp_file()?;
        let sheet_path = self
            .file
            .get_sheet_xml_path(&self.sheet)
            .unwrap_or_default();
        self.file.pkg.remove(&sheet_path);
        self.file.sheet.remove(&sheet_path);
        self.file.checked.remove(&sheet_path);
        if let Some(tmp_path) = tmp_path {
            let state = crate::file::StreamState {
                tmp_path,
                sheet_path: sheet_path.clone(),
            };
            self.file.streams.borrow_mut().insert(sheet_path, state);
        }
        self.flushed = true;
        Ok(())
    }

    fn write_sheet_data(&mut self) -> Result<()> {
        if !self.sheet_written {
            bulk_append_fields(&mut self.raw_data, &self.worksheet, 5, 6)?;
            if let Some(cols) = &self.worksheet.cols {
                self.raw_data.write_str("<cols>")?;
                for col in &cols.col {
                    self.raw_data
                        .write_str(&format!(r#"<col min="{}" max="{}""#, col.min, col.max))?;
                    if let Some(width) = col.width {
                        self.raw_data
                            .write_str(&format!(r#" width="{width}" customWidth="1""#))?;
                    }
                    if let Some(style) = col.style {
                        self.raw_data.write_str(&format!(r#" style="{style}""#))?;
                    }
                    if col.hidden.unwrap_or(false) {
                        self.raw_data.write_str(r#" hidden="1""#)?;
                    }
                    if let Some(level) = col.outline_level {
                        self.raw_data
                            .write_str(&format!(r#" outlineLevel="{level}""#))?;
                    }
                    if col.collapsed.unwrap_or(false) {
                        self.raw_data.write_str(r#" collapsed="1""#)?;
                    }
                    if col.best_fit.unwrap_or(false) {
                        self.raw_data.write_str(r#" bestFit="1""#)?;
                    }
                    self.raw_data.write_str("/>")?;
                }
                self.raw_data.write_str("</cols>")?;
            }
            self.raw_data.write_str("<sheetData>")?;
            self.sheet_written = true;
        }
        Ok(())
    }

    fn set_cell_val_func(&self, c: &mut XlsxC, value: CellValue) -> Result<()> {
        match value {
            CellValue::Int(n) => {
                c.t = None;
                c.v = Some(n.to_string());
            }
            CellValue::Float(n) => {
                set_cell_float(c, n, -1, 64);
            }
            CellValue::String(s) => {
                set_cell_value(c, &s);
            }
            CellValue::Bool(b) => {
                c.t = Some("b".to_string());
                c.v = Some(if b { "1".to_string() } else { "0".to_string() });
            }
            CellValue::Formula(f) => {
                set_cell_formula(c, &f);
            }
            CellValue::DateTime(dt) => {
                set_cell_time(self, c, dt, true)?;
            }
            CellValue::Date(d) => {
                set_cell_time(self, c, d.and_hms_opt(0, 0, 0).unwrap_or_default(), false)?;
            }
            CellValue::Time(t) => {
                c.t = None;
                c.v = Some(format!("{:.15}", date::time_to_excel_serial(t)));
                if c.s.unwrap_or(0) == 0 {
                    let style_id = self.file.new_style(&Style {
                        num_fmt: 21,
                        ..Default::default()
                    })?;
                    c.s = Some(style_id as i64);
                }
            }
            CellValue::Duration(d) => {
                c.t = None;
                c.v = Some(format!(
                    "{}",
                    format_float(d.as_nanos() as f64 / 86_400_000_000_000.0)
                ));
            }
            CellValue::RichText(runs) => {
                c.t = Some("inlineStr".to_string());
                c.is = Some(crate::cell::runs_to_xlsx_si(&runs));
            }
        }
        Ok(())
    }

    fn get_row_values(&mut self, h_row: i32, h_col: i32, v_col: i32) -> Result<Vec<String>> {
        let mut res = vec![String::new(); (v_col - h_col + 1) as usize];
        let sst = self.file.shared_strings_reader().ok();
        let reader = self.raw_data.reader()?;
        let mut reader = Reader::from_reader(BufReader::new(reader));
        let mut buf = Vec::new();
        let mut in_target = false;
        let mut in_cell = false;
        let mut in_v = false;
        let mut in_is = false;
        let mut cell_type = None::<String>;
        let mut current_col = 0;
        let mut current_val = String::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                    let local_name = e.local_name();
                    let name = local_name.as_ref();
                    if !in_target && name == b"row" {
                        if let Ok(Some(attr)) = e.try_get_attribute("r") {
                            if let Ok(r_str) = attr.decode_and_unescape_value(reader.decoder()) {
                                if r_str.parse::<i32>().unwrap_or(0) == h_row {
                                    in_target = true;
                                }
                            }
                        }
                    } else if in_target && name == b"c" {
                        in_cell = true;
                        current_col = 0;
                        cell_type = None;
                        current_val.clear();
                        if let Ok(Some(attr)) = e.try_get_attribute("r") {
                            if let Ok(r_str) = attr.decode_and_unescape_value(reader.decoder()) {
                                if let Ok((col, _)) = cell_name_to_coordinates(&r_str) {
                                    current_col = col;
                                }
                            }
                        }
                        if let Ok(Some(attr)) = e.try_get_attribute("t") {
                            if let Ok(t) = attr.decode_and_unescape_value(reader.decoder()) {
                                cell_type = Some(t.to_string());
                            }
                        }
                    } else if in_cell && name == b"v" {
                        in_v = true;
                        current_val.clear();
                    } else if in_cell && name == b"is" {
                        in_is = true;
                        current_val.clear();
                    }
                }
                Ok(Event::Text(e)) => {
                    if in_v || in_is {
                        if let Ok(text) = e.unescape() {
                            current_val.push_str(&text);
                        }
                    }
                }
                Ok(Event::End(e)) => {
                    let local_name = e.local_name();
                    let name = local_name.as_ref();
                    if name == b"row" && in_target {
                        break;
                    }
                    if in_target && name == b"v" {
                        in_v = false;
                        let val = if cell_type.as_deref() == Some("s") {
                            if let Ok(idx) = current_val.parse::<usize>() {
                                sst.as_ref()
                                    .and_then(|s| s.si.get(idx))
                                    .map(|si| extract_si_text(si))
                                    .unwrap_or_default()
                            } else {
                                String::new()
                            }
                        } else {
                            current_val.clone()
                        };
                        if current_col >= h_col && current_col <= v_col {
                            res[(current_col - h_col) as usize] = val;
                        }
                    } else if in_target && name == b"is" {
                        in_is = false;
                        if current_col >= h_col && current_col <= v_col {
                            res[(current_col - h_col) as usize] = current_val.clone();
                        }
                    } else if in_target && name == b"c" {
                        in_cell = false;
                        cell_type = None;
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(Box::new(e)),
                _ => {}
            }
            buf.clear();
        }
        Ok(res)
    }
}

impl<'a> Drop for StreamWriter<'a> {
    fn drop(&mut self) {
        if !self.flushed {
            let _ = self.flush();
        }
    }
}

impl File {
    /// Create a new stream writer for the given worksheet.
    pub fn new_stream_writer(&self, sheet: &str) -> Result<StreamWriter<'_>> {
        StreamWriter::new(self, sheet)
    }
}

impl RowOpts {
    fn marshal_attrs(&self) -> Result<String> {
        if self.height > MAX_ROW_HEIGHT as f64 {
            return Err(Box::new(ErrMaxRowHeight));
        }
        if self.outline_level > 7 {
            return Err(Box::new(ErrOutlineLevel));
        }
        let mut attrs = String::new();
        if self.style_id > 0 {
            attrs.push_str(&format!(r#" s="{}" customFormat="1""#, self.style_id));
        }
        if self.height > 0.0 {
            attrs.push_str(&format!(
                r#" ht="{}" customHeight="1""#,
                format_float(self.height)
            ));
        }
        if self.outline_level > 0 {
            attrs.push_str(&format!(r#" outlineLevel="{}""#, self.outline_level));
        }
        if self.hidden {
            attrs.push_str(r#" hidden="1""#);
        }
        Ok(attrs)
    }
}

fn set_cell_formula(c: &mut XlsxC, formula: &str) {
    if !formula.is_empty() {
        c.t = Some("str".to_string());
        c.f = Some(XlsxF {
            content: formula.to_string(),
            ..Default::default()
        });
    }
}

fn set_cell_time(
    sw: &StreamWriter<'_>,
    c: &mut XlsxC,
    val: NaiveDateTime,
    has_time: bool,
) -> Result<()> {
    let date1904 = sw
        .file
        .workbook_reader()?
        .workbook_pr
        .as_ref()
        .and_then(|p| p.date1904)
        .unwrap_or(false);
    let serial = date::datetime_to_excel_serial(val, date1904);
    let is_num = serial > 0.0;
    if is_num {
        c.v = Some(format!("{:.15}", serial));
        if c.s.unwrap_or(0) == 0 {
            let num_fmt = if has_time && val.date() != NaiveDate::default() {
                22
            } else if has_time {
                21
            } else {
                14
            };
            let style_id = sw.file.new_style(&Style {
                num_fmt,
                ..Default::default()
            })?;
            c.s = Some(style_id as i64);
        }
    } else {
        c.v = Some(val.format("%Y-%m-%dT%H:%M:%S").to_string());
    }
    Ok(())
}

fn set_cell_float(c: &mut XlsxC, value: f64, precision: i32, _bit_size: u32) {
    if value.is_nan() || value.is_infinite() {
        c.t = Some("inlineStr".to_string());
        c.v = Some(String::new());
        c.is = Some(XlsxSi {
            t: Some(XlsxT {
                space: None,
                val: value.to_string(),
            }),
            ..Default::default()
        });
        return;
    }
    c.t = None;
    c.v = Some(if precision < 0 {
        format_float(value)
    } else {
        format!("{value:.precision$}", precision = precision as usize)
    });
}

fn set_cell_value(c: &mut XlsxC, val: &str) {
    if c.f.is_some() {
        c.t = Some("str".to_string());
        c.v = Some(val.to_string());
        if needs_space_preserve(val) {
            c.xml_space = Some("preserve".to_string());
        }
    } else {
        c.t = Some("inlineStr".to_string());
        c.v = Some(String::new());
        c.is = Some(XlsxSi {
            t: Some(XlsxT {
                space: if needs_space_preserve(val) {
                    Some("preserve".to_string())
                } else {
                    None
                },
                val: val.to_string(),
            }),
            ..Default::default()
        });
    }
}

fn write_cell(buf: &mut BufferedWriter, c: &XlsxC) -> Result<()> {
    buf.write_str("<c")?;
    if let Some(space) = &c.xml_space {
        buf.write_str(&format!(r#" xml:space="{space}""#))?;
    }
    if let Some(r) = &c.r {
        buf.write_str(&format!(r#" r="{r}""#))?;
    }
    if let Some(s) = c.s {
        if s != 0 {
            buf.write_str(&format!(r#" s="{s}""#))?;
        }
    }
    if let Some(t) = &c.t {
        buf.write_str(&format!(r#" t="{t}""#))?;
    }
    buf.write_str(">")?;

    if let Some(f) = &c.f {
        buf.write_str("<f>")?;
        buf.write_str(&escape_xml(&f.content))?;
        buf.write_str("</f>")?;
    }
    if let Some(v) = &c.v {
        if !v.is_empty() {
            buf.write_str("<v>")?;
            buf.write_str(&escape_xml(v))?;
            buf.write_str("</v>")?;
        }
    }
    if let Some(is) = &c.is {
        if !is.r.is_empty() {
            let runs = xml_to_string(&is.r)?;
            buf.write_str("<is>")?;
            buf.write_str(&runs)?;
            buf.write_str("</is>")?;
        } else if let Some(t) = &is.t {
            buf.write_str("<is><t")?;
            if let Some(space) = &t.space {
                buf.write_str(&format!(r#" xml:space="{space}""#))?;
            }
            buf.write_str(">")?;
            buf.write_str(&escape_xml(&t.val))?;
            buf.write_str("</t></is>")?;
        }
    }
    buf.write_str("</c>")?;
    Ok(())
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

fn bulk_append_fields<W: Write>(
    w: &mut W,
    ws: &XlsxWorksheet,
    from: usize,
    to: usize,
) -> Result<()> {
    for i in from..=to {
        match i {
            3 => append_field_option(w, &ws.sheet_pr)?,
            4 => append_field_option(w, &ws.dimension)?,
            5 => append_field_option(w, &ws.sheet_views)?,
            6 => append_field_option(w, &ws.sheet_format_pr)?,
            9 => append_field_option(w, &ws.sheet_calc_pr)?,
            10 => append_field_option(w, &ws.sheet_protection)?,
            11 => append_field_option(w, &ws.protected_ranges)?,
            12 => append_field_option(w, &ws.scenarios)?,
            13 => append_field_option(w, &ws.auto_filter)?,
            14 => append_field_option(w, &ws.sort_state)?,
            15 => append_field_option(w, &ws.data_consolidate)?,
            16 => append_field_option(w, &ws.custom_sheet_views)?,
            18 => append_field_option(w, &ws.phonetic_pr)?,
            19 => append_field_vec(w, &ws.conditional_formatting)?,
            20 => append_field_option(w, &ws.data_validations)?,
            21 => append_field_option(w, &ws.hyperlinks)?,
            22 => append_field_option(w, &ws.print_options)?,
            23 => append_field_option(w, &ws.page_margins)?,
            24 => append_field_option(w, &ws.page_setup)?,
            25 => append_field_option(w, &ws.header_footer)?,
            26 => append_field_option(w, &ws.row_breaks)?,
            27 => append_field_option(w, &ws.col_breaks)?,
            28 => append_field_option(w, &ws.custom_properties)?,
            29 => append_field_option(w, &ws.cell_watches)?,
            30 => append_field_option(w, &ws.ignored_errors)?,
            31 => append_field_option(w, &ws.smart_tags)?,
            32 => append_field_option(w, &ws.drawing)?,
            33 => append_field_option(w, &ws.legacy_drawing)?,
            34 => append_field_option(w, &ws.legacy_drawing_hf)?,
            35 => append_field_option(w, &ws.drawing_hf)?,
            36 => append_field_option(w, &ws.picture)?,
            37 => append_field_option(w, &ws.ole_objects)?,
            38 => append_field_option(w, &ws.controls)?,
            39 => append_field_option(w, &ws.web_publish_items)?,
            41 => append_field_option(w, &ws.table_parts)?,
            _ => {}
        }
    }
    Ok(())
}

fn append_field_option<W: Write, T: serde::Serialize>(w: &mut W, opt: &Option<T>) -> Result<()> {
    if let Some(v) = opt {
        w.write_all(xml_to_string(v)?.as_bytes())?;
    }
    Ok(())
}

fn append_field_vec<W: Write, T: serde::Serialize>(w: &mut W, vec: &[T]) -> Result<()> {
    if !vec.is_empty() {
        for v in vec {
            w.write_all(xml_to_string(v)?.as_bytes())?;
        }
    }
    Ok(())
}

// ------------------------------------------------------------------
// Column helpers
// ------------------------------------------------------------------

fn set_col_visible_ws(ws: &mut XlsxWorksheet, min_val: i32, max_val: i32, visible: bool) {
    let col_data = XlsxCol {
        min: min_val as i64,
        max: max_val as i64,
        width: Some(DEFAULT_COL_WIDTH),
        hidden: Some(!visible),
        custom_width: Some(true),
        ..Default::default()
    };
    ws.cols = Some(XlsxCols {
        col: flat_cols(
            col_data,
            ws.cols.as_ref().map(|c| c.col.as_slice()).unwrap_or(&[]),
            |fc, c| {
                fc.best_fit = c.best_fit;
                fc.collapsed = c.collapsed;
                fc.custom_width = c.custom_width;
                fc.outline_level = c.outline_level;
                fc.phonetic = c.phonetic;
                fc.style = c.style;
                fc.width = c.width;
            },
        ),
    });
}

fn set_col_outline_level_ws(ws: &mut XlsxWorksheet, col_num: i32, level: u8) {
    let col_data = XlsxCol {
        min: col_num as i64,
        max: col_num as i64,
        outline_level: Some(level),
        custom_width: Some(true),
        ..Default::default()
    };
    ws.cols = Some(XlsxCols {
        col: flat_cols(
            col_data,
            ws.cols.as_ref().map(|c| c.col.as_slice()).unwrap_or(&[]),
            |fc, c| {
                fc.best_fit = c.best_fit;
                fc.collapsed = c.collapsed;
                fc.custom_width = c.custom_width;
                fc.hidden = c.hidden;
                fc.phonetic = c.phonetic;
                fc.style = c.style;
                fc.width = c.width;
            },
        ),
    });
}

fn set_col_style_ws(ws: &mut XlsxWorksheet, min_val: i32, max_val: i32, style_id: i32) {
    let width = ws
        .sheet_format_pr
        .as_ref()
        .and_then(|f| f.default_col_width)
        .unwrap_or(DEFAULT_COL_WIDTH);
    let col_data = XlsxCol {
        min: min_val as i64,
        max: max_val as i64,
        width: Some(width),
        style: Some(style_id as i64),
        ..Default::default()
    };
    ws.cols = Some(XlsxCols {
        col: flat_cols(
            col_data,
            ws.cols.as_ref().map(|c| c.col.as_slice()).unwrap_or(&[]),
            |fc, c| {
                fc.best_fit = c.best_fit;
                fc.collapsed = c.collapsed;
                fc.custom_width = c.custom_width;
                fc.hidden = c.hidden;
                fc.outline_level = c.outline_level;
                fc.phonetic = c.phonetic;
                fc.width = c.width;
            },
        ),
    });
}

fn set_col_width_ws(ws: &mut XlsxWorksheet, min_val: i32, max_val: i32, width: f64) {
    let col_data = XlsxCol {
        min: min_val as i64,
        max: max_val as i64,
        width: Some(width),
        custom_width: Some(true),
        ..Default::default()
    };
    ws.cols = Some(XlsxCols {
        col: flat_cols(
            col_data,
            ws.cols.as_ref().map(|c| c.col.as_slice()).unwrap_or(&[]),
            |fc, c| {
                fc.best_fit = c.best_fit;
                fc.collapsed = c.collapsed;
                fc.hidden = c.hidden;
                fc.outline_level = c.outline_level;
                fc.phonetic = c.phonetic;
                fc.style = c.style;
            },
        ),
    });
}

fn flat_cols<F>(col: XlsxCol, cols: &[XlsxCol], mut replacer: F) -> Vec<XlsxCol>
where
    F: FnMut(&mut XlsxCol, &XlsxCol),
{
    let mut fc: HashMap<i64, XlsxCol> = HashMap::new();
    for i in col.min..=col.max {
        let mut c = col.clone();
        c.min = i;
        c.max = i;
        fc.insert(i, c);
    }
    for column in cols {
        for i in column.min..=column.max {
            if let Some(existing) = fc.get_mut(&i) {
                replacer(existing, column);
            } else {
                let mut c = column.clone();
                c.min = i;
                c.max = i;
                fc.insert(i, c);
            }
        }
    }
    let mut result: Vec<XlsxCol> = fc.into_values().collect();
    result.sort_by_key(|c| c.min);
    result
}

// ------------------------------------------------------------------
// Panes and page breaks
// ------------------------------------------------------------------

fn set_panes_ws(ws: &mut XlsxWorksheet, panes: &Panes) -> Result<()> {
    if panes.selection.is_empty() && !panes.freeze && !panes.split {
        if let Some(views) = ws.sheet_views.as_mut() {
            if let Some(view) = views.sheet_view.last_mut() {
                view.pane = None;
            }
        }
        return Ok(());
    }

    let pane = XlsxPane {
        active_pane: Some(panes.active_pane.clone()).filter(|s| !s.is_empty()),
        top_left_cell: Some(panes.top_left_cell.clone()).filter(|s| !s.is_empty()),
        x_split: Some(panes.x_split as f64),
        y_split: Some(panes.y_split as f64),
        state: if panes.freeze {
            Some("frozen".to_string())
        } else {
            None
        },
    };

    if ws.sheet_views.is_none() {
        ws.sheet_views = Some(XlsxSheetViews::default());
    }
    let views = ws.sheet_views.as_mut().unwrap();
    if views.sheet_view.is_empty() {
        views
            .sheet_view
            .push(crate::xml::worksheet::XlsxSheetView::default());
    }
    let view = views.sheet_view.last_mut().unwrap();
    view.pane = Some(pane);

    let selections: Vec<XlsxSelection> = panes
        .selection
        .iter()
        .map(|s| XlsxSelection {
            active_cell: Some(s.active_cell.clone()).filter(|s| !s.is_empty()),
            active_cell_id: None,
            pane: Some(s.pane.clone()).filter(|s| !s.is_empty()),
            sqref: Some(s.sqref.clone()).filter(|s| !s.is_empty()),
        })
        .collect();
    view.selection = selections;
    Ok(())
}

fn insert_page_break_ws(ws: &mut XlsxWorksheet, cell: &str) -> Result<()> {
    let (mut col, mut row) = cell_name_to_coordinates(cell).map_err(|e| io_err(e))?;
    col -= 1;
    row -= 1;
    if col == 0 && row == 0 {
        return Ok(());
    }
    if ws.row_breaks.is_none() {
        ws.row_breaks = Some(XlsxRowBreaks {
            breaks: XlsxBreaks::default(),
        });
    }
    if ws.col_breaks.is_none() {
        ws.col_breaks = Some(XlsxColBreaks {
            breaks: XlsxBreaks::default(),
        });
    }

    let row_breaks = ws.row_breaks.as_mut().unwrap();
    let col_breaks = ws.col_breaks.as_mut().unwrap();

    let row_exists = row_breaks
        .breaks
        .brk
        .iter()
        .any(|b| b.id == Some(row as i64));
    let col_exists = col_breaks
        .breaks
        .brk
        .iter()
        .any(|b| b.id == Some(col as i64));

    if row != 0 && !row_exists {
        row_breaks.breaks.brk.push(XlsxBrk {
            id: Some(row as i64),
            max: Some(MAX_COLUMNS as i64 - 1),
            min: None,
            man: Some(true),
            pt: None,
        });
        row_breaks.breaks.manual_break_count =
            Some(row_breaks.breaks.manual_break_count.unwrap_or(0) + 1);
    }
    if col != 0 && !col_exists {
        col_breaks.breaks.brk.push(XlsxBrk {
            id: Some(col as i64),
            max: Some(TOTAL_ROWS as i64 - 1),
            min: None,
            man: Some(true),
            pt: None,
        });
        col_breaks.breaks.manual_break_count =
            Some(col_breaks.breaks.manual_break_count.unwrap_or(0) + 1);
    }
    row_breaks.breaks.count = Some(row_breaks.breaks.brk.len() as i64);
    col_breaks.breaks.count = Some(col_breaks.breaks.brk.len() as i64);
    Ok(())
}

// ------------------------------------------------------------------
// Table helpers
// ------------------------------------------------------------------

fn extract_si_text(si: &XlsxSi) -> String {
    if let Some(t) = &si.t {
        return t.val.clone();
    }
    si.r.iter()
        .filter_map(|r| r.t.as_ref().map(|t| t.val.clone()))
        .collect()
}

fn parse_table_options(opts: &TableOptions) -> Result<TableOptions> {
    let mut options = opts.clone();
    if options.show_row_stripes.is_none() {
        options.show_row_stripes = Some(true);
    }
    if !options.name.is_empty() {
        check_defined_name(&options.name)?;
    }
    Ok(options)
}

fn check_defined_name(name: &str) -> Result<()> {
    if name.len() > 255 {
        return Err(Box::new(crate::errors::ErrNameLength));
    }
    let mut chars = name.chars();
    if let Some(first) = chars.next() {
        if !first.is_ascii_alphabetic() && first != '_' {
            return Err(crate::errors::new_invalid_name_error(name).into());
        }
        for c in chars {
            if !c.is_ascii_alphanumeric() && c != '_' && c != '.' {
                return Err(crate::errors::new_invalid_name_error(name).into());
            }
        }
    }
    Ok(())
}

// ------------------------------------------------------------------
// Buffered writer with optional temp-file backing
// ------------------------------------------------------------------

const TEMPLATE_NAMESPACE_ID_MAP: &str = crate::templates::TEMPLATE_NAMESPACE_ID_MAP;

#[derive(Debug)]
struct BufferedWriter {
    tmp_dir: String,
    tmp: Option<FsFile>,
    tmp_path: Option<PathBuf>,
    buf: Vec<u8>,
}

impl BufferedWriter {
    fn new(tmp_dir: String) -> Self {
        Self {
            tmp_dir,
            tmp: None,
            tmp_path: None,
            buf: Vec::new(),
        }
    }

    fn write(&mut self, p: &[u8]) -> io::Result<()> {
        self.buf.extend_from_slice(p);
        Ok(())
    }

    fn write_str(&mut self, s: &str) -> io::Result<()> {
        self.write(s.as_bytes())
    }

    fn reader(&mut self) -> io::Result<Box<dyn Read + '_>> {
        if self.tmp.is_none() {
            return Ok(Box::new(Cursor::new(&self.buf)));
        }
        self.flush()?;
        let mut content = Vec::new();
        if let Some(path) = &self.tmp_path {
            let mut f = FsFile::open(path)?;
            f.read_to_end(&mut content)?;
        }
        Ok(Box::new(Cursor::new(content)))
    }

    fn sync(&mut self) -> io::Result<()> {
        if self.buf.len() < STREAM_CHUNK_SIZE as usize {
            return Ok(());
        }
        if self.tmp.is_none() {
            match create_temp(&self.tmp_dir) {
                Ok((f, p)) => {
                    self.tmp = Some(f);
                    self.tmp_path = Some(p);
                }
                Err(_) => return Ok(()),
            }
        }
        self.flush()
    }

    fn flush(&mut self) -> io::Result<()> {
        if let Some(tmp) = &mut self.tmp {
            tmp.write_all(&self.buf)?;
            tmp.sync_all()?;
            self.buf.clear();
        }
        Ok(())
    }

    fn close(&mut self) -> io::Result<()> {
        self.buf.clear();
        if let Some(tmp) = self.tmp.take() {
            drop(tmp);
            if let Some(path) = self.tmp_path.take() {
                let _ = fs::remove_file(path);
            }
        }
        Ok(())
    }

    /// Flush any buffered data, close the temporary file handle, and return
    /// the path to the backing temporary file. The caller is responsible for
    /// deleting the file once it is no longer needed.
    fn into_temp_file(&mut self) -> io::Result<Option<PathBuf>> {
        self.flush()?;
        if let Some(tmp) = self.tmp.take() {
            self.buf.clear();
            drop(tmp);
            return Ok(self.tmp_path.take());
        }
        if !self.buf.is_empty() {
            let (mut f, path) = create_temp(&self.tmp_dir)?;
            f.write_all(&self.buf)?;
            f.sync_all()?;
            drop(f);
            self.buf.clear();
            return Ok(Some(path));
        }
        self.buf.clear();
        Ok(None)
    }
}

impl Write for BufferedWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.buf.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.flush()
    }
}

impl Drop for BufferedWriter {
    fn drop(&mut self) {
        let _ = self.close();
    }
}

fn create_temp(tmp_dir: &str) -> io::Result<(FsFile, PathBuf)> {
    const MAX_RETRIES: usize = 100;

    let dir = if tmp_dir.is_empty() {
        std::env::temp_dir()
    } else {
        PathBuf::from(tmp_dir)
    };
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let thread_id = format!("{:?}", thread::current().id())
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect::<String>();

    for _ in 0..MAX_RETRIES {
        let path = dir.join(format!(
            "excelize-{}-{}-{}-{}.xml",
            now.as_secs(),
            now.subsec_nanos(),
            thread_id,
            rand::random::<u32>()
        ));
        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
        {
            Ok(file) => return Ok((file, path)),
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(e),
        }
    }
    Err(io::Error::new(
        io::ErrorKind::Other,
        "failed to create a unique temporary file after 100 attempts",
    ))
}

// ------------------------------------------------------------------
// XML / string helpers
// ------------------------------------------------------------------

fn escape_xml(s: &str) -> String {
    escape(s).into_owned()
}

fn io_err(s: String) -> Box<dyn std::error::Error + Send + Sync> {
    Box::new(io::Error::new(io::ErrorKind::InvalidData, s))
}

fn needs_space_preserve(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    let prefix = s.as_bytes().first().copied().unwrap_or(0);
    let suffix = s.as_bytes().last().copied().unwrap_or(0);
    [b' ', b'\t', b'\n', b'\r'].contains(&prefix) || [b' ', b'\t', b'\n', b'\r'].contains(&suffix)
}

fn format_float(value: f64) -> String {
    let s = format!("{value:.15}");
    if s.contains('.') {
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    } else {
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::options::Options;
    use chrono::{NaiveDate, NaiveTime};

    #[test]
    fn stream_writer_basic_round_trip() {
        let mut f = File::new_with_options(Options::default());
        let mut sw = f.new_stream_writer("Sheet1").unwrap();

        sw.set_row(
            "A1",
            &[
                &CellValue::String("Data".to_string()),
                &CellValue::Int(42),
                &CellValue::Float(3.14),
                &CellValue::Bool(true),
            ],
            None,
        )
        .unwrap();
        sw.set_row(
            "A2",
            &[&CellValue::Int(1), &CellValue::Int(2)],
            Some(RowOpts::default()),
        )
        .unwrap();
        sw.flush().unwrap();
        drop(sw);

        let tmp = std::env::temp_dir().join("excelize_stream_basic_test.xlsx");
        f.save_as(tmp.to_str().unwrap()).unwrap();

        let f2 = File::open_file(tmp.to_str().unwrap(), Options::default()).unwrap();
        assert_eq!(f2.get_cell_value("Sheet1", "A1").unwrap(), "Data");
        assert_eq!(f2.get_cell_value("Sheet1", "B1").unwrap(), "42");
        assert_eq!(f2.get_cell_value("Sheet1", "C1").unwrap(), "3.14");
        assert_eq!(f2.get_cell_value("Sheet1", "D1").unwrap(), "TRUE");
        assert_eq!(f2.get_cell_value("Sheet1", "A2").unwrap(), "1");
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn stream_writer_cell_with_style_and_formula() {
        let mut f = File::new_with_options(Options::default());
        let style_id = f
            .new_style(&Style {
                font: Some(crate::styles::Font {
                    color: Some("777777".to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            })
            .unwrap();

        let mut sw = f.new_stream_writer("Sheet1").unwrap();
        sw.set_row(
            "A1",
            &[
                &Cell {
                    style_id,
                    value: Some(CellValue::String("Styled".to_string())),
                    ..Default::default()
                },
                &Cell {
                    formula: "SUM(A1,A1)".to_string(),
                    value: Some(CellValue::String("formula value".to_string())),
                    ..Default::default()
                },
            ],
            None,
        )
        .unwrap();
        sw.flush().unwrap();
        drop(sw);

        let tmp = std::env::temp_dir().join("excelize_stream_formula_test.xlsx");
        f.save_as(tmp.to_str().unwrap()).unwrap();
        let f2 = File::open_file(tmp.to_str().unwrap(), Options::default()).unwrap();
        assert_eq!(f2.get_cell_value("Sheet1", "A1").unwrap(), "Styled");
        assert_eq!(f2.get_cell_value("Sheet1", "B1").unwrap(), "formula value");
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn stream_writer_row_and_col_options() {
        let mut f = File::new_with_options(Options::default());
        let mut sw = f.new_stream_writer("Sheet1").unwrap();
        sw.set_col_width(1, 3, 20.0).unwrap();
        sw.set_col_visible(2, 2, false).unwrap();
        sw.set_row(
            "A1",
            &[&CellValue::Int(1)],
            Some(RowOpts {
                height: 30.0,
                hidden: false,
                style_id: 0,
                outline_level: 2,
            }),
        )
        .unwrap();
        sw.flush().unwrap();
        drop(sw);

        let tmp = std::env::temp_dir().join("excelize_stream_options_test.xlsx");
        f.save_as(tmp.to_str().unwrap()).unwrap();
        let f2 = File::open_file(tmp.to_str().unwrap(), Options::default()).unwrap();
        assert_eq!(f2.get_row_height("Sheet1", 1).unwrap(), 30.0);
        assert!(!f2.get_col_visible("Sheet1", "B").unwrap());
        assert_eq!(f2.get_col_width("Sheet1", "A").unwrap(), 20.0);
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn stream_writer_merge_cell() {
        let mut f = File::new_with_options(Options::default());
        let mut sw = f.new_stream_writer("Sheet1").unwrap();
        sw.set_row("A1", &[&CellValue::Int(1)], None).unwrap();
        sw.merge_cell("A1", "B2").unwrap();
        sw.flush().unwrap();
        drop(sw);

        let tmp = std::env::temp_dir().join("excelize_stream_merge_test.xlsx");
        f.save_as(tmp.to_str().unwrap()).unwrap();
        let f2 = File::open_file(tmp.to_str().unwrap(), Options::default()).unwrap();
        let ws = f2.work_sheet_reader("Sheet1").unwrap();
        assert!(ws.merge_cells.is_some());
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn stream_writer_date_and_time() {
        let mut f = File::new_with_options(Options::default());
        let mut sw = f.new_stream_writer("Sheet1").unwrap();
        let dt = NaiveDate::from_ymd_opt(2024, 7, 13)
            .unwrap()
            .and_hms_opt(12, 30, 0)
            .unwrap();
        sw.set_row(
            "A1",
            &[
                &CellValue::DateTime(dt),
                &CellValue::Time(NaiveTime::from_hms_opt(14, 30, 0).unwrap()),
            ],
            None,
        )
        .unwrap();
        sw.flush().unwrap();
        drop(sw);

        let tmp = std::env::temp_dir().join("excelize_stream_date_test.xlsx");
        f.save_as(tmp.to_str().unwrap()).unwrap();
        let f2 = File::open_file(tmp.to_str().unwrap(), Options::default()).unwrap();
        assert_eq!(f2.get_cell_value("Sheet1", "A1").unwrap(), "7/13/24 12:30");
        assert_eq!(f2.get_cell_value("Sheet1", "B1").unwrap(), "14:30:00");
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn stream_writer_row_order_error() {
        let f = File::new_with_options(Options::default());
        let mut sw = f.new_stream_writer("Sheet1").unwrap();
        sw.set_row("A2", &[&CellValue::Int(1)], None).unwrap();
        assert!(sw.set_row("A1", &[&CellValue::Int(1)], None).is_err());
    }

    #[test]
    fn stream_writer_invalid_column_errors() {
        let f = File::new_with_options(Options::default());
        let mut sw = f.new_stream_writer("Sheet1").unwrap();
        assert!(sw.set_col_width(0, 1, 10.0).is_err());
        assert!(sw.set_col_width(1, 1, 300.0).is_err());
        assert!(sw.set_col_outline_level(1, 8).is_err());
    }

    #[test]
    fn stream_writer_mixed_types_and_nil_values() {
        let mut f = File::new_with_options(Options::default());
        let style_id = f
            .new_style(&Style {
                font: Some(crate::styles::Font {
                    color: Some("777777".to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            })
            .unwrap();

        let mut sw = f.new_stream_writer("Sheet1").unwrap();
        sw.set_row(
            "A1",
            &[
                &1i32 as &dyn StreamCellValue,
                &2.5f64,
                &"hello",
                &true,
                &None::<CellValue>,
                &Cell {
                    style_id,
                    value: None,
                    ..Default::default()
                },
                &Cell {
                    value: Some(CellValue::String("with value".to_string())),
                    ..Default::default()
                },
            ],
            None,
        )
        .unwrap();
        sw.flush().unwrap();
        drop(sw);

        let tmp = std::env::temp_dir().join("excelize_stream_mixed_test.xlsx");
        f.save_as(tmp.to_str().unwrap()).unwrap();
        let f2 = File::open_file(tmp.to_str().unwrap(), Options::default()).unwrap();
        assert_eq!(f2.get_cell_value("Sheet1", "A1").unwrap(), "1");
        assert_eq!(f2.get_cell_value("Sheet1", "B1").unwrap(), "2.5");
        assert_eq!(f2.get_cell_value("Sheet1", "C1").unwrap(), "hello");
        assert_eq!(f2.get_cell_value("Sheet1", "D1").unwrap(), "TRUE");
        assert_eq!(f2.get_cell_value("Sheet1", "E1").unwrap(), "");
        assert_eq!(f2.get_cell_value("Sheet1", "F1").unwrap(), "");
        assert_eq!(f2.get_cell_value("Sheet1", "G1").unwrap(), "with value");
        assert_eq!(f2.get_cell_style("Sheet1", "F1").unwrap(), style_id);
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn stream_writer_empty_string_is_written() {
        let mut f = File::new_with_options(Options::default());
        let mut sw = f.new_stream_writer("Sheet1").unwrap();
        sw.set_row(
            "A1",
            &[&CellValue::String("".to_string()) as &dyn StreamCellValue],
            None,
        )
        .unwrap();
        sw.flush().unwrap();
        drop(sw);

        let tmp = std::env::temp_dir().join("excelize_stream_empty_str_test.xlsx");
        f.save_as(tmp.to_str().unwrap()).unwrap();
        let f2 = File::open_file(tmp.to_str().unwrap(), Options::default()).unwrap();
        assert_eq!(f2.get_cell_value("Sheet1", "A1").unwrap(), "");
        let _ = std::fs::remove_file(&tmp);
    }
}
