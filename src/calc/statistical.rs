//! Statistical formula functions.

use std::collections::HashMap;

use crate::calc::arg::*;
use crate::calc::{CalcContext, FormulaFn};
use statrs::distribution::{
    Beta, ChiSquared, Continuous, ContinuousCDF, Discrete, DiscreteCDF, Exp, FisherSnedecor, Gamma,
    Hypergeometric, LogNormal, NegativeBinomial, Normal, StudentsT,
};
use statrs::function::gamma::{gamma as statrs_gamma, ln_gamma as statrs_ln_gamma};

pub fn register(m: &mut HashMap<&'static str, FormulaFn>) {
    m.insert("AVEDEV", avedev);
    m.insert("AVERAGEA", averagea);
    m.insert("AVERAGEIF", averageif);
    m.insert("AVERAGEIFS", averageifs);
    m.insert("BETAdotDIST", beta_dist);
    m.insert("BETADIST", betadist);
    m.insert("BETAINV", betainv);
    m.insert("BETAdotINV", beta_dot_inv);
    m.insert("BINOMdotDIST", binomdist);
    m.insert("BINOMDIST", binomdist);
    m.insert("BINOMdotDISTdotRANGE", binomdist_range);
    m.insert("BINOMdotINV", binominv);
    m.insert("CHIDIST", chi_dist_rt);
    m.insert("CHIINV", chi_inv_rt);
    m.insert("CHITEST", chi_sq_test);
    m.insert("CHISQdotDIST", chi_sq_dist);
    m.insert("CHISQdotDISTdotRT", chi_sq_dist_rt);
    m.insert("CHISQdotTEST", chi_sq_test);
    m.insert("CHISQdotINV", chi_sq_inv);
    m.insert("CHISQdotINVdotRT", chi_sq_inv_rt);
    m.insert("CONFIDENCE", confidence_norm);
    m.insert("CONFIDENCEdotNORM", confidence_norm);
    m.insert("CONFIDENCEdotT", confidence_t);
    m.insert("COVAR", covariance_p);
    m.insert("COVARIANCEdotP", covariance_p);
    m.insert("COVARIANCEdotS", covariance_s);
    m.insert("CORREL", correl);
    m.insert("COUNTA", counta);
    m.insert("COUNTBLANK", countblank);
    m.insert("COUNTIF", countif);
    m.insert("COUNTIFS", countifs);
    m.insert("CRITBINOM", critbinom);
    m.insert("DEVSQ", devsq);
    m.insert("FISHER", fisher);
    m.insert("FISHERINV", fisherinv);
    m.insert("FORECAST", forecast);
    m.insert("FORECASTdotLINEAR", forecast);
    m.insert("FREQUENCY", frequency);
    m.insert("GAMMA", gamma);
    m.insert("GAMMAdotDIST", gamma_dist);
    m.insert("GAMMADIST", gammadist);
    m.insert("GAMMAdotINV", gamma_inv);
    m.insert("GAMMAINV", gammainv);
    m.insert("GAMMALN", gammaln);
    m.insert("GAMMALNdotPRECISE", gammaln_precise);
    m.insert("GAUSS", gauss);
    m.insert("GEOMEAN", geomean);
    m.insert("GROWTH", growth);
    m.insert("HARMEAN", harmean);
    m.insert("HYPGEOMdotDIST", hypgeom_dist);
    m.insert("HYPGEOMDIST", hypgeomdist);
    m.insert("INTERCEPT", intercept);
    m.insert("KURT", kurt);
    m.insert("EXPONdotDIST", expon_dist);
    m.insert("EXPONDIST", expondist);
    m.insert("FdotDIST", f_dist);
    m.insert("FDIST", fdist);
    m.insert("FdotDISTdotRT", f_dist_rt);
    m.insert("FdotINV", f_inv);
    m.insert("FdotINVdotRT", f_inv_rt);
    m.insert("FINV", finv);
    m.insert("FdotTEST", ftest);
    m.insert("FTEST", ftest);
    m.insert("LOGINV", loginv);
    m.insert("LOGNORMdotINV", lognorm_inv);
    m.insert("LOGNORMdotDIST", lognorm_dist);
    m.insert("LOGNORMDIST", lognormdist);
    m.insert("MODE", mode_sngl);
    m.insert("MODEdotMULT", mode_sngl);
    m.insert("MODEdotSNGL", mode_sngl);
    m.insert("NEGBINOMdotDIST", negbinom_dist);
    m.insert("NEGBINOMDIST", negbinomdist);
    m.insert("NORMdotDIST", norm_dist);
    m.insert("NORMDIST", normdist);
    m.insert("NORMdotINV", norm_inv);
    m.insert("NORMINV", norminv);
    m.insert("NORMdotSdotDIST", normsdist);
    m.insert("NORMSDIST", normsdist);
    m.insert("NORMdotSdotINV", normsinv);
    m.insert("NORMSINV", normsinv);
    m.insert("LARGE", large);
    m.insert("MAXA", maxa);
    m.insert("MAXIFS", maxifs);
    m.insert("MEDIAN", median);
    m.insert("MINA", mina);
    m.insert("MINIFS", minifs);
    m.insert("PEARSON", pearson);
    m.insert("PERCENTILEdotEXC", percentile_exc);
    m.insert("PERCENTILEdotINC", percentile_inc);
    m.insert("PERCENTILE", percentile);
    m.insert("PERCENTRANKdotEXC", percentrank_exc);
    m.insert("PERCENTRANKdotINC", percentrank_inc);
    m.insert("PERCENTRANK", percentrank_inc);
    m.insert("PERMUT", permut);
    m.insert("PERMUTATIONA", permutationa);
    m.insert("PHI", phi);
    m.insert("QUARTILE", quartile);
    m.insert("QUARTILEdotEXC", quartile_exc);
    m.insert("QUARTILEdotINC", quartile_inc);
    m.insert("RANKdotEQ", rank_eq);
    m.insert("RANK", rank_eq);
    m.insert("RSQ", rsq);
    m.insert("SKEW", skew);
    m.insert("SKEWdotP", skew_p);
    m.insert("SLOPE", slope);
    m.insert("SMALL", small);
    m.insert("STANDARDIZE", standardize);
    m.insert("STDEVP", stdevp);
    m.insert("STDEVdotP", stdevp);
    m.insert("STDEVA", stdeva);
    m.insert("STDEVPA", stdevpa);
    m.insert("STEYX", steyx);
    m.insert("POISSONdotDIST", poisson_dist);
    m.insert("POISSON", poisson_dist);
    m.insert("PROB", prob);
    m.insert("SUMIF", sumif);
    m.insert("SUMIFS", sumifs);
    m.insert("SUMPRODUCT", sumproduct);
    m.insert("SUMX2MY2", sumx2my2);
    m.insert("SUMX2PY2", sumx2py2);
    m.insert("SUMXMY2", sumxmy2);
    m.insert("TdotDIST", t_dist);
    m.insert("TdotDISTdot2T", t_dist_2t);
    m.insert("TdotDISTdotRT", t_dist_rt);
    m.insert("TDIST", tdist);
    m.insert("TdotINV", t_inv);
    m.insert("TdotINVdot2T", t_inv_2t);
    m.insert("TINV", tinv);
    m.insert("TTEST", ttest);
    m.insert("TdotTEST", ttest);
    m.insert("TREND", trend);
    m.insert("TRIMMEAN", trimmean);
    m.insert("VAR", vars);
    m.insert("VARA", vara);
    m.insert("VARP", varp);
    m.insert("VARdotP", varp);
    m.insert("VARdotS", vars);
    m.insert("VARPA", varpa);
    m.insert("WEIBULL", weibull);
    m.insert("WEIBULLdotDIST", weibull);
    m.insert("ZdotTEST", z_test);
    m.insert("ZTEST", ztest);
}

// ------------------------------------------------------------------
// Helpers
// ------------------------------------------------------------------

fn flatten_args(args: &[FormulaArg]) -> Vec<FormulaArg> {
    let mut out = Vec::new();
    for a in args {
        out.extend(a.to_list());
    }
    out
}

fn numeric_values(args: &[FormulaArg]) -> Vec<f64> {
    flatten_args(args)
        .into_iter()
        .filter(|a| a.typ != ArgType::Empty)
        .filter_map(|a| a.to_number().as_number())
        .collect()
}

/// Pair numeric values from two equally-sized arrays.  Non-numeric pairs are
/// skipped, matching Excel's treatment of empty/text cells in regression
/// functions.
fn paired_numeric_values(x_arg: &FormulaArg, y_arg: &FormulaArg) -> Option<(Vec<f64>, Vec<f64>)> {
    let x_list = x_arg.to_list();
    let y_list = y_arg.to_list();
    if x_list.len() != y_list.len() {
        return None;
    }
    let mut xs = Vec::new();
    let mut ys = Vec::new();
    for (x, y) in x_list.into_iter().zip(y_list.into_iter()) {
        if x.typ == ArgType::Number && y.typ == ArgType::Number {
            xs.push(x.number);
            ys.push(y.number);
        }
    }
    Some((xs, ys))
}

fn count_sum(count_text: bool, args: &[FormulaArg]) -> (f64, f64) {
    let mut count = 0.0;
    let mut sum = 0.0;
    for a in &flatten_args(args) {
        match a.typ {
            ArgType::Number if !a.boolean => {
                count += 1.0;
                sum += a.number;
            }
            ArgType::Number if count_text && a.boolean => {
                count += 1.0;
                sum += a.number;
            }
            ArgType::String if count_text => {
                count += 1.0;
                if let Some(n) = a.to_number().as_number() {
                    sum += n;
                }
            }
            _ => {}
        }
    }
    (count, sum)
}

fn average_internal(count_text: bool, args: &[FormulaArg]) -> FormulaArg {
    let (count, sum) = count_sum(count_text, args);
    if count == 0.0 {
        return new_error_formula_arg(FORMULA_ERROR_DIV);
    }
    new_number_formula_arg(sum / count)
}

#[derive(Debug, Clone)]
enum Criteria {
    NumberOp(String, f64),
    Text(String),
}

