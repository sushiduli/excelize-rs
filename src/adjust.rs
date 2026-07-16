//! Row/column adjustment helpers.
//!
//! This module corresponds to `adjust.go` in the Go implementation. It updates
//! cell references, formulas, merged cells, auto filters, tables, hyperlinks,
//! conditional formats, data validations and the calculation chain when rows or
//! columns are inserted or deleted.

#![allow(dead_code, unused_imports)]

use quick_xml::de::from_reader as xml_from_reader;
use quick_xml::se::to_string as xml_to_string;

use crate::constants::{MAX_COLUMNS, TOTAL_ROWS};
use crate::errors::{ErrColumnNumber, ErrMaxRows, Result, new_not_worksheet_error};
use crate::file::{File, namespace_strict_to_transitional};
use crate::lib_util::{
    cell_name_to_coordinates, column_number_to_name, coordinates_to_cell_name,
    coordinates_to_range_ref, join_cell_name, range_ref_to_coordinates, split_cell_name,
};
use crate::xml::common::XlsxInnerXml;
use crate::xml::drawing::{XdrCellAnchor, XlsxFrom, XlsxTo};
use crate::xml::table::{XlsxAutoFilter, XlsxTable};
use crate::xml::workbook::XlsxWorkbook;
use crate::xml::worksheet::{
    XlsxC, XlsxConditionalFormatting, XlsxDataValidation, XlsxDataValidations, XlsxF,
    XlsxHyperlinks, XlsxMergeCells, XlsxRow, XlsxTableParts, XlsxWorksheet,
};

/// Direction of the adjustment operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdjustDirection {
    Columns,
    Rows,
}

impl File {
    /// Main entry point used by row/column insertion and deletion routines.
    pub(crate) fn adjust_helper(
        &self,
        sheet: &str,
        dir: AdjustDirection,
        num: i32,
        offset: i32,
    ) -> Result<()> {
        let path =
            self.get_sheet_xml_path(sheet)
                .ok_or_else(|| crate::errors::ErrSheetNotExist {
                    sheet_name: sheet.to_string(),
                })?;
        let mut ws = self.work_sheet_reader(sheet)?;
        self.clear_calc_cache();
        let sheet_id = self.get_sheet_id(sheet);

        if dir == AdjustDirection::Rows {
            self.adjust_row_dimensions(sheet, &mut ws, num, offset)?;
        } else {
            self.adjust_col_dimensions(sheet, &mut ws, num, offset)?;
        }

        self.adjust_hyperlinks(&mut ws, sheet, dir, num, offset);
        let _ = self.check_sheet(&mut ws);
        let _ = self.check_row(&mut ws);

        self.adjust_conditional_formats(&mut ws, sheet, dir, num, offset, sheet_id)?;
        self.adjust_data_validations(&mut ws, sheet, dir, num, offset, sheet_id)?;
        self.adjust_defined_names(sheet, dir, num, offset)?;
        self.adjust_drawings(&mut ws, sheet, dir, num, offset)?;
        self.adjust_merge_cells(&mut ws, sheet, dir, num, offset, sheet_id)?;
        self.adjust_auto_filter(&mut ws, sheet, dir, num, offset, sheet_id)?;
        self.adjust_calc_chain(&mut ws, sheet, dir, num, offset, sheet_id)?;
        self.adjust_table(&mut ws, sheet, dir, num, offset, sheet_id)?;
        self.adjust_volatile_deps(&mut ws, sheet, dir, num, offset, sheet_id)?;

        if let Some(merge_cells) = &ws.merge_cells {
            if merge_cells.cells.is_empty() {
                ws.merge_cells = None;
            }
        }

        self.sheet.insert(path, ws);
        Ok(())
    }

    // ------------------------------------------------------------------
    // Dimension adjustments
    // ------------------------------------------------------------------

    fn adjust_cols(&self, ws: &mut XlsxWorksheet, col: i32, offset: i32) -> Result<()> {
        let cols = match ws.cols.as_mut() {
            Some(c) => c,
            None => return Ok(()),
        };

        let mut i = 0;
        while i < cols.col.len() {
            if offset > 0 {
                if cols.col[i].min >= col as i64 {
                    let new_min = cols.col[i].min + offset as i64;
                    if new_min > MAX_COLUMNS as i64 {
                        cols.col.remove(i);
                        continue;
                    }
                    cols.col[i].min = new_min;
                }
                if cols.col[i].max >= col as i64 || cols.col[i].max + 1 == col as i64 {
                    let new_max = cols.col[i].max + offset as i64;
                    cols.col[i].max = new_max.min(MAX_COLUMNS as i64);
                }
                i += 1;
                continue;
            }

            if cols.col[i].min == col as i64 && cols.col[i].max == col as i64 {
                cols.col.remove(i);
                continue;
            }
            if cols.col[i].min > col as i64 {
                cols.col[i].min += offset as i64;
            }
            if cols.col[i].max >= col as i64 {
                cols.col[i].max += offset as i64;
            }
            i += 1;
        }

        if cols.col.is_empty() {
            ws.cols = None;
        }
        Ok(())
    }

