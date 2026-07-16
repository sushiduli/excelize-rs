//! Slicer support.
//!
//! Ported from Go `slicer.go`.

use std::collections::HashMap;

use quick_xml::de::from_reader as xml_from_reader;
use quick_xml::se::to_string as xml_to_string;

use crate::constants::{
    CONTENT_TYPE_SLICER, CONTENT_TYPE_SLICER_CACHE, DEFAULT_SLICER_HEIGHT, DEFAULT_SLICER_WIDTH,
    EMU, EXT_URI_PIVOT_CACHE_DEFINITION, EXT_URI_SLICER_CACHE_DEFINITION,
    EXT_URI_SLICER_CACHES_X14, EXT_URI_SLICER_CACHES_X15, EXT_URI_SLICER_LIST_X14,
    EXT_URI_SLICER_LIST_X15, NAMESPACE_DRAWING_ML_A14, NAMESPACE_DRAWING_ML_SLICER,
    NAMESPACE_DRAWING_ML_SLICER_X15, NAMESPACE_SPREADSHEET, NAMESPACE_SPREADSHEET_X14,
    NAMESPACE_SPREADSHEET_X15, NAMESPACE_SPREADSHEET_XR10, SOURCE_RELATIONSHIP,
    SOURCE_RELATIONSHIP_SLICER, SOURCE_RELATIONSHIP_SLICER_CACHE, WORKBOOK_EXT_URI_PRIORITY,
    WORKSHEET_EXT_URI_PRIORITY,
};
use crate::errors::Result;
use crate::errors::{
    ErrParameterInvalid, new_invalid_slicer_name_error, new_no_exist_slicer_error,
    new_no_exist_table_error, new_pivot_table_selected_item_error,
};
use crate::lib_util::{cell_name_to_coordinates, coordinates_to_cell_name, in_str_slice};
use crate::xml::common::{AttrValString, XlsxExt, XlsxExtLst, XlsxInnerXml};
use crate::xml::decode_drawing::DecodeChoice;
use crate::xml::drawing::{
    ABodyPr, ALn, AP, AR, ASolidFill, ASrgbClr, GraphicOptions, XdrCNvSpPr, XdrCellAnchor,
    XdrClientData, XdrNvSpPr, XdrSp, XdrTxBody, XlsxAlternateContent, XlsxCNvPr, XlsxFrom,
    XlsxGraphic, XlsxGraphicData, XlsxGraphicFrame, XlsxNvGraphicFramePr, XlsxOff,
    XlsxPositiveSize2D, XlsxPrstGeom, XlsxSle, XlsxSpPr, XlsxTo, XlsxWsDr, XlsxXfrm,
};
use crate::xml::pivot_cache::{
    DecodeX14PivotCacheDefinition, XlsxPivotCacheDefinition, XlsxSharedItem, XlsxSharedItems,
    XlsxX14PivotCacheDefinition,
};
use crate::xml::slicers::{
    DecodeSlicerCaches, DecodeSlicerList, DecodeTableSlicerCache, XlsxSlicer as XlsxSlicerEl,
    XlsxSlicerCacheData, XlsxSlicerCacheDefinition, XlsxSlicerCachePivotTable,
    XlsxSlicerCachePivotTables, XlsxSlicers, XlsxTableSlicerCache, XlsxTabularSlicerCache,
    XlsxTabularSlicerCacheItem, XlsxTabularSlicerCacheItems, XlsxTimelines, XlsxX14Slicer,
    XlsxX14SlicerCache, XlsxX14SlicerCaches, XlsxX14SlicerList, XlsxX15SlicerCaches,
};

use crate::File;
use crate::xml::table::Table;
use crate::xml::workbook::DefinedName;

/// Slicer creation and read-back options.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct SlicerOptions {
    pub(crate) slicer_xml: String,
    pub(crate) slicer_cache_xml: String,
    pub(crate) slicer_cache_name: String,
    pub(crate) slicer_sheet_name: String,
    pub(crate) slicer_sheet_rid: String,
    pub(crate) drawing_xml: String,

    /// Name specifies the slicer name, should be an existing field name of the
    /// given table or pivot table.
    pub name: String,
    /// Cell specifies the left top cell coordinates for inserting the slicer.
    pub cell: String,
    /// TableSheet specifies the worksheet name of the table or pivot table.
    pub table_sheet: String,
    /// TableName specifies the name of the table or pivot table.
    pub table_name: String,
    /// Caption specifies the caption of the slicer.
    pub caption: String,
    /// Macro used for set macro for the slicer.
    pub macro_name: String,
    /// Width specifies the width of the slicer.
    pub width: i32,
    /// Height specifies the height of the slicer.
    pub height: i32,
    /// DisplayHeader specifies if display header of the slicer.
    pub display_header: Option<bool>,
    /// ItemDesc specifies descending (Z-A) item sorting.
    pub item_desc: bool,
    /// Format specifies the format of the slicer.
    pub format: GraphicOptions,
    /// SelectedItems option is used to specify the default selected items.
    pub selected_items: Vec<String>,
}

impl File {
    /// AddSlicer inserts a slicer by giving the worksheet name and slicer
    /// settings.
    pub fn add_slicer(&self, sheet: &str, opts: &SlicerOptions) -> Result<()> {
        let mut opts = parse_slicer_options(opts)?;
        let (table, pivot_table, col_idx) = self.get_slicer_source(&opts)?;
        let (ext_uri, ns) = if table.is_some() {
            (EXT_URI_SLICER_LIST_X15, NAMESPACE_DRAWING_ML_SLICER_X15)
        } else {
            (EXT_URI_SLICER_LIST_X14, NAMESPACE_DRAWING_ML_A14)
        };
        let slicer_id = self.add_sheet_slicer(sheet, ext_uri)?;
        let slicer_cache_name =
            self.set_slicer_cache(col_idx, &mut opts, table.as_ref(), pivot_table.as_ref())?;
        let slicer_name = self.gen_slicer_name(&opts.name);
        self.add_drawing_slicer(sheet, &slicer_name, ns, &opts)?;
        self.add_slicer_part(
            slicer_id,
            XlsxSlicerEl {
                name: slicer_name,
                cache: slicer_cache_name,
                caption: Some(opts.caption.clone()).filter(|s| !s.is_empty()),
                show_caption: opts.display_header,
                row_height: 251883,
                ..Default::default()
            },
        )
    }

    /// GetSlicers provides all slicers in a worksheet by a given worksheet name.
    pub fn get_slicers(&self, sheet: &str) -> Result<Vec<SlicerOptions>> {
        let mut slicers = Vec::new();
        let ws = self.work_sheet_reader(sheet)?;
        if ws.ext_lst.is_none() {
            return Ok(slicers);
        }
        let drawing_xml = if let Some(drawing) = &ws.drawing {
            let target = self
                .get_sheet_relationships_target_by_id(sheet, drawing.rid.as_deref().unwrap_or(""));
            target
                .replace("..", "xl")
                .trim_start_matches('/')
                .to_string()
        } else {
            String::new()
        };
        for ext in &ws.ext_lst.as_ref().unwrap().ext {
            if ext.uri.as_deref() == Some(EXT_URI_SLICER_LIST_X14)
                || ext.uri.as_deref() == Some(EXT_URI_SLICER_LIST_X15)
            {
                let slicer_list: DecodeSlicerList =
                    xml_from_reader(wrap_ext_content(&ext.content).as_bytes())?;
                for slicer in slicer_list.slicer {
                    if !slicer.rid.is_empty() {
                        let mut opts =
                            self.get_slicers_internal(sheet, &slicer.rid, &drawing_xml)?;
                        slicers.append(&mut opts);
                    }
                }
            }
        }
        Ok(slicers)
    }

