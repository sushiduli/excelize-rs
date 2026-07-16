//! Number format parser and applier.
//!
//! Ports the most commonly used parts of Go `numfmt.go`. It maps built-in
//! number-format IDs to their format strings and can apply many custom format
//! codes for numbers, percentages, scientific notation, fractions, currency,
//! accounting, dates and times.

use std::collections::HashMap;
use std::sync::LazyLock;

use crate::date::excel_serial_to_datetime;

/// Built-in number format map (English localization).
///
/// These IDs are reserved by Excel and do not need to be stored in
/// `xl/styles.xml`.
pub static BUILT_IN_NUM_FMT: LazyLock<HashMap<i32, &'static str>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert(0, "General");
    m.insert(1, "0");
    m.insert(2, "0.00");
    m.insert(3, "#,##0");
    m.insert(4, "#,##0.00");
    m.insert(9, "0%");
    m.insert(10, "0.00%");
    m.insert(11, "0.00E+00");
    m.insert(12, "# ?/?");
    m.insert(13, "# ??/??");
    m.insert(14, "mm-dd-yy");
    m.insert(15, "d-mmm-yy");
    m.insert(16, "d-mmm");
    m.insert(17, "mmm-yy");
    m.insert(18, "h:mm AM/PM");
    m.insert(19, "h:mm:ss AM/PM");
    m.insert(20, "hh:mm");
    m.insert(21, "hh:mm:ss");
    m.insert(22, "m/d/yy hh:mm");
    m.insert(37, "#,##0 ;(#,##0)");
    m.insert(38, "#,##0 ;[Red](#,##0)");
    m.insert(39, "#,##0.00 ;(#,##0.00)");
    m.insert(40, "#,##0.00 ;[Red](#,##0.00)");
    m.insert(45, "mm:ss");
    m.insert(46, "[h]:mm:ss");
    m.insert(47, "mm:ss.0");
    m.insert(48, "##0.0E+0");
    m.insert(49, "@");
    m
});

/// Return the built-in format code for an ID, if one exists.
pub fn built_in_num_fmt_code(num_fmt_id: i32) -> Option<&'static str> {
    BUILT_IN_NUM_FMT.get(&num_fmt_id).copied()
}

/// Apply a number format to a numeric value.
///
/// * `value` - the raw numeric cell value (Excel serial for dates/times).
/// * `num_fmt_id` - Excel number format ID.
/// * `format_code` - Optional explicit format code (used for custom formats
///   with IDs >= 164 or when overriding a built-in ID).
/// * `date1904` - Whether the workbook uses the 1904 date system.
///
/// Falls back to the value formatted with default precision when the format
/// code is not supported.
pub fn apply_number_format(
    value: f64,
    num_fmt_id: i32,
    format_code: Option<&str>,
    date1904: bool,
) -> String {
    let code = format_code
        .filter(|c| !c.is_empty())
        .or_else(|| built_in_num_fmt_code(num_fmt_id))
        .unwrap_or("General");

    apply_format_code(value, code, date1904)
}

/// Apply a format code directly to a numeric value without an Excel numFmt ID.
///
/// This is the entry point used by formula functions such as `TEXT`, where the
/// format is supplied as a string rather than a style ID.
pub fn format_number(value: f64, code: &str, date1904: bool) -> String {
    apply_format_code(value, code, date1904)
}

fn apply_format_code(value: f64, code: &str, date1904: bool) -> String {
    let code = code.trim();
    if code.eq_ignore_ascii_case("General") || code.is_empty() {
        return format_general(value);
    }

    // Excel number formats can contain up to four semicolon-separated sections.
    // Select the appropriate section for the value, strip color/condition
    // metadata, and remember whether the selected section is the dedicated
    // negative section so we avoid adding a duplicate minus sign.
    let (section, is_negative_section, switch_argument, locale_code) =
        choose_format_section(code, value);
    let abs_value = value.abs();

    let mut result = if is_date_format(&section) {
        if let Ok(dt) = excel_serial_to_datetime(abs_value, date1904) {
            format_date_time(&section, abs_value, dt, locale_code.as_deref())
        } else {
            abs_value.to_string()
        }
    } else {
        format_numeric(&section, abs_value)
    };

    // If the selected section is not the dedicated negative section but the
    // value is negative, prepend a minus sign (matching Go's usePositive
    // behavior when no negative section exists).
    if !is_negative_section && value < 0.0 && result != "0" {
        result = format!("-{}", result);
    }

    // Apply East Asian DBNum switch arguments.
    if let Some(sw) = switch_argument {
        result = apply_switch_argument(&result, &sw);
    }

    result
}

fn apply_switch_argument(result: &str, sw: &str) -> String {
    match sw.to_ascii_uppercase().as_str() {
        "DBNUM1" => result
            .chars()
            .map(|c| match c {
                '0' => '\u{25CB}',
                '1' => '\u{4E00}',
                '2' => '\u{4E8C}',
                '3' => '\u{4E09}',
                '4' => '\u{56DB}',
                '5' => '\u{4E94}',
                '6' => '\u{516D}',
                '7' => '\u{4E03}',
                '8' => '\u{516B}',
                '9' => '\u{4E5D}',
                _ => c,
            })
            .collect(),
        "DBNUM2" => result
            .chars()
            .map(|c| match c {
                '0' => '\u{96F6}',
                '1' => '\u{58F9}',
                '2' => '\u{8D30}',
                '3' => '\u{53C1}',
                '4' => '\u{8086}',
                '5' => '\u{4F0D}',
                '6' => '\u{9646}',
                '7' => '\u{67D2}',
                '8' => '\u{634C}',
                '9' => '\u{7396}',
                _ => c,
            })
            .collect(),
        "DBNUM3" => result
            .chars()
            .map(|c| match c {
                '0' => '\u{FF10}',
                '1' => '\u{FF11}',
                '2' => '\u{FF12}',
                '3' => '\u{FF13}',
                '4' => '\u{FF14}',
                '5' => '\u{FF15}',
                '6' => '\u{FF16}',
                '7' => '\u{FF17}',
                '8' => '\u{FF18}',
                '9' => '\u{FF19}',
                _ => c,
            })
            .collect(),
        _ => result.to_string(),
    }
}

