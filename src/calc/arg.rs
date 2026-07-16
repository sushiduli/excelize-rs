//! Formula argument representation.
//!
//! This mirrors the Go `formulaArg` struct so that the 450+ Excel formula
//! functions can be translated mechanically.

use std::fmt;

use crate::calc::CellRef;

/// Excel formula argument data type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArgType {
    Unknown,
    Number,
    String,
    List,
    Matrix,
    Error,
    Empty,
}

/// A single formula argument, value, or result.
#[derive(Debug, Clone)]
pub struct FormulaArg {
    pub sheet_name: String,
    pub number: f64,
    pub string: String,
    pub list: Vec<FormulaArg>,
    pub matrix: Vec<Vec<FormulaArg>>,
    pub boolean: bool,
    pub error: String,
    pub cell_refs: Vec<CellRef>,
    pub cell_ranges: Vec<(CellRef, CellRef)>,
    pub typ: ArgType,
}

impl Default for FormulaArg {
    fn default() -> Self {
        new_empty_formula_arg()
    }
}

/// Create an empty/unset formula argument.
pub fn new_empty_formula_arg() -> FormulaArg {
    FormulaArg {
        sheet_name: String::new(),
        number: 0.0,
        string: String::new(),
        list: Vec::new(),
        matrix: Vec::new(),
        boolean: false,
        error: String::new(),
        cell_refs: Vec::new(),
        cell_ranges: Vec::new(),
        typ: ArgType::Empty,
    }
}

/// Create a numeric formula argument.
pub fn new_number_formula_arg(n: f64) -> FormulaArg {
    FormulaArg {
        sheet_name: String::new(),
        number: n,
        string: String::new(),
        list: Vec::new(),
        matrix: Vec::new(),
        boolean: false,
        error: String::new(),
        cell_refs: Vec::new(),
        cell_ranges: Vec::new(),
        typ: ArgType::Number,
    }
}

/// Create a string formula argument.
pub fn new_string_formula_arg(s: impl Into<String>) -> FormulaArg {
    FormulaArg {
        sheet_name: String::new(),
        number: 0.0,
        string: s.into(),
        list: Vec::new(),
        matrix: Vec::new(),
        boolean: false,
        error: String::new(),
        cell_refs: Vec::new(),
        cell_ranges: Vec::new(),
        typ: ArgType::String,
    }
}

/// Create a boolean formula argument.  In Excel compatibility booleans are
/// stored as numbers with the boolean flag set.
pub fn new_bool_formula_arg(b: bool) -> FormulaArg {
    FormulaArg {
        sheet_name: String::new(),
        number: if b { 1.0 } else { 0.0 },
        string: String::new(),
        list: Vec::new(),
        matrix: Vec::new(),
        boolean: true,
        error: String::new(),
        cell_refs: Vec::new(),
        cell_ranges: Vec::new(),
        typ: ArgType::Number,
    }
}

/// Create an error formula argument.
pub fn new_error_formula_arg(err: impl Into<String>) -> FormulaArg {
    FormulaArg {
        sheet_name: String::new(),
        number: 0.0,
        string: String::new(),
        list: Vec::new(),
        matrix: Vec::new(),
        boolean: false,
        error: err.into(),
        cell_refs: Vec::new(),
        cell_ranges: Vec::new(),
        typ: ArgType::Error,
    }
}

/// Create a one-dimensional list argument.
pub fn new_list_formula_arg(list: Vec<FormulaArg>) -> FormulaArg {
    FormulaArg {
        sheet_name: String::new(),
        number: 0.0,
        string: String::new(),
        list,
        matrix: Vec::new(),
        boolean: false,
        error: String::new(),
        cell_refs: Vec::new(),
        cell_ranges: Vec::new(),
        typ: ArgType::List,
    }
}

/// Create a two-dimensional matrix argument.
pub fn new_matrix_formula_arg(matrix: Vec<Vec<FormulaArg>>) -> FormulaArg {
    FormulaArg {
        sheet_name: String::new(),
        number: 0.0,
        string: String::new(),
        list: Vec::new(),
        matrix,
        boolean: false,
        error: String::new(),
        cell_refs: Vec::new(),
        cell_ranges: Vec::new(),
        typ: ArgType::Matrix,
    }
}

impl FormulaArg {
    /// Return the Excel textual representation of this argument.
    pub fn value(&self) -> String {
        match self.typ {
            ArgType::Number => {
                if self.boolean {
                    if self.number == 0.0 {
                        "FALSE".to_string()
                    } else {
                        "TRUE".to_string()
                    }
                } else {
                    format!("{}", self.number)
                }
            }
            ArgType::String => self.string.clone(),
            ArgType::Matrix => {
                if let Some(first) = self.to_list().first() {
                    return first.value();
                }
                String::new()
            }
            ArgType::Error => self.error.clone(),
            _ => String::new(),
        }
    }

    /// Convert the argument to a number argument, returning `#VALUE!` on
    /// failure.
    pub fn to_number(&self) -> FormulaArg {
        match self.typ {
            ArgType::String => match self.string.parse::<f64>() {
                Ok(n) => new_number_formula_arg(n),
                Err(_) => new_error_formula_arg("#VALUE!"),
            },
            ArgType::Number => new_number_formula_arg(self.number),
            ArgType::Matrix => {
                if let Some(first) = self.to_list().first() {
                    return first.to_number();
                }
                new_number_formula_arg(0.0)
            }
            ArgType::Empty => new_number_formula_arg(0.0),
            _ => new_error_formula_arg("#VALUE!"),
        }
    }

