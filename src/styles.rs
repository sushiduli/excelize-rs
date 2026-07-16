//! Public style API and XML conversion.
//!
//! Ported from Go `styles.go` and `xmlStyles.go`.

use std::collections::HashMap;
use std::sync::LazyLock;

use quick_xml::de::from_reader as xml_from_reader;
use quick_xml::se::to_string as xml_to_string;
use serde::{Deserialize, Serialize};

use crate::constants::{
    EXT_URI_CONDITIONAL_FORMATTING_RULE_ID, EXT_URI_CONDITIONAL_FORMATTINGS, MAX_CELL_STYLES,
    MAX_COLUMNS, MAX_FONT_FAMILY_LENGTH, MAX_FONT_SIZE, MIN_FONT_SIZE,
    NAMESPACE_SPREADSHEET_EXCEL_2006_MAIN, NAMESPACE_SPREADSHEET_X14, TOTAL_ROWS,
    WORKSHEET_EXT_URI_PRIORITY,
};
use crate::data_validation::delete_cells_from_sqref;
use crate::errors::{
    ErrCellStyles, ErrCustomNumFmt, ErrFillGradientColor, ErrFillGradientShading, ErrFillPattern,
    ErrFillPatternColor, ErrFillType, ErrFontLength, ErrFontSize, ErrParameterInvalid,
    ErrParameterRequired, Result,
};
use crate::file::File;
use crate::hsl::theme_color;
use crate::lib_util::{
    cell_name_to_coordinates, column_name_to_number, coordinates_to_cell_name, count_utf16_string,
    flat_sqref, in_str_slice,
};
use crate::templates::{
    INDEXED_COLOR_MAPPING, SUPPORTED_UNDERLINE_TYPES, SUPPORTED_VERT_ALIGN_TYPES,
};
use crate::xml::common::{
    AttrValBool, AttrValFloat, AttrValInt, AttrValString, XlsxColor, XlsxExt, XlsxExtLst,
};
use crate::xml::styles::{
    XlsxAlignment, XlsxBorder, XlsxBorders, XlsxCellStyles, XlsxCellXfs, XlsxDxf, XlsxDxfs,
    XlsxFill, XlsxFills, XlsxFont, XlsxFonts, XlsxGradientFill, XlsxGradientFillStop, XlsxLine,
    XlsxNumFmt, XlsxNumFmts, XlsxPatternFill, XlsxProtection, XlsxStyleSheet, XlsxXf,
};
use crate::xml::worksheet::{
    ConditionalFormatOptions, DecodeX14CfRule, DecodeX14Cfvo, DecodeX14ConditionalFormattingRules,
    DecodeX14DataBar, DecodeX14IconSet, Xlsx14Cfvo, Xlsx14DataBar, Xlsx14IconSet, XlsxCfRule,
    XlsxCfvo, XlsxColorScale, XlsxConditionalFormatting, XlsxDataBar, XlsxIconSet, XlsxWorksheet,
    XlsxX14CfRule, XlsxX14ConditionalFormatting,
};

// ------------------------------------------------------------------
// Public types
// ------------------------------------------------------------------

/// Directly maps the alignment settings of the cells.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Alignment {
    #[serde(default)]
    pub horizontal: String,
    #[serde(default)]
    pub indent: i64,
    #[serde(default)]
    pub justify_last_line: bool,
    #[serde(default)]
    pub reading_order: u64,
    #[serde(default)]
    pub relative_indent: i64,
    #[serde(default)]
    pub shrink_to_fit: bool,
    #[serde(default)]
    pub text_rotation: i64,
    #[serde(default)]
    pub vertical: String,
    #[serde(default)]
    pub wrap_text: bool,
}

/// Directly maps the font settings.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct Font {
    #[serde(rename = "sz", default, skip_serializing_if = "Option::is_none")]
    pub size: Option<f64>,
    /// Font family name (e.g. `"Calibri"`). Equivalent to Go `Font.Family`.
    #[serde(rename = "name", default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Font family number (e.g. `2` for Swiss).
    #[serde(rename = "family", default, skip_serializing_if = "Option::is_none")]
    pub family: Option<i64>,
    #[serde(rename = "b", default, skip_serializing_if = "Option::is_none")]
    pub bold: Option<bool>,
    #[serde(rename = "i", default, skip_serializing_if = "Option::is_none")]
    pub italic: Option<bool>,
    #[serde(rename = "strike", default, skip_serializing_if = "Option::is_none")]
    pub strike: Option<bool>,
    #[serde(rename = "u", default, skip_serializing_if = "Option::is_none")]
    pub underline: Option<String>,
    /// RGB color in `RRGGBB` notation.
    #[serde(rename = "color", default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color_indexed: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color_theme: Option<i64>,
    #[serde(default)]
    pub color_tint: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vert_align: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub charset: Option<i64>,
}

/// Directly maps the border settings of cells.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Border {
    #[serde(default)]
    pub r#type: String,
    #[serde(default)]
    pub color: String,
    #[serde(default)]
    pub style: i64,
}

/// Directly maps the fill settings of cells. Supports both `gradient` and
/// `pattern` fills via the `r#type` field.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fill {
    #[serde(default)]
    pub r#type: String,
    #[serde(default)]
    pub pattern: i64,
    #[serde(default)]
    pub color: Vec<String>,
    #[serde(default)]
    pub shading: i64,
    #[serde(default)]
    pub transparency: i64,
}

/// Directly maps the protection settings of cells.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Protection {
    #[serde(default)]
    pub hidden: bool,
    #[serde(default)]
    pub locked: bool,
}

/// Directly maps the style settings of cells.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct Style {
    #[serde(default)]
    pub border: Vec<Border>,
    #[serde(default)]
    pub fill: Fill,
    #[serde(default)]
    pub font: Option<Font>,
    #[serde(default)]
    pub alignment: Option<Alignment>,
    #[serde(default)]
    pub protection: Option<Protection>,
    #[serde(default)]
    pub num_fmt: i64,
    #[serde(default)]
    pub decimal_places: Option<i64>,
    #[serde(default)]
    pub custom_num_fmt: Option<String>,
    #[serde(default)]
    pub neg_red: bool,
}

// ------------------------------------------------------------------
// Public API
// ------------------------------------------------------------------

impl File {
    /// Create a new cell style and return its style ID.
    ///
    /// Equivalent to Go `NewStyle`.
    pub fn new_style(&self, style: &Style) -> Result<i32> {
        let mut ss = self.styles.lock().unwrap();
        let mut style_sheet = ss.take().unwrap_or_default();

        let mut fs = parse_format_style_set(style)?;

        if let Some(dp) = fs.decimal_places {
            if dp < 0 || dp > 30 {
                // Match Go behavior: reset out-of-range decimal places to 2.
                fs.decimal_places = Some(2);
            }
        }

        let existing = get_style_id(&style_sheet, &fs);
        if existing != -1 {
            *ss = Some(style_sheet);
            return Ok(existing);
        }

        let num_fmt_id = new_num_fmt(&mut style_sheet, &fs);

        let font_id = if let Some(ref font) = fs.font {
            let fid = get_font_id(&style_sheet, font);
            if fid == -1 {
                ensure_fonts(&mut style_sheet);
                style_sheet
                    .fonts
                    .as_mut()
                    .unwrap()
                    .font
                    .push(new_font(font));
                let id = style_sheet.fonts.as_ref().unwrap().count;
                style_sheet.fonts.as_mut().unwrap().count = id + 1;
                id as i32
            } else {
                fid
            }
        } else {
            0
        };

        let border_id = {
            let bid = get_border_id(&style_sheet, &fs);
            if bid == -1 {
                if fs.border.is_empty() {
                    0
                } else {
                    ensure_borders(&mut style_sheet);
                    style_sheet
                        .borders
                        .as_mut()
                        .unwrap()
                        .border
                        .push(new_borders(&fs));
                    let id = style_sheet.borders.as_ref().unwrap().count;
                    style_sheet.borders.as_mut().unwrap().count = id + 1;
                    id as i32
                }
            } else {
                bid
            }
        };

        let fill_id = {
            let fid = get_fill_id(&style_sheet, &fs);
            if fid == -1 {
                if let Some(xfill) = new_fills(&fs, true) {
                    ensure_fills(&mut style_sheet);
                    style_sheet.fills.as_mut().unwrap().fill.push(xfill);
                    let id = style_sheet.fills.as_ref().unwrap().count;
                    style_sheet.fills.as_mut().unwrap().count = id + 1;
                    id as i32
                } else {
                    0
                }
            } else {
                fid
            }
        };

        let apply_alignment = fs.alignment.is_some();
        let alignment = new_alignment(&fs);
        let apply_protection = fs.protection.is_some();
        let protection = new_protection(&fs);

        let id = set_cell_xfs(
            &mut style_sheet,
            font_id,
            num_fmt_id,
            fill_id,
            border_id,
            apply_alignment,
            apply_protection,
            alignment,
            protection,
        )?;

        *ss = Some(style_sheet);
        Ok(id)
    }

    /// Get the style definition for a given style ID.
    ///
    /// Equivalent to Go `GetStyle`.
    pub fn get_style(&self, style_id: i32) -> Result<Style> {
        // Clone the style sheet while holding the lock and release it before
        // calling the extract helpers. Those helpers may call back into the
        // File (e.g. `theme_reader`/`styles_reader` through `get_theme_color`),
        // and re-entering this lock would deadlock.
        let style_sheet = {
            let ss = self.styles.lock().unwrap();
            let style_sheet = ss
                .as_ref()
                .ok_or_else(|| crate::errors::new_invalid_style_id_error(style_id))?;
            if style_id < 0
                || style_sheet.cell_xfs.is_none()
                || style_sheet.cell_xfs.as_ref().unwrap().xf.len() as i32 <= style_id
            {
                return Err(crate::errors::new_invalid_style_id_error(style_id).into());
            }
            style_sheet.clone()
        };

        let xf = &style_sheet.cell_xfs.as_ref().unwrap().xf[style_id as usize];
        let mut style = Style::default();

        if should_extract_fill(xf, &style_sheet) {
            if let Some(fill) = style_sheet
                .fills
                .as_ref()
                .and_then(|f| f.fill.get(*xf.fill_id.as_ref().unwrap() as usize))
            {
                extract_fill(self, fill, &mut style);
            }
        }
        if should_extract_border(xf, &style_sheet) {
            if let Some(border) = style_sheet
                .borders
                .as_ref()
                .and_then(|b| b.border.get(*xf.border_id.as_ref().unwrap() as usize))
            {
                extract_border(self, border, &mut style);
            }
        }
        if should_extract_font(xf, &style_sheet) {
            if let Some(font) = style_sheet
                .fonts
                .as_ref()
                .and_then(|f| f.font.get(*xf.font_id.as_ref().unwrap() as usize))
            {
                style.font = Some(extract_font(font));
            }
        }
        if should_extract_alignment(xf, &style_sheet) {
            if let Some(ref alignment) = xf.alignment {
                style.alignment = Some(extract_alignment(alignment));
            }
        }
        if should_extract_protection(xf, &style_sheet) {
            if let Some(ref protection) = xf.protection {
                style.protection = Some(extract_protection(protection));
            }
        }
        if let Some(num_fmt_id) = xf.num_fmt_id {
            extract_num_fmt(num_fmt_id as i32, &style_sheet, &mut style);
        }
        Ok(style)
    }

    /// Return the default font name in the workbook.
    ///
    /// Equivalent to Go `GetDefaultFont`.
    pub fn get_default_font(&self) -> Result<String> {
        let ss = self.styles.lock().unwrap();
        let style_sheet = ss
            .as_ref()
            .ok_or_else(|| crate::errors::new_invalid_style_id_error(0))?;
        if style_sheet
            .fonts
            .as_ref()
            .map(|f| f.font.is_empty())
            .unwrap_or(true)
        {
            return Ok("宋体".to_string());
        }
        Ok(style_sheet.fonts.as_ref().unwrap().font[0]
            .name
            .as_ref()
            .and_then(|n| n.val.clone())
            .unwrap_or_default())
    }

    /// Change the default font in the workbook.
    ///
    /// Equivalent to Go `SetDefaultFont`.
    pub fn set_default_font(&self, font_name: &str) -> Result<()> {
        let mut ss = self.styles.lock().unwrap();
        let mut style_sheet = ss.take().unwrap_or_default();
        if style_sheet
            .fonts
            .as_ref()
            .map(|f| f.font.is_empty())
            .unwrap_or(true)
        {
            *ss = Some(style_sheet);
            return Ok(());
        }
        style_sheet.fonts.as_mut().unwrap().font[0].name = Some(AttrValString {
            val: Some(font_name.to_string()),
        });
        if let Some(ref mut cs) = style_sheet.cell_styles {
            if !cs.cell_style.is_empty() {
                cs.cell_style[0].custom_built_in = Some(true);
            }
        }
        *ss = Some(style_sheet);
        Ok(())
    }

    /// Create a new style for conditional formats and return its dxf ID.
    ///
    /// Equivalent to Go `NewConditionalStyle`.
    pub fn new_conditional_style(&self, style: &Style) -> Result<i32> {
        let mut ss = self.styles.lock().unwrap();
        let mut style_sheet = ss.take().unwrap_or_default();
        let mut fs = parse_format_style_set(style)?;
        if let Some(dp) = fs.decimal_places {
            if dp < 0 || dp > 30 {
                // Match Go behavior: reset out-of-range decimal places to 2.
                fs.decimal_places = Some(2);
            }
        }
        let mut dxf = XlsxDxf::default();
        dxf.fill = new_fills(&fs, false);
        dxf.alignment = new_alignment(&fs);
        if !fs.border.is_empty() {
            dxf.border = Some(new_borders(&fs));
        }
        if let Some(ref font) = fs.font {
            dxf.font = Some(new_font(font));
        }
        dxf.protection = new_protection(&fs);
        dxf.num_fmt = new_dxf_num_fmt(&style_sheet, style);
        if style_sheet.dxfs.is_none() {
            style_sheet.dxfs = Some(XlsxDxfs::default());
        }
        let dxfs = style_sheet.dxfs.as_mut().unwrap();
        dxfs.count += 1;
        dxfs.dxfs.push(dxf);
        let id = dxfs.count - 1;
        *ss = Some(style_sheet);
        Ok(id as i32)
    }

    /// Get the conditional format style definition for a given dxf ID.
    ///
    /// Equivalent to Go `GetConditionalStyle`.
    pub fn get_conditional_style(&self, style_id: i32) -> Result<Style> {
        // Clone the needed data while holding the lock and release it before
        // calling extract helpers that may re-enter the styles lock.
        let (dxf, num_fmt_id) = {
            let ss = self.styles.lock().unwrap();
            let style_sheet = ss
                .as_ref()
                .ok_or_else(|| crate::errors::new_invalid_style_id_error(style_id))?;
            if style_id < 0
                || style_sheet.dxfs.is_none()
                || style_sheet.dxfs.as_ref().unwrap().dxfs.len() as i32 <= style_id
            {
                return Err(crate::errors::new_invalid_style_id_error(style_id).into());
            }
            let dxf = style_sheet.dxfs.as_ref().unwrap().dxfs[style_id as usize].clone();
            let num_fmt_id = dxf.num_fmt.as_ref().map(|n| n.num_fmt_id as i32);
            (dxf, num_fmt_id)
        };

        let mut style = Style::default();
        if let Some(mut fill) = dxf.fill {
            if fill.pattern_fill.is_some()
                && fill
                    .pattern_fill
                    .as_ref()
                    .unwrap()
                    .pattern_type
                    .as_deref()
                    .unwrap_or("")
                    == ""
            {
                fill.pattern_fill.as_mut().unwrap().pattern_type = Some("solid".to_string());
            }
            extract_fill(self, &fill, &mut style);
        }
        if let Some(ref border) = dxf.border {
            extract_border(self, border, &mut style);
        }
        if let Some(ref font) = dxf.font {
            style.font = Some(extract_font(font));
        }
        if let Some(ref alignment) = dxf.alignment {
            style.alignment = Some(extract_alignment(alignment));
        }
        if let Some(ref protection) = dxf.protection {
            style.protection = Some(extract_protection(protection));
        }
        if let Some(num_fmt_id) = num_fmt_id {
            let style_sheet = self.styles_reader()?;
            extract_num_fmt(num_fmt_id, &style_sheet, &mut style);
        }
        Ok(style)
    }

