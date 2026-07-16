//! Engineering formula functions.

use std::collections::HashMap;
use std::str::FromStr;
use std::sync::OnceLock;

use num_complex::Complex64;

use crate::calc::arg::*;
use crate::calc::{CalcContext, FormulaFn};

pub fn register(m: &mut HashMap<&'static str, FormulaFn>) {
    m.insert("BESSELI", besseli);
    m.insert("BESSELJ", besselj);
    m.insert("BESSELK", besselk);
    m.insert("BESSELY", bessely);
    m.insert("BIN2DEC", bin2dec);
    m.insert("BIN2HEX", bin2hex);
    m.insert("BIN2OCT", bin2oct);
    m.insert("BITAND", bitand);
    m.insert("BITLSHIFT", bitlshift);
    m.insert("BITOR", bitor);
    m.insert("BITRSHIFT", bitrshift);
    m.insert("BITXOR", bitxor);
    m.insert("COMPLEX", complex);
    m.insert("CONVERT", convert);
    m.insert("DEC2BIN", dec2bin);
    m.insert("DEC2HEX", dec2hex);
    m.insert("DEC2OCT", dec2oct);
    m.insert("DELTA", delta);
    m.insert("ERF", erf);
    m.insert("ERFdotPRECISE", erfdotprecise);
    m.insert("ERFC", erfc);
    m.insert("ERFCdotPRECISE", erfcdotprecise);
    m.insert("GESTEP", gestep);
    m.insert("HEX2BIN", hex2bin);
    m.insert("HEX2DEC", hex2dec);
    m.insert("HEX2OCT", hex2oct);
    m.insert("IMABS", imabs);
    m.insert("IMAGINARY", imaginary);
    m.insert("IMARGUMENT", imargument);
    m.insert("IMCONJUGATE", imconjugate);
    m.insert("IMCOS", imcos);
    m.insert("IMCOSH", imcosh);
    m.insert("IMCOT", imcot);
    m.insert("IMCSC", imcsc);
    m.insert("IMCSCH", imcsch);
    m.insert("IMDIV", imdiv);
    m.insert("IMEXP", imexp);
    m.insert("IMLN", imln);
    m.insert("IMLOG10", imlog10);
    m.insert("IMLOG2", imlog2);
    m.insert("IMPOWER", impower);
    m.insert("IMPRODUCT", improduct);
    m.insert("IMREAL", imreal);
    m.insert("IMSEC", imsec);
    m.insert("IMSECH", imsech);
    m.insert("IMSIN", imsin);
    m.insert("IMSINH", imsinh);
    m.insert("IMSQRT", imsqrt);
    m.insert("IMSUB", imsub);
    m.insert("IMSUM", imsum);
    m.insert("IMTAN", imtan);
    m.insert("OCT2BIN", oct2bin);
    m.insert("OCT2DEC", oct2dec);
    m.insert("OCT2HEX", oct2hex);
}

// ------------------------------------------------------------------
// Bessel functions
// ------------------------------------------------------------------

fn fact(number: f64) -> f64 {
    let mut val = 1.0;
    let mut i = 2.0;
    while i <= number {
        val *= i;
        i += 1.0;
    }
    val
}

fn besseli(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let x = args[0].to_number();
    if x.typ != ArgType::Number {
        return x;
    }
    let n = args[1].to_number();
    if n.typ != ArgType::Number {
        return n;
    }
    new_number_formula_arg(bessel_i_j(x.number, n.number, true))
}

fn besselj(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let x = args[0].to_number();
    if x.typ != ArgType::Number {
        return x;
    }
    let n = args[1].to_number();
    if n.typ != ArgType::Number {
        return n;
    }
    new_number_formula_arg(bessel_i_j(x.number, n.number, false))
}

fn bessel_i_j(x: f64, n: f64, modified: bool) -> f64 {
    let mut max_val = 100;
    let mut x1 = x * 0.5;
    let x2 = x1 * x1;
    x1 = x1.powf(n);
    let mut n1 = fact(n);
    let mut n2 = 1.0;
    let mut n3 = 0.0;
    let mut n4 = n;
    let mut add = false;
    let mut result = x1 / n1;
    let mut t = result * 0.9;
    while result != t && max_val != 0 {
        x1 *= x2;
        n3 += 1.0;
        n1 *= n3;
        n4 += 1.0;
        n2 *= n4;
        t = result;
        let r = x1 / n1 / n2;
        if modified || add {
            result += r;
        } else {
            result -= r;
        }
        max_val -= 1;
        add = !add;
    }
    result
}

