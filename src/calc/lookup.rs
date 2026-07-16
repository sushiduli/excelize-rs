//! Lookup and reference formula functions.

use std::cmp::Ordering;
use std::collections::HashMap;

use crate::calc::arg::*;
use crate::calc::{CalcContext, CellRef, FormulaFn, eval, find_cell, parse_formula};
use crate::constants::{MAX_COLUMNS, TOTAL_ROWS};
use crate::lib_util::{cell_name_to_coordinates, column_number_to_name, coordinates_to_cell_name};

pub fn register(m: &mut HashMap<&'static str, FormulaFn>) {
    m.insert("VLOOKUP", vlookup);
    m.insert("HLOOKUP", hlookup);
    m.insert("MATCH", match_fn);
    m.insert("INDEX", index);
    m.insert("CHOOSE", choose);
    m.insert("ADDRESS", address);
    m.insert("COLUMN", column);
    m.insert("COLUMNS", columns);
    m.insert("ROW", row);
    m.insert("ROWS", rows);
    m.insert("TRANSPOSE", transpose);
    m.insert("LOOKUP", lookup);
    m.insert("INDIRECT", indirect);
    m.insert("XLOOKUP", xlookup);
    m.insert("ANCHORARRAY", anchorarray);
    m.insert("FORMULATEXT", formulatext);
    m.insert("HYPERLINK", hyperlink);
}

// Excel-style comparison result constants used by the lookup functions.
const CRITERIA_EQ: u8 = 0;
const CRITERIA_LE: u8 = 1;
const CRITERIA_GE: u8 = 2;
const CRITERIA_NE: u8 = 3;
const CRITERIA_L: u8 = 4;
const CRITERIA_G: u8 = 5;
const CRITERIA_ERR: u8 = 6;

const MATCH_MODE_MAX_LESS: i32 = -1;
const MATCH_MODE_MIN_GREATER: i32 = 1;
const MATCH_MODE_WILDCARD: i32 = 2;

const SEARCH_MODE_LINEAR: i32 = 1;
const SEARCH_MODE_ASC_BINARY: i32 = 2;
const SEARCH_MODE_DESC_BINARY: i32 = -2;

/// Compare two formula arguments using Excel rules. Returns a criteria byte or
/// `None` when the values are not comparable (criteriaNe).
fn compare_formula_arg(
    lhs: &FormulaArg,
    rhs: &FormulaArg,
    match_mode: i32,
    case_sensitive: bool,
) -> Option<u8> {
    if lhs.typ != rhs.typ {
        return None;
    }
    match lhs.typ {
        ArgType::Number => {
            if lhs.number == rhs.number {
                return Some(CRITERIA_EQ);
            }
            if lhs.number < rhs.number {
                return Some(CRITERIA_L);
            }
            Some(CRITERIA_G)
        }
        ArgType::String => {
            let mut ls = lhs.value();
            let mut rs = rhs.value();
            if !case_sensitive {
                ls = ls.to_lowercase();
                rs = rs.to_lowercase();
            }
            // Wildcard match is handled outside for lookup functions by
            // treating match_mode == MATCH_MODE_WILDCARD as equality.
            Some(match ls.cmp(&rs) {
                Ordering::Equal => CRITERIA_EQ,
                Ordering::Less => CRITERIA_L,
                Ordering::Greater => CRITERIA_G,
            })
        }
        ArgType::Empty => Some(CRITERIA_EQ),
        ArgType::List => compare_formula_arg_list(lhs, rhs, match_mode, case_sensitive),
        ArgType::Matrix => compare_formula_arg_matrix(lhs, rhs, match_mode, case_sensitive),
        _ => Some(CRITERIA_ERR),
    }
}

fn compare_formula_arg_list(
    lhs: &FormulaArg,
    rhs: &FormulaArg,
    match_mode: i32,
    case_sensitive: bool,
) -> Option<u8> {
    if lhs.list.len() < rhs.list.len() {
        return Some(CRITERIA_L);
    }
    if lhs.list.len() > rhs.list.len() {
        return Some(CRITERIA_G);
    }
    for i in 0..lhs.list.len() {
        let c = compare_formula_arg(&lhs.list[i], &rhs.list[i], match_mode, case_sensitive)?;
        if c != CRITERIA_EQ {
            return Some(c);
        }
    }
    Some(CRITERIA_EQ)
}