    /// Create conditional formatting rules for a range of cells.
    ///
    /// Equivalent to Go `SetConditionalFormat`.
    pub fn set_conditional_format(
        &self,
        sheet: &str,
        range_ref: &str,
        opts: &[ConditionalFormatOptions],
    ) -> Result<()> {
        let path = self.get_sheet_xml_path(sheet).ok_or_else(|| {
            Box::new(crate::errors::ErrSheetNotExist {
                sheet_name: sheet.to_string(),
            }) as Box<dyn std::error::Error + Send + Sync>
        })?;
        let mut ws = self.work_sheet_reader(sheet)?;
        let (sqref, mast_cell) = prepare_conditional_format_range(range_ref)?;
        let mut rules: i64 = 0;
        for cf in &ws.conditional_formatting {
            rules += cf.cf_rule.len() as i64;
        }
        let no_criteria_types = [
            "containsBlanks",
            "notContainsBlanks",
            "containsErrors",
            "notContainsErrors",
            "expression",
            "iconSet",
        ];
        let sheet_id = self.get_sheet_id(sheet) as u32;
        let mut cf_rules: Vec<XlsxCfRule> = Vec::new();
        for (i, opt) in opts.iter().enumerate() {
            let vt = VALID_TYPE.get(opt.r#type.as_str()).copied();
            let Some(vt) = vt else {
                return Err(Box::new(ErrParameterInvalid));
            };
            let ct = CRITERIA_TYPE
                .get(opt.criteria.as_str())
                .copied()
                .unwrap_or("");
            let ct_ok = !ct.is_empty() || in_str_slice(&no_criteria_types, vt, true) != -1;
            if !ct_ok {
                return Err(Box::new(ErrParameterInvalid));
            }
            let Some(draw_func) = DRAW_COND_FMT_FUNC.get(vt) else {
                return Err(Box::new(ErrParameterInvalid));
            };
            let priority = rules + i as i64;
            let guid = format!(
                "{{00000000-0000-0000-{priority:04X}-{sheet_id:012X}}}",
                priority = priority,
                sheet_id = sheet_id
            );
            let (rule, x14_rule) = draw_func(priority as i32, ct, &mast_cell, &guid, opt);
            if rule.is_none() && x14_rule.is_none() {
                return Err(Box::new(ErrParameterInvalid));
            }
            if let Some(rule) = rule {
                cf_rules.push(rule);
            }
            if let Some(x14_rule) = x14_rule {
                append_cf_rule(&mut ws, &x14_rule, &sqref)?;
                self.add_sheet_name_space(sheet, NAMESPACE_SPREADSHEET_X14);
            }
        }
        if !cf_rules.is_empty() {
            ws.conditional_formatting.push(XlsxConditionalFormatting {
                sqref: Some(sqref.clone()),
                cf_rule: cf_rules,
                ..Default::default()
            });
        }
        self.sheet.insert(path, ws);
        Ok(())
    }

    /// Return conditional format settings by worksheet name.
    ///
    /// Equivalent to Go `GetConditionalFormats`.
    pub fn get_conditional_formats(
        &self,
        sheet: &str,
    ) -> Result<HashMap<String, Vec<ConditionalFormatOptions>>> {
        let mut result: HashMap<String, Vec<ConditionalFormatOptions>> = HashMap::new();
        let ws = self.work_sheet_reader(sheet)?;
        for cf in &ws.conditional_formatting {
            let (_, mast_cell) =
                prepare_conditional_format_range(cf.sqref.as_deref().unwrap_or(""))?;
            let mut opts = Vec::new();
            for cr in &cf.cf_rule {
                if let Some(t) = cr.r#type.as_deref() {
                    if let Some(extract_func) = EXTRACT_COND_FMT_FUNC.get(t) {
                        opts.push(extract_func(self, &mast_cell, cr, ws.ext_lst.as_ref()));
                    }
                }
            }
            if let Some(sqref) = cf.sqref.clone() {
                result.insert(sqref, opts);
            }
        }
        if let Some(ext_lst) = ws.ext_lst.as_ref() {
            for ext in &ext_lst.ext {
                if ext.uri.as_deref() == Some(EXT_URI_CONDITIONAL_FORMATTINGS) {
                    let decoded = parse_x14_conditional_formattings(&ext.content)?;
                    for cond_fmt in &decoded.cond_fmt {
                        let mut opts = Vec::new();
                        for rule in &cond_fmt.cf_rule {
                            if rule.r#type.as_deref() == Some("iconSet") {
                                opts.push(extract_x14_cond_fmt_icon_set(rule));
                            }
                        }
                        if let Some(sqref) = cond_fmt.sqref.as_deref() {
                            result.entry(sqref.to_string()).or_default().extend(opts);
                        }
                    }
                }
            }
        }
        Ok(result)
    }

    /// Unset conditional formats for a range of cells.
    ///
    /// Equivalent to Go `UnsetConditionalFormat`.
    pub fn unset_conditional_format(&self, sheet: &str, range_ref: &str) -> Result<()> {
        let path = self.get_sheet_xml_path(sheet).ok_or_else(|| {
            Box::new(crate::errors::ErrSheetNotExist {
                sheet_name: sheet.to_string(),
            }) as Box<dyn std::error::Error + Send + Sync>
        })?;
        let mut ws = self.work_sheet_reader(sheet)?;
        let del_cells = flat_sqref(range_ref)?;
        let mut i = 0;
        while i < ws.conditional_formatting.len() {
            let sqref = ws.conditional_formatting[i]
                .sqref
                .as_deref()
                .unwrap_or("")
                .to_string();
            let new_sqref = delete_cells_from_sqref(&sqref, &del_cells)?;
            if new_sqref.is_empty() {
                ws.conditional_formatting.remove(i);
            } else {
                ws.conditional_formatting[i].sqref = Some(new_sqref);
                i += 1;
            }
        }
        if let Some(ext_lst) = ws.ext_lst.as_mut() {
            let mut idx = 0;
            while idx < ext_lst.ext.len() {
                if ext_lst.ext[idx].uri.as_deref() == Some(EXT_URI_CONDITIONAL_FORMATTINGS) {
                    let content = ext_lst.ext[idx].content.clone();
                    let decoded = parse_x14_conditional_formattings(&content)?;
                    let new_content = delete_x14_cf_rule(&decoded, &del_cells)?;
                    ext_lst.ext[idx].content = new_content;
                    if ext_lst.ext[idx].content.len() == 57 {
                        ext_lst.ext.remove(idx);
                        continue;
                    }
                }
                idx += 1;
            }
            sort_ext_lst(ext_lst);
            if ext_lst.ext.is_empty() {
                ws.ext_lst = None;
            }
        }
        self.sheet.insert(path, ws);
        Ok(())
    }

    /// Return the preferred hex color code from a hex color, indexed color, or
    /// theme color.
    ///
    /// Equivalent to Go `GetBaseColor`.
    pub fn get_base_color(
        &self,
        hex_color: &str,
        indexed_color: i64,
        theme_color: Option<i64>,
    ) -> String {
        if let Ok(Some(theme)) = self.theme_reader() {
            if let Some(tc) = theme_color {
                let clr_scheme = &theme.theme_elements.clr_scheme;
                let choices: [(i64, &crate::xml::theme::DecodeCtColor); 10] = [
                    (0, &clr_scheme.lt1),
                    (1, &clr_scheme.dk1),
                    (2, &clr_scheme.lt2),
                    (3, &clr_scheme.dk2),
                    (4, &clr_scheme.accent1),
                    (5, &clr_scheme.accent2),
                    (6, &clr_scheme.accent3),
                    (7, &clr_scheme.accent4),
                    (8, &clr_scheme.accent5),
                    (9, &clr_scheme.accent6),
                ];
                for (idx, clr) in choices {
                    if idx == tc {
                        if let Some(val) = decode_ct_color_choice(clr) {
                            return val;
                        }
                        break;
                    }
                }
            }
        }
        if hex_color.len() == 6 {
            return hex_color.to_string();
        }
        if hex_color.len() == 8 {
            return hex_color
                .strip_prefix("FF")
                .unwrap_or(hex_color)
                .to_string();
        }
        if let Ok(styles) = self.styles_reader() {
            if let Some(colors) = styles.colors {
                if let Some(indexed) = colors.indexed_colors {
                    if let Some(rgb) = indexed.rgb_color.get(indexed_color as usize) {
                        if let Some(rgb_val) = rgb.rgb.as_ref() {
                            return rgb_val.strip_prefix("FF").unwrap_or(rgb_val).to_string();
                        }
                    }
                }
            }
        }
        if let Some(color) = INDEXED_COLOR_MAPPING.get(indexed_color as usize) {
            return color.to_string();
        }
        hex_color.to_string()
    }
}

// ------------------------------------------------------------------
// Validation
// ------------------------------------------------------------------

fn parse_format_style_set(style: &Style) -> Result<Style> {
    let fs = style.clone();
    if let Some(ref font) = fs.font {
        if count_utf16_string(font.name.as_deref().unwrap_or("")) > MAX_FONT_FAMILY_LENGTH {
            return Err(Box::new(ErrFontLength));
        }
        if let Some(size) = font.size {
            if size > MAX_FONT_SIZE as f64 {
                return Err(Box::new(ErrFontSize));
            }
        }
    }
    match fs.fill.r#type.as_str() {
        "gradient" => {
            if fs.fill.color.len() != 2 {
                return Err(Box::new(ErrFillGradientColor));
            }
            if fs.fill.shading < 0 || fs.fill.shading > 16 {
                return Err(Box::new(ErrFillGradientShading));
            }
        }
        "pattern" => {
            if fs.fill.color.len() > 1 {
                return Err(Box::new(ErrFillPatternColor));
            }
            if fs.fill.pattern < 0 || fs.fill.pattern > 18 {
                return Err(Box::new(ErrFillPattern));
            }
        }
        "" => {}
        _ => return Err(Box::new(ErrFillType)),
    }
    if let Some(ref cnf) = fs.custom_num_fmt {
        if cnf.is_empty() {
            return Err(Box::new(ErrCustomNumFmt));
        }
    }
    Ok(fs)
}

// ------------------------------------------------------------------
// XML builders
// ------------------------------------------------------------------

fn new_font(font: &Font) -> XlsxFont {
    let mut xfont = XlsxFont {
        family: Some(AttrValInt { val: Some(2) }),
        ..Default::default()
    };
    if let Some(size) = font.size {
        if size >= MIN_FONT_SIZE as f64 {
            xfont.sz = Some(AttrValFloat { val: Some(size) });
        }
    }
    if let Some(name) = &font.name {
        xfont.name = Some(AttrValString {
            val: Some(name.clone()),
        });
    }
    if let Some(charset) = font.charset {
        xfont.charset = Some(AttrValInt { val: Some(charset) });
    }
    xfont.color = new_font_color(font);
    if let Some(bold) = font.bold {
        xfont.b = Some(AttrValBool { val: Some(bold) });
    }
    if let Some(italic) = font.italic {
        xfont.i = Some(AttrValBool { val: Some(italic) });
    }
    if let Some(strike) = font.strike {
        xfont.strike = Some(AttrValBool { val: Some(strike) });
    }
    if let Some(underline) = &font.underline {
        if SUPPORTED_UNDERLINE_TYPES.contains(&underline.as_str()) {
            xfont.u = Some(AttrValString {
                val: Some(underline.clone()),
            });
        }
    }
    if let Some(vert_align) = &font.vert_align {
        if SUPPORTED_VERT_ALIGN_TYPES.contains(&vert_align.as_str()) {
            xfont.vert_align = Some(AttrValString {
                val: Some(vert_align.clone()),
            });
        }
    }
    xfont
}

fn new_font_color(font: &Font) -> Option<XlsxColor> {
    let mut color: Option<XlsxColor> = None;
    if let Some(c) = &font.color {
        color.get_or_insert_default().rgb = Some(get_palette_color(c));
    }
    if let Some(indexed) = font.color_indexed {
        if indexed >= 0 && indexed <= (INDEXED_COLOR_MAPPING.len() as i64 + 1) {
            color.get_or_insert_default().indexed = Some(indexed);
        }
    }
    if let Some(theme) = font.color_theme {
        color.get_or_insert_default().theme = Some(theme);
    }
    if font.color_tint != 0.0 {
        color.get_or_insert_default().tint = Some(font.color_tint);
    }
    color
}

fn decode_ct_color_choice(clr: &crate::xml::theme::DecodeCtColor) -> Option<String> {
    if let Some(srgb) = &clr.srgb_clr {
        if let Some(val) = &srgb.val {
            return Some(val.clone());
        }
    }
    if let Some(sys) = &clr.sys_clr {
        return Some(sys.last_clr.clone());
    }
    None
}

pub(crate) fn get_palette_color(color: &str) -> String {
    format!("FF{}", color.to_uppercase().replace('#', ""))
}

fn get_font_id(style_sheet: &XlsxStyleSheet, font: &Font) -> i32 {
    if style_sheet.fonts.is_none() {
        return -1;
    }
    let target = new_font(font);
    for (idx, fnt) in style_sheet.fonts.as_ref().unwrap().font.iter().enumerate() {
        if *fnt == target {
            return idx as i32;
        }
    }
    -1
}

fn new_fills(style: &Style, fg: bool) -> Option<XlsxFill> {
    let mut fill = XlsxFill::default();
    match style.fill.r#type.as_str() {
        "gradient" => {
            let mut variants = style_fill_variants();
            let shading = style.fill.shading as usize;
            if shading >= variants.len() {
                return None;
            }
            let mut gradient = variants.remove(shading);
            if let Some(c) = style.fill.color.get(0) {
                if gradient.stop.is_empty() {
                    gradient.stop.push(XlsxGradientFillStop::default());
                }
                gradient.stop[0].color = Some(XlsxColor {
                    rgb: Some(get_palette_color(c)),
                    ..Default::default()
                });
            }
            if let Some(c) = style.fill.color.get(1) {
                if gradient.stop.len() < 2 {
                    gradient.stop.push(XlsxGradientFillStop {
                        position: 1.0,
                        color: None,
                    });
                }
                gradient.stop[1].color = Some(XlsxColor {
                    rgb: Some(get_palette_color(c)),
                    ..Default::default()
                });
            }
            if gradient.stop.len() == 3 {
                if let Some(c) = style.fill.color.get(0) {
                    gradient.stop[2].color = Some(XlsxColor {
                        rgb: Some(get_palette_color(c)),
                        ..Default::default()
                    });
                }
            }
            fill.gradient_fill = Some(gradient);
        }
        "pattern" => {
            let mut pattern = XlsxPatternFill::default();
            let pattern_idx = style.fill.pattern as usize;
            if pattern_idx < STYLE_FILL_PATTERNS.len() {
                pattern.pattern_type = Some(STYLE_FILL_PATTERNS[pattern_idx].to_string());
            }
            if style.fill.color.is_empty() {
                if style.fill.pattern == 1 {
                    pattern.fg_color = Some(XlsxColor {
                        auto: Some(true),
                        ..Default::default()
                    });
                    pattern.bg_color = Some(XlsxColor {
                        auto: Some(true),
                        ..Default::default()
                    });
                }
            } else if fg {
                pattern.fg_color = Some(XlsxColor {
                    rgb: Some(get_palette_color(&style.fill.color[0])),
                    ..Default::default()
                });
            } else {
                pattern.bg_color = Some(XlsxColor {
                    rgb: Some(get_palette_color(&style.fill.color[0])),
                    ..Default::default()
                });
            }
            fill.pattern_fill = Some(pattern);
        }
        _ => return None,
    }
    Some(fill)
}

fn get_fill_id(style_sheet: &XlsxStyleSheet, style: &Style) -> i32 {
    if style_sheet.fills.is_none() || style.fill.r#type.is_empty() {
        return -1;
    }
    let target = match new_fills(style, true) {
        Some(f) => f,
        None => return -1,
    };
    for (idx, fill) in style_sheet.fills.as_ref().unwrap().fill.iter().enumerate() {
        if *fill == target {
            return idx as i32;
        }
    }
    -1
}

fn new_borders(style: &Style) -> XlsxBorder {
    let mut border = XlsxBorder::default();
    for v in &style.border {
        if v.style >= 0 && v.style < STYLE_BORDERS.len() as i64 {
            let line = XlsxLine {
                style: Some(STYLE_BORDERS[v.style as usize].to_string()),
                color: Some(XlsxColor {
                    rgb: Some(get_palette_color(&v.color)),
                    ..Default::default()
                }),
            };
            match v.r#type.as_str() {
                "left" => border.left = Some(line),
                "right" => border.right = Some(line),
                "top" => border.top = Some(line),
                "bottom" => border.bottom = Some(line),
                "diagonalUp" => {
                    border.diagonal = Some(line);
                    border.diagonal_up = Some(true);
                }
                "diagonalDown" => {
                    border.diagonal = Some(line);
                    border.diagonal_down = Some(true);
                }
                _ => {}
            }
        }
    }
    border
}

fn get_border_id(style_sheet: &XlsxStyleSheet, style: &Style) -> i32 {
    if style_sheet.borders.is_none() || style.border.is_empty() {
        return -1;
    }
    let target = new_borders(style);
    for (idx, border) in style_sheet
        .borders
        .as_ref()
        .unwrap()
        .border
        .iter()
        .enumerate()
    {
        if *border == target {
            return idx as i32;
        }
    }
    -1
}

fn new_alignment(style: &Style) -> Option<XlsxAlignment> {
    style.alignment.as_ref().map(|a| XlsxAlignment {
        horizontal: Some(a.horizontal.clone()).filter(|s| !s.is_empty()),
        indent: Some(a.indent).filter(|&v| v != 0),
        justify_last_line: Some(a.justify_last_line).filter(|&v| v),
        reading_order: Some(a.reading_order).filter(|&v| v != 0),
        relative_indent: Some(a.relative_indent).filter(|&v| v != 0),
        shrink_to_fit: Some(a.shrink_to_fit).filter(|&v| v),
        text_rotation: Some(a.text_rotation).filter(|&v| v != 0),
        vertical: Some(a.vertical.clone()).filter(|s| !s.is_empty()),
        wrap_text: Some(a.wrap_text).filter(|&v| v),
    })
}

fn new_protection(style: &Style) -> Option<XlsxProtection> {
    style.protection.as_ref().map(|p| XlsxProtection {
        hidden: Some(p.hidden).filter(|&v| v),
        locked: Some(p.locked).filter(|&v| v),
    })
}

fn set_cell_xfs(
    style_sheet: &mut XlsxStyleSheet,
    font_id: i32,
    num_fmt_id: i32,
    fill_id: i32,
    border_id: i32,
    apply_alignment: bool,
    apply_protection: bool,
    alignment: Option<XlsxAlignment>,
    protection: Option<XlsxProtection>,
) -> Result<i32> {
    ensure_cell_xfs(style_sheet);
    let cell_xfs = style_sheet.cell_xfs.as_mut().unwrap();
    if cell_xfs.xf.len() as i32 >= MAX_CELL_STYLES {
        return Err(Box::new(ErrCellStyles));
    }
    let mut xf = XlsxXf::default();
    xf.font_id = Some(font_id as i64);
    if font_id != 0 {
        xf.apply_font = Some(true);
    }
    xf.num_fmt_id = Some(num_fmt_id as i64);
    if num_fmt_id != 0 {
        xf.apply_number_format = Some(true);
    }
    xf.fill_id = Some(fill_id as i64);
    if fill_id != 0 {
        xf.apply_fill = Some(true);
    }
    xf.border_id = Some(border_id as i64);
    if border_id != 0 {
        xf.apply_border = Some(true);
    }
    xf.xf_id = Some(0);
    xf.alignment = alignment;
    if apply_alignment {
        xf.apply_alignment = Some(true);
    }
    if apply_protection {
        xf.apply_protection = Some(true);
        xf.protection = protection;
    }
    cell_xfs.xf.push(xf);
    cell_xfs.count = cell_xfs.xf.len() as i64;
    Ok((cell_xfs.count - 1) as i32)
}

// ------------------------------------------------------------------
// Deduplication
// ------------------------------------------------------------------

fn get_style_id(style_sheet: &XlsxStyleSheet, style: &Style) -> i32 {
    if style_sheet.cell_xfs.is_none() {
        return -1;
    }
    if style.font.is_none() {
        return -1;
    }
    let num_fmt_id = get_num_fmt_id(style_sheet, style);
    let border_id = get_border_id(style_sheet, style);
    let fill_id = get_fill_id(style_sheet, style);
    let font_id = get_font_id(style_sheet, style.font.as_ref().unwrap());
    let num_fmt_id_for_custom = if style.custom_num_fmt.is_some() {
        get_custom_num_fmt_id(style_sheet, style)
    } else {
        num_fmt_id
    };

    for (idx, xf) in style_sheet.cell_xfs.as_ref().unwrap().xf.iter().enumerate() {
        if !match_num_fmt(num_fmt_id_for_custom, xf, style) {
            continue;
        }
        if !match_font(font_id, xf, style) {
            continue;
        }
        if !match_fill(fill_id, xf, style) {
            continue;
        }
        if !match_border(border_id, xf, style) {
            continue;
        }
        if !match_alignment(xf, style) {
            continue;
        }
        if !match_protection(xf, style) {
            continue;
        }
        return idx as i32;
    }
    -1
}

fn match_num_fmt(num_fmt_id: i32, xf: &XlsxXf, style: &Style) -> bool {
    if style.custom_num_fmt.is_none() && num_fmt_id == -1 {
        return xf.num_fmt_id.map(|v| v == 0).unwrap_or(false);
    }
    if style.neg_red || style.decimal_places.map(|v| v != 2).unwrap_or(false) {
        return false;
    }
    xf.num_fmt_id == Some(num_fmt_id as i64)
}

fn match_font(font_id: i32, xf: &XlsxXf, style: &Style) -> bool {
    if style.font.is_none() || font_id == 0 {
        return (xf.font_id.is_none() || xf.font_id == Some(0))
            && (xf.apply_font.is_none() || !xf.apply_font.unwrap());
    }
    xf.font_id == Some(font_id as i64) && xf.apply_font == Some(true)
}

fn match_fill(fill_id: i32, xf: &XlsxXf, style: &Style) -> bool {
    if style.fill.r#type.is_empty() || fill_id == 0 {
        return (xf.fill_id.is_none() || xf.fill_id == Some(0))
            && (xf.apply_fill.is_none() || !xf.apply_fill.unwrap());
    }
    xf.fill_id == Some(fill_id as i64) && xf.apply_fill == Some(true)
}

