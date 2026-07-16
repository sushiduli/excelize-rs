//! Worksheet-level API.
//!
//! This module corresponds to `sheet.go`, `sheetpr.go` and `sheetview.go` in
//! the Go implementation.

use std::collections::HashMap;
use std::io::Cursor;
use std::path::Path;

use quick_xml::Reader;
use quick_xml::events::Event;

use crate::constants::{
    CONTENT_TYPE_SPREADSHEET_ML_WORKSHEET, MAX_COLUMNS, MAX_FIELD_LENGTH, SOURCE_RELATIONSHIP,
    SOURCE_RELATIONSHIP_IMAGE, SOURCE_RELATIONSHIP_SHARED_STRINGS, SOURCE_RELATIONSHIP_WORKSHEET,
    TOTAL_ROWS,
};
use crate::crypt::gen_iso_passwd_hash;
use crate::errors::Result;
use crate::errors::{
    ErrGroupSheets, ErrImgExt, ErrParameterInvalid, ErrSheetIdx, ErrSheetNotExist,
    ErrUnprotectSheet, ErrUnprotectSheetPassword, ErrWorkbook, new_field_length_error,
};
use crate::file::{File, namespace_strict_to_transitional};
use crate::lib_util::{
    cell_name_to_coordinates, coordinates_to_range_ref, count_utf16_string, in_str_slice,
    range_ref_to_coordinates, sort_coordinates,
};
use crate::xml::content_types::{XlsxContentTypeEntry, XlsxDefault, XlsxOverride};
use crate::xml::workbook::{XlsxBookViews, XlsxSheet, XlsxWorkBookView};
pub use crate::xml::worksheet::{Panes, Selection, SheetProtectionOptions};
use crate::xml::worksheet::{
    XlsxBreaks, XlsxBrk, XlsxColBreaks, XlsxDimension, XlsxHeaderFooter, XlsxIgnoredError,
    XlsxIgnoredErrors, XlsxPageMargins, XlsxPageSetUp, XlsxPane, XlsxPicture, XlsxPrintOptions,
    XlsxRowBreaks, XlsxSelection, XlsxSheetFormatPr, XlsxSheetPr, XlsxSheetProtection,
    XlsxSheetView, XlsxSheetViews, XlsxWorksheet,
};

/// Type of error to ignore for a range of cells.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IgnoredErrorsType {
    EvalError = 0,
    TwoDigitTextYear = 1,
    NumberStoredAsText = 2,
    Formula = 3,
    FormulaRange = 4,
    UnlockedFormula = 5,
    EmptyCellReference = 6,
    ListDataValidation = 7,
    CalculatedColumn = 8,
}

/// Options for `SetSheetProps`.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct SheetPropsOptions {
    pub code_name: Option<String>,
    pub enable_format_conditions_calculation: Option<bool>,
    pub published: Option<bool>,
    pub auto_page_breaks: Option<bool>,
    pub fit_to_page: Option<bool>,
    pub tab_color_indexed: Option<i64>,
    pub tab_color_rgb: Option<String>,
    pub tab_color_theme: Option<i64>,
    pub tab_color_tint: Option<f64>,
    pub outline_pr_apply_styles: Option<bool>,
    pub outline_pr_summary_below: Option<bool>,
    pub outline_pr_summary_right: Option<bool>,
    pub outline_pr_show_outline_symbols: Option<bool>,
    pub base_col_width: Option<u8>,
    pub default_col_width: Option<f64>,
    pub default_row_height: Option<f64>,
    pub custom_height: Option<bool>,
    pub zero_height: Option<bool>,
    pub thick_top: Option<bool>,
    pub thick_bottom: Option<bool>,
}

/// Options for `SetSheetView`.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct SheetViewOptions {
    pub default_grid_color: Option<bool>,
    pub right_to_left: Option<bool>,
    pub show_formulas: Option<bool>,
    pub show_grid_lines: Option<bool>,
    pub show_outline_symbols: Option<bool>,
    pub show_row_col_headers: Option<bool>,
    pub show_ruler: Option<bool>,
    pub show_white_space: Option<bool>,
    pub show_zeros: Option<bool>,
    pub tab_selected: Option<bool>,
    pub top_left_cell: Option<String>,
    pub view: Option<String>,
    pub window_protection: Option<bool>,
    pub zoom_scale: Option<f64>,
}

/// Options for `SetPageLayout`.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct PageLayoutOptions {
    pub black_and_white: Option<bool>,
    pub cell_comments: Option<String>,
    pub copies: Option<i64>,
    pub draft: Option<bool>,
    pub errors: Option<String>,
    pub first_page_number: Option<String>,
    pub fit_to_height: Option<i64>,
    pub fit_to_width: Option<i64>,
    pub horizontal_dpi: Option<String>,
    pub orientation: Option<String>,
    pub page_order: Option<String>,
    pub paper_size: Option<i64>,
    pub scale: Option<i64>,
    pub use_first_page_number: Option<bool>,
    pub use_printer_defaults: Option<bool>,
    pub vertical_dpi: Option<String>,
}

/// Options for `SetPageMargins`.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct PageMarginsOptions {
    pub left: Option<f64>,
    pub right: Option<f64>,
    pub top: Option<f64>,
    pub bottom: Option<f64>,
    pub header: Option<f64>,
    pub footer: Option<f64>,
    pub horizontally: Option<bool>,
    pub vertically: Option<bool>,
}

/// Header/footer options for `SetHeaderFooter`.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct HeaderFooterOptions {
    pub different_first: Option<bool>,
    pub different_odd_even: Option<bool>,
    pub scale_with_doc: Option<bool>,
    pub align_with_margins: Option<bool>,
    pub odd_header: Option<String>,
    pub odd_footer: Option<String>,
    pub even_header: Option<String>,
    pub even_footer: Option<String>,
    pub first_header: Option<String>,
    pub first_footer: Option<String>,
}

// ------------------------------------------------------------------
// Public API
// ------------------------------------------------------------------

impl File {
    /// Get a list of worksheet / chart / dialog sheet names.
    pub fn get_sheet_list(&self) -> Vec<String> {
        let wb = self.workbook_reader().ok();
        let mut list = Vec::new();
        if let Some(wb) = wb {
            for sheet in &wb.sheets.sheet {
                if let Some(name) = &sheet.name {
                    list.push(name.clone());
                }
            }
        }
        list
    }

    /// Create a new worksheet.
    pub fn new_sheet(&self, name: &str) -> Result<i32> {
        crate::excelize::check_sheet_name(name)?;
        if let Ok(index) = self.get_sheet_index(name) {
            return Ok(index);
        }

        let mut wb = self.workbook_reader()?;
        let mut rels = self
            .rels_reader(&self.get_workbook_rels_path())?
            .unwrap_or_default();

        // Find next sheet id and relationship id.
        let mut max_sheet_id = 0i64;
        let mut max_rid = 0i32;
        for sheet in &wb.sheets.sheet {
            if let Some(id) = sheet.sheet_id {
                if id > max_sheet_id {
                    max_sheet_id = id;
                }
            }
            if let Some(rid) = &sheet.id {
                if let Ok(n) = rid.trim_start_matches("rId").parse::<i32>() {
                    if n > max_rid {
                        max_rid = n;
                    }
                }
            }
        }
        for rel in &rels.relationships {
            if let Ok(n) = rel.id.trim_start_matches("rId").parse::<i32>() {
                if n > max_rid {
                    max_rid = n;
                }
            }
        }
        max_sheet_id += 1;
        max_rid += 1;

        let new_rid = format!("rId{max_rid}");
        let sheet_xml = format!("xl/worksheets/sheet{max_sheet_id}.xml");

        wb.sheets.sheet.push(XlsxSheet {
            name: Some(name.to_string()),
            sheet_id: Some(max_sheet_id),
            id: Some(new_rid.clone()),
            plain_id: None,
            state: None,
        });

        rels.relationships
            .push(crate::xml::workbook::XlsxRelationship {
                id: new_rid,
                r#type: SOURCE_RELATIONSHIP_WORKSHEET.to_string(),
                target: format!("worksheets/sheet{max_sheet_id}.xml"),
                target_mode: None,
            });

        // Insert worksheet part.
        let ws = new_worksheet();
        self.sheet.insert(sheet_xml.clone(), ws.clone());
        self.pkg.insert(sheet_xml.clone(), worksheet_bytes(&ws));
        self.checked.insert(sheet_xml.clone(), true);

        // Update content types.
        let mut ct = self.content_types_reader()?;
        ct.entries
            .push(XlsxContentTypeEntry::Override(XlsxOverride {
                part_name: format!("/{sheet_xml}"),
                content_type: CONTENT_TYPE_SPREADSHEET_ML_WORKSHEET.to_string(),
            }));

        // Ensure shared strings relationship exists.
        ensure_shared_strings_rel(&mut rels);

        *self.workbook.lock().unwrap() = Some(wb);
        self.relationships
            .insert(self.get_workbook_rels_path(), rels);
        *self.content_types.lock().unwrap() = Some(ct);
        self.sheet_map
            .lock()
            .unwrap()
            .insert(name.to_string(), sheet_xml);

        let mut count = self.sheet_count.lock().unwrap();
        *count += 1;
        Ok(max_sheet_id as i32)
    }

    /// Get the worksheet name from a 1-based sheet index.
    pub fn get_sheet_name(&self, index: i32) -> Result<String> {
        let wb = self.workbook_reader()?;
        if index < 1 || index as usize > wb.sheets.sheet.len() {
            return Err(Box::new(ErrSheetIdx));
        }
        Ok(wb.sheets.sheet[index as usize - 1]
            .name
            .clone()
            .unwrap_or_default())
    }

    /// Get the 1-based sheet index of a worksheet by name.
    pub fn get_sheet_index(&self, name: &str) -> Result<i32> {
        let wb = self.workbook_reader()?;
        for (i, sheet) in wb.sheets.sheet.iter().enumerate() {
            if sheet
                .name
                .as_deref()
                .map(|n| n.eq_ignore_ascii_case(name))
                .unwrap_or(false)
            {
                return Ok(i as i32 + 1);
            }
        }
        Err(Box::new(ErrSheetNotExist {
            sheet_name: name.to_string(),
        }))
    }

    /// Delete a worksheet by name.
    pub fn delete_sheet(&self, name: &str) -> Result<()> {
        crate::excelize::check_sheet_name(name)?;
        let index = self.get_sheet_index(name)?;
        if self.get_sheet_list().len() <= 1 {
            return Err(Box::new(ErrWorkbook));
        }
        let path = self
            .get_sheet_xml_path(name)
            .ok_or_else(|| ErrSheetNotExist {
                sheet_name: name.to_string(),
            })?;

        self.clear_calc_cache();
        let active_index = self.get_active_sheet_index()?;
        let sheet_id = self.get_sheet_id(name);

        let mut wb = self.workbook_reader()?;
        wb.sheets.sheet.remove(index as usize - 1);

        let rel_path = self.get_workbook_rels_path();
        let mut rels = self.rels_reader(&rel_path)?.unwrap_or_default();
        rels.relationships.retain(|r| {
            let target = r.target.to_lowercase();
            !target.contains(&format!("/{path}").to_lowercase()) && target != path.to_lowercase()
        });

        let sheet_rels = format!(
            "xl/worksheets/_rels/{}.rels",
            path.trim_start_matches("xl/worksheets/")
        );

        *self.workbook.lock().unwrap() = Some(wb);
        self.relationships.insert(rel_path, rels);
        self.sheet.remove(&path);
        self.pkg.remove(&path);
        self.checked.remove(&path);
        self.relationships.remove(&sheet_rels);
        self.pkg.remove(&sheet_rels);
        self.sheet_map.lock().unwrap().remove(name);

        self.remove_content_types_part(CONTENT_TYPE_SPREADSHEET_ML_WORKSHEET, &format!("/{path}"))?;
        if sheet_id > 0 {
            let _ = self.delete_calc_chain(sheet_id, "");
        }

        let mut new_active = active_index;
        if active_index == index {
            new_active = 1;
        } else if active_index > index {
            new_active = active_index - 1;
        }
        self.set_active_sheet(new_active)?;

        let mut count = self.sheet_count.lock().unwrap();
        *count -= 1;
        Ok(())
    }