fn compare_formula_arg_matrix(
    lhs: &FormulaArg,
    rhs: &FormulaArg,
    match_mode: i32,
    case_sensitive: bool,
) -> Option<u8> {
    if lhs.matrix.len() < rhs.matrix.len() {
        return Some(CRITERIA_L);
    }
    if lhs.matrix.len() > rhs.matrix.len() {
        return Some(CRITERIA_G);
    }
    for i in 0..lhs.matrix.len() {
        let left = &lhs.matrix[i];
        let right = &rhs.matrix[i];
        if left.len() < right.len() {
            return Some(CRITERIA_L);
        }
        if left.len() > right.len() {
            return Some(CRITERIA_G);
        }
        for j in 0..left.len() {
            let c = compare_formula_arg(&left[j], &right[j], match_mode, case_sensitive)?;
            if c != CRITERIA_EQ {
                return Some(c);
            }
        }
    }
    Some(CRITERIA_EQ)
}

/// Build a `FormulaArg` representing the whole table array for matrix lookup.
fn matrix_arg(matrix: &[Vec<FormulaArg>]) -> FormulaArg {
    new_matrix_formula_arg(matrix.to_vec())
}

/// Shared argument handling for VLOOKUP/HLOOKUP modelled after Go's
/// `checkHVLookupArgs`.
fn check_hv_lookup_args(
    _name: &str,
    args: &[FormulaArg],
) -> Result<(usize, FormulaArg, Vec<Vec<FormulaArg>>, i32), FormulaArg> {
    if args.len() < 3 || args.len() > 4 {
        return Err(new_error_formula_arg(FORMULA_ERROR_VALUE));
    }
    let lookup_value = args[0].clone();
    let table_array = args[1].clone();
    let table_matrix = match table_array.typ {
        ArgType::Matrix => table_array.matrix.clone(),
        ArgType::List => vec![table_array.list.clone()],
        _ => return Err(new_error_formula_arg(FORMULA_ERROR_VALUE)),
    };
    let idx_arg = &args[2];
    if idx_arg.typ != ArgType::Number || idx_arg.boolean {
        return Err(new_error_formula_arg(FORMULA_ERROR_VALUE));
    }
    let idx = match idx_arg.as_number() {
        Some(n) if n >= 1.0 => n as usize - 1,
        _ => return Err(new_error_formula_arg(FORMULA_ERROR_VALUE)),
    };
    let mut match_mode = MATCH_MODE_MAX_LESS;
    if args.len() == 4 {
        let range_lookup = args[3].to_bool();
        if range_lookup.typ == ArgType::Error {
            return Err(range_lookup);
        }
        if range_lookup.number == 0.0 {
            match_mode = MATCH_MODE_WILDCARD;
        }
    }
    Ok((idx, lookup_value, table_matrix, match_mode))
}

/// Sequential search used for wildcard mode or full-column references.
fn lookup_linear_search(
    vertical: bool,
    lookup_value: &FormulaArg,
    table_matrix: &[Vec<FormulaArg>],
    match_mode: i32,
    _search_mode: i32,
) -> (i32, bool) {
    let mut match_idx = -1i32;
    let mut was_exact = false;
    let table_arg = matrix_arg(table_matrix);
    let mut search = |i: usize, cell: &FormulaArg| -> bool {
        let mut lhs = cell.clone();
        if lookup_value.typ == ArgType::Number {
            let conv = cell.to_number();
            if conv.typ != ArgType::Error {
                lhs = conv;
            }
        } else if lookup_value.typ == ArgType::Matrix {
            lhs = table_arg.clone();
        } else if table_arg.typ == ArgType::String {
            lhs = new_string_formula_arg(cell.value());
        }
        if compare_formula_arg(&lhs, lookup_value, match_mode, false) == Some(CRITERIA_EQ) {
            match_idx = i as i32;
            was_exact = true;
            return true;
        }
        false
    };

    if vertical {
        for (i, row) in table_matrix.iter().enumerate() {
            if !row.is_empty() && search(i, &row[0]) {
                break;
            }
        }
    } else {
        for (i, cell) in table_matrix[0].iter().enumerate() {
            if search(i, cell) {
                break;
            }
        }
    }
    (match_idx, was_exact)
}