fn match_border(border_id: i32, xf: &XlsxXf, style: &Style) -> bool {
    if style.border.is_empty() {
        return (xf.border_id.is_none() || xf.border_id == Some(0))
            && (xf.apply_border.is_none() || !xf.apply_border.unwrap());
    }
    xf.border_id == Some(border_id as i64) && xf.apply_border == Some(true)
}

fn match_alignment(xf: &XlsxXf, style: &Style) -> bool {
    if style.alignment.is_none() {
        return xf.apply_alignment.is_none() || !xf.apply_alignment.unwrap();
    }
    xf.alignment == new_alignment(style)
}

fn match_protection(xf: &XlsxXf, style: &Style) -> bool {
    if style.protection.is_none() {
        return xf.apply_protection.is_none() || !xf.apply_protection.unwrap();
    }
    xf.protection == new_protection(style) && xf.apply_protection == Some(true)
}

// ------------------------------------------------------------------
// Number formats
// ------------------------------------------------------------------

static BUILT_IN_NUM_FMT: LazyLock<HashMap<i32, &'static str>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert(0, "general");
    m.insert(1, "0");
    m.insert(2, "0.00");
    m.insert(3, "#,##0");
    m.insert(4, "#,##0.00");
    m.insert(9, "0%");
    m.insert(10, "0.00%");
    m.insert(11, "0.00E+00");
    m.insert(12, "# ?/?");
    m.insert(13, "# ??/??");
    m.insert(14, "mm-dd-yy");
    m.insert(15, "d-mmm-yy");
    m.insert(16, "d-mmm");
    m.insert(17, "mmm-yy");
    m.insert(18, "h:mm AM/PM");
    m.insert(19, "h:mm:ss AM/PM");
    m.insert(20, "hh:mm");
    m.insert(21, "hh:mm:ss");
    m.insert(22, "m/d/yy hh:mm");
    m.insert(37, "#,##0 ;(#,##0)");
    m.insert(38, "#,##0 ;[red](#,##0)");
    m.insert(39, "#,##0.00 ;(#,##0.00)");
    m.insert(40, "#,##0.00 ;[red](#,##0.00)");
    m.insert(41, "_(* #,##0_);_(* \\(#,##0\\);_(* \"-\"_);_(@_)");
    m.insert(
        42,
        "_(\"$\"* #,##0_);_(\"$\"* \\(#,##0\\);_(\"$\"* \"-\"_);_(@_)",
    );
    m.insert(43, "_(* #,##0.00_);_(* \\(#,##0.00\\);_(* \"-\"??_);_(@_)");
    m.insert(
        44,
        "_(\"$\"* #,##0.00_);_(\"$\"* \\(#,##0.00\\);_(\"$\"* \"-\"??_);_(@_)",
    );
    m.insert(45, "mm:ss");
    m.insert(46, "[h]:mm:ss");
    m.insert(47, "mm:ss.0");
    m.insert(48, "##0.0E+0");
    m.insert(49, "@");
    m
});

static CURRENCY_NUM_FMT: LazyLock<HashMap<i32, &'static str>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert(164, "\"\u{00A5}\"#,##0.00");
    m.insert(165, "[$$-409]#,##0.00");
    m.insert(166, "[$$-45C]#,##0.00");
    m.insert(167, "[$$-1004]#,##0.00");
    m.insert(168, "[$$-404]#,##0.00");
    m.insert(169, "[$$-C09]#,##0.00");
    m.insert(170, "[$$-2809]#,##0.00");
    m.insert(171, "[$$-1009]#,##0.00");
    m.insert(172, "[$$-2009]#,##0.00");
    m.insert(173, "[$$-1409]#,##0.00");
    m.insert(174, "[$$-4809]#,##0.00");
    m.insert(175, "[$$-2C09]#,##0.00");
    m.insert(176, "[$$-2409]#,##0.00");
    m.insert(177, "[$$-1000]#,##0.00");
    m.insert(178, "#,##0.00\\ [$$-C0C]");
    m.insert(179, "[$$-475]#,##0.00");
    m.insert(180, "[$$-83E]#,##0.00");
    m.insert(181, "[$$-86B]\\ #,##0.00");
    m.insert(182, "[$$-340A]\\ #,##0.00");
    m.insert(183, "[$$-240A]#,##0.00");
    m.insert(184, "[$$-300A]\\ #,##0.00");
    m.insert(185, "[$$-440A]#,##0.00");
    m.insert(186, "[$$-80A]#,##0.00");
    m.insert(187, "[$$-500A]#,##0.00");
    m.insert(188, "[$$-540A]#,##0.00");
    m.insert(189, "[$$-380A]\\ #,##0.00");
    m.insert(190, "[$\u{00A3}-809]#,##0.00");
    m.insert(191, "[$\u{00A3}-491]#,##0.00");
    m.insert(192, "[$\u{00A3}-452]#,##0.00");
    m.insert(193, "[$\u{00A5}-804]#,##0.00");
    m.insert(194, "[$\u{00A5}-411]#,##0.00");
    m.insert(195, "[$\u{00A5}-478]#,##0.00");
    m.insert(196, "[$\u{00A5}-451]#,##0.00");
    m.insert(197, "[$\u{00A5}-480]#,##0.00");
    m.insert(198, "#,##0.00\\ [$\u{058F}-42B]");
    m.insert(199, "[$\u{060B}-463]#,##0.00");
    m.insert(200, "[$\u{060B}-48C]#,##0.00");
    m.insert(201, "[$\u{09F3}-845]\\ #,##0.00");
    m.insert(202, "#,##0.00[$\u{17DB}-453]");
    m.insert(203, "[$\u{20A1}-140A]#,##0.00");
    m.insert(204, "[$\u{20A6}-468]\\ #,##0.00");
    m.insert(205, "[$\u{20A6}-470]\\ #,##0.00");
    m.insert(206, "[$\u{20A9}-412]#,##0.00");
    m.insert(207, "[$\u{20AA}-40D]\\ #,##0.00");
    m.insert(208, "#,##0.00\\ [$\u{20AB}-42A]");
    m.insert(209, "#,##0.00\\ [$\u{20AC}-42D]");
    m.insert(210, "#,##0.00\\ [$\u{20AC}-47E]");
    m.insert(211, "#,##0.00\\ [$\u{20AC}-403]");
    m.insert(212, "#,##0.00\\ [$\u{20AC}-483]");
    m.insert(213, "[$\u{20AC}-813]\\ #,##0.00");
    m.insert(214, "[$\u{20AC}-413]\\ #,##0.00");
    m.insert(215, "[$\u{20AC}-1809]#,##0.00");
    m.insert(216, "#,##0.00\\ [$\u{20AC}-425]");
    m.insert(217, "[$\u{20AC}-2]\\ #,##0.00");
    m.insert(218, "#,##0.00\\ [$\u{20AC}-1]");
    m.insert(219, "#,##0.00\\ [$\u{20AC}-40B]");
    m.insert(220, "#,##0.00\\ [$\u{20AC}-80C]");
    m.insert(221, "#,##0.00\\ [$\u{20AC}-40C]");
    m.insert(222, "#,##0.00\\ [$\u{20AC}-140C]");
    m.insert(223, "#,##0.00\\ [$\u{20AC}-180C]");
    m.insert(224, "[$\u{20AC}-200C]#,##0.00");
    m.insert(225, "#,##0.00\\ [$\u{20AC}-456]");
    m.insert(226, "#,##0.00\\ [$\u{20AC}-C07]");
    m.insert(227, "#,##0.00\\ [$\u{20AC}-407]");
    m.insert(228, "#,##0.00\\ [$\u{20AC}-1007]");
    m.insert(229, "#,##0.00\\ [$\u{20AC}-408]");
    m.insert(230, "#,##0.00\\ [$\u{20AC}-243B]");
    m.insert(231, "[$\u{20AC}-83C]#,##0.00");
    m.insert(232, "[$\u{20AC}-410]\\ #,##0.00");
    m.insert(233, "[$\u{20AC}-476]#,##0.00");
    m.insert(234, "#,##0.00\\ [$\u{20AC}-2C1A]");
    m.insert(235, "[$\u{20AC}-426]\\ #,##0.00");
    m.insert(236, "#,##0.00\\ [$\u{20AC}-427]");
    m.insert(237, "#,##0.00\\ [$\u{20AC}-82E]");
    m.insert(238, "#,##0.00\\ [$\u{20AC}-46E]");
    m.insert(239, "[$\u{20AC}-43A]#,##0.00");
    m.insert(240, "#,##0.00\\ [$\u{20AC}-C3B]");
    m.insert(241, "#,##0.00\\ [$\u{20AC}-482]");
    m.insert(242, "#,##0.00\\ [$\u{20AC}-816]");
    m.insert(243, "#,##0.00\\ [$\u{20AC}-301A]");
    m.insert(244, "#,##0.00\\ [$\u{20AC}-203B]");
    m.insert(245, "[$\u{20AC}-41B]#,##0.00");
    m.insert(246, "#,##0.00\\ [$\u{20AC}-424]");
    m.insert(247, "#,##0.00\\ [$\u{20AC}-C0A]");
    m.insert(248, "#,##0.00\\ [$\u{20AC}-81D]");
    m.insert(249, "#,##0.00\\ [$\u{20AC}-484]");
    m.insert(250, "#,##0.00\\ [$\u{20AC}-42E]");
    m.insert(251, "[$\u{20AC}-462]\\ #,##0.00");
    m.insert(252, "#,##0.00\\ [$\u{20AD}-454]");
    m.insert(253, "#,##0.00\\ [$\u{20AE}-450]");
    m.insert(254, "[$\u{20AE}-C50]#,##0.00");
    m.insert(255, "[$\u{20B1}-3409]#,##0.00");
    m.insert(256, "[$\u{20B1}-464]#,##0.00");
    m.insert(257, "#,##0.00[$\u{20B4}-422]");
    m.insert(258, "[$\u{20B8}-43F]#,##0.00");
    m.insert(259, "[$\u{20B9}-460]#,##0.00");
    m.insert(260, "[$\u{20B9}-4009]\\ #,##0.00");
    m.insert(261, "[$\u{20B9}-447]\\ #,##0.00");
    m.insert(262, "[$\u{20B9}-439]\\ #,##0.00");
    m.insert(263, "[$\u{20B9}-44B]\\ #,##0.00");
    m.insert(264, "[$\u{20B9}-860]#,##0.00");
    m.insert(265, "[$\u{20B9}-457]\\ #,##0.00");
    m.insert(266, "[$\u{20B9}-458]#,##0.00");
    m.insert(267, "[$\u{20B9}-44E]\\ #,##0.00");
    m.insert(268, "[$\u{20B9}-861]#,##0.00");
    m.insert(269, "[$\u{20B9}-448]\\ #,##0.00");
    m.insert(270, "[$\u{20B9}-446]\\ #,##0.00");
    m.insert(271, "[$\u{20B9}-44F]\\ #,##0.00");
    m.insert(272, "[$\u{20B9}-459]#,##0.00");
    m.insert(273, "[$\u{20B9}-449]\\ #,##0.00");
    m.insert(274, "[$\u{20B9}-820]#,##0.00");
    m.insert(275, "#,##0.00\\ [$\u{20BA}-41F]");
    m.insert(276, "#,##0.00\\ [$\u{20BC}-42C]");
    m.insert(277, "#,##0.00\\ [$\u{20BC}-82C]");
    m.insert(278, "#,##0.00\\ [$\u{20BD}-419]");
    m.insert(279, "#,##0.00[$\u{20BD}-485]");
    m.insert(280, "#,##0.00\\ [$\u{20BE}-437]");
    m.insert(281, "[$B/.-180A]\\ #,##0.00");
    m.insert(282, "[$Br-472]#,##0.00");
    m.insert(283, "[$Br-477]#,##0.00");
    m.insert(284, "#,##0.00[$Br-473]");
    m.insert(285, "[$Bs-46B]\\ #,##0.00");
    m.insert(286, "[$Bs-400A]\\ #,##0.00");
    m.insert(287, "[$Bs.-200A]\\ #,##0.00");
    m.insert(288, "[$BWP-832]\\ #,##0.00");
    m.insert(289, "[$C$-4C0A]#,##0.00");
    m.insert(290, "[$CA$-85D]#,##0.00");
    m.insert(291, "[$CA$-47C]#,##0.00");
    m.insert(292, "[$CA$-45D]#,##0.00");
    m.insert(293, "[$CFA-340C]#,##0.00");
    m.insert(294, "[$CFA-280C]#,##0.00");
    m.insert(295, "#,##0.00\\ [$CFA-867]");
    m.insert(296, "#,##0.00\\ [$CFA-488]");
    m.insert(297, "#,##0.00\\ [$CHF-100C]");
    m.insert(298, "[$CHF-1407]\\ #,##0.00");
    m.insert(299, "[$CHF-807]\\ #,##0.00");
    m.insert(300, "[$CHF-810]\\ #,##0.00");
    m.insert(301, "[$CHF-417]\\ #,##0.00");
    m.insert(302, "[$CLP-47A]\\ #,##0.00");
    m.insert(303, "[$CN\u{00A5}-850]#,##0.00");
    m.insert(304, "#,##0.00\\ [$DZD-85F]");
    m.insert(305, "[$FCFA-2C0C]#,##0.00");
    m.insert(306, "#,##0.00\\ [$Ft-40E]");
    m.insert(307, "[$G-3C0C]#,##0.00");
    m.insert(308, "[$Gs.-3C0A]\\ #,##0.00");
    m.insert(309, "[$GTQ-486]#,##0.00");
    m.insert(310, "[$HK$-C04]#,##0.00");
    m.insert(311, "[$HK$-3C09]#,##0.00");
    m.insert(312, "#,##0.00\\ [$HRK-41A]");
    m.insert(313, "[$IDR-3809]#,##0.00");
    m.insert(314, "[$IQD-492]#,##0.00");
    m.insert(315, "#,##0.00\\ [$ISK-40F]");
    m.insert(316, "[$K-455]#,##0.00");
    m.insert(317, "#,##0.00\\ [$K\u{010D}-405]");
    m.insert(318, "#,##0.00\\ [$KM-141A]");
    m.insert(319, "#,##0.00\\ [$KM-101A]");
    m.insert(320, "#,##0.00\\ [$KM-181A]");
    m.insert(321, "[$kr-438]\\ #,##0.00");
    m.insert(322, "[$kr-43B]\\ #,##0.00");
    m.insert(323, "#,##0.00\\ [$kr-83B]");
    m.insert(324, "[$kr-414]\\ #,##0.00");
    m.insert(325, "[$kr-814]\\ #,##0.00");
    m.insert(326, "#,##0.00\\ [$kr-41D]");
    m.insert(327, "[$kr.-406]\\ #,##0.00");
    m.insert(328, "[$kr.-46F]\\ #,##0.00");
    m.insert(329, "[$Ksh-441]#,##0.00");
    m.insert(330, "[$L-818]#,##0.00");
    m.insert(331, "[$L-819]#,##0.00");
    m.insert(332, "[$L-480A]\\ #,##0.00");
    m.insert(333, "#,##0.00\\ [$Lek\u{00EB}-41C]");
    m.insert(334, "[$MAD-45F]#,##0.00");
    m.insert(335, "[$MAD-380C]#,##0.00");
    m.insert(336, "#,##0.00\\ [$MAD-105F]");
    m.insert(337, "[$MOP$-1404]#,##0.00");
    m.insert(338, "#,##0.00\\ [$MVR-465]_-");
    m.insert(339, "#,##0.00[$Nfk-873]");
    m.insert(340, "[$NGN-466]#,##0.00");
    m.insert(341, "[$NGN-467]#,##0.00");
    m.insert(342, "[$NGN-469]#,##0.00");
    m.insert(343, "[$NGN-471]#,##0.00");
    m.insert(344, "[$NOK-103B]\\ #,##0.00");
    m.insert(345, "[$NOK-183B]\\ #,##0.00");
    m.insert(346, "[$NZ$-481]#,##0.00");
    m.insert(347, "[$PKR-859]\\ #,##0.00");
    m.insert(348, "[$PYG-474]#,##0.00");
    m.insert(349, "[$Q-100A]#,##0.00");
    m.insert(350, "[$R-436]\\ #,##0.00");
    m.insert(351, "[$R-1C09]\\ #,##0.00");
    m.insert(352, "[$R-435]\\ #,##0.00");
    m.insert(353, "[$R$-416]\\ #,##0.00");
    m.insert(354, "[$RD$-1C0A]#,##0.00");
    m.insert(355, "#,##0.00\\ [$RF-487]");
    m.insert(356, "[$RM-4409]#,##0.00");
    m.insert(357, "[$RM-43E]#,##0.00");
    m.insert(358, "#,##0.00\\ [$RON-418]");
    m.insert(359, "[$Rp-421]#,##0.00");
    m.insert(360, "[$Rs-420]#,##0.00_-");
    m.insert(361, "[$Rs.-849]\\ #,##0.00");
    m.insert(362, "#,##0.00\\ [$RSD-81A]");
    m.insert(363, "#,##0.00\\ [$RSD-C1A]");
    m.insert(364, "#,##0.00\\ [$RUB-46D]");
    m.insert(365, "#,##0.00\\ [$RUB-444]");
    m.insert(366, "[$S/.-C6B]\\ #,##0.00");
    m.insert(367, "[$S/.-280A]\\ #,##0.00");
    m.insert(368, "#,##0.00\\ [$SEK-143B]");
    m.insert(369, "#,##0.00\\ [$SEK-1C3B]");
    m.insert(370, "#,##0.00\\ [$so\u{02BB}m-443]");
    m.insert(371, "#,##0.00\\ [$so\u{02BB}m-843]");
    m.insert(372, "#,##0.00\\ [$SYP-45A]");
    m.insert(373, "[$THB-41E]#,##0.00");
    m.insert(374, "#,##0.00[$TMT-442]");
    m.insert(375, "[$US$-3009]#,##0.00");
    m.insert(376, "[$ZAR-46C]\\ #,##0.00");
    m.insert(377, "[$ZAR-430]#,##0.00");
    m.insert(378, "[$ZAR-431]#,##0.00");
    m.insert(379, "[$ZAR-432]\\ #,##0.00");
    m.insert(380, "[$ZAR-433]#,##0.00");
    m.insert(381, "[$ZAR-434]\\ #,##0.00");
    m.insert(382, "#,##0.00\\ [$z\u{0142}-415]");
    m.insert(383, "#,##0.00\\ [$\u{0434}\u{0435}\u{043D}-42F]");
    m.insert(384, "#,##0.00\\ [$\u{041A}\u{041C}-201A]");
    m.insert(385, "#,##0.00\\ [$\u{041A}\u{041C}-1C1A]");
    m.insert(386, "#,##0.00\\ [$\u{043B}\u{0432}.-402]");
    m.insert(387, "#,##0.00\\ [$\u{0440}.-423]");
    m.insert(388, "#,##0.00\\ [$\u{0441}\u{043E}\u{043C}-440]");
    m.insert(389, "#,##0.00\\ [$\u{0441}\u{043E}\u{043C}-428]");
    m.insert(390, "[$\u{062C}.\u{0645}.-C01]\\ #,##0.00_-");
    m.insert(391, "[$\u{062F}.\u{0623}.-2C01]\\ #,##0.00_-");
    m.insert(392, "[$\u{062F}.\u{0625}.-3801]\\ #,##0.00_-");
    m.insert(393, "[$\u{062F}.\u{0628}.-3C01]\\ #,##0.00_-");
    m.insert(394, "[$\u{062F}.\u{062A}.-1C01]\\ #,##0.00_-");
    m.insert(395, "[$\u{062F}.\u{062C}.-1401]\\ #,##0.00_-");
    m.insert(396, "[$\u{062F}.\u{0639}.-801]\\ #,##0.00_-");
    m.insert(397, "[$\u{062F}.\u{0643}.-3401]\\ #,##0.00_-");
    m.insert(398, "[$\u{062F}.\u{0644}.-1001]#,##0.00_-");
    m.insert(399, "[$\u{062F}.\u{0645}.-1801]\\ #,##0.00_-");
    m.insert(400, "[$\u{0631}-846]\\ #,##0.00");
    m.insert(401, "[$\u{0631}.\u{0633}.-401]\\ #,##0.00_-");
    m.insert(402, "[$\u{0631}.\u{0639}.-2001]\\ #,##0.00_-");
    m.insert(403, "[$\u{0631}.\u{0642}.-4001]\\ #,##0.00_-");
    m.insert(404, "[$\u{0631}.\u{064A}.-2401]\\ #,##0.00_-");
    m.insert(405, "[$\u{0631}\u{06CC}\u{0627}\u{0644}-429]#,##0.00_-");
    m.insert(406, "[$\u{0644}.\u{0633}.-2801]\\ #,##0.00_-");
    m.insert(407, "[$\u{0644}.\u{0644}.-3001]\\ #,##0.00_-");
    m.insert(408, "[$\u{1265}\u{122D}-45E]#,##0.00");
    m.insert(409, "[$\u{0930}\u{0942}-461]#,##0.00");
    m.insert(410, "[$\u{0DBB}\u{0DD4}.-45B]\\ #,##0.00");
    m.insert(411, "[$ADP]\\ #,##0.00");
    m.insert(412, "[$AED]\\ #,##0.00");
    m.insert(413, "[$AFA]\\ #,##0.00");
    m.insert(414, "[$AFN]\\ #,##0.00");
    m.insert(415, "[$ALL]\\ #,##0.00");
    m.insert(416, "[$AMD]\\ #,##0.00");
    m.insert(417, "[$ANG]\\ #,##0.00");
    m.insert(418, "[$AOA]\\ #,##0.00");
    m.insert(419, "[$ARS]\\ #,##0.00");
    m.insert(420, "[$ATS]\\ #,##0.00");
    m.insert(421, "[$AUD]\\ #,##0.00");
    m.insert(422, "[$AWG]\\ #,##0.00");
    m.insert(423, "[$AZM]\\ #,##0.00");
    m.insert(424, "[$AZN]\\ #,##0.00");
    m.insert(425, "[$BAM]\\ #,##0.00");
    m.insert(426, "[$BBD]\\ #,##0.00");
    m.insert(427, "[$BDT]\\ #,##0.00");
    m.insert(428, "[$BEF]\\ #,##0.00");
    m.insert(429, "[$BGL]\\ #,##0.00");
    m.insert(430, "[$BGN]\\ #,##0.00");
    m.insert(431, "[$BHD]\\ #,##0.00");
    m.insert(432, "[$BIF]\\ #,##0.00");
    m.insert(433, "[$BMD]\\ #,##0.00");
    m.insert(434, "[$BND]\\ #,##0.00");
    m.insert(435, "[$BOB]\\ #,##0.00");
    m.insert(436, "[$BOV]\\ #,##0.00");
    m.insert(437, "[$BRL]\\ #,##0.00");
    m.insert(438, "[$BSD]\\ #,##0.00");
    m.insert(439, "[$BTN]\\ #,##0.00");
    m.insert(440, "[$BWP]\\ #,##0.00");
    m.insert(441, "[$BYR]\\ #,##0.00");
    m.insert(442, "[$BZD]\\ #,##0.00");
    m.insert(443, "[$CAD]\\ #,##0.00");
    m.insert(444, "[$CDF]\\ #,##0.00");
    m.insert(445, "[$CHE]\\ #,##0.00");
    m.insert(446, "[$CHF]\\ #,##0.00");
    m.insert(447, "[$CHW]\\ #,##0.00");
    m.insert(448, "[$CLF]\\ #,##0.00");
    m.insert(449, "[$CLP]\\ #,##0.00");
    m.insert(450, "[$CNY]\\ #,##0.00");
    m.insert(451, "[$COP]\\ #,##0.00");
    m.insert(452, "[$COU]\\ #,##0.00");
    m.insert(453, "[$CRC]\\ #,##0.00");
    m.insert(454, "[$CSD]\\ #,##0.00");
    m.insert(455, "[$CUC]\\ #,##0.00");
    m.insert(456, "[$CVE]\\ #,##0.00");
    m.insert(457, "[$CYP]\\ #,##0.00");
    m.insert(458, "[$CZK]\\ #,##0.00");
    m.insert(459, "[$DEM]\\ #,##0.00");
    m.insert(460, "[$DJF]\\ #,##0.00");
    m.insert(461, "[$DKK]\\ #,##0.00");
    m.insert(462, "[$DOP]\\ #,##0.00");
    m.insert(463, "[$DZD]\\ #,##0.00");
    m.insert(464, "[$ECS]\\ #,##0.00");
    m.insert(465, "[$ECV]\\ #,##0.00");
    m.insert(466, "[$EEK]\\ #,##0.00");
    m.insert(467, "[$EGP]\\ #,##0.00");
    m.insert(468, "[$ERN]\\ #,##0.00");
    m.insert(469, "[$ESP]\\ #,##0.00");
    m.insert(470, "[$ETB]\\ #,##0.00");
    m.insert(471, "[$EUR]\\ #,##0.00");
    m.insert(472, "[$FIM]\\ #,##0.00");
    m.insert(473, "[$FJD]\\ #,##0.00");
    m.insert(474, "[$FKP]\\ #,##0.00");
    m.insert(475, "[$FRF]\\ #,##0.00");
    m.insert(476, "[$GBP]\\ #,##0.00");
    m.insert(477, "[$GEL]\\ #,##0.00");
    m.insert(478, "[$GHC]\\ #,##0.00");
    m.insert(479, "[$GHS]\\ #,##0.00");
    m.insert(480, "[$GIP]\\ #,##0.00");
    m.insert(481, "[$GMD]\\ #,##0.00");
    m.insert(482, "[$GNF]\\ #,##0.00");
    m.insert(483, "[$GRD]\\ #,##0.00");
    m.insert(484, "[$GTQ]\\ #,##0.00");
    m.insert(485, "[$GYD]\\ #,##0.00");
    m.insert(486, "[$HKD]\\ #,##0.00");
    m.insert(487, "[$HNL]\\ #,##0.00");
    m.insert(488, "[$HRK]\\ #,##0.00");
    m.insert(489, "[$HTG]\\ #,##0.00");
    m.insert(490, "[$HUF]\\ #,##0.00");
    m.insert(491, "[$IDR]\\ #,##0.00");
    m.insert(492, "[$IEP]\\ #,##0.00");
    m.insert(493, "[$ILS]\\ #,##0.00");
    m.insert(494, "[$INR]\\ #,##0.00");
    m.insert(495, "[$IQD]\\ #,##0.00");
    m.insert(496, "[$IRR]\\ #,##0.00");
    m.insert(497, "[$ISK]\\ #,##0.00");
    m.insert(498, "[$ITL]\\ #,##0.00");
    m.insert(499, "[$JMD]\\ #,##0.00");
    m.insert(500, "[$JOD]\\ #,##0.00");
    m.insert(501, "[$JPY]\\ #,##0.00");
    m.insert(502, "[$KAF]\\ #,##0.00");
    m.insert(503, "[$KES]\\ #,##0.00");
    m.insert(504, "[$KGS]\\ #,##0.00");
    m.insert(505, "[$KHR]\\ #,##0.00");
    m.insert(506, "[$KMF]\\ #,##0.00");
    m.insert(507, "[$KPW]\\ #,##0.00");
    m.insert(508, "[$KRW]\\ #,##0.00");
    m.insert(509, "[$KWD]\\ #,##0.00");
    m.insert(510, "[$KYD]\\ #,##0.00");
    m.insert(511, "[$KZT]\\ #,##0.00");
    m.insert(512, "[$LAK]\\ #,##0.00");
    m.insert(513, "[$LBP]\\ #,##0.00");
    m.insert(514, "[$LKR]\\ #,##0.00");
    m.insert(515, "[$LRD]\\ #,##0.00");
    m.insert(516, "[$LSL]\\ #,##0.00");
    m.insert(517, "[$LTL]\\ #,##0.00");
    m.insert(518, "[$LUF]\\ #,##0.00");
    m.insert(519, "[$LVL]\\ #,##0.00");
    m.insert(520, "[$LYD]\\ #,##0.00");
    m.insert(521, "[$MAD]\\ #,##0.00");
    m.insert(522, "[$MDL]\\ #,##0.00");
    m.insert(523, "[$MGA]\\ #,##0.00");
    m.insert(524, "[$MGF]\\ #,##0.00");
    m.insert(525, "[$MKD]\\ #,##0.00");
    m.insert(526, "[$MMK]\\ #,##0.00");
    m.insert(527, "[$MNT]\\ #,##0.00");
    m.insert(528, "[$MOP]\\ #,##0.00");
    m.insert(529, "[$MRO]\\ #,##0.00");
    m.insert(530, "[$MTL]\\ #,##0.00");
    m.insert(531, "[$MUR]\\ #,##0.00");
    m.insert(532, "[$MVR]\\ #,##0.00");
    m.insert(533, "[$MWK]\\ #,##0.00");
    m.insert(534, "[$MXN]\\ #,##0.00");
    m.insert(535, "[$MXV]\\ #,##0.00");
    m.insert(536, "[$MYR]\\ #,##0.00");
    m.insert(537, "[$MZM]\\ #,##0.00");
    m.insert(538, "[$MZN]\\ #,##0.00");
    m.insert(539, "[$NAD]\\ #,##0.00");
    m.insert(540, "[$NGN]\\ #,##0.00");
    m.insert(541, "[$NIO]\\ #,##0.00");
    m.insert(542, "[$NLG]\\ #,##0.00");
    m.insert(543, "[$NOK]\\ #,##0.00");
    m.insert(544, "[$NPR]\\ #,##0.00");
    m.insert(545, "[$NTD]\\ #,##0.00");
    m.insert(546, "[$NZD]\\ #,##0.00");
    m.insert(547, "[$OMR]\\ #,##0.00");
    m.insert(548, "[$PAB]\\ #,##0.00");
    m.insert(549, "[$PEN]\\ #,##0.00");
    m.insert(550, "[$PGK]\\ #,##0.00");
    m.insert(551, "[$PHP]\\ #,##0.00");
    m.insert(552, "[$PKR]\\ #,##0.00");
    m.insert(553, "[$PLN]\\ #,##0.00");
    m.insert(554, "[$PTE]\\ #,##0.00");
    m.insert(555, "[$PYG]\\ #,##0.00");
    m.insert(556, "[$QAR]\\ #,##0.00");
    m.insert(557, "[$ROL]\\ #,##0.00");
    m.insert(558, "[$RON]\\ #,##0.00");
    m.insert(559, "[$RSD]\\ #,##0.00");
    m.insert(560, "[$RUB]\\ #,##0.00");
    m.insert(561, "[$RUR]\\ #,##0.00");
    m.insert(562, "[$RWF]\\ #,##0.00");
    m.insert(563, "[$SAR]\\ #,##0.00");
    m.insert(564, "[$SBD]\\ #,##0.00");
    m.insert(565, "[$SCR]\\ #,##0.00");
    m.insert(566, "[$SDD]\\ #,##0.00");
    m.insert(567, "[$SDG]\\ #,##0.00");
    m.insert(568, "[$SDP]\\ #,##0.00");
    m.insert(569, "[$SEK]\\ #,##0.00");
    m.insert(570, "[$SGD]\\ #,##0.00");
    m.insert(571, "[$SHP]\\ #,##0.00");
    m.insert(572, "[$SIT]\\ #,##0.00");
    m.insert(573, "[$SKK]\\ #,##0.00");
    m.insert(574, "[$SLL]\\ #,##0.00");
    m.insert(575, "[$SOS]\\ #,##0.00");
    m.insert(576, "[$SPL]\\ #,##0.00");
    m.insert(577, "[$SRD]\\ #,##0.00");
    m.insert(578, "[$SRG]\\ #,##0.00");
    m.insert(579, "[$STD]\\ #,##0.00");
    m.insert(580, "[$SVC]\\ #,##0.00");
    m.insert(581, "[$SYP]\\ #,##0.00");
    m.insert(582, "[$SZL]\\ #,##0.00");
    m.insert(583, "[$THB]\\ #,##0.00");
    m.insert(584, "[$TJR]\\ #,##0.00");
    m.insert(585, "[$TJS]\\ #,##0.00");
    m.insert(586, "[$TMM]\\ #,##0.00");
    m.insert(587, "[$TMT]\\ #,##0.00");
    m.insert(588, "[$TND]\\ #,##0.00");
    m.insert(589, "[$TOP]\\ #,##0.00");
    m.insert(590, "[$TRL]\\ #,##0.00");
    m.insert(591, "[$TRY]\\ #,##0.00");
    m.insert(592, "[$TTD]\\ #,##0.00");
    m.insert(593, "[$TWD]\\ #,##0.00");
    m.insert(594, "[$TZS]\\ #,##0.00");
    m.insert(595, "[$UAH]\\ #,##0.00");
    m.insert(596, "[$UGX]\\ #,##0.00");
    m.insert(597, "[$USD]\\ #,##0.00");
    m.insert(598, "[$USN]\\ #,##0.00");
    m.insert(599, "[$USS]\\ #,##0.00");
    m.insert(600, "[$UYI]\\ #,##0.00");
    m.insert(601, "[$UYU]\\ #,##0.00");
    m.insert(602, "[$UZS]\\ #,##0.00");
    m.insert(603, "[$VEB]\\ #,##0.00");
    m.insert(604, "[$VEF]\\ #,##0.00");
    m.insert(605, "[$VND]\\ #,##0.00");
    m.insert(606, "[$VUV]\\ #,##0.00");
    m.insert(607, "[$WST]\\ #,##0.00");
    m.insert(608, "[$XAF]\\ #,##0.00");
    m.insert(609, "[$XAG]\\ #,##0.00");
    m.insert(610, "[$XAU]\\ #,##0.00");
    m.insert(611, "[$XB5]\\ #,##0.00");
    m.insert(612, "[$XBA]\\ #,##0.00");
    m.insert(613, "[$XBB]\\ #,##0.00");
    m.insert(614, "[$XBC]\\ #,##0.00");
    m.insert(615, "[$XBD]\\ #,##0.00");
    m.insert(616, "[$XCD]\\ #,##0.00");
    m.insert(617, "[$XDR]\\ #,##0.00");
    m.insert(618, "[$XFO]\\ #,##0.00");
    m.insert(619, "[$XFU]\\ #,##0.00");
    m.insert(620, "[$XOF]\\ #,##0.00");
    m.insert(621, "[$XPD]\\ #,##0.00");
    m.insert(622, "[$XPF]\\ #,##0.00");
    m.insert(623, "[$XPT]\\ #,##0.00");
    m.insert(624, "[$XTS]\\ #,##0.00");
    m.insert(625, "[$XXX]\\ #,##0.00");
    m.insert(626, "[$YER]\\ #,##0.00");
    m.insert(627, "[$YUM]\\ #,##0.00");
    m.insert(628, "[$ZAR]\\ #,##0.00");
    m.insert(629, "[$ZMK]\\ #,##0.00");
    m.insert(630, "[$ZMW]\\ #,##0.00");
    m.insert(631, "[$ZWD]\\ #,##0.00");
    m.insert(632, "[$ZWL]\\ #,##0.00");
    m.insert(633, "[$ZWN]\\ #,##0.00");
    m.insert(634, "[$ZWR]\\ #,##0.00");
    m
});

