//! PivotTable support.
//!
//! Ported from Go `pivotTable.go`.

use std::collections::HashMap;

use quick_xml::de::from_reader as xml_from_reader;
use quick_xml::se::to_string as xml_to_string;
use regex::Regex;

use crate::constants::{
    EXT_URI_PIVOT_DATA_FIELD, MAX_FIELD_LENGTH, NAMESPACE_SPREADSHEET, NAMESPACE_SPREADSHEET_X14,
    PIVOT_TABLE_REFRESHED_VERSION, PIVOT_TABLE_VERSION, SOURCE_RELATIONSHIP_PIVOT_CACHE,
    SOURCE_RELATIONSHIP_PIVOT_TABLE,
};
use crate::errors::{
    ErrNameLength, ErrParameterInvalid, ErrParameterRequired, ErrPivotTableClassicLayout,
    ErrPivotTableShowValuesAsBaseField, ErrPivotTableShowValuesAsBaseItem, ErrSheetNotExist,
    ErrUnsupportedPivotTableShowValuesAsType, Result, new_no_exist_table_error,
    new_pivot_table_col_fields_error, new_pivot_table_data_range_error,
    new_pivot_table_range_error, new_pivot_table_row_fields_error,
    new_pivot_table_selected_item_error, new_pivot_table_show_values_as_base_field_error,
    new_unsupported_pivot_cache_source_type_error,
};
use crate::file::{File, namespace_strict_to_transitional};
use crate::lib_util::{
    coordinates_to_cell_name, count_utf16_string, in_str_slice, is_numeric,
    range_ref_to_coordinates, truncate_utf16_units,
};
use crate::numfmt::built_in_num_fmt_code;
use crate::xml::common::{XlsxExt, XlsxExtLst};
use crate::xml::pivot_cache::{
    DecodeX14PivotCacheDefinition, XlsxCacheField, XlsxCacheFields, XlsxCacheSource,
    XlsxPivotCacheDefinition, XlsxSharedItem, XlsxSharedItemData, XlsxSharedItems,
    XlsxWorksheetSource,
};
use crate::xml::pivot_table::{
    XlsxColFields, XlsxColItems, XlsxDataField, XlsxDataFields, XlsxField, XlsxI, XlsxItem,
    XlsxItems, XlsxLocation, XlsxPageField, XlsxPageFields, XlsxPivotField, XlsxPivotFields,
    XlsxPivotTableDefinition, XlsxPivotTableStyleInfo, XlsxRowFields, XlsxRowItems, XlsxX,
    XlsxX14DataField,
};
use crate::xml::workbook::{XlsxPivotCache, XlsxPivotCaches};

// ------------------------------------------------------------------
// Public types
// ------------------------------------------------------------------

/// Pivot table show-values-as type.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PivotTableShowValuesAsType(pub u8);

impl PivotTableShowValuesAsType {
    pub const NO_CALCULATION: PivotTableShowValuesAsType = PivotTableShowValuesAsType(0);
    pub const PERCENT_OF_GRAND_TOTAL: PivotTableShowValuesAsType = PivotTableShowValuesAsType(1);
    pub const PERCENT_OF_COLUMN_TOTAL: PivotTableShowValuesAsType = PivotTableShowValuesAsType(2);
    pub const PERCENT_OF_ROW_TOTAL: PivotTableShowValuesAsType = PivotTableShowValuesAsType(3);
    pub const PERCENT_OF: PivotTableShowValuesAsType = PivotTableShowValuesAsType(4);
    pub const PERCENT_OF_PARENT_ROW_TOTAL: PivotTableShowValuesAsType =
        PivotTableShowValuesAsType(5);
    pub const PERCENT_OF_PARENT_COLUMN_TOTAL: PivotTableShowValuesAsType =
        PivotTableShowValuesAsType(6);
    pub const PERCENT_OF_PARENT_TOTAL: PivotTableShowValuesAsType = PivotTableShowValuesAsType(7);
    pub const DIFFERENCE_FROM: PivotTableShowValuesAsType = PivotTableShowValuesAsType(8);
    pub const PERCENT_DIFFERENCE_FROM: PivotTableShowValuesAsType = PivotTableShowValuesAsType(9);
    pub const RUNNING_TOTAL_IN: PivotTableShowValuesAsType = PivotTableShowValuesAsType(10);
    pub const PERCENT_RUNNING_TOTAL_IN: PivotTableShowValuesAsType = PivotTableShowValuesAsType(11);
    pub const RANK_SMALLEST_TO_LARGEST: PivotTableShowValuesAsType = PivotTableShowValuesAsType(12);
    pub const RANK_LARGEST_TO_SMALLEST: PivotTableShowValuesAsType = PivotTableShowValuesAsType(13);
    pub const INDEX: PivotTableShowValuesAsType = PivotTableShowValuesAsType(14);
}

/// Pivot table show-values-as settings.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct PivotTableShowValuesAs {
    pub r#type: PivotTableShowValuesAsType,
    pub base_field: String,
    pub base_item: String,
}

/// Pivot table field options.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct PivotTableField {
    pub compact: bool,
    pub data: String,
    pub name: String,
    pub outline: bool,
    pub show_all: bool,
    pub insert_blank_row: bool,
    pub subtotal: String,
    pub default_subtotal: bool,
    pub num_fmt: i32,
    pub selected_items: Vec<String>,
    pub show_values_as: PivotTableShowValuesAs,
}

/// Pivot table creation options.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct PivotTableOptions {
    pub data_range: String,
    pub pivot_table_range: String,
    pub name: String,
    pub rows: Vec<PivotTableField>,
    pub columns: Vec<PivotTableField>,
    pub data: Vec<PivotTableField>,
    pub filter: Vec<PivotTableField>,
    pub row_grand_totals: bool,
    pub col_grand_totals: bool,
    pub show_drill: bool,
    pub use_auto_formatting: bool,
    pub page_over_then_down: bool,
    pub merge_item: bool,
    pub classic_layout: bool,
    pub compact_data: bool,
    pub show_error: bool,
    pub show_row_headers: bool,
    pub show_col_headers: bool,
    pub show_row_stripes: bool,
    pub show_col_stripes: bool,
    pub show_last_column: bool,
    pub field_print_titles: bool,
    pub item_print_titles: bool,
    pub pivot_table_style_name: String,

    // Internal state used while building the pivot table parts.
    items: HashMap<String, Vec<XlsxItem>>,
    shared_items: HashMap<String, XlsxSharedItems>,
    pivot_table_xml: String,
    pivot_cache_xml: String,
    pivot_sheet_name: String,
    pivot_data_range: String,
    named_data_range: bool,
}

// ------------------------------------------------------------------
// Internal helpers
// ------------------------------------------------------------------

const FORMULA_ERRORS: &[&str] = &[
    "#DIV/0!",
    "#NAME?",
    "#N/A",
    "#NUM!",
    "#VALUE!",
    "#REF!",
    "#NULL!",
    "#SPILL!",
    "#CALC!",
    "#GETTING_DATA",
];

fn show_values_as_map() -> HashMap<PivotTableShowValuesAsType, &'static str> {
    [
        (
            PivotTableShowValuesAsType::PERCENT_OF_GRAND_TOTAL,
            "percentOfTotal",
        ),
        (
            PivotTableShowValuesAsType::PERCENT_OF_COLUMN_TOTAL,
            "percentOfCol",
        ),
        (
            PivotTableShowValuesAsType::PERCENT_OF_ROW_TOTAL,
            "percentOfRow",
        ),
        (PivotTableShowValuesAsType::PERCENT_OF, "percent"),
        (
            PivotTableShowValuesAsType::PERCENT_OF_PARENT_ROW_TOTAL,
            "percentOfParentRow",
        ),
        (
            PivotTableShowValuesAsType::PERCENT_OF_PARENT_COLUMN_TOTAL,
            "percentOfParentCol",
        ),
        (
            PivotTableShowValuesAsType::PERCENT_OF_PARENT_TOTAL,
            "percentOfParent",
        ),
        (PivotTableShowValuesAsType::DIFFERENCE_FROM, "difference"),
        (
            PivotTableShowValuesAsType::PERCENT_DIFFERENCE_FROM,
            "percentDiff",
        ),
        (PivotTableShowValuesAsType::RUNNING_TOTAL_IN, "runTotal"),
        (
            PivotTableShowValuesAsType::PERCENT_RUNNING_TOTAL_IN,
            "percentOfRunningTotal",
        ),
        (
            PivotTableShowValuesAsType::RANK_SMALLEST_TO_LARGEST,
            "rankAscending",
        ),
        (
            PivotTableShowValuesAsType::RANK_LARGEST_TO_SMALLEST,
            "rankDescending",
        ),
        (PivotTableShowValuesAsType::INDEX, "index"),
    ]
    .into_iter()
    .collect()
}

fn x14_show_values_as_types() -> HashMap<PivotTableShowValuesAsType, bool> {
    [
        PivotTableShowValuesAsType::PERCENT_OF_PARENT_ROW_TOTAL,
        PivotTableShowValuesAsType::PERCENT_OF_PARENT_COLUMN_TOTAL,
        PivotTableShowValuesAsType::PERCENT_OF_PARENT_TOTAL,
        PivotTableShowValuesAsType::PERCENT_RUNNING_TOTAL_IN,
        PivotTableShowValuesAsType::RANK_SMALLEST_TO_LARGEST,
        PivotTableShowValuesAsType::RANK_LARGEST_TO_SMALLEST,
        PivotTableShowValuesAsType::INDEX,
    ]
    .into_iter()
    .map(|t| (t, true))
    .collect()
}

fn base_field_required() -> HashMap<PivotTableShowValuesAsType, bool> {
    [
        PivotTableShowValuesAsType::PERCENT_OF,
        PivotTableShowValuesAsType::PERCENT_OF_PARENT_TOTAL,
        PivotTableShowValuesAsType::DIFFERENCE_FROM,
        PivotTableShowValuesAsType::PERCENT_DIFFERENCE_FROM,
        PivotTableShowValuesAsType::RUNNING_TOTAL_IN,
        PivotTableShowValuesAsType::PERCENT_RUNNING_TOTAL_IN,
        PivotTableShowValuesAsType::RANK_SMALLEST_TO_LARGEST,
        PivotTableShowValuesAsType::RANK_LARGEST_TO_SMALLEST,
    ]
    .into_iter()
    .map(|t| (t, true))
    .collect()
}

