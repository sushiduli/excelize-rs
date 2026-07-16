//! Financial formula functions.

use std::collections::HashMap;

use crate::calc::arg::*;
use crate::calc::{CalcContext, FormulaFn};

pub fn register(m: &mut HashMap<&'static str, FormulaFn>) {
    m.insert("ACCRINT", accrint);
    m.insert("ACCRINTM", accrintm);
    m.insert("AMORDEGRC", amordegrc);
    m.insert("AMORLINC", amorlinc);
    m.insert("COUPDAYBS", coupdaybs);
    m.insert("COUPDAYS", coupdays);
    m.insert("COUPDAYSNC", coupdaysnc);
    m.insert("COUPNCD", coupncd);
    m.insert("COUPNUM", coupnum);
    m.insert("COUPPCD", couppcd);
    m.insert("CUMIPMT", cumipmt);
    m.insert("CUMPRINC", cumprinc);
    m.insert("DB", db);
    m.insert("DDB", ddb);
    m.insert("DISC", disc);
    m.insert("DOLLAR", dollar);
    m.insert("DOLLARDE", dollarde);
    m.insert("DOLLARFR", dollarfr);
    m.insert("DURATION", duration);
    m.insert("EFFECT", effect);
    m.insert("EUROCONVERT", euroconvert);
    m.insert("FV", fv);
    m.insert("FVSCHEDULE", fvschedule);
    m.insert("INTRATE", intrate);
    m.insert("IPMT", ipmt);
    m.insert("IRR", irr);
    m.insert("ISPMT", ispmt);
    m.insert("MDURATION", mduration);
    m.insert("MIRR", mirr);
    m.insert("NOMINAL", nominal);
    m.insert("NPER", nper);
    m.insert("NPV", npv);
    m.insert("ODDFPRICE", oddfprice);
    m.insert("ODDFYIELD", oddfyield);
    m.insert("ODDLPRICE", oddlprice);
    m.insert("ODDLYIELD", oddlyield);
    m.insert("PDURATION", pduration);
    m.insert("PMT", pmt);
    m.insert("PPMT", ppmt);
    m.insert("PRICE", price);
    m.insert("PRICEDISC", pricedisc);
    m.insert("PRICEMAT", pricemat);
    m.insert("PV", pv);
    m.insert("RATE", rate);
    m.insert("RECEIVED", received);
    m.insert("RRI", rri);
    m.insert("SLN", sln);
    m.insert("SYD", syd);
    m.insert("TBILLEQ", tbilleq);
    m.insert("TBILLPRICE", tbillprice);
    m.insert("TBILLYIELD", tbillyield);
    m.insert("VDB", vdb);
    m.insert("XIRR", xirr);
    m.insert("XNPV", xnpv);
    m.insert("YIELD", bond_yield_fn);
    m.insert("YIELDDISC", yielddisc_fn);
    m.insert("YIELDMAT", yieldmat_fn);
}

// ------------------------------------------------------------------
// Time value of money
// ------------------------------------------------------------------