fn get_num_fmt_id(style_sheet: &XlsxStyleSheet, style: &Style) -> i32 {
    let num_fmt = style.num_fmt as i32;
    if BUILT_IN_NUM_FMT.contains_key(&num_fmt) {
        return num_fmt;
    }
    if (27..=36).contains(&num_fmt) || (50..=81).contains(&num_fmt) {
        return num_fmt;
    }
    if let Some(fmt_code) = CURRENCY_NUM_FMT.get(&num_fmt) {
        if let Some(ref num_fmts) = style_sheet.num_fmts {
            for num_fmt_entry in &num_fmts.num_fmt {
                if &num_fmt_entry.format_code == fmt_code {
                    return num_fmt_entry.num_fmt_id as i32;
                }
            }
        }
        return num_fmt;
    }
    -1
}

fn new_num_fmt(style_sheet: &mut XlsxStyleSheet, style: &Style) -> i32 {
    let mut dp = String::from("0");
    if let Some(places) = style.decimal_places {
        if places > 0 {
            dp.push('.');
            for _ in 0..places {
                dp.push('0');
            }
        }
    }
    if style.custom_num_fmt.is_some() {
        let id = get_custom_num_fmt_id(style_sheet, style);
        if id != -1 {
            return id;
        }
        return set_custom_num_fmt(style_sheet, style);
    }
    let num_fmt = style.num_fmt as i32;
    if BUILT_IN_NUM_FMT.contains_key(&num_fmt) {
        return num_fmt;
    }
    if let Some(fmt_code) = CURRENCY_NUM_FMT.get(&num_fmt).copied() {
        let mut fc = fmt_code.to_string();
        if let Some(places) = style.decimal_places {
            if places > 0 {
                fc = fc.replace("0.00", &dp);
            }
        }
        if style.neg_red {
            fc = format!("{fc};[Red]{fc}");
        }
        ensure_num_fmts(style_sheet);
        let num_fmts = style_sheet.num_fmts.as_mut().unwrap();
        let mut num_fmt_id = 164;
        if let Some(last) = num_fmts.num_fmt.last() {
            num_fmt_id = last.num_fmt_id as i32 + 1;
        }
        num_fmts.num_fmt.push(XlsxNumFmt {
            num_fmt_id: num_fmt_id as i64,
            format_code: fc,
            format_code_16: None,
        });
        num_fmts.count += 1;
        return num_fmt_id;
    }
    if is_lang_num_fmt(num_fmt) {
        return num_fmt;
    }
    0
}

fn set_custom_num_fmt(style_sheet: &mut XlsxStyleSheet, style: &Style) -> i32 {
    let mut nf = XlsxNumFmt {
        num_fmt_id: 163,
        format_code: style.custom_num_fmt.as_ref().unwrap().clone(),
        format_code_16: None,
    };
    ensure_num_fmts(style_sheet);
    let num_fmts = style_sheet.num_fmts.as_mut().unwrap();
    for num_fmt in &num_fmts.num_fmt {
        if num_fmt.num_fmt_id >= nf.num_fmt_id {
            nf.num_fmt_id = num_fmt.num_fmt_id;
        }
    }
    nf.num_fmt_id += 1;
    let id = nf.num_fmt_id;
    num_fmts.num_fmt.push(nf);
    num_fmts.count = num_fmts.num_fmt.len() as i64;
    id as i32
}

fn get_custom_num_fmt_id(style_sheet: &XlsxStyleSheet, style: &Style) -> i32 {
    if style_sheet.num_fmts.is_none() {
        return -1;
    }
    if let Some(ref custom) = style.custom_num_fmt {
        for num_fmt in &style_sheet.num_fmts.as_ref().unwrap().num_fmt {
            if &num_fmt.format_code == custom {
                return num_fmt.num_fmt_id as i32;
            }
        }
    }
    -1
}

