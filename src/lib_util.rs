//! General utility functions used across the crate.
//!
//! These functions are ported from Go `lib.go` and do not depend on the `File`
//! struct. File-specific helpers live in `file.rs` and `excelize.rs`.

use std::cmp;
use std::collections::HashMap;
use std::sync::LazyLock;

use regex::Regex;

use crate::constants::{MAX_COLUMNS, MIN_COLUMNS, TOTAL_ROWS};
use crate::errors::{
    ErrColumnNumber, ErrCoordinates, ErrMaxRows, ErrParameterInvalid,
    new_coordinates_to_cell_name_error, new_invalid_cell_name_error, new_invalid_column_name_error,
    new_invalid_row_number_error,
};

// ------------------------------------------------------------------
// Cell / coordinate conversion
// ------------------------------------------------------------------

/// Splits a cell name into column name and row number.
///
/// Example: `"AK74"` → `("AK", 74)`.
pub fn split_cell_name(cell: &str) -> Result<(String, i32), String> {
    let alpha = |r: char| ('A' <= r && r <= 'Z') || ('a' <= r && r <= 'z') || (r == '$');

    if cell.chars().next().map_or(false, alpha) {
        if let Some(i) = cell.rfind(alpha) {
            if i < cell.len() - 1 {
                let col = cell[..=i].replace('$', "");
                let row_str = &cell[i + 1..];
                if let Ok(row) = row_str.parse::<i32>() {
                    if row > 0 {
                        return Ok((col, row));
                    }
                }
            }
        }
    }
    Err(new_invalid_cell_name_error(cell))
}

/// Joins column name and row number into a cell name.
pub fn join_cell_name(col: &str, row: i32) -> Result<String, String> {
    let norm_col: String = col
        .chars()
        .filter_map(|ch| {
            if 'A' <= ch && ch <= 'Z' {
                Some(ch)
            } else if 'a' <= ch && ch <= 'z' {
                Some((ch as u8 - b'a' + b'A') as char)
            } else {
                None
            }
        })
        .collect();

    if col.is_empty() || col.len() != norm_col.len() {
        return Err(new_invalid_column_name_error(col));
    }
    if row < 1 {
        return Err(new_invalid_row_number_error(row));
    }
    Ok(format!("{norm_col}{row}"))
}

/// Converts an Excel column name to a 1-based column number.
pub fn column_name_to_number(name: &str) -> Result<i32, String> {
    if name.is_empty() {
        return Err(new_invalid_column_name_error(name));
    }
    let mut col = 0_i64;
    let mut multi = 1_i64;
    for r in name.bytes().rev() {
        let value = if b'A' <= r && r <= b'Z' {
            (r - b'A' + 1) as i64
        } else if b'a' <= r && r <= b'z' {
            (r - b'a' + 1) as i64
        } else {
            return Err(new_invalid_column_name_error(name));
        };
        col += value * multi;
        multi *= 26;
    }
    if col > MAX_COLUMNS as i64 {
        return Err(ErrColumnNumber.to_string());
    }
    Ok(col as i32)
}

static COLUMN_NAMES: LazyLock<Vec<String>> = LazyLock::new(|| {
    let mut names = vec![String::new(); (MAX_COLUMNS + 1) as usize];
    for i in 1..=MAX_COLUMNS {
        let mut num = i;
        let mut l = 0usize;
        let mut n = i;
        while n > 0 {
            l += 1;
            n = (n - 1) / 26;
        }
        let mut buf = vec![0u8; l as usize];
        while num > 0 {
            l -= 1;
            buf[l] = ((num - 1) % 26) as u8 + b'A';
            num = (num - 1) / 26;
        }
        names[i as usize] = String::from_utf8(buf).unwrap();
    }
    names
});

/// Converts a 1-based column number to an Excel column name.
pub fn column_number_to_name(num: i32) -> Result<String, String> {
    if num < MIN_COLUMNS || num > MAX_COLUMNS {
        return Err(ErrColumnNumber.to_string());
    }
    Ok(COLUMN_NAMES[num as usize].clone())
}