    /// DeleteSlicer deletes a slicer by a given slicer name.
    pub fn delete_slicer(&self, name: &str) -> Result<()> {
        let all = self.get_all_slicers()?;
        for (_, slicers) in &all {
            for slicer in slicers {
                if slicer.name != name {
                    continue;
                }
                self.delete_slicer_internal(slicer)?;
                return self.delete_slicer_cache(&all, slicer);
            }
        }
        Err(new_no_exist_slicer_error(name).into())
    }
}

// ------------------------------------------------------------------
// Option parsing
// ------------------------------------------------------------------

fn parse_slicer_options(opts: &SlicerOptions) -> Result<SlicerOptions> {
    if opts.name.is_empty()
        || opts.cell.is_empty()
        || opts.table_sheet.is_empty()
        || opts.table_name.is_empty()
    {
        return Err(Box::new(ErrParameterInvalid));
    }
    let mut out = opts.clone();
    if out.width == 0 {
        out.width = DEFAULT_SLICER_WIDTH;
    }
    if out.height == 0 {
        out.height = DEFAULT_SLICER_HEIGHT;
    }
    out.format = crate::chart::parse_graphic_options(&out.format)?;
    Ok(out)
}

// ------------------------------------------------------------------
// Slicer source resolution
// ------------------------------------------------------------------

impl File {
    fn get_slicer_source(
        &self,
        opts: &SlicerOptions,
    ) -> Result<(
        Option<Table>,
        Option<crate::pivot_table::PivotTableOptions>,
        i32,
    )> {
        let mut table: Option<Table> = None;
        let mut pivot_table: Option<crate::pivot_table::PivotTableOptions> = None;
        let mut data_range = String::new();

        let tables = self.get_tables(&opts.table_sheet)?;
        for tbl in tables {
            if tbl.name == opts.table_name {
                table = Some(tbl.clone());
                data_range = format!("{}!{}", opts.table_sheet, tbl.range);
                break;
            }
        }

        if table.is_none() {
            let pivot_tables = self.get_pivot_tables(&opts.table_sheet)?;
            for pt in pivot_tables {
                if pt.data_range.is_empty() {
                    continue;
                }
                if let Some(ref name) = self.find_pivot_table_name(&pt) {
                    if name == &opts.table_name {
                        pivot_table = Some(pt.clone());
                        data_range = pt.data_range.clone();
                        break;
                    }
                }
            }
            if pivot_table.is_none() {
                return Err(new_no_exist_table_error(&opts.table_name).into());
            }
        }

        let order = self.get_slicer_table_fields_order(&data_range)?;
        let col_idx = in_str_slice(&order, &opts.name, true);
        if col_idx == -1 {
            return Err(new_invalid_slicer_name_error(&opts.name).into());
        }
        Ok((table, pivot_table, col_idx))
    }

    fn find_pivot_table_name(&self, pt: &crate::pivot_table::PivotTableOptions) -> Option<String> {
        if pt.name.is_empty() {
            None
        } else {
            Some(pt.name.clone())
        }
    }

    fn get_slicer_table_fields_order(&self, data_range: &str) -> Result<Vec<String>> {
        let (sheet, range) = split_data_range(data_range);
        let coords = crate::lib_util::range_ref_to_coordinates(&range)?;
        let mut order = Vec::new();
        for col in coords[0]..=coords[2] {
            let cell = coordinates_to_cell_name(col, coords[1], false)?;
            let name = self.get_cell_value(&sheet, &cell)?;
            if name.is_empty() {
                return Err(Box::new(ErrParameterInvalid));
            }
            order.push(name);
        }
        Ok(order)
    }
}

fn split_data_range(data_range: &str) -> (String, String) {
    if let Some(pos) = data_range.find('!') {
        (
            data_range[..pos].to_string(),
            data_range[pos + 1..].to_string(),
        )
    } else {
        (String::new(), data_range.to_string())
    }
}

// ------------------------------------------------------------------
// Worksheet slicer list
// ------------------------------------------------------------------

impl File {
    fn add_sheet_slicer(&self, sheet: &str, ext_uri: &str) -> Result<i32> {
        let mut slicer_id = self.count_slicers() + 1;
        let ws = self.work_sheet_reader(sheet)?;
        if let Some(ext_lst) = &ws.ext_lst {
            for ext in &ext_lst.ext {
                if ext.uri.as_deref() == Some(ext_uri) {
                    let slicer_list: DecodeSlicerList =
                        xml_from_reader(wrap_ext_content(&ext.content).as_bytes())?;
                    for slicer in slicer_list.slicer {
                        if !slicer.rid.is_empty() {
                            let target =
                                self.get_sheet_relationships_target_by_id(sheet, &slicer.rid);
                            if let Some(id_str) = target
                                .strip_prefix("../slicers/slicer")
                                .and_then(|s| s.strip_suffix(".xml"))
                            {
                                slicer_id = id_str.parse::<i32>().unwrap_or(slicer_id);
                            }
                            return Ok(slicer_id);
                        }
                    }
                }
            }
        }

        let sheet_xml_path = self.get_sheet_xml_path(sheet).unwrap_or_default();
        let sheet_rels = format!(
            "xl/worksheets/_rels/{}.rels",
            sheet_xml_path.trim_start_matches("xl/worksheets/")
        );
        let sheet_relationships_slicer_xml = format!("../slicers/slicer{slicer_id}.xml");
        let r_id = self.add_rels(
            &sheet_rels,
            SOURCE_RELATIONSHIP_SLICER,
            &sheet_relationships_slicer_xml,
            "",
        );
        self.add_sheet_name_space(sheet, NAMESPACE_SPREADSHEET_X14);
        self.add_sheet_table_slicer(sheet, r_id, ext_uri)?;
        Ok(slicer_id)
    }

    fn add_sheet_table_slicer(&self, sheet: &str, r_id: i32, ext_uri: &str) -> Result<()> {
        let mut ws = self.work_sheet_reader(sheet)?;
        if ws.ext_lst.is_none() {
            ws.ext_lst = Some(XlsxExtLst::default());
        }
        let ext_lst = ws.ext_lst.as_mut().unwrap();

        let slicer_list_bytes = xml_to_string(&XlsxX14SlicerList {
            slicer: vec![XlsxX14Slicer {
                rid: format!("rId{r_id}"),
            }],
        })?;

        let ns_attr = if ext_uri == EXT_URI_SLICER_LIST_X15 {
            NAMESPACE_SPREADSHEET_X15
        } else {
            NAMESPACE_SPREADSHEET_X14
        };

        ext_lst.ext.push(XlsxExt {
            uri: Some(ext_uri.to_string()),
            content: format!(
                r#"<x14:slicerList xmlns:x14="{}">{}</x14:slicerList>"#,
                ns_attr,
                extract_inner(&slicer_list_bytes)
            ),
            ..Default::default()
        });
        sort_ext_lst(&mut ext_lst.ext, WORKSHEET_EXT_URI_PRIORITY);
        if let Some(path) = self.get_sheet_xml_path(sheet) {
            self.sheet.insert(path, ws);
        }
        Ok(())
    }
}