    /// Get the index of the active worksheet (1-based).
    pub fn get_active_sheet_index(&self) -> Result<i32> {
        let wb = self.workbook_reader()?;
        let views = wb.book_views.as_ref().and_then(|v| v.workbook_view.first());
        if let Some(view) = views {
            return Ok(view.active_tab.unwrap_or(0) as i32 + 1);
        }
        Ok(1)
    }

    /// Set the active worksheet by 1-based index.
    pub fn set_active_sheet(&self, index: i32) -> Result<()> {
        let mut wb = self.workbook_reader()?;
        if index < 1 || index as usize > wb.sheets.sheet.len() {
            return Err(Box::new(ErrSheetIdx));
        }
        if wb.book_views.is_none() {
            wb.book_views = Some(XlsxBookViews::default());
        }
        let views = wb.book_views.as_mut().unwrap();
        if views.workbook_view.is_empty() {
            views.workbook_view.push(XlsxWorkBookView::default());
        }
        views.workbook_view[0].active_tab = Some(index as i64 - 1);
        *self.workbook.lock().unwrap() = Some(wb);

        let target = index as usize;
        for (idx, name) in self.get_sheet_list().iter().enumerate() {
            let Ok(mut ws) = self.work_sheet_reader(name) else {
                continue;
            };
            if ws.sheet_views.is_none() {
                ws.sheet_views = Some(XlsxSheetViews {
                    sheet_view: vec![XlsxSheetView::default()],
                });
            }
            let sv = ws.sheet_views.as_mut().unwrap();
            if sv.sheet_view.is_empty() {
                sv.sheet_view.push(XlsxSheetView::default());
            }
            sv.sheet_view[0].tab_selected = Some(idx + 1 == target);
            if let Some(path) = self.get_sheet_xml_path(name) {
                self.sheet.insert(path, ws);
            }
        }
        Ok(())
    }

    /// Set worksheet properties.
    pub fn set_sheet_props(&self, sheet: &str, opts: &SheetPropsOptions) -> Result<()> {
        let path = self
            .get_sheet_xml_path(sheet)
            .ok_or_else(|| ErrSheetNotExist {
                sheet_name: sheet.to_string(),
            })?;
        let mut ws = self.work_sheet_reader(sheet)?;
        if ws.sheet_pr.is_none() {
            ws.sheet_pr = Some(XlsxSheetPr::default());
        }
        let pr = ws.sheet_pr.as_mut().unwrap();
        if opts.code_name.is_some() {
            pr.code_name = opts.code_name.clone();
        }
        if opts.enable_format_conditions_calculation.is_some() {
            pr.enable_format_conditions_calculation = opts.enable_format_conditions_calculation;
        }
        if opts.published.is_some() {
            pr.published = opts.published;
        }
        if let Some(indexed) = opts.tab_color_indexed {
            if pr.tab_color.is_none() {
                pr.tab_color = Some(crate::xml::common::XlsxColor::default());
            }
            pr.tab_color.as_mut().unwrap().indexed = Some(indexed);
        }
        if let Some(ref rgb) = opts.tab_color_rgb {
            if pr.tab_color.is_none() {
                pr.tab_color = Some(crate::xml::common::XlsxColor::default());
            }
            pr.tab_color.as_mut().unwrap().rgb = Some(rgb.clone());
        }
        if let Some(theme) = opts.tab_color_theme {
            if pr.tab_color.is_none() {
                pr.tab_color = Some(crate::xml::common::XlsxColor::default());
            }
            pr.tab_color.as_mut().unwrap().theme = Some(theme);
        }
        if let Some(tint) = opts.tab_color_tint {
            if pr.tab_color.is_none() {
                pr.tab_color = Some(crate::xml::common::XlsxColor::default());
            }
            pr.tab_color.as_mut().unwrap().tint = Some(tint);
        }
        let outline_needed = opts.outline_pr_apply_styles.is_some()
            || opts.outline_pr_summary_below.is_some()
            || opts.outline_pr_summary_right.is_some()
            || opts.outline_pr_show_outline_symbols.is_some();
        if outline_needed {
            if pr.outline_pr.is_none() {
                pr.outline_pr = Some(crate::xml::worksheet::XlsxOutlinePr::default());
            }
            let outline = pr.outline_pr.as_mut().unwrap();
            if opts.outline_pr_apply_styles.is_some() {
                outline.apply_styles = opts.outline_pr_apply_styles;
            }
            if opts.outline_pr_summary_below.is_some() {
                outline.summary_below = opts.outline_pr_summary_below;
            }
            if opts.outline_pr_summary_right.is_some() {
                outline.summary_right = opts.outline_pr_summary_right;
            }
            if opts.outline_pr_show_outline_symbols.is_some() {
                outline.show_outline_symbols = opts.outline_pr_show_outline_symbols;
            }
        }
        if opts.auto_page_breaks.is_some() || opts.fit_to_page.is_some() {
            if pr.page_set_up_pr.is_none() {
                pr.page_set_up_pr = Some(crate::xml::worksheet::XlsxPageSetUpPr::default());
            }
            let pup = pr.page_set_up_pr.as_mut().unwrap();
            if opts.auto_page_breaks.is_some() {
                pup.auto_page_breaks = opts.auto_page_breaks;
            }
            if opts.fit_to_page.is_some() {
                pup.fit_to_page = opts.fit_to_page;
            }
        }
        if ws.sheet_format_pr.is_none() {
            ws.sheet_format_pr = Some(XlsxSheetFormatPr {
                default_row_height: 15.0,
                ..Default::default()
            });
        }
        let sfp = ws.sheet_format_pr.as_mut().unwrap();
        if let Some(v) = opts.base_col_width {
            sfp.base_col_width = Some(v);
        }
        if let Some(v) = opts.default_col_width {
            sfp.default_col_width = Some(v);
        }
        if let Some(v) = opts.default_row_height {
            sfp.default_row_height = v;
        }
        if let Some(v) = opts.custom_height {
            sfp.custom_height = Some(v);
        }
        if let Some(v) = opts.zero_height {
            sfp.zero_height = Some(v);
        }
        if let Some(v) = opts.thick_top {
            sfp.thick_top = Some(v);
        }
        if let Some(v) = opts.thick_bottom {
            sfp.thick_bottom = Some(v);
        }
        self.sheet.insert(path, ws);
        Ok(())
    }

    /// Get worksheet properties.
    pub fn get_sheet_props(&self, sheet: &str) -> Result<SheetPropsOptions> {
        let ws = self.work_sheet_reader(sheet)?;
        let mut opts = SheetPropsOptions {
            enable_format_conditions_calculation: Some(true),
            published: Some(true),
            auto_page_breaks: Some(true),
            outline_pr_summary_below: Some(true),
            base_col_width: Some(8),
            ..Default::default()
        };
        if let Some(pr) = ws.sheet_pr {
            opts.code_name = pr.code_name;
            if pr.enable_format_conditions_calculation.is_some() {
                opts.enable_format_conditions_calculation = pr.enable_format_conditions_calculation;
            }
            if pr.published.is_some() {
                opts.published = pr.published;
            }
            if let Some(tc) = pr.tab_color {
                opts.tab_color_indexed = tc.indexed;
                opts.tab_color_rgb = tc.rgb;
                opts.tab_color_theme = tc.theme;
                opts.tab_color_tint = tc.tint;
            }
            if let Some(o) = pr.outline_pr {
                opts.outline_pr_apply_styles = o.apply_styles;
                opts.outline_pr_summary_below = o.summary_below;
                opts.outline_pr_summary_right = o.summary_right;
                opts.outline_pr_show_outline_symbols = o.show_outline_symbols;
            }
            if let Some(pup) = pr.page_set_up_pr {
                opts.auto_page_breaks = pup.auto_page_breaks;
                opts.fit_to_page = pup.fit_to_page;
            }
        }
        if let Some(sfp) = ws.sheet_format_pr {
            opts.base_col_width = sfp.base_col_width.or(Some(8));
            opts.default_col_width = sfp.default_col_width;
            opts.default_row_height = Some(sfp.default_row_height);
            opts.custom_height = sfp.custom_height;
            opts.zero_height = sfp.zero_height;
            opts.thick_top = sfp.thick_top;
            opts.thick_bottom = sfp.thick_bottom;
        }
        Ok(opts)
    }

    /// Set worksheet view properties on the given sheet view. The view index may
    /// be negative and counts backward from the last view.
    pub fn set_sheet_view(
        &self,
        sheet: &str,
        view_index: i32,
        opts: &SheetViewOptions,
    ) -> Result<()> {
        let path = self
            .get_sheet_xml_path(sheet)
            .ok_or_else(|| ErrSheetNotExist {
                sheet_name: sheet.to_string(),
            })?;
        let mut ws = self.work_sheet_reader(sheet)?;
        if ws.sheet_views.is_none() {
            ws.sheet_views = Some(XlsxSheetViews {
                sheet_view: vec![XlsxSheetView::default()],
            });
        }
        let views = ws.sheet_views.as_mut().unwrap();
        let len = views.sheet_view.len();
        let idx = if view_index < 0 {
            if view_index < -(len as i32) {
                return Err(crate::errors::new_view_idx_error(view_index).into());
            }
            (len as i32 + view_index) as usize
        } else if view_index >= len as i32 {
            return Err(crate::errors::new_view_idx_error(view_index).into());
        } else {
            view_index as usize
        };
        let view = &mut views.sheet_view[idx];
        if opts.default_grid_color.is_some() {
            view.default_grid_color = opts.default_grid_color;
        }
        if opts.right_to_left.is_some() {
            view.right_to_left = opts.right_to_left;
        }
        if opts.show_formulas.is_some() {
            view.show_formulas = opts.show_formulas;
        }
        if opts.show_grid_lines.is_some() {
            view.show_grid_lines = opts.show_grid_lines;
        }
        if opts.show_outline_symbols.is_some() {
            view.show_outline_symbols = opts.show_outline_symbols;
        }
        if opts.show_row_col_headers.is_some() {
            view.show_row_col_headers = opts.show_row_col_headers;
        }
        if opts.show_ruler.is_some() {
            view.show_ruler = opts.show_ruler;
        }
        if opts.show_white_space.is_some() {
            view.show_white_space = opts.show_white_space;
        }
        if opts.show_zeros.is_some() {
            view.show_zeros = opts.show_zeros;
        }
        if opts.tab_selected.is_some() {
            view.tab_selected = opts.tab_selected;
        }
        if opts.top_left_cell.is_some() {
            view.top_left_cell = opts.top_left_cell.clone();
        }
        if let Some(ref v) = opts.view {
            if in_str_slice(&["normal", "pageLayout", "pageBreakPreview"], v, true) != -1 {
                view.view = opts.view.clone();
            }
        }
        if opts.window_protection.is_some() {
            view.window_protection = opts.window_protection;
        }
        if let Some(zoom) = opts.zoom_scale {
            if zoom >= 10.0 && zoom <= 400.0 {
                view.zoom_scale = Some(zoom);
            }
        }
        self.sheet.insert(path, ws);
        Ok(())
    }

