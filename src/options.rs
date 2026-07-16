//! Options for opening and reading spreadsheet files.
//!
//! Mirrors the Go `Options` struct from `excelize.go`.

use crate::constants::{STREAM_CHUNK_SIZE, UNZIP_SIZE_LIMIT};

/// Culture identifier used when applying language-sensitive number formats.
pub type CultureName = u8;

/// Unknown / unspecified culture.
pub const CULTURE_NAME_UNKNOWN: CultureName = 0;

/// Options for opening, reading and saving spreadsheet files.
#[derive(Debug, Clone)]
pub struct Options {
    /// Maximum iterations for iterative calculation.
    pub max_calc_iterations: u32,
    /// Plain-text password for encrypted workbooks.
    pub password: String,
    /// Return raw cell values without applying number formats.
    pub raw_cell_value: bool,
    /// Total unzip size limit in bytes.
    pub unzip_size_limit: i64,
    /// Per-XML memory limit before extracting to a temporary file.
    pub unzip_xml_size_limit: i64,
    /// Directory for temporary files (empty = system default).
    pub tmp_dir: String,
    /// Short date number format pattern.
    pub short_date_pattern: String,
    /// Long date number format pattern.
    pub long_date_pattern: String,
    /// Long time number format pattern.
    pub long_time_pattern: String,
    /// Country code for built-in language number formats.
    pub culture_info: CultureName,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            max_calc_iterations: 0,
            password: String::new(),
            raw_cell_value: false,
            unzip_size_limit: UNZIP_SIZE_LIMIT,
            unzip_xml_size_limit: STREAM_CHUNK_SIZE,
            tmp_dir: String::new(),
            short_date_pattern: String::new(),
            long_date_pattern: String::new(),
            long_time_pattern: String::new(),
            culture_info: CULTURE_NAME_UNKNOWN,
        }
    }
}