fn format_general(value: f64) -> String {
    if value.fract() == 0.0 && value.abs() < 1e15 {
        format!("{:.0}", value)
    } else {
        format!("{}", value)
    }
}

/// Check whether a format code contains date/time tokens.
fn is_date_format(code: &str) -> bool {
    // Scientific notation stays numeric even though it contains `E`.
    let upper = code.to_ascii_uppercase();
    if upper.contains("E+0") || upper.contains("E-0") {
        return false;
    }
    // A section is a date/time format when it contains date/time token
    // letters (y, d, h, s, m, e, g) outside quoted literals and escapes, or
    // elapsed-time tokens ([h], [m], [s]) in brackets. Mirrors nfp's section
    // typing in Go, which treats standalone month/weekday tokens such as
    // `mmm` or `mmmmm` as date formats too.
    let mut chars = code.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '"' => {
                for c in chars.by_ref() {
                    if c == '"' {
                        break;
                    }
                }
            }
            '\\' => {
                chars.next();
            }
            '[' => {
                let mut content = String::new();
                for c in chars.by_ref() {
                    if c == ']' {
                        break;
                    }
                    content.push(c);
                }
                if matches!(content.to_ascii_lowercase().as_str(), "h" | "m" | "s") {
                    return true;
                }
            }
            'y' | 'Y' | 'd' | 'D' | 'h' | 'H' | 's' | 'S' | 'm' | 'M' | 'e' | 'E' | 'g' | 'G' => {
                return true;
            }
            _ => {}
        }
    }
    false
}

// ------------------------------------------------------------------
// Format-section splitting and selection
// ------------------------------------------------------------------

/// Split a number format code into its semicolon-separated sections, respecting
/// quoted literals and escaped characters.
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

/// Return `true` if `token` is a currency/language bracket such as `$-409`
/// or `€-40C`. These brackets carry locale/currency metadata and should not
/// be printed as literal text.
fn is_currency_language_token(token: &str) -> bool {
    // Common currency symbols.
    if token.contains('$')
        || token.contains('€')
        || token.contains('£')
        || token.contains('¥')
    {
        return true;
    }
    // Locale ID like `-409` or `-804` (hex digits).
    if token.starts_with('-') && token[1..].chars().all(|c| c.is_ascii_hexdigit()) {
        return true;
    }
    // Special locale aliases used by Excel.
    matches!(token.to_ascii_lowercase().as_str(), "f800" | "f400" | "x-sysdate" | "x-systime" | "1010000")
}

/// Extract a currency symbol from a currency/language bracket token such as
/// `$$` (`$`) or `$€-40C` (`€`). Returns `None` for locale-only brackets like
/// `$-409`.
fn extract_currency_symbol(token: &str) -> Option<&str> {
    // Special case: `$$` means a literal dollar sign.
    if token == "$$" {
        return Some("$");
    }
    // Non-dollar currency symbols inside the bracket are explicit currency
    // strings (e.g. `$€-40C` carries the Euro symbol).
    for sym in &["€", "£", "¥"] {
        if let Some(pos) = token.find(sym) {
            let len = sym.len();
            return Some(&token[pos..pos + len]);
        }
    }
    // `$-409` and similar are locale-only brackets; do not emit the dollar sign.
    None
}

/// Extract a DBNum switch argument such as `[DBNum1]`.
fn extract_switch_argument(token: &str) -> Option<String> {
    if token.len() >= 6 && token[..5].eq_ignore_ascii_case("DBNum") {
        if token[5..].chars().all(|c| c.is_ascii_digit()) {
            return Some(token.to_string());
        }
    }
    None
}

/// Extract the locale ID from a currency/language bracket token such as
/// `$-409` or `€-40C`.
fn extract_locale_code(token: &str) -> Option<String> {
    if token.starts_with('-') {
        return Some(token[1..].to_string());
    }
    if let Some(pos) = token.find('-') {
        return Some(token[pos + 1..].to_string());
    }
    None
}

/// Strip leading color, condition, currency/language and DBNum bracket tokens
/// from a format section.
fn strip_section_metadata(section: &str) -> (Option<FormatCondition>, String, Option<String>, Option<String>) {
    let mut section = section;
    let mut condition: Option<FormatCondition> = None;
    let mut currency_prefix = String::new();
    let mut switch_argument: Option<String> = None;
    let mut locale_code: Option<String> = None;

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

        if is_currency_language_token(token) {
            if let Some(sym) = extract_currency_symbol(token) {
                currency_prefix.push_str(sym);
            }
            if let Some(code) = extract_locale_code(token) {
                locale_code = Some(code);
            }
            section = &rest[end + 1..];
            continue;
        }

        if let Some(sw) = extract_switch_argument(token) {
            switch_argument = Some(sw);
            section = &rest[end + 1..];
            continue;
        }

        // Anything else (elapsed-time codes, etc.) is part of the format body
        // and stops metadata stripping.
        break;
    }

    let mut body = currency_prefix;
    body.push_str(section);
    (condition, body, switch_argument, locale_code)
}

/// Select the format section that applies to a numeric value.
///
/// Mirrors Go's positional section selection: conditions such as `[>100]`
/// are stripped as metadata but never evaluated. Non-negative values always
/// use the first (positive) section; negative values use the second section
/// when present (rendered with the absolute value), otherwise the positive
/// section with a minus sign prepended by the caller.
fn choose_format_section(code: &str, value: f64) -> (String, bool, Option<String>, Option<String>) {
    let sections = split_format_sections(code);
    let parsed: Vec<(Option<FormatCondition>, String, Option<String>, Option<String>)> =
        sections.iter().map(|s| strip_section_metadata(s)).collect();

    let selected_idx = if value >= 0.0 {
        0
    } else if parsed.len() >= 2 {
        1
    } else {
        0
    };
    let is_negative = value < 0.0 && selected_idx == 1;
    (parsed[selected_idx].1.clone(), is_negative, parsed[selected_idx].2.clone(), parsed[selected_idx].3.clone())
}