fn is_lang_num_fmt(id: i32) -> bool {
    (27..=36).contains(&id) || (50..=62).contains(&id) || (67..=81).contains(&id)
}

// ------------------------------------------------------------------
// Style extraction (GetStyle)
// ------------------------------------------------------------------

fn should_extract_fill(xf: &XlsxXf, ss: &XlsxStyleSheet) -> bool {
    (xf.apply_fill.is_none() || xf.apply_fill.unwrap())
        && xf.fill_id.is_some()
        && ss.fills.is_some()
        && (*xf.fill_id.as_ref().unwrap() as usize) < ss.fills.as_ref().unwrap().fill.len()
}

fn should_extract_border(xf: &XlsxXf, ss: &XlsxStyleSheet) -> bool {
    (xf.apply_border.is_none() || xf.apply_border.unwrap())
        && xf.border_id.is_some()
        && ss.borders.is_some()
        && (*xf.border_id.as_ref().unwrap() as usize) < ss.borders.as_ref().unwrap().border.len()
}

fn should_extract_font(xf: &XlsxXf, ss: &XlsxStyleSheet) -> bool {
    (xf.apply_font.is_none() || xf.apply_font.unwrap())
        && xf.font_id.is_some()
        && ss.fonts.is_some()
        && (*xf.font_id.as_ref().unwrap() as usize) < ss.fonts.as_ref().unwrap().font.len()
}

fn should_extract_alignment(xf: &XlsxXf, _ss: &XlsxStyleSheet) -> bool {
    xf.apply_alignment.is_none() || xf.apply_alignment.unwrap()
}

fn should_extract_protection(xf: &XlsxXf, _ss: &XlsxStyleSheet) -> bool {
    xf.apply_protection.is_none() || xf.apply_protection.unwrap()
}

fn extract_border(f: &File, border: &XlsxBorder, style: &mut Style) {
    let mut extract = |line_type: &str, line: &XlsxLine| {
        if line
            .style
            .as_deref()
            .map(|s| !s.is_empty())
            .unwrap_or(false)
        {
            let style_idx =
                index_in_static_slice(STYLE_BORDERS, line.style.as_deref().unwrap(), false);
            if style_idx != -1 {
                style.border.push(Border {
                    r#type: line_type.to_string(),
                    color: get_theme_color(f, line.color.as_ref()),
                    style: style_idx as i64,
                });
            }
        }
    };
    let types = [
        "left",
        "right",
        "top",
        "bottom",
        "diagonalUp",
        "diagonalDown",
    ];
    let lines = [
        &border.left,
        &border.right,
        &border.top,
        &border.bottom,
        &border.diagonal,
        &border.diagonal,
    ];
    for (i, line) in lines.iter().enumerate() {
        if let Some(line) = line {
            if i < 4 {
                extract(types[i], line);
            }
            if i == 4 && border.diagonal_up == Some(true) {
                extract(types[i], line);
            }
            if i == 5 && border.diagonal_down == Some(true) {
                extract(types[i], line);
            }
        }
    }
}

fn get_theme_color(f: &File, clr: Option<&XlsxColor>) -> String {
    let Some(c) = clr else {
        return String::new();
    };
    let rgb = f.get_base_color(
        c.rgb.as_deref().unwrap_or(""),
        c.indexed.unwrap_or(0),
        c.theme,
    );
    if rgb.is_empty() {
        return String::new();
    }
    let tint = c.tint.unwrap_or(0.0);
    if tint != 0.0 {
        if let Some(s) = theme_color(&rgb, tint).strip_prefix("FF") {
            s.to_string()
        } else {
            rgb
        }
    } else {
        rgb
    }
}

fn extract_fill(f: &File, fill: &XlsxFill, style: &mut Style) {
    // Prefer pattern fills. Empty sibling elements like `<gradientFill/>` can be
    // emitted by quick_xml when `skip_serializing_if` is not used, so checking
    // pattern first makes round-trips robust.
    if let Some(ref pf) = fill.pattern_fill {
        if pf.pattern_type.is_some() || pf.fg_color.is_some() || pf.bg_color.is_some() {
            style.fill.r#type = "pattern".to_string();
            if let Some(ref pt) = pf.pattern_type {
                style.fill.pattern = index_in_static_slice(STYLE_FILL_PATTERNS, pt, false) as i64;
            }
            if let Some(ref bg) = pf.bg_color {
                if bg.auto != Some(true) {
                    style.fill.color = vec![get_theme_color(f, Some(bg))];
                }
            }
            if let Some(ref fg) = pf.fg_color {
                if fg.auto != Some(true) {
                    style.fill.color = vec![get_theme_color(f, Some(fg))];
                }
            }
            return;
        }
    }
    if let Some(ref gf) = fill.gradient_fill {
        style.fill.r#type = "gradient".to_string();
        for (shading, variant) in style_fill_variants().iter().enumerate() {
            if gf.bottom == variant.bottom
                && gf.degree == variant.degree
                && gf.left == variant.left
                && gf.right == variant.right
                && gf.top == variant.top
                && gf.r#type == variant.r#type
            {
                style.fill.shading = shading as i64;
                break;
            }
        }
        for stop in &gf.stop {
            style
                .fill
                .color
                .push(get_theme_color(f, stop.color.as_ref()));
        }
    }
}

fn extract_font(fnt: &XlsxFont) -> Font {
    let mut font = Font::default();
    font.charset = fnt.charset.as_ref().and_then(|c| c.val);
    font.bold = fnt.b.as_ref().and_then(|b| b.val);
    font.italic = fnt.i.as_ref().and_then(|i| i.val);
    font.strike = fnt.strike.as_ref().and_then(|s| s.val);
    if let Some(ref u) = fnt.u {
        font.underline = u.val.clone();
        if font.underline.as_deref() == Some("") {
            font.underline = Some("single".to_string());
        }
    }
    font.name = fnt.name.as_ref().and_then(|n| n.val.clone());
    font.size = fnt.sz.as_ref().and_then(|s| s.val);
    if let Some(ref color) = fnt.color {
        font.color = color
            .rgb
            .as_ref()
            .map(|rgb| rgb.strip_prefix("FF").unwrap_or(rgb).to_string());
        font.color_indexed = color.indexed;
        font.color_theme = color.theme;
        font.color_tint = color.tint.unwrap_or(0.0);
    }
    font.vert_align = fnt.vert_align.as_ref().and_then(|v| v.val.clone());
    font
}

fn extract_alignment(a: &XlsxAlignment) -> Alignment {
    Alignment {
        horizontal: a.horizontal.clone().unwrap_or_default(),
        indent: a.indent.unwrap_or(0),
        justify_last_line: a.justify_last_line.unwrap_or(false),
        reading_order: a.reading_order.unwrap_or(0),
        relative_indent: a.relative_indent.unwrap_or(0),
        shrink_to_fit: a.shrink_to_fit.unwrap_or(false),
        text_rotation: a.text_rotation.unwrap_or(0),
        vertical: a.vertical.clone().unwrap_or_default(),
        wrap_text: a.wrap_text.unwrap_or(false),
    }
}

fn extract_protection(p: &XlsxProtection) -> Protection {
    Protection {
        hidden: p.hidden.unwrap_or(false),
        locked: p.locked.unwrap_or(false),
    }
}

fn extract_num_fmt(num_fmt_id: i32, style_sheet: &XlsxStyleSheet, style: &mut Style) {
    if let Some(code) = BUILT_IN_NUM_FMT.get(&num_fmt_id) {
        style.num_fmt = num_fmt_id as i64;
        if let Some(dp) = extract_num_fmt_decimal_places(code) {
            style.decimal_places = Some(dp);
        }
        return;
    }
    if is_lang_num_fmt(num_fmt_id) {
        style.num_fmt = num_fmt_id as i64;
        return;
    }
    if let Some(ref num_fmts) = style_sheet.num_fmts {
        for num_fmt in &num_fmts.num_fmt {
            if num_fmt.num_fmt_id as i32 == num_fmt_id {
                if let Some(dp) = extract_num_fmt_decimal_places(&num_fmt.format_code) {
                    style.decimal_places = Some(dp);
                }
                style.custom_num_fmt = Some(num_fmt.format_code.clone());
                if num_fmt.format_code.contains(";[Red]") {
                    style.neg_red = true;
                }
                for (&id, fmt_code) in CURRENCY_NUM_FMT.iter() {
                    let mut fc = fmt_code.to_string();
                    if style.neg_red {
                        fc = format!("{fc};[Red]{fc}");
                    }
                    if num_fmt.format_code == fc {
                        style.num_fmt = id as i64;
                    }
                }
            }
        }
    }
}

fn extract_num_fmt_decimal_places(code: &str) -> Option<i64> {
    if let Some(pos) = code.find('.') {
        let rest = &code[pos + 1..];
        let count = rest.chars().take_while(|c| c.is_ascii_digit()).count();
        if count > 0 {
            return Some(count as i64);
        }
    }
    None
}

// ------------------------------------------------------------------
// Ensure stylesheet containers exist
// ------------------------------------------------------------------

fn ensure_fonts(ss: &mut XlsxStyleSheet) {
    if ss.fonts.is_none() {
        ss.fonts = Some(XlsxFonts::default());
    }
}

fn ensure_fills(ss: &mut XlsxStyleSheet) {
    if ss.fills.is_none() {
        ss.fills = Some(XlsxFills::default());
    }
}

fn ensure_borders(ss: &mut XlsxStyleSheet) {
    if ss.borders.is_none() {
        ss.borders = Some(XlsxBorders::default());
    }
}

fn ensure_cell_xfs(ss: &mut XlsxStyleSheet) {
    if ss.cell_xfs.is_none() {
        ss.cell_xfs = Some(XlsxCellXfs::default());
    }
}

fn ensure_num_fmts(ss: &mut XlsxStyleSheet) {
    if ss.num_fmts.is_none() {
        ss.num_fmts = Some(XlsxNumFmts::default());
    }
}

fn new_dxf_num_fmt(style_sheet: &XlsxStyleSheet, style: &Style) -> Option<XlsxNumFmt> {
    let mut dp = String::from("0");
    if let Some(places) = style.decimal_places {
        if places > 0 {
            dp.push('.');
            for _ in 0..places {
                dp.push('0');
            }
        }
    }
    if let Some(ref custom) = style.custom_num_fmt {
        let mut num_fmt_id = 164i64;
        if let Some(ref dxfs) = style_sheet.dxfs {
            for d in &dxfs.dxfs {
                if let Some(ref nf) = d.num_fmt {
                    if nf.num_fmt_id > num_fmt_id {
                        num_fmt_id = nf.num_fmt_id;
                    }
                }
            }
        }
        return Some(XlsxNumFmt {
            num_fmt_id: num_fmt_id + 1,
            format_code: custom.clone(),
            format_code_16: None,
        });
    }
    let num_fmt = style.num_fmt as i32;
    if let Some(code) = BUILT_IN_NUM_FMT.get(&num_fmt) {
        return Some(XlsxNumFmt {
            num_fmt_id: num_fmt as i64,
            format_code: code.to_string(),
            format_code_16: None,
        });
    }
    if let Some(fmt_code) = CURRENCY_NUM_FMT.get(&num_fmt).copied() {
        let mut fc = fmt_code.to_string();
        if let Some(places) = style.decimal_places {
            if places > 0 {
                fc = fc.replace("0.00", &dp);
            }
        }
        if style.neg_red {
            fc = format!("{fc};[Red]{fc}");
        }
        return Some(XlsxNumFmt {
            num_fmt_id: num_fmt as i64,
            format_code: fc,
            format_code_16: None,
        });
    }
    None
}

#[allow(dead_code)]
fn ensure_cell_styles(ss: &mut XlsxStyleSheet) {
    if ss.cell_styles.is_none() {
        ss.cell_styles = Some(XlsxCellStyles::default());
    }
}

// ------------------------------------------------------------------
// Lookup tables
// ------------------------------------------------------------------

const STYLE_BORDERS: &[&str] = &[
    "none",
    "thin",
    "medium",
    "dashed",
    "dotted",
    "thick",
    "double",
    "hair",
    "mediumDashed",
    "dashDot",
    "mediumDashDot",
    "dashDotDot",
    "mediumDashDotDot",
    "slantDashDot",
];

const STYLE_FILL_PATTERNS: &[&str] = &[
    "none",
    "solid",
    "mediumGray",
    "darkGray",
    "lightGray",
    "darkHorizontal",
    "darkVertical",
    "darkDown",
    "darkUp",
    "darkGrid",
    "darkTrellis",
    "lightHorizontal",
    "lightVertical",
    "lightDown",
    "lightUp",
    "lightGrid",
    "lightTrellis",
    "gray125",
    "gray0625",
];

fn index_in_static_slice(slice: &[&str], value: &str, case_sensitive: bool) -> i64 {
    for (idx, item) in slice.iter().enumerate() {
        let matches = if case_sensitive {
            *item == value
        } else {
            item.eq_ignore_ascii_case(value)
        };
        if matches {
            return idx as i64;
        }
    }
    -1
}

fn style_fill_variants() -> Vec<XlsxGradientFill> {
    vec![
        XlsxGradientFill {
            degree: Some(90.0),
            stop: vec![
                XlsxGradientFillStop::default(),
                XlsxGradientFillStop {
                    position: 1.0,
                    color: None,
                },
            ],
            ..Default::default()
        },
        XlsxGradientFill {
            degree: Some(270.0),
            stop: vec![
                XlsxGradientFillStop::default(),
                XlsxGradientFillStop {
                    position: 1.0,
                    color: None,
                },
            ],
            ..Default::default()
        },
        XlsxGradientFill {
            degree: Some(90.0),
            stop: vec![
                XlsxGradientFillStop::default(),
                XlsxGradientFillStop {
                    position: 0.5,
                    color: None,
                },
                XlsxGradientFillStop {
                    position: 1.0,
                    color: None,
                },
            ],
            ..Default::default()
        },
        XlsxGradientFill {
            stop: vec![
                XlsxGradientFillStop::default(),
                XlsxGradientFillStop {
                    position: 1.0,
                    color: None,
                },
            ],
            ..Default::default()
        },
        XlsxGradientFill {
            degree: Some(180.0),
            stop: vec![
                XlsxGradientFillStop::default(),
                XlsxGradientFillStop {
                    position: 1.0,
                    color: None,
                },
            ],
            ..Default::default()
        },
        XlsxGradientFill {
            stop: vec![
                XlsxGradientFillStop::default(),
                XlsxGradientFillStop {
                    position: 0.5,
                    color: None,
                },
                XlsxGradientFillStop {
                    position: 1.0,
                    color: None,
                },
            ],
            ..Default::default()
        },
        XlsxGradientFill {
            degree: Some(45.0),
            stop: vec![
                XlsxGradientFillStop::default(),
                XlsxGradientFillStop {
                    position: 1.0,
                    color: None,
                },
            ],
            ..Default::default()
        },
        XlsxGradientFill {
            degree: Some(255.0),
            stop: vec![
                XlsxGradientFillStop::default(),
                XlsxGradientFillStop {
                    position: 1.0,
                    color: None,
                },
            ],
            ..Default::default()
        },
        XlsxGradientFill {
            degree: Some(45.0),
            stop: vec![
                XlsxGradientFillStop::default(),
                XlsxGradientFillStop {
                    position: 0.5,
                    color: None,
                },
                XlsxGradientFillStop {
                    position: 1.0,
                    color: None,
                },
            ],
            ..Default::default()
        },
        XlsxGradientFill {
            degree: Some(135.0),
            stop: vec![
                XlsxGradientFillStop::default(),
                XlsxGradientFillStop {
                    position: 1.0,
                    color: None,
                },
            ],
            ..Default::default()
        },
        XlsxGradientFill {
            degree: Some(315.0),
            stop: vec![
                XlsxGradientFillStop::default(),
                XlsxGradientFillStop {
                    position: 1.0,
                    color: None,
                },
            ],
            ..Default::default()
        },
        XlsxGradientFill {
            degree: Some(135.0),
            stop: vec![
                XlsxGradientFillStop::default(),
                XlsxGradientFillStop {
                    position: 0.5,
                    color: None,
                },
                XlsxGradientFillStop {
                    position: 1.0,
                    color: None,
                },
            ],
            ..Default::default()
        },
        XlsxGradientFill {
            r#type: Some("path".to_string()),
            stop: vec![
                XlsxGradientFillStop::default(),
                XlsxGradientFillStop {
                    position: 1.0,
                    color: None,
                },
            ],
            ..Default::default()
        },
        XlsxGradientFill {
            r#type: Some("path".to_string()),
            left: Some(1.0),
            right: Some(1.0),
            stop: vec![
                XlsxGradientFillStop::default(),
                XlsxGradientFillStop {
                    position: 1.0,
                    color: None,
                },
            ],
            ..Default::default()
        },
        XlsxGradientFill {
            r#type: Some("path".to_string()),
            bottom: Some(1.0),
            top: Some(1.0),
            stop: vec![
                XlsxGradientFillStop::default(),
                XlsxGradientFillStop {
                    position: 1.0,
                    color: None,
                },
            ],
            ..Default::default()
        },
        XlsxGradientFill {
            r#type: Some("path".to_string()),
            bottom: Some(1.0),
            left: Some(1.0),
            right: Some(1.0),
            top: Some(1.0),
            stop: vec![
                XlsxGradientFillStop::default(),
                XlsxGradientFillStop {
                    position: 1.0,
                    color: None,
                },
            ],
            ..Default::default()
        },
        XlsxGradientFill {
            r#type: Some("path".to_string()),
            bottom: Some(0.5),
            left: Some(0.5),
            right: Some(0.5),
            top: Some(0.5),
            stop: vec![
                XlsxGradientFillStop::default(),
                XlsxGradientFillStop {
                    position: 1.0,
                    color: None,
                },
            ],
            ..Default::default()
        },
    ]
}

// ------------------------------------------------------------------
// Conditional formatting lookup tables
// ------------------------------------------------------------------

static VALID_TYPE: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert("cell", "cellIs");
    m.insert("average", "aboveAverage");
    m.insert("duplicate", "duplicateValues");
    m.insert("unique", "uniqueValues");
    m.insert("top", "top10");
    m.insert("bottom", "top10");
    m.insert("text", "text");
    m.insert("time_period", "timePeriod");
    m.insert("blanks", "containsBlanks");
    m.insert("no_blanks", "notContainsBlanks");
    m.insert("errors", "containsErrors");
    m.insert("no_errors", "notContainsErrors");
    m.insert("2_color_scale", "2_color_scale");
    m.insert("3_color_scale", "3_color_scale");
    m.insert("data_bar", "dataBar");
    m.insert("formula", "expression");
    m.insert("icon_set", "iconSet");
    m
});

static CRITERIA_TYPE: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert("!=", "notEqual");
    m.insert("<", "lessThan");
    m.insert("<=", "lessThanOrEqual");
    m.insert("<>", "notEqual");
    m.insert("=", "equal");
    m.insert("==", "equal");
    m.insert(">", "greaterThan");
    m.insert(">=", "greaterThanOrEqual");
    m.insert("begins with", "beginsWith");
    m.insert("between", "between");
    m.insert("containing", "containsText");
    m.insert("continue month", "nextMonth");
    m.insert("continue week", "nextWeek");
    m.insert("ends with", "endsWith");
    m.insert("equal to", "equal");
    m.insert("greater than or equal to", "greaterThanOrEqual");
    m.insert("greater than", "greaterThan");
    m.insert("last 7 days", "last7Days");
    m.insert("last month", "lastMonth");
    m.insert("last week", "lastWeek");
    m.insert("less than or equal to", "lessThanOrEqual");
    m.insert("less than", "lessThan");
    m.insert("not between", "notBetween");
    m.insert("not containing", "notContains");
    m.insert("not equal to", "notEqual");
    m.insert("this month", "thisMonth");
    m.insert("this week", "thisWeek");
    m.insert("today", "today");
    m.insert("tomorrow", "tomorrow");
    m.insert("yesterday", "yesterday");
    m
});