/// Converts an alphanumeric cell name to `[col, row]` coordinates.
pub fn cell_name_to_coordinates(cell: &str) -> Result<(i32, i32), String> {
    let (col_name, row) = split_cell_name(cell)?;
    if row > TOTAL_ROWS {
        return Err(ErrMaxRows.to_string());
    }
    let col = column_name_to_number(&col_name)?;
    Ok((col, row))
}

/// Converts `[col, row]` coordinates to an alphanumeric cell name.
pub fn coordinates_to_cell_name(col: i32, row: i32, abs: bool) -> Result<String, String> {
    if col < 1 || row < 1 {
        return Err(new_coordinates_to_cell_name_error(col, row));
    }
    if row > TOTAL_ROWS {
        return Err(ErrMaxRows.to_string());
    }
    let col_name = column_number_to_name(col)?;
    if abs {
        Ok(format!("${col_name}${row}"))
    } else {
        Ok(format!("{col_name}{row}"))
    }
}

/// Converts a range reference such as `"A1:B2"` to `[x1, y1, x2, y2]`.
pub fn range_ref_to_coordinates(ref_str: &str) -> Result<Vec<i32>, String> {
    let normalized = ref_str.replace('$', "");
    let rng: Vec<&str> = normalized.split(':').collect();
    if rng.len() < 2 {
        return Err(ErrParameterInvalid.to_string());
    }
    cell_refs_to_coordinates(rng[0], rng[1])
}

/// Converts two cell references to a coordinate tuple.
pub fn cell_refs_to_coordinates(first_cell: &str, last_cell: &str) -> Result<Vec<i32>, String> {
    let mut coordinates = vec![0; 4];
    let (col, row) = cell_name_to_coordinates(first_cell)?;
    coordinates[0] = col;
    coordinates[1] = row;
    let (col, row) = cell_name_to_coordinates(last_cell)?;
    coordinates[2] = col;
    coordinates[3] = row;
    Ok(coordinates)
}

/// Corrects a cell range so that the top-left and bottom-right corners are
/// ordered correctly.
pub fn sort_coordinates(coordinates: &mut [i32]) -> Result<(), String> {
    if coordinates.len() != 4 {
        return Err(ErrCoordinates.to_string());
    }
    if coordinates[2] < coordinates[0] {
        coordinates.swap(2, 0);
    }
    if coordinates[3] < coordinates[1] {
        coordinates.swap(3, 1);
    }
    Ok(())
}

/// Converts a coordinate tuple back to a range reference.
pub fn coordinates_to_range_ref(coordinates: &[i32], abs: bool) -> Result<String, String> {
    if coordinates.len() != 4 {
        return Err(ErrCoordinates.to_string());
    }
    let first = coordinates_to_cell_name(coordinates[0], coordinates[1], abs)?;
    let last = coordinates_to_cell_name(coordinates[2], coordinates[3], abs)?;
    Ok(format!("{first}:{last}"))
}

/// Converts a reference sequence such as `"A1 A2:B3"` to a map of column number
/// to a sorted list of `[col, row]` coordinates.
pub fn flat_sqref(sqref: &str) -> Result<std::collections::HashMap<i32, Vec<Vec<i32>>>, String> {
    let mut cells: std::collections::HashMap<i32, Vec<Vec<i32>>> = std::collections::HashMap::new();
    for r#ref in sqref.split_whitespace() {
        let rng: Vec<&str> = r#ref.split(':').collect();
        match rng.len() {
            1 => {
                let (col, row) = cell_name_to_coordinates(rng[0])?;
                cells.entry(col).or_default().push(vec![col, row]);
            }
            2 => {
                let mut coordinates = range_ref_to_coordinates(r#ref)?;
                sort_coordinates(&mut coordinates)?;
                for c in coordinates[0]..=coordinates[2] {
                    for r in coordinates[1]..=coordinates[3] {
                        cells.entry(c).or_default().push(vec![c, r]);
                    }
                }
            }
            _ => return Err(ErrParameterInvalid.to_string()),
        }
    }
    for col_cells in cells.values_mut() {
        col_cells.sort_by_key(|c| c[1]);
    }
    Ok(cells)
}

/// Returns the index of `x` in the coordinate list `a`, or `-1` if not found.
pub fn in_coordinates(a: &[Vec<i32>], x: &[i32]) -> i32 {
    for (idx, n) in a.iter().enumerate() {
        if x.len() >= 2 && n.len() >= 2 && x[0] == n[0] && x[1] == n[1] {
            return idx as i32;
        }
    }
    -1
}

