//! Formula calculation engine.
//!
//! This is a Rust port of the Go `calc.go` formula evaluator.  The parser and
//! evaluator are intentionally kept close to the original so that the 450+
//! Excel functions can be translated mechanically into the `src/calc/`
//! submodules.

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::OnceLock;

use crate::calc::arg::*;
use crate::cell::read_cell_value;
use crate::errors::Result;
use crate::file::File;
use crate::lib_util::{
    column_name_to_number, column_number_to_name, coordinates_to_cell_name, split_cell_name,
};
use crate::options::Options;
use crate::xml::shared_strings::XlsxSi;
use crate::xml::worksheet::{XlsxC, XlsxWorksheet};

pub mod arg;
pub mod common;
pub mod database;
pub mod date;
pub mod engineering;
pub mod financial;
pub mod info;
pub mod logical;
pub mod lookup;
pub mod math;
pub mod statistical;
pub mod text;
pub mod web;

// ------------------------------------------------------------------
// Public value types
// ------------------------------------------------------------------

/// A parsed cell reference.
#[derive(Debug, Clone)]
pub struct CellRef {
    /// Sheet name, if explicitly qualified.
    pub sheet: Option<String>,
    /// 1-based column number.
    pub col: i32,
    /// 1-based row number.
    pub row: i32,
    /// Column was absolute (`$A`).
    pub col_abs: bool,
    /// Row was absolute (`$1`).
    pub row_abs: bool,
    /// Whole-column reference (e.g. `A:A`).
    pub whole_col: bool,
    /// Whole-row reference (e.g. `1:1`).
    pub whole_row: bool,
}

impl CellRef {
    /// Return the canonical `A1` style name for this reference.
    pub fn to_cell_name(&self) -> String {
        coordinates_to_cell_name(self.col, self.row, false).unwrap_or_default()
    }
}

/// Evaluation context used to resolve references and detect circular
/// dependencies.
#[derive(Debug)]
pub struct CalcContext<'a> {
    pub file: &'a File,
    pub sheet: &'a str,
    pub cell: String,
    pub entry: String,
    pub stack: RefCell<Vec<(String, String)>>,
    pub max_calc_iterations: u32,
    pub iterations: RefCell<HashMap<String, u32>>,
    pub iterations_cache: RefCell<HashMap<String, FormulaArg>>,
    pub sheet_bounds: RefCell<HashMap<String, (i32, i32)>>,
}

impl<'a> CalcContext<'a> {
    pub fn new(file: &'a File, sheet: &'a str) -> Self {
        Self {
            file,
            sheet,
            cell: String::new(),
            entry: String::new(),
            stack: RefCell::new(Vec::new()),
            max_calc_iterations: 0,
            iterations: RefCell::new(HashMap::new()),
            iterations_cache: RefCell::new(HashMap::new()),
            sheet_bounds: RefCell::new(HashMap::new()),
        }
    }

    pub fn new_with_cell(file: &'a File, sheet: &'a str, cell: impl Into<String>) -> Self {
        Self {
            file,
            sheet,
            cell: cell.into(),
            entry: String::new(),
            stack: RefCell::new(Vec::new()),
            max_calc_iterations: 0,
            iterations: RefCell::new(HashMap::new()),
            iterations_cache: RefCell::new(HashMap::new()),
            sheet_bounds: RefCell::new(HashMap::new()),
        }
    }

    /// Return the maximum used row and column for a sheet, cached per context.
    pub fn worksheet_bounds(&self, sheet: &str) -> (i32, i32) {
        if let Some(bounds) = self.sheet_bounds.borrow().get(sheet).copied() {
            return bounds;
        }
        let mut max_row = 0;
        let mut max_col = 0;
        if let Ok(ws) = self.file.work_sheet_reader(sheet) {
            for row in &ws.sheet_data.row {
                if let Some(r) = row.r {
                    max_row = max_row.max(r as i32);
                }
                for c in &row.c {
                    if let Some(ref name) = c.r {
                        if let Ok((col_name, row_num)) = split_cell_name(name) {
                            max_row = max_row.max(row_num);
                            if let Ok(col_num) = column_name_to_number(&col_name) {
                                max_col = max_col.max(col_num);
                            }
                        }
                    }
                }
            }
        }
        let bounds = (max_row.max(1), max_col.max(1));
        self.sheet_bounds
            .borrow_mut()
            .insert(sheet.to_string(), bounds);
        bounds
    }
}

// ------------------------------------------------------------------
// AST
// ------------------------------------------------------------------

#[derive(Debug, Clone)]
pub(crate) enum Expr {
    Number(f64),
    String(String),
    Bool(bool),
    Cell(CellRef),
    Range(CellRef, CellRef),
    Range3D(String, String, CellRef, CellRef),
    Name(String),
    Array(Vec<Vec<Expr>>),
    Call(String, Vec<Expr>),
    Unary(String, Box<Expr>),
    Binary(String, Box<Expr>, Box<Expr>),
}

// ------------------------------------------------------------------
// Public parser entry point
// ------------------------------------------------------------------

pub(crate) fn parse_formula(formula: &str) -> Result<Expr> {
    let formula = formula.trim_start_matches('=').trim();
    if formula.is_empty() {
        return Ok(Expr::String(String::new()));
    }
    let mut parser = Parser::new(formula)?;
    let expr = parser.parse_expr()?;
    // Go's efp-based parser stops evaluating once a complete expression has
    // been consumed and silently ignores any trailing tokens.
    Ok(expr)
}

// ------------------------------------------------------------------
// Lexer
// ------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Number(f64),
    Text(String),
    Bool(bool),
    Ident(String),
    Op(String),
    LParen,
    RParen,
    Comma,
    Semicolon,
    LBrace,
    RBrace,
    Eof,
}

struct Lexer<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> Lexer<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    fn peek(&self) -> char {
        self.input[self.pos..].chars().next().unwrap_or('\0')
    }

    fn peek_next(&self) -> char {
        self.input[self.pos..].chars().nth(1).unwrap_or('\0')
    }

    fn advance(&mut self) -> char {
        let c = self.peek();
        self.pos += c.len_utf8();
        c
    }

    fn skip_ws(&mut self) {
        while self.peek().is_ascii_whitespace() {
            self.advance();
        }
    }

    /// Peek the next `n` tokens without advancing the lexer.
    fn peek_tokens(&self, n: usize) -> Result<Vec<Token>> {
        let mut tmp = Lexer::new(self.input);
        tmp.pos = self.pos;
        let mut tokens = Vec::with_capacity(n);
        for _ in 0..n {
            tokens.push(tmp.next_token()?);
        }
        Ok(tokens)
    }

    fn next_token(&mut self) -> Result<Token> {
        self.skip_ws();
        let c = self.peek();
        if c == '\0' {
            return Ok(Token::Eof);
        }

        if c == '"' {
            return self.read_string();
        }

        if c.is_ascii_digit() || (c == '.' && self.peek_next().is_ascii_digit()) {
            return self.read_number();
        }

        if c.is_ascii_alphabetic() || c == '_' || c == '$' {
            let ident = self.read_ident();
            let upper = ident.to_uppercase();
            if upper == "TRUE" {
                return Ok(Token::Bool(true));
            }
            if upper == "FALSE" {
                return Ok(Token::Bool(false));
            }
            return Ok(Token::Ident(ident));
        }

        self.advance();
        match c {
            '(' => Ok(Token::LParen),
            ')' => Ok(Token::RParen),
            '{' => Ok(Token::LBrace),
            '}' => Ok(Token::RBrace),
            ',' => Ok(Token::Comma),
            ';' => Ok(Token::Semicolon),
            '+' => Ok(Token::Op("+".to_string())),
            '-' => Ok(Token::Op("-".to_string())),
            '*' => Ok(Token::Op("*".to_string())),
            '/' => Ok(Token::Op("/".to_string())),
            '^' => Ok(Token::Op("^".to_string())),
            '&' => Ok(Token::Op("&".to_string())),
            '!' => Ok(Token::Op("!".to_string())),
            ':' => Ok(Token::Op(":".to_string())),
            '=' => Ok(Token::Op("=".to_string())),
            '<' => {
                if self.peek() == '=' {
                    self.advance();
                    Ok(Token::Op("<=".to_string()))
                } else if self.peek() == '>' {
                    self.advance();
                    Ok(Token::Op("<>".to_string()))
                } else {
                    Ok(Token::Op("<".to_string()))
                }
            }
            '>' => {
                if self.peek() == '=' {
                    self.advance();
                    Ok(Token::Op(">=".to_string()))
                } else {
                    Ok(Token::Op(">".to_string()))
                }
            }
            _ => Err(format!("unexpected character {:?} in formula", c).into()),
        }
    }

    fn read_number(&mut self) -> Result<Token> {
        let start = self.pos;
        let mut has_dot = false;
        let mut has_exp = false;
        while !self.input[self.pos..].is_empty() {
            let c = self.peek();
            if c.is_ascii_digit() {
                self.advance();
            } else if c == '.' && !has_dot && !has_exp {
                has_dot = true;
                self.advance();
            } else if (c == 'e' || c == 'E') && !has_exp {
                has_exp = true;
                self.advance();
                if self.peek() == '+' || self.peek() == '-' {
                    self.advance();
                }
            } else {
                break;
            }
        }
        let s = &self.input[start..self.pos];
        match s.parse::<f64>() {
            Ok(n) => Ok(Token::Number(n)),
            Err(_) => Err(format!("invalid numeric literal {}", s).into()),
        }
    }

    fn read_string(&mut self) -> Result<Token> {
        self.advance(); // opening quote
        let mut s = String::new();
        while !self.input[self.pos..].is_empty() {
            let c = self.advance();
            if c == '"' {
                if self.peek() == '"' {
                    self.advance();
                    s.push('"');
                } else {
                    break;
                }
            } else {
                s.push(c);
            }
        }
        Ok(Token::Text(s))
    }

    fn read_ident(&mut self) -> String {
        let start = self.pos;
        while !self.input[self.pos..].is_empty() {
            let c = self.peek();
            if c.is_ascii_alphanumeric() || c == '_' || c == '.' || c == '?' || c == '$' {
                self.advance();
            } else {
                break;
            }
        }
        self.input[start..self.pos].to_string()
    }
}

