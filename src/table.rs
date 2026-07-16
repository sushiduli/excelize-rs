//! Table and AutoFilter support.
//!
//! Ported from Go `table.go`.

use std::collections::HashMap;
use std::sync::LazyLock;

use quick_xml::de::from_reader as xml_from_reader;
use quick_xml::se::to_string as xml_to_string;
use regex::Regex;

use crate::constants::{
    BUILT_IN_DEFINED_NAME_FILTER_DATABASE, CONTENT_TYPE_SPREADSHEET_ML_TABLE, MAX_FIELD_LENGTH,
    NAMESPACE_SPREADSHEET, SOURCE_RELATIONSHIP, SOURCE_RELATIONSHIP_TABLE,
    SUPPORTED_DEFINED_NAME_AFTER_START_CHAR_CODE_RANGE,
    SUPPORTED_DEFINED_NAME_AT_START_CHAR_CODE_RANGE,
};
use crate::errors::Result;
use crate::errors::{
    ErrExistsTableName, ErrNameLength, ErrParameterInvalid, ErrSheetNotExist,
    new_invalid_auto_filter_column_error, new_invalid_auto_filter_exp_error,
    new_invalid_auto_filter_operator_error, new_invalid_name_error, new_no_exist_table_error,
    new_unknown_filter_token_error,
};
use crate::file::File;
use crate::lib_util::{
    bool_ptr, column_name_to_number, coordinates_to_cell_name, coordinates_to_range_ref,
    count_utf16_string, in_str_slice, range_ref_to_coordinates, sort_coordinates,
};
use crate::xml::table::{
    AutoFilterOptions, Table, XlsxAutoFilter, XlsxCustomFilter, XlsxCustomFilters, XlsxFilter,
    XlsxFilterColumn, XlsxFilters, XlsxTable, XlsxTableColumn, XlsxTableColumns,
    XlsxTableStyleInfo,
};
use crate::xml::workbook::XlsxDefinedName;
use crate::xml::worksheet::{XlsxTablePart, XlsxTableParts};

static EXPRESSION_FORMAT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#""(?:[^"]|"")*"|\S+"#).unwrap());
static CONDITION_FORMAT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(or|\|\|)").unwrap());
static BLANK_FORMAT: LazyLock<Regex> = LazyLock::new(|| Regex::new("blanks|nonblanks").unwrap());
static MATCH_FORMAT: LazyLock<Regex> = LazyLock::new(|| Regex::new("[*?]").unwrap());

// ------------------------------------------------------------------
// Public API
// ------------------------------------------------------------------

impl File {
    /// Add a table to a worksheet.
    pub fn add_table(&self, sheet: &str, table: Option<&Table>) -> Result<()> {
        let options = parse_table_options(table)?;
        if options.range.is_empty() {
            return Err(Box::new(ErrParameterInvalid));
        }
        let mut exist = false;
        for entry in self.pkg.iter() {
            let k = entry.key();
            if k.contains("xl/tables/table") {
                let data = crate::file::namespace_strict_to_transitional(entry.value());
                if let Ok(t) = xml_from_reader::<_, XlsxTable>(data.as_slice()) {
                    if t.name == options.name {
                        exist = true;
                        break;
                    }
                }
            }
        }
        if exist {
            return Err(Box::new(ErrExistsTableName));
        }

        let mut coordinates = range_ref_to_coordinates(&options.range)?;
        sort_coordinates(&mut coordinates)?;
        let table_id = self.count_tables() + 1;
        let sheet_relationships_table_xml = format!("../tables/table{table_id}.xml");
        let table_xml = sheet_relationships_table_xml.replace("..", "xl");
        let sheet_xml_path = self.get_sheet_xml_path(sheet).unwrap_or_default();
        let sheet_rels = format!(
            "xl/worksheets/_rels/{}.rels",
            sheet_xml_path.trim_start_matches("xl/worksheets/")
        );
        let r_id = self.add_rels(
            &sheet_rels,
            SOURCE_RELATIONSHIP_TABLE,
            &sheet_relationships_table_xml,
            "",
        );
        self.add_sheet_table(sheet, r_id)?;
        self.add_sheet_name_space(sheet, SOURCE_RELATIONSHIP);
        self.clear_calc_cache();
        self.add_table_internal(
            sheet,
            &table_xml,
            coordinates[0],
            coordinates[1],
            coordinates[2],
            coordinates[3],
            table_id,
            &options,
        )?;
        self.add_content_type_part(table_id, "table")
    }

