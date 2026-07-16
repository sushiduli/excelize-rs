//! Mathematical and trigonometric formula functions.

use std::collections::HashMap;
use std::sync::OnceLock;

use rand::Rng;
use regex::Regex;

use crate::calc::arg::*;
use crate::calc::statistical::{
    large, median, mode_sngl, percentile_exc, percentile_inc, quartile_exc, quartile_inc, small,
};
use crate::calc::{CalcContext, FormulaFn};

pub fn register(m: &mut HashMap<&'static str, FormulaFn>) {
    m.insert("SUM", sum);
    m.insert("AVERAGE", average);
    m.insert("COUNT", count);
    m.insert("MAX", max);
    m.insert("MIN", min);
    m.insert("ABS", abs);
    m.insert("ROUND", round);
    m.insert("PRODUCT", product);
    m.insert("SUMSQ", sumsq);

    m.insert("ACOS", acos);
    m.insert("ACOSH", acosh);
    m.insert("ACOT", acot);
    m.insert("ACOTH", acoth);
    m.insert("AGGREGATE", aggregate);
    m.insert("ARABIC", arabic);
    m.insert("ASIN", asin);
    m.insert("ASINH", asinh);
    m.insert("ATAN", atan);
    m.insert("ATANH", atanh);
    m.insert("ATAN2", atan2);
    m.insert("BASE", base);
    m.insert("CEILING", ceiling);
    m.insert("CEILINGdotMATH", ceiling_math);
    m.insert("CEILINGdotPRECISE", ceiling_precise);
    m.insert("COMBIN", combin);
    m.insert("COMBINA", combina);
    m.insert("COS", cos);
    m.insert("COSH", cosh);
    m.insert("COT", cot);
    m.insert("COTH", coth);
    m.insert("CSC", csc);
    m.insert("CSCH", csch);
    m.insert("DECIMAL", decimal);
    m.insert("DEGREES", degrees);
    m.insert("EVEN", even);
    m.insert("EXP", exp);
    m.insert("FACT", fact_fn);
    m.insert("FACTDOUBLE", factdouble);
    m.insert("FLOOR", floor);
    m.insert("FLOORdotMATH", floor_math);
    m.insert("FLOORdotPRECISE", floor_precise);
    m.insert("GCD", gcd);
    m.insert("INT", int);
    m.insert("ISOdotCEILING", iso_ceiling);
    m.insert("LCM", lcm);
    m.insert("LN", ln);
    m.insert("LOG", log);
    m.insert("LOG10", log10);
    m.insert("MDETERM", mdeterm);
    m.insert("MINVERSE", minverse);
    m.insert("MMULT", mmult);
    m.insert("MOD", mod_fn);
    m.insert("MROUND", mround);
    m.insert("MULTINOMIAL", multinomial);
    m.insert("MUNIT", munit);
    m.insert("ODD", odd);
    m.insert("PI", pi);
    m.insert("POWER", power);
    m.insert("QUOTIENT", quotient);
    m.insert("RADIANS", radians);
    m.insert("RAND", rand_fn);
    m.insert("RANDBETWEEN", randbetween);
    m.insert("ROMAN", roman);
    m.insert("ROUNDDOWN", rounddown);
    m.insert("ROUNDUP", roundup);
    m.insert("SEC", sec);
    m.insert("SECH", sech);
    m.insert("SERIESSUM", seriessum);
    m.insert("SIGN", sign);
    m.insert("SIN", sin);
    m.insert("SINH", sinh);
    m.insert("SQRT", sqrt);
    m.insert("SQRTPI", sqrtpi);
    m.insert("STDEV", stdev_fn);
    m.insert("STDEVdotS", stdev_s);
    m.insert("STDEVA", stdeva);
    m.insert("POISSONdotDIST", poisson_dist);
    m.insert("POISSON", poisson);
    m.insert("PROB", prob);
    m.insert("SUBTOTAL", subtotal);
    m.insert("SUMIF", sumif);
    m.insert("SUMIFS", sumifs);
    m.insert("SUMPRODUCT", sumproduct);
    m.insert("SUMX2MY2", sumx2my2);
    m.insert("SUMX2PY2", sumx2py2);
    m.insert("SUMXMY2", sumxmy2);
    m.insert("TAN", tan);
    m.insert("TANH", tanh);
    m.insert("TRUNC", trunc);
}

// ------------------------------------------------------------------
// Argument helpers
// ------------------------------------------------------------------

fn to_number(arg: &FormulaArg) -> Result<f64, FormulaArg> {
    if arg.is_error() {
        return Err(arg.clone());
    }
    let n = arg.to_number();
    if n.is_error() { Err(n) } else { Ok(n.number) }
}

macro_rules! num {
    ($e:expr) => {
        match to_number($e) {
            Ok(n) => n,
            Err(e) => return e,
        }
    };
}

fn to_bool_number(arg: &FormulaArg) -> Result<f64, FormulaArg> {
    if arg.is_error() {
        return Err(arg.clone());
    }
    let b = arg.to_bool();
    if b.is_error() { Err(b) } else { Ok(b.number) }
}

// ------------------------------------------------------------------
// Existing aggregate helpers (ported from Go)
// ------------------------------------------------------------------

fn sum(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    let mut total = 0.0;
    for arg in args {
        match arg.typ {
            ArgType::Error => return arg.clone(),
            ArgType::String => {
                if let Some(n) = arg.to_number().as_number() {
                    total += n;
                }
            }
            ArgType::Number => total += arg.number,
            ArgType::List | ArgType::Matrix => {
                for cell in arg.to_list() {
                    if let Some(n) = cell.to_number().as_number() {
                        total += n;
                    }
                }
            }
            _ => {}
        }
    }
    new_number_formula_arg(total)
}

fn count_sum(count_text: bool, args: &[FormulaArg]) -> (f64, f64) {
    let mut count = 0.0;
    let mut sum = 0.0;
    for arg in args {
        match arg.typ {
            ArgType::Number => {
                if count_text || !arg.boolean {
                    sum += arg.number;
                    count += 1.0;
                }
            }
            ArgType::String => {
                let val = arg.value();
                if !count_text && (val == "TRUE" || val == "FALSE") {
                    continue;
                }
                if count_text && (val == "TRUE" || val == "FALSE") {
                    if let Ok(b) = to_bool_number(arg) {
                        sum += b;
                        count += 1.0;
                    }
                    continue;
                }
                if let Some(n) = arg.to_number().as_number() {
                    sum += n;
                    count += 1.0;
                }
            }
            ArgType::List | ArgType::Matrix => {
                let (c, s) = count_sum(count_text, &arg.to_list());
                count += c;
                sum += s;
            }
            _ => {}
        }
    }
    (count, sum)
}

fn average(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    let (count, sum) = count_sum(false, args);
    if count == 0.0 {
        return new_error_formula_arg(FORMULA_ERROR_DIV);
    }
    new_number_formula_arg(sum / count)
}

fn averagea(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    let (count, sum) = count_sum(true, args);
    if count == 0.0 {
        return new_error_formula_arg(FORMULA_ERROR_DIV);
    }
    new_number_formula_arg(sum / count)
}

fn count(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    let mut total = 0;
    for arg in args {
        match arg.typ {
            ArgType::String => {
                if arg.to_number().as_number().is_some() {
                    total += 1;
                }
            }
            ArgType::Number => total += 1,
            ArgType::List | ArgType::Matrix => {
                for cell in arg.to_list() {
                    if cell.typ == ArgType::Number {
                        total += 1;
                    }
                }
            }
            _ => {}
        }
    }
    new_number_formula_arg(total as f64)
}

