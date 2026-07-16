//! Core spreadsheet methods ported from `excelize.go`.
//!
//! These methods deal with workbook-level operations, lazy worksheet access,
//! and content-type / VBA bookkeeping.

use crate::errors::Result;
use crate::file::File;
use crate::xml::workbook::{CalcPropsOptions, WorkbookPropsOptions, WorkbookProtectionOptions};
use crate::xml::worksheet::XlsxWorksheet;

/// Validate a worksheet name.
pub fn check_sheet_name(name: &str) -> Result<()> {
    <File as ExcelizeCore>::check_sheet_name(name)
}

/// Core methods that mirror the Go `File` API.
pub trait ExcelizeCore {
    /// Validate that a worksheet name is legal.
    fn check_sheet_name(name: &str) -> Result<()>;

    /// Validate options used when opening a reader.
    fn check_open_reader_options(&self) -> Result<()>;

    /// Deserialize and return a worksheet by name.
    fn work_sheet_reader(&self, sheet: &str) -> Result<XlsxWorksheet>;

    /// Update linked cell values so Excel recalculates them on open.
    fn update_linked_value(&self) -> Result<()>;

    /// Add a VBA project binary to the workbook.
    fn add_vba_project(&self, data: &[u8]) -> Result<()>;

    /// Set the workbook content type and `.bin` default for macro workbooks.
    fn set_content_type_part_project_extensions(&self, content_type: &str) -> Result<()>;

    /// Set workbook properties.
    fn set_workbook_props(&self, opts: &WorkbookPropsOptions) -> Result<()>;

    /// Get workbook properties.
    fn get_workbook_props(&self) -> Result<WorkbookPropsOptions>;

    /// Set calculation properties.
    fn set_calc_props(&self, opts: &CalcPropsOptions) -> Result<()>;

    /// Get calculation properties.
    fn get_calc_props(&self) -> Result<CalcPropsOptions>;

    /// Protect the workbook.
    fn protect_workbook(&self, opts: &WorkbookProtectionOptions) -> Result<()>;

    /// Remove workbook protection, optionally verifying the password.
    fn unprotect_workbook(&self, password: Option<&str>) -> Result<()>;
}

impl ExcelizeCore for File {
    fn check_sheet_name(name: &str) -> Result<()> {
        if name.is_empty() {
            return Err(Box::new(crate::errors::ErrSheetNameBlank));
        }
        if crate::lib_util::count_utf16_string(name) > crate::constants::MAX_SHEET_NAME_LENGTH {
            return Err(Box::new(crate::errors::ErrSheetNameLength));
        }
        if name.starts_with('\'') || name.ends_with('\'') {
            return Err(Box::new(crate::errors::ErrSheetNameSingleQuote));
        }
        if name.contains(|c: char| matches!(c, ':'))
            || name.contains('\\')
            || name.contains('/')
            || name.contains('?')
            || name.contains('*')
            || name.contains('[')
            || name.contains(']')
        {
            return Err(Box::new(crate::errors::ErrSheetNameInvalid));
        }
        Ok(())
    }

    fn check_open_reader_options(&self) -> Result<()> {
        self.check_open_reader_options()
    }

    fn work_sheet_reader(&self, sheet: &str) -> Result<XlsxWorksheet> {
        File::work_sheet_reader(self, sheet)
    }

    fn update_linked_value(&self) -> Result<()> {
        let mut wb = self.workbook_reader()?;
        // Drop the calcPr so Excel recalculates on open.
        wb.calc_pr = None;
        *self.workbook.lock().unwrap() = Some(wb);
        for name in self.get_sheet_list() {
            let mut ws = self.work_sheet_reader(&name)?;
            for row in &mut ws.sheet_data.row {
                for cell in &mut row.c {
                    if cell.f.is_some() && cell.v.is_some() {
                        cell.v = None;
                        cell.t = None;
                    }
                }
            }
            if let Some(path) = self.get_sheet_xml_path(&name) {
                self.sheet.insert(path, ws);
            }
        }
        Ok(())
    }

    fn add_vba_project(&self, data: &[u8]) -> Result<()> {
        crate::file::add_vba_project(self, data)
    }

    fn set_content_type_part_project_extensions(&self, content_type: &str) -> Result<()> {
        crate::file::set_content_type_part_project_extensions(self, content_type)
    }

    fn set_workbook_props(&self, opts: &WorkbookPropsOptions) -> Result<()> {
        File::set_workbook_props(self, opts)
    }

    fn get_workbook_props(&self) -> Result<WorkbookPropsOptions> {
        File::get_workbook_props(self)
    }

    fn set_calc_props(&self, opts: &CalcPropsOptions) -> Result<()> {
        File::set_calc_props(self, opts)
    }

    fn get_calc_props(&self) -> Result<CalcPropsOptions> {
        File::get_calc_props(self)
    }

    fn protect_workbook(&self, opts: &WorkbookProtectionOptions) -> Result<()> {
        File::protect_workbook(self, opts)
    }

    fn unprotect_workbook(&self, password: Option<&str>) -> Result<()> {
        File::unprotect_workbook(self, password)
    }
}