    /// Get the tables in a worksheet.
    pub fn get_tables(&self, sheet: &str) -> Result<Vec<Table>> {
        let mut tables = Vec::new();
        let ws = self.work_sheet_reader(sheet)?;
        let table_parts = match ws.table_parts {
            Some(tp) => tp,
            None => return Ok(tables),
        };
        for tbl in table_parts.table_part {
            let target =
                self.get_sheet_relationships_target_by_id(sheet, tbl.rid.as_deref().unwrap_or(""));
            let table_xml = target.replace("..", "xl");
            let content = self.read_xml(&table_xml);
            if content.is_empty() {
                continue;
            }
            let data = crate::file::namespace_strict_to_transitional(&content);
            let t: XlsxTable = xml_from_reader(data.as_slice())?;
            let mut table = Table {
                r_id: tbl.rid.unwrap_or_default(),
                t_id: t.id,
                table_xml,
                range: t.r#ref,
                name: t.name,
                ..Default::default()
            };
            if let Some(info) = t.table_style_info {
                table.style_name = info.name.unwrap_or_default();
                table.show_column_stripes = info.show_column_stripes;
                table.show_first_column = info.show_first_column;
                table.show_last_column = info.show_last_column;
                table.show_row_stripes = Some(info.show_row_stripes);
            }
            tables.push(table);
        }
        Ok(tables)
    }

    /// Delete a table by name.
    pub fn delete_table(&self, name: &str) -> Result<()> {
        check_defined_name(name)?;
        let tbls = self.get_tables_for_workbook()?;
        self.clear_calc_cache();
        for (sheet, tables) in tbls {
            for table in tables {
                if table.name != name {
                    continue;
                }
                let mut ws = self.work_sheet_reader(&sheet)?;
                let mut found = false;
                if let Some(ref mut tp) = ws.table_parts {
                    let mut i = 0;
                    while i < tp.table_part.len() {
                        if tp.table_part[i].rid.as_deref() == Some(table.r_id.as_str()) {
                            tp.table_part.remove(i);
                            found = true;
                            break;
                        }
                        i += 1;
                    }
                    if found {
                        self.pkg.remove(&table.table_xml);
                        self.remove_content_types_part(
                            CONTENT_TYPE_SPREADSHEET_ML_TABLE,
                            &format!("/{}", table.table_xml),
                        )?;
                        self.delete_sheet_relationships(&sheet, &table.r_id);
                        tp.count = Some(tp.table_part.len() as i64);
                        if tp.count == Some(0) {
                            ws.table_parts = None;
                        }
                    }
                }
                if found {
                    if let Some(path) = self.get_sheet_xml_path(&sheet) {
                        self.sheet.insert(path, ws);
                    }
                    return Ok(());
                }
            }
        }
        Err(new_no_exist_table_error(name).into())
    }

    /// Apply an AutoFilter to a worksheet range.
    pub fn auto_filter(
        &self,
        sheet: &str,
        range_ref: &str,
        opts: &[AutoFilterOptions],
    ) -> Result<()> {
        let mut coordinates = range_ref_to_coordinates(range_ref)?;
        sort_coordinates(&mut coordinates)?;
        let ref_str = coordinates_to_range_ref(&coordinates, true)?;

        let mut wb = self.workbook_reader()?;
        let sheet_id = self.get_sheet_index(sheet)?;
        let filter_range = format!("'{sheet}'!{ref_str}");
        let d = XlsxDefinedName {
            name: Some(BUILT_IN_DEFINED_NAME_FILTER_DATABASE.to_string()),
            hidden: Some(true),
            local_sheet_id: Some((sheet_id - 1) as i64),
            data: filter_range,
            ..Default::default()
        };

        if wb.defined_names.is_none() {
            wb.defined_names = Some(crate::xml::workbook::XlsxDefinedNames {
                defined_name: vec![d],
            });
        } else {
            let names = wb.defined_names.as_mut().unwrap();
            let mut defined_name_exists = false;
            for idx in 0..names.defined_name.len() {
                let defined_name = &names.defined_name[idx];
                let local_sheet_id = defined_name.local_sheet_id.unwrap_or(0) as i32;
                if defined_name.name.as_deref().unwrap_or("")
                    == BUILT_IN_DEFINED_NAME_FILTER_DATABASE
                    && local_sheet_id == sheet_id - 1
                    && defined_name.hidden.unwrap_or(false)
                {
                    names.defined_name[idx].data = d.data.clone();
                    defined_name_exists = true;
                }
            }
            if !defined_name_exists {
                names.defined_name.push(d);
            }
        }
        *self.workbook.lock().unwrap() = Some(wb);

        let columns = coordinates[2] - coordinates[0];
        self.auto_filter_internal(sheet, &ref_str, columns, coordinates[0], opts)
    }
}

