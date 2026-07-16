//! Information formula functions.

use std::collections::HashMap;

use crate::calc::arg::*;
use crate::calc::{CalcContext, FormulaFn, find_cell};

pub fn register(m: &mut HashMap<&'static str, FormulaFn>) {
    m.insert("ERRORdotTYPE", error_type);
    m.insert("ISBLANK", isblank);
    m.insert("ISERR", iserr);
    m.insert("ISERROR", iserror);
    m.insert("ISEVEN", iseven);
    m.insert("ISFORMULA", is_formula);
    m.insert("ISLOGICAL", islogical);
    m.insert("ISNA", isna);
    m.insert("ISNONTEXT", isnontext);
    m.insert("ISNUMBER", isnumber);
    m.insert("ISODD", isodd);
    m.insert("ISREF", is_ref);
    m.insert("ISTEXT", istext);
    m.insert("N", n_fn);
    m.insert("NA", na);
    m.insert("SHEET", sheet_fn);
    m.insert("SHEETS", sheets_fn);
    m.insert("T", t_fn);
    m.insert("TYPE", type_fn);
}

fn na(_ctx: &CalcContext, _args: &[FormulaArg]) -> FormulaArg {
    new_error_formula_arg(FORMULA_ERROR_NA)
}

fn error_type(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_NA);
    }
    let code = match args[0].error.as_str() {
        "#NULL!" => 1.0,
        "#DIV/0!" => 2.0,
        "#VALUE!" => 3.0,
        "#REF!" => 4.0,
        "#NAME?" => 5.0,
        "#NUM!" => 6.0,
        "#N/A" => 7.0,
        "#GETTING_DATA" => 8.0,
        _ => return new_error_formula_arg(FORMULA_ERROR_NA),
    };
    new_number_formula_arg(code)
}

fn isblank(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    new_bool_formula_arg(args[0].typ == ArgType::Empty)
}

fn iserr(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let is_err = args[0].is_error() && args[0].error != FORMULA_ERROR_NA;
    new_bool_formula_arg(is_err)
}

fn iserror(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    new_bool_formula_arg(args[0].is_error())
}

fn iseven(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    match args[0].to_number().as_number() {
        Some(n) => new_bool_formula_arg(n.round() as i64 % 2 == 0),
        None => new_error_formula_arg(FORMULA_ERROR_VALUE),
    }
}

fn isodd(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    match args[0].to_number().as_number() {
        Some(n) => new_bool_formula_arg(n.round() as i64 % 2 != 0),
        None => new_error_formula_arg(FORMULA_ERROR_VALUE),
    }
}

fn islogical(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let val = args[0].value().to_uppercase();
    if val == "TRUE" || val == "FALSE" || (args[0].typ == ArgType::Number && args[0].boolean) {
        new_bool_formula_arg(true)
    } else {
        new_bool_formula_arg(false)
    }
}

fn isna(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    new_bool_formula_arg(args[0].typ == ArgType::Error && args[0].error == FORMULA_ERROR_NA)
}

fn isnontext(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    new_bool_formula_arg(args[0].typ != ArgType::String)
}

fn isnumber(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let arg = implicit_intersect(args[0].clone());
    new_bool_formula_arg(arg.typ == ArgType::Number && !arg.boolean)
}

fn istext(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    new_bool_formula_arg(args[0].typ == ArgType::String)
}

fn type_fn(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let code = match args[0].typ {
        ArgType::Number if args[0].boolean => 4.0,
        ArgType::Number => 1.0,
        ArgType::String => 2.0,
        ArgType::Error => 16.0,
        ArgType::List | ArgType::Matrix => 64.0,
        _ => 1.0,
    };
    new_number_formula_arg(code)
}

fn n_fn(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let arg = &args[0];
    if arg.is_error() {
        return arg.clone();
    }
    let mut num = arg.to_number().as_number().unwrap_or(0.0);
    if arg.value().eq_ignore_ascii_case("TRUE") {
        num = 1.0;
    }
    new_number_formula_arg(num)
}

fn t_fn(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let arg = &args[0];
    if arg.is_error() {
        return arg.clone();
    }
    if arg.typ == ArgType::String {
        new_string_formula_arg(arg.string.clone())
    } else {
        new_string_formula_arg("")
    }
}

fn is_formula(ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let (sheet, cell) = match first_reference(ctx, &args[0]) {
        Some(r) => r,
        None => return new_bool_formula_arg(false),
    };
    let ws = match ctx.file.work_sheet_reader(sheet) {
        Ok(ws) => ws,
        Err(_) => return new_error_formula_arg(FORMULA_ERROR_NA),
    };
    let c = find_cell(&ws, &cell);
    new_bool_formula_arg(
        c.as_ref()
            .and_then(|c| c.f.as_ref())
            .map(|f| !f.content.is_empty())
            .unwrap_or(false),
    )
}

fn is_ref(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    if args[0].is_error() {
        return args[0].clone();
    }
    new_bool_formula_arg(!args[0].cell_refs.is_empty() || !args[0].cell_ranges.is_empty())
}

fn sheet_fn(ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    let sheet = if args.is_empty() {
        ctx.sheet.to_string()
    } else if args[0].typ == ArgType::String {
        args[0].string.clone()
    } else {
        match first_reference(ctx, &args[0]) {
            Some((sheet, _)) => sheet.to_string(),
            None => ctx.sheet.to_string(),
        }
    };
    match ctx.file.get_sheet_index(&sheet) {
        Ok(idx) if idx >= 1 => new_number_formula_arg(idx as f64),
        _ => new_error_formula_arg(FORMULA_ERROR_NA),
    }
}

/// Apply Excel's implicit intersection: when a matrix is passed to a scalar
/// function, return the cell in the same row as the formula cell.
fn implicit_intersect(arg: FormulaArg) -> FormulaArg {
    if arg.typ != ArgType::Matrix || arg.matrix.is_empty() {
        return arg;
    }
    // The matrix was built starting at row 1, so row N corresponds to index N-1.
    // We have no caller row context here, so use the first row as the fallback.
    let row = arg.matrix.first().unwrap();
    if row.is_empty() {
        return new_empty_formula_arg();
    }
    row[0].clone()
}

fn sheets_fn(ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.is_empty() {
        return new_number_formula_arg(ctx.file.get_sheet_list().len() as f64);
    }
    let arg = &args[0];
    let mut sheet_set: std::collections::HashSet<String> = std::collections::HashSet::new();
    for (start, _) in &arg.cell_ranges {
        sheet_set.insert(start.sheet.clone().unwrap_or_else(|| ctx.sheet.to_string()));
    }
    for r in &arg.cell_refs {
        sheet_set.insert(r.sheet.clone().unwrap_or_else(|| ctx.sheet.to_string()));
    }
    if sheet_set.is_empty() {
        return new_error_formula_arg("#N/A");
    }
    new_number_formula_arg(sheet_set.len() as f64)
}

/// Return the `(sheet_name, cell_name)` of the first cell/range reference
/// carried by `arg`, using the calculation context as the default sheet.
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