static OPERATOR_TYPE: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert("beginsWith", "begins with");
    m.insert("between", "between");
    m.insert("containsText", "containing");
    m.insert("endsWith", "ends with");
    m.insert("equal", "equal to");
    m.insert("greaterThan", "greater than");
    m.insert("greaterThanOrEqual", "greater than or equal to");
    m.insert("last7Days", "last 7 days");
    m.insert("lastMonth", "last month");
    m.insert("lastWeek", "last week");
    m.insert("lessThan", "less than");
    m.insert("lessThanOrEqual", "less than or equal to");
    m.insert("nextMonth", "continue month");
    m.insert("nextWeek", "continue week");
    m.insert("notBetween", "not between");
    m.insert("notContains", "not containing");
    m.insert("notEqual", "not equal to");
    m.insert("thisMonth", "this month");
    m.insert("thisWeek", "this week");
    m.insert("today", "today");
    m.insert("tomorrow", "tomorrow");
    m.insert("yesterday", "yesterday");
    m
});

const CELL_IS_CRITERIA_TYPE: &[&str] = &[
    "equal",
    "notEqual",
    "greaterThan",
    "lessThan",
    "greaterThanOrEqual",
    "lessThanOrEqual",
    "containsText",
    "notContains",
    "beginsWith",
    "endsWith",
];

static ICON_SET_PRESETS: LazyLock<HashMap<&'static str, XlsxCfRule>> = LazyLock::new(|| {
    let cfvo3 = XlsxIconSet {
        cfvo: vec![
            XlsxCfvo {
                r#type: Some("percent".to_string()),
                val: Some("0".to_string()),
                ..Default::default()
            },
            XlsxCfvo {
                r#type: Some("percent".to_string()),
                val: Some("33".to_string()),
                ..Default::default()
            },
            XlsxCfvo {
                r#type: Some("percent".to_string()),
                val: Some("67".to_string()),
                ..Default::default()
            },
        ],
        ..Default::default()
    };
    let cfvo4 = XlsxIconSet {
        cfvo: vec![
            XlsxCfvo {
                r#type: Some("percent".to_string()),
                val: Some("0".to_string()),
                ..Default::default()
            },
            XlsxCfvo {
                r#type: Some("percent".to_string()),
                val: Some("25".to_string()),
                ..Default::default()
            },
            XlsxCfvo {
                r#type: Some("percent".to_string()),
                val: Some("50".to_string()),
                ..Default::default()
            },
            XlsxCfvo {
                r#type: Some("percent".to_string()),
                val: Some("75".to_string()),
                ..Default::default()
            },
        ],
        ..Default::default()
    };
    let cfvo5 = XlsxIconSet {
        cfvo: vec![
            XlsxCfvo {
                r#type: Some("percent".to_string()),
                val: Some("0".to_string()),
                ..Default::default()
            },
            XlsxCfvo {
                r#type: Some("percent".to_string()),
                val: Some("20".to_string()),
                ..Default::default()
            },
            XlsxCfvo {
                r#type: Some("percent".to_string()),
                val: Some("40".to_string()),
                ..Default::default()
            },
            XlsxCfvo {
                r#type: Some("percent".to_string()),
                val: Some("60".to_string()),
                ..Default::default()
            },
            XlsxCfvo {
                r#type: Some("percent".to_string()),
                val: Some("80".to_string()),
                ..Default::default()
            },
        ],
        ..Default::default()
    };
    let mut m = HashMap::new();
    for name in [
        "3Arrows",
        "3ArrowsGray",
        "3Flags",
        "3Signs",
        "3Symbols",
        "3Symbols2",
        "3TrafficLights1",
        "3TrafficLights2",
    ] {
        m.insert(
            name,
            XlsxCfRule {
                icon_set: Some(cfvo3.clone()),
                ..Default::default()
            },
        );
    }
    for name in [
        "4Arrows",
        "4ArrowsGray",
        "4Rating",
        "4RedToBlack",
        "4TrafficLights",
    ] {
        m.insert(
            name,
            XlsxCfRule {
                icon_set: Some(cfvo4.clone()),
                ..Default::default()
            },
        );
    }
    for name in ["5Arrows", "5ArrowsGray", "5Quarters", "5Rating"] {
        m.insert(
            name,
            XlsxCfRule {
                icon_set: Some(cfvo5.clone()),
                ..Default::default()
            },
        );
    }
    m
});

static X14_ICON_SET_PRESETS: LazyLock<HashMap<&'static str, XlsxX14CfRule>> = LazyLock::new(|| {
    let cfvo3 = Xlsx14IconSet {
        cfvo: vec![
            Xlsx14Cfvo {
                r#type: Some("percent".to_string()),
                f: Some("0".to_string()),
                ..Default::default()
            },
            Xlsx14Cfvo {
                r#type: Some("percent".to_string()),
                f: Some("33".to_string()),
                ..Default::default()
            },
            Xlsx14Cfvo {
                r#type: Some("percent".to_string()),
                f: Some("67".to_string()),
                ..Default::default()
            },
        ],
        ..Default::default()
    };
    let cfvo5 = Xlsx14IconSet {
        cfvo: vec![
            Xlsx14Cfvo {
                r#type: Some("percent".to_string()),
                f: Some("0".to_string()),
                ..Default::default()
            },
            Xlsx14Cfvo {
                r#type: Some("percent".to_string()),
                f: Some("20".to_string()),
                ..Default::default()
            },
            Xlsx14Cfvo {
                r#type: Some("percent".to_string()),
                f: Some("40".to_string()),
                ..Default::default()
            },
            Xlsx14Cfvo {
                r#type: Some("percent".to_string()),
                f: Some("60".to_string()),
                ..Default::default()
            },
            Xlsx14Cfvo {
                r#type: Some("percent".to_string()),
                f: Some("80".to_string()),
                ..Default::default()
            },
        ],
        ..Default::default()
    };
    let mut m = HashMap::new();
    m.insert(
        "3Stars",
        XlsxX14CfRule {
            icon_set: Some(cfvo3.clone()),
            ..Default::default()
        },
    );
    m.insert(
        "3Triangles",
        XlsxX14CfRule {
            icon_set: Some(cfvo3.clone()),
            ..Default::default()
        },
    );
    m.insert(
        "5Boxes",
        XlsxX14CfRule {
            icon_set: Some(cfvo5.clone()),
            ..Default::default()
        },
    );
    m
});

static DRAW_COND_FMT_FUNC: LazyLock<
    HashMap<
        &'static str,
        fn(
            i32,
            &str,
            &str,
            &str,
            &ConditionalFormatOptions,
        ) -> (Option<XlsxCfRule>, Option<XlsxX14CfRule>),
    >,
> = LazyLock::new(|| {
    let mut m: HashMap<
        &'static str,
        fn(
            i32,
            &str,
            &str,
            &str,
            &ConditionalFormatOptions,
        ) -> (Option<XlsxCfRule>, Option<XlsxX14CfRule>),
    > = HashMap::new();
    m.insert("cellIs", draw_cond_fmt_cell_is);
    m.insert("timePeriod", draw_cond_fmt_time_period);
    m.insert("text", draw_cond_fmt_text);
    m.insert("top10", draw_cond_fmt_top10);
    m.insert("aboveAverage", draw_cond_fmt_above_average);
    m.insert("duplicateValues", draw_cond_fmt_duplicate_unique);
    m.insert("uniqueValues", draw_cond_fmt_duplicate_unique);
    m.insert("containsBlanks", draw_cond_fmt_blanks);
    m.insert("notContainsBlanks", draw_cond_fmt_no_blanks);
    m.insert("containsErrors", draw_cond_fmt_errors);
    m.insert("notContainsErrors", draw_cond_fmt_no_errors);
    m.insert("2_color_scale", draw_cond_fmt_color_scale);
    m.insert("3_color_scale", draw_cond_fmt_color_scale);
    m.insert("dataBar", draw_cond_fmt_data_bar);
    m.insert("expression", draw_cond_fmt_exp);
    m.insert("iconSet", draw_cond_fmt_icon_set);
    m
});

static EXTRACT_COND_FMT_FUNC: LazyLock<
    HashMap<
        &'static str,
        fn(&File, &str, &XlsxCfRule, Option<&XlsxExtLst>) -> ConditionalFormatOptions,
    >,
> = LazyLock::new(|| {
    let mut m: HashMap<
        &'static str,
        fn(&File, &str, &XlsxCfRule, Option<&XlsxExtLst>) -> ConditionalFormatOptions,
    > = HashMap::new();
    m.insert("cellIs", extract_cond_fmt_cell_is);
    m.insert("timePeriod", extract_cond_fmt_time_period);
    m.insert("containsText", extract_cond_fmt_text);
    m.insert("notContainsText", extract_cond_fmt_text);
    m.insert("beginsWith", extract_cond_fmt_text);
    m.insert("endsWith", extract_cond_fmt_text);
    m.insert("top10", extract_cond_fmt_top10);
    m.insert("aboveAverage", extract_cond_fmt_above_average);
    m.insert("duplicateValues", extract_cond_fmt_duplicate_unique);
    m.insert("uniqueValues", extract_cond_fmt_duplicate_unique);
    m.insert("containsBlanks", extract_cond_fmt_blanks);
    m.insert("notContainsBlanks", extract_cond_fmt_no_blanks);
    m.insert("containsErrors", extract_cond_fmt_errors);
    m.insert("notContainsErrors", extract_cond_fmt_no_errors);
    m.insert("colorScale", extract_cond_fmt_color_scale);
    m.insert("dataBar", extract_cond_fmt_data_bar);
    m.insert("expression", extract_cond_fmt_exp);
    m.insert("iconSet", extract_cond_fmt_icon_set);
    m
});

// ------------------------------------------------------------------
// Conditional formatting range preparation
// ------------------------------------------------------------------

fn prepare_conditional_format_range(range_ref: &str) -> Result<(String, String)> {
    if range_ref.is_empty() {
        return Err(Box::new(ErrParameterRequired));
    }
    let range_ref = range_ref.replace(',', " ");
    let mut sqref_parts = Vec::new();
    let mut mast_cell = String::new();
    for (i, cell_range) in range_ref.split_whitespace().enumerate() {
        let mut cell_names = Vec::new();
        for (j, ref_str) in cell_range.split(':').enumerate() {
            if j > 1 {
                return Err(Box::new(ErrParameterInvalid));
            }
            let (mut col, mut row) = parse_conditional_ref(ref_str)?;
            if col == -1 {
                col = if j == 0 { 1 } else { MAX_COLUMNS };
            }
            if row == -1 {
                row = if j == 0 { 1 } else { TOTAL_ROWS };
            }
            let cell_name = coordinates_to_cell_name(col, row, false)?;
            cell_names.push(cell_name.clone());
            if i == 0 && j == 0 {
                mast_cell = cell_name;
            }
        }
        sqref_parts.push(cell_names.join(":"));
    }
    Ok((sqref_parts.join(" "), mast_cell))
}

fn parse_conditional_ref(ref_str: &str) -> Result<(i32, i32)> {
    // Strip an optional sheet qualifier such as 'Sheet1'!A1 or Sheet1!A1.
    let cell_ref = ref_str.rsplit_once('!').map(|(_, r)| r).unwrap_or(ref_str);
    if let Ok((col, row)) = cell_name_to_coordinates(cell_ref) {
        return Ok((col, row));
    }
    if let Ok(col) = column_name_to_number(cell_ref) {
        return Ok((col, -1));
    }
    if let Ok(row) = cell_ref.parse::<i32>() {
        if row < 1 || row > TOTAL_ROWS {
            return Err(Box::new(ErrParameterInvalid));
        }
        return Ok((-1, row));
    }
    Err(Box::new(ErrParameterInvalid))
}

// ------------------------------------------------------------------
// Conditional formatting x14 extension helpers
// ------------------------------------------------------------------

fn append_cf_rule(ws: &mut XlsxWorksheet, rule: &XlsxX14CfRule, sqref: &str) -> Result<()> {
    let cond_fmt = XlsxX14ConditionalFormatting {
        xmlns_xm: Some(NAMESPACE_SPREADSHEET_EXCEL_2006_MAIN.to_string()),
        sqref: Some(sqref.to_string()),
        cf_rule: vec![rule.clone()],
        ..Default::default()
    };
    let mut rule_xml = xml_to_string(&cond_fmt)?;
    if let Some(pos) = rule_xml.find("?>") {
        rule_xml = rule_xml[pos + 2..].to_string();
    }

    let ext_lst = ws.ext_lst.get_or_insert_with(XlsxExtLst::default);
    let closing = "</x14:conditionalFormattings>";
    let mut append_mode = false;
    for ext in &mut ext_lst.ext {
        if ext.uri.as_deref() == Some(EXT_URI_CONDITIONAL_FORMATTINGS) {
            if ext.content.is_empty() {
                ext.content =
                    format!("<x14:conditionalFormattings>{rule_xml}</x14:conditionalFormattings>");
            } else if let Some(pos) = ext.content.rfind(closing) {
                ext.content.insert_str(pos, &rule_xml);
            } else {
                ext.content.push_str(&rule_xml);
            }
            append_mode = true;
            break;
        }
    }
    if !append_mode {
        ext_lst.ext.push(XlsxExt {
            uri: Some(EXT_URI_CONDITIONAL_FORMATTINGS.to_string()),
            xmlns_x14: Some(NAMESPACE_SPREADSHEET_X14.to_string()),
            content: format!("<x14:conditionalFormattings>{rule_xml}</x14:conditionalFormattings>"),
            ..Default::default()
        });
    }
    sort_ext_lst(ext_lst);
    Ok(())
}

fn sort_ext_lst(ext_lst: &mut XlsxExtLst) {
    ext_lst.ext.sort_by(|a, b| {
        let ai = in_str_slice(
            WORKSHEET_EXT_URI_PRIORITY,
            a.uri.as_deref().unwrap_or(""),
            false,
        );
        let bi = in_str_slice(
            WORKSHEET_EXT_URI_PRIORITY,
            b.uri.as_deref().unwrap_or(""),
            false,
        );
        ai.cmp(&bi)
    });
}

fn parse_x14_conditional_formattings(content: &str) -> Result<DecodeX14ConditionalFormattingRules> {
    let stripped = content
        .replace("<x14:", "<")
        .replace("</x14:", "</")
        .replace("<xm:", "<")
        .replace("</xm:", "</");
    Ok(xml_from_reader(stripped.as_bytes())?)
}

fn delete_x14_cf_rule(
    rules: &DecodeX14ConditionalFormattingRules,
    del_cells: &HashMap<i32, Vec<Vec<i32>>>,
) -> Result<String> {
    let mut inner = String::new();
    for cond_fmt in &rules.cond_fmt {
        let sqref = cond_fmt.sqref.as_deref().unwrap_or("").to_string();
        let new_sqref = delete_cells_from_sqref(&sqref, del_cells)?;
        if new_sqref.is_empty() {
            continue;
        }
        for rule in &cond_fmt.cf_rule {
            let x14_rule = build_x14_cf_rule(rule);
            let x14_cond_fmt = XlsxX14ConditionalFormatting {
                xmlns_xm: Some(NAMESPACE_SPREADSHEET_EXCEL_2006_MAIN.to_string()),
                pivot: cond_fmt.pivot,
                sqref: Some(new_sqref.clone()),
                cf_rule: vec![x14_rule],
                ext_lst: cond_fmt.ext_lst.clone(),
            };
            let mut bytes = xml_to_string(&x14_cond_fmt)?;
            if let Some(pos) = bytes.find("?>") {
                bytes = bytes[pos + 2..].to_string();
            }
            inner.push_str(&bytes);
        }
    }
    Ok(format!(
        "<x14:conditionalFormattings>{inner}</x14:conditionalFormattings>"
    ))
}

fn build_x14_cf_rule(rule: &DecodeX14CfRule) -> XlsxX14CfRule {
    XlsxX14CfRule {
        r#type: rule.r#type.clone(),
        priority: rule.priority,
        stop_if_true: rule.stop_if_true,
        above_average: rule.above_average,
        percent: rule.percent,
        bottom: rule.bottom,
        operator: rule.operator.clone(),
        text: rule.text.clone(),
        time_period: rule.time_period.clone(),
        rank: rule.rank,
        std_dev: rule.std_dev,
        equal_average: rule.equal_average,
        active_present: rule.active_present,
        id: rule.id.clone(),
        f: rule.f.clone(),
        color_scale: rule.color_scale.clone(),
        dxf: rule.dxf.clone(),
        ext_lst: rule.ext_lst.clone(),
        data_bar: rule.data_bar.as_ref().map(build_x14_data_bar),
        icon_set: rule.icon_set.as_ref().map(build_x14_icon_set),
    }
}

fn build_x14_data_bar(db: &DecodeX14DataBar) -> Xlsx14DataBar {
    Xlsx14DataBar {
        max_length: db.max_length,
        min_length: db.min_length,
        border: db.border,
        gradient: db.gradient,
        show_value: db.show_value,
        direction: db.direction.clone(),
        cfvo: db.cfvo.iter().map(build_x14_cfvo).collect(),
        border_color: db.border_color.clone(),
        negative_fill_color: db.negative_fill_color.clone(),
        axis_color: db.axis_color.clone(),
    }
}

fn build_x14_icon_set(icon_set: &DecodeX14IconSet) -> Xlsx14IconSet {
    Xlsx14IconSet {
        icon_set: icon_set.icon_set.clone(),
        show_value: icon_set.show_value,
        percent: icon_set.percent,
        reverse: icon_set.reverse,
        custom: icon_set.custom,
        cfvo: icon_set.cfvo.iter().map(build_x14_cfvo).collect(),
        cf_icon: icon_set.cf_icon.clone(),
    }
}

fn build_x14_cfvo(cfvo: &DecodeX14Cfvo) -> Xlsx14Cfvo {
    Xlsx14Cfvo {
        r#type: cfvo.r#type.clone(),
        gte: cfvo.gte,
        f: cfvo.f.clone(),
        ext_lst: cfvo.ext_lst.clone(),
    }
}

// ------------------------------------------------------------------
// Conditional formatting draw helpers
// ------------------------------------------------------------------

fn draw_cond_fmt_cell_is(
    p: i32,
    ct: &str,
    _ref: &str,
    _guid: &str,
    format: &ConditionalFormatOptions,
) -> (Option<XlsxCfRule>, Option<XlsxX14CfRule>) {
    let mut c = XlsxCfRule {
        priority: Some(p as i64 + 1),
        stop_if_true: Some(format.stop_if_true),
        r#type: Some("cellIs".to_string()),
        operator: Some(ct.to_string()),
        dxf_id: format.format,
        ..Default::default()
    };
    if ct == "between" || ct == "notBetween" {
        c.formula.push(format.min_value.clone());
        c.formula.push(format.max_value.clone());
    }
    if in_str_slice(CELL_IS_CRITERIA_TYPE, ct, true) != -1 {
        c.formula.push(format.value.clone());
    }
    (Some(c), None)
}