// ------------------------------------------------------------------
// Internal helpers
// ------------------------------------------------------------------

impl File {
    /// Get all tables in the workbook keyed by worksheet name.
    pub(crate) fn get_tables_for_workbook(&self) -> Result<HashMap<String, Vec<Table>>> {
        let mut tables: HashMap<String, Vec<Table>> = HashMap::new();
        for sheet_name in self.get_sheet_list() {
            match self.get_tables(&sheet_name) {
                Ok(tbls) => {
                    tables.insert(sheet_name, tbls);
                }
                Err(e) => {
                    if e.downcast_ref::<ErrSheetNotExist>().is_none() {
                        return Err(e);
                    }
                }
            }
        }
        Ok(tables)
    }

    /// Add a `tablePart` reference to a worksheet.
    fn add_sheet_table(&self, sheet: &str, r_id: i32) -> Result<()> {
        let mut ws = self.work_sheet_reader(sheet)?;
        let path = self
            .get_sheet_xml_path(sheet)
            .ok_or_else(|| ErrSheetNotExist {
                sheet_name: sheet.to_string(),
            })?;
        let table = XlsxTablePart {
            rid: Some(format!("rId{r_id}")),
        };
        if ws.table_parts.is_none() {
            ws.table_parts = Some(XlsxTableParts::default());
        }
        let tp = ws.table_parts.as_mut().unwrap();
        tp.count = tp.count.map(|c| c + 1).or(Some(1));
        tp.table_part.push(table);
        self.sheet.insert(path, ws);
        Ok(())
    }

    /// Set the table column names from the header row cells.
    pub(crate) fn set_table_columns(
        &self,
        sheet: &str,
        show_header_row: bool,
        x1: i32,
        y1: i32,
        x2: i32,
        tbl: &mut XlsxTable,
    ) -> Result<()> {
        let mut idx = 0;
        let mut header: Vec<String> = Vec::new();
        let mut table_columns: Vec<XlsxTableColumn> = Vec::new();

        let get_table_column = |name: &str| -> Option<XlsxTableColumn> {
            if let Some(ref cols) = tbl.table_columns {
                for column in &cols.table_column {
                    if column.name == name {
                        return Some(column.clone());
                    }
                }
            }
            None
        };

        for i in x1..=x2 {
            idx += 1;
            let cell = coordinates_to_cell_name(i, y1, false)?;
            let name = self.get_cell_value(sheet, &cell)?;
            if name.parse::<i64>().is_ok() && show_header_row {
                let _ = self.set_cell_str(sheet, &cell, &name);
            }
            let mut name = name;
            if name.is_empty() || in_str_slice(&header, &name, true) != -1 {
                name = format!("Column{idx}");
                if show_header_row {
                    let _ = self.set_cell_str(sheet, &cell, &name);
                }
            }
            header.push(name.clone());
            if let Some(mut column) = get_table_column(&name) {
                column.id = idx as i64;
                column.data_dxf_id = 0;
                column.query_table_field_id = 0;
                table_columns.push(column);
                continue;
            }
            table_columns.push(XlsxTableColumn {
                id: idx as i64,
                name,
                ..Default::default()
            });
        }
        tbl.table_columns = Some(XlsxTableColumns {
            count: table_columns.len() as i64,
            table_column: table_columns,
        });
        Ok(())
    }

