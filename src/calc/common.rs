//! Shared helpers for the formula function submodules.
//!
//! Most helpers live in `crate::calc::arg`; this file is reserved for helpers
//! that are specific to the function-category submodules.

use crate::calc::arg::*;

/// Return the first error found in a slice of arguments, if any.
pub fn first_error(args: &[FormulaArg]) -> Option<FormulaArg> {
    args.iter().find(|a| a.is_error()).cloned()
}

/// Count the number of numeric values in a (possibly nested) argument.
pub fn count_numeric(arg: &FormulaArg) -> usize {
    match arg.typ {
        ArgType::Number => 1,
        ArgType::List => arg.list.iter().map(count_numeric).sum(),
        ArgType::Matrix => arg
            .matrix
            .iter()
            .flat_map(|r| r.iter())
            .map(count_numeric)
            .sum(),
        _ => 0,
    }
}
