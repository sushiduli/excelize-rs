//! Database formula functions.

use std::collections::HashMap;

use crate::calc::arg::*;
use crate::calc::{CalcContext, FormulaFn};

pub fn register(m: &mut HashMap<&'static str, FormulaFn>) {
    m.insert("DAVERAGE", daverage);
    m.insert("DCOUNT", dcount);
    m.insert("DCOUNTA", dcounta);
    m.insert("DGET", dget);
    m.insert("DMAX", dmax);
    m.insert("DMIN", dmin);
    m.insert("DPRODUCT", dproduct);
    m.insert("DSTDEV", dstdev);
    m.insert("DSTDEVP", dstdevp);
    m.insert("DSUM", dsum);
    m.insert("DVAR", dvar);
    m.insert("DVARP", dvarp);
    m.insert("DISPIMG", dispimg);
    m.insert("SORTBY", sortby);
}

struct CalcDatabase<'a> {
    database: &'a Vec<Vec<FormulaArg>>,
    criteria: &'a Vec<Vec<FormulaArg>>,
    col: i32,
    row: usize,
    index_map: Vec<Option<usize>>,
}

impl<'a> CalcDatabase<'a> {
    fn new(
        database: &'a FormulaArg,
        field: &'a FormulaArg,
        criteria: &'a FormulaArg,
    ) -> Option<Self> {
        let database = match &database.typ {
            ArgType::Matrix => &database.matrix,
            _ => return None,
        };
        let criteria = match &criteria.typ {
            ArgType::Matrix => &criteria.matrix,
            _ => return None,
        };
        if database.len() < 2
            || database[0].is_empty()
            || criteria.len() < 2
            || criteria[0].is_empty()
        {
            return None;
        }
        let col = if field.typ == ArgType::Empty {
            -1
        } else {
            Self::column_index(database, field)? as i32
        };
        Some(Self {
            database,
            criteria,
            col,
            row: 0,
            index_map: Vec::new(),
        })
    }

    fn column_index(database: &[Vec<FormulaArg>], field: &FormulaArg) -> Option<usize> {
        if let Some(n) = field.to_number().as_number() {
            let idx = n as usize;
            if idx > 0 && idx <= database[0].len() {
                return Some(idx - 1);
            }
            return None;
        }
        let name = field.value();
        for (i, title) in database[0].iter().enumerate() {
            if title.value().eq_ignore_ascii_case(&name) {
                return Some(i);
            }
        }
        None
    }

    fn criteria_eval(&mut self) -> bool {
        if self.index_map.is_empty() {
            self.index_map = vec![None; self.criteria[0].len()];
            for (j, field) in self.criteria[0].iter().enumerate() {
                if field.value().is_empty() {
                    continue;
                }
                self.index_map[j] = Self::column_index(self.database, field);
            }
        }

        for i in 1..self.criteria.len() {
            let mut matched = true;
            for (j, criteria_cell) in self.criteria[i].iter().enumerate() {
                if criteria_cell.value().is_empty() {
                    continue;
                }
                let col_idx = match self.index_map.get(j).copied().flatten() {
                    Some(idx) => idx,
                    None => {
                        matched = false;
                        break;
                    }
                };
                let cell = &self.database[self.row][col_idx];
                if !criteria_matches(cell, criteria_cell) {
                    matched = false;
                    break;
                }
            }
            if matched {
                return true;
            }
        }
        false
    }

    fn value(&self) -> FormulaArg {
        if self.col == -1 {
            let last = self.database[self.row].len() - 1;
            return self.database[self.row][last].clone();
        }
        self.database[self.row][self.col as usize].clone()
    }

    fn next(&mut self) -> bool {
        while self.row + 1 < self.database.len() {
            self.row += 1;
            if self.criteria_eval() {
                return true;
            }
        }
        false
    }
}

fn parse_criteria_text(criteria: &str) -> (String, f64) {
    let s = criteria.trim();
    if s.starts_with(">=") {
        if let Some(n) = s[2..].parse::<f64>().ok() {
            return (">=".to_string(), n);
        }
    } else if s.starts_with("<=") {
        if let Some(n) = s[2..].parse::<f64>().ok() {
            return ("<=".to_string(), n);
        }
    } else if s.starts_with("<>") {
        if let Some(n) = s[2..].parse::<f64>().ok() {
            return ("<>".to_string(), n);
        }
    } else if s.starts_with('>') {
        if let Some(n) = s[1..].parse::<f64>().ok() {
            return (">".to_string(), n);
        }
    } else if s.starts_with('<') {
        if let Some(n) = s[1..].parse::<f64>().ok() {
            return ("<".to_string(), n);
        }
    }
    if let Some(n) = s.parse::<f64>().ok() {
        return ("=".to_string(), n);
    }
    ("=".to_string(), 0.0)
}

