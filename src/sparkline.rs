//! Sparkline support.
//!
//! Ported from Go `sparkline.go`.

use quick_xml::se::to_string as xml_to_string;

use crate::File;
use crate::constants::{
    EXT_URI_SPARKLINE_GROUPS, NAMESPACE_SPREADSHEET_EXCEL_2006_MAIN, NAMESPACE_SPREADSHEET_X14,
    WORKSHEET_EXT_URI_PRIORITY,
};
use crate::errors::{
    ErrSparkline, ErrSparklineLocation, ErrSparklineRange, ErrSparklineStyle, ErrSparklineType,
    Result,
};
use crate::lib_util::{in_str_slice, int_ptr};
use crate::xml::common::{XlsxColor, XlsxExt, XlsxExtLst};
use crate::xml::worksheet::{
    SparklineOptions, XlsxWorksheet, XlsxX14Sparkline, XlsxX14SparklineGroup,
    XlsxX14SparklineGroups,
};

impl File {
    /// Add sparklines to a worksheet.
    ///
    /// Sparklines are small charts that fit in a single cell and are used to
    /// show trends in data. Sparklines are a feature of Excel 2010 and later
    /// only.
    pub fn add_sparkline(&self, sheet: &str, opts: &SparklineOptions) -> Result<()> {
        let mut ws = self.parse_format_add_sparkline_set(sheet, opts)?;

        let spark_types = [
            ("line", "line"),
            ("column", "column"),
            ("win_loss", "stacked"),
        ]
        .iter()
        .cloned()
        .collect::<std::collections::HashMap<_, _>>();
        let mut spark_type = "line".to_string();
        if !opts.r#type.is_empty() {
            spark_type = spark_types
                .get(opts.r#type.as_str())
                .ok_or_else(|| ErrSparklineType)?
                .to_string();
        }

        let presets = get_sparkline_group_presets();
        let mut group = presets
            .get(opts.style as usize)
            .cloned()
            .unwrap_or_else(|| presets[0].clone());
        group.r#type = Some(spark_type);
        group.color_axis = Some(XlsxColor {
            rgb: Some("FF000000".to_string()),
            ..Default::default()
        });
        group.display_empty_cells_as = Some("gap".to_string());
        group.high = Some(opts.high).filter(|&v| v);
        group.low = Some(opts.low).filter(|&v| v);
        group.first = Some(opts.first).filter(|&v| v);
        group.last = Some(opts.last).filter(|&v| v);
        group.negative = Some(opts.negative).filter(|&v| v);
        group.display_x_axis = Some(opts.axis).filter(|&v| v);
        group.markers = Some(opts.markers).filter(|&v| v);
        if !opts.series_color.is_empty() {
            group.color_series = Some(XlsxColor {
                rgb: Some(crate::styles::get_palette_color(&opts.series_color)),
                ..Default::default()
            });
        }
        if opts.reverse {
            group.right_to_left = Some(opts.reverse);
        }

        add_sparkline_items(opts, &mut group);
        self.append_sparkline_group(&mut ws, &group)?;
        self.add_sheet_name_space(sheet, NAMESPACE_SPREADSHEET_X14);

        let path =
            self.get_sheet_xml_path(sheet)
                .ok_or_else(|| crate::errors::ErrSheetNotExist {
                    sheet_name: sheet.to_string(),
                })?;
        self.sheet.insert(path, ws);
        Ok(())
    }

    /// Validate sparkline properties.
    fn parse_format_add_sparkline_set(
        &self,
        sheet: &str,
        opts: &SparklineOptions,
    ) -> Result<XlsxWorksheet> {
        let mut ws = self.work_sheet_reader(sheet)?;
        if opts.location.is_empty() {
            return Err(Box::new(ErrSparklineLocation));
        }
        if opts.range.is_empty() {
            return Err(Box::new(ErrSparklineRange));
        }
        if opts.location.len() != opts.range.len() {
            return Err(Box::new(ErrSparkline));
        }
        if opts.style < 0 || opts.style > 35 {
            return Err(Box::new(ErrSparklineStyle));
        }
        if ws.ext_lst.is_none() {
            ws.ext_lst = Some(XlsxExtLst::default());
        }
        Ok(ws)
    }

