//! Text formula functions.

use std::collections::HashMap;

use regex::Regex;

use crate::calc::arg::*;
use crate::calc::{CalcContext, FormulaFn};
use crate::constants::{MAX_FIELD_LENGTH, TOTAL_CELL_CHARS};
use crate::date::{date_to_excel_serial, time_to_excel_serial};
use crate::numfmt::format_number;

// Culture identifiers mirror the Go `CultureName` iota values.
const CULTURE_NAME_JA_JP: u8 = 2;
const CULTURE_NAME_ZH_CN: u8 = 4;
const CULTURE_NAME_ZH_TW: u8 = 5;

pub fn register(m: &mut HashMap<&'static str, FormulaFn>) {
    m.insert("CONCATENATE", concatenate);
    m.insert("CONCAT", concat);
    m.insert("ARRAYTOTEXT", arraytotext);
    m.insert("BAHTTEXT", bahttext);
    m.insert("CHAR", char_fn);
    m.insert("CLEAN", clean);
    m.insert("CODE", code);
    m.insert("DBCS", dbcs);
    m.insert("EXACT", exact);
    m.insert("FIXED", fixed);
    m.insert("FIND", find);
    m.insert("FINDB", findb);
    m.insert("LEFT", left);
    m.insert("LEFTB", leftb);
    m.insert("LEN", len);
    m.insert("LENB", lenb);
    m.insert("LOWER", lower);
    m.insert("MID", mid);
    m.insert("MIDB", midb);
    m.insert("PROPER", proper);
    m.insert("REPLACE", replace);
    m.insert("REPLACEB", replaceb);
    m.insert("REPT", rept);
    m.insert("RIGHT", right);
    m.insert("RIGHTB", rightb);
    m.insert("SEARCH", search);
    m.insert("SEARCHB", searchb);
    m.insert("SUBSTITUTE", substitute);
    m.insert("TEXT", text);
    m.insert("TEXTAFTER", textafter);
    m.insert("TEXTBEFORE", textbefore);
    m.insert("TEXTJOIN", textjoin);
    m.insert("TRIM", trim);
    m.insert("UNICHAR", unichar);
    m.insert("UNICODE", unicode);
    m.insert("UNIQUE", unique);
    m.insert("UPPER", upper);
    m.insert("VALUE", value);
    m.insert("VALUETOTEXT", valuetotext);
}

fn concatenate(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    let mut out = String::new();
    for a in args {
        out.push_str(&flatten_string(a));
    }
    new_string_formula_arg(out)
}

fn concat(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    concatenate(_ctx, args)
}

// ------------------------------------------------------------------
// ARRAYTOTEXT / VALUETOTEXT helpers
// ------------------------------------------------------------------