/// Check whether a number-format pattern is suitable for use as a date/time
/// pattern.
///
/// This is a minimal token scanner (not a full `nfp` parser). It rejects
/// patterns that contain numeric-only tokens such as `0`, `#`, `?`, `%`, the
/// text placeholder `@`, scientific notation (`E+0`, `E-0`) or fractions
/// (`?/?`). Quoted literals, escaped characters, bracketed expressions
/// (colors, locales, elapsed time) and common date/time separators are
/// allowed. An empty pattern is allowed.
pub fn is_date_time_pattern(pattern: &str) -> bool {
    if pattern.is_empty() {
        return true;
    }
    for section in pattern.split(';') {
        if !is_date_time_section(section) {
            return false;
        }
    }
    true
}

fn is_date_time_section(section: &str) -> bool {
    let mut chars = section.chars().peekable();
    let mut prev: Option<char> = None;

    while let Some(ch) = chars.next() {
        match ch {
            '"' => {
                if !consume_quoted_literal(&mut chars) {
                    return false;
                }
                prev = Some('"');
            }
            '\\' => {
                if chars.next().is_none() {
                    return false;
                }
                prev = Some('\\');
            }
            '[' => {
                if !consume_bracketed_expr(&mut chars) {
                    return false;
                }
                prev = Some(']');
            }
            '@' | '0' | '#' | '?' | '%' => {
                return false;
            }
            '/' => {
                // Reject slashes that look like fraction operators (adjacent
                // to numeric placeholders). Slashes used as date separators
                // (e.g. `m/d/yyyy`) are surrounded by date/time letters and
                // are allowed.
                if prev.map_or(false, is_numeric_placeholder) {
                    return false;
                }
                if chars.peek().copied().map_or(false, is_numeric_placeholder) {
                    return false;
                }
                prev = Some('/');
            }
            'e' | 'E' => {
                // Reject scientific notation E+0 / E-0.
                if let Some(&next) = chars.peek() {
                    if next == '+' || next == '-' {
                        let mut tmp = chars.clone();
                        tmp.next();
                        if tmp.peek().copied().map_or(false, is_numeric_placeholder) {
                            return false;
                        }
                    }
                }
                prev = Some(ch);
            }
            _ => {
                prev = Some(ch);
            }
        }
    }
    true
}

fn consume_quoted_literal(chars: &mut std::iter::Peekable<impl Iterator<Item = char>>) -> bool {
    for c in chars {
        if c == '"' {
            return true;
        }
    }
    false
}

fn consume_bracketed_expr(chars: &mut std::iter::Peekable<impl Iterator<Item = char>>) -> bool {
    for c in chars {
        if c == ']' {
            return true;
        }
    }
    false
}

fn is_numeric_placeholder(c: char) -> bool {
    matches!(c, '0' | '#' | '?')
}

// ------------------------------------------------------------------
// Numeric formatting
// ------------------------------------------------------------------

/// Remove commas that appear after the last digit placeholder; each such
/// comma scales the value down by a factor of 1000. Mirrors Go's
/// `scalingFactor` handling in `applyThousandsSeparatorToken`.
fn strip_scaling_commas(code: &str) -> (usize, String) {
    let mut last_placeholder: Option<usize> = None;
    let mut chars = code.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        match c {
            '"' => {
                for (_, c) in chars.by_ref() {
                    if c == '"' {
                        break;
                    }
                }
            }
            '\\' => {
                chars.next();
            }
            '0' | '#' | '?' => last_placeholder = Some(i),
            _ => {}
        }
    }
    let Some(last) = last_placeholder else {
        return (0, code.to_string());
    };
    let mut factor = 0;
    let mut result = String::with_capacity(code.len());
    let mut in_quote = false;
    let mut escaped = false;
    for (i, c) in code.chars().enumerate() {
        if escaped {
            result.push(c);
            escaped = false;
            continue;
        }
        match c {
            '\\' => {
                result.push(c);
                escaped = true;
            }
            '"' => {
                result.push(c);
                in_quote = !in_quote;
            }
            ',' if !in_quote && i > last => factor += 1,
            _ => result.push(c),
        }
    }
    (factor, result)
}

/// Build a placeholder-only view of a format body: quoted literals, escaped
/// characters, `_x` space literals and `*x` fill literals are removed so
/// digit placeholders can be counted and located reliably.
fn analysis_pattern(code: &str) -> String {
    let mut out = String::with_capacity(code.len());
    let mut chars = code.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '"' => {
                for c in chars.by_ref() {
                    if c == '"' {
                        break;
                    }
                }
            }
            '\\' | '_' | '*' => {
                chars.next();
            }
            _ => out.push(c),
        }
    }
    out
}