fn counta(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    let mut total = 0;
    for arg in args {
        match arg.typ {
            ArgType::String => {
                if !arg.string.is_empty() {
                    total += 1;
                }
            }
            ArgType::Number => total += 1,
            ArgType::List | ArgType::Matrix => {
                for cell in arg.to_list() {
                    match cell.typ {
                        ArgType::String => {
                            if !cell.string.is_empty() {
                                total += 1;
                            }
                        }
                        ArgType::Number => total += 1,
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
    new_number_formula_arg(total as f64)
}

fn max_value(maxa: bool, args: &[FormulaArg]) -> FormulaArg {
    let mut max_val = f64::NEG_INFINITY;
    for arg in args {
        match arg.typ {
            ArgType::Error => return arg.clone(),
            ArgType::String => {
                let val = arg.value();
                if !maxa && (val == "TRUE" || val == "FALSE") {
                    continue;
                }
                if maxa {
                    if let Ok(b) = to_bool_number(arg) {
                        if b > max_val {
                            max_val = b;
                        }
                        continue;
                    }
                }
                if let Some(n) = arg.to_number().as_number() {
                    if n > max_val {
                        max_val = n;
                    }
                }
            }
            ArgType::Number => {
                if arg.number > max_val {
                    max_val = arg.number;
                }
            }
            ArgType::List | ArgType::Matrix => {
                max_val = list_matrix_max(maxa, max_val, arg);
            }
            _ => {}
        }
    }
    if max_val == f64::NEG_INFINITY {
        max_val = 0.0;
    }
    new_number_formula_arg(max_val)
}

fn list_matrix_max(maxa: bool, mut max_val: f64, arg: &FormulaArg) -> f64 {
    for cell in arg.to_list() {
        if cell.typ == ArgType::Number && cell.number > max_val {
            if (maxa && cell.boolean) || !cell.boolean {
                max_val = cell.number;
            }
        }
    }
    max_val
}

fn max(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.is_empty() {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    max_value(false, args)
}

fn min_value(mina: bool, args: &[FormulaArg]) -> FormulaArg {
    let mut min_val = f64::INFINITY;
    for arg in args {
        match arg.typ {
            ArgType::Error => return arg.clone(),
            ArgType::String => {
                let val = arg.value();
                if !mina && (val == "TRUE" || val == "FALSE") {
                    continue;
                }
                if mina {
                    if let Ok(b) = to_bool_number(arg) {
                        if b < min_val {
                            min_val = b;
                        }
                        continue;
                    }
                }
                if let Some(n) = arg.to_number().as_number() {
                    if n < min_val {
                        min_val = n;
                    }
                }
            }
            ArgType::Number => {
                if arg.number < min_val {
                    min_val = arg.number;
                }
            }
            ArgType::List | ArgType::Matrix => {
                min_val = list_matrix_min(mina, min_val, arg);
            }
            _ => {}
        }
    }
    if min_val == f64::INFINITY {
        min_val = 0.0;
    }
    new_number_formula_arg(min_val)
}

fn list_matrix_min(mina: bool, mut min_val: f64, arg: &FormulaArg) -> f64 {
    for cell in arg.to_list() {
        if cell.typ == ArgType::Number && cell.number < min_val {
            if (mina && cell.boolean) || !cell.boolean {
                min_val = cell.number;
            }
        }
    }
    min_val
}

fn min(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.is_empty() {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    min_value(false, args)
}

fn product(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    let mut prod = 1.0;
    for arg in args {
        match arg.typ {
            ArgType::String => {
                if let Some(n) = arg.to_number().as_number() {
                    prod *= n;
                }
            }
            ArgType::Number => prod *= arg.number,
            ArgType::List | ArgType::Matrix => {
                for cell in arg.to_list() {
                    if cell.typ == ArgType::Number {
                        prod *= cell.number;
                    }
                }
            }
            _ => {}
        }
    }
    new_number_formula_arg(prod)
}

fn sumsq(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    let mut sq = 0.0;
    for arg in args {
        match arg.typ {
            ArgType::String => {
                if arg.string.is_empty() {
                    continue;
                }
                if let Some(n) = arg.to_number().as_number() {
                    sq += n * n;
                } else {
                    return new_error_formula_arg(FORMULA_ERROR_VALUE);
                }
            }
            ArgType::Number => sq += arg.number * arg.number,
            ArgType::List | ArgType::Matrix => {
                for cell in arg.to_list() {
                    if cell.value().is_empty() {
                        continue;
                    }
                    if let Some(n) = cell.to_number().as_number() {
                        sq += n * n;
                    } else {
                        return new_error_formula_arg(FORMULA_ERROR_VALUE);
                    }
                }
            }
            _ => {}
        }
    }
    new_number_formula_arg(sq)
}

// ------------------------------------------------------------------
// Simple numeric functions
// ------------------------------------------------------------------

fn abs(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let n = num!(&args[0]);
    new_number_formula_arg(n.abs())
}

fn acos(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    new_number_formula_arg(num!(&args[0]).acos())
}

fn acosh(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    new_number_formula_arg(num!(&args[0]).acosh())
}

fn acot(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let n = num!(&args[0]);
    new_number_formula_arg(std::f64::consts::FRAC_PI_2 - n.atan())
}

fn acoth(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let n = num!(&args[0]);
    new_number_formula_arg((1.0 / n).atanh())
}

fn asin(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    new_number_formula_arg(num!(&args[0]).asin())
}

fn asinh(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    new_number_formula_arg(num!(&args[0]).asinh())
}

fn atan(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    new_number_formula_arg(num!(&args[0]).atan())
}

fn atanh(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    new_number_formula_arg(num!(&args[0]).atanh())
}

fn atan2(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let x = num!(&args[1]);
    let y = num!(&args[0]);
    new_number_formula_arg(x.atan2(y))
}

fn cos(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    new_number_formula_arg(num!(&args[0]).cos())
}

fn cosh(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    new_number_formula_arg(num!(&args[0]).cosh())
}

fn cot(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let n = num!(&args[0]);
    if n == 0.0 {
        return new_error_formula_arg(FORMULA_ERROR_DIV);
    }
    new_number_formula_arg(1.0 / n.tan())
}

fn coth(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let n = num!(&args[0]);
    if n == 0.0 {
        return new_error_formula_arg(FORMULA_ERROR_DIV);
    }
    new_number_formula_arg((n.exp() + (-n).exp()) / (n.exp() - (-n).exp()))
}

fn csc(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let n = num!(&args[0]);
    if n == 0.0 {
        return new_error_formula_arg(FORMULA_ERROR_DIV);
    }
    new_number_formula_arg(1.0 / n.sin())
}

fn csch(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let n = num!(&args[0]);
    if n == 0.0 {
        return new_error_formula_arg(FORMULA_ERROR_DIV);
    }
    new_number_formula_arg(1.0 / n.sinh())
}

fn sec(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    new_number_formula_arg(num!(&args[0]).cos())
}

fn sech(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let n = num!(&args[0]);
    new_number_formula_arg(1.0 / n.cosh())
}

fn sin(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    new_number_formula_arg(num!(&args[0]).sin())
}

fn sinh(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    new_number_formula_arg(num!(&args[0]).sinh())
}

fn tan(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    new_number_formula_arg(num!(&args[0]).tan())
}

fn tanh(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    new_number_formula_arg(num!(&args[0]).tanh())
}

fn degrees(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let n = num!(&args[0]);
    if n == 0.0 {
        return new_error_formula_arg(FORMULA_ERROR_DIV);
    }
    new_number_formula_arg(180.0 / std::f64::consts::PI * n)
}

fn radians(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    new_number_formula_arg(std::f64::consts::PI / 180.0 * num!(&args[0]))
}

fn exp(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    new_number_formula_arg(num!(&args[0]).exp())
}

fn ln(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    new_number_formula_arg(num!(&args[0]).ln())
}

fn log10(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    new_number_formula_arg(num!(&args[0]).log10())
}

fn log(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.is_empty() {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    if args.len() > 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let number = num!(&args[0]);
    let mut base = 10.0;
    if args.len() > 1 {
        base = num!(&args[1]);
    }
    if number == 0.0 {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    if base == 0.0 {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    if base == 1.0 {
        return new_error_formula_arg(FORMULA_ERROR_DIV);
    }
    new_number_formula_arg(number.ln() / base.ln())
}

fn pi(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if !args.is_empty() {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    new_number_formula_arg(std::f64::consts::PI)
}

fn power(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let x = num!(&args[0]);
    let y = num!(&args[1]);
    if x == 0.0 && y == 0.0 {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    if x == 0.0 && y < 0.0 {
        return new_error_formula_arg(FORMULA_ERROR_DIV);
    }
    new_number_formula_arg(x.powf(y))
}

fn sqrt(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let n = num!(&args[0]);
    if n < 0.0 {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    new_number_formula_arg(n.sqrt())
}

fn sqrtpi(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    new_number_formula_arg((num!(&args[0]) * std::f64::consts::PI).sqrt())
}

fn sign(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let n = num!(&args[0]);
    if n < 0.0 {
        new_number_formula_arg(-1.0)
    } else if n > 0.0 {
        new_number_formula_arg(1.0)
    } else {
        new_number_formula_arg(0.0)
    }
}

fn int(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    new_number_formula_arg(num!(&args[0]).floor())
}

fn quotient(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let x = num!(&args[0]);
    let y = num!(&args[1]);
    if y == 0.0 {
        return new_error_formula_arg(FORMULA_ERROR_DIV);
    }
    new_number_formula_arg((x / y).trunc())
}

fn mod_fn(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let number = num!(&args[0]);
    let divisor = num!(&args[1]);
    if divisor == 0.0 {
        return new_error_formula_arg(FORMULA_ERROR_DIV);
    }
    let mut trunc = (number / divisor).trunc();
    if (number / divisor).fract() < 0.0 {
        trunc -= 1.0;
    }
    new_number_formula_arg(number - divisor * trunc)
}

// ------------------------------------------------------------------
// Rounding functions
// ------------------------------------------------------------------

#[derive(Clone, Copy)]
enum RoundMode {
    Closest,
    Down,
    Up,
}

fn round_number(number: f64, digits: f64, mode: RoundMode) -> f64 {
    let digits = digits.trunc() as i32;
    let mult = 10.0_f64.powi(digits.abs());
    let scaled = if digits >= 0 {
        number * mult
    } else {
        number / mult
    };
    let rounded = match mode {
        RoundMode::Closest => scaled.round(),
        RoundMode::Down => scaled.trunc(),
        RoundMode::Up => {
            if scaled >= 0.0 {
                scaled.ceil()
            } else {
                scaled.floor()
            }
        }
    };
    if digits >= 0 {
        rounded / mult
    } else {
        rounded * mult
    }
}

fn round(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let number = num!(&args[0]);
    let digits = num!(&args[1]);
    new_number_formula_arg(round_number(number, digits, RoundMode::Closest))
}

fn rounddown(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let number = num!(&args[0]);
    let digits = num!(&args[1]);
    new_number_formula_arg(round_number(number, digits, RoundMode::Down))
}

fn roundup(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let number = num!(&args[0]);
    let digits = num!(&args[1]);
    new_number_formula_arg(round_number(number, digits, RoundMode::Up))
}

fn trunc(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.is_empty() {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let number = num!(&args[0]);
    let mut digits = 0.0;
    if args.len() > 1 {
        digits = num!(&args[1]).floor();
    }
    let adjust = 10.0_f64.powf(digits);
    new_number_formula_arg((number * adjust).trunc() / adjust)
}

fn mround(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let n = num!(&args[0]);
    let multiple = num!(&args[1]);
    if multiple == 0.0 {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    if (multiple < 0.0 && n > 0.0) || (multiple > 0.0 && n < 0.0) {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    let ratio = n / multiple;
    let mut q = ratio.trunc();
    let res = ratio.fract();
    if (res + 0.5).trunc() > 0.0 {
        q += 1.0;
    }
    new_number_formula_arg(q * multiple)
}

fn ceiling(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.is_empty() {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    if args.len() > 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let number = num!(&args[0]);
    let mut significance = if number < 0.0 { -1.0 } else { 1.0 };
    if args.len() > 1 {
        significance = num!(&args[1]);
    }
    if significance < 0.0 && number > 0.0 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    if args.len() == 1 {
        return new_number_formula_arg(number.ceil());
    }
    let ratio = number / significance;
    let mut val = ratio.trunc();
    let res = ratio.fract();
    if res > 0.0 {
        val += 1.0;
    }
    new_number_formula_arg(val * significance)
}

fn ceiling_math(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.is_empty() {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    if args.len() > 3 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let number = num!(&args[0]);
    let mut significance = if number < 0.0 { -1.0 } else { 1.0 };
    if args.len() > 1 {
        significance = num!(&args[1]);
    }
    if args.len() == 1 {
        return new_number_formula_arg(number.ceil());
    }
    let mut mode = 1.0;
    if args.len() > 2 {
        mode = num!(&args[2]);
    }
    let ratio = number / significance;
    let mut val = ratio.trunc();
    let res = ratio.fract();
    if res != 0.0 {
        if number > 0.0 {
            val += 1.0;
        } else if mode < 0.0 {
            val -= 1.0;
        }
    }
    new_number_formula_arg(val * significance)
}

fn ceiling_precise(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.is_empty() {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    if args.len() > 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let number = num!(&args[0]);
    if args.len() == 1 {
        return new_number_formula_arg(number.ceil());
    }
    let mut significance = num!(&args[1]);
    significance = significance.abs();
    if significance == 0.0 {
        return new_number_formula_arg(significance);
    }
    let ratio = number / significance;
    let mut val = ratio.trunc();
    let res = ratio.fract();
    if res != 0.0 && number > 0.0 {
        val += 1.0;
    }
    new_number_formula_arg(val * significance)
}

fn iso_ceiling(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.is_empty() {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    if args.len() > 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let number = num!(&args[0]);
    if args.len() == 1 {
        return new_number_formula_arg(number.ceil());
    }
    let mut significance = num!(&args[1]);
    significance = significance.abs();
    if significance == 0.0 {
        return new_number_formula_arg(significance);
    }
    let ratio = number / significance;
    let mut val = ratio.trunc();
    let res = ratio.fract();
    if res != 0.0 && number > 0.0 {
        val += 1.0;
    }
    new_number_formula_arg(val * significance)
}

fn floor(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let number = num!(&args[0]);
    let significance = num!(&args[1]);
    if significance < 0.0 && number >= 0.0 {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    let ratio = number / significance;
    let mut val = ratio.trunc();
    let res = ratio.fract();
    if res != 0.0 && number < 0.0 && res < 0.0 {
        val -= 1.0;
    }
    new_number_formula_arg(val * significance)
}

fn floor_math(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.is_empty() {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    if args.len() > 3 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let number = num!(&args[0]);
    let mut significance = if number < 0.0 { -1.0 } else { 1.0 };
    if args.len() > 1 {
        significance = num!(&args[1]);
    }
    if args.len() == 1 {
        return new_number_formula_arg(number.floor());
    }
    let mut mode = 1.0;
    if args.len() > 2 {
        mode = num!(&args[2]);
    }
    let ratio = number / significance;
    let mut val = ratio.trunc();
    let res = ratio.fract();
    if res != 0.0 && number < 0.0 && mode > 0.0 {
        val -= 1.0;
    }
    new_number_formula_arg(val * significance)
}

fn floor_precise(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.is_empty() {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    if args.len() > 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let number = num!(&args[0]);
    if args.len() == 1 {
        return new_number_formula_arg(number.floor());
    }
    let mut significance = num!(&args[1]);
    significance = significance.abs();
    if significance == 0.0 {
        return new_number_formula_arg(significance);
    }
    let ratio = number / significance;
    let mut val = ratio.trunc();
    let res = ratio.fract();
    if res != 0.0 && number < 0.0 {
        val -= 1.0;
    }
    new_number_formula_arg(val * significance)
}

fn even(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let number = num!(&args[0]);
    let sign = number.is_sign_negative();
    let m = (number / 2.0).trunc();
    let frac = (number / 2.0).fract();
    let mut val = m * 2.0;
    if frac != 0.0 {
        if !sign {
            val += 2.0;
        } else {
            val -= 2.0;
        }
    }
    new_number_formula_arg(val)
}

fn odd(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let number = num!(&args[0]);
    if number == 0.0 {
        return new_number_formula_arg(1.0);
    }
    let sign = number.is_sign_negative();
    let m = ((number - 1.0) / 2.0).trunc();
    let frac = ((number - 1.0) / 2.0).fract();
    let mut val = m * 2.0 + 1.0;
    if frac != 0.0 {
        if !sign {
            val += 2.0;
        } else {
            val -= 2.0;
        }
    }
    new_number_formula_arg(val)
}

// ------------------------------------------------------------------
// Factorials, combinations, etc.
// ------------------------------------------------------------------

fn fact(number: f64) -> f64 {
    let mut val = 1.0;
    let n = number.trunc();
    let mut i = 2.0;
    while i <= n {
        val *= i;
        i += 1.0;
    }
    val
}

fn fact_fn(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let n = num!(&args[0]);
    if n < 0.0 {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    new_number_formula_arg(fact(n))
}

fn factdouble(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let n = num!(&args[0]);
    if n < 0.0 {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    let mut val = 1.0;
    let mut i = n.trunc();
    while i > 1.0 {
        val *= i;
        i -= 2.0;
    }
    new_string_formula_arg(format!("{}", val).to_uppercase())
}

fn combin_value(number: f64, chosen: f64) -> f64 {
    let number = number.trunc();
    let chosen = chosen.trunc();
    if chosen == number || chosen == 0.0 {
        return 1.0;
    }
    let mut val = 1.0;
    let mut c = 1.0;
    while c <= chosen {
        val *= (number + 1.0 - c) / c;
        c += 1.0;
    }
    val.ceil()
}

fn combin(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let number = num!(&args[0]);
    let chosen = num!(&args[1]);
    let number_t = number.trunc();
    let chosen_t = chosen.trunc();
    if chosen_t > number_t {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    new_number_formula_arg(combin_value(number, chosen))
}

fn combina(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let number = num!(&args[0]);
    let chosen = num!(&args[1]);
    let number_t = number.trunc();
    let chosen_t = chosen.trunc();
    if number_t < chosen_t {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    if number_t == 0.0 {
        return new_number_formula_arg(number_t);
    }
    new_number_formula_arg(combin_value(number + chosen - 1.0, number - 1.0))
}

fn multinomial(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    let mut num = 0.0;
    let mut denom = 1.0;
    for arg in args {
        let val = match arg.typ {
            ArgType::String => {
                if arg.string.is_empty() {
                    continue;
                }
                match arg.string.parse::<f64>() {
                    Ok(v) => v,
                    Err(_) => return new_error_formula_arg(FORMULA_ERROR_VALUE),
                }
            }
            ArgType::Number => arg.number,
            _ => continue,
        };
        num += val;
        denom *= fact(val);
    }
    new_number_formula_arg(fact(num) / denom)
}

// ------------------------------------------------------------------
// GCD / LCM
// ------------------------------------------------------------------

fn gcd_pair(mut x: f64, mut y: f64) -> f64 {
    x = x.trunc();
    y = y.trunc();
    if x == 0.0 {
        return y;
    }
    if y == 0.0 {
        return x;
    }
    while x != y {
        if x > y {
            x -= y;
        } else {
            y -= x;
        }
    }
    x
}

fn lcm_pair(a: f64, b: f64) -> f64 {
    let a = a.trunc();
    let b = b.trunc();
    if a == 0.0 && b == 0.0 {
        return 0.0;
    }
    a * b / gcd_pair(a, b)
}

fn gcd(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.is_empty() {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let mut nums = Vec::new();
    for arg in args {
        let val = match arg.typ {
            ArgType::String => {
                if let Some(n) = arg.to_number().as_number() {
                    n
                } else {
                    return new_error_formula_arg(FORMULA_ERROR_VALUE);
                }
            }
            ArgType::Number => arg.number,
            _ => continue,
        };
        nums.push(val);
    }
    if nums[0] < 0.0 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    if nums.len() == 1 {
        return new_number_formula_arg(nums[0]);
    }
    let mut cd = nums[0];
    for i in 1..nums.len() {
        if nums[i] < 0.0 {
            return new_error_formula_arg(FORMULA_ERROR_VALUE);
        }
        cd = gcd_pair(cd, nums[i]);
    }
    new_number_formula_arg(cd)
}

fn lcm(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.is_empty() {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let mut nums = Vec::new();
    for arg in args {
        let val = match arg.typ {
            ArgType::String => {
                if arg.string.is_empty() {
                    continue;
                }
                match arg.string.parse::<f64>() {
                    Ok(v) => v,
                    Err(_) => return new_error_formula_arg(FORMULA_ERROR_VALUE),
                }
            }
            ArgType::Number => arg.number,
            _ => continue,
        };
        if val < 0.0 {
            return new_error_formula_arg(FORMULA_ERROR_VALUE);
        }
        nums.push(val);
    }
    if nums.is_empty() {
        return new_number_formula_arg(0.0);
    }
    if nums.len() == 1 {
        return new_number_formula_arg(nums[0]);
    }
    let mut cm = nums[0];
    for i in 1..nums.len() {
        cm = lcm_pair(cm, nums[i]);
    }
    new_number_formula_arg(cm)
}

// ------------------------------------------------------------------
// Base conversions
// ------------------------------------------------------------------

fn format_int_base(mut n: i64, radix: i32) -> String {
    if n == 0 {
        return "0".to_string();
    }
    let digits = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let mut out = Vec::new();
    let neg = n < 0;
    while n != 0 {
        let r = (n % radix as i64).abs() as usize;
        out.push(digits[r] as char);
        n /= radix as i64;
    }
    if neg {
        out.push('-');
    }
    out.into_iter().rev().collect()
}

fn base(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() < 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    if args.len() > 3 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let number = num!(&args[0]);
    let radix = num!(&args[1]) as i32;
    if radix < 2 || radix > 36 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let mut min_length = 0;
    if args.len() > 2 {
        match args[2].value().parse::<usize>() {
            Ok(n) => min_length = n,
            Err(_) => return new_error_formula_arg(FORMULA_ERROR_VALUE),
        }
    }
    let mut result = format_int_base(number as i64, radix);
    if result.len() < min_length {
        result = "0".repeat(min_length - result.len()) + &result;
    }
    new_string_formula_arg(result.to_uppercase())
}

fn decimal(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let mut text = args[0].value();
    let radix = num!(&args[1]) as u32;
    if text.len() > 2 && (text.starts_with("0x") || text.starts_with("0X")) {
        text = text[2..].to_string();
    }
    match i64::from_str_radix(&text, radix) {
        Ok(v) => new_number_formula_arg(v as f64),
        Err(_) => new_error_formula_arg(FORMULA_ERROR_VALUE),
    }
}

fn arabic(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let text = args[0].value().to_uppercase();
    if text.chars().count() > 32767 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let mut index: i64 = text.len() as i64 - 1;
    let mut actual_start: i64 = 0;
    while index >= 0 && text.chars().nth(index as usize) == Some(' ') {
        index -= 1;
    }
    while actual_start <= index && text.chars().nth(actual_start as usize) == Some(' ') {
        actual_start += 1;
    }
    let mut is_negative = false;
    if actual_start <= index && text.chars().nth(actual_start as usize) == Some('-') {
        is_negative = true;
        actual_start += 1;
    }
    let char_map: std::collections::HashMap<char, i32> = [
        ('I', 1),
        ('V', 5),
        ('X', 10),
        ('L', 50),
        ('C', 100),
        ('D', 500),
        ('M', 1000),
    ]
    .into_iter()
    .collect();
    let mut number = 0;
    let mut prev_char_value = -1;
    let mut subtract_number = 0;
    let mut idx = index;
    while idx >= actual_start {
        let start_index = idx;
        let start_char = text.chars().nth(start_index as usize).unwrap();
        idx -= 1;
        while idx >= actual_start {
            let c = text.chars().nth(idx as usize).unwrap();
            let lower = c.to_ascii_lowercase();
            let start_lower = start_char.to_ascii_lowercase();
            if (lower as u32 | ' ' as u32) == (start_lower as u32 | ' ' as u32) {
                idx -= 1;
            } else {
                break;
            }
        }
        let current_char_value = *char_map.get(&start_char).unwrap_or(&0);
        let current_part_value = ((start_index - idx) as i32) * current_char_value;
        if current_char_value >= prev_char_value {
            number += current_part_value - subtract_number;
            prev_char_value = current_char_value;
            subtract_number = 0;
        } else {
            subtract_number += current_part_value;
        }
    }
    if subtract_number != 0 {
        number -= subtract_number;
    }
    if is_negative {
        number = -number;
    }
    new_number_formula_arg(number as f64)
}

// ------------------------------------------------------------------
// Roman numerals
// ------------------------------------------------------------------

#[derive(Clone)]
struct RomanNumeral {
    n: f64,
    s: &'static str,
}

fn roman_table() -> &'static [Vec<RomanNumeral>] {
    static TABLE: OnceLock<Vec<Vec<RomanNumeral>>> = OnceLock::new();
    TABLE.get_or_init(|| {
        vec![
            vec![
                RomanNumeral { n: 1000.0, s: "M" },
                RomanNumeral { n: 900.0, s: "CM" },
                RomanNumeral { n: 500.0, s: "D" },
                RomanNumeral { n: 400.0, s: "CD" },
                RomanNumeral { n: 100.0, s: "C" },
                RomanNumeral { n: 90.0, s: "XC" },
                RomanNumeral { n: 50.0, s: "L" },
                RomanNumeral { n: 40.0, s: "XL" },
                RomanNumeral { n: 10.0, s: "X" },
                RomanNumeral { n: 9.0, s: "IX" },
                RomanNumeral { n: 5.0, s: "V" },
                RomanNumeral { n: 4.0, s: "IV" },
                RomanNumeral { n: 1.0, s: "I" },
            ],
            vec![
                RomanNumeral { n: 1000.0, s: "M" },
                RomanNumeral { n: 950.0, s: "LM" },
                RomanNumeral { n: 900.0, s: "CM" },
                RomanNumeral { n: 500.0, s: "D" },
                RomanNumeral { n: 450.0, s: "LD" },
                RomanNumeral { n: 400.0, s: "CD" },
                RomanNumeral { n: 100.0, s: "C" },
                RomanNumeral { n: 95.0, s: "VC" },
                RomanNumeral { n: 90.0, s: "XC" },
                RomanNumeral { n: 50.0, s: "L" },
                RomanNumeral { n: 45.0, s: "VL" },
                RomanNumeral { n: 40.0, s: "XL" },
                RomanNumeral { n: 10.0, s: "X" },
                RomanNumeral { n: 9.0, s: "IX" },
                RomanNumeral { n: 5.0, s: "V" },
                RomanNumeral { n: 4.0, s: "IV" },
                RomanNumeral { n: 1.0, s: "I" },
            ],
            vec![
                RomanNumeral { n: 1000.0, s: "M" },
                RomanNumeral { n: 990.0, s: "XM" },
                RomanNumeral { n: 950.0, s: "LM" },
                RomanNumeral { n: 900.0, s: "CM" },
                RomanNumeral { n: 500.0, s: "D" },
                RomanNumeral { n: 490.0, s: "XD" },
                RomanNumeral { n: 450.0, s: "LD" },
                RomanNumeral { n: 400.0, s: "CD" },
                RomanNumeral { n: 100.0, s: "C" },
                RomanNumeral { n: 99.0, s: "IC" },
                RomanNumeral { n: 90.0, s: "XC" },
                RomanNumeral { n: 50.0, s: "L" },
                RomanNumeral { n: 45.0, s: "VL" },
                RomanNumeral { n: 40.0, s: "XL" },
                RomanNumeral { n: 10.0, s: "X" },
                RomanNumeral { n: 9.0, s: "IX" },
                RomanNumeral { n: 5.0, s: "V" },
                RomanNumeral { n: 4.0, s: "IV" },
                RomanNumeral { n: 1.0, s: "I" },
            ],
            vec![
                RomanNumeral { n: 1000.0, s: "M" },
                RomanNumeral { n: 995.0, s: "VM" },
                RomanNumeral { n: 990.0, s: "XM" },
                RomanNumeral { n: 950.0, s: "LM" },
                RomanNumeral { n: 900.0, s: "CM" },
                RomanNumeral { n: 500.0, s: "D" },
                RomanNumeral { n: 495.0, s: "VD" },
                RomanNumeral { n: 490.0, s: "XD" },
                RomanNumeral { n: 450.0, s: "LD" },
                RomanNumeral { n: 400.0, s: "CD" },
                RomanNumeral { n: 100.0, s: "C" },
                RomanNumeral { n: 99.0, s: "IC" },
                RomanNumeral { n: 90.0, s: "XC" },
                RomanNumeral { n: 50.0, s: "L" },
                RomanNumeral { n: 45.0, s: "VL" },
                RomanNumeral { n: 40.0, s: "XL" },
                RomanNumeral { n: 10.0, s: "X" },
                RomanNumeral { n: 9.0, s: "IX" },
                RomanNumeral { n: 5.0, s: "V" },
                RomanNumeral { n: 4.0, s: "IV" },
                RomanNumeral { n: 1.0, s: "I" },
            ],
            vec![
                RomanNumeral { n: 1000.0, s: "M" },
                RomanNumeral { n: 999.0, s: "IM" },
                RomanNumeral { n: 995.0, s: "VM" },
                RomanNumeral { n: 990.0, s: "XM" },
                RomanNumeral { n: 950.0, s: "LM" },
                RomanNumeral { n: 900.0, s: "CM" },
                RomanNumeral { n: 500.0, s: "D" },
                RomanNumeral { n: 499.0, s: "ID" },
                RomanNumeral { n: 495.0, s: "VD" },
                RomanNumeral { n: 490.0, s: "XD" },
                RomanNumeral { n: 450.0, s: "LD" },
                RomanNumeral { n: 400.0, s: "CD" },
                RomanNumeral { n: 100.0, s: "C" },
                RomanNumeral { n: 99.0, s: "IC" },
                RomanNumeral { n: 90.0, s: "XC" },
                RomanNumeral { n: 50.0, s: "L" },
                RomanNumeral { n: 45.0, s: "VL" },
                RomanNumeral { n: 40.0, s: "XL" },
                RomanNumeral { n: 10.0, s: "X" },
                RomanNumeral { n: 9.0, s: "IX" },
                RomanNumeral { n: 5.0, s: "V" },
                RomanNumeral { n: 4.0, s: "IV" },
                RomanNumeral { n: 1.0, s: "I" },
            ],
        ]
    })
}

fn roman(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.is_empty() {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    if args.len() > 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let number = num!(&args[0]);
    let mut form = 0;
    if args.len() > 1 {
        form = num!(&args[1]) as i32;
        if form < 0 {
            form = 0;
        } else if form > 4 {
            form = 4;
        }
    }
    let table = roman_table();
    let decimal_table = &table[form as usize];
    let mut val = number.trunc();
    let mut buf = String::new();
    for r in decimal_table {
        while val >= r.n {
            buf.push_str(r.s);
            val -= r.n;
        }
    }
    new_string_formula_arg(buf)
}

// ------------------------------------------------------------------
// Random functions
// ------------------------------------------------------------------

fn rand_fn(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if !args.is_empty() {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    new_number_formula_arg(rand::random::<f64>())
}

fn randbetween(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let bottom = num!(&args[0]) as i64;
    let top = num!(&args[1]) as i64;
    if top < bottom {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    let n = rand::thread_rng().gen_range(bottom..=top);
    new_number_formula_arg(n as f64)
}

// ------------------------------------------------------------------
// Matrix functions
// ------------------------------------------------------------------

fn number_matrix(arg: &FormulaArg, phalanx: bool) -> Result<Vec<Vec<f64>>, FormulaArg> {
    let rows = arg.matrix.len();
    let mut out = Vec::with_capacity(rows);
    for row in &arg.matrix {
        if phalanx && row.len() != rows {
            return Err(new_error_formula_arg(FORMULA_ERROR_VALUE));
        }
        let mut r = Vec::with_capacity(row.len());
        for cell in row {
            if cell.typ != ArgType::Number {
                return Err(new_error_formula_arg(FORMULA_ERROR_VALUE));
            }
            r.push(cell.number);
        }
        out.push(r);
    }
    Ok(out)
}

fn formula_arg_matrix(num_mtx: &[Vec<f64>]) -> Vec<Vec<FormulaArg>> {
    num_mtx
        .iter()
        .map(|row| row.iter().map(|&n| new_number_formula_arg(n)).collect())
        .collect()
}

fn minor(sq_mtx: &[Vec<f64>], idx: usize) -> Vec<Vec<f64>> {
    sq_mtx
        .iter()
        .enumerate()
        .skip(1)
        .map(|(_i, row)| {
            row.iter()
                .enumerate()
                .filter(|(j, _)| *j != idx)
                .map(|(_, &v)| v)
                .collect()
        })
        .collect()
}

fn det(sq_mtx: &[Vec<f64>]) -> f64 {
    // Match Go's excelize behavior: 1x1 determinants evaluate to 0, so 2x2
    // matrix inverses produced by the adjugate/determinant path are all zeros.
    if sq_mtx.len() == 1 {
        return 0.0;
    }
    if sq_mtx.len() == 2 {
        let m00 = sq_mtx[0][0];
        let m01 = sq_mtx[0][1];
        let m10 = sq_mtx[1][0];
        let m11 = sq_mtx[1][1];
        return m00 * m11 - m10 * m01;
    }
    let mut res = 0.0;
    let mut sgn = 1.0;
    for (j, _) in sq_mtx[0].iter().enumerate() {
        res += sgn * sq_mtx[0][j] * det(&minor(sq_mtx, j));
        sgn *= -1.0;
    }
    res
}

fn cofactor_matrix(i: usize, j: usize, a: &[Vec<f64>]) -> f64 {
    let n = a.len();
    let sign = if (i + j) % 2 == 0 { 1.0 } else { -1.0 };
    let mut b: Vec<Vec<f64>> = a.iter().map(|row| row.to_vec()).collect();
    for m in 0..n {
        for n_idx in (j + 1..n).rev() {
            b[m][n_idx - 1] = b[m][n_idx];
        }
        b[m].pop();
    }
    for k in (i + 1..n).rev() {
        b[k - 1] = b[k].clone();
    }
    b.pop();
    sign * det(&b)
}

fn adjugate_matrix(a: &[Vec<f64>]) -> Vec<Vec<f64>> {
    let n = a.len();
    let mut adj = vec![vec![0.0; n]; n];
    for i in 0..n {
        for j in 0..n {
            let mut b: Vec<Vec<f64>> = Vec::with_capacity(n);
            for _ in 0..n {
                b.push(vec![0.0; n]);
            }
            for m in 0..n {
                for n_idx in 0..n {
                    b[m][n_idx] = a[m][n_idx];
                }
            }
            adj[i][j] = cofactor_matrix(j, i, &b);
        }
    }
    adj
}

fn mdeterm(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() < 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let num_mtx = match number_matrix(&args[0], true) {
        Ok(m) => m,
        Err(e) => return e,
    };
    new_number_formula_arg(det(&num_mtx))
}

fn minverse(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let num_mtx = match number_matrix(&args[0], true) {
        Ok(m) => m,
        Err(e) => return e,
    };
    let det_m = det(&num_mtx);
    if det_m != 0.0 {
        let dat_m = 1.0 / det_m;
        let mut invert_m = adjugate_matrix(&num_mtx);
        for row in &mut invert_m {
            for cell in row {
                *cell *= dat_m;
            }
        }
        return new_matrix_formula_arg(formula_arg_matrix(&invert_m));
    }
    new_error_formula_arg(FORMULA_ERROR_NUM)
}

fn mmult(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let arr1 = &args[0];
    let arr2 = &args[1];
    if arr1.typ == ArgType::Number && arr2.typ == ArgType::Number {
        return new_number_formula_arg(arr1.number * arr2.number);
    }
    let num_mtx1 = match number_matrix(arr1, false) {
        Ok(m) => m,
        Err(e) => return e,
    };
    let num_mtx2 = match number_matrix(arr2, false) {
        Ok(m) => m,
        Err(e) => return e,
    };
    let array2_rows = num_mtx2.len();
    let array2_cols = num_mtx2[0].len();
    if num_mtx1[0].len() != array2_rows {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let mut out: Vec<Vec<f64>> = Vec::with_capacity(num_mtx1.len());
    for i in 0..num_mtx1.len() {
        let mut row = vec![0.0; array2_cols];
        let row1 = &num_mtx1[i];
        for j in 0..array2_cols {
            let mut sum = 0.0;
            for k in 0..array2_rows {
                sum += row1[k] * num_mtx2[k][j];
            }
            row[j] = sum;
        }
        out.push(row);
    }
    new_matrix_formula_arg(formula_arg_matrix(&out))
}

fn munit(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let dimension = num!(&args[0]);
    if dimension < 0.0 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let n = dimension as usize;
    let mut matrix = vec![vec![new_number_formula_arg(0.0); n]; n];
    for i in 0..n {
        matrix[i][i] = new_number_formula_arg(1.0);
    }
    new_matrix_formula_arg(matrix)
}

fn seriessum(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 4 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let x = num!(&args[0]);
    let n = num!(&args[1]);
    let m = num!(&args[2]);
    let mut result = 0.0;
    let mut i = 0.0;
    for coefficient in args[3].to_list() {
        if coefficient.value().is_empty() {
            continue;
        }
        let num = match to_number(&coefficient) {
            Ok(v) => v,
            Err(e) => return e,
        };
        result += num * x.powf(n + m * i);
        i += 1.0;
    }
    new_number_formula_arg(result)
}

// ------------------------------------------------------------------
// Standard deviation / variance helpers
// ------------------------------------------------------------------

fn calc_stdev_pow(result: f64, count: f64, n: f64, mean: f64) -> (f64, f64) {
    let new_result = if result == -1.0 {
        (n - mean).powi(2)
    } else {
        result + (n - mean).powi(2)
    };
    (new_result, count + 1.0)
}

fn calc_stdev(
    stdeva: bool,
    mut result: f64,
    mut count: f64,
    mean: f64,
    token: &FormulaArg,
) -> (f64, f64) {
    for row in token.to_list() {
        if row.typ == ArgType::Number || row.typ == ArgType::String {
            let val = row.value();
            if !stdeva && (val == "TRUE" || val == "FALSE") {
                continue;
            } else if stdeva && (val == "TRUE" || val == "FALSE") {
                if let Ok(b) = to_bool_number(&row) {
                    let (r, c) = calc_stdev_pow(result, count, b, mean);
                    result = r;
                    count = c;
                }
                continue;
            } else {
                if let Ok(n) = to_number(&row) {
                    let (r, c) = calc_stdev_pow(result, count, n, mean);
                    result = r;
                    count = c;
                }
            }
        }
    }
    (result, count)
}

fn stdev_impl_a(ctx: &CalcContext, stdeva: bool, args: &[FormulaArg]) -> FormulaArg {
    if args.is_empty() {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let mean = if stdeva {
        averagea(ctx, args)
    } else {
        average(ctx, args)
    };
    if mean.is_error() {
        return mean;
    }
    let mean = mean.number;
    let mut count = -1.0;
    let mut result = -1.0;
    for arg in args {
        match arg.typ {
            ArgType::String | ArgType::Number => {
                let val = arg.value();
                if !stdeva && (val == "TRUE" || val == "FALSE") {
                    continue;
                } else if stdeva && (val == "TRUE" || val == "FALSE") {
                    if let Ok(b) = to_bool_number(arg) {
                        let (r, c) = calc_stdev_pow(result, count, b, mean);
                        result = r;
                        count = c;
                    }
                    continue;
                } else {
                    if let Ok(n) = to_number(arg) {
                        let (r, c) = calc_stdev_pow(result, count, n, mean);
                        result = r;
                        count = c;
                    }
                }
            }
            ArgType::List | ArgType::Matrix => {
                let (r, c) = calc_stdev(stdeva, result, count, mean, arg);
                result = r;
                count = c;
            }
            _ => {}
        }
    }
    if count > 0.0 && result >= 0.0 {
        new_number_formula_arg((result / count).sqrt())
    } else {
        new_error_formula_arg(FORMULA_ERROR_DIV)
    }
}

fn stdev_fn(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    stdev_impl(true, args)
}

fn stdev_s(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    stdev_impl(true, args)
}

fn stdeva(ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    stdev_impl_a(ctx, true, args)
}

fn stdevp(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    stdev_impl(false, args)
}

fn stdev_impl(sample: bool, args: &[FormulaArg]) -> FormulaArg {
    let v = variance_impl(sample, args);
    if v.is_nan() {
        return new_error_formula_arg(FORMULA_ERROR_DIV);
    }
    new_number_formula_arg(v.sqrt())
}

fn variance_impl(sample: bool, args: &[FormulaArg]) -> f64 {
    fn collect_nums(args: &[FormulaArg]) -> Vec<f64> {
        let mut out = Vec::new();
        for a in args {
            match a.typ {
                ArgType::Number if !a.boolean => out.push(a.number),
                ArgType::List | ArgType::Matrix => out.extend(collect_nums(&a.to_list())),
                _ => {}
            }
        }
        out
    }
    let nums = collect_nums(args);
    let min_len = if sample { 2 } else { 1 };
    if nums.len() < min_len {
        return f64::NAN;
    }
    let mean = nums.iter().sum::<f64>() / nums.len() as f64;
    let sum_sq = nums.iter().map(|n| (n - mean).powi(2)).sum::<f64>();
    if sample {
        sum_sq / (nums.len() - 1) as f64
    } else {
        sum_sq / nums.len() as f64
    }
}

fn variance_sample(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    let v = variance_impl(true, args);
    if v.is_nan() {
        return new_error_formula_arg(FORMULA_ERROR_DIV);
    }
    new_number_formula_arg(v)
}

fn variance_pop(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    let v = variance_impl(false, args);
    if v.is_nan() {
        return new_error_formula_arg(FORMULA_ERROR_DIV);
    }
    new_number_formula_arg(v)
}

// ------------------------------------------------------------------
// Poisson / probability
// ------------------------------------------------------------------

fn poisson_impl(args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 3 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let x = num!(&args[0]);
    let mean = num!(&args[1]);
    let cumulative = to_bool_number(&args[2]).unwrap_or(0.0);
    if x < 0.0 || mean <= 0.0 {
        return new_error_formula_arg(FORMULA_ERROR_NA);
    }
    if cumulative == 1.0 {
        let mut summer = 0.0;
        let floor = x.floor() as i64;
        for i in 0..=floor {
            summer += mean.powi(i as i32) / fact(i as f64);
        }
        new_number_formula_arg((-mean).exp() * summer)
    } else {
        new_number_formula_arg((-mean).exp() * mean.powf(x) / fact(x))
    }
}

fn poisson(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    poisson_impl(args)
}

fn poisson_dist(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    poisson_impl(args)
}

fn prob(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() < 3 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    if args.len() > 4 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let x_range = &args[0];
    let prob_range = &args[1];
    let lower = num!(&args[2]);
    let upper = if args.len() == 4 {
        num!(&args[3])
    } else {
        lower
    };

    let n_r1 = x_range.matrix.len();
    let n_r2 = prob_range.matrix.len();
    if n_r1 == 0 || n_r2 == 0 {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    if n_r1 != n_r2 {
        return new_error_formula_arg(FORMULA_ERROR_NA);
    }
    let n_c1 = x_range.matrix[0].len();
    let n_c2 = prob_range.matrix[0].len();
    if n_c1 != n_c2 {
        return new_error_formula_arg(FORMULA_ERROR_NA);
    }

    let mut sum = 0.0;
    let mut res = 0.0;
    let mut stop = false;
    for r in 0..x_range.matrix.len() {
        for c in 0..x_range.matrix[0].len() {
            if stop {
                break;
            }
            let p = &prob_range.matrix[r][c];
            let x = &x_range.matrix[r][c];
            if p.typ == ArgType::Number && x.typ == ArgType::Number {
                let fp = p.number;
                let fw = x.number;
                if fp < 0.0 || fp > 1.0 {
                    stop = true;
                    continue;
                }
                sum += fp;
                if fw >= lower && fw <= upper {
                    res += fp;
                }
                continue;
            }
            return new_error_formula_arg(FORMULA_ERROR_NUM);
        }
    }
    if stop || (sum - 1.0).abs() > 1.0e-7 {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    new_number_formula_arg(res)
}

// ------------------------------------------------------------------
// Criteria helpers for SUMIF / SUMIFS
// ------------------------------------------------------------------

#[derive(Clone, Copy)]
enum CriteriaType {
    Eq,
    Le,
    Ge,
    Ne,
    Lt,
    Gt,
    Regex,
}

#[derive(Clone)]
struct FormulaCriteria {
    typ: CriteriaType,
    condition: FormulaArg,
}

fn formula_formats() -> &'static [Regex] {
    static RE: OnceLock<Vec<Regex>> = OnceLock::new();
    RE.get_or_init(|| {
        vec![
            Regex::new(r"^(\d+)$").unwrap(),
            Regex::new(r"^=(.*)$").unwrap(),
            Regex::new(r"^<>(.*)$").unwrap(),
            Regex::new(r"^<=(.*)$").unwrap(),
            Regex::new(r"^>=(.*)$").unwrap(),
            Regex::new(r"^<(.*)$").unwrap(),
            Regex::new(r"^>(.*)$").unwrap(),
        ]
    })
}

fn criteria_types() -> &'static [CriteriaType] {
    static TYPES: OnceLock<Vec<CriteriaType>> = OnceLock::new();
    TYPES.get_or_init(|| {
        vec![
            CriteriaType::Eq,
            CriteriaType::Eq,
            CriteriaType::Ne,
            CriteriaType::Le,
            CriteriaType::Ge,
            CriteriaType::Lt,
            CriteriaType::Gt,
        ]
    })
}

fn prepare_criteria_value(cond: &str) -> Result<f64, ()> {
    let mut s = cond.to_string();
    let mut percentile = 1.0;
    if s.ends_with('%') {
        s.pop();
        percentile /= 100.0;
    }
    s.parse::<f64>().map(|n| n * percentile).map_err(|_| ())
}

fn formula_criteria_parser(exp: &FormulaArg) -> FormulaCriteria {
    let val = exp.value();
    if val.is_empty() {
        return FormulaCriteria {
            typ: CriteriaType::Eq,
            condition: new_string_formula_arg(""),
        };
    }
    let formats = formula_formats();
    let types = criteria_types();
    for (re, t) in formats.iter().zip(types.iter()) {
        if let Some(caps) = re.captures(&val) {
            let cond = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let condition = if let Ok(n) = prepare_criteria_value(cond) {
                new_number_formula_arg(n)
            } else {
                new_string_formula_arg(cond.to_string())
            };
            return FormulaCriteria { typ: *t, condition };
        }
    }

    let re_wild = Regex::new(r"~[*?~]|[*?]|[\s\S]").unwrap();
    let mut has_wildcard = false;
    let mut pattern = String::new();
    for m in re_wild.find_iter(&val) {
        let tok = m.as_str();
        if tok == "*" || tok == "?" {
            has_wildcard = true;
        }
        match tok {
            "~*" => pattern.push_str(r"\*"),
            "~?" => pattern.push_str(r"\?"),
            "~~" => pattern.push('~'),
            "*" => pattern.push_str(".*"),
            "?" => pattern.push('.'),
            _ => pattern.push_str(&regex::escape(tok)),
        }
    }
    if has_wildcard {
        return FormulaCriteria {
            typ: CriteriaType::Regex,
            condition: new_string_formula_arg(format!("(?i)^{}$", pattern)),
        };
    }

    let unescaped = val
        .replace("~~", "\x01")
        .replace("~*", "\x02")
        .replace("~?", "\x03")
        .replace("\x01", "~")
        .replace("\x02", "*")
        .replace("\x03", "?");
    let condition = {
        let tmp = new_string_formula_arg(unescaped);
        if let Some(n) = tmp.to_number().as_number() {
            new_number_formula_arg(n)
        } else {
            tmp
        }
    };
    FormulaCriteria {
        typ: CriteriaType::Eq,
        condition,
    }
}

fn cmp_eq(cond: &FormulaArg, val: &FormulaArg) -> bool {
    if cond.typ == ArgType::String && val.typ == ArgType::String {
        cond.value().eq_ignore_ascii_case(&val.value())
    } else {
        cond.value() == val.value()
    }
}

fn cmp_ne(cond: &FormulaArg, val: &FormulaArg) -> bool {
    if cond.typ == ArgType::String && val.typ == ArgType::String {
        !cond.value().eq_ignore_ascii_case(&val.value())
    } else {
        cond.value() != val.value()
    }
}

fn cmp_l(cond: &FormulaArg, val: &FormulaArg) -> bool {
    if let (Some(a), Some(b)) = (cond.to_number().as_number(), val.to_number().as_number()) {
        a < b
    } else if cond.typ == ArgType::String && val.typ == ArgType::String {
        cond.value() < val.value()
    } else if cond.to_number().as_number().is_some() && val.typ == ArgType::String {
        false
    } else {
        true
    }
}

fn cmp_le(cond: &FormulaArg, val: &FormulaArg) -> bool {
    if let (Some(a), Some(b)) = (cond.to_number().as_number(), val.to_number().as_number()) {
        a <= b
    } else if cond.typ == ArgType::String && val.typ == ArgType::String {
        cond.value() <= val.value()
    } else if cond.to_number().as_number().is_some() && val.typ == ArgType::String {
        false
    } else {
        true
    }
}

fn cmp_g(cond: &FormulaArg, val: &FormulaArg) -> bool {
    if let (Some(a), Some(b)) = (cond.to_number().as_number(), val.to_number().as_number()) {
        a > b
    } else if cond.typ == ArgType::String && val.typ == ArgType::String {
        cond.value() > val.value()
    } else if cond.to_number().as_number().is_some() && val.typ == ArgType::String {
        true
    } else {
        false
    }
}

fn cmp_ge(cond: &FormulaArg, val: &FormulaArg) -> bool {
    if let (Some(a), Some(b)) = (cond.to_number().as_number(), val.to_number().as_number()) {
        a >= b
    } else if cond.typ == ArgType::String && val.typ == ArgType::String {
        cond.value() >= val.value()
    } else if cond.to_number().as_number().is_some() && val.typ == ArgType::String {
        true
    } else {
        false
    }
}

fn formula_criteria_eval(val: &FormulaArg, criteria: &FormulaCriteria) -> bool {
    match criteria.typ {
        CriteriaType::Eq => cmp_eq(&criteria.condition, val),
        CriteriaType::Ne => cmp_ne(&criteria.condition, val),
        CriteriaType::Lt => cmp_l(&criteria.condition, val),
        CriteriaType::Le => cmp_le(&criteria.condition, val),
        CriteriaType::Gt => cmp_g(&criteria.condition, val),
        CriteriaType::Ge => cmp_ge(&criteria.condition, val),
        CriteriaType::Regex => {
            let pat = criteria.condition.value();
            let re = Regex::new(&pat).unwrap_or_else(|_| Regex::new("^$").unwrap());
            re.is_match(&val.value())
        }
    }
}

// ------------------------------------------------------------------
// SUMIF / SUMIFS / SUMPRODUCT / SUMX*
// ------------------------------------------------------------------

fn sumif(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() < 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let criteria = formula_criteria_parser(&args[1]);
    let range_mtx = &args[0].matrix;
    let sum_range = if args.len() == 3 {
        &args[2].matrix
    } else {
        &[][..]
    };
    let mut sum = 0.0;
    for (row_idx, row) in range_mtx.iter().enumerate() {
        for (col_idx, cell) in row.iter().enumerate() {
            let mut arg = cell.clone();
            if arg.typ == ArgType::Empty {
                continue;
            }
            if formula_criteria_eval(&arg, &criteria) {
                if args.len() == 3 {
                    if row_idx < sum_range.len() && col_idx < sum_range[row_idx].len() {
                        arg = sum_range[row_idx][col_idx].clone();
                    }
                }
                if arg.typ == ArgType::Number {
                    sum += arg.number;
                }
            }
        }
    }
    new_number_formula_arg(sum)
}

#[derive(Clone)]
struct CellRef {
    row: usize,
    col: usize,
}

fn formula_ifs_match(args: &[FormulaArg]) -> Vec<CellRef> {
    let mut refs: Vec<CellRef> = Vec::new();
    let mut i = 0;
    while i < args.len() {
        let matrix = &args[i].matrix;
        let criteria = formula_criteria_parser(&args[i + 1]);
        if i == 0 {
            for (row_idx, row) in matrix.iter().enumerate() {
                for (col_idx, cell) in row.iter().enumerate() {
                    if formula_criteria_eval(cell, &criteria) {
                        refs.push(CellRef {
                            row: row_idx,
                            col: col_idx,
                        });
                    }
                }
            }
        } else {
            refs.retain(|r| {
                if r.row < matrix.len() && r.col < matrix[r.row].len() {
                    formula_criteria_eval(&matrix[r.row][r.col], &criteria)
                } else {
                    false
                }
            });
        }
        i += 2;
    }
    refs
}

fn sumifs(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() < 3 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    if args.len() % 2 != 1 {
        return new_error_formula_arg(FORMULA_ERROR_NA);
    }
    let sum_range = &args[0].matrix;
    let criteria_args = &args[1..];
    let mut sum = 0.0;
    for r in formula_ifs_match(criteria_args) {
        if r.row >= sum_range.len() || r.col >= sum_range[r.row].len() {
            return new_error_formula_arg(FORMULA_ERROR_VALUE);
        }
        if let Some(n) = sum_range[r.row][r.col].to_number().as_number() {
            sum += n;
        }
    }
    new_number_formula_arg(sum)
}

fn sumproduct_impl(ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    let mut arg_type = ArgType::Unknown;
    let mut n = 0usize;
    let mut res: Vec<f64> = Vec::new();
    let mut sum = 0.0;

    for arg in args {
        if arg_type == ArgType::Unknown {
            arg_type = arg.typ;
        }
        if arg.typ != arg_type {
            return new_error_formula_arg(FORMULA_ERROR_VALUE);
        }
        match arg.typ {
            ArgType::String | ArgType::Number => {
                if arg.to_number().as_number().is_some() {
                    sum = product(ctx, args).number;
                    continue;
                }
                return new_error_formula_arg(FORMULA_ERROR_VALUE);
            }
            ArgType::Matrix => {
                let list = arg.to_list();
                if res.is_empty() {
                    n = list.len();
                    res.resize(n, 1.0);
                }
                if list.len() != n {
                    return new_error_formula_arg(FORMULA_ERROR_VALUE);
                }
                for (i, value) in list.iter().enumerate() {
                    let txt = value.value();
                    let num = value.to_number();
                    if num.typ != ArgType::Number && !txt.is_empty() {
                        return new_error_formula_arg(FORMULA_ERROR_VALUE);
                    }
                    res[i] *= num.number;
                }
            }
            _ => {}
        }
    }
    for r in res {
        sum += r;
    }
    new_number_formula_arg(sum)
}

fn sumproduct(ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.is_empty() {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    for arg in args {
        if arg.typ == ArgType::Error {
            return arg.clone();
        }
    }
    sumproduct_impl(ctx, args)
}

fn sumx(name: &str, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let left = args[0].to_list();
    let right = args[1].to_list();
    if left.len() != right.len() {
        return new_error_formula_arg(FORMULA_ERROR_NA);
    }
    let mut result = 0.0;
    for i in 0..left.len() {
        let lhs = left[i].to_number().number;
        let rhs = right[i].to_number().number;
        if lhs != 0.0 && rhs != 0.0 {
            result += match name {
                "SUMX2MY2" => lhs * lhs - rhs * rhs,
                "SUMX2PY2" => lhs * lhs + rhs * rhs,
                _ => (lhs - rhs) * (lhs - rhs),
            };
        }
    }
    new_number_formula_arg(result)
}

fn sumx2my2(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    sumx("SUMX2MY2", args)
}

fn sumx2py2(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    sumx("SUMX2PY2", args)
}

fn sumxmy2(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    sumx("SUMXMY2", args)
}

// ------------------------------------------------------------------
// AGGREGATE / SUBTOTAL
// ------------------------------------------------------------------

fn aggregate(ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() < 3 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let fn_num = num!(&args[0]) as i32;
    let opts = num!(&args[1]) as i32;
    if opts < 0 || opts > 7 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let sub_args = &args[2..];
    match fn_num {
        1 => average(ctx, sub_args),
        2 => count(ctx, sub_args),
        3 => counta(ctx, sub_args),
        4 => max(ctx, sub_args),
        5 => min(ctx, sub_args),
        6 => product(ctx, sub_args),
        7 => stdev_s(ctx, sub_args),
        8 => stdevp(ctx, sub_args),
        9 => sum(ctx, sub_args),
        10 => variance_sample(ctx, sub_args),
        11 => variance_pop(ctx, sub_args),
        12 => median(ctx, sub_args),
        13 => mode_sngl(ctx, sub_args),
        14 => large(ctx, sub_args),
        15 => small(ctx, sub_args),
        16 => percentile_inc(ctx, sub_args),
        17 => quartile_inc(ctx, sub_args),
        18 => percentile_exc(ctx, sub_args),
        19 => quartile_exc(ctx, sub_args),
        _ => new_error_formula_arg(FORMULA_ERROR_VALUE),
    }
}

fn subtotal(ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() < 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let fn_num = num!(&args[0]) as i32;
    let sub_args = &args[1..];
    match fn_num {
        1 | 101 => average(ctx, sub_args),
        2 | 102 => count(ctx, sub_args),
        3 | 103 => counta(ctx, sub_args),
        4 | 104 => max(ctx, sub_args),
        5 | 105 => min(ctx, sub_args),
        6 | 106 => product(ctx, sub_args),
        7 | 107 => stdev_fn(ctx, sub_args),
        8 | 108 => stdevp(ctx, sub_args),
        9 | 109 => sum(ctx, sub_args),
        10 | 110 => variance_sample(ctx, sub_args),
        11 | 111 => variance_pop(ctx, sub_args),
        _ => new_error_formula_arg(FORMULA_ERROR_VALUE),
    }
}