    /// Get worksheet view properties from the given sheet view. The view index may
    /// be negative and counts backward from the last view.
    pub fn get_sheet_view(&self, sheet: &str, view_index: i32) -> Result<SheetViewOptions> {
        let ws = self.work_sheet_reader(sheet)?;
        let views = ws.sheet_views.unwrap_or_else(|| XlsxSheetViews {
            sheet_view: vec![XlsxSheetView::default()],
        });
        let len = views.sheet_view.len();
        let idx = if view_index < 0 {
            if view_index < -(len as i32) {
                return Err(crate::errors::new_view_idx_error(view_index).into());
            }
            (len as i32 + view_index) as usize
        } else if view_index >= len as i32 {
            return Err(crate::errors::new_view_idx_error(view_index).into());
        } else {
            view_index as usize
        };
        let view = &views.sheet_view[idx];
        let mut opts = SheetViewOptions {
            default_grid_color: Some(true),
            show_grid_lines: Some(true),
            show_row_col_headers: Some(true),
            show_ruler: Some(true),
            show_zeros: Some(true),
            view: Some("normal".to_string()),
            zoom_scale: Some(100.0),
            ..Default::default()
        };
        if let Some(v) = view.default_grid_color {
            opts.default_grid_color = Some(v);
        }
        opts.right_to_left = Some(view.right_to_left.unwrap_or(false));
        opts.show_formulas = Some(view.show_formulas.unwrap_or(false));
        if let Some(v) = view.show_grid_lines {
            opts.show_grid_lines = Some(v);
        }
        if let Some(v) = view.show_row_col_headers {
            opts.show_row_col_headers = Some(v);
        }
        if let Some(v) = view.show_ruler {
            opts.show_ruler = Some(v);
        }
        if let Some(v) = view.show_white_space {
            opts.show_white_space = Some(v);
        }
        if let Some(v) = view.show_zeros {
            opts.show_zeros = Some(v);
        }
        opts.show_outline_symbols = Some(view.show_outline_symbols.unwrap_or(false));
        opts.tab_selected = Some(view.tab_selected.unwrap_or(false));
        opts.top_left_cell = Some(view.top_left_cell.clone().unwrap_or_default());
        opts.window_protection = Some(view.window_protection.unwrap_or(false));
        if let Some(ref v) = view.view {
            if !v.is_empty() {
                opts.view = Some(v.clone());
            }
        }
        if let Some(zoom) = view.zoom_scale {
            if zoom >= 10.0 && zoom <= 400.0 {
                opts.zoom_scale = Some(zoom);
            }
        }
        Ok(opts)
    }

    /// Set page layout properties.
    pub fn set_page_layout(&self, sheet: &str, opts: &PageLayoutOptions) -> Result<()> {
        let path = self
            .get_sheet_xml_path(sheet)
            .ok_or_else(|| ErrSheetNotExist {
                sheet_name: sheet.to_string(),
            })?;
        let mut ws = self.work_sheet_reader(sheet)?;
        if ws.page_setup.is_none() {
            ws.page_setup = Some(XlsxPageSetUp::default());
        }
        let ps = ws.page_setup.as_mut().unwrap();
        if opts.black_and_white.is_some() {
            ps.black_and_white = opts.black_and_white;
        }
        if opts.cell_comments.is_some() {
            ps.cell_comments = opts.cell_comments.clone();
        }
        if opts.copies.is_some() {
            ps.copies = opts.copies;
        }
        if opts.draft.is_some() {
            ps.draft = opts.draft;
        }
        if opts.errors.is_some() {
            ps.errors = opts.errors.clone();
        }
        if opts.first_page_number.is_some() {
            ps.first_page_number = opts.first_page_number.clone();
        }
        if opts.fit_to_height.is_some() {
            ps.fit_to_height = opts.fit_to_height;
        }
        if opts.fit_to_width.is_some() {
            ps.fit_to_width = opts.fit_to_width;
        }
        if opts.horizontal_dpi.is_some() {
            ps.horizontal_dpi = opts.horizontal_dpi.clone();
        }
        if opts.orientation.is_some() {
            ps.orientation = opts.orientation.clone();
        }
        if opts.page_order.is_some() {
            ps.page_order = opts.page_order.clone();
        }
        if opts.paper_size.is_some() {
            ps.paper_size = opts.paper_size;
        }
        if opts.scale.is_some() {
            ps.scale = opts.scale;
        }
        if opts.use_first_page_number.is_some() {
            ps.use_first_page_number = opts.use_first_page_number;
        }
        if opts.use_printer_defaults.is_some() {
            ps.use_printer_defaults = opts.use_printer_defaults;
        }
        if opts.vertical_dpi.is_some() {
            ps.vertical_dpi = opts.vertical_dpi.clone();
        }
        self.sheet.insert(path, ws);
        Ok(())
    }

    /// Get page layout properties.
    pub fn get_page_layout(&self, sheet: &str) -> Result<PageLayoutOptions> {
        let ws = self.work_sheet_reader(sheet)?;
        let mut opts = PageLayoutOptions::default();
        if let Some(ps) = ws.page_setup {
            opts.black_and_white = ps.black_and_white;
            opts.cell_comments = ps.cell_comments;
            opts.copies = ps.copies;
            opts.draft = ps.draft;
            opts.errors = ps.errors;
            opts.first_page_number = ps.first_page_number;
            opts.fit_to_height = ps.fit_to_height;
            opts.fit_to_width = ps.fit_to_width;
            opts.horizontal_dpi = ps.horizontal_dpi;
            opts.orientation = ps.orientation;
            opts.page_order = ps.page_order;
            opts.paper_size = ps.paper_size;
            opts.scale = ps.scale;
            opts.use_first_page_number = ps.use_first_page_number;
            opts.use_printer_defaults = ps.use_printer_defaults;
            opts.vertical_dpi = ps.vertical_dpi;
        }
        Ok(opts)
    }

    /// Set page margins.
    pub fn set_page_margins(&self, sheet: &str, opts: &PageMarginsOptions) -> Result<()> {
        let path = self
            .get_sheet_xml_path(sheet)
            .ok_or_else(|| ErrSheetNotExist {
                sheet_name: sheet.to_string(),
            })?;
        let mut ws = self.work_sheet_reader(sheet)?;
        let margins_needed = opts.left.is_some()
            || opts.right.is_some()
            || opts.top.is_some()
            || opts.bottom.is_some()
            || opts.header.is_some()
            || opts.footer.is_some();
        if margins_needed {
            if ws.page_margins.is_none() {
                ws.page_margins = Some(XlsxPageMargins::default());
            }
            let pm = ws.page_margins.as_mut().unwrap();
            if let Some(v) = opts.left {
                pm.left = v;
            }
            if let Some(v) = opts.right {
                pm.right = v;
            }
            if let Some(v) = opts.top {
                pm.top = v;
            }
            if let Some(v) = opts.bottom {
                pm.bottom = v;
            }
            if let Some(v) = opts.header {
                pm.header = v;
            }
            if let Some(v) = opts.footer {
                pm.footer = v;
            }
        }
        if opts.horizontally.is_some() || opts.vertically.is_some() {
            if ws.print_options.is_none() {
                ws.print_options = Some(XlsxPrintOptions::default());
            }
            let po = ws.print_options.as_mut().unwrap();
            if let Some(v) = opts.horizontally {
                po.horizontal_centered = Some(v);
            }
            if let Some(v) = opts.vertically {
                po.vertical_centered = Some(v);
            }
        }
        self.sheet.insert(path, ws);
        Ok(())
    }

    /// Get page margins.
    pub fn get_page_margins(&self, sheet: &str) -> Result<PageMarginsOptions> {
        let ws = self.work_sheet_reader(sheet)?;
        let mut opts = PageMarginsOptions {
            left: Some(0.7),
            right: Some(0.7),
            top: Some(0.75),
            bottom: Some(0.75),
            header: Some(0.3),
            footer: Some(0.3),
            ..Default::default()
        };
        if let Some(pm) = ws.page_margins {
            opts.left = Some(pm.left);
            opts.right = Some(pm.right);
            opts.top = Some(pm.top);
            opts.bottom = Some(pm.bottom);
            opts.header = Some(pm.header);
            opts.footer = Some(pm.footer);
        }
        if let Some(po) = ws.print_options {
            opts.horizontally = po.horizontal_centered;
            opts.vertically = po.vertical_centered;
        }
        Ok(opts)
    }

    /// Set header/footer text.
    pub fn set_header_footer(&self, sheet: &str, opts: &HeaderFooterOptions) -> Result<()> {
        let path = self
            .get_sheet_xml_path(sheet)
            .ok_or_else(|| ErrSheetNotExist {
                sheet_name: sheet.to_string(),
            })?;
        let mut ws = self.work_sheet_reader(sheet)?;
        for (field, value) in [
            ("OddHeader", opts.odd_header.as_deref()),
            ("OddFooter", opts.odd_footer.as_deref()),
            ("EvenHeader", opts.even_header.as_deref()),
            ("EvenFooter", opts.even_footer.as_deref()),
            ("FirstHeader", opts.first_header.as_deref()),
            ("FirstFooter", opts.first_footer.as_deref()),
        ] {
            if let Some(v) = value {
                if count_utf16_string(v) > MAX_FIELD_LENGTH {
                    return Err(new_field_length_error(field).into());
                }
            }
        }
        if ws.header_footer.is_none() {
            ws.header_footer = Some(XlsxHeaderFooter::default());
        }
        let hf = ws.header_footer.as_mut().unwrap();
        if opts.different_first.is_some() {
            hf.different_first = opts.different_first;
        }
        if opts.different_odd_even.is_some() {
            hf.different_odd_even = opts.different_odd_even;
        }
        if opts.scale_with_doc.is_some() {
            hf.scale_with_doc = opts.scale_with_doc;
        }
        if opts.align_with_margins.is_some() {
            hf.align_with_margins = opts.align_with_margins;
        }
        if opts.odd_header.is_some() {
            hf.odd_header = opts.odd_header.clone();
        }
        if opts.odd_footer.is_some() {
            hf.odd_footer = opts.odd_footer.clone();
        }
        if opts.even_header.is_some() {
            hf.even_header = opts.even_header.clone();
        }
        if opts.even_footer.is_some() {
            hf.even_footer = opts.even_footer.clone();
        }
        if opts.first_header.is_some() {
            hf.first_header = opts.first_header.clone();
        }
        if opts.first_footer.is_some() {
            hf.first_footer = opts.first_footer.clone();
        }
        self.sheet.insert(path, ws);
        Ok(())
    }

    /// Get header/footer text.
    pub fn get_header_footer(&self, sheet: &str) -> Result<HeaderFooterOptions> {
        let ws = self.work_sheet_reader(sheet)?;
        let mut opts = HeaderFooterOptions::default();
        if let Some(hf) = ws.header_footer {
            opts.different_first = hf.different_first;
            opts.different_odd_even = hf.different_odd_even;
            opts.scale_with_doc = hf.scale_with_doc;
            opts.align_with_margins = hf.align_with_margins;
            opts.odd_header = hf.odd_header;
            opts.odd_footer = hf.odd_footer;
            opts.even_header = hf.even_header;
            opts.even_footer = hf.even_footer;
            opts.first_header = hf.first_header;
            opts.first_footer = hf.first_footer;
        }
        Ok(opts)
    }