// ------------------------------------------------------------------
// Slice helpers
// ------------------------------------------------------------------

/// Returns the index of `x` in `a`, or `-1` if not found.
pub fn in_str_slice<T: AsRef<str>>(a: &[T], x: &str, case_sensitive: bool) -> i32 {
    for (idx, n) in a.iter().enumerate() {
        let n = n.as_ref();
        if (!case_sensitive && n.eq_ignore_ascii_case(x)) || (case_sensitive && n == x) {
            return idx as i32;
        }
    }
    -1
}

/// Returns the index of `x` in `a`, or `-1` if not found.
pub fn in_float64_slice(a: &[f64], x: f64) -> i32 {
    for (idx, n) in a.iter().enumerate() {
        if *n == x {
            return idx as i32;
        }
    }
    -1
}

// ------------------------------------------------------------------
// Numeric parsing
// ------------------------------------------------------------------

/// Determines whether `s` is a valid numeric expression and returns its
/// precision and value.
pub fn is_numeric(s: &str) -> (bool, i32, f64) {
    if s.contains('_') {
        return (false, 0, 0.0);
    }
    if let Ok(flt) = s.parse::<f64>() {
        let no_scientific = format!("{flt:.15}").trim_end_matches('0').to_string();
        let precision = no_scientific.len() as i32 - no_scientific.matches('.').count() as i32;
        return (true, precision, flt);
    }
    (false, 0, 0.0)
}

// ------------------------------------------------------------------
// String utilities
// ------------------------------------------------------------------

/// Counts the number of UTF-16 code units in a string.
pub fn count_utf16_string(s: &str) -> usize {
    s.chars().map(|r| r.len_utf16()).sum()
}

/// Truncates a string to a maximum number of UTF-16 code units.
pub fn truncate_utf16_units(s: &str, max: usize) -> String {
    let mut cnt = 0usize;
    s.chars()
        .take_while(|r| {
            let len = r.len_utf16();
            if cnt + len > max {
                return false;
            }
            cnt += len;
            true
        })
        .collect()
}

// ------------------------------------------------------------------
// Map helpers
// ------------------------------------------------------------------

/// Helper to build a `HashMap<String, String>` from a static slice of pairs.
pub fn str_map(pairs: &[(&str, &str)]) -> HashMap<String, String> {
    pairs
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

/// Returns a reference-counted pointer-like `Box` containing `true` or `false`.
pub fn bool_ptr(b: bool) -> Option<bool> {
    Some(b)
}

/// Returns a `Some` wrapper for an integer value.
pub fn int_ptr(i: i64) -> Option<i64> {
    Some(i)
}

/// Returns a `Some` wrapper for a string value.
pub fn string_ptr(s: impl Into<String>) -> Option<String> {
    Some(s.into())
}

/// Returns a `Some` wrapper for an unsigned 32-bit value.
pub fn uint_ptr(u: u32) -> Option<u32> {
    Some(u)
}

/// Returns a `Some` wrapper for a `f64` value.
pub fn float64_ptr(f: f64) -> Option<f64> {
    Some(f)
}

// ------------------------------------------------------------------
// Binary basic string (bstr) escape handling
// ------------------------------------------------------------------

static BSTR_EXP: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"_x[a-fA-F\d]{4}_").unwrap());
static BSTR_ESCAPE_EXP: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"x[a-fA-F\d]{4}_").unwrap());

fn unquote_unicode_hex(hex: &str) -> Option<String> {
    let code = u32::from_str_radix(hex, 16).ok()?;
    char::from_u32(code).map(|c| c.to_string())
}

/// Parses the binary basic string, trimming escaped literals which are not
/// permitted in an XML 1.0 document. Escapes have the form `_xHHHH_`.
pub fn bstr_unmarshal(s: &str) -> String {
    if !s.contains("_x") {
        return s.to_string();
    }
    let mut result = String::with_capacity(s.len());
    let mut cursor = 0;
    for m in BSTR_EXP.find_iter(s) {
        result.push_str(&s[cursor..m.start()]);
        let sub = m.as_str();
        if sub == "_x005F_" {
            cursor = m.end();
            result.push('_');
            continue;
        }
        if let Some(ch) = unquote_unicode_hex(&s[m.start() + 2..m.end() - 1]) {
            cursor = m.end();
            result.push_str(&ch);
        }
    }
    if cursor < s.len() {
        result.push_str(&s[cursor..]);
    }
    result
}