    /// Add a table part with the given coordinates and format.
    fn add_table_internal(
        &self,
        sheet: &str,
        table_xml: &str,
        x1: i32,
        mut y1: i32,
        x2: i32,
        mut y2: i32,
        i: i32,
        opts: &Table,
    ) -> Result<()> {
        if y1 == y2 {
            y2 += 1;
        }
        let hide_header_row = opts.show_header_row == Some(false);
        if hide_header_row {
            y1 += 1;
        }
        let ref_str = coordinates_to_range_ref(&[x1, y1, x2, y2], false)?;
        let name = if opts.name.is_empty() {
            format!("Table{i}")
        } else {
            opts.name.clone()
        };
        let mut t = XlsxTable {
            xmlns: Some(NAMESPACE_SPREADSHEET.to_string()),
            id: i as i64,
            name: name.clone(),
            display_name: Some(name),
            r#ref: ref_str.clone(),
            auto_filter: Some(XlsxAutoFilter {
                r#ref: ref_str.clone(),
                ..Default::default()
            }),
            table_style_info: Some(XlsxTableStyleInfo {
                name: Some(opts.style_name.clone()),
                show_first_column: opts.show_first_column,
                show_last_column: opts.show_last_column,
                show_row_stripes: opts.show_row_stripes.unwrap_or(true),
                show_column_stripes: opts.show_column_stripes,
            }),
            ..Default::default()
        };
        let _ = self.set_table_columns(sheet, !hide_header_row, x1, y1, x2, &mut t);
        if hide_header_row {
            t.auto_filter = None;
            t.header_row_count = Some(0);
        }
        let mut table = xml_to_string(&t)?.into_bytes();
        crate::file::strip_empty_attributes(&mut table);
        self.save_file_list(table_xml, &table);
        Ok(())
    }

    /// Apply an AutoFilter to a worksheet.
    fn auto_filter_internal(
        &self,
        sheet: &str,
        ref_str: &str,
        columns: i32,
        col: i32,
        opts: &[AutoFilterOptions],
    ) -> Result<()> {
        let path = self
            .get_sheet_xml_path(sheet)
            .ok_or_else(|| ErrSheetNotExist {
                sheet_name: sheet.to_string(),
            })?;
        let mut ws = self.work_sheet_reader(sheet)?;
        if ws.sheet_pr.is_some() {
            ws.sheet_pr.as_mut().unwrap().filter_mode = Some(true);
        } else {
            ws.sheet_pr = Some(crate::xml::worksheet::XlsxSheetPr {
                filter_mode: Some(true),
                ..Default::default()
            });
        }
        let mut filter = XlsxAutoFilter {
            r#ref: ref_str.to_string(),
            ..Default::default()
        };
        for opt in opts {
            if opt.column.is_empty() || opt.expression.is_empty() {
                continue;
            }
            let fs_col = column_name_to_number(&opt.column)?;
            let offset = fs_col - col;
            if offset < 0 || offset > columns {
                return Err(new_invalid_auto_filter_column_error(&opt.column).into());
            }
            let mut fc = XlsxFilterColumn {
                col_id: offset as i64,
                ..Default::default()
            };
            let token = EXPRESSION_FORMAT
                .find_iter(&opt.expression)
                .map(|m| m.as_str().to_string())
                .collect::<Vec<_>>();
            if token.len() != 3 && token.len() != 7 {
                return Err(new_invalid_auto_filter_exp_error(&opt.expression).into());
            }
            let (expressions, tokens) = self.parse_filter_expression(&opt.expression, &token)?;
            self.write_auto_filter(&mut fc, &expressions, &tokens);
            filter.filter_column.push(fc);
        }
        ws.auto_filter = Some(filter);
        self.sheet.insert(path, ws);
        Ok(())
    }