fn besselk(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let x = args[0].to_number();
    if x.typ != ArgType::Number {
        return x;
    }
    let n = args[1].to_number();
    if n.typ != ArgType::Number {
        return n;
    }
    if x.number <= 0.0 || n.number < 0.0 {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    let result = match n.number.floor() as i64 {
        0 => bessel_k0(x.number),
        1 => bessel_k1(x.number),
        _ => bessel_k2(x.number, n.number),
    };
    new_number_formula_arg(result)
}

fn bessel_k0(x: f64) -> f64 {
    if x <= 2.0 {
        let n2 = x * 0.5;
        let y = n2 * n2;
        -n2.ln() * bessel_i_j(x, 0.0, true)
            + (-0.57721566
                + y * (0.42278420
                    + y * (0.23069756
                        + y * (0.03488590 + y * (0.00262698 + y * (0.00010750 + y * 0.0000074))))))
    } else {
        let y = 2.0 / x;
        (-x).exp() / x.sqrt()
            * (1.25331414
                + y * (-0.07832358
                    + y * (0.02189568
                        + y * (-0.01062446
                            + y * (0.00587872 + y * (-0.00251540 + y * 0.00053208))))))
    }
}

fn bessel_k1(x: f64) -> f64 {
    if x <= 2.0 {
        let n2 = x * 0.5;
        let y = n2 * n2;
        n2.ln() * bessel_i_j(x, 1.0, true)
            + (1.0
                + y * (0.15443144
                    + y * (-0.67278579
                        + y * (-0.18156897
                            + y * (-0.01919402 + y * (-0.00110404 + y * (-0.00004686)))))))
                / x
    } else {
        let y = 2.0 / x;
        (-x).exp() / x.sqrt()
            * (1.25331414
                + y * (0.23498619
                    + y * (-0.03655620
                        + y * (0.01504268
                            + y * (-0.00780353 + y * (0.00325614 + y * (-0.00068245)))))))
    }
}

fn bessel_k2(x: f64, n: f64) -> f64 {
    let tox = 2.0 / x;
    let mut bkm = bessel_k0(x);
    let mut bk = bessel_k1(x);
    let mut bkp;
    let mut i = 1.0;
    while i < n {
        bkp = (i * tox).mul_add(bk, bkm);
        bkm = bk;
        bk = bkp;
        i += 1.0;
    }
    bk
}

fn bessely(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let x = args[0].to_number();
    if x.typ != ArgType::Number {
        return x;
    }
    let n = args[1].to_number();
    if n.typ != ArgType::Number {
        return n;
    }
    if x.number <= 0.0 || n.number < 0.0 {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    let result = match n.number.floor() as i64 {
        0 => bessel_y0(x.number),
        1 => bessel_y1(x.number),
        _ => bessel_y2(x.number, n.number),
    };
    new_number_formula_arg(result)
}

fn bessel_y0(x: f64) -> f64 {
    if x < 8.0 {
        let y = x * x;
        let f1 = -2957821389.0
            + y * (7062834065.0
                + y * (-512359803.6 + y * (10879881.29 + y * (-86327.92757 + y * 228.4622733))));
        let f2 = 40076544269.0
            + y * (745249964.8 + y * (7189466.438 + y * (47447.26470 + y * (226.1030244 + y))));
        f1 / f2 + 0.636619772 * bessel_i_j(x, 0.0, false) * x.ln()
    } else {
        let z = 8.0 / x;
        let y = z * z;
        let xx = x - 0.785398164;
        let f1 = 1.0
            + y * (-0.001098628627
                + y * (0.00002734510407 + y * (-0.000002073370639 + y * 0.0000002093887211)));
        let f2 = -0.015624999995
            + y * (0.0001430488765
                + y * (-0.000006911147651 + y * (0.0000007621095161 + y * (-0.0000000934945152))));
        (0.636619772 / x).sqrt() * (xx.sin() * f1 + z * xx.cos() * f2)
    }
}

fn bessel_y1(x: f64) -> f64 {
    if x < 8.0 {
        let y = x * x;
        let f1 = x
            * (-0.4900604943e13
                + y * (0.1275274390e13
                    + y * (-0.5153438139e11
                        + y * (0.7349264551e9 + y * (-0.4237922726e7 + y * 8511.937935e0)))));
        let f2 = 0.2499580570e14
            + y * (0.4244419664e12
                + y * (0.3733650367e10
                    + y * (0.2245904002e8 + y * (0.1020426050e6 + y * (354.9632885 + y)))));
        f1 / f2 + 0.636619772 * (bessel_i_j(x, 1.0, false) * x.ln() - 1.0 / x)
    } else {
        (0.636619772 / x).sqrt() * (x - 2.356194491).sin()
    }
}

fn bessel_y2(x: f64, n: f64) -> f64 {
    let tox = 2.0 / x;
    let mut bym = bessel_y0(x);
    let mut by = bessel_y1(x);
    let mut byp;
    let mut i = 1.0;
    while i < n {
        byp = (i * tox).mul_add(by, -bym);
        bym = by;
        by = byp;
        i += 1.0;
    }
    by
}

// ------------------------------------------------------------------
// Base conversion helpers
// ------------------------------------------------------------------

fn bin2dec_str(number: &str) -> Result<f64, String> {
    let length = number.len();
    let mut decimal = 0.0;
    for i in (1..=length).rev() {
        let idx = length - i;
        let s = number.chars().nth(idx).unwrap();
        if i == 10 && s == '1' {
            decimal += (-2.0f64).powi((i - 1) as i32);
            continue;
        }
        if s == '1' {
            decimal += 2.0f64.powi((i - 1) as i32);
            continue;
        }
        if s != '0' {
            return Err(FORMULA_ERROR_NUM.to_string());
        }
    }
    Ok(decimal)
}

fn oct2dec_str(number: &str) -> Result<f64, String> {
    let length = number.len();
    let mut decimal = 0.0;
    for i in (1..=length).rev() {
        let idx = length - i;
        let c = number.chars().nth(idx).unwrap();
        let num = c.to_digit(10).unwrap_or(0) as f64;
        if i == 10 && c == '7' {
            decimal += (-8.0f64).powi((i - 1) as i32);
            continue;
        }
        decimal += num * 8.0f64.powi((i - 1) as i32);
    }
    Ok(decimal)
}

fn hex2dec_str(number: &str) -> Result<f64, String> {
    let length = number.len();
    let mut decimal = 0.0;
    for i in (1..=length).rev() {
        let idx = length - i;
        let c = number.chars().nth(idx).unwrap();
        let num = i64::from_str_radix(&c.to_string(), 16).map_err(|e| e.to_string())?;
        if i == 10 && c.to_ascii_uppercase() == 'F' {
            decimal += (-16.0f64).powi((i - 1) as i32);
            continue;
        }
        decimal += (num as f64) * 16.0f64.powi((i - 1) as i32);
    }
    Ok(decimal)
}

fn dec2x(name: &str, args: &[FormulaArg]) -> FormulaArg {
    if args.is_empty() {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    if args.len() > 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let decimal = args[0].to_number();
    if decimal.typ != ArgType::Number {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }

    let max_limit = match name {
        "DEC2BIN" | "HEX2BIN" | "OCT2BIN" => 511.0,
        "BIN2HEX" | "DEC2HEX" | "OCT2HEX" => 549755813887.0,
        "BIN2OCT" | "DEC2OCT" | "HEX2OCT" => 536870911.0,
        _ => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let min_limit = -max_limit - 1.0;
    let base = match name {
        "DEC2BIN" | "HEX2BIN" | "OCT2BIN" => 2,
        "BIN2HEX" | "DEC2HEX" | "OCT2HEX" => 16,
        "BIN2OCT" | "DEC2OCT" | "HEX2OCT" => 8,
        _ => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };

    if decimal.number < min_limit || decimal.number > max_limit {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }

    let n = decimal.number as i64;
    let mut binary = format_radix(n as u64, base);

    if args.len() == 2 {
        let places = args[1].to_number();
        if places.typ != ArgType::Number {
            return new_error_formula_arg(FORMULA_ERROR_VALUE);
        }
        let binary_places = binary.len();
        if places.number < 0.0 || places.number > 10.0 || binary_places > places.number as usize {
            return new_error_formula_arg(FORMULA_ERROR_NUM);
        }
        let pad = places.number as usize - binary_places;
        binary = format!("{}{}", "0".repeat(pad), binary);
        return new_string_formula_arg(binary.to_uppercase());
    }

    if decimal.number < 0.0 && binary.len() > 10 {
        binary = binary[binary.len() - 10..].to_string();
    }
    new_string_formula_arg(binary.to_uppercase())
}

fn format_radix(mut x: u64, radix: i32) -> String {
    if x == 0 {
        return "0".to_string();
    }
    let mut result = String::new();
    while x > 0 {
        let digit = (x % radix as u64) as u32;
        let c = std::char::from_digit(digit, radix as u32).unwrap();
        result.push(c);
        x /= radix as u64;
    }
    result.chars().rev().collect()
}

fn bin2dec(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let token = &args[0];
    let number = token.to_number();
    if number.typ != ArgType::Number {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    match bin2dec_str(&token.value()) {
        Ok(d) => new_number_formula_arg(d),
        Err(_) => new_error_formula_arg(FORMULA_ERROR_NUM),
    }
}

fn bin2hex(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.is_empty() {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    if args.len() > 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let token = &args[0];
    let number = token.to_number();
    if number.typ != ArgType::Number {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let decimal = match bin2dec_str(&token.value()) {
        Ok(d) => new_number_formula_arg(d),
        Err(_) => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let mut new_args = vec![decimal];
    if args.len() == 2 {
        new_args.push(args[1].clone());
    }
    dec2x("BIN2HEX", &new_args)
}

fn bin2oct(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.is_empty() {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    if args.len() > 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let token = &args[0];
    let number = token.to_number();
    if number.typ != ArgType::Number {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let decimal = match bin2dec_str(&token.value()) {
        Ok(d) => new_number_formula_arg(d),
        Err(_) => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let mut new_args = vec![decimal];
    if args.len() == 2 {
        new_args.push(args[1].clone());
    }
    dec2x("BIN2OCT", &new_args)
}

fn dec2bin(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    dec2x("DEC2BIN", args)
}

fn dec2hex(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    dec2x("DEC2HEX", args)
}

fn dec2oct(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    dec2x("DEC2OCT", args)
}

fn hex2bin(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.is_empty() {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    if args.len() > 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let decimal = match hex2dec_str(&args[0].value()) {
        Ok(d) => new_number_formula_arg(d),
        Err(_) => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let mut new_args = vec![decimal];
    if args.len() == 2 {
        new_args.push(args[1].clone());
    }
    dec2x("HEX2BIN", &new_args)
}

fn hex2dec(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    match hex2dec_str(&args[0].value()) {
        Ok(d) => new_number_formula_arg(d),
        Err(_) => new_error_formula_arg(FORMULA_ERROR_NUM),
    }
}

fn hex2oct(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.is_empty() {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    if args.len() > 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let decimal = match hex2dec_str(&args[0].value()) {
        Ok(d) => new_number_formula_arg(d),
        Err(_) => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let mut new_args = vec![decimal];
    if args.len() == 2 {
        new_args.push(args[1].clone());
    }
    dec2x("HEX2OCT", &new_args)
}

fn oct2bin(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.is_empty() {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    if args.len() > 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let decimal = match oct2dec_str(&args[0].value()) {
        Ok(d) => new_number_formula_arg(d),
        Err(_) => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let mut new_args = vec![decimal];
    if args.len() == 2 {
        new_args.push(args[1].clone());
    }
    dec2x("OCT2BIN", &new_args)
}

fn oct2dec(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    match oct2dec_str(&args[0].value()) {
        Ok(d) => new_number_formula_arg(d),
        Err(_) => new_error_formula_arg(FORMULA_ERROR_NUM),
    }
}

fn oct2hex(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.is_empty() {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    if args.len() > 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let decimal = match oct2dec_str(&args[0].value()) {
        Ok(d) => new_number_formula_arg(d),
        Err(_) => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let mut new_args = vec![decimal];
    if args.len() == 2 {
        new_args.push(args[1].clone());
    }
    dec2x("OCT2HEX", &new_args)
}

// ------------------------------------------------------------------
// Bitwise functions
// ------------------------------------------------------------------

fn bitand(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    bitwise("BITAND", args)
}

fn bitlshift(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    bitwise("BITLSHIFT", args)
}

fn bitor(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    bitwise("BITOR", args)
}

fn bitrshift(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    bitwise("BITRSHIFT", args)
}

fn bitxor(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    bitwise("BITXOR", args)
}

fn bitwise(name: &str, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let num1 = args[0].to_number();
    let num2 = args[1].to_number();
    if num1.typ != ArgType::Number || num2.typ != ArgType::Number {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    let max_val = 2.0f64.powi(48) - 1.0;
    if num1.number < 0.0 || num1.number > max_val || num2.number < 0.0 || num2.number > max_val {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    let a = num1.number as i64;
    let b = num2.number as i64;
    let result = match name {
        "BITAND" => a & b,
        "BITLSHIFT" => a << (b as u32),
        "BITOR" => a | b,
        "BITRSHIFT" => a >> (b as u32),
        "BITXOR" => a ^ b,
        _ => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    new_number_formula_arg(result as f64)
}

// ------------------------------------------------------------------
// Complex number functions
// ------------------------------------------------------------------

fn str2cmplx(c: &str) -> String {
    let mut c = c.replace('j', "i");
    if c == "i" {
        c = "1i".to_string();
    }
    c = c.replace("+i", "+1i").replace("-i", "-1i");
    if c.ends_with('i') && !c.contains('+') && !c[..c.len() - 1].contains('-') {
        return format!("0+{}", c);
    }
    if !c.contains('i') {
        return format!("{}+0i", c);
    }
    c
}

fn parse_complex(value: &str) -> Result<Complex64, String> {
    let s = str2cmplx(value);
    Complex64::from_str(&s).map_err(|e| e.to_string())
}

fn complex_suffix(value: &str) -> &str {
    if value.to_ascii_lowercase().ends_with('j') {
        "j"
    } else {
        "i"
    }
}

fn fmt_float(x: f64) -> String {
    if x == 0.0 {
        return "0".to_string();
    }
    if x.fract() == 0.0 && x.abs() <= (i64::MAX as f64) {
        return format!("{}", x as i64);
    }
    let abs_x = x.abs();
    if abs_x >= 1e15 || (abs_x < 1e-15 && abs_x > 0.0) {
        return format!("{:.15E}", x);
    }
    let mut s = format!("{:.15}", x);
    if s.contains('.') {
        s = s.trim_end_matches('0').trim_end_matches('.').to_string();
    }
    if s == "-0" {
        s = "0".to_string();
    }
    s
}

fn cmplx2str(num: Complex64, suffix: &str) -> String {
    let real_part = fmt_float(num.re);
    let imag_part = fmt_float(num.im);
    let mut c = real_part.clone();
    if num.im > 0.0 {
        c.push('+');
    }
    if num.im != 0.0 {
        c.push_str(&imag_part);
        c.push('i');
    }
    c = c.trim_start_matches('(').trim_end_matches(')').to_string();
    if let Some(rest) = c.strip_prefix("+0+") {
        c = rest.to_string();
    }
    if let Some(rest) = c.strip_prefix("-0+") {
        c = rest.to_string();
    }
    if let Some(rest) = c.strip_prefix("0+") {
        c = rest.to_string();
    }
    if c.starts_with("0-") {
        c = format!("-{}", &c[2..]);
    }
    if let Some(rest) = c.strip_prefix("0+") {
        c = rest.to_string();
    }
    if c.ends_with("+0i") {
        c.truncate(c.len() - 3);
    } else if c.ends_with("-0i") {
        c.truncate(c.len() - 3);
    }
    c = c.replace("+1i", "+i").replace("-1i", "-i");
    c = c.replace('i', suffix);
    c
}

fn is_inf_complex(num: Complex64) -> bool {
    num.re.is_infinite() || num.im.is_infinite()
}

fn complex(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() < 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    if args.len() > 3 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let real_num = args[0].to_number();
    if real_num.typ != ArgType::Number {
        return real_num;
    }
    let i = args[1].to_number();
    if i.typ != ArgType::Number {
        return i;
    }
    let mut suffix = "i";
    if args.len() == 3 {
        let s = args[2].value().to_ascii_lowercase();
        if s != "i" && s != "j" {
            return new_error_formula_arg(FORMULA_ERROR_VALUE);
        }
        suffix = if s == "j" { "j" } else { "i" };
    }
    new_string_formula_arg(cmplx2str(Complex64::new(real_num.number, i.number), suffix))
}

fn imabs(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    match parse_complex(&args[0].value()) {
        Ok(num) => new_number_formula_arg(num.norm()),
        Err(_) => new_error_formula_arg(FORMULA_ERROR_NUM),
    }
}

fn imaginary(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    match parse_complex(&args[0].value()) {
        Ok(num) => new_number_formula_arg(num.im),
        Err(_) => new_error_formula_arg(FORMULA_ERROR_NUM),
    }
}

fn imargument(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    match parse_complex(&args[0].value()) {
        Ok(num) => new_number_formula_arg(num.arg()),
        Err(_) => new_error_formula_arg(FORMULA_ERROR_NUM),
    }
}

fn imconjugate(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let value = args[0].value();
    match parse_complex(&value) {
        Ok(num) => {
            let suffix = complex_suffix(&value);
            new_string_formula_arg(cmplx2str(num.conj(), suffix))
        }
        Err(_) => new_error_formula_arg(FORMULA_ERROR_NUM),
    }
}

fn imcos(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let value = args[0].value();
    match parse_complex(&value) {
        Ok(num) => new_string_formula_arg(cmplx2str(num.cos(), complex_suffix(&value))),
        Err(_) => new_error_formula_arg(FORMULA_ERROR_NUM),
    }
}

fn imcosh(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let value = args[0].value();
    match parse_complex(&value) {
        Ok(num) => new_string_formula_arg(cmplx2str(num.cosh(), complex_suffix(&value))),
        Err(_) => new_error_formula_arg(FORMULA_ERROR_NUM),
    }
}

fn imcot(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let value = args[0].value();
    match parse_complex(&value) {
        Ok(num) => new_string_formula_arg(cmplx2str(num.cos() / num.sin(), complex_suffix(&value))),
        Err(_) => new_error_formula_arg(FORMULA_ERROR_NUM),
    }
}

fn imcsc(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let value = args[0].value();
    match parse_complex(&value) {
        Ok(num) => {
            let result = Complex64::new(1.0, 0.0) / num.sin();
            if is_inf_complex(result) {
                return new_error_formula_arg(FORMULA_ERROR_NUM);
            }
            new_string_formula_arg(cmplx2str(result, complex_suffix(&value)))
        }
        Err(_) => new_error_formula_arg(FORMULA_ERROR_NUM),
    }
}

fn imcsch(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let value = args[0].value();
    match parse_complex(&value) {
        Ok(num) => {
            let x = num.re;
            let y = num.im;
            let denom = (2.0 * x).cosh() - (2.0 * y).cos();
            if denom == 0.0 {
                return new_error_formula_arg(FORMULA_ERROR_NUM);
            }
            let result = Complex64::new(
                (x.sinh() * y.cos() * 2.0) / denom,
                -(x.cosh() * y.sin() * 2.0) / denom,
            );
            new_string_formula_arg(cmplx2str(result, complex_suffix(&value)))
        }
        Err(_) => new_error_formula_arg(FORMULA_ERROR_NUM),
    }
}

fn imdiv(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let value = args[0].value();
    let num1 = match parse_complex(&value) {
        Ok(n) => n,
        Err(_) => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let num2 = match parse_complex(&args[1].value()) {
        Ok(n) => n,
        Err(_) => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    if num2 == Complex64::new(0.0, 0.0) {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    let result = num1 / num2;
    if is_inf_complex(result) {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    new_string_formula_arg(cmplx2str(result, complex_suffix(&value)))
}

fn imexp(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let value = args[0].value();
    match parse_complex(&value) {
        Ok(num) => new_string_formula_arg(cmplx2str(num.exp(), complex_suffix(&value))),
        Err(_) => new_error_formula_arg(FORMULA_ERROR_NUM),
    }
}

fn imln(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let value = args[0].value();
    match parse_complex(&value) {
        Ok(num) => {
            let result = num.ln();
            if is_inf_complex(result) {
                return new_error_formula_arg(FORMULA_ERROR_NUM);
            }
            new_string_formula_arg(cmplx2str(result, complex_suffix(&value)))
        }
        Err(_) => new_error_formula_arg(FORMULA_ERROR_NUM),
    }
}

fn imlog10(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let value = args[0].value();
    match parse_complex(&value) {
        Ok(num) => {
            let result = num.ln() / 10.0f64.ln();
            if is_inf_complex(result) {
                return new_error_formula_arg(FORMULA_ERROR_NUM);
            }
            new_string_formula_arg(cmplx2str(result, complex_suffix(&value)))
        }
        Err(_) => new_error_formula_arg(FORMULA_ERROR_NUM),
    }
}

fn imlog2(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let value = args[0].value();
    match parse_complex(&value) {
        Ok(num) => {
            let result = num.ln() / 2.0f64.ln();
            if is_inf_complex(result) {
                return new_error_formula_arg(FORMULA_ERROR_NUM);
            }
            new_string_formula_arg(cmplx2str(result, complex_suffix(&value)))
        }
        Err(_) => new_error_formula_arg(FORMULA_ERROR_NUM),
    }
}

fn impower(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let value = args[0].value();
    let base = match parse_complex(&value) {
        Ok(n) => n,
        Err(_) => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let exp = match parse_complex(&args[1].value()) {
        Ok(n) => n,
        Err(_) => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    if base == Complex64::new(0.0, 0.0) && exp == Complex64::new(0.0, 0.0) {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    let result = base.powc(exp);
    if is_inf_complex(result) {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    new_string_formula_arg(cmplx2str(result, complex_suffix(&value)))
}

fn improduct(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    let mut product = Complex64::new(1.0, 0.0);
    for arg in args {
        match arg.typ {
            ArgType::String => {
                if arg.value().is_empty() {
                    continue;
                }
                match parse_complex(&arg.value()) {
                    Ok(n) => product *= n,
                    Err(_) => return new_error_formula_arg(FORMULA_ERROR_NUM),
                }
            }
            ArgType::Number => {
                product *= Complex64::new(arg.number, 0.0);
            }
            ArgType::Matrix => {
                for row in &arg.matrix {
                    for value in row {
                        if value.value().is_empty() {
                            continue;
                        }
                        match parse_complex(&value.value()) {
                            Ok(n) => product *= n,
                            Err(_) => return new_error_formula_arg(FORMULA_ERROR_NUM),
                        }
                    }
                }
            }
            _ => {}
        }
    }
    new_string_formula_arg(cmplx2str(product, "i"))
}

fn imreal(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    match parse_complex(&args[0].value()) {
        Ok(num) => new_string_formula_arg(fmt_float(num.re)),
        Err(_) => new_error_formula_arg(FORMULA_ERROR_NUM),
    }
}

fn imsec(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let value = args[0].value();
    match parse_complex(&value) {
        Ok(num) => new_string_formula_arg(cmplx2str(
            Complex64::new(1.0, 0.0) / num.cos(),
            complex_suffix(&value),
        )),
        Err(_) => new_error_formula_arg(FORMULA_ERROR_NUM),
    }
}

fn imsech(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let value = args[0].value();
    match parse_complex(&value) {
        Ok(num) => {
            let a = num.re;
            let b = num.im;
            let cos_b = b.cos();
            let sinh_a = a.sinh();
            let denom = cos_b * cos_b + sinh_a * sinh_a;
            let result = Complex64::new(a.cosh() * cos_b / denom, -(sinh_a * b.sin()) / denom);
            new_string_formula_arg(cmplx2str(result, complex_suffix(&value)))
        }
        Err(_) => new_error_formula_arg(FORMULA_ERROR_NUM),
    }
}

fn imsin(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let value = args[0].value();
    match parse_complex(&value) {
        Ok(num) => new_string_formula_arg(cmplx2str(num.sin(), complex_suffix(&value))),
        Err(_) => new_error_formula_arg(FORMULA_ERROR_NUM),
    }
}

fn imsinh(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let value = args[0].value();
    match parse_complex(&value) {
        Ok(num) => new_string_formula_arg(cmplx2str(num.sinh(), complex_suffix(&value))),
        Err(_) => new_error_formula_arg(FORMULA_ERROR_NUM),
    }
}

fn imsqrt(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let value = args[0].value();
    match parse_complex(&value) {
        Ok(num) => {
            let a = num.re;
            let b = num.im;
            let modulus = a.hypot(b);
            let sqrt_mod = modulus.sqrt();
            let arg = b.atan2(a);
            let re = sqrt_mod * (arg / 2.0).cos();
            let im = sqrt_mod * (arg / 2.0).sin();
            new_string_formula_arg(cmplx2str(Complex64::new(re, im), complex_suffix(&value)))
        }
        Err(_) => new_error_formula_arg(FORMULA_ERROR_NUM),
    }
}

fn imsub(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let i1 = match parse_complex(&args[0].value()) {
        Ok(n) => n,
        Err(_) => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let i2 = match parse_complex(&args[1].value()) {
        Ok(n) => n,
        Err(_) => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    new_string_formula_arg(cmplx2str(i1 - i2, "i"))
}

fn imsum(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.is_empty() {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let mut result = Complex64::new(0.0, 0.0);
    for arg in args {
        match parse_complex(&arg.value()) {
            Ok(n) => result += n,
            Err(_) => return new_error_formula_arg(FORMULA_ERROR_NUM),
        }
    }
    new_string_formula_arg(cmplx2str(result, "i"))
}

fn imtan(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let value = args[0].value();
    match parse_complex(&value) {
        Ok(num) => new_string_formula_arg(cmplx2str(num.tan(), complex_suffix(&value))),
        Err(_) => new_error_formula_arg(FORMULA_ERROR_NUM),
    }
}

// ------------------------------------------------------------------
// Unit conversion
// ------------------------------------------------------------------

#[derive(Clone, Copy)]
struct ConversionUnit {
    group: u8,
    allow_prefix: bool,
}

fn conversion_units() -> &'static HashMap<&'static str, ConversionUnit> {
    static UNITS: OnceLock<HashMap<&'static str, ConversionUnit>> = OnceLock::new();
    UNITS.get_or_init(|| {
        HashMap::from([
            (
                "g",
                ConversionUnit {
                    group: 9,
                    allow_prefix: true,
                },
            ),
            (
                "sg",
                ConversionUnit {
                    group: 9,
                    allow_prefix: false,
                },
            ),
            (
                "lbm",
                ConversionUnit {
                    group: 9,
                    allow_prefix: false,
                },
            ),
            (
                "u",
                ConversionUnit {
                    group: 9,
                    allow_prefix: true,
                },
            ),
            (
                "ozm",
                ConversionUnit {
                    group: 9,
                    allow_prefix: false,
                },
            ),
            (
                "grain",
                ConversionUnit {
                    group: 9,
                    allow_prefix: false,
                },
            ),
            (
                "cwt",
                ConversionUnit {
                    group: 9,
                    allow_prefix: false,
                },
            ),
            (
                "shweight",
                ConversionUnit {
                    group: 9,
                    allow_prefix: false,
                },
            ),
            (
                "uk_cwt",
                ConversionUnit {
                    group: 9,
                    allow_prefix: false,
                },
            ),
            (
                "lcwt",
                ConversionUnit {
                    group: 9,
                    allow_prefix: false,
                },
            ),
            (
                "hweight",
                ConversionUnit {
                    group: 9,
                    allow_prefix: false,
                },
            ),
            (
                "stone",
                ConversionUnit {
                    group: 9,
                    allow_prefix: false,
                },
            ),
            (
                "ton",
                ConversionUnit {
                    group: 9,
                    allow_prefix: false,
                },
            ),
            (
                "uk_ton",
                ConversionUnit {
                    group: 9,
                    allow_prefix: false,
                },
            ),
            (
                "LTON",
                ConversionUnit {
                    group: 9,
                    allow_prefix: false,
                },
            ),
            (
                "brton",
                ConversionUnit {
                    group: 9,
                    allow_prefix: false,
                },
            ),
            (
                "m",
                ConversionUnit {
                    group: 10,
                    allow_prefix: true,
                },
            ),
            (
                "mi",
                ConversionUnit {
                    group: 10,
                    allow_prefix: false,
                },
            ),
            (
                "Nmi",
                ConversionUnit {
                    group: 10,
                    allow_prefix: false,
                },
            ),
            (
                "in",
                ConversionUnit {
                    group: 10,
                    allow_prefix: false,
                },
            ),
            (
                "ft",
                ConversionUnit {
                    group: 10,
                    allow_prefix: false,
                },
            ),
            (
                "yd",
                ConversionUnit {
                    group: 10,
                    allow_prefix: false,
                },
            ),
            (
                "ang",
                ConversionUnit {
                    group: 10,
                    allow_prefix: true,
                },
            ),
            (
                "ell",
                ConversionUnit {
                    group: 10,
                    allow_prefix: false,
                },
            ),
            (
                "ly",
                ConversionUnit {
                    group: 10,
                    allow_prefix: false,
                },
            ),
            (
                "parsec",
                ConversionUnit {
                    group: 10,
                    allow_prefix: false,
                },
            ),
            (
                "pc",
                ConversionUnit {
                    group: 10,
                    allow_prefix: false,
                },
            ),
            (
                "Pica",
                ConversionUnit {
                    group: 10,
                    allow_prefix: false,
                },
            ),
            (
                "Picapt",
                ConversionUnit {
                    group: 10,
                    allow_prefix: false,
                },
            ),
            (
                "pica",
                ConversionUnit {
                    group: 10,
                    allow_prefix: false,
                },
            ),
            (
                "survey_mi",
                ConversionUnit {
                    group: 10,
                    allow_prefix: false,
                },
            ),
            (
                "yr",
                ConversionUnit {
                    group: 11,
                    allow_prefix: false,
                },
            ),
            (
                "day",
                ConversionUnit {
                    group: 11,
                    allow_prefix: false,
                },
            ),
            (
                "d",
                ConversionUnit {
                    group: 11,
                    allow_prefix: false,
                },
            ),
            (
                "hr",
                ConversionUnit {
                    group: 11,
                    allow_prefix: false,
                },
            ),
            (
                "mn",
                ConversionUnit {
                    group: 11,
                    allow_prefix: false,
                },
            ),
            (
                "min",
                ConversionUnit {
                    group: 11,
                    allow_prefix: false,
                },
            ),
            (
                "sec",
                ConversionUnit {
                    group: 11,
                    allow_prefix: true,
                },
            ),
            (
                "s",
                ConversionUnit {
                    group: 11,
                    allow_prefix: true,
                },
            ),
            (
                "Pa",
                ConversionUnit {
                    group: 12,
                    allow_prefix: true,
                },
            ),
            (
                "p",
                ConversionUnit {
                    group: 12,
                    allow_prefix: true,
                },
            ),
            (
                "atm",
                ConversionUnit {
                    group: 12,
                    allow_prefix: true,
                },
            ),
            (
                "at",
                ConversionUnit {
                    group: 12,
                    allow_prefix: true,
                },
            ),
            (
                "mmHg",
                ConversionUnit {
                    group: 12,
                    allow_prefix: true,
                },
            ),
            (
                "psi",
                ConversionUnit {
                    group: 12,
                    allow_prefix: true,
                },
            ),
            (
                "Torr",
                ConversionUnit {
                    group: 12,
                    allow_prefix: true,
                },
            ),
            (
                "N",
                ConversionUnit {
                    group: 13,
                    allow_prefix: true,
                },
            ),
            (
                "dyn",
                ConversionUnit {
                    group: 13,
                    allow_prefix: true,
                },
            ),
            (
                "dy",
                ConversionUnit {
                    group: 13,
                    allow_prefix: true,
                },
            ),
            (
                "lbf",
                ConversionUnit {
                    group: 13,
                    allow_prefix: false,
                },
            ),
            (
                "pond",
                ConversionUnit {
                    group: 13,
                    allow_prefix: true,
                },
            ),
            (
                "J",
                ConversionUnit {
                    group: 14,
                    allow_prefix: true,
                },
            ),
            (
                "e",
                ConversionUnit {
                    group: 14,
                    allow_prefix: true,
                },
            ),
            (
                "c",
                ConversionUnit {
                    group: 14,
                    allow_prefix: true,
                },
            ),
            (
                "cal",
                ConversionUnit {
                    group: 14,
                    allow_prefix: true,
                },
            ),
            (
                "eV",
                ConversionUnit {
                    group: 14,
                    allow_prefix: true,
                },
            ),
            (
                "ev",
                ConversionUnit {
                    group: 14,
                    allow_prefix: true,
                },
            ),
            (
                "HPh",
                ConversionUnit {
                    group: 14,
                    allow_prefix: false,
                },
            ),
            (
                "hh",
                ConversionUnit {
                    group: 14,
                    allow_prefix: false,
                },
            ),
            (
                "Wh",
                ConversionUnit {
                    group: 14,
                    allow_prefix: true,
                },
            ),
            (
                "wh",
                ConversionUnit {
                    group: 14,
                    allow_prefix: true,
                },
            ),
            (
                "flb",
                ConversionUnit {
                    group: 14,
                    allow_prefix: false,
                },
            ),
            (
                "BTU",
                ConversionUnit {
                    group: 14,
                    allow_prefix: false,
                },
            ),
            (
                "btu",
                ConversionUnit {
                    group: 14,
                    allow_prefix: false,
                },
            ),
            (
                "HP",
                ConversionUnit {
                    group: 15,
                    allow_prefix: false,
                },
            ),
            (
                "h",
                ConversionUnit {
                    group: 15,
                    allow_prefix: false,
                },
            ),
            (
                "W",
                ConversionUnit {
                    group: 15,
                    allow_prefix: true,
                },
            ),
            (
                "w",
                ConversionUnit {
                    group: 15,
                    allow_prefix: true,
                },
            ),
            (
                "PS",
                ConversionUnit {
                    group: 15,
                    allow_prefix: false,
                },
            ),
            (
                "T",
                ConversionUnit {
                    group: 16,
                    allow_prefix: true,
                },
            ),
            (
                "ga",
                ConversionUnit {
                    group: 16,
                    allow_prefix: true,
                },
            ),
            (
                "C",
                ConversionUnit {
                    group: 17,
                    allow_prefix: false,
                },
            ),
            (
                "cel",
                ConversionUnit {
                    group: 17,
                    allow_prefix: false,
                },
            ),
            (
                "F",
                ConversionUnit {
                    group: 17,
                    allow_prefix: false,
                },
            ),
            (
                "fah",
                ConversionUnit {
                    group: 17,
                    allow_prefix: false,
                },
            ),
            (
                "K",
                ConversionUnit {
                    group: 17,
                    allow_prefix: false,
                },
            ),
            (
                "kel",
                ConversionUnit {
                    group: 17,
                    allow_prefix: false,
                },
            ),
            (
                "Rank",
                ConversionUnit {
                    group: 17,
                    allow_prefix: false,
                },
            ),
            (
                "Reau",
                ConversionUnit {
                    group: 17,
                    allow_prefix: false,
                },
            ),
            (
                "l",
                ConversionUnit {
                    group: 18,
                    allow_prefix: true,
                },
            ),
            (
                "L",
                ConversionUnit {
                    group: 18,
                    allow_prefix: true,
                },
            ),
            (
                "lt",
                ConversionUnit {
                    group: 18,
                    allow_prefix: true,
                },
            ),
            (
                "tsp",
                ConversionUnit {
                    group: 18,
                    allow_prefix: false,
                },
            ),
            (
                "tspm",
                ConversionUnit {
                    group: 18,
                    allow_prefix: false,
                },
            ),
            (
                "tbs",
                ConversionUnit {
                    group: 18,
                    allow_prefix: false,
                },
            ),
            (
                "oz",
                ConversionUnit {
                    group: 18,
                    allow_prefix: false,
                },
            ),
            (
                "cup",
                ConversionUnit {
                    group: 18,
                    allow_prefix: false,
                },
            ),
            (
                "pt",
                ConversionUnit {
                    group: 18,
                    allow_prefix: false,
                },
            ),
            (
                "us_pt",
                ConversionUnit {
                    group: 18,
                    allow_prefix: false,
                },
            ),
            (
                "uk_pt",
                ConversionUnit {
                    group: 18,
                    allow_prefix: false,
                },
            ),
            (
                "qt",
                ConversionUnit {
                    group: 18,
                    allow_prefix: false,
                },
            ),
            (
                "uk_qt",
                ConversionUnit {
                    group: 18,
                    allow_prefix: false,
                },
            ),
            (
                "gal",
                ConversionUnit {
                    group: 18,
                    allow_prefix: false,
                },
            ),
            (
                "uk_gal",
                ConversionUnit {
                    group: 18,
                    allow_prefix: false,
                },
            ),
            (
                "ang3",
                ConversionUnit {
                    group: 18,
                    allow_prefix: true,
                },
            ),
            (
                "ang^3",
                ConversionUnit {
                    group: 18,
                    allow_prefix: true,
                },
            ),
            (
                "barrel",
                ConversionUnit {
                    group: 18,
                    allow_prefix: false,
                },
            ),
            (
                "bushel",
                ConversionUnit {
                    group: 18,
                    allow_prefix: false,
                },
            ),
            (
                "in3",
                ConversionUnit {
                    group: 18,
                    allow_prefix: false,
                },
            ),
            (
                "in^3",
                ConversionUnit {
                    group: 18,
                    allow_prefix: false,
                },
            ),
            (
                "ft3",
                ConversionUnit {
                    group: 18,
                    allow_prefix: false,
                },
            ),
            (
                "ft^3",
                ConversionUnit {
                    group: 18,
                    allow_prefix: false,
                },
            ),
            (
                "ly3",
                ConversionUnit {
                    group: 18,
                    allow_prefix: false,
                },
            ),
            (
                "ly^3",
                ConversionUnit {
                    group: 18,
                    allow_prefix: false,
                },
            ),
            (
                "m3",
                ConversionUnit {
                    group: 18,
                    allow_prefix: true,
                },
            ),
            (
                "m^3",
                ConversionUnit {
                    group: 18,
                    allow_prefix: true,
                },
            ),
            (
                "mi3",
                ConversionUnit {
                    group: 18,
                    allow_prefix: false,
                },
            ),
            (
                "mi^3",
                ConversionUnit {
                    group: 18,
                    allow_prefix: false,
                },
            ),
            (
                "yd3",
                ConversionUnit {
                    group: 18,
                    allow_prefix: false,
                },
            ),
            (
                "yd^3",
                ConversionUnit {
                    group: 18,
                    allow_prefix: false,
                },
            ),
            (
                "Nmi3",
                ConversionUnit {
                    group: 18,
                    allow_prefix: false,
                },
            ),
            (
                "Nmi^3",
                ConversionUnit {
                    group: 18,
                    allow_prefix: false,
                },
            ),
            (
                "Pica3",
                ConversionUnit {
                    group: 18,
                    allow_prefix: false,
                },
            ),
            (
                "Pica^3",
                ConversionUnit {
                    group: 18,
                    allow_prefix: false,
                },
            ),
            (
                "Picapt3",
                ConversionUnit {
                    group: 18,
                    allow_prefix: false,
                },
            ),
            (
                "Picapt^3",
                ConversionUnit {
                    group: 18,
                    allow_prefix: false,
                },
            ),
            (
                "GRT",
                ConversionUnit {
                    group: 18,
                    allow_prefix: false,
                },
            ),
            (
                "regton",
                ConversionUnit {
                    group: 18,
                    allow_prefix: false,
                },
            ),
            (
                "MTON",
                ConversionUnit {
                    group: 18,
                    allow_prefix: false,
                },
            ),
            (
                "ha",
                ConversionUnit {
                    group: 19,
                    allow_prefix: true,
                },
            ),
            (
                "uk_acre",
                ConversionUnit {
                    group: 19,
                    allow_prefix: false,
                },
            ),
            (
                "us_acre",
                ConversionUnit {
                    group: 19,
                    allow_prefix: false,
                },
            ),
            (
                "ang2",
                ConversionUnit {
                    group: 19,
                    allow_prefix: true,
                },
            ),
            (
                "ang^2",
                ConversionUnit {
                    group: 19,
                    allow_prefix: true,
                },
            ),
            (
                "ar",
                ConversionUnit {
                    group: 19,
                    allow_prefix: true,
                },
            ),
            (
                "ft2",
                ConversionUnit {
                    group: 19,
                    allow_prefix: false,
                },
            ),
            (
                "ft^2",
                ConversionUnit {
                    group: 19,
                    allow_prefix: false,
                },
            ),
            (
                "in2",
                ConversionUnit {
                    group: 19,
                    allow_prefix: false,
                },
            ),
            (
                "in^2",
                ConversionUnit {
                    group: 19,
                    allow_prefix: false,
                },
            ),
            (
                "ly2",
                ConversionUnit {
                    group: 19,
                    allow_prefix: false,
                },
            ),
            (
                "ly^2",
                ConversionUnit {
                    group: 19,
                    allow_prefix: false,
                },
            ),
            (
                "m2",
                ConversionUnit {
                    group: 19,
                    allow_prefix: true,
                },
            ),
            (
                "m^2",
                ConversionUnit {
                    group: 19,
                    allow_prefix: true,
                },
            ),
            (
                "Morgen",
                ConversionUnit {
                    group: 19,
                    allow_prefix: false,
                },
            ),
            (
                "mi2",
                ConversionUnit {
                    group: 19,
                    allow_prefix: false,
                },
            ),
            (
                "mi^2",
                ConversionUnit {
                    group: 19,
                    allow_prefix: false,
                },
            ),
            (
                "Nmi2",
                ConversionUnit {
                    group: 19,
                    allow_prefix: false,
                },
            ),
            (
                "Nmi^2",
                ConversionUnit {
                    group: 19,
                    allow_prefix: false,
                },
            ),
            (
                "Pica2",
                ConversionUnit {
                    group: 19,
                    allow_prefix: false,
                },
            ),
            (
                "Pica^2",
                ConversionUnit {
                    group: 19,
                    allow_prefix: false,
                },
            ),
            (
                "Picapt2",
                ConversionUnit {
                    group: 19,
                    allow_prefix: false,
                },
            ),
            (
                "Picapt^2",
                ConversionUnit {
                    group: 19,
                    allow_prefix: false,
                },
            ),
            (
                "yd2",
                ConversionUnit {
                    group: 19,
                    allow_prefix: false,
                },
            ),
            (
                "yd^2",
                ConversionUnit {
                    group: 19,
                    allow_prefix: false,
                },
            ),
            (
                "byte",
                ConversionUnit {
                    group: 20,
                    allow_prefix: true,
                },
            ),
            (
                "bit",
                ConversionUnit {
                    group: 20,
                    allow_prefix: true,
                },
            ),
            (
                "m/s",
                ConversionUnit {
                    group: 21,
                    allow_prefix: true,
                },
            ),
            (
                "m/sec",
                ConversionUnit {
                    group: 21,
                    allow_prefix: true,
                },
            ),
            (
                "m/h",
                ConversionUnit {
                    group: 21,
                    allow_prefix: true,
                },
            ),
            (
                "m/hr",
                ConversionUnit {
                    group: 21,
                    allow_prefix: true,
                },
            ),
            (
                "mph",
                ConversionUnit {
                    group: 21,
                    allow_prefix: false,
                },
            ),
            (
                "admkn",
                ConversionUnit {
                    group: 21,
                    allow_prefix: false,
                },
            ),
            (
                "kn",
                ConversionUnit {
                    group: 21,
                    allow_prefix: false,
                },
            ),
        ])
    })
}

fn conversion_multipliers() -> &'static HashMap<&'static str, f64> {
    static MULTIPLIERS: OnceLock<HashMap<&'static str, f64>> = OnceLock::new();
    MULTIPLIERS.get_or_init(|| {
        HashMap::from([
            ("Y", 1e24),
            ("Z", 1e21),
            ("E", 1e18),
            ("P", 1e15),
            ("T", 1e12),
            ("G", 1e9),
            ("M", 1e6),
            ("k", 1e3),
            ("h", 1e2),
            ("e", 1e1),
            ("da", 1e1),
            ("d", 1e-1),
            ("c", 1e-2),
            ("m", 1e-3),
            ("u", 1e-6),
            ("n", 1e-9),
            ("p", 1e-12),
            ("f", 1e-15),
            ("a", 1e-18),
            ("z", 1e-21),
            ("y", 1e-24),
            ("Yi", 2.0f64.powi(80)),
            ("Zi", 2.0f64.powi(70)),
            ("Ei", 2.0f64.powi(60)),
            ("Pi", 2.0f64.powi(50)),
            ("Ti", 2.0f64.powi(40)),
            ("Gi", 2.0f64.powi(30)),
            ("Mi", 2.0f64.powi(20)),
            ("ki", 2.0f64.powi(10)),
        ])
    })
}

fn unit_conversions() -> &'static HashMap<u8, HashMap<&'static str, f64>> {
    static CONVERSIONS: OnceLock<HashMap<u8, HashMap<&'static str, f64>>> = OnceLock::new();
    CONVERSIONS.get_or_init(|| {
        HashMap::from([
            (
                9,
                HashMap::from([
                    ("g", 1.0),
                    ("sg", 6.85217658567918e-05),
                    ("lbm", 2.20462262184878e-03),
                    ("u", 6.02214179421676e+23),
                    ("ozm", 3.52739619495804e-02),
                    ("grain", 1.54323583529414e+01),
                    ("cwt", 2.20462262184878e-05),
                    ("shweight", 2.20462262184878e-05),
                    ("uk_cwt", 1.96841305522212e-05),
                    ("lcwt", 1.96841305522212e-05),
                    ("hweight", 1.96841305522212e-05),
                    ("stone", 1.57473044417770e-04),
                    ("ton", 1.10231131092439e-06),
                    ("uk_ton", 9.84206527611061e-07),
                    ("LTON", 9.84206527611061e-07),
                    ("brton", 9.84206527611061e-07),
                ]),
            ),
            (
                10,
                HashMap::from([
                    ("m", 1.0),
                    ("mi", 6.21371192237334e-04),
                    ("Nmi", 5.39956803455724e-04),
                    ("in", 3.93700787401575e+01),
                    ("ft", 3.28083989501312e+00),
                    ("yd", 1.09361329833771e+00),
                    ("ang", 1.0e+10),
                    ("ell", 8.74890638670166e-01),
                    ("ly", 1.05700083402462e-16),
                    ("parsec", 3.24077928966473e-17),
                    ("pc", 3.24077928966473e-17),
                    ("Pica", 2.83464566929134e+03),
                    ("Picapt", 2.83464566929134e+03),
                    ("pica", 2.36220472440945e+02),
                    ("survey_mi", 6.21369949494950e-04),
                ]),
            ),
            (
                11,
                HashMap::from([
                    ("yr", 3.16880878140289e-08),
                    ("day", 1.15740740740741e-05),
                    ("d", 1.15740740740741e-05),
                    ("hr", 2.77777777777778e-04),
                    ("mn", 1.66666666666667e-02),
                    ("min", 1.66666666666667e-02),
                    ("sec", 1.0),
                    ("s", 1.0),
                ]),
            ),
            (
                12,
                HashMap::from([
                    ("Pa", 1.0),
                    ("p", 1.0),
                    ("atm", 9.86923266716013e-06),
                    ("at", 9.86923266716013e-06),
                    ("mmHg", 7.50063755419211e-03),
                    ("psi", 1.45037737730209e-04),
                    ("Torr", 7.50061682704170e-03),
                ]),
            ),
            (
                13,
                HashMap::from([
                    ("N", 1.0),
                    ("dyn", 1.0e+5),
                    ("dy", 1.0e+5),
                    ("lbf", 2.24808923655339e-01),
                    ("pond", 1.01971621297793e+02),
                ]),
            ),
            (
                14,
                HashMap::from([
                    ("J", 1.0),
                    ("e", 9.99999519343231e+06),
                    ("c", 2.39006249473467e-01),
                    ("cal", 2.38846190642017e-01),
                    ("eV", 6.24145700000000e+18),
                    ("ev", 6.24145700000000e+18),
                    ("HPh", 3.72506430801000e-07),
                    ("hh", 3.72506430801000e-07),
                    ("Wh", 2.77777916238711e-04),
                    ("wh", 2.77777916238711e-04),
                    ("flb", 2.37304222192651e+01),
                    ("BTU", 9.47815067349015e-04),
                    ("btu", 9.47815067349015e-04),
                ]),
            ),
            (
                15,
                HashMap::from([
                    ("HP", 1.0),
                    ("h", 1.0),
                    ("W", 7.45699871582270e+02),
                    ("w", 7.45699871582270e+02),
                    ("PS", 1.01386966542400e+00),
                ]),
            ),
            (16, HashMap::from([("T", 1.0), ("ga", 10000.0)])),
            (
                18,
                HashMap::from([
                    ("l", 1.0),
                    ("L", 1.0),
                    ("lt", 1.0),
                    ("tsp", 2.02884136211058e+02),
                    ("tspm", 2.0e+02),
                    ("tbs", 6.76280454036860e+01),
                    ("oz", 3.38140227018430e+01),
                    ("cup", 4.22675283773038e+00),
                    ("pt", 2.11337641886519e+00),
                    ("us_pt", 2.11337641886519e+00),
                    ("uk_pt", 1.75975398639270e+00),
                    ("qt", 1.05668820943259e+00),
                    ("uk_qt", 8.79876993196351e-01),
                    ("gal", 2.64172052358148e-01),
                    ("uk_gal", 2.19969248299088e-01),
                    ("ang3", 1.0e+27),
                    ("ang^3", 1.0e+27),
                    ("barrel", 6.28981077043211e-03),
                    ("bushel", 2.83775932584017e-02),
                    ("in3", 6.10237440947323e+01),
                    ("in^3", 6.10237440947323e+01),
                    ("ft3", 3.53146667214886e-02),
                    ("ft^3", 3.53146667214886e-02),
                    ("ly3", 1.18093498844171e-51),
                    ("ly^3", 1.18093498844171e-51),
                    ("m3", 1.0e-03),
                    ("m^3", 1.0e-03),
                    ("mi3", 2.39912758578928e-13),
                    ("mi^3", 2.39912758578928e-13),
                    ("yd3", 1.30795061931439e-03),
                    ("yd^3", 1.30795061931439e-03),
                    ("Nmi3", 1.57426214685811e-13),
                    ("Nmi^3", 1.57426214685811e-13),
                    ("Pica3", 2.27769904358706e+07),
                    ("Pica^3", 2.27769904358706e+07),
                    ("Picapt3", 2.27769904358706e+07),
                    ("Picapt^3", 2.27769904358706e+07),
                    ("GRT", 3.53146667214886e-04),
                    ("regton", 3.53146667214886e-04),
                    ("MTON", 8.82866668037215e-04),
                ]),
            ),
            (
                19,
                HashMap::from([
                    ("ha", 1.0),
                    ("uk_acre", 2.47105381467165e+00),
                    ("us_acre", 2.47104393046628e+00),
                    ("ang2", 1.0e+24),
                    ("ang^2", 1.0e+24),
                    ("ar", 1.0e+02),
                    ("ft2", 1.07639104167097e+05),
                    ("ft^2", 1.07639104167097e+05),
                    ("in2", 1.55000310000620e+07),
                    ("in^2", 1.55000310000620e+07),
                    ("ly2", 1.11725076312873e-28),
                    ("ly^2", 1.11725076312873e-28),
                    ("m2", 1.0e+04),
                    ("m^2", 1.0e+04),
                    ("Morgen", 4.0e+00),
                    ("mi2", 3.86102158542446e-03),
                    ("mi^2", 3.86102158542446e-03),
                    ("Nmi2", 2.91553349598123e-03),
                    ("Nmi^2", 2.91553349598123e-03),
                    ("Pica2", 8.03521607043214e+10),
                    ("Pica^2", 8.03521607043214e+10),
                    ("Picapt2", 8.03521607043214e+10),
                    ("Picapt^2", 8.03521607043214e+10),
                    ("yd2", 1.19599004630108e+04),
                    ("yd^2", 1.19599004630108e+04),
                ]),
            ),
            (20, HashMap::from([("bit", 1.0), ("byte", 0.125)])),
            (
                21,
                HashMap::from([
                    ("m/s", 1.0),
                    ("m/sec", 1.0),
                    ("m/h", 3.60e+03),
                    ("m/hr", 3.60e+03),
                    ("mph", 2.23693629205440e+00),
                    ("admkn", 1.94260256941567e+00),
                    ("kn", 1.94384449244060e+00),
                ]),
            ),
        ])
    })
}

fn get_unit_details(uom: &str) -> Option<(&str, u8, f64)> {
    if uom.is_empty() {
        return None;
    }
    let units = conversion_units();
    if let Some(unit) = units.get(uom) {
        return Some((uom, unit.group, 1.0));
    }
    let multipliers = conversion_multipliers();
    // 1 character prefix
    if !uom.is_empty() {
        let (prefix, rest) = uom.split_at(1);
        if let Some(unit) = units.get(rest) {
            if let Some(multiplier) = multipliers.get(prefix) {
                if unit.allow_prefix {
                    return Some((rest, unit.group, *multiplier));
                }
            }
        }
    }
    // 2 character prefix
    if uom.len() >= 2 {
        let (prefix, rest) = uom.split_at(2);
        if let Some(unit) = units.get(rest) {
            if let Some(multiplier) = multipliers.get(prefix) {
                if unit.allow_prefix {
                    return Some((rest, unit.group, *multiplier));
                }
            }
        }
    }
    None
}

fn resolve_temperature_synonyms(uom: &str) -> &str {
    match uom {
        "fah" => "F",
        "cel" => "C",
        "kel" => "K",
        _ => uom,
    }
}

fn convert_temperature(from_uom: &str, to_uom: &str, mut value: f64) -> f64 {
    let from = resolve_temperature_synonyms(from_uom);
    let to = resolve_temperature_synonyms(to_uom);
    if from == to {
        return value;
    }
    match from {
        "F" => value = (value - 32.0) / 1.8 + 273.15,
        "C" => value += 273.15,
        "Rank" => value /= 1.8,
        "Reau" => value = value * 1.25 + 273.15,
        _ => {}
    }
    match to {
        "F" => value = (value - 273.15) * 1.8 + 32.0,
        "C" => value -= 273.15,
        "Rank" => value *= 1.8,
        "Reau" => value = (value - 273.15) * 0.8,
        _ => {}
    }
    value
}

fn convert(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 3 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let num = args[0].to_number();
    if num.typ != ArgType::Number {
        return num;
    }
    let from_uom = args[1].value();
    let from = match get_unit_details(&from_uom) {
        Some(v) => v,
        None => return new_error_formula_arg(FORMULA_ERROR_NA),
    };
    let to_uom = args[2].value();
    let to = match get_unit_details(&to_uom) {
        Some(v) => v,
        None => return new_error_formula_arg(FORMULA_ERROR_NA),
    };
    if from.1 != to.1 {
        return new_error_formula_arg(FORMULA_ERROR_NA);
    }
    let val = num.number * from.2;
    if from.0 == to.0 && from.2 == to.2 {
        return new_number_formula_arg(val / from.2);
    } else if from.0 == to.0 {
        return new_number_formula_arg(val / to.2);
    } else if from.1 == 17 {
        return new_number_formula_arg(convert_temperature(from.0, to.0, val));
    }
    let conversions = unit_conversions();
    let from_conversion = conversions
        .get(&from.1)
        .and_then(|m| m.get(from.0))
        .copied()
        .unwrap_or(1.0);
    let to_conversion = conversions
        .get(&to.1)
        .and_then(|m| m.get(to.0))
        .copied()
        .unwrap_or(1.0);
    let base_value = val * (1.0 / from_conversion);
    new_number_formula_arg((base_value * to_conversion) / to.2)
}

// ------------------------------------------------------------------
// Delta, error, and step functions
// ------------------------------------------------------------------

fn delta(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.is_empty() {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    if args.len() > 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let number1 = args[0].to_number();
    if number1.typ != ArgType::Number {
        return number1;
    }
    let mut number2 = new_number_formula_arg(0.0);
    if args.len() == 2 {
        number2 = args[1].to_number();
        if number2.typ != ArgType::Number {
            return number2;
        }
    }
    new_bool_formula_arg(number1.number == number2.number).to_number()
}

fn erf(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.is_empty() {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    if args.len() > 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let lower = args[0].to_number();
    if lower.typ != ArgType::Number {
        return lower;
    }
    if args.len() == 2 {
        let upper = args[1].to_number();
        if upper.typ != ArgType::Number {
            return upper;
        }
        return new_number_formula_arg(libm::erf(upper.number) - libm::erf(lower.number));
    }
    new_number_formula_arg(libm::erf(lower.number))
}

fn erfdotprecise(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let x = args[0].to_number();
    if x.typ != ArgType::Number {
        return x;
    }
    new_number_formula_arg(libm::erf(x.number))
}

fn erfc(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    erfc_impl(args, "ERFC")
}

fn erfcdotprecise(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    erfc_impl(args, "ERFC.PRECISE")
}

fn erfc_impl(args: &[FormulaArg], _name: &str) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let x = args[0].to_number();
    if x.typ != ArgType::Number {
        return x;
    }
    new_number_formula_arg(libm::erfc(x.number))
}

fn gestep(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.is_empty() {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    if args.len() > 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let number = args[0].to_number();
    if number.typ != ArgType::Number {
        return number;
    }
    let mut step = new_number_formula_arg(0.0);
    if args.len() == 2 {
        step = args[1].to_number();
        if step.typ != ArgType::Number {
            return step;
        }
    }
    new_bool_formula_arg(number.number >= step.number).to_number()
}