fn pmt(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() < 3 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let rate = match args[0].to_number().as_number() {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let nper = match args[1].to_number().as_number() {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let pv = match args[2].to_number().as_number() {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let fv = args
        .get(3)
        .and_then(|a| a.to_number().as_number())
        .unwrap_or(0.0);
    let typ = args.get(4).map(|a| a.as_bool()).unwrap_or(false);

    new_number_formula_arg(pmt_internal(rate, nper, pv, fv, typ))
}

fn fv(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() < 3 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let rate = match args[0].to_number().as_number() {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let nper = match args[1].to_number().as_number() {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let pmt = match args[2].to_number().as_number() {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let pv = args
        .get(3)
        .and_then(|a| a.to_number().as_number())
        .unwrap_or(0.0);
    let typ = args.get(4).map(|a| a.as_bool()).unwrap_or(false);

    if rate == 0.0 {
        return new_number_formula_arg(-(pv + pmt * nper));
    }
    let factor = (1.0 + rate).powf(nper);
    let result = -pv * factor - pmt * (factor - 1.0) / rate * (if typ { 1.0 + rate } else { 1.0 });
    new_number_formula_arg(result)
}

fn pv(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() < 3 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let rate = match args[0].to_number().as_number() {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let nper = match args[1].to_number().as_number() {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let pmt = match args[2].to_number().as_number() {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let fv = args
        .get(3)
        .and_then(|a| a.to_number().as_number())
        .unwrap_or(0.0);
    let typ = args.get(4).map(|a| a.as_bool()).unwrap_or(false);

    if rate == 0.0 {
        return new_number_formula_arg(-(fv + pmt * nper));
    }
    let factor = (1.0 + rate).powf(nper);
    let result =
        (fv + pmt * (1.0 + rate * (if typ { 1.0 } else { 0.0 })) * (factor - 1.0) / rate) / factor;
    new_number_formula_arg(-result)
}

fn nper(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() < 3 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let rate = match args[0].to_number().as_number() {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let pmt = match args[1].to_number().as_number() {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let pv = match args[2].to_number().as_number() {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let fv = args
        .get(3)
        .and_then(|a| a.to_number().as_number())
        .unwrap_or(0.0);
    let typ = args.get(4).map(|a| a.as_bool()).unwrap_or(false);

    if rate == 0.0 {
        return new_number_formula_arg(-(pv + fv) / pmt);
    }
    let adjusted_pmt = pmt * (1.0 + rate * (if typ { 1.0 } else { 0.0 }));
    let num = adjusted_pmt - fv * rate;
    let den = pv * rate + adjusted_pmt;
    new_number_formula_arg((num / den).ln() / (1.0 + rate).ln())
}

fn rate(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() < 3 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let nper = match args[0].to_number().as_number() {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let pmt = match args[1].to_number().as_number() {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let pv = match args[2].to_number().as_number() {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let fv = args
        .get(3)
        .and_then(|a| a.to_number().as_number())
        .unwrap_or(0.0);
    let typ = args.get(4).map(|a| a.as_bool()).unwrap_or(false);
    let guess = args
        .get(5)
        .and_then(|a| a.to_number().as_number())
        .unwrap_or(0.1);

    let mut r = guess;
    for _ in 0..100 {
        let factor = (1.0 + r).powf(nper);
        let f =
            pv * factor + pmt * (1.0 + r * (if typ { 1.0 } else { 0.0 })) * (factor - 1.0) / r + fv;
        if f.abs() < 1e-10 {
            return new_number_formula_arg(r);
        }
        let df = pv * nper * (1.0 + r).powf(nper - 1.0)
            + pmt
                * (1.0 + r * (if typ { 1.0 } else { 0.0 }))
                * ((nper * r * (1.0 + r).powf(nper - 1.0) - factor + 1.0) / (r * r))
            + pmt * (if typ { 1.0 } else { 0.0 }) * (factor - 1.0) / r;
        if df == 0.0 {
            break;
        }
        r -= f / df;
    }
    new_number_formula_arg(r)
}

fn ipmt(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    calc_ipmt_ppmt("IPMT", args)
}

fn ppmt(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    calc_ipmt_ppmt("PPMT", args)
}

fn calc_ipmt_ppmt(name: &str, args: &[FormulaArg]) -> FormulaArg {
    if args.len() < 4 || args.len() > 6 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let rate = match args[0].to_number().as_number() {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let per = match args[1].to_number().as_number() {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let nper = match args[2].to_number().as_number() {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let pv = match args[3].to_number().as_number() {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let fv = args
        .get(4)
        .and_then(|a| a.to_number().as_number())
        .unwrap_or(0.0);
    let typ_arg = args
        .get(5)
        .map(|a| a.to_bool())
        .unwrap_or_else(|| new_number_formula_arg(0.0));
    if typ_arg.typ == ArgType::Error {
        return typ_arg;
    }
    let typ_num = typ_arg.number;
    if typ_num != 0.0 && typ_num != 1.0 {
        return new_error_formula_arg(FORMULA_ERROR_NA);
    }
    let typ = typ_num != 0.0;
    if per <= 0.0 || per > nper {
        return new_error_formula_arg(FORMULA_ERROR_NA);
    }
    let pmt = pmt_internal(rate, nper, pv, fv, typ);
    let mut capital = pv;
    let mut interest = 0.0;
    let mut principal = 0.0;
    for i in 1..=per as i32 {
        if typ && i == 1 {
            interest = 0.0;
        } else {
            interest = -capital * rate;
        }
        principal = pmt - interest;
        capital += principal;
    }
    if name == "IPMT" {
        new_number_formula_arg(interest)
    } else {
        new_number_formula_arg(principal)
    }
}

fn cumipmt(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    cumip("CUMIPMT", args)
}

fn cumprinc(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    cumip("CUMPRINC", args)
}

fn cumip(name: &str, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 6 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let rate = match args[0].to_number().as_number() {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let nper = match args[1].to_number().as_number() {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let pv = match args[2].to_number().as_number() {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let start = match args[3].to_number().as_number() {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let end = match args[4].to_number().as_number() {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let typ_arg = args[5].to_bool();
    if typ_arg.typ == ArgType::Error {
        return typ_arg;
    }
    let typ_num = typ_arg.number;
    if typ_num != 0.0 && typ_num != 1.0 {
        return new_error_formula_arg(FORMULA_ERROR_NA);
    }
    if start < 1.0 || start > end || end > nper {
        return new_error_formula_arg(FORMULA_ERROR_NA);
    }
    let mut total = 0.0;
    for per in start as i32..=end as i32 {
        let per_args = [
            new_number_formula_arg(rate),
            new_number_formula_arg(per as f64),
            new_number_formula_arg(nper),
            new_number_formula_arg(pv),
            new_number_formula_arg(0.0),
            new_number_formula_arg(typ_num),
        ];
        let val = calc_ipmt_ppmt(if name == "CUMIPMT" { "IPMT" } else { "PPMT" }, &per_args);
        if let Some(n) = val.as_number() {
            total += n;
        }
    }
    new_number_formula_arg(total)
}

fn pmt_internal(rate: f64, nper: f64, pv: f64, fv: f64, typ: bool) -> f64 {
    if rate == 0.0 {
        return -(pv + fv) / nper;
    }
    let factor = (1.0 + rate).powf(nper);
    let typ_adj = if typ { 1.0 } else { 0.0 };
    (-fv - pv * factor) / (1.0 + rate * typ_adj) / ((factor - 1.0) / rate)
}

fn balance_at_period(rate: f64, per: f64, nper: f64, pv: f64, fv: f64, typ: bool) -> f64 {
    let payment = pmt_internal(rate, nper, pv, fv, typ);
    let prev = per - 1.0;
    let factor = (1.0 + rate).powf(prev);
    pv * factor + payment * (factor - 1.0) / rate * (if typ { 1.0 + rate } else { 1.0 })
}

// ------------------------------------------------------------------
// Investment analysis
// ------------------------------------------------------------------

fn npv(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() < 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let rate = match args[0].to_number().as_number() {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let mut npv = 0.0;
    let mut i = 1;
    for arg in &args[1..] {
        for a in arg.to_list() {
            let num = a.to_number();
            if num.typ != ArgType::Number {
                continue;
            }
            npv += num.number / (1.0 + rate).powi(i);
            i += 1;
        }
    }
    new_number_formula_arg(npv)
}

fn irr(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.is_empty() {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let values: Vec<f64> = args[0]
        .to_list()
        .iter()
        .filter_map(|a| a.to_number().as_number())
        .collect();
    let guess = args
        .get(1)
        .and_then(|a| a.to_number().as_number())
        .unwrap_or(0.1);

    if values.len() < 2 {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    let mut r = guess;
    for _ in 0..100 {
        let mut npv = 0.0;
        let mut dnpv = 0.0;
        for (i, &v) in values.iter().enumerate() {
            let factor = (1.0 + r).powi(i as i32);
            npv += v / factor;
            dnpv -= (i as f64) * v / ((1.0 + r) * factor);
        }
        if npv.abs() < 1e-10 {
            return new_number_formula_arg(r);
        }
        if dnpv == 0.0 {
            break;
        }
        r -= npv / dnpv;
    }
    new_number_formula_arg(r)
}

fn mirr(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() < 3 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let values: Vec<f64> = args[0]
        .to_list()
        .iter()
        .filter_map(|a| a.to_number().as_number())
        .collect();
    let finance_rate = match args[1].to_number().as_number() {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let reinvest_rate = match args[2].to_number().as_number() {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };

    let mut npv_neg = 0.0;
    let mut fv_pos = 0.0;
    let n = values.len();
    for (i, &v) in values.iter().enumerate() {
        if v < 0.0 {
            npv_neg += v / (1.0 + finance_rate).powi(i as i32);
        } else {
            fv_pos += v * (1.0 + reinvest_rate).powi((n - i - 1) as i32);
        }
    }
    if npv_neg == 0.0 || fv_pos == 0.0 {
        return new_error_formula_arg(FORMULA_ERROR_DIV);
    }
    let r = (-fv_pos / npv_neg).powf(1.0 / (n as f64 - 1.0)) - 1.0;
    new_number_formula_arg(r)
}

fn fvschedule(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let principal = match args[0].to_number().as_number() {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let schedule: Vec<f64> = args[1]
        .to_list()
        .iter()
        .filter_map(|a| a.to_number().as_number())
        .collect();
    let mut result = principal;
    for rate in schedule {
        result *= 1.0 + rate;
    }
    new_number_formula_arg(result)
}

fn pduration(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 3 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let rate = match args[0].to_number().as_number() {
        Some(n) if n > 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let pv = match args[1].to_number().as_number() {
        Some(n) if n > 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let fv = match args[2].to_number().as_number() {
        Some(n) if n > 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    new_number_formula_arg((fv / pv).ln() / (1.0 + rate).ln())
}

fn rri(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 3 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let nper = match args[0].to_number().as_number() {
        Some(n) if n > 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let pv = match args[1].to_number().as_number() {
        Some(n) if n > 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let fv = match args[2].to_number().as_number() {
        Some(n) if n > 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    new_number_formula_arg((fv / pv).powf(1.0 / nper) - 1.0)
}

fn ispmt(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 4 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let rate = match args[0].to_number().as_number() {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let per = match args[1].to_number().as_number() {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let nper = match args[2].to_number().as_number() {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let pv = match args[3].to_number().as_number() {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let mut pr = pv;
    let payment = pv / nper;
    let mut num = 0.0;
    for i in 0..=per as i32 {
        num = -rate * pr;
        pr -= payment;
        if i == nper as i32 {
            num = 0.0;
        }
    }
    new_number_formula_arg(num)
}

// ------------------------------------------------------------------
// Depreciation
// ------------------------------------------------------------------

fn sln(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 3 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let cost = match args[0].to_number().as_number() {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let salvage = match args[1].to_number().as_number() {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let life = match args[2].to_number().as_number() {
        Some(n) if n > 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    new_number_formula_arg((cost - salvage) / life)
}

fn syd(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 4 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let cost = match args[0].to_number().as_number() {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let salvage = match args[1].to_number().as_number() {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let life = match args[2].to_number().as_number() {
        Some(n) if n > 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let per = match args[3].to_number().as_number() {
        Some(n) if n > 0.0 && n <= life => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    new_number_formula_arg((cost - salvage) * (life - per + 1.0) * 2.0 / (life * (life + 1.0)))
}

fn ddb(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() < 4 || args.len() > 5 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let cost = match args[0].to_number().as_number() {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let salvage = match args[1].to_number().as_number() {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let life = match args[2].to_number().as_number() {
        Some(n) if n > 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NA),
    };
    let per = match args[3].to_number().as_number() {
        Some(n) if n > 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NA),
    };
    let factor = match args
        .get(4)
        .map(|a| a.to_number().as_number())
        .unwrap_or(Some(2.0))
    {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };

    if cost == 0.0 {
        return new_number_formula_arg(0.0);
    }
    if cost <= 0.0
        || (salvage / cost) < 0.0
        || life <= 0.0
        || per < 1.0
        || factor <= 0.0
        || per > life
    {
        return new_error_formula_arg(FORMULA_ERROR_NA);
    }
    let mut pd = 0.0;
    let mut depreciation = 0.0;
    for _ in 1..=per as i32 {
        depreciation = ((cost - pd) * (factor / life)).min(cost - salvage - pd);
        pd += depreciation;
    }
    new_number_formula_arg(depreciation)
}

fn db(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() < 4 || args.len() > 5 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let cost = match args[0].to_number().as_number() {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let salvage = match args[1].to_number().as_number() {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let life = match args[2].to_number().as_number() {
        Some(n) if n > 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NA),
    };
    let period = match args[3].to_number().as_number() {
        Some(n) if n > 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NA),
    };
    let month = match args
        .get(4)
        .map(|a| a.to_number().as_number())
        .unwrap_or(Some(12.0))
    {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };

    if cost == 0.0 {
        return new_number_formula_arg(0.0);
    }
    if cost <= 0.0 || (salvage / cost) < 0.0 || life <= 0.0 || period < 1.0 || month < 1.0 {
        return new_error_formula_arg(FORMULA_ERROR_NA);
    }
    let mut rate = 1.0 - (salvage / cost).powf(1.0 / life);
    rate = (rate * 1000.0).round() / 1000.0;

    let mut pd = 0.0;
    let mut depreciation = 0.0;
    for per in 1..=period as i32 {
        depreciation = if per == 1 {
            cost * rate * month / 12.0
        } else if per == life as i32 + 1 {
            (cost - pd) * rate * (12.0 - month) / 12.0
        } else {
            (cost - pd) * rate
        };
        pd += depreciation;
    }
    new_number_formula_arg(depreciation)
}

// ------------------------------------------------------------------
// Interest conversion and securities helpers
// ------------------------------------------------------------------

fn effect(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let nominal = match args[0].to_number().as_number() {
        Some(n) if n > 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let npery = match args[1].to_number().as_number() {
        Some(n) if n >= 1.0 && n.fract() == 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    new_number_formula_arg((1.0 + nominal / npery).powf(npery) - 1.0)
}

fn nominal(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let effect_rate = match args[0].to_number().as_number() {
        Some(n) if n > 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let npery = match args[1].to_number().as_number() {
        Some(n) if n >= 1.0 && n.fract() == 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    new_number_formula_arg(npery * ((1.0 + effect_rate).powf(1.0 / npery) - 1.0))
}

fn dollarde(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let fractional = match args[0].to_number().as_number() {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let fraction = match args[1].to_number().as_number() {
        Some(n) if n > 0.0 && n.fract() == 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let integer = fractional.trunc();
    let frac = fractional.fract().abs();
    // Convert the fractional digits to an integer numerator.
    let mut numerator = frac;
    let mut scale = 1.0;
    while (numerator - numerator.round()).abs() > 1e-12 && scale < 1e12 {
        numerator *= 10.0;
        scale *= 10.0;
    }
    let numerator = numerator.round();
    let decimal = numerator / fraction;
    new_number_formula_arg(integer + decimal.copysign(fractional))
}

fn dollarfr(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let decimal = match args[0].to_number().as_number() {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let fraction = match args[1].to_number().as_number() {
        Some(n) if n > 0.0 && n.fract() == 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let integer = decimal.trunc();
    let frac = decimal.fract().abs() * fraction;
    new_number_formula_arg(integer + frac.copysign(decimal) / 100.0)
}

fn dollar(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.is_empty() || args.len() > 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let number = match args[0].to_number().as_number() {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let decimals = args
        .get(1)
        .and_then(|a| a.to_number().as_number())
        .unwrap_or(2.0) as i32;
    let mult = 10f64.powi(decimals);
    let rounded = (number.abs() * mult).round() / mult;
    let formatted = format!(
        "{:.decimals$}",
        rounded,
        decimals = decimals.max(0) as usize
    );
    // Insert thousands separators.
    let mut parts = formatted.splitn(2, '.');
    let int_part = parts.next().unwrap_or("");
    let frac_part = parts.next();
    let chars: Vec<char> = int_part.chars().collect();
    let mut s = String::new();
    for (i, c) in chars.iter().enumerate() {
        if i > 0 && (chars.len() - i) % 3 == 0 {
            s.push(',');
        }
        s.push(*c);
    }
    if let Some(frac) = frac_part {
        s.push('.');
        s.push_str(frac);
    }
    if number < 0.0 {
        new_string_formula_arg(format!("(${})", s))
    } else {
        new_string_formula_arg(format!("${}", s))
    }
}

// ------------------------------------------------------------------
// Bond / advanced financial functions
// ------------------------------------------------------------------

// ---- small date helpers (Excel 1900 serial system, std only) ----

fn is_leap(y: i32) -> bool {
    (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0)
}

fn month_days(y: i32, m: u32) -> u32 {
    match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap(y) {
                29
            } else {
                28
            }
        }
        _ => 0,
    }
}

fn valid_ymd(y: i32, m: u32, d: u32) -> bool {
    m >= 1 && m <= 12 && d >= 1 && d <= month_days(y, m)
}

fn date_to_days(y: i32, m: u32, d: u32) -> i64 {
    let mut days = 0i64;
    for yy in 1..y {
        days += if is_leap(yy) { 366 } else { 365 };
    }
    for mm in 1..m {
        days += month_days(y, mm) as i64;
    }
    days + d as i64 - 1
}

fn days_to_ymd(mut days: i64) -> (i32, u32, u32) {
    let mut y = 1;
    loop {
        let ylen = if is_leap(y) { 366 } else { 365 };
        if days < ylen {
            break;
        }
        days -= ylen;
        y += 1;
    }
    let mut m = 1u32;
    loop {
        let mlen = month_days(y, m) as i64;
        if days < mlen {
            break;
        }
        days -= mlen;
        m += 1;
    }
    (y, m, days as u32 + 1)
}

const EXCEL_EPOCH_Y: i32 = 1899;
const EXCEL_EPOCH_M: u32 = 12;
const EXCEL_EPOCH_D: u32 = 31;

fn excel_epoch_days() -> i64 {
    date_to_days(EXCEL_EPOCH_Y, EXCEL_EPOCH_M, EXCEL_EPOCH_D)
}

fn serial_to_ymd(s: f64) -> Option<(i32, u32, u32)> {
    if s < 0.0 {
        return None;
    }
    let days = if s < 60.0 {
        excel_epoch_days() + s as i64
    } else {
        excel_epoch_days() + s as i64 - 1
    };
    Some(days_to_ymd(days))
}

fn ymd_to_serial(y: i32, m: u32, d: u32) -> Option<f64> {
    if !valid_ymd(y, m, d) {
        return None;
    }
    let days = date_to_days(y, m, d);
    let mut s = days - excel_epoch_days();
    if s >= 60 {
        s += 1;
    }
    Some(s as f64)
}

fn parse_date_string(s: &str) -> Option<(i32, u32, u32)> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    // yyyy-mm-dd
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() == 3 {
        if let (Ok(y), Ok(m), Ok(d)) = (parts[0].parse(), parts[1].parse(), parts[2].parse()) {
            return Some((y, m, d));
        }
    }
    // mm/dd/yyyy
    let parts: Vec<&str> = s.split('/').collect();
    if parts.len() == 3 {
        if let (Ok(m), Ok(d), Ok(mut y)) = (
            parts[0].parse::<u32>(),
            parts[1].parse::<u32>(),
            parts[2].parse::<i32>(),
        ) {
            if y < 100 {
                y = if y < 30 { y + 2000 } else { y + 1900 };
            }
            return Some((y, m, d));
        }
    }
    None
}

fn to_serial(arg: &FormulaArg) -> Option<f64> {
    match arg.typ {
        ArgType::Number => Some(arg.number),
        ArgType::String => {
            let s = arg.string.trim();
            if let Ok(n) = s.parse::<f64>() {
                return Some(n);
            }
            parse_date_string(s).and_then(|(y, m, d)| ymd_to_serial(y, m, d))
        }
        ArgType::Empty => Some(0.0),
        _ => None,
    }
}

fn req_serial(arg: &FormulaArg) -> Option<f64> {
    to_serial(arg)
}

fn req_num(arg: &FormulaArg) -> Option<f64> {
    arg.to_number().as_number()
}

fn opt_num(args: &[FormulaArg], idx: usize, default: f64) -> f64 {
    args.get(idx)
        .and_then(|a| a.to_number().as_number())
        .unwrap_or(default)
}

fn opt_bool(args: &[FormulaArg], idx: usize, default: bool) -> bool {
    args.get(idx).map(|a| a.as_bool()).unwrap_or(default)
}

// ---- day count helpers ----

fn days360_us(start: f64, end: f64) -> f64 {
    let (y1, m1, d1) = serial_to_ymd(start).unwrap_or((2000, 1, 1));
    let (y2, m2, d2) = serial_to_ymd(end).unwrap_or((2000, 1, 1));
    let mut dd1 = d1 as i32;
    let mut dd2 = d2 as i32;
    if dd1 == 31 {
        dd1 = 30;
    }
    if dd2 == 31 && dd1 == 30 {
        dd2 = 30;
    }
    ((y2 - y1) * 360 + (m2 as i32 - m1 as i32) * 30 + (dd2 - dd1)) as f64
}

fn days360_eu(start: f64, end: f64) -> f64 {
    let (y1, m1, d1) = serial_to_ymd(start).unwrap_or((2000, 1, 1));
    let (y2, m2, d2) = serial_to_ymd(end).unwrap_or((2000, 1, 1));
    let dd1 = if d1 == 31 { 30 } else { d1 } as i32;
    let dd2 = if d2 == 31 { 30 } else { d2 } as i32;
    ((y2 - y1) * 360 + (m2 as i32 - m1 as i32) * 30 + (dd2 - dd1)) as f64
}

fn days_between(start: f64, end: f64, basis: i32) -> f64 {
    if start > end {
        return -days_between(end, start, basis);
    }
    match basis {
        0 => days360_us(start, end),
        1 | 2 | 3 => end - start,
        4 => days360_eu(start, end),
        _ => end - start,
    }
}

fn year_days(serial: f64, basis: i32) -> f64 {
    match basis {
        1 => {
            let (y, _, _) = serial_to_ymd(serial).unwrap_or((2000, 1, 1));
            if is_leap(y) { 366.0 } else { 365.0 }
        }
        3 => 365.0,
        _ => 360.0,
    }
}

fn year_frac_basis_cond(sy: i32, sm: u32, sd: u32, ey: i32, em: u32, ed: u32) -> bool {
    (is_leap(sy) && (sm < 2 || (sm == 2 && sd <= 29)))
        || (is_leap(ey) && (em > 2 || (em == 2 && ed == 29)))
}

fn year_frac_basis0(start: f64, end: f64) -> f64 {
    let (sy, mut sm, mut sd) = serial_to_ymd(start).unwrap_or((2000, 1, 1));
    let (ey, mut em, mut ed) = serial_to_ymd(end).unwrap_or((2000, 1, 1));
    if sd == 31 {
        sd = 30;
    }
    if sd == 30 && ed == 31 {
        ed = 30;
    } else if is_leap(sy) && sm == 2 && sd == 29 {
        sd = 30;
        if is_leap(ey) && em == 2 && ed == 29 {
            ed = 30;
        }
    } else if !is_leap(sy) && sm == 2 && sd == 28 {
        sd = 30;
        if !is_leap(ey) && em == 2 && ed == 28 {
            ed = 30;
        }
    }
    let day_diff = (ey - sy) * 360 + (em as i32 - sm as i32) * 30 + (ed as i32 - sd as i32);
    day_diff as f64 / 360.0
}

fn year_frac_basis1(start: f64, end: f64) -> f64 {
    let (sy, sm, sd) = serial_to_ymd(start).unwrap_or((2000, 1, 1));
    let (ey, em, ed) = serial_to_ymd(end).unwrap_or((2000, 1, 1));
    let mut day_diff = end - start;
    let days_in_year = if sy != ey {
        if ey != sy + 1 || sm < em || (sm == em && sd < ed) {
            let mut count = 0.0;
            for y in sy..=ey {
                count += if is_leap(y) { 366.0 } else { 365.0 };
            }
            count / (ey - sy + 1) as f64
        } else if year_frac_basis_cond(sy, sm, sd, ey, em, ed) {
            366.0
        } else {
            365.0
        }
    } else if is_leap(sy) {
        366.0
    } else {
        365.0
    };
    day_diff / days_in_year
}

fn year_frac_basis4(start: f64, end: f64) -> f64 {
    let (sy, mut sm, mut sd) = serial_to_ymd(start).unwrap_or((2000, 1, 1));
    let (ey, mut em, mut ed) = serial_to_ymd(end).unwrap_or((2000, 1, 1));
    if sd == 31 {
        sd = 30;
    }
    if ed == 31 {
        ed = 30;
    }
    let day_diff = (ey - sy) * 360 + (em as i32 - sm as i32) * 30 + (ed as i32 - sd as i32);
    day_diff as f64 / 360.0
}

fn year_frac(start: f64, end: f64, basis: i32) -> f64 {
    if start == end {
        return 0.0;
    }
    match basis {
        0 => year_frac_basis0(start, end),
        1 => year_frac_basis1(start, end),
        2 => (end - start) / 360.0,
        3 => (end - start) / 365.0,
        4 => year_frac_basis4(start, end),
        _ => (end - start) / 360.0,
    }
}

fn add_months(y: i32, m: u32, d: u32, months: i32) -> (i32, u32, u32) {
    let total = y as i32 * 12 + (m as i32) - 1 + months;
    let ny = total / 12;
    let nm = (total % 12) + 1;
    let nd = d.min(month_days(ny, nm as u32));
    (ny, nm as u32, nd)
}

fn prev_coupon_date(settlement: f64, maturity: f64, freq: i32) -> f64 {
    let (my, mm, md) = serial_to_ymd(maturity).unwrap_or((2000, 1, 1));
    let step = 12 / freq;
    let mut y = my;
    let mut m = mm as i32;
    loop {
        if let Some(s) = ymd_to_serial(y, m as u32, md.min(month_days(y, m as u32))) {
            if s <= settlement {
                return s;
            }
        }
        m -= step;
        if m <= 0 {
            y -= 1;
            m += 12;
        }
    }
}

fn next_coupon_date(settlement: f64, maturity: f64, freq: i32) -> f64 {
    let pcd = prev_coupon_date(settlement, maturity, freq);
    let (y, m, d) = serial_to_ymd(pcd).unwrap_or((2000, 1, 1));
    let (ny, nm, nd) = add_months(y, m, d, 12 / freq);
    ymd_to_serial(ny, nm, nd).unwrap_or(pcd)
}

fn coupon_dates(settlement: f64, maturity: f64, freq: i32) -> (f64, f64) {
    let pcd = prev_coupon_date(settlement, maturity, freq);
    let ncd = next_coupon_date(settlement, maturity, freq);
    (pcd, ncd)
}

fn coupon_period_days(basis: i32, freq: i32) -> f64 {
    match basis {
        1 => 365.25 / freq as f64,
        3 => 365.0 / freq as f64,
        _ => 360.0 / freq as f64,
    }
}

fn coup_num(settlement: f64, maturity: f64, freq: i32) -> i32 {
    let frac = year_frac(settlement, maturity, 0);
    (frac * freq as f64).ceil() as i32
}

// ---- ODDFPRICE helpers (matching Go excelize implementation) ----

fn is_30_basis(basis: i32) -> bool {
    basis == 0 || basis == 4
}

fn get_year_days(_year: i32, basis: i32) -> i32 {
    match basis {
        1 => 365,
        3 => 365,
        _ => 360,
    }
}

fn day_on_basis(y: i32, m: i32, d: i32, basis: i32) -> i32 {
    if !is_30_basis(basis) {
        return d;
    }
    let dim = month_days(y, m as u32) as i32;
    let mut day = d;
    if day > 30 || d >= dim || day >= dim {
        day = 30;
    }
    day
}

fn days_in_month_range(from_month: i32, to_month: i32) -> i32 {
    if from_month > to_month {
        0
    } else {
        (to_month - from_month + 1) * 30
    }
}

fn coupdays_go(from: f64, to: f64, basis: i32) -> f64 {
    if !is_30_basis(basis) {
        return to - from;
    }
    let (fy, fm, fd) = serial_to_ymd(from).unwrap_or((2000, 1, 1));
    let (ty, tm, td) = serial_to_ymd(to).unwrap_or((2000, 1, 1));
    let mut from_day = day_on_basis(fy, fm as i32, fd as i32, basis);
    let mut to_day = day_on_basis(ty, tm as i32, td as i32, basis);
    if basis == 0 {
        if (fm == 2 || from_day < 30) && td == 31 {
            to_day = 31;
        }
    } else {
        if fm == 2 && from_day == 30 {
            from_day = month_days(fy, 2) as i32;
        }
        if tm == 2 && to_day == 30 {
            to_day = month_days(ty, 2) as i32;
        }
    }
    let mut days = 0;
    if fy < ty || (fy == ty && fm < tm) {
        days = 30 - from_day + 1;
        let mut date_y = fy;
        let mut date_m = fm as i32 + 1;
        if date_m > 12 {
            date_y += 1;
            date_m = 1;
        }
        if date_y < ty {
            days += days_in_month_range(date_m, 12);
            date_y += 1;
            date_m = 1;
        }
        days += days_in_month_range(date_m, tm as i32 - 1);
        from_day = 1;
    }
    days += to_day - from_day;
    if days > 0 { days as f64 } else { 0.0 }
}

fn change_month(date: f64, num_months: i32, return_last_month: bool) -> f64 {
    let (y, m, d) = serial_to_ymd(date).unwrap_or((2000, 1, 1));
    let mut offset_day = 0;
    if return_last_month && d == month_days(y, m) {
        offset_day = -1;
    }
    let (ny, nm, nd) = add_months(y, m, d, num_months);
    let new_serial = ymd_to_serial(ny, nm, nd).unwrap_or(date);
    let new_serial = new_serial + offset_day as f64;
    if return_last_month {
        let (ny2, nm2, _) = serial_to_ymd(new_serial).unwrap_or((ny, nm, nd));
        let last_day = month_days(ny2, nm2);
        ymd_to_serial(ny2, nm2, last_day).unwrap_or(new_serial)
    } else {
        new_serial
    }
}

fn dates_aggregate<F>(
    start: f64,
    end: f64,
    num_months: i32,
    init_acc: f64,
    return_last_month: bool,
    mut f: F,
) -> (f64, f64, f64)
where
    F: FnMut(f64, f64) -> f64,
{
    let mut front_date = start;
    let mut trailing_date = end;
    let mut stop = if num_months > 0 {
        front_date >= end
    } else {
        end >= front_date
    };
    let mut acc = init_acc;
    while !stop {
        trailing_date = front_date;
        front_date = change_month(front_date, num_months, return_last_month);
        acc += f(front_date, trailing_date);
        stop = if num_months > 0 {
            front_date >= end
        } else {
            end >= front_date
        };
    }
    (front_date, trailing_date, acc)
}

fn coup_number(maturity: f64, settlement: f64, num_months: i32) -> f64 {
    let (my, mm, md) = serial_to_ymd(maturity).unwrap_or((2000, 1, 1));
    let (sy, sm, sd) = serial_to_ymd(settlement).unwrap_or((2000, 1, 1));
    let end_of_month_temp = md == month_days(my, mm);
    let mut end_of_month = end_of_month_temp;
    if !end_of_month_temp && mm != 2 && md > 28 && md < month_days(my, mm) {
        end_of_month = sd == month_days(sy, sm);
    }
    let start_date = change_month(settlement, 0, end_of_month);
    let mut coupons = 0.0;
    if start_date > settlement {
        coupons += 1.0;
    }
    let date = change_month(start_date, num_months, end_of_month);
    let (_, _, result) =
        dates_aggregate(date, maturity, num_months, coupons, end_of_month, |_, _| {
            1.0
        });
    result
}

fn aggr_between<F>(start_period: f64, end_period: f64, initial: Vec<f64>, mut f: F) -> Vec<f64>
where
    F: FnMut(&[f64], f64) -> Vec<f64>,
{
    let mut acc = initial;
    let start = start_period as i64;
    let end = end_period as i64;
    if start <= end {
        for i in start..=end {
            acc = f(&acc, i as f64);
        }
    } else {
        for i in (end..=start).rev() {
            acc = f(&acc, i as f64);
        }
    }
    acc
}

fn coupons_internal(settlement: f64, maturity: f64, freq: i32, name: &str) -> f64 {
    let (set_y, set_m, set_d) = serial_to_ymd(settlement).unwrap_or((2000, 1, 1));
    let (mat_y, mat_m, mat_d) = serial_to_ymd(maturity).unwrap_or((2000, 1, 1));
    let maturity_days = (mat_y - set_y) * 12 + (mat_m as i32 - set_m as i32);
    let coupon = 12 / freq;
    let md = maturity_days % coupon;
    let mut year = set_y;
    let mut month = set_m as i32;
    if md == 0 && set_d >= mat_d {
        month += coupon;
    } else {
        month += md;
    }
    if name == "COUPPCD" {
        month -= coupon;
    }
    while month > 12 {
        year += 1;
        month -= 12;
    }
    while month < 1 {
        year -= 1;
        month += 12;
    }
    let mut day = mat_d;
    let days_in_target_month = month_days(year, month as u32);
    if month_days(mat_y, mat_m) == mat_d {
        day = days_in_target_month;
    } else if day > 27 && day > days_in_target_month {
        day = days_in_target_month;
    }
    ymd_to_serial(year, month as u32, day).unwrap_or(settlement)
}

fn coup_ncd_internal(settlement: f64, maturity: f64, freq: i32) -> f64 {
    coupons_internal(settlement, maturity, freq, "COUPNCD")
}

fn coup_pcd_internal(settlement: f64, maturity: f64, freq: i32) -> f64 {
    coupons_internal(settlement, maturity, freq, "COUPPCD")
}

fn coup_days_internal(settlement: f64, maturity: f64, freq: i32, basis: i32) -> f64 {
    if basis == 1 {
        let pcd = coup_pcd_internal(settlement, maturity, freq);
        let (y, m, d) = serial_to_ymd(pcd).unwrap_or((2000, 1, 1));
        let (ny, nm, nd) = add_months(y, m, d, 12 / freq);
        let ncd = ymd_to_serial(ny, nm, nd).unwrap_or(pcd);
        return coupdays_go(pcd, ncd, basis);
    }
    get_year_days(0, basis) as f64 / freq as f64
}

fn coup_num_internal(settlement: f64, maturity: f64, freq: i32) -> f64 {
    let frac = year_frac(settlement, maturity, 0);
    (frac * freq as f64).ceil()
}

// ---- bond price / yield / duration helpers ----

fn bond_price(
    settlement: f64,
    maturity: f64,
    rate: f64,
    yld: f64,
    redemption: f64,
    freq: i32,
    basis: i32,
) -> f64 {
    let n = coup_num(settlement, maturity, freq) as f64;
    if n <= 0.0 {
        return redemption;
    }
    let (pcd, ncd) = coupon_dates(settlement, maturity, freq);
    let e = days_between(pcd, ncd, basis);
    let a = days_between(pcd, settlement, basis);
    let dsc_ratio = days_between(settlement, ncd, basis) / e;
    let coupon = 100.0 * rate / freq as f64;
    let y = yld / freq as f64;
    let mut ret = 0.0;
    if n > 1.0 {
        ret = redemption / (1.0 + y).powf(n - 1.0 + dsc_ratio);
        ret -= coupon * a / e;
        let t2 = 1.0 + y;
        for k in 0..n as i32 {
            ret += coupon / t2.powf(k as f64 + dsc_ratio);
        }
    } else {
        let dsc = e - a;
        let t1 = coupon + redemption;
        let t2 = y * (dsc / e) + 1.0;
        let t3 = coupon * (a / e);
        ret = t1 / t2 - t3;
    }
    ret
}

fn bond_yield(
    settlement: f64,
    maturity: f64,
    rate: f64,
    pr: f64,
    redemption: f64,
    freq: i32,
    basis: i32,
) -> Option<f64> {
    let mut yield1 = 0.0;
    let mut yield2 = 1.0;
    let mut price1 = bond_price(settlement, maturity, rate, yield1, redemption, freq, basis);
    let mut price2 = bond_price(settlement, maturity, rate, yield2, redemption, freq, basis);
    let mut yield_n = (yield2 - yield1) * 0.5;
    for _ in 0..100 {
        let price_n = bond_price(settlement, maturity, rate, yield_n, redemption, freq, basis);
        if (price_n - pr).abs() < 1e-12 {
            return Some(yield_n);
        }
        if (pr - price1).abs() < 1e-12 {
            return Some(yield1);
        }
        if (pr - price2).abs() < 1e-12 {
            return Some(yield2);
        }
        if pr < price2 {
            yield2 *= 2.0;
            price2 = bond_price(settlement, maturity, rate, yield2, redemption, freq, basis);
            yield_n = (yield2 - yield1) * 0.5;
        } else {
            if pr < price_n {
                yield1 = yield_n;
                price1 = price_n;
            } else {
                yield2 = yield_n;
                price2 = price_n;
            }
            let f1 = (yield2 - yield1) * ((pr - price2) / (price1 - price2));
            yield_n = yield2 - f1;
        }
    }
    Some(yield_n)
}

fn macaulay_duration(
    settlement: f64,
    maturity: f64,
    rate: f64,
    yld: f64,
    redemption: f64,
    freq: i32,
    _basis: i32,
) -> f64 {
    let n = coup_num(settlement, maturity, freq);
    if n <= 0 || yld <= -1.0 {
        return 0.0;
    }
    let coupon = redemption * rate / freq as f64;
    let y = yld / freq as f64;
    let mut pv = 0.0;
    let mut weighted = 0.0;
    for i in 1..=n {
        let cf = if i == n { coupon + redemption } else { coupon };
        let p = cf / (1.0 + y).powi(i);
        pv += p;
        weighted += i as f64 * p;
    }
    if pv == 0.0 {
        return 0.0;
    }
    (weighted / pv) / freq as f64
}

// ---- XNPV / XIRR helpers ----

fn xnpv_rate(rate: f64, values: &[f64], dates: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let first = dates[0];
    values
        .iter()
        .zip(dates.iter())
        .map(|(&v, &d)| v / (1.0 + rate).powf((d - first) / 365.0))
        .sum()
}

fn xirr_solve(values: &[f64], dates: &[f64], guess: f64) -> Option<f64> {
    let mut r = guess;
    for _ in 0..100 {
        let npv = xnpv_rate(r, values, dates);
        if npv.abs() < 1e-12 {
            return Some(r);
        }
        let first = dates[0];
        let der: f64 = values
            .iter()
            .zip(dates.iter())
            .map(|(&v, &d)| {
                let e = (d - first) / 365.0;
                -e * v / (1.0 + r).powf(e + 1.0)
            })
            .sum();
        if der == 0.0 {
            break;
        }
        r -= npv / der;
        if r <= -1.0 {
            r = -0.9999;
        }
    }
    // Fallback bisection.
    let mut low = -0.999999;
    let mut high = 10.0;
    let mut f_low = xnpv_rate(low, values, dates);
    let mut _f_high = xnpv_rate(high, values, dates);
    if f_low.is_infinite() || f_low.is_nan() {
        f_low = 1e308;
    }
    if _f_high.is_infinite() || _f_high.is_nan() {
        _f_high = -1e308;
    }
    if f_low.signum() == _f_high.signum() {
        return None;
    }
    for _ in 0..100 {
        let mid = (low + high) / 2.0;
        let f_mid = xnpv_rate(mid, values, dates);
        if f_mid.abs() < 1e-12 {
            return Some(mid);
        }
        if f_mid.signum() == f_low.signum() {
            low = mid;
            f_low = f_mid;
        } else {
            high = mid;
            _f_high = f_mid;
        }
    }
    Some((low + high) / 2.0)
}

// ---- VDB helpers ----

fn ddb_period_dep(cost: f64, salvage: f64, life: f64, per: i32, factor: f64) -> f64 {
    let mut pd = 0.0;
    let mut depreciation = 0.0;
    for _ in 1..=per {
        depreciation = ((cost - pd) * (factor / life)).min(cost - salvage - pd);
        pd += depreciation;
    }
    depreciation
}

/// `vdb_internal` mirrors Go's lower-case `vdb`: cumulative depreciation from
/// period 0 up to `period` using DDB/SLN switching.
fn vdb_internal(cost: f64, salvage: f64, life: f64, life1: f64, period: f64, factor: f64) -> f64 {
    let end_int = period.ceil();
    let cs = cost - salvage;
    let mut ddb_dep: f64;
    let mut sln_dep: f64;
    let mut term: f64;
    let mut vdb = 0.0;
    let mut cs_remaining = cs;
    let mut now_sln = false;
    let mut current_sln = 0.0;
    for i in 1..=end_int as i32 {
        if !now_sln {
            ddb_dep = ddb_period_dep(cost, salvage, life, i, factor);
            sln_dep = cs_remaining / (life1 - i as f64 + 1.0);
            if sln_dep > ddb_dep && i as f64 != end_int {
                term = sln_dep;
                now_sln = true;
                current_sln = sln_dep;
            } else {
                term = ddb_dep;
                cs_remaining -= ddb_dep;
            }
        } else {
            term = current_sln;
        }
        if i as f64 == end_int {
            term *= period + 1.0 - end_int;
        }
        vdb += term;
    }
    vdb
}

// ---- function implementations ----

fn accrint(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() < 6 || args.len() > 8 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let issue = match req_serial(&args[0]) {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let first = match req_serial(&args[1]) {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let settlement = match req_serial(&args[2]) {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let rate = match req_num(&args[3]) {
        Some(n) if n > 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let par = match req_num(&args[4]) {
        Some(n) if n > 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let freq = match args[5].to_number().as_number() {
        Some(n) if n == 1.0 || n == 2.0 || n == 4.0 => n as i32,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let basis = opt_num(args, 6, 0.0) as i32;
    if basis < 0 || basis > 4 {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    let calc_method = opt_bool(args, 7, true);
    if settlement <= issue {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    let a = if calc_method {
        days_between(issue, settlement, basis)
    } else {
        days_between(first, settlement, basis)
    };
    let period_days = coupon_period_days(basis, freq);
    new_number_formula_arg(par * rate / freq as f64 * (a / period_days))
}

fn accrintm(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() < 4 || args.len() > 5 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let issue = match req_serial(&args[0]) {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let maturity = match req_serial(&args[1]) {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let rate = match req_num(&args[2]) {
        Some(n) if n > 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let par = match req_num(&args[3]) {
        Some(n) if n > 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let basis = opt_num(args, 4, 0.0) as i32;
    if basis < 0 || basis > 4 || maturity <= issue {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    let a = days_between(issue, maturity, basis);
    let ydays = year_days(issue, basis);
    new_number_formula_arg(par * rate * (a / ydays))
}

fn amorlinc(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() < 6 || args.len() > 7 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let cost = match req_num(&args[0]) {
        Some(n) if n > 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let purchase = match req_serial(&args[1]) {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let first = match req_serial(&args[2]) {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let salvage = match req_num(&args[3]) {
        Some(n) if n >= 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let period = match req_num(&args[4]) {
        Some(n) if n >= 0.0 => n as i32,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let rate = match req_num(&args[5]) {
        Some(n) if n > 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let basis = opt_num(args, 6, 0.0) as i32;
    if basis < 0 || basis > 4 {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    if first <= purchase {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    let frac = days_between(purchase, first, basis) / year_days(purchase, basis);
    let annual = cost * rate;
    let first_dep = annual * frac;
    let dep = if period == 0 {
        first_dep
    } else {
        let book = cost - first_dep - (period - 1) as f64 * annual;
        if book <= salvage {
            0.0
        } else {
            (book - salvage).min(annual)
        }
    };
    new_number_formula_arg(dep.max(0.0).min(cost - salvage))
}

fn amordegrc(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() < 6 || args.len() > 7 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let cost = match req_num(&args[0]) {
        Some(n) if n > 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let purchase = match req_serial(&args[1]) {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let first = match req_serial(&args[2]) {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let salvage = match req_num(&args[3]) {
        Some(n) if n >= 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let period = match req_num(&args[4]) {
        Some(n) if n >= 0.0 => n as i32,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let rate = match req_num(&args[5]) {
        Some(n) if n > 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let basis = opt_num(args, 6, 0.0) as i32;
    if basis < 0 || basis > 4 || first <= purchase {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    let life = 1.0 / rate;
    let coef = if life < 3.0 {
        1.0
    } else if life < 5.0 {
        1.5
    } else if life <= 6.0 {
        2.0
    } else {
        2.5
    };
    let frac = days_between(purchase, first, basis) / year_days(purchase, basis);
    let db_rate = coef / life;
    let mut book = cost - salvage;
    let first_dep = book * db_rate * frac;
    if period == 0 {
        return new_number_formula_arg(first_dep.min(cost - salvage));
    }
    book -= first_dep;
    for _ in 1..period {
        let dep = (book * db_rate).min(book);
        book -= dep;
    }
    let dep = (book * db_rate).min(book);
    new_number_formula_arg(dep.min(cost - salvage).max(0.0))
}

fn coupdaybs(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() < 3 || args.len() > 4 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let settlement = match req_serial(&args[0]) {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let maturity = match req_serial(&args[1]) {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let freq = match args[2].to_number().as_number() {
        Some(n) if n == 1.0 || n == 2.0 || n == 4.0 => n as i32,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let basis = opt_num(args, 3, 0.0) as i32;
    if basis < 0 || basis > 4 || maturity <= settlement {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    let (pcd, _) = coupon_dates(settlement, maturity, freq);
    new_number_formula_arg(days_between(pcd, settlement, basis))
}

fn coupdays(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() < 3 || args.len() > 4 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let settlement = match req_serial(&args[0]) {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let maturity = match req_serial(&args[1]) {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let freq = match args[2].to_number().as_number() {
        Some(n) if n == 1.0 || n == 2.0 || n == 4.0 => n as i32,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let basis = opt_num(args, 3, 0.0) as i32;
    if basis < 0 || basis > 4 || maturity <= settlement {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    let (pcd, ncd) = coupon_dates(settlement, maturity, freq);
    new_number_formula_arg(days_between(pcd, ncd, basis))
}

fn coupdaysnc(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() < 3 || args.len() > 4 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let settlement = match req_serial(&args[0]) {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let maturity = match req_serial(&args[1]) {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let freq = match args[2].to_number().as_number() {
        Some(n) if n == 1.0 || n == 2.0 || n == 4.0 => n as i32,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let basis = opt_num(args, 3, 0.0) as i32;
    if basis < 0 || basis > 4 || maturity <= settlement {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    let (_, ncd) = coupon_dates(settlement, maturity, freq);
    new_number_formula_arg(days_between(settlement, ncd, basis))
}

fn coupncd(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() < 3 || args.len() > 4 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let settlement = match req_serial(&args[0]) {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let maturity = match req_serial(&args[1]) {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let freq = match args[2].to_number().as_number() {
        Some(n) if n == 1.0 || n == 2.0 || n == 4.0 => n as i32,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let basis = opt_num(args, 3, 0.0) as i32;
    if basis < 0 || basis > 4 || maturity <= settlement {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    let (_, ncd) = coupon_dates(settlement, maturity, freq);
    new_number_formula_arg(ncd)
}

fn coupnum(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() < 3 || args.len() > 4 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let settlement = match req_serial(&args[0]) {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let maturity = match req_serial(&args[1]) {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let freq = match args[2].to_number().as_number() {
        Some(n) if n == 1.0 || n == 2.0 || n == 4.0 => n as i32,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let basis = opt_num(args, 3, 0.0) as i32;
    if basis < 0 || basis > 4 || maturity <= settlement {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    new_number_formula_arg(coup_num(settlement, maturity, freq) as f64)
}

fn couppcd(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() < 3 || args.len() > 4 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let settlement = match req_serial(&args[0]) {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let maturity = match req_serial(&args[1]) {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let freq = match args[2].to_number().as_number() {
        Some(n) if n == 1.0 || n == 2.0 || n == 4.0 => n as i32,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let basis = opt_num(args, 3, 0.0) as i32;
    if basis < 0 || basis > 4 || maturity <= settlement {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    let (pcd, _) = coupon_dates(settlement, maturity, freq);
    new_number_formula_arg(pcd)
}

fn disc(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() < 4 || args.len() > 5 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let settlement = match req_serial(&args[0]) {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let maturity = match req_serial(&args[1]) {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let pr = match req_num(&args[2]) {
        Some(n) if n > 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let redemption = match req_num(&args[3]) {
        Some(n) if n > 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let basis = opt_num(args, 4, 0.0) as i32;
    if basis < 0 || basis > 4 || maturity <= settlement {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    let a = days_between(settlement, maturity, basis);
    if a <= 0.0 {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    let ydays = year_days(settlement, basis);
    new_number_formula_arg((redemption - pr) / redemption * (ydays / a))
}

fn duration(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() < 5 || args.len() > 6 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let settlement = match req_serial(&args[0]) {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let maturity = match req_serial(&args[1]) {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let coupon_rate = match req_num(&args[2]) {
        Some(n) if n >= 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let yld = match req_num(&args[3]) {
        Some(n) if n >= 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let freq = match args[4].to_number().as_number() {
        Some(n) if n == 1.0 || n == 2.0 || n == 4.0 => n as i32,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let basis = opt_num(args, 5, 0.0) as i32;
    if basis < 0 || basis > 4 || maturity <= settlement {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    new_number_formula_arg(macaulay_duration(
        settlement,
        maturity,
        coupon_rate,
        yld,
        100.0,
        freq,
        basis,
    ))
}

fn mduration(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() < 5 || args.len() > 6 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let settlement = match req_serial(&args[0]) {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let maturity = match req_serial(&args[1]) {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let coupon_rate = match req_num(&args[2]) {
        Some(n) if n >= 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let yld = match req_num(&args[3]) {
        Some(n) if n >= 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let freq = match args[4].to_number().as_number() {
        Some(n) if n == 1.0 || n == 2.0 || n == 4.0 => n as i32,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let basis = opt_num(args, 5, 0.0) as i32;
    if basis < 0 || basis > 4 || maturity <= settlement {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    let dur = macaulay_duration(settlement, maturity, coupon_rate, yld, 100.0, freq, basis);
    new_number_formula_arg(dur / (1.0 + yld / freq as f64))
}

fn euroconvert(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() < 3 || args.len() > 5 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let amount = match args[0].to_number().as_number() {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let source_currency = args[1].value().trim().to_uppercase();
    let target_currency = args[2].value().trim().to_uppercase();

    let mut full_prec = new_bool_formula_arg(false);
    if args.len() >= 4 {
        full_prec = args[3].to_bool();
        if full_prec.typ == ArgType::Error {
            return full_prec;
        }
    }
    let mut triangulation_prec = new_number_formula_arg(0.0);
    if args.len() == 5 {
        triangulation_prec = args[4].to_number();
        if triangulation_prec.typ != ArgType::Number {
            return triangulation_prec;
        }
    }

    let table: HashMap<&str, (f64, i32)> = [
        ("EUR", (1.0, 2)),
        ("ATS", (13.7603, 2)),
        ("BEF", (40.3399, 0)),
        ("DEM", (1.95583, 2)),
        ("ESP", (166.386, 0)),
        ("FIM", (5.94573, 2)),
        ("FRF", (6.55957, 2)),
        ("IEP", (0.787564, 2)),
        ("ITL", (1936.27, 0)),
        ("LUF", (40.3399, 0)),
        ("NLG", (2.20371, 2)),
        ("PTE", (200.482, 2)),
        ("GRD", (340.75, 2)),
        ("SIT", (239.64, 2)),
        ("MTL", (0.4293, 2)),
        ("CYP", (0.585274, 2)),
        ("SKK", (30.126, 2)),
        ("EEK", (15.6466, 2)),
        ("LVL", (0.702804, 2)),
        ("LTL", (3.4528, 2)),
    ]
    .iter()
    .cloned()
    .collect();

    let source = match table.get(source_currency.as_str()) {
        Some(v) => v,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let target = match table.get(target_currency.as_str()) {
        Some(v) => v,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    if source_currency == target_currency {
        return new_number_formula_arg(amount);
    }
    let res = if source_currency == "EUR" {
        amount * target.0
    } else {
        let mut intermediate = amount / source.0;
        if triangulation_prec.number != 0.0 {
            let ratio = 10f64.powf(triangulation_prec.number);
            intermediate = (intermediate * ratio).round() / ratio;
        }
        intermediate * target.0
    };
    let res = if full_prec.number != 1.0 {
        let ratio = 10f64.powi(target.1);
        (res * ratio).round() / ratio
    } else {
        res
    };
    new_number_formula_arg(res)
}

fn intrate(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() < 4 || args.len() > 5 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let settlement = match req_serial(&args[0]) {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let maturity = match req_serial(&args[1]) {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let investment = match req_num(&args[2]) {
        Some(n) if n > 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let redemption = match req_num(&args[3]) {
        Some(n) if n > 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let basis = opt_num(args, 4, 0.0) as i32;
    if basis < 0 || basis > 4 || maturity <= settlement {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    let a = days_between(settlement, maturity, basis);
    if a <= 0.0 {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    let ydays = year_days(settlement, basis);
    new_number_formula_arg((redemption - investment) / investment * (ydays / a))
}

fn oddfprice_internal(
    settlement: f64,
    maturity: f64,
    issue: f64,
    first_coupon: f64,
    rate: f64,
    yld: f64,
    redemption: f64,
    freq: i32,
    basis: i32,
) -> f64 {
    let (mat_y, mat_m, mat_d) = serial_to_ymd(maturity).unwrap_or((2000, 1, 1));
    let month_days_mat = month_days(mat_y, mat_m);
    let return_last_month = month_days_mat == mat_d;
    let num_months = 12 / freq;
    let num_months_neg = -(num_months as i32);

    let mat = change_month(maturity, num_months_neg, return_last_month);
    let (pcd, _, _) = dates_aggregate(
        mat,
        first_coupon,
        num_months_neg,
        0.0,
        return_last_month,
        |_, _| 0.0,
    );
    if (pcd - first_coupon).abs() > 1e-9 {
        return f64::NAN;
    }

    let e = coup_days_internal(settlement, maturity, freq, basis);
    let n = coup_num_internal(settlement, maturity, freq);
    let m = freq as f64;
    let dfc = coupdays_go(issue, first_coupon, basis);

    if dfc < e {
        let dsc = coupdays_go(settlement, first_coupon, basis);
        let a = coupdays_go(issue, settlement, basis);
        let x = yld / m + 1.0;
        let y = dsc / e;
        let p3 = x.powf(n - 1.0 + y);
        let term1 = redemption / p3;
        let term2 = 100.0 * rate / m * dfc / e / x.powf(y);
        let term3 = aggr_between(2.0, n.floor(), vec![0.0], |acc, index| {
            vec![acc[0] + 100.0 * rate / m / x.powf(index - 1.0 + y)]
        });
        let p2 = rate / m;
        let term4 = a / e * p2 * 100.0;
        return term1 + term2 + term3[0] - term4;
    }

    let nc = coup_num_internal(issue, first_coupon, freq);
    let mut last_coupon = first_coupon;
    let ag = aggr_between(nc.floor(), 1.0, vec![0.0, 0.0], |acc, index| {
        let (ly, lm, ld) = serial_to_ymd(last_coupon).unwrap_or((2000, 1, 1));
        let (ey, em, ed) = add_months(ly, lm, ld, num_months_neg);
        let early_coupon = ymd_to_serial(ey, em, ed).unwrap_or(last_coupon);
        let mut nl = e;
        if basis == 1 {
            nl = coupdays_go(early_coupon, last_coupon, basis);
        }
        let mut dci = coupdays_go(issue, last_coupon, basis);
        if index > 1.0 {
            dci = nl;
        }
        let start_date = issue.max(early_coupon);
        let end_date = settlement.min(last_coupon);
        let a = coupdays_go(start_date, end_date, basis);
        last_coupon = early_coupon;
        let dcnl = acc[0];
        let anl = acc[1];
        vec![dcnl + dci / nl, anl + a / nl]
    });
    let dcnl = ag[0];
    let anl = ag[1];

    let dsc = if basis == 2 || basis == 3 {
        let d = coup_ncd_internal(settlement, first_coupon, freq);
        coupdays_go(settlement, d, basis)
    } else {
        let d = coup_pcd_internal(settlement, first_coupon, freq);
        let a = coupdays_go(d, settlement, basis);
        e - a
    };

    let nq = coup_number(first_coupon, settlement, num_months as i32);
    let n2 = coup_num_internal(first_coupon, maturity, freq);
    let x = yld / m + 1.0;
    let y = dsc / e;
    let p3 = x.powf(y + nq + n2);
    let term1 = redemption / p3;
    let term2 = 100.0 * rate / m * dcnl / x.powf(nq + y);
    let term3 = aggr_between(1.0, n2.floor(), vec![0.0], |acc, index| {
        vec![acc[0] + 100.0 * rate / m / x.powf(index + nq + y)]
    });
    let term4 = 100.0 * rate / m * anl;
    term1 + term2 + term3[0] - term4
}

fn oddfprice(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() < 8 || args.len() > 9 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let settlement = match req_serial(&args[0]) {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let maturity = match req_serial(&args[1]) {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let issue = match req_serial(&args[2]) {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let first_coupon = match req_serial(&args[3]) {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let rate = match req_num(&args[4]) {
        Some(n) if n >= 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let yld = match req_num(&args[5]) {
        Some(n) if n >= 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let redemption = match req_num(&args[6]) {
        Some(n) if n > 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let freq = match args[7].to_number().as_number() {
        Some(n) if n == 1.0 || n == 2.0 || n == 4.0 => n as i32,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let basis = opt_num(args, 8, 0.0) as i32;
    if basis < 0
        || basis > 4
        || maturity <= settlement
        || settlement <= issue
        || first_coupon <= settlement
        || first_coupon >= maturity
    {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    new_number_formula_arg(oddfprice_internal(
        settlement,
        maturity,
        issue,
        first_coupon,
        rate,
        yld,
        redemption,
        freq,
        basis,
    ))
}

fn oddfyield(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() < 8 || args.len() > 9 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let settlement = match req_serial(&args[0]) {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let maturity = match req_serial(&args[1]) {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let issue = match req_serial(&args[2]) {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let first_coupon = match req_serial(&args[3]) {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let rate = match req_num(&args[4]) {
        Some(n) if n >= 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let pr = match req_num(&args[5]) {
        Some(n) if n > 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let redemption = match req_num(&args[6]) {
        Some(n) if n > 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let freq = match args[7].to_number().as_number() {
        Some(n) if n == 1.0 || n == 2.0 || n == 4.0 => n as i32,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let basis = opt_num(args, 8, 0.0) as i32;
    if basis < 0
        || basis > 4
        || maturity <= settlement
        || settlement <= issue
        || first_coupon <= settlement
    {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    let price_fn = |yld: f64| -> f64 {
        oddfprice_internal(
            settlement,
            maturity,
            issue,
            first_coupon,
            rate,
            yld,
            redemption,
            freq,
            basis,
        )
    };
    // Bisection search for yield.
    let mut low = -0.999999;
    let mut high = 0.1;
    if price_fn(low).is_nan() || price_fn(high).is_nan() {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    while price_fn(high) > pr && high < 1e12 {
        high *= 2.0;
    }
    if price_fn(low) < pr || price_fn(high) > pr {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    for _ in 0..100 {
        let mid = (low + high) / 2.0;
        let p = price_fn(mid);
        if (p - pr).abs() < 1e-10 {
            return new_number_formula_arg(mid);
        }
        if p > pr {
            low = mid;
        } else {
            high = mid;
        }
    }
    new_number_formula_arg((low + high) / 2.0)
}

fn oddlprice_internal(
    settlement: f64,
    maturity: f64,
    last_interest: f64,
    rate: f64,
    yld: f64,
    redemption: f64,
    freq: i32,
    basis: i32,
) -> f64 {
    let coupon = redemption * rate / freq as f64;
    let y = yld / freq as f64;
    let normal_days = coupon_period_days(basis, freq);
    let last_period_days = days_between(last_interest, maturity, basis).max(1.0);
    let last_coupon = coupon * last_period_days / normal_days;

    // Coupon dates after last_interest up to maturity.
    let mut dates = Vec::new();
    let (ly, lm, ld) = serial_to_ymd(last_interest).unwrap_or((2000, 1, 1));
    let step = 12 / freq;
    let mut y_c = ly;
    let mut m_c = lm as i32 + step;
    loop {
        while m_c > 12 {
            y_c += 1;
            m_c -= 12;
        }
        let d = ld.min(month_days(y_c, m_c as u32));
        if let Some(s) = ymd_to_serial(y_c, m_c as u32, d) {
            if s > maturity {
                break;
            }
            dates.push(s);
            m_c += step;
        } else {
            break;
        }
    }

    let mut pv = 0.0;
    for &d in &dates {
        let cf = if d == maturity {
            last_coupon + redemption
        } else {
            coupon
        };
        let t = days_between(settlement, d, basis).max(0.0);
        pv += cf / (1.0 + y).powf(t / normal_days);
    }
    let accrued = coupon * days_between(last_interest, settlement, basis) / normal_days;
    pv - accrued
}

fn oddlprice(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() < 7 || args.len() > 8 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let settlement = match req_serial(&args[0]) {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let maturity = match req_serial(&args[1]) {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let last_interest = match req_serial(&args[2]) {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let rate = match req_num(&args[3]) {
        Some(n) if n >= 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let yld = match req_num(&args[4]) {
        Some(n) if n >= 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let redemption = match req_num(&args[5]) {
        Some(n) if n > 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let freq = match args[6].to_number().as_number() {
        Some(n) if n == 1.0 || n == 2.0 || n == 4.0 => n as i32,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let basis = opt_num(args, 7, 0.0) as i32;
    if basis < 0 || basis > 4 || maturity <= settlement || settlement <= last_interest {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    new_number_formula_arg(oddlprice_internal(
        settlement,
        maturity,
        last_interest,
        rate,
        yld,
        redemption,
        freq,
        basis,
    ))
}

fn oddlyield(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() < 7 || args.len() > 8 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let settlement = match req_serial(&args[0]) {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let maturity = match req_serial(&args[1]) {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let last_interest = match req_serial(&args[2]) {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let rate = match req_num(&args[3]) {
        Some(n) if n >= 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let pr = match req_num(&args[4]) {
        Some(n) if n > 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let redemption = match req_num(&args[5]) {
        Some(n) if n > 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let freq = match args[6].to_number().as_number() {
        Some(n) if n == 1.0 || n == 2.0 || n == 4.0 => n as i32,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let basis = opt_num(args, 7, 0.0) as i32;
    if basis < 0 || basis > 4 || maturity <= settlement || settlement <= last_interest {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    let price_fn = |yld: f64| -> f64 {
        oddlprice_internal(
            settlement,
            maturity,
            last_interest,
            rate,
            yld,
            redemption,
            freq,
            basis,
        )
    };
    let mut low = -0.999999;
    let mut high = 0.1;
    if price_fn(low).is_nan() || price_fn(high).is_nan() {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    while price_fn(high) > pr && high < 1e12 {
        high *= 2.0;
    }
    if price_fn(low) < pr || price_fn(high) > pr {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    for _ in 0..100 {
        let mid = (low + high) / 2.0;
        let p = price_fn(mid);
        if (p - pr).abs() < 1e-10 {
            return new_number_formula_arg(mid);
        }
        if p > pr {
            low = mid;
        } else {
            high = mid;
        }
    }
    new_number_formula_arg((low + high) / 2.0)
}

fn price(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() < 6 || args.len() > 7 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let settlement = match req_serial(&args[0]) {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let maturity = match req_serial(&args[1]) {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let rate = match req_num(&args[2]) {
        Some(n) if n >= 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let yld = match req_num(&args[3]) {
        Some(n) if n >= 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let redemption = match req_num(&args[4]) {
        Some(n) if n > 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let freq = match args[5].to_number().as_number() {
        Some(n) if n == 1.0 || n == 2.0 || n == 4.0 => n as i32,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let basis = opt_num(args, 6, 0.0) as i32;
    if basis < 0 || basis > 4 || maturity <= settlement {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    new_number_formula_arg(bond_price(
        settlement, maturity, rate, yld, redemption, freq, basis,
    ))
}

fn pricedisc(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() < 4 || args.len() > 5 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let settlement = match req_serial(&args[0]) {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let maturity = match req_serial(&args[1]) {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let discount = match req_num(&args[2]) {
        Some(n) if n >= 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let redemption = match req_num(&args[3]) {
        Some(n) if n > 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let basis = opt_num(args, 4, 0.0) as i32;
    if basis < 0 || basis > 4 || maturity <= settlement {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    let a = days_between(settlement, maturity, basis);
    let ydays = year_days(settlement, basis);
    new_number_formula_arg(redemption - discount * redemption * (a / ydays))
}

fn pricemat(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() < 5 || args.len() > 6 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let settlement = match req_serial(&args[0]) {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let maturity = match req_serial(&args[1]) {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let issue = match req_serial(&args[2]) {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let rate = match req_num(&args[3]) {
        Some(n) if n >= 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let yld = match req_num(&args[4]) {
        Some(n) if n >= 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let basis = opt_num(args, 5, 0.0) as i32;
    if basis < 0 || basis > 4 || maturity <= settlement || settlement <= issue {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    let dim = days_between(issue, maturity, basis);
    let a = days_between(issue, settlement, basis);
    let dsm = days_between(settlement, maturity, basis);
    let ydays = year_days(issue, basis);
    new_number_formula_arg(
        (100.0 + dim / ydays * rate * 100.0) / (1.0 + dsm / ydays * yld) - a / ydays * rate * 100.0,
    )
}

fn received(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() < 4 || args.len() > 5 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let settlement = match req_serial(&args[0]) {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let maturity = match req_serial(&args[1]) {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let investment = match req_num(&args[2]) {
        Some(n) if n > 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let discount = match req_num(&args[3]) {
        Some(n) if n >= 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let basis = opt_num(args, 4, 0.0) as i32;
    if basis < 0 || basis > 4 || maturity <= settlement {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    let dsm = days_between(settlement, maturity, basis);
    let ydays = year_days(settlement, basis);
    let denom = 1.0 - discount * (dsm / ydays);
    if denom <= 0.0 {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    new_number_formula_arg(investment / denom)
}

fn tbilleq(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 3 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let settlement = match req_serial(&args[0]) {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let maturity = match req_serial(&args[1]) {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let discount = match req_num(&args[2]) {
        Some(n) if n >= 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let dsm = maturity - settlement;
    if dsm <= 0.0 || dsm > 366.0 {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    new_number_formula_arg((365.0 * discount) / (360.0 - discount * dsm))
}

fn tbillprice(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 3 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let settlement = match req_serial(&args[0]) {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let maturity = match req_serial(&args[1]) {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let discount = match req_num(&args[2]) {
        Some(n) if n >= 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let dsm = maturity - settlement;
    if dsm <= 0.0 {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    new_number_formula_arg(100.0 * (1.0 - discount * dsm / 360.0))
}

fn tbillyield(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 3 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let settlement = match req_serial(&args[0]) {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let maturity = match req_serial(&args[1]) {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let pr = match req_num(&args[2]) {
        Some(n) if n > 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let dsm = maturity - settlement;
    if dsm <= 0.0 {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    new_number_formula_arg((100.0 - pr) / pr * (360.0 / dsm))
}

fn vdb(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() < 5 || args.len() > 7 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let cost = match args[0].to_number().as_number() {
        Some(n) if n >= 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let salvage = match args[1].to_number().as_number() {
        Some(n) if n >= 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let life = match args[2].to_number().as_number() {
        Some(n) if n > 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let start_period = match args[3].to_number().as_number() {
        Some(n) if n >= 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let end_period = match args[4].to_number().as_number() {
        Some(n) if n >= start_period && n <= life => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let factor = match args
        .get(5)
        .map(|a| a.to_number().as_number())
        .unwrap_or(Some(2.0))
    {
        Some(n) if n >= 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let no_switch = match args
        .get(6)
        .map(|a| a.to_bool().as_number())
        .unwrap_or(Some(0.0))
    {
        Some(n) => n != 0.0,
        None => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };

    let start_int = start_period.floor();
    let end_int = end_period.ceil();
    if no_switch {
        let mut total = 0.0;
        for i in (start_int as i32 + 1)..=end_int as i32 {
            let mut term = ddb_period_dep(cost, salvage, life, i, factor);
            if i == start_int as i32 + 1 {
                term *= end_period.min(start_int + 1.0) - start_period;
            } else if i as f64 == end_int {
                term *= end_period + 1.0 - end_int;
            }
            total += term;
        }
        return new_number_formula_arg(total);
    }
    let pre = vdb_internal(cost, salvage, life, life, start_period, factor);
    let remaining_cost = cost - pre;
    new_number_formula_arg(vdb_internal(
        remaining_cost,
        salvage,
        life,
        life - start_period,
        end_period - start_period,
        factor,
    ))
}

fn xnpv(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 3 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let rate = match req_num(&args[0]) {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let values: Vec<f64> = args[1]
        .to_list()
        .iter()
        .filter_map(|a| a.to_number().as_number())
        .collect();
    let dates: Vec<f64> = args[2]
        .to_list()
        .iter()
        .filter_map(|a| to_serial(a))
        .collect();
    if values.len() != dates.len() || values.is_empty() {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    if dates.windows(2).any(|w| w[0] >= w[1]) {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    new_number_formula_arg(xnpv_rate(rate, &values, &dates))
}

fn xirr(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() < 2 || args.len() > 3 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let values: Vec<f64> = args[0]
        .to_list()
        .iter()
        .filter_map(|a| a.to_number().as_number())
        .collect();
    let dates: Vec<f64> = args[1]
        .to_list()
        .iter()
        .filter_map(|a| to_serial(a))
        .collect();
    if values.len() != dates.len() || values.len() < 2 {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    if dates.windows(2).any(|w| w[0] >= w[1]) {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    let has_pos = values.iter().any(|&v| v > 0.0);
    let has_neg = values.iter().any(|&v| v < 0.0);
    if !has_pos || !has_neg {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    let guess = opt_num(args, 2, 0.1);
    match xirr_solve(&values, &dates, guess) {
        Some(r) => new_number_formula_arg(r),
        None => new_error_formula_arg(FORMULA_ERROR_NUM),
    }
}

fn bond_yield_fn(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() < 6 || args.len() > 7 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let settlement = match req_serial(&args[0]) {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let maturity = match req_serial(&args[1]) {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let rate = match req_num(&args[2]) {
        Some(n) if n >= 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let pr = match req_num(&args[3]) {
        Some(n) if n > 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let redemption = match req_num(&args[4]) {
        Some(n) if n > 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let freq = match args[5].to_number().as_number() {
        Some(n) if n == 1.0 || n == 2.0 || n == 4.0 => n as i32,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let basis = opt_num(args, 6, 0.0) as i32;
    if basis < 0 || basis > 4 || maturity <= settlement {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    if rate == 0.0 {
        let a = days_between(settlement, maturity, basis);
        let ydays = year_days(settlement, basis);
        return new_number_formula_arg((redemption - pr) / pr * (ydays / a));
    }
    match bond_yield(settlement, maturity, rate, pr, redemption, freq, basis) {
        Some(y) => new_number_formula_arg(y),
        None => new_error_formula_arg(FORMULA_ERROR_NUM),
    }
}

fn yielddisc_fn(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() < 4 || args.len() > 5 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let settlement = match req_serial(&args[0]) {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let maturity = match req_serial(&args[1]) {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let pr = match req_num(&args[2]) {
        Some(n) if n > 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let redemption = match req_num(&args[3]) {
        Some(n) if n > 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let basis = opt_num(args, 4, 0.0) as i32;
    if basis < 0 || basis > 4 || maturity <= settlement {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    let a = days_between(settlement, maturity, basis);
    if a <= 0.0 {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    let ydays = year_days(settlement, basis);
    new_number_formula_arg((redemption - pr) / pr * (ydays / a))
}

fn yieldmat_fn(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() < 5 || args.len() > 6 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let settlement = match req_serial(&args[0]) {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let maturity = match req_serial(&args[1]) {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let issue = match req_serial(&args[2]) {
        Some(n) => n,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let rate = match req_num(&args[3]) {
        Some(n) if n >= 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let pr = match req_num(&args[4]) {
        Some(n) if n > 0.0 => n,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let basis = opt_num(args, 5, 0.0) as i32;
    if basis < 0 || basis > 4 || maturity <= settlement || settlement <= issue {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    let dim = days_between(issue, maturity, basis);
    let a = days_between(issue, settlement, basis);
    let dsm = days_between(settlement, maturity, basis);
    let ydays = year_days(issue, basis);
    let fv = 100.0 * (1.0 + dim / ydays * rate);
    let pv = pr + a / ydays * rate * 100.0;
    if pv <= 0.0 || dsm <= 0.0 {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    new_number_formula_arg((fv / pv - 1.0) * (ydays / dsm))
}