fn parse_criteria(criteria: &FormulaArg) -> Criteria {
    let s = criteria.value();
    if s.is_empty() {
        return Criteria::NumberOp("=".to_string(), 0.0);
    }
    let s = s.trim();
    let (op, rest) = if s.starts_with(">=") {
        (">=", &s[2..])
    } else if s.starts_with("<=") {
        ("<=", &s[2..])
    } else if s.starts_with("<>") {
        ("<>", &s[2..])
    } else if s.starts_with('>') {
        (">", &s[1..])
    } else if s.starts_with('<') {
        ("<", &s[1..])
    } else if s.starts_with('=') {
        ("=", &s[1..])
    } else {
        ("=", s)
    };
    if let Ok(n) = rest.parse::<f64>() {
        return Criteria::NumberOp(op.to_string(), n);
    }
    Criteria::Text(s.to_string())
}

fn criteria_matches(value: &FormulaArg, criteria: &Criteria) -> bool {
    match criteria {
        Criteria::NumberOp(op, target) => {
            let n = match value.to_number().as_number() {
                Some(n) => n,
                None => return false,
            };
            match op.as_str() {
                "=" => (n - target).abs() < 1e-12,
                "<>" => (n - target).abs() >= 1e-12,
                ">" => n > *target,
                "<" => n < *target,
                ">=" => n >= *target,
                "<=" => n <= *target,
                _ => false,
            }
        }
        Criteria::Text(pattern) => {
            if value.is_error() {
                return false;
            }
            let text = value.value();
            criteria_text_matches(&text, pattern)
        }
    }
}

fn criteria_text_matches(text: &str, pattern: &str) -> bool {
    let (op, pat) = if pattern.starts_with(">=") {
        (">=", &pattern[2..])
    } else if pattern.starts_with("<=") {
        ("<=", &pattern[2..])
    } else if pattern.starts_with("<>") {
        ("<>", &pattern[2..])
    } else if pattern.starts_with('>') {
        (">", &pattern[1..])
    } else if pattern.starts_with('<') {
        ("<", &pattern[1..])
    } else {
        ("=", pattern)
    };
    match op {
        "=" => wildcard_match(text, pat),
        "<>" => !wildcard_match(text, pat),
        ">" => text.to_uppercase() > pat.to_uppercase(),
        "<" => text.to_uppercase() < pat.to_uppercase(),
        ">=" => text.to_uppercase() >= pat.to_uppercase(),
        "<=" => text.to_uppercase() <= pat.to_uppercase(),
        _ => false,
    }
}

fn wildcard_match(text: &str, pattern: &str) -> bool {
    let text = text.to_uppercase();
    let pattern = pattern.to_uppercase();
    let mut t = 0;
    let mut p = 0;
    let mut star = None;
    let mut match_index = 0;
    let tchars: Vec<char> = text.chars().collect();
    let pchars: Vec<char> = pattern.chars().collect();
    while t < tchars.len() {
        if p < pchars.len() && (pchars[p] == '?' || pchars[p] == tchars[t]) {
            t += 1;
            p += 1;
        } else if p < pchars.len() && pchars[p] == '*' {
            star = Some(p);
            p += 1;
            match_index = t;
        } else if let Some(star_pos) = star {
            p = star_pos + 1;
            match_index += 1;
            t = match_index;
        } else {
            return false;
        }
    }
    while p < pchars.len() && pchars[p] == '*' {
        p += 1;
    }
    p == pchars.len()
}

fn fact(n: f64) -> f64 {
    if n < 0.0 || n.fract() != 0.0 {
        return f64::NAN;
    }
    let mut result = 1.0;
    for i in 1..=n as u64 {
        result *= i as f64;
    }
    result
}

// ------------------------------------------------------------------
// Implemented functions
// ------------------------------------------------------------------