    /// Append a sparkline group to the worksheet extension list.
    fn append_sparkline_group(
        &self,
        ws: &mut XlsxWorksheet,
        group: &XlsxX14SparklineGroup,
    ) -> Result<()> {
        let new_group_xml = xml_to_string(group)?;
        let mut append_mode = false;

        let ext_lst = ws.ext_lst.as_mut().unwrap();
        for ext in &mut ext_lst.ext {
            if ext.uri.as_deref() == Some(EXT_URI_SPARKLINE_GROUPS) {
                let new_content = if ext.content.contains("</x14:sparklineGroups>") {
                    ext.content.replacen(
                        "</x14:sparklineGroups>",
                        &format!("{new_group_xml}</x14:sparklineGroups>"),
                        1,
                    )
                } else {
                    format!(
                        r#"<x14:sparklineGroups xmlns:xm="{}">{new_group_xml}</x14:sparklineGroups>"#,
                        NAMESPACE_SPREADSHEET_EXCEL_2006_MAIN
                    )
                };
                ext.content = new_content;
                append_mode = true;
            }
        }

        if !append_mode {
            let groups = XlsxX14SparklineGroups {
                xmlns_xm: Some(NAMESPACE_SPREADSHEET_EXCEL_2006_MAIN.to_string()),
                sparkline_groups: vec![group.clone()],
                ..Default::default()
            };
            let groups_xml = xml_to_string(&groups)?;
            ext_lst.ext.push(XlsxExt {
                uri: Some(EXT_URI_SPARKLINE_GROUPS.to_string()),
                content: groups_xml,
                ..Default::default()
            });
        }

        ext_lst.ext.sort_by(|a, b| {
            let ia = in_str_slice(
                WORKSHEET_EXT_URI_PRIORITY,
                a.uri.as_deref().unwrap_or(""),
                false,
            );
            let ib = in_str_slice(
                WORKSHEET_EXT_URI_PRIORITY,
                b.uri.as_deref().unwrap_or(""),
                false,
            );
            ia.cmp(&ib)
        });

        Ok(())
    }
}

/// Add individual sparkline entries to a sparkline group.
fn add_sparkline_items(opts: &SparklineOptions, group: &mut XlsxX14SparklineGroup) {
    for (location, range) in opts.location.iter().zip(opts.range.iter()) {
        group.sparklines.sparkline.push(XlsxX14Sparkline {
            f: range.clone(),
            sqref: location.clone(),
        });
    }
}

/// Build a color from a theme index and optional tint.
fn theme_color(theme: i64, tint: Option<f64>) -> Option<XlsxColor> {
    Some(XlsxColor {
        theme: int_ptr(theme),
        tint,
        ..Default::default()
    })
}

/// Build a color from an RGB hex string.
fn rgb_color(rgb: &str) -> Option<XlsxColor> {
    Some(XlsxColor {
        rgb: Some(rgb.to_string()),
        ..Default::default()
    })
}