    /// Write the filter column as default or custom filters.
    fn write_auto_filter(&self, fc: &mut XlsxFilterColumn, exp: &[i32], tokens: &[String]) {
        if exp.len() == 1 && exp[0] == 2 {
            let filters = vec![XlsxFilter {
                val: Some(tokens[0].clone()),
            }];
            fc.filters = Some(XlsxFilters {
                filter: filters,
                ..Default::default()
            });
            return;
        }
        if exp.len() == 3 && exp[0] == 2 && exp[1] == 1 && exp[2] == 2 {
            let filters = tokens
                .iter()
                .map(|v| XlsxFilter {
                    val: Some(v.clone()),
                })
                .collect();
            fc.filters = Some(XlsxFilters {
                filter: filters,
                ..Default::default()
            });
            return;
        }
        let exp_rel = [0, 2];
        let and_rel = [true, false];
        for (k, v) in tokens.iter().enumerate() {
            self.write_custom_filter(fc, exp[exp_rel[k]], v);
            if k == 1 {
                if let Some(ref mut cf) = fc.custom_filters {
                    cf.and = and_rel[exp[k] as usize];
                }
            }
        }
    }

    /// Write a single `<customFilter>` element.
    fn write_custom_filter(&self, fc: &mut XlsxFilterColumn, operator: i32, val: &str) {
        let operators: HashMap<i32, &str> = [
            (1, "lessThan"),
            (2, "equal"),
            (3, "lessThanOrEqual"),
            (4, "greaterThan"),
            (5, "notEqual"),
            (6, "greaterThanOrEqual"),
            (22, "equal"),
        ]
        .into_iter()
        .collect();
        let custom_filter = XlsxCustomFilter {
            operator: operators.get(&operator).map(|s| s.to_string()),
            val: Some(val.to_string()),
        };
        if let Some(ref mut cf) = fc.custom_filters {
            cf.custom_filter.push(custom_filter);
            return;
        }
        fc.custom_filters = Some(XlsxCustomFilters {
            custom_filter: vec![custom_filter],
            ..Default::default()
        });
    }

    /// Parse the filter expression tokens into operator codes.
    fn parse_filter_expression(
        &self,
        expression: &str,
        tokens: &[String],
    ) -> Result<(Vec<i32>, Vec<String>)> {
        if tokens.len() == 7 {
            let conditional = if CONDITION_FORMAT.is_match(&tokens[3]) {
                1
            } else {
                0
            };
            let (expression1, token1) = self.parse_filter_tokens(expression, &tokens[..3])?;
            let (expression2, token2) = self.parse_filter_tokens(expression, &tokens[4..7])?;
            return Ok((
                vec![expression1[0], conditional, expression2[0]],
                vec![token1, token2],
            ));
        }
        let (exp, token) = self.parse_filter_tokens(expression, tokens)?;
        Ok((exp, vec![token]))
    }

    /// Parse a single 3-token filter expression.
    fn parse_filter_tokens(
        &self,
        expression: &str,
        tokens: &[String],
    ) -> Result<(Vec<i32>, String)> {
        let operators: HashMap<String, i32> = [
            ("==", 2),
            ("=", 2),
            ("=~", 2),
            ("eq", 2),
            ("!=", 5),
            ("!~", 5),
            ("ne", 5),
            ("<>", 5),
            ("<", 1),
            ("<=", 3),
            (">", 4),
            (">=", 6),
        ]
        .into_iter()
        .map(|(k, v)| (k.to_string(), v))
        .collect();
        let operator = operators
            .get(&tokens[1].to_lowercase())
            .copied()
            .ok_or_else(|| new_unknown_filter_token_error(&tokens[1]))?;
        let mut token = tokens[2].clone();
        let mut operator = operator;

        if BLANK_FORMAT.is_match(&token.to_lowercase()) {
            if operator != 2 && operator != 5 {
                return Err(new_invalid_auto_filter_operator_error(&tokens[1], expression).into());
            }
            token = token.to_lowercase();
            if token == "blanks" {
                if operator == 5 {
                    token = " ".to_string();
                }
            } else {
                if operator == 5 {
                    operator = 2;
                    token = "blanks".to_string();
                } else {
                    operator = 5;
                    token = " ".to_string();
                }
            }
        }

        if MATCH_FORMAT.is_match(&token) && operator == 2 {
            operator = 22;
        }
        Ok((vec![operator], token))
    }
}