fn format_numeric(code: &str, value: f64) -> String {
    let (scaling, code) = strip_scaling_commas(code);
    let code = code.as_str();
    let value = value / 1000f64.powi(scaling as i32);
    let analysis = analysis_pattern(code);

    // Percentage.
    let percent_count = analysis.chars().filter(|&c| c == '%').count();
    let base_value = value * (100.0_f64.powi(percent_count as i32));

    // Scientific notation.
    if analysis.to_ascii_uppercase().contains("E+0")
        || analysis.to_ascii_uppercase().contains("E-0")
    {
        return format_scientific(&analysis, base_value);
    }

    // Fraction.
    if analysis.contains('/') {
        return format_fraction(code, base_value);
    }

    // Sections that contain no numeric placeholders are rendered as literal text.
    let has_placeholders = analysis.chars().any(|c| matches!(c, '0' | '#' | '?'));
    if !has_placeholders {
        return merge_literals(code, "", false);
    }

    // Extract integer and decimal patterns.
    let parts: Vec<&str> = analysis.split('.').collect();
    let int_pattern = parts.first().copied().unwrap_or("");
    let frac_pattern = parts.get(1).copied().unwrap_or("");

    let use_thousands = int_pattern.contains(',');
    let frac_digits = count_digit_placeholders(frac_pattern);
    let force_decimal = frac_pattern.contains('0');

    let rounded = round_to_digits(base_value, frac_digits);

    let mut int_part = rounded.trunc().abs() as i64;
    // Handle the case where rounding pushes us to the next integer.
    if (rounded.fract().abs() - 1.0).abs() < 1e-12 {
        int_part += 1;
    }

    let mut result = String::new();
    result.push_str(&format_integer(int_part, int_pattern));

    if force_decimal || (!frac_pattern.is_empty() && frac_digits > 0) {
        let frac = rounded.fract().abs();
        let frac_str = format_fractional(frac, frac_digits, frac_pattern.contains('0'));
        if !frac_str.is_empty() {
            result.push('.');
            result.push_str(&frac_str);
        } else if force_decimal {
            result.push('.');
            result.push_str(&"0".repeat(frac_digits.max(1)));
        }
    }

    if use_thousands {
        result = add_thousands(result);
    }

    // Append literals and currency symbols that were stripped during parsing.
    result = merge_literals(code, &result, true);

    if percent_count > 0 {
        result.push('%');
    }

    result
}

fn count_digit_placeholders(pattern: &str) -> usize {
    pattern
        .chars()
        .filter(|&c| c == '0' || c == '#' || c == '?')
        .count()
}

fn format_integer(value: i64, pattern: &str) -> String {
    let min_digits = pattern.chars().filter(|&c| c == '0').count();
    let has_question = pattern.contains('?');
    let digits_needed = pattern
        .chars()
        .filter(|&c| c == '#' || c == '0' || c == '?')
        .count();

    let mut s = value.to_string();
    if s.len() < min_digits {
        s = format!("{}{}", "0".repeat(min_digits - s.len()), s);
    }
    // Question-mark placeholders right-align with spaces; hash placeholders do
    // not pad. Only enforce the total width when the pattern explicitly uses '?'.
    if has_question && s.len() < digits_needed {
        s = format!("{}{}", " ".repeat(digits_needed - s.len()), s);
    }
    s
}

fn format_fractional(frac: f64, digits: usize, force_zero: bool) -> String {
    if digits == 0 {
        return String::new();
    }
    let scaled = (frac * 10.0_f64.powi(digits as i32)).round() as i64;
    if scaled == 0 && !force_zero {
        return String::new();
    }
    let mut s = format!("{:0digits$}", scaled, digits = digits);
    if !force_zero {
        s = s.trim_end_matches('0').to_string();
    }
    s
}

fn add_thousands(text: String) -> String {
    let mut result = String::new();
    let (int_part, frac_part) = if let Some(pos) = text.find('.') {
        (&text[..pos], Some(&text[pos..]))
    } else {
        (text.as_str(), None)
    };

    let (sign, digits) = if int_part.starts_with('-') {
        ("-", &int_part[1..])
    } else {
        ("", int_part)
    };

    result.push_str(sign);
    for (i, ch) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            result.push(',');
        }
        result.push(ch);
    }

    if let Some(frac) = frac_part {
        result.push_str(frac);
    }
    result
}

fn format_scientific(code: &str, value: f64) -> String {
    let upper = code.to_ascii_uppercase();
    let mantissa_code = upper.split('E').next().unwrap_or("");
    let frac_digits = mantissa_code
        .split('.')
        .nth(1)
        .map(|s| s.chars().filter(|&c| c == '0' || c == '#').count())
        .unwrap_or(0);

    if value == 0.0 {
        let mantissa = format!("{:.*}", frac_digits, 0.0);
        return format!("{}E+00", mantissa);
    }

    let sign = if value < 0.0 { "-" } else { "" };
    let abs = value.abs();
    let exponent = abs.log10().floor() as i32;
    let mantissa = abs / 10.0_f64.powi(exponent);
    format!("{}{:.*}E{:+03}", sign, frac_digits, mantissa, exponent)
}

fn format_fraction(code: &str, value: f64) -> String {
    let analysis = analysis_pattern(code);
    let Some(slash) = analysis.find('/') else {
        return merge_literals(code, &value.to_string(), true);
    };
    let before = &analysis[..slash];
    let after = &analysis[slash + 1..];

    // The numerator is the run of `?` placeholders right before the slash.
    let num_width = before.chars().rev().take_while(|&c| c == '?').count();
    // The denominator is either a run of `?` placeholders or explicit digits.
    let den_token: String = after
        .chars()
        .take_while(|&c| c == '?' || c.is_ascii_digit())
        .collect();

    let abs = value.abs();
    let whole = abs.trunc() as i64;
    let frac = abs.fract();

    // Integer part: the placeholder run preceding the numerator (Go always
    // renders it, even when it is zero), plus any literal between it and the
    // numerator (e.g. the space in `# ?/?`).
    let int_region = &before[..before.len() - num_width];
    let int_width = int_region
        .chars()
        .rev()
        .take_while(|&c| matches!(c, '0' | '#' | '?'))
        .count();
    let int_pattern = &int_region[int_region.len() - int_width..];
    let between = &int_region[..int_region.len() - int_width];

    let mut result = String::new();
    if int_width > 0 {
        result.push_str(&format_integer(whole, int_pattern));
    }
    result.push_str(between);

    if den_token.chars().all(|c| c == '?') && !den_token.is_empty() {
        result.push_str(&float_to_fraction(frac, num_width, den_token.len()));
    } else if let Ok(denom) = den_token.parse::<f64>() {
        // Explicit denominator: rounded numerator over the literal value.
        let num = (frac * denom).round() as i64;
        result.push_str(&format!("{}/{}", num, denom as i64));
    }
    result
}

