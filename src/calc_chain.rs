//! Calculation chain and volatile dependencies helpers.
//!
//! Ported from Go `calcchain.go`.

use quick_xml::de::from_reader as xml_from_reader;
use quick_xml::se::to_string as xml_to_string;

use crate::constants::{DEFAULT_XML_PATH_CALC_CHAIN, DEFAULT_XML_PATH_VOLATILE_DEPS};
use crate::errors::Result;
use crate::file::{File, namespace_strict_to_transitional};
use crate::xml::calc_chain::{XlsxCalcChain, XlsxCalcChainC, XlsxVolTypes};
use crate::xml::content_types::XlsxContentTypeEntry;

impl File {
    /// Lazy reader for `xl/calcChain.xml`.
    pub fn calc_chain_reader(&self) -> Result<XlsxCalcChain> {
        if self.calc_chain.lock().unwrap().is_none() {
            let data =
                self.apply_charset_transcoder(&self.read_xml(DEFAULT_XML_PATH_CALC_CHAIN))?;
            let data = namespace_strict_to_transitional(&data);
            let cc = if data.is_empty() {
                XlsxCalcChain::default()
            } else {
                xml_from_reader(data.as_slice())?
            };
            *self.calc_chain.lock().unwrap() = Some(cc);
        }
        Ok(self.calc_chain.lock().unwrap().clone().unwrap())
    }

    /// Serialize and save `xl/calcChain.xml` if it contains entries.
    pub fn calc_chain_writer(&self) {
        if let Some(cc) = self.calc_chain.lock().unwrap().clone() {
            if !cc.c.is_empty() {
                if let Ok(mut output) = xml_to_string(&cc).map(|s| s.into_bytes()) {
                    self.replace_namespace_bytes_if_needed(
                        DEFAULT_XML_PATH_CALC_CHAIN,
                        &mut output,
                    );
                    self.save_file_list(DEFAULT_XML_PATH_CALC_CHAIN, &output);
                }
            }
        }
    }

    /// Remove a cell reference from the calculation chain.
    pub fn delete_calc_chain(&self, index: i32, cell: &str) -> Result<()> {
        let mut calc = self.calc_chain_reader()?;
        calc.c = filter_calc_chain(calc.c, |c| {
            (c.i != index || c.r != cell)
                && (c.i != index || !cell.is_empty())
                && (c.i != 0 || c.r != cell)
        });
        if calc.c.is_empty() {
            *self.calc_chain.lock().unwrap() = None;
            self.pkg.remove(DEFAULT_XML_PATH_CALC_CHAIN);
            let mut ct = self.content_types_reader()?;
            ct.entries.retain(|e| {
                if let XlsxContentTypeEntry::Override(o) = e {
                    o.part_name != "/xl/calcChain.xml"
                } else {
                    true
                }
            });
            *self.content_types.lock().unwrap() = Some(ct);
        } else {
            *self.calc_chain.lock().unwrap() = Some(calc);
        }
        Ok(())
    }

    /// Lazy reader for `xl/volatileDependencies.xml`.
    pub fn volatile_deps_reader(&self) -> Result<Option<XlsxVolTypes>> {
        if self.volatile_deps.lock().unwrap().is_none() {
            if !self.pkg.contains_key(DEFAULT_XML_PATH_VOLATILE_DEPS)
                && !self.temp_files.contains_key(DEFAULT_XML_PATH_VOLATILE_DEPS)
            {
                return Ok(None);
            }
            let data =
                self.apply_charset_transcoder(&self.read_xml(DEFAULT_XML_PATH_VOLATILE_DEPS))?;
            let data = namespace_strict_to_transitional(&data);
            let vt = if data.is_empty() {
                XlsxVolTypes::default()
            } else {
                xml_from_reader(data.as_slice())?
            };
            *self.volatile_deps.lock().unwrap() = Some(vt);
        }
        Ok(self.volatile_deps.lock().unwrap().clone())
    }

    /// Serialize and save `xl/volatileDependencies.xml`.
    pub fn volatile_deps_writer(&self) {
        if let Some(vt) = self.volatile_deps.lock().unwrap().clone() {
            if let Ok(mut output) = xml_to_string(&vt).map(|s| s.into_bytes()) {
                self.replace_namespace_bytes_if_needed(DEFAULT_XML_PATH_VOLATILE_DEPS, &mut output);
                self.save_file_list(DEFAULT_XML_PATH_VOLATILE_DEPS, &output);
            }
        }
    }
}