// ------------------------------------------------------------------
// Slicer part
// ------------------------------------------------------------------

impl File {
    fn add_slicer_part(&self, slicer_id: i32, slicer: XlsxSlicerEl) -> Result<()> {
        let slicer_xml = format!("xl/slicers/slicer{slicer_id}.xml");
        let mut slicers = self.slicer_reader(&slicer_xml)?;
        self.add_content_type_part(slicer_id, "slicer")?;
        slicers.slicer.push(slicer);
        let output = xml_to_string(&slicers)?;
        self.save_file_list(&slicer_xml, output.as_bytes());
        Ok(())
    }

    fn slicer_reader(&self, slicer_xml: &str) -> Result<XlsxSlicers> {
        let content = self.read_xml(slicer_xml);
        let mut slicers = XlsxSlicers {
            xmlns_mc: Some(SOURCE_RELATIONSHIP.to_string()),
            xmlns_x: Some(NAMESPACE_SPREADSHEET.to_string()),
            xmlns_xr10: Some(NAMESPACE_SPREADSHEET_XR10.to_string()),
            ..Default::default()
        };
        if !content.is_empty() {
            slicers = xml_from_reader(
                crate::file::namespace_strict_to_transitional(&content).as_slice(),
            )?;
        }
        Ok(slicers)
    }

    fn slicer_cache_reader(&self, slicer_cache_xml: &str) -> Result<XlsxSlicerCacheDefinition> {
        let content = self.read_xml(slicer_cache_xml);
        let mut slicer_cache = XlsxSlicerCacheDefinition::default();
        if !content.is_empty() {
            slicer_cache = xml_from_reader(
                crate::file::namespace_strict_to_transitional(&content).as_slice(),
            )?;
        }
        Ok(slicer_cache)
    }

    fn timeline_reader(&self, timeline_xml: &str) -> Result<XlsxTimelines> {
        let content = self.read_xml(timeline_xml);
        let mut timelines = XlsxTimelines {
            xmlns_mc: Some(SOURCE_RELATIONSHIP.to_string()),
            xmlns_x: Some(NAMESPACE_SPREADSHEET.to_string()),
            xmlns_xr10: Some(NAMESPACE_SPREADSHEET_XR10.to_string()),
            ..Default::default()
        };
        if !content.is_empty() {
            timelines = xml_from_reader(
                crate::file::namespace_strict_to_transitional(&content).as_slice(),
            )?;
        }
        Ok(timelines)
    }
}

// ------------------------------------------------------------------
// Name generation
// ------------------------------------------------------------------

impl File {
    fn gen_slicer_name(&self, name: &str) -> String {
        let mut names: Vec<String> = Vec::new();
        for entry in self.pkg.iter() {
            let k = entry.key();
            if k.contains("xl/slicers/slicer") {
                if let Ok(slicers) = self.slicer_reader(k) {
                    for slicer in slicers.slicer {
                        names.push(slicer.name);
                    }
                }
            }
            if k.contains("xl/timelines/timeline") {
                if let Ok(timelines) = self.timeline_reader(k) {
                    for timeline in timelines.timeline {
                        names.push(timeline.name);
                    }
                }
            }
        }

        let mut cnt = 0;
        let slicer_name = name.to_string();
        loop {
            let tmp = if cnt > 0 {
                format!("{slicer_name} {cnt}")
            } else {
                slicer_name.clone()
            };
            if in_str_slice(&names, &tmp, true) == -1 {
                return tmp;
            }
            cnt += 1;
        }
    }

    fn gen_slicer_cache_name(&self, name: &str) -> String {
        let mut defined_names: Vec<String> = Vec::new();
        if let Ok(dns) = self.get_defined_names() {
            for dn in dns {
                if dn.scope == "Workbook" {
                    defined_names.push(dn.name);
                }
            }
        }

        let mut slicer_cache_name = String::new();
        for (i, c) in name.chars().enumerate() {
            if c.is_alphabetic() {
                slicer_cache_name.push(c);
            } else if i > 0 && (c.is_ascii_digit() || c == '.') {
                slicer_cache_name.push(c);
            } else {
                slicer_cache_name.push('_');
            }
        }
        slicer_cache_name = format!("Slicer_{slicer_cache_name}");

        let mut cnt = 0;
        loop {
            let tmp = if cnt > 0 {
                format!("{slicer_cache_name}{cnt}")
            } else {
                slicer_cache_name.clone()
            };
            if in_str_slice(&defined_names, &tmp, true) == -1 {
                return tmp;
            }
            cnt += 1;
        }
    }
}

// ------------------------------------------------------------------
// Slicer cache
// ------------------------------------------------------------------

impl File {
    fn set_slicer_cache(
        &self,
        col_idx: i32,
        opts: &mut SlicerOptions,
        table: Option<&Table>,
        pivot_table: Option<&crate::pivot_table::PivotTableOptions>,
    ) -> Result<String> {
        let mut ok = false;
        let mut slicer_cache_name = String::new();

        for entry in self.pkg.iter() {
            let k = entry.key();
            if !k.contains("xl/slicerCaches/slicerCache") {
                continue;
            }
            let slicer_cache = self.slicer_cache_reader(k)?;
            if let Some(ref pts) = slicer_cache.pivot_tables {
                if let Some(ref pt) = pivot_table {
                    for tbl in &pts.pivot_table {
                        if tbl.name == pt.data_range || tbl.name == opts.table_name {
                            ok = true;
                            slicer_cache_name = slicer_cache.name.clone();
                            break;
                        }
                    }
                }
            }
            if ok {
                break;
            }
            if table.is_none() || slicer_cache.ext_lst.is_none() {
                continue;
            }
            let ext = decode_first_ext(&slicer_cache.ext_lst.as_ref().unwrap().ext)?;
            if ext.uri.as_deref() == Some(EXT_URI_SLICER_CACHE_DEFINITION) {
                let tsc: DecodeTableSlicerCache =
                    xml_from_reader(wrap_ext_content(&ext.content).as_bytes())?;
                if let Some(tbl) = table {
                    if tsc.table_id == tbl.t_id && tsc.column == col_idx as i64 + 1 {
                        ok = true;
                        slicer_cache_name = slicer_cache.name.clone();
                        break;
                    }
                }
            }
        }

        if ok {
            return Ok(slicer_cache_name);
        }
        let slicer_cache_name = self.gen_slicer_cache_name(&opts.name);
        self.add_slicer_cache(&slicer_cache_name, col_idx, opts, table, pivot_table)?;
        Ok(slicer_cache_name)
    }