/// Convert the fractional part to a fraction string with the numerator
/// left-padded and the denominator right-padded to the placeholder widths.
/// Mirrors Go's `floatToFraction` (continued-fraction approximation).
fn float_to_fraction(x: f64, numerator_place_holder: usize, denominator_place_holder: usize) -> String {
    if denominator_place_holder == 0 {
        return String::new();
    }
    let limit = 10i64.pow(denominator_place_holder as u32);
    let (num, den) = float_to_frac_continued(x, limit);
    if num == 0 {
        return " ".repeat(numerator_place_holder + denominator_place_holder + 1);
    }
    let num_str = num.to_string();
    let den_str = den.to_string();
    let num_pad = numerator_place_holder.saturating_sub(num_str.len());
    let den_pad = denominator_place_holder.saturating_sub(den_str.len());
    format!(
        "{}{}/{}{}",
        " ".repeat(num_pad),
        num_str,
        den_str,
        " ".repeat(den_pad)
    )
}

/// Convert a floating-point decimal to a fraction using continued fractions
/// and recurrence relations. Mirrors Go's `floatToFracUseContinuedFraction`.
fn float_to_frac_continued(mut r: f64, denominator_limit: i64) -> (i64, i64) {
    let mut p1: i64 = 1;
    let mut q1: i64 = 0;
    let mut p2: i64 = 0;
    let mut q2: i64 = 1;
    let mut lasta: i64 = 0;
    let mut lastb: i64 = 0;
    loop {
        let a = r.floor() as i64;
        let curra = a * p1 + p2;
        let currb = a * q1 + q2;
        p2 = p1;
        q2 = q1;
        p1 = curra;
        q1 = currb;
        let frac = r - a as f64;
        if q1 >= denominator_limit {
            return (lasta, lastb);
        }
        if frac.abs() < 1e-12 {
            return (curra, currb);
        }
        lasta = curra;
        lastb = currb;
        r = 1.0 / frac;
    }
}

fn merge_literals(code: &str, formatted_number: &str, append_number: bool) -> String {
    // This is a best-effort merge: replace the first contiguous run of numeric
    // placeholders in the code with the formatted integer/decimal digits.
    let mut result = String::new();
    let mut digit_run_seen = false;
    let mut chars = code.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '"' {
            // Quoted literal.
            while let Some(c) = chars.next() {
                if c == '"' {
                    break;
                }
                result.push(c);
            }
            continue;
        }
        if ch == '\\' {
            if let Some(c) = chars.next() {
                result.push(c);
            }
            continue;
        }
        if ch == '_' {
            // Underscore alignment: leave a space equal to the width of the
            // next character and consume that character.
            if chars.next().is_some() {
                result.push(' ');
            }
            continue;
        }
        if ch == '*' {
            // Repeat alignment: consume the next character and do not emit it.
            chars.next();
            continue;
        }
        if ch == '(' || ch == ')' || ch == '[' || ch == ']' {
            result.push(ch);
            continue;
        }
        if ch == '%' {
            // Percent symbol is appended once after formatting.
            continue;
        }
        if matches!(ch, '$' | '€' | '£' | '¥') {
            result.push(ch);
            continue;
        }
        if ch == '-' {
            // A literal minus sign in the format code is always emitted (the
            // caller has already selected the appropriate signed section).
            result.push('-');
            continue;
        }
        if ch.is_ascii_digit() || ch == '#' || ch == '?' || ch == ',' || ch == '.' {
            if !digit_run_seen {
                result.push_str(formatted_number);
                digit_run_seen = true;
            }
            continue;
        }
        result.push(ch);
    }

    if !digit_run_seen && append_number {
        result.push_str(formatted_number);
    }

    result
}

fn round_to_digits(value: f64, digits: usize) -> f64 {
    if digits == 0 {
        return value.round();
    }
    let factor = 10.0_f64.powi(digits as i32);
    (value * factor).round() / factor
}

// ------------------------------------------------------------------
// Date/time formatting
// ------------------------------------------------------------------

fn fraction_to_time(fraction: f64) -> chrono::NaiveTime {
    let total_seconds = (fraction.abs().fract() * 24.0 * 60.0 * 60.0).round() as u32;
    let hours = (total_seconds / 3600).min(23);
    let minutes = ((total_seconds % 3600) / 60).min(59);
    let seconds = (total_seconds % 60).min(59);
    chrono::NaiveTime::from_hms_opt(hours, minutes, seconds).unwrap_or(chrono::NaiveTime::MIN)
}