    fn adjust_col_dimensions(
        &self,
        sheet: &str,
        ws: &mut XlsxWorksheet,
        col: i32,
        offset: i32,
    ) -> Result<()> {
        for row in &ws.sheet_data.row {
            for cell in &row.c {
                if let Some(r) = &cell.r {
                    if let Ok((cell_col, _)) = cell_name_to_coordinates(r) {
                        if col <= cell_col {
                            let new_col = cell_col + offset;
                            if new_col > 0 && new_col > MAX_COLUMNS {
                                return Err(Box::new(ErrColumnNumber));
                            }
                        }
                    }
                }
            }
        }

        for sheet_n in self.get_sheet_list() {
            let mut worksheet = match self.work_sheet_reader(&sheet_n) {
                Ok(w) => w,
                Err(e) => {
                    if e.to_string() == new_not_worksheet_error(&sheet_n) {
                        continue;
                    }
                    return Err(e);
                }
            };
            let path = match self.get_sheet_xml_path(&sheet_n) {
                Some(p) => p,
                None => continue,
            };

            for row_idx in 0..worksheet.sheet_data.row.len() {
                for col_idx in 0..worksheet.sheet_data.row[row_idx].c.len() {
                    let cell = &worksheet.sheet_data.row[row_idx].c[col_idx];
                    if let Some(r) = &cell.r {
                        if let Ok((cell_col, cell_row)) = cell_name_to_coordinates(r) {
                            if sheet_n.eq_ignore_ascii_case(sheet) && col <= cell_col {
                                let new_col = cell_col + offset;
                                if new_col > 0 {
                                    worksheet.sheet_data.row[row_idx].c[col_idx].r =
                                        Some(coordinates_to_cell_name(new_col, cell_row, false)?);
                                }
                            }
                        }
                    }
                    let cell_ref = &mut worksheet.sheet_data.row[row_idx].c[col_idx];
                    self.adjust_formula(
                        sheet,
                        &sheet_n,
                        cell_ref,
                        AdjustDirection::Columns,
                        col,
                        offset,
                        false,
                    )?;
                }
            }
            self.sheet.insert(path, worksheet);
        }

        // Synchronize `ws` with the updated active-sheet copy that was written
        // back into the sheet cache during the loop above.
        let path = self.get_sheet_xml_path(sheet).unwrap_or_default();
        if let Some(updated) = self.sheet.get(&path) {
            *ws = updated.clone();
        }

        self.adjust_cols(ws, col, offset)
    }

    fn adjust_row_dimensions(
        &self,
        sheet: &str,
        ws: &mut XlsxWorksheet,
        row: i32,
        offset: i32,
    ) -> Result<()> {
        for sheet_n in self.get_sheet_list() {
            if sheet_n.eq_ignore_ascii_case(sheet) {
                continue;
            }
            let mut worksheet = match self.work_sheet_reader(&sheet_n) {
                Ok(w) => w,
                Err(e) => {
                    if e.to_string() == new_not_worksheet_error(&sheet_n) {
                        continue;
                    }
                    return Err(e);
                }
            };
            let path = self.get_sheet_xml_path(&sheet_n).unwrap_or_default();
            for i in 0..worksheet.sheet_data.row.len() {
                let r = &mut worksheet.sheet_data.row[i];
                self.adjust_single_row_formulas(sheet, &sheet_n, r, row, offset, false)?;
            }
            self.sheet.insert(path, worksheet);
        }

        let total_rows = ws.sheet_data.row.len();
        if total_rows == 0 {
            return Ok(());
        }
        if let Some(last_r) = ws.sheet_data.row[total_rows - 1].r {
            let new_row = last_r + offset as i64;
            if last_r >= row as i64 && new_row > 0 && new_row > TOTAL_ROWS as i64 {
                return Err(Box::new(ErrMaxRows));
            }
        }

        for i in 0..ws.sheet_data.row.len() {
            if let Some(row_num) = ws.sheet_data.row[i].r {
                let new_row = row_num + offset as i64;
                if row_num >= row as i64 && new_row > 0 {
                    adjust_single_row_dimensions(&mut ws.sheet_data.row[i], offset);
                }
            }
            let r = &mut ws.sheet_data.row[i];
            self.adjust_single_row_formulas(sheet, sheet, r, row, offset, false)?;
        }
        Ok(())
    }

    pub(crate) fn adjust_single_row_formulas(
        &self,
        sheet: &str,
        sheet_n: &str,
        r: &mut XlsxRow,
        num: i32,
        offset: i32,
        si: bool,
    ) -> Result<()> {
        for i in 0..r.c.len() {
            self.adjust_formula(
                sheet,
                sheet_n,
                &mut r.c[i],
                AdjustDirection::Rows,
                num,
                offset,
                si,
            )?;
        }
        Ok(())
    }

    // ------------------------------------------------------------------
    // Cell reference / formula adjustment
    // ------------------------------------------------------------------

    fn adjust_cell_ref(
        &self,
        cell_ref: &str,
        dir: AdjustDirection,
        num: i32,
        offset: i32,
    ) -> Result<String> {
        let mut sq_ref = Vec::new();
        for ref_ in cell_ref.split(' ').filter(|s| !s.is_empty()) {
            let mut expanded = ref_.to_string();
            if !expanded.contains(':') {
                expanded.push(':');
                expanded.push_str(ref_);
            }
            let mut coordinates = range_ref_to_coordinates(&expanded)?;
            if dir == AdjustDirection::Columns {
                if offset < 0 && coordinates[0] == coordinates[2] && num == coordinates[0] {
                    continue;
                }
                apply_offset(&mut coordinates, 0, 2, MAX_COLUMNS, num, offset);
            } else {
                if offset < 0 && coordinates[1] == coordinates[3] && num == coordinates[1] {
                    continue;
                }
                apply_offset(&mut coordinates, 1, 3, TOTAL_ROWS, num, offset);
            }
            sq_ref.push(coordinates_to_range_ref(&coordinates, false)?);
        }
        Ok(sq_ref.join(" "))
    }