    /// Convert the argument to a boolean argument.
    pub fn to_bool(&self) -> FormulaArg {
        match self.typ {
            ArgType::String => match self.string.to_lowercase().parse::<bool>() {
                Ok(b) => new_bool_formula_arg(b),
                Err(_) => new_error_formula_arg("#VALUE!"),
            },
            ArgType::Number => new_bool_formula_arg(self.number == 1.0),
            ArgType::Matrix => {
                if let Some(first) = self.to_list().first() {
                    return first.to_bool();
                }
                new_bool_formula_arg(false)
            }
            ArgType::Empty => new_bool_formula_arg(false),
            _ => new_error_formula_arg("#VALUE!"),
        }
    }

    /// Flatten a matrix/list/number/string/error to a one-dimensional list.
    pub fn to_list(&self) -> Vec<FormulaArg> {
        match self.typ {
            ArgType::Matrix => self
                .matrix
                .iter()
                .flat_map(|row| row.iter().cloned())
                .collect(),
            ArgType::List => self.list.clone(),
            ArgType::Empty => Vec::new(),
            _ => vec![self.clone()],
        }
    }

    /// Try to interpret the argument as a number.
    pub fn as_number(&self) -> Option<f64> {
        match self.typ {
            ArgType::Number => Some(self.number),
            ArgType::String => self.string.parse::<f64>().ok(),
            ArgType::Empty => Some(0.0),
            _ => None,
        }
    }

    /// Interpret the argument as a boolean.
    pub fn as_bool(&self) -> bool {
        match self.typ {
            ArgType::Empty => false,
            ArgType::Number => self.number != 0.0,
            ArgType::String => !self.string.is_empty(),
            ArgType::Error => false,
            ArgType::List => !self.list.is_empty(),
            ArgType::Matrix => !self.matrix.is_empty(),
            ArgType::Unknown => false,
        }
    }

    /// Convert the argument to its string representation.
    pub fn as_string(&self) -> String {
        match self.typ {
            ArgType::Number => {
                if self.number.is_nan() || self.number.is_infinite() {
                    return "#NUM!".to_string();
                }
                if self.boolean {
                    return self.value();
                }
                if self.number.fract() == 0.0 {
                    format!("{}", self.number as i64)
                } else {
                    format!("{}", self.number)
                }
            }
            ArgType::String => self.string.clone(),
            ArgType::Empty => String::new(),
            ArgType::Error => self.error.clone(),
            ArgType::List | ArgType::Matrix => String::new(),
            ArgType::Unknown => String::new(),
        }
    }

    /// Return `true` if this argument represents an Excel error.
    pub fn is_error(&self) -> bool {
        self.typ == ArgType::Error
    }
}

impl fmt::Display for FormulaArg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value())
    }
}

/// Collect all numeric values from a (possibly nested) argument into `out`,
/// propagating the first error found.  Returns the error argument if one is
/// encountered.
pub fn collect_numbers(arg: &FormulaArg, out: &mut Vec<f64>) -> Option<FormulaArg> {
    match arg.typ {
        ArgType::Number => out.push(arg.number),
        ArgType::List => {
            for x in &arg.list {
                if let Some(e) = collect_numbers(x, out) {
                    return Some(e);
                }
            }
        }
        ArgType::Matrix => {
            for row in &arg.matrix {
                for x in row {
                    if let Some(e) = collect_numbers(x, out) {
                        return Some(e);
                    }
                }
            }
        }
        ArgType::Error => return Some(arg.clone()),
        _ => {}
    }
    None
}

/// Extract all numbers from a slice of arguments, propagating errors.
pub fn numbers_from_args(args: &[FormulaArg]) -> (Vec<f64>, Option<FormulaArg>) {
    let mut out = Vec::new();
    for a in args {
        if let Some(e) = collect_numbers(a, &mut out) {
            return (out, Some(e));
        }
    }
    (out, None)
}

/// Flatten an argument to a single string.
pub fn flatten_string(arg: &FormulaArg) -> String {
    match arg.typ {
        ArgType::String => arg.string.clone(),
        ArgType::List => arg.list.iter().map(flatten_string).collect(),
        ArgType::Matrix => arg
            .matrix
            .iter()
            .flat_map(|r| r.iter().map(flatten_string))
            .collect(),
        _ => arg.as_string(),
    }
}

/// Compare two arguments for equality using Excel rules (numeric when
/// possible, otherwise case-insensitive string comparison).
pub fn compare_equal(a: &FormulaArg, b: &FormulaArg) -> bool {
    if let (Some(a), Some(b)) = (a.as_number(), b.as_number()) {
        return a == b;
    }
    a.as_string().to_uppercase() == b.as_string().to_uppercase()
}

/// Excel formula error constants.
pub const FORMULA_ERROR_DIV: &str = "#DIV/0!";
pub const FORMULA_ERROR_NAME: &str = "#NAME?";
pub const FORMULA_ERROR_NA: &str = "#N/A";
pub const FORMULA_ERROR_NUM: &str = "#NUM!";
pub const FORMULA_ERROR_VALUE: &str = "#VALUE!";
pub const FORMULA_ERROR_REF: &str = "#REF!";
pub const FORMULA_ERROR_NULL: &str = "#NULL!";
pub const FORMULA_ERROR_SPILL: &str = "#SPILL!";
pub const FORMULA_ERROR_CALC: &str = "#CALC!";
pub const FORMULA_ERROR_GETTING_DATA: &str = "#GETTING_DATA";
