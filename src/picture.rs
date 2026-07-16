//! Picture API.
//!
//! Ported from Go `picture.go` and `drawing.go`.
//!
//! This is a functional subset supporting the most common image formats
//! (PNG, JPEG, GIF, BMP, SVG, TIFF, WMF, EMF) placed over cells.

use std::collections::HashMap;

use quick_xml::de::from_reader as xml_from_reader;
use quick_xml::se::to_string as xml_to_string;

use crate::calc::arg::FORMULA_ERROR_VALUE;
use crate::constants::{
    DEFAULT_DRAWING_SCALE, DEFAULT_XML_PATH_CELL_IMAGES, DEFAULT_XML_PATH_CELL_IMAGES_RELS, EMU,
    EXT_URI_SVG, MAX_GRAPHIC_ALT_TEXT_LENGTH, MAX_GRAPHIC_NAME_LENGTH, NAMESPACE_DRAWING_2016_SVG,
    NAMESPACE_SPREADSHEET, SOURCE_RELATIONSHIP, SOURCE_RELATIONSHIP_HYPER_LINK,
    SOURCE_RELATIONSHIP_IMAGE,
};
use crate::errors::Result;
use crate::errors::{
    ErrImgExt, ErrImgLoad, ErrMaxGraphicAltTextLength, ErrMaxGraphicNameLength, ErrParameterInvalid,
};
use crate::file::File;
use crate::lib_util::{
    cell_name_to_coordinates, coordinates_to_cell_name, count_utf16_string,
    range_ref_to_coordinates, sort_coordinates,
};
use crate::xml::decode_drawing::DecodeCellImages;
use crate::xml::drawing::{
    XdrCellAnchor, XlsxBlip, XlsxBlipFill, XlsxCNvPicPr, XlsxCNvPr, XlsxFrom, XlsxNvPicPr, XlsxOff,
    XlsxPic, XlsxPicLocks, XlsxPositiveSize2D, XlsxPrstGeom, XlsxSpPr, XlsxStretch, XlsxTo,
    XlsxXfrm,
};
use crate::xml::worksheet::XlsxC;

// ------------------------------------------------------------------
// Re-exports of public types
// ------------------------------------------------------------------

pub use crate::xml::drawing::{GraphicOptions, Picture, PictureInsertType};

// ------------------------------------------------------------------
// Image type support
// ------------------------------------------------------------------

fn supported_image_types() -> HashMap<String, String> {
    [
        (".bmp".to_string(), ".bmp".to_string()),
        (".emf".to_string(), ".emf".to_string()),
        (".emz".to_string(), ".emz".to_string()),
        (".gif".to_string(), ".gif".to_string()),
        (".ico".to_string(), ".ico".to_string()),
        (".jpeg".to_string(), ".jpeg".to_string()),
        (".jpg".to_string(), ".jpg".to_string()),
        (".png".to_string(), ".png".to_string()),
        (".svg".to_string(), ".svg".to_string()),
        (".tif".to_string(), ".tif".to_string()),
        (".tiff".to_string(), ".tiff".to_string()),
        (".wmf".to_string(), ".wmf".to_string()),
        (".wmz".to_string(), ".wmz".to_string()),
    ]
    .into_iter()
    .collect()
}

fn detect_image_extension(data: &[u8]) -> Option<String> {
    if data.starts_with(b"\x89PNG\r\n\x1a\n") {
        Some(".png".to_string())
    } else if data.starts_with(b"\xFF\xD8\xFF") {
        Some(".jpg".to_string())
    } else if data.starts_with(b"GIF87a") || data.starts_with(b"GIF89a") {
        Some(".gif".to_string())
    } else if data.starts_with(b"BM") {
        Some(".bmp".to_string())
    } else if data.starts_with(b"<?xml") || data.starts_with(b"<svg") {
        Some(".svg".to_string())
    } else if data.starts_with(b"II*\0") || data.starts_with(b"MM\0*") {
        Some(".tiff".to_string())
    } else {
        None
    }
}

fn image_dimensions(data: &[u8], ext: &str) -> Option<(i32, i32)> {
    if ext.eq_ignore_ascii_case(".svg") {
        return Some((64, 64));
    }
    if let Ok(reader) = image::ImageReader::new(std::io::Cursor::new(data)).with_guessed_format() {
        if let Ok(dim) = reader.into_dimensions() {
            return Some((dim.0 as i32, dim.1 as i32));
        }
    }
    None
}

// ------------------------------------------------------------------
// Public API
// ------------------------------------------------------------------

impl File {
    /// Add a picture to a worksheet from a file path.
    pub fn add_picture(
        &mut self,
        sheet: &str,
        cell: &str,
        path: &str,
        opts: Option<&GraphicOptions>,
    ) -> Result<()> {
        self.add_picture_from_file(sheet, cell, path, PictureInsertType::PLACE_OVER_CELLS, opts)
    }

    /// Add a picture to a worksheet from a file path with an explicit insert
    /// type, for example [PictureInsertType::PLACE_IN_CELL] (Excel 365) or
    /// [PictureInsertType::DISPIMG] (WPS) to embed the picture in the cell.
    pub fn add_picture_from_file(
        &mut self,
        sheet: &str,
        cell: &str,
        path: &str,
        insert_type: PictureInsertType,
        opts: Option<&GraphicOptions>,
    ) -> Result<()> {
        let ext = std::path::Path::new(path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        let ext = format!(".{ext}");
        if !supported_image_types().contains_key(&ext) {
            return Err(Box::new(ErrImgExt));
        }
        let file = std::fs::read(path)?;
        let pic = Picture {
            extension: ext,
            file,
            format: opts.cloned(),
            insert_type,
            ..Default::default()
        };
        self.add_picture_from_bytes(sheet, cell, &pic)
    }

    /// Add a picture to a worksheet from raw bytes.
    pub fn add_picture_from_bytes(&mut self, sheet: &str, cell: &str, pic: &Picture) -> Result<()> {
        if pic.insert_type == PictureInsertType::PLACE_IN_CELL {
            return self.add_in_cell_picture(sheet, cell, pic);
        }
        if pic.insert_type == PictureInsertType::DISPIMG {
            return self.add_dispimg_picture(sheet, cell, pic);
        }
        if pic.insert_type != PictureInsertType::PLACE_OVER_CELLS {
            return Err(Box::new(ErrParameterInvalid));
        }
        let mut ext = pic.extension.clone();
        if ext.is_empty() {
            if let Some(detected) = detect_image_extension(&pic.file) {
                ext = detected;
            }
        }
        let types = supported_image_types();
        let mapped_ext = types.get(&ext.to_lowercase()).cloned().unwrap_or(ext);
        if !types.contains_key(&mapped_ext.to_lowercase()) {
            return Err(Box::new(ErrImgExt));
        }
        let options = parse_picture_options(pic)?;
        let dims = image_dimensions(&pic.file, &mapped_ext).ok_or_else(|| Box::new(ErrImgLoad))?;
        let _ = self.work_sheet_reader(sheet)?;

        let drawing_id = self.count_drawings() + 1;
        let drawing_xml = format!("xl/drawings/drawing{drawing_id}.xml");
        let (drawing_id, drawing_xml) = self.prepare_drawing(sheet, drawing_id, &drawing_xml)?;
        let drawing_rels = format!("xl/drawings/_rels/drawing{drawing_id}.xml.rels");

        let media = self.add_media(&pic.file, &mapped_ext);
        let media_target = format!("..{}", media.trim_start_matches("xl"));
        let mut drawing_rid = 0;
        if let Ok(Some(rels)) = self.rels_reader(&drawing_rels) {
            for rel in &rels.relationships {
                if rel.r#type == SOURCE_RELATIONSHIP_IMAGE && rel.target == media_target {
                    drawing_rid = rel.id.trim_start_matches("rId").parse().unwrap_or(0);
                    break;
                }
            }
        }
        if drawing_rid == 0 {
            drawing_rid =
                self.add_rels(&drawing_rels, SOURCE_RELATIONSHIP_IMAGE, &media_target, "");
        }

        let mut hyperlink_rid = 0;
        let mut hyperlink_type = String::new();
        if !options.hyperlink.is_empty() && !options.hyperlink_type.is_empty() {
            if options.hyperlink_type == "External" {
                hyperlink_type = "External".to_string();
            }
            hyperlink_rid = self.add_rels(
                &drawing_rels,
                SOURCE_RELATIONSHIP_HYPER_LINK,
                &options.hyperlink,
                &hyperlink_type,
            );
        }

        self.add_drawing_picture(
            sheet,
            &drawing_xml,
            cell,
            &mapped_ext,
            drawing_rid,
            hyperlink_rid,
            dims,
            &options,
        )?;
        self.add_content_type_part(drawing_id, "drawings")?;
        self.add_sheet_name_space(sheet, NAMESPACE_SPREADSHEET);
        Ok(())
    }

