//! Date and time conversion helpers.
//!
//! Mirrors the logic in Go `date.go`, converting between Excel serial numbers
//! and `chrono` naive date/time types while accounting for the 1900/1904 date
//! system and the Excel 1900 leap-year bug.

use chrono::{NaiveDate, NaiveDateTime, NaiveTime, SubsecRound, Timelike};

use crate::errors::{ErrInvalidDate, Result};

const NANOS_IN_A_DAY: f64 = 24.0 * 60.0 * 60.0 * 1_000_000_000.0;
const ROUND_EPSILON: f64 = 1e-9;

/// Forward-conversion epoch for the 1900 date system (1899-12-31).
///
/// Excel's 1900 serial system counts from 1899-12-31 as day 0.
pub const EXCEL_EPOCH_1900: fn() -> NaiveDateTime = || {
    NaiveDate::from_ymd_opt(1899, 12, 31)
        .unwrap()
        .and_hms_opt(0, 0, 0)
        .unwrap()
};

/// Reverse-conversion epoch for the 1900 date system (1899-12-30).
///
/// Used after the 1900 leap-year bug so that serial arithmetic lines up
/// with Excel's internal day count.
pub const EXCEL_REVERSE_EPOCH_1900: fn() -> NaiveDateTime = || {
    NaiveDate::from_ymd_opt(1899, 12, 30)
        .unwrap()
        .and_hms_opt(0, 0, 0)
        .unwrap()
};

/// Excel epoch for the 1904 date system (1904-01-01).
pub const EXCEL_EPOCH_1904: fn() -> NaiveDateTime = || {
    NaiveDate::from_ymd_opt(1904, 1, 1)
        .unwrap()
        .and_hms_opt(0, 0, 0)
        .unwrap()
};

/// Earliest representable date in the 1900 date system.
pub const EXCEL_MIN_TIME_1900: fn() -> NaiveDateTime = || {
    NaiveDate::from_ymd_opt(1899, 12, 31)
        .unwrap()
        .and_hms_opt(0, 0, 0)
        .unwrap()
};

/// Start of the period affected by the Excel 1900 leap-year bug (1900-03-01).
pub const EXCEL_BUGGY_PERIOD_START: fn() -> NaiveDateTime = || {
    NaiveDate::from_ymd_opt(1900, 3, 1)
        .unwrap()
        .and_hms_opt(0, 0, 0)
        .unwrap()
};