/// Filter a collection of calculation-chain entries, matching the Go
/// `xlsxCalcChainCollection.Filter` helper.
fn filter_calc_chain(
    collection: Vec<XlsxCalcChainC>,
    predicate: impl Fn(&XlsxCalcChainC) -> bool,
) -> Vec<XlsxCalcChainC> {
    collection.into_iter().filter(predicate).collect()
}

/// Remove a topic reference from the volatile dependencies topic.
pub fn delete_vol_topic_ref(vt: &mut XlsxVolTypes, i1: usize, i2: usize, i3: usize, i4: usize) {
    if i1 < vt.vol_type.len()
        && i2 < vt.vol_type[i1].main.len()
        && i3 < vt.vol_type[i1].main[i2].tp.len()
        && i4 < vt.vol_type[i1].main[i2].tp[i3].tr.len()
    {
        vt.vol_type[i1].main[i2].tp[i3].tr.remove(i4);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::DEFAULT_XML_PATH_CONTENT_TYPES;
    use crate::file::File;
    use crate::options::Options;
    use crate::xml::calc_chain::{XlsxVolMain, XlsxVolTopic, XlsxVolTopicRef, XlsxVolType};
    use crate::xml::content_types::{XlsxContentTypeEntry, XlsxOverride};

    /// Invalid UTF-8 bytes used by the Go tests to force an XML decode error.
    const MACINTOSH_CYRILLIC_CHARSET: &[u8] =
        &[0x8F, 0xF0, 0xE8, 0xE2, 0xE5, 0xF2, 0x20, 0xEC, 0xE8, 0xF0];

    #[test]
    fn calc_chain_reader_empty_new_file() {
        let f = File::new_with_options(Options::default());
        let cc = f.calc_chain_reader().unwrap();
        assert!(cc.c.is_empty());
    }

    #[test]
    fn calc_chain_reader_unsupported_charset() {
        let f = File::new_with_options(Options::default());
        *f.calc_chain.lock().unwrap() = None;
        f.pkg.insert(
            DEFAULT_XML_PATH_CALC_CHAIN.to_string(),
            MACINTOSH_CYRILLIC_CHARSET.to_vec(),
        );
        assert!(f.calc_chain_reader().is_err());
    }

    #[test]
    fn calc_chain_reader_round_trip_from_pkg() {
        let f = File::new_with_options(Options::default());
        let xml = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<calcChain xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
    <c r="A1" i="1"/>
    <c r="B2" i="2" l="1" s="1" t="1" a="1"/>
</calcChain>"#;
        f.pkg
            .insert(DEFAULT_XML_PATH_CALC_CHAIN.to_string(), xml.to_vec());
        *f.calc_chain.lock().unwrap() = None;

        let cc = f.calc_chain_reader().unwrap();
        assert_eq!(cc.c.len(), 2);
        assert_eq!(cc.c[0].r, "A1");
        assert_eq!(cc.c[0].i, 1);
        assert_eq!(cc.c[1].r, "B2");
        assert_eq!(cc.c[1].i, 2);
        assert!(cc.c[1].l);
        assert!(cc.c[1].s);
        assert!(cc.c[1].t);
        assert!(cc.c[1].a);
    }

    #[test]
    fn calc_chain_writer_saves_entries() {
        let f = File::new_with_options(Options::default());
        let cc = XlsxCalcChain {
            c: vec![XlsxCalcChainC {
                r: "B2".to_string(),
                i: 1,
                l: true,
                s: true,
                t: true,
                a: true,
            }],
        };
        *f.calc_chain.lock().unwrap() = Some(cc);
        f.calc_chain_writer();

        let raw = f.read_xml(DEFAULT_XML_PATH_CALC_CHAIN);
        let xml = String::from_utf8(raw).unwrap();
        assert!(xml.contains("<calcChain"));
        assert!(xml.contains("B2"));
    }

    #[test]
    fn calc_chain_writer_skips_empty_chain() {
        let f = File::new_with_options(Options::default());
        *f.calc_chain.lock().unwrap() = Some(XlsxCalcChain::default());
        f.calc_chain_writer();
        assert!(!f.pkg.contains_key(DEFAULT_XML_PATH_CALC_CHAIN));
    }

    #[test]
    fn delete_calc_chain_filters_entries_by_index_and_cell() {
        let f = File::new_with_options(Options::default());
        *f.calc_chain.lock().unwrap() = Some(XlsxCalcChain {
            c: vec![
                XlsxCalcChainC {
                    r: "A1".to_string(),
                    i: 1,
                    ..Default::default()
                },
                XlsxCalcChainC {
                    r: "A2".to_string(),
                    i: 1,
                    ..Default::default()
                },
                XlsxCalcChainC {
                    r: "A1".to_string(),
                    i: 2,
                    ..Default::default()
                },
                XlsxCalcChainC {
                    r: "A3".to_string(),
                    i: 0,
                    ..Default::default()
                },
            ],
        });
        f.delete_calc_chain(1, "A1").unwrap();

        let cc = f.calc_chain_reader().unwrap();
        assert_eq!(cc.c.len(), 3);
        assert!(cc.c.iter().any(|c| c.r == "A2" && c.i == 1));
        assert!(cc.c.iter().any(|c| c.r == "A1" && c.i == 2));
        assert!(cc.c.iter().any(|c| c.r == "A3" && c.i == 0));
    }

    #[test]
    fn delete_calc_chain_filters_all_entries_for_index_when_cell_empty() {
        let f = File::new_with_options(Options::default());
        *f.calc_chain.lock().unwrap() = Some(XlsxCalcChain {
            c: vec![
                XlsxCalcChainC {
                    r: "A1".to_string(),
                    i: 1,
                    ..Default::default()
                },
                XlsxCalcChainC {
                    r: "A2".to_string(),
                    i: 1,
                    ..Default::default()
                },
                XlsxCalcChainC {
                    r: "A1".to_string(),
                    i: 2,
                    ..Default::default()
                },
            ],
        });
        f.delete_calc_chain(1, "").unwrap();

        let cc = f.calc_chain_reader().unwrap();
        assert_eq!(cc.c.len(), 1);
        assert_eq!(cc.c[0].r, "A1");
        assert_eq!(cc.c[0].i, 2);
    }

    #[test]
    fn delete_calc_chain_filters_entries_with_index_zero() {
        let f = File::new_with_options(Options::default());
        *f.calc_chain.lock().unwrap() = Some(XlsxCalcChain {
            c: vec![
                XlsxCalcChainC {
                    r: "A1".to_string(),
                    i: 1,
                    ..Default::default()
                },
                XlsxCalcChainC {
                    r: "A1".to_string(),
                    i: 0,
                    ..Default::default()
                },
                XlsxCalcChainC {
                    r: "A2".to_string(),
                    i: 0,
                    ..Default::default()
                },
            ],
        });
        f.delete_calc_chain(0, "A1").unwrap();

        let cc = f.calc_chain_reader().unwrap();
        assert_eq!(cc.c.len(), 2);
        assert!(cc.c.iter().any(|c| c.r == "A1" && c.i == 1));
        assert!(cc.c.iter().any(|c| c.r == "A2" && c.i == 0));
    }

    #[test]
    fn delete_calc_chain_removes_empty_chain() {
        let f = File::new_with_options(Options::default());
        *f.calc_chain.lock().unwrap() = Some(XlsxCalcChain { c: Vec::new() });
        f.pkg.insert(
            DEFAULT_XML_PATH_CALC_CHAIN.to_string(),
            b"<calcChain/>".to_vec(),
        );
        let mut ct = f.content_types_reader().unwrap();
        ct.entries
            .push(XlsxContentTypeEntry::Override(XlsxOverride {
                part_name: "/xl/calcChain.xml".to_string(),
                content_type:
                    "application/vnd.openxmlformats-officedocument.spreadsheetml.calcChain+xml"
                        .to_string(),
            }));
        *f.content_types.lock().unwrap() = Some(ct);

        f.delete_calc_chain(1, "A1").unwrap();

        assert!(f.calc_chain.lock().unwrap().is_none());
        assert!(!f.pkg.contains_key(DEFAULT_XML_PATH_CALC_CHAIN));
        let ct = f.content_types_reader().unwrap();
        assert!(!ct.entries.iter().any(
            |e| matches!(e, XlsxContentTypeEntry::Override(o) if o.part_name == "/xl/calcChain.xml")
        ));
    }

    #[test]
    fn delete_calc_chain_unsupported_charset() {
        let f = File::new_with_options(Options::default());
        *f.calc_chain.lock().unwrap() = None;
        f.pkg.insert(
            DEFAULT_XML_PATH_CALC_CHAIN.to_string(),
            MACINTOSH_CYRILLIC_CHARSET.to_vec(),
        );
        assert!(f.delete_calc_chain(1, "A1").is_err());
    }

    #[test]
    fn delete_calc_chain_unsupported_charset_content_types() {
        let f = File::new_with_options(Options::default());
        *f.calc_chain.lock().unwrap() = Some(XlsxCalcChain { c: Vec::new() });
        *f.content_types.lock().unwrap() = None;
        f.pkg.insert(
            DEFAULT_XML_PATH_CONTENT_TYPES.to_string(),
            MACINTOSH_CYRILLIC_CHARSET.to_vec(),
        );
        assert!(f.delete_calc_chain(1, "A1").is_err());
    }

    #[test]
    fn volatile_deps_reader_missing_file() {
        let f = File::new_with_options(Options::default());
        assert_eq!(f.volatile_deps_reader().unwrap(), None);
    }

    #[test]
    fn volatile_deps_reader_unsupported_charset() {
        let f = File::new_with_options(Options::default());
        f.pkg.insert(
            DEFAULT_XML_PATH_VOLATILE_DEPS.to_string(),
            MACINTOSH_CYRILLIC_CHARSET.to_vec(),
        );
        assert!(f.volatile_deps_reader().is_err());
    }

    #[test]
    fn volatile_deps_writer_round_trip() {
        let f = File::new_with_options(Options::default());
        let vt = XlsxVolTypes {
            vol_type: vec![XlsxVolType {
                r#type: "realTimeData".to_string(),
                main: vec![XlsxVolMain {
                    first: "A1".to_string(),
                    tp: vec![XlsxVolTopic {
                        t: None,
                        v: String::new(),
                        stp: Vec::new(),
                        tr: vec![XlsxVolTopicRef {
                            r: "A1".to_string(),
                            s: 1,
                        }],
                    }],
                }],
            }],
            ext_lst: None,
        };
        *f.volatile_deps.lock().unwrap() = Some(vt);
        f.volatile_deps_writer();

        let raw = f.read_xml(DEFAULT_XML_PATH_VOLATILE_DEPS);
        let xml = String::from_utf8(raw).unwrap();
        assert!(xml.contains("<volTypes"));
        assert!(xml.contains("realTimeData"));
    }

    #[test]
    fn delete_vol_topic_ref_removes_index() {
        let mut vt = XlsxVolTypes {
            vol_type: vec![XlsxVolType {
                r#type: "realTimeData".to_string(),
                main: vec![XlsxVolMain {
                    first: "A1".to_string(),
                    tp: vec![XlsxVolTopic {
                        t: None,
                        v: String::new(),
                        stp: Vec::new(),
                        tr: vec![
                            XlsxVolTopicRef {
                                r: "A1".to_string(),
                                s: 1,
                            },
                            XlsxVolTopicRef {
                                r: "A2".to_string(),
                                s: 2,
                            },
                            XlsxVolTopicRef {
                                r: "A3".to_string(),
                                s: 3,
                            },
                        ],
                    }],
                }],
            }],
            ext_lst: None,
        };
        delete_vol_topic_ref(&mut vt, 0, 0, 0, 1);
        assert_eq!(vt.vol_type[0].main[0].tp[0].tr.len(), 2);
        assert_eq!(vt.vol_type[0].main[0].tp[0].tr[0].r, "A1");
        assert_eq!(vt.vol_type[0].main[0].tp[0].tr[1].r, "A3");
    }

    #[test]
    fn delete_vol_topic_ref_out_of_bounds_is_no_op() {
        let mut vt = XlsxVolTypes::default();
        delete_vol_topic_ref(&mut vt, 0, 0, 0, 0);
        assert!(vt.vol_type.is_empty());
    }
}