    /// Embed a picture into a cell using the Microsoft rich data mechanism
    /// (Excel 365 "Place in Cell").
    fn add_in_cell_picture(&mut self, sheet: &str, cell: &str, pic: &Picture) -> Result<()> {
        use crate::constants::*;
        let mapped_ext = resolve_picture_extension(pic)?;
        let _ = self.work_sheet_reader(sheet)?;
        let media = self.add_media(&pic.file, &mapped_ext);
        let media_target = format!("..{}", media.trim_start_matches("xl"));

        // Reference the media part from `xl/richData/_rels/richValueRel.xml.rels`.
        let mut embed_rid = 0;
        if let Ok(Some(rels)) = self.rels_reader(DEFAULT_XML_PATH_RD_RICH_VALUE_REL_RELS) {
            for rel in &rels.relationships {
                if rel.r#type == SOURCE_RELATIONSHIP_IMAGE && rel.target == media_target {
                    embed_rid = rel.id.trim_start_matches("rId").parse().unwrap_or(0);
                    break;
                }
            }
        }
        if embed_rid == 0 {
            embed_rid = self.add_rels(
                DEFAULT_XML_PATH_RD_RICH_VALUE_REL_RELS,
                SOURCE_RELATIONSHIP_IMAGE,
                &media_target,
                "",
            );
        }

        // Append the relationship ID to `xl/richData/richValueRel.xml`.
        let mut rvr = self.rich_value_rel_reader()?;
        rvr.rels.push(crate::xml::metadata::XlsxRichValueRelRelationship {
            id: format!("rId{embed_rid}"),
        });
        let rel_idx = rvr.rels.len() - 1;
        let mut out = xml_to_string(&rvr).unwrap_or_default().into_bytes();
        crate::file::replace_root_namespace_attributes(
            &mut out,
            &format!("xmlns=\"{NAMESPACE_RICH_DATA_2}\""),
        )?;
        self.save_file_list(DEFAULT_XML_PATH_RD_RICH_VALUE_REL, &out);

        // Ensure the `_localImage` structure exists in `rdRichValueStructure.xml`.
        let mut structs = self.rich_value_structures_reader()?;
        let struct_idx = match structs.s.iter().position(|s| s.t == "_localImage") {
            Some(idx) => idx,
            None => {
                structs
                    .s
                    .push(crate::xml::metadata::XlsxRichValueStructure {
                        t: "_localImage".to_string(),
                        k: vec![
                            crate::xml::metadata::XlsxRichValueKey {
                                n: "_rvRel:LocalImageIdentifier".to_string(),
                                t: Some("i".to_string()),
                            },
                            crate::xml::metadata::XlsxRichValueKey {
                                n: "CalcOrigin".to_string(),
                                t: Some("i".to_string()),
                            },
                        ],
                    });
                structs.s.len() - 1
            }
        };
        structs.count = Some(structs.s.len() as i64);
        let mut out = xml_to_string(&structs).unwrap_or_default().into_bytes();
        crate::file::replace_root_namespace_attributes(
            &mut out,
            &format!(
                "xmlns=\"{NAMESPACE_RICH_DATA}\" count=\"{}\"",
                structs.s.len()
            ),
        )?;
        self.save_file_list(DEFAULT_XML_PATH_RD_RICH_VALUE_STRUCTURE, &out);

        // Append the rich value to `rdrichvalue.xml`.
        let mut rvdata = self.rich_value_reader()?;
        rvdata.rv.push(crate::xml::metadata::XlsxRichValue {
            s: struct_idx as i64,
            v: vec![rel_idx.to_string(), "5".to_string()],
            fb: None,
        });
        rvdata.count = Some(rvdata.rv.len() as i64);
        let rv_idx = rvdata.rv.len() - 1;
        let mut out = xml_to_string(&rvdata).unwrap_or_default().into_bytes();
        crate::file::replace_root_namespace_attributes(
            &mut out,
            &format!(
                "xmlns=\"{NAMESPACE_RICH_DATA}\" count=\"{}\"",
                rvdata.rv.len()
            ),
        )?;
        self.save_file_list(DEFAULT_XML_PATH_RD_RICH_VALUE, &out);

        // Register the rich value in `xl/metadata.xml` and tag the cell.
        let vm = self.upsert_in_cell_metadata(rv_idx);
        let path = self
            .get_sheet_xml_path(sheet)
            .ok_or_else(|| crate::errors::ErrSheetNotExist {
                sheet_name: sheet.to_string(),
            })?;
        let mut ws = self.work_sheet_reader(sheet)?;
        {
            let c = crate::cell::get_or_make_cell(&mut ws, cell);
            c.t = Some("e".to_string());
            c.v = Some(FORMULA_ERROR_VALUE.to_string());
            c.vm = Some(vm);
        }
        crate::cell::update_dimension(&mut ws)?;
        self.sheet.insert(path, ws);