/// Return the preset list of sparkline groups used to create x14:sparklineGroups.
#[allow(clippy::too_many_lines)]
fn get_sparkline_group_presets() -> Vec<XlsxX14SparklineGroup> {
    vec![
        XlsxX14SparklineGroup {
            color_series: theme_color(4, Some(-0.499984740745262)),
            color_negative: theme_color(5, None),
            color_markers: theme_color(4, Some(-0.499984740745262)),
            color_first: theme_color(4, Some(0.39997558519241921)),
            color_last: theme_color(4, Some(0.39997558519241921)),
            color_high: theme_color(4, None),
            color_low: theme_color(4, None),
            ..Default::default()
        }, // 0
        XlsxX14SparklineGroup {
            color_series: theme_color(4, Some(-0.499984740745262)),
            color_negative: theme_color(5, None),
            color_markers: theme_color(4, Some(-0.499984740745262)),
            color_first: theme_color(4, Some(0.39997558519241921)),
            color_last: theme_color(4, Some(0.39997558519241921)),
            color_high: theme_color(4, None),
            color_low: theme_color(4, None),
            ..Default::default()
        }, // 1
        XlsxX14SparklineGroup {
            color_series: theme_color(5, Some(-0.499984740745262)),
            color_negative: theme_color(6, None),
            color_markers: theme_color(5, Some(-0.499984740745262)),
            color_first: theme_color(5, Some(0.39997558519241921)),
            color_last: theme_color(5, Some(0.39997558519241921)),
            color_high: theme_color(5, None),
            color_low: theme_color(5, None),
            ..Default::default()
        }, // 2
        XlsxX14SparklineGroup {
            color_series: theme_color(6, Some(-0.499984740745262)),
            color_negative: theme_color(7, None),
            color_markers: theme_color(6, Some(-0.499984740745262)),
            color_first: theme_color(6, Some(0.39997558519241921)),
            color_last: theme_color(6, Some(0.39997558519241921)),
            color_high: theme_color(6, None),
            color_low: theme_color(6, None),
            ..Default::default()
        }, // 3
        XlsxX14SparklineGroup {
            color_series: theme_color(7, Some(-0.499984740745262)),
            color_negative: theme_color(8, None),
            color_markers: theme_color(7, Some(-0.499984740745262)),
            color_first: theme_color(7, Some(0.39997558519241921)),
            color_last: theme_color(7, Some(0.39997558519241921)),
            color_high: theme_color(7, None),
            color_low: theme_color(7, None),
            ..Default::default()
        }, // 4
        XlsxX14SparklineGroup {
            color_series: theme_color(8, Some(-0.499984740745262)),
            color_negative: theme_color(9, None),
            color_markers: theme_color(8, Some(-0.499984740745262)),
            color_first: theme_color(8, Some(0.39997558519241921)),
            color_last: theme_color(8, Some(0.39997558519241921)),
            color_high: theme_color(8, None),
            color_low: theme_color(8, None),
            ..Default::default()
        }, // 5
        XlsxX14SparklineGroup {
            color_series: theme_color(9, Some(-0.499984740745262)),
            color_negative: theme_color(4, None),
            color_markers: theme_color(9, Some(-0.499984740745262)),
            color_first: theme_color(9, Some(0.39997558519241921)),
            color_last: theme_color(9, Some(0.39997558519241921)),
            color_high: theme_color(9, None),
            color_low: theme_color(9, None),
            ..Default::default()
        }, // 6
        XlsxX14SparklineGroup {
            color_series: theme_color(4, Some(-0.249977111117893)),
            color_negative: theme_color(5, None),
            color_markers: theme_color(5, Some(-0.249977111117893)),
            color_first: theme_color(5, Some(-0.249977111117893)),
            color_last: theme_color(5, Some(-0.249977111117893)),
            color_high: theme_color(5, None),
            color_low: theme_color(5, None),
            ..Default::default()
        }, // 7
        XlsxX14SparklineGroup {
            color_series: theme_color(5, Some(-0.249977111117893)),
            color_negative: theme_color(6, None),
            color_markers: theme_color(6, Some(-0.249977111117893)),
            color_first: theme_color(6, Some(-0.249977111117893)),
            color_last: theme_color(6, Some(-0.249977111117893)),
            color_high: theme_color(6, Some(-0.249977111117893)),
            color_low: theme_color(6, Some(-0.249977111117893)),
            ..Default::default()
        }, // 8
        XlsxX14SparklineGroup {
            color_series: theme_color(6, Some(-0.249977111117893)),
            color_negative: theme_color(7, None),
            color_markers: theme_color(7, Some(-0.249977111117893)),
            color_first: theme_color(7, Some(-0.249977111117893)),
            color_last: theme_color(7, Some(-0.249977111117893)),
            color_high: theme_color(7, Some(-0.249977111117893)),
            color_low: theme_color(7, Some(-0.249977111117893)),
            ..Default::default()
        }, // 9
        XlsxX14SparklineGroup {
            color_series: theme_color(7, Some(-0.249977111117893)),
            color_negative: theme_color(8, None),
            color_markers: theme_color(8, Some(-0.249977111117893)),
            color_first: theme_color(8, Some(-0.249977111117893)),
            color_last: theme_color(8, Some(-0.249977111117893)),
            color_high: theme_color(8, Some(-0.249977111117893)),
            color_low: theme_color(8, Some(-0.249977111117893)),
            ..Default::default()
        }, // 10
        XlsxX14SparklineGroup {
            color_series: theme_color(8, Some(-0.249977111117893)),
            color_negative: theme_color(9, None),
            color_markers: theme_color(9, Some(-0.249977111117893)),
            color_first: theme_color(9, Some(-0.249977111117893)),
            color_last: theme_color(9, Some(-0.249977111117893)),
            color_high: theme_color(9, Some(-0.249977111117893)),
            color_low: theme_color(9, Some(-0.249977111117893)),
            ..Default::default()
        }, // 11
        XlsxX14SparklineGroup {
            color_series: theme_color(9, Some(-0.249977111117893)),
            color_negative: theme_color(4, None),
            color_markers: theme_color(4, Some(-0.249977111117893)),
            color_first: theme_color(4, Some(-0.249977111117893)),
            color_last: theme_color(4, Some(-0.249977111117893)),
            color_high: theme_color(4, Some(-0.249977111117893)),
            color_low: theme_color(4, Some(-0.249977111117893)),
            ..Default::default()
        }, // 12
        XlsxX14SparklineGroup {
            color_series: theme_color(4, None),
            color_negative: theme_color(5, None),
            color_markers: theme_color(4, Some(-0.249977111117893)),
            color_first: theme_color(4, Some(-0.249977111117893)),
            color_last: theme_color(4, Some(-0.249977111117893)),
            color_high: theme_color(4, Some(-0.249977111117893)),
            color_low: theme_color(4, Some(-0.249977111117893)),
            ..Default::default()
        }, // 13
        XlsxX14SparklineGroup {
            color_series: theme_color(5, None),
            color_negative: theme_color(6, None),
            color_markers: theme_color(5, Some(-0.249977111117893)),
            color_first: theme_color(5, Some(-0.249977111117893)),
            color_last: theme_color(5, Some(-0.249977111117893)),
            color_high: theme_color(5, Some(-0.249977111117893)),
            color_low: theme_color(5, Some(-0.249977111117893)),
            ..Default::default()
        }, // 14
        XlsxX14SparklineGroup {
            color_series: theme_color(6, None),
            color_negative: theme_color(7, None),
            color_markers: theme_color(6, Some(-0.249977111117893)),
            color_first: theme_color(6, Some(-0.249977111117893)),
            color_last: theme_color(6, Some(-0.249977111117893)),
            color_high: theme_color(6, Some(-0.249977111117893)),
            color_low: theme_color(6, Some(-0.249977111117893)),
            ..Default::default()
        }, // 15
        XlsxX14SparklineGroup {
            color_series: theme_color(7, None),
            color_negative: theme_color(8, None),
            color_markers: theme_color(7, Some(-0.249977111117893)),
            color_first: theme_color(7, Some(-0.249977111117893)),
            color_last: theme_color(7, Some(-0.249977111117893)),
            color_high: theme_color(7, Some(-0.249977111117893)),
            color_low: theme_color(7, Some(-0.249977111117893)),
            ..Default::default()
        }, // 16
        XlsxX14SparklineGroup {
            color_series: theme_color(8, None),
            color_negative: theme_color(9, None),
            color_markers: theme_color(8, Some(-0.249977111117893)),
            color_first: theme_color(8, Some(-0.249977111117893)),
            color_last: theme_color(8, Some(-0.249977111117893)),
            color_high: theme_color(8, Some(-0.249977111117893)),
            color_low: theme_color(8, Some(-0.249977111117893)),
            ..Default::default()
        }, // 17
        XlsxX14SparklineGroup {
            color_series: theme_color(9, None),
            color_negative: theme_color(4, None),
            color_markers: theme_color(9, Some(-0.249977111117893)),
            color_first: theme_color(9, Some(-0.249977111117893)),
            color_last: theme_color(9, Some(-0.249977111117893)),
            color_high: theme_color(9, Some(-0.249977111117893)),
            color_low: theme_color(9, Some(-0.249977111117893)),
            ..Default::default()
        }, // 18
        XlsxX14SparklineGroup {
            color_series: theme_color(4, Some(0.39997558519241921)),
            color_negative: theme_color(0, Some(-0.499984740745262)),
            color_markers: theme_color(4, Some(0.79998168889431442)),
            color_first: theme_color(4, Some(-0.249977111117893)),
            color_last: theme_color(4, Some(-0.249977111117893)),
            color_high: theme_color(4, Some(-0.499984740745262)),
            color_low: theme_color(4, Some(-0.499984740745262)),
            ..Default::default()
        }, // 19
        XlsxX14SparklineGroup {
            color_series: theme_color(5, Some(0.39997558519241921)),
            color_negative: theme_color(0, Some(-0.499984740745262)),
            color_markers: theme_color(5, Some(0.79998168889431442)),
            color_first: theme_color(5, Some(-0.249977111117893)),
            color_last: theme_color(5, Some(-0.249977111117893)),
            color_high: theme_color(5, Some(-0.499984740745262)),
            color_low: theme_color(5, Some(-0.499984740745262)),
            ..Default::default()
        }, // 20
        XlsxX14SparklineGroup {
            color_series: theme_color(6, Some(0.39997558519241921)),
            color_negative: theme_color(0, Some(-0.499984740745262)),
            color_markers: theme_color(6, Some(0.79998168889431442)),
            color_first: theme_color(6, Some(-0.249977111117893)),
            color_last: theme_color(6, Some(-0.249977111117893)),
            color_high: theme_color(6, Some(-0.499984740745262)),
            color_low: theme_color(6, Some(-0.499984740745262)),
            ..Default::default()
        }, // 21
        XlsxX14SparklineGroup {
            color_series: theme_color(7, Some(0.39997558519241921)),
            color_negative: theme_color(0, Some(-0.499984740745262)),
            color_markers: theme_color(7, Some(0.79998168889431442)),
            color_first: theme_color(7, Some(-0.249977111117893)),
            color_last: theme_color(7, Some(-0.249977111117893)),
            color_high: theme_color(7, Some(-0.499984740745262)),
            color_low: theme_color(7, Some(-0.499984740745262)),
            ..Default::default()
        }, // 22
        XlsxX14SparklineGroup {
            color_series: theme_color(8, Some(0.39997558519241921)),
            color_negative: theme_color(0, Some(-0.499984740745262)),
            color_markers: theme_color(8, Some(0.79998168889431442)),
            color_first: theme_color(8, Some(-0.249977111117893)),
            color_last: theme_color(8, Some(-0.249977111117893)),
            color_high: theme_color(8, Some(-0.499984740745262)),
            color_low: theme_color(8, Some(-0.499984740745262)),
            ..Default::default()
        }, // 23
        XlsxX14SparklineGroup {
            color_series: theme_color(9, Some(0.39997558519241921)),
            color_negative: theme_color(0, Some(-0.499984740745262)),
            color_markers: theme_color(9, Some(0.79998168889431442)),
            color_first: theme_color(9, Some(-0.249977111117893)),
            color_last: theme_color(9, Some(-0.249977111117893)),
            color_high: theme_color(9, Some(-0.499984740745262)),
            color_low: theme_color(9, Some(-0.499984740745262)),
            ..Default::default()
        }, // 24
        XlsxX14SparklineGroup {
            color_series: theme_color(1, Some(0.499984740745262)),
            color_negative: theme_color(1, Some(0.249977111117893)),
            color_markers: theme_color(1, Some(0.249977111117893)),
            color_first: theme_color(1, Some(0.249977111117893)),
            color_last: theme_color(1, Some(0.249977111117893)),
            color_high: theme_color(1, Some(0.249977111117893)),
            color_low: theme_color(1, Some(0.249977111117893)),
            ..Default::default()
        }, // 25
        XlsxX14SparklineGroup {
            color_series: theme_color(1, Some(0.34998626667073579)),
            color_negative: theme_color(0, Some(0.249977111117893)),
            color_markers: theme_color(0, Some(0.249977111117893)),
            color_first: theme_color(0, Some(0.249977111117893)),
            color_last: theme_color(0, Some(0.249977111117893)),
            color_high: theme_color(0, Some(0.249977111117893)),
            color_low: theme_color(0, Some(0.249977111117893)),
            ..Default::default()
        }, // 26
        XlsxX14SparklineGroup {
            color_series: rgb_color("FF323232"),
            color_negative: rgb_color("FFD00000"),
            color_markers: rgb_color("FFD00000"),
            color_first: rgb_color("FFD00000"),
            color_last: rgb_color("FFD00000"),
            color_high: rgb_color("FFD00000"),
            color_low: rgb_color("FFD00000"),
            ..Default::default()
        }, // 27
        XlsxX14SparklineGroup {
            color_series: rgb_color("FF000000"),
            color_negative: rgb_color("FF0070C0"),
            color_markers: rgb_color("FF0070C0"),
            color_first: rgb_color("FF0070C0"),
            color_last: rgb_color("FF0070C0"),
            color_high: rgb_color("FF0070C0"),
            color_low: rgb_color("FF0070C0"),
            ..Default::default()
        }, // 28
        XlsxX14SparklineGroup {
            color_series: rgb_color("FF376092"),
            color_negative: rgb_color("FFD00000"),
            color_markers: rgb_color("FFD00000"),
            color_first: rgb_color("FFD00000"),
            color_last: rgb_color("FFD00000"),
            color_high: rgb_color("FFD00000"),
            color_low: rgb_color("FFD00000"),
            ..Default::default()
        }, // 29
        XlsxX14SparklineGroup {
            color_series: rgb_color("FF0070C0"),
            color_negative: rgb_color("FF000000"),
            color_markers: rgb_color("FF000000"),
            color_first: rgb_color("FF000000"),
            color_last: rgb_color("FF000000"),
            color_high: rgb_color("FF000000"),
            color_low: rgb_color("FF000000"),
            ..Default::default()
        }, // 30
        XlsxX14SparklineGroup {
            color_series: rgb_color("FF5F5F5F"),
            color_negative: rgb_color("FFFFB620"),
            color_markers: rgb_color("FFD70077"),
            color_first: rgb_color("FF5687C2"),
            color_last: rgb_color("FF359CEB"),
            color_high: rgb_color("FF56BE79"),
            color_low: rgb_color("FFFF5055"),
            ..Default::default()
        }, // 31
        XlsxX14SparklineGroup {
            color_series: rgb_color("FF5687C2"),
            color_negative: rgb_color("FFFFB620"),
            color_markers: rgb_color("FFD70077"),
            color_first: rgb_color("FF777777"),
            color_last: rgb_color("FF359CEB"),
            color_high: rgb_color("FF56BE79"),
            color_low: rgb_color("FFFF5055"),
            ..Default::default()
        }, // 32
        XlsxX14SparklineGroup {
            color_series: rgb_color("FFC6EFCE"),
            color_negative: rgb_color("FFFFC7CE"),
            color_markers: rgb_color("FF8CADD6"),
            color_first: rgb_color("FFFFDC47"),
            color_last: rgb_color("FFFFEB9C"),
            color_high: rgb_color("FF60D276"),
            color_low: rgb_color("FFFF5367"),
            ..Default::default()
        }, // 33
        XlsxX14SparklineGroup {
            color_series: rgb_color("FF00B050"),
            color_negative: rgb_color("FFFF0000"),
            color_markers: rgb_color("FF0070C0"),
            color_first: rgb_color("FFFFC000"),
            color_last: rgb_color("FFFFC000"),
            color_high: rgb_color("FF00B050"),
            color_low: rgb_color("FFFF0000"),
            ..Default::default()
        }, // 34
        XlsxX14SparklineGroup {
            color_series: theme_color(3, None),
            color_negative: theme_color(9, None),
            color_markers: theme_color(8, None),
            color_first: theme_color(4, None),
            color_last: theme_color(5, None),
            color_high: theme_color(6, None),
            color_low: theme_color(7, None),
            ..Default::default()
        }, // 35
        XlsxX14SparklineGroup {
            color_series: theme_color(1, None),
            color_negative: theme_color(9, None),
            color_markers: theme_color(8, None),
            color_first: theme_color(4, None),
            color_last: theme_color(5, None),
            color_high: theme_color(6, None),
            color_low: theme_color(7, None),
            ..Default::default()
        }, // 36
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::options::Options;
    use crate::xml::common::serialize_ext_lst;

    #[test]
    fn add_sparkline_basic() {
        let f = File::new_with_options(Options::default());
        f.add_sparkline(
            "Sheet1",
            &SparklineOptions {
                location: vec!["A2".to_string()],
                range: vec!["Sheet2!A1:J1".to_string()],
                ..Default::default()
            },
        )
        .unwrap();

        let ws = f.work_sheet_reader("Sheet1").unwrap();
        let ext_lst = ws.ext_lst.as_ref().unwrap();
        assert_eq!(ext_lst.ext.len(), 1);
        assert_eq!(
            ext_lst.ext[0].uri.as_deref(),
            Some(EXT_URI_SPARKLINE_GROUPS)
        );
        assert!(ext_lst.ext[0].content.contains("x14:sparklineGroup"));
        assert!(ext_lst.ext[0].content.contains("Sheet2!A1:J1"));
        assert!(ext_lst.ext[0].content.contains("A2"));
    }

    #[test]
    fn add_sparkline_grouped() {
        let f = File::new_with_options(Options::default());
        f.add_sparkline(
            "Sheet1",
            &SparklineOptions {
                location: vec!["A27".to_string(), "A28".to_string(), "A29".to_string()],
                range: vec![
                    "Sheet3!A5:J5".to_string(),
                    "Sheet3!A6:J6".to_string(),
                    "Sheet3!A7:J7".to_string(),
                ],
                markers: true,
                ..Default::default()
            },
        )
        .unwrap();

        let ws = f.work_sheet_reader("Sheet1").unwrap();
        let ext_lst = ws.ext_lst.as_ref().unwrap();
        assert_eq!(ext_lst.ext.len(), 1);
        let content = &ext_lst.ext[0].content;
        assert_eq!(content.matches("</x14:sparklineGroup>").count(), 1);
        assert_eq!(content.matches("<x14:sparkline>").count(), 3);
    }

    #[test]
    fn add_sparkline_invalid_type() {
        let f = File::new_with_options(Options::default());
        let err = f
            .add_sparkline(
                "Sheet1",
                &SparklineOptions {
                    location: vec!["A1".to_string()],
                    range: vec!["Sheet2!A1:J1".to_string()],
                    r#type: "unknown_type".to_string(),
                    ..Default::default()
                },
            )
            .unwrap_err();
        assert!(err.to_string().contains("'Type' value must be one of"));
    }

    #[test]
    fn add_sparkline_invalid_style() {
        let f = File::new_with_options(Options::default());
        let err = f
            .add_sparkline(
                "Sheet1",
                &SparklineOptions {
                    location: vec!["A1".to_string()],
                    range: vec!["Sheet2!A1:J1".to_string()],
                    style: -1,
                    ..Default::default()
                },
            )
            .unwrap_err();
        assert!(err.to_string().contains("'Style' value must be an integer"));
    }

    #[test]
    fn add_sparkline_mismatched_location_range() {
        let f = File::new_with_options(Options::default());
        let err = f
            .add_sparkline(
                "Sheet1",
                &SparklineOptions {
                    location: vec!["A1".to_string(), "A2".to_string()],
                    range: vec!["Sheet2!A1:J1".to_string()],
                    ..Default::default()
                },
            )
            .unwrap_err();
        assert!(err.to_string().contains("Location") && err.to_string().contains("Range"));
    }

    #[test]
    fn add_sparkline_append_mode() {
        let f = File::new_with_options(Options::default());
        f.add_sparkline(
            "Sheet1",
            &SparklineOptions {
                location: vec!["A2".to_string()],
                range: vec!["Sheet2!A1:J1".to_string()],
                ..Default::default()
            },
        )
        .unwrap();
        f.add_sparkline(
            "Sheet1",
            &SparklineOptions {
                location: vec!["A3".to_string()],
                range: vec!["Sheet2!A2:J2".to_string()],
                r#type: "column".to_string(),
                ..Default::default()
            },
        )
        .unwrap();

        let ws = f.work_sheet_reader("Sheet1").unwrap();
        let ext_lst = ws.ext_lst.as_ref().unwrap();
        assert_eq!(ext_lst.ext.len(), 1);
        let content = &ext_lst.ext[0].content;
        assert_eq!(content.matches("</x14:sparklineGroup>").count(), 2);
    }

    #[test]
    fn add_sparkline_with_existing_ext() {
        let f = File::new_with_options(Options::default());
        let path = f.get_sheet_xml_path("Sheet1").unwrap();
        let mut ws = f.work_sheet_reader("Sheet1").unwrap();
        ws.ext_lst = Some(XlsxExtLst {
            ext: vec![
                XlsxExt {
                    uri: Some("{A8765BA9-456A-4dab-B4F3-ACF838C121DE}".to_string()),
                    content: "<x14:slicerList />".to_string(),
                    ..Default::default()
                },
                XlsxExt {
                    uri: Some(EXT_URI_SPARKLINE_GROUPS.to_string()),
                    content: format!(
                        r#"<x14:sparklineGroups xmlns:xm="{}"></x14:sparklineGroups>"#,
                        NAMESPACE_SPREADSHEET_EXCEL_2006_MAIN
                    ),
                    ..Default::default()
                },
            ],
        });
        f.sheet.insert(path, ws);

        f.add_sparkline(
            "Sheet1",
            &SparklineOptions {
                location: vec!["A3".to_string()],
                range: vec!["Sheet2!A2:J2".to_string()],
                r#type: "column".to_string(),
                ..Default::default()
            },
        )
        .unwrap();

        let ws = f.work_sheet_reader("Sheet1").unwrap();
        let ext_lst = ws.ext_lst.as_ref().unwrap();
        assert_eq!(ext_lst.ext.len(), 2);
        // Sparkline groups have higher priority than slicer list, so they come first.
        assert_eq!(
            ext_lst.ext[0].uri.as_deref(),
            Some(EXT_URI_SPARKLINE_GROUPS)
        );
        assert_eq!(
            ext_lst.ext[1].uri.as_deref(),
            Some("{A8765BA9-456A-4dab-B4F3-ACF838C121DE}")
        );
        assert!(ext_lst.ext[0].content.contains("x14:sparklineGroup"));
    }

    #[test]
    fn sparkline_presets_count() {
        // Go defines 37 preset entries (labeled 0..36), but only styles 0..35
        // are accepted by the validation check.
        assert_eq!(get_sparkline_group_presets().len(), 37);
    }

    #[test]
    fn serialize_ext_lst_round_trip() {
        let ext_lst = XlsxExtLst {
            ext: vec![XlsxExt {
                uri: Some(EXT_URI_SPARKLINE_GROUPS.to_string()),
                content: format!(
                    r#"<x14:sparklineGroups xmlns:xm="{}"><x14:sparklineGroup type="line"><x14:sparklines><x14:sparkline><xm:f>Sheet2!A1:J1</xm:f><xm:sqref>A2</xm:sqref></x14:sparkline></x14:sparklines></x14:sparklineGroup></x14:sparklineGroups>"#,
                    NAMESPACE_SPREADSHEET_EXCEL_2006_MAIN
                ),
                ..Default::default()
            }],
        };
        let xml = serialize_ext_lst(&ext_lst);
        assert!(xml.contains("x14:sparklineGroup"));
        assert!(xml.contains("Sheet2!A1:J1"));
    }

    #[test]
    fn save_sparkline_writes_valid_xml() {
        let mut f = File::new_with_options(Options::default());
        f.add_sparkline(
            "Sheet1",
            &SparklineOptions {
                location: vec!["A2".to_string()],
                range: vec!["Sheet2!A1:J1".to_string()],
                markers: true,
                ..Default::default()
            },
        )
        .unwrap();

        let tmp = std::env::temp_dir().join("excelize_rust_sparkline_test.xlsx");
        f.save_as(tmp.to_str().unwrap()).unwrap();

        let file = std::fs::File::open(&tmp).unwrap();
        let mut archive = zip::ZipArchive::new(file).unwrap();
        let mut sheet = archive.by_name("xl/worksheets/sheet1.xml").unwrap();
        let mut xml = String::new();
        std::io::Read::read_to_string(&mut sheet, &mut xml).unwrap();
        drop(sheet);
        drop(archive);
        let _ = std::fs::remove_file(&tmp);

        assert!(xml.contains("x14:sparklineGroups"));
        assert!(xml.contains("x14:sparklineGroup"));
        assert!(xml.contains("Sheet2!A1:J1"));
        assert!(xml.contains("A2"));
        assert!(xml.contains("xmlns:x14"));
    }
}