// ------------------------------------------------------------------
// Free functions
// ------------------------------------------------------------------

/// Parse table options and apply defaults.
fn parse_table_options(opts: Option<&Table>) -> Result<Table> {
    let mut options = match opts {
        Some(o) => o.clone(),
        None => {
            return Ok(Table {
                show_row_stripes: bool_ptr(true),
                ..Default::default()
            });
        }
    };
    if options.show_row_stripes.is_none() {
        options.show_row_stripes = bool_ptr(true);
    }
    check_defined_name(&options.name)?;
    Ok(options)
}

/// Check whether a defined name or table name contains illegal characters.
fn check_defined_name(name: &str) -> Result<()> {
    if count_utf16_string(name) > MAX_FIELD_LENGTH {
        return Err(Box::new(ErrNameLength));
    }
    for (i, c) in name.chars().enumerate() {
        let ranges = if i == 0 {
            SUPPORTED_DEFINED_NAME_AT_START_CHAR_CODE_RANGE
        } else {
            SUPPORTED_DEFINED_NAME_AFTER_START_CHAR_CODE_RANGE
        };
        if !in_code_range(c as u32, ranges) {
            return Err(new_invalid_name_error(name).into());
        }
    }
    Ok(())
}

/// Returns true if `code` lies within any of the inclusive ranges in `tbl`.
fn in_code_range(code: u32, tbl: &[(u32, u32)]) -> bool {
    for (start, end) in tbl {
        if *start <= code && code <= *end {
            return true;
        }
    }
    false
}