/// Encodes characters as binary basic string escapes so that the value can be
/// stored in an XML 1.0 document.
pub fn bstr_marshal(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut cursor = 0;
    let len = s.len();
    for m in BSTR_EXP.find_iter(s) {
        result.push_str(&s[cursor..m.start()]);
        let sub = m.as_str();
        if sub == "_x005F_" {
            cursor = m.end();
            if cursor + 6 <= len && BSTR_ESCAPE_EXP.is_match(&s[cursor..cursor + 6]) {
                let hex = &s[cursor + 1..cursor + 5];
                if unquote_unicode_hex(hex).is_some() {
                    result.push_str(sub);
                    result.push_str("x005F");
                    result.push_str(sub);
                    continue;
                }
            }
            result.push_str(sub);
            result.push_str("x005F_");
            continue;
        }
        if unquote_unicode_hex(&s[m.start() + 2..m.end() - 1]).is_some() {
            cursor = m.end();
            result.push_str("_x005F");
            result.push_str(sub);
        }
    }
    if cursor < s.len() {
        result.push_str(&s[cursor..]);
    }
    result
}

// ------------------------------------------------------------------
// Fraction conversion
// ------------------------------------------------------------------

/// Converts a floating-point number to a fraction string representation with
/// the specified placeholder widths for numerator and denominator.
pub fn float_to_fraction(
    x: f64,
    numerator_placeholder: i64,
    denominator_placeholder: i64,
) -> String {
    if denominator_placeholder <= 0 {
        return String::new();
    }
    let denominator_limit = 10_i64.pow(denominator_placeholder as u32);
    let (num, den) = float_to_frac_use_continued_fraction(x, denominator_limit);
    if num == 0 {
        return " ".repeat((numerator_placeholder + denominator_placeholder + 1) as usize);
    }
    let num_str = num.to_string();
    let den_str = den.to_string();
    let numerator_pad = cmp::max(numerator_placeholder - num_str.len() as i64, 0) as usize;
    let denominator_pad = cmp::max(denominator_placeholder - den_str.len() as i64, 0) as usize;
    format!(
        "{}{}/{}{}",
        " ".repeat(numerator_pad),
        num_str,
        den_str,
        " ".repeat(denominator_pad)
    )
}

/// Converts a floating-point decimal to a fraction using continued fractions
/// and recurrence relations.
pub fn float_to_frac_use_continued_fraction(r: f64, denominator_limit: i64) -> (i64, i64) {
    // Use i128 for intermediate values to avoid the signed 64-bit overflow
    // that Go silently wraps. The final numerator/denominator still fit in i64.
    let mut p1: i128 = 1;
    let mut q1: i128 = 0;
    let mut p2: i128 = 0;
    let mut q2: i128 = 1;
    let mut last_a: i128 = 0;
    let mut last_b: i128 = 0;
    let mut r = r;
    let limit = denominator_limit as i128;
    loop {
        let a = r.floor() as i128;
        let cur_a = a * p1 + p2;
        let cur_b = a * q1 + q2;
        p2 = p1;
        q2 = q1;
        p1 = cur_a;
        q1 = cur_b;
        let frac = r - a as f64;
        if q1 >= limit {
            return (last_a as i64, last_b as i64);
        }
        if frac.abs() < 1e-12 {
            return (cur_a as i64, cur_b as i64);
        }
        last_a = cur_a;
        last_b = cur_b;
        r = 1.0 / frac;
    }
}

// ------------------------------------------------------------------
// Stack
// ------------------------------------------------------------------

/// A simple stack abstraction.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Stack<T> {
    items: Vec<T>,
}

impl<T> Stack<T> {
    /// Creates a new empty stack.
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    /// Pushes a value onto the top of the stack.
    pub fn push(&mut self, value: T) {
        self.items.push(value);
    }

