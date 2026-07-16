//! Date and time formula functions.

use std::collections::HashMap;
use std::sync::LazyLock;

use chrono::{Datelike, Duration, Local, NaiveDate, NaiveDateTime, Timelike};
use regex::Regex;

use crate::calc::arg::*;
use crate::calc::{CalcContext, FormulaFn};
use crate::date as date_util;
use crate::file::File;

pub fn register(m: &mut HashMap<&'static str, FormulaFn>) {
    m.insert("TODAY", today);
    m.insert("NOW", now);
    m.insert("DATE", date);
    m.insert("DATEDIF", datedif);
    m.insert("DATEVALUE", datevalue);
    m.insert("DAY", day);
    m.insert("DAYS", days);
    m.insert("DAYS360", days360);
    m.insert("ISOWEEKNUM", isoweeknum);
    m.insert("EDATE", edate);
    m.insert("EOMONTH", eomonth);
    m.insert("HOUR", hour);
    m.insert("MINUTE", minute);
    m.insert("MONTH", month);
    m.insert("NETWORKDAYS", networkdays);
    m.insert("NETWORKDAYSdotINTL", networkdaysintl);
    m.insert("WORKDAY", workday);
    m.insert("WORKDAYdotINTL", workdayintl);
    m.insert("YEAR", year);
    m.insert("YEARFRAC", yearfrac);
    m.insert("SECOND", second);
    m.insert("TIME", time);
    m.insert("TIMEVALUE", timevalue);
    m.insert("WEEKDAY", weekday);
    m.insert("WEEKNUM", weeknum);
}

fn date_1904(file: &File) -> bool {
    file.workbook_reader()
        .ok()
        .and_then(|wb| wb.workbook_pr.as_ref().and_then(|p| p.date1904))
        .unwrap_or(false)
}

fn today(ctx: &CalcContext, _args: &[FormulaArg]) -> FormulaArg {
    let d = Local::now().date_naive();
    new_number_formula_arg(date_util::date_to_excel_serial(d, date_1904(ctx.file)))
}

fn now(ctx: &CalcContext, _args: &[FormulaArg]) -> FormulaArg {
    let dt = Local::now().naive_local();
    new_number_formula_arg(date_util::datetime_to_excel_serial(dt, date_1904(ctx.file)))
}

// ------------------------------------------------------------------
// Date/time string parsing helpers
// ------------------------------------------------------------------

fn month_name_to_num(name: &str) -> Option<u32> {
    match name.to_lowercase().as_str() {
        "january" | "jan" => Some(1),
        "february" | "feb" => Some(2),
        "march" | "mar" => Some(3),
        "april" | "apr" => Some(4),
        "may" => Some(5),
        "june" | "jun" => Some(6),
        "july" | "jul" => Some(7),
        "august" | "aug" => Some(8),
        "september" | "sep" => Some(9),
        "october" | "oct" => Some(10),
        "november" | "nov" => Some(11),
        "december" | "dec" => Some(12),
        _ => None,
    }
}

fn date_only_regexes() -> &'static [Regex; 4] {
    static RE: LazyLock<[Regex; 4]> = LazyLock::new(|| {
        [
            Regex::new(r"(?i)^(\d{1,4})/(\d{1,4})/(\d{1,4})$").unwrap(),
            Regex::new(r"(?i)^([a-z]+) (\d{1,2}), (\d{1,4})$").unwrap(),
            Regex::new(r"(?i)^(\d{1,4})-(\d{1,4})-(\d{1,4})$").unwrap(),
            Regex::new(r"(?i)^(\d{1,2})-([a-z]+)-(\d{1,4})$").unwrap(),
        ]
    });
    &RE
}

fn time_only_regexes() -> &'static [Regex; 4] {
    static RE: LazyLock<[Regex; 4]> = LazyLock::new(|| {
        [
            Regex::new(r"(?i)^(\d+)\s*(am|pm)$").unwrap(),
            Regex::new(r"(?i)^(\d+):(\d+)(\s*(am|pm))?$").unwrap(),
            Regex::new(r"(?i)^(\d+):(\d+\.\d+)(\s*(am|pm))?$").unwrap(),
            Regex::new(r"(?i)^(\d+):(\d+):(\d+(\.\d+)?)(\s*(am|pm))?$").unwrap(),
        ]
    });
    &RE
}

fn time_suffix_regex() -> &'static Regex {
    static RE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"(?i)\s+(\d+\s*(am|pm)|\d+:\d+:\d+(\.\d+)?(\s*(am|pm))?|\d+:\d+\.\d+(\s*(am|pm))?|\d+:\d+(\s*(am|pm))?)$").unwrap()
    });
    &RE
}

fn date_prefix_regex() -> &'static Regex {
    static RE: LazyLock<Regex> = LazyLock::new(|| {
        let month = r"(january|february|march|april|may|june|july|august|september|october|november|december|jan|feb|mar|apr|jun|jul|aug|sep|oct|nov|dec)";
        let s = format!(
            r"(?i)^(\d{{1,4}}/\d{{1,4}}/\d{{1,4}}|{} \d{{1,2}}, \d{{1,4}}|\d{{1,4}}-\d{{1,4}}-\d{{1,4}}|\d{{1,2}}-{}-\d{{1,4}})\s+",
            month, month
        );
        Regex::new(&s).unwrap()
    });
    &RE
}

fn is_date_only_fmt(s: &str) -> bool {
    date_only_regexes().iter().any(|re| re.is_match(s))
}