    fn add_slicer_cache(
        &self,
        slicer_cache_name: &str,
        col_idx: i32,
        opts: &SlicerOptions,
        table: Option<&Table>,
        pivot_table: Option<&crate::pivot_table::PivotTableOptions>,
    ) -> Result<()> {
        let mut sort_order = String::new();
        let mut ext_uri = EXT_URI_SLICER_CACHES_X14;
        let slicer_cache_id = self.count_slicer_cache() + 1;

        if opts.item_desc {
            sort_order = "descending".to_string();
        }

        let mut slicer_cache = XlsxSlicerCacheDefinition {
            xmlns_mc: Some(SOURCE_RELATIONSHIP.to_string()),
            xmlns_x: Some(NAMESPACE_SPREADSHEET.to_string()),
            xmlns_x15: Some(NAMESPACE_SPREADSHEET_X15.to_string()),
            xmlns_xr10: Some(NAMESPACE_SPREADSHEET_XR10.to_string()),
            name: slicer_cache_name.to_string(),
            source_name: opts.name.clone(),
            ..Default::default()
        };

        if let Some(pt) = pivot_table {
            let pivot_cache_id = self.add_pivot_cache_slicer(pt)?;
            slicer_cache.pivot_tables = Some(XlsxSlicerCachePivotTables {
                pivot_table: vec![XlsxSlicerCachePivotTable {
                    tab_id: self.get_sheet_id(&opts.table_sheet) as i64,
                    name: opts.table_name.clone(),
                }],
            });
            let items = self.build_slicer_items(pt, opts)?;
            slicer_cache.data = Some(XlsxSlicerCacheData {
                tabular: Some(XlsxTabularSlicerCache {
                    pivot_cache_id: pivot_cache_id as i64,
                    sort_order: Some(sort_order.clone()).filter(|s| !s.is_empty()),
                    show_missing: Some(false),
                    items: Some(XlsxTabularSlicerCacheItems {
                        count: Some(items.len() as i64),
                        i: items,
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            });
        }

        if let Some(tbl) = table {
            let table_slicer_bytes = xml_to_string(&XlsxTableSlicerCache {
                table_id: tbl.t_id,
                column: col_idx as i64 + 1,
                sort_order: Some(sort_order.clone()).filter(|s| !s.is_empty()),
                ..Default::default()
            })?;
            slicer_cache.ext_lst = Some(XlsxExtLst {
                ext: vec![XlsxExt {
                    uri: Some(EXT_URI_SLICER_CACHE_DEFINITION.to_string()),
                    content: format!(
                        r#"<x15:tableSlicerCache xmlns:x15="{}">{}</x15:tableSlicerCache>"#,
                        NAMESPACE_SPREADSHEET_X15,
                        extract_inner(&table_slicer_bytes)
                    ),
                    ..Default::default()
                }],
            });
            ext_uri = EXT_URI_SLICER_CACHES_X15;
        }

        let slicer_cache_xml = format!("xl/slicerCaches/slicerCache{slicer_cache_id}.xml");
        let slicer_cache_bytes = xml_to_string(&slicer_cache)?;
        self.save_file_list(&slicer_cache_xml, slicer_cache_bytes.as_bytes());
        self.add_content_type_part(slicer_cache_id, "slicerCache")?;
        self.add_workbook_slicer_cache(slicer_cache_id, ext_uri)?;
        self.set_defined_name(&DefinedName {
            name: slicer_cache_name.to_string(),
            refers_to: "#N/A".to_string(),
            scope: "Workbook".to_string(),
            ..Default::default()
        })
    }

    fn build_slicer_items(
        &self,
        pivot_table: &crate::pivot_table::PivotTableOptions,
        opts: &SlicerOptions,
    ) -> Result<Vec<XlsxTabularSlicerCacheItem>> {
        let mut items: Vec<XlsxTabularSlicerCacheItem> = Vec::new();
        let pivot_cache_xml = self.find_pivot_cache_xml(pivot_table)?;
        let pc = self.slicer_pivot_cache_reader(&pivot_cache_xml)?;
        let mut shared_items: Option<&XlsxSharedItems> = None;
        if let Some(ref fields) = pc.cache_fields {
            for field in &fields.cache_field {
                if field.name == opts.name {
                    shared_items = field.shared_items.as_ref();
                    break;
                }
            }
        }
        let shared_items = match shared_items {
            Some(si) => si,
            None => {
                return Ok(vec![XlsxTabularSlicerCacheItem {
                    s: Some(true),
                    ..Default::default()
                }]);
            }
        };

        let mut i = 0;
        for item in &shared_items.items {
            let (variant, v) = match item {
                XlsxSharedItem::M(d) => ("m", d.v.clone().unwrap_or_default()),
                XlsxSharedItem::B(d) => ("b", d.v.clone().unwrap_or_default()),
                XlsxSharedItem::N(d) => ("n", d.v.clone().unwrap_or_default()),
                XlsxSharedItem::E(d) => ("e", d.v.clone().unwrap_or_default()),
                XlsxSharedItem::S(d) => ("s", d.v.clone().unwrap_or_default()),
                XlsxSharedItem::D(d) => ("d", d.v.clone().unwrap_or_default()),
            };
            let selected = match variant {
                "m" => in_str_slice(&opts.selected_items, "", true) != -1,
                "b" => in_str_slice(&opts.selected_items, &v, false) != -1,
                _ => in_str_slice(&opts.selected_items, &v, true) != -1,
            };
            items.push(XlsxTabularSlicerCacheItem {
                x: i,
                s: Some(selected),
                nd: None,
            });
            i += 1;
        }
        check_selected_items(shared_items, &opts.name, &opts.selected_items)?;
        if items.is_empty() {
            items.push(XlsxTabularSlicerCacheItem {
                s: Some(true),
                ..Default::default()
            });
        }
        Ok(items)
    }

    fn find_pivot_cache_xml(
        &self,
        _pivot_table: &crate::pivot_table::PivotTableOptions,
    ) -> Result<String> {
        for entry in self.pkg.iter() {
            let k = entry.key();
            if k.contains("xl/pivotCache/pivotCacheDefinition") {
                return Ok(k.clone());
            }
        }
        Err(Box::new(ErrParameterInvalid))
    }

    fn slicer_pivot_cache_reader(&self, pivot_cache_xml: &str) -> Result<XlsxPivotCacheDefinition> {
        let content = self.read_xml(pivot_cache_xml);
        let mut pc = XlsxPivotCacheDefinition::default();
        if !content.is_empty() {
            pc = xml_from_reader(
                crate::file::namespace_strict_to_transitional(&content).as_slice(),
            )?;
        }
        Ok(pc)
    }

    fn gen_slicer_pivot_cache_id(&self) -> i32 {
        let mut id = 0;
        for entry in self.pkg.iter() {
            let k = entry.key();
            if !k.contains("xl/pivotCache/pivotCacheDefinition") {
                continue;
            }
            if let Ok(pc) = self.slicer_pivot_cache_reader(k) {
                if let Some(ref ext_lst) = pc.ext_lst {
                    let decoded = decode_ext_lst(&ext_lst.ext).unwrap_or_default();
                    for ext in decoded {
                        if ext.uri.as_deref() == Some(EXT_URI_PIVOT_CACHE_DEFINITION) {
                            let def: crate::xml::pivot_cache::DecodeX14PivotCacheDefinition =
                                xml_from_reader(wrap_ext_content(&ext.content).as_bytes())
                                    .unwrap_or_default();
                            if id < def.pivot_cache_id {
                                id = def.pivot_cache_id;
                            }
                        }
                    }
                }
            }
        }
        id + 1
    }

    fn add_pivot_cache_slicer(
        &self,
        _pivot_table: &crate::pivot_table::PivotTableOptions,
    ) -> Result<i32> {
        let pivot_cache_xml = self.find_first_pivot_cache_xml()?;
        let mut pc = self.slicer_pivot_cache_reader(&pivot_cache_xml)?;
        let mut decode_ext_lst = XlsxExtLst::default();
        if let Some(ref ext_lst) = pc.ext_lst {
            for ext in &ext_lst.ext {
                if ext.uri.as_deref() == Some(EXT_URI_PIVOT_CACHE_DEFINITION) {
                    let def: DecodeX14PivotCacheDefinition =
                        xml_from_reader(wrap_ext_content(&ext.content).as_bytes())?;
                    return Ok(def.pivot_cache_id);
                }
            }
        }
        let pivot_cache_id = self.gen_slicer_pivot_cache_id();
        let pivot_cache_bytes = xml_to_string(&XlsxX14PivotCacheDefinition {
            xmlns: Some(NAMESPACE_SPREADSHEET_X14.to_string()),
            pivot_cache_id,
        })?;
        decode_ext_lst.ext.push(XlsxExt {
            uri: Some(EXT_URI_PIVOT_CACHE_DEFINITION.to_string()),
            content: format!(
                r#"<x14:pivotCacheDefinition xmlns:x14="{}">{}</x14:pivotCacheDefinition>"#,
                NAMESPACE_SPREADSHEET_X14,
                extract_inner(&pivot_cache_bytes)
            ),
            ..Default::default()
        });
        pc.ext_lst = Some(decode_ext_lst);
        let pivot_cache = xml_to_string(&pc)?;
        self.save_file_list(&pivot_cache_xml, pivot_cache.as_bytes());
        Ok(pivot_cache_id)
    }

    fn find_first_pivot_cache_xml(&self) -> Result<String> {
        for entry in self.pkg.iter() {
            let k = entry.key();
            if k.contains("xl/pivotCache/pivotCacheDefinition") {
                return Ok(k.clone());
            }
        }
        Err(Box::new(ErrParameterInvalid))
    }
}

fn check_selected_items(
    si: &XlsxSharedItems,
    field: &str,
    selected_items: &[String],
) -> Result<()> {
    for shared_item in selected_items {
        let mut found = false;
        for item in &si.items {
            let (variant, v) = match item {
                XlsxSharedItem::M(d) => ("m", d.v.clone().unwrap_or_default()),
                XlsxSharedItem::B(d) => ("b", d.v.clone().unwrap_or_default()),
                XlsxSharedItem::N(d) => ("n", d.v.clone().unwrap_or_default()),
                XlsxSharedItem::E(d) => ("e", d.v.clone().unwrap_or_default()),
                XlsxSharedItem::S(d) => ("s", d.v.clone().unwrap_or_default()),
                XlsxSharedItem::D(d) => ("d", d.v.clone().unwrap_or_default()),
            };
            found = match variant {
                "m" => shared_item.is_empty(),
                "b" => shared_item.eq_ignore_ascii_case(&v),
                _ => shared_item == &v,
            };
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

// ------------------------------------------------------------------
// Drawing slicer shape
// ------------------------------------------------------------------

impl File {
    fn add_drawing_slicer(
        &self,
        sheet: &str,
        slicer_name: &str,
        ns: &str,
        opts: &SlicerOptions,
    ) -> Result<()> {
        let drawing_id = self.count_drawings() + 1;
        let drawing_xml = format!("xl/drawings/drawing{drawing_id}.xml");
        let (drawing_id, drawing_xml) = self.prepare_drawing(sheet, drawing_id, &drawing_xml)?;
        let (mut content, mut cell_anchor, c_nv_pr_id) = self.cell_anchor_shape(
            sheet,
            &drawing_xml,
            &opts.cell,
            opts.width,
            opts.height,
            &opts.format,
        )?;

        let graphic_frame = XlsxGraphicFrame {
            macro_name: opts.macro_name.clone(),
            nv_graphic_frame_pr: XlsxNvGraphicFramePr {
                c_nv_pr: Some(XlsxCNvPr {
                    id: c_nv_pr_id,
                    name: slicer_name.to_string(),
                    descr: opts.format.alt_text.clone(),
                    ..Default::default()
                }),
                c_nv_graphic_frame_pr: String::new(),
            },
            xfrm: XlsxXfrm {
                off: XlsxOff { x: 0, y: 0 },
                ext: XlsxPositiveSize2D { cx: 0, cy: 0 },
            },
            graphic: Some(XlsxGraphic {
                graphic_data: Some(XlsxGraphicData {
                    uri: NAMESPACE_DRAWING_ML_SLICER.to_string(),
                    sle: Some(XlsxSle {
                        xmlns_sle: ns.to_string(),
                        name: slicer_name.to_string(),
                    }),
                    ..Default::default()
                }),
            }),
        };
        let graphic = xml_to_string(&graphic_frame)?;

        let sp = XdrSp {
            macro_name: opts.macro_name.clone(),
            nv_sp_pr: Some(XdrNvSpPr {
                c_nv_pr: Some(XlsxCNvPr {
                    id: c_nv_pr_id,
                    descr: opts.format.alt_text.clone(),
                    ..Default::default()
                }),
                c_nv_sp_pr: Some(XdrCNvSpPr { tx_box: true }),
            }),
            sp_pr: Some(XlsxSpPr {
                xfrm: XlsxXfrm {
                    off: XlsxOff { x: 2914650, y: 152400 },
                    ext: XlsxPositiveSize2D { cx: 1828800, cy: 2238375 },
                },
                solid_fill: Some(ASolidFill {
                    srgb_clr: Some(ASrgbClr { val: Some("FFFFFF".to_string()), ..Default::default() }),
                    ..Default::default()
                }),
                prst_geom: XlsxPrstGeom { prst: "rect".to_string() },
                ln: Some(ALn {
                    w: Some(1),
                    solid_fill: Some(ASolidFill {
                        prst_clr: Some(AttrValString { val: Some("black".to_string()) }),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
            }),
            tx_body: Some(XdrTxBody {
                body_pr: Some(ABodyPr {
                    vert_overflow: Some("clip".to_string()),
                    horz_overflow: Some("clip".to_string()),
                    ..Default::default()
                }),
                p: vec![
                    AP {
                        r: Some(AR { t: Some("This shape represents a table slicer. Table slicers are not supported in this version of Excel.".to_string()), ..Default::default() }),
                        ..Default::default()
                    },
                    AP {
                        r: Some(AR { t: Some("If the shape was modified in an earlier version of Excel, or if the workbook was saved in Excel 2007 or earlier, the slicer can't be used.".to_string()), ..Default::default() }),
                        ..Default::default()
                    },
                ],
            }),
            ..Default::default()
        };
        let shape = xml_to_string(&sp)?;

        cell_anchor.client_data = Some(XdrClientData {
            f_locks_with_sheet: opts.format.locked.unwrap_or(true),
            f_prints_with_sheet: opts.format.print_object.unwrap_or(true),
        });

        let prefix = ns_to_prefix(ns);
        let xmlns_attr = format!(r#"xmlns:{}="{}""#, prefix, ns);
        let choice = format!(
            r#"<mc:Choice {} Requires="{}">{}</mc:Choice>"#,
            xmlns_attr, prefix, graphic
        );
        let fallback = format!(r#"<mc:Fallback>{}</mc:Fallback>"#, shape);
        cell_anchor.alternate_content.push(XlsxAlternateContent {
            xmlns_mc: Some(SOURCE_RELATIONSHIP.to_string()),
            content: XlsxInnerXml {
                content: format!("{choice}{fallback}"),
            },
        });

        if opts.format.positioning == "oneCell" {
            content.one_cell_anchor.push(cell_anchor);
        } else {
            content.two_cell_anchor.push(cell_anchor);
        }
        self.drawings.insert(drawing_xml, content);
        self.add_content_type_part(drawing_id, "drawings")
    }

    fn cell_anchor_shape(
        &self,
        sheet: &str,
        drawing_xml: &str,
        cell: &str,
        width: i32,
        height: i32,
        opts: &GraphicOptions,
    ) -> Result<(XlsxWsDr, XdrCellAnchor, i64)> {
        let (col, row) = cell_name_to_coordinates(cell)?;
        let width = (width as f64 * opts.scale_x) as i32;
        let height = (height as f64 * opts.scale_y) as i32;
        let (col_start, row_start, col_end, row_end, x1, y1, x2, y2) =
            self.position_object_pixels(sheet, col, row, width, height, opts)?;
        let (content, c_nv_pr_id) = self.drawing_parser(drawing_xml)?;

        let mut anchor = XdrCellAnchor {
            edit_as: if opts.positioning.is_empty() {
                None
            } else {
                Some(opts.positioning.clone())
            },
            from: Some(XlsxFrom {
                col: col_start as i64,
                col_off: x1 as i64 * EMU as i64,
                row: row_start as i64,
                row_off: y1 as i64 * EMU as i64,
            }),
            ..Default::default()
        };
        if opts.positioning == "oneCell" {
            anchor.ext = Some(XlsxPositiveSize2D {
                cx: x2 as i64 * EMU as i64,
                cy: y2 as i64 * EMU as i64,
            });
        } else {
            anchor.to = Some(XlsxTo {
                col: col_end as i64,
                col_off: x2 as i64 * EMU as i64,
                row: row_end as i64,
                row_off: y2 as i64 * EMU as i64,
            });
        }
        Ok((content, anchor, c_nv_pr_id))
    }
}

fn ns_to_prefix(ns: &str) -> &'static str {
    match ns {
        NAMESPACE_DRAWING_ML_A14 => "a14",
        NAMESPACE_DRAWING_ML_SLICER_X15 => "sle15",
        _ => "a14",
    }
}

// ------------------------------------------------------------------
// Workbook slicer cache references
// ------------------------------------------------------------------

impl File {
    fn add_workbook_slicer_cache(&self, slicer_cache_id: i32, uri: &str) -> Result<()> {
        let mut wb = self.workbook_reader()?;
        let r_id = self.add_rels(
            &self.get_workbook_rels_path(),
            SOURCE_RELATIONSHIP_SLICER_CACHE,
            &format!("/xl/slicerCaches/slicerCache{slicer_cache_id}.xml"),
            "",
        );

        if wb.ext_lst.is_none() {
            wb.ext_lst = Some(XlsxExtLst::default());
        }
        let ext_lst = wb.ext_lst.as_mut().unwrap();
        let mut append_mode = false;
        for ext in &mut ext_lst.ext {
            if ext.uri.as_deref() == Some(uri) {
                let decode_slicer_caches: DecodeSlicerCaches =
                    xml_from_reader(wrap_ext_content(&ext.content).as_bytes())?;
                let slicer_cache = XlsxX14SlicerCache {
                    rid: format!("rId{r_id}"),
                };
                let slicer_cache_bytes = xml_to_string(&slicer_cache)?;
                let content = format!(
                    "{}{}",
                    decode_slicer_caches.content,
                    extract_inner(&slicer_cache_bytes)
                );
                let caches_bytes = if uri == EXT_URI_SLICER_CACHES_X14 {
                    xml_to_string(&XlsxX14SlicerCaches {
                        xmlns: NAMESPACE_SPREADSHEET_X14.to_string(),
                        content,
                    })?
                } else {
                    xml_to_string(&XlsxX15SlicerCaches {
                        xmlns: NAMESPACE_SPREADSHEET_X14.to_string(),
                        content,
                    })?
                };
                ext.content = extract_inner(&caches_bytes);
                append_mode = true;
            }
        }

        if !append_mode {
            let slicer_cache = XlsxX14SlicerCache {
                rid: format!("rId{r_id}"),
            };
            let slicer_cache_bytes = xml_to_string(&slicer_cache)?;
            let (content, ns) = (
                extract_inner(&slicer_cache_bytes),
                NAMESPACE_SPREADSHEET_X14,
            );
            let caches_bytes = if uri == EXT_URI_SLICER_CACHES_X14 {
                xml_to_string(&XlsxX14SlicerCaches {
                    xmlns: ns.to_string(),
                    content,
                })?
            } else {
                xml_to_string(&XlsxX15SlicerCaches {
                    xmlns: ns.to_string(),
                    content,
                })?
            };
            ext_lst.ext.push(XlsxExt {
                uri: Some(uri.to_string()),
                content: extract_inner(&caches_bytes),
                ..Default::default()
            });
        }

        sort_ext_lst(&mut ext_lst.ext, WORKBOOK_EXT_URI_PRIORITY);
        *self.workbook.lock().unwrap() = Some(wb);
        Ok(())
    }
}

// ------------------------------------------------------------------
// Get slicers
// ------------------------------------------------------------------

impl File {
    fn get_all_slicers(&self) -> Result<HashMap<String, Vec<SlicerOptions>>> {
        let mut slicers: HashMap<String, Vec<SlicerOptions>> = HashMap::new();
        for sheet_name in self.get_sheet_list() {
            match self.get_slicers(&sheet_name) {
                Ok(sles) => {
                    slicers.insert(sheet_name, sles);
                }
                Err(_) => {}
            }
        }
        Ok(slicers)
    }

    fn get_slicer_cache(
        &self,
        slicer_cache_name: &str,
        opt: &mut SlicerOptions,
    ) -> Option<XlsxSlicerCacheDefinition> {
        let mut result: Option<XlsxSlicerCacheDefinition> = None;
        for entry in self.pkg.iter() {
            let k = entry.key();
            if !k.contains("xl/slicerCaches/slicerCache") {
                continue;
            }
            if let Ok(slicer_cache) = self.slicer_cache_reader(k) {
                if slicer_cache.name == slicer_cache_name {
                    opt.slicer_cache_xml = k.clone();
                    result = Some(slicer_cache);
                    break;
                }
            }
        }
        result
    }

    fn get_slicers_internal(
        &self,
        sheet: &str,
        r_id: &str,
        drawing_xml: &str,
    ) -> Result<Vec<SlicerOptions>> {
        let mut opts = Vec::new();
        let sheet_relationships_slicer_xml = self.get_sheet_relationships_target_by_id(sheet, r_id);
        let slicer_xml = sheet_relationships_slicer_xml.replace("..", "xl");
        let slicers = self.slicer_reader(&slicer_xml)?;
        for slicer in slicers.slicer {
            let mut opt = SlicerOptions {
                slicer_xml: slicer_xml.clone(),
                slicer_cache_name: slicer.cache.clone(),
                slicer_sheet_name: sheet.to_string(),
                slicer_sheet_rid: r_id.to_string(),
                drawing_xml: drawing_xml.to_string(),
                name: slicer.name.clone(),
                caption: slicer.caption.clone().unwrap_or_default(),
                display_header: slicer.show_caption,
                ..Default::default()
            };
            if let Some(slicer_cache) = self.get_slicer_cache(&slicer.cache, &mut opt) {
                self.extract_table_slicer(&slicer_cache, &mut opt)?;
                self.extract_pivot_table_slicer(&slicer_cache, &mut opt)?;
                self.extract_slicer_cell_anchor(drawing_xml, &mut opt)?;
                opts.push(opt);
            }
        }
        Ok(opts)
    }

    fn extract_table_slicer(
        &self,
        slicer_cache: &XlsxSlicerCacheDefinition,
        opt: &mut SlicerOptions,
    ) -> Result<()> {
        if let Some(ref ext_lst) = slicer_cache.ext_lst {
            let tables = self.get_tables_internal()?;
            let ext = decode_first_ext(&ext_lst.ext)?;
            if ext.uri.as_deref() == Some(EXT_URI_SLICER_CACHE_DEFINITION) {
                let tsc: DecodeTableSlicerCache =
                    xml_from_reader(wrap_ext_content(&ext.content).as_bytes())?;
                opt.item_desc = tsc.sort_order == "descending";
                for (sheet_name, sheet_tables) in tables {
                    for table in sheet_tables {
                        if tsc.table_id == table.t_id {
                            opt.table_name = table.name.clone();
                            opt.table_sheet = sheet_name.clone();
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn extract_pivot_table_slicer(
        &self,
        slicer_cache: &XlsxSlicerCacheDefinition,
        opt: &mut SlicerOptions,
    ) -> Result<()> {
        let pivot_tables = self.get_pivot_tables_internal()?;
        if let Some(ref pts) = slicer_cache.pivot_tables {
            let mut pivot_cache_xml = String::new();
            for pt in &pts.pivot_table {
                opt.table_name = pt.name.clone();
                for (sheet_name, sheet_pts) in &pivot_tables {
                    for pivot_table in sheet_pts {
                        if opt.table_name == pivot_table.data_range {
                            opt.table_sheet = sheet_name.clone();
                        }
                        if pt.name == pivot_table.data_range {
                            pivot_cache_xml =
                                self.find_pivot_cache_xml(pivot_table).unwrap_or_default();
                        }
                    }
                }
            }
            if let Some(ref data) = slicer_cache.data {
                if let Some(ref tabular) = data.tabular {
                    opt.item_desc = tabular.sort_order.as_deref() == Some("descending");
                }
            }
            self.extract_slicer_selected_items(&pivot_cache_xml, slicer_cache, opt)?;
        }
        Ok(())
    }

    fn extract_slicer_selected_items(
        &self,
        pivot_cache_xml: &str,
        slicer_cache: &XlsxSlicerCacheDefinition,
        opt: &mut SlicerOptions,
    ) -> Result<()> {
        if pivot_cache_xml.is_empty() {
            return Ok(());
        }
        let pc = self.slicer_pivot_cache_reader(pivot_cache_xml)?;
        if let Some(ref data) = slicer_cache.data {
            if let Some(ref tabular) = data.tabular {
                if let Some(ref items) = tabular.items {
                    for item in &items.i {
                        if item.s.unwrap_or(false) {
                            if let Some(ref fields) = pc.cache_fields {
                                for field in &fields.cache_field {
                                    if field.name == slicer_cache.source_name {
                                        if let Some(ref shared_items) = field.shared_items {
                                            let idx = item.x as usize;
                                            if idx < shared_items.items.len() {
                                                let val = match &shared_items.items[idx] {
                                                    XlsxSharedItem::M(d) => d.v.clone(),
                                                    XlsxSharedItem::B(d) => d.v.clone(),
                                                    XlsxSharedItem::N(d) => d.v.clone(),
                                                    XlsxSharedItem::E(d) => d.v.clone(),
                                                    XlsxSharedItem::S(d) => d.v.clone(),
                                                    XlsxSharedItem::D(d) => d.v.clone(),
                                                };
                                                if let Some(v) = val {
                                                    opt.selected_items.push(v);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn extract_slicer_cell_anchor(&self, drawing_xml: &str, opt: &mut SlicerOptions) -> Result<()> {
        if drawing_xml.is_empty() {
            return Ok(());
        }
        let (ws_dr, _) = self.drawing_parser(drawing_xml)?;
        for anchor in &ws_dr.one_cell_anchor {
            self.extract_slicer_from_anchor(anchor, opt)?;
            self.extract_slicer_from_decode_anchor(anchor, opt)?;
        }
        for anchor in &ws_dr.two_cell_anchor {
            self.extract_slicer_from_anchor(anchor, opt)?;
            self.extract_slicer_from_decode_anchor(anchor, opt)?;
        }
        Ok(())
    }

    fn extract_slicer_from_anchor(
        &self,
        anchor: &XdrCellAnchor,
        opt: &mut SlicerOptions,
    ) -> Result<()> {
        for ac in &anchor.alternate_content {
            let de_choice: DecodeChoice =
                xml_from_reader(ac.content.content.as_bytes()).unwrap_or_default();
            if de_choice.xmlns_sle15.as_deref() == Some(NAMESPACE_DRAWING_ML_SLICER_X15)
                || de_choice.xmlns_a14.as_deref() == Some(NAMESPACE_DRAWING_ML_A14)
            {
                if de_choice.graphic_frame.nv_graphic_frame_pr.c_nv_pr.name == opt.name {
                    opt.macro_name = de_choice.graphic_frame.macro_name.clone();
                    if let Some(ref from) = anchor.from {
                        opt.cell = coordinates_to_cell_name(
                            (from.col + 1) as i32,
                            (from.row + 1) as i32,
                            false,
                        )?;
                    }
                }
            }
        }
        Ok(())
    }

    fn extract_slicer_from_decode_anchor(
        &self,
        anchor: &XdrCellAnchor,
        opt: &mut SlicerOptions,
    ) -> Result<()> {
        if let Some(ref gf) = anchor.graphic_frame {
            if let Some(ref graphic) = gf.graphic {
                if let Some(ref data) = graphic.graphic_data {
                    if data.uri == NAMESPACE_DRAWING_ML_SLICER
                        || data.uri == NAMESPACE_DRAWING_ML_SLICER_X15
                    {
                        if let Some(ref sle) = data.sle {
                            if sle.name == opt.name {
                                opt.macro_name = gf.macro_name.clone();
                                if let Some(ref from) = anchor.from {
                                    opt.cell = coordinates_to_cell_name(
                                        (from.col + 1) as i32,
                                        (from.row + 1) as i32,
                                        false,
                                    )?;
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn get_tables_internal(&self) -> Result<HashMap<String, Vec<Table>>> {
        let mut tables: HashMap<String, Vec<Table>> = HashMap::new();
        for sheet_name in self.get_sheet_list() {
            if let Ok(tbls) = self.get_tables(&sheet_name) {
                tables.insert(sheet_name, tbls);
            }
        }
        Ok(tables)
    }

    fn get_pivot_tables_internal(
        &self,
    ) -> Result<HashMap<String, Vec<crate::pivot_table::PivotTableOptions>>> {
        let mut pivot_tables: HashMap<String, Vec<crate::pivot_table::PivotTableOptions>> =
            HashMap::new();
        for sheet_name in self.get_sheet_list() {
            if let Ok(pts) = self.get_pivot_tables(&sheet_name) {
                pivot_tables.insert(sheet_name, pts);
            }
        }
        Ok(pivot_tables)
    }
}

// ------------------------------------------------------------------
// Delete slicer
// ------------------------------------------------------------------

impl File {
    fn delete_slicer_internal(&self, opts: &SlicerOptions) -> Result<()> {
        let mut slicers = self.slicer_reader(&opts.slicer_xml)?;
        slicers.slicer.retain(|s| s.name != opts.name);
        if slicers.slicer.is_empty() {
            let mut ws = self.work_sheet_reader(&opts.slicer_sheet_name)?;
            if let Some(ref mut ext_lst) = ws.ext_lst {
                let original_len = ext_lst.ext.len();
                ext_lst.ext.retain(|ext| {
                    if ext.uri.as_deref() != Some(EXT_URI_SLICER_LIST_X14)
                        && ext.uri.as_deref() != Some(EXT_URI_SLICER_LIST_X15)
                    {
                        return true;
                    }
                    let slicer_list: DecodeSlicerList =
                        xml_from_reader(wrap_ext_content(&ext.content).as_bytes())
                            .unwrap_or_default();
                    !slicer_list
                        .slicer
                        .iter()
                        .any(|s| s.rid == opts.slicer_sheet_rid)
                });
                if ext_lst.ext.len() != original_len {
                    if ext_lst.ext.is_empty() {
                        ws.ext_lst = None;
                    }
                    if let Some(path) = self.get_sheet_xml_path(&opts.slicer_sheet_name) {
                        self.sheet.insert(path, ws);
                    }
                    self.pkg.remove(&opts.slicer_xml);
                    self.remove_content_types_part(
                        CONTENT_TYPE_SLICER,
                        &format!("/{}", opts.slicer_xml),
                    )?;
                    self.delete_sheet_relationships(
                        &opts.slicer_sheet_name,
                        &opts.slicer_sheet_rid,
                    );
                    return Ok(());
                }
            }
        }
        let output = xml_to_string(&slicers)?;
        self.save_file_list(&opts.slicer_xml, output.as_bytes());
        Ok(())
    }

    fn delete_slicer_cache(
        &self,
        all: &HashMap<String, Vec<SlicerOptions>>,
        opts: &SlicerOptions,
    ) -> Result<()> {
        for (_, slicers) in all {
            for slicer in slicers {
                if slicer.name != opts.name && slicer.slicer_cache_name == opts.slicer_cache_name {
                    return Ok(());
                }
            }
        }
        self.delete_defined_name(&DefinedName {
            name: opts.slicer_cache_name.clone(),
            scope: "Workbook".to_string(),
            ..Default::default()
        })?;
        self.pkg.remove(&opts.slicer_cache_xml);
        self.remove_content_types_part(
            CONTENT_TYPE_SLICER_CACHE,
            &format!("/{}", opts.slicer_cache_xml),
        )
    }
}

// ------------------------------------------------------------------
// Extension list helpers
// ------------------------------------------------------------------

fn decode_ext_lst(exts: &[XlsxExt]) -> Result<Vec<XlsxExt>> {
    let mut out = Vec::new();
    for ext in exts {
        out.push(XlsxExt {
            uri: ext.uri.clone(),
            xmlns_x14: ext.xmlns_x14.clone(),
            xmlns_xm: ext.xmlns_xm.clone(),
            content: ext.content.clone(),
        });
    }
    Ok(out)
}

fn decode_first_ext(exts: &[XlsxExt]) -> Result<XlsxExt> {
    decode_ext_lst(exts)?
        .into_iter()
        .next()
        .ok_or_else(|| ErrParameterInvalid.into())
}

fn wrap_ext_content(content: &str) -> String {
    format!("<ext>{content}</ext>")
}

fn extract_inner(s: &str) -> String {
    // Strip the outermost XML element, leaving only its children/text.
    let s = s.trim();
    let start = s.find('>').map(|i| i + 1).unwrap_or(0);
    let end = s.rfind('<').unwrap_or(s.len());
    if end > start {
        s[start..end].to_string()
    } else {
        String::new()
    }
}

fn sort_ext_lst(exts: &mut Vec<XlsxExt>, priority: &[&str]) {
    exts.sort_by(|a, b| {
        let ai = a
            .uri
            .as_deref()
            .and_then(|u| priority.iter().position(|p| *p == u))
            .unwrap_or(usize::MAX);
        let bi = b
            .uri
            .as_deref()
            .and_then(|u| priority.iter().position(|p| *p == u))
            .unwrap_or(usize::MAX);
        ai.cmp(&bi)
    });
}

// ------------------------------------------------------------------
// Tests
// ------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Options;

    fn sample_opts() -> SlicerOptions {
        SlicerOptions {
            name: "Column1".to_string(),
            cell: "E1".to_string(),
            table_sheet: "Sheet1".to_string(),
            table_name: "Table1".to_string(),
            caption: "Column1".to_string(),
            width: 200,
            height: 200,
            ..Default::default()
        }
    }

    #[test]
    fn parse_slicer_options_defaults() {
        let opts = sample_opts();
        let parsed = parse_slicer_options(&opts).unwrap();
        assert_eq!(parsed.width, 200);
        assert_eq!(parsed.height, 200);
        assert_eq!(parsed.format.print_object, Some(true));
        assert_eq!(parsed.format.locked, Some(true));
    }

    #[test]
    fn parse_slicer_options_missing_required() {
        let mut opts = sample_opts();
        opts.name.clear();
        assert!(parse_slicer_options(&opts).is_err());

        let mut opts = sample_opts();
        opts.cell.clear();
        assert!(parse_slicer_options(&opts).is_err());
    }

    #[test]
    fn parse_slicer_options_fills_defaults() {
        let mut opts = sample_opts();
        opts.width = 0;
        opts.height = 0;
        let parsed = parse_slicer_options(&opts).unwrap();
        assert_eq!(parsed.width, DEFAULT_SLICER_WIDTH);
        assert_eq!(parsed.height, DEFAULT_SLICER_HEIGHT);
    }

    #[test]
    fn gen_slicer_cache_name_cleans_invalid_chars() {
        let f = File::new_with_options(Options::default());
        assert!(
            f.gen_slicer_cache_name("Column 1")
                .starts_with("Slicer_Column_1")
        );
        assert!(f.gen_slicer_cache_name("A.B").starts_with("Slicer_A.B"));
    }

    #[test]
    fn get_slicers_empty_workbook() {
        let f = File::new_with_options(Options::default());
        let slicers = f.get_slicers("Sheet1").unwrap();
        assert!(slicers.is_empty());
    }

    #[test]
    fn delete_nonexistent_slicer_fails() {
        let f = File::new_with_options(Options::default());
        assert!(f.delete_slicer("NoSuchSlicer").is_err());
    }

    #[test]
    fn add_slicer_requires_source_table_or_pivot() {
        let f = File::new_with_options(Options::default());
        let opts = sample_opts();
        // No table/pivot exists in a blank workbook.
        assert!(f.add_slicer("Sheet1", &opts).is_err());
    }

    #[test]
    fn count_slicers_starts_at_zero() {
        let f = File::new_with_options(Options::default());
        assert_eq!(f.count_slicers(), 0);
        assert_eq!(f.count_slicer_cache(), 0);
    }
}