fn format_date_time(
    code: &str,
    value: f64,
    dt: chrono::NaiveDateTime,
    locale_code: Option<&str>,
) -> String {
    use chrono::{Datelike, Timelike};

    let mut result = String::new();
    let mut chars = code.chars().peekable();
    let mut prev_token_was_hour = false;
    let mut use_elapsed_hour = false;
    let mut current_dt = dt;

    while let Some(ch) = chars.next() {
        if ch == '[' {
            let mut bracket = String::new();
            while let Some(c) = chars.next() {
                if c == ']' {
                    break;
                }
                bracket.push(c);
            }
            match bracket.to_ascii_uppercase().as_str() {
                "H" => {
                    let total_hours = (value * 24.0).floor();
                    result.push_str(&format!("{:.0}", total_hours));
                    let remaining = value - total_hours / 24.0;
                    current_dt = dt.date().and_time(fraction_to_time(remaining));
                    use_elapsed_hour = true;
                }
                "M" => result.push_str(&(value * 24.0 * 60.0).floor().to_string()),
                "S" => result.push_str(&(value * 24.0 * 60.0 * 60.0).floor().to_string()),
                _ => {
                    result.push('[');
                    result.push_str(&bracket);
                    result.push(']');
                }
            }
            prev_token_was_hour = false;
            continue;
        }
        if ch == '"' {
            while let Some(c) = chars.next() {
                if c == '"' {
                    break;
                }
                result.push(c);
            }
            continue;
        }
        if ch == '\\' {
            if let Some(c) = chars.next() {
                result.push(c);
            }
            continue;
        }

        let upper = ch.to_ascii_uppercase();
        if upper == 'Y' {
            let count = 1 + count_repeats(&mut chars, 'y', 'Y');
            let year = dt.year();
            if count >= 4 {
                result.push_str(&format!("{:04}", year));
            } else {
                result.push_str(&format!("{:02}", year % 100));
            }
            prev_token_was_hour = false;
        } else if upper == 'M' {
            let count = 1 + count_repeats(&mut chars, 'm', 'M');
            let dt = if use_elapsed_hour { current_dt } else { dt };
            if prev_token_was_hour || use_elapsed_hour {
                // Minutes.
                result.push_str(&format!("{:02}", dt.minute()));
            } else {
                // Month.
                match count {
                    1 => result.push_str(&dt.month().to_string()),
                    2 => result.push_str(&format!("{:02}", dt.month())),
                    3 => result.push_str(short_month_name(dt.month(), locale_code)),
                    4 => result.push_str(long_month_name(dt.month(), locale_code)),
                    5 => result.push(short_month_name(dt.month(), locale_code).chars().next().unwrap()),
                    _ => result.push_str(short_month_name(dt.month(), locale_code)),
                }
            }
            prev_token_was_hour = false;
        } else if upper == 'D' {
            let count = 1 + count_repeats(&mut chars, 'd', 'D');
            match count {
                1 => result.push_str(&dt.day().to_string()),
                2 => result.push_str(&format!("{:02}", dt.day())),
                3 => result.push_str(short_weekday_name(dt.weekday(), locale_code)),
                4 => result.push_str(long_weekday_name(dt.weekday(), locale_code)),
                _ => result.push_str(&format!("{:02}", dt.day())),
            }
            prev_token_was_hour = false;
        } else if upper == 'H' {
            let count = 1 + count_repeats(&mut chars, 'h', 'H');
            let dt = if use_elapsed_hour { current_dt } else { dt };
            let hour24 = dt.hour();
            let hour12 = if hour24 == 0 {
                12
            } else if hour24 > 12 {
                hour24 - 12
            } else {
                hour24
            };
            if count >= 2 {
                result.push_str(&format!("{:02}", hour24));
            } else {
                result.push_str(&hour12.to_string());
            }
            prev_token_was_hour = true;
        } else if upper == 'S' {
            let count = 1 + count_repeats(&mut chars, 's', 'S');
            let dt = if use_elapsed_hour { current_dt } else { dt };
            if count >= 2 {
                result.push_str(&format!("{:02}", dt.second()));
            } else {
                result.push_str(&dt.second().to_string());
            }
            prev_token_was_hour = false;
        } else if upper == 'A' {
            // AM/PM or A/P
            let ahead: String = chars.clone().take(4).collect();
            let ahead_upper = ahead.to_ascii_uppercase();
            if ahead_upper.starts_with("M/PM") {
                result.push_str(if dt.hour() < 12 { "AM" } else { "PM" });
                for _ in 0..4 {
                    chars.next();
                }
            } else if ahead_upper.starts_with("/P") {
                result.push(if dt.hour() < 12 { 'A' } else { 'P' });
                chars.next(); // '/'
                chars.next(); // 'P'
            } else {
                result.push(ch);
            }
            prev_token_was_hour = false;
        } else if ch == '0' {
            // Milliseconds: 0..000
            let count = 1 + count_repeats(&mut chars, '0', '0').min(2);
            let ms = dt.nanosecond() / 1_000_000;
            result.push_str(&format!("{:03}", ms)[..count]);
            prev_token_was_hour = false;
        } else {
            result.push(ch);
            // ':' and '/' are separators and do not break the hour→minute context.
            if ch != ':' && ch != '/' {
                prev_token_was_hour = false;
            }
        }
    }

    result
}

fn count_repeats(
    chars: &mut std::iter::Peekable<std::str::Chars<'_>>,
    lower: char,
    upper: char,
) -> usize {
    let mut count = 0;
    while let Some(&c) = chars.peek() {
        if c == lower || c == upper {
            chars.next();
            count += 1;
        } else {
            break;
        }
    }
    count
}