fn prepare_to_text(_name: &str, args: &[FormulaArg]) -> FormulaArg {
    if args.is_empty() {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    if args.len() > 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let mut format = new_number_formula_arg(0.0);
    if args.len() == 2 {
        format = args[1].to_number();
        if format.typ != ArgType::Number {
            return format;
        }
    }
    if format.number != 0.0 && format.number != 1.0 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    format
}

fn arraytotext(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    let format = prepare_to_text("ARRAYTOTEXT", args);
    if format.typ != ArgType::Number {
        return format;
    }
    let first = &args[0];
    let matrix = if first.typ == ArgType::Matrix {
        &first.matrix
    } else {
        return new_string_formula_arg(first.value());
    };
    let mut mtx: Vec<Vec<String>> = Vec::new();
    for rows in matrix {
        let mut row: Vec<String> = Vec::new();
        for cell in rows {
            if cell.to_number().typ != ArgType::Number && format.number == 1.0 {
                row.push(format!("\"{}\"", cell.value()));
            } else {
                row.push(cell.value());
            }
        }
        mtx.push(row);
    }
    let mut text: Vec<String> = Vec::new();
    for row in &mtx {
        if format.number == 1.0 {
            text.push(row.join(","));
        } else {
            text.push(row.join(", "));
        }
    }
    if format.number == 1.0 {
        new_string_formula_arg(format!("{{{}}}", text.join(";")))
    } else {
        new_string_formula_arg(text.join(", "))
    }
}

fn valuetotext(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    let format = prepare_to_text("VALUETOTEXT", args);
    if format.typ != ArgType::Number {
        return format;
    }
    let cell = &args[0];
    if cell.to_number().typ != ArgType::Number && format.number == 1.0 {
        return new_string_formula_arg(format!("\"{}\"", cell.value()));
    }
    new_string_formula_arg(cell.value())
}

// ------------------------------------------------------------------
// BAHTTEXT
// ------------------------------------------------------------------

const TH_0: &str = "\u{0E28}\u{0E39}\u{0E19}\u{0E22}\u{0E4C}";
const TH_1: &str = "\u{0E2B}\u{0E19}\u{0E36}\u{0E48}\u{0E07}";
const TH_2: &str = "\u{0E2A}\u{0E2D}\u{0E07}";
const TH_3: &str = "\u{0E2A}\u{0E32}\u{0E21}";
const TH_4: &str = "\u{0E2A}\u{0E35}\u{0E48}";
const TH_5: &str = "\u{0E2B}\u{0E49}\u{0E32}";
const TH_6: &str = "\u{0E2B}\u{0E01}";
const TH_7: &str = "\u{0E40}\u{0E08}\u{0E47}\u{0E14}";
const TH_8: &str = "\u{0E41}\u{0E1B}\u{0E14}";
const TH_9: &str = "\u{0E40}\u{0E01}\u{0E49}\u{0E32}";
const TH_10: &str = "\u{0E2A}\u{0E34}\u{0E1A}";
const TH_11: &str = "\u{0E40}\u{0E2D}\u{0E47}\u{0E14}";
const TH_20: &str = "\u{0E22}\u{0E35}\u{0E48}";
const TH_1E2: &str = "\u{0E23}\u{0E49}\u{0E2D}\u{0E22}";
const TH_1E3: &str = "\u{0E1E}\u{0E31}\u{0E19}";
const TH_1E4: &str = "\u{0E2B}\u{0E21}\u{0E37}\u{0E48}\u{0E19}";
const TH_1E5: &str = "\u{0E41}\u{0E2A}\u{0E19}";
const TH_1E6: &str = "\u{0E25}\u{0E49}\u{0E32}\u{0E19}";
const TH_DOT_0: &str = "\u{0E16}\u{0E49}\u{0E27}\u{0E19}";
const TH_BAHT: &str = "\u{0E1A}\u{0E32}\u{0E17}";
const TH_SATANG: &str = "\u{0E2A}\u{0E15}\u{0E32}\u{0E07}\u{0E04}\u{0E4C}";
const TH_MINUS: &str = "\u{0E25}\u{0E1A}";

const TH_DIGITS: [&str; 10] = [TH_0, TH_1, TH_2, TH_3, TH_4, TH_5, TH_6, TH_7, TH_8, TH_9];

fn bahttext_append_digit(text: &str, digit: usize) -> String {
    if digit <= 9 {
        format!("{}{}", text, TH_DIGITS[digit])
    } else {
        text.to_string()
    }
}

fn bahttext_append_pow10(text: &str, digit: usize, pow10: usize) -> String {
    let mut text = bahttext_append_digit(text, digit);
    match pow10 {
        2 => text.push_str(TH_1E2),
        3 => text.push_str(TH_1E3),
        4 => text.push_str(TH_1E4),
        5 => text.push_str(TH_1E5),
        _ => {}
    }
    text
}

fn bahttext_append_block(text: &str, val: usize) -> String {
    let mut text = text.to_string();
    let mut val = val;
    if val >= 100000 {
        text = bahttext_append_pow10(&text, val / 100000, 5);
        val %= 100000;
    }
    if val >= 10000 {
        text = bahttext_append_pow10(&text, val / 10000, 4);
        val %= 10000;
    }
    if val >= 1000 {
        text = bahttext_append_pow10(&text, val / 1000, 3);
        val %= 1000;
    }
    if val >= 100 {
        text = bahttext_append_pow10(&text, val / 100, 2);
        val %= 100;
    }
    if val > 0 {
        let n10 = val / 10;
        let n1 = val % 10;
        if n10 >= 1 {
            if n10 >= 3 {
                text = bahttext_append_digit(&text, n10);
            } else if n10 == 2 {
                text.push_str(TH_20);
            }
            text.push_str(TH_10);
        }
        if n10 > 0 && n1 == 1 {
            text.push_str(TH_11);
        } else if n1 > 0 {
            text = bahttext_append_digit(&text, n1);
        }
    }
    text
}

fn split_block(val: f64, size: f64) -> (f64, usize) {
    let div = (val + 0.1) / size;
    let integer = div.trunc();
    let frac = (div - integer) * size + 0.1;
    (integer, frac as usize)
}

fn bahttext(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let number = args[0].to_number();
    if number.typ != ArgType::Number {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let minus = number.number < 0.0;
    let num = (number.number.abs() * 100.0).floor() + 0.5;
    let (mut baht, satang) = split_block(num, 100.0);
    let mut text = String::new();
    if baht == 0.0 {
        if satang == 0 {
            text.push_str(TH_0);
        }
    } else {
        while baht > 0.0 {
            let mut block = String::new();
            let n_block;
            (baht, n_block) = split_block(baht, 1_000_000.0);
            if n_block > 0 {
                block = bahttext_append_block(&block, n_block);
            }
            if baht > 0.0 {
                block = format!("{}{}", TH_1E6, block);
            }
            text = format!("{}{}", block, text);
        }
    }
    if !text.is_empty() {
        text.push_str(TH_BAHT);
    }
    if satang == 0 {
        text.push_str(TH_DOT_0);
    } else {
        text = bahttext_append_block(&text, satang);
        text.push_str(TH_SATANG);
    }
    if minus {
        text = format!("{}{}", TH_MINUS, text);
    }
    new_string_formula_arg(text)
}

// ------------------------------------------------------------------
// CHAR / UNICHAR / CODE / UNICODE
// ------------------------------------------------------------------

fn char_fn(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let arg = args[0].to_number();
    if arg.typ != ArgType::Number {
        return arg;
    }
    let num = arg.number as usize;
    if num > MAX_FIELD_LENGTH {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    if let Some(c) = char::from_u32(num as u32) {
        new_string_formula_arg(c.to_string())
    } else {
        new_error_formula_arg(FORMULA_ERROR_VALUE)
    }
}

fn unichar(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let num_arg = args[0].to_number();
    if num_arg.typ != ArgType::Number {
        return num_arg;
    }
    if num_arg.number <= 0.0 || num_arg.number > 55295.0 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    if let Some(c) = char::from_u32(num_arg.number as u32) {
        new_string_formula_arg(c.to_string())
    } else {
        new_error_formula_arg(FORMULA_ERROR_VALUE)
    }
}

fn code_fn(_ctx: &CalcContext, args: &[FormulaArg], name: &str) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let text = args[0].value();
    if text.is_empty() {
        if name == "CODE" {
            return new_number_formula_arg(0.0);
        }
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    new_number_formula_arg(text.as_bytes()[0] as f64)
}

fn code(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    code_fn(_ctx, args, "CODE")
}

fn unicode(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    code_fn(_ctx, args, "UNICODE")
}

// ------------------------------------------------------------------
// CLEAN / LOWER / UPPER / TRIM
// ------------------------------------------------------------------

fn clean(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let out: String = args[0]
        .value()
        .chars()
        .filter(|&c| c > '\u{001F}')
        .collect();
    new_string_formula_arg(out)
}

fn lower(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    new_string_formula_arg(args[0].value().to_lowercase())
}

fn upper(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    new_string_formula_arg(args[0].value().to_uppercase())
}

fn proper(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let mut out = String::new();
    let mut is_letter = false;
    for c in args[0].value().chars() {
        if !is_letter && c.is_alphabetic() {
            out.push(c.to_uppercase().next().unwrap_or(c));
        } else {
            out.push(c.to_lowercase().next().unwrap_or(c));
        }
        is_letter = c.is_alphabetic();
    }
    new_string_formula_arg(out)
}

fn trim(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    new_string_formula_arg(args[0].value().trim().to_string())
}

// ------------------------------------------------------------------
// DBCS
// ------------------------------------------------------------------

fn dbcs(ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let arg = &args[0];
    if arg.typ == ArgType::Error {
        return arg.clone();
    }
    let culture = ctx.file.options.lock().unwrap().culture_info;
    if culture == CULTURE_NAME_JA_JP
        || culture == CULTURE_NAME_ZH_CN
        || culture == CULTURE_NAME_ZH_TW
    {
        let mut chars = String::new();
        for r in arg.value().chars() {
            let mut code = r as u32;
            if code == 32 {
                code = 12288;
            } else {
                code += 65248;
            }
            if (code < 32 || code > 126) && r != '\u{00A5}' && code < 65381 {
                chars.push(char::from_u32(code).unwrap_or(r));
            } else {
                chars.push(r);
            }
        }
        return new_string_formula_arg(chars);
    }
    arg.clone()
}

// ------------------------------------------------------------------
// EXACT / REPT
// ------------------------------------------------------------------

fn exact(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    new_bool_formula_arg(args[0].value() == args[1].value())
}

fn rept(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let text = &args[0];
    if text.typ != ArgType::String {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let times = args[1].to_number();
    if times.typ != ArgType::Number {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    if times.number < 0.0 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    if times.number == 0.0 {
        return new_string_formula_arg("");
    }
    new_string_formula_arg(text.string.repeat(times.number as usize))
}

// ------------------------------------------------------------------
// FIXED
// ------------------------------------------------------------------

fn format_with_commas(value: f64, precision: usize) -> String {
    let rounded = (value * 10f64.powi(precision as i32)).round() / 10f64.powi(precision as i32);
    let sign = if rounded < 0.0 { "-" } else { "" };
    let abs = rounded.abs();
    let int_part = abs.trunc() as i64;
    let frac_part = (abs.fract() * 10f64.powi(precision as i32)).round() as i64;
    let int_str = int_part.to_string();
    let mut with_commas = String::new();
    for (i, c) in int_str.chars().enumerate() {
        if i > 0 && (int_str.len() - i) % 3 == 0 {
            with_commas.push(',');
        }
        with_commas.push(c);
    }
    let mut result = format!("{}{}", sign, with_commas);
    if precision > 0 {
        result.push('.');
        result.push_str(&format!("{:0precision$}", frac_part, precision = precision));
    }
    result
}

fn fixed(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.is_empty() {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    if args.len() > 3 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let num_arg = args[0].to_number();
    if num_arg.typ != ArgType::Number {
        return num_arg;
    }
    let precision: usize;
    let mut decimals = 0i32;
    let value_str = args[0].value();
    let s: Vec<&str> = value_str.split('.').collect();
    if args.len() == 1 && s.len() == 2 {
        decimals = s[1].len() as i32;
    }
    if args.len() >= 2 {
        let d = args[1].to_number();
        if d.typ != ArgType::Number {
            return d;
        }
        decimals = d.number as i32;
    }
    let mut no_commas = false;
    if args.len() == 3 {
        let nc = args[2].to_bool();
        if nc.typ == ArgType::Error {
            return nc;
        }
        no_commas = nc.boolean;
    }
    let n = 10f64.powi(decimals);
    let r = num_arg.number * n;
    let fixed = (r + 0.5f64.copysign(r)).trunc() / n;
    if decimals > 0 {
        precision = decimals as usize;
    } else {
        precision = 0;
    }
    if no_commas {
        new_string_formula_arg(format!("{:.*}", precision, fixed))
    } else {
        new_string_formula_arg(format_with_commas(fixed, precision))
    }
}

// ------------------------------------------------------------------
// UTF-16 helpers (mirror Go's utf16 helpers)
// ------------------------------------------------------------------

fn count_utf16_string(s: &str) -> usize {
    s.encode_utf16().count()
}

fn truncate_utf16_units(s: &str, length: usize) -> String {
    let mut cnt = 0;
    let mut out = String::new();
    for c in s.chars() {
        let units = c.encode_utf16(&mut [0; 2]).len();
        if cnt + units > length {
            break;
        }
        out.push(c);
        cnt += units;
    }
    out
}

// ------------------------------------------------------------------
// FIND / SEARCH / FINDB / SEARCHB
// ------------------------------------------------------------------

fn match_pattern_to_regexp(find_text: &str, dbcs: bool) -> (String, bool) {
    let mark = if dbcs {
        r"(?:(?:[\x00-\x81])|(?:[\xFF61-\xFFA0])|(?:[\xF8F1-\xF8F4])|[0-9A-Za-z])"
    } else {
        "."
    };
    let mut exp = String::from("^");
    let mut wild_card = false;
    for c in find_text.chars() {
        match c {
            '.' | '+' | '$' | '^' | '[' | ']' | '(' | ')' | '{' | '}' | '|' | '/' => {
                exp.push('\\');
                exp.push(c);
            }
            '?' => {
                wild_card = true;
                exp.push_str(mark);
            }
            '*' => {
                wild_card = true;
                exp.push_str(".*");
            }
            _ => exp.push(c),
        }
    }
    (exp, wild_card)
}

fn match_pattern(
    find_text: &str,
    within_text: &str,
    dbcs: bool,
    start_num: usize,
) -> (usize, bool) {
    let (exp, wild_card) = match_pattern_to_regexp(find_text, dbcs);
    let re = Regex::new(&exp).unwrap();
    let mut offset = 1;
    for (idx, _) in within_text.char_indices() {
        if offset < start_num {
            offset += 1;
            continue;
        }
        if wild_card && re.is_match(&within_text[idx..]) {
            break;
        }
        if within_text[idx..].starts_with(find_text) {
            break;
        }
        offset += 1;
    }
    let ok = count_utf16_string(within_text) != offset - 1;
    (offset, ok)
}

fn find_impl(name: &str, args: &[FormulaArg]) -> FormulaArg {
    if args.len() < 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    if args.len() > 3 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let mut start_num = 1usize;
    if args.len() == 3 {
        let num_arg = args[2].to_number();
        if num_arg.typ != ArgType::Number {
            return num_arg;
        }
        if num_arg.number < 0.0 {
            return new_error_formula_arg(FORMULA_ERROR_VALUE);
        }
        start_num = num_arg.number as usize;
    }
    let find_text_arg = &args[0];
    let within_text = args[1].value();
    let dbcs = name == "FINDB" || name == "SEARCHB";
    let search = name == "SEARCH" || name == "SEARCHB";
    let find = |find_text: &str| -> FormulaArg {
        if find_text.is_empty() {
            return new_number_formula_arg(start_num as f64);
        }
        let (ft, wt) = if search {
            (find_text.to_uppercase(), within_text.to_uppercase())
        } else {
            (find_text.to_string(), within_text.clone())
        };
        let (offset, ok) = match_pattern(&ft, &wt, dbcs, start_num);
        if !ok {
            return new_error_formula_arg(FORMULA_ERROR_VALUE);
        }
        let mut result = offset;
        if dbcs {
            let mut pre = 0;
            for (idx, _) in wt.char_indices() {
                if pre > offset {
                    break;
                }
                if idx - pre > 1 {
                    result += 1;
                }
                pre = idx;
            }
        }
        new_number_formula_arg(result as f64)
    };
    if find_text_arg.typ == ArgType::Matrix {
        let mut mtx: Vec<Vec<FormulaArg>> = Vec::new();
        for row in &find_text_arg.matrix {
            let mut array: Vec<FormulaArg> = Vec::new();
            for cell in row {
                array.push(find(&cell.value()));
            }
            mtx.push(array);
        }
        new_matrix_formula_arg(mtx)
    } else {
        find(&find_text_arg.value())
    }
}

fn find(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    find_impl("FIND", args)
}

fn findb(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    find_impl("FINDB", args)
}

fn search(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    find_impl("SEARCH", args)
}

fn searchb(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    find_impl("SEARCHB", args)
}

// ------------------------------------------------------------------
// LEFT / RIGHT / LEFTB / RIGHTB
// ------------------------------------------------------------------

fn left_right(name: &str, args: &[FormulaArg]) -> FormulaArg {
    if args.is_empty() {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    if args.len() > 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let text = args[0].value();
    let mut num_chars = 1usize;
    if args.len() == 2 {
        let num_arg = args[1].to_number();
        if num_arg.typ != ArgType::Number {
            return num_arg;
        }
        if num_arg.number < 0.0 {
            return new_error_formula_arg(FORMULA_ERROR_VALUE);
        }
        num_chars = num_arg.number as usize;
    }
    if name == "LEFTB" || name == "RIGHTB" {
        if text.len() > num_chars {
            if name == "LEFTB" {
                return new_string_formula_arg(text[..num_chars].to_string());
            }
            return new_string_formula_arg(text[text.len() - num_chars..].to_string());
        }
        return new_string_formula_arg(text);
    }
    let text_len = count_utf16_string(&text);
    if text_len > num_chars {
        if name == "LEFT" {
            return new_string_formula_arg(truncate_utf16_units(&text, num_chars));
        }
        let runes: Vec<char> = text.chars().collect();
        let start = runes.len() - num_chars;
        return new_string_formula_arg(runes[start..].iter().collect::<String>());
    }
    new_string_formula_arg(text)
}

fn left(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    left_right("LEFT", args)
}

fn leftb(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    left_right("LEFTB", args)
}

fn right(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    left_right("RIGHT", args)
}

fn rightb(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    left_right("RIGHTB", args)
}

fn len(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    new_number_formula_arg(count_utf16_string(&args[0].value()) as f64)
}

fn lenb(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let mut result = 0;
    for c in args[0].value().chars() {
        let b = c.len_utf8();
        if b == 1 {
            result += 1;
        } else if b > 1 {
            result += 2;
        }
    }
    new_number_formula_arg(result as f64)
}

// ------------------------------------------------------------------
// MID / MIDB
// ------------------------------------------------------------------

fn mid_impl(name: &str, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 3 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let text = args[0].value();
    let start_num_arg = args[1].to_number();
    let num_chars_arg = args[2].to_number();
    if start_num_arg.typ != ArgType::Number {
        return start_num_arg;
    }
    if num_chars_arg.typ != ArgType::Number {
        return num_chars_arg;
    }
    let start_num = start_num_arg.number as usize;
    if start_num < 1 || num_chars_arg.number < 0.0 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    if name == "MIDB" {
        let mut result = String::new();
        let mut cnt = 0usize;
        let mut offset = 0usize;
        for c in text.chars() {
            offset += 1;
            let rune_len = c.len_utf8();
            let dbcs = rune_len > 1;
            if dbcs {
                offset += 1;
            }
            if cnt == num_chars_arg.number as usize {
                break;
            }
            if offset + 1 > start_num {
                if dbcs {
                    if cnt + 2 > num_chars_arg.number as usize {
                        // Go can produce invalid UTF-8 here; Rust cannot, so
                        // append the replacement character to keep a result.
                        result.push('\u{FFFD}');
                        break;
                    }
                    result.push(c);
                    cnt += 2;
                } else {
                    result.push(c);
                    cnt += 1;
                }
            }
        }
        return new_string_formula_arg(result);
    }
    let text_len = count_utf16_string(&text);
    if start_num > text_len {
        return new_string_formula_arg("");
    }
    let start = start_num - 1;
    let end = start + num_chars_arg.number as usize;
    let runes: Vec<char> = text.chars().collect();
    if end > text_len + 1 {
        return new_string_formula_arg(runes[start..].iter().collect::<String>());
    }
    new_string_formula_arg(runes[start..end].iter().collect::<String>())
}

fn mid(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    mid_impl("MID", args)
}

fn midb(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    mid_impl("MIDB", args)
}

// ------------------------------------------------------------------
// REPLACE / REPLACEB
// ------------------------------------------------------------------

fn replace_impl(_name: &str, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 4 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let source_text = args[0].value();
    let target_text = args[3].value();
    let start_num_arg = args[1].to_number();
    let num_chars_arg = args[2].to_number();
    if start_num_arg.typ != ArgType::Number {
        return start_num_arg;
    }
    if num_chars_arg.typ != ArgType::Number {
        return num_chars_arg;
    }
    let source_text_len = source_text.len();
    let mut start_idx = start_num_arg.number as usize;
    if start_idx > source_text_len {
        start_idx = source_text_len + 1;
    }
    let mut end_idx = start_idx + num_chars_arg.number as usize;
    if end_idx > source_text_len {
        end_idx = source_text_len + 1;
    }
    if start_idx < 1 || end_idx < 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let mut result = String::new();
    result.push_str(&source_text[..start_idx - 1]);
    result.push_str(&target_text);
    result.push_str(&source_text[end_idx - 1..]);
    new_string_formula_arg(result)
}

fn replace(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    replace_impl("REPLACE", args)
}

fn replaceb(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    replace_impl("REPLACEB", args)
}

// ------------------------------------------------------------------
// SUBSTITUTE
// ------------------------------------------------------------------

fn substitute(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 3 && args.len() != 4 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let text = args[0].value();
    let source_text = args[1].value();
    let target_text = args[2].value();
    if args.len() == 3 {
        return new_string_formula_arg(text.replace(&source_text, &target_text));
    }
    let instance_num_arg = args[3].to_number();
    if instance_num_arg.typ != ArgType::Number {
        return instance_num_arg;
    }
    let instance_num = instance_num_arg.number as usize;
    if instance_num < 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let source_text_len = source_text.len();
    let mut count = instance_num;
    let mut chars = 0;
    let mut pos;
    let mut remaining = text.clone();
    loop {
        count -= 1;
        if let Some(index) = remaining.find(&source_text) {
            pos = (chars + index) as i32;
            if count == 0 {
                break;
            }
            let idx = source_text_len + index;
            chars += idx;
            remaining = remaining[idx..].to_string();
        } else {
            pos = -1;
            break;
        }
    }
    if pos == -1 {
        return new_string_formula_arg(text);
    }
    let pos = pos as usize;
    let mut result = String::new();
    result.push_str(&text[..pos]);
    result.push_str(&target_text);
    result.push_str(&text[pos + source_text_len..]);
    new_string_formula_arg(result)
}

// ------------------------------------------------------------------
// TEXT
// ------------------------------------------------------------------

/// Split a number format code into its semicolon-separated sections, respecting
/// quoted literals so that semicolons inside `"..."` do not start a new section.
fn split_format_sections(code: &str) -> Vec<String> {
    let mut sections: Vec<String> = vec![String::new()];
    let mut in_quote = false;
    let mut chars = code.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '"' {
            in_quote = !in_quote;
            sections.last_mut().unwrap().push(ch);
        } else if ch == '\\' && !in_quote {
            sections.last_mut().unwrap().push(ch);
            if let Some(next) = chars.next() {
                sections.last_mut().unwrap().push(next);
            }
        } else if ch == ';' && !in_quote {
            sections.push(String::new());
        } else {
            sections.last_mut().unwrap().push(ch);
        }
    }

    sections
}

/// A condition that may prefix a number-format section, e.g. `[>100]`.
#[derive(Debug, Clone, PartialEq)]
enum FormatCondition {
    Greater(f64),
    GreaterOrEqual(f64),
    Less(f64),
    LessOrEqual(f64),
    Equal(f64),
    NotEqual(f64),
}

impl FormatCondition {
    /// Return `true` if the value satisfies this condition.
    fn matches(&self, value: f64) -> bool {
        match self {
            FormatCondition::Greater(threshold) => value > *threshold,
            FormatCondition::GreaterOrEqual(threshold) => value >= *threshold,
            FormatCondition::Less(threshold) => value < *threshold,
            FormatCondition::LessOrEqual(threshold) => value <= *threshold,
            FormatCondition::Equal(threshold) => value == *threshold,
            FormatCondition::NotEqual(threshold) => value != *threshold,
        }
    }
}

/// Parse a leading bracketed condition from a format-section body.
///
/// Returns `Some(condition)` if the section starts with a recognized condition
/// such as `[>100]`, together with the remainder of the section after the
/// condition has been removed.
fn parse_condition(section: &str) -> (Option<FormatCondition>, &str) {
    let Some(rest) = section.strip_prefix('[') else {
        return (None, section);
    };
    let Some(end) = rest.find(']') else {
        return (None, section);
    };
    let token = &rest[..end];
    let after = &rest[end + 1..];

    let op_and_value = if token.starts_with(">=") {
        Some((">=", &token[2..]))
    } else if token.starts_with("<>") {
        // `<>` is the not-equal operator and must be checked before `<=`/`<`.
        Some(("<>", &token[2..]))
    } else if token.starts_with("<=") {
        Some(("<=", &token[2..]))
    } else if token.starts_with('<') {
        Some(("<", &token[1..]))
    } else if token.starts_with('>') {
        Some((">", &token[1..]))
    } else if token.starts_with('=') {
        Some(("=", &token[1..]))
    } else {
        None
    };

    if let Some((op, val_str)) = op_and_value {
        let val_str = val_str.trim();
        if let Ok(value) = val_str.parse::<f64>() {
            let condition = match op {
                ">" => FormatCondition::Greater(value),
                ">=" => FormatCondition::GreaterOrEqual(value),
                "<" => FormatCondition::Less(value),
                "<=" => FormatCondition::LessOrEqual(value),
                "=" => FormatCondition::Equal(value),
                "<>" => FormatCondition::NotEqual(value),
                _ => return (None, section),
            };
            return (Some(condition), after);
        }
    }

    (None, section)
}

/// Return `true` if `token` is an Excel color name such as `Red` or `Color5`.
fn is_color_token(token: &str) -> bool {
    const NAMED_COLORS: &[&str] = &[
        "Black", "Blue", "Cyan", "Green", "Magenta", "Red", "White", "Yellow",
    ];
    if NAMED_COLORS
        .iter()
        .any(|c| c.eq_ignore_ascii_case(token))
    {
        return true;
    }
    // `[Color1]` .. `[Color56]` are also valid color specifiers.
    if token.len() >= 6 && token[..5].eq_ignore_ascii_case("Color") {
        token[5..].chars().all(|c| c.is_ascii_digit())
    } else {
        false
    }
}

/// Strip leading color and condition bracket tokens from a format section.
///
/// The returned body is the format code that should be passed to the formatter,
/// and the optional condition is used to select this section.
fn strip_section_metadata(section: &str) -> (Option<FormatCondition>, String) {
    let mut section = section;
    let mut condition: Option<FormatCondition> = None;

    while let Some(rest) = section.strip_prefix('[') {
        let Some(end) = rest.find(']') else { break };
        let token = &rest[..end];

        // Conditions take precedence: only the first condition in a section is
        // meaningful, but additional color tokens may appear before or after it.
        if condition.is_none() {
            let (cond, after) = parse_condition(section);
            if let Some(c) = cond {
                condition = Some(c);
                section = after;
                continue;
            }
        }

        if is_color_token(token) {
            section = &rest[end + 1..];
            continue;
        }

        // Anything else (locale codes, elapsed-time codes, etc.) is part of
        // the format body and stops metadata stripping.
        break;
    }

    (condition, section.to_string())
}

/// Select the format section that applies to a numeric value.
///
/// Excel number formats can contain up to four sections separated by `;`:
/// positive, negative, zero, and text. Sections may also start with a condition
/// (`[>100]`) and/or a color (`[Red]`). This function returns the appropriate
/// section body (with metadata stripped) together with a flag that indicates
/// whether the selected section is the dedicated negative section; text values
/// are handled separately.
fn choose_format_section(code: &str, value: f64) -> (String, bool) {
    let sections = split_format_sections(code);
    let parsed: Vec<(Option<FormatCondition>, String)> = sections
        .iter()
        .map(|s| strip_section_metadata(s))
        .collect();

    // First, evaluate conditional sections in order.
    for (i, (cond, body)) in parsed.iter().enumerate() {
        if let Some(cond) = cond {
            if cond.matches(value) {
                let is_negative = value < 0.0 && i == 1 && sections.len() >= 2;
                return (body.clone(), is_negative);
            }
        }
    }

    // No condition matched: fall back to the default positive/negative/zero
    // rules, ignoring sections that have conditions when determining the slot
    // order.
    let default_indices: Vec<usize> = parsed
        .iter()
        .enumerate()
        .filter(|(_, (cond, _))| cond.is_none())
        .map(|(i, _)| i)
        .collect();

    let selected_idx = if value > 0.0 {
        *default_indices.get(0).unwrap_or(&0)
    } else if value < 0.0 {
        *default_indices
            .get(1)
            .unwrap_or_else(|| default_indices.get(0).unwrap_or(&0))
    } else {
        *default_indices
            .get(2)
            .unwrap_or_else(|| default_indices.get(0).unwrap_or(&0))
    };

    let is_negative = value < 0.0 && selected_idx == 1 && sections.len() >= 2;
    (parsed[selected_idx].1.clone(), is_negative)
}

/// Return the text section of a four-section format code, if present, with any
/// leading color/condition metadata stripped.
fn choose_text_section(code: &str) -> Option<String> {
    let sections = split_format_sections(code);
    if sections.len() >= 4 {
        let (_, body) = strip_section_metadata(&sections[3]);
        Some(body)
    } else {
        None
    }
}

/// Tokens used when applying a text-only number-format section.
#[derive(Debug, Clone, PartialEq)]
enum TextFormatToken {
    /// A literal character or string that should be copied to the output.
    Literal(String),
    /// The text placeholder `@`, replaced with the cell text.
    TextPlaceHolder,
    /// A zero placeholder (`0`+), also replaced with the cell text in the
    /// text section (mirrors Go `textHandler`).
    ZeroPlaceHolder,
    /// Any other number-format token (e.g. `#`, `?`, `%`, date/time codes,
    /// `General`) that is ignored in the text section.
    Other,
}

/// Characters that can form date/time codes in an Excel number format.
const DATE_TIME_CODE_CHARS: &str = "ABDEGHMRSY";

fn is_date_time_char(ch: char) -> bool {
    DATE_TIME_CODE_CHARS.contains(ch.to_ascii_uppercase())
}

fn starts_with_ignore_ascii_case(s: &str, prefix: &str) -> bool {
    s.len() >= prefix.len() && s[..prefix.len()].eq_ignore_ascii_case(prefix)
}

fn match_am_pm(s: &str) -> Option<&'static str> {
    const PATTERNS: &[&str] = &["AM/PM", "A/P", "上午/下午"];
    for pattern in PATTERNS {
        if s.len() >= pattern.len() && s[..pattern.len()].eq_ignore_ascii_case(pattern) {
            return Some(pattern);
        }
    }
    None
}

/// Drop whitespace-only literal tokens that immediately precede a token that
/// the Go `nfp` parser treats as a "swallowing" pattern (General / AM/PM).
fn drop_trailing_whitespace(tokens: &mut Vec<TextFormatToken>) {
    while let Some(TextFormatToken::Literal(s)) = tokens.last() {
        if s.chars().all(|c| c.is_whitespace()) {
            tokens.pop();
        } else {
            break;
        }
    }
}

/// Tokenize a text-section format code, mirroring the subset of `nfp` that the
/// Go `textHandler` cares about.
fn tokenize_text_format(code: &str) -> Vec<TextFormatToken> {
    let mut tokens = Vec::new();
    let bytes = code.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        let ch = bytes[i] as char;
        if ch == '"' {
            let mut lit = String::new();
            i += 1;
            while i < bytes.len() && bytes[i] as char != '"' {
                lit.push(bytes[i] as char);
                i += 1;
            }
            if i < bytes.len() {
                i += 1; // skip closing quote
            }
            tokens.push(TextFormatToken::Literal(lit));
        } else if ch == '\\' && i + 1 < bytes.len() {
            tokens.push(TextFormatToken::Literal((bytes[i + 1] as char).to_string()));
            i += 2;
        } else if ch == '[' {
            // Bracketed expressions (locale codes, elapsed-time codes, etc.)
            // are not literal text in the text section.
            i += 1;
            while i < bytes.len() && bytes[i] as char != ']' {
                i += 1;
            }
            if i < bytes.len() {
                i += 1;
            }
            tokens.push(TextFormatToken::Other);
        } else if ch == '@' {
            tokens.push(TextFormatToken::TextPlaceHolder);
            i += 1;
        } else if ch == '0' {
            while i < bytes.len() && bytes[i] as char == '0' {
                i += 1;
            }
            tokens.push(TextFormatToken::ZeroPlaceHolder);
        } else if ch == '#' {
            while i < bytes.len() && bytes[i] as char == '#' {
                i += 1;
            }
            tokens.push(TextFormatToken::Other);
        } else if ch == '?' {
            while i < bytes.len() && bytes[i] as char == '?' {
                i += 1;
            }
            tokens.push(TextFormatToken::Other);
        } else if ch == '%' {
            tokens.push(TextFormatToken::Other);
            i += 1;
        } else if ch == '.' {
            // In number-format sections a dot directly following a zero
            // placeholder is a decimal point (ignored in the text section);
            // otherwise it is treated as a literal character.
            let prev_is_zero = matches!(tokens.last(), Some(TextFormatToken::ZeroPlaceHolder));
            tokens.push(if prev_is_zero {
                TextFormatToken::Other
            } else {
                TextFormatToken::Literal(".".to_string())
            });
            i += 1;
        } else if starts_with_ignore_ascii_case(&code[i..], "General") {
            drop_trailing_whitespace(&mut tokens);
            i += "General".len();
            tokens.push(TextFormatToken::Other);
        } else if let Some(pattern) = match_am_pm(&code[i..]) {
            drop_trailing_whitespace(&mut tokens);
            i += pattern.len();
            tokens.push(TextFormatToken::Other);
        } else if is_date_time_char(ch) {
            while i < bytes.len() && is_date_time_char(bytes[i] as char) {
                i += 1;
            }
            tokens.push(TextFormatToken::Other);
        } else {
            tokens.push(TextFormatToken::Literal(ch.to_string()));
            i += 1;
        }
    }

    tokens
}

/// Apply a text-only format section to a string value.
///
/// The `@` placeholder is replaced with the cell text; `0` placeholders are
/// also replaced with the cell text (matching Go excelize). Quoted literals,
/// escaped characters and other literal text are preserved; numeric, date/time
/// and other non-text tokens are ignored.
fn apply_text_format(text: &str, code: &str) -> String {
    let mut result = String::new();
    for token in tokenize_text_format(code) {
        match token {
            TextFormatToken::Literal(s) => result.push_str(&s),
            TextFormatToken::TextPlaceHolder | TextFormatToken::ZeroPlaceHolder => {
                result.push_str(text)
            }
            TextFormatToken::Other => {}
        }
    }
    result
}

fn text(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let value = &args[0];
    let fmt_text = &args[1];
    if value.typ == ArgType::Error {
        return value.clone();
    }
    if fmt_text.typ == ArgType::Error {
        return fmt_text.clone();
    }

    let code = fmt_text.value();

    if let Some(n) = value.to_number().as_number() {
        let (section, is_negative_section) = choose_format_section(&code, n);
        // The dedicated negative section supplies its own sign (or parentheses),
        // so format the absolute value to avoid a duplicate leading minus.
        let n = if is_negative_section { n.abs() } else { n };
        return new_string_formula_arg(format_number(n, &section, false));
    }

    // Non-numeric values use the explicit text section (the fourth section),
    // if the format code provides one. Otherwise the value is returned as-is,
    // matching Go excelize behavior.
    if let Some(section) = choose_text_section(&code) {
        return new_string_formula_arg(apply_text_format(&value.value(), &section));
    }

    new_string_formula_arg(value.value())
}

// ------------------------------------------------------------------
// TEXTAFTER / TEXTBEFORE
// ------------------------------------------------------------------

fn prepare_text_after_before(_name: &str, args: &[FormulaArg]) -> FormulaArg {
    let args_len = args.len();
    if args_len < 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    if args_len > 6 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let mut text = args[0].clone();
    let mut delimiter = args[1].clone();
    let mut instance_num = new_number_formula_arg(1.0);
    let mut match_mode = new_bool_formula_arg(false);
    let mut match_end = new_bool_formula_arg(false);
    let mut if_not_found = new_empty_formula_arg();
    if args_len > 2 {
        instance_num = args[2].to_number();
        if instance_num.typ != ArgType::Number {
            return instance_num;
        }
    }
    if args_len > 3 {
        match_mode = args[3].to_bool();
        if match_mode.typ != ArgType::Number {
            return match_mode;
        }
        if match_mode.number == 1.0 {
            text = new_string_formula_arg(text.value().to_lowercase());
            delimiter = new_string_formula_arg(delimiter.value().to_lowercase());
        }
    }
    if args_len > 4 {
        match_end = args[4].to_bool();
        if match_end.typ != ArgType::Number {
            return match_end;
        }
    }
    if args_len > 5 {
        if_not_found = args[5].clone();
    }
    if text.value().is_empty() {
        return new_error_formula_arg(FORMULA_ERROR_NA);
    }
    let text_len = count_utf16_string(&args[0].value()) as f64;
    if instance_num.number == 0.0 || instance_num.number > text_len {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let reverse_search = instance_num.number < 0.0;
    let start_pos = if reverse_search { text_len } else { 0.0 };
    new_list_formula_arg(vec![
        text,
        delimiter,
        instance_num,
        match_mode,
        match_end,
        if_not_found,
        new_number_formula_arg(text_len),
        new_bool_formula_arg(reverse_search),
        new_number_formula_arg(start_pos),
    ])
}

fn text_after_before_search(
    text: &str,
    delimiter: &[String],
    start_pos: usize,
    reverse_search: bool,
) -> (isize, String) {
    let mut idx = -1isize;
    let mut modified_delimiter = String::new();
    for d in delimiter {
        let next_idx = if reverse_search {
            text[..start_pos].rfind(d).map(|i| i as isize)
        } else {
            text[start_pos..].find(d).map(|i| (i + start_pos) as isize)
        };
        if let Some(next_idx) = next_idx {
            if idx == -1
                || (((next_idx < idx && !reverse_search) || (next_idx > idx && reverse_search))
                    && idx != -1)
            {
                idx = next_idx;
                modified_delimiter = d.clone();
            }
        }
    }
    (idx, modified_delimiter)
}

fn text_after_before_result(
    name: &str,
    modified_delimiter: &str,
    text: &[char],
    found_idx: usize,
    _repeat_zero: usize,
    text_len: usize,
    match_end_active: bool,
    match_end: bool,
    reverse_search: bool,
) -> FormulaArg {
    if name == "TEXTAFTER" {
        let mut end_pos = modified_delimiter.len();
        if match_end_active && match_end && reverse_search {
            end_pos = 0;
        }
        if found_idx + end_pos >= text_len {
            return new_empty_formula_arg();
        }
        return new_string_formula_arg(
            text[found_idx + end_pos..text_len]
                .iter()
                .collect::<String>(),
        );
    }
    new_string_formula_arg(text[..found_idx].iter().collect::<String>())
}

fn text_after_before(name: &str, args: &[FormulaArg]) -> FormulaArg {
    let prepared = prepare_text_after_before(name, args);
    if prepared.typ != ArgType::List {
        return prepared;
    }
    let list = &prepared.list;
    let original_text: Vec<char> = args[0].value().chars().collect();
    let modified_text = list[0].value();
    let delimiter = vec![list[1].value()];
    let instance_num = list[2].number;
    let match_end = list[4].number == 1.0;
    let if_not_found = list[5].clone();
    let text_len = list[6].number as usize;
    let reverse_search = list[7].number == 1.0;
    let mut found_idx = -1isize;
    let mut repeat_zero = 0usize;
    let mut match_end_active = false;
    let mut modified_delimiter = String::new();
    let mut start_pos = if reverse_search { text_len } else { 0 };
    if reverse_search {
        start_pos = list[8].number as usize;
    }
    let iterations = instance_num.abs() as usize;
    for i in 0..iterations {
        let (idx, delim) =
            text_after_before_search(&modified_text, &delimiter, start_pos, reverse_search);
        if idx == 0 {
            repeat_zero += 1;
        }
        if idx == -1 {
            if match_end && i == iterations - 1 {
                found_idx = if reverse_search { 0 } else { text_len as isize };
                match_end_active = true;
            }
            break;
        }
        found_idx = idx;
        modified_delimiter = delim;
        let delim_len = modified_delimiter.len();
        if reverse_search {
            start_pos = if idx as usize >= delim_len {
                idx as usize - delim_len
            } else {
                0
            };
        } else {
            start_pos = idx as usize + delim_len;
        }
    }
    if found_idx == -1 {
        return if_not_found;
    }
    text_after_before_result(
        name,
        &modified_delimiter,
        &original_text,
        found_idx as usize,
        repeat_zero,
        text_len,
        match_end_active,
        match_end,
        reverse_search,
    )
}

fn textafter(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    text_after_before("TEXTAFTER", args)
}

fn textbefore(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    text_after_before("TEXTBEFORE", args)
}

// ------------------------------------------------------------------
// TEXTJOIN
// ------------------------------------------------------------------

fn text_join(arg: &FormulaArg, ignore_empty: bool) -> (Vec<String>, Option<FormulaArg>) {
    let mut arr = Vec::new();
    match arg.typ {
        ArgType::Error => return (arr, Some(arg.clone())),
        ArgType::String | ArgType::Empty => {
            let val = arg.value();
            if !val.is_empty() || !ignore_empty {
                arr.push(val);
            }
        }
        ArgType::Number => arr.push(arg.value()),
        ArgType::List => {
            for x in &arg.list {
                let (mut sub, err) = text_join(x, ignore_empty);
                if let Some(e) = err {
                    return (arr, Some(e));
                }
                arr.append(&mut sub);
            }
        }
        ArgType::Matrix => {
            for row in &arg.matrix {
                for cell in row {
                    let (mut sub, err) = text_join(cell, ignore_empty);
                    if let Some(e) = err {
                        return (arr, Some(e));
                    }
                    arr.append(&mut sub);
                }
            }
        }
        _ => {}
    }
    (arr, None)
}

fn textjoin(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() < 3 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    if args.len() > 252 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let delimiter = args[0].value();
    let ignore_empty_arg = args[1].to_bool();
    if ignore_empty_arg.is_error() || !ignore_empty_arg.boolean {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let ignore_empty = ignore_empty_arg.number != 0.0;
    let mut result = Vec::new();
    for arg in &args[2..] {
        let (mut sub, err) = text_join(arg, ignore_empty);
        if let Some(e) = err {
            return e;
        }
        result.append(&mut sub);
    }
    let joined = result.join(&delimiter);
    if count_utf16_string(&joined) > TOTAL_CELL_CHARS {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    new_string_formula_arg(joined)
}

// ------------------------------------------------------------------
// UNIQUE
// ------------------------------------------------------------------

fn transpose_matrix(matrix: &[Vec<FormulaArg>]) -> Vec<Vec<FormulaArg>> {
    if matrix.is_empty() {
        return Vec::new();
    }
    let rows = matrix.len();
    let cols = matrix[0].len();
    let mut out = vec![Vec::with_capacity(rows); cols];
    for r in 0..rows {
        for c in 0..cols {
            out[c].push(matrix[r][c].clone());
        }
    }
    out
}

fn unique(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.is_empty() {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    if args.len() > 3 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let matrix: Vec<Vec<FormulaArg>> = match args[0].typ {
        ArgType::Matrix => args[0].matrix.clone(),
        ArgType::List => args[0].list.iter().map(|a| vec![a.clone()]).collect(),
        _ => vec![vec![args[0].clone()]],
    };
    if matrix.is_empty() || matrix[0].is_empty() {
        return new_matrix_formula_arg(Vec::new());
    }
    let mut rows = matrix.len();
    let mut cols = matrix[0].len();
    let mut data = matrix;
    let by_column = args.get(1).map(|a| a.as_bool()).unwrap_or(false);
    let exactly_once = args.get(2).map(|a| a.as_bool()).unwrap_or(false);
    if by_column {
        data = transpose_matrix(&data);
        std::mem::swap(&mut rows, &mut cols);
    }
    let mut counts: HashMap<String, usize> = HashMap::new();
    for row in &data {
        let key = row.iter().map(|c| c.value()).collect::<String>();
        *counts.entry(key).or_insert(0) += 1;
    }
    let mut unique_axes: Vec<Vec<FormulaArg>> = Vec::new();
    for row in &data {
        let key = row.iter().map(|c| c.value()).collect::<String>();
        let cnt = counts.get(&key).copied().unwrap_or(0);
        if (exactly_once && cnt == 1) || (!exactly_once && cnt >= 1) {
            unique_axes.push(row.clone());
        }
        counts.remove(&key);
    }
    if by_column {
        unique_axes = transpose_matrix(&unique_axes);
    }
    new_matrix_formula_arg(unique_axes)
}

// ------------------------------------------------------------------
// VALUE
// ------------------------------------------------------------------

fn value(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let text0 = args[0].value();
    let text = text0.replace(',', "");
    let mut percent = 1.0;
    let mut s = text;
    if s.ends_with('%') {
        percent = 0.01;
        s = s[..s.len() - 1].to_string();
    }
    if let Ok(n) = s.parse::<f64>() {
        return new_number_formula_arg(n * percent);
    }
    let date1904 = false;
    let date_time_formats = [
        "%m/%d/%Y %H:%M:%S",
        "%Y-%m-%d %H:%M:%S",
        "%m/%d/%Y %H:%M",
        "%Y-%m-%d %H:%M",
    ];
    for fmt in &date_time_formats {
        if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(&s, fmt) {
            return new_number_formula_arg(
                date_to_excel_serial(dt.date(), date1904) + time_to_excel_serial(dt.time()),
            );
        }
    }
    let date_formats = ["%m/%d/%Y", "%Y-%m-%d"];
    for fmt in &date_formats {
        if let Ok(d) = chrono::NaiveDate::parse_from_str(&s, fmt) {
            return new_number_formula_arg(date_to_excel_serial(d, date1904));
        }
    }
    let time_formats = ["%H:%M:%S", "%H:%M"];
    for fmt in &time_formats {
        if let Ok(t) = chrono::NaiveTime::parse_from_str(&s, fmt) {
            return new_number_formula_arg(time_to_excel_serial(t));
        }
    }
    new_error_formula_arg(FORMULA_ERROR_VALUE)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_choose_format_section() {
        assert_eq!(
            choose_format_section("0.00", 1.0),
            ("0.00".to_string(), false)
        );
        assert_eq!(
            choose_format_section("0.00;(0.00)", 5.0),
            ("0.00".to_string(), false)
        );
        assert_eq!(
            choose_format_section("0.00;(0.00)", -5.0),
            ("(0.00)".to_string(), true)
        );
        assert_eq!(
            choose_format_section("0.00;(0.00);zero", 0.0),
            ("zero".to_string(), false)
        );
        assert_eq!(
            choose_format_section("0.00;(0.00);zero;@", 1.0),
            ("0.00".to_string(), false)
        );
        assert_eq!(
            choose_format_section("0.00;(0.00);zero;@", -1.0),
            ("(0.00)".to_string(), true)
        );
        assert_eq!(
            choose_format_section("0.00;(0.00);zero;@", 0.0),
            ("zero".to_string(), false)
        );
    }

    #[test]
    fn test_apply_text_format() {
        assert_eq!(apply_text_format("abc", "@"), "abc");
        assert_eq!(apply_text_format("abc", "\"prefix \"@"), "prefix abc");
        // Zero placeholders are replaced with the text, just like @. Consecutive
        // zeros form a single placeholder token.
        assert_eq!(apply_text_format("abc", "@ 0000"), "abc abc");
        assert_eq!(
            apply_text_format("abc", "0000-00-00 @"),
            "abc-abc-abc abc"
        );
        assert_eq!(
            apply_text_format("abc", "@ on 0000-00-00"),
            "abc on abc-abc-abc"
        );
        assert_eq!(apply_text_format("abc", "@ 0.00"), "abc abcabc");
        // Hash, question-mark and percent placeholders are ignored in text sections.
        assert_eq!(apply_text_format("abc", "@ # ##0"), "abc  abc");
        assert_eq!(apply_text_format("abc", "@ ???"), "abc ");
        assert_eq!(apply_text_format("abc", "@ 0%"), "abc abc");
        assert_eq!(apply_text_format("abc", "@ $0.00"), "abc $abcabc");
        // Date/time codes are ignored.
        assert_eq!(apply_text_format("abc", "@ on yyyy-mm-dd"), "abc on --");
        assert_eq!(apply_text_format("abc", "yyyy-mm-dd @"), "-- abc");
        // General token is ignored.
        assert_eq!(apply_text_format("abc", "@ General"), "abc");
        assert_eq!(apply_text_format("abc", "General @"), " abc");
        // Bracketed expressions are ignored.
        assert_eq!(apply_text_format("abc", "@ [red]"), "abc ");
    }

    #[test]
    fn test_choose_format_section_conditional() {
        // Condition selects the first section.
        assert_eq!(
            choose_format_section("[>100]0.00;0.0", 150.0),
            ("0.00".to_string(), false)
        );
        // Default section is used when the condition is not met.
        assert_eq!(
            choose_format_section("[>100]0.00;0.0", 50.0),
            ("0.0".to_string(), false)
        );
        // Color and condition combined.
        assert_eq!(
            choose_format_section("[Red][>100]0.00;0.00", 50.0),
            ("0.00".to_string(), false)
        );
        assert_eq!(
            choose_format_section("[Red][>100]0.00;0.00", 150.0),
            ("0.00".to_string(), false)
        );
        // Multiple conditional sections with text bodies.
        assert_eq!(
            choose_format_section("[>=90]\"A\";[>=60]\"B\";\"C\"", 85.0),
            ("\"B\"".to_string(), false)
        );
        assert_eq!(
            choose_format_section("[>=90]\"A\";[>=60]\"B\";\"C\"", 95.0),
            ("\"A\"".to_string(), false)
        );
        assert_eq!(
            choose_format_section("[>=90]\"A\";[>=60]\"B\";\"C\"", 55.0),
            ("\"C\"".to_string(), false)
        );
        // Negative default section with a leading color.
        assert_eq!(
            choose_format_section("[>0]0.00;[Red](0.00)", -10.0),
            ("(0.00)".to_string(), true)
        );
    }
}