fn avedev(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.is_empty() {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let nums = numeric_values(args);
    if nums.is_empty() {
        return new_error_formula_arg(FORMULA_ERROR_DIV);
    }
    let avg = nums.iter().sum::<f64>() / nums.len() as f64;
    let sum_dev = nums.iter().map(|n| (n - avg).abs()).sum::<f64>();
    new_number_formula_arg(sum_dev / nums.len() as f64)
}

fn averagea(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    average_internal(true, args)
}

fn averageif(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() < 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let range = &args[0];
    let criteria = parse_criteria(&args[1]);
    let sum_range = args.get(2).unwrap_or(range);

    let mut sum = 0.0;
    let mut count = 0.0;
    let range_flat = range.to_list();
    let sum_flat = sum_range.to_list();

    for (i, item) in range_flat.iter().enumerate() {
        if criteria_matches(item, &criteria) {
            if let Some(v) = sum_flat.get(i).and_then(|a| a.to_number().as_number()) {
                sum += v;
                count += 1.0;
            }
        }
    }
    if count == 0.0 {
        return new_error_formula_arg(FORMULA_ERROR_DIV);
    }
    new_number_formula_arg(sum / count)
}

fn averageifs(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() < 3 || args.len() % 2 == 0 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let sum_range = args[0].to_list();
    let criteria_count = (args.len() - 1) / 2;
    let mut criteria: Vec<(Vec<FormulaArg>, Criteria)> = Vec::new();
    for i in 0..criteria_count {
        let range = args[i * 2 + 1].to_list();
        let crit = parse_criteria(&args[i * 2 + 2]);
        criteria.push((range, crit));
    }

    let mut sum = 0.0;
    let mut count = 0.0;
    for i in 0..sum_range.len() {
        let mut ok = true;
        for (range, crit) in &criteria {
            if let Some(item) = range.get(i) {
                if !criteria_matches(item, crit) {
                    ok = false;
                    break;
                }
            } else {
                ok = false;
                break;
            }
        }
        if ok {
            if let Some(n) = sum_range[i].to_number().as_number() {
                sum += n;
                count += 1.0;
            }
        }
    }
    if count == 0.0 {
        return new_error_formula_arg(FORMULA_ERROR_DIV);
    }
    new_number_formula_arg(sum / count)
}

fn counta(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    let mut count = 0.0;
    for a in &flatten_args(args) {
        if a.typ != ArgType::Empty {
            count += 1.0;
        }
    }
    new_number_formula_arg(count)
}

fn countblank(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let mut count = 0.0;
    for a in &args[0].to_list() {
        if a.typ == ArgType::Empty {
            count += 1.0;
        }
    }
    new_number_formula_arg(count)
}

fn countif(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let range = args[0].to_list();
    let criteria = parse_criteria(&args[1]);
    let mut count = 0.0;
    for item in &range {
        if criteria_matches(item, &criteria) {
            count += 1.0;
        }
    }
    new_number_formula_arg(count)
}

fn countifs(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() < 2 || args.len() % 2 != 0 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let criteria_count = args.len() / 2;
    let mut criteria: Vec<(Vec<FormulaArg>, Criteria)> = Vec::new();
    for i in 0..criteria_count {
        let range = args[i * 2].to_list();
        let crit = parse_criteria(&args[i * 2 + 1]);
        criteria.push((range, crit));
    }

    let len = criteria[0].0.len();
    let mut count = 0.0;
    for i in 0..len {
        let mut ok = true;
        for (range, crit) in &criteria {
            if let Some(item) = range.get(i) {
                if !criteria_matches(item, crit) {
                    ok = false;
                    break;
                }
            } else {
                ok = false;
                break;
            }
        }
        if ok {
            count += 1.0;
        }
    }
    new_number_formula_arg(count)
}

fn devsq(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    let nums = numeric_values(args);
    if nums.is_empty() {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    let avg = nums.iter().sum::<f64>() / nums.len() as f64;
    new_number_formula_arg(nums.iter().map(|n| (n - avg).powi(2)).sum())
}

fn fisher(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    match args[0].to_number().as_number() {
        Some(n) if n > -1.0 && n < 1.0 => {
            new_number_formula_arg(0.5 * ((1.0 + n) / (1.0 - n)).ln())
        }
        _ => new_error_formula_arg(FORMULA_ERROR_NUM),
    }
}

fn fisherinv(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    match args[0].to_number().as_number() {
        Some(n) => {
            let e2n = (2.0 * n).exp();
            new_number_formula_arg((e2n - 1.0) / (e2n + 1.0))
        }
        None => new_error_formula_arg(FORMULA_ERROR_VALUE),
    }
}

fn gamma(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    match args[0].to_number().as_number() {
        Some(n) if n > 0.0 => new_number_formula_arg(statrs_gamma(n)),
        _ => new_error_formula_arg(FORMULA_ERROR_NUM),
    }
}

fn gammaln(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    match args[0].to_number().as_number() {
        Some(n) if n > 0.0 => new_number_formula_arg(statrs_ln_gamma(n)),
        _ => new_error_formula_arg(FORMULA_ERROR_NUM),
    }
}

fn gammaln_precise(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    gammaln(_ctx, args)
}

fn gauss(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    match args[0].to_number().as_number() {
        Some(n) => new_number_formula_arg(normsdist_value(n) - 0.5),
        None => new_error_formula_arg(FORMULA_ERROR_VALUE),
    }
}

fn geomean(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    let nums = numeric_values(args);
    if nums.is_empty() || nums.iter().any(|&n| n <= 0.0) {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    let sum_log = nums.iter().map(|n| n.ln()).sum::<f64>();
    new_number_formula_arg((sum_log / nums.len() as f64).exp())
}

fn harmean(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    let nums = numeric_values(args);
    if nums.is_empty() || nums.iter().any(|&n| n <= 0.0) {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    let sum_inv = nums.iter().map(|n| 1.0 / n).sum::<f64>();
    new_number_formula_arg(nums.len() as f64 / sum_inv)
}

fn normsdist_value(x: f64) -> f64 {
    Normal::new(0.0, 1.0).unwrap().cdf(x)
}

fn normsdist(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    // NORM.S.DIST(z, [cumulative]) or NORMSDIST(z)
    if args.len() != 1 && args.len() != 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let z = match args[0].to_number().as_number() {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let cumulative = if args.len() == 2 {
        args[1].as_bool()
    } else {
        true
    };
    let dist = Normal::new(0.0, 1.0).unwrap();
    if cumulative {
        new_number_formula_arg(dist.cdf(z))
    } else {
        new_number_formula_arg(dist.pdf(z))
    }
}

fn normsinv(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    match args[0].to_number().as_number() {
        Some(n) if n > 0.0 && n < 1.0 => {
            new_number_formula_arg(Normal::new(0.0, 1.0).unwrap().inverse_cdf(n))
        }
        _ => new_error_formula_arg(FORMULA_ERROR_NUM),
    }
}

pub(crate) fn large(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let mut nums = numeric_values(&[args[0].clone()]);
    let k = match args[1].to_number().as_number() {
        Some(n) if n >= 1.0 => n as usize,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    nums.sort_by(|a, b| a.partial_cmp(b).unwrap());
    if k > nums.len() {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    new_number_formula_arg(nums[nums.len() - k])
}

pub(crate) fn small(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let mut nums = numeric_values(&[args[0].clone()]);
    let k = match args[1].to_number().as_number() {
        Some(n) if n >= 1.0 => n as usize,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    nums.sort_by(|a, b| a.partial_cmp(b).unwrap());
    if k > nums.len() {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    new_number_formula_arg(nums[k - 1])
}

fn maxa(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    let mut max = f64::NEG_INFINITY;
    let mut has_value = false;
    for a in &flatten_args(args) {
        let n = match a.typ {
            ArgType::Number if !a.boolean => a.number,
            ArgType::String => a.to_number().as_number().unwrap_or(0.0),
            ArgType::Number if a.boolean => {
                if a.as_bool() {
                    1.0
                } else {
                    0.0
                }
            }
            _ => continue,
        };
        has_value = true;
        if n > max {
            max = n;
        }
    }
    if has_value {
        new_number_formula_arg(max)
    } else {
        new_number_formula_arg(0.0)
    }
}

fn mina(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    let mut min = f64::INFINITY;
    let mut has_value = false;
    for a in &flatten_args(args) {
        let n = match a.typ {
            ArgType::Number if !a.boolean => a.number,
            ArgType::String => a.to_number().as_number().unwrap_or(0.0),
            ArgType::Number if a.boolean => {
                if a.as_bool() {
                    1.0
                } else {
                    0.0
                }
            }
            _ => continue,
        };
        has_value = true;
        if n < min {
            min = n;
        }
    }
    if has_value {
        new_number_formula_arg(min)
    } else {
        new_number_formula_arg(0.0)
    }
}

pub(crate) fn median(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    let mut nums = numeric_values(args);
    if nums.is_empty() {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    nums.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let mid = nums.len() / 2;
    if nums.len() % 2 == 1 {
        new_number_formula_arg(nums[mid])
    } else {
        new_number_formula_arg((nums[mid - 1] + nums[mid]) / 2.0)
    }
}

fn percentile(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    percentile_inc(_ctx, args)
}

pub(crate) fn percentile_inc(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let mut nums = numeric_values(&[args[0].clone()]);
    let k = match args[1].to_number().as_number() {
        Some(n) if n >= 0.0 && n <= 1.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    if nums.is_empty() {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    nums.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = nums.len() as f64;
    let rank = k * (n - 1.0);
    let lower = rank.floor() as usize;
    let upper = rank.ceil() as usize;
    let frac = rank - lower as f64;
    if upper >= nums.len() {
        return new_number_formula_arg(nums[lower]);
    }
    new_number_formula_arg(nums[lower] * (1.0 - frac) + nums[upper] * frac)
}

pub(crate) fn percentile_exc(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let mut nums = numeric_values(&[args[0].clone()]);
    let k = match args[1].to_number().as_number() {
        Some(n) if n > 0.0 && n < 1.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    if nums.len() < 2 {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    nums.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = nums.len() as f64;
    let rank = k * (n + 1.0);
    if rank < 1.0 || rank > n {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    let lower = rank.floor() as usize - 1;
    let upper = rank.ceil() as usize - 1;
    let frac = rank - rank.floor();
    new_number_formula_arg(nums[lower] * (1.0 - frac) + nums[upper] * frac)
}

fn quartile(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    quartile_inc(_ctx, args)
}

pub(crate) fn quartile_inc(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let q = match args[1].to_number().as_number() {
        Some(n) if n >= 0.0 && n <= 4.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    percentile_inc(_ctx, &[args[0].clone(), new_number_formula_arg(q / 4.0)])
}

pub(crate) fn quartile_exc(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let q = match args[1].to_number().as_number() {
        Some(n) if n >= 1.0 && n <= 3.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    percentile_exc(_ctx, &[args[0].clone(), new_number_formula_arg(q / 4.0)])
}

fn permut(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let n = match args[0].to_number().as_number() {
        Some(n) if n >= 0.0 && n.fract() == 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let k = match args[1].to_number().as_number() {
        Some(k) if k >= 0.0 && k.fract() == 0.0 => k,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    if k > n {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    new_number_formula_arg(fact(n) / fact(n - k))
}

fn permutationa(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let n = match args[0].to_number().as_number() {
        Some(n) if n >= 0.0 && n.fract() == 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let k = match args[1].to_number().as_number() {
        Some(k) if k >= 0.0 && k.fract() == 0.0 => k,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    new_number_formula_arg(n.powf(k))
}

fn phi(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    match args[0].to_number().as_number() {
        Some(n) => {
            let v = (-0.5 * n * n).exp() / (2.0 * std::f64::consts::PI).sqrt();
            new_number_formula_arg(v)
        }
        None => new_error_formula_arg(FORMULA_ERROR_VALUE),
    }
}

fn correl(ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    pearson(ctx, args)
}

fn covariance_common(args: &[FormulaArg]) -> Option<(Vec<f64>, Vec<f64>)> {
    if args.len() != 2 {
        return None;
    }
    paired_numeric_values(&args[0], &args[1]).filter(|(x, _)| !x.is_empty())
}

fn covariance_p(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    let (x, y) = match covariance_common(args) {
        Some(v) => v,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let n = x.len() as f64;
    let mean_x = x.iter().sum::<f64>() / n;
    let mean_y = y.iter().sum::<f64>() / n;
    let sum = x
        .iter()
        .zip(y.iter())
        .map(|(a, b)| (a - mean_x) * (b - mean_y))
        .sum::<f64>();
    new_number_formula_arg(sum / n)
}

fn covariance_s(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    let (x, y) = match covariance_common(args) {
        Some(v) => v,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let n = x.len() as f64;
    if n < 2.0 {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    let mean_x = x.iter().sum::<f64>() / n;
    let mean_y = y.iter().sum::<f64>() / n;
    let sum = x
        .iter()
        .zip(y.iter())
        .map(|(a, b)| (a - mean_x) * (b - mean_y))
        .sum::<f64>();
    new_number_formula_arg(sum / (n - 1.0))
}

fn pearson(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let (xs, ys) = match paired_numeric_values(&args[0], &args[1]) {
        Some(v) if !v.0.is_empty() => v,
        _ => return new_error_formula_arg(FORMULA_ERROR_NA),
    };
    let n = xs.len() as f64;
    let mean_x = xs.iter().sum::<f64>() / n;
    let mean_y = ys.iter().sum::<f64>() / n;
    let mut num = 0.0;
    let mut den_x = 0.0;
    let mut den_y = 0.0;
    for i in 0..xs.len() {
        let dx = xs[i] - mean_x;
        let dy = ys[i] - mean_y;
        num += dx * dy;
        den_x += dx * dx;
        den_y += dy * dy;
    }
    let den = den_x * den_y;
    if den == 0.0 {
        return new_error_formula_arg(FORMULA_ERROR_DIV);
    }
    new_number_formula_arg(num / den.sqrt())
}

fn rsq(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    let r = pearson(_ctx, args);
    match r.as_number() {
        Some(n) => new_number_formula_arg(n * n),
        None => r,
    }
}

fn slope(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let (xs, ys) = match paired_numeric_values(&args[0], &args[1]) {
        Some(v) if v.0.len() >= 2 => v,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let n = xs.len() as f64;
    let sum_x = xs.iter().sum::<f64>();
    let sum_y = ys.iter().sum::<f64>();
    let mut num = 0.0;
    let mut den = 0.0;
    for i in 0..xs.len() {
        num += (xs[i] - sum_x / n) * (ys[i] - sum_y / n);
        den += (xs[i] - sum_x / n).powi(2);
    }
    if den == 0.0 {
        return new_error_formula_arg(FORMULA_ERROR_DIV);
    }
    new_number_formula_arg(num / den)
}

fn standardize(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 3 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let x = match args[0].to_number().as_number() {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let mean = match args[1].to_number().as_number() {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let sd = match args[2].to_number().as_number() {
        Some(n) if n > 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    new_number_formula_arg((x - mean) / sd)
}

fn variance_internal(args: &[FormulaArg], sample: bool, count_text: bool) -> FormulaArg {
    let mut summer_a = 0.0;
    let mut summer_b = 0.0;
    let mut count = 0.0;
    let minimum = if sample { 1.0 } else { 0.0 };
    for a in args {
        for token in a.to_list() {
            if token.value().is_empty() {
                continue;
            }
            let num = token.to_number();
            let value = token.value();
            // Numeric values (including numbers parsed from text) are counted at
            // face value.  Logical values are handled separately because their
            // textual representation is "TRUE"/"FALSE".
            if value != "TRUE" && value != "FALSE" && num.typ == ArgType::Number {
                summer_a += num.number * num.number;
                summer_b += num.number;
                count += 1.0;
                continue;
            }
            // Logical values (TRUE/FALSE) and text representations of them.
            if token.typ == ArgType::Number
                || (token.typ == ArgType::String && (value == "TRUE" || value == "FALSE"))
            {
                let v = if token.as_bool() { 1.0 } else { 0.0 };
                summer_a += v * v;
                summer_b += v;
                count += 1.0;
                continue;
            }
            // Non-logical text is treated as 0 for the A-versions only.
            if count_text {
                count += 1.0;
            }
        }
    }
    if count > minimum {
        summer_a *= count;
        let summer_b_sq = summer_b * summer_b;
        new_number_formula_arg((summer_a - summer_b_sq) / (count * (count - minimum)))
    } else {
        new_error_formula_arg(FORMULA_ERROR_DIV)
    }
}

fn stdeva(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    let var = variance_internal(args, true, true);
    match var.as_number() {
        Some(n) if n >= 0.0 => new_number_formula_arg(n.sqrt()),
        _ => var,
    }
}

fn stdevpa(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    let var = variance_internal(args, false, true);
    match var.as_number() {
        Some(n) if n >= 0.0 => new_number_formula_arg(n.sqrt()),
        _ => var,
    }
}

fn stdevp(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    let var = variance_internal(args, false, false);
    match var.as_number() {
        Some(n) if n >= 0.0 => new_number_formula_arg(n.sqrt()),
        _ => var,
    }
}

fn varp(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    variance_internal(args, false, false)
}

fn vars(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    variance_internal(args, true, false)
}

fn vara(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    variance_internal(args, true, true)
}

fn varpa(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    variance_internal(args, false, true)
}

fn binomdist(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 4 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let s = match args[0].to_number().as_number() {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let trials = match args[1].to_number().as_number() {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let p = match args[2].to_number().as_number() {
        Some(n) if n >= 0.0 && n <= 1.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let cumulative = args[3].as_bool();
    if s < 0.0 || s > trials {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    if cumulative {
        let mut sum = 0.0;
        for i in 0..=s as i32 {
            sum += binom_coeff(trials, i as f64) * p.powi(i) * (1.0 - p).powf(trials - i as f64);
        }
        new_number_formula_arg(sum)
    } else {
        new_number_formula_arg(binom_coeff(trials, s) * p.powf(s) * (1.0 - p).powf(trials - s))
    }
}

fn binom_coeff(n: f64, k: f64) -> f64 {
    if k < 0.0 || k > n || k.fract() != 0.0 || n.fract() != 0.0 {
        return 0.0;
    }
    let k = k.min(n - k);
    let mut res = 1.0;
    for i in 0..k as i32 {
        res = res * (n - i as f64) / (i as f64 + 1.0);
    }
    res
}

fn binomdist_range(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() < 3 || args.len() > 4 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let trials = match args[0].to_number().as_number() {
        Some(n) if n >= 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let p = match args[1].to_number().as_number() {
        Some(n) if n >= 0.0 && n <= 1.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let num1 = match args[2].to_number().as_number() {
        Some(n) if n >= 0.0 && n <= trials => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let num2 = if args.len() == 4 {
        match args[3].to_number().as_number() {
            Some(n) if n >= 0.0 && n <= trials => n,
            _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
        }
    } else {
        num1
    };
    let mut sum = 0.0;
    let start = num1 as i32;
    let end = num2 as i32;
    for i in start..=end {
        sum += binom_coeff(trials, i as f64) * p.powi(i) * (1.0 - p).powf(trials - i as f64);
    }
    new_number_formula_arg(sum)
}

fn binominv(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 3 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let trials = match args[0].to_number().as_number() {
        Some(n) if n >= 0.0 => n.floor(),
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let p = match args[1].to_number().as_number() {
        Some(n) if n > 0.0 && n < 1.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let alpha = match args[2].to_number().as_number() {
        Some(n) if n > 0.0 && n < 1.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let q = 1.0 - p;
    if q > p {
        let mut log_factor = trials * q.ln();
        let mut sum = log_factor.exp();
        let mut i = 0.0;
        while i < trials && sum < alpha {
            log_factor += (trials - i).ln() - (i + 1.0).ln() + p.ln() - q.ln();
            sum += log_factor.exp();
            i += 1.0;
        }
        new_number_formula_arg(i)
    } else {
        let mut log_factor = trials * p.ln();
        let factor = log_factor.exp();
        let mut sum = 1.0 - factor;
        let mut i = 0.0;
        while i < trials && sum >= alpha {
            log_factor += (trials - i).ln() - (i + 1.0).ln() + q.ln() - p.ln();
            sum -= log_factor.exp();
            i += 1.0;
        }
        new_number_formula_arg(trials - i)
    }
}

fn poisson_dist(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 3 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let x = match args[0].to_number().as_number() {
        Some(n) if n >= 0.0 && n.fract() == 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let mean = match args[1].to_number().as_number() {
        Some(n) if n >= 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let cumulative = args[2].as_bool();
    if cumulative {
        let mut sum = 0.0;
        for i in 0..=x as i32 {
            sum += mean.powi(i) * (-mean).exp() / fact(i as f64);
        }
        new_number_formula_arg(sum)
    } else {
        new_number_formula_arg(mean.powf(x) * (-mean).exp() / fact(x))
    }
}

fn sumif(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() < 2 || args.len() > 3 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let range = args[0].to_list();
    let criteria = parse_criteria(&args[1]);
    let sum_range = args.get(2).unwrap_or(&args[0]).to_list();
    let mut sum = 0.0;
    for (i, item) in range.iter().enumerate() {
        if criteria_matches(item, &criteria) {
            if let Some(n) = sum_range.get(i).and_then(|a| a.to_number().as_number()) {
                sum += n;
            }
        }
    }
    new_number_formula_arg(sum)
}

fn sumifs(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() < 3 || args.len() % 2 == 0 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let sum_range = args[0].to_list();
    let criteria_count = (args.len() - 1) / 2;
    let mut criteria: Vec<(Vec<FormulaArg>, Criteria)> = Vec::new();
    for i in 0..criteria_count {
        let range = args[i * 2 + 1].to_list();
        let crit = parse_criteria(&args[i * 2 + 2]);
        criteria.push((range, crit));
    }
    let mut sum = 0.0;
    for i in 0..sum_range.len() {
        let mut ok = true;
        for (range, crit) in &criteria {
            if let Some(item) = range.get(i) {
                if !criteria_matches(item, crit) {
                    ok = false;
                    break;
                }
            } else {
                ok = false;
                break;
            }
        }
        if ok {
            if let Some(n) = sum_range[i].to_number().as_number() {
                sum += n;
            }
        }
    }
    new_number_formula_arg(sum)
}

fn sumproduct(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.is_empty() {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let first = args[0].to_list();
    let len = first.len();
    let mut sum = 0.0;
    for i in 0..len {
        let mut product = 1.0;
        for arg in args {
            if let Some(n) = arg.to_list().get(i).and_then(|a| a.to_number().as_number()) {
                product *= n;
            } else {
                product = 0.0;
                break;
            }
        }
        sum += product;
    }
    new_number_formula_arg(sum)
}

fn sumx2my2(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let (x, y) = match paired_numeric_values(&args[0], &args[1]) {
        Some(v) => v,
        None => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let sum = x
        .iter()
        .zip(y.iter())
        .map(|(a, b)| a * a - b * b)
        .sum::<f64>();
    new_number_formula_arg(sum)
}

fn sumx2py2(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let (x, y) = match paired_numeric_values(&args[0], &args[1]) {
        Some(v) => v,
        None => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let sum = x
        .iter()
        .zip(y.iter())
        .map(|(a, b)| a * a + b * b)
        .sum::<f64>();
    new_number_formula_arg(sum)
}

fn sumxmy2(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let (x, y) = match paired_numeric_values(&args[0], &args[1]) {
        Some(v) => v,
        None => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let sum = x
        .iter()
        .zip(y.iter())
        .map(|(a, b)| (a - b).powi(2))
        .sum::<f64>();
    new_number_formula_arg(sum)
}

fn trimmean(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let mut nums = numeric_values(&[args[0].clone()]);
    let percent = match args[1].to_number().as_number() {
        Some(n) if n >= 0.0 && n < 1.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    if nums.is_empty() {
        return new_error_formula_arg(FORMULA_ERROR_DIV);
    }
    nums.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let exclude = (nums.len() as f64 * percent / 2.0).floor() as usize;
    let trimmed = &nums[exclude..nums.len() - exclude];
    if trimmed.is_empty() {
        return new_error_formula_arg(FORMULA_ERROR_DIV);
    }
    let sum: f64 = trimmed.iter().sum();
    new_number_formula_arg(sum / trimmed.len() as f64)
}

fn weibull(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 4 && args.len() != 3 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let x = match args[0].to_number().as_number() {
        Some(n) if n >= 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let alpha = match args[1].to_number().as_number() {
        Some(n) if n > 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let beta = match args[2].to_number().as_number() {
        Some(n) if n > 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let cumulative = if args.len() == 4 {
        args[3].as_bool()
    } else {
        true
    };
    if cumulative {
        new_number_formula_arg(1.0 - (-(x / beta).powf(alpha)).exp())
    } else {
        new_number_formula_arg(
            (alpha / beta) * (x / beta).powf(alpha - 1.0) * (-(x / beta).powf(alpha)).exp(),
        )
    }
}

fn critbinom(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 3 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    binominv(_ctx, args)
}

fn forecast_common(args: &[FormulaArg]) -> Option<(f64, Vec<f64>, Vec<f64>)> {
    let (x, y_arg, x_arg) = if args.len() == 3 {
        (args[0].to_number().as_number()?, &args[1], &args[2])
    } else if args.len() == 2 {
        // INTERCEPT(known_y's, known_x's)
        (0.0, &args[0], &args[1])
    } else {
        return None;
    };
    let (ys, xs) = paired_numeric_values(y_arg, x_arg)?;
    if xs.len() < 2 {
        return None;
    }
    Some((x, xs, ys))
}

fn forecast(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    let (x, xs, ys) = match forecast_common(args) {
        Some(v) => v,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let n = xs.len() as f64;
    let mean_x = xs.iter().sum::<f64>() / n;
    let mean_y = ys.iter().sum::<f64>() / n;
    let mut num = 0.0;
    let mut den = 0.0;
    for i in 0..xs.len() {
        num += (xs[i] - mean_x) * (ys[i] - mean_y);
        den += (xs[i] - mean_x).powi(2);
    }
    if den == 0.0 {
        return new_error_formula_arg(FORMULA_ERROR_DIV);
    }
    let slope = num / den;
    let intercept = mean_y - slope * mean_x;
    new_number_formula_arg(intercept + slope * x)
}

fn intercept(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    let (_, xs, ys) = match forecast_common(args) {
        Some(v) => v,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let n = xs.len() as f64;
    let mean_x = xs.iter().sum::<f64>() / n;
    let mean_y = ys.iter().sum::<f64>() / n;
    let mut num = 0.0;
    let mut den = 0.0;
    for i in 0..xs.len() {
        num += (xs[i] - mean_x) * (ys[i] - mean_y);
        den += (xs[i] - mean_x).powi(2);
    }
    if den == 0.0 {
        return new_error_formula_arg(FORMULA_ERROR_DIV);
    }
    let slope = num / den;
    new_number_formula_arg(mean_y - slope * mean_x)
}

fn skew_common(args: &[FormulaArg], population: bool) -> FormulaArg {
    let values = numeric_values(args);
    let n = values.len() as f64;
    if n < 3.0 || (!population && n < 3.0) {
        return new_error_formula_arg(FORMULA_ERROR_DIV);
    }
    let mean = values.iter().sum::<f64>() / n;
    let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>()
        / if population { n } else { n - 1.0 };
    if variance == 0.0 {
        return new_error_formula_arg(FORMULA_ERROR_DIV);
    }
    let std = variance.sqrt();
    let sum_cubed = values
        .iter()
        .map(|v| ((v - mean) / std).powi(3))
        .sum::<f64>();
    if population {
        new_number_formula_arg(sum_cubed / n)
    } else {
        new_number_formula_arg((n / ((n - 1.0) * (n - 2.0))) * sum_cubed)
    }
}

fn skew(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    skew_common(args, false)
}

fn skew_p(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    skew_common(args, true)
}

fn kurt(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    let values = numeric_values(args);
    let n = values.len() as f64;
    if n < 4.0 {
        return new_error_formula_arg(FORMULA_ERROR_DIV);
    }
    let mean = values.iter().sum::<f64>() / n;
    let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / (n - 1.0);
    if variance == 0.0 {
        return new_error_formula_arg(FORMULA_ERROR_DIV);
    }
    let std = variance.sqrt();
    let sum_fourth = values
        .iter()
        .map(|v| ((v - mean) / std).powi(4))
        .sum::<f64>();
    let term1 = n * (n + 1.0) / ((n - 1.0) * (n - 2.0) * (n - 3.0)) * sum_fourth;
    let term2 = 3.0 * (n - 1.0).powi(2) / ((n - 2.0) * (n - 3.0));
    new_number_formula_arg(term1 - term2)
}

fn ftest(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let a: Vec<f64> = args[0]
        .to_list()
        .iter()
        .filter_map(|x| x.to_number().as_number())
        .collect();
    let b: Vec<f64> = args[1]
        .to_list()
        .iter()
        .filter_map(|x| x.to_number().as_number())
        .collect();
    if a.len() < 2 || b.len() < 2 {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    let mean_a = a.iter().sum::<f64>() / a.len() as f64;
    let mean_b = b.iter().sum::<f64>() / b.len() as f64;
    let var_a = a.iter().map(|x| (x - mean_a).powi(2)).sum::<f64>() / (a.len() - 1) as f64;
    let var_b = b.iter().map(|x| (x - mean_b).powi(2)).sum::<f64>() / (b.len() - 1) as f64;
    if var_b == 0.0 {
        return new_error_formula_arg(FORMULA_ERROR_DIV);
    }
    new_number_formula_arg(var_a / var_b)
}

pub(crate) fn mode_sngl(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    let values = numeric_values(args);
    if values.is_empty() {
        return new_error_formula_arg(FORMULA_ERROR_NA);
    }
    use std::collections::HashMap;
    let mut counts: HashMap<i64, usize> = HashMap::new();
    for v in &values {
        let key = (v * 1e12).round() as i64;
        *counts.entry(key).or_insert(0) += 1;
    }
    let max_count = counts.values().copied().max().unwrap_or(0);
    if max_count <= 1 {
        return new_error_formula_arg(FORMULA_ERROR_NA);
    }
    let mut mode = f64::NAN;
    for (key, count) in counts {
        if count == max_count {
            let candidate = key as f64 / 1e12;
            if mode.is_nan() || candidate < mode {
                mode = candidate;
            }
        }
    }
    new_number_formula_arg(mode)
}

fn rank_eq(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() < 2 || args.len() > 3 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let number = match args[0].to_number().as_number() {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let mut refs = numeric_values(&[args[1].clone()]);
    let order = args.get(2).map(|a| a.as_bool()).unwrap_or(false);
    refs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    if order {
        for (i, &v) in refs.iter().enumerate() {
            if (v - number).abs() < 1e-12 {
                return new_number_formula_arg((i + 1) as f64);
            }
        }
    } else {
        for (i, &v) in refs.iter().enumerate().rev() {
            if (v - number).abs() < 1e-12 {
                return new_number_formula_arg((refs.len() - i) as f64);
            }
        }
    }
    new_error_formula_arg(FORMULA_ERROR_NA)
}

fn percentrank_inc(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() < 2 || args.len() > 3 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let x = match args[1].to_number().as_number() {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let mut arr = numeric_values(&[args[0].clone()]);
    if arr.len() < 2 {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    let significance = match args.get(2) {
        Some(a) => match a.to_number().as_number() {
            Some(n) if n >= 1.0 => n as usize,
            _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
        },
        None => 3,
    };
    arr.sort_by(|a, b| a.partial_cmp(b).unwrap());
    if x < arr[0] || x > arr[arr.len() - 1] {
        return new_error_formula_arg(FORMULA_ERROR_NA);
    }
    let n = arr.len() - 1;
    for i in 0..n {
        if (arr[i] - x).abs() < 1e-12 {
            let p = i as f64 / n as f64;
            return new_number_formula_arg(format_sig(p, significance));
        }
        if arr[i] < x && x < arr[i + 1] {
            let p = (i as f64 + (x - arr[i]) / (arr[i + 1] - arr[i])) / n as f64;
            return new_number_formula_arg(format_sig(p, significance));
        }
    }
    if (arr[n] - x).abs() < 1e-12 {
        return new_number_formula_arg(1.0);
    }
    new_error_formula_arg(FORMULA_ERROR_NA)
}

fn percentrank_exc(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() < 2 || args.len() > 3 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let x = match args[1].to_number().as_number() {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let mut arr = numeric_values(&[args[0].clone()]);
    if arr.len() < 2 {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    let significance = match args.get(2) {
        Some(a) => match a.to_number().as_number() {
            Some(n) if n >= 1.0 => n as usize,
            _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
        },
        None => 3,
    };
    arr.sort_by(|a, b| a.partial_cmp(b).unwrap());
    if x < arr[0] || x > arr[arr.len() - 1] {
        return new_error_formula_arg(FORMULA_ERROR_NA);
    }
    let n = arr.len() as f64;
    for i in 0..arr.len() {
        if (arr[i] - x).abs() < 1e-12 {
            let p = (i as f64 + 1.0) / (n + 1.0);
            return new_number_formula_arg(format_sig(p, significance));
        }
        if i + 1 < arr.len() && arr[i] < x && x < arr[i + 1] {
            let p = (i as f64 + 1.0 + (x - arr[i]) / (arr[i + 1] - arr[i])) / (n + 1.0);
            return new_number_formula_arg(format_sig(p, significance));
        }
    }
    new_error_formula_arg(FORMULA_ERROR_NA)
}

fn format_sig(value: f64, significance: usize) -> f64 {
    let mult = 10f64.powi(significance as i32);
    (value * mult).floor() / mult
}

fn ttest(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 4 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let a: Vec<f64> = args[0]
        .to_list()
        .iter()
        .filter_map(|x| x.to_number().as_number())
        .collect();
    let b: Vec<f64> = args[1]
        .to_list()
        .iter()
        .filter_map(|x| x.to_number().as_number())
        .collect();
    let tails = match args[2].to_number().as_number() {
        Some(n) if n == 1.0 || n == 2.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let typ = match args[3].to_number().as_number() {
        Some(n) if n == 1.0 || n == 2.0 || n == 3.0 => n as i32,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };

    if a.len() < 2 || b.len() < 2 {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }

    let (t_stat, _df) = match typ {
        1 => {
            if a.len() != b.len() {
                return new_error_formula_arg(FORMULA_ERROR_NUM);
            }
            let diffs: Vec<f64> = a.iter().zip(b.iter()).map(|(x, y)| x - y).collect();
            let mean_d = diffs.iter().sum::<f64>() / diffs.len() as f64;
            let var_d =
                diffs.iter().map(|d| (d - mean_d).powi(2)).sum::<f64>() / (diffs.len() - 1) as f64;
            let se = (var_d / diffs.len() as f64).sqrt();
            if se == 0.0 {
                return new_error_formula_arg(FORMULA_ERROR_DIV);
            }
            (mean_d / se, (diffs.len() - 1) as f64)
        }
        2 => {
            let n1 = a.len() as f64;
            let n2 = b.len() as f64;
            let mean_a = a.iter().sum::<f64>() / n1;
            let mean_b = b.iter().sum::<f64>() / n2;
            let var_a = a.iter().map(|x| (x - mean_a).powi(2)).sum::<f64>() / (n1 - 1.0);
            let var_b = b.iter().map(|x| (x - mean_b).powi(2)).sum::<f64>() / (n2 - 1.0);
            let pooled_var = ((n1 - 1.0) * var_a + (n2 - 1.0) * var_b) / (n1 + n2 - 2.0);
            let se = (pooled_var * (1.0 / n1 + 1.0 / n2)).sqrt();
            if se == 0.0 {
                return new_error_formula_arg(FORMULA_ERROR_DIV);
            }
            ((mean_a - mean_b) / se, n1 + n2 - 2.0)
        }
        3 => {
            let n1 = a.len() as f64;
            let n2 = b.len() as f64;
            let mean_a = a.iter().sum::<f64>() / n1;
            let mean_b = b.iter().sum::<f64>() / n2;
            let var_a = a.iter().map(|x| (x - mean_a).powi(2)).sum::<f64>() / (n1 - 1.0);
            let var_b = b.iter().map(|x| (x - mean_b).powi(2)).sum::<f64>() / (n2 - 1.0);
            let se = (var_a / n1 + var_b / n2).sqrt();
            if se == 0.0 {
                return new_error_formula_arg(FORMULA_ERROR_DIV);
            }
            let numerator = var_a / n1 + var_b / n2;
            let df = numerator.powi(2)
                / ((var_a / n1).powi(2) / (n1 - 1.0) + (var_b / n2).powi(2) / (n2 - 1.0));
            ((mean_a - mean_b) / se, df)
        }
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };

    // Two-tailed p-value approximation using normal distribution for large df.
    let p = 2.0 * (1.0 - normsdist_value(t_stat.abs()));
    new_number_formula_arg(if tails == 1.0 { p / 2.0 } else { p })
}

// ------------------------------------------------------------------
// Distribution and advanced statistical functions
// ------------------------------------------------------------------

fn beta_dist(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    // BETA.DIST(x, alpha, beta, cumulative, [A], [B])
    if args.len() < 4 || args.len() > 6 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let x = match args[0].to_number().as_number() {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let alpha = match args[1].to_number().as_number() {
        Some(n) if n > 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let beta = match args[2].to_number().as_number() {
        Some(n) if n > 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let cumulative = args[3].as_bool();
    let a = args
        .get(4)
        .and_then(|a| a.to_number().as_number())
        .unwrap_or(0.0);
    let b = args
        .get(5)
        .and_then(|a| a.to_number().as_number())
        .unwrap_or(1.0);
    if a >= b {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    if x < a || x > b {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    let t = (x - a) / (b - a);
    let dist = Beta::new(alpha, beta).unwrap();
    if cumulative {
        new_number_formula_arg(dist.cdf(t))
    } else {
        new_number_formula_arg(dist.pdf(t) / (b - a))
    }
}

fn betadist(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    // BETADIST(x, alpha, beta, [A], [B]) - cumulative always true
    if args.len() < 3 || args.len() > 5 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let x = match args[0].to_number().as_number() {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let alpha = match args[1].to_number().as_number() {
        Some(n) if n > 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let beta = match args[2].to_number().as_number() {
        Some(n) if n > 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let a = args
        .get(3)
        .and_then(|a| a.to_number().as_number())
        .unwrap_or(0.0);
    let b = args
        .get(4)
        .and_then(|a| a.to_number().as_number())
        .unwrap_or(1.0);
    if a >= b {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    if x < a || x > b {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    let t = (x - a) / (b - a);
    new_number_formula_arg(Beta::new(alpha, beta).unwrap().cdf(t))
}

fn beta_dot_inv(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    // BETA.INV(p, alpha, beta, [A], [B])
    if args.len() < 3 || args.len() > 5 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let p = match args[0].to_number().as_number() {
        Some(n) if n >= 0.0 && n <= 1.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let alpha = match args[1].to_number().as_number() {
        Some(n) if n > 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let beta = match args[2].to_number().as_number() {
        Some(n) if n > 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let a = args
        .get(3)
        .and_then(|a| a.to_number().as_number())
        .unwrap_or(0.0);
    let b = args
        .get(4)
        .and_then(|a| a.to_number().as_number())
        .unwrap_or(1.0);
    if a >= b {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    let t = if p <= 0.0 {
        0.0
    } else if p >= 1.0 {
        1.0
    } else {
        Beta::new(alpha, beta).unwrap().inverse_cdf(p)
    };
    new_number_formula_arg(a + t * (b - a))
}

fn betainv(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    beta_dot_inv(_ctx, args)
}

fn chi_sq_dist(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    // CHISQ.DIST(x, deg_freedom, cumulative)
    if args.len() != 3 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let x = match args[0].to_number().as_number() {
        Some(n) if n >= 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let df = match args[1].to_number().as_number() {
        Some(n) if n > 0.0 && n.fract() == 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let cumulative = args[2].as_bool();
    let dist = ChiSquared::new(df).unwrap();
    if cumulative {
        new_number_formula_arg(dist.cdf(x))
    } else {
        new_number_formula_arg(dist.pdf(x))
    }
}

fn chi_sq_dist_rt(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    // CHISQ.DIST.RT(x, deg_freedom)
    if args.len() != 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let x = match args[0].to_number().as_number() {
        Some(n) if n >= 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let df = match args[1].to_number().as_number() {
        Some(n) if n > 0.0 && n.fract() == 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    new_number_formula_arg(ChiSquared::new(df).unwrap().sf(x))
}

fn chi_dist_rt(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    // CHIDIST(x, deg_freedom)
    chi_sq_dist_rt(_ctx, args)
}

fn chi_sq_inv(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    // CHISQ.INV(p, deg_freedom)
    if args.len() != 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let p = match args[0].to_number().as_number() {
        Some(n) if n >= 0.0 && n <= 1.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let df = match args[1].to_number().as_number() {
        Some(n) if n > 0.0 && n.fract() == 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let dist = ChiSquared::new(df).unwrap();
    if p <= 0.0 {
        new_number_formula_arg(0.0)
    } else if p >= 1.0 {
        new_number_formula_arg(f64::INFINITY)
    } else {
        new_number_formula_arg(dist.inverse_cdf(p))
    }
}

fn chi_sq_inv_rt(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    // CHISQ.INV.RT(p, deg_freedom)
    if args.len() != 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let p = match args[0].to_number().as_number() {
        Some(n) if n >= 0.0 && n <= 1.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let df = match args[1].to_number().as_number() {
        Some(n) if n > 0.0 && n.fract() == 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let dist = ChiSquared::new(df).unwrap();
    if p <= 0.0 {
        new_number_formula_arg(f64::INFINITY)
    } else if p >= 1.0 {
        new_number_formula_arg(0.0)
    } else {
        new_number_formula_arg(dist.inverse_cdf(1.0 - p))
    }
}

fn chi_inv_rt(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    // CHIINV(p, deg_freedom)
    chi_sq_inv_rt(_ctx, args)
}

fn chi_sq_test(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    // CHISQ.TEST(actual_range, expected_range)
    if args.len() != 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let actual = args[0].to_list();
    let expected = args[1].to_list();
    if actual.len() != expected.len() || actual.is_empty() {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    let mut stat = 0.0;
    for (a, e) in actual.iter().zip(expected.iter()) {
        let av = match a.to_number().as_number() {
            Some(n) => n,
            None => return new_error_formula_arg(FORMULA_ERROR_NUM),
        };
        let ev = match e.to_number().as_number() {
            Some(n) if n > 0.0 => n,
            _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
        };
        stat += (av - ev).powi(2) / ev;
    }
    let df = (actual.len() as f64 - 1.0).max(1.0);
    new_number_formula_arg(ChiSquared::new(df).unwrap().sf(stat))
}

fn confidence_norm(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    // CONFIDENCE(alpha, standard_dev, size)
    if args.len() != 3 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let alpha = match args[0].to_number().as_number() {
        Some(n) if n > 0.0 && n < 1.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let std_dev = match args[1].to_number().as_number() {
        Some(n) if n > 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let size = match args[2].to_number().as_number() {
        Some(n) if n >= 1.0 && n.fract() == 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let z = Normal::new(0.0, 1.0)
        .unwrap()
        .inverse_cdf(1.0 - alpha / 2.0);
    new_number_formula_arg(z * std_dev / size.sqrt())
}

fn confidence_t(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    // CONFIDENCE.T(alpha, standard_dev, size)
    if args.len() != 3 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let alpha = match args[0].to_number().as_number() {
        Some(n) if n > 0.0 && n < 1.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let std_dev = match args[1].to_number().as_number() {
        Some(n) if n > 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let size = match args[2].to_number().as_number() {
        Some(n) if n >= 1.0 && n.fract() == 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let df = size - 1.0;
    let t = StudentsT::new(0.0, 1.0, df)
        .unwrap()
        .inverse_cdf(1.0 - alpha / 2.0);
    new_number_formula_arg(t * std_dev / size.sqrt())
}

fn frequency(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    // FREQUENCY(data_array, bins_array) - matches Go's excelize behavior.
    if args.len() != 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }

    let data_matrix = if args[0].typ == ArgType::Matrix {
        args[0].clone()
    } else {
        new_matrix_formula_arg(vec![vec![args[0].clone()]])
    };
    let bins_matrix = if args[1].typ == ArgType::Matrix {
        args[1].clone()
    } else {
        new_matrix_formula_arg(vec![vec![args[1].clone()]])
    };

    let mut data: Vec<(usize, f64)> = Vec::new();
    for row in &data_matrix.matrix {
        for cell in row {
            if cell.typ == ArgType::Number {
                data.push((data.len(), cell.number));
            }
        }
    }
    data.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

    let mut bins: Vec<(usize, f64)> = Vec::new();
    for (row_idx, row) in bins_matrix.matrix.iter().enumerate() {
        for (col_idx, cell) in row.iter().enumerate() {
            if cell.typ == ArgType::Number {
                bins.push((row_idx * row.len() + col_idx, cell.number));
            }
        }
    }
    if bins.is_empty() {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    bins.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

    let mut out: Vec<Vec<FormulaArg>> = vec![vec![new_number_formula_arg(0.0); 1]; bins.len() + 1];
    let mut i = 0;
    for (_, original_idx, bin_val) in bins.iter().map(|(idx, val)| (*idx, *idx, *val)) {
        let mut n = 0.0;
        while i < data.len() && data[i].1 <= bin_val {
            n += 1.0;
            i += 1;
        }
        out[original_idx][0] = new_number_formula_arg(n);
    }
    out[bins.len()][0] = new_number_formula_arg((data.len() - i) as f64);
    new_matrix_formula_arg(out)
}

fn gamma_dist(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    // GAMMA.DIST(x, alpha, beta, cumulative)
    if args.len() != 4 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let x = match args[0].to_number().as_number() {
        Some(n) if n >= 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let alpha = match args[1].to_number().as_number() {
        Some(n) if n > 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let beta = match args[2].to_number().as_number() {
        Some(n) if n > 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let cumulative = args[3].as_bool();
    let dist = Gamma::new(alpha, 1.0 / beta).unwrap();
    if cumulative {
        new_number_formula_arg(dist.cdf(x))
    } else {
        new_number_formula_arg(dist.pdf(x))
    }
}

fn gammadist(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    gamma_dist(_ctx, args)
}

fn gamma_inv(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    // GAMMA.INV(p, alpha, beta)
    if args.len() != 3 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let p = match args[0].to_number().as_number() {
        Some(n) if n >= 0.0 && n <= 1.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let alpha = match args[1].to_number().as_number() {
        Some(n) if n > 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let beta = match args[2].to_number().as_number() {
        Some(n) if n > 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let dist = Gamma::new(alpha, 1.0 / beta).unwrap();
    if p <= 0.0 {
        new_number_formula_arg(0.0)
    } else if p >= 1.0 {
        new_number_formula_arg(f64::INFINITY)
    } else {
        new_number_formula_arg(dist.inverse_cdf(p))
    }
}

fn gammainv(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    gamma_inv(_ctx, args)
}

fn growth(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    // GROWTH(known_y, [known_x], [new_x], [const])
    if args.is_empty() || args.len() > 4 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let known_y: Vec<f64> = args[0]
        .to_list()
        .iter()
        .filter_map(|a| a.to_number().as_number())
        .collect();
    if known_y.len() < 2 {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    let n = known_y.len();
    let known_x: Vec<f64> = if args.len() >= 2 {
        let v: Vec<f64> = args[1]
            .to_list()
            .iter()
            .filter_map(|a| a.to_number().as_number())
            .collect();
        if v.len() != n {
            return new_error_formula_arg(FORMULA_ERROR_NUM);
        }
        v
    } else {
        (1..=n).map(|i| i as f64).collect()
    };
    let use_const = args.get(3).map(|a| a.as_bool()).unwrap_or(true);
    let ly: Vec<f64> = known_y.iter().map(|y| y.ln()).collect();
    let (slope, intercept) = linear_regression(&known_x, &ly, use_const);
    let new_x: Vec<f64> = if args.len() >= 3 {
        args[2]
            .to_list()
            .iter()
            .filter_map(|a| a.to_number().as_number())
            .collect()
    } else {
        known_x.clone()
    };
    let result: Vec<FormulaArg> = new_x
        .iter()
        .map(|x| new_number_formula_arg((intercept + slope * *x).exp()))
        .collect();
    new_list_formula_arg(result)
}

fn trend(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    // TREND(known_y, [known_x], [new_x], [const])
    if args.is_empty() || args.len() > 4 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let known_y: Vec<f64> = args[0]
        .to_list()
        .iter()
        .filter_map(|a| a.to_number().as_number())
        .collect();
    if known_y.len() < 2 {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    let n = known_y.len();
    let known_x: Vec<f64> = if args.len() >= 2 {
        let v: Vec<f64> = args[1]
            .to_list()
            .iter()
            .filter_map(|a| a.to_number().as_number())
            .collect();
        if v.len() != n {
            return new_error_formula_arg(FORMULA_ERROR_NUM);
        }
        v
    } else {
        (1..=n).map(|i| i as f64).collect()
    };
    let use_const = args.get(3).map(|a| a.as_bool()).unwrap_or(true);
    let (slope, intercept) = linear_regression(&known_x, &known_y, use_const);
    let new_x: Vec<f64> = if args.len() >= 3 {
        args[2]
            .to_list()
            .iter()
            .filter_map(|a| a.to_number().as_number())
            .collect()
    } else {
        known_x.clone()
    };
    let result: Vec<FormulaArg> = new_x
        .iter()
        .map(|x| new_number_formula_arg(intercept + slope * x))
        .collect();
    new_list_formula_arg(result)
}

fn linear_regression(x: &[f64], y: &[f64], use_const: bool) -> (f64, f64) {
    if !use_const {
        let num: f64 = x.iter().zip(y.iter()).map(|(xi, yi)| xi * yi).sum();
        let den: f64 = x.iter().map(|xi| xi * xi).sum();
        if den == 0.0 {
            return (0.0, 0.0);
        }
        return (num / den, 0.0);
    }
    let n = x.len() as f64;
    let mean_x = x.iter().sum::<f64>() / n;
    let mean_y = y.iter().sum::<f64>() / n;
    let mut num = 0.0;
    let mut den = 0.0;
    for i in 0..x.len() {
        num += (x[i] - mean_x) * (y[i] - mean_y);
        den += (x[i] - mean_x).powi(2);
    }
    if den == 0.0 {
        return (0.0, mean_y);
    }
    let slope = num / den;
    (slope, mean_y - slope * mean_x)
}

fn hypgeom_dist(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    // HYPGEOM.DIST(sample_s, number_sample, population_s, number_pop, cumulative)
    if args.len() != 5 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let sample_s = match args[0].to_number().as_number() {
        Some(n) if n >= 0.0 && n.fract() == 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let number_sample = match args[1].to_number().as_number() {
        Some(n) if n >= 0.0 && n.fract() == 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let population_s = match args[2].to_number().as_number() {
        Some(n) if n >= 0.0 && n.fract() == 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let number_pop = match args[3].to_number().as_number() {
        Some(n) if n >= 0.0 && n.fract() == 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let cumulative = args[4].as_bool();
    if sample_s > number_sample.min(population_s)
        || number_sample > number_pop
        || population_s > number_pop
        || sample_s < (number_sample - (number_pop - population_s)).max(0.0)
    {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    let dist =
        Hypergeometric::new(number_pop as u64, population_s as u64, number_sample as u64).unwrap();
    if cumulative {
        new_number_formula_arg(dist.cdf(sample_s as u64))
    } else {
        new_number_formula_arg(dist.pmf(sample_s as u64))
    }
}

fn hypgeomdist(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    // HYPGEOMDIST(sample_s, number_sample, population_s, number_pop) - PMF only
    if args.len() != 4 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let sample_s = match args[0].to_number().as_number() {
        Some(n) if n >= 0.0 && n.fract() == 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let number_sample = match args[1].to_number().as_number() {
        Some(n) if n >= 0.0 && n.fract() == 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let population_s = match args[2].to_number().as_number() {
        Some(n) if n >= 0.0 && n.fract() == 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let number_pop = match args[3].to_number().as_number() {
        Some(n) if n >= 0.0 && n.fract() == 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let dist =
        Hypergeometric::new(number_pop as u64, population_s as u64, number_sample as u64).unwrap();
    new_number_formula_arg(dist.pmf(sample_s as u64))
}

fn expon_dist(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    // EXPON.DIST(x, lambda, cumulative)
    if args.len() != 3 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let x = match args[0].to_number().as_number() {
        Some(n) if n >= 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let lambda = match args[1].to_number().as_number() {
        Some(n) if n > 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let cumulative = args[2].as_bool();
    let dist = Exp::new(lambda).unwrap();
    if cumulative {
        new_number_formula_arg(dist.cdf(x))
    } else {
        new_number_formula_arg(dist.pdf(x))
    }
}

fn expondist(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    expon_dist(_ctx, args)
}

fn f_dist(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    // F.DIST(x, deg_freedom1, deg_freedom2, cumulative)
    if args.len() != 4 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let x = match args[0].to_number().as_number() {
        Some(n) if n >= 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let df1 = match args[1].to_number().as_number() {
        Some(n) if n > 0.0 && n.fract() == 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let df2 = match args[2].to_number().as_number() {
        Some(n) if n > 0.0 && n.fract() == 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let cumulative = args[3].as_bool();
    let dist = FisherSnedecor::new(df1, df2).unwrap();
    if cumulative {
        new_number_formula_arg(dist.cdf(x))
    } else {
        new_number_formula_arg(dist.pdf(x))
    }
}

fn fdist(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    // FDIST(x, deg_freedom1, deg_freedom2) - right tail
    if args.len() != 3 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let x = match args[0].to_number().as_number() {
        Some(n) if n >= 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let df1 = match args[1].to_number().as_number() {
        Some(n) if n > 0.0 && n.fract() == 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let df2 = match args[2].to_number().as_number() {
        Some(n) if n > 0.0 && n.fract() == 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    new_number_formula_arg(FisherSnedecor::new(df1, df2).unwrap().sf(x))
}

fn f_dist_rt(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    // F.DIST.RT(x, deg_freedom1, deg_freedom2)
    if args.len() != 3 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let x = match args[0].to_number().as_number() {
        Some(n) if n >= 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let df1 = match args[1].to_number().as_number() {
        Some(n) if n > 0.0 && n.fract() == 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let df2 = match args[2].to_number().as_number() {
        Some(n) if n > 0.0 && n.fract() == 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    new_number_formula_arg(FisherSnedecor::new(df1, df2).unwrap().sf(x))
}

fn f_inv(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    // F.INV(p, deg_freedom1, deg_freedom2)
    if args.len() != 3 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let p = match args[0].to_number().as_number() {
        Some(n) if n >= 0.0 && n <= 1.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let df1 = match args[1].to_number().as_number() {
        Some(n) if n > 0.0 && n.fract() == 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let df2 = match args[2].to_number().as_number() {
        Some(n) if n > 0.0 && n.fract() == 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let dist = FisherSnedecor::new(df1, df2).unwrap();
    if p <= 0.0 {
        new_number_formula_arg(0.0)
    } else if p >= 1.0 {
        new_number_formula_arg(f64::INFINITY)
    } else {
        new_number_formula_arg(dist.inverse_cdf(p))
    }
}

fn finv(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    // FINV(p, deg_freedom1, deg_freedom2) - inverse right tail
    if args.len() != 3 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let p = match args[0].to_number().as_number() {
        Some(n) if n >= 0.0 && n <= 1.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let df1 = match args[1].to_number().as_number() {
        Some(n) if n > 0.0 && n.fract() == 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let df2 = match args[2].to_number().as_number() {
        Some(n) if n > 0.0 && n.fract() == 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let dist = FisherSnedecor::new(df1, df2).unwrap();
    if p <= 0.0 {
        new_number_formula_arg(f64::INFINITY)
    } else if p >= 1.0 {
        new_number_formula_arg(0.0)
    } else {
        new_number_formula_arg(dist.inverse_cdf(1.0 - p))
    }
}

fn f_inv_rt(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    finv(_ctx, args)
}

fn lognorm_dist(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    // LOGNORM.DIST(x, mean, standard_dev, cumulative)
    if args.len() != 4 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let x = match args[0].to_number().as_number() {
        Some(n) if n > 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let mean = match args[1].to_number().as_number() {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let sd = match args[2].to_number().as_number() {
        Some(n) if n > 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let cumulative = args[3].as_bool();
    let dist = LogNormal::new(mean, sd).unwrap();
    if cumulative {
        new_number_formula_arg(dist.cdf(x))
    } else {
        new_number_formula_arg(dist.pdf(x))
    }
}

fn lognormdist(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    // LOGNORMDIST(x, mean, standard_dev) - cumulative
    if args.len() != 3 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let x = match args[0].to_number().as_number() {
        Some(n) if n > 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let mean = match args[1].to_number().as_number() {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let sd = match args[2].to_number().as_number() {
        Some(n) if n > 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    new_number_formula_arg(LogNormal::new(mean, sd).unwrap().cdf(x))
}

fn lognorm_inv(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    // LOGNORM.INV(p, mean, standard_dev)
    if args.len() != 3 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let p = match args[0].to_number().as_number() {
        Some(n) if n > 0.0 && n < 1.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let mean = match args[1].to_number().as_number() {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let sd = match args[2].to_number().as_number() {
        Some(n) if n > 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let dist = LogNormal::new(mean, sd).unwrap();
    if p <= 0.0 {
        new_number_formula_arg(0.0)
    } else if p >= 1.0 {
        new_number_formula_arg(f64::INFINITY)
    } else {
        new_number_formula_arg(dist.inverse_cdf(p))
    }
}

fn loginv(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    lognorm_inv(_ctx, args)
}

fn negbinom_dist(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    // NEGBINOM.DIST(x, r, p, cumulative)
    if args.len() != 4 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let x = match args[0].to_number().as_number() {
        Some(n) if n >= 0.0 && n.fract() == 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let r = match args[1].to_number().as_number() {
        Some(n) if n >= 0.0 && n.fract() == 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let p = match args[2].to_number().as_number() {
        Some(n) if n > 0.0 && n <= 1.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let cumulative = args[3].as_bool();
    let dist = NegativeBinomial::new(r, p).unwrap();
    if cumulative {
        new_number_formula_arg(dist.cdf(x as u64))
    } else {
        new_number_formula_arg(dist.pmf(x as u64))
    }
}

fn negbinomdist(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    // NEGBINOMDIST(x, r, p) - PMF only
    if args.len() != 3 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let x = match args[0].to_number().as_number() {
        Some(n) if n >= 0.0 && n.fract() == 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let r = match args[1].to_number().as_number() {
        Some(n) if n >= 0.0 && n.fract() == 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let p = match args[2].to_number().as_number() {
        Some(n) if n > 0.0 && n <= 1.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    new_number_formula_arg(NegativeBinomial::new(r, p).unwrap().pmf(x as u64))
}

fn norm_dist(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    // NORM.DIST(x, mean, standard_dev, cumulative)
    if args.len() != 4 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let x = match args[0].to_number().as_number() {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let mean = match args[1].to_number().as_number() {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let sd = match args[2].to_number().as_number() {
        Some(n) if n > 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let cumulative = args[3].as_bool();
    let dist = Normal::new(mean, sd).unwrap();
    if cumulative {
        new_number_formula_arg(dist.cdf(x))
    } else {
        new_number_formula_arg(dist.pdf(x))
    }
}

fn normdist(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    norm_dist(_ctx, args)
}

fn norm_inv(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    // NORM.INV(p, mean, standard_dev)
    if args.len() != 3 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let p = match args[0].to_number().as_number() {
        Some(n) if n > 0.0 && n < 1.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let mean = match args[1].to_number().as_number() {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let sd = match args[2].to_number().as_number() {
        Some(n) if n > 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    new_number_formula_arg(Normal::new(mean, sd).unwrap().inverse_cdf(p))
}

fn norminv(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    norm_inv(_ctx, args)
}

fn maxifs(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    // MAXIFS(max_range, criteria_range1, criteria1, ...)
    if args.len() < 3 || args.len() % 2 == 0 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let max_range = args[0].to_list();
    let criteria_count = (args.len() - 1) / 2;
    let mut criteria: Vec<(Vec<FormulaArg>, Criteria)> = Vec::new();
    for i in 0..criteria_count {
        let range = args[i * 2 + 1].to_list();
        let c = parse_criteria(&args[i * 2 + 2]);
        criteria.push((range, c));
    }
    let mut max_val = f64::NEG_INFINITY;
    let mut has_value = false;
    for i in 0..max_range.len() {
        let mut ok = true;
        for (range, c) in &criteria {
            if let Some(item) = range.get(i) {
                if !criteria_matches(item, c) {
                    ok = false;
                    break;
                }
            } else {
                ok = false;
                break;
            }
        }
        if ok {
            if let Some(n) = max_range[i].to_number().as_number() {
                if !has_value || n > max_val {
                    max_val = n;
                    has_value = true;
                }
            }
        }
    }
    if has_value {
        new_number_formula_arg(max_val)
    } else {
        new_number_formula_arg(0.0)
    }
}

fn minifs(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    // MINIFS(min_range, criteria_range1, criteria1, ...)
    if args.len() < 3 || args.len() % 2 == 0 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let min_range = args[0].to_list();
    let criteria_count = (args.len() - 1) / 2;
    let mut criteria: Vec<(Vec<FormulaArg>, Criteria)> = Vec::new();
    for i in 0..criteria_count {
        let range = args[i * 2 + 1].to_list();
        let c = parse_criteria(&args[i * 2 + 2]);
        criteria.push((range, c));
    }
    let mut min_val = f64::INFINITY;
    let mut has_value = false;
    for i in 0..min_range.len() {
        let mut ok = true;
        for (range, c) in &criteria {
            if let Some(item) = range.get(i) {
                if !criteria_matches(item, c) {
                    ok = false;
                    break;
                }
            } else {
                ok = false;
                break;
            }
        }
        if ok {
            if let Some(n) = min_range[i].to_number().as_number() {
                if !has_value || n < min_val {
                    min_val = n;
                    has_value = true;
                }
            }
        }
    }
    if has_value {
        new_number_formula_arg(min_val)
    } else {
        new_number_formula_arg(0.0)
    }
}

fn steyx(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    // STEYX(known_y, known_x)
    if args.len() != 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let (y, x) = match paired_numeric_values(&args[0], &args[1]) {
        Some(v) if v.0.len() >= 3 => v,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let n = x.len() as f64;
    let mean_x = x.iter().sum::<f64>() / n;
    let mean_y = y.iter().sum::<f64>() / n;
    let mut num = 0.0;
    let mut den = 0.0;
    for i in 0..x.len() {
        num += (x[i] - mean_x) * (y[i] - mean_y);
        den += (x[i] - mean_x).powi(2);
    }
    if den == 0.0 {
        return new_error_formula_arg(FORMULA_ERROR_DIV);
    }
    let slope = num / den;
    let intercept = mean_y - slope * mean_x;
    let mut ss_res = 0.0;
    for i in 0..x.len() {
        let yhat = intercept + slope * x[i];
        ss_res += (y[i] - yhat).powi(2);
    }
    new_number_formula_arg((ss_res / (n - 2.0)).sqrt())
}

fn prob(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    // PROB(x_range, prob_range, lower, [upper])
    if args.len() < 3 || args.len() > 4 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let x: Vec<f64> = args[0]
        .to_list()
        .iter()
        .filter_map(|a| a.to_number().as_number())
        .collect();
    let p: Vec<f64> = args[1]
        .to_list()
        .iter()
        .filter_map(|a| a.to_number().as_number())
        .collect();
    if x.len() != p.len() || x.is_empty() {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    let lower = match args[2].to_number().as_number() {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let upper = if args.len() == 4 {
        match args[3].to_number().as_number() {
            Some(n) => n,
            None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
        }
    } else {
        lower
    };
    let mut sum = 0.0;
    for i in 0..x.len() {
        if x[i] >= lower && x[i] <= upper {
            sum += p[i];
        }
    }
    new_number_formula_arg(sum)
}

fn t_dist(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    // T.DIST(x, deg_freedom, cumulative)
    if args.len() != 3 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let x = match args[0].to_number().as_number() {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let df = match args[1].to_number().as_number() {
        Some(n) if n > 0.0 && n.fract() == 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let cumulative = args[2].as_bool();
    let dist = StudentsT::new(0.0, 1.0, df).unwrap();
    if cumulative {
        new_number_formula_arg(dist.cdf(x))
    } else {
        new_number_formula_arg(dist.pdf(x))
    }
}

fn t_dist_2t(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    // T.DIST.2T(x, deg_freedom)
    if args.len() != 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let x = match args[0].to_number().as_number() {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let df = match args[1].to_number().as_number() {
        Some(n) if n > 0.0 && n.fract() == 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let dist = StudentsT::new(0.0, 1.0, df).unwrap();
    new_number_formula_arg(2.0 * dist.sf(x.abs()))
}

fn t_dist_rt(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    // T.DIST.RT(x, deg_freedom)
    if args.len() != 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let x = match args[0].to_number().as_number() {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let df = match args[1].to_number().as_number() {
        Some(n) if n > 0.0 && n.fract() == 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    new_number_formula_arg(StudentsT::new(0.0, 1.0, df).unwrap().sf(x))
}

fn tdist(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    // TDIST(x, deg_freedom, tails)
    if args.len() != 3 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let x = match args[0].to_number().as_number() {
        Some(n) if n >= 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let df = match args[1].to_number().as_number() {
        Some(n) if n > 0.0 && n.fract() == 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let tails = match args[2].to_number().as_number() {
        Some(n) if n == 1.0 || n == 2.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let p = StudentsT::new(0.0, 1.0, df).unwrap().sf(x);
    new_number_formula_arg(if tails == 1.0 { p } else { 2.0 * p })
}

fn t_inv(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    // T.INV(p, deg_freedom)
    if args.len() != 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let p = match args[0].to_number().as_number() {
        Some(n) if n > 0.0 && n < 1.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let df = match args[1].to_number().as_number() {
        Some(n) if n > 0.0 && n.fract() == 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    new_number_formula_arg(StudentsT::new(0.0, 1.0, df).unwrap().inverse_cdf(p))
}

fn t_inv_2t(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    // T.INV.2T(p, deg_freedom)
    if args.len() != 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let p = match args[0].to_number().as_number() {
        Some(n) if n > 0.0 && n <= 1.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let df = match args[1].to_number().as_number() {
        Some(n) if n > 0.0 && n.fract() == 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    if p == 1.0 {
        return new_number_formula_arg(0.0);
    }
    new_number_formula_arg(
        StudentsT::new(0.0, 1.0, df)
            .unwrap()
            .inverse_cdf(1.0 - p / 2.0),
    )
}

fn tinv(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    // TINV(p, deg_freedom) - two-tailed inverse
    if args.len() != 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let p = match args[0].to_number().as_number() {
        Some(n) if n > 0.0 && n <= 1.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let df = match args[1].to_number().as_number() {
        Some(n) if n > 0.0 && n.fract() == 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    if p == 1.0 {
        return new_number_formula_arg(0.0);
    }
    new_number_formula_arg(
        StudentsT::new(0.0, 1.0, df)
            .unwrap()
            .inverse_cdf(1.0 - p / 2.0),
    )
}

fn z_test(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    // Z.TEST(data, mu, [sigma])
    if args.len() < 2 || args.len() > 3 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let data: Vec<f64> = args[0]
        .to_list()
        .iter()
        .filter_map(|a| a.to_number().as_number())
        .collect();
    if data.is_empty() {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    let mu = match args[1].to_number().as_number() {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let n = data.len() as f64;
    let xbar = data.iter().sum::<f64>() / n;
    let se = if args.len() == 3 {
        let sigma = match args[2].to_number().as_number() {
            Some(n) if n > 0.0 => n,
            _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
        };
        sigma / n.sqrt()
    } else {
        let var = data.iter().map(|v| (v - xbar).powi(2)).sum::<f64>() / (n - 1.0);
        var.sqrt() / n.sqrt()
    };
    if se == 0.0 {
        return new_error_formula_arg(FORMULA_ERROR_DIV);
    }
    let z = (xbar - mu) / se;
    new_number_formula_arg(Normal::new(0.0, 1.0).unwrap().sf(z))
}

fn ztest(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    z_test(_ctx, args)
}