struct LocaleNames {
    short_months: [&'static str; 12],
    long_months: [&'static str; 12],
    short_weekdays: [&'static str; 7],
    long_weekdays: [&'static str; 7],
}

fn locale_names(locale_code: Option<&str>) -> &'static LocaleNames {
    const ENGLISH: LocaleNames = LocaleNames {
        short_months: ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"],
        long_months: ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"],
        short_weekdays: ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"],
        long_weekdays: ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"],
    };
    const ZH_CN: LocaleNames = LocaleNames {
        short_months: ["一月", "二月", "三月", "四月", "五月", "六月", "七月", "八月", "九月", "十月", "十一月", "十二月"],
        long_months: ["一月", "二月", "三月", "四月", "五月", "六月", "七月", "八月", "九月", "十月", "十一月", "十二月"],
        short_weekdays: ["周一", "周二", "周三", "周四", "周五", "周六", "周日"],
        long_weekdays: ["星期一", "星期二", "星期三", "星期四", "星期五", "星期六", "星期日"],
    };
    const ZH_TW: LocaleNames = LocaleNames {
        short_months: ["1月", "2月", "3月", "4月", "5月", "6月", "7月", "8月", "9月", "10月", "11月", "12月"],
        long_months: ["1月", "2月", "3月", "4月", "5月", "6月", "7月", "8月", "9月", "10月", "11月", "12月"],
        short_weekdays: ["週一", "週二", "週三", "週四", "週五", "週六", "週日"],
        long_weekdays: ["星期一", "星期二", "星期三", "星期四", "星期五", "星期六", "星期日"],
    };
    const JA_JP: LocaleNames = LocaleNames {
        short_months: ["1月", "2月", "3月", "4月", "5月", "6月", "7月", "8月", "9月", "10月", "11月", "12月"],
        long_months: ["1月", "2月", "3月", "4月", "5月", "6月", "7月", "8月", "9月", "10月", "11月", "12月"],
        short_weekdays: ["月", "火", "水", "木", "金", "土", "日"],
        long_weekdays: ["月曜日", "火曜日", "水曜日", "木曜日", "金曜日", "土曜日", "日曜日"],
    };
    const KO_KR: LocaleNames = LocaleNames {
        short_months: ["1월", "2월", "3월", "4월", "5월", "6월", "7월", "8월", "9월", "10월", "11월", "12월"],
        long_months: ["1월", "2월", "3월", "4월", "5월", "6월", "7월", "8월", "9월", "10월", "11월", "12월"],
        short_weekdays: ["월", "화", "수", "목", "금", "토", "일"],
        long_weekdays: ["월요일", "화요일", "수요일", "목요일", "금요일", "토요일", "일요일"],
    };
    const DE_DE: LocaleNames = LocaleNames {
        short_months: ["Jan", "Feb", "Mär", "Apr", "Mai", "Jun", "Jul", "Aug", "Sep", "Okt", "Nov", "Dez"],
        long_months: ["Januar", "Februar", "März", "April", "Mai", "Juni", "Juli", "August", "September", "Oktober", "November", "Dezember"],
        short_weekdays: ["Mo", "Di", "Mi", "Do", "Fr", "Sa", "So"],
        long_weekdays: ["Montag", "Dienstag", "Mittwoch", "Donnerstag", "Freitag", "Samstag", "Sonntag"],
    };
    const FR_FR: LocaleNames = LocaleNames {
        short_months: ["janv.", "févr.", "mars", "avr.", "mai", "juin", "juil.", "août", "sept.", "oct.", "nov.", "déc."],
        long_months: ["janvier", "février", "mars", "avril", "mai", "juin", "juillet", "août", "septembre", "octobre", "novembre", "décembre"],
        short_weekdays: ["lun.", "mar.", "mer.", "jeu.", "ven.", "sam.", "dim."],
        long_weekdays: ["lundi", "mardi", "mercredi", "jeudi", "vendredi", "samedi", "dimanche"],
    };
    const ES_ES: LocaleNames = LocaleNames {
        short_months: ["ene", "feb", "mar", "abr", "mayo", "jun", "jul", "ago", "sept", "oct", "nov", "dic"],
        long_months: ["enero", "febrero", "marzo", "abril", "mayo", "junio", "julio", "agosto", "septiembre", "octubre", "noviembre", "diciembre"],
        short_weekdays: ["lun", "mar", "mié", "jue", "vie", "sáb", "dom"],
        long_weekdays: ["lunes", "martes", "miércoles", "jueves", "viernes", "sábado", "domingo"],
    };
    const IT_IT: LocaleNames = LocaleNames {
        short_months: ["gen", "feb", "mar", "apr", "mag", "giu", "lug", "ago", "set", "ott", "nov", "dic"],
        long_months: ["gennaio", "febbraio", "marzo", "aprile", "maggio", "giugno", "luglio", "agosto", "settembre", "ottobre", "novembre", "dicembre"],
        short_weekdays: ["lun", "mar", "mer", "gio", "ven", "sab", "dom"],
        long_weekdays: ["lunedì", "martedì", "mercoledì", "giovedì", "venerdì", "sabato", "domenica"],
    };
    const RU_RU: LocaleNames = LocaleNames {
        short_months: ["янв", "фев", "мар", "апр", "май", "июн", "июл", "авг", "сен", "окт", "ноя", "дек"],
        long_months: ["январь", "февраль", "март", "апрель", "май", "июнь", "июль", "август", "сентябрь", "октябрь", "ноябрь", "декабрь"],
        short_weekdays: ["Пн", "Вт", "Ср", "Чт", "Пт", "Сб", "Вс"],
        long_weekdays: ["понедельник", "вторник", "среда", "четверг", "пятница", "суббота", "воскресенье"],
    };

    let code = locale_code.unwrap_or("").to_ascii_uppercase();
    // Excel locale IDs are hex strings; map by exact match or by primary language prefix.
    match code.as_str() {
        "804" | "1004" | "20804" => &ZH_CN,
        "404" | "1404" | "0C04" | "100C" | "140C" | "0404" => &ZH_TW,
        "411" | "10411" => &JA_JP,
        "412" | "10412" => &KO_KR,
        "407" => &DE_DE,
        "40C" => &FR_FR,
        "40A" => &ES_ES,
        "410" => &IT_IT,
        "419" => &RU_RU,
        _ => &ENGLISH,
    }
}

fn short_month_name(month: u32, locale_code: Option<&str>) -> &'static str {
    if month < 1 || month > 12 {
        return "";
    }
    locale_names(locale_code).short_months[(month - 1) as usize]
}

fn long_month_name(month: u32, locale_code: Option<&str>) -> &'static str {
    if month < 1 || month > 12 {
        return "";
    }
    locale_names(locale_code).long_months[(month - 1) as usize]
}

fn short_weekday_name(wd: chrono::Weekday, locale_code: Option<&str>) -> &'static str {
    let idx = wd.num_days_from_monday() as usize;
    locale_names(locale_code).short_weekdays[idx]
}