    /// Set the worksheet name by given source and target worksheet names.
    pub fn set_sheet_name(&self, source: &str, target: &str) -> Result<()> {
        crate::excelize::check_sheet_name(target)?;
        if source.eq_ignore_ascii_case(target) {
            return Ok(());
        }
        self.clear_calc_cache();
        let mut wb = self.workbook_reader()?;
        for k in 0..wb.sheets.sheet.len() {
            if wb.sheets.sheet[k]
                .name
                .as_deref()
                .map(|n| n.eq_ignore_ascii_case(source))
                .unwrap_or(false)
            {
                wb.sheets.sheet[k].name = Some(target.to_string());
                let mut map = self.sheet_map.lock().unwrap();
                if let Some(path) = map.get(source).cloned() {
                    map.remove(source);
                    map.insert(target.to_string(), path);
                }
            }
        }
        if let Some(names) = wb.defined_names.as_mut() {
            for dn in &mut names.defined_name {
                dn.data = adjust_range_sheet_name(&dn.data, source, target);
            }
        }
        *self.workbook.lock().unwrap() = Some(wb);
        Ok(())
    }

    /// Set background picture by given worksheet name and file path.
    pub fn set_sheet_background(&self, sheet: &str, picture: &str) -> Result<()> {
        let ext = Path::new(picture)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_string();
        let file = std::fs::read(picture)?;
        self.set_sheet_background_from_bytes(sheet, &ext, &file)
    }

    /// Set background picture by given worksheet name, extension name and image data.
    pub fn set_sheet_background_from_bytes(
        &self,
        sheet: &str,
        extension: &str,
        picture: &[u8],
    ) -> Result<()> {
        if picture.is_empty() {
            return Err(Box::new(ErrParameterInvalid));
        }
        let ext = if !extension.is_empty() {
            format!(".{extension}")
        } else {
            String::new()
        };
        let image_type = image_extension_map()
            .get(&ext.to_lowercase())
            .cloned()
            .ok_or(ErrImgExt)?;
        let name = self.add_media(picture, &image_type);
        let sheet_xml_path = self
            .get_sheet_xml_path(sheet)
            .ok_or_else(|| ErrSheetNotExist {
                sheet_name: sheet.to_string(),
            })?;
        let sheet_rels = format!(
            "xl/worksheets/_rels/{}.rels",
            sheet_xml_path.trim_start_matches("xl/worksheets/")
        );
        let target = name.replacen("xl", "..", 1);
        let r_id = self.add_rels(&sheet_rels, SOURCE_RELATIONSHIP_IMAGE, &target, "");
        self.add_sheet_picture(sheet, r_id)?;
        self.add_sheet_name_space(sheet, SOURCE_RELATIONSHIP);
        set_content_type_part_image_extensions(self)
    }

    /// Move a sheet to a specified position in the workbook.
    /// Indices are 1-based and the source sheet is placed before the target sheet.
    pub fn move_sheet(&self, source: &str, target: &str) -> Result<()> {
        if source.eq_ignore_ascii_case(target) {
            return Ok(());
        }
        let mut wb = self.workbook_reader()?;
        let source_idx = self.get_sheet_index(source)?;
        let target_idx = self.get_sheet_index(target)?;
        if source_idx < 1 {
            return Err(Box::new(ErrSheetNotExist {
                sheet_name: source.to_string(),
            }));
        }
        if target_idx < 1 {
            return Err(Box::new(ErrSheetNotExist {
                sheet_name: target.to_string(),
            }));
        }
        let _ = self.ungroup_sheets();
        let active_sheet_name = self.get_sheet_name(self.get_active_sheet_index()?)?;
        let source_sheet = wb.sheets.sheet.remove(source_idx as usize - 1);
        let mut target_idx = target_idx;
        if target_idx > source_idx {
            target_idx -= 1;
        }
        wb.sheets
            .sheet
            .insert(target_idx as usize - 1, source_sheet);
        *self.workbook.lock().unwrap() = Some(wb);
        let active_idx = self.get_sheet_index(&active_sheet_name)?;
        self.set_active_sheet(active_idx)?;
        Ok(())
    }

    /// Duplicate a worksheet by source and target 1-based sheet indices.
    pub fn copy_sheet(&self, from: i32, to: i32) -> Result<()> {
        if from < 1 || to < 1 || from == to {
            return Err(Box::new(ErrSheetIdx));
        }
        let from_name = self.get_sheet_name(from)?;
        let to_name = self.get_sheet_name(to)?;
        if from_name.is_empty() || to_name.is_empty() {
            return Err(Box::new(ErrSheetIdx));
        }
        self.clear_calc_cache();
        let ws = self.work_sheet_reader(&from_name)?;
        let mut worksheet = ws.clone();
        if let Some(views) = worksheet.sheet_views.as_mut() {
            if let Some(view) = views.sheet_view.first_mut() {
                view.tab_selected = Some(false);
            }
        }
        worksheet.drawing = None;
        worksheet.table_parts = None;
        worksheet.page_setup = None;
        let to_sheet_id = self.get_sheet_id(&to_name);
        let sheet_xml_path = format!("xl/worksheets/sheet{to_sheet_id}.xml");
        self.sheet.insert(sheet_xml_path.clone(), worksheet);
        let to_rels = format!("xl/worksheets/_rels/sheet{to_sheet_id}.xml.rels");
        let from_sheet_id = self.get_sheet_id(&from_name);
        let from_rels = format!("xl/worksheets/_rels/sheet{from_sheet_id}.xml.rels");
        if let Some(rels) = self.pkg.get(&from_rels) {
            self.pkg.insert(to_rels, rels.clone());
        }
        let from_sheet_xml_path = self.get_sheet_xml_path(&from_name).unwrap_or_default();
        if let Some(attrs) = self.xml_attr.get(&from_sheet_xml_path) {
            self.xml_attr.insert(sheet_xml_path, attrs.clone());
        }
        Ok(())
    }

    /// Set worksheet visible by given worksheet name.
    pub fn set_sheet_visible(
        &self,
        sheet: &str,
        visible: bool,
        very_hidden: Option<bool>,
    ) -> Result<()> {
        crate::excelize::check_sheet_name(sheet)?;
        let mut wb = self.workbook_reader()?;
        if visible {
            for k in 0..wb.sheets.sheet.len() {
                if wb.sheets.sheet[k]
                    .name
                    .as_deref()
                    .map(|n| n.eq_ignore_ascii_case(sheet))
                    .unwrap_or(false)
                {
                    wb.sheets.sheet[k].state = None;
                }
            }
            *self.workbook.lock().unwrap() = Some(wb);
            return Ok(());
        }
        let state = get_sheet_state(false, very_hidden);
        let mut count = 0;
        for s in &wb.sheets.sheet {
            if s.state.as_deref().unwrap_or("") != state {
                count += 1;
            }
        }
        for k in 0..wb.sheets.sheet.len() {
            let name = wb.sheets.sheet[k].name.clone().unwrap_or_default();
            let ws = self.work_sheet_reader(&name)?;
            let mut tab_selected = false;
            if let Some(views) = ws.sheet_views.as_ref() {
                if let Some(view) = views.sheet_view.first() {
                    tab_selected = view.tab_selected.unwrap_or(false);
                }
            }
            if name.eq_ignore_ascii_case(sheet) && count > 1 && !tab_selected {
                wb.sheets.sheet[k].state = Some(state.to_string());
            }
        }
        *self.workbook.lock().unwrap() = Some(wb);
        Ok(())
    }

    /// Get worksheet visible by given worksheet name.
    pub fn get_sheet_visible(&self, sheet: &str) -> Result<bool> {
        crate::excelize::check_sheet_name(sheet)?;
        let wb = self.workbook_reader()?;
        for s in &wb.sheets.sheet {
            if s.name
                .as_deref()
                .map(|n| n.eq_ignore_ascii_case(sheet))
                .unwrap_or(false)
            {
                let state = s.state.as_deref().unwrap_or("");
                return Ok(state.is_empty() || state.eq_ignore_ascii_case("visible"));
            }
        }
        Ok(false)
    }

    /// Create and remove freeze panes and split panes by given worksheet name and panes options.
    pub fn set_panes(&self, sheet: &str, panes: &Panes) -> Result<()> {
        let path = self
            .get_sheet_xml_path(sheet)
            .ok_or_else(|| ErrSheetNotExist {
                sheet_name: sheet.to_string(),
            })?;
        let mut ws = self.work_sheet_reader(sheet)?;
        set_panes_ws(&mut ws, panes)?;
        self.sheet.insert(path, ws);
        Ok(())
    }

    /// Get freeze panes, split panes, and worksheet views by given worksheet name.
    pub fn get_panes(&self, sheet: &str) -> Result<Panes> {
        let ws = self.work_sheet_reader(sheet)?;
        Ok(get_panes_ws(&ws))
    }

    /// Search cell reference by given worksheet name, cell value, and regular expression.
    pub fn search_sheet(&self, sheet: &str, value: &str, reg: Option<bool>) -> Result<Vec<String>> {
        crate::excelize::check_sheet_name(sheet)?;
        let reg_search = reg.unwrap_or(false);
        let name = self
            .get_sheet_xml_path(sheet)
            .ok_or_else(|| ErrSheetNotExist {
                sheet_name: sheet.to_string(),
            })?;
        if let Some(ws) = self.sheet.get(&name) {
            let ws = ws.clone();
            self.save_file_list(&name, &worksheet_bytes(&ws));
        }
        search_sheet_impl(self, sheet, value, reg_search)
    }

    /// Protect worksheet to prevent accidental or deliberate changes.
    pub fn protect_sheet(&self, sheet: &str, opts: &SheetProtectionOptions) -> Result<()> {
        let path = self
            .get_sheet_xml_path(sheet)
            .ok_or_else(|| ErrSheetNotExist {
                sheet_name: sheet.to_string(),
            })?;
        let mut ws = self.work_sheet_reader(sheet)?;
        let mut sp = XlsxSheetProtection {
            auto_filter: !opts.auto_filter,
            delete_columns: !opts.delete_columns,
            delete_rows: !opts.delete_rows,
            format_cells: !opts.format_cells,
            format_columns: !opts.format_columns,
            format_rows: !opts.format_rows,
            insert_columns: !opts.insert_columns,
            insert_hyperlinks: !opts.insert_hyperlinks,
            insert_rows: !opts.insert_rows,
            objects: !opts.edit_objects,
            pivot_tables: !opts.pivot_tables,
            scenarios: !opts.edit_scenarios,
            select_locked_cells: !opts.select_locked_cells,
            select_unlocked_cells: !opts.select_unlocked_cells,
            sheet: true,
            sort: !opts.sort,
            ..Default::default()
        };
        if !opts.password.is_empty() {
            if opts.algorithm_name.is_empty() {
                sp.password = Some(gen_sheet_passwd(&opts.password));
            } else {
                let (hash_value, salt_value) =
                    gen_iso_passwd_hash(&opts.password, &opts.algorithm_name, "", 100_000)?;
                sp.password = None;
                sp.algorithm_name = Some(opts.algorithm_name.clone());
                sp.salt_value = Some(salt_value);
                sp.hash_value = Some(hash_value);
                sp.spin_count = Some(100_000);
            }
        }
        ws.sheet_protection = Some(sp);
        self.sheet.insert(path, ws);
        Ok(())
    }

