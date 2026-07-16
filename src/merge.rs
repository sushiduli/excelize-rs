//! Merge-cell API.
//!
//! This module corresponds to `merge.go` in the Go implementation.

use crate::cell::find_cell_mut;
use crate::errors::Result;
use crate::errors::{ErrParameterInvalid, ErrSheetNotExist};
use crate::file::File;
use crate::lib_util::{
    cell_name_to_coordinates, coordinates_to_cell_name, coordinates_to_range_ref, sort_coordinates,
};

impl File {
    /// Merge a range of cells.
    pub fn merge_cell(&self, sheet: &str, top_left: &str, bottom_right: &str) -> Result<()> {
        let range = normalize_range(top_left, bottom_right)?;
        let path = self
            .get_sheet_xml_path(sheet)
            .ok_or_else(|| ErrSheetNotExist {
                sheet_name: sheet.to_string(),
            })?;
        let mut ws = self.work_sheet_reader(sheet)?;
        ensure_merge_cells(&mut ws);
        let merges = ws.merge_cells.as_mut().unwrap();
        // Remove any overlapping existing merges.
        merges.cells.retain(|m| {
            if let Some(existing) = &m.r#ref {
                !ranges_overlap(existing, &range)
            } else {
                true
            }
        });
        merges.cells.push(crate::xml::worksheet::XlsxMergeCell {
            r#ref: Some(range.clone()),
        });
        merges.count = Some(merges.cells.len() as i64);
        self.sheet.insert(path, ws);
        Ok(())
    }

    /// Unmerge a range of cells.
    pub fn unmerge_cell(&self, sheet: &str, top_left: &str, bottom_right: &str) -> Result<()> {
        let range = normalize_range(top_left, bottom_right)?;
        let path = self
            .get_sheet_xml_path(sheet)
            .ok_or_else(|| ErrSheetNotExist {
                sheet_name: sheet.to_string(),
            })?;
        let mut ws = self.work_sheet_reader(sheet)?;
        if let Some(merges) = ws.merge_cells.as_mut() {
            merges.cells.retain(|m| m.r#ref.as_deref() != Some(&range));
            merges.count = Some(merges.cells.len() as i64);
            if merges.cells.is_empty() {
                ws.merge_cells = None;
            }
        }
        self.sheet.insert(path, ws);
        Ok(())
    }

    /// Get all merged cell ranges in a worksheet.
    pub fn get_merge_cells(&self, sheet: &str) -> Result<Vec<String>> {
        let ws = self.work_sheet_reader(sheet)?;
        let mut out = Vec::new();
        if let Some(merges) = ws.merge_cells {
            for m in merges.cells {
                if let Some(r) = m.r#ref {
                    out.push(r);
                }
            }
        }
        Ok(out)
    }
}

/// Add a merged-cell range directly to a worksheet in memory.
///
/// Equivalent to [`File::merge_cell`] but without writing the worksheet back to
/// the file cache, so callers that already hold a mutable worksheet can use it.
pub(crate) fn add_merge(
    ws: &mut crate::xml::worksheet::XlsxWorksheet,
    top_left: &str,
    bottom_right: &str,
) -> Result<()> {
    let range = normalize_range(top_left, bottom_right)?;
    ensure_merge_cells(ws);
    let merges = ws.merge_cells.as_mut().unwrap();
    merges.cells.retain(|m| {
        if let Some(existing) = &m.r#ref {
            !ranges_overlap(existing, &range)
        } else {
            true
        }
    });
    merges.cells.push(crate::xml::worksheet::XlsxMergeCell {
        r#ref: Some(range.clone()),
    });
    merges.count = Some(merges.cells.len() as i64);

    // Clear values/formulas from every cell in the range except the top-left.
    let parts: Vec<&str> = range.split(':').collect();
    let (tl, br) = (parts[0], parts[1]);
    let mut coords = cell_refs_to_coordinates(tl, br)?;
    sort_coordinates(&mut coords)?;
    let (min_col, min_row, max_col, max_row) = (coords[0], coords[1], coords[2], coords[3]);
    for col in min_col..=max_col {
        for row in min_row..=max_row {
            if col == min_col && row == min_row {
                continue;
            }
            let cell_ref = coordinates_to_cell_name(col, row, false)?;
            if let Some(c) = find_cell_mut(ws, &cell_ref) {
                c.t = None;
                c.v = None;
                c.f = None;
                c.is = None;
            }
        }
    }
    Ok(())
}

// ------------------------------------------------------------------
// Internal helpers
// ------------------------------------------------------------------

fn ensure_merge_cells(ws: &mut crate::xml::worksheet::XlsxWorksheet) {
    if ws.merge_cells.is_none() {
        ws.merge_cells = Some(crate::xml::worksheet::XlsxMergeCells::default());
    }
}

fn normalize_range(top_left: &str, bottom_right: &str) -> Result<String> {
    let mut coords = cell_refs_to_coordinates(top_left, bottom_right)?;
    sort_coordinates(&mut coords)?;
    Ok(coordinates_to_range_ref(&coords, false)?)
}

fn cell_refs_to_coordinates(first: &str, last: &str) -> Result<Vec<i32>> {
    let mut coords = vec![0; 4];
    let (col, row) = cell_name_to_coordinates(first)?;
    coords[0] = col;
    coords[1] = row;
    let (col, row) = cell_name_to_coordinates(last)?;
    coords[2] = col;
    coords[3] = row;
    Ok(coords)
}

fn ranges_overlap(a: &str, b: &str) -> bool {
    let Ok(ca) = normalize_single_range(a) else {
        return true;
    };
    let Ok(cb) = normalize_single_range(b) else {
        return true;
    };
    if ca[2] < cb[0] || ca[0] > cb[2] || ca[3] < cb[1] || ca[1] > cb[3] {
        return false;
    }
    true
}

fn normalize_single_range(r: &str) -> Result<Vec<i32>> {
    let parts: Vec<&str> = r.split(':').collect();
    if parts.len() != 2 {
        return Err(ErrParameterInvalid.to_string().into());
    }
    let mut coords = cell_refs_to_coordinates(parts[0], parts[1])?;
    sort_coordinates(&mut coords)?;
    Ok(coords)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_and_unmerge() {
        let f = File::new_with_options(crate::options::Options::default());
        f.merge_cell("Sheet1", "A1", "B2").unwrap();
        assert_eq!(f.get_merge_cells("Sheet1").unwrap(), vec!["A1:B2"]);
        f.merge_cell("Sheet1", "C1", "C3").unwrap();
        assert!(
            f.get_merge_cells("Sheet1")
                .unwrap()
                .contains(&"C1:C3".to_string())
        );
        f.unmerge_cell("Sheet1", "A1", "B2").unwrap();
        assert!(
            !f.get_merge_cells("Sheet1")
                .unwrap()
                .contains(&"A1:B2".to_string())
        );
    }
}
