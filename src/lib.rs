//! # Excelize for Rust
//!
//! A Rust port of the Go [excelize](https://github.com/xuri/excelize) library,
//! providing read/write support for XLSX / XLSM / XLTX / XLTM / XLAM files.

pub mod adjust;
pub mod calc;
pub mod calc_chain;
pub mod cell;
pub mod chart;
pub mod col;
pub mod constants;
pub mod crypt;
pub mod data_validation;
pub mod date;
pub mod doc_props;
pub mod errors;
pub mod excelize;
pub mod file;
pub mod hsl;
pub mod lib_util;
pub mod merge;
pub mod numfmt;
pub mod options;
pub mod picture;
pub mod pivot_table;
pub mod row;
pub mod shape;
pub mod sheet;
pub mod slicer;
pub mod sparkline;
pub mod stream;
pub mod styles;
pub mod table;
pub mod templates;
pub mod vml;
pub mod xml;

pub use file::File;
pub use options::Options;