const DAYS_IN_MONTH: [u32; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

/// Determine whether a year is a leap year.
pub fn is_leap_year(y: i32) -> bool {
    (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0)
}

/// Return the number of days in the given month.
pub fn get_days_in_month(y: i32, m: u32) -> u32 {
    if m == 2 && is_leap_year(y) {
        29
    } else {
        DAYS_IN_MONTH[m as usize - 1]
    }
}

/// Validate whether the given year, month and day form a legal date.
pub fn validate_date(y: i32, m: u32, d: u32) -> bool {
    if m < 1 || m > 12 || d < 1 {
        return false;
    }
    d <= get_days_in_month(y, m)
}

/// Convert a 2-digit year into a 4-digit year using Excel's 30/1900 rule.
pub fn format_year(y: i32) -> i32 {
    if y < 1900 {
        if y < 30 { y + 2000 } else { y + 1900 }
    } else {
        y
    }
}

/// Convert a `NaiveDateTime` to an Excel serial number.
///
/// Applies the 1900/1904 date system and the 1900 leap-year bug compatibility.
pub fn datetime_to_excel_serial(dt: NaiveDateTime, date1904: bool) -> f64 {
    let min_date = if date1904 {
        EXCEL_EPOCH_1904()
    } else {
        EXCEL_MIN_TIME_1900()
    };
    if dt < min_date {
        return 0.0;
    }

    let base = if date1904 {
        EXCEL_EPOCH_1904()
    } else {
        EXCEL_EPOCH_1900()
    };
    let duration = dt.signed_duration_since(base);
    let mut days = duration.num_seconds() as f64 / 86400.0;

    // Excel treats 1900 as a leap year for compatibility with Lotus 1-2-3.
    if !date1904 && dt >= EXCEL_BUGGY_PERIOD_START() {
        days += 1.0;
    }
    days
}

/// Convert a `NaiveDate` to an Excel serial number.
pub fn date_to_excel_serial(d: NaiveDate, date1904: bool) -> f64 {
    datetime_to_excel_serial(d.and_hms_opt(0, 0, 0).unwrap_or_default(), date1904)
}

/// Convert a `NaiveTime` to the fractional part of an Excel day.
pub fn time_to_excel_serial(t: NaiveTime) -> f64 {
    let seconds = t.num_seconds_from_midnight() as f64;
    let nano = t.nanosecond() as f64 / 1_000_000_000.0;
    (seconds + nano) / 86400.0
}

/// Convert an Excel serial number to a `NaiveDateTime`.
///
/// Negative serial numbers are invalid and return an error.
pub fn excel_serial_to_datetime(excel_time: f64, date1904: bool) -> Result<NaiveDateTime> {
    if excel_time < 0.0 {
        return Err(Box::new(ErrInvalidDate));
    }
    Ok(time_from_excel_time(excel_time, date1904))
}

fn time_from_excel_time(excel_time: f64, date1904: bool) -> NaiveDateTime {
    let whole_days_part = excel_time as i64;

    // Excel uses Julian dates prior to March 1st 1900 and Gregorian thereafter.
    if whole_days_part <= 61 {
        const OFFSET1900: f64 = 15018.0;
        const OFFSET1904: f64 = 16480.0;
        const MJD0: f64 = 2400000.5;
        let offset = if date1904 { OFFSET1904 } else { OFFSET1900 };
        return julian_date_to_gregorian_time(MJD0, excel_time + offset);
    }

    let float_part = excel_time - (whole_days_part as f64) + ROUND_EPSILON;
    let base = if date1904 {
        EXCEL_EPOCH_1904()
    } else {
        EXCEL_REVERSE_EPOCH_1900()
    };

    let duration = chrono::Duration::nanoseconds((NANOS_IN_A_DAY * float_part) as i64);
    let mut date = base
        .checked_add_signed(chrono::Duration::days(whole_days_part))
        .unwrap_or(base)
        .checked_add_signed(duration)
        .unwrap_or(base);

    // Round or truncate to the nearest second to avoid nanosecond noise.
    if date.nanosecond() / 1_000_000 > 500 {
        date = date.round_subsecs(0);
    } else {
        date = date.trunc_subsecs(0);
    }
    date
}

fn shift_julian_to_noon(julian_days: f64, julian_fraction: f64) -> (f64, f64) {
    match julian_fraction {
        f if (-0.5..0.5).contains(&f) => (julian_days, julian_fraction + 0.5),
        f if f >= 0.5 => (julian_days + 1.0, julian_fraction - 0.5),
        f if f <= -0.5 => (julian_days - 1.0, julian_fraction + 1.5),
        _ => (julian_days, julian_fraction),
    }
}

fn fraction_of_a_day(fraction: f64) -> (i32, i32, i32, i32) {
    const C1US: i64 = 1_000;
    const C1S: i64 = 1_000_000_000;
    const C1DAY: i64 = 24 * 60 * 60 * C1S;

    let mut frac = (C1DAY as f64 * fraction + C1US as f64 / 2.0) as i64;
    let nanoseconds = ((frac % C1S) / C1US) as i32 * C1US as i32;
    frac /= C1S;
    let seconds = (frac % 60) as i32;
    frac /= 60;
    let minutes = (frac % 60) as i32;
    let hours = (frac / 60) as i32;
    (hours, minutes, seconds, nanoseconds)
}

fn julian_date_to_gregorian_time(part1: f64, part2: f64) -> NaiveDateTime {
    let part1_i = part1.trunc();
    let part1_f = part1.fract();
    let part2_i = part2.trunc();
    let part2_f = part2.fract();

    let mut julian_days = part1_i + part2_i;
    let mut julian_fraction = part1_f + part2_f;
    (julian_days, julian_fraction) = shift_julian_to_noon(julian_days, julian_fraction);

    let (day, month, year) = fliegel_van_flandern(julian_days as i32);
    let (hours, minutes, seconds, nanoseconds) = fraction_of_a_day(julian_fraction);

    NaiveDate::from_ymd_opt(year, month as u32, day as u32)
        .unwrap_or_else(|| NaiveDate::from_ymd_opt(1899, 12, 30).unwrap())
        .and_hms_nano_opt(
            hours as u32,
            minutes as u32,
            seconds as u32,
            nanoseconds as u32,
        )
        .unwrap_or_default()
}

fn fliegel_van_flandern(jd: i32) -> (i32, i32, i32) {
    let mut l = jd + 68569;
    let n = (4 * l) / 146097;
    l = l - (146097 * n + 3) / 4;
    let i = (4000 * (l + 1)) / 1461001;
    l = l - (1461 * i) / 4 + 31;
    let j = (80 * l) / 2447;
    let d = l - (2447 * j) / 80;
    l = j / 11;
    let m = j + 2 - (12 * l);
    let y = 100 * (n - 49) + i + l;
    (d, m, y)
}

/// Convenience helper for date/time conversions used elsewhere in the crate.
pub struct Date;

impl Date {
    /// Convert a `NaiveDateTime` to an Excel serial number.
    pub fn datetime_to_serial(dt: NaiveDateTime, date1904: bool) -> f64 {
        datetime_to_excel_serial(dt, date1904)
    }

    /// Convert a `NaiveDate` to an Excel serial number.
    pub fn date_to_serial(d: NaiveDate, date1904: bool) -> f64 {
        date_to_excel_serial(d, date1904)
    }

    /// Convert a `NaiveTime` to the fractional part of an Excel day.
    pub fn time_to_serial(t: NaiveTime) -> f64 {
        time_to_excel_serial(t)
    }

    /// Convert an Excel serial number to a `NaiveDateTime`.
    pub fn serial_to_datetime(serial: f64, date1904: bool) -> Result<NaiveDateTime> {
        excel_serial_to_datetime(serial, date1904)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_datetime_to_excel_serial_1900() {
        let dt = NaiveDate::from_ymd_opt(2024, 7, 13)
            .unwrap()
            .and_hms_opt(12, 0, 0)
            .unwrap();
        // 2024-07-13 12:00 is serial 45486.5 in the 1900 system.
        assert!((datetime_to_excel_serial(dt, false) - 45486.5).abs() < 1e-9);
    }

    #[test]
    fn test_datetime_to_excel_serial_1904() {
        let dt = NaiveDate::from_ymd_opt(2024, 7, 13)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();
        // 1904 system is 1462 days behind the 1900 system.
        assert!((datetime_to_excel_serial(dt, true) - (45486.0 - 1462.0)).abs() < 1e-9);
    }

    #[test]
    fn test_1900_leap_year_bug() {
        // 1900-03-01 serial in 1900 system should be 61 (Excel counts 2/29/1900).
        let dt = NaiveDate::from_ymd_opt(1900, 3, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();
        assert_eq!(datetime_to_excel_serial(dt, false), 61.0);
    }

    #[test]
    fn test_excel_serial_to_datetime_1900() {
        let dt = excel_serial_to_datetime(45486.5, false).unwrap();
        assert_eq!(dt.date(), NaiveDate::from_ymd_opt(2024, 7, 13).unwrap());
        assert_eq!(dt.time(), NaiveTime::from_hms_opt(12, 0, 0).unwrap());
    }

    #[test]
    fn test_excel_serial_to_datetime_1904() {
        let dt = excel_serial_to_datetime(44024.5, true).unwrap();
        assert_eq!(dt.date(), NaiveDate::from_ymd_opt(2024, 7, 13).unwrap());
        assert_eq!(dt.time(), NaiveTime::from_hms_opt(12, 0, 0).unwrap());
    }

    #[test]
    fn test_time_to_excel_serial() {
        let t = NaiveTime::from_hms_opt(12, 0, 0).unwrap();
        assert!((time_to_excel_serial(t) - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_negative_serial_is_error() {
        assert!(excel_serial_to_datetime(-1.0, false).is_err());
    }

    #[test]
    fn test_is_leap_year() {
        assert!(is_leap_year(2024));
        assert!(!is_leap_year(2023));
        assert!(!is_leap_year(1900));
        assert!(is_leap_year(2000));
    }

    #[test]
    fn test_get_days_in_month() {
        assert_eq!(get_days_in_month(2024, 2), 29);
        assert_eq!(get_days_in_month(2023, 2), 28);
        assert_eq!(get_days_in_month(2024, 1), 31);
        assert_eq!(get_days_in_month(2024, 4), 30);
    }
}