// ------------------------------------------------------------------
// Tests
// ------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::options::Options;

    fn new_file() -> File {
        File::new_with_options(Options::default())
    }

    #[test]
    fn add_table_basic() {
        let f = new_file();
        f.set_cell_str("Sheet1", "A1", "Header1").unwrap();
        f.set_cell_str("Sheet1", "B1", "Header2").unwrap();
        let table = Table {
            range: "A1:B3".to_string(),
            ..Default::default()
        };
        assert!(f.add_table("Sheet1", Some(&table)).is_ok());

        let tables = f.get_tables("Sheet1").unwrap();
        assert_eq!(tables.len(), 1);
        assert_eq!(tables[0].range, "A1:B3");
        assert_eq!(tables[0].name, "Table1");
    }

    #[test]
    fn add_table_with_options() {
        let f = new_file();
        f.set_cell_str("Sheet1", "A1", "A").unwrap();
        f.set_cell_str("Sheet1", "B1", "B").unwrap();
        let table = Table {
            range: "A1:B5".to_string(),
            name: "table".to_string(),
            style_name: "TableStyleMedium2".to_string(),
            show_column_stripes: true,
            show_first_column: true,
            show_last_column: true,
            show_row_stripes: Some(false),
            ..Default::default()
        };
        assert!(f.add_table("Sheet1", Some(&table)).is_ok());

        let tables = f.get_tables("Sheet1").unwrap();
        assert_eq!(tables[0].name, "table");
        assert_eq!(tables[0].style_name, "TableStyleMedium2");
        assert!(tables[0].show_column_stripes);
    }

    #[test]
    fn add_table_nil_returns_parameter_invalid() {
        let f = new_file();
        let err = f.add_table("Sheet1", None).unwrap_err();
        assert!(err.downcast_ref::<ErrParameterInvalid>().is_some());
    }

    #[test]
    fn add_table_duplicate_name_fails() {
        let f = new_file();
        let table = Table {
            range: "A1:B2".to_string(),
            name: "Table1".to_string(),
            ..Default::default()
        };
        assert!(f.add_table("Sheet1", Some(&table)).is_ok());
        let err = f.add_table("Sheet1", Some(&table)).unwrap_err();
        assert!(err.downcast_ref::<ErrExistsTableName>().is_some());
    }

    #[test]
    fn add_table_invalid_sheet_name() {
        let f = new_file();
        let table = Table {
            range: "A1:B2".to_string(),
            ..Default::default()
        };
        let err = f.add_table("Sheet:1", Some(&table)).unwrap_err();
        assert!(
            err.downcast_ref::<crate::errors::ErrSheetNameInvalid>()
                .is_some()
        );
    }

    #[test]
    fn add_table_invalid_range() {
        let f = new_file();
        let table = Table {
            range: "A:B1".to_string(),
            ..Default::default()
        };
        assert!(f.add_table("Sheet1", Some(&table)).is_err());
    }

    #[test]
    fn add_table_invalid_name() {
        let f = new_file();
        let cases = vec!["1Table", "-Table", "'Table", "Table 1", "A&B", "_1Table'"];
        for name in cases {
            let table = Table {
                range: "A1:B2".to_string(),
                name: name.to_string(),
                ..Default::default()
            };
            assert!(f.add_table("Sheet1", Some(&table)).is_err());
        }
    }

    #[test]
    fn get_tables_empty() {
        let f = new_file();
        let tables = f.get_tables("Sheet1").unwrap();
        assert!(tables.is_empty());
    }

    #[test]
    fn get_tables_not_exist_sheet() {
        let f = new_file();
        let err = f.get_tables("SheetN").unwrap_err();
        assert!(err.downcast_ref::<ErrSheetNotExist>().is_some());
    }

    #[test]
    fn delete_table_round_trip() {
        let f = new_file();
        let table1 = Table {
            range: "A1:B2".to_string(),
            name: "Table1".to_string(),
            ..Default::default()
        };
        let table2 = Table {
            range: "B1:C2".to_string(),
            name: "Table2".to_string(),
            ..Default::default()
        };
        f.add_table("Sheet1", Some(&table1)).unwrap();
        f.add_table("Sheet1", Some(&table2)).unwrap();
        assert!(f.delete_table("Table2").is_ok());
        assert!(f.delete_table("Table1").is_ok());
        assert!(f.get_tables("Sheet1").unwrap().is_empty());
    }

    #[test]
    fn delete_table_not_exist() {
        let f = new_file();
        let err = f.delete_table("Missing").unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("table Missing does not exist"));
    }

    #[test]
    fn auto_filter_basic() {
        let f = new_file();
        f.set_cell_str("Sheet1", "A1", "Name").unwrap();
        f.set_cell_str("Sheet1", "B1", "Score").unwrap();
        f.set_cell_int("Sheet1", "A2", 1).unwrap();
        f.set_cell_int("Sheet1", "B2", 10).unwrap();
        assert!(f.auto_filter("Sheet1", "A1:B2", &[]).is_ok());

        let ws = f.work_sheet_reader("Sheet1").unwrap();
        assert!(ws.auto_filter.is_some());
        assert_eq!(ws.auto_filter.as_ref().unwrap().r#ref, "$A$1:$B$2");
    }

    #[test]
    fn auto_filter_with_expression() {
        let f = new_file();
        let opts = vec![AutoFilterOptions {
            column: "B".to_string(),
            expression: "x != blanks".to_string(),
        }];
        assert!(f.auto_filter("Sheet1", "A1:B4", &opts).is_ok());
    }

    #[test]
    fn auto_filter_invalid_column() {
        let f = new_file();
        let opts = vec![AutoFilterOptions {
            column: "A".to_string(),
            expression: "x == 1".to_string(),
        }];
        let err = f.auto_filter("Sheet1", "B1:C4", &opts).unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("incorrect index of column"));
    }

    #[test]
    fn auto_filter_invalid_expression_tokens() {
        let f = new_file();
        let opts = vec![AutoFilterOptions {
            column: "B".to_string(),
            expression: "x -- y".to_string(),
        }];
        assert!(f.auto_filter("Sheet1", "A1:B4", &opts).is_err());
    }

    #[test]
    fn parse_filter_tokens_unknown_operator() {
        let f = new_file();
        let tokens = vec!["".to_string(), "!".to_string(), "".to_string()];
        let err = f.parse_filter_tokens("", &tokens).unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("unknown operator: !"));
    }

    #[test]
    fn check_defined_name_cases() {
        assert!(check_defined_name("Table1").is_ok());
        assert!(check_defined_name("_Table").is_ok());
        assert!(check_defined_name("1Table").is_err());
        assert!(check_defined_name("Table 1").is_err());
        assert!(check_defined_name("A&B").is_err());
    }
}