        // Register package-level content types and workbook relationships.
        crate::sheet::set_content_type_part_image_extensions(self)?;
        self.ensure_content_type_override("/xl/metadata.xml", CONTENT_TYPE_SHEET_METADATA)?;
        self.ensure_content_type_override(
            "/xl/richData/rdrichvalue.xml",
            CONTENT_TYPE_RD_RICH_VALUE,
        )?;
        self.ensure_content_type_override(
            "/xl/richData/rdRichValueStructure.xml",
            CONTENT_TYPE_RD_RICH_VALUE_STRUCTURE,
        )?;
        self.ensure_content_type_override(
            "/xl/richData/richValueRel.xml",
            CONTENT_TYPE_RICH_VALUE_REL,
        )?;
        self.ensure_workbook_rel(SOURCE_RELATIONSHIP_SHEET_METADATA, "metadata.xml");
        self.ensure_workbook_rel(SOURCE_RELATIONSHIP_RD_RICH_VALUE, "richData/rdrichvalue.xml");
        self.ensure_workbook_rel(
            SOURCE_RELATIONSHIP_RD_RICH_VALUE_STRUCTURE,
            "richData/rdRichValueStructure.xml",
        );
        self.ensure_workbook_rel(SOURCE_RELATIONSHIP_RICH_VALUE_REL, "richData/richValueRel.xml");
        Ok(())
    }

    /// Embed a picture into a cell using the Kingsoft WPS Office `DISPIMG`
    /// mechanism.
    fn add_dispimg_picture(&mut self, sheet: &str, cell: &str, pic: &Picture) -> Result<()> {
        use crate::constants::*;
        let mapped_ext = resolve_picture_extension(pic)?;
        let dims =
            image_dimensions(&pic.file, &mapped_ext).ok_or_else(|| Box::new(ErrImgLoad))?;
        let _ = self.work_sheet_reader(sheet)?;
        let media = self.add_media(&pic.file, &mapped_ext);
        let media_target = media.trim_start_matches("xl/").to_string();

        // Reference the media part from `xl/_rels/cellimages.xml.rels`.
        let mut embed_rid = 0;
        if let Ok(Some(rels)) = self.rels_reader(DEFAULT_XML_PATH_CELL_IMAGES_RELS) {
            for rel in &rels.relationships {
                if rel.r#type == SOURCE_RELATIONSHIP_IMAGE && rel.target == media_target {
                    embed_rid = rel.id.trim_start_matches("rId").parse().unwrap_or(0);
                    break;
                }
            }
        }
        if embed_rid == 0 {
            embed_rid = self.add_rels(
                DEFAULT_XML_PATH_CELL_IMAGES_RELS,
                SOURCE_RELATIONSHIP_IMAGE,
                &media_target,
                "",
            );
        }

        // Append the picture entry to `xl/cellimages.xml`.
        let img_id = format!("ID_{:032X}", rand::random::<u128>());
        let alt_text = pic
            .format
            .as_ref()
            .map(|f| f.alt_text.clone())
            .unwrap_or_default();
        let mut cell_images = self.cell_images_reader()?;
        let cnv_id = 1000 + cell_images.cell_image.len() as i32 + 1;
        cell_images
            .cell_image
            .push(crate::xml::decode_drawing::DecodeCellImage {
                pic: crate::xml::decode_drawing::DecodePic {
                    nv_pic_pr: crate::xml::decode_drawing::DecodeNvPicPr {
                        c_nv_pr: crate::xml::decode_drawing::DecodeCNvPr {
                            id: cnv_id,
                            name: img_id.clone(),
                            descr: alt_text,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    blip_fill: crate::xml::decode_drawing::DecodeBlipFill {
                        blip: crate::xml::decode_drawing::DecodeBlip {
                            embed: format!("rId{embed_rid}"),
                            cstate: Some("print".to_string()),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    sp_pr: crate::xml::decode_drawing::DecodeSpPr {
                        xfrm: crate::xml::decode_drawing::DecodeXfrm {
                            ext: crate::xml::decode_drawing::DecodePositiveSize2D {
                                cx: dims.0 as i64 * EMU as i64,
                                cy: dims.1 as i64 * EMU as i64,
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                },
            });
        self.save_file_list(
            DEFAULT_XML_PATH_CELL_IMAGES,
            build_cell_images_xml(&cell_images).as_bytes(),
        );

        // Write the DISPIMG formula into the target cell.
        let path = self
            .get_sheet_xml_path(sheet)
            .ok_or_else(|| crate::errors::ErrSheetNotExist {
                sheet_name: sheet.to_string(),
            })?;
        let mut ws = self.work_sheet_reader(sheet)?;
        {
            let c = crate::cell::get_or_make_cell(&mut ws, cell);
            c.t = Some("str".to_string());
            c.f = Some(crate::xml::worksheet::XlsxF {
                content: format!("_xlfn.DISPIMG(\"{img_id}\",1)"),
                ..Default::default()
            });
            c.v = Some(img_id);
        }
        crate::cell::update_dimension(&mut ws)?;
        self.sheet.insert(path, ws);

        // Register package-level content types and the workbook relationship.
        crate::sheet::set_content_type_part_image_extensions(self)?;
        self.ensure_content_type_override("/xl/cellimages.xml", CONTENT_TYPE_WPS_CELL_IMAGES)?;
        self.ensure_workbook_rel(SOURCE_RELATIONSHIP_CELL_IMAGES, "cellimages.xml");
        Ok(())
    }

    /// Ensure `[Content_Types].xml` contains an override for the given part.
    fn ensure_content_type_override(&self, part_name: &str, content_type: &str) -> Result<()> {
        let mut ct = self.content_types_reader()?;
        let exists = ct.entries.iter().any(|e| {
            matches!(e, crate::xml::content_types::XlsxContentTypeEntry::Override(o) if o.part_name == part_name)
        });
        if !exists {
            ct.entries
                .push(crate::xml::content_types::XlsxContentTypeEntry::Override(
                    crate::xml::content_types::XlsxOverride {
                        part_name: part_name.to_string(),
                        content_type: content_type.to_string(),
                    },
                ));
            *self.content_types.lock().unwrap() = Some(ct);
        }
        Ok(())
    }

    /// Ensure the workbook relationships part contains a relationship of the
    /// given type.
    fn ensure_workbook_rel(&self, rel_type: &str, target: &str) {
        let path = self.get_workbook_rels_path();
        let mut rels = self.rels_reader(&path).unwrap_or_default().unwrap_or_default();
        if rels.relationships.iter().any(|r| r.r#type == rel_type) {
            return;
        }
        let max_rid = rels
            .relationships
            .iter()
            .filter_map(|r| r.id.trim_start_matches("rId").parse::<i32>().ok())
            .max()
            .unwrap_or(0);
        rels.relationships
            .push(crate::xml::workbook::XlsxRelationship {
                id: format!("rId{}", max_rid + 1),
                r#type: rel_type.to_string(),
                target: target.to_string(),
                target_mode: None,
            });
        self.relationships.insert(path, rels);
    }

    /// Add or update `xl/metadata.xml` for a new in-cell rich value, returning
    /// the 1-based value metadata index (`vm`) for the cell.
    fn upsert_in_cell_metadata(&self, rv_idx: usize) -> u64 {
        use crate::constants::*;
        let raw = self.read_xml(DEFAULT_XML_PATH_METADATA);
        let text = String::from_utf8_lossy(&raw).into_owned();
        let body = match text.find("?>") {
            Some(pos) => &text[pos + 2..],
            None => text.as_str(),
        };
        let rvb = format!(
            "<bk><extLst><ext uri=\"{{3e2802c4-a4d2-4d8b-9148-e3be6c30e623}}\"><xlrd:rvb i=\"{rv_idx}\"/></ext></extLst></bk>"
        );
        if !body.contains("<metadata") {
            let content = format!(
                "<metadata xmlns=\"{NAMESPACE_SPREADSHEET}\" xmlns:xlrd=\"{NAMESPACE_RICH_DATA_2}\">\
<metadataTypes count=\"1\"><metadataType name=\"XLRICHVALUE\" minSupportedVersion=\"120000\" copy=\"1\" pasteAll=\"1\" pasteValues=\"1\" merge=\"1\" splitFirst=\"1\" rowColShift=\"1\" clearAll=\"1\" clearFormats=\"1\" clearContents=\"1\" clearComments=\"1\" assign=\"1\" coerce=\"1\"/></metadataTypes>\
<futureMetadata name=\"XLRICHVALUE\" count=\"1\">{rvb}</futureMetadata>\
<valueMetadata count=\"1\"><bk><rc t=\"1\" v=\"{rv_idx}\"/></bk></valueMetadata>\
</metadata>"
            );
            self.save_file_list(DEFAULT_XML_PATH_METADATA, content.as_bytes());
            return 1;
        }
        let mut s = body.to_string();
        // Append a value metadata block after the existing ones.
        let vm = if let Some(open) = s.find("<valueMetadata") {
            let gt = open + s[open..].find('>').unwrap_or(0);
            let tag = s[open..=gt].to_string();
            let count = parse_count_attr(&tag).unwrap_or(0);
            let new_tag = set_count_attr(&tag, count + 1);
            s.replace_range(open..=gt, &new_tag);
            if let Some(close) = s.find("</valueMetadata>") {
                s.insert_str(close, &format!("<bk><rc t=\"1\" v=\"{rv_idx}\"/></bk>"));
            }
            count + 1
        } else {
            if let Some(close) = s.find("</metadata>") {
                s.insert_str(
                    close,
                    &format!("<valueMetadata count=\"1\"><bk><rc t=\"1\" v=\"{rv_idx}\"/></bk></valueMetadata>"),
                );
            }
            1
        };
        // Keep the XLRICHVALUE future metadata in sync.
        if let Some(open) = s.find("<futureMetadata name=\"XLRICHVALUE\"") {
            let gt = open + s[open..].find('>').unwrap_or(0);
            let tag = s[open..=gt].to_string();
            let count = parse_count_attr(&tag).unwrap_or(0);
            let new_tag = set_count_attr(&tag, count + 1);
            s.replace_range(open..=gt, &new_tag);
            if let Some(close) = s[open..].find("</futureMetadata>") {
                s.insert_str(open + close, &rvb);
            }
        } else {
            let fm = format!("<futureMetadata name=\"XLRICHVALUE\" count=\"1\">{rvb}</futureMetadata>");
            let pos = s
                .find("<cellMetadata")
                .or_else(|| s.find("<valueMetadata"))
                .or_else(|| s.find("</metadata>"));
            if let Some(pos) = pos {
                s.insert_str(pos, &fm);
            }
        }
        self.save_file_list(DEFAULT_XML_PATH_METADATA, s.as_bytes());
        vm as u64
    }

    /// Return all pictures anchored at a given cell in a worksheet.
    pub fn get_pictures(&self, sheet: &str, cell: &str) -> Result<Vec<Picture>> {
        let mut pics = self.get_cell_images(sheet, cell)?;
        let ws = self.work_sheet_reader(sheet)?;
        if ws.drawing.is_none() {
            return Ok(pics);
        }
        let (col, row) = cell_name_to_coordinates(cell)?;
        let col = col - 1;
        let row = row - 1;
        let drawing_xml = self
            .get_sheet_relationships_target_by_id(
                sheet,
                ws.drawing.as_ref().unwrap().rid.as_deref().unwrap_or(""),
            )
            .replace("..", "xl")
            .trim_start_matches('/')
            .to_string();
        let drawing_rels = drawing_xml
            .replace("xl/drawings", "xl/drawings/_rels")
            .replace(".xml", ".xml.rels");
        for pic in self.get_picture(row, col, &drawing_xml, &drawing_rels)? {
            if !pics.iter().any(|p| p.file == pic.file) {
                pics.push(pic);
            }
        }
        Ok(pics)
    }

    /// Return all picture cell references in a worksheet.
    ///
    /// Equivalent to Go `GetPictureCells`.
    pub fn get_picture_cells(&self, sheet: &str) -> Result<Vec<String>> {
        let mut cells = self.get_image_cells(sheet)?;
        let ws = self.work_sheet_reader(sheet)?;
        if ws.drawing.is_none() {
            return Ok(cells);
        }
        let drawing_xml = self
            .get_sheet_relationships_target_by_id(
                sheet,
                ws.drawing.as_ref().unwrap().rid.as_deref().unwrap_or(""),
            )
            .replace("..", "xl")
            .trim_start_matches('/')
            .to_string();
        let drawing_rels = drawing_xml
            .replace("xl/drawings", "xl/drawings/_rels")
            .replace(".xml", ".xml.rels");
        let (wsdr, _) = self.drawing_parser(&drawing_xml)?;
        for anchor in wsdr
            .one_cell_anchor
            .iter()
            .chain(wsdr.two_cell_anchor.iter())
        {
            let Some(pic) = &anchor.pic else {
                continue;
            };
            let r_id = &pic.blip_fill.blip.embed;
            if self.get_drawing_relationship(&drawing_rels, r_id).is_none() {
                continue;
            }
            if let Some(from) = &anchor.from {
                let cell =
                    coordinates_to_cell_name(from.col as i32 + 1, from.row as i32 + 1, false)?;
                if !cells.contains(&cell) {
                    cells.push(cell);
                }
            }
        }
        Ok(cells)
    }

    /// Read the Kingsoft WPS Office embedded cell images part.
    fn cell_images_reader(&self) -> Result<DecodeCellImages> {
        let data = self.apply_charset_transcoder(&self.read_xml(DEFAULT_XML_PATH_CELL_IMAGES))?;
        let data = crate::file::namespace_strict_to_transitional(&data);
        if data.is_empty() {
            return Ok(DecodeCellImages::default());
        }
        Ok(xml_from_reader(data.as_slice()).unwrap_or_default())
    }

    /// Return all cell images and Kingsoft WPS Office embedded image cells in a
    /// worksheet.
    fn get_image_cells(&self, sheet: &str) -> Result<Vec<String>> {
        let ws = self.work_sheet_reader(sheet)?;
        let mut cells = Vec::new();
        for row in &ws.sheet_data.row {
            for c in &row.c {
                let Some(cell_ref) = c.r.as_deref() else {
                    continue;
                };
                if let Some(f) = &c.f {
                    if !f.content.is_empty() && is_dispimg_formula(&f.content) {
                        self.calc_cell_value(sheet, cell_ref)?;
                        cells.push(cell_ref.to_string());
                    }
                }
                let mut pic = Picture {
                    format: Some(GraphicOptions::default()),
                    ..Default::default()
                };
                if self.get_image_cell_rel(c, &mut pic)?.is_some() {
                    cells.push(cell_ref.to_string());
                }
            }
        }
        Ok(cells)
    }

    /// Return the relationship of a cell image by a rich value rel index.
    fn get_rich_data_rich_value_rel(
        &self,
        val: &str,
    ) -> Result<Option<crate::xml::workbook::XlsxRelationship>> {
        let idx = val.parse::<usize>()?;
        let rich_value_rel = self.rich_value_rel_reader()?;
        let r_id = match rich_value_rel.rels.get(idx) {
            Some(rel) => rel.id.clone(),
            None => return Ok(None),
        };
        let rel = self.get_rich_data_rich_value_rel_relationship(&r_id);
        if rel
            .as_ref()
            .map_or(false, |r| r.r#type != SOURCE_RELATIONSHIP_IMAGE)
        {
            return Ok(None);
        }
        Ok(rel)
    }

    /// Return the relationship of a web image by a web image rich value index.
    fn get_rich_data_web_images_rel(
        &self,
        val: &str,
    ) -> Result<Option<crate::xml::workbook::XlsxRelationship>> {
        let idx = val.parse::<usize>()?;
        let web_images = self.rich_value_web_image_reader()?;
        let r_id = match web_images.web_image_srd.get(idx) {
            Some(img) => img.blip.r_id.clone().unwrap_or_default(),
            None => return Ok(None),
        };
        let rel = self.get_rich_value_web_image_relationship(&r_id);
        if rel
            .as_ref()
            .map_or(false, |r| r.r#type != SOURCE_RELATIONSHIP_IMAGE)
        {
            return Ok(None);
        }
        Ok(rel)
    }

    /// Return the cell image relationship for a worksheet cell.
    fn get_image_cell_rel(
        &self,
        c: &XlsxC,
        pic: &mut Picture,
    ) -> Result<Option<crate::xml::workbook::XlsxRelationship>> {
        let vm = match c.vm {
            Some(vm) => vm,
            None => return Ok(None),
        };
        if c.v.as_deref() != Some(FORMULA_ERROR_VALUE) {
            return Ok(None);
        }
        let metadata = self.metadata_reader()?;
        let vmd = match metadata.value_metadata {
            Some(vmd) => vmd,
            None => return Ok(None),
        };
        let rc = match vmd
            .bk
            .get((vm as usize).saturating_sub(1))
            .and_then(|b| b.rc.first())
        {
            Some(rc) => rc,
            None => return Ok(None),
        };
        let rich_value_idx = rc.v as usize;
        let rich_value = self.rich_value_reader()?;
        let rv = match rich_value.rv.get(rich_value_idx) {
            Some(rv) => rv,
            None => return Ok(None),
        };
        let rv_structures = self.rich_value_structures_reader()?;
        let rv_struct = match rv_structures.s.get(rv.s as usize) {
            Some(s) => s,
            None => return Ok(None),
        };
        if rv_struct.k.len() != rv.v.len() {
            return Ok(None);
        }
        if let Some(idx) = rich_value_key_idx(&rv_struct.k, "Text") {
            if let Some(fmt) = &mut pic.format {
                fmt.alt_text = rv.v[idx].clone();
            }
        }
        if let Some(idx) = rich_value_key_idx(&rv_struct.k, "_rvRel:LocalImageIdentifier") {
            pic.insert_type = PictureInsertType::PLACE_IN_CELL;
            return self.get_rich_data_rich_value_rel(&rv.v[idx]);
        }
        if let Some(idx) = rich_value_key_idx(&rv_struct.k, "WebImageIdentifier") {
            pic.insert_type = PictureInsertType::IMAGE;
            return self.get_rich_data_web_images_rel(&rv.v[idx]);
        }
        Ok(None)
    }

    /// Return cell images and Kingsoft WPS Office embedded cell images for a
    /// given worksheet and cell reference.
    fn get_cell_images(&self, sheet: &str, cell: &str) -> Result<Vec<Picture>> {
        let mut pics = self.get_disp_images(sheet, cell)?;
        let ws = self.work_sheet_reader(sheet)?;
        let c = match crate::cell::find_cell(&ws, cell) {
            Some(c) => c.clone(),
            None => return Ok(pics),
        };
        let mut pic = Picture {
            format: Some(GraphicOptions::default()),
            insert_type: PictureInsertType::PLACE_IN_CELL,
            ..Default::default()
        };
        if let Some(rel) = self.get_image_cell_rel(&c, &mut pic)? {
            pic.extension = std::path::Path::new(&rel.target)
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| format!(".{e}"))
                .unwrap_or_default();
            let target = rel
                .target
                .replace("..", "xl")
                .trim_start_matches('/')
                .to_string();
            if let Some(bytes) = self.pkg.get(&target) {
                pic.file = bytes.value().clone();
                pics.push(pic);
            }
        }
        Ok(pics)
    }

    /// Return Kingsoft WPS Office embedded cell images for a given worksheet and
    /// cell reference.
    fn get_disp_images(&self, sheet: &str, cell: &str) -> Result<Vec<Picture>> {
        let formula = self.get_cell_formula(sheet, cell)?;
        if !is_dispimg_formula(&formula) {
            return Ok(Vec::new());
        }
        let img_id = self.calc_cell_value(sheet, cell)?;
        let cell_images = self.cell_images_reader()?;
        let rels = match self.rels_reader(DEFAULT_XML_PATH_CELL_IMAGES_RELS)? {
            Some(rels) => rels,
            None => return Ok(Vec::new()),
        };
        let mut pics = Vec::new();
        for cell_img in &cell_images.cell_image {
            if cell_img.pic.nv_pic_pr.c_nv_pr.name != img_id {
                continue;
            }
            for rel in &rels.relationships {
                if rel.id == cell_img.pic.blip_fill.blip.embed {
                    let mut pic = Picture {
                        extension: std::path::Path::new(&rel.target)
                            .extension()
                            .and_then(|e| e.to_str())
                            .map(|e| format!(".{e}"))
                            .unwrap_or_default(),
                        format: Some(GraphicOptions::default()),
                        insert_type: PictureInsertType::DISPIMG,
                        ..Default::default()
                    };
                    let target = format!("xl/{}", rel.target);
                    if let Some(bytes) = self.pkg.get(&target) {
                        pic.file = bytes.value().clone();
                        if let Some(fmt) = &mut pic.format {
                            fmt.alt_text = cell_img.pic.nv_pic_pr.c_nv_pr.descr.clone();
                            fmt.name = cell_img.pic.nv_pic_pr.c_nv_pr.name.clone();
                        }
                        pics.push(pic);
                    }
                }
            }
        }
        Ok(pics)
    }
}

fn is_dispimg_formula(content: &str) -> bool {
    let content = content.strip_prefix('=').unwrap_or(content);
    let content = content.strip_prefix("_xlfn.").unwrap_or(content);
    content.starts_with("DISPIMG")
}

/// Resolve and validate the image file extension of a picture.
fn resolve_picture_extension(pic: &Picture) -> Result<String> {
    let mut ext = pic.extension.clone();
    if ext.is_empty() {
        if let Some(detected) = detect_image_extension(&pic.file) {
            ext = detected;
        }
    }
    let types = supported_image_types();
    let mapped_ext = types.get(&ext.to_lowercase()).cloned().unwrap_or(ext);
    if !types.contains_key(&mapped_ext.to_lowercase()) {
        return Err(Box::new(ErrImgExt));
    }
    Ok(mapped_ext)
}

/// Serialize the WPS `xl/cellimages.xml` part. The decode structs intentionally
/// ignore namespace prefixes, so the document is built by hand to keep the
/// conventional `etc:`, `xdr:`, `a:` and `r:` prefixes used by WPS Office.
fn build_cell_images_xml(images: &DecodeCellImages) -> String {
    use crate::constants::*;
    let mut out = format!(
        "<etc:cellImages xmlns:etc=\"{NAMESPACE_WPS_ET_CUSTOM_DATA}\" xmlns:xdr=\"http://schemas.openxmlformats.org/drawingml/2006/spreadsheetDrawing\" xmlns:a=\"{NAMESPACE_DRAWING_ML_MAIN}\" xmlns:r=\"{SOURCE_RELATIONSHIP}\">"
    );
    for img in &images.cell_image {
        let pic = &img.pic;
        let cnv = &pic.nv_pic_pr.c_nv_pr;
        let blip = &pic.blip_fill.blip;
        let cstate = blip.cstate.as_deref().unwrap_or("");
        out.push_str(&format!(
            "<etc:cellImage><xdr:pic><xdr:nvPicPr><xdr:cNvPr id=\"{}\" name=\"{}\" descr=\"{}\"/><xdr:cNvPicPr/></xdr:nvPicPr><xdr:blipFill><a:blip r:embed=\"{}\" cstate=\"{}\"/><a:stretch><a:fillRect/></a:stretch></xdr:blipFill><xdr:spPr><a:xfrm><a:off x=\"0\" y=\"0\"/><a:ext cx=\"{}\" cy=\"{}\"/></a:xfrm><a:prstGeom prst=\"rect\"><a:avLst/></a:prstGeom></xdr:spPr></xdr:pic></etc:cellImage>",
            cnv.id,
            escape_xml_attr(&cnv.name),
            escape_xml_attr(&cnv.descr),
            blip.embed,
            cstate,
            pic.sp_pr.xfrm.ext.cx,
            pic.sp_pr.xfrm.ext.cy,
        ));
    }
    out.push_str("</etc:cellImages>");
    out
}

/// Escape a string for use in an XML attribute value.
fn escape_xml_attr(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Return the value of the `count` attribute in an XML start tag.
fn parse_count_attr(tag: &str) -> Option<usize> {
    let re = regex::Regex::new(r#"count="(\d+)""#).unwrap();
    re.captures(tag)
        .and_then(|c| c.get(1))
        .and_then(|m| m.as_str().parse().ok())
}

/// Replace the value of the `count` attribute in an XML start tag.
fn set_count_attr(tag: &str, count: usize) -> String {
    let re = regex::Regex::new(r#"count="\d+""#).unwrap();
    re.replace(tag, format!("count=\"{count}\"")).into_owned()
}

fn cell_in_merge_range(cell: &str, range_ref: &str) -> Result<bool> {
    let (col, row) = cell_name_to_coordinates(cell)?;
    if !range_ref.contains(':') {
        return Ok(false);
    }
    let mut coords = range_ref_to_coordinates(range_ref)?;
    sort_coordinates(&mut coords)?;
    Ok(col >= coords[0] && col <= coords[2] && row >= coords[1] && row <= coords[3])
}

fn rich_value_key_idx(
    keys: &[crate::xml::metadata::XlsxRichValueKey],
    name: &str,
) -> Option<usize> {
    keys.iter().position(|k| k.n == name)
}

impl File {
    /// Delete all pictures in a cell.
    pub fn delete_picture(&mut self, sheet: &str, cell: &str) -> Result<()> {
        let (col, row) = cell_name_to_coordinates(cell)?;
        let col = col - 1;
        let row = row - 1;
        let ws = self.work_sheet_reader(sheet)?;
        if ws.drawing.is_none() {
            return Ok(());
        }
        let drawing_xml = self
            .get_sheet_relationships_target_by_id(
                sheet,
                ws.drawing.as_ref().unwrap().rid.as_deref().unwrap_or(""),
            )
            .replace("..", "xl")
            .trim_start_matches('/')
            .to_string();
        let drawing_rels = format!(
            "xl/drawings/_rels/{}.rels",
            std::path::Path::new(&drawing_xml)
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
        );
        let r_ids = self.delete_drawing(col, row, &drawing_xml, "Pic")?;
        for r_id in r_ids {
            if let Some(rel) = self.get_drawing_relationship(&drawing_rels, &r_id) {
                let target = rel
                    .target
                    .replace("../", "xl/")
                    .trim_start_matches('/')
                    .to_string();
                let mut used = false;
                for entry in self.pkg.iter() {
                    if entry.key().contains("xl/drawings/_rels/drawing")
                        && *entry.key() != drawing_rels
                    {
                        if let Ok(Some(rels)) = self.rels_reader(entry.key()) {
                            for r in &rels.relationships {
                                if r.r#type == SOURCE_RELATIONSHIP_IMAGE
                                    && std::path::Path::new(&r.target)
                                        .file_name()
                                        .unwrap_or_default()
                                        == std::path::Path::new(&rel.target)
                                            .file_name()
                                            .unwrap_or_default()
                                {
                                    used = true;
                                }
                            }
                        }
                    }
                }
                if !used {
                    self.pkg.remove(&target);
                }
            }
            self.delete_drawing_rels(&drawing_rels, &r_id);
        }
        Ok(())
    }
}

// ------------------------------------------------------------------
// Option parsing
// ------------------------------------------------------------------

fn parse_picture_options(pic: &Picture) -> Result<GraphicOptions> {
    let mut opts = pic.format.clone().unwrap_or_default();
    if opts.print_object.is_none() {
        opts.print_object = Some(true);
    }
    if opts.locked.is_none() {
        opts.locked = Some(true);
    }
    if opts.scale_x == 0.0 {
        opts.scale_x = DEFAULT_DRAWING_SCALE;
    }
    if opts.scale_y == 0.0 {
        opts.scale_y = DEFAULT_DRAWING_SCALE;
    }
    if !opts.positioning.is_empty()
        && !["oneCell", "twoCell", "absolute"].contains(&opts.positioning.as_str())
    {
        return Err(crate::errors::new_invalid_optional_value(
            "Positioning",
            &opts.positioning,
            &["oneCell", "twoCell", "absolute"],
        )
        .into());
    }
    if count_utf16_string(&opts.alt_text) > MAX_GRAPHIC_ALT_TEXT_LENGTH {
        return Err(Box::new(ErrMaxGraphicAltTextLength));
    }
    if count_utf16_string(&opts.name) > MAX_GRAPHIC_NAME_LENGTH {
        return Err(Box::new(ErrMaxGraphicNameLength));
    }
    Ok(opts)
}

// ------------------------------------------------------------------
// Media management
// ------------------------------------------------------------------

impl File {
    fn count_media(&self) -> i32 {
        let mut count = 0;
        for entry in self.pkg.iter() {
            if entry.key().contains("xl/media/image") {
                count += 1;
            }
        }
        count
    }

    pub(crate) fn add_media(&self, file: &[u8], ext: &str) -> String {
        for entry in self.pkg.iter() {
            if !entry.key().starts_with("xl/media/image") {
                continue;
            }
            if entry.value() == file {
                return entry.key().clone();
            }
        }
        let count = self.count_media();
        let name = format!("xl/media/image{}{}", count + 1, ext);
        self.pkg.insert(name.clone(), file.to_vec());
        name
    }
}

// ------------------------------------------------------------------
// Drawing helpers
// ------------------------------------------------------------------

impl File {
    fn drawing_resize(
        &self,
        sheet: &str,
        cell: &str,
        width: f64,
        height: f64,
        opts: &GraphicOptions,
    ) -> Result<(i32, i32, i32, i32)> {
        let (mut col, mut row) = cell_name_to_coordinates(cell)?;
        let mut cell_width = self.get_col_width_pixels(sheet, col)? as f64;
        let mut cell_height = self.get_row_height_pixels(sheet, row)? as f64;
        let merge_cells = self.get_merge_cells(sheet)?;
        let mut rng = Vec::new();
        let mut in_merge_cell = false;
        for merge in merge_cells {
            if in_merge_cell {
                break;
            }
            if let Ok(true) = cell_in_merge_range(cell, &merge) {
                if let Ok(mut coords) = range_ref_to_coordinates(&merge) {
                    sort_coordinates(&mut coords)?;
                    rng = coords;
                    in_merge_cell = true;
                }
            }
        }
        if in_merge_cell {
            cell_width = 0.0;
            cell_height = 0.0;
            col = rng[0];
            row = rng[1];
            for c in rng[0]..=rng[2] {
                cell_width += self.get_col_width_pixels(sheet, c)? as f64;
            }
            for r in rng[1]..=rng[3] {
                cell_height += self.get_row_height_pixels(sheet, r)? as f64;
            }
        }
        let (mut width, mut height) = (width, height);
        if cell_width < width || cell_height < height {
            let asp_width = cell_width / width;
            let asp_height = cell_height / height;
            let asp = asp_width.min(asp_height);
            width *= asp;
            height *= asp;
        }
        if opts.auto_fit_ignore_aspect {
            width = cell_width;
            height = cell_height;
        }
        Ok((
            (width * opts.scale_x) as i32,
            (height * opts.scale_y) as i32,
            col,
            row,
        ))
    }

    fn add_drawing_picture(
        &self,
        sheet: &str,
        drawing_xml: &str,
        cell: &str,
        ext: &str,
        r_id: i32,
        hyperlink_rid: i32,
        dims: (i32, i32),
        opts: &GraphicOptions,
    ) -> Result<()> {
        let (mut col, mut row) = cell_name_to_coordinates(cell)?;
        let (mut width, mut height) = dims;
        if !opts.positioning.is_empty()
            && !["oneCell", "twoCell", "absolute"].contains(&opts.positioning.as_str())
        {
            return Err(crate::errors::new_invalid_optional_value(
                "Positioning",
                &opts.positioning,
                &["oneCell", "twoCell", "absolute"],
            )
            .into());
        }
        if opts.auto_fit {
            (width, height, col, row) =
                self.drawing_resize(sheet, cell, width as f64, height as f64, opts)?;
        } else {
            width = (width as f64 * opts.scale_x) as i32;
            height = (height as f64 * opts.scale_y) as i32;
        }
        let (col_start, row_start, col_end, row_end, x1, y1, x2, y2) =
            self.position_object_pixels(sheet, col, row, width, height, opts)?;
        let (mut content, c_nv_pr_id) = self.drawing_parser(drawing_xml)?;

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
        if opts.positioning != "oneCell" {
            anchor.to = Some(XlsxTo {
                col: col_end as i64,
                col_off: x2 as i64 * EMU as i64,
                row: row_end as i64,
                row_off: y2 as i64 * EMU as i64,
            });
        }

        let mut pic = XlsxPic {
            nv_pic_pr: XlsxNvPicPr {
                c_nv_pr: XlsxCNvPr {
                    id: c_nv_pr_id,
                    descr: opts.alt_text.clone(),
                    name: if opts.name.is_empty() {
                        format!("Picture {c_nv_pr_id}")
                    } else {
                        opts.name.clone()
                    },
                    ..Default::default()
                },
                c_nv_pic_pr: XlsxCNvPicPr {
                    pic_locks: XlsxPicLocks {
                        no_change_aspect: opts.lock_aspect_ratio,
                        ..Default::default()
                    },
                },
            },
            blip_fill: XlsxBlipFill {
                blip: XlsxBlip {
                    embed: format!("rId{r_id}"),
                    xmlns_r: SOURCE_RELATIONSHIP.to_string(),
                    ..Default::default()
                },
                stretch: XlsxStretch {
                    fill_rect: String::new(),
                },
            },
            sp_pr: XlsxSpPr {
                xfrm: XlsxXfrm {
                    off: XlsxOff { x: 0, y: 0 },
                    ext: XlsxPositiveSize2D { cx: 0, cy: 0 },
                },
                prst_geom: XlsxPrstGeom {
                    prst: "rect".to_string(),
                },
                ..Default::default()
            },
        };
        if hyperlink_rid != 0 {
            pic.nv_pic_pr.c_nv_pr.hlink_click = Some(crate::xml::drawing::XlsxHlinkClick {
                r_id: Some(format!("rId{hyperlink_rid}")),
                xmlns_r: Some(SOURCE_RELATIONSHIP.to_string()),
                ..Default::default()
            });
        }
        if ext == ".svg" {
            pic.blip_fill.blip.ext_list = Some(crate::xml::drawing::XlsxEGOfficeArtExtensionList {
                ext: vec![crate::xml::drawing::XlsxCTOfficeArtExtension {
                    uri: EXT_URI_SVG.to_string(),
                    svg_blip: crate::xml::drawing::XlsxCTSVGBlip {
                        xmlns_asvg: NAMESPACE_DRAWING_2016_SVG.to_string(),
                        embed: format!("rId{r_id}"),
                        ..Default::default()
                    },
                }],
            });
        }
        pic.sp_pr.xfrm.ext = XlsxPositiveSize2D {
            cx: width as i64 * EMU as i64,
            cy: height as i64 * EMU as i64,
        };
        if opts.positioning == "oneCell" {
            let cx = x2 as i64 * EMU as i64;
            let cy = y2 as i64 * EMU as i64;
            anchor.ext = Some(XlsxPositiveSize2D { cx, cy });
            pic.sp_pr.xfrm.ext = XlsxPositiveSize2D { cx, cy };
        }
        anchor.pic = Some(pic);
        anchor.client_data = Some(crate::xml::drawing::XdrClientData {
            f_locks_with_sheet: opts.locked.unwrap_or(true),
            f_prints_with_sheet: opts.print_object.unwrap_or(true),
        });

        if opts.positioning == "oneCell" {
            content.one_cell_anchor.push(anchor);
        } else {
            content.two_cell_anchor.push(anchor);
        }
        self.drawings.insert(drawing_xml.to_string(), content);
        Ok(())
    }

    fn get_picture(
        &self,
        row: i32,
        col: i32,
        drawing_xml: &str,
        drawing_rels: &str,
    ) -> Result<Vec<Picture>> {
        let (wsdr, _) = self.drawing_parser(drawing_xml)?;
        let mut pics = Vec::new();
        for anchor in wsdr
            .one_cell_anchor
            .iter()
            .chain(wsdr.two_cell_anchor.iter())
        {
            if anchor.pic.is_none() {
                continue;
            }
            if let Some(from) = &anchor.from {
                if from.col != col as i64 || from.row != row as i64 {
                    continue;
                }
            }
            let pic = anchor.pic.as_ref().unwrap();
            let r_id = &pic.blip_fill.blip.embed;
            if let Some(rel) = self.get_drawing_relationship(drawing_rels, r_id) {
                let target = rel
                    .target
                    .replace("../", "xl/")
                    .trim_start_matches('/')
                    .to_string();
                if let Some(bytes) = self.pkg.get(&target) {
                    let extension = std::path::Path::new(&target)
                        .extension()
                        .and_then(|e| e.to_str())
                        .map(|e| format!(".{e}"))
                        .unwrap_or_default();
                    let mut format = GraphicOptions {
                        scale_x: DEFAULT_DRAWING_SCALE,
                        scale_y: DEFAULT_DRAWING_SCALE,
                        ..Default::default()
                    };
                    if let Some(client) = &anchor.client_data {
                        format.locked = Some(client.f_locks_with_sheet);
                        format.print_object = Some(client.f_prints_with_sheet);
                    }
                    if anchor.to.is_none() {
                        format.positioning = "oneCell".to_string();
                    }
                    if let Some(from) = &anchor.from {
                        format.offset_x = from.col_off / EMU as i64;
                        format.offset_y = from.row_off / EMU as i64;
                    }
                    format.lock_aspect_ratio = pic.nv_pic_pr.c_nv_pic_pr.pic_locks.no_change_aspect;
                    format.alt_text = pic.nv_pic_pr.c_nv_pr.descr.clone();
                    format.name = pic.nv_pic_pr.c_nv_pr.name.clone();
                    calculate_picture_scale(&mut format, bytes.value(), &pic.sp_pr.xfrm.ext);
                    pics.push(Picture {
                        extension,
                        file: bytes.value().clone(),
                        format: Some(format),
                        insert_type: PictureInsertType::PLACE_OVER_CELLS,
                    });
                }
            }
        }
        Ok(pics)
    }

    fn delete_drawing_rels(&self, rels: &str, r_id: &str) {
        if let Ok(Some(mut rels_obj)) = self.rels_reader(rels) {
            rels_obj.relationships.retain(|r| r.id != r_id);
            self.relationships.insert(rels.to_string(), rels_obj);
        }
    }
}

fn calculate_picture_scale(format: &mut GraphicOptions, file: &[u8], ext: &XlsxPositiveSize2D) {
    if ext.cx <= 0 || ext.cy <= 0 {
        return;
    }
    if let Some((w, h)) = image_dimensions(file, ".png") {
        if w > 0 && h > 0 {
            format.scale_x = ((ext.cx / EMU as i64) as f64 / w as f64 * 100.0).round() / 100.0;
            format.scale_y = ((ext.cy / EMU as i64) as f64 / h as f64 * 100.0).round() / 100.0;
        }
    }
}

// ------------------------------------------------------------------
// Tests
// ------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Options;

    #[test]
    fn picture_round_trip() {
        let mut f = File::new_with_options(Options::default());
        // 1x1 red PNG
        let png = vec![
            0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48,
            0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00,
            0x00, 0x90, 0x77, 0x53, 0xde, 0x00, 0x00, 0x00, 0x0c, 0x49, 0x44, 0x41, 0x54, 0x08,
            0x99, 0x63, 0xf8, 0x0f, 0x00, 0x00, 0x01, 0x01, 0x00, 0x05, 0x18, 0xd8, 0x4e, 0x00,
            0x00, 0x00, 0x00, 0x49, 0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
        ];
        let pic = Picture {
            extension: ".png".to_string(),
            file: png.clone(),
            format: Some(GraphicOptions {
                name: "RedPixel".to_string(),
                ..Default::default()
            }),
            ..Default::default()
        };
        f.add_picture_from_bytes("Sheet1", "B2", &pic).unwrap();
        let pics = f.get_pictures("Sheet1", "B2").unwrap();
        assert_eq!(pics.len(), 1);
        assert_eq!(pics[0].extension, ".png");
        assert_eq!(pics[0].format.as_ref().unwrap().name, "RedPixel");

        let cells = f.get_picture_cells("Sheet1").unwrap();
        assert_eq!(cells, vec!["B2"]);
    }

    fn red_pixel_png() -> Vec<u8> {
        vec![
            0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48,
            0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00,
            0x00, 0x90, 0x77, 0x53, 0xde, 0x00, 0x00, 0x00, 0x0c, 0x49, 0x44, 0x41, 0x54, 0x08,
            0x99, 0x63, 0xf8, 0x0f, 0x00, 0x00, 0x01, 0x01, 0x00, 0x05, 0x18, 0xd8, 0x4e, 0x00,
            0x00, 0x00, 0x00, 0x49, 0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
        ]
    }

    #[test]
    fn in_cell_picture_round_trip() {
        let png = red_pixel_png();
        let path = std::env::temp_dir().join("excelize_rs_in_cell_picture.xlsx");
        let path_str = path.to_string_lossy().to_string();
        {
            let mut f = File::new_with_options(Options::default());
            f.add_picture_from_bytes(
                "Sheet1",
                "B2",
                &Picture {
                    extension: ".png".to_string(),
                    file: png.clone(),
                    insert_type: PictureInsertType::PLACE_IN_CELL,
                    ..Default::default()
                },
            )
            .unwrap();
            let pics = f.get_pictures("Sheet1", "B2").unwrap();
            assert!(
                pics.iter()
                    .any(|p| p.insert_type == PictureInsertType::PLACE_IN_CELL && p.file == png),
                "in-cell picture should be readable before save"
            );
            f.save_as(&path_str).unwrap();
            f.close().unwrap();
        }
        let mut f = File::open_file(&path_str, Options::default()).unwrap();
        let pics = f.get_pictures("Sheet1", "B2").unwrap();
        assert_eq!(pics.len(), 1);
        assert_eq!(pics[0].insert_type, PictureInsertType::PLACE_IN_CELL);
        assert_eq!(pics[0].file, png);
        f.close().unwrap();
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn add_picture_from_file_round_trip() {
        let png = red_pixel_png();
        let img_path = std::env::temp_dir().join("excelize_rs_add_picture_from_file.png");
        std::fs::write(&img_path, &png).unwrap();
        let img_path_str = img_path.to_string_lossy().to_string();
        let path = std::env::temp_dir().join("excelize_rs_add_picture_from_file.xlsx");
        let path_str = path.to_string_lossy().to_string();
        {
            let mut f = File::new_with_options(Options::default());
            f.add_picture_from_file(
                "Sheet1",
                "B2",
                &img_path_str,
                PictureInsertType::PLACE_IN_CELL,
                None,
            )
            .unwrap();
            f.add_picture_from_file(
                "Sheet1",
                "B3",
                &img_path_str,
                PictureInsertType::DISPIMG,
                Some(&GraphicOptions {
                    alt_text: "wps image".to_string(),
                    ..Default::default()
                }),
            )
            .unwrap();
            assert!(
                f.add_picture_from_file(
                    "Sheet1",
                    "B4",
                    "image.xyz",
                    PictureInsertType::PLACE_OVER_CELLS,
                    None,
                )
                .is_err(),
                "unsupported extension should be rejected"
            );
            f.save_as(&path_str).unwrap();
            f.close().unwrap();
        }
        let mut f = File::open_file(&path_str, Options::default()).unwrap();
        let pics = f.get_pictures("Sheet1", "B2").unwrap();
        assert!(
            pics.iter()
                .any(|p| p.insert_type == PictureInsertType::PLACE_IN_CELL && p.file == png),
            "expected a PLACE_IN_CELL picture, got {pics:?}"
        );
        let pics = f.get_pictures("Sheet1", "B3").unwrap();
        assert!(
            pics.iter()
                .any(|p| p.insert_type == PictureInsertType::DISPIMG && p.file == png),
            "expected a DISPIMG picture, got {pics:?}"
        );
        f.close().unwrap();
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(&img_path);
    }

    #[test]
    fn dispimg_picture_round_trip() {
        let png = red_pixel_png();
        let path = std::env::temp_dir().join("excelize_rs_dispimg_picture.xlsx");
        let path_str = path.to_string_lossy().to_string();
        {
            let mut f = File::new_with_options(Options::default());
            f.add_picture_from_bytes(
                "Sheet1",
                "B2",
                &Picture {
                    extension: ".png".to_string(),
                    file: png.clone(),
                    insert_type: PictureInsertType::DISPIMG,
                    format: Some(GraphicOptions {
                        alt_text: "wps image".to_string(),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
            )
            .unwrap();
            assert!(
                f.get_cell_formula("Sheet1", "B2")
                    .unwrap()
                    .contains("DISPIMG")
            );
            f.save_as(&path_str).unwrap();
            f.close().unwrap();
        }
        let mut f = File::open_file(&path_str, Options::default()).unwrap();
        let pics = f.get_pictures("Sheet1", "B2").unwrap();
        assert!(
            pics.iter()
                .any(|p| p.insert_type == PictureInsertType::DISPIMG && p.file == png),
            "expected a DISPIMG picture, got {pics:?}"
        );
        f.close().unwrap();
        let _ = std::fs::remove_file(&path);
    }
}