fn long_weekday_name(wd: chrono::Weekday, locale_code: Option<&str>) -> &'static str {
    let idx = wd.num_days_from_monday() as usize;
    locale_names(locale_code).long_weekdays[idx]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_built_in_map() {
        assert_eq!(built_in_num_fmt_code(14), Some("mm-dd-yy"));
        assert_eq!(built_in_num_fmt_code(0), Some("General"));
        assert_eq!(built_in_num_fmt_code(9999), None);
    }

    #[test]
    fn test_general() {
        assert_eq!(apply_number_format(1234.56, 0, None, false), "1234.56");
        assert_eq!(apply_number_format(100.0, 0, None, false), "100");
    }

    #[test]
    fn test_number() {
        assert_eq!(apply_number_format(1234.5, 1, None, false), "1235");
        assert_eq!(apply_number_format(1234.5, 2, None, false), "1234.50");
        assert_eq!(apply_number_format(1234.5, 3, None, false), "1,235");
        assert_eq!(apply_number_format(1234.5, 4, None, false), "1,234.50");
    }

    #[test]
    fn test_percentage() {
        assert_eq!(apply_number_format(0.1234, 9, None, false), "12%");
        assert_eq!(apply_number_format(0.1234, 10, None, false), "12.34%");
    }

    #[test]
    fn test_scientific() {
        assert_eq!(apply_number_format(12345.0, 11, None, false), "1.23E+04");
        // Format 48 (##0.0E+0) is rendered with standard scientific notation
        // in this implementation.
        assert_eq!(apply_number_format(12345.0, 48, None, false), "1.2E+04");
    }

    #[test]
    fn test_date() {
        let serial = 45486.0; // 2024-07-13
        assert_eq!(apply_number_format(serial, 14, None, false), "07-13-24");
        assert_eq!(apply_number_format(serial, 15, None, false), "13-Jul-24");
        assert_eq!(apply_number_format(serial, 16, None, false), "13-Jul");
        assert_eq!(apply_number_format(serial, 17, None, false), "Jul-24");
    }

    #[test]
    fn test_time() {
        let serial = 0.5; // 12:00:00
        assert_eq!(apply_number_format(serial, 18, None, false), "12:00 PM");
        assert_eq!(apply_number_format(serial, 20, None, false), "12:00");
        assert_eq!(apply_number_format(serial, 21, None, false), "12:00:00");
    }

    #[test]
    fn test_custom_currency() {
        assert_eq!(
            apply_number_format(1234.5, 164, Some("$#,##0.00"), false),
            "$1,234.50"
        );
    }

    #[test]
    fn test_custom_date() {
        let serial = 45486.0;
        assert_eq!(
            apply_number_format(serial, 164, Some("yyyy-mm-dd"), false),
            "2024-07-13"
        );
    }

    #[test]
    fn test_is_date_time_pattern() {
        assert!(is_date_time_pattern(""));
        assert!(is_date_time_pattern("yyyy-mm-dd"));
        assert!(is_date_time_pattern("hh:mm:ss"));
        assert!(is_date_time_pattern("[h]:mm:ss"));
        assert!(is_date_time_pattern("m/d/yyyy h:mm AM/PM"));
        assert!(is_date_time_pattern("[$-409]dddd, mmmm d, yyyy"));
        assert!(is_date_time_pattern("[red]yyyy-mm-dd"));
        assert!(is_date_time_pattern("\\yyyy-mm-dd"));
        assert!(is_date_time_pattern("\"Date: \"yyyy-mm-dd"));

        assert!(!is_date_time_pattern("0.00"));
        assert!(!is_date_time_pattern("#,##0"));
        assert!(!is_date_time_pattern("0%"));
        assert!(!is_date_time_pattern("0.00E+00"));
        assert!(!is_date_time_pattern("# ?/?"));
        assert!(!is_date_time_pattern("yyyy-mm-dd;0.00"));
        assert!(!is_date_time_pattern("@"));
        assert!(!is_date_time_pattern("yyyy-mm-dd;@"));
        assert!(!is_date_time_pattern("h:mm;@"));
        assert!(is_date_time_pattern("\"@\"yyyy-mm-dd"));
    }

    #[test]
    fn test_builtin_accounting_formats() {
        // Built-in accounting formats reserve a trailing space in the positive
        // section so that positive and negative values line up.
        assert_eq!(apply_number_format(1234.0, 37, None, false), "1,234 ");
        assert_eq!(apply_number_format(-1234.0, 37, None, false), "(1,234)");
        assert_eq!(apply_number_format(1234.0, 38, None, false), "1,234 ");
        assert_eq!(apply_number_format(-1234.0, 38, None, false), "(1,234)");
        assert_eq!(apply_number_format(1234.5, 39, None, false), "1,234.50 ");
        assert_eq!(apply_number_format(-1234.5, 39, None, false), "(1,234.50)");
        assert_eq!(apply_number_format(1234.5, 40, None, false), "1,234.50 ");
        assert_eq!(apply_number_format(-1234.5, 40, None, false), "(1,234.50)");
    }

    #[test]
    fn test_elapsed_time() {
        assert_eq!(format_number(1.5, "[h]", false), "36");
        assert_eq!(format_number(1.5, "[m]", false), "2160");
        assert_eq!(format_number(1.5, "[s]", false), "129600");
        assert_eq!(format_number(1.5, "[h]:mm:ss", false), "36:00:00");
    }

    #[test]
    fn test_locale_date_names() {
        let serial = 45486.0; // 2024-07-13 Saturday
        assert_eq!(
            format_number(serial, "[$-409]dddd, mmmm d, yyyy", false),
            "Saturday, July 13, 2024"
        );
        assert_eq!(
            format_number(serial, "[$-804]dddd, mmmm d, yyyy", false),
            "星期六, 七月 13, 2024"
        );
        assert_eq!(
            format_number(serial, "[$-411]dddd, mmmm d, yyyy", false),
            "土曜日, 7月 13, 2024"
        );
        assert_eq!(
            format_number(serial, "[$-412]dddd, mmmm d, yyyy", false),
            "토요일, 7월 13, 2024"
        );
    }

    #[test]
    fn test_dbnum_switch() {
        assert_eq!(format_number(1234.0, "[DBNum1]0", false), "一二三四");
        assert_eq!(format_number(1234.0, "[DBNum2]0", false), "壹贰叁肆");
        assert_eq!(format_number(1234.0, "[DBNum3]0", false), "１２３４");
    }

    #[test]
    fn test_currency_locale_brackets() {
        assert_eq!(
            format_number(1234.5, "[$$]#,##0.00", false),
            "$1,234.50"
        );
        assert_eq!(
            format_number(1234.5, "[$€-40C]#,##0.00", false),
            "€1,234.50"
        );
        assert_eq!(
            format_number(45486.0, "[$-409]yyyy-mm-dd", false),
            "2024-07-13"
        );
    }
}