fn is_time_only_fmt(s: &str) -> bool {
    if time_only_regexes().iter().any(|re| re.is_match(s)) {
        return true;
    }
    let re_prefix = date_prefix_regex();
    if let Some(m) = re_prefix.find(s) {
        let rest = &s[m.end()..];
        if time_only_regexes().iter().any(|re| re.is_match(rest)) {
            return true;
        }
    }
    false
}

fn parse_date_components(caps: &regex::Captures, pattern_idx: usize) -> Option<(i32, u32, u32)> {
    match pattern_idx {
        0 => {
            // mm/dd/yy
            let m = caps.get(1)?.as_str().parse::<u32>().ok()?;
            let d = caps.get(2)?.as_str().parse::<u32>().ok()?;
            let y = caps.get(3)?.as_str().parse::<i32>().ok()?;
            let y = date_util::format_year(y);
            Some((y, m, d))
        }
        1 => {
            // month dd, yy
            let mon = caps.get(1)?.as_str();
            let m = month_name_to_num(mon)?;
            let d = caps.get(2)?.as_str().parse::<u32>().ok()?;
            let y = caps.get(3)?.as_str().parse::<i32>().ok()?;
            let y = date_util::format_year(y);
            Some((y, m, d))
        }
        2 => {
            // yy-mm-dd or mm-dd-yy
            let a = caps.get(1)?.as_str().parse::<i32>().ok()?;
            let b = caps.get(2)?.as_str().parse::<u32>().ok()?;
            let c = caps.get(3)?.as_str().parse::<u32>().ok()?;
            if a >= 1900 && a < 10000 {
                Some((a, b, c))
            } else if a > 0 && a < 13 {
                Some((date_util::format_year(c as i32), a as u32, b))
            } else {
                None
            }
        }
        3 => {
            // dd-mon-yy
            let d = caps.get(1)?.as_str().parse::<u32>().ok()?;
            let mon = caps.get(2)?.as_str();
            let m = month_name_to_num(mon)?;
            let y = caps.get(3)?.as_str().parse::<i32>().ok()?;
            let y = date_util::format_year(y);
            Some((y, m, d))
        }
        _ => None,
    }
}

fn str_to_date(s: &str) -> (i32, u32, u32, bool, FormulaArg) {
    let trimmed = s.trim();
    let mut time_is_empty = true;
    let date_part = if let Some(m) = time_suffix_regex().find(trimmed) {
        time_is_empty = false;
        &trimmed[..m.start()]
    } else {
        trimmed
    };

    for (idx, re) in date_only_regexes().iter().enumerate() {
        if let Some(caps) = re.captures(date_part) {
            if let Some((y, m, d)) = parse_date_components(&caps, idx) {
                if date_util::validate_date(y, m, d) {
                    return (y, m, d, time_is_empty, new_empty_formula_arg());
                }
            }
            return (0, 0, 0, false, new_error_formula_arg(FORMULA_ERROR_VALUE));
        }
    }
    (0, 0, 0, false, new_error_formula_arg(FORMULA_ERROR_VALUE))
}

fn str_to_time(s: &str) -> (i32, i32, f64, bool, bool, FormulaArg) {
    let trimmed = s.trim();
    let mut date_is_empty = true;
    let time_part = if let Some(m) = date_prefix_regex().find(trimmed) {
        date_is_empty = false;
        &trimmed[m.end()..]
    } else {
        trimmed
    };

    for (idx, re) in time_only_regexes().iter().enumerate() {
        if let Some(caps) = re.captures(time_part) {
            let mut hours = 0i32;
            let mut minutes = 0i32;
            let mut seconds = 0.0f64;
            let mut am = false;
            let mut pm = false;

            match idx {
                0 => {
                    // hh am/pm
                    hours = caps.get(1).unwrap().as_str().parse::<i32>().unwrap();
                    let ampm = caps.get(2).unwrap().as_str().to_lowercase();
                    am = ampm == "am";
                    pm = ampm == "pm";
                }
                1 => {
                    // hh:mm
                    hours = caps.get(1).unwrap().as_str().parse::<i32>().unwrap();
                    minutes = caps.get(2).unwrap().as_str().parse::<i32>().unwrap();
                    if let Some(ampm) = caps.get(4) {
                        let ampm = ampm.as_str().to_lowercase();
                        am = ampm == "am";
                        pm = ampm == "pm";
                    }
                }
                2 => {
                    // mm:ss
                    minutes = caps.get(1).unwrap().as_str().parse::<i32>().unwrap();
                    seconds = caps.get(2).unwrap().as_str().parse::<f64>().unwrap();
                    if let Some(ampm) = caps.get(4) {
                        let ampm = ampm.as_str().to_lowercase();
                        am = ampm == "am";
                        pm = ampm == "pm";
                    }
                }
                3 => {
                    // hh:mm:ss
                    hours = caps.get(1).unwrap().as_str().parse::<i32>().unwrap();
                    minutes = caps.get(2).unwrap().as_str().parse::<i32>().unwrap();
                    seconds = caps.get(3).unwrap().as_str().parse::<f64>().unwrap();
                    if let Some(ampm) = caps.get(6) {
                        let ampm = ampm.as_str().to_lowercase();
                        am = ampm == "am";
                        pm = ampm == "pm";
                    }
                }
                _ => {}
            }

            if minutes >= 60 {
                return (
                    0,
                    0,
                    0.0,
                    false,
                    false,
                    new_error_formula_arg(FORMULA_ERROR_VALUE),
                );
            }
            if am || pm {
                if hours > 12 || seconds >= 60.0 {
                    return (
                        0,
                        0,
                        0.0,
                        false,
                        false,
                        new_error_formula_arg(FORMULA_ERROR_VALUE),
                    );
                } else if hours == 12 {
                    hours = 0;
                }
            } else if hours >= 24 || seconds >= 10000.0 {
                return (
                    0,
                    0,
                    0.0,
                    false,
                    false,
                    new_error_formula_arg(FORMULA_ERROR_VALUE),
                );
            }
            return (
                hours,
                minutes,
                seconds,
                pm,
                date_is_empty,
                new_empty_formula_arg(),
            );
        }
    }
    (
        0,
        0,
        0.0,
        false,
        false,
        new_error_formula_arg(FORMULA_ERROR_VALUE),
    )
}