    /// Remove protection for a sheet.
    pub fn unprotect_sheet(&self, sheet: &str, password: Option<&str>) -> Result<()> {
        let path = self
            .get_sheet_xml_path(sheet)
            .ok_or_else(|| ErrSheetNotExist {
                sheet_name: sheet.to_string(),
            })?;
        let mut ws = self.work_sheet_reader(sheet)?;
        if let Some(passwd) = password {
            let sp = ws.sheet_protection.as_ref().ok_or(ErrUnprotectSheet)?;
            if sp.algorithm_name.as_deref().unwrap_or("").is_empty() {
                if sp.password.as_deref().unwrap_or("") != gen_sheet_passwd(passwd) {
                    return Err(Box::new(ErrUnprotectSheetPassword));
                }
            } else {
                let (hash_value, _) = gen_iso_passwd_hash(
                    passwd,
                    sp.algorithm_name.as_deref().unwrap_or(""),
                    sp.salt_value.as_deref().unwrap_or(""),
                    sp.spin_count.unwrap_or(0) as i32,
                )?;
                if sp.hash_value.as_deref().unwrap_or("") != hash_value {
                    return Err(Box::new(ErrUnprotectSheetPassword));
                }
            }
        }
        ws.sheet_protection = None;
        self.sheet.insert(path, ws);
        Ok(())
    }

    /// Get worksheet protection settings by given worksheet name.
    pub fn get_sheet_protection(&self, sheet: &str) -> Result<SheetProtectionOptions> {
        let ws = self.work_sheet_reader(sheet)?;
        let mut opts = SheetProtectionOptions::default();
        if let Some(sp) = ws.sheet_protection {
            opts.algorithm_name = sp.algorithm_name.unwrap_or_default();
            opts.auto_filter = !sp.auto_filter;
            opts.delete_columns = !sp.delete_columns;
            opts.delete_rows = !sp.delete_rows;
            opts.edit_objects = !sp.objects;
            opts.edit_scenarios = !sp.scenarios;
            opts.format_cells = !sp.format_cells;
            opts.format_columns = !sp.format_columns;
            opts.format_rows = !sp.format_rows;
            opts.insert_columns = !sp.insert_columns;
            opts.insert_hyperlinks = !sp.insert_hyperlinks;
            opts.insert_rows = !sp.insert_rows;
            opts.pivot_tables = !sp.pivot_tables;
            opts.select_locked_cells = !sp.select_locked_cells;
            opts.select_unlocked_cells = !sp.select_unlocked_cells;
            opts.sort = !sp.sort;
        }
        Ok(opts)
    }

    /// Group worksheets by given worksheet names.
    pub fn group_sheets(&self, sheets: &[String]) -> Result<()> {
        let active_sheet = self.get_active_sheet_index()?;
        let sheet_list = self.get_sheet_list();
        let mut in_active_sheet = false;
        for (idx, sheet_name) in sheet_list.iter().enumerate() {
            for s in sheets {
                if s.eq_ignore_ascii_case(sheet_name) && (idx + 1) as i32 == active_sheet {
                    in_active_sheet = true;
                }
            }
        }
        if !in_active_sheet {
            return Err(Box::new(ErrGroupSheets));
        }
        for sheet in sheets {
            let mut ws = self.work_sheet_reader(sheet)?;
            let path = self.get_sheet_xml_path(sheet).unwrap_or_default();
            if let Some(views) = ws.sheet_views.as_mut() {
                for view in &mut views.sheet_view {
                    view.tab_selected = Some(true);
                }
            }
            self.sheet.insert(path, ws);
        }
        Ok(())
    }

    /// Ungroup worksheets.
    pub fn ungroup_sheets(&self) -> Result<()> {
        let active_sheet = self.get_active_sheet_index()?;
        for (index, sheet) in self.get_sheet_list().iter().enumerate() {
            if active_sheet == (index + 1) as i32 {
                continue;
            }
            let mut ws = self.work_sheet_reader(sheet)?;
            let path = self.get_sheet_xml_path(sheet).unwrap_or_default();
            if let Some(views) = ws.sheet_views.as_mut() {
                for view in &mut views.sheet_view {
                    view.tab_selected = Some(false);
                }
            }
            self.sheet.insert(path, ws);
        }
        Ok(())
    }

    /// Create a page break by given worksheet name and cell reference.
    pub fn insert_page_break(&self, sheet: &str, cell: &str) -> Result<()> {
        let path = self
            .get_sheet_xml_path(sheet)
            .ok_or_else(|| ErrSheetNotExist {
                sheet_name: sheet.to_string(),
            })?;
        let mut ws = self.work_sheet_reader(sheet)?;
        insert_page_break_ws(&mut ws, cell)?;
        self.sheet.insert(path, ws);
        Ok(())
    }

    /// Remove a page break by given worksheet name and cell reference.
    pub fn remove_page_break(&self, sheet: &str, cell: &str) -> Result<()> {
        let path = self
            .get_sheet_xml_path(sheet)
            .ok_or_else(|| ErrSheetNotExist {
                sheet_name: sheet.to_string(),
            })?;
        let mut ws = self.work_sheet_reader(sheet)?;
        remove_page_break_ws(&mut ws, cell)?;
        self.sheet.insert(path, ws);
        Ok(())
    }