fn base_item_required() -> HashMap<PivotTableShowValuesAsType, bool> {
    [
        PivotTableShowValuesAsType::PERCENT_OF,
        PivotTableShowValuesAsType::DIFFERENCE_FROM,
        PivotTableShowValuesAsType::PERCENT_DIFFERENCE_FROM,
    ]
    .into_iter()
    .map(|t| (t, true))
    .collect()
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum CellType {
    Unset,
    Number,
    Bool,
    Error,
    Formula,
    InlineString,
    SharedString,
}

// ------------------------------------------------------------------
// File implementation
// ------------------------------------------------------------------

impl File {
    /// Add a pivot table.
    pub fn add_pivot_table(&self, opts: &mut PivotTableOptions) -> Result<()> {
        let (_, pivot_table_sheet_path) = self.parse_format_pivot_table_set(opts)?;
        self.clear_calc_cache();
        let pivot_table_id = self.count_pivot_tables() + 1;
        let pivot_cache_id = self.count_pivot_cache() + 1;

        let sheet_relationships_pivot_table_xml =
            format!("../pivotTables/pivotTable{pivot_table_id}.xml");
        opts.pivot_table_xml = sheet_relationships_pivot_table_xml.replace("..", "xl");
        opts.pivot_cache_xml = format!("xl/pivotCache/pivotCacheDefinition{pivot_cache_id}.xml");
        self.add_pivot_cache(opts)?;

        let workbook_pivot_cache_rid = self.add_rels(
            &self.get_workbook_rels_path(),
            SOURCE_RELATIONSHIP_PIVOT_CACHE,
            &opts.pivot_cache_xml.trim_start_matches("xl/"),
            "",
        );
        let cache_id = self.add_workbook_pivot_cache(workbook_pivot_cache_rid);

        let pivot_cache_rels = format!("xl/pivotTables/_rels/pivotTable{pivot_table_id}.xml.rels");
        let _ = self.add_rels(
            &pivot_cache_rels,
            SOURCE_RELATIONSHIP_PIVOT_CACHE,
            &format!("../pivotCache/pivotCacheDefinition{pivot_cache_id}.xml"),
            "",
        );
        self.add_pivot_table_internal(cache_id, pivot_table_id, opts)?;

        let pivot_table_sheet_rels = format!(
            "xl/worksheets/_rels/{}.rels",
            pivot_table_sheet_path.trim_start_matches("xl/worksheets/")
        );
        self.add_rels(
            &pivot_table_sheet_rels,
            SOURCE_RELATIONSHIP_PIVOT_TABLE,
            &sheet_relationships_pivot_table_xml,
            "",
        );
        self.add_content_type_part(pivot_table_id, "pivotTable")?;
        self.add_content_type_part(pivot_cache_id, "pivotCache")
    }

    /// Get pivot tables on a worksheet.
    pub fn get_pivot_tables(&self, sheet: &str) -> Result<Vec<PivotTableOptions>> {
        let mut pivot_tables = Vec::new();
        let Some(name) = self.get_sheet_xml_path(sheet) else {
            return Err(Box::new(ErrSheetNotExist {
                sheet_name: sheet.to_string(),
            }));
        };
        let rels = format!(
            "xl/worksheets/_rels/{}.rels",
            name.trim_start_matches("xl/worksheets/")
        );
        let sheet_rels = match self.rels_reader(&rels)? {
            Some(r) => r,
            None => crate::xml::workbook::XlsxRelationships::default(),
        };
        for v in &sheet_rels.relationships {
            if v.r#type == SOURCE_RELATIONSHIP_PIVOT_TABLE {
                let pivot_table_xml = v.target.replace("..", "xl");
                let basename = std::path::Path::new(&v.target)
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("");
                let pivot_cache_rels = format!("xl/pivotTables/_rels/{basename}.rels");
                pivot_tables.push(self.get_pivot_table(
                    sheet,
                    &pivot_table_xml,
                    &pivot_cache_rels,
                )?);
            }
        }
        Ok(pivot_tables)
    }

    /// Delete a pivot table.
    pub fn delete_pivot_table(&self, sheet: &str, name: &str) -> Result<()> {
        let Some(sheet_xml) = self.get_sheet_xml_path(sheet) else {
            return Err(Box::new(ErrSheetNotExist {
                sheet_name: sheet.to_string(),
            }));
        };
        let rels = format!(
            "xl/worksheets/_rels/{}.rels",
            sheet_xml.trim_start_matches("xl/worksheets/")
        );
        let sheet_rels = match self.rels_reader(&rels)? {
            Some(r) => r,
            None => crate::xml::workbook::XlsxRelationships::default(),
        };
        let opts = self.get_pivot_tables(sheet)?;
        self.clear_calc_cache();
        let mut pivot_table_caches: HashMap<String, i32> = HashMap::new();
        for sheet_name in self.get_sheet_list() {
            if let Ok(pts) = self.get_pivot_tables(&sheet_name) {
                for pt in pts {
                    *pivot_table_caches.entry(pt.pivot_cache_xml).or_default() += 1;
                }
            }
        }

        for v in &sheet_rels.relationships.clone() {
            if v.r#type == SOURCE_RELATIONSHIP_PIVOT_TABLE {
                let pivot_table_xml = v.target.replace("..", "xl");
                for opt in &opts {
                    if opt.name == name && opt.pivot_table_xml == pivot_table_xml {
                        if pivot_table_caches
                            .get(&opt.pivot_cache_xml)
                            .copied()
                            .unwrap_or(0)
                            == 1
                        {
                            self.delete_workbook_pivot_cache(opt)?;
                        }
                        self.delete_sheet_relationships(sheet, &v.id);
                        return Ok(());
                    }
                }
            }
        }
        Err(new_no_exist_table_error(name).into())
    }

    // ------------------------------------------------------------------
    // Build helpers
    // ------------------------------------------------------------------

    fn parse_format_pivot_table_set(
        &self,
        opts: &mut PivotTableOptions,
    ) -> Result<(crate::xml::worksheet::XlsxWorksheet, String)> {
        let (pivot_table_sheet_name, _) =
            self.adjust_range(&opts.pivot_table_range).map_err(|err| {
                let err: Box<dyn std::error::Error + Send + Sync> =
                    new_pivot_table_range_error(&err.to_string()).into();
                err
            })?;
        if count_utf16_string(&opts.name) > MAX_FIELD_LENGTH {
            return Err(Box::new(ErrNameLength));
        }
        opts.pivot_sheet_name = pivot_table_sheet_name.clone();
        self.get_pivot_table_data_range(opts)?;
        let (data_sheet_name, _) = self.adjust_range(&opts.pivot_data_range).map_err(|err| {
            let err: Box<dyn std::error::Error + Send + Sync> =
                new_pivot_table_data_range_error(&err.to_string()).into();
            err
        })?;
        let data_sheet = self.work_sheet_reader(&data_sheet_name)?;
        let Some(pivot_table_sheet_path) = self.get_sheet_xml_path(&pivot_table_sheet_name) else {
            return Err(Box::new(ErrSheetNotExist {
                sheet_name: pivot_table_sheet_name,
            }));
        };
        if opts.compact_data && opts.classic_layout {
            return Err(Box::new(ErrPivotTableClassicLayout));
        }

        let mut col_data_fields = Vec::new();
        let mut row_data_fields = Vec::new();
        for f in &opts.filter {
            if in_pivot_table_field(&opts.columns, &f.data) != -1 {
                col_data_fields.push(f.data.clone());
            }
            if in_pivot_table_field(&opts.rows, &f.data) != -1 {
                row_data_fields.push(f.data.clone());
            }
        }
        if !col_data_fields.is_empty() {
            return Err(new_pivot_table_col_fields_error(&col_data_fields).into());
        }
        if !row_data_fields.is_empty() {
            return Err(new_pivot_table_row_fields_error(&row_data_fields).into());
        }
        Ok((data_sheet, pivot_table_sheet_path))
    }

    fn adjust_range(&self, range_str: &str) -> Result<(String, Vec<i32>)> {
        if range_str.is_empty() {
            return Err(Box::new(ErrParameterRequired));
        }
        let rng: Vec<&str> = range_str.split('!').collect();
        if rng.len() != 2 {
            return Err(Box::new(ErrParameterInvalid));
        }
        let trim_rng = rng[1].replace('$', "");
        let mut coordinates = range_ref_to_coordinates(&trim_rng)?;
        let (x1, y1, x2, y2) = (
            coordinates[0],
            coordinates[1],
            coordinates[2],
            coordinates[3],
        );
        if x1 == x2 && y1 == y2 {
            return Err(Box::new(ErrParameterInvalid));
        }
        if x2 < x1 {
            coordinates.swap(0, 2);
        }
        if y2 < y1 {
            coordinates.swap(1, 3);
        }
        Ok((rng[0].to_string(), coordinates))
    }

    fn get_table_fields_order(&self, opts: &PivotTableOptions) -> Result<Vec<String>> {
        let mut order = Vec::new();
        let mut opts = opts.clone();
        self.get_pivot_table_data_range(&mut opts)?;
        let (data_sheet, coordinates) = self.adjust_range(&opts.pivot_data_range)?;
        for col in coordinates[0]..=coordinates[2] {
            let coordinate = coordinates_to_cell_name(col, coordinates[1], false)?;
            let name = self.get_cell_value(&data_sheet, &coordinate)?;
            if name.is_empty() {
                return Err(ErrParameterInvalid.into());
            }
            order.push(name);
        }
        Ok(order)
    }

    fn add_missing_item(shared_items: &mut XlsxSharedItems) {
        for item in &shared_items.items {
            if matches!(item, XlsxSharedItem::M(_)) {
                return;
            }
        }
        shared_items
            .items
            .push(XlsxSharedItem::M(XlsxSharedItemData {
                xmlns: Some("".to_string()),
                ..Default::default()
            }));
        shared_items.contains_blank = Some(true);
    }

    fn add_number_item(shared_items: &mut XlsxSharedItems, val: &str) {
        for item in &shared_items.items {
            if let XlsxSharedItem::N(data) = item {
                if data.v.as_deref() == Some(val) {
                    return;
                }
            }
        }
        shared_items
            .items
            .push(XlsxSharedItem::N(XlsxSharedItemData {
                v: Some(val.to_string()),
                xmlns: Some("".to_string()),
                ..Default::default()
            }));
        shared_items.contains_number = Some(true);
    }

    fn add_boolean_item(shared_items: &mut XlsxSharedItems, val: &str) {
        let v = (val.to_uppercase() == "TRUE").to_string();
        for item in &shared_items.items {
            if let XlsxSharedItem::B(data) = item {
                if data.v.as_deref() == Some(&v) {
                    return;
                }
            }
        }
        shared_items
            .items
            .push(XlsxSharedItem::B(XlsxSharedItemData {
                v: Some(v),
                xmlns: Some("".to_string()),
                ..Default::default()
            }));
    }

    fn add_error_item(shared_items: &mut XlsxSharedItems, val: &str) {
        for item in &shared_items.items {
            if let XlsxSharedItem::E(data) = item {
                if data.v.as_deref() == Some(val) {
                    return;
                }
            }
        }
        shared_items
            .items
            .push(XlsxSharedItem::E(XlsxSharedItemData {
                v: Some(val.to_string()),
                xmlns: Some("".to_string()),
                ..Default::default()
            }));
    }

    fn add_string_item(shared_items: &mut XlsxSharedItems, val: &str) {
        for item in &shared_items.items {
            if let XlsxSharedItem::S(data) = item {
                if data.v.as_deref() == Some(val) {
                    return;
                }
            }
        }
        shared_items
            .items
            .push(XlsxSharedItem::S(XlsxSharedItemData {
                v: Some(val.to_string()),
                xmlns: Some("".to_string()),
                ..Default::default()
            }));
        shared_items.contains_string = Some(true);
    }

    fn check_selected_items(
        shared_items: &XlsxSharedItems,
        field: &str,
        selected_items: &[String],
    ) -> Result<()> {
        for shared_item in selected_items {
            let mut found = false;
            for item in &shared_items.items {
                match item {
                    XlsxSharedItem::M(_) => {
                        if shared_item.is_empty() {
                            found = true;
                        }
                    }
                    XlsxSharedItem::B(data) => {
                        if data
                            .v
                            .as_deref()
                            .map(|v| v.eq_ignore_ascii_case(shared_item))
                            .unwrap_or(false)
                        {
                            found = true;
                        }
                    }
                    XlsxSharedItem::N(data)
                    | XlsxSharedItem::E(data)
                    | XlsxSharedItem::S(data)
                    | XlsxSharedItem::D(data) => {
                        if data.v.as_deref() == Some(shared_item.as_str()) {
                            found = true;
                        }
                    }
                }
                if found {
                    break;
                }
            }
            if !found {
                return Err(new_pivot_table_selected_item_error(shared_item, field).into());
            }
        }
        Ok(())
    }

    fn cell_type(&self, sheet: &str, cell: &str) -> Result<CellType> {
        let ws = self.work_sheet_reader(sheet)?;
        let c = match crate::cell::find_cell(&ws, cell) {
            Some(c) => c,
            None => return Ok(CellType::Unset),
        };
        match c.t.as_deref() {
            Some("b") => return Ok(CellType::Bool),
            Some("e") => return Ok(CellType::Error),
            Some("inlineStr") => return Ok(CellType::InlineString),
            Some("s") => return Ok(CellType::SharedString),
            Some("str") => return Ok(CellType::Formula),
            _ => {}
        }
        if c.f.is_some() {
            return Ok(CellType::Formula);
        }
        let val = c.v.as_deref().unwrap_or("");
        if val.is_empty() {
            return Ok(CellType::Unset);
        }
        if is_numeric(val).0 {
            return Ok(CellType::Number);
        }
        Ok(CellType::SharedString)
    }

    fn add_shared_items(
        &self,
        sheet: &str,
        col: i32,
        from_row: i32,
        to_row: i32,
    ) -> Result<XlsxSharedItems> {
        let mut si = XlsxSharedItems::default();
        for row in from_row..=to_row {
            let cell = coordinates_to_cell_name(col, row, false)?;
            let val = match self.calc_cell_value(sheet, &cell) {
                Ok(v) => v,
                Err(err) => {
                    let msg = err.to_string();
                    if FORMULA_ERRORS.iter().any(|e| msg.contains(e)) {
                        Self::add_error_item(&mut si, &msg);
                        continue;
                    }
                    return Err(err);
                }
            };
            if val.is_empty() {
                Self::add_missing_item(&mut si);
                continue;
            }
            let cell_type = self.cell_type(sheet, &cell)?;
            match cell_type {
                CellType::Unset | CellType::Number => Self::add_number_item(&mut si, &val),
                CellType::Bool => Self::add_boolean_item(&mut si, &val),
                CellType::Error => Self::add_error_item(&mut si, &val),
                CellType::Formula => {
                    if is_numeric(&val).0 {
                        Self::add_number_item(&mut si, &val);
                    } else {
                        Self::add_string_item(&mut si, &val);
                    }
                }
                CellType::InlineString | CellType::SharedString => {
                    Self::add_string_item(&mut si, &val)
                }
            }
        }
        Ok(si)
    }

    fn build_pivot_shared_items(
        &self,
        opts: &mut PivotTableOptions,
        idx: i32,
        coordinates: &[i32],
        field: &PivotTableField,
    ) -> Result<()> {
        let mut items = Vec::new();
        let mut shared_items = self.add_shared_items(
            &opts.pivot_sheet_name,
            coordinates[0] + idx,
            coordinates[1] + 1,
            coordinates[3],
        )?;
        let mut i = 0i32;
        for item in &shared_items.items {
            let hidden = if field.selected_items.is_empty() {
                false
            } else {
                match item {
                    XlsxSharedItem::M(_) => in_str_slice(&field.selected_items, "", true) == -1,
                    XlsxSharedItem::B(data) => {
                        in_str_slice(
                            &field.selected_items,
                            data.v.as_deref().unwrap_or(""),
                            false,
                        ) == -1
                    }
                    XlsxSharedItem::N(data)
                    | XlsxSharedItem::E(data)
                    | XlsxSharedItem::S(data)
                    | XlsxSharedItem::D(data) => {
                        in_str_slice(&field.selected_items, data.v.as_deref().unwrap_or(""), true)
                            == -1
                    }
                }
            };
            items.push(XlsxItem {
                h: Some(hidden),
                x: Some(i as i64),
                ..Default::default()
            });
            i += 1;
        }
        Self::check_selected_items(&shared_items, &field.data, &field.selected_items)?;

        let mut types_set = HashMap::new();
        let mut num_count = 0i32;
        for item in &shared_items.items {
            let tag = match item {
                XlsxSharedItem::M(_) => "m",
                XlsxSharedItem::N(_) => "n",
                XlsxSharedItem::B(_) => "b",
                XlsxSharedItem::E(_) => "e",
                XlsxSharedItem::S(_) => "s",
                XlsxSharedItem::D(_) => "d",
            };
            types_set.insert(tag, true);
            if tag == "n" {
                num_count += 1;
            }
        }
        shared_items.contains_mixed_types = Some(types_set.len() > 1);
        if num_count == i {
            shared_items.contains_integer = Some(true);
            shared_items.contains_string = Some(false);
            shared_items.contains_semi_mixed_types = Some(false);
        }
        shared_items.count = i;

        opts.items.insert(field.data.clone(), items);
        opts.shared_items.insert(field.data.clone(), shared_items);
        Ok(())
    }

    fn add_pivot_shared_items(
        &self,
        opts: &mut PivotTableOptions,
        coordinates: &[i32],
        fields_type: &str,
    ) -> Result<()> {
        let fields: Vec<PivotTableField> = match fields_type {
            "filters" => opts.filter.clone(),
            "cols" => opts.columns.clone(),
            "rows" => opts.rows.clone(),
            _ => Vec::new(),
        };
        let mut show_values_as_base_field_required = false;
        for field in &opts.data {
            if let Some(t) = show_values_as_map().get(&field.show_values_as.r#type) {
                let _ = t;
                if base_field_required()
                    .get(&field.show_values_as.r#type)
                    .copied()
                    .unwrap_or(false)
                {
                    show_values_as_base_field_required = true;
                }
            }
        }
        let field_refs: Vec<&PivotTableField> = fields.iter().collect();
        let fields_index = self.get_pivot_fields_index(&field_refs, opts)?;
        for (i, field) in fields.iter().enumerate() {
            if !field.selected_items.is_empty() || show_values_as_base_field_required {
                self.build_pivot_shared_items(opts, fields_index[i], coordinates, field)?;
            }
        }
        Ok(())
    }

    fn add_pivot_cache(&self, opts: &mut PivotTableOptions) -> Result<()> {
        let (data_sheet, coordinates) = self.adjust_range(&opts.pivot_data_range)?;
        let order = self.get_table_fields_order(opts)?;
        let top_left_cell = coordinates_to_cell_name(coordinates[0], coordinates[1], false)?;
        let bottom_right_cell = coordinates_to_cell_name(coordinates[2], coordinates[3], false)?;
        let mut pc = XlsxPivotCacheDefinition {
            xmlns: Some(NAMESPACE_SPREADSHEET.to_string()),
            save_data: false,
            refresh_on_load: Some(true),
            created_version: Some(PIVOT_TABLE_VERSION),
            refreshed_version: Some(PIVOT_TABLE_REFRESHED_VERSION),
            min_refreshable_version: Some(PIVOT_TABLE_VERSION),
            cache_source: Some(XlsxCacheSource {
                r#type: "worksheet".to_string(),
                worksheet_source: Some(XlsxWorksheetSource {
                    r#ref: Some(format!("{top_left_cell}:{bottom_right_cell}")),
                    sheet: Some(data_sheet),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            cache_fields: Some(XlsxCacheFields::default()),
            ..Default::default()
        };
        if opts.named_data_range {
            if let Some(source) = pc.cache_source.as_mut() {
                source.worksheet_source = Some(XlsxWorksheetSource {
                    name: Some(opts.data_range.clone()),
                    ..Default::default()
                });
            }
        }
        self.add_pivot_shared_items(opts, &coordinates, "filters")?;
        self.add_pivot_shared_items(opts, &coordinates, "cols")?;
        self.add_pivot_shared_items(opts, &coordinates, "rows")?;

        let cache_fields = pc.cache_fields.as_mut().unwrap();
        for name in &order {
            let si = opts
                .shared_items
                .get(name)
                .cloned()
                .unwrap_or_else(|| XlsxSharedItems {
                    contains_blank: Some(true),
                    items: vec![XlsxSharedItem::M(XlsxSharedItemData {
                        xmlns: Some("".to_string()),
                        ..Default::default()
                    })],
                    ..Default::default()
                });
            cache_fields.cache_field.push(XlsxCacheField {
                name: name.clone(),
                shared_items: Some(si),
                ..Default::default()
            });
        }
        cache_fields.count = cache_fields.cache_field.len() as i32;
        let mut pivot_cache = xml_to_string(&pc)?.into_bytes();
        crate::file::strip_empty_attributes(&mut pivot_cache);
        self.save_file_list(&opts.pivot_cache_xml, &pivot_cache);
        Ok(())
    }

    fn add_pivot_table_internal(
        &self,
        cache_id: i64,
        pivot_table_id: i32,
        opts: &mut PivotTableOptions,
    ) -> Result<()> {
        let (_, coordinates) = self.adjust_range(&opts.pivot_table_range)?;
        let top_left_cell = coordinates_to_cell_name(coordinates[0], coordinates[1], false)?;
        let bottom_right_cell = coordinates_to_cell_name(coordinates[2], coordinates[3], false)?;

        let pivot_table_style = if opts.pivot_table_style_name.is_empty() {
            "PivotStyleLight16".to_string()
        } else {
            opts.pivot_table_style_name.clone()
        };
        let mut pt = XlsxPivotTableDefinition {
            xmlns: Some(NAMESPACE_SPREADSHEET.to_string()),
            name: opts.name.clone(),
            cache_id,
            row_grand_totals: Some(opts.row_grand_totals),
            col_grand_totals: Some(opts.col_grand_totals),
            updated_version: Some(PIVOT_TABLE_REFRESHED_VERSION as i64),
            min_refreshable_version: Some(PIVOT_TABLE_VERSION as i64),
            show_drill: Some(opts.show_drill),
            use_auto_formatting: Some(opts.use_auto_formatting),
            page_over_then_down: Some(opts.page_over_then_down),
            merge_item: Some(opts.merge_item),
            created_version: Some(PIVOT_TABLE_VERSION as i64),
            compact_data: Some(opts.compact_data),
            grid_drop_zones: Some(opts.classic_layout),
            show_error: Some(opts.show_error),
            field_print_titles: Some(opts.field_print_titles),
            item_print_titles: Some(opts.item_print_titles),
            data_caption: "Values".to_string(),
            location: Some(XlsxLocation {
                r#ref: format!("{top_left_cell}:{bottom_right_cell}"),
                first_data_col: 1,
                first_data_row: 1,
                first_header_row: 1,
                ..Default::default()
            }),
            pivot_fields: Some(XlsxPivotFields::default()),
            row_items: Some(XlsxRowItems {
                count: 1,
                i: vec![XlsxI {
                    x: vec![XlsxX::default()],
                }],
            }),
            col_items: Some(XlsxColItems {
                count: 1,
                i: vec![XlsxI::default()],
            }),
            pivot_table_style_info: Some(XlsxPivotTableStyleInfo {
                name: pivot_table_style,
                show_row_headers: opts.show_row_headers,
                show_col_headers: opts.show_col_headers,
                show_row_stripes: if opts.show_row_stripes {
                    Some(true)
                } else {
                    None
                },
                show_col_stripes: if opts.show_col_stripes {
                    Some(true)
                } else {
                    None
                },
                show_last_column: if opts.show_last_column {
                    Some(true)
                } else {
                    None
                },
            }),
            ..Default::default()
        };
        if pt.name.is_empty() {
            pt.name = format!("PivotTable{pivot_table_id}");
        }
        opts.name = pt.name.clone();
        if opts.classic_layout {
            pt.compact = Some(false);
            pt.compact_data = Some(false);
        }

        self.add_pivot_fields(&mut pt, opts)?;
        if let Some(pivot_fields) = pt.pivot_fields.as_mut() {
            pivot_fields.count = pivot_fields.pivot_field.len() as i64;
        }
        let _ = self.add_pivot_row_fields(&mut pt, opts);
        let _ = self.add_pivot_col_fields(&mut pt, opts);
        let _ = self.add_pivot_page_fields(&mut pt, opts);
        self.add_pivot_data_fields(&mut pt, opts)?;

        // Preserve data field extension lists as raw XML so that x14 elements
        // are not escaped by serde.
        let mut data_field_ext_xmls = Vec::new();
        if let Some(ref mut data_fields) = pt.data_fields {
            for df in &mut data_fields.data_field {
                if let Some(ext_lst) = df.ext_lst.take() {
                    data_field_ext_xmls.push(Some(serialize_data_field_ext_lst(&ext_lst)));
                } else {
                    data_field_ext_xmls.push(None);
                }
            }
        }

        let mut pivot_table = xml_to_string(&pt)?.into_bytes();
        crate::file::strip_empty_attributes(&mut pivot_table);
        inject_data_field_ext_lst(&mut pivot_table, &data_field_ext_xmls);
        self.save_file_list(&opts.pivot_table_xml, &pivot_table);
        Ok(())
    }

    fn add_pivot_row_fields(
        &self,
        pt: &mut XlsxPivotTableDefinition,
        opts: &PivotTableOptions,
    ) -> Result<()> {
        let row_fields_index =
            self.get_pivot_fields_index(&opts.rows.iter().collect::<Vec<_>>(), opts)?;
        for field_idx in row_fields_index {
            if pt.row_fields.is_none() {
                pt.row_fields = Some(XlsxRowFields::default());
            }
            pt.row_fields.as_mut().unwrap().field.push(XlsxField {
                x: field_idx as i64,
            });
        }
        if let Some(row_fields) = pt.row_fields.as_mut() {
            row_fields.count = row_fields.field.len() as i64;
        }
        Ok(())
    }

    fn add_pivot_page_fields(
        &self,
        pt: &mut XlsxPivotTableDefinition,
        opts: &PivotTableOptions,
    ) -> Result<()> {
        let page_fields_index =
            self.get_pivot_fields_index(&opts.filter.iter().collect::<Vec<_>>(), opts)?;
        let page_fields_name = self.get_pivot_table_fields_name(&opts.filter);
        for (idx, page_field) in page_fields_index.iter().enumerate() {
            if pt.page_fields.is_none() {
                pt.page_fields = Some(XlsxPageFields::default());
            }
            pt.page_fields
                .as_mut()
                .unwrap()
                .page_field
                .push(XlsxPageField {
                    fld: *page_field as i64,
                    name: Some(page_fields_name[idx].clone()),
                    ..Default::default()
                });
        }
        if let Some(page_fields) = pt.page_fields.as_mut() {
            page_fields.count = page_fields.page_field.len() as i64;
        }
        Ok(())
    }

    fn add_pivot_data_fields(
        &self,
        pt: &mut XlsxPivotTableDefinition,
        opts: &mut PivotTableOptions,
    ) -> Result<()> {
        let data_fields_index =
            self.get_pivot_fields_index(&opts.data.iter().collect::<Vec<_>>(), opts)?;
        let order = self.get_table_fields_order(opts)?;
        let data_fields_subtotals = self.get_pivot_table_fields_subtotal(&opts.data);
        let data_fields_name = self.get_pivot_table_fields_name(&opts.data);
        let data_fields_num_fmt_id = self.get_pivot_table_fields_num_fmt_id(&opts.data);
        for (idx, data_field) in data_fields_index.iter().enumerate() {
            if pt.data_fields.is_none() {
                pt.data_fields = Some(XlsxDataFields::default());
            }
            let mut df = XlsxDataField {
                name: Some(data_fields_name[idx].clone()),
                fld: *data_field as i64,
                subtotal: Some(data_fields_subtotals[idx].clone()),
                num_fmt_id: Some(data_fields_num_fmt_id[idx] as i64),
                ..Default::default()
            };
            self.set_pivot_table_show_values_as(&mut df, idx, &order, opts)?;
            pt.data_fields.as_mut().unwrap().data_field.push(df);
        }
        if let Some(data_fields) = pt.data_fields.as_mut() {
            data_fields.count = data_fields.data_field.len() as i64;
        }
        Ok(())
    }

    fn set_pivot_table_show_values_as(
        &self,
        df: &mut XlsxDataField,
        idx: usize,
        order: &[String],
        opts: &PivotTableOptions,
    ) -> Result<()> {
        let show_values_as_type = opts.data[idx].show_values_as.r#type;
        if show_values_as_type == PivotTableShowValuesAsType::NO_CALCULATION {
            return Ok(());
        }
        let map = show_values_as_map();
        let Some(show_data_as) = map.get(&show_values_as_type).copied() else {
            return Err(Box::new(ErrUnsupportedPivotTableShowValuesAsType));
        };
        df.show_data_as = Some(show_data_as.to_string());
        if x14_show_values_as_types()
            .get(&show_values_as_type)
            .copied()
            .unwrap_or(false)
        {
            df.show_data_as = None;
            let x14_df = XlsxX14DataField {
                xmlns_x14: Some(NAMESPACE_SPREADSHEET_X14.to_string()),
                pivot_show_as: Some(show_data_as.to_string()),
                ..Default::default()
            };
            let data_field_bytes = xml_to_string(&x14_df)?;
            let ext = XlsxExt {
                uri: Some(EXT_URI_PIVOT_DATA_FIELD.to_string()),
                content: data_field_bytes,
                ..Default::default()
            };
            df.ext_lst = Some(XlsxExtLst { ext: vec![ext] });
        }
        if base_field_required()
            .get(&show_values_as_type)
            .copied()
            .unwrap_or(false)
        {
            let base_field = &opts.data[idx].show_values_as.base_field;
            if base_field.is_empty() {
                return Err(ErrPivotTableShowValuesAsBaseField.into());
            }
            let Some(shared_items) = opts.shared_items.get(base_field) else {
                return Err(new_pivot_table_show_values_as_base_field_error(base_field).into());
            };
            let base_field_index = in_str_slice(order, base_field, true);
            df.base_field = Some(base_field_index as i64);
            if base_item_required()
                .get(&show_values_as_type)
                .copied()
                .unwrap_or(false)
            {
                self.set_pivot_table_show_values_as_base_item(
                    df,
                    base_field,
                    &opts.data[idx].show_values_as.base_item,
                    shared_items,
                )?;
            }
        }
        Ok(())
    }

    fn set_pivot_table_show_values_as_base_item(
        &self,
        df: &mut XlsxDataField,
        base_field: &str,
        base_item: &str,
        shared_items: &XlsxSharedItems,
    ) -> Result<()> {
        if base_item.is_empty() {
            return Err(Box::new(ErrPivotTableShowValuesAsBaseItem));
        }
        Self::check_selected_items(shared_items, base_field, &[base_item.to_string()])?;
        for (i, item) in shared_items.items.iter().enumerate() {
            match item {
                XlsxSharedItem::B(data) => {
                    if data
                        .v
                        .as_deref()
                        .map(|v| v.eq_ignore_ascii_case(base_item))
                        .unwrap_or(false)
                    {
                        df.base_item = Some(i as i64);
                    }
                }
                XlsxSharedItem::N(data)
                | XlsxSharedItem::E(data)
                | XlsxSharedItem::S(data)
                | XlsxSharedItem::D(data) => {
                    if data.v.as_deref() == Some(base_item) {
                        df.base_item = Some(i as i64);
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn add_pivot_col_fields(
        &self,
        pt: &mut XlsxPivotTableDefinition,
        opts: &PivotTableOptions,
    ) -> Result<()> {
        if opts.columns.is_empty() {
            if opts.data.len() <= 1 {
                return Ok(());
            }
            pt.col_fields = Some(XlsxColFields {
                count: 1,
                field: vec![XlsxField { x: -2 }],
            });
            return Ok(());
        }
        pt.col_fields = Some(XlsxColFields::default());
        let col_fields_index =
            self.get_pivot_fields_index(&opts.columns.iter().collect::<Vec<_>>(), opts)?;
        for field_idx in col_fields_index {
            pt.col_fields.as_mut().unwrap().field.push(XlsxField {
                x: field_idx as i64,
            });
        }
        if opts.data.len() > 1 {
            pt.col_fields
                .as_mut()
                .unwrap()
                .field
                .push(XlsxField { x: -2 });
        }
        pt.col_fields.as_mut().unwrap().count = pt.col_fields.as_ref().unwrap().field.len() as i64;
        Ok(())
    }

    fn set_classic_layout(fld: &mut XlsxPivotField, classic_layout: bool) {
        if classic_layout {
            fld.compact = Some(false);
            fld.outline = Some(false);
        }
    }

    fn add_pivot_fields(
        &self,
        pt: &mut XlsxPivotTableDefinition,
        opts: &mut PivotTableOptions,
    ) -> Result<()> {
        let order = self.get_table_fields_order(opts)?;
        let x = 0i64;
        for name in &order {
            if in_pivot_table_field(&opts.rows, name) != -1 {
                let (row_options, ok) = self.get_pivot_table_field_options(name, &opts.rows);
                let mut items = opts.items.get(name).cloned().unwrap_or_default();
                if ok && row_options.default_subtotal {
                    items.push(XlsxItem {
                        t: Some("default".to_string()),
                        ..Default::default()
                    });
                }
                if items.is_empty() {
                    items.push(XlsxItem {
                        x: Some(x),
                        ..Default::default()
                    });
                }
                let mut fld = XlsxPivotField {
                    name: Some(self.get_pivot_table_field_name(name, &opts.rows)),
                    axis: Some("axisRow".to_string()),
                    data_field: Some(in_pivot_table_field(&opts.data, name) != -1),
                    compact: Some(row_options.compact),
                    outline: Some(row_options.outline),
                    multiple_item_selection_allowed: Some(
                        !opts.items.get(name).map(|v| v.is_empty()).unwrap_or(true),
                    ),
                    show_all: Some(row_options.show_all),
                    insert_blank_row: Some(row_options.insert_blank_row),
                    default_subtotal: Some(row_options.default_subtotal),
                    items: Some(XlsxItems {
                        count: items.len() as i64,
                        item: items,
                    }),
                    ..Default::default()
                };
                Self::set_classic_layout(&mut fld, opts.classic_layout);
                pt.pivot_fields.as_mut().unwrap().pivot_field.push(fld);
                continue;
            }
            if in_pivot_table_field(&opts.filter, name) != -1 {
                let mut items = opts.items.get(name).cloned().unwrap_or_default();
                items.push(XlsxItem {
                    t: Some("default".to_string()),
                    ..Default::default()
                });
                let mut fld = XlsxPivotField {
                    axis: Some("axisPage".to_string()),
                    data_field: Some(in_pivot_table_field(&opts.data, name) != -1),
                    multiple_item_selection_allowed: Some(
                        !opts.items.get(name).map(|v| v.is_empty()).unwrap_or(true),
                    ),
                    name: Some(self.get_pivot_table_field_name(name, &opts.columns)),
                    items: Some(XlsxItems {
                        count: items.len() as i64,
                        item: items,
                    }),
                    ..Default::default()
                };
                Self::set_classic_layout(&mut fld, opts.classic_layout);
                pt.pivot_fields.as_mut().unwrap().pivot_field.push(fld);
                continue;
            }
            if in_pivot_table_field(&opts.columns, name) != -1 {
                let (column_options, ok) = self.get_pivot_table_field_options(name, &opts.columns);
                let mut items = opts.items.get(name).cloned().unwrap_or_default();
                if ok && column_options.default_subtotal {
                    items.push(XlsxItem {
                        t: Some("default".to_string()),
                        ..Default::default()
                    });
                }
                if items.is_empty() {
                    items.push(XlsxItem {
                        x: Some(x),
                        ..Default::default()
                    });
                }
                let mut fld = XlsxPivotField {
                    name: Some(self.get_pivot_table_field_name(name, &opts.columns)),
                    axis: Some("axisCol".to_string()),
                    data_field: Some(in_pivot_table_field(&opts.data, name) != -1),
                    compact: Some(column_options.compact),
                    outline: Some(column_options.outline),
                    multiple_item_selection_allowed: Some(
                        !opts.items.get(name).map(|v| v.is_empty()).unwrap_or(true),
                    ),
                    show_all: Some(column_options.show_all),
                    insert_blank_row: Some(column_options.insert_blank_row),
                    default_subtotal: Some(column_options.default_subtotal),
                    items: Some(XlsxItems {
                        count: items.len() as i64,
                        item: items,
                    }),
                    ..Default::default()
                };
                Self::set_classic_layout(&mut fld, opts.classic_layout);
                pt.pivot_fields.as_mut().unwrap().pivot_field.push(fld);
                continue;
            }
            if in_pivot_table_field(&opts.data, name) != -1 {
                let mut fld = XlsxPivotField {
                    data_field: Some(true),
                    ..Default::default()
                };
                Self::set_classic_layout(&mut fld, opts.classic_layout);
                pt.pivot_fields.as_mut().unwrap().pivot_field.push(fld);
                continue;
            }
            let mut fld = XlsxPivotField::default();
            Self::set_classic_layout(&mut fld, opts.classic_layout);
            pt.pivot_fields.as_mut().unwrap().pivot_field.push(fld);
        }
        Ok(())
    }

    fn get_pivot_fields_index(
        &self,
        fields: &[&PivotTableField],
        opts: &PivotTableOptions,
    ) -> Result<Vec<i32>> {
        let mut pivot_fields_index = Vec::new();
        let orders = self.get_table_fields_order(opts)?;
        for field in fields {
            let pos = in_str_slice(&orders, &field.data, true);
            if pos != -1 {
                pivot_fields_index.push(pos);
            }
        }
        Ok(pivot_fields_index)
    }

    fn get_pivot_table_fields_subtotal(&self, fields: &[PivotTableField]) -> Vec<String> {
        let enums = [
            "average",
            "count",
            "countNums",
            "max",
            "min",
            "product",
            "stdDev",
            "stdDevp",
            "sum",
            "var",
            "varp",
        ];
        let mut result = Vec::with_capacity(fields.len());
        for fld in fields {
            let mut val = "sum".to_string();
            for e in &enums {
                if e.eq_ignore_ascii_case(&fld.subtotal) {
                    val = (*e).to_string();
                    break;
                }
            }
            result.push(val);
        }
        result
    }

    fn get_pivot_table_fields_name(&self, fields: &[PivotTableField]) -> Vec<String> {
        let mut result = Vec::with_capacity(fields.len());
        for fld in fields {
            if count_utf16_string(&fld.name) > MAX_FIELD_LENGTH {
                result.push(truncate_utf16_units(&fld.name, MAX_FIELD_LENGTH));
            } else {
                result.push(fld.name.clone());
            }
        }
        result
    }

    fn get_pivot_table_field_name(&self, name: &str, fields: &[PivotTableField]) -> String {
        let fields_name = self.get_pivot_table_fields_name(fields);
        for (idx, field) in fields.iter().enumerate() {
            if field.data == name {
                return fields_name[idx].clone();
            }
        }
        String::new()
    }

    fn get_pivot_table_fields_num_fmt_id(&self, fields: &[PivotTableField]) -> Vec<i32> {
        let mut result = Vec::with_capacity(fields.len());
        for fld in fields {
            if built_in_num_fmt_code(fld.num_fmt).is_some() {
                result.push(fld.num_fmt);
                continue;
            }
            if (27..=36).contains(&fld.num_fmt) || (50..=81).contains(&fld.num_fmt) {
                result.push(fld.num_fmt);
                continue;
            }
            result.push(0);
        }
        result
    }

    fn get_pivot_table_field_options(
        &self,
        name: &str,
        fields: &[PivotTableField],
    ) -> (PivotTableField, bool) {
        for field in fields {
            if field.data == name {
                return (field.clone(), true);
            }
        }
        (PivotTableField::default(), false)
    }

    fn add_workbook_pivot_cache(&self, rid: i32) -> i64 {
        let mut wb = self.workbook_reader().unwrap_or_default();
        if wb.pivot_caches.is_none() {
            wb.pivot_caches = Some(XlsxPivotCaches::default());
        }
        let mut cache_id = 1i64;
        if let Some(ref caches) = wb.pivot_caches {
            for pivot_cache in &caches.pivot_cache {
                if pivot_cache.cache_id > cache_id {
                    cache_id = pivot_cache.cache_id;
                }
            }
        }
        cache_id += 1;
        wb.pivot_caches
            .as_mut()
            .unwrap()
            .pivot_cache
            .push(XlsxPivotCache {
                cache_id,
                rid: Some(format!("rId{rid}")),
            });
        *self.workbook.lock().unwrap() = Some(wb);
        cache_id
    }

    // ------------------------------------------------------------------
    // Read helpers
    // ------------------------------------------------------------------

    fn get_pivot_table_data_range(&self, opts: &mut PivotTableOptions) -> Result<()> {
        if opts.data_range.is_empty() {
            return Err(new_pivot_table_data_range_error(&ErrParameterRequired.to_string()).into());
        }
        if !opts.pivot_data_range.is_empty() {
            return Ok(());
        }
        if opts.data_range.contains('!') {
            opts.pivot_data_range = opts.data_range.clone();
            return Ok(());
        }
        let tables = self.get_tables_for_workbook()?;
        for (sheet_name, sheet_tables) in tables {
            for table in sheet_tables {
                if table.name == opts.data_range {
                    opts.pivot_data_range = format!("{}!{}", sheet_name, table.range);
                    opts.named_data_range = true;
                    return Ok(());
                }
            }
        }
        if !opts.named_data_range {
            let refers_to = self.get_defined_name_ref_to(&opts.data_range, &opts.pivot_sheet_name);
            if !refers_to.is_empty() {
                opts.pivot_data_range = refers_to;
                opts.named_data_range = true;
                return Ok(());
            }
        }
        Err(new_pivot_table_data_range_error(&ErrParameterInvalid.to_string()).into())
    }

    fn get_pivot_table(
        &self,
        sheet: &str,
        pivot_table_xml: &str,
        pivot_cache_rels: &str,
    ) -> Result<PivotTableOptions> {
        let rels = match self.rels_reader(pivot_cache_rels)? {
            Some(r) => r,
            None => crate::xml::workbook::XlsxRelationships::default(),
        };
        let mut pivot_cache_xml = String::new();
        for v in &rels.relationships {
            if v.r#type == SOURCE_RELATIONSHIP_PIVOT_CACHE {
                pivot_cache_xml = v.target.replace("..", "xl");
                break;
            }
        }
        let pc = self.pivot_cache_reader(&pivot_cache_xml)?;
        let pt = self.pivot_table_reader(pivot_table_xml)?;
        let Some(ref cache_source) = pc.cache_source else {
            return Err(new_unsupported_pivot_cache_source_type_error("").into());
        };
        let Some(ref worksheet_source) = cache_source.worksheet_source else {
            return Err(new_unsupported_pivot_cache_source_type_error(&cache_source.r#type).into());
        };
        let data_range = format!(
            "{}!{}",
            worksheet_source.sheet.as_deref().unwrap_or(""),
            worksheet_source.r#ref.as_deref().unwrap_or("")
        );
        let location_ref = pt
            .location
            .as_ref()
            .map(|l| l.r#ref.clone())
            .unwrap_or_default();
        let mut opts = PivotTableOptions {
            pivot_table_xml: pivot_table_xml.to_string(),
            pivot_cache_xml,
            pivot_sheet_name: sheet.to_string(),
            data_range,
            pivot_table_range: format!("{}!{}", sheet, location_ref),
            name: pt.name.clone(),
            classic_layout: pt.grid_drop_zones.unwrap_or(false),
            field_print_titles: pt.field_print_titles.unwrap_or(false),
            item_print_titles: pt.item_print_titles.unwrap_or(false),
            ..Default::default()
        };
        if let Some(ref name) = worksheet_source.name {
            opts.data_range = name.clone();
            let _ = self.get_pivot_table_data_range(&mut opts);
        }
        opts.row_grand_totals = pt.row_grand_totals.unwrap_or(false);
        opts.col_grand_totals = pt.col_grand_totals.unwrap_or(false);
        opts.show_drill = pt.show_drill.unwrap_or(false);
        opts.use_auto_formatting = pt.use_auto_formatting.unwrap_or(false);
        opts.page_over_then_down = pt.page_over_then_down.unwrap_or(false);
        opts.merge_item = pt.merge_item.unwrap_or(false);
        opts.compact_data = pt.compact_data.unwrap_or(false);
        opts.show_error = pt.show_error.unwrap_or(false);
        if let Some(ref si) = pt.pivot_table_style_info {
            opts.show_row_headers = si.show_row_headers;
            opts.show_col_headers = si.show_col_headers;
            opts.show_row_stripes = si.show_row_stripes.unwrap_or(false);
            opts.show_col_stripes = si.show_col_stripes.unwrap_or(false);
            opts.show_last_column = si.show_last_column.unwrap_or(false);
            opts.pivot_table_style_name = si.name.clone();
        }
        let _ = self.get_pivot_table_data_range(&mut opts);
        self.extract_pivot_table_fields(&pt, &pc, &mut opts);
        Ok(opts)
    }

    fn pivot_table_reader(&self, path: &str) -> Result<XlsxPivotTableDefinition> {
        let content = namespace_strict_to_transitional(&self.read_xml(path));
        let mut pivot_table = XlsxPivotTableDefinition::default();
        if !content.is_empty() {
            let content_str = String::from_utf8_lossy(&content);
            let (content_without_ext, ext_xmls) = extract_data_field_ext_lst(&content_str);
            pivot_table = xml_from_reader(content_without_ext.as_bytes())?;
            if let Some(ref mut data_fields) = pivot_table.data_fields {
                for (i, df) in data_fields.data_field.iter_mut().enumerate() {
                    if let Some(inner) = ext_xmls.get(i).and_then(|o| o.as_ref()) {
                        df.ext_lst = Some(crate::xml::common::parse_ext_lst_content(inner)?);
                    }
                }
            }
        }
        Ok(pivot_table)
    }

    fn pivot_cache_reader(&self, path: &str) -> Result<XlsxPivotCacheDefinition> {
        let content = namespace_strict_to_transitional(&self.read_xml(path));
        let mut pivot_cache = XlsxPivotCacheDefinition::default();
        if !content.is_empty() {
            pivot_cache = xml_from_reader(content.as_slice())?;
        }
        Ok(pivot_cache)
    }

    fn extract_pivot_table_fields(
        &self,
        pt: &XlsxPivotTableDefinition,
        pc: &XlsxPivotCacheDefinition,
        opts: &mut PivotTableOptions,
    ) {
        let order = pc.get_pivot_cache_fields_name();
        if let Some(ref pivot_fields) = pt.pivot_fields {
            for (field_idx, field) in pivot_fields.pivot_field.iter().enumerate() {
                let name = order.get(field_idx).cloned().unwrap_or_default();
                match field.axis.as_deref() {
                    Some("axisRow") => opts.rows.push(pc.extract_pivot_table_field(&name, field)),
                    Some("axisCol") => opts
                        .columns
                        .push(pc.extract_pivot_table_field(&name, field)),
                    Some("axisPage") => {
                        opts.filter.push(pc.extract_pivot_table_field(&name, field))
                    }
                    _ => {}
                }
            }
        }
        if let Some(ref data_fields) = pt.data_fields {
            for field in &data_fields.data_field {
                let mut data_field = PivotTableField {
                    data: order.get(field.fld as usize).cloned().unwrap_or_default(),
                    name: field.name.clone().unwrap_or_default(),
                    subtotal: title_case(field.subtotal.as_deref().unwrap_or("sum")),
                    num_fmt: field.num_fmt_id.unwrap_or(0) as i32,
                    ..Default::default()
                };
                if field.show_data_as.is_some() || field.ext_lst.is_some() {
                    self.extract_pivot_table_show_values_as(pc, field, &mut data_field);
                }
                opts.data.push(data_field);
            }
        }
    }

    fn extract_pivot_table_show_values_as(
        &self,
        pc: &XlsxPivotCacheDefinition,
        df: &XlsxDataField,
        data_field: &mut PivotTableField,
    ) {
        let order = pc.get_pivot_cache_fields_name();
        let mut show_data_as = df.show_data_as.clone().unwrap_or_default();
        if let Some(ref ext_lst) = df.ext_lst {
            for ext in &ext_lst.ext {
                if ext.uri.as_deref() == Some(EXT_URI_PIVOT_DATA_FIELD) {
                    if let Ok(parsed) =
                        xml_from_reader::<_, XlsxX14DataField>(ext.content.as_bytes())
                    {
                        if let Some(ref psa) = parsed.pivot_show_as {
                            show_data_as = psa.clone();
                        }
                    }
                }
            }
        }
        for (k, v) in show_values_as_map() {
            if v == show_data_as {
                data_field.show_values_as.r#type = k;
                break;
            }
        }
        if let Some(base_field_idx) = df.base_field {
            if base_field_idx < order.len() as i64 {
                data_field.show_values_as.base_field = order[base_field_idx as usize].clone();
            }
        }
        if df.base_item.is_none() {
            return;
        }
        let base_item_idx = df.base_item.unwrap() as usize;
        for cache_field in pc
            .cache_fields
            .as_ref()
            .map(|f| &f.cache_field)
            .into_iter()
            .flatten()
        {
            if cache_field.name == data_field.show_values_as.base_field {
                if let Some(ref shared_items) = cache_field.shared_items {
                    if base_item_idx < shared_items.items.len() {
                        data_field.show_values_as.base_item =
                            shared_items.items[base_item_idx].v().unwrap_or_default();
                    }
                }
            }
        }
    }

    fn gen_pivot_cache_definition_id(&self) -> i32 {
        let mut id = 0i32;
        for entry in self.pkg.iter() {
            let k = entry.key();
            if k.contains("xl/pivotCache/pivotCacheDefinition") {
                if let Ok(pc) = self.pivot_cache_reader(k) {
                    if let Some(ref ext_lst) = pc.ext_lst {
                        for ext in &ext_lst.ext {
                            if ext.uri.as_deref()
                                == Some(crate::constants::EXT_URI_PIVOT_CACHE_DEFINITION)
                            {
                                if let Ok(parsed) =
                                    xml_from_reader::<_, DecodeX14PivotCacheDefinition>(
                                        ext.content.as_bytes(),
                                    )
                                {
                                    if parsed.pivot_cache_id > id {
                                        id = parsed.pivot_cache_id;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        id + 1
    }

    fn delete_workbook_pivot_cache(&self, opt: &PivotTableOptions) -> Result<()> {
        let target = opt
            .pivot_cache_xml
            .trim_start_matches('/')
            .trim_start_matches("xl/");
        let r_id = self.delete_workbook_rels(SOURCE_RELATIONSHIP_PIVOT_CACHE, target)?;
        let mut wb = self.workbook_reader()?;
        if let Some(ref mut caches) = wb.pivot_caches {
            caches
                .pivot_cache
                .retain(|c| c.rid.as_deref() != Some(&r_id));
            if caches.pivot_cache.is_empty() {
                wb.pivot_caches = None;
            }
        }
        *self.workbook.lock().unwrap() = Some(wb);
        Ok(())
    }
}

// ------------------------------------------------------------------
// Extension trait for XlsxPivotCacheDefinition
// ------------------------------------------------------------------

trait PivotCacheExt {
    fn get_pivot_cache_fields_name(&self) -> Vec<String>;
    fn extract_pivot_table_field(&self, data: &str, fld: &XlsxPivotField) -> PivotTableField;
}

impl PivotCacheExt for XlsxPivotCacheDefinition {
    fn get_pivot_cache_fields_name(&self) -> Vec<String> {
        let mut order = Vec::new();
        if let Some(ref cache_fields) = self.cache_fields {
            for cf in &cache_fields.cache_field {
                order.push(cf.name.clone());
            }
        }
        order
    }

    fn extract_pivot_table_field(&self, data: &str, fld: &XlsxPivotField) -> PivotTableField {
        let mut pivot_table_field = PivotTableField {
            data: data.to_string(),
            show_all: fld.show_all.unwrap_or(false),
            insert_blank_row: fld.insert_blank_row.unwrap_or(false),
            ..Default::default()
        };
        if let Some(ref items) = fld.items {
            for item in &items.item {
                if item.h.unwrap_or(false) || item.x.is_none() {
                    continue;
                }
                let idx = item.x.unwrap() as usize;
                if let Some(ref cache_fields) = self.cache_fields {
                    for field in &cache_fields.cache_field {
                        if field.name == data {
                            if let Some(ref shared_items) = field.shared_items {
                                if !shared_items.items.is_empty() && idx < shared_items.items.len()
                                {
                                    let value = shared_items.items[idx].v().unwrap_or_default();
                                    pivot_table_field.selected_items.push(value);
                                }
                            }
                        }
                    }
                }
            }
        }
        pivot_table_field.compact = fld.compact.unwrap_or(false);
        pivot_table_field.outline = fld.outline.unwrap_or(false);
        pivot_table_field.default_subtotal = fld.default_subtotal.unwrap_or(false);
        pivot_table_field
    }
}

// ------------------------------------------------------------------
// Free functions
// ------------------------------------------------------------------

fn in_pivot_table_field(fields: &[PivotTableField], x: &str) -> i32 {
    for (idx, n) in fields.iter().enumerate() {
        if n.data == x {
            return idx as i32;
        }
    }
    -1
}

fn title_case(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().to_string() + chars.as_str(),
        None => String::new(),
    }
}

/// Serialize the extension list that belongs to a data field as raw XML that
/// can be injected into the serialized pivot table definition.
fn serialize_data_field_ext_lst(ext_lst: &XlsxExtLst) -> String {
    format!(
        "<extLst>{}</extLst>",
        crate::xml::common::serialize_ext_lst(ext_lst)
    )
}

/// Inject raw `<extLst>` blocks into self-closing `<dataField/>` tags.
/// `ext_xmls` must be in the same order as the `<dataField/>` tags in `xml`.
fn inject_data_field_ext_lst(xml: &mut Vec<u8>, ext_xmls: &[Option<String>]) {
    let s = String::from_utf8_lossy(xml).to_string();
    let re = Regex::new(r#"<dataField\b[^>]*?/>"#).unwrap();
    let extra = ext_xmls
        .iter()
        .map(|o| o.as_ref().map_or(0, |x| x.len() + 20))
        .sum::<usize>();
    let mut out = String::with_capacity(s.len() + extra);
    let mut last = 0usize;
    let mut idx = 0usize;
    for m in re.find_iter(&s) {
        out.push_str(&s[last..m.start()]);
        let tag = m.as_str();
        if let Some(ext) = ext_xmls.get(idx).and_then(|o| o.as_ref()) {
            out.push_str(&tag[..tag.len() - 2]);
            out.push('>');
            out.push_str(ext);
            out.push_str("</dataField>");
        } else {
            out.push_str(tag);
        }
        last = m.end();
        idx += 1;
    }
    out.push_str(&s[last..]);
    *xml = out.into_bytes();
}

/// Extract the raw inner XML of each `<extLst>` inside a `<dataField>` element
/// and remove the `<extLst>` block so that serde can deserialize the remainder.
/// The returned vector is in data field order.
fn extract_data_field_ext_lst(xml: &str) -> (String, Vec<Option<String>>) {
    let data_field_re = Regex::new(r#"(?s)<dataField\b([^>]*?)(?:/>|>(.*?)</dataField>)"#).unwrap();
    let ext_re = Regex::new(r#"(?s)<extLst>(.*?)</extLst>"#).unwrap();
    let mut out = String::with_capacity(xml.len());
    let mut ext_xmls = Vec::new();
    let mut last = 0usize;
    for cap in data_field_re.captures_iter(xml) {
        let m = cap.get(0).unwrap();
        out.push_str(&xml[last..m.start()]);
        let attrs = cap.get(1).unwrap().as_str();
        if let Some(content_match) = cap.get(2) {
            let content = content_match.as_str();
            let mut ext_opt = None;
            let content_without_ext = if let Some(ext_cap) = ext_re.captures(content) {
                ext_opt = Some(ext_cap.get(1).unwrap().as_str().to_string());
                let full_ext = ext_cap.get(0).unwrap();
                let mut c = String::with_capacity(content.len());
                c.push_str(&content[..full_ext.start()]);
                c.push_str(&content[full_ext.end()..]);
                c
            } else {
                content.to_string()
            };
            ext_xmls.push(ext_opt);
            out.push_str("<dataField");
            out.push_str(attrs);
            out.push('>');
            out.push_str(&content_without_ext);
            out.push_str("</dataField>");
        } else {
            ext_xmls.push(None);
            out.push_str(m.as_str());
        }
        last = m.end();
    }
    out.push_str(&xml[last..]);
    (out, ext_xmls)
}

trait SharedItemValue {
    fn v(&self) -> Option<String>;
}

impl SharedItemValue for XlsxSharedItem {
    fn v(&self) -> Option<String> {
        match self {
            XlsxSharedItem::M(d)
            | XlsxSharedItem::N(d)
            | XlsxSharedItem::B(d)
            | XlsxSharedItem::E(d)
            | XlsxSharedItem::S(d)
            | XlsxSharedItem::D(d) => d.v.clone(),
        }
    }
}

// ------------------------------------------------------------------
// Tests
// ------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::options::Options;
    use crate::slicer::SlicerOptions;
    use crate::xml::table::Table;
    use crate::xml::workbook::DefinedName;

    fn new_file() -> File {
        File::new_with_options(Options::default())
    }

    fn create_sample_data(f: &File) {
        let headers = ["Month", "Year", "Type", "Revenue", "Region"];
        for (i, h) in headers.iter().enumerate() {
            f.set_cell_str("Sheet1", &format!("{}1", (b'A' + i as u8) as char), h)
                .unwrap();
        }
        let months = [
            "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
        ];
        let years = [2017i32, 2018, 2019];
        let types = ["Meat", "Dairy", "Beverages", "Produce"];
        let revenue = [3217, 4512, 3891, 4738, 3054, 4265, 3643, 4901, 3378, 4126];
        let regions = ["East", "West", "North", "South"];
        for row in 2..32 {
            f.set_cell_str(
                "Sheet1",
                &format!("A{row}"),
                months[(row - 2) % months.len()],
            )
            .unwrap();
            f.set_cell_value("Sheet1", &format!("B{row}"), years[(row - 2) % years.len()])
                .unwrap();
            f.set_cell_str("Sheet1", &format!("C{row}"), types[(row - 2) % types.len()])
                .unwrap();
            f.set_cell_value(
                "Sheet1",
                &format!("D{row}"),
                revenue[(row - 2) % revenue.len()],
            )
            .unwrap();
            f.set_cell_str(
                "Sheet1",
                &format!("E{row}"),
                regions[(row - 2) % regions.len()],
            )
            .unwrap();
        }
    }

    #[test]
    fn add_and_get_pivot_table() {
        let f = new_file();
        create_sample_data(&f);
        let mut expected = PivotTableOptions {
            data_range: "Sheet1!A1:E31".to_string(),
            pivot_table_range: "Sheet1!G4:M30".to_string(),
            rows: vec![
                PivotTableField {
                    data: "Month".to_string(),
                    show_all: true,
                    default_subtotal: true,
                    ..Default::default()
                },
                PivotTableField {
                    data: "Year".to_string(),
                    ..Default::default()
                },
            ],
            filter: vec![PivotTableField {
                data: "Region".to_string(),
                ..Default::default()
            }],
            columns: vec![PivotTableField {
                data: "Type".to_string(),
                show_all: true,
                insert_blank_row: true,
                default_subtotal: true,
                ..Default::default()
            }],
            data: vec![PivotTableField {
                data: "Revenue".to_string(),
                subtotal: "Sum".to_string(),
                name: "Summarize".to_string(),
                ..Default::default()
            }],
            row_grand_totals: true,
            col_grand_totals: true,
            show_drill: true,
            classic_layout: true,
            show_error: true,
            show_row_headers: true,
            show_col_headers: true,
            show_last_column: true,
            field_print_titles: true,
            item_print_titles: true,
            pivot_table_style_name: "PivotStyleLight16".to_string(),
            ..Default::default()
        };
        f.add_pivot_table(&mut expected).unwrap();

        // Fields without an explicit selection and without a default subtotal item
        // are read back with a placeholder selected item to match Go's behavior.
        expected.rows[1].selected_items = vec!["".to_string()];

        assert!(f.pkg.contains_key("xl/pivotTables/pivotTable1.xml"));
        assert!(
            f.pkg
                .contains_key("xl/pivotCache/pivotCacheDefinition1.xml")
        );

        let pivot_tables = f.get_pivot_tables("Sheet1").unwrap();
        assert_eq!(pivot_tables.len(), 1);
        assert_eq!(pivot_tables[0], expected);
    }

    #[test]
    fn pivot_table_show_values_as_round_trip() {
        let f = new_file();
        create_sample_data(&f);
        let mut opts = PivotTableOptions {
            data_range: "Sheet1!A1:E31".to_string(),
            pivot_table_range: "Sheet1!W2:AC28".to_string(),
            rows: vec![PivotTableField {
                data: "Month".to_string(),
                default_subtotal: true,
                ..Default::default()
            }],
            columns: vec![PivotTableField {
                data: "Region".to_string(),
                ..Default::default()
            }],
            data: vec![PivotTableField {
                data: "Revenue".to_string(),
                subtotal: "Count".to_string(),
                name: "Summarize by Count".to_string(),
                show_values_as: PivotTableShowValuesAs {
                    r#type: PivotTableShowValuesAsType::PERCENT_OF,
                    base_field: "Region".to_string(),
                    base_item: "East".to_string(),
                },
                ..Default::default()
            }],
            row_grand_totals: true,
            col_grand_totals: true,
            show_drill: true,
            show_row_headers: true,
            show_col_headers: true,
            show_last_column: true,
            ..Default::default()
        };
        f.add_pivot_table(&mut opts).unwrap();

        let pivot_tables = f.get_pivot_tables("Sheet1").unwrap();
        assert_eq!(pivot_tables.len(), 1);
        assert_eq!(pivot_tables[0].data.len(), 1);
        let data = &pivot_tables[0].data[0];
        assert_eq!(data.data, "Revenue");
        assert_eq!(data.subtotal, "Count");
        assert_eq!(
            data.show_values_as.r#type,
            PivotTableShowValuesAsType::PERCENT_OF
        );
        assert_eq!(data.show_values_as.base_field, "Region");
        assert_eq!(data.show_values_as.base_item, "East");
    }

    #[test]
    fn delete_pivot_table() {
        let f = new_file();
        create_sample_data(&f);
        let mut opts = PivotTableOptions {
            data_range: "Sheet1!A1:E31".to_string(),
            pivot_table_range: "Sheet1!G4:M30".to_string(),
            rows: vec![PivotTableField {
                data: "Month".to_string(),
                default_subtotal: true,
                ..Default::default()
            }],
            data: vec![PivotTableField {
                data: "Revenue".to_string(),
                subtotal: "Sum".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        };
        f.add_pivot_table(&mut opts).unwrap();
        assert_eq!(f.get_pivot_tables("Sheet1").unwrap().len(), 1);
        f.delete_pivot_table("Sheet1", "PivotTable1").unwrap();
        assert!(f.get_pivot_tables("Sheet1").unwrap().is_empty());
    }

    #[test]
    fn pivot_table_errors() {
        let f = new_file();
        create_sample_data(&f);

        // Classic layout and compact data conflict.
        let mut opts = PivotTableOptions {
            data_range: "Sheet1!A1:E31".to_string(),
            pivot_table_range: "Sheet1!G2:M34".to_string(),
            compact_data: true,
            classic_layout: true,
            ..Default::default()
        };
        let err = f.add_pivot_table(&mut opts).unwrap_err();
        assert!(err.downcast_ref::<ErrPivotTableClassicLayout>().is_some());

        // Invalid data range.
        let mut opts = PivotTableOptions {
            data_range: "Sheet1!A1:A1".to_string(),
            pivot_table_range: "Sheet1!G2:M34".to_string(),
            rows: vec![PivotTableField {
                data: "Month".to_string(),
                ..Default::default()
            }],
            data: vec![PivotTableField {
                data: "Revenue".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        };
        let err = f.add_pivot_table(&mut opts).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("DataRange"));

        // Same field in filter and rows.
        let mut opts = PivotTableOptions {
            data_range: "Sheet1!A1:E31".to_string(),
            pivot_table_range: "Sheet1!G2:M34".to_string(),
            rows: vec![PivotTableField {
                data: "Month".to_string(),
                default_subtotal: true,
                ..Default::default()
            }],
            columns: vec![PivotTableField {
                data: "Type".to_string(),
                default_subtotal: true,
                ..Default::default()
            }],
            data: vec![PivotTableField {
                data: "Revenue".to_string(),
                ..Default::default()
            }],
            filter: vec![PivotTableField {
                data: "Month".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        };
        let err = f.add_pivot_table(&mut opts).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("row fields"));

        // Selected item does not exist.
        let mut opts = PivotTableOptions {
            data_range: "Sheet1!A1:E31".to_string(),
            pivot_table_range: "Sheet1!G2:M34".to_string(),
            rows: vec![PivotTableField {
                data: "Year".to_string(),
                ..Default::default()
            }],
            columns: vec![PivotTableField {
                data: "Type".to_string(),
                ..Default::default()
            }],
            data: vec![PivotTableField {
                data: "Revenue".to_string(),
                ..Default::default()
            }],
            filter: vec![PivotTableField {
                data: "Month".to_string(),
                selected_items: vec!["x".to_string()],
                ..Default::default()
            }],
            ..Default::default()
        };
        let err = f.add_pivot_table(&mut opts).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("selected item x"));

        // Unsupported show value as type.
        let mut opts = PivotTableOptions {
            data_range: "Sheet1!A1:E31".to_string(),
            pivot_table_range: "Sheet1!G2:M34".to_string(),
            rows: vec![PivotTableField {
                data: "Year".to_string(),
                ..Default::default()
            }],
            columns: vec![PivotTableField {
                data: "Type".to_string(),
                ..Default::default()
            }],
            data: vec![PivotTableField {
                data: "Revenue".to_string(),
                show_values_as: PivotTableShowValuesAs {
                    r#type: PivotTableShowValuesAsType(15),
                    ..Default::default()
                },
                ..Default::default()
            }],
            filter: vec![PivotTableField {
                data: "Month".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        };
        let err = f.add_pivot_table(&mut opts).unwrap_err();
        assert!(
            err.downcast_ref::<ErrUnsupportedPivotTableShowValuesAsType>()
                .is_some()
        );

        // Missing base field for show value as.
        let mut opts = PivotTableOptions {
            data_range: "Sheet1!A1:E31".to_string(),
            pivot_table_range: "Sheet1!G2:M34".to_string(),
            rows: vec![PivotTableField {
                data: "Year".to_string(),
                ..Default::default()
            }],
            columns: vec![PivotTableField {
                data: "Type".to_string(),
                ..Default::default()
            }],
            data: vec![PivotTableField {
                data: "Revenue".to_string(),
                show_values_as: PivotTableShowValuesAs {
                    r#type: PivotTableShowValuesAsType::RUNNING_TOTAL_IN,
                    ..Default::default()
                },
                ..Default::default()
            }],
            filter: vec![PivotTableField {
                data: "Month".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        };
        let err = f.add_pivot_table(&mut opts).unwrap_err();
        assert!(
            err.downcast_ref::<ErrPivotTableShowValuesAsBaseField>()
                .is_some()
        );
    }

    #[test]
    fn pivot_table_comprehensive() {
        let f = new_file();
        create_sample_data(&f);

        let mut expected = PivotTableOptions {
            data_range: "Sheet1!A1:E31".to_string(),
            pivot_table_range: "Sheet1!G4:M30".to_string(),
            rows: vec![
                PivotTableField {
                    data: "Month".to_string(),
                    show_all: true,
                    default_subtotal: true,
                    ..Default::default()
                },
                PivotTableField {
                    data: "Year".to_string(),
                    ..Default::default()
                },
            ],
            filter: vec![PivotTableField {
                data: "Region".to_string(),
                ..Default::default()
            }],
            columns: vec![PivotTableField {
                data: "Type".to_string(),
                show_all: true,
                insert_blank_row: true,
                default_subtotal: true,
                ..Default::default()
            }],
            data: vec![PivotTableField {
                data: "Revenue".to_string(),
                subtotal: "Sum".to_string(),
                name: "Summarize by Sum".to_string(),
                num_fmt: 38,
                ..Default::default()
            }],
            row_grand_totals: true,
            col_grand_totals: true,
            show_drill: true,
            classic_layout: true,
            show_error: true,
            show_row_headers: true,
            show_col_headers: true,
            show_last_column: true,
            field_print_titles: true,
            item_print_titles: true,
            pivot_table_style_name: "PivotStyleLight16".to_string(),
            ..Default::default()
        };
        f.add_pivot_table(&mut expected).unwrap();

        // Fields without an explicit selection and without a default subtotal item
        // are read back with a placeholder selected item to match Go's behavior.
        expected.rows[1].selected_items = vec!["".to_string()];

        let pivot_tables = f.get_pivot_tables("Sheet1").unwrap();
        assert_eq!(pivot_tables.len(), 1);
        assert_eq!(pivot_tables[0], expected);

        // Different coordinate order should be normalized.
        let mut opts = PivotTableOptions {
            data_range: "Sheet1!A1:E31".to_string(),
            pivot_table_range: "Sheet1!U29:O2".to_string(),
            rows: vec![
                PivotTableField {
                    data: "Month".to_string(),
                    default_subtotal: true,
                    ..Default::default()
                },
                PivotTableField {
                    data: "Year".to_string(),
                    ..Default::default()
                },
            ],
            columns: vec![PivotTableField {
                data: "Type".to_string(),
                default_subtotal: true,
                ..Default::default()
            }],
            data: vec![PivotTableField {
                data: "Revenue".to_string(),
                subtotal: "Average".to_string(),
                name: "Summarize by Average".to_string(),
                ..Default::default()
            }],
            row_grand_totals: true,
            col_grand_totals: true,
            show_drill: true,
            show_row_headers: true,
            show_col_headers: true,
            show_last_column: true,
            ..Default::default()
        };
        f.add_pivot_table(&mut opts).unwrap();
        let pivot_tables = f.get_pivot_tables("Sheet1").unwrap();
        assert_eq!(pivot_tables.len(), 2);
        assert_eq!(pivot_tables[1].pivot_table_style_name, "PivotStyleLight16");

        // Show values as with base field and base item.
        let mut opts = PivotTableOptions {
            data_range: "Sheet1!A1:E31".to_string(),
            pivot_table_range: "Sheet1!W2:AC28".to_string(),
            rows: vec![PivotTableField {
                data: "Month".to_string(),
                default_subtotal: true,
                ..Default::default()
            }],
            columns: vec![PivotTableField {
                data: "Region".to_string(),
                ..Default::default()
            }],
            data: vec![PivotTableField {
                data: "Revenue".to_string(),
                subtotal: "Count".to_string(),
                name: "Summarize by Count".to_string(),
                show_values_as: PivotTableShowValuesAs {
                    r#type: PivotTableShowValuesAsType::PERCENT_OF,
                    base_field: "Region".to_string(),
                    base_item: "East".to_string(),
                },
                ..Default::default()
            }],
            row_grand_totals: true,
            col_grand_totals: true,
            show_drill: true,
            show_row_headers: true,
            show_col_headers: true,
            show_last_column: true,
            ..Default::default()
        };
        f.add_pivot_table(&mut opts).unwrap();

        let mut opts = PivotTableOptions {
            data_range: "Sheet1!A1:E31".to_string(),
            pivot_table_range: "Sheet1!G34:X49".to_string(),
            rows: vec![PivotTableField {
                data: "Month".to_string(),
                ..Default::default()
            }],
            columns: vec![
                PivotTableField {
                    data: "Region".to_string(),
                    default_subtotal: true,
                    ..Default::default()
                },
                PivotTableField {
                    data: "Year".to_string(),
                    ..Default::default()
                },
            ],
            data: vec![PivotTableField {
                data: "Revenue".to_string(),
                subtotal: "CountNums".to_string(),
                name: "Summarize by CountNums".to_string(),
                show_values_as: PivotTableShowValuesAs {
                    r#type: PivotTableShowValuesAsType::PERCENT_OF,
                    base_field: "Month".to_string(),
                    base_item: "Jan".to_string(),
                },
                ..Default::default()
            }],
            row_grand_totals: true,
            col_grand_totals: true,
            show_drill: true,
            show_row_headers: true,
            show_col_headers: true,
            show_last_column: true,
            ..Default::default()
        };
        f.add_pivot_table(&mut opts).unwrap();

        let pivot_tables = f.get_pivot_tables("Sheet1").unwrap();
        assert_eq!(pivot_tables.len(), 4);
        assert_eq!(pivot_tables[3].data.len(), 1);
        assert_eq!(
            pivot_tables[3].data[0].show_values_as,
            PivotTableShowValuesAs {
                r#type: PivotTableShowValuesAsType::PERCENT_OF,
                base_field: "Month".to_string(),
                base_item: "Jan".to_string(),
            }
        );

        // x14 show values as types.
        let mut opts = PivotTableOptions {
            data_range: "Sheet1!A1:E31".to_string(),
            pivot_table_range: "Sheet1!AE2:AH28".to_string(),
            rows: vec![
                PivotTableField {
                    data: "Month".to_string(),
                    default_subtotal: true,
                    ..Default::default()
                },
                PivotTableField {
                    data: "Year".to_string(),
                    ..Default::default()
                },
            ],
            data: vec![
                PivotTableField {
                    data: "Revenue".to_string(),
                    subtotal: "Max".to_string(),
                    name: "Summarize by Max".to_string(),
                    show_values_as: PivotTableShowValuesAs {
                        r#type: PivotTableShowValuesAsType::PERCENT_RUNNING_TOTAL_IN,
                        base_field: "Year".to_string(),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                PivotTableField {
                    data: "Revenue".to_string(),
                    subtotal: "Average".to_string(),
                    name: "Average of Sales".to_string(),
                    show_values_as: PivotTableShowValuesAs {
                        r#type: PivotTableShowValuesAsType::RUNNING_TOTAL_IN,
                        base_field: "Year".to_string(),
                        ..Default::default()
                    },
                    ..Default::default()
                },
            ],
            row_grand_totals: true,
            col_grand_totals: true,
            show_drill: true,
            show_row_headers: true,
            show_col_headers: true,
            show_last_column: true,
            ..Default::default()
        };
        f.add_pivot_table(&mut opts).unwrap();

        let pivot_tables = f.get_pivot_tables("Sheet1").unwrap();
        let x14_pt = pivot_tables
            .iter()
            .find(|pt| pt.name == "PivotTable5")
            .unwrap();
        assert_eq!(x14_pt.data.len(), 2);
        assert_eq!(
            x14_pt.data[0].show_values_as.r#type,
            PivotTableShowValuesAsType::PERCENT_RUNNING_TOTAL_IN
        );
        assert_eq!(
            x14_pt.data[1].show_values_as.r#type,
            PivotTableShowValuesAsType::RUNNING_TOTAL_IN
        );

        // Empty subtotal field name and specified style.
        let mut opts = PivotTableOptions {
            data_range: "Sheet1!A1:E31".to_string(),
            pivot_table_range: "Sheet1!AJ4:AK30".to_string(),
            rows: vec![
                PivotTableField {
                    data: "Month".to_string(),
                    default_subtotal: true,
                    ..Default::default()
                },
                PivotTableField {
                    data: "Year".to_string(),
                    ..Default::default()
                },
            ],
            filter: vec![PivotTableField {
                data: "Region".to_string(),
                ..Default::default()
            }],
            columns: vec![],
            data: vec![PivotTableField {
                subtotal: "Sum".to_string(),
                name: "Summarize by Sum".to_string(),
                ..Default::default()
            }],
            row_grand_totals: true,
            col_grand_totals: true,
            show_drill: true,
            show_row_headers: true,
            show_col_headers: true,
            show_last_column: true,
            pivot_table_style_name: "PivotStyleLight19".to_string(),
            ..Default::default()
        };
        f.add_pivot_table(&mut opts).unwrap();

        // Cross-worksheet data range.
        f.new_sheet("Sheet2").unwrap();
        let mut opts = PivotTableOptions {
            data_range: "Sheet1!A1:E31".to_string(),
            pivot_table_range: "Sheet2!A1:AV17".to_string(),
            rows: vec![PivotTableField {
                data: "Month".to_string(),
                ..Default::default()
            }],
            columns: vec![
                PivotTableField {
                    data: "Region".to_string(),
                    default_subtotal: true,
                    ..Default::default()
                },
                PivotTableField {
                    data: "Type".to_string(),
                    default_subtotal: true,
                    ..Default::default()
                },
                PivotTableField {
                    data: "Year".to_string(),
                    ..Default::default()
                },
            ],
            data: vec![PivotTableField {
                data: "Revenue".to_string(),
                subtotal: "Min".to_string(),
                name: "Summarize by Min".to_string(),
                num_fmt: 32,
                ..Default::default()
            }],
            row_grand_totals: true,
            col_grand_totals: true,
            show_drill: true,
            show_row_headers: true,
            show_col_headers: true,
            show_last_column: true,
            ..Default::default()
        };
        f.add_pivot_table(&mut opts).unwrap();
        let pivot_tables = f.get_pivot_tables("Sheet2").unwrap();
        assert_eq!(pivot_tables.len(), 1);
        assert_eq!(pivot_tables[0].data_range, "Sheet1!A1:E31");

        // Selected items in rows, columns and filters.
        let mut opts = PivotTableOptions {
            data_range: "Sheet1!A1:E31".to_string(),
            pivot_table_range: "Sheet1!AM4:AQ12".to_string(),
            rows: vec![PivotTableField {
                data: "Month".to_string(),
                selected_items: vec![
                    "Jan".to_string(),
                    "Feb".to_string(),
                    "Mar".to_string(),
                    "Apr".to_string(),
                    "May".to_string(),
                    "Jun".to_string(),
                    "Jul".to_string(),
                    "Aug".to_string(),
                    "Sep".to_string(),
                    "Oct".to_string(),
                    "Nov".to_string(),
                ],
                ..Default::default()
            }],
            filter: vec![PivotTableField {
                data: "Year".to_string(),
                selected_items: vec!["2017".to_string(), "2018".to_string()],
                ..Default::default()
            }],
            columns: vec![PivotTableField {
                data: "Type".to_string(),
                selected_items: vec![
                    "Meat".to_string(),
                    "Dairy".to_string(),
                    "Beverages".to_string(),
                ],
                ..Default::default()
            }],
            data: vec![PivotTableField {
                data: "Revenue".to_string(),
                subtotal: "Sum".to_string(),
                name: "Summarize by Sum".to_string(),
                ..Default::default()
            }],
            row_grand_totals: true,
            col_grand_totals: true,
            show_drill: true,
            show_error: true,
            show_row_headers: true,
            show_col_headers: true,
            show_last_column: true,
            field_print_titles: true,
            item_print_titles: true,
            pivot_table_style_name: "PivotStyleLight16".to_string(),
            ..Default::default()
        };
        f.add_pivot_table(&mut opts).unwrap();
        let pivot_tables = f.get_pivot_tables("Sheet1").unwrap();
        assert_eq!(pivot_tables.len(), 7);

        f.delete_pivot_table("Sheet1", "PivotTable1").unwrap();
        let pivot_tables = f.get_pivot_tables("Sheet1").unwrap();
        assert_eq!(pivot_tables.len(), 6);
    }

    #[test]
    fn pivot_table_data_range_with_table() {
        let f = new_file();
        let table = Table {
            name: "Table1".to_string(),
            range: "A1:D5".to_string(),
            ..Default::default()
        };
        f.add_table("Sheet1", Some(&table)).unwrap();
        for row in 2..6 {
            f.set_cell_value("Sheet1", &format!("A{row}"), 1).unwrap();
            f.set_cell_value("Sheet1", &format!("B{row}"), 2).unwrap();
            f.set_cell_value("Sheet1", &format!("C{row}"), 3).unwrap();
            f.set_cell_value("Sheet1", &format!("D{row}"), 4).unwrap();
        }

        let mut opts = PivotTableOptions {
            data_range: "Table1".to_string(),
            pivot_table_range: "Sheet1!G2:K7".to_string(),
            rows: vec![PivotTableField {
                data: "Column1".to_string(),
                ..Default::default()
            }],
            columns: vec![PivotTableField {
                data: "Column2".to_string(),
                ..Default::default()
            }],
            row_grand_totals: true,
            col_grand_totals: true,
            show_drill: true,
            show_row_headers: true,
            show_col_headers: true,
            show_last_column: true,
            show_error: true,
            pivot_table_style_name: "PivotStyleLight16".to_string(),
            ..Default::default()
        };
        f.add_pivot_table(&mut opts).unwrap();
        f.delete_pivot_table("Sheet1", "PivotTable1").unwrap();
        let pivot_tables = f.get_pivot_tables("Sheet1").unwrap();
        assert!(pivot_tables.is_empty());
    }

    #[test]
    fn pivot_table_data_range_with_defined_name() {
        let f = new_file();
        create_sample_data(&f);
        f.set_defined_name(&DefinedName {
            name: "dataRange".to_string(),
            refers_to: "Sheet1!A1:E31".to_string(),
            comment: "Pivot Table Data Range".to_string(),
            scope: "Sheet1".to_string(),
        })
        .unwrap();

        let mut opts = PivotTableOptions {
            data_range: "dataRange".to_string(),
            pivot_table_range: "Sheet1!G2:M34".to_string(),
            rows: vec![PivotTableField {
                data: "Month".to_string(),
                default_subtotal: true,
                ..Default::default()
            }],
            data: vec![PivotTableField {
                data: "Revenue".to_string(),
                subtotal: "Sum".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        };
        f.add_pivot_table(&mut opts).unwrap();
        let pivot_tables = f.get_pivot_tables("Sheet1").unwrap();
        assert_eq!(pivot_tables.len(), 1);
        assert_eq!(pivot_tables[0].data_range, "dataRange");
    }

    #[test]
    fn pivot_table_shared_items_mixed_types() {
        let f = new_file();
        for (r, row) in [
            vec!["Type", "Value"],
            vec!["Blank"],
            vec!["Blank"],
            vec!["Integer", "100"],
            vec!["Integer", "100"],
            vec!["Float", "0.01"],
            vec!["Float", "0.01"],
            vec!["Boolean", "true"],
            vec!["Boolean", "true"],
            vec!["String", "text"],
            vec!["String", "text"],
            vec!["Error"],
            vec!["Error"],
            vec!["Formula1"],
            vec!["Formula1"],
            vec!["Formula2"],
            vec!["Formula2"],
            vec!["FormulaError"],
            vec!["FormulaError"],
            vec!["InlineString"],
            vec!["InlineString"],
        ]
        .iter()
        .enumerate()
        {
            for (c, val) in row.iter().enumerate() {
                f.set_cell_str(
                    "Sheet1",
                    &format!("{}{}", (b'A' + c as u8) as char, r + 1),
                    val,
                )
                .unwrap();
            }
        }
        f.set_cell_formula("Sheet1", "B12", "1/0").unwrap();
        f.set_cell_formula("Sheet1", "B13", "1/0").unwrap();
        f.set_cell_formula("Sheet1", "B14", "1+1").unwrap();
        f.set_cell_formula("Sheet1", "B15", "1+1").unwrap();
        f.set_cell_formula("Sheet1", "B16", "_xlfn.TEXTAFTER(\"ab\", \"a\")")
            .unwrap();
        f.set_cell_formula("Sheet1", "B17", "_xlfn.TEXTAFTER(\"ab\", \"a\")")
            .unwrap();

        let selected_items = vec![
            "".to_string(),
            "100".to_string(),
            "0.01".to_string(),
            "true".to_string(),
            "text".to_string(),
            "#DIV/0!".to_string(),
            "2".to_string(),
        ];
        let mut opts = PivotTableOptions {
            data_range: "Sheet1!A1:B21".to_string(),
            pivot_table_range: "Sheet1!D4:E12".to_string(),
            rows: vec![PivotTableField {
                data: "Type".to_string(),
                ..Default::default()
            }],
            data: vec![PivotTableField {
                data: "Type".to_string(),
                subtotal: "Count".to_string(),
                name: "Count of Type".to_string(),
                ..Default::default()
            }],
            filter: vec![PivotTableField {
                data: "Value".to_string(),
                selected_items: selected_items.clone(),
                ..Default::default()
            }],
            row_grand_totals: true,
            col_grand_totals: true,
            show_drill: true,
            show_row_headers: true,
            show_col_headers: true,
            show_last_column: true,
            show_error: true,
            field_print_titles: true,
            item_print_titles: true,
            pivot_table_style_name: "PivotStyleLight16".to_string(),
            ..Default::default()
        };
        f.add_pivot_table(&mut opts).unwrap();
        let pivot_tables = f.get_pivot_tables("Sheet1").unwrap();
        assert_eq!(pivot_tables.len(), 1);
        assert_eq!(pivot_tables[0].filter[0].selected_items, selected_items);

        f.add_slicer(
            "Sheet1",
            &SlicerOptions {
                name: "Value".to_string(),
                cell: "G2".to_string(),
                table_sheet: "Sheet1".to_string(),
                table_name: "PivotTable1".to_string(),
                caption: "Value".to_string(),
                selected_items: vec!["true".to_string()],
                ..Default::default()
            },
        )
        .unwrap();
    }
}