fn draw_cond_fmt_time_period(
    p: i32,
    ct: &str,
    ref_str: &str,
    _guid: &str,
    format: &ConditionalFormatOptions,
) -> (Option<XlsxCfRule>, Option<XlsxX14CfRule>) {
    let formula = match ct {
        "yesterday" => format!("FLOOR({ref_str},1)=TODAY()-1"),
        "today" => format!("FLOOR({ref_str},1)=TODAY()"),
        "tomorrow" => format!("FLOOR({ref_str},1)=TODAY()+1"),
        "last7Days" => format!("AND(TODAY()-FLOOR({ref_str},1)<=6,FLOOR({ref_str},1)<=TODAY())"),
        "lastWeek" => format!(
            "AND(TODAY()-ROUNDDOWN({ref_str},0)>=(WEEKDAY(TODAY())),TODAY()-ROUNDDOWN({ref_str},0)<(WEEKDAY(TODAY())+7))"
        ),
        "thisWeek" => format!(
            "AND(TODAY()-ROUNDDOWN({ref_str},0)<=WEEKDAY(TODAY())-1,ROUNDDOWN({ref_str},0)-TODAY()>=7-WEEKDAY(TODAY()))"
        ),
        "nextWeek" => format!(
            "AND(ROUNDDOWN({ref_str},0)-TODAY()>(7-WEEKDAY(TODAY())),ROUNDDOWN({ref_str},0)-TODAY()<(15-WEEKDAY(TODAY())))"
        ),
        "lastMonth" => format!(
            "AND(MONTH({ref_str})=MONTH(TODAY())-1,OR(YEAR({ref_str})=YEAR(TODAY()),AND(MONTH({ref_str})=1,YEAR({ref_str})=YEAR(TODAY())-1)))"
        ),
        "thisMonth" => {
            format!("AND(MONTH({ref_str})=MONTH(TODAY()),YEAR({ref_str})=YEAR(TODAY()))")
        }
        "nextMonth" => format!(
            "AND(MONTH({ref_str})=MONTH(TODAY())+1,OR(YEAR({ref_str})=YEAR(TODAY()),AND(MONTH({ref_str})=12,YEAR({ref_str})=YEAR(TODAY())+1)))"
        ),
        _ => String::new(),
    };
    let c = XlsxCfRule {
        priority: Some(p as i64 + 1),
        stop_if_true: Some(format.stop_if_true),
        r#type: Some("timePeriod".to_string()),
        formula: vec![formula],
        dxf_id: format.format,
        ..Default::default()
    };
    (Some(c), None)
}

fn draw_cond_fmt_text(
    p: i32,
    ct: &str,
    ref_str: &str,
    _guid: &str,
    format: &ConditionalFormatOptions,
) -> (Option<XlsxCfRule>, Option<XlsxX14CfRule>) {
    let rule_type = match ct {
        "containsText" => "containsText",
        "notContains" => "notContainsText",
        "beginsWith" => "beginsWith",
        "endsWith" => "endsWith",
        _ => "",
    };
    let escaped = format.value.replace('"', "\"\"");
    let formula = match ct {
        "containsText" => format!("NOT(ISERROR(SEARCH(\"{escaped}\",{ref_str})))"),
        "notContains" => format!("ISERROR(SEARCH(\"{escaped}\",{ref_str}))"),
        "beginsWith" => format!("LEFT({ref_str},LEN(\"{escaped}\"))=\"{escaped}\""),
        "endsWith" => format!("RIGHT({ref_str},LEN(\"{escaped}\"))=\"{escaped}\""),
        _ => String::new(),
    };
    let c = XlsxCfRule {
        priority: Some(p as i64 + 1),
        stop_if_true: Some(format.stop_if_true),
        r#type: Some(rule_type.to_string()),
        operator: Some(ct.to_string()),
        text: Some(format.value.clone()),
        formula: vec![formula],
        dxf_id: format.format,
        ..Default::default()
    };
    (Some(c), None)
}