    /// Pops the top item from the stack and returns it.
    pub fn pop(&mut self) -> Option<T> {
        self.items.pop()
    }

    /// Returns a reference to the top item without removing it.
    pub fn peek(&self) -> Option<&T> {
        self.items.last()
    }

    /// Returns the number of items in the stack.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Returns `true` if the stack contains no items.
    pub fn empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Returns `true` if the stack contains no items.
    pub fn is_empty(&self) -> bool {
        self.items.len() == 0
    }
}

impl<T> FromIterator<T> for Stack<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self {
            items: Vec::from_iter(iter),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bstr_unmarshal() {
        let cases = [
            ("*_x0008_", "*\u{0008}"),
            ("*_x0008_*", "*\u{0008}*"),
            ("*_x005F__x0008_*", "*_\u{0008}*"),
            ("*_x005F_x0001_*", "*_x0001_*"),
            ("*_x005f_x005F__x0008_*", "*_x005F_\u{0008}*"),
            ("*_x005F_x005F_xG05F_x0006_*", "*_x005F_xG05F\u{0006}*"),
            ("*_x005F_x005F_x005F_x0006_*", "*_x005F_x0006_*"),
            ("_x005F__x0008_******", "_\u{0008}******"),
            ("******_x005F__x0008_", "******_\u{0008}"),
            ("_x000x_x005F_x000x_", "_x000x_x000x_"),
        ];
        for (input, expected) in cases {
            assert_eq!(bstr_unmarshal(input), expected, "input: {}", input);
        }
    }

    #[test]
    fn test_bstr_marshal() {
        let cases = [
            ("*_x0008_*", "*_x005F_x0008_*"),
            ("*_x005F_*", "*_x005F_x005F_*"),
            ("*_x005F_xG006_*", "*_x005F_x005F_xG006_*"),
            ("*_x005F_x0006_*", "*_x005F_x005F_x005F_x0006_*"),
        ];
        for (input, expected) in cases {
            assert_eq!(bstr_marshal(input), expected, "input: {}", input);
        }
    }

    #[test]
    fn test_float_to_fraction() {
        assert_eq!(float_to_fraction(0.19, 0, 0), "");
        assert_eq!(float_to_fraction(0.19, 1, 1), "1/5");
        assert_eq!(float_to_fraction(0.9999, 10, 10).trim(), "9999/10000");
        assert_eq!(
            float_to_fraction(std::f64::consts::E, 1, 18),
            "954888175898973913/351283728530932463"
        );
    }

    #[test]
    fn test_float_to_frac_use_continued_fraction() {
        assert_eq!(float_to_frac_use_continued_fraction(0.19, 10), (1, 5));
        assert_eq!(
            float_to_frac_use_continued_fraction(0.9999, 10_000_000_000),
            (9999, 10000)
        );
    }

    #[test]
    fn test_count_and_truncate_utf16() {
        let s = "a\u{10000}b"; // surrogate pair in the middle
        assert_eq!(count_utf16_string(s), 4);
        assert_eq!(truncate_utf16_units(s, 1), "a");
        assert_eq!(truncate_utf16_units(s, 2), "a");
        assert_eq!(truncate_utf16_units(s, 3), "a\u{10000}");
        assert_eq!(truncate_utf16_units(s, 4), s);
    }

    #[test]
    fn test_ptr_helpers() {
        assert_eq!(uint_ptr(42), Some(42));
        assert_eq!(float64_ptr(1.5), Some(1.5));
        assert_eq!(bool_ptr(false), Some(false));
        assert_eq!(int_ptr(-7), Some(-7));
    }

    #[test]
    fn test_stack() {
        let mut stack = Stack::new();
        assert!(stack.is_empty());
        assert!(stack.empty());
        stack.push(1);
        stack.push(2);
        stack.push(3);
        assert_eq!(stack.len(), 3);
        assert_eq!(stack.peek(), Some(&3));
        assert_eq!(stack.pop(), Some(3));
        assert_eq!(stack.pop(), Some(2));
        assert_eq!(stack.len(), 1);
        assert!(!stack.is_empty());
        assert_eq!(stack.pop(), Some(1));
        assert_eq!(stack.pop(), None);
    }
}