    fn adjust_formula(
        &self,
        sheet: &str,
        sheet_n: &str,
        cell: &mut XlsxC,
        dir: AdjustDirection,
        num: i32,
        offset: i32,
        si: bool,
    ) -> Result<()> {
        if let Some(f) = &mut cell.f {
            if !f.content.is_empty() {
                f.content =
                    self.adjust_formula_ref(sheet, sheet_n, &f.content, false, dir, num, offset)?;
            }
            if f.r#ref.is_some() && sheet == sheet_n {
                if let Some(ref_) = &mut f.r#ref {
                    *ref_ = self.adjust_cell_ref(ref_, dir, num, offset)?;
                }
                if si {
                    if let Some(si_val) = f.si.as_mut() {
                        *si_val += 1;
                    }
                }
            }
        }
        Ok(())
    }

    /// Formula reference rewriter.
    ///
    /// Parses the formula with the calculation-engine parser, walks the AST and
    /// shifts every cell/range reference that belongs to the sheet being
    /// adjusted, then re-serializes the formula.  If parsing fails the original
    /// formula is returned unchanged so that file operations are not broken by
    /// constructs the parser does not yet handle.

    fn adjust_formula_ref(
        &self,
        sheet: &str,
        sheet_n: &str,
        formula: &str,
        keep_relative: bool,
        dir: AdjustDirection,
        num: i32,
        offset: i32,
    ) -> Result<String> {
        if formula.trim().is_empty() {
            return Ok(formula.to_string());
        }
        let had_leading_eq = formula.starts_with('=');
        let expr = match crate::calc::parse_formula(formula) {
            Ok(e) => e,
            Err(_) => return Ok(formula.to_string()),
        };
        let mut expr = expr;
        adjust_expr(&mut expr, dir, num, offset, keep_relative, sheet, sheet_n);
        let mut result = format_expr(&expr)?;
        if had_leading_eq {
            result.insert(0, '=');
        }
        Ok(result)
    }

    // ------------------------------------------------------------------
    // Hyperlinks
    // ------------------------------------------------------------------

    fn adjust_hyperlinks(
        &self,
        ws: &mut XlsxWorksheet,
        sheet: &str,
        dir: AdjustDirection,
        num: i32,
        offset: i32,
    ) {
        if ws.hyperlinks.is_none() || ws.hyperlinks.as_ref().unwrap().hyperlink.is_empty() {
            return;
        }

        if offset < 0 {
            let mut i = ws.hyperlinks.as_ref().unwrap().hyperlink.len();
            while i > 0 {
                i -= 1;
                let link_data = ws.hyperlinks.as_ref().unwrap().hyperlink[i].clone();
                if let Ok((col_num, row_num)) = cell_name_to_coordinates(&link_data.r#ref) {
                    if (dir == AdjustDirection::Rows && num == row_num)
                        || (dir == AdjustDirection::Columns && num == col_num)
                    {
                        if let Some(rid) = &link_data.rid {
                            self.delete_sheet_relationships(sheet, rid);
                        }
                        if ws.hyperlinks.as_ref().unwrap().hyperlink.len() > 1 {
                            ws.hyperlinks.as_mut().unwrap().hyperlink.remove(i);
                        } else {
                            ws.hyperlinks = None;
                        }
                    }
                }
            }
        }

        if ws.hyperlinks.is_none() {
            return;
        }
        for link in ws.hyperlinks.as_mut().unwrap().hyperlink.iter_mut() {
            if let Ok(new_ref) =
                self.adjust_formula_ref(sheet, sheet, &link.r#ref, false, dir, num, offset)
            {
                link.r#ref = new_ref;
            }
        }
    }

    // ------------------------------------------------------------------
    // Tables
    // ------------------------------------------------------------------

    fn adjust_table(
        &self,
        ws: &mut XlsxWorksheet,
        sheet: &str,
        dir: AdjustDirection,
        num: i32,
        offset: i32,
        _sheet_id: i32,
    ) -> Result<()> {
        if ws.table_parts.is_none() || ws.table_parts.as_ref().unwrap().table_part.is_empty() {
            return Ok(());
        }

        let mut idx = 0;
        while idx < ws.table_parts.as_ref().unwrap().table_part.len() {
            let tbl = ws.table_parts.as_ref().unwrap().table_part[idx].clone();
            let target =
                self.get_sheet_relationships_target_by_id(sheet, tbl.rid.as_deref().unwrap_or(""));
            if target.is_empty() {
                idx += 1;
                continue;
            }
            let table_xml = target.replace("..", "xl");
            let content = self.read_xml(&table_xml);
            if content.is_empty() {
                idx += 1;
                continue;
            }
            let mut t: XlsxTable =
                xml_from_reader(namespace_strict_to_transitional(&content).as_slice())?;
            let mut coordinates = range_ref_to_coordinates(&t.r#ref)?;

            if dir == AdjustDirection::Rows && num == coordinates[1] && offset == -1 {
                ws.table_parts.as_mut().unwrap().table_part.remove(idx);
                continue;
            }

            coordinates = self.adjust_auto_filter_helper(dir, coordinates, num, offset);
            let (x1, y1, x2, y2) = (
                coordinates[0],
                coordinates[1],
                coordinates[2],
                coordinates[3],
            );
            if y2 - y1 < 1 || x2 - x1 < 0 {
                ws.table_parts.as_mut().unwrap().table_part.remove(idx);
                continue;
            }

            t.r#ref = coordinates_to_range_ref(&coordinates, false)?;
            if let Some(af) = &mut t.auto_filter {
                af.r#ref = t.r#ref.clone();
            }
            self.set_table_columns(sheet, true, x1, y1, x2, &mut t)?;
            t.table_type = None;
            t.totals_row_count = 0;
            t.connection_id = 0;

            let table = xml_to_string(&t)?.into_bytes();
            self.save_file_list(&table_xml, &table);
            idx += 1;
        }

        if let Some(parts) = ws.table_parts.as_mut() {
            parts.count = Some(parts.table_part.len() as i64);
            if parts.table_part.is_empty() {
                ws.table_parts = None;
            }
        }
        Ok(())
    }

    // ------------------------------------------------------------------
    // Auto filter
    // ------------------------------------------------------------------

    fn adjust_auto_filter(
        &self,
        ws: &mut XlsxWorksheet,
        _sheet: &str,
        dir: AdjustDirection,
        num: i32,
        offset: i32,
        _sheet_id: i32,
    ) -> Result<()> {
        if ws.auto_filter.is_none() {
            return Ok(());
        }
        let af = ws.auto_filter.as_mut().unwrap();
        let mut coordinates = range_ref_to_coordinates(&af.r#ref)?;
        let (x1, y1, x2, y2) = (
            coordinates[0],
            coordinates[1],
            coordinates[2],
            coordinates[3],
        );

        if (dir == AdjustDirection::Rows && y1 == num && offset < 0)
            || (dir == AdjustDirection::Columns && x1 == num && x2 == num)
        {
            ws.auto_filter = None;
            for row in &mut ws.sheet_data.row {
                if let Some(r) = row.r {
                    if r > y1 as i64 && r <= y2 as i64 {
                        row.hidden = Some(false);
                    }
                }
            }
            return Ok(());
        }

        coordinates = self.adjust_auto_filter_helper(dir, coordinates, num, offset);
        af.r#ref = coordinates_to_range_ref(&coordinates, false)?;
        Ok(())
    }

    fn adjust_auto_filter_helper(
        &self,
        dir: AdjustDirection,
        mut coordinates: Vec<i32>,
        num: i32,
        offset: i32,
    ) -> Vec<i32> {
        if dir == AdjustDirection::Rows {
            if coordinates[1] >= num {
                coordinates[1] += offset;
            }
            if coordinates[3] >= num {
                coordinates[3] += offset;
            }
        } else {
            if coordinates[0] >= num {
                coordinates[0] += offset;
            }
            if coordinates[2] >= num {
                coordinates[2] += offset;
            }
        }
        coordinates
    }

    // ------------------------------------------------------------------
    // Merge cells
    // ------------------------------------------------------------------

    fn adjust_merge_cells(
        &self,
        ws: &mut XlsxWorksheet,
        _sheet: &str,
        dir: AdjustDirection,
        num: i32,
        offset: i32,
        _sheet_id: i32,
    ) -> Result<()> {
        if ws.merge_cells.is_none() {
            return Ok(());
        }

        let mut i = 0;
        while i < ws.merge_cells.as_ref().unwrap().cells.len() {
            let merged_cells_ref = ws.merge_cells.as_ref().unwrap().cells[i]
                .r#ref
                .clone()
                .unwrap_or_default();
            let mut merged_ref = merged_cells_ref.clone();
            if !merged_ref.contains(':') {
                merged_ref.push(':');
                merged_ref.push_str(&merged_cells_ref);
            }
            let mut coordinates = range_ref_to_coordinates(&merged_ref)?;
            let (mut x1, mut y1, mut x2, mut y2) = (
                coordinates[0],
                coordinates[1],
                coordinates[2],
                coordinates[3],
            );

            if dir == AdjustDirection::Rows {
                if y1 == num && y2 == num && offset < 0 {
                    delete_merge_cell(ws, i);
                    continue;
                }
                (y1, y2) = adjust_merge_cells_helper(y1, y2, num, offset);
            } else {
                if x1 == num && x2 == num && offset < 0 {
                    delete_merge_cell(ws, i);
                    continue;
                }
                (x1, x2) = adjust_merge_cells_helper(x1, x2, num, offset);
            }

            if x1 == x2 && y1 == y2 {
                delete_merge_cell(ws, i);
                continue;
            }

            coordinates = vec![x1, y1, x2, y2];
            if let Some(cell) = ws.merge_cells.as_mut().unwrap().cells.get_mut(i) {
                cell.r#ref = Some(coordinates_to_range_ref(&coordinates, false)?);
            }
            i += 1;
        }
        Ok(())
    }

    // ------------------------------------------------------------------
    // Calculation chain
    // ------------------------------------------------------------------

    fn adjust_calc_chain(
        &self,
        _ws: &mut XlsxWorksheet,
        _sheet: &str,
        dir: AdjustDirection,
        num: i32,
        offset: i32,
        sheet_id: i32,
    ) -> Result<()> {
        let mut guard = self.calc_chain.lock().unwrap();
        let cc = match guard.as_mut() {
            Some(c) => c,
            None => return Ok(()),
        };

        let mut prev_sheet_id = 0;
        let mut i = 0;
        while i < cc.c.len() {
            let mut ci = cc.c[i].clone();
            if ci.i == 0 {
                ci.i = prev_sheet_id;
            }
            prev_sheet_id = ci.i;
            if ci.i != sheet_id {
                i += 1;
                continue;
            }

            let (col_num, row_num) = cell_name_to_coordinates(&ci.r)?;
            let mut updated = false;
            if dir == AdjustDirection::Rows && num <= row_num {
                if num == row_num && offset == -1 {
                    cc.c.remove(i);
                    continue;
                }
                cc.c[i].r = adjust_cell_name(&ci.r, dir, col_num, row_num, offset)?;
                updated = true;
            }
            if !updated && dir == AdjustDirection::Columns && num <= col_num {
                if num == col_num && offset == -1 {
                    cc.c.remove(i);
                    continue;
                }
                cc.c[i].r = adjust_cell_name(&ci.r, dir, col_num, row_num, offset)?;
            }
            i += 1;
        }
        Ok(())
    }

    // ------------------------------------------------------------------
    // Conditional formatting
    // ------------------------------------------------------------------

    fn adjust_conditional_formats(
        &self,
        ws: &mut XlsxWorksheet,
        _sheet: &str,
        dir: AdjustDirection,
        num: i32,
        offset: i32,
        _sheet_id: i32,
    ) -> Result<()> {
        let mut i = 0;
        while i < ws.conditional_formatting.len() {
            let sqref = match &ws.conditional_formatting[i].sqref {
                Some(s) => s.clone(),
                None => {
                    i += 1;
                    continue;
                }
            };
            let ref_ = self.adjust_cell_ref(&sqref, dir, num, offset)?;
            if ref_.is_empty() {
                ws.conditional_formatting.remove(i);
                continue;
            }
            ws.conditional_formatting[i].sqref = Some(ref_);
            i += 1;
        }
        Ok(())
    }

    // ------------------------------------------------------------------
    // Data validations
    // ------------------------------------------------------------------

    fn adjust_data_validations(
        &self,
        _ws: &mut XlsxWorksheet,
        sheet: &str,
        dir: AdjustDirection,
        num: i32,
        offset: i32,
        _sheet_id: i32,
    ) -> Result<()> {
        for sheet_n in self.get_sheet_list() {
            let mut worksheet = match self.work_sheet_reader(&sheet_n) {
                Ok(w) => w,
                Err(e) => {
                    if e.to_string() == new_not_worksheet_error(&sheet_n) {
                        continue;
                    }
                    return Err(e);
                }
            };
            let path = match self.get_sheet_xml_path(&sheet_n) {
                Some(p) => p,
                None => continue,
            };

            if worksheet.data_validations.is_none() {
                continue;
            }

            let dvs = worksheet.data_validations.as_mut().unwrap();
            let mut i = 0;
            while i < dvs.data_validation.len() {
                if sheet.eq_ignore_ascii_case(&sheet_n) {
                    let ref_ =
                        self.adjust_cell_ref(&dvs.data_validation[i].sqref, dir, num, offset)?;
                    if ref_.is_empty() {
                        dvs.data_validation.remove(i);
                        continue;
                    }
                    dvs.data_validation[i].sqref = ref_;
                }

                if let Some(f1) = dvs.data_validation[i].formula1.as_mut() {
                    if inner_xml_is_formula(&f1.content) {
                        let formula = formula_unescaper_replace(&f1.content);
                        let adjusted = self.adjust_formula_ref(
                            sheet, &sheet_n, &formula, false, dir, num, offset,
                        )?;
                        f1.content = formula_escaper_replace(&adjusted);
                    }
                }
                if let Some(f2) = dvs.data_validation[i].formula2.as_mut() {
                    if inner_xml_is_formula(&f2.content) {
                        let formula = formula_unescaper_replace(&f2.content);
                        let adjusted = self.adjust_formula_ref(
                            sheet, &sheet_n, &formula, false, dir, num, offset,
                        )?;
                        f2.content = formula_escaper_replace(&adjusted);
                    }
                }
                i += 1;
            }
            dvs.count = Some(dvs.data_validation.len() as i64);
            if dvs.data_validation.is_empty() {
                worksheet.data_validations = None;
            }
            self.sheet.insert(path, worksheet);
        }
        Ok(())
    }

    // ------------------------------------------------------------------
    // Drawings
    // ------------------------------------------------------------------

    fn adjust_drawings(
        &self,
        ws: &mut XlsxWorksheet,
        sheet: &str,
        dir: AdjustDirection,
        num: i32,
        offset: i32,
    ) -> Result<()> {
        let drawing_rid = match ws.drawing.as_ref().and_then(|d| d.rid.clone()) {
            Some(rid) => rid,
            None => return Ok(()),
        };
        let target = self.get_sheet_relationships_target_by_id(sheet, &drawing_rid);
        if target.is_empty() {
            return Ok(());
        }
        let drawing_xml = target
            .replace("..", "xl")
            .trim_start_matches('/')
            .to_string();
        let (mut ws_dr, _) = self.drawing_parser(&drawing_xml)?;

        for anchor in ws_dr.two_cell_anchor.iter_mut() {
            adjust_cell_anchor(anchor, dir, num, offset)?;
        }
        for anchor in ws_dr.one_cell_anchor.iter_mut() {
            adjust_cell_anchor(anchor, dir, num, offset)?;
        }

        self.drawings.insert(drawing_xml, ws_dr);
        Ok(())
    }

    // ------------------------------------------------------------------
    // Defined names
    // ------------------------------------------------------------------

    fn adjust_defined_names(
        &self,
        sheet: &str,
        dir: AdjustDirection,
        num: i32,
        offset: i32,
    ) -> Result<()> {
        let mut wb = self.workbook_reader()?;
        if let Some(dns) = wb.defined_names.as_mut() {
            for i in 0..dns.defined_name.len() {
                let data = dns.defined_name[i].data.clone();
                if let Ok(adjusted) =
                    self.adjust_formula_ref(sheet, "", &data, true, dir, num, offset)
                {
                    dns.defined_name[i].data = adjusted;
                }
            }
        }
        *self.workbook.lock().unwrap() = Some(wb);
        Ok(())
    }

    // ------------------------------------------------------------------
    // Volatile dependencies
    // ------------------------------------------------------------------

    fn adjust_volatile_deps(
        &self,
        _ws: &mut XlsxWorksheet,
        _sheet: &str,
        dir: AdjustDirection,
        num: i32,
        offset: i32,
        sheet_id: i32,
    ) -> Result<()> {
        let mut vol_types = match self.volatile_deps_reader()? {
            Some(vt) => vt,
            None => return Ok(()),
        };

        let mut i1 = 0;
        while i1 < vol_types.vol_type.len() {
            let mut i2 = 0;
            while i2 < vol_types.vol_type[i1].main.len() {
                let mut i3 = 0;
                while i3 < vol_types.vol_type[i1].main[i2].tp.len() {
                    let mut i4 = 0;
                    while i4 < vol_types.vol_type[i1].main[i2].tp[i3].tr.len() {
                        let tr = &vol_types.vol_type[i1].main[i2].tp[i3].tr[i4];
                        if tr.s != sheet_id {
                            i4 += 1;
                            continue;
                        }
                        let (col, row) = cell_name_to_coordinates(&tr.r)?;
                        let should_delete = match dir {
                            AdjustDirection::Rows => num <= row && num == row && offset == -1,
                            AdjustDirection::Columns => num <= col && num == col && offset == -1,
                        };
                        if should_delete {
                            crate::calc_chain::delete_vol_topic_ref(&mut vol_types, i1, i2, i3, i4);
                            continue;
                        }
                        let adjusted = match dir {
                            AdjustDirection::Rows if num <= row => {
                                adjust_cell_name(&tr.r, dir, col, row, offset)?
                            }
                            AdjustDirection::Columns if num <= col => {
                                adjust_cell_name(&tr.r, dir, col, row, offset)?
                            }
                            _ => tr.r.clone(),
                        };
                        vol_types.vol_type[i1].main[i2].tp[i3].tr[i4].r = adjusted;
                        i4 += 1;
                    }
                    i3 += 1;
                }
                i2 += 1;
            }
            i1 += 1;
        }

        *self.volatile_deps.lock().unwrap() = Some(vol_types);
        Ok(())
    }

    // ------------------------------------------------------------------
    // Internal helpers used by the adjusters
    // ------------------------------------------------------------------

    /// Ensure every row element has an `r` attribute and then fill each row so
    /// that column references are continuous.
    ///
    /// Equivalent to Go `xlsxWorksheet.checkSheet` simplified to the core
    /// "fill row numbers + check rows" behaviour.
    pub(crate) fn check_sheet(&self, ws: &mut XlsxWorksheet) -> Result<()> {
        for (row_idx, row) in ws.sheet_data.row.iter_mut().enumerate() {
            if row.r.is_none() {
                row.r = Some(row_idx as i64 + 1);
            }
        }
        self.check_row(ws)
    }

    /// Check and fill each column element for all rows, making cell references
    /// continuous within a worksheet.
    ///
    /// Equivalent to Go `xlsxWorksheet.checkRow`.
    pub(crate) fn check_row(&self, ws: &mut XlsxWorksheet) -> Result<()> {
        for (row_idx, row) in ws.sheet_data.row.iter_mut().enumerate() {
            let row_num = row.r.unwrap_or((row_idx as i64) + 1);
            let col_count = row.c.len();
            if col_count == 0 {
                continue;
            }

            // Fill missing `r` attributes in a row element.
            let mut r_count = 0;
            for cell in &mut row.c {
                r_count += 1;
                if let Some(ref name) = cell.r {
                    if let Ok((col, _)) = cell_name_to_coordinates(name) {
                        if col > r_count {
                            r_count = col;
                        }
                    }
                    continue;
                }
                cell.r = coordinates_to_cell_name(r_count, row_num as i32, false).ok();
            }

            let last_col = if let Some(ref name) = row.c.last().and_then(|c| c.r.as_ref()) {
                cell_name_to_coordinates(name)?.0
            } else {
                continue;
            };

            if col_count < last_col as usize {
                let mut target = Vec::with_capacity(last_col as usize);
                for col_idx in 0..last_col {
                    let cell_name = coordinates_to_cell_name(col_idx + 1, row_num as i32, false)?;
                    target.push(XlsxC {
                        r: Some(cell_name),
                        ..Default::default()
                    });
                }
                for cell in row.c.drain(..) {
                    if let Some(ref name) = cell.r {
                        if let Ok((col, _)) = cell_name_to_coordinates(name) {
                            target[(col - 1) as usize] = cell;
                        }
                    }
                }
                row.c = target;
            }
        }
        Ok(())
    }
}

// ------------------------------------------------------------------
// Free helpers
// ------------------------------------------------------------------

fn adjust_single_row_dimensions(row: &mut XlsxRow, offset: i32) {
    if let Some(r) = row.r.as_mut() {
        *r += offset as i64;
    }
    let new_row = row.r.unwrap_or(0);
    for cell in &mut row.c {
        if let Some(name) = &cell.r {
            if let Ok((col, _)) = split_cell_name(name) {
                if let Ok(new_name) = join_cell_name(&col, new_row as i32) {
                    cell.r = Some(new_name);
                }
            }
        }
    }
}

fn adjust_expr(
    expr: &mut crate::calc::Expr,
    dir: AdjustDirection,
    num: i32,
    offset: i32,
    keep_relative: bool,
    sheet: &str,
    active_sheet: &str,
) {
    use crate::calc::Expr;
    match expr {
        Expr::Cell(r) => {
            if should_adjust_ref(r, sheet, active_sheet) {
                adjust_cell_reference(r, dir, num, offset, keep_relative);
            }
        }
        Expr::Range(start, end) => {
            if should_adjust_ref(start, sheet, active_sheet) {
                adjust_cell_reference(start, dir, num, offset, keep_relative);
            }
            if should_adjust_ref(end, sheet, active_sheet) {
                adjust_cell_reference(end, dir, num, offset, keep_relative);
            }
        }
        Expr::Call(_, args) => {
            for arg in args {
                adjust_expr(arg, dir, num, offset, keep_relative, sheet, active_sheet);
            }
        }
        Expr::Unary(_, e) => {
            adjust_expr(e, dir, num, offset, keep_relative, sheet, active_sheet);
        }
        Expr::Binary(_, l, r) => {
            adjust_expr(l, dir, num, offset, keep_relative, sheet, active_sheet);
            adjust_expr(r, dir, num, offset, keep_relative, sheet, active_sheet);
        }
        _ => {}
    }
}

fn should_adjust_ref(r: &crate::calc::CellRef, sheet: &str, active_sheet: &str) -> bool {
    match &r.sheet {
        Some(s) => s.eq_ignore_ascii_case(sheet),
        None => active_sheet.eq_ignore_ascii_case(sheet),
    }
}

fn adjust_cell_reference(
    r: &mut crate::calc::CellRef,
    dir: AdjustDirection,
    num: i32,
    offset: i32,
    keep_relative: bool,
) {
    match dir {
        AdjustDirection::Columns => {
            if (!keep_relative || r.col_abs) && r.col >= num {
                r.col += offset;
                if r.col < 1 {
                    r.col = 1;
                }
                if r.col > MAX_COLUMNS {
                    r.col = MAX_COLUMNS;
                }
            }
        }
        AdjustDirection::Rows => {
            if (!keep_relative || r.row_abs) && r.row >= num {
                r.row += offset;
                if r.row < 1 {
                    r.row = 1;
                }
                if r.row > TOTAL_ROWS {
                    r.row = TOTAL_ROWS;
                }
            }
        }
    }
}

fn format_expr(expr: &crate::calc::Expr) -> Result<String> {
    use crate::calc::Expr;
    Ok(match expr {
        Expr::Number(n) => {
            if n.fract() == 0.0 {
                format!("{}", *n as i64)
            } else {
                format!("{}", n)
            }
        }
        Expr::String(s) => format!("\"{}\"", s.replace('"', "\"\"")),
        Expr::Bool(true) => "TRUE".to_string(),
        Expr::Bool(false) => "FALSE".to_string(),
        Expr::Cell(r) => format_cell_ref(r),
        Expr::Range(start, end) => {
            if start.sheet.is_some() && start.sheet == end.sheet {
                let sheet = start.sheet.as_deref().unwrap();
                format!(
                    "{}!{}:{}",
                    escape_sheet_name(sheet),
                    format_cell_ref_without_sheet(start),
                    format_cell_ref_without_sheet(end)
                )
            } else {
                format!("{}:{}", format_cell_ref(start), format_cell_ref(end))
            }
        }
        Expr::Call(name, args) => {
            let args_str = args
                .iter()
                .map(format_expr)
                .collect::<Result<Vec<_>>>()?
                .join(",");
            format!("{}({})", name, args_str)
        }
        Expr::Unary(op, e) => format!("{}{}", op, format_expr(e)?),
        Expr::Binary(op, l, r) => {
            format!("{}{}{}", format_expr(l)?, op, format_expr(r)?)
        }
        Expr::Range3D(s1, s2, start, end) => {
            format!(
                "{}:{}!{}:{}",
                escape_sheet_name(s1),
                escape_sheet_name(s2),
                format_cell_ref_without_sheet(start),
                format_cell_ref_without_sheet(end)
            )
        }
        Expr::Name(name) => name.clone(),
        Expr::Array(rows) => {
            let rows_str = rows
                .iter()
                .map(|row| {
                    row.iter()
                        .map(format_expr)
                        .collect::<Result<Vec<_>>>()
                        .map(|v| v.join(","))
                })
                .collect::<Result<Vec<_>>>()?
                .join(";");
            format!("{{{}}}", rows_str)
        }
    })
}

fn format_cell_ref(r: &crate::calc::CellRef) -> String {
    let sheet_prefix = r
        .sheet
        .as_deref()
        .map(|s| format!("{}!", escape_sheet_name(s)))
        .unwrap_or_default();
    format!("{}{}", sheet_prefix, format_cell_ref_without_sheet(r))
}

fn format_cell_ref_without_sheet(r: &crate::calc::CellRef) -> String {
    let col_name = column_number_to_name(r.col).unwrap_or_default();
    let col_part = if r.col_abs {
        format!("${}", col_name)
    } else {
        col_name
    };
    let row_part = if r.row_abs {
        format!("${}", r.row)
    } else {
        r.row.to_string()
    };
    format!("{}{}", col_part, row_part)
}

fn apply_offset(
    coordinates: &mut [i32],
    idx1: usize,
    idx2: usize,
    max_val: i32,
    num: i32,
    offset: i32,
) {
    if coordinates[idx1] >= num {
        coordinates[idx1] += offset;
    }
    if coordinates[idx2] >= num {
        coordinates[idx2] += offset;
        if coordinates[idx2] > max_val {
            coordinates[idx2] = max_val;
        }
    }
}

fn adjust_merge_cells_helper(mut p1: i32, mut p2: i32, num: i32, offset: i32) -> (i32, i32) {
    if p2 < p1 {
        std::mem::swap(&mut p1, &mut p2);
    }
    if offset >= 0 {
        if num <= p1 {
            p1 += offset;
            p2 += offset;
        } else if num <= p2 {
            p2 += offset;
        }
    } else {
        if num < p1 || (num == p1 && num == p2) {
            p1 += offset;
            p2 += offset;
        } else if num <= p2 {
            p2 += offset;
        }
    }
    (p1, p2)
}

fn delete_merge_cell(ws: &mut XlsxWorksheet, idx: usize) {
    if ws.merge_cells.is_none() {
        return;
    }
    let merges = ws.merge_cells.as_mut().unwrap();
    if idx >= merges.cells.len() {
        return;
    }
    merges.cells.remove(idx);
    merges.count = Some(merges.cells.len() as i64);
}

fn adjust_cell_name(
    _cell: &str,
    dir: AdjustDirection,
    c: i32,
    r: i32,
    offset: i32,
) -> Result<String> {
    if dir == AdjustDirection::Rows {
        let rn = r + offset;
        if rn > 0 {
            return coordinates_to_cell_name(c, rn, false).map_err(|e| e.into());
        }
    }
    coordinates_to_cell_name(c + offset, r, false).map_err(|e| e.into())
}

fn inner_xml_is_formula(content: &str) -> bool {
    let trimmed = content.trim_start();
    !trimmed.is_empty() && (trimmed.starts_with('=') || trimmed.contains("<formula"))
}

fn formula_unescaper_replace(s: &str) -> String {
    s.to_string()
}

fn formula_escaper_replace(s: &str) -> String {
    s.to_string()
}

fn escape_sheet_name(name: &str) -> String {
    if name.chars().any(|r| !r.is_alphanumeric()) {
        format!("'{}'", name.replace('\'', "''"))
    } else {
        name.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::options::Options;

    #[test]
    fn adjust_helper_shifts_on_row_insert() {
        let f = File::new_with_options(Options::default());
        f.merge_cell("Sheet1", "A1", "B2").unwrap();
        f.set_cell_value("Sheet1", "C5", "value").unwrap();
        f.adjust_helper("Sheet1", AdjustDirection::Rows, 2, 1)
            .unwrap();

        let merges = f.get_merge_cells("Sheet1").unwrap();
        assert!(merges.contains(&"A1:B3".to_string()));
        assert!(f.get_cell_value("Sheet1", "C6").unwrap().contains("value"));
    }

    fn get_cell_formula(f: &File, sheet: &str, cell: &str) -> String {
        let ws = f.work_sheet_reader(sheet).unwrap();
        for row in &ws.sheet_data.row {
            for c in &row.c {
                if c.r
                    .as_deref()
                    .map(|r| r.eq_ignore_ascii_case(cell))
                    .unwrap_or(false)
                {
                    return c.f.as_ref().map(|f| f.content.clone()).unwrap_or_default();
                }
            }
        }
        String::new()
    }

    #[test]
    fn adjust_formula_ref_shifts_relative_row_refs() {
        let f = File::new_with_options(Options::default());
        assert_eq!(
            f.adjust_formula_ref("Sheet1", "Sheet1", "A1", false, AdjustDirection::Rows, 1, 1)
                .unwrap(),
            "A2"
        );
    }

    #[test]
    fn adjust_formula_ref_keeps_refs_below_row_insert() {
        let f = File::new_with_options(Options::default());
        assert_eq!(
            f.adjust_formula_ref("Sheet1", "Sheet1", "A1", false, AdjustDirection::Rows, 2, 1)
                .unwrap(),
            "A1"
        );
    }

    #[test]
    fn adjust_formula_ref_shifts_absolute_row_refs() {
        let f = File::new_with_options(Options::default());
        assert_eq!(
            f.adjust_formula_ref(
                "Sheet1",
                "Sheet1",
                "$A$1",
                false,
                AdjustDirection::Rows,
                1,
                1
            )
            .unwrap(),
            "$A$2"
        );
    }

    #[test]
    fn adjust_formula_ref_shifts_relative_col_refs() {
        let f = File::new_with_options(Options::default());
        assert_eq!(
            f.adjust_formula_ref(
                "Sheet1",
                "Sheet1",
                "A1",
                false,
                AdjustDirection::Columns,
                1,
                1
            )
            .unwrap(),
            "B1"
        );
    }

    #[test]
    fn adjust_formula_ref_keeps_other_sheet_refs() {
        let f = File::new_with_options(Options::default());
        assert_eq!(
            f.adjust_formula_ref(
                "Sheet1",
                "Sheet1",
                "Sheet2!A1",
                false,
                AdjustDirection::Rows,
                1,
                1
            )
            .unwrap(),
            "Sheet2!A1"
        );
    }

    #[test]
    fn adjust_formula_ref_adjusts_same_sheet_qualified_refs() {
        let f = File::new_with_options(Options::default());
        assert_eq!(
            f.adjust_formula_ref(
                "Sheet1",
                "Sheet1",
                "Sheet1!A1",
                false,
                AdjustDirection::Rows,
                1,
                1
            )
            .unwrap(),
            "Sheet1!A2"
        );
    }

    #[test]
    fn adjust_formula_ref_adjusts_ranges() {
        let f = File::new_with_options(Options::default());
        assert_eq!(
            f.adjust_formula_ref(
                "Sheet1",
                "Sheet1",
                "SUM(A1:B3)",
                false,
                AdjustDirection::Rows,
                2,
                1
            )
            .unwrap(),
            "SUM(A1:B4)"
        );
    }

    #[test]
    fn adjust_formula_ref_keep_relative_only_adjusts_absolute() {
        let f = File::new_with_options(Options::default());
        assert_eq!(
            f.adjust_formula_ref(
                "Sheet1",
                "Sheet1",
                "$A$1+A1",
                true,
                AdjustDirection::Rows,
                1,
                1
            )
            .unwrap(),
            "$A$2+A1"
        );
    }

    #[test]
    fn adjust_formula_ref_preserves_defined_names() {
        let f = File::new_with_options(Options::default());
        assert_eq!(
            f.adjust_formula_ref(
                "Sheet1",
                "Sheet1",
                "=MYRANGE+A1",
                false,
                AdjustDirection::Rows,
                1,
                1
            )
            .unwrap(),
            "=MYRANGE+A2"
        );
    }

    #[test]
    fn adjust_helper_updates_cell_formula_on_row_insert() {
        let f = File::new_with_options(Options::default());
        f.set_cell_formula("Sheet1", "C5", "A5+B5").unwrap();
        f.adjust_helper("Sheet1", AdjustDirection::Rows, 3, 1)
            .unwrap();
        assert_eq!(get_cell_formula(&f, "Sheet1", "C6"), "A6+B6");
    }

    #[test]
    fn adjust_helper_updates_cell_formula_on_col_insert() {
        let f = File::new_with_options(Options::default());
        f.set_cell_formula("Sheet1", "D1", "A1+B1").unwrap();
        f.adjust_helper("Sheet1", AdjustDirection::Columns, 2, 1)
            .unwrap();
        assert_eq!(get_cell_formula(&f, "Sheet1", "E1"), "A1+C1");
    }
}

fn adjust_cell_anchor(
    anchor: &mut XdrCellAnchor,
    dir: AdjustDirection,
    num: i32,
    offset: i32,
) -> Result<()> {
    let edit_as = anchor.edit_as.as_deref().unwrap_or("");
    if (anchor.from.is_none() && (anchor.to.is_none() || anchor.ext.is_none()))
        || edit_as == "absolute"
    {
        return Ok(());
    }
    let ok = adjust_from(anchor.from.as_mut().unwrap(), dir, num, offset, edit_as)?;
    if let Some(to) = anchor.to.as_mut() {
        adjust_to(to, dir, num, offset, ok || edit_as.is_empty())?;
    }
    Ok(())
}

fn adjust_from(
    from: &mut XlsxFrom,
    dir: AdjustDirection,
    num: i32,
    offset: i32,
    edit_as: &str,
) -> Result<bool> {
    let mut ok = false;
    if dir == AdjustDirection::Columns
        && from.col + 1 >= num as i64
        && from.col + offset as i64 >= 0
    {
        if from.col + offset as i64 >= MAX_COLUMNS as i64 {
            return Err(Box::new(ErrColumnNumber));
        }
        from.col += offset as i64;
        ok = edit_as == "oneCell";
    }
    if dir == AdjustDirection::Rows && from.row + 1 >= num as i64 && from.row + offset as i64 >= 0 {
        if from.row + offset as i64 >= TOTAL_ROWS as i64 {
            return Err(Box::new(ErrMaxRows));
        }
        from.row += offset as i64;
        ok = edit_as == "oneCell";
    }
    Ok(ok)
}

fn adjust_to(to: &mut XlsxTo, dir: AdjustDirection, num: i32, offset: i32, ok: bool) -> Result<()> {
    if !ok {
        return Ok(());
    }
    if dir == AdjustDirection::Columns && to.col + 1 >= num as i64 && to.col + offset as i64 >= 0 {
        if to.col + offset as i64 >= MAX_COLUMNS as i64 {
            return Err(Box::new(ErrColumnNumber));
        }
        to.col += offset as i64;
    }
    if dir == AdjustDirection::Rows && to.row + 1 >= num as i64 && to.row + offset as i64 >= 0 {
        if to.row + offset as i64 >= TOTAL_ROWS as i64 {
            return Err(Box::new(ErrMaxRows));
        }
        to.row += offset as i64;
    }
    Ok(())
}
