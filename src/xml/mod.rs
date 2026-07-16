//! OpenXML / SpreadsheetML data structures used by Excel files.
//!
//! Each submodule in this directory corresponds to one of the Go source files
//! named `xml*.go` in the original repository.

pub mod app;
pub mod calc_chain;
pub mod chart;
pub mod chart_sheet;
pub mod comments;
pub mod common;
pub mod content_types;
pub mod core;
pub mod custom;
pub mod decode_chart;
pub mod decode_drawing;
pub mod drawing;
pub mod metadata;
pub mod pivot_cache;
pub mod pivot_table;
pub mod shared_strings;
pub mod slicers;
pub mod styles;
pub mod table;
pub mod theme;
pub mod vml;
pub mod workbook;
pub mod worksheet;