    /// Set or remove the used range of the worksheet by a given range reference.
    pub fn set_sheet_dimension(&self, sheet: &str, range_ref: &str) -> Result<()> {
        let path = self
            .get_sheet_xml_path(sheet)
            .ok_or_else(|| ErrSheetNotExist {
                sheet_name: sheet.to_string(),
            })?;
        let mut ws = self.work_sheet_reader(sheet)?;
        if range_ref.is_empty() {
            ws.dimension = None;
            self.sheet.insert(path, ws);
            return Ok(());
        }
        let parts: Vec<&str> = range_ref.split(':').collect();
        if parts.len() == 1 {
            let _ = cell_name_to_coordinates(range_ref)?;
            ws.dimension = Some(XlsxDimension {
                r#ref: range_ref.to_uppercase(),
            });
            self.sheet.insert(path, ws);
            return Ok(());
        }
        if parts.len() != 2 {
            return Err(Box::new(ErrParameterInvalid));
        }
        let mut coordinates = range_ref_to_coordinates(range_ref)?;
        sort_coordinates(&mut coordinates)?;
        let r#ref = coordinates_to_range_ref(&coordinates, false)?;
        ws.dimension = Some(XlsxDimension { r#ref });
        self.sheet.insert(path, ws);
        Ok(())
    }

    /// Get the used range of the worksheet.
    pub fn get_sheet_dimension(&self, sheet: &str) -> Result<String> {
        crate::excelize::check_sheet_name(sheet)?;
        let name = self
            .get_sheet_xml_path(sheet)
            .ok_or_else(|| ErrSheetNotExist {
                sheet_name: sheet.to_string(),
            })?;
        if let Some(ws) = self.sheet.get(&name) {
            return Ok(ws
                .dimension
                .as_ref()
                .map(|d| d.r#ref.clone())
                .unwrap_or_default());
        }
        let data = self.read_bytes(&name);
        let s = String::from_utf8_lossy(&data);
        if let Some(start) = s.find("<dimension") {
            if let Some(end) = s[start..].find('>') {
                let tag = &s[start..start + end + 1];
                if let Some(pos) = tag.find("ref=\"") {
                    let rest = &tag[pos + 6..];
                    if let Some(q) = rest.find('"') {
                        return Ok(rest[..q].to_string());
                    }
                }
            }
        }
        Ok(String::new())
    }

    /// Ignore error for a range of cells.
    pub fn add_ignored_errors(
        &self,
        sheet: &str,
        range_ref: &str,
        ignored_errors_type: IgnoredErrorsType,
    ) -> Result<()> {
        if range_ref.is_empty() {
            return Err(Box::new(ErrParameterInvalid));
        }
        let path = self
            .get_sheet_xml_path(sheet)
            .ok_or_else(|| ErrSheetNotExist {
                sheet_name: sheet.to_string(),
            })?;
        let mut ws = self.work_sheet_reader(sheet)?;
        if ws.ignored_errors.is_none() {
            ws.ignored_errors = Some(XlsxIgnoredErrors::default());
        }
        let ie = ignored_error_for_type(range_ref, ignored_errors_type);
        let ignored = ws.ignored_errors.as_mut().unwrap();
        if ignored
            .ignored_error
            .iter()
            .any(|e| e.sqref == ie.sqref && same_ignored_error_flags(e, &ie))
        {
            self.sheet.insert(path, ws);
            return Ok(());
        }
        ignored.ignored_error.push(ie);
        self.sheet.insert(path, ws);
        Ok(())
    }
}

// ------------------------------------------------------------------
// Internal helpers
// ------------------------------------------------------------------

impl File {
    /// Build a map of worksheet name → XML path from the workbook and its rels.
    pub fn get_sheet_name_to_path_map(&self) -> Result<HashMap<String, String>> {
        let mut map = HashMap::new();
        let wb = self.workbook_reader()?;
        let rels = self.rels_reader(&self.get_workbook_rels_path())?;
        let Some(rels) = rels else {
            return Ok(map);
        };
        for sheet in &wb.sheets.sheet {
            let Some(name) = &sheet.name else { continue };
            let id = sheet.id.as_ref().or(sheet.plain_id.as_ref());
            let Some(id) = id else { continue };
            for rel in &rels.relationships {
                if &rel.id == id {
                    let sheet_xml = self.get_worksheet_path(&rel.target);
                    if self.pkg.contains_key(&sheet_xml) || self.temp_files.contains_key(&sheet_xml)
                    {
                        map.insert(name.clone(), sheet_xml);
                    }
                }
            }
        }
        Ok(map)
    }

    /// Get worksheets, chart sheets and dialog sheets ID and name map of the workbook.
    pub fn get_sheet_map(&self) -> HashMap<i32, String> {
        let mut sheet_map = HashMap::new();
        if let Ok(wb) = self.workbook_reader() {
            for sheet in &wb.sheets.sheet {
                if let (Some(id), Some(name)) = (sheet.sheet_id, sheet.name.clone()) {
                    sheet_map.insert(id as i32, name);
                }
            }
        }
        sheet_map
    }

    /// Convert a relationship id to a 0-based sheet index.
    pub fn rel_id_to_sheet_id(&self, r_id: &str) -> Result<i32> {
        let wb = self.workbook_reader()?;
        for (i, sheet) in wb.sheets.sheet.iter().enumerate() {
            let id = sheet.id.as_deref().or(sheet.plain_id.as_deref());
            if id == Some(r_id) {
                return Ok(i as i32);
            }
        }
        Err(Box::new(ErrSheetIdx))
    }
}

fn new_worksheet() -> XlsxWorksheet {
    XlsxWorksheet {
        sheet_format_pr: Some(XlsxSheetFormatPr {
            default_row_height: 15.0,
            ..Default::default()
        }),
        ..Default::default()
    }
}

fn worksheet_bytes(ws: &XlsxWorksheet) -> Vec<u8> {
    use quick_xml::se::to_string;
    let body = to_string(ws).unwrap_or_default().into_bytes();
    [crate::constants::XML_HEADER.as_bytes(), &body].concat()
}

impl File {
    /// Set a picture reference on a worksheet for use as a background image.
    pub(crate) fn add_sheet_picture(&self, sheet: &str, r_id: i32) -> Result<()> {
        let path = self
            .get_sheet_xml_path(sheet)
            .ok_or_else(|| ErrSheetNotExist {
                sheet_name: sheet.to_string(),
            })?;
        let mut ws = self.work_sheet_reader(sheet)?;
        ws.picture = Some(XlsxPicture {
            rid: Some(format!("rId{r_id}")),
        });
        self.sheet.insert(path, ws);
        Ok(())
    }
}

fn set_panes_ws(ws: &mut XlsxWorksheet, panes: &Panes) -> Result<()> {
    let mut p = XlsxPane {
        active_pane: Some(panes.active_pane.clone()),
        top_left_cell: Some(panes.top_left_cell.clone()),
        x_split: Some(panes.x_split as f64),
        y_split: Some(panes.y_split as f64),
        ..Default::default()
    };
    if panes.freeze {
        p.state = Some("frozen".to_string());
    }
    if ws.sheet_views.is_none() {
        ws.sheet_views = Some(XlsxSheetViews::default());
    }
    let views = ws.sheet_views.as_mut().unwrap();
    if views.sheet_view.is_empty() {
        views.sheet_view.push(XlsxSheetView::default());
    }
    let idx = views.sheet_view.len() - 1;
    if !panes.freeze && !panes.split {
        views.sheet_view[idx].pane = None;
    } else {
        views.sheet_view[idx].pane = Some(p);
    }
    let mut selection = Vec::new();
    for s in &panes.selection {
        selection.push(XlsxSelection {
            active_cell: Some(s.active_cell.clone()),
            pane: Some(s.pane.clone()),
            sqref: Some(s.sqref.clone()),
            ..Default::default()
        });
    }
    views.sheet_view[idx].selection = selection;
    Ok(())
}

fn get_panes_ws(ws: &XlsxWorksheet) -> Panes {
    let mut panes = Panes::default();
    let Some(views) = ws.sheet_views.as_ref() else {
        return panes;
    };
    let Some(view) = views.sheet_view.last() else {
        return panes;
    };
    for s in &view.selection {
        panes.selection.push(Selection {
            sqref: s.sqref.clone().unwrap_or_default(),
            active_cell: s.active_cell.clone().unwrap_or_default(),
            pane: s.pane.clone().unwrap_or_default(),
        });
    }
    let Some(p) = view.pane.as_ref() else {
        return panes;
    };
    panes.active_pane = p.active_pane.clone().unwrap_or_default();
    if p.state.as_deref() == Some("frozen") {
        panes.freeze = true;
    }
    panes.top_left_cell = p.top_left_cell.clone().unwrap_or_default();
    panes.x_split = p.x_split.unwrap_or(0.0) as i64;
    panes.y_split = p.y_split.unwrap_or(0.0) as i64;
    panes
}

fn search_sheet_impl(file: &File, sheet: &str, value: &str, reg: bool) -> Result<Vec<String>> {
    let name = file
        .get_sheet_xml_path(sheet)
        .ok_or_else(|| ErrSheetNotExist {
            sheet_name: sheet.to_string(),
        })?;
    let raw = {
        let raw = file.read_xml(&name);
        if !raw.is_empty() {
            raw
        } else if let Some(ws) = file.sheet.get(&name) {
            worksheet_bytes(&ws)
        } else {
            file.read_bytes(&name)
        }
    };
    let data = file.apply_charset_transcoder(&raw)?;
    let data = namespace_strict_to_transitional(&data);
    let sst = file.shared_strings_reader().ok();

    let regex = if reg {
        Some(regex::Regex::new(value).map_err(|e| e.to_string())?)
    } else {
        None
    };

    let mut result = Vec::new();
    let mut merge_cells = Vec::new();

    let mut reader = Reader::from_reader(Cursor::new(data));
    let mut buf = Vec::new();

    let mut _current_row = 0;
    let mut in_cell = false;
    let mut in_v = false;
    let mut in_is = false;
    let mut in_is_t = false;
    let mut cell_ref = String::new();
    let mut cell_type = None::<String>;
    let mut current_val = String::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let local_name = e.local_name();
                let name = local_name.as_ref();
                if name == b"row" {
                    if let Ok(Some(attr)) = e.try_get_attribute("r") {
                        if let Ok(r_str) = attr.decode_and_unescape_value(reader.decoder()) {
                            _current_row = r_str.parse::<i32>().unwrap_or(0);
                        }
                    }
                } else if name == b"c" {
                    in_cell = true;
                    in_v = false;
                    in_is = false;
                    in_is_t = false;
                    cell_ref.clear();
                    cell_type = None;
                    current_val.clear();
                    if let Ok(Some(attr)) = e.try_get_attribute("r") {
                        if let Ok(r) = attr.decode_and_unescape_value(reader.decoder()) {
                            cell_ref = r.to_string();
                        }
                    }
                    if let Ok(Some(attr)) = e.try_get_attribute("t") {
                        if let Ok(t) = attr.decode_and_unescape_value(reader.decoder()) {
                            cell_type = Some(t.to_string());
                        }
                    }
                } else if in_cell && name == b"v" {
                    in_v = true;
                    current_val.clear();
                } else if in_cell && name == b"is" {
                    in_is = true;
                    current_val.clear();
                } else if in_is && name == b"t" {
                    in_is_t = true;
                }
            }
            Ok(Event::Empty(e)) => {
                let local_name = e.local_name();
                let name = local_name.as_ref();
                if name == b"c" {
                    cell_ref.clear();
                    cell_type = None;
                    current_val.clear();
                    if let Ok(Some(attr)) = e.try_get_attribute("r") {
                        if let Ok(r) = attr.decode_and_unescape_value(reader.decoder()) {
                            cell_ref = r.to_string();
                        }
                    }
                    if let Ok(Some(attr)) = e.try_get_attribute("t") {
                        if let Ok(t) = attr.decode_and_unescape_value(reader.decoder()) {
                            cell_type = Some(t.to_string());
                        }
                    }
                    evaluate_cell(
                        &mut result,
                        &cell_ref,
                        cell_type.as_deref(),
                        &current_val,
                        sst.as_ref(),
                        value,
                        reg,
                        regex.as_ref(),
                    );
                } else if name == b"mergeCell" {
                    if let Ok(Some(attr)) = e.try_get_attribute("ref") {
                        if let Ok(r) = attr.decode_and_unescape_value(reader.decoder()) {
                            merge_cells.push(r.to_string());
                        }
                    }
                }
            }
            Ok(Event::Text(e)) => {
                if in_v || in_is_t {
                    if let Ok(text) = e.unescape() {
                        current_val.push_str(&text);
                    }
                }
            }
            Ok(Event::End(e)) => {
                let local_name = e.local_name();
                let name = local_name.as_ref();
                if in_cell && name == b"c" {
                    evaluate_cell(
                        &mut result,
                        &cell_ref,
                        cell_type.as_deref(),
                        &current_val,
                        sst.as_ref(),
                        value,
                        reg,
                        regex.as_ref(),
                    );
                    in_cell = false;
                    in_v = false;
                    in_is = false;
                    in_is_t = false;
                    cell_ref.clear();
                    cell_type = None;
                    current_val.clear();
                } else if in_v && name == b"v" {
                    in_v = false;
                } else if in_is && name == b"is" {
                    in_is = false;
                    in_is_t = false;
                } else if in_is_t && name == b"t" {
                    in_is_t = false;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(Box::new(e)),
            _ => {}
        }
        buf.clear();
    }

    if !merge_cells.is_empty() {
        for cell_ref in &mut result {
            for merge_ref in &merge_cells {
                if cell_in_range(cell_ref, merge_ref) {
                    if let Some(top_left) = merge_ref.split(':').next() {
                        *cell_ref = top_left.to_string();
                    }
                }
            }
        }
    }

    Ok(result)
}

fn evaluate_cell(
    result: &mut Vec<String>,
    cell_ref: &str,
    cell_type: Option<&str>,
    current_val: &str,
    sst: Option<&crate::xml::shared_strings::XlsxSst>,
    value: &str,
    reg: bool,
    regex: Option<&regex::Regex>,
) {
    if cell_ref.is_empty() {
        return;
    }
    let val = match cell_type {
        Some("s") => {
            if let Ok(idx) = current_val.parse::<usize>() {
                sst.and_then(|s| s.si.get(idx))
                    .map(|si| extract_si_text(si))
                    .unwrap_or_default()
            } else {
                String::new()
            }
        }
        Some("inlineStr") => current_val.to_string(),
        Some("str") => current_val.to_string(),
        _ => current_val.to_string(),
    };
    let matched = if reg {
        regex.unwrap().is_match(&val)
    } else {
        val == value
    };
    if matched {
        result.push(cell_ref.to_string());
    }
}

fn extract_si_text(si: &crate::xml::shared_strings::XlsxSi) -> String {
    if let Some(t) = &si.t {
        return t.val.clone();
    }
    si.r.iter()
        .filter_map(|r| r.t.as_ref().map(|t| t.val.clone()))
        .collect()
}

fn cell_in_range(cell: &str, range: &str) -> bool {
    let parts: Vec<&str> = range.split(':').collect();
    if parts.len() != 2 {
        return false;
    }
    let (c, r) = cell_name_to_coordinates(cell).unwrap_or((0, 0));
    let (c1, r1) = cell_name_to_coordinates(parts[0]).unwrap_or((0, 0));
    let (c2, r2) = cell_name_to_coordinates(parts[1]).unwrap_or((0, 0));
    c >= c1 && c <= c2 && r >= r1 && r <= r2
}

fn insert_page_break_ws(ws: &mut XlsxWorksheet, cell: &str) -> Result<()> {
    let (col, row) = cell_name_to_coordinates(cell).map_err(|e| e.to_string())?;
    let col = col - 1;
    let row = row - 1;
    if col == 0 && row == 0 {
        return Ok(());
    }
    if ws.row_breaks.is_none() {
        ws.row_breaks = Some(XlsxRowBreaks {
            breaks: XlsxBreaks::default(),
        });
    }
    if ws.col_breaks.is_none() {
        ws.col_breaks = Some(XlsxColBreaks {
            breaks: XlsxBreaks::default(),
        });
    }
    let row_breaks = ws.row_breaks.as_mut().unwrap();
    let col_breaks = ws.col_breaks.as_mut().unwrap();
    let mut row_brk = -1;
    let mut col_brk = -1;
    for (idx, brk) in row_breaks.breaks.brk.iter().enumerate() {
        if brk.id == Some(row as i64) {
            row_brk = idx as i32;
        }
    }
    for (idx, brk) in col_breaks.breaks.brk.iter().enumerate() {
        if brk.id == Some(col as i64) {
            col_brk = idx as i32;
        }
    }
    if row != 0 && row_brk == -1 {
        row_breaks.breaks.brk.push(XlsxBrk {
            id: Some(row as i64),
            max: Some((MAX_COLUMNS - 1) as i64),
            man: Some(true),
            ..Default::default()
        });
        row_breaks.breaks.manual_break_count =
            Some(row_breaks.breaks.manual_break_count.unwrap_or(0) + 1);
    }
    if col != 0 && col_brk == -1 {
        col_breaks.breaks.brk.push(XlsxBrk {
            id: Some(col as i64),
            max: Some((TOTAL_ROWS - 1) as i64),
            man: Some(true),
            ..Default::default()
        });
        col_breaks.breaks.manual_break_count =
            Some(col_breaks.breaks.manual_break_count.unwrap_or(0) + 1);
    }
    row_breaks.breaks.count = Some(row_breaks.breaks.brk.len() as i64);
    col_breaks.breaks.count = Some(col_breaks.breaks.brk.len() as i64);
    Ok(())
}

fn remove_page_break_ws(ws: &mut XlsxWorksheet, cell: &str) -> Result<()> {
    let (col, row) = cell_name_to_coordinates(cell).map_err(|e| e.to_string())?;
    let col = col - 1;
    let row = row - 1;
    if col == 0 && row == 0 {
        return Ok(());
    }
    if ws.row_breaks.is_none() || ws.col_breaks.is_none() {
        return Ok(());
    }
    let row_breaks = ws.row_breaks.as_mut().unwrap();
    let col_breaks = ws.col_breaks.as_mut().unwrap();
    let row_brks = row_breaks.breaks.brk.len();
    let col_brks = col_breaks.breaks.brk.len();
    let mut remove_brk = |id: i64, brks: &mut Vec<XlsxBrk>| {
        let mut i = 0;
        while i < brks.len() {
            if brks[i].id == Some(id) {
                brks.remove(i);
            } else {
                i += 1;
            }
        }
    };
    if row_brks > 0 && row_brks == col_brks {
        remove_brk(row as i64, &mut row_breaks.breaks.brk);
        remove_brk(col as i64, &mut col_breaks.breaks.brk);
        row_breaks.breaks.count = Some(row_breaks.breaks.brk.len() as i64);
        col_breaks.breaks.count = Some(col_breaks.breaks.brk.len() as i64);
        row_breaks.breaks.manual_break_count =
            Some(row_breaks.breaks.manual_break_count.unwrap_or(1) - 1);
        col_breaks.breaks.manual_break_count =
            Some(col_breaks.breaks.manual_break_count.unwrap_or(1) - 1);
        return Ok(());
    }
    if row_brks > 0 && row_brks > col_brks {
        remove_brk(row as i64, &mut row_breaks.breaks.brk);
        row_breaks.breaks.count = Some(row_breaks.breaks.brk.len() as i64);
        row_breaks.breaks.manual_break_count =
            Some(row_breaks.breaks.manual_break_count.unwrap_or(1) - 1);
        return Ok(());
    }
    if col_brks > 0 && col_brks > row_brks {
        remove_brk(col as i64, &mut col_breaks.breaks.brk);
        col_breaks.breaks.count = Some(col_breaks.breaks.brk.len() as i64);
        col_breaks.breaks.manual_break_count =
            Some(col_breaks.breaks.manual_break_count.unwrap_or(1) - 1);
    }
    Ok(())
}

fn get_sheet_state(visible: bool, very_hidden: Option<bool>) -> &'static str {
    if !visible && very_hidden == Some(true) {
        "veryHidden"
    } else {
        "hidden"
    }
}

fn gen_sheet_passwd(plaintext: &str) -> String {
    let mut password: i64 = 0x0000;
    let mut char_pos: u32 = 1;
    for v in plaintext.chars() {
        let value = (v as i64) << char_pos;
        char_pos += 1;
        let rotated_bits = value >> 15;
        let value = value & 0x7fff;
        password ^= value | rotated_bits;
    }
    password ^= plaintext.chars().count() as i64;
    password ^= 0xCE4B;
    format!("{password:X}")
}

fn image_extension_map() -> HashMap<String, String> {
    [
        (".bmp".to_string(), ".bmp".to_string()),
        (".emf".to_string(), ".emf".to_string()),
        (".emz".to_string(), ".emz".to_string()),
        (".gif".to_string(), ".gif".to_string()),
        (".ico".to_string(), ".ico".to_string()),
        (".jpg".to_string(), ".jpeg".to_string()),
        (".jpeg".to_string(), ".jpeg".to_string()),
        (".png".to_string(), ".png".to_string()),
        (".svg".to_string(), ".svg".to_string()),
        (".tif".to_string(), ".tiff".to_string()),
        (".tiff".to_string(), ".tiff".to_string()),
        (".wmf".to_string(), ".wmf".to_string()),
        (".wmz".to_string(), ".wmz".to_string()),
    ]
    .into_iter()
    .collect()
}

pub(crate) fn set_content_type_part_image_extensions(file: &File) -> Result<()> {
    // Matches Go setContentTypePartImageExtensions: register image types by
    // file extension as Defaults in [Content_Types].xml.
    let image_types: HashMap<String, String> = [
        ("bmp".to_string(), "image/".to_string()),
        ("ico".to_string(), "image/x-".to_string()),
        ("jpeg".to_string(), "image/".to_string()),
        ("png".to_string(), "image/".to_string()),
        ("gif".to_string(), "image/".to_string()),
        ("svg".to_string(), "image/".to_string()),
        ("tiff".to_string(), "image/".to_string()),
        ("emf".to_string(), "image/x-".to_string()),
        ("wmf".to_string(), "image/x-".to_string()),
        ("emz".to_string(), "image/x-".to_string()),
        ("wmz".to_string(), "image/x-".to_string()),
    ]
    .into_iter()
    .collect();
    let mut ct = file.content_types_reader()?;
    for (extension, prefix) in image_types {
        let exists = ct.entries.iter().any(|e| {
            if let crate::xml::content_types::XlsxContentTypeEntry::Default(d) = e {
                d.extension == extension
            } else {
                false
            }
        });
        if !exists {
            let content_type = format!("{prefix}{extension}");
            ct.entries.push(XlsxContentTypeEntry::Default(XlsxDefault {
                extension: extension.clone(),
                content_type,
            }));
        }
    }
    *file.content_types.lock().unwrap() = Some(ct);
    Ok(())
}

fn ignored_error_for_type(range_ref: &str, t: IgnoredErrorsType) -> XlsxIgnoredError {
    let base = XlsxIgnoredError {
        sqref: range_ref.to_string(),
        ..Default::default()
    };
    match t {
        IgnoredErrorsType::EvalError => XlsxIgnoredError {
            eval_error: Some(true),
            ..base
        },
        IgnoredErrorsType::TwoDigitTextYear => XlsxIgnoredError {
            two_digit_text_year: Some(true),
            ..base
        },
        IgnoredErrorsType::NumberStoredAsText => XlsxIgnoredError {
            number_stored_as_text: Some(true),
            ..base
        },
        IgnoredErrorsType::Formula => XlsxIgnoredError {
            formula: Some(true),
            ..base
        },
        IgnoredErrorsType::FormulaRange => XlsxIgnoredError {
            formula_range: Some(true),
            ..base
        },
        IgnoredErrorsType::UnlockedFormula => XlsxIgnoredError {
            unlocked_formula: Some(true),
            ..base
        },
        IgnoredErrorsType::EmptyCellReference => XlsxIgnoredError {
            empty_cell_reference: Some(true),
            ..base
        },
        IgnoredErrorsType::ListDataValidation => XlsxIgnoredError {
            list_data_validation: Some(true),
            ..base
        },
        IgnoredErrorsType::CalculatedColumn => XlsxIgnoredError {
            calculated_column: Some(true),
            ..base
        },
    }
}

fn same_ignored_error_flags(a: &XlsxIgnoredError, b: &XlsxIgnoredError) -> bool {
    a.eval_error == b.eval_error
        && a.two_digit_text_year == b.two_digit_text_year
        && a.number_stored_as_text == b.number_stored_as_text
        && a.formula == b.formula
        && a.formula_range == b.formula_range
        && a.unlocked_formula == b.unlocked_formula
        && a.empty_cell_reference == b.empty_cell_reference
        && a.list_data_validation == b.list_data_validation
        && a.calculated_column == b.calculated_column
}

fn adjust_range_sheet_name(data: &str, source: &str, target: &str) -> String {
    // Replace sheet name references in defined name formulas.
    let mut out = data.to_string();
    let patterns = vec![format!("{source}!"), format!("'{source}'!")];
    for p in patterns {
        out = out.replace(&p, &format!("{target}!"));
    }
    out
}

fn ensure_shared_strings_rel(rels: &mut crate::xml::workbook::XlsxRelationships) {
    for rel in &rels.relationships {
        if rel.r#type == SOURCE_RELATIONSHIP_SHARED_STRINGS {
            return;
        }
    }
    let mut max_rid = 0i32;
    for rel in &rels.relationships {
        if let Ok(n) = rel.id.trim_start_matches("rId").parse::<i32>() {
            if n > max_rid {
                max_rid = n;
            }
        }
    }
    rels.relationships
        .push(crate::xml::workbook::XlsxRelationship {
            id: format!("rId{}", max_rid + 1),
            r#type: SOURCE_RELATIONSHIP_SHARED_STRINGS.to_string(),
            target: "sharedStrings.xml".to_string(),
            target_mode: None,
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_sheet_round_trip() {
        let f = File::new_with_options(crate::options::Options::default());
        let idx = f.new_sheet("Sheet2").unwrap();
        assert_eq!(idx, 2);
        assert_eq!(f.get_sheet_name(idx).unwrap(), "Sheet2");
        assert_eq!(f.get_sheet_index("Sheet2").unwrap(), idx);
        assert!(f.get_sheet_list().contains(&"Sheet2".to_string()));
    }

    #[test]
    fn sheet_name_and_map_round_trip() {
        let f = File::new_with_options(crate::options::Options::default());
        f.new_sheet("Sheet2").unwrap();
        f.set_sheet_name("Sheet2", "NewName").unwrap();
        assert_eq!(f.get_sheet_index("NewName").unwrap(), 2);
        let map = f.get_sheet_map();
        assert!(map.values().any(|n| n == "NewName"));
    }

    #[test]
    fn sheet_visible_round_trip() {
        let f = File::new_with_options(crate::options::Options::default());
        f.new_sheet("Sheet2").unwrap();
        assert!(f.get_sheet_visible("Sheet2").unwrap());
        f.set_sheet_visible("Sheet2", false, None).unwrap();
        assert!(!f.get_sheet_visible("Sheet2").unwrap());
        f.set_sheet_visible("Sheet2", true, None).unwrap();
        assert!(f.get_sheet_visible("Sheet2").unwrap());
    }

    #[test]
    fn panes_round_trip() {
        let f = File::new_with_options(crate::options::Options::default());
        let panes = Panes {
            freeze: true,
            x_split: 1,
            top_left_cell: "B1".to_string(),
            active_pane: "topRight".to_string(),
            selection: vec![Selection {
                sqref: "K16".to_string(),
                active_cell: "K16".to_string(),
                pane: "topRight".to_string(),
            }],
            ..Default::default()
        };
        f.set_panes("Sheet1", &panes).unwrap();
        let got = f.get_panes("Sheet1").unwrap();
        assert_eq!(got.freeze, true);
        assert_eq!(got.x_split, 1);
        assert_eq!(got.top_left_cell, "B1");
    }

    #[test]
    fn sheet_protection_round_trip() {
        let f = File::new_with_options(crate::options::Options::default());
        let opts = SheetProtectionOptions {
            password: "password".to_string(),
            select_locked_cells: true,
            ..Default::default()
        };
        f.protect_sheet("Sheet1", &opts).unwrap();
        let got = f.get_sheet_protection("Sheet1").unwrap();
        assert!(got.select_locked_cells);
        f.unprotect_sheet("Sheet1", Some("password")).unwrap();
        let got = f.get_sheet_protection("Sheet1").unwrap();
        assert!(!got.select_locked_cells);
    }

    #[test]
    fn page_break_round_trip() {
        let f = File::new_with_options(crate::options::Options::default());
        f.insert_page_break("Sheet1", "B2").unwrap();
        let ws = f.work_sheet_reader("Sheet1").unwrap();
        assert!(ws.row_breaks.is_some());
        assert!(ws.col_breaks.is_some());
        f.remove_page_break("Sheet1", "B2").unwrap();
        let ws = f.work_sheet_reader("Sheet1").unwrap();
        assert!(ws.row_breaks.as_ref().unwrap().breaks.brk.is_empty());
    }

    #[test]
    fn sheet_dimension_round_trip() {
        let f = File::new_with_options(crate::options::Options::default());
        f.set_sheet_dimension("Sheet1", "A1:D5").unwrap();
        assert_eq!(f.get_sheet_dimension("Sheet1").unwrap(), "A1:D5");
        f.set_sheet_dimension("Sheet1", "").unwrap();
        assert!(f.get_sheet_dimension("Sheet1").unwrap().is_empty());
    }

    #[test]
    fn search_sheet_basic() {
        let f = File::new_with_options(crate::options::Options::default());
        f.set_cell_str("Sheet1", "A1", "hello").unwrap();
        f.set_cell_str("Sheet1", "B2", "world").unwrap();
        let refs = f.search_sheet("Sheet1", "hello", None).unwrap();
        assert!(refs.contains(&"A1".to_string()));
    }

    #[test]
    fn search_sheet_shared_string() {
        let f = File::new_with_options(crate::options::Options::default());
        f.set_cell_str("Sheet1", "A1", "shared_text").unwrap();
        let refs = f.search_sheet("Sheet1", "shared_text", None).unwrap();
        assert_eq!(refs, vec!["A1".to_string()]);
    }

    #[test]
    fn search_sheet_regex() {
        let f = File::new_with_options(crate::options::Options::default());
        f.set_cell_str("Sheet1", "A1", "hello world").unwrap();
        f.set_cell_str("Sheet1", "B2", "hello rust").unwrap();
        let refs = f.search_sheet("Sheet1", r"hello \w+", Some(true)).unwrap();
        assert_eq!(refs.len(), 2);
        assert!(refs.contains(&"A1".to_string()));
        assert!(refs.contains(&"B2".to_string()));
    }

    #[test]
    fn search_sheet_inline_string() {
        let f = File::new_with_options(crate::options::Options::default());
        f.set_cell_value(
            "Sheet1",
            "A1",
            vec![crate::xml::common::RichTextRun {
                text: "inline".to_string(),
                ..Default::default()
            }],
        )
        .unwrap();
        let refs = f.search_sheet("Sheet1", "inline", None).unwrap();
        assert_eq!(refs, vec!["A1".to_string()]);
    }

    #[test]
    fn search_sheet_rich_text_shared_string() {
        let f = File::new_with_options(crate::options::Options::default());
        let runs = vec![
            crate::xml::common::RichTextRun {
                text: "bold".to_string(),
                ..Default::default()
            },
            crate::xml::common::RichTextRun {
                text: "text".to_string(),
                ..Default::default()
            },
        ];
        f.set_cell_rich_text("Sheet1", "A1", runs).unwrap();
        let refs = f.search_sheet("Sheet1", "boldtext", None).unwrap();
        assert_eq!(refs, vec!["A1".to_string()]);
    }

    #[test]
    fn search_sheet_merge_cells() {
        let f = File::new_with_options(crate::options::Options::default());
        f.set_cell_str("Sheet1", "A1", "merged").unwrap();
        f.merge_cell("Sheet1", "A1", "B2").unwrap();
        let refs = f.search_sheet("Sheet1", "merged", None).unwrap();
        assert_eq!(refs, vec!["A1".to_string()]);
    }

    #[test]
    fn search_sheet_after_save_and_reopen() {
        let f = File::new_with_options(crate::options::Options::default());
        f.set_cell_str("Sheet1", "A1", "persisted").unwrap();
        let buf = f.write_to_buffer().unwrap();
        let f2 = File::open_reader(
            std::io::Cursor::new(buf),
            0,
            crate::options::Options::default(),
        )
        .unwrap();
        let refs = f2.search_sheet("Sheet1", "persisted", None).unwrap();
        assert_eq!(refs, vec!["A1".to_string()]);
    }

    #[test]
    fn ignored_errors_round_trip() {
        let f = File::new_with_options(crate::options::Options::default());
        f.add_ignored_errors("Sheet1", "A1:A2", IgnoredErrorsType::NumberStoredAsText)
            .unwrap();
        let ws = f.work_sheet_reader("Sheet1").unwrap();
        let errors = ws.ignored_errors.as_ref().unwrap();
        assert_eq!(errors.ignored_error.len(), 1);
        assert!(errors.ignored_error[0].number_stored_as_text.unwrap());
    }

    #[test]
    fn move_and_copy_sheet() {
        let f = File::new_with_options(crate::options::Options::default());
        f.new_sheet("Sheet2").unwrap();
        f.new_sheet("Sheet3").unwrap();
        f.move_sheet("Sheet3", "Sheet1").unwrap();
        let list = f.get_sheet_list();
        assert_eq!(list[0], "Sheet3");
        f.copy_sheet(1, 2).unwrap();
    }

    #[test]
    fn sheet_background_from_bytes() {
        let f = File::new_with_options(crate::options::Options::default());
        // 1x1 red PNG
        let png = vec![
            0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48,
            0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00,
            0x00, 0x90, 0x77, 0x53, 0xde, 0x00, 0x00, 0x00, 0x0c, 0x49, 0x44, 0x41, 0x54, 0x08,
            0x99, 0x63, 0xf8, 0x0f, 0x00, 0x00, 0x01, 0x01, 0x00, 0x05, 0x18, 0xd8, 0x4e, 0x00,
            0x00, 0x00, 0x00, 0x49, 0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
        ];
        f.set_sheet_background_from_bytes("Sheet1", "png", &png)
            .unwrap();
        let ws = f.work_sheet_reader("Sheet1").unwrap();
        assert!(ws.picture.is_some());
    }

    #[test]
    fn sheet_props_round_trip() {
        let f = File::new_with_options(crate::options::Options::default());
        let expected = SheetPropsOptions {
            code_name: Some("code".to_string()),
            enable_format_conditions_calculation: Some(true),
            published: Some(true),
            auto_page_breaks: Some(true),
            fit_to_page: Some(true),
            tab_color_indexed: Some(1),
            tab_color_rgb: Some("FFFF00".to_string()),
            tab_color_theme: Some(1),
            tab_color_tint: Some(1.0),
            outline_pr_summary_below: Some(true),
            outline_pr_summary_right: Some(true),
            base_col_width: Some(8),
            default_col_width: Some(10.0),
            default_row_height: Some(10.0),
            custom_height: Some(true),
            zero_height: Some(true),
            thick_top: Some(true),
            thick_bottom: Some(true),
            ..Default::default()
        };
        f.set_sheet_props("Sheet1", &expected).unwrap();
        let got = f.get_sheet_props("Sheet1").unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn page_margins_round_trip() {
        let f = File::new_with_options(crate::options::Options::default());
        let expected = PageMarginsOptions {
            left: Some(1.0),
            right: Some(1.0),
            top: Some(1.0),
            bottom: Some(1.0),
            header: Some(1.0),
            footer: Some(1.0),
            horizontally: Some(true),
            vertically: Some(true),
        };
        f.set_page_margins("Sheet1", &expected).unwrap();
        let got = f.get_page_margins("Sheet1").unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn sheet_view_round_trip() {
        let f = File::new_with_options(crate::options::Options::default());
        let expected = SheetViewOptions {
            default_grid_color: Some(false),
            right_to_left: Some(false),
            show_formulas: Some(false),
            show_grid_lines: Some(false),
            show_row_col_headers: Some(false),
            show_ruler: Some(false),
            show_zeros: Some(false),
            top_left_cell: Some("A1".to_string()),
            view: Some("normal".to_string()),
            zoom_scale: Some(120.0),
            // The new file template has tabSelected="1", so this stays true.
            tab_selected: Some(true),
            // Value-type fields that are not present in the XML default to false.
            show_outline_symbols: Some(false),
            show_white_space: Some(false),
            window_protection: Some(false),
        };
        f.set_sheet_view("Sheet1", 0, &expected).unwrap();
        let got = f.get_sheet_view("Sheet1", 0).unwrap();
        assert_eq!(expected, got);

        // Negative index points at the same single view.
        f.set_sheet_view("Sheet1", -1, &expected).unwrap();
        let got = f.get_sheet_view("Sheet1", -1).unwrap();
        assert_eq!(expected, got);

        // Invalid view values are ignored.
        let invalid_view = SheetViewOptions {
            view: Some("invalid".to_string()),
            ..Default::default()
        };
        f.set_sheet_view("Sheet1", 0, &invalid_view).unwrap();
        let got = f.get_sheet_view("Sheet1", 0).unwrap();
        assert_eq!(Some("normal".to_string()), got.view);

        // Invalid zoom values are ignored.
        let invalid_zoom = SheetViewOptions {
            zoom_scale: Some(5.0),
            ..Default::default()
        };
        f.set_sheet_view("Sheet1", 0, &invalid_zoom).unwrap();
        let got = f.get_sheet_view("Sheet1", 0).unwrap();
        assert_eq!(Some(120.0), got.zoom_scale);

        // Out-of-range indices return an error.
        let empty = SheetViewOptions::default();
        assert_eq!(
            "view index 1 out of range",
            f.set_sheet_view("Sheet1", 1, &empty)
                .unwrap_err()
                .to_string()
        );
        assert_eq!(
            "view index -2 out of range",
            f.set_sheet_view("Sheet1", -2, &empty)
                .unwrap_err()
                .to_string()
        );
        assert!(f.get_sheet_view("Sheet1", 1).is_err());
        assert!(f.get_sheet_view("Sheet1", -2).is_err());

        // Non-existent worksheet returns an error.
        assert!(f.set_sheet_view("SheetN", 0, &empty).is_err());
        assert!(f.get_sheet_view("SheetN", 0).is_err());
    }

    #[test]
    fn sheet_view_page_break_preview() {
        let f = File::new_with_options(crate::options::Options::default());
        let opts = SheetViewOptions {
            view: Some("pageBreakPreview".to_string()),
            zoom_scale: Some(200.0),
            ..Default::default()
        };
        f.set_sheet_view("Sheet1", 0, &opts).unwrap();
        let got = f.get_sheet_view("Sheet1", 0).unwrap();
        assert_eq!(Some("pageBreakPreview".to_string()), got.view);
        assert_eq!(Some(200.0), got.zoom_scale);
    }

    #[test]
    fn sheet_props_defaults() {
        let f = File::new_with_options(crate::options::Options::default());
        let opts = f.get_sheet_props("Sheet1").unwrap();
        assert_eq!(Some(true), opts.enable_format_conditions_calculation);
        assert_eq!(Some(true), opts.published);
        assert_eq!(Some(true), opts.auto_page_breaks);
        assert_eq!(Some(true), opts.outline_pr_summary_below);
        assert_eq!(Some(8), opts.base_col_width);
        assert_eq!(Some(15.0), opts.default_row_height);
    }

    #[test]
    fn page_margins_defaults() {
        let f = File::new_with_options(crate::options::Options::default());
        let opts = f.get_page_margins("Sheet1").unwrap();
        assert_eq!(Some(0.7), opts.left);
        assert_eq!(Some(0.7), opts.right);
        assert_eq!(Some(0.75), opts.top);
        assert_eq!(Some(0.75), opts.bottom);
        assert_eq!(Some(0.3), opts.header);
        assert_eq!(Some(0.3), opts.footer);
    }
}