fn draw_cond_fmt_top10(
    p: i32,
    _ct: &str,
    _ref: &str,
    _guid: &str,
    format: &ConditionalFormatOptions,
) -> (Option<XlsxCfRule>, Option<XlsxX14CfRule>) {
    let rank = format.value.parse::<i64>().unwrap_or(10);
    let c = XlsxCfRule {
        priority: Some(p as i64 + 1),
        stop_if_true: Some(format.stop_if_true),
        r#type: Some("top10".to_string()),
        bottom: Some(format.r#type == "bottom"),
        rank: Some(rank),
        percent: Some(format.percent),
        dxf_id: format.format,
        ..Default::default()
    };
    (Some(c), None)
}

fn draw_cond_fmt_above_average(
    p: i32,
    _ct: &str,
    _ref: &str,
    _guid: &str,
    format: &ConditionalFormatOptions,
) -> (Option<XlsxCfRule>, Option<XlsxX14CfRule>) {
    let c = XlsxCfRule {
        priority: Some(p as i64 + 1),
        stop_if_true: Some(format.stop_if_true),
        r#type: Some("aboveAverage".to_string()),
        above_average: Some(format.above_average),
        dxf_id: format.format,
        ..Default::default()
    };
    (Some(c), None)
}

fn draw_cond_fmt_duplicate_unique(
    p: i32,
    _ct: &str,
    _ref: &str,
    _guid: &str,
    format: &ConditionalFormatOptions,
) -> (Option<XlsxCfRule>, Option<XlsxX14CfRule>) {
    let c = XlsxCfRule {
        priority: Some(p as i64 + 1),
        stop_if_true: Some(format.stop_if_true),
        r#type: Some(
            VALID_TYPE
                .get(format.r#type.as_str())
                .copied()
                .unwrap_or("duplicateValues")
                .to_string(),
        ),
        dxf_id: format.format,
        ..Default::default()
    };
    (Some(c), None)
}

fn draw_cond_fmt_color_scale(
    p: i32,
    _ct: &str,
    _ref: &str,
    _guid: &str,
    format: &ConditionalFormatOptions,
) -> (Option<XlsxCfRule>, Option<XlsxX14CfRule>) {
    let min_value = if format.min_value.is_empty() {
        "0"
    } else {
        &format.min_value
    };
    let max_value = if format.max_value.is_empty() {
        "0"
    } else {
        &format.max_value
    };
    let mid_value = if format.mid_value.is_empty() {
        "50"
    } else {
        &format.mid_value
    };
    let mut cfvo = vec![XlsxCfvo {
        r#type: Some(format.min_type.clone()),
        val: Some(min_value.to_string()),
        ..Default::default()
    }];
    let mut color = vec![XlsxColor {
        rgb: Some(get_palette_color(&format.min_color)),
        ..Default::default()
    }];
    if VALID_TYPE.get(format.r#type.as_str()).copied() == Some("3_color_scale") {
        cfvo.push(XlsxCfvo {
            r#type: Some(format.mid_type.clone()),
            val: Some(mid_value.to_string()),
            ..Default::default()
        });
        color.push(XlsxColor {
            rgb: Some(get_palette_color(&format.mid_color)),
            ..Default::default()
        });
    }
    cfvo.push(XlsxCfvo {
        r#type: Some(format.max_type.clone()),
        val: Some(max_value.to_string()),
        ..Default::default()
    });
    color.push(XlsxColor {
        rgb: Some(get_palette_color(&format.max_color)),
        ..Default::default()
    });
    let c = XlsxCfRule {
        priority: Some(p as i64 + 1),
        stop_if_true: Some(format.stop_if_true),
        r#type: Some("colorScale".to_string()),
        color_scale: Some(XlsxColorScale { cfvo, color }),
        ..Default::default()
    };
    (Some(c), None)
}

fn draw_cond_fmt_data_bar(
    p: i32,
    _ct: &str,
    _ref: &str,
    guid: &str,
    format: &ConditionalFormatOptions,
) -> (Option<XlsxCfRule>, Option<XlsxX14CfRule>) {
    let mut x14_rule: Option<XlsxX14CfRule> = None;
    if format.bar_solid
        || format.bar_direction == "leftToRight"
        || format.bar_direction == "rightToLeft"
        || !format.bar_border_color.is_empty()
    {
        let mut data_bar = Xlsx14DataBar {
            max_length: Some(100),
            border: Some(!format.bar_border_color.is_empty()),
            gradient: Some(!format.bar_solid),
            direction: Some(format.bar_direction.clone()).filter(|s| !s.is_empty()),
            cfvo: vec![
                Xlsx14Cfvo {
                    r#type: Some("autoMin".to_string()),
                    ..Default::default()
                },
                Xlsx14Cfvo {
                    r#type: Some("autoMax".to_string()),
                    ..Default::default()
                },
            ],
            negative_fill_color: Some(XlsxColor {
                rgb: Some("FFFF0000".to_string()),
                ..Default::default()
            }),
            axis_color: Some(XlsxColor {
                rgb: Some("FFFF0000".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };
        if data_bar.border == Some(true) {
            data_bar.border_color = Some(XlsxColor {
                rgb: Some(get_palette_color(&format.bar_border_color)),
                ..Default::default()
            });
        }
        x14_rule = Some(XlsxX14CfRule {
            r#type: Some("dataBar".to_string()),
            id: Some(guid.to_string()),
            data_bar: Some(data_bar),
            ..Default::default()
        });
    }
    let ext_lst = x14_rule.as_ref().map(|_| XlsxExtLst {
        ext: vec![XlsxExt {
            uri: Some(EXT_URI_CONDITIONAL_FORMATTING_RULE_ID.to_string()),
            xmlns_x14: Some(NAMESPACE_SPREADSHEET_X14.to_string()),
            content: format!("<x14:id>{guid}</x14:id>"),
            ..Default::default()
        }],
    });
    let c = XlsxCfRule {
        priority: Some(p as i64 + 1),
        stop_if_true: Some(format.stop_if_true),
        r#type: Some("dataBar".to_string()),
        data_bar: Some(XlsxDataBar {
            show_value: Some(!format.bar_only),
            cfvo: vec![
                XlsxCfvo {
                    r#type: Some(format.min_type.clone()).filter(|s| !s.is_empty()),
                    val: Some(format.min_value.clone()).filter(|s| !s.is_empty()),
                    ..Default::default()
                },
                XlsxCfvo {
                    r#type: Some(format.max_type.clone()).filter(|s| !s.is_empty()),
                    val: Some(format.max_value.clone()).filter(|s| !s.is_empty()),
                    ..Default::default()
                },
            ],
            color: vec![XlsxColor {
                rgb: Some(get_palette_color(&format.bar_color)),
                ..Default::default()
            }],
            ..Default::default()
        }),
        ext_lst,
        ..Default::default()
    };
    (Some(c), x14_rule)
}

fn draw_cond_fmt_exp(
    p: i32,
    _ct: &str,
    _ref: &str,
    _guid: &str,
    format: &ConditionalFormatOptions,
) -> (Option<XlsxCfRule>, Option<XlsxX14CfRule>) {
    let c = XlsxCfRule {
        priority: Some(p as i64 + 1),
        stop_if_true: Some(format.stop_if_true),
        r#type: Some("expression".to_string()),
        formula: vec![format.criteria.clone()],
        dxf_id: format.format,
        ..Default::default()
    };
    (Some(c), None)
}

fn draw_cond_fmt_errors(
    p: i32,
    _ct: &str,
    ref_str: &str,
    _guid: &str,
    format: &ConditionalFormatOptions,
) -> (Option<XlsxCfRule>, Option<XlsxX14CfRule>) {
    let c = XlsxCfRule {
        priority: Some(p as i64 + 1),
        stop_if_true: Some(format.stop_if_true),
        r#type: Some("containsErrors".to_string()),
        formula: vec![format!("ISERROR({ref_str})")],
        dxf_id: format.format,
        ..Default::default()
    };
    (Some(c), None)
}

fn draw_cond_fmt_no_errors(
    p: i32,
    _ct: &str,
    ref_str: &str,
    _guid: &str,
    format: &ConditionalFormatOptions,
) -> (Option<XlsxCfRule>, Option<XlsxX14CfRule>) {
    let c = XlsxCfRule {
        priority: Some(p as i64 + 1),
        stop_if_true: Some(format.stop_if_true),
        r#type: Some("notContainsErrors".to_string()),
        formula: vec![format!("NOT(ISERROR({ref_str}))")],
        dxf_id: format.format,
        ..Default::default()
    };
    (Some(c), None)
}

fn draw_cond_fmt_blanks(
    p: i32,
    _ct: &str,
    ref_str: &str,
    _guid: &str,
    format: &ConditionalFormatOptions,
) -> (Option<XlsxCfRule>, Option<XlsxX14CfRule>) {
    let c = XlsxCfRule {
        priority: Some(p as i64 + 1),
        stop_if_true: Some(format.stop_if_true),
        r#type: Some("containsBlanks".to_string()),
        formula: vec![format!("LEN(TRIM({ref_str}))=0")],
        dxf_id: format.format,
        ..Default::default()
    };
    (Some(c), None)
}

fn draw_cond_fmt_no_blanks(
    p: i32,
    _ct: &str,
    ref_str: &str,
    _guid: &str,
    format: &ConditionalFormatOptions,
) -> (Option<XlsxCfRule>, Option<XlsxX14CfRule>) {
    let c = XlsxCfRule {
        priority: Some(p as i64 + 1),
        stop_if_true: Some(format.stop_if_true),
        r#type: Some("notContainsBlanks".to_string()),
        formula: vec![format!("LEN(TRIM({ref_str}))>0")],
        dxf_id: format.format,
        ..Default::default()
    };
    (Some(c), None)
}

fn draw_cond_fmt_icon_set(
    p: i32,
    _ct: &str,
    _ref: &str,
    guid: &str,
    format: &ConditionalFormatOptions,
) -> (Option<XlsxCfRule>, Option<XlsxX14CfRule>) {
    if let Some(preset) = ICON_SET_PRESETS.get(format.icon_style.as_str()) {
        let mut c = preset.clone();
        c.priority = Some(p as i64 + 1);
        c.r#type = Some("iconSet".to_string());
        if let Some(ref mut icon_set) = c.icon_set {
            icon_set.icon_set = Some(format.icon_style.clone());
            icon_set.reverse = Some(format.reverse_icons);
            icon_set.show_value = Some(!format.icons_only);
        }
        return (Some(c), None);
    }
    if let Some(preset) = X14_ICON_SET_PRESETS.get(format.icon_style.as_str()) {
        let mut x14 = preset.clone();
        x14.r#type = Some("iconSet".to_string());
        x14.priority = Some(p as i64 + 1);
        x14.stop_if_true = Some(format.stop_if_true);
        x14.above_average = Some(format.above_average);
        x14.percent = Some(format.percent);
        x14.id = Some(guid.to_string());
        if let Some(ref mut icon_set) = x14.icon_set {
            icon_set.icon_set = Some(format.icon_style.clone());
            icon_set.reverse = Some(format.reverse_icons);
            icon_set.show_value = Some(!format.icons_only);
        }
        return (None, Some(x14));
    }
    (None, None)
}

// ------------------------------------------------------------------
// Conditional formatting extract helpers
// ------------------------------------------------------------------

fn extract_cond_fmt_cell_is(
    _f: &File,
    _ref: &str,
    c: &XlsxCfRule,
    _ext_lst: Option<&XlsxExtLst>,
) -> ConditionalFormatOptions {
    let mut format = ConditionalFormatOptions {
        format: c.dxf_id,
        stop_if_true: c.stop_if_true.unwrap_or(false),
        r#type: "cell".to_string(),
        criteria: OPERATOR_TYPE
            .get(c.operator.as_deref().unwrap_or(""))
            .copied()
            .unwrap_or("")
            .to_string(),
        ..Default::default()
    };
    if c.formula.len() == 2 {
        format.min_value = c.formula[0].clone();
        format.max_value = c.formula[1].clone();
    } else if !c.formula.is_empty() {
        format.value = c.formula[0].clone();
    }
    format
}

fn time_period_criteria(ref_str: &str, formula: &str) -> Option<&'static str> {
    if formula == format!("FLOOR({ref_str},1)=TODAY()-1") {
        return Some("yesterday");
    }
    if formula == format!("FLOOR({ref_str},1)=TODAY()") {
        return Some("today");
    }
    if formula == format!("FLOOR({ref_str},1)=TODAY()+1") {
        return Some("tomorrow");
    }
    if formula == format!("AND(TODAY()-FLOOR({ref_str},1)<=6,FLOOR({ref_str},1)<=TODAY())") {
        return Some("last 7 days");
    }
    if formula
        == format!(
            "AND(TODAY()-ROUNDDOWN({ref_str},0)>=(WEEKDAY(TODAY())),TODAY()-ROUNDDOWN({ref_str},0)<(WEEKDAY(TODAY())+7))"
        )
    {
        return Some("last week");
    }
    if formula
        == format!(
            "AND(TODAY()-ROUNDDOWN({ref_str},0)<=WEEKDAY(TODAY())-1,ROUNDDOWN({ref_str},0)-TODAY()>=7-WEEKDAY(TODAY()))"
        )
    {
        return Some("this week");
    }
    if formula
        == format!(
            "AND(ROUNDDOWN({ref_str},0)-TODAY()>(7-WEEKDAY(TODAY())),ROUNDDOWN({ref_str},0)-TODAY()<(15-WEEKDAY(TODAY())))"
        )
    {
        return Some("continue week");
    }
    if formula
        == format!(
            "AND(MONTH({ref_str})=MONTH(TODAY())-1,OR(YEAR({ref_str})=YEAR(TODAY()),AND(MONTH({ref_str})=1,YEAR({ref_str})=YEAR(TODAY())-1)))"
        )
    {
        return Some("last month");
    }
    if formula == format!("AND(MONTH({ref_str})=MONTH(TODAY()),YEAR({ref_str})=YEAR(TODAY()))") {
        return Some("this month");
    }
    if formula
        == format!(
            "AND(MONTH({ref_str})=MONTH(TODAY())+1,OR(YEAR({ref_str})=YEAR(TODAY()),AND(MONTH({ref_str})=12,YEAR({ref_str})=YEAR(TODAY())+1)))"
        )
    {
        return Some("continue month");
    }
    None
}

fn extract_cond_fmt_time_period(
    _f: &File,
    ref_str: &str,
    c: &XlsxCfRule,
    _ext_lst: Option<&XlsxExtLst>,
) -> ConditionalFormatOptions {
    let mut criteria = String::new();
    for formula in &c.formula {
        if let Some(cr) = time_period_criteria(ref_str, formula) {
            criteria = cr.to_string();
            break;
        }
    }
    ConditionalFormatOptions {
        format: c.dxf_id,
        stop_if_true: c.stop_if_true.unwrap_or(false),
        r#type: "time_period".to_string(),
        criteria,
        ..Default::default()
    }
}

fn extract_cond_fmt_text(
    _f: &File,
    _ref: &str,
    c: &XlsxCfRule,
    _ext_lst: Option<&XlsxExtLst>,
) -> ConditionalFormatOptions {
    ConditionalFormatOptions {
        format: c.dxf_id,
        stop_if_true: c.stop_if_true.unwrap_or(false),
        r#type: "text".to_string(),
        criteria: OPERATOR_TYPE
            .get(c.operator.as_deref().unwrap_or(""))
            .copied()
            .unwrap_or("")
            .to_string(),
        value: c.text.clone().unwrap_or_default(),
        ..Default::default()
    }
}

fn extract_cond_fmt_top10(
    _f: &File,
    _ref: &str,
    c: &XlsxCfRule,
    _ext_lst: Option<&XlsxExtLst>,
) -> ConditionalFormatOptions {
    let mut format = ConditionalFormatOptions {
        format: c.dxf_id,
        stop_if_true: c.stop_if_true.unwrap_or(false),
        r#type: "top".to_string(),
        criteria: "=".to_string(),
        percent: c.percent.unwrap_or(false),
        value: c.rank.unwrap_or(10).to_string(),
        ..Default::default()
    };
    if c.bottom == Some(true) {
        format.r#type = "bottom".to_string();
    }
    format
}

fn extract_cond_fmt_above_average(
    _f: &File,
    _ref: &str,
    c: &XlsxCfRule,
    _ext_lst: Option<&XlsxExtLst>,
) -> ConditionalFormatOptions {
    ConditionalFormatOptions {
        format: c.dxf_id,
        stop_if_true: c.stop_if_true.unwrap_or(false),
        r#type: "average".to_string(),
        criteria: "=".to_string(),
        above_average: c.above_average.unwrap_or(false),
        ..Default::default()
    }
}

fn extract_cond_fmt_duplicate_unique(
    _f: &File,
    _ref: &str,
    c: &XlsxCfRule,
    _ext_lst: Option<&XlsxExtLst>,
) -> ConditionalFormatOptions {
    let t = match c.r#type.as_deref() {
        Some("duplicateValues") => "duplicate",
        Some("uniqueValues") => "unique",
        _ => "",
    };
    ConditionalFormatOptions {
        format: c.dxf_id,
        stop_if_true: c.stop_if_true.unwrap_or(false),
        r#type: t.to_string(),
        criteria: "=".to_string(),
        ..Default::default()
    }
}

fn extract_cond_fmt_blanks(
    _f: &File,
    _ref: &str,
    c: &XlsxCfRule,
    _ext_lst: Option<&XlsxExtLst>,
) -> ConditionalFormatOptions {
    ConditionalFormatOptions {
        format: c.dxf_id,
        stop_if_true: c.stop_if_true.unwrap_or(false),
        r#type: "blanks".to_string(),
        ..Default::default()
    }
}

fn extract_cond_fmt_no_blanks(
    _f: &File,
    _ref: &str,
    c: &XlsxCfRule,
    _ext_lst: Option<&XlsxExtLst>,
) -> ConditionalFormatOptions {
    ConditionalFormatOptions {
        format: c.dxf_id,
        stop_if_true: c.stop_if_true.unwrap_or(false),
        r#type: "no_blanks".to_string(),
        ..Default::default()
    }
}

fn extract_cond_fmt_errors(
    _f: &File,
    _ref: &str,
    c: &XlsxCfRule,
    _ext_lst: Option<&XlsxExtLst>,
) -> ConditionalFormatOptions {
    ConditionalFormatOptions {
        format: c.dxf_id,
        stop_if_true: c.stop_if_true.unwrap_or(false),
        r#type: "errors".to_string(),
        ..Default::default()
    }
}

fn extract_cond_fmt_no_errors(
    _f: &File,
    _ref: &str,
    c: &XlsxCfRule,
    _ext_lst: Option<&XlsxExtLst>,
) -> ConditionalFormatOptions {
    ConditionalFormatOptions {
        format: c.dxf_id,
        stop_if_true: c.stop_if_true.unwrap_or(false),
        r#type: "no_errors".to_string(),
        ..Default::default()
    }
}

fn extract_cond_fmt_color_scale(
    f: &File,
    _ref: &str,
    c: &XlsxCfRule,
    _ext_lst: Option<&XlsxExtLst>,
) -> ConditionalFormatOptions {
    let mut format = ConditionalFormatOptions {
        stop_if_true: c.stop_if_true.unwrap_or(false),
        r#type: "2_color_scale".to_string(),
        criteria: "=".to_string(),
        ..Default::default()
    };
    if let Some(ref cs) = c.color_scale {
        let values = cs.cfvo.len();
        let colors = cs.color.len();
        if colors > 1 && values > 1 {
            format.min_type = cs.cfvo[0].r#type.clone().unwrap_or_default();
            if cs.cfvo[0].val.as_deref() != Some("0") {
                format.min_value = cs.cfvo[0].val.clone().unwrap_or_default();
            }
            format.min_color = theme_color_to_hex(f, Some(&cs.color[0]));
            format.max_type = cs.cfvo[1].r#type.clone().unwrap_or_default();
            if cs.cfvo[1].val.as_deref() != Some("0") {
                format.max_value = cs.cfvo[1].val.clone().unwrap_or_default();
            }
            format.max_color = theme_color_to_hex(f, Some(&cs.color[1]));
        }
        if colors == 3 {
            format.r#type = "3_color_scale".to_string();
            format.mid_type = cs.cfvo[1].r#type.clone().unwrap_or_default();
            if cs.cfvo[1].val.as_deref() != Some("0") {
                format.mid_value = cs.cfvo[1].val.clone().unwrap_or_default();
            }
            format.mid_color = theme_color_to_hex(f, Some(&cs.color[1]));
            format.max_type = cs.cfvo[2].r#type.clone().unwrap_or_default();
            if cs.cfvo[2].val.as_deref() != Some("0") {
                format.max_value = cs.cfvo[2].val.clone().unwrap_or_default();
            }
            format.max_color = theme_color_to_hex(f, Some(&cs.color[2]));
        }
    }
    format
}

fn theme_color_to_hex(f: &File, color: Option<&XlsxColor>) -> String {
    let Some(color) = color else {
        return String::new();
    };
    let rgb = f.get_base_color(
        color.rgb.as_deref().unwrap_or(""),
        color.indexed.unwrap_or(0),
        color.theme,
    );
    if rgb.is_empty() {
        return String::new();
    }
    let tint = color.tint.unwrap_or(0.0);
    let hex = if tint != 0.0 {
        if let Some(s) = theme_color(&rgb, tint).strip_prefix("FF") {
            s.to_string()
        } else {
            rgb
        }
    } else {
        rgb
    };
    format!("#{}", hex)
}

fn extract_cond_fmt_data_bar(
    f: &File,
    _ref: &str,
    c: &XlsxCfRule,
    ext_lst: Option<&XlsxExtLst>,
) -> ConditionalFormatOptions {
    let mut format = ConditionalFormatOptions {
        r#type: "data_bar".to_string(),
        criteria: "=".to_string(),
        stop_if_true: c.stop_if_true.unwrap_or(false),
        ..Default::default()
    };
    if let Some(ref db) = c.data_bar {
        if !db.cfvo.is_empty() {
            format.min_type = db.cfvo[0].r#type.clone().unwrap_or_default();
            format.min_value = db.cfvo[0].val.clone().unwrap_or_default();
        }
        if db.cfvo.len() > 1 {
            format.max_type = db.cfvo[1].r#type.clone().unwrap_or_default();
            format.max_value = db.cfvo[1].val.clone().unwrap_or_default();
        }
        if let Some(ref color) = db.color.first() {
            format.bar_color = theme_color_to_hex(f, Some(color));
        }
        if let Some(show) = db.show_value {
            format.bar_only = !show;
        }
    }
    if let Some(ref ext) = c.ext_lst {
        if let Some(id_ext) = ext.ext.first() {
            let id = x14_rule_id_from_ext(&id_ext.content);
            if let Some(ext_lst) = ext_lst {
                extract_cond_fmt_data_bar_rule(f, &id, &mut format, ext_lst);
            }
        }
    }
    format
}

fn x14_rule_id_from_ext(content: &str) -> String {
    content
        .trim_start_matches("<x14:id>")
        .trim_end_matches("</x14:id>")
        .to_string()
}

fn extract_cond_fmt_data_bar_rule(
    f: &File,
    id: &str,
    format: &mut ConditionalFormatOptions,
    ext_lst: &XlsxExtLst,
) {
    for ext in &ext_lst.ext {
        if ext.uri.as_deref() == Some(EXT_URI_CONDITIONAL_FORMATTINGS) {
            if let Ok(decoded) = parse_x14_conditional_formattings(&ext.content) {
                for cond_fmt in &decoded.cond_fmt {
                    for rule in &cond_fmt.cf_rule {
                        if rule.data_bar.is_some() && rule.id.as_deref() == Some(id) {
                            if let Some(ref db) = rule.data_bar {
                                format.bar_direction = db.direction.clone().unwrap_or_default();
                                if let Some(gradient) = db.gradient {
                                    if !gradient {
                                        format.bar_solid = true;
                                    }
                                }
                                if let Some(ref bc) = db.border_color {
                                    format.bar_border_color = theme_color_to_hex(f, Some(bc));
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn extract_cond_fmt_exp(
    _f: &File,
    _ref: &str,
    c: &XlsxCfRule,
    _ext_lst: Option<&XlsxExtLst>,
) -> ConditionalFormatOptions {
    let mut format = ConditionalFormatOptions {
        format: c.dxf_id,
        stop_if_true: c.stop_if_true.unwrap_or(false),
        r#type: "formula".to_string(),
        ..Default::default()
    };
    if !c.formula.is_empty() {
        format.criteria = c.formula[0].clone();
    }
    format
}

fn extract_cond_fmt_icon_set(
    _f: &File,
    _ref: &str,
    c: &XlsxCfRule,
    _ext_lst: Option<&XlsxExtLst>,
) -> ConditionalFormatOptions {
    let mut format = ConditionalFormatOptions {
        r#type: "icon_set".to_string(),
        ..Default::default()
    };
    if let Some(ref icon_set) = c.icon_set {
        if let Some(show) = icon_set.show_value {
            format.icons_only = !show;
        }
        format.icon_style = icon_set.icon_set.clone().unwrap_or_default();
        format.reverse_icons = icon_set.reverse.unwrap_or(false);
    }
    format
}

fn extract_x14_cond_fmt_icon_set(c: &DecodeX14CfRule) -> ConditionalFormatOptions {
    let mut format = ConditionalFormatOptions {
        r#type: "icon_set".to_string(),
        ..Default::default()
    };
    if let Some(ref icon_set) = c.icon_set {
        if let Some(show) = icon_set.show_value {
            format.icons_only = !show;
        }
        format.icon_style = icon_set.icon_set.clone().unwrap_or_default();
        format.reverse_icons = icon_set.reverse.unwrap_or(false);
    }
    format
}

// ------------------------------------------------------------------
// Tests
// ------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::options::Options;

    #[test]
    fn create_and_read_style() {
        let f = File::new_with_options(Options::default());

        let style = Style {
            font: Some(Font {
                name: Some("Arial".to_string()),
                size: Some(12.0),
                bold: Some(true),
                color: Some("FF0000".to_string()),
                ..Default::default()
            }),
            fill: Fill {
                r#type: "pattern".to_string(),
                pattern: 1,
                color: vec!["FFFF00".to_string()],
                ..Default::default()
            },
            alignment: Some(Alignment {
                horizontal: "center".to_string(),
                vertical: "middle".to_string(),
                ..Default::default()
            }),
            ..Default::default()
        };

        let id = f.new_style(&style).unwrap();
        assert!(id > 0);

        let read = f.get_style(id).unwrap();
        assert_eq!(read.font.as_ref().unwrap().name, Some("Arial".to_string()));
        assert_eq!(read.font.as_ref().unwrap().size, Some(12.0));
        assert_eq!(read.font.as_ref().unwrap().bold, Some(true));
        assert_eq!(
            read.font.as_ref().unwrap().color,
            Some("FF0000".to_string())
        );
        assert_eq!(read.fill.r#type, "pattern");
        assert_eq!(read.fill.pattern, 1);
        assert_eq!(read.fill.color, vec!["FFFF00".to_string()]);
        assert_eq!(read.alignment.as_ref().unwrap().horizontal, "center");
        assert_eq!(read.alignment.as_ref().unwrap().vertical, "middle");
    }

    #[test]
    fn font_color_rgb_strip_alpha() {
        // 8-char RGBA with opaque alpha: strip the leading FF.
        let fnt = XlsxFont {
            color: Some(XlsxColor {
                rgb: Some("FFFF0000".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };
        assert_eq!(extract_font(&fnt).color, Some("FF0000".to_string()));

        // 6-char RGB that happens to start with FF is also trimmed, matching Go.
        let fnt6 = XlsxFont {
            color: Some(XlsxColor {
                rgb: Some("FF0000".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };
        assert_eq!(extract_font(&fnt6).color, Some("0000".to_string()));

        // 6-char RGB not starting with FF is preserved.
        let fnt_green = XlsxFont {
            color: Some(XlsxColor {
                rgb: Some("00FF00".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };
        assert_eq!(extract_font(&fnt_green).color, Some("00FF00".to_string()));
    }

    #[test]
    fn font_color_indexed_only_when_set() {
        let font = Font {
            color: Some("FF0000".to_string()),
            ..Default::default()
        };
        let color = new_font_color(&font).unwrap();
        assert!(color.indexed.is_none());

        let font_indexed = Font {
            color: Some("FF0000".to_string()),
            color_indexed: Some(0),
            ..Default::default()
        };
        let color_indexed = new_font_color(&font_indexed).unwrap();
        assert_eq!(color_indexed.indexed, Some(0));
    }

    #[test]
    fn default_font_round_trip() {
        let f = File::new_with_options(Options::default());
        assert_eq!(f.get_default_font().unwrap(), "宋体");
        f.set_default_font("Arial").unwrap();
        assert_eq!(f.get_default_font().unwrap(), "Arial");
    }

    #[test]
    fn set_and_get_conditional_format_data_bar() {
        let f = File::new_with_options(Options::default());
        f.set_conditional_format(
            "Sheet1",
            "A1:A10",
            &[ConditionalFormatOptions {
                r#type: "data_bar".to_string(),
                criteria: "=".to_string(),
                min_type: "min".to_string(),
                max_type: "max".to_string(),
                bar_color: "#638EC6".to_string(),
                bar_solid: true,
                bar_direction: "leftToRight".to_string(),
                bar_border_color: "#000000".to_string(),
                ..Default::default()
            }],
        )
        .unwrap();

        let cfs = f.get_conditional_formats("Sheet1").unwrap();
        let rules = cfs.get("A1:A10").unwrap();
        eprintln!("rules[0] = {:?}", rules[0]);
        assert_eq!(rules[0].r#type, "data_bar");
        assert_eq!(rules[0].bar_color, "#638EC6");
        assert!(rules[0].bar_solid, "bar_solid should be true");
        assert_eq!(rules[0].bar_direction, "leftToRight");
        assert_eq!(rules[0].bar_border_color, "#000000");
    }

    #[test]
    fn conditional_format_round_trip() {
        let tmp = std::env::temp_dir().join("excelize_rust_conditional_format_round_trip.xlsx");
        let _ = std::fs::remove_file(&tmp);

        let mut f = File::new_with_options(Options::default());
        f.set_conditional_format(
            "Sheet1",
            "A1:A10",
            &[ConditionalFormatOptions {
                r#type: "duplicate".to_string(),
                criteria: "=".to_string(),
                ..Default::default()
            }],
        )
        .unwrap();

        f.save_as(tmp.to_str().unwrap()).unwrap();
        let f2 = File::open_file(tmp.to_str().unwrap(), Options::default()).unwrap();
        let cfs = f2.get_conditional_formats("Sheet1").unwrap();
        assert_eq!(cfs.len(), 1);
        let rules = cfs.get("A1:A10").unwrap();
        assert_eq!(rules[0].r#type, "duplicate");
        assert_eq!(rules[0].criteria, "=");
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn conditional_style_round_trip() {
        let f = File::new_with_options(Options::default());
        let style = Style {
            font: Some(Font {
                color: Some("9A0511".to_string()),
                ..Default::default()
            }),
            fill: Fill {
                r#type: "pattern".to_string(),
                pattern: 1,
                color: vec!["FEC7CE".to_string()],
                ..Default::default()
            },
            ..Default::default()
        };
        let id = f.new_conditional_style(&style).unwrap();
        let read = f.get_conditional_style(id).unwrap();
        assert_eq!(read.fill.r#type, "pattern");
        assert_eq!(read.fill.pattern, 1);
        assert_eq!(read.fill.color, vec!["FEC7CE".to_string()]);
        assert_eq!(
            read.font.as_ref().unwrap().color,
            Some("9A0511".to_string())
        );
    }

    #[test]
    fn set_and_get_conditional_format_cell() {
        let f = File::new_with_options(Options::default());
        let style = Style {
            font: Some(Font {
                color: Some("9A0511".to_string()),
                ..Default::default()
            }),
            fill: Fill {
                r#type: "pattern".to_string(),
                pattern: 1,
                color: vec!["FEC7CE".to_string()],
                ..Default::default()
            },
            ..Default::default()
        };
        let format = f.new_conditional_style(&style).unwrap();
        f.set_conditional_format(
            "Sheet1",
            "A1:A10",
            &[ConditionalFormatOptions {
                r#type: "cell".to_string(),
                criteria: ">".to_string(),
                value: "5".to_string(),
                format: Some(format as i64),
                ..Default::default()
            }],
        )
        .unwrap();

        let cfs = f.get_conditional_formats("Sheet1").unwrap();
        assert!(cfs.contains_key("A1:A10"));
        let rules = cfs.get("A1:A10").unwrap();
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].r#type, "cell");
        assert_eq!(rules[0].criteria, "greater than");
        assert_eq!(rules[0].value, "5");
        assert_eq!(rules[0].format, Some(format as i64));
    }

    #[test]
    fn set_and_get_conditional_format_color_scale() {
        let f = File::new_with_options(Options::default());
        f.set_conditional_format(
            "Sheet1",
            "A1:A10",
            &[ConditionalFormatOptions {
                r#type: "2_color_scale".to_string(),
                criteria: "=".to_string(),
                min_type: "min".to_string(),
                max_type: "max".to_string(),
                min_color: "#F8696B".to_string(),
                max_color: "#63BE7B".to_string(),
                ..Default::default()
            }],
        )
        .unwrap();

        let cfs = f.get_conditional_formats("Sheet1").unwrap();
        let rules = cfs.get("A1:A10").unwrap();
        assert_eq!(rules[0].r#type, "2_color_scale");
        assert_eq!(rules[0].min_type, "min");
        assert_eq!(rules[0].max_type, "max");
        assert_eq!(rules[0].min_color, "#F8696B");
        assert_eq!(rules[0].max_color, "#63BE7B");
    }

    #[test]
    fn unset_conditional_format() {
        let f = File::new_with_options(Options::default());
        f.set_conditional_format(
            "Sheet1",
            "A1:A10",
            &[ConditionalFormatOptions {
                r#type: "duplicate".to_string(),
                criteria: "=".to_string(),
                ..Default::default()
            }],
        )
        .unwrap();
        let cfs = f.get_conditional_formats("Sheet1").unwrap();
        assert_eq!(cfs.len(), 1);

        f.unset_conditional_format("Sheet1", "A1:A5").unwrap();
        let cfs = f.get_conditional_formats("Sheet1").unwrap();
        assert_eq!(cfs.len(), 1);

        f.unset_conditional_format("Sheet1", "A6:A10").unwrap();
        let cfs = f.get_conditional_formats("Sheet1").unwrap();
        assert!(cfs.is_empty());
    }

    #[test]
    fn get_base_color_basic() {
        let f = File::new_with_options(Options::default());
        assert_eq!(f.get_base_color("FF0000", 0, None), "FF0000");
        assert_eq!(f.get_base_color("FFFF0000", 0, None), "FF0000");
        assert_eq!(f.get_base_color("", 3, None), INDEXED_COLOR_MAPPING[3]);
        assert_eq!(f.get_base_color("unknown", 99, None), "unknown");
    }
}