// ------------------------------------------------------------------
// Common date argument conversion helpers
// ------------------------------------------------------------------

fn datevalue_impl(arg: &FormulaArg) -> FormulaArg {
    let text = arg.value().to_lowercase();
    if !is_date_only_fmt(&text) {
        let (_, _, _, _, _, err) = str_to_time(&text);
        if err.typ == ArgType::Error {
            return err;
        }
    }
    let (y, m, d, _, err) = str_to_date(&text);
    if err.typ == ArgType::Error {
        return err;
    }
    let dt = NaiveDate::from_ymd_opt(y, m, d)
        .unwrap()
        .and_hms_opt(0, 0, 0)
        .unwrap();
    new_number_formula_arg(date_util::datetime_to_excel_serial(dt, false))
}

fn to_excel_date_arg(arg: &FormulaArg) -> FormulaArg {
    let num = arg.to_number();
    if num.typ != ArgType::Number {
        let text = arg.value().to_lowercase();
        if !is_date_only_fmt(&text) {
            let (_, _, _, _, _, err) = str_to_time(&text);
            if err.typ == ArgType::Error {
                return err;
            }
        }
        let (y, m, d, _, err) = str_to_date(&text);
        if err.typ == ArgType::Error {
            return err;
        }
        let dt = NaiveDate::from_ymd_opt(y, m, d)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();
        let serial = date_util::datetime_to_excel_serial(dt, false);
        return new_number_formula_arg(serial);
    }
    if arg.number < 0.0 {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    num
}

fn prepare_data_value_args(args: &[FormulaArg], n: usize) -> FormulaArg {
    let mut values = Vec::new();
    for i in 0..n {
        let arg = match args.get(i) {
            Some(a) => a,
            None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
        };
        let value = match arg.typ {
            ArgType::Number => arg.clone(),
            ArgType::String => {
                let num = arg.to_number();
                if num.typ == ArgType::Number {
                    num
                } else {
                    let dv = datevalue_impl(arg);
                    if dv.typ == ArgType::Error {
                        return new_error_formula_arg(FORMULA_ERROR_VALUE);
                    }
                    dv
                }
            }
            _ => return new_error_formula_arg(FORMULA_ERROR_VALUE),
        };
        values.push(value);
    }
    new_list_formula_arg(values)
}

fn prepare_holidays(arg: &FormulaArg) -> Vec<i32> {
    let mut holidays = Vec::new();
    for item in arg.to_list() {
        let num = to_excel_date_arg(&item);
        if num.typ == ArgType::Number {
            holidays.push(num.number.ceil() as i32);
        }
    }
    holidays.sort_unstable();
    holidays
}

// ------------------------------------------------------------------
// Workday / weekend helpers
// ------------------------------------------------------------------

fn gen_weekend_mask(weekend: i32) -> Option<Vec<u8>> {
    let masks: &[usize] = match weekend {
        1 => &[5, 6],
        2 => &[6, 0],
        3 => &[0, 1],
        4 => &[1, 2],
        5 => &[2, 3],
        6 => &[3, 4],
        7 => &[4, 5],
        11 => &[6],
        12 => &[0],
        13 => &[1],
        14 => &[2],
        15 => &[3],
        16 => &[4],
        17 => &[5],
        _ => return None,
    };
    let mut mask = vec![0u8; 7];
    for &idx in masks {
        mask[idx] = 1;
    }
    Some(mask)
}

fn is_workday(weekend_mask: &[u8], serial: f64) -> bool {
    let dt = date_util::Date::serial_to_datetime(serial, false).unwrap_or_default();
    let mut weekday = dt.weekday().num_days_from_sunday() as usize;
    if weekday == 0 {
        weekday = 7;
    }
    weekend_mask[weekday - 1] == 0
}

fn prepare_workday(weekend: &FormulaArg) -> (Option<Vec<u8>>, i32) {
    let num = weekend.to_number();
    let mut mask = None;
    if weekend.typ == ArgType::String && weekend.string.len() == 7 {
        let mut m = Vec::new();
        let mut valid = true;
        for c in weekend.string.chars() {
            if c == '0' {
                m.push(0);
            } else if c == '1' {
                m.push(1);
            } else {
                valid = false;
                break;
            }
        }
        if valid {
            mask = Some(m);
        }
    } else if num.typ == ArgType::Number {
        mask = gen_weekend_mask(num.number as i32);
    }
    let workdays_per_week = mask
        .as_ref()
        .map_or(0, |m| m.iter().filter(|&&x| x == 0).count() as i32);
    (mask, workdays_per_week)
}

fn workday_intl(
    end_date: i32,
    sign: i32,
    holidays: &[i32],
    weekend_mask: &[u8],
    start_date: f64,
) -> i32 {
    let mut end_date = end_date;
    for &holiday in holidays {
        if sign > 0 {
            if holiday > end_date {
                break;
            }
        } else if holiday < end_date {
            break;
        }

        if sign > 0 {
            if (holiday as f64) > start_date.ceil() {
                if is_workday(weekend_mask, holiday as f64) {
                    end_date += sign;
                    while !is_workday(weekend_mask, end_date as f64) {
                        end_date += sign;
                    }
                }
            }
        } else if (holiday as f64) < start_date.ceil() {
            if is_workday(weekend_mask, holiday as f64) {
                end_date += sign;
                while !is_workday(weekend_mask, end_date as f64) {
                    end_date += sign;
                }
            }
        }
    }
    end_date
}

// ------------------------------------------------------------------
// Date functions
// ------------------------------------------------------------------

fn date(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 3 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let y = args[0].to_number();
    let m = args[1].to_number();
    let d = args[2].to_number();
    if y.typ != ArgType::Number || m.typ != ArgType::Number || d.typ != ArgType::Number {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }

    // Match Go's time.Date behavior: out-of-range month/day values are
    // normalized by rolling over into adjacent months/years.
    let year = y.number as i32;
    let month = m.number as i32;
    let day = d.number as i64;

    let year_offset = (month - 1).div_euclid(12);
    let month_in_year = (month - 1).rem_euclid(12) + 1;

    let Some(first_of_month) = NaiveDate::from_ymd_opt(year + year_offset, month_in_year as u32, 1)
    else {
        return new_number_formula_arg(0.0);
    };
    let Some(dt) = first_of_month.checked_add_signed(Duration::days(day - 1)) else {
        return new_number_formula_arg(0.0);
    };
    let dt = dt.and_hms_opt(0, 0, 0).unwrap_or_default();
    new_number_formula_arg(date_util::datetime_to_excel_serial(dt, false))
}

fn datedif(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 3 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let start_arg = args[0].to_number();
    let end_arg = args[1].to_number();
    if start_arg.typ != ArgType::Number || end_arg.typ != ArgType::Number {
        return start_arg;
    }
    if start_arg.number > end_arg.number {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    if start_arg.number == end_arg.number {
        return new_number_formula_arg(0.0);
    }
    let unit = args[2].value().to_lowercase();
    let start_dt = date_util::Date::serial_to_datetime(start_arg.number, false).unwrap_or_default();
    let end_dt = date_util::Date::serial_to_datetime(end_arg.number, false).unwrap_or_default();
    let (sy, sm, sd) = (
        start_dt.year(),
        start_dt.month() as i32,
        start_dt.day() as i32,
    );
    let (ey, em, ed) = (end_dt.year(), end_dt.month() as i32, end_dt.day() as i32);

    let diff = match unit.as_str() {
        "y" => {
            let mut diff = ey - sy;
            if em < sm || (em == sm && ed < sd) {
                diff -= 1;
            }
            diff as f64
        }
        "m" => {
            let mut y_diff = ey - sy;
            let mut m_diff = em - sm;
            if ed < sd {
                m_diff -= 1;
            }
            if m_diff < 0 {
                y_diff -= 1;
                m_diff += 12;
            }
            (y_diff * 12 + m_diff) as f64
        }
        "d" => end_arg.number - start_arg.number,
        "md" => {
            let mut sm_md = em;
            if ed < sd {
                sm_md -= 1;
            }
            let dt = NaiveDate::from_ymd_opt(ey, sm_md as u32, sd as u32)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap();
            end_arg.number - date_util::datetime_to_excel_serial(dt, false)
        }
        "ym" => {
            let mut diff = em - sm;
            if ed < sd {
                diff -= 1;
            }
            if diff < 0 {
                diff += 12;
            }
            diff as f64
        }
        "yd" => {
            let mut sy_yd = sy;
            if em < sm || (em == sm && ed < sd) {
                sy_yd += 1;
            }
            let s_dt = NaiveDate::from_ymd_opt(sy_yd, em as u32, ed as u32)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap();
            let e_dt = NaiveDate::from_ymd_opt(sy, sm as u32, sd as u32)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap();
            date_util::datetime_to_excel_serial(s_dt, false)
                - date_util::datetime_to_excel_serial(e_dt, false)
        }
        _ => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    new_number_formula_arg(diff)
}

fn datevalue(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    datevalue_impl(&args[0])
}

fn day(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let arg = &args[0];
    let num = arg.to_number();
    if num.typ != ArgType::Number {
        let text = arg.value().to_lowercase();
        if !is_date_only_fmt(&text) {
            let (_, _, _, _, _, err) = str_to_time(&text);
            if err.typ == ArgType::Error {
                return err;
            }
        }
        let (_, _, d, _, err) = str_to_date(&text);
        if err.typ == ArgType::Error {
            return err;
        }
        return new_number_formula_arg(d as f64);
    }
    if num.number < 0.0 {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    if num.number <= 60.0 {
        return new_number_formula_arg(num.number % 31.0);
    }
    let dt = date_util::Date::serial_to_datetime(num.number, false).unwrap_or_default();
    new_number_formula_arg(dt.day() as f64)
}

fn days(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let prepared = prepare_data_value_args(args, 2);
    if prepared.typ != ArgType::List {
        return prepared;
    }
    let end = &prepared.list[0];
    let start = &prepared.list[1];
    new_number_formula_arg(end.number - start.number)
}

fn days360(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() < 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    if args.len() > 3 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let start_date = to_excel_date_arg(&args[0]);
    if start_date.typ != ArgType::Number {
        return start_date;
    }
    let end_date = to_excel_date_arg(&args[1]);
    if end_date.typ != ArgType::Number {
        return end_date;
    }
    let start_dt =
        date_util::Date::serial_to_datetime(start_date.number, false).unwrap_or_default();
    let end_dt = date_util::Date::serial_to_datetime(end_date.number, false).unwrap_or_default();
    let (sy, sm, mut sd) = (
        start_dt.year(),
        start_dt.month() as i32,
        start_dt.day() as i32,
    );
    let (ey, mut em, mut ed) = (end_dt.year(), end_dt.month() as i32, end_dt.day() as i32);

    let method = if args.len() > 2 {
        args[2].to_bool()
    } else {
        new_bool_formula_arg(false)
    };
    if method.typ != ArgType::Number {
        return method;
    }

    if method.number == 1.0 {
        if sd == 31 {
            sd -= 1;
        }
        if ed == 31 {
            ed -= 1;
        }
    } else {
        if date_util::get_days_in_month(sy, sm as u32) == sd as u32 {
            sd = 30;
        }
        if ed > 30 {
            if sd < 30 {
                em += 1;
                ed = 1;
            } else {
                ed = 30;
            }
        }
    }
    new_number_formula_arg((360 * (ey - sy) + 30 * (em - sm) + (ed - sd)) as f64)
}

fn isoweeknum(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let arg = &args[0];
    let num = arg.to_number();
    let week_num = if num.typ != ArgType::Number {
        let text = arg.value().to_lowercase();
        if !is_date_only_fmt(&text) {
            let (_, _, _, _, _, err) = str_to_time(&text);
            if err.typ == ArgType::Error {
                return err;
            }
        }
        let (y, m, d, _, err) = str_to_date(&text);
        if err.typ == ArgType::Error {
            return err;
        }
        let dt = NaiveDate::from_ymd_opt(y, m, d).unwrap();
        dt.iso_week().week() as i32
    } else {
        if num.number < 0.0 {
            return new_error_formula_arg(FORMULA_ERROR_NUM);
        }
        let dt = date_util::Date::serial_to_datetime(num.number, false).unwrap_or_default();
        dt.iso_week().week() as i32
    };
    new_number_formula_arg(week_num as f64)
}

fn edate(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let arg = &args[0];
    let num = arg.to_number();
    let date_time = if num.typ != ArgType::Number {
        let text = arg.value().to_lowercase();
        if !is_date_only_fmt(&text) {
            let (_, _, _, _, _, err) = str_to_time(&text);
            if err.typ == ArgType::Error {
                return err;
            }
        }
        let (y, m, d, _, err) = str_to_date(&text);
        if err.typ == ArgType::Error {
            return err;
        }
        NaiveDate::from_ymd_opt(y, m, d)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap()
    } else {
        if num.number < 0.0 {
            return new_error_formula_arg(FORMULA_ERROR_NUM);
        }
        date_util::Date::serial_to_datetime(num.number, false).unwrap_or_default()
    };

    let months = args[1].to_number();
    if months.typ != ArgType::Number {
        return months;
    }
    let mut y = date_time.year();
    let mut d = date_time.day() as i32;
    let mut m = date_time.month() as i32 + months.number as i32;
    if months.number < 0.0 {
        y -= f64::ceil((-1.0 * m as f64) / 12.0) as i32;
    }
    if months.number > 11.0 {
        y += (m as f64 / 12.0).floor() as i32;
    }
    m = m % 12;
    if m < 0 {
        m += 12;
    }
    if m == 0 {
        m = 12;
        y -= 1;
    }
    if d > 28 {
        let days = date_util::get_days_in_month(y, m as u32) as i32;
        if d > days {
            d = days;
        }
    }
    let dt = NaiveDate::from_ymd_opt(y, m as u32, d as u32)
        .unwrap()
        .and_hms_opt(0, 0, 0)
        .unwrap();
    new_number_formula_arg(date_util::datetime_to_excel_serial(dt, false))
}

fn eomonth(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let arg = &args[0];
    let num = arg.to_number();
    let date_time = if num.typ != ArgType::Number {
        let text = arg.value().to_lowercase();
        if !is_date_only_fmt(&text) {
            let (_, _, _, _, _, err) = str_to_time(&text);
            if err.typ == ArgType::Error {
                return err;
            }
        }
        let (y, m, d, _, err) = str_to_date(&text);
        if err.typ == ArgType::Error {
            return err;
        }
        NaiveDate::from_ymd_opt(y, m, d)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap()
    } else {
        if num.number < 0.0 {
            return new_error_formula_arg(FORMULA_ERROR_NUM);
        }
        date_util::Date::serial_to_datetime(num.number, false).unwrap_or_default()
    };

    let months = args[1].to_number();
    if months.typ != ArgType::Number {
        return months;
    }
    let mut y = date_time.year();
    let mut m = date_time.month() as i32 + months.number as i32 - 1;
    if m < 0 {
        y -= f64::ceil((-1.0 * m as f64) / 12.0) as i32;
    }
    if m > 11 {
        y += (m as f64 / 12.0).floor() as i32;
    }
    m = m % 12;
    if m < 0 {
        m += 12;
    }
    let days = date_util::get_days_in_month(y, (m + 1) as u32);
    let dt = NaiveDate::from_ymd_opt(y, (m + 1) as u32, days)
        .unwrap()
        .and_hms_opt(0, 0, 0)
        .unwrap();
    new_number_formula_arg(date_util::datetime_to_excel_serial(dt, false))
}

fn hour(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let arg = &args[0];
    let num = arg.to_number();
    if num.typ != ArgType::Number {
        let text = arg.value().to_lowercase();
        if !is_time_only_fmt(&text) {
            let (_, _, _, _, err) = str_to_date(&text);
            if err.typ == ArgType::Error {
                return err;
            }
        }
        let (mut h, _, _, pm, _, err) = str_to_time(&text);
        if err.typ == ArgType::Error {
            return err;
        }
        if pm {
            h += 12;
        }
        return new_number_formula_arg(h as f64);
    }
    if num.number < 0.0 {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    let dt = date_util::Date::serial_to_datetime(num.number, false).unwrap_or_default();
    new_number_formula_arg(dt.hour() as f64)
}

fn minute(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let arg = &args[0];
    let num = arg.to_number();
    if num.typ != ArgType::Number {
        let text = arg.value().to_lowercase();
        if !is_time_only_fmt(&text) {
            let (_, _, _, _, err) = str_to_date(&text);
            if err.typ == ArgType::Error {
                return err;
            }
        }
        let (_, m, _, _, _, err) = str_to_time(&text);
        if err.typ == ArgType::Error {
            return err;
        }
        return new_number_formula_arg(m as f64);
    }
    if num.number < 0.0 {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    let dt = date_util::Date::serial_to_datetime(num.number, false).unwrap_or_default();
    new_number_formula_arg(dt.minute() as f64)
}

fn month(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let arg = &args[0];
    let num = arg.to_number();
    if num.typ != ArgType::Number {
        let text = arg.value().to_lowercase();
        if !is_date_only_fmt(&text) {
            let (_, _, _, _, _, err) = str_to_time(&text);
            if err.typ == ArgType::Error {
                return err;
            }
        }
        let (_, m, _, _, err) = str_to_date(&text);
        if err.typ == ArgType::Error {
            return err;
        }
        return new_number_formula_arg(m as f64);
    }
    if num.number < 0.0 {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    let dt = date_util::Date::serial_to_datetime(num.number, false).unwrap_or_default();
    new_number_formula_arg(dt.month() as f64)
}

fn networkdays(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() < 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    if args.len() > 3 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let mut intl_args = Vec::new();
    intl_args.push(args[0].clone());
    intl_args.push(args[1].clone());
    intl_args.push(new_number_formula_arg(1.0));
    if args.len() == 3 {
        intl_args.push(args[2].clone());
    }
    networkdaysintl(_ctx, &intl_args)
}

fn networkdaysintl(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() < 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    if args.len() > 4 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let start_date = to_excel_date_arg(&args[0]);
    if start_date.typ != ArgType::Number {
        return start_date;
    }
    let end_date = to_excel_date_arg(&args[1]);
    if end_date.typ != ArgType::Number {
        return end_date;
    }
    let default_weekend = new_number_formula_arg(1.0);
    let weekend = args.get(2).unwrap_or(&default_weekend);
    let holidays = if args.len() == 4 {
        prepare_holidays(&args[3])
    } else {
        Vec::new()
    };
    let (weekend_mask, workdays_per_week) = prepare_workday(weekend);
    if workdays_per_week == 0 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let weekend_mask = weekend_mask.unwrap();

    let mut sign = 1;
    let (mut start, mut end) = (start_date.number, end_date.number);
    if start > end {
        sign = -1;
        std::mem::swap(&mut start, &mut end);
    }
    let offset = end - start;
    let mut count = (offset / 7.0).floor() * workdays_per_week as f64;
    let mut days_mod = (offset as i32) % 7;
    while days_mod >= 0 {
        if is_workday(&weekend_mask, end - days_mod as f64) {
            count += 1.0;
        }
        days_mod -= 1;
    }
    for holiday in holidays {
        let h = holiday as f64;
        if is_workday(&weekend_mask, h) && h >= start && h <= end {
            count -= 1.0;
        }
    }
    new_number_formula_arg(sign as f64 * count)
}

fn workday(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() < 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    if args.len() > 3 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let mut intl_args = Vec::new();
    intl_args.push(args[0].clone());
    intl_args.push(args[1].clone());
    intl_args.push(new_number_formula_arg(1.0));
    if args.len() == 3 {
        intl_args.push(args[2].clone());
    }
    workdayintl(_ctx, &intl_args)
}

fn workdayintl(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() < 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    if args.len() > 4 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let start_date = to_excel_date_arg(&args[0]);
    if start_date.typ != ArgType::Number {
        return start_date;
    }
    let days = args[1].to_number();
    if days.typ != ArgType::Number {
        return days;
    }
    let default_weekend = new_number_formula_arg(1.0);
    let weekend = args.get(2).unwrap_or(&default_weekend);
    let holidays = if args.len() == 4 {
        prepare_holidays(&args[3])
    } else {
        Vec::new()
    };
    if days.number == 0.0 {
        return new_number_formula_arg(start_date.number.ceil());
    }
    let (weekend_mask, workdays_per_week) = prepare_workday(weekend);
    if workdays_per_week == 0 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let weekend_mask = weekend_mask.unwrap();

    let sign = if days.number < 0.0 { -1 } else { 1 };
    let offset = (days.number as i32) / workdays_per_week;
    let mut days_mod = (days.number as i32) % workdays_per_week;
    let mut end_date = (start_date.number.ceil() as i32) + offset * 7;

    if days_mod == 0 {
        while !is_workday(&weekend_mask, end_date as f64) {
            end_date -= sign;
        }
    } else {
        while days_mod != 0 {
            end_date += sign;
            if is_workday(&weekend_mask, end_date as f64) {
                if days_mod < 0 {
                    days_mod += 1;
                    continue;
                }
                days_mod -= 1;
            }
        }
    }
    new_number_formula_arg(
        workday_intl(end_date, sign, &holidays, &weekend_mask, start_date.number) as f64,
    )
}

fn year(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let arg = &args[0];
    let num = arg.to_number();
    if num.typ != ArgType::Number {
        let text = arg.value().to_lowercase();
        if !is_date_only_fmt(&text) {
            let (_, _, _, _, _, err) = str_to_time(&text);
            if err.typ == ArgType::Error {
                return err;
            }
        }
        let (y, _, _, _, err) = str_to_date(&text);
        if err.typ == ArgType::Error {
            return err;
        }
        return new_number_formula_arg(y as f64);
    }
    if num.number < 0.0 {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    let dt = date_util::Date::serial_to_datetime(num.number, false).unwrap_or_default();
    new_number_formula_arg(dt.year() as f64)
}

fn yearfrac_basis_cond(sy: i32, sm: i32, sd: i32, ey: i32, em: i32, ed: i32) -> bool {
    (date_util::is_leap_year(sy) && (sm < 2 || (sm == 2 && sd <= 29)))
        || (date_util::is_leap_year(ey) && (em > 2 || (em == 2 && ed == 29)))
}

fn yearfrac_basis0(start_date: f64, end_date: f64) -> (f64, f64) {
    let start_dt = date_util::Date::serial_to_datetime(start_date, false).unwrap_or_default();
    let end_dt = date_util::Date::serial_to_datetime(end_date, false).unwrap_or_default();
    let (sy, sm, sd) = (
        start_dt.year(),
        start_dt.month() as i32,
        start_dt.day() as i32,
    );
    let (ey, em, ed) = (end_dt.year(), end_dt.month() as i32, end_dt.day() as i32);
    let mut sd = sd;
    let mut ed = ed;
    if sd == 31 {
        sd -= 1;
    }
    if sd == 30 && ed == 31 {
        ed -= 1;
    } else {
        let leap = date_util::is_leap_year(sy);
        if sm == 2 && ((leap && sd == 29) || (!leap && sd == 28)) {
            sd = 30;
            let leap = date_util::is_leap_year(ey);
            if em == 2 && ((leap && ed == 29) || (!leap && ed == 28)) {
                ed = 30;
            }
        }
    }
    let day_diff = ((ey - sy) * 360 + (em - sm) * 30 + (ed - sd)) as f64;
    (day_diff, 360.0)
}

fn yearfrac_basis1(start_date: f64, end_date: f64) -> (f64, f64) {
    let start_dt = date_util::Date::serial_to_datetime(start_date, false).unwrap_or_default();
    let end_dt = date_util::Date::serial_to_datetime(end_date, false).unwrap_or_default();
    let (sy, sm, sd) = (
        start_dt.year(),
        start_dt.month() as i32,
        start_dt.day() as i32,
    );
    let (ey, em, ed) = (end_dt.year(), end_dt.month() as i32, end_dt.day() as i32);
    let day_diff = end_date - start_date;
    let is_year_different = sy != ey;
    let days_in_year = if is_year_different && (ey != sy + 1 || sm < em || (sm == em && sd < ed)) {
        let mut day_count = 0;
        for y in sy..=ey {
            day_count += if date_util::is_leap_year(y) { 366 } else { 365 };
        }
        day_count as f64 / (ey - sy + 1) as f64
    } else if !is_year_different && date_util::is_leap_year(sy) {
        366.0
    } else if is_year_different && yearfrac_basis_cond(sy, sm, sd, ey, em, ed) {
        366.0
    } else {
        365.0
    };
    (day_diff, days_in_year)
}

fn yearfrac_basis4(start_date: f64, end_date: f64) -> (f64, f64) {
    let start_dt = date_util::Date::serial_to_datetime(start_date, false).unwrap_or_default();
    let end_dt = date_util::Date::serial_to_datetime(end_date, false).unwrap_or_default();
    let (sy, sm, mut sd) = (
        start_dt.year(),
        start_dt.month() as i32,
        start_dt.day() as i32,
    );
    let (ey, em, mut ed) = (end_dt.year(), end_dt.month() as i32, end_dt.day() as i32);
    if sd == 31 {
        sd -= 1;
    }
    if ed == 31 {
        ed -= 1;
    }
    let day_diff = ((ey - sy) * 360 + (em - sm) * 30 + (ed - sd)) as f64;
    (day_diff, 360.0)
}

fn yearfrac_impl(start_date: f64, end_date: f64, basis: i32) -> FormulaArg {
    let start_dt = date_util::Date::serial_to_datetime(start_date, false).unwrap_or_default();
    let end_dt = date_util::Date::serial_to_datetime(end_date, false).unwrap_or_default();
    if start_dt == end_dt {
        return new_number_formula_arg(0.0);
    }
    let (day_diff, days_in_year) = match basis {
        0 => yearfrac_basis0(start_date, end_date),
        1 => yearfrac_basis1(start_date, end_date),
        2 => (end_date - start_date, 360.0),
        3 => (end_date - start_date, 365.0),
        4 => yearfrac_basis4(start_date, end_date),
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    new_number_formula_arg(day_diff / days_in_year)
}

fn yearfrac(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 2 && args.len() != 3 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let prepared = prepare_data_value_args(args, 2);
    if prepared.typ != ArgType::List {
        return prepared;
    }
    let start = &prepared.list[0];
    let end = &prepared.list[1];
    let basis = if args.len() == 3 {
        args[2].to_number()
    } else {
        new_number_formula_arg(0.0)
    };
    if basis.typ != ArgType::Number {
        return basis;
    }
    yearfrac_impl(start.number, end.number, basis.number as i32)
}

fn second(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let arg = &args[0];
    let num = arg.to_number();
    if num.typ != ArgType::Number {
        let text = arg.value().to_lowercase();
        if !is_time_only_fmt(&text) {
            let (_, _, _, _, err) = str_to_date(&text);
            if err.typ == ArgType::Error {
                return err;
            }
        }
        let (_, _, s, _, _, err) = str_to_time(&text);
        if err.typ == ArgType::Error {
            return err;
        }
        return new_number_formula_arg((s as i32 % 60) as f64);
    }
    if num.number < 0.0 {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    let dt = date_util::Date::serial_to_datetime(num.number, false).unwrap_or_default();
    new_number_formula_arg(dt.second() as f64)
}

fn time(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 3 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let h = args[0].to_number();
    let m = args[1].to_number();
    let s = args[2].to_number();
    if h.typ != ArgType::Number || m.typ != ArgType::Number || s.typ != ArgType::Number {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let t = (h.number * 3600.0 + m.number * 60.0 + s.number) / 86400.0;
    if t < 0.0 {
        return new_error_formula_arg(FORMULA_ERROR_NUM);
    }
    new_number_formula_arg(t)
}

fn timevalue(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 1 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let arg = &args[0];
    let text = arg.value().to_lowercase();
    if !is_time_only_fmt(&text) {
        let (_, _, _, _, err) = str_to_date(&text);
        if err.typ == ArgType::Error {
            return err;
        }
    }
    let (mut h, m, s, pm, _, err) = str_to_time(&text);
    if err.typ == ArgType::Error {
        return err;
    }
    if pm {
        h += 12;
    }
    time(
        _ctx,
        &[
            new_number_formula_arg(h as f64),
            new_number_formula_arg(m as f64),
            new_number_formula_arg(s),
        ],
    )
}

fn weekday(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.is_empty() {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    if args.len() > 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let arg = &args[0];
    let num = arg.to_number();
    let weekday = if num.typ != ArgType::Number {
        let text = arg.value().to_lowercase();
        if !is_date_only_fmt(&text) {
            let (_, _, _, _, _, err) = str_to_time(&text);
            if err.typ == ArgType::Error {
                return err;
            }
        }
        let (y, m, d, _, err) = str_to_date(&text);
        if err.typ == ArgType::Error {
            return err;
        }
        let dt = NaiveDate::from_ymd_opt(y, m, d).unwrap();
        dt.weekday().num_days_from_sunday() as i32
    } else {
        if num.number < 0.0 {
            return new_error_formula_arg(FORMULA_ERROR_NUM);
        }
        let dt = date_util::Date::serial_to_datetime(num.number, false).unwrap_or_default();
        dt.weekday().num_days_from_sunday() as i32
    };

    let mut return_type = 1;
    if args.len() == 2 {
        let rt = args[1].to_number();
        if rt.typ != ArgType::Number {
            return rt;
        }
        return_type = rt.number as i32;
    }
    if return_type == 2 {
        return_type = 11;
    }
    let weekday = weekday + 1;
    match return_type {
        1 => new_number_formula_arg(weekday as f64),
        3 => new_number_formula_arg(((weekday + 6 - 1) % 7) as f64),
        rt if rt >= 11 && rt <= 17 => {
            new_number_formula_arg(((weekday + 6 - (rt - 10)) % 7 + 1) as f64)
        }
        _ => new_error_formula_arg(FORMULA_ERROR_VALUE),
    }
}

fn weeknum_impl(sn_time: NaiveDateTime, return_type: i32) -> FormulaArg {
    let days = sn_time.ordinal() as i32;
    let mut week_mod = days % 7;
    let mut week_num = (days as f64 / 7.0).ceil();
    if week_mod == 0 {
        week_mod = 7;
    }
    let year = sn_time.year();
    let first_weekday = NaiveDate::from_ymd_opt(year, 1, 1)
        .unwrap()
        .weekday()
        .num_days_from_sunday() as i32;
    let offset = match return_type {
        1 | 17 => 0,
        2 | 11 | 21 => 1,
        12 | 13 | 14 | 15 | 16 => return_type - 10,
        _ => return new_error_formula_arg(FORMULA_ERROR_NUM),
    };
    let mut padding = offset + 7 - first_weekday;
    if padding > 7 {
        padding -= 7;
    }
    if week_mod > padding {
        week_num += 1.0;
    }
    if return_type == 21 && (first_weekday == 0 || first_weekday > 4) {
        week_num -= 1.0;
        if week_num < 1.0 {
            week_num = 52.0;
            let prev_first = NaiveDate::from_ymd_opt(year - 1, 1, 1)
                .unwrap()
                .weekday()
                .num_days_from_sunday() as i32;
            if prev_first < 4 {
                week_num += 1.0;
            }
        }
    }
    new_number_formula_arg(week_num)
}

fn weeknum(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.is_empty() {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    if args.len() > 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let arg = &args[0];
    let num = arg.to_number();
    let sn_time = if num.typ != ArgType::Number {
        let text = arg.value().to_lowercase();
        if !is_date_only_fmt(&text) {
            let (_, _, _, _, _, err) = str_to_time(&text);
            if err.typ == ArgType::Error {
                return err;
            }
        }
        let (y, m, d, _, err) = str_to_date(&text);
        if err.typ == ArgType::Error {
            return err;
        }
        NaiveDate::from_ymd_opt(y, m, d)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap()
    } else {
        if num.number < 0.0 {
            return new_error_formula_arg(FORMULA_ERROR_NUM);
        }
        date_util::Date::serial_to_datetime(num.number, false).unwrap_or_default()
    };
    let mut return_type = 1;
    if args.len() == 2 {
        let rt = args[1].to_number();
        if rt.typ != ArgType::Number {
            return rt;
        }
        return_type = rt.number as i32;
    }
    weeknum_impl(sn_time, return_type)
}