/// Binary search for range-lookup mode, matching Go's `lookupBinarySearch`.
fn lookup_binary_search(
    vertical: bool,
    lookup_value: &FormulaArg,
    table_matrix: &[Vec<FormulaArg>],
    match_mode: i32,
    search_mode: i32,
) -> (i32, bool) {
    let table_arg = matrix_arg(table_matrix);
    let table_array: Vec<FormulaArg> = if vertical {
        table_matrix
            .iter()
            .map(|row| row.first().cloned().unwrap_or_default())
            .collect()
    } else {
        table_matrix[0].clone()
    };
    let mut low = 0i32;
    let mut high = table_array.len() as i32 - 1;
    let mut last_match_idx = -1i32;
    let count = high;
    let mut match_idx = -1i32;
    let mut was_exact = false;
    while low <= high {
        let mid = low + (high - low) / 2;
        let cell = &table_array[mid as usize];
        let mut lhs = cell.clone();
        if lookup_value.typ == ArgType::Number {
            let conv = cell.to_number();
            if conv.typ != ArgType::Error {
                lhs = conv;
            }
        } else if lookup_value.typ == ArgType::Matrix && vertical {
            lhs = table_arg.clone();
        } else if lookup_value.typ == ArgType::String {
            lhs = new_string_formula_arg(cell.value());
        }
        let result = compare_formula_arg(&lhs, lookup_value, match_mode, false);
        match result {
            Some(CRITERIA_EQ) => {
                match_idx = mid;
                was_exact = true;
                if search_mode == SEARCH_MODE_DESC_BINARY {
                    match_idx = count - match_idx;
                }
                return (match_idx, was_exact);
            }
            Some(CRITERIA_G) => {
                high = mid - 1;
            }
            Some(CRITERIA_L) => {
                match_idx = mid;
                if cell.typ != ArgType::Empty {
                    last_match_idx = match_idx;
                }
                low = mid + 1;
            }
            _ => return (-1, false),
        }
    }
    match_idx = last_match_idx;
    was_exact = true;
    (match_idx, was_exact)
}

// VLOOKUP(lookup_value, table_array, col_index_num, [range_lookup])
fn vlookup(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    let (col_idx, lookup_value, table_matrix, match_mode) =
        match check_hv_lookup_args("VLOOKUP", args) {
            Ok(v) => v,
            Err(e) => return e,
        };
    if table_matrix.is_empty() {
        return new_error_formula_arg(FORMULA_ERROR_NA);
    }
    let (match_idx, was_exact) =
        if match_mode == MATCH_MODE_WILDCARD || table_matrix.len() == TOTAL_ROWS as usize {
            lookup_linear_search(
                true,
                &lookup_value,
                &table_matrix,
                match_mode,
                SEARCH_MODE_LINEAR,
            )
        } else {
            lookup_binary_search(
                true,
                &lookup_value,
                &table_matrix,
                match_mode,
                SEARCH_MODE_ASC_BINARY,
            )
        };
    if match_idx < 0 {
        return new_error_formula_arg(FORMULA_ERROR_NA);
    }
    let row = match table_matrix.get(match_idx as usize) {
        Some(r) => r,
        None => return new_error_formula_arg(FORMULA_ERROR_NA),
    };
    if col_idx >= row.len() {
        return new_error_formula_arg(FORMULA_ERROR_NA);
    }
    if was_exact || match_mode == MATCH_MODE_WILDCARD {
        row[col_idx].clone()
    } else {
        new_error_formula_arg(FORMULA_ERROR_NA)
    }
}