fn criteria_matches(cell: &FormulaArg, criteria: &FormulaArg) -> bool {
    let criteria_text = criteria.value();
    let (op, target) = parse_criteria_text(&criteria_text);
    if let Some(n) = cell.to_number().as_number() {
        let matched = match op.as_str() {
            "=" => (n - target).abs() < 1e-12,
            "<>" => (n - target).abs() >= 1e-12,
            ">" => n > target,
            "<" => n < target,
            ">=" => n >= target,
            "<=" => n <= target,
            _ => false,
        };
        if matched {
            return true;
        }
    }
    // Fallback: string equality.
    cell.value().eq_ignore_ascii_case(&criteria_text)
}

fn collect_db(_name: &str, args: &[FormulaArg], op: DbOp) -> FormulaArg {
    if args.len() != 3 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let mut db = match CalcDatabase::new(&args[0], &args[1], &args[2]) {
        Some(db) => db,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let mut values: Vec<f64> = Vec::new();
    while db.next() {
        let v = db.value();
        if let Some(n) = v.to_number().as_number() {
            values.push(n);
        } else if matches!(op, DbOp::CountA) && v.typ != ArgType::Empty {
            values.push(0.0);
        }
    }

    match op {
        DbOp::Average => {
            if values.is_empty() {
                return new_error_formula_arg(FORMULA_ERROR_DIV);
            }
            new_number_formula_arg(values.iter().sum::<f64>() / values.len() as f64)
        }
        DbOp::Count | DbOp::CountA => new_number_formula_arg(values.len() as f64),
        DbOp::Max => {
            if let Some(&m) = values.iter().max_by(|a, b| a.partial_cmp(b).unwrap()) {
                new_number_formula_arg(m)
            } else {
                new_number_formula_arg(0.0)
            }
        }
        DbOp::Min => {
            if let Some(&m) = values.iter().min_by(|a, b| a.partial_cmp(b).unwrap()) {
                new_number_formula_arg(m)
            } else {
                new_number_formula_arg(0.0)
            }
        }
        DbOp::Product => new_number_formula_arg(values.iter().product()),
        DbOp::StDev | DbOp::StDevP | DbOp::Var | DbOp::VarP => {
            if values.is_empty() {
                return new_error_formula_arg(FORMULA_ERROR_DIV);
            }
            let n = values.len() as f64;
            let avg = values.iter().sum::<f64>() / n;
            let sum_sq = values.iter().map(|v| (v - avg).powi(2)).sum::<f64>();
            let denom = match op {
                DbOp::StDev | DbOp::Var => n - 1.0,
                _ => n,
            };
            if denom <= 0.0 {
                return new_error_formula_arg(FORMULA_ERROR_DIV);
            }
            let variance = sum_sq / denom;
            match op {
                DbOp::StDev | DbOp::StDevP => new_number_formula_arg(variance.sqrt()),
                _ => new_number_formula_arg(variance),
            }
        }
        DbOp::Sum => new_number_formula_arg(values.iter().sum()),
    }
}

#[derive(Clone, Copy)]
enum DbOp {
    Average,
    Count,
    CountA,
    Max,
    Min,
    Product,
    StDev,
    StDevP,
    Sum,
    Var,
    VarP,
}

fn daverage(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    collect_db("DAVERAGE", args, DbOp::Average)
}

fn dcount(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() < 2 || args.len() > 3 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    collect_db("DCOUNT", args, DbOp::Count)
}

fn dcounta(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() < 2 || args.len() > 3 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    collect_db("DCOUNTA", args, DbOp::CountA)
}

fn dget(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 3 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let mut db = match CalcDatabase::new(&args[0], &args[1], &args[2]) {
        Some(db) => db,
        None => return new_error_formula_arg(FORMULA_ERROR_VALUE),
    };
    let mut result: Option<FormulaArg> = None;
    while db.next() {
        if result.is_some() {
            return new_error_formula_arg(FORMULA_ERROR_NUM);
        }
        result = Some(db.value());
    }
    result.unwrap_or_else(|| new_error_formula_arg(FORMULA_ERROR_VALUE))
}

fn dmax(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    collect_db("DMAX", args, DbOp::Max)
}

fn dmin(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    collect_db("DMIN", args, DbOp::Min)
}

fn dproduct(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    collect_db("DPRODUCT", args, DbOp::Product)
}

fn dstdev(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    collect_db("DSTDEV", args, DbOp::StDev)
}

fn dstdevp(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    collect_db("DSTDEVP", args, DbOp::StDevP)
}

fn dsum(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    collect_db("DSUM", args, DbOp::Sum)
}

fn dvar(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    collect_db("DVAR", args, DbOp::Var)
}

fn dvarp(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    collect_db("DVARP", args, DbOp::VarP)
}

fn dispimg(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.len() != 2 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    args[0].clone()
}

fn sortby(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    // SORTBY(array, by_array1, [sort_order1], [by_array2, sort_order2], ...)
    // Accepts 2, 3, 5, or 7 arguments (up to three sort keys).
    if args.len() < 2 || args.len() > 7 || ![2, 3, 5, 7].contains(&args.len()) {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }

    // Parse the array to be sorted.
    let array = match args[0].typ {
        ArgType::Matrix => args[0].matrix.clone(),
        ArgType::List => args[0].list.iter().map(|x| vec![x.clone()]).collect(),
        ArgType::Empty => return new_error_formula_arg(FORMULA_ERROR_VALUE),
        ArgType::Error => return args[0].clone(),
        _ => vec![vec![args[0].clone()]],
    };
    if array.is_empty() {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let rows = array.len();

    // Parse each by_array / sort_order pair.
    struct SortKey {
        keys: Vec<Vec<FormulaArg>>,
        ascending: bool,
    }
    let mut sort_keys: Vec<SortKey> = Vec::new();
    let mut i = 1;
    while i < args.len() {
        let by_array = match args[i].typ {
            ArgType::Matrix => args[i].matrix.clone(),
            ArgType::List => args[i].list.iter().map(|x| vec![x.clone()]).collect(),
            ArgType::Empty => return new_error_formula_arg(FORMULA_ERROR_VALUE),
            ArgType::Error => return args[i].clone(),
            _ => vec![vec![args[i].clone()]],
        };
        if by_array.is_empty() {
            return new_error_formula_arg(FORMULA_ERROR_VALUE);
        }
        if by_array.len() != rows {
            return new_error_formula_arg(FORMULA_ERROR_VALUE);
        }

        let mut ascending = true;
        if i + 1 < args.len() {
            match args[i + 1].to_number().as_number() {
                Some(1.0) => ascending = true,
                Some(-1.0) => ascending = false,
                _ => return new_error_formula_arg(FORMULA_ERROR_VALUE),
            }
            i += 2;
        } else {
            i += 1;
        }

        sort_keys.push(SortKey {
            keys: by_array,
            ascending,
        });
    }

    // Stable sort row indices using the keys in order.
    let mut indices: Vec<usize> = (0..rows).collect();
    indices.sort_by(|&a, &b| {
        for key in &sort_keys {
            let lhs = &key.keys[a];
            let rhs = &key.keys[b];
            let min_cols = lhs.len().min(rhs.len());
            for col in 0..min_cols {
                let cmp = compare_sort_values(&lhs[col], &rhs[col]);
                if cmp != std::cmp::Ordering::Equal {
                    return if key.ascending { cmp } else { cmp.reverse() };
                }
            }
            if lhs.len() != rhs.len() {
                let cmp = lhs.len().cmp(&rhs.len());
                return if key.ascending { cmp } else { cmp.reverse() };
            }
        }
        std::cmp::Ordering::Equal
    });

    let result: Vec<Vec<FormulaArg>> = indices.into_iter().map(|idx| array[idx].clone()).collect();
    new_matrix_formula_arg(result)
}

/// Compare two formula arguments for sorting. Numeric values compare numerically;
/// non-numeric values compare by case-insensitive string value; errors sort last.
fn compare_sort_values(a: &FormulaArg, b: &FormulaArg) -> std::cmp::Ordering {
    if a.is_error() && b.is_error() {
        return a.error.cmp(&b.error);
    }
    if a.is_error() {
        return std::cmp::Ordering::Greater;
    }
    if b.is_error() {
        return std::cmp::Ordering::Less;
    }
    if let (Some(an), Some(bn)) = (a.as_number(), b.as_number()) {
        return an.partial_cmp(&bn).unwrap_or(std::cmp::Ordering::Equal);
    }
    a.as_string()
        .to_uppercase()
        .cmp(&b.as_string().to_uppercase())
}