// ------------------------------------------------------------------
// Parser
// ------------------------------------------------------------------

struct Parser<'a> {
    lexer: Lexer<'a>,
    current: Token,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Result<Self> {
        let mut lexer = Lexer::new(input);
        let current = lexer.next_token()?;
        Ok(Self { lexer, current })
    }

    fn bump(&mut self) -> Result<()> {
        self.current = self.lexer.next_token()?;
        Ok(())
    }

    fn expect_rparen(&mut self) -> Result<()> {
        if !matches!(self.current, Token::RParen) {
            return Err(format!("expected ')' got {:?}", self.current).into());
        }
        self.bump()
    }

    fn parse_expr(&mut self) -> Result<Expr> {
        self.parse_compare()
    }

    fn parse_compare(&mut self) -> Result<Expr> {
        let mut left = self.parse_concat()?;
        if let Token::Op(ref op) = self.current {
            if is_comparison(op) {
                let op = op.clone();
                self.bump()?;
                let right = self.parse_concat()?;
                left = Expr::Binary(op, Box::new(left), Box::new(right));
            }
        }
        Ok(left)
    }

    fn parse_concat(&mut self) -> Result<Expr> {
        let mut left = self.parse_add_sub()?;
        while let Token::Op(ref op) = self.current {
            if op == "&" {
                self.bump()?;
                let right = self.parse_add_sub()?;
                left = Expr::Binary("&".to_string(), Box::new(left), Box::new(right));
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_add_sub(&mut self) -> Result<Expr> {
        let mut left = self.parse_mul_div()?;
        while let Token::Op(ref op) = self.current {
            if op == "+" || op == "-" {
                let op = op.clone();
                self.bump()?;
                let right = self.parse_mul_div()?;
                left = Expr::Binary(op, Box::new(left), Box::new(right));
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_mul_div(&mut self) -> Result<Expr> {
        let mut left = self.parse_power()?;
        while let Token::Op(ref op) = self.current {
            if op == "*" || op == "/" {
                let op = op.clone();
                self.bump()?;
                let right = self.parse_power()?;
                left = Expr::Binary(op, Box::new(left), Box::new(right));
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_power(&mut self) -> Result<Expr> {
        let left = self.parse_unary()?;
        if let Token::Op(ref op) = self.current {
            if op == "^" {
                let op = op.clone();
                self.bump()?;
                let right = self.parse_power()?;
                return Ok(Expr::Binary(op, Box::new(left), Box::new(right)));
            }
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr> {
        if let Token::Op(ref op) = self.current {
            if op == "+" || op == "-" {
                let op = op.clone();
                self.bump()?;
                let expr = self.parse_unary()?;
                return Ok(Expr::Unary(op, Box::new(expr)));
            }
        }
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<Expr> {
        match self.current.clone() {
            Token::Number(n) => {
                self.bump()?;
                // Whole-row reference, e.g. `1:1` or `3:5`.
                if n.fract() == 0.0 && n >= 1.0 {
                    if let Token::Op(ref op) = self.current {
                        if op == ":" {
                            let first = CellRef {
                                sheet: None,
                                col: 0,
                                row: n as i32,
                                col_abs: false,
                                row_abs: false,
                                whole_col: false,
                                whole_row: true,
                            };
                            return self.parse_multi_area_range(first);
                        }
                    }
                }
                Ok(Expr::Number(n))
            }
            Token::Text(s) => {
                self.bump()?;
                Ok(Expr::String(s))
            }
            Token::Bool(b) => {
                self.bump()?;
                // Excel also accepts TRUE() and FALSE() as zero-argument functions.
                if let Token::LParen = self.current {
                    self.bump()?;
                    self.expect_rparen()?;
                    let name = if b { "TRUE" } else { "FALSE" }.to_string();
                    return Ok(Expr::Call(name, Vec::new()));
                }
                Ok(Expr::Bool(b))
            }
            Token::LParen => {
                self.bump()?;
                let expr = self.parse_expr()?;
                self.expect_rparen()?;
                Ok(expr)
            }
            Token::LBrace => self.parse_array(),
            Token::Ident(name) => self.parse_ident_expr(name),
            ref t => Err(format!("unexpected token {:?}", t).into()),
        }
    }

    fn parse_array(&mut self) -> Result<Expr> {
        self.bump()?; // consume '{'
        let mut rows: Vec<Vec<Expr>> = vec![Vec::new()];
        loop {
            if matches!(self.current, Token::RBrace) {
                self.bump()?;
                break;
            }
            let expr = self.parse_expr()?;
            rows.last_mut().unwrap().push(expr);
            match self.current {
                Token::Comma => self.bump()?,
                Token::Semicolon => {
                    self.bump()?;
                    rows.push(Vec::new());
                }
                Token::RBrace => {
                    self.bump()?;
                    break;
                }
                _ => return Err("expected ',', ';' or '}' in array literal".into()),
            }
        }
        Ok(Expr::Array(rows))
    }

    fn parse_ident_expr(&mut self, name: String) -> Result<Expr> {
        self.bump()?; // consume identifier

        // Function call: NAME(...)
        if let Token::LParen = self.current {
            self.bump()?;
            let mut args = Vec::new();
            if !matches!(self.current, Token::RParen) {
                loop {
                    args.push(self.parse_expr()?);
                    if let Token::Comma = self.current {
                        self.bump()?;
                        continue;
                    }
                    break;
                }
            }
            self.expect_rparen()?;
            return Ok(Expr::Call(normalize_function_name(&name), args));
        }

        // 3D reference: Sheet1:Sheet2!A1 or Sheet1:Sheet2!A1:B2
        if let Token::Op(ref op) = self.current {
            if op == ":" {
                let peek = self.lexer.peek_tokens(2)?;
                if let [Token::Ident(last_sheet), Token::Op(bang)] = peek.as_slice() {
                    if bang == "!" {
                        let last = last_sheet.clone();
                        self.bump()?; // ':'
                        self.bump()?; // last sheet
                        self.bump()?; // '!'
                        let expr = self.parse_expr()?;
                        let (start, end) = match expr {
                            Expr::Cell(r) => (r.clone(), r),
                            Expr::Range(s, e) => (s, e),
                            _ => {
                                return Err(
                                    "expected cell or range reference in 3D reference".into()
                                );
                            }
                        };
                        return Ok(Expr::Range3D(name, last, start, end));
                    }
                }
            }
        }

        // Sheet-qualified reference: Sheet!A1 or Sheet!1:1
        let first_ref = if let Token::Op(ref op) = self.current {
            if op == "!" {
                self.bump()?;
                let sheet = Some(name.clone());
                match self.current.clone() {
                    Token::Ident(n) => {
                        self.bump()?;
                        parse_cell_ref(&n, sheet)?
                    }
                    Token::Number(n) if n.fract() == 0.0 && n >= 1.0 => {
                        self.bump()?;
                        CellRef {
                            sheet,
                            col: 0,
                            row: n as i32,
                            col_abs: false,
                            row_abs: false,
                            whole_col: false,
                            whole_row: true,
                        }
                    }
                    t => {
                        return Err(
                            format!("expected cell reference after '!', got {:?}", t).into()
                        );
                    }
                }
            } else {
                match parse_cell_ref(&name, None) {
                    Ok(r) => r,
                    Err(_) => {
                        if op == ":" {
                            return Err(format!("invalid cell reference {}", name).into());
                        }
                        return Ok(Expr::Name(name));
                    }
                }
            }
        } else {
            match parse_cell_ref(&name, None) {
                Ok(r) => r,
                Err(_) => return Ok(Expr::Name(name)),
            }
        };

        // Range reference: A1:B2, Sheet!A1:B2, or multi-area A1:B1:C1.
        if let Token::Op(ref op) = self.current {
            if op == ":" {
                return self.parse_multi_area_range(first_ref);
            }
        }

        Ok(Expr::Cell(first_ref))
    }

    /// Parse a multi-area reference starting with `first`. Subsequent refs may be
    /// cell references (`A1`), optionally sheet-qualified (`Sheet!A1`), or whole
    /// rows for row-only references (`1:1`). The result is a single bounding range.
    fn parse_multi_area_range(&mut self, first: CellRef) -> Result<Expr> {
        let mut refs = vec![first];
        while let Token::Op(ref op) = self.current {
            if op != ":" {
                break;
            }
            self.bump()?;
            let (next_sheet, cell_name, is_row_number) = if let Token::Ident(n) =
                self.current.clone()
            {
                self.bump()?;
                if let Token::Op(ref op) = self.current {
                    if op == "!" {
                        self.bump()?;
                        let cell = match self.current.clone() {
                            Token::Ident(c) => c,
                            t => {
                                return Err(format!(
                                    "expected cell reference after '!', got {:?}",
                                    t
                                )
                                .into());
                            }
                        };
                        self.bump()?;
                        (Some(n), cell, false)
                    } else {
                        (None, n, false)
                    }
                } else {
                    (None, n, false)
                }
            } else if let Token::Number(n) = self.current {
                self.bump()?;
                if n.fract() == 0.0 && n >= 1.0 {
                    (None, (n as i32).to_string(), true)
                } else {
                    return Err("expected integer row reference".into());
                }
            } else {
                return Err(
                    format!("expected cell reference after ':', got {:?}", self.current).into(),
                );
            };
            let inherited = next_sheet.or_else(|| refs[0].sheet.clone());
            let cell_ref = if is_row_number {
                CellRef {
                    sheet: inherited,
                    col: 0,
                    row: cell_name.parse::<i32>().unwrap(),
                    col_abs: false,
                    row_abs: false,
                    whole_col: false,
                    whole_row: true,
                }
            } else {
                parse_cell_ref(&cell_name, inherited)?
            };
            refs.push(cell_ref);
        }
        let (start, end) = build_bounding_range(&refs)?;
        Ok(Expr::Range(start, end))
    }
}

fn is_comparison(op: &str) -> bool {
    matches!(op, "=" | "<>" | "<" | ">" | "<=" | ">=")
}

/// Build a single bounding range from a list of cell references joined by `:`.
/// This replicates Go's excelize behavior for multi-area references such as
/// `A1:B1:C1` or `Sheet1!A1:Sheet1!A1:A2`: the result is the smallest range
/// that contains every referenced cell, not a true union of disjoint areas.
fn build_bounding_range(refs: &[CellRef]) -> Result<(CellRef, CellRef)> {
    use crate::constants::{MAX_COLUMNS, TOTAL_ROWS};
    if refs.is_empty() {
        return Err("empty reference list".into());
    }
    let sheet = refs[0].sheet.clone();
    if refs.iter().any(|r| r.sheet != sheet) {
        return Err("multi-sheet reference is not supported".into());
    }

    let mut all_whole_col = true;
    let mut all_whole_row = true;
    let mut any_ref = false;
    let mut min_col = i32::MAX;
    let mut max_col = 0;
    let mut min_row = i32::MAX;
    let mut max_row = 0;

    for r in refs {
        if r.whole_col {
            all_whole_row = false;
            any_ref = true;
            min_col = min_col.min(r.col);
            max_col = max_col.max(r.col);
            min_row = min_row.min(1);
            max_row = max_row.max(TOTAL_ROWS as i32);
        } else if r.whole_row {
            all_whole_col = false;
            any_ref = true;
            min_row = min_row.min(r.row);
            max_row = max_row.max(r.row);
            min_col = min_col.min(1);
            max_col = max_col.max(MAX_COLUMNS as i32);
        } else {
            all_whole_col = false;
            all_whole_row = false;
            any_ref = true;
            min_col = min_col.min(r.col);
            max_col = max_col.max(r.col);
            min_row = min_row.min(r.row);
            max_row = max_row.max(r.row);
        }
    }

    if !any_ref {
        return Err("no valid references".into());
    }

    let start = CellRef {
        sheet: sheet.clone(),
        col: min_col,
        row: min_row,
        col_abs: false,
        row_abs: false,
        whole_col: all_whole_col,
        whole_row: all_whole_row,
    };
    let end = CellRef {
        sheet,
        col: max_col,
        row: max_row,
        col_abs: false,
        row_abs: false,
        whole_col: all_whole_col,
        whole_row: all_whole_row,
    };
    Ok((start, end))
}

fn parse_cell_ref(name: &str, sheet: Option<String>) -> Result<CellRef> {
    let without_dollar = name.replace('$', "");

    // Whole-column reference (e.g. `A:A`).
    if without_dollar.chars().all(|c| c.is_ascii_alphabetic()) {
        let col = column_name_to_number(&without_dollar)
            .map_err(|e| format!("invalid column {}: {}", name, e))?;
        return Ok(CellRef {
            sheet,
            col,
            row: 0,
            col_abs: name.starts_with('$'),
            row_abs: false,
            whole_col: true,
            whole_row: false,
        });
    }

    // Whole-row reference (e.g. `1:1`).
    if without_dollar.chars().all(|c| c.is_ascii_digit()) {
        let row = without_dollar
            .parse::<i32>()
            .map_err(|_| format!("invalid row {}: {}", name, without_dollar))?;
        return Ok(CellRef {
            sheet,
            col: 0,
            row,
            col_abs: false,
            row_abs: name.starts_with('$'),
            whole_col: false,
            whole_row: true,
        });
    }

    let (col_name, row) = split_cell_name(&without_dollar)
        .map_err(|e| format!("invalid cell reference {}: {}", name, e))?;
    let col = column_name_to_number(&col_name)
        .map_err(|e| format!("invalid column {}: {}", col_name, e))?;

    let col_abs = name.starts_with('$');
    let row_abs = {
        let without_leading = name.strip_prefix('$').unwrap_or(name);
        without_leading.contains('$')
    };

    Ok(CellRef {
        sheet,
        col,
        row,
        col_abs,
        row_abs,
        whole_col: false,
        whole_row: false,
    })
}

/// Normalize an Excel function name to the registry key used by this port.
/// Replicates Go's `formulaFnNameReplacer`: removes `_xlfn.` and replaces
/// `.` with `dot`.
pub(crate) fn normalize_function_name(name: &str) -> String {
    name.to_uppercase()
        .replace("_XLFN.", "")
        .replace('.', "dot")
}

// ------------------------------------------------------------------
// Function registry
// ------------------------------------------------------------------

pub(crate) type FormulaFn = fn(&CalcContext, &[FormulaArg]) -> FormulaArg;

fn build_registry() -> HashMap<&'static str, FormulaFn> {
    let mut m: HashMap<&'static str, FormulaFn> = HashMap::with_capacity(512);

    // Common functions already implemented in this module.
    m.insert("IF", calc_if);

    // Math and trigonometric functions.
    math::register(&mut m);
    // Statistical functions.
    statistical::register(&mut m);
    // Engineering functions.
    engineering::register(&mut m);
    // Logical functions.
    logical::register(&mut m);
    // Information functions.
    info::register(&mut m);
    // Lookup and reference functions.
    lookup::register(&mut m);
    // Text functions.
    text::register(&mut m);
    // Date and time functions.
    date::register(&mut m);
    // Financial functions.
    financial::register(&mut m);
    // Database functions.
    database::register(&mut m);
    // Web and miscellaneous functions.
    web::register(&mut m);

    m
}

fn formula_registry() -> &'static HashMap<&'static str, FormulaFn> {
    static REGISTRY: OnceLock<HashMap<&'static str, FormulaFn>> = OnceLock::new();
    REGISTRY.get_or_init(build_registry)
}

// ------------------------------------------------------------------
// Evaluation
// ------------------------------------------------------------------

fn eval(ctx: &CalcContext, expr: &Expr) -> FormulaArg {
    match expr {
        Expr::Number(n) => new_number_formula_arg(*n),
        Expr::String(s) => new_string_formula_arg(s.clone()),
        Expr::Bool(b) => new_bool_formula_arg(*b),
        Expr::Cell(r) => eval_cell_ref(ctx, r),
        Expr::Range(start, end) => eval_range(ctx, start, end),
        Expr::Range3D(s1, s2, start, end) => eval_range_3d(ctx, s1, s2, start, end),
        Expr::Name(name) => eval_name(ctx, name),
        Expr::Array(rows) => eval_array(ctx, rows),
        Expr::Call(name, args) => eval_call(ctx, name, args),
        Expr::Unary(op, e) => eval_unary(op, eval(ctx, e)),
        Expr::Binary(op, l, r) => eval_binary(op, eval(ctx, l), eval(ctx, r)),
    }
}

fn eval_call(ctx: &CalcContext, name: &str, args: &[Expr]) -> FormulaArg {
    let evaluated: Vec<FormulaArg> = args.iter().map(|e| eval_arg(ctx, e)).collect();
    dispatch_function(ctx, name, &evaluated)
}

/// Evaluate a single argument expression.  If the top-level expression is a
/// cell or range reference, the resulting `FormulaArg` keeps that metadata so
/// reference-aware functions such as `ISREF`, `ROW`, `COLUMN`, and
/// `FORMULATEXT` can inspect it.
fn eval_arg(ctx: &CalcContext, expr: &Expr) -> FormulaArg {
    let mut arg = eval(ctx, expr);
    match expr {
        Expr::Cell(r) => arg.cell_refs.push(r.clone()),
        Expr::Range(start, end) => arg.cell_ranges.push((start.clone(), end.clone())),
        _ => {}
    }
    arg
}

fn eval_array(ctx: &CalcContext, rows: &[Vec<Expr>]) -> FormulaArg {
    let mut matrix = Vec::new();
    for row in rows {
        let mut line = Vec::new();
        for e in row {
            let v = eval(ctx, e);
            if v.is_error() {
                return v;
            }
            line.push(v);
        }
        matrix.push(line);
    }
    new_matrix_formula_arg(matrix)
}

fn eval_range_3d(
    ctx: &CalcContext,
    first_sheet: &str,
    last_sheet: &str,
    start: &CellRef,
    end: &CellRef,
) -> FormulaArg {
    let sheets = match ctx.file.expand_3d_sheet_range(first_sheet, last_sheet) {
        Ok(v) => v,
        Err(_) => return new_error_formula_arg(FORMULA_ERROR_REF),
    };
    let mut matrix = Vec::new();
    for sheet in sheets {
        let mut start_ref = start.clone();
        let mut end_ref = end.clone();
        start_ref.sheet = Some(sheet.clone());
        end_ref.sheet = Some(sheet.clone());
        let arg = eval_range(ctx, &start_ref, &end_ref);
        if arg.is_error() {
            return arg;
        }
        match arg.typ {
            ArgType::Matrix => matrix.extend(arg.matrix),
            _ => matrix.push(vec![arg]),
        }
    }
    new_matrix_formula_arg(matrix)
}

fn eval_name(ctx: &CalcContext, name: &str) -> FormulaArg {
    let ref_to = ctx.file.get_defined_name_ref_to(name, ctx.sheet);
    if ref_to.is_empty() {
        return new_error_formula_arg(FORMULA_ERROR_NAME);
    }
    let formula = ref_to.trim_start_matches('=').trim();
    if formula.is_empty() {
        return new_error_formula_arg(FORMULA_ERROR_NAME);
    }
    match parse_formula(formula) {
        Ok(expr) => eval(ctx, &expr),
        Err(_) => new_error_formula_arg(FORMULA_ERROR_NAME),
    }
}

fn dispatch_function(ctx: &CalcContext, name: &str, args: &[FormulaArg]) -> FormulaArg {
    let registry = formula_registry();
    // Registry stores Excel dotted function names with `dot` replacing `.`
    // (e.g. `BETA.DIST` is registered as `BETAdotDIST`).
    let key = name.replace('.', "dot");
    match registry.get(key.as_str()) {
        Some(f) => f(ctx, args),
        None => new_error_formula_arg(FORMULA_ERROR_NAME),
    }
}

fn calc_if(_ctx: &CalcContext, args: &[FormulaArg]) -> FormulaArg {
    if args.is_empty() || args.len() > 3 {
        return new_error_formula_arg(FORMULA_ERROR_VALUE);
    }
    let cond = match args[0].typ {
        ArgType::String => match args[0].string.parse::<bool>() {
            Ok(b) => b,
            Err(_) => return new_error_formula_arg(FORMULA_ERROR_VALUE),
        },
        ArgType::Number => args[0].number == 1.0,
        _ => args[0].as_bool(),
    };
    if args.len() == 1 {
        return new_bool_formula_arg(cond);
    }
    if cond {
        if args[1].typ == ArgType::Number {
            args[1].to_number()
        } else {
            new_string_formula_arg(args[1].value())
        }
    } else if args.len() == 2 {
        new_bool_formula_arg(false)
    } else if args[2].typ == ArgType::Number {
        args[2].to_number()
    } else {
        new_string_formula_arg(args[2].value())
    }
}

fn eval_unary(op: &str, arg: FormulaArg) -> FormulaArg {
    if arg.is_error() {
        return arg;
    }
    match op {
        "+" => arg,
        "-" => match arg.to_number().as_number() {
            Some(n) => new_number_formula_arg(-n),
            None => new_error_formula_arg(FORMULA_ERROR_VALUE),
        },
        _ => new_error_formula_arg(FORMULA_ERROR_VALUE),
    }
}

fn eval_binary(op: &str, left: FormulaArg, right: FormulaArg) -> FormulaArg {
    if left.is_error() {
        return left;
    }
    if right.is_error() {
        return right;
    }
    match op {
        "+" => numeric_op(left, right, |a, b| a + b),
        "-" => numeric_op(left, right, |a, b| a - b),
        "*" => numeric_op(left, right, |a, b| a * b),
        "/" => {
            let a = left.to_number().as_number();
            let b = right.to_number().as_number();
            match (a, b) {
                (Some(_), Some(0.0)) => new_error_formula_arg(FORMULA_ERROR_DIV),
                (Some(x), Some(y)) => new_number_formula_arg(x / y),
                _ => new_error_formula_arg(FORMULA_ERROR_VALUE),
            }
        }
        "^" => numeric_op(left, right, |a, b| a.powf(b)),
        "&" => new_string_formula_arg(format!("{}{}", left.value(), right.value())),
        "=" => new_bool_formula_arg(compare_equal(&left, &right)),
        "<>" => new_bool_formula_arg(!compare_equal(&left, &right)),
        "<" | ">" | "<=" | ">=" => comparison_op(op, left, right),
        _ => new_error_formula_arg(FORMULA_ERROR_VALUE),
    }
}

fn numeric_op<F: FnOnce(f64, f64) -> f64>(left: FormulaArg, right: FormulaArg, f: F) -> FormulaArg {
    match (left.to_number().as_number(), right.to_number().as_number()) {
        (Some(a), Some(b)) => new_number_formula_arg(f(a, b)),
        _ => new_error_formula_arg(FORMULA_ERROR_VALUE),
    }
}

fn comparison_op(op: &str, left: FormulaArg, right: FormulaArg) -> FormulaArg {
    fn as_number(arg: &FormulaArg) -> Option<f64> {
        match arg.typ {
            ArgType::Number => Some(arg.number),
            ArgType::Empty => Some(0.0),
            _ => None,
        }
    }

    // Two numeric operands (including empty cells and booleans, which are
    // stored as numbers) compare by value.
    if let (Some(a), Some(b)) = (as_number(&left), as_number(&right)) {
        let res = match op {
            "<" => a < b,
            ">" => a > b,
            "<=" => a <= b,
            ">=" => a >= b,
            _ => false,
        };
        return new_bool_formula_arg(res);
    }

    // Mixed number/text: numbers are always less than text strings.
    let number_vs_text = |number_on_left: bool| match op {
        "<" | "<=" => number_on_left,
        ">" | ">=" => !number_on_left,
        _ => false,
    };
    if left.typ == ArgType::Number && right.typ == ArgType::String {
        return new_bool_formula_arg(number_vs_text(true));
    }
    if left.typ == ArgType::String && right.typ == ArgType::Number {
        return new_bool_formula_arg(number_vs_text(false));
    }

    // Text vs text (or any other mix) compares the displayed values.
    let a = left.value().to_uppercase();
    let b = right.value().to_uppercase();
    let res = match op {
        "<" => a < b,
        ">" => a > b,
        "<=" => a <= b,
        ">=" => a >= b,
        _ => false,
    };
    new_bool_formula_arg(res)
}

fn eval_cell_ref(ctx: &CalcContext, reference: &CellRef) -> FormulaArg {
    let cell_name = reference.to_cell_name();
    let sheet = reference.sheet.as_deref().unwrap_or(ctx.sheet);
    let ref_key = format!("{}!{}", sheet, cell_name);

    // Whole-column/whole-row references on the active sheet may be defined
    // names. Resolve them when the cell name itself is not a valid A1 address.
    if reference.sheet.is_none() && (reference.whole_col || reference.whole_row) {
        let name = if reference.whole_col {
            column_number_to_name(reference.col).unwrap_or_default()
        } else {
            reference.row.to_string()
        };
        let ref_to = ctx.file.get_defined_name_ref_to(&name, ctx.sheet);
        if !ref_to.is_empty() {
            return eval_name(ctx, &ref_to);
        }
    }

    // Match Go's excelize behavior for the top-level formula cell: when a
    // formula references its own cell, return the raw stored value instead of
    // evaluating the formula recursively. This avoids circular-reference
    // errors for cases like `=A1+(B1-C1)` in C1.
    if sheet == ctx.sheet && cell_name == ctx.cell {
        let ws = match ctx.file.work_sheet_reader(sheet) {
            Ok(ws) => ws,
            Err(_) => return new_error_formula_arg(FORMULA_ERROR_REF),
        };
        return if let Some(c) = find_cell(&ws, &cell_name) {
            cell_to_arg(ctx.file, &c)
        } else {
            new_empty_formula_arg()
        };
    }

    if ctx
        .stack
        .borrow()
        .contains(&(sheet.to_string(), cell_name.clone()))
    {
        return new_error_formula_arg(FORMULA_ERROR_REF);
    }

    let ws = match ctx.file.work_sheet_reader(sheet) {
        Ok(ws) => ws,
        Err(_) => return new_error_formula_arg(FORMULA_ERROR_REF),
    };

    let cell = find_cell(&ws, &cell_name);
    let result = if let Some(ref c) = cell {
        if let Some(f) = &c.f {
            let formula = f.content.trim_start_matches('=').trim();
            if !formula.is_empty() {
                // Reuse cached argument values for non-entry cells.
                if ref_key != ctx.entry {
                    if let Some(cached) = ctx
                        .file
                        .formula_arg_cache
                        .lock()
                        .unwrap()
                        .get(&ref_key)
                        .cloned()
                    {
                        return cached;
                    }
                }

                // Iterative calculation for cells that are not the entry point.
                if ctx.max_calc_iterations > 0 && ref_key != ctx.entry {
                    let mut iterations = ctx.iterations.borrow_mut();
                    let count = iterations.entry(ref_key.clone()).or_insert(0);
                    if *count <= ctx.max_calc_iterations {
                        *count += 1;
                        drop(iterations);
                        ctx.stack
                            .borrow_mut()
                            .push((sheet.to_string(), cell_name.clone()));
                        let res = match parse_formula(formula) {
                            Ok(expr) => eval(ctx, &expr),
                            Err(_) => new_error_formula_arg(FORMULA_ERROR_VALUE),
                        };
                        ctx.stack.borrow_mut().pop();
                        ctx.iterations_cache
                            .borrow_mut()
                            .insert(ref_key.clone(), res.clone());
                        ctx.file
                            .formula_arg_cache
                            .lock()
                            .unwrap()
                            .insert(ref_key.clone(), res.clone());
                        return res;
                    }
                    return ctx
                        .iterations_cache
                        .borrow()
                        .get(&ref_key)
                        .cloned()
                        .unwrap_or_else(new_empty_formula_arg);
                }

                ctx.stack
                    .borrow_mut()
                    .push((sheet.to_string(), cell_name.clone()));
                let res = match parse_formula(formula) {
                    Ok(expr) => eval(ctx, &expr),
                    Err(_) => new_error_formula_arg(FORMULA_ERROR_VALUE),
                };
                ctx.stack.borrow_mut().pop();
                if ref_key != ctx.entry {
                    ctx.file
                        .formula_arg_cache
                        .lock()
                        .unwrap()
                        .insert(ref_key.clone(), res.clone());
                }
                res
            } else {
                cell_to_arg(ctx.file, c)
            }
        } else {
            cell_to_arg(ctx.file, c)
        }
    } else {
        new_empty_formula_arg()
    };

    result
}

fn eval_range(ctx: &CalcContext, start: &CellRef, end: &CellRef) -> FormulaArg {
    let sheet = start.sheet.as_deref().unwrap_or(ctx.sheet);
    let (max_row, max_col) = ctx.worksheet_bounds(sheet);
    let (col1, col2, row1, row2) = if start.whole_col && end.whole_col {
        let c1 = start.col.min(end.col);
        let c2 = start.col.max(end.col);
        (c1, c2, 1, max_row)
    } else if start.whole_row && end.whole_row {
        let r1 = start.row.min(end.row);
        let r2 = start.row.max(end.row);
        (1, max_col, r1, r2)
    } else {
        let c1 = start.col.min(end.col);
        let c2 = start.col.max(end.col);
        let r1 = start.row.min(end.row);
        let r2 = start.row.max(end.row);
        (c1, c2, r1, r2)
    };

    let mut matrix = Vec::new();
    for row in row1..=row2 {
        let mut line = Vec::new();
        for col in col1..=col2 {
            let reference = CellRef {
                sheet: Some(sheet.to_string()),
                col,
                row,
                col_abs: false,
                row_abs: false,
                whole_col: false,
                whole_row: false,
            };
            let value = eval_cell_ref(ctx, &reference);
            if value.is_error() {
                return value;
            }
            line.push(value);
        }
        matrix.push(line);
    }
    new_matrix_formula_arg(matrix)
}

fn find_cell(ws: &XlsxWorksheet, cell: &str) -> Option<XlsxC> {
    for row in &ws.sheet_data.row {
        for c in &row.c {
            if c.r
                .as_deref()
                .map(|r| r.eq_ignore_ascii_case(cell))
                .unwrap_or(false)
            {
                return Some(c.clone());
            }
        }
    }
    None
}

fn cell_to_arg(file: &File, c: &XlsxC) -> FormulaArg {
    match c.t.as_deref() {
        Some("s") => {
            if let Some(v) = &c.v {
                if let Ok(idx) = v.parse::<i32>() {
                    return new_string_formula_arg(read_shared_string(file, idx));
                }
            }
            new_empty_formula_arg()
        }
        Some("inlineStr") => new_string_formula_arg(inline_string_text(c.is.as_ref())),
        Some("b") => new_bool_formula_arg(c.v.as_deref() == Some("1")),
        Some("str") => new_string_formula_arg(c.v.clone().unwrap_or_default()),
        _ => {
            if let Some(v) = &c.v {
                if let Ok(n) = v.parse::<f64>() {
                    new_number_formula_arg(n)
                } else {
                    new_string_formula_arg(v.clone())
                }
            } else {
                new_empty_formula_arg()
            }
        }
    }
}

fn read_shared_string(file: &File, idx: i32) -> String {
    let sst = file.shared_strings_reader().unwrap_or_default();
    sst.si
        .get(idx as usize)
        .map(|si| string_from_si(si))
        .unwrap_or_default()
}

fn inline_string_text(si: Option<&XlsxSi>) -> String {
    si.map(string_from_si).unwrap_or_default()
}

fn string_from_si(si: &XlsxSi) -> String {
    if let Some(t) = &si.t {
        return t.val.clone();
    }
    si.r.iter()
        .filter_map(|r| r.t.as_ref().map(|t| t.val.clone()))
        .collect()
}

// ------------------------------------------------------------------
// File extension: CalcCellValue equivalent
// ------------------------------------------------------------------

impl File {
    /// Calculate the value of a single cell.  If the cell contains a formula,
    /// the formula is parsed and evaluated; otherwise the cached/raw value is
    /// returned. Uses the [`Options`] stored on the workbook.
    pub fn calc_cell_value(&self, sheet: &str, cell: &str) -> Result<String> {
        let opts = self.options.lock().unwrap().clone();
        self.calc_cell_value_with_options(sheet, cell, &opts)
    }

    /// Calculate the value of a single cell with an explicit set of options.
    pub fn calc_cell_value_with_options(
        &self,
        sheet: &str,
        cell: &str,
        opts: &Options,
    ) -> Result<String> {
        let cell_upper = cell.to_uppercase();
        let entry = format!("{}!{}", sheet, cell_upper);
        if opts.raw_cell_value {
            if let Some(v) = self.calc_raw_cache.lock().unwrap().get(&entry).cloned() {
                return Ok(v);
            }
        } else {
            if let Some(v) = self.calc_cache.lock().unwrap().get(&entry).cloned() {
                return Ok(v);
            }
        }

        let ws = self.work_sheet_reader(sheet)?;
        let c = find_cell(&ws, &cell_upper);

        let token = if let Some(c) = c {
            if let Some(f) = &c.f {
                let formula = f.content.trim_start_matches('=').trim();
                if !formula.is_empty() {
                    let mut ctx = CalcContext::new_with_cell(self, sheet, cell_upper.clone());
                    ctx.entry = entry.clone();
                    ctx.max_calc_iterations = opts.max_calc_iterations;
                    ctx.stack
                        .borrow_mut()
                        .push((sheet.to_string(), cell_upper.clone()));
                    let expr = parse_formula(formula)?;
                    let result = eval(&ctx, &expr);
                    ctx.stack.borrow_mut().pop();
                    result
                } else {
                    cell_to_arg(self, &c)
                }
            } else {
                cell_to_arg(self, &c)
            }
        } else {
            new_empty_formula_arg()
        };

        let style_idx = if opts.raw_cell_value {
            0
        } else {
            self.get_cell_style(sheet, &cell_upper).unwrap_or(0)
        };

        let result = format_calc_result(self, &token, style_idx, opts.raw_cell_value)?;
        if opts.raw_cell_value {
            self.calc_raw_cache
                .lock()
                .unwrap()
                .insert(entry, result.clone());
        } else {
            self.calc_cache
                .lock()
                .unwrap()
                .insert(entry, result.clone());
        }
        Ok(result)
    }
}

/// Format a calculated formula result into the textual value returned by
/// [`File::calc_cell_value`], applying the cell's number format unless raw
/// values were requested.
fn format_calc_result(
    file: &File,
    token: &FormulaArg,
    style_idx: i32,
    raw: bool,
) -> Result<String> {
    if token.typ == ArgType::Number && !token.boolean {
        let mut c = XlsxC::default();
        c.s = Some(style_idx as i64);
        c.v = Some(format_number_for_calc(token.number));
        Ok(read_cell_value(file, &c, raw))
    } else {
        let mut c = XlsxC::default();
        c.t = Some("str".to_string());
        c.v = Some(token.value());
        Ok(read_cell_value(file, &c, raw))
    }
}

/// Format a numeric result the way Go `calcCellValue` does: integers are
/// printed without a decimal point, very large/small values use uppercase
/// scientific notation, and everything else uses Rust's default fixed-point
/// representation.
fn format_number_for_calc(n: f64) -> String {
    if n.is_nan() || n.is_infinite() {
        return n.to_string();
    }
    if n.fract() == 0.0 && n >= i64::MIN as f64 && n <= i64::MAX as f64 {
        return format!("{}", n as i64);
    }
    let abs = n.abs();
    if abs >= 1e15 || (abs > 0.0 && abs < 1e-15) {
        format!("{:.14e}", n).to_uppercase()
    } else {
        format!("{}", n)
    }
}

// ------------------------------------------------------------------
// Tests
// ------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn new_file() -> File {
        File::new_with_options(crate::options::Options::default())
    }

    #[test]
    fn parse_numeric_literal() {
        let expr = parse_formula("123.45").unwrap();
        assert!(matches!(expr, Expr::Number(n) if (n - 123.45).abs() < 1e-9));
    }

    #[test]
    fn parse_string_literal() {
        let expr = parse_formula("\"hello\"").unwrap();
        assert!(matches!(expr, Expr::String(s) if s == "hello"));
    }

    #[test]
    fn parse_boolean_literals() {
        let expr = parse_formula("TRUE").unwrap();
        assert!(matches!(expr, Expr::Bool(true)));
        let expr = parse_formula("FALSE").unwrap();
        assert!(matches!(expr, Expr::Bool(false)));
    }

    #[test]
    fn parse_cell_reference() {
        let expr = parse_formula("$A$1").unwrap();
        match expr {
            Expr::Cell(r) => {
                assert_eq!(r.col, 1);
                assert_eq!(r.row, 1);
                assert!(r.col_abs);
                assert!(r.row_abs);
            }
            _ => panic!("expected cell reference"),
        }
    }

    #[test]
    fn parse_sheet_qualified_reference() {
        let expr = parse_formula("Sheet1!B2").unwrap();
        match expr {
            Expr::Cell(r) => {
                assert_eq!(r.sheet.as_deref(), Some("Sheet1"));
                assert_eq!(r.col, 2);
                assert_eq!(r.row, 2);
            }
            _ => panic!("expected qualified reference"),
        }
    }

    #[test]
    fn parse_range_reference() {
        let expr = parse_formula("A1:B3").unwrap();
        match expr {
            Expr::Range(start, end) => {
                assert_eq!(start.col, 1);
                assert_eq!(start.row, 1);
                assert_eq!(end.col, 2);
                assert_eq!(end.row, 3);
            }
            _ => panic!("expected range"),
        }
    }

    #[test]
    fn parse_function_call() {
        let expr = parse_formula("SUM(A1:A3, 5)").unwrap();
        match expr {
            Expr::Call(name, args) => {
                assert_eq!(name, "SUM");
                assert_eq!(args.len(), 2);
            }
            _ => panic!("expected function call"),
        }
    }

    #[test]
    fn parse_binary_operators() {
        let expr = parse_formula("1+2*3^2").unwrap();
        match expr {
            Expr::Binary(op, _, _) => assert_eq!(op, "+"),
            _ => panic!("expected binary expression"),
        }
    }

    #[test]
    fn calc_sum_range() {
        let f = new_file();
        f.set_cell_int("Sheet1", "A1", 1).unwrap();
        f.set_cell_int("Sheet1", "A2", 2).unwrap();
        f.set_cell_int("Sheet1", "A3", 3).unwrap();
        f.set_cell_formula("Sheet1", "A4", "SUM(A1:A3)").unwrap();
        assert_eq!(f.calc_cell_value("Sheet1", "A4").unwrap(), "6");
    }

    #[test]
    fn calc_average_and_count() {
        let f = new_file();
        f.set_cell_int("Sheet1", "B1", 10).unwrap();
        f.set_cell_int("Sheet1", "B2", 20).unwrap();
        f.set_cell_int("Sheet1", "B3", 30).unwrap();
        f.set_cell_formula("Sheet1", "B4", "AVERAGE(B1:B3)")
            .unwrap();
        f.set_cell_formula("Sheet1", "B5", "COUNT(B1:B3)").unwrap();
        assert_eq!(f.calc_cell_value("Sheet1", "B4").unwrap(), "20");
        assert_eq!(f.calc_cell_value("Sheet1", "B5").unwrap(), "3");
    }

    #[test]
    fn calc_max_min() {
        let f = new_file();
        f.set_cell_int("Sheet1", "C1", 5).unwrap();
        f.set_cell_int("Sheet1", "C2", 9).unwrap();
        f.set_cell_int("Sheet1", "C3", 2).unwrap();
        f.set_cell_formula("Sheet1", "C4", "MAX(C1:C3)").unwrap();
        f.set_cell_formula("Sheet1", "C5", "MIN(C1:C3)").unwrap();
        assert_eq!(f.calc_cell_value("Sheet1", "C4").unwrap(), "9");
        assert_eq!(f.calc_cell_value("Sheet1", "C5").unwrap(), "2");
    }

    #[test]
    fn calc_if() {
        let f = new_file();
        f.set_cell_int("Sheet1", "D1", 5).unwrap();
        f.set_cell_formula("Sheet1", "D2", "IF(D1>3,\"yes\",\"no\")")
            .unwrap();
        f.set_cell_formula("Sheet1", "D3", "IF(D1<3,\"yes\",\"no\")")
            .unwrap();
        assert_eq!(f.calc_cell_value("Sheet1", "D2").unwrap(), "yes");
        assert_eq!(f.calc_cell_value("Sheet1", "D3").unwrap(), "no");
    }

    #[test]
    fn calc_vlookup_exact() {
        let f = new_file();
        f.set_cell_int("Sheet1", "A1", 1).unwrap();
        f.set_cell_str("Sheet1", "B1", "one").unwrap();
        f.set_cell_int("Sheet1", "A2", 2).unwrap();
        f.set_cell_str("Sheet1", "B2", "two").unwrap();
        f.set_cell_int("Sheet1", "A3", 3).unwrap();
        f.set_cell_str("Sheet1", "B3", "three").unwrap();
        f.set_cell_formula("Sheet1", "C1", "VLOOKUP(2,A1:B3,2,FALSE)")
            .unwrap();
        assert_eq!(f.calc_cell_value("Sheet1", "C1").unwrap(), "two");
    }

    #[test]
    fn calc_concatenate_and_ampersand() {
        let f = new_file();
        f.set_cell_str("Sheet1", "E1", "Hello").unwrap();
        f.set_cell_str("Sheet1", "E2", "World").unwrap();
        f.set_cell_formula("Sheet1", "E3", "CONCATENATE(E1,\" \",E2)")
            .unwrap();
        f.set_cell_formula("Sheet1", "E4", "E1&\" \"&E2").unwrap();
        assert_eq!(f.calc_cell_value("Sheet1", "E3").unwrap(), "Hello World");
        assert_eq!(f.calc_cell_value("Sheet1", "E4").unwrap(), "Hello World");
    }

    #[test]
    fn calc_abs_and_round() {
        let f = new_file();
        f.set_cell_formula("Sheet1", "F1", "ABS(-12.5)").unwrap();
        f.set_cell_formula("Sheet1", "F2", "ROUND(3.14159,2)")
            .unwrap();
        assert_eq!(f.calc_cell_value("Sheet1", "F1").unwrap(), "12.5");
        assert_eq!(f.calc_cell_value("Sheet1", "F2").unwrap(), "3.14");
    }

    #[test]
    fn calc_cross_sheet_reference() {
        let f = new_file();
        f.set_cell_int("Sheet1", "G1", 42).unwrap();
        f.set_cell_formula("Sheet1", "G2", "Sheet1!G1+8").unwrap();
        assert_eq!(f.calc_cell_value("Sheet1", "G2").unwrap(), "50");
    }

    #[test]
    fn calc_today_and_now_are_serials() {
        let f = new_file();
        f.set_cell_formula("Sheet1", "H1", "TODAY()").unwrap();
        f.set_cell_formula("Sheet1", "H2", "NOW()").unwrap();
        let today = f.calc_cell_value("Sheet1", "H1").unwrap();
        assert!(today.parse::<f64>().is_ok(), "TODAY() returned {:?}", today);
        assert!(today.parse::<f64>().unwrap() > 0.0);

        let now = f.calc_cell_value("Sheet1", "H2").unwrap();
        assert!(now.parse::<f64>().is_ok(), "NOW() returned {:?}", now);
        assert!(now.parse::<f64>().unwrap() > 0.0);
    }

    #[test]
    fn calc_unsupported_function_returns_name_error() {
        let f = new_file();
        f.set_cell_formula("Sheet1", "I1", "NOTSUPPORTED(1)")
            .unwrap();
        assert_eq!(f.calc_cell_value("Sheet1", "I1").unwrap(), "#NAME?");
    }

    #[test]
    fn calc_non_formula_cell_falls_back() {
        let f = new_file();
        f.set_cell_str("Sheet1", "J1", "plain").unwrap();
        assert_eq!(f.calc_cell_value("Sheet1", "J1").unwrap(), "plain");
    }

    #[test]
    fn calc_indirect() {
        let f = new_file();
        f.set_cell_int("Sheet1", "A1", 42).unwrap();
        f.set_cell_formula("Sheet1", "B1", "INDIRECT(\"A1\")")
            .unwrap();
        assert_eq!(f.calc_cell_value("Sheet1", "B1").unwrap(), "42");

        f.set_cell_formula("Sheet1", "C1", "INDIRECT(\"R1C1\",FALSE)")
            .unwrap();
        assert_eq!(f.calc_cell_value("Sheet1", "C1").unwrap(), "42");
    }

    #[test]
    fn calc_formulatext_and_isref() {
        let f = new_file();
        f.set_cell_formula("Sheet1", "A1", "1+2").unwrap();
        f.set_cell_formula("Sheet1", "B1", "FORMULATEXT(A1)")
            .unwrap();
        assert_eq!(f.calc_cell_value("Sheet1", "B1").unwrap(), "=1+2");

        f.set_cell_formula("Sheet1", "C1", "ISREF(A1)").unwrap();
        f.set_cell_formula("Sheet1", "D1", "ISREF(\"A1\")").unwrap();
        assert_eq!(f.calc_cell_value("Sheet1", "C1").unwrap(), "TRUE");
        assert_eq!(f.calc_cell_value("Sheet1", "D1").unwrap(), "FALSE");
    }

    #[test]
    fn calc_column_row_sheet_sheets() {
        let f = new_file();
        f.new_sheet("Sheet2").unwrap();
        f.set_cell_formula("Sheet1", "A1", "COLUMN(B3)").unwrap();
        f.set_cell_formula("Sheet1", "A2", "ROW(B3)").unwrap();
        f.set_cell_formula("Sheet1", "A3", "SHEETS()").unwrap();
        f.set_cell_formula("Sheet1", "A4", "SHEET(Sheet1!A1)")
            .unwrap();
        f.set_cell_formula("Sheet1", "A5", "SHEET(Sheet2!A1)")
            .unwrap();
        assert_eq!(f.calc_cell_value("Sheet1", "A1").unwrap(), "2");
        assert_eq!(f.calc_cell_value("Sheet1", "A2").unwrap(), "3");
        assert_eq!(f.calc_cell_value("Sheet1", "A3").unwrap(), "2");
        let sheet1: i32 = f.calc_cell_value("Sheet1", "A4").unwrap().parse().unwrap();
        let sheet2: i32 = f.calc_cell_value("Sheet1", "A5").unwrap().parse().unwrap();
        assert!(sheet1 < sheet2);
    }

    #[test]
    fn calc_norm_dist_and_sortby() {
        let f = new_file();
        f.set_cell_formula("Sheet1", "A1", "NORM.DIST(0,0,1,FALSE)")
            .unwrap();
        let v: f64 = f.calc_cell_value("Sheet1", "A1").unwrap().parse().unwrap();
        assert!((v - 0.398_942_280_4).abs() < 1e-6);

        f.set_cell_int("Sheet1", "C1", 3).unwrap();
        f.set_cell_int("Sheet1", "C2", 1).unwrap();
        f.set_cell_int("Sheet1", "C3", 2).unwrap();
        f.set_cell_int("Sheet1", "D1", 30).unwrap();
        f.set_cell_int("Sheet1", "D2", 10).unwrap();
        f.set_cell_int("Sheet1", "D3", 20).unwrap();
        f.set_cell_formula("Sheet1", "E1", "SORTBY(C1:C3,D1:D3)")
            .unwrap();
        assert_eq!(f.calc_cell_value("Sheet1", "E1").unwrap(), "1");
    }

    #[test]
    fn calc_array_literal() {
        let f = new_file();
        f.set_cell_formula("Sheet1", "A1", "SUM({1,2;3,4})")
            .unwrap();
        assert_eq!(f.calc_cell_value("Sheet1", "A1").unwrap(), "10");
    }

    #[test]
    fn calc_3d_reference() {
        let f = new_file();
        f.new_sheet("Sheet2").unwrap();
        f.new_sheet("Sheet3").unwrap();
        f.set_cell_int("Sheet1", "A1", 1).unwrap();
        f.set_cell_int("Sheet2", "A1", 10).unwrap();
        f.set_cell_int("Sheet3", "A1", 100).unwrap();
        f.set_cell_formula("Sheet1", "B1", "SUM(Sheet1:Sheet3!A1)")
            .unwrap();
        assert_eq!(f.calc_cell_value("Sheet1", "B1").unwrap(), "111");
    }

    #[test]
    fn calc_defined_name() {
        let f = new_file();
        f.set_cell_int("Sheet1", "A1", 5).unwrap();
        f.set_cell_int("Sheet1", "A2", 7).unwrap();
        f.set_defined_name(&crate::xml::workbook::DefinedName {
            name: "MyRange".to_string(),
            refers_to: "A1:A2".to_string(),
            ..Default::default()
        })
        .unwrap();
        f.set_cell_formula("Sheet1", "B1", "SUM(MyRange)").unwrap();
        assert_eq!(f.calc_cell_value("Sheet1", "B1").unwrap(), "12");
    }

    #[test]
    fn calc_iterative_circular() {
        let mut opts = crate::options::Options::default();
        opts.max_calc_iterations = 5;
        let f = File::new_with_options(opts);
        f.set_cell_formula("Sheet1", "A1", "A2+1").unwrap();
        f.set_cell_formula("Sheet1", "A2", "A1+1").unwrap();
        assert_eq!(f.calc_cell_value("Sheet1", "A1").unwrap(), "2");
    }

    #[test]
    fn calc_raw_cell_value_cache() {
        let mut opts = crate::options::Options::default();
        opts.raw_cell_value = true;
        let f = File::new_with_options(opts);
        f.set_cell_formula("Sheet1", "A1", "1+1").unwrap();
        assert_eq!(f.calc_cell_value("Sheet1", "A1").unwrap(), "2");
        // set_cell_value does not clear the calc cache, so the cached result
        // should still be returned until the cache is explicitly cleared.
        f.set_cell_int("Sheet1", "A1", 99).unwrap();
        assert_eq!(f.calc_cell_value("Sheet1", "A1").unwrap(), "2");
        f.clear_calc_cache();
        assert_eq!(f.calc_cell_value("Sheet1", "A1").unwrap(), "99");
    }

    #[test]
    fn calc_iferror_arity() {
        let f = new_file();
        f.set_cell_formula("Sheet1", "A1", "IFERROR(1,2,3)")
            .unwrap();
        assert_eq!(f.calc_cell_value("Sheet1", "A1").unwrap(), "#VALUE!");
    }

    #[test]
    fn calc_sumif_text_numbers() {
        let f = new_file();
        f.set_cell_str("Sheet1", "A1", "1").unwrap();
        f.set_cell_str("Sheet1", "A2", "2").unwrap();
        f.set_cell_int("Sheet1", "A3", 3).unwrap();
        f.set_cell_formula("Sheet1", "A4", "SUMIF(A1:A3,\">0\")")
            .unwrap();
        assert_eq!(f.calc_cell_value("Sheet1", "A4").unwrap(), "6");
    }

    #[test]
    fn calc_text_formats() {
        let f = new_file();

        // Common numeric formats.
        f.set_cell_formula("Sheet1", "A1", "TEXT(1234.5,\"#,##0.00\")")
            .unwrap();
        assert_eq!(f.calc_cell_value("Sheet1", "A1").unwrap(), "1,234.50");

        // Underscore alignment: positive section leaves a space the width of ')'.
        f.set_cell_formula("Sheet1", "A2", "TEXT(1234.5,\"0.00_);(0.00)\")")
            .unwrap();
        assert_eq!(f.calc_cell_value("Sheet1", "A2").unwrap(), "1234.50 ");

        // Three-section format picks the positive section.
        f.set_cell_formula("Sheet1", "A3", "TEXT(1234.5,\"0.00;[Red](0.00);zero\")")
            .unwrap();
        assert_eq!(f.calc_cell_value("Sheet1", "A3").unwrap(), "1234.50");

        // Zero section.
        f.set_cell_formula("Sheet1", "A4", "TEXT(0,\"0.00;[Red](0.00);zero\")")
            .unwrap();
        assert_eq!(f.calc_cell_value("Sheet1", "A4").unwrap(), "zero");

        // Fixed-width integer pattern.
        f.set_cell_formula("Sheet1", "A5", "TEXT(1234,\"0000\")")
            .unwrap();
        assert_eq!(f.calc_cell_value("Sheet1", "A5").unwrap(), "1234");

        // Four-section format: text value uses the fourth section.
        f.set_cell_formula("Sheet1", "A6", "TEXT(\"abc\",\"0.00;[Red](0.00);zero;@\")")
            .unwrap();
        assert_eq!(f.calc_cell_value("Sheet1", "A6").unwrap(), "abc");

        // Go-compatible baseline cases.
        f.set_cell_formula("Sheet1", "A7", "TEXT(\"07/07/2015\",\"mm/dd/yyyy\")")
            .unwrap();
        assert_eq!(f.calc_cell_value("Sheet1", "A7").unwrap(), "07/07/2015");
        f.set_cell_formula("Sheet1", "A8", "TEXT(42192,\"mm/dd/yyyy\")")
            .unwrap();
        assert_eq!(f.calc_cell_value("Sheet1", "A8").unwrap(), "07/07/2015");
        f.set_cell_formula("Sheet1", "A9", "TEXT(42192,\"mmm dd yyyy\")")
            .unwrap();
        assert_eq!(f.calc_cell_value("Sheet1", "A9").unwrap(), "Jul 07 2015");
        f.set_cell_formula("Sheet1", "A10", "TEXT(0.75,\"hh:mm\")")
            .unwrap();
        assert_eq!(f.calc_cell_value("Sheet1", "A10").unwrap(), "18:00");
        f.set_cell_formula("Sheet1", "A11", "TEXT(36.363636,\"0.00\")")
            .unwrap();
        assert_eq!(f.calc_cell_value("Sheet1", "A11").unwrap(), "36.36");
        f.set_cell_formula("Sheet1", "A12", "TEXT(567.9,\"$#,##0.00\")")
            .unwrap();
        assert_eq!(f.calc_cell_value("Sheet1", "A12").unwrap(), "$567.90");
        f.set_cell_formula(
            "Sheet1",
            "A13",
            "TEXT(-5,\"+ $#,##0.00;- $#,##0.00;$0.00\")",
        )
        .unwrap();
        assert_eq!(f.calc_cell_value("Sheet1", "A13").unwrap(), "- $5.00");
        f.set_cell_formula("Sheet1", "A14", "TEXT(5,\"+ $#,##0.00;- $#,##0.00;$0.00\")")
            .unwrap();
        assert_eq!(f.calc_cell_value("Sheet1", "A14").unwrap(), "+ $5.00");
    }

    #[test]
    fn calc_text_conditional_formats() {
        let f = new_file();

        // First conditional section matches.
        f.set_cell_formula("Sheet1", "A1", "TEXT(150,\"[>100]0.00;0.0\")")
            .unwrap();
        assert_eq!(f.calc_cell_value("Sheet1", "A1").unwrap(), "150.00");

        // Value does not meet the condition, use the default section.
        f.set_cell_formula("Sheet1", "A2", "TEXT(50,\"[>100]0.00;0.0\")")
            .unwrap();
        assert_eq!(f.calc_cell_value("Sheet1", "A2").unwrap(), "50.0");

        // Multiple conditional sections with text-only bodies.
        f.set_cell_formula(
            "Sheet1",
            "A3",
            "TEXT(85,\"[>=90]\"\"A\"\";[>=60]\"\"B\"\";\"\"C\"\"\")",
        )
        .unwrap();
        assert_eq!(f.calc_cell_value("Sheet1", "A3").unwrap(), "B");

        // Color metadata is stripped; negative default section keeps parentheses.
        f.set_cell_formula("Sheet1", "A4", "TEXT(-10,\"[>0]0.00;[Red](0.00)\")")
            .unwrap();
        assert_eq!(f.calc_cell_value("Sheet1", "A4").unwrap(), "(10.00)");
    }
}
