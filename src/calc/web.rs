//! Web and miscellaneous formula functions.

use std::collections::HashMap;

use crate::calc::arg::*;
use crate::calc::{CalcContext, FormulaFn};

pub fn register(m: &mut HashMap<&'static str, FormulaFn>) {
    m.insert("ENCODEURL", encodeurl);
}

fn encodeurl(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let text = args[0].value();
    new_string_formula_arg(urlencode(&text))
}

fn urlencode(s: &str) -> String {
    let mut out = String::new();
    for byte in s.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char)
            }
            _ => out.push_str(&format!("%{:02X}", byte)),
        }
    }
    out
}