// ADDRESS(row_num, column_num, [abs_num], [a1], [sheet_text])
fn address(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() < 2 || args.len() > 5 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let row = match args[0].to_number().as_number() {
        Some(n) if n >= 1.0 && n <= TOTAL_ROWS as f64 => n as i32,
        _ => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let col = match args[1].to_number().as_number() {
        Some(n) if n >= 1.0 && n <= MAX_COLUMNS as f64 => n as i32,
        _ => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let abs_num = if args.len() >= 3 {
        match args[2].to_number().as_number() {
            Some(n) if n >= 1.0 && n <= 4.0 => n as i32,
            _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
        }
    } else {
        1
    };
    let a1 = if args.len() >= 4 {
        match args[3].to_bool().as_number() {
            Some(n) => n != 0.0,
            None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
        }
    } else {
        true
    };
    let sheet_text = if args.len() == 5 {
        format!("{}!", args[4].value())
    } else {
        String::new()
    };
    let addr = match format_address(col, row, abs_num, a1) {
        Ok(s) => s,
        Err(_) => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    new_string_formula_arg(format!("{}{}", sheet_text, addr))
}

fn format_address(col: i32, row: i32, abs_num: i32, a1: bool) -> Result<String, String> {
    match (abs_num, a1) {
        (1, true) => coordinates_to_cell_name(col, row, true),
        (1, false) => Ok(format!("R{}C{}", row, col)),
        (2, true) => {
            let c = column_number_to_name(col)?;
            Ok(format!("{}${}", c, row))
        }
        (2, false) => Ok(format!("R{}C[{}]", row, col)),
        (3, true) => {
            let c = column_number_to_name(col)?;
            Ok(format!("${}{}", c, row))
        }
        (3, false) => Ok(format!("R[{}]C{}", row, col)),
        (4, true) => coordinates_to_cell_name(col, row, false),
        (4, false) => Ok(format!("R[{}]C[{}]", row, col)),
        _ => Err("invalid abs_num".to_string()),
    }
}

// CHOOSE(index_num, value1, [value2], ...)
fn choose(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() < 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let idx = match args[0].to_number().as_number() {
        Some(n) if n >= 1.0 && n <= (args.len() - 1) as f64 => n as usize - 1,
        _ => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    args[idx + 1].clone()
}

// COLUMN([reference])
fn column(ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() > 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    if args.len() == 1 {
        if let Some(cr) = args[0].cell_ranges.first() {
            return new_number_formula_arg(cr.0.col as f64);
        }
        if let Some(cr) = args[0].cell_refs.first() {
            return new_number_formula_arg(cr.col as f64);
        }
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let col = match cell_name_to_coordinates(&ctx.cell) {
        Ok((col, _)) => col,
        Err(_) => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    new_number_formula_arg(col as f64)
}

// COLUMNS(array)
fn columns(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let (min_col, max_col) = calc_cols_rows_min_max(true, &args[0]);
    if max_col == MAX_COLUMNS {
        return new_number_formula_arg(MAX_COLUMNS as f64);
    }
    if max_col == 0 && min_col == 0 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    new_number_formula_arg((max_col - min_col + 1) as f64)
}

// ROW([reference])
fn row(ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() > 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    if args.len() == 1 {
        if let Some(cr) = args[0].cell_ranges.first() {
            return new_number_formula_arg(cr.0.row as f64);
        }
        if let Some(cr) = args[0].cell_refs.first() {
            return new_number_formula_arg(cr.row as f64);
        }
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let row = match cell_name_to_coordinates(&ctx.cell) {
        Ok((_, row)) => row,
        Err(_) => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    new_number_formula_arg(row as f64)
}

// ROWS(array)
fn rows(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let (min_row, max_row) = calc_cols_rows_min_max(false, &args[0]);
    if max_row == TOTAL_ROWS {
        return new_number_formula_arg(TOTAL_ROWS as f64);
    }
    if max_row == 0 && min_row == 0 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    new_number_formula_arg((max_row - min_row + 1) as f64)
}

fn calc_cols_rows_min_max(cols: bool, arg: &FormulaArg) -> (i32, i32) {
    let mut min_val = 0;
    let mut max_val = 0;
    let get_val = |cell: &CellRef| if cols { cell.col } else { cell.row };
    for cr in &arg.cell_ranges {
        if min_val == 0 {
            min_val = get_val(&cr.0);
        }
        if max_val < get_val(&cr.1) {
            max_val = get_val(&cr.1);
        }
    }
    for cr in &arg.cell_refs {
        if min_val == 0 {
            min_val = get_val(cr);
        }
        if max_val < get_val(cr) {
            max_val = get_val(cr);
        }
    }
    (min_val, max_val)
}

// TRANSPOSE(array)
fn transpose(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let matrix = match &args[0].typ {
        ArgType::Matrix => args[0].matrix.clone(),
        ArgType::List => vec![args[0].list.clone()],
        _ => vec![vec![args[0].clone()]],
    };
    if matrix.is_empty() || matrix[0].is_empty() {
        return new_matrix_formula_arg(Vec::new());
    }
    let rows = matrix.len();
    let cols = matrix[0].len();
    let mut result = vec![vec![new_empty_formula_arg(); rows]; cols];
    for r in 0..rows {
        for c in 0..cols {
            result[c][r] = matrix[r][c].clone();
        }
    }
    new_matrix_formula_arg(result)
}

// HYPERLINK(link_location, [friendly_name])
fn hyperlink(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.is_empty() || args.len() > 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    args.last().unwrap().clone()
}

// HLOOKUP(lookup_value, table_array, row_index_num, [range_lookup])
fn hlookup(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    let (row_idx, lookup_value, table_matrix, match_mode) =
        match check_hv_lookup_args("HLOOKUP", args) {
            Ok(v) => v,
            Err(e) => return e,
        };
    if table_matrix.is_empty() || table_matrix[0].is_empty() {
        return new_error_formula_arg(FORMULA_ERROR_NA);
    }
    let (match_idx, was_exact) =
        if match_mode == MATCH_MODE_WILDCARD || table_matrix.len() == TOTAL_ROWS as usize {
            lookup_linear_search(
                false,
                &lookup_value,
                &table_matrix,
                match_mode,
                SEARCH_MODE_LINEAR,
            )
        } else {
            lookup_binary_search(
                false,
                &lookup_value,
                &table_matrix,
                match_mode,
                SEARCH_MODE_ASC_BINARY,
            )
        };
    if match_idx < 0 {
        return new_error_formula_arg(FORMULA_ERROR_NA);
    }
    if row_idx >= table_matrix.len() {
        return new_error_formula_arg(FORMULA_ERROR_NA);
    }
    let row = &table_matrix[row_idx];
    if match_idx as usize >= row.len() {
        return new_error_formula_arg(FORMULA_ERROR_NA);
    }
    if was_exact || match_mode == MATCH_MODE_WILDCARD {
        row[match_idx as usize].clone()
    } else {
        new_error_formula_arg(FORMULA_ERROR_NA)
    }
}

// MATCH(lookup_value, lookup_array, [match_type])
fn match_fn(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 2 && args.len() != 3 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let match_type = if args.len() == 3 {
        match args[2].to_number().as_number() {
            Some(n) if n == -1.0 || n == 0.0 || n == 1.0 => n as i32,
            _ => return new_error_formula_arg(FORMULA_ERROR_NA),
        }
    } else {
        1
    };
    let lookup_array = match &args[1].typ {
        ArgType::Matrix => {
            let m = &args[1].matrix;
            if m.len() != 1 && m[0].len() != 1 {
                return new_error_formula_arg(FORMULA_ERROR_NA);
            }
            args[1].to_list()
        }
        ArgType::List => args[1].list.clone(),
        _ => return new_error_formula_arg(FORMULA_ERROR_NA),
    };
    let lookup = &args[0];

    match match_type {
        0 => {
            for (i, cell) in lookup_array.iter().enumerate() {
                if compare_equal(lookup, cell) {
                    return new_number_formula_arg((i + 1) as f64);
                }
            }
            new_error_formula_arg(FORMULA_ERROR_NA)
        }
        1 => {
            let target = match lookup.to_number().as_number() {
                Some(n) => n,
                None => return new_error_formula_arg(FORMULA_ERROR_NA),
            };
            let mut idx = -1;
            for (i, cell) in lookup_array.iter().enumerate() {
                if let Some(v) = cell.to_number().as_number() {
                    if v <= target {
                        idx = i as i32;
                    } else {
                        break;
                    }
                }
            }
            if idx == -1 {
                new_error_formula_arg(FORMULA_ERROR_NA)
            } else {
                new_number_formula_arg((idx + 1) as f64)
            }
        }
        -1 => {
            let target = match lookup.to_number().as_number() {
                Some(n) => n,
                None => return new_error_formula_arg(FORMULA_ERROR_NA),
            };
            let mut idx = -1;
            for (i, cell) in lookup_array.iter().enumerate() {
                if let Some(v) = cell.to_number().as_number() {
                    if v >= target {
                        idx = i as i32;
                        break;
                    }
                }
            }
            if idx == -1 {
                new_error_formula_arg(FORMULA_ERROR_NA)
            } else {
                new_number_formula_arg((idx + 1) as f64)
            }
        }
        _ => new_error_formula_arg(FORMULA_ERROR_NA),
    }
}

// INDEX(array, row_num, [col_num])
fn index(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() < 2 || args.len() > 3 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let array = match &args[0].typ {
        ArgType::Matrix => args[0].matrix.clone(),
        ArgType::List => vec![args[0].list.clone()],
        _ => vec![vec![args[0].clone()]],
    };
    if array.is_empty() {
        return new_error_formula_arg(FORMULA_ERROR_REF);
    }
    let row_num = match args[1].to_number().as_number() {
        Some(n) => n as i32,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let col_num = if args.len() == 3 {
        match args[2].to_number().as_number() {
            Some(n) => n as i32,
            None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
        }
    } else {
        0
    };

    if row_num == 0 && col_num == 0 {
        if array.len() == 1 && array[0].len() == 1 {
            return array[0][0].clone();
        }
        let list = array.into_iter().flat_map(|r| r.into_iter()).collect();
        return new_list_formula_arg(list);
    }

    let row_idx = row_num - 1;
    let col_idx = col_num - 1;
    let cells = index_internal(&array, row_idx, col_idx);
    if cells.typ != ArgType::List {
        return cells;
    }
    if col_idx == -1 {
        return new_matrix_formula_arg(vec![cells.list]);
    }
    cells
        .list
        .get(col_idx as usize)
        .cloned()
        .unwrap_or(new_error_formula_arg(FORMULA_ERROR_REF))
}

fn index_internal(array: &[Vec<FormulaArg>], row_idx: i32, col_idx: i32) -> FormulaArg {
    if row_idx < -1 || row_idx >= array.len() as i32 {
        return new_error_formula_arg(FORMULA_ERROR_REF);
    }
    if row_idx == -1 {
        if col_idx < 0 || col_idx >= array[0].len() as i32 {
            return new_error_formula_arg(FORMULA_ERROR_REF);
        }
        let column: Vec<Vec<FormulaArg>> = array
            .iter()
            .map(|row| vec![row[col_idx as usize].clone()])
            .collect();
        return new_matrix_formula_arg(column);
    }
    let row = array[row_idx as usize].clone();
    if col_idx < -1 || col_idx >= row.len() as i32 {
        return new_error_formula_arg(FORMULA_ERROR_REF);
    }
    new_list_formula_arg(row)
}

// LOOKUP(lookup_value, lookup_vector, [result_vector])
fn lookup(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() < 2 || args.len() > 3 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let lookup_value = &args[0];
    let lookup_vector = &args[1];
    let (array_form, cols) = match &lookup_vector.typ {
        ArgType::Matrix => {
            let m = &lookup_vector.matrix;
            if m.is_empty() {
                return new_error_formula_arg(FORMULA_ERROR_VALUE);
            }
            (
                true,
                m.iter()
                    .map(|row| row.get(0).cloned().unwrap_or(new_empty_formula_arg()))
                    .collect::<Vec<_>>(),
            )
        }
        ArgType::List => (false, lookup_vector.list.clone()),
        _ => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };

    let target = lookup_value.to_number();
    let mut match_idx = -1;
    let mut ok = false;
    for (idx, col) in cols.iter().enumerate() {
        let lhs = if col.typ == ArgType::Number && target.typ == ArgType::Number {
            target.clone()
        } else {
            lookup_value.clone()
        };
        let cmp = compare_for_lookup(&lhs, col);
        if cmp == Ordering::Equal {
            match_idx = idx as i32;
            break;
        }
        if idx == 0 {
            ok = cmp == Ordering::Greater;
        } else if ok && cmp == Ordering::Less && match_idx == -1 {
            match_idx = (idx - 1) as i32;
        }
    }
    if ok && match_idx == -1 {
        match_idx = (cols.len().saturating_sub(1)) as i32;
    }

    let result_col = if args.len() == 3 {
        match &args[2].typ {
            ArgType::Matrix => args[2]
                .matrix
                .iter()
                .map(|row| row.get(0).cloned().unwrap_or(new_empty_formula_arg()))
                .collect(),
            ArgType::List => args[2].list.clone(),
            _ => return new_error_formula_arg(FORMULA_ERROR_VALUE),
        }
    } else if array_form && lookup_vector.matrix[0].len() > 1 {
        lookup_vector
            .matrix
            .iter()
            .map(|row| row.get(1).cloned().unwrap_or(new_empty_formula_arg()))
            .collect()
    } else {
        cols
    };

    if match_idx < 0 || match_idx >= result_col.len() as i32 {
        return new_error_formula_arg(FORMULA_ERROR_NA);
    }
    result_col[match_idx as usize].clone()
}

fn compare_for_lookup(a: &FormulaArg, b: &FormulaArg) -> Ordering {
    if let (Some(a), Some(b)) = (a.as_number(), b.as_number()) {
        return a.partial_cmp(&b).unwrap_or(Ordering::Equal);
    }
    a.value().to_uppercase().cmp(&b.value().to_uppercase())
}

// XLOOKUP(lookup_value, lookup_array, return_array, [if_not_found], [match_mode], [search_mode])
fn xlookup(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() < 3 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    if args.len() > 6 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let lookup_value = &args[0];
    let lookup_array = match &args[1].typ {
        ArgType::Matrix => args[1].matrix.clone(),
        ArgType::List => vec![args[1].list.clone()],
        _ => return new_error_formula_arg(FORMULA_ERROR_NA),
    };
    let return_array = match &args[2].typ {
        ArgType::Matrix => args[2].matrix.clone(),
        ArgType::List => vec![args[2].list.clone()],
        _ => return new_error_formula_arg(FORMULA_ERROR_NA),
    };
    let if_not_found = args
        .get(3)
        .cloned()
        .unwrap_or(new_error_formula_arg(FORMULA_ERROR_NA));
    let match_mode = if args.len() > 4 {
        match args[4].to_number().as_number() {
            Some(n) if n == 0.0 || n == 1.0 || n == -1.0 || n == 2.0 => n as i32,
            _ => return new_error_formula_arg(FORMULA_ERROR_VALUE),
        }
    } else {
        0
    };
    let search_mode = if args.len() > 5 {
        match args[5].to_number().as_number() {
            Some(n) if n == 1.0 || n == -1.0 || n == 2.0 || n == -2.0 => n as i32,
            _ => return new_error_formula_arg(FORMULA_ERROR_VALUE),
        }
    } else {
        1
    };

    let lookup_rows = lookup_array.len();
    let lookup_cols = if lookup_rows > 0 {
        lookup_array[0].len()
    } else {
        0
    };
    if lookup_rows != 1 && lookup_cols != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let vertical_lookup = lookup_rows >= lookup_cols;

    let lookup_vector: Vec<FormulaArg> = if vertical_lookup {
        lookup_array.iter().map(|row| row[0].clone()).collect()
    } else {
        lookup_array[0].clone()
    };

    let match_idx = search_lookup(lookup_value, &lookup_vector, match_mode, search_mode);
    if match_idx == -1 {
        return if_not_found;
    }

    let return_rows = return_array.len();
    let return_cols = if return_rows > 0 {
        return_array[0].len()
    } else {
        0
    };

    if lookup_rows == 1 && lookup_cols == 1 {
        if return_rows == 1 {
            return return_array[0]
                .get(match_idx as usize)
                .cloned()
                .unwrap_or(if_not_found);
        }
        if return_cols == 1 {
            return return_array
                .get(match_idx as usize)
                .and_then(|r| r.first())
                .cloned()
                .unwrap_or(if_not_found);
        }
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }

    if vertical_lookup {
        return_array
            .get(match_idx as usize)
            .cloned()
            .map(new_list_formula_arg)
            .unwrap_or(if_not_found)
    } else {
        let col: Vec<FormulaArg> = return_array
            .iter()
            .map(|row| {
                row.get(match_idx as usize)
                    .cloned()
                    .unwrap_or(new_empty_formula_arg())
            })
            .collect();
        new_list_formula_arg(col)
    }
}

fn search_lookup(
    lookup: &FormulaArg,
    vector: &[FormulaArg],
    match_mode: i32,
    search_mode: i32,
) -> i32 {
    let indices: Vec<usize> = match search_mode {
        1 | 2 => (0..vector.len()).collect(),
        -1 | -2 => (0..vector.len()).rev().collect(),
        _ => (0..vector.len()).collect(),
    };

    match match_mode {
        0 | 2 => {
            for &i in &indices {
                if compare_equal(lookup, &vector[i]) {
                    return i as i32;
                }
            }
            -1
        }
        1 | -1 => {
            let target = match lookup.to_number().as_number() {
                Some(n) => n,
                None => return -1,
            };
            let mut best: Option<(usize, f64)> = None;
            for &i in &indices {
                if let Some(v) = vector[i].to_number().as_number() {
                    if match_mode == 1 {
                        if v <= target {
                            best = Some((i, v));
                        } else if search_mode == 2 || search_mode == -2 {
                            break;
                        }
                    } else if v >= target {
                        best = Some((i, v));
                        break;
                    }
                }
            }
            best.map(|(i, _)| i as i32).unwrap_or(-1)
        }
        _ => -1,
    }
}

fn indirect(ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.is_empty() || args.len() > 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let ref_text = args[0].as_string();
    if ref_text.is_empty() {
        return new_error_formula_arg(FORMULA_ERROR_REF);
    }
    let a1_style = args.get(1).map(|a| a.as_bool()).unwrap_or(true);

    let a1_text = if a1_style {
        normalize_ref_text(&ref_text)
    } else {
        match r1c1_to_a1(ctx, &ref_text) {
            Some(s) => s,
            None => return new_error_formula_arg(FORMULA_ERROR_REF),
        }
    };

    let expr = match parse_formula(&a1_text) {
        Ok(e) => e,
        Err(_) => return new_error_formula_arg(FORMULA_ERROR_REF),
    };
    eval(ctx, &expr)
}

fn formulatext(ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let (sheet, cell) = match first_reference(ctx, &args[0]) {
        Some(r) => r,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let ws = match ctx.file.work_sheet_reader(sheet) {
        Ok(ws) => ws,
        Err(_) => return new_error_formula_arg(FORMULA_ERROR_NA),
    };
    match find_cell(&ws, &cell) {
        Some(c) => match c.f.as_ref() {
            Some(f) if !f.content.is_empty() => new_string_formula_arg(format!("={}", f.content)),
            _ => new_string_formula_arg(String::new()),
        },
        None => new_error_formula_arg(FORMULA_ERROR_NA),
    }
}

fn anchorarray(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    if let Some(r) = args[0].cell_refs.first() {
        return new_list_formula_arg(vec![new_string_formula_arg(r.to_cell_name())]);
    }
    if let Some((start, _)) = args[0].cell_ranges.first() {
        return new_list_formula_arg(vec![new_string_formula_arg(start.to_cell_name())]);
    }
    new_error_formula_arg(FORMULA_ERROR_VALUE)
}

/// Return the first reference carried by `arg` as `(sheet_name, cell_name)`.
fn first_reference<'a>(ctx: &'a CalcContext, arg: &'a FormulaArg) -> Option<(&'a str, String)> {
    if let Some(r) = arg.cell_refs.first() {
        return Some((r.sheet.as_deref().unwrap_or(ctx.sheet), r.to_cell_name()));
    }
    if let Some((start, _)) = arg.cell_ranges.first() {
        return Some((
            start.sheet.as_deref().unwrap_or(ctx.sheet),
            start.to_cell_name(),
        ));
    }
    None
}

/// Strip surrounding single quotes from a sheet name and keep the rest of the
/// reference intact so that the parser can understand it.
fn normalize_ref_text(text: &str) -> String {
    if let Some(pos) = text.find("!'") {
        // Sheet name is quoted and followed by a trailing quote before `!` is
        // not possible; handle `'Sheet 1'!A1` by removing both quotes.
        let (sheet_part, rest) = text.split_at(pos + 1);
        let sheet = sheet_part.trim_start_matches('\'').trim_end_matches('\'');
        return format!("{}!{}", sheet, &rest[1..]);
    }
    text.to_string()
}

/// Convert an R1C1-style reference to an A1-style reference relative to the
/// current calculation cell. Supports sheet-qualified references and ranges.
fn r1c1_to_a1(ctx: &CalcContext, text: &str) -> Option<String> {
    let (base_col, base_row) = cell_name_to_coordinates(&ctx.cell).unwrap_or((1, 1));
    let parts: Vec<&str> = text.split(':').collect();
    if parts.is_empty() || parts.len() > 2 {
        return None;
    }
    let mut a1_parts = Vec::new();
    for part in parts {
        let (sheet, local) = if let Some(bang) = part.rfind('!') {
            let s = part[..bang].trim().trim_matches('\'');
            (Some(s), &part[bang + 1..])
        } else {
            (None, part)
        };
        let local = local.to_uppercase();
        let row = parse_r1c1_component(&local, 'R', base_row)?;
        let col = parse_r1c1_component(&local, 'C', base_col)?;
        if row < 1 || row > TOTAL_ROWS || col < 1 || col > MAX_COLUMNS {
            return None;
        }
        let cell = coordinates_to_cell_name(col, row, false).ok()?;
        a1_parts.push(match sheet {
            Some(s) => format!("{}!{}", s, cell),
            None => cell,
        });
    }
    Some(a1_parts.join(":"))
}

fn parse_r1c1_component(local: &str, letter: char, base: i32) -> Option<i32> {
    let idx = local.find(letter)?;
    let rest = &local[idx + 1..];
    if rest.starts_with('[') {
        let end = rest.find(']')?;
        let offset: i32 = rest[1..end].parse().ok()?;
        Some(base + offset)
    } else {
        let num_str: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
        if num_str.is_empty() {
            return Some(base);
        }
        num_str.parse().ok()
    }
}
