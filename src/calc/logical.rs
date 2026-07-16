//! Logical formula functions.

use std::collections::HashMap;

use crate::calc::arg::*;
use crate::calc::{CalcContext, FormulaFn};

pub fn register(m: &mut HashMap<&'static str, FormulaFn>) {
    m.insert("AND", and);
    m.insert("FALSE", false_fn);
    m.insert("IFERROR", iferror);
    m.insert("IFNA", ifna);
    m.insert("IFS", ifs);
    m.insert("NOT", not);
    m.insert("OR", or);
    m.insert("SWITCH", switch);
    m.insert("TRUE", true_fn);
    m.insert("XOR", xor);
}

fn false_fn(_ctx: &CalcContext, _args: &[FormulaArg]) -> FormulaArg {
    new_bool_formula_arg(false)
}

fn true_fn(_ctx: &CalcContext, _args: &[FormulaArg]) -> FormulaArg {
    new_bool_formula_arg(true)
}

fn and(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.is_empty() {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let mut all_true = true;
    for a in args {
        match a.typ {
            ArgType::Matrix => return new_error_formula_arg(FORMULA_ERROR_VALUE),
            ArgType::String => {
                if a.string == "TRUE" {
                    continue;
                }
                if a.string == "FALSE" {
                    return new_bool_formula_arg(false);
                }
                return new_error_formula_arg(FORMULA_ERROR_VALUE);
            }
            _ => {}
        }
        let list = a.to_list();
        if list.is_empty() {
            all_true = false;
            continue;
        }
        for item in &list {
            if item.is_error() {
                return item.clone();
            }
            if item.typ == ArgType::String {
                if item.string == "TRUE" {
                    continue;
                }
                if item.string == "FALSE" {
                    return new_bool_formula_arg(false);
                }
                return new_error_formula_arg(FORMULA_ERROR_VALUE);
            }
            if !item.as_bool() {
                all_true = false;
            }
        }
    }
    new_bool_formula_arg(all_true)
}

fn or(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.is_empty() {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let mut any_true = false;
    for a in args {
        match a.typ {
            ArgType::Matrix => return or(_ctx, &a.to_list()),
            ArgType::String => {
                if a.string == "TRUE" {
                    return new_bool_formula_arg(true);
                }
                if a.string == "FALSE" {
                    continue;
                }
                return new_error_formula_arg(FORMULA_ERROR_VALUE);
            }
            _ => {}
        }
        let list = a.to_list();
        for item in &list {
            if item.is_error() {
                return item.clone();
            }
            if item.typ == ArgType::String {
                if item.string == "TRUE" {
                    return new_bool_formula_arg(true);
                }
                if item.string == "FALSE" {
                    continue;
                }
                return new_error_formula_arg(FORMULA_ERROR_VALUE);
            }
            if item.as_bool() {
                any_true = true;
            }
        }
    }
    new_bool_formula_arg(any_true)
}

fn xor(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.is_empty() {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let mut count_true = 0;
    let mut has_number = false;
    for a in args {
        match a.typ {
            ArgType::Number => {
                has_number = true;
                if a.number != 0.0 {
                    count_true += 1;
                }
            }
            ArgType::Matrix | ArgType::List => {
                for item in a.to_list() {
                    if let Some(n) = item.to_number().as_number() {
                        has_number = true;
                        if n != 0.0 {
                            count_true += 1;
                        }
                    }
                }
            }
            _ => {}
        }
    }
    if !has_number {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    new_bool_formula_arg(count_true % 2 == 1)
}

fn not(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let b = args[0].to_bool();
    if b.is_error() {
        return b;
    }
    new_bool_formula_arg(!b.as_bool())
}

fn iferror(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    if args[0].is_error() {
        args[1].clone()
    } else if args[0].typ == ArgType::Empty {
        new_number_formula_arg(0.0)
    } else {
        args[0].clone()
    }
}

fn ifna(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() < 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    if args[0].typ == ArgType::Error && args[0].error == FORMULA_ERROR_NA {
        args[1].clone()
    } else {
        args[0].clone()
    }
}

fn ifs(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.is_empty() || args.len() % 2 != 0 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    for pair in args.chunks_exact(2) {
        if pair[0].is_error() {
            return pair[0].clone();
        }
        if pair[0].as_bool() {
            return pair[1].clone();
        }
    }
    new_error_formula_arg(FORMULA_ERROR_NA)
}

fn switch(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() < 3 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let value = &args[0];
    let rest = &args[1..];
    let mut pairs = rest.chunks_exact(2);
    for pair in &mut pairs {
        if compare_equal(value, &pair[0]) {
            return pair[1].clone();
        }
    }
    // Remainder is the optional default value.
    let remainder = pairs.remainder();
    if remainder.len() == 1 {
        remainder[0].clone()
    } else {
        new_error_formula_arg(FORMULA_ERROR_NA)
    }
}
