//! File lifecycle and lazy XML readers/writers.
//!
//! This module corresponds to `file.go` and the `File` struct/methods from
//! `excelize.go` in the original Go implementation.

use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::io::{self, Cursor, Read, Seek, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use dashmap::DashMap;
use quick_xml::de::from_reader as xml_from_reader;
use quick_xml::se::to_string as xml_to_string;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive};

use crate::calc::arg::FormulaArg;
use crate::constants::{
    CONTENT_TYPE_RELATIONSHIPS, CONTENT_TYPE_VBA, DEFAULT_XML_PATH_CONTENT_TYPES,
    DEFAULT_XML_PATH_DOC_PROPS_APP, DEFAULT_XML_PATH_DOC_PROPS_CORE, DEFAULT_XML_PATH_METADATA,
    DEFAULT_XML_PATH_RD_RICH_VALUE, DEFAULT_XML_PATH_RD_RICH_VALUE_REL,
    DEFAULT_XML_PATH_RD_RICH_VALUE_REL_RELS, DEFAULT_XML_PATH_RD_RICH_VALUE_STRUCTURE,
    DEFAULT_XML_PATH_RD_RICH_VALUE_WEB_IMAGE, DEFAULT_XML_PATH_RD_RICH_VALUE_WEB_IMAGE_RELS,
    DEFAULT_XML_PATH_RELS, DEFAULT_XML_PATH_SHARED_STRINGS, DEFAULT_XML_PATH_SHEET,
    DEFAULT_XML_PATH_STYLES, DEFAULT_XML_PATH_THEME, DEFAULT_XML_PATH_WORKBOOK,
    DEFAULT_XML_PATH_WORKBOOK_RELS, MAX_FILE_PATH_LENGTH,
    NAMESPACE_DOCUMENT_PROPERTIES_VARIANT_TYPES, NAMESPACE_DRAWING_ML_MAIN,
    NAMESPACE_EXTENDED_PROPERTIES, NAMESPACE_SPREADSHEET, NAMESPACE_SPREADSHEET_X14,
    OLE_IDENTIFIER, SOURCE_RELATIONSHIP, SOURCE_RELATIONSHIP_CHART, SOURCE_RELATIONSHIP_COMMENTS,
    SOURCE_RELATIONSHIP_CUSTOM_PROPERTIES, SOURCE_RELATIONSHIP_EXTEND_PROPERTIES,
    SOURCE_RELATIONSHIP_IMAGE, SOURCE_RELATIONSHIP_OFFICE_DOCUMENT,
    SOURCE_RELATIONSHIP_SHARED_STRINGS, SOURCE_RELATIONSHIP_VBA_PROJECT, STREAM_CHUNK_SIZE,
    STRICT_NAMESPACE_DOCUMENT_PROPERTIES_VARIANT_TYPES, STRICT_NAMESPACE_DRAWING_ML_MAIN,
    STRICT_NAMESPACE_EXTENDED_PROPERTIES, STRICT_NAMESPACE_SPREADSHEET, STRICT_SOURCE_RELATIONSHIP,
    STRICT_SOURCE_RELATIONSHIP_CHART, STRICT_SOURCE_RELATIONSHIP_COMMENTS,
    STRICT_SOURCE_RELATIONSHIP_EXTEND_PROPERTIES, STRICT_SOURCE_RELATIONSHIP_IMAGE,
    STRICT_SOURCE_RELATIONSHIP_OFFICE_DOCUMENT, UNZIP_SIZE_LIMIT, XML_HEADER,
};
use crate::crypt;
use crate::errors::Result;
use crate::errors::{
    ErrDefinedNameDuplicate, ErrDefinedNameScope, ErrMaxFilePathLength, ErrOptionsUnzipSizeLimit,
    ErrParameterInvalid, ErrSave, ErrUnprotectWorkbook, ErrUnprotectWorkbookPassword,
    ErrWorkbookFileFormat, ErrWorkbookPassword,
};
use crate::lib_util::{count_utf16_string, in_str_slice};
use crate::numfmt;
use crate::options::{CULTURE_NAME_UNKNOWN, Options};
use crate::templates::{
    TEMPLATE_CONTENT_TYPES, TEMPLATE_DOC_PROPS_APP, TEMPLATE_DOC_PROPS_CORE, TEMPLATE_RELS,
    TEMPLATE_SHEET, TEMPLATE_STYLES, TEMPLATE_THEME, TEMPLATE_WORKBOOK, TEMPLATE_WORKBOOK_RELS,
};
use crate::xml::calc_chain::{XlsxCalcChain, XlsxVolTypes};
use crate::xml::content_types::{XlsxDefault, XlsxTypes};
use crate::xml::drawing::XlsxWsDr;
use crate::xml::styles::XlsxStyleSheet;
use crate::xml::table::{XlsxSingleXmlCells, XlsxTable};
use crate::xml::theme::{
    DecodeTheme, XlsxBaseStyles, XlsxCtColor, XlsxFontCollection, XlsxSysClr, XlsxTheme,
};
use crate::xml::workbook::{
    CalcPropsOptions, WorkbookPropsOptions, WorkbookProtectionOptions, XlsxCalcPr,
    XlsxRelationship, XlsxRelationships, XlsxWorkbook, XlsxWorkbookPr, XlsxWorkbookProtection,
};
use crate::xml::worksheet::XlsxWorksheet;

const SUPPORTED_CALC_MODE: &[&str] = &["manual", "auto", "autoNoTable"];
const SUPPORTED_REF_MODE: &[&str] = &["A1", "R1C1"];
const WORKBOOK_PROTECTION_SPIN_COUNT: i32 = 100_000;

/// Combined writer trait used by the ZIP writer factory.
pub trait WriteSeek: Write + Seek {}
impl<T: Write + Seek> WriteSeek for T {}

/// Trait for user-provided ZIP writers.
///
/// Mirrors the subset of `zip::ZipWriter` used when saving a workbook so that
/// callers can inject custom archive implementations.
pub trait ZipWriter {
    /// Start a new file in the archive.
    fn start_file(&mut self, name: &str, options: SimpleFileOptions) -> Result<()>;
    /// Write bytes to the current archive entry.
    fn write_all(&mut self, buf: &[u8]) -> Result<()>;
    /// Finish writing the archive.
    fn finish(self: Box<Self>) -> Result<()>;
}

impl<W: Write + Seek> ZipWriter for zip::ZipWriter<W> {
    fn start_file(&mut self, name: &str, options: SimpleFileOptions) -> Result<()> {
        zip::ZipWriter::start_file(self, name, options)?;
        Ok(())
    }
    fn write_all(&mut self, buf: &[u8]) -> Result<()> {
        std::io::Write::write_all(self, buf)?;
        Ok(())
    }
    fn finish(self: Box<Self>) -> Result<()> {
        zip::ZipWriter::finish(*self)?;
        Ok(())
    }
}

/// Factory used to create a [`ZipWriter`] for a given output writer.
pub type ZipWriterFactory =
    Arc<dyn for<'a> Fn(&'a mut dyn WriteSeek) -> Box<dyn ZipWriter + 'a> + Send + Sync>;

/// Charset transcoder: converts a named encoding to UTF-8 bytes.
pub type CharsetTranscoderFn =
    Arc<dyn Fn(&str, Box<dyn Read>) -> Result<Box<dyn Read>> + Send + Sync>;

/// Backing state for a worksheet that is being written via [`StreamWriter`].
///
/// The temporary file is written directly to the ZIP package at save time
/// without being loaded into memory.
#[allow(dead_code)]
pub(crate) struct StreamState {
    pub tmp_path: PathBuf,
    pub sheet_path: String,
}

/// Populated spreadsheet file.
pub struct File {
    /// Path used by `Save`.
    pub path: Mutex<String>,
    /// User-provided options.
    pub options: Mutex<Options>,
    /// Number of worksheets in the workbook.
    pub sheet_count: Mutex<i32>,
    /// In-memory package parts (path → raw bytes).
    pub pkg: DashMap<String, Vec<u8>>,
    /// Parsed worksheets (path → worksheet).
    pub sheet: DashMap<String, XlsxWorksheet>,
    /// Parsed relationship parts (path → relationships).
    pub relationships: DashMap<String, XlsxRelationships>,
    /// Temporary file mapping (path → filesystem path).
    pub temp_files: DashMap<String, String>,
    /// Map of worksheet names to XML paths.
    pub sheet_map: Mutex<HashMap<String, String>>,
    /// Captured root namespace attributes for each XML part.
    pub xml_attr: DashMap<String, String>,
    /// Marks worksheets that have already been validated.
    pub checked: DashMap<String, bool>,
    /// Streaming worksheet writers that have not yet been written to the ZIP.
    pub(crate) streams: RefCell<HashMap<String, StreamState>>,
    /// Entries that exceed 4GB and need a ZIP64 LFH patch.
    pub zip64_entries: Mutex<Vec<String>>,
    /// Lazily loaded [Content_Types].xml.
    pub content_types: Mutex<Option<XlsxTypes>>,
    /// Lazily loaded xl/styles.xml.
    pub styles: Mutex<Option<XlsxStyleSheet>>,
    /// Lazily loaded xl/workbook.xml.
    pub workbook: Mutex<Option<XlsxWorkbook>>,
    /// Lazily loaded xl/calcChain.xml.
    pub calc_chain: Mutex<Option<XlsxCalcChain>>,
    /// Lazily loaded xl/volatileDependencies.xml.
    pub volatile_deps: Mutex<Option<XlsxVolTypes>>,
    /// Cached calculated cell values (formatted).
    pub calc_cache: Mutex<HashMap<String, String>>,
    /// Cached calculated cell values (raw).
    pub calc_raw_cache: Mutex<HashMap<String, String>>,
    /// Cached formula argument values for dependent cell reuse.
    pub formula_arg_cache: Mutex<HashMap<String, FormulaArg>>,
    /// Lazily loaded theme part.
    pub theme: Mutex<Option<DecodeTheme>>,
    /// Lazily loaded shared string table.
    pub shared_strings: Mutex<Option<crate::xml::shared_strings::XlsxSst>>,
    /// Index map for shared strings.
    pub shared_strings_map: Mutex<HashMap<String, i32>>,
    /// Parsed worksheet drawings (path → drawing).
    pub drawings: DashMap<String, XlsxWsDr>,
    /// Parsed comments parts (path → comments).
    pub comments: DashMap<String, crate::xml::comments::XlsxComments>,
    /// Parsed VML drawings (path → VML drawing).
    pub vml_drawing: DashMap<String, crate::xml::vml::VmlDrawing>,
    /// Optional codepage transcoder for non-UTF-8 XML package parts.
    pub charset_transcoder: Mutex<Option<CharsetTranscoderFn>>,
    /// Optional ZIP writer factory used when saving the workbook.
    pub zip_writer_factory: Mutex<Option<ZipWriterFactory>>,
}

impl fmt::Debug for File {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("File")
            .field("path", &self.path)
            .field("options", &self.options)
            .field("sheet_count", &self.sheet_count)
            .field("pkg", &self.pkg)
            .field("sheet", &self.sheet)
            .field("relationships", &self.relationships)
            .field("temp_files", &self.temp_files)
            .field("sheet_map", &self.sheet_map)
            .field("xml_attr", &self.xml_attr)
            .field("checked", &self.checked)
            .field("streams", &"...")
            .field("zip64_entries", &self.zip64_entries)
            .field("content_types", &self.content_types)
            .field("styles", &self.styles)
            .field("workbook", &self.workbook)
            .field("calc_chain", &self.calc_chain)
            .field("volatile_deps", &self.volatile_deps)
            .field("calc_cache", &"...")
            .field("calc_raw_cache", &"...")
            .field("formula_arg_cache", &"...")
            .field("theme", &self.theme)
            .field("shared_strings", &self.shared_strings)
            .field("shared_strings_map", &self.shared_strings_map)
            .field("drawings", &self.drawings)
            .field("comments", &self.comments)
            .field("vml_drawing", &self.vml_drawing)
            .field("charset_transcoder", &"...")
            .field("zip_writer_factory", &"...")
            .finish()
    }
}

impl File {
    /// Object builder. Use `new_with_options` or `open_file` instead.
    fn new_file() -> Self {
        Self {
            path: Mutex::new(String::new()),
            options: Mutex::new(Options::default()),
            sheet_count: Mutex::new(0),
            pkg: DashMap::new(),
            sheet: DashMap::new(),
            relationships: DashMap::new(),
            temp_files: DashMap::new(),
            sheet_map: Mutex::new(HashMap::new()),
            xml_attr: DashMap::new(),
            checked: DashMap::new(),
            streams: RefCell::new(HashMap::new()),
            zip64_entries: Mutex::new(Vec::new()),
            content_types: Mutex::new(None),
            styles: Mutex::new(None),
            workbook: Mutex::new(None),
            calc_chain: Mutex::new(None),
            volatile_deps: Mutex::new(None),
            calc_cache: Mutex::new(HashMap::new()),
            calc_raw_cache: Mutex::new(HashMap::new()),
            formula_arg_cache: Mutex::new(HashMap::new()),
            theme: Mutex::new(None),
            shared_strings: Mutex::new(None),
            shared_strings_map: Mutex::new(HashMap::new()),
            drawings: DashMap::new(),
            comments: DashMap::new(),
            vml_drawing: DashMap::new(),
            charset_transcoder: Mutex::new(None),
            zip_writer_factory: Mutex::new(None),
        }
    }

    /// Create a new blank workbook.
    pub fn new() -> Self {
        Self::new_with_options(Options::default())
    }

    /// Create a new blank workbook with options.
    pub fn new_with_options(opts: Options) -> Self {
        Self::try_new_with_options(opts).expect("failed to create new workbook")
    }

    fn try_new_with_options(opts: Options) -> Result<Self> {
        let f = Self::new_file();
        *f.options.lock().unwrap() = normalize_options(opts);

        f.store_template(DEFAULT_XML_PATH_RELS, TEMPLATE_RELS);
        f.store_template(DEFAULT_XML_PATH_DOC_PROPS_APP, TEMPLATE_DOC_PROPS_APP);
        f.store_template(DEFAULT_XML_PATH_DOC_PROPS_CORE, TEMPLATE_DOC_PROPS_CORE);
        f.store_template(DEFAULT_XML_PATH_WORKBOOK_RELS, TEMPLATE_WORKBOOK_RELS);
        f.store_template(DEFAULT_XML_PATH_THEME, TEMPLATE_THEME);
        f.store_template(DEFAULT_XML_PATH_SHEET, TEMPLATE_SHEET);
        f.store_template(DEFAULT_XML_PATH_STYLES, TEMPLATE_STYLES);
        f.store_template(DEFAULT_XML_PATH_WORKBOOK, TEMPLATE_WORKBOOK);
        f.store_template(DEFAULT_XML_PATH_CONTENT_TYPES, TEMPLATE_CONTENT_TYPES);

        *f.sheet_count.lock().unwrap() = 1;

        // Prime lazy readers.
        let _ = f.calc_chain_reader()?;
        let _ = f.content_types_reader()?;
        let _ = f.styles_reader()?;
        let _ = f.workbook_reader()?;

        f.relationships.insert(
            DEFAULT_XML_PATH_WORKBOOK_RELS.to_string(),
            f.rels_reader(DEFAULT_XML_PATH_WORKBOOK_RELS)?
                .unwrap_or_default(),
        );

        f.sheet_map
            .lock()
            .unwrap()
            .insert("Sheet1".to_string(), DEFAULT_XML_PATH_SHEET.to_string());

        let ws = f.work_sheet_reader("Sheet1")?;
        f.sheet.insert(DEFAULT_XML_PATH_SHEET.to_string(), ws);

        let _ = f.theme_reader();
        Ok(f)
    }

    /// Open a workbook from the filesystem.
    pub fn open_file(path: &str, opts: Options) -> Result<Self> {
        let file = fs::File::open(Path::new(path))?;
        let size = file.metadata()?.len();
        let f = Self::open_reader(file, size, opts)?;
        *f.path.lock().unwrap() = path.to_string();
        Ok(f)
    }

    /// Open a workbook from a readable/seekable source.
    pub fn open_reader<R: Read + Seek>(mut reader: R, _size: u64, opts: Options) -> Result<Self> {
        let mut header = [0u8; 8];
        reader.read_exact(&mut header)?;
        reader.seek(io::SeekFrom::Start(0))?;

        let has_password = !opts.password.is_empty();
        if header == OLE_IDENTIFIER {
            if !has_password {
                return Err(Box::new(ErrWorkbookFileFormat));
            }
            let mut encrypted = Vec::new();
            reader.read_to_end(&mut encrypted)?;
            let decrypted = crypt::decrypt(&encrypted, &opts)?;
            return Self::open_reader_internal(Cursor::new(decrypted), opts, true);
        }
        if has_password {
            return Err(Box::new(ErrWorkbookPassword));
        }
        Self::open_reader_internal(reader, opts, false)
    }

    fn open_reader_internal<R: Read + Seek>(
        reader: R,
        opts: Options,
        has_password: bool,
    ) -> Result<Self> {
        let f = Self::new_file();
        *f.options.lock().unwrap() = normalize_options(opts);
        f.check_open_reader_options()?;

        let zip_result = (|| -> Result<()> {
            let mut archive = ZipArchive::new(reader)?;
            let (files, sheet_count) = f.read_zip_archive(&mut archive)?;
            drop(archive);

            for (k, v) in files {
                f.pkg.insert(k, v);
            }
            *f.sheet_count.lock().unwrap() = sheet_count;
            Ok(())
        })();

        match zip_result {
            Ok(()) => {
                let _ = f.calc_chain_reader()?;
                {
                    let map = f.get_sheet_name_to_path_map()?;
                    *f.sheet_map.lock().unwrap() = map;
                }
                let _ = f.styles_reader()?;
                let _ = f.theme_reader();
                Ok(f)
            }
            Err(_) if has_password => Err(Box::new(ErrWorkbookPassword)),
            Err(e) => Err(e),
        }
    }

    /// Save to the original path.
    pub fn save(&self) -> Result<()> {
        let path = self.path.lock().unwrap().clone();
        if path.is_empty() {
            return Err(Box::new(ErrSave));
        }
        self.write_to_path(&path)
    }

    /// Save the workbook to the given path.
    pub fn save_as(&mut self, path: &str) -> Result<()> {
        {
            let mut p = self.path.lock().unwrap();
            *p = path.to_string();
        }
        self.write_to_path(path)
    }

    fn write_to_path(&self, path: &str) -> Result<()> {
        if count_utf16_string(path) > MAX_FILE_PATH_LENGTH {
            return Err(Box::new(ErrMaxFilePathLength));
        }
        let ext = Path::new(path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        if !crate::templates::SUPPORTED_CONTENT_TYPES.contains_key(&format!(".{ext}")) {
            return Err(Box::new(ErrWorkbookFileFormat));
        }
        let file = fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(Path::new(path))?;
        self.write_to(file)
    }

    /// Write the workbook to any writer.
    pub fn write_to<W: Write>(&self, mut writer: W) -> Result<()> {
        let path = self.path.lock().unwrap().clone();
        if !path.is_empty() {
            let ext = Path::new(&path)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();
            if let Some(ct) = crate::templates::SUPPORTED_CONTENT_TYPES.get(&format!(".{ext}")) {
                self.set_content_type_part_project_extensions(ct)?;
            } else {
                return Err(Box::new(ErrWorkbookFileFormat));
            }
        }
        let buf = self.write_to_buffer()?;
        writer.write_all(&buf)?;
        Ok(())
    }

    /// Clean up temporary files.
    pub fn close(&mut self) -> Result<()> {
        let mut first_err: Option<io::Error> = None;
        for entry in self.temp_files.iter() {
            if let Err(e) = fs::remove_file(entry.value()) {
                if first_err.is_none() {
                    first_err = Some(e);
                }
            }
        }
        self.temp_files.clear();

        for state in self.streams.borrow().values() {
            if let Err(e) = fs::remove_file(&state.tmp_path) {
                if first_err.is_none() {
                    first_err = Some(e);
                }
            }
        }
        self.streams.borrow_mut().clear();

        match first_err {
            Some(e) => Err(Box::new(e)),
            None => Ok(()),
        }
    }

    /// Set a user-defined charset transcoder for non-UTF-8 XML package parts.
    ///
    /// Mirrors Go `File.CharsetTranscoder`.
    pub fn charset_transcoder<F>(&self, transcoder: F) -> &Self
    where
        F: Fn(&str, Box<dyn Read>) -> Result<Box<dyn Read>> + Send + Sync + 'static,
    {
        *self.charset_transcoder.lock().unwrap() = Some(Arc::new(transcoder));
        self
    }

    /// Set a user-defined ZIP writer factory for saving the workbook.
    ///
    /// Mirrors Go `File.SetZipWriter`. The factory receives a writer that also
    /// implements `Seek` because the underlying `zip::ZipWriter` needs to seek
    /// back to write the central directory.
    pub fn set_zip_writer<F>(&self, factory: F) -> &Self
    where
        F: for<'a> Fn(&'a mut dyn WriteSeek) -> Box<dyn ZipWriter + 'a> + Send + Sync + 'static,
    {
        *self.zip_writer_factory.lock().unwrap() = Some(Arc::new(factory));
        self
    }

    /// Apply the configured charset transcoder when the XML declaration names a
    /// non-UTF-8 encoding.
    pub(crate) fn apply_charset_transcoder(&self, data: &[u8]) -> Result<Vec<u8>> {
        let encoding = detect_xml_encoding(data).unwrap_or("UTF-8");
        if encoding.eq_ignore_ascii_case("UTF-8") || encoding.eq_ignore_ascii_case("UTF8") {
            return Ok(data.to_vec());
        }
        if let Some(transcoder) = self.charset_transcoder.lock().unwrap().clone() {
            let input: Box<dyn Read> = Box::new(Cursor::new(data.to_vec()));
            let mut converted = transcoder(encoding, input)?;
            let mut out = Vec::new();
            converted.read_to_end(&mut out)?;
            return Ok(out);
        }
        Ok(data.to_vec())
    }

    // ------------------------------------------------------------------
    // Internal readers / helpers
    // ------------------------------------------------------------------

    fn store_template(&self, path: &str, template: &str) {
        let bytes = format!("{XML_HEADER}{template}").into_bytes();
        if let Some(attrs) = extract_root_namespace_attributes(&bytes) {
            self.xml_attr.insert(path.to_string(), attrs);
        }
        self.pkg.insert(path.to_string(), bytes);
    }

    /// Read XML content as bytes from the in-memory package.
    pub fn read_xml(&self, name: &str) -> Vec<u8> {
        self.pkg
            .get(name)
            .map(|e| e.value().clone())
            .unwrap_or_default()
    }

    /// Read file content as bytes, falling back to temporary files.
    pub fn read_bytes(&self, name: &str) -> Vec<u8> {
        let content = self.read_xml(name);
        if !content.is_empty() {
            return content;
        }
        match self.read_temp(name) {
            Ok(mut file) => {
                let mut content = Vec::new();
                if file.read_to_end(&mut content).is_ok() {
                    self.pkg.insert(name.to_string(), content.clone());
                }
                content
            }
            Err(_) => Vec::new(),
        }
    }

    fn read_temp(&self, name: &str) -> io::Result<fs::File> {
        let path = self
            .temp_files
            .get(name)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no temp file"))?
            .clone();
        fs::File::open(path)
    }

    /// Update or add a package part, prefixing the standard XML header.
    pub fn save_file_list(&self, name: &str, content: &[u8]) {
        let mut out = Vec::with_capacity(XML_HEADER.len() + content.len());
        out.extend_from_slice(XML_HEADER.as_bytes());
        out.extend_from_slice(content);
        self.pkg.insert(name.to_string(), out);
    }

    /// Extract a workbook from a `ZipArchive`.
    fn read_zip_archive<R: Read + Seek>(
        &self,
        archive: &mut ZipArchive<R>,
    ) -> Result<(HashMap<String, Vec<u8>>, i32)> {
        let mut files = HashMap::new();
        let mut worksheets = 0;
        let mut unzip_size: i64 = 0;
        let opts = self.options.lock().unwrap();

        for i in 0..archive.len() {
            let mut zip_file = archive.by_index(i)?;
            let file_size = zip_file.size() as i64;
            unzip_size += file_size;
            if unzip_size > opts.unzip_size_limit {
                return Err(Box::new(io::Error::new(
                    io::ErrorKind::InvalidData,
                    crate::errors::new_unzip_size_limit_error(opts.unzip_size_limit),
                )));
            }
            let mut file_name = zip_file.name().replace('\\', "/");
            let lower = file_name.to_lowercase();
            if lower == "[content_types].xml" {
                file_name = DEFAULT_XML_PATH_CONTENT_TYPES.to_string();
            } else if lower == "xl/sharedstrings.xml" {
                file_name = DEFAULT_XML_PATH_SHARED_STRINGS.to_string();
            }

            if lower == DEFAULT_XML_PATH_SHARED_STRINGS.to_lowercase()
                && file_size > opts.unzip_xml_size_limit
            {
                if let Ok(tmp) = self.unzip_to_temp(&mut zip_file) {
                    self.temp_files.insert(file_name, tmp);
                }
                continue;
            }
            if lower.starts_with("xl/worksheets/sheet") {
                worksheets += 1;
                if file_size > opts.unzip_xml_size_limit && !zip_file.is_dir() {
                    if let Ok(tmp) = self.unzip_to_temp(&mut zip_file) {
                        self.temp_files.insert(file_name, tmp);
                    }
                    continue;
                }
            }

            let mut data = Vec::with_capacity(file_size.max(0) as usize);
            zip_file.read_to_end(&mut data)?;
            files.insert(file_name, data);
        }
        Ok((files, worksheets))
    }

    fn unzip_to_temp(&self, zip_file: &mut zip::read::ZipFile<'_>) -> io::Result<String> {
        let tmp_dir = {
            let opts = self.options.lock().unwrap();
            if opts.tmp_dir.is_empty() {
                std::env::temp_dir()
            } else {
                PathBuf::from(&opts.tmp_dir)
            }
        };
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        let name = format!("excelize-{}-{}.xml", now.as_secs(), now.subsec_nanos());
        let path = tmp_dir.join(name);
        let mut file = fs::File::create(&path)?;
        io::copy(zip_file, &mut file)?;
        file.sync_all()?;
        Ok(path.to_string_lossy().to_string())
    }

    /// Lazy reader for `[Content_Types].xml`.
    pub fn content_types_reader(&self) -> Result<XlsxTypes> {
        if self.content_types.lock().unwrap().is_none() {
            let mut ct = XlsxTypes::default();
            let data =
                self.apply_charset_transcoder(&self.read_xml(DEFAULT_XML_PATH_CONTENT_TYPES))?;
            let data = namespace_strict_to_transitional(&data);
            if !data.is_empty() {
                ct = xml_from_reader(data.as_slice())?;
            }
            *self.content_types.lock().unwrap() = Some(ct);
        }
        Ok(self.content_types.lock().unwrap().clone().unwrap())
    }

    /// Lazy reader for `xl/styles.xml`.
    pub fn styles_reader(&self) -> Result<XlsxStyleSheet> {
        if self.styles.lock().unwrap().is_none() {
            let data = self.apply_charset_transcoder(&self.read_xml(DEFAULT_XML_PATH_STYLES))?;
            let data = namespace_strict_to_transitional(&data);
            let st = if data.is_empty() {
                XlsxStyleSheet::default()
            } else {
                if let Some(attrs) = extract_root_namespace_attributes(&data) {
                    self.xml_attr
                        .insert(DEFAULT_XML_PATH_STYLES.to_string(), attrs);
                }
                xml_from_reader(data.as_slice()).unwrap_or_default()
            };
            *self.styles.lock().unwrap() = Some(st);
        }
        Ok(self.styles.lock().unwrap().clone().unwrap())
    }

    /// Lazy reader for `xl/workbook.xml`.
    pub fn workbook_reader(&self) -> Result<XlsxWorkbook> {
        if self.workbook.lock().unwrap().is_none() {
            let wb_path = self.get_workbook_path();
            let data = self.apply_charset_transcoder(&self.read_xml(&wb_path))?;
            let data = namespace_strict_to_transitional(&data);
            // Strip extension-list blocks before deserializing: quick-xml/serde
            // cannot capture arbitrary nested XML in `<ext><$value/></ext>`.
            let data = strip_xml_element(&data, "extLst");
            let mut wb: XlsxWorkbook = if data.is_empty() {
                XlsxWorkbook::default()
            } else {
                if let Some(attrs) = extract_root_namespace_attributes(&data) {
                    self.xml_attr.insert(wb_path.clone(), attrs);
                }
                xml_from_reader(data.as_slice()).unwrap_or_default()
            };
            // quick-xml/serde surfaces the namespaced `r:id` attribute under
            // its local name; move it back so sheets serialize with the
            // required `r:id` attribute.
            for sheet in &mut wb.sheets.sheet {
                if sheet.id.is_none() {
                    sheet.id = sheet.plain_id.take();
                }
            }
            *self.workbook.lock().unwrap() = Some(wb);
        }
        Ok(self.workbook.lock().unwrap().clone().unwrap())
    }

    // Calculation chain helpers moved to `calc_chain.rs`.

    /// Lazy reader for the theme part.
    pub fn theme_reader(&self) -> Result<Option<DecodeTheme>> {
        if self.theme.lock().unwrap().is_none() {
            if self.pkg.contains_key(DEFAULT_XML_PATH_THEME)
                || self.temp_files.contains_key(DEFAULT_XML_PATH_THEME)
            {
                let data = self.apply_charset_transcoder(&self.read_xml(DEFAULT_XML_PATH_THEME))?;
                let data = namespace_strict_to_transitional(&data);
                if !data.is_empty() {
                    if let Some(attrs) = extract_root_namespace_attributes(&data) {
                        self.xml_attr
                            .insert(DEFAULT_XML_PATH_THEME.to_string(), attrs);
                    }
                    if let Ok(theme) = xml_from_reader::<_, DecodeTheme>(data.as_slice()) {
                        *self.theme.lock().unwrap() = Some(theme);
                    }
                }
            }
        }
        Ok(self.theme.lock().unwrap().clone())
    }

    /// Lazy reader for the shared string table.
    pub fn shared_strings_reader(&self) -> Result<crate::xml::shared_strings::XlsxSst> {
        if self.shared_strings.lock().unwrap().is_none() {
            let mut sst = crate::xml::shared_strings::XlsxSst::default();
            if self.pkg.contains_key(DEFAULT_XML_PATH_SHARED_STRINGS)
                || self
                    .temp_files
                    .contains_key(DEFAULT_XML_PATH_SHARED_STRINGS)
            {
                let data = self
                    .apply_charset_transcoder(&self.read_bytes(DEFAULT_XML_PATH_SHARED_STRINGS))?;
                let data = namespace_strict_to_transitional(&data);
                if !data.is_empty() {
                    sst = xml_from_reader(data.as_slice()).unwrap_or_default();
                }
            }
            let mut map = self.shared_strings_map.lock().unwrap();
            map.clear();
            for (i, si) in sst.si.iter().enumerate() {
                let text = if let Some(t) = &si.t {
                    t.val.clone()
                } else {
                    si.r.iter()
                        .filter_map(|r| r.t.as_ref().map(|t| t.val.clone()))
                        .collect()
                };
                map.insert(text, i as i32);
            }
            *self.shared_strings.lock().unwrap() = Some(sst);
        }
        Ok(self.shared_strings.lock().unwrap().clone().unwrap())
    }

    /// Lazy reader for relationship parts.
    pub fn rels_reader(&self, path: &str) -> Result<Option<XlsxRelationships>> {
        if let Some(rels) = self.relationships.get(path) {
            return Ok(Some(rels.clone()));
        }
        if self.pkg.contains_key(path) || self.temp_files.contains_key(path) {
            let data = self.apply_charset_transcoder(&self.read_xml(path))?;
            let data = namespace_strict_to_transitional(&data);
            let rels = if data.is_empty() {
                XlsxRelationships::default()
            } else {
                if let Some(attrs) = extract_root_namespace_attributes(&data) {
                    self.xml_attr.insert(path.to_string(), attrs);
                }
                xml_from_reader(data.as_slice()).unwrap_or_default()
            };
            self.relationships.insert(path.to_string(), rels.clone());
            Ok(Some(rels))
        } else {
            Ok(None)
        }
    }

    /// Lazy reader for `xl/metadata.xml`.
    pub fn metadata_reader(&self) -> Result<crate::xml::metadata::XlsxMetadata> {
        let data = namespace_strict_to_transitional(
            &self.apply_charset_transcoder(&self.read_xml(DEFAULT_XML_PATH_METADATA))?,
        );
        if data.is_empty() {
            return Ok(crate::xml::metadata::XlsxMetadata::default());
        }
        // Strip blocks with arbitrary nested XML that quick-xml/serde cannot
        // capture in `$value` fields; the value metadata used by in-cell
        // pictures is preserved.
        let data = strip_xml_element(&data, "metadataTypes");
        let data = strip_xml_element(&data, "extLst");
        Ok(xml_from_reader(data.as_slice())?)
    }

    /// Lazy reader for `xl/richData/rdrichvalue.xml`.
    pub fn rich_value_reader(&self) -> Result<crate::xml::metadata::XlsxRichValueData> {
        let data = namespace_strict_to_transitional(
            &self.apply_charset_transcoder(&self.read_xml(DEFAULT_XML_PATH_RD_RICH_VALUE))?,
        );
        if data.is_empty() {
            return Ok(crate::xml::metadata::XlsxRichValueData::default());
        }
        Ok(xml_from_reader(data.as_slice())?)
    }

    /// Lazy reader for `xl/richData/richValueRel.xml`.
    pub fn rich_value_rel_reader(&self) -> Result<crate::xml::metadata::XlsxRichValueRels> {
        let data = namespace_strict_to_transitional(
            &self.apply_charset_transcoder(&self.read_xml(DEFAULT_XML_PATH_RD_RICH_VALUE_REL))?,
        );
        if data.is_empty() {
            return Ok(crate::xml::metadata::XlsxRichValueRels::default());
        }
        Ok(xml_from_reader(data.as_slice())?)
    }

    /// Lazy reader for `xl/richData/rdrichvaluestructure.xml`.
    pub fn rich_value_structures_reader(
        &self,
    ) -> Result<crate::xml::metadata::XlsxRichValueStructures> {
        let data =
            namespace_strict_to_transitional(&self.apply_charset_transcoder(
                &self.read_xml(DEFAULT_XML_PATH_RD_RICH_VALUE_STRUCTURE),
            )?);
        if data.is_empty() {
            return Ok(crate::xml::metadata::XlsxRichValueStructures::default());
        }
        Ok(xml_from_reader(data.as_slice())?)
    }

    /// Lazy reader for `xl/richData/rdRichValueWebImage.xml`.
    pub fn rich_value_web_image_reader(
        &self,
    ) -> Result<crate::xml::metadata::XlsxWebImagesSupportingRichData> {
        let data =
            namespace_strict_to_transitional(&self.apply_charset_transcoder(
                &self.read_xml(DEFAULT_XML_PATH_RD_RICH_VALUE_WEB_IMAGE),
            )?);
        if data.is_empty() {
            return Ok(crate::xml::metadata::XlsxWebImagesSupportingRichData::default());
        }
        Ok(xml_from_reader(data.as_slice())?)
    }

    /// Get a relationship from `xl/richData/_rels/richValueRel.xml.rels` by ID.
    pub(crate) fn get_rich_data_rich_value_rel_relationship(
        &self,
        r_id: &str,
    ) -> Option<XlsxRelationship> {
        self.rels_reader(DEFAULT_XML_PATH_RD_RICH_VALUE_REL_RELS)
            .ok()
            .flatten()
            .and_then(|rels| rels.relationships.into_iter().find(|rel| rel.id == r_id))
    }

    /// Get a relationship from `xl/richData/_rels/rdRichValueWebImage.xml.rels` by ID.
    pub(crate) fn get_rich_value_web_image_relationship(
        &self,
        r_id: &str,
    ) -> Option<XlsxRelationship> {
        self.rels_reader(DEFAULT_XML_PATH_RD_RICH_VALUE_WEB_IMAGE_RELS)
            .ok()
            .flatten()
            .and_then(|rels| rels.relationships.into_iter().find(|rel| rel.id == r_id))
    }

    /// Set an existing relationship entry, or add a new one if `r_id` is empty.
    pub fn set_rels(
        &self,
        r_id: &str,
        rel_path: &str,
        rel_type: &str,
        target: &str,
        target_mode: &str,
    ) -> i32 {
        if r_id.is_empty() {
            return self.add_rels(rel_path, rel_type, target, target_mode);
        }
        let mut rels = self
            .rels_reader(rel_path)
            .unwrap_or_default()
            .unwrap_or_default();
        let mut out_id = 0;
        for rel in &mut rels.relationships {
            if rel.id == r_id {
                rel.r#type = rel_type.to_string();
                rel.target = target.to_string();
                rel.target_mode = Some(target_mode.to_string());
                out_id = r_id.trim_start_matches("rId").parse().unwrap_or(0);
                break;
            }
        }
        self.relationships.insert(rel_path.to_string(), rels);
        out_id
    }

    /// Add a relationship to a relationship part.
    pub fn add_rels(&self, rel_path: &str, rel_type: &str, target: &str, target_mode: &str) -> i32 {
        let uniq_part: HashMap<String, String> = [
            (
                SOURCE_RELATIONSHIP_CUSTOM_PROPERTIES.to_string(),
                "/docProps/custom.xml".to_string(),
            ),
            (
                SOURCE_RELATIONSHIP_SHARED_STRINGS.to_string(),
                "/xl/sharedStrings.xml".to_string(),
            ),
        ]
        .into_iter()
        .collect();

        let mut rels = self
            .rels_reader(rel_path)
            .unwrap_or_default()
            .unwrap_or_default();
        let mut r_id = 0;
        for rel in &rels.relationships {
            let id: i32 = rel.id.trim_start_matches("rId").parse().unwrap_or(0);
            if id > r_id {
                r_id = id;
            }
            if rel.r#type == rel_type {
                if let Some(part_name) = uniq_part.get(&rel.r#type) {
                    // This branch is handled by the caller updating the target.
                    let _ = part_name;
                }
            }
        }
        r_id += 1;
        let new_id = format!("rId{r_id}");
        rels.relationships.push(XlsxRelationship {
            id: new_id,
            r#type: rel_type.to_string(),
            target: target.to_string(),
            target_mode: if target_mode.is_empty() {
                None
            } else {
                Some(target_mode.to_string())
            },
        });
        self.relationships.insert(rel_path.to_string(), rels);
        r_id
    }

    /// Get the workbook XML path from the root relationships.
    pub fn get_workbook_path(&self) -> String {
        if let Ok(Some(rels)) = self.rels_reader(DEFAULT_XML_PATH_RELS) {
            for rel in &rels.relationships {
                if rel.r#type == crate::constants::SOURCE_RELATIONSHIP_OFFICE_DOCUMENT {
                    return rel.target.trim_start_matches('/').to_string();
                }
            }
        }
        DEFAULT_XML_PATH_WORKBOOK.to_string()
    }

    /// Get the workbook relationships path.
    pub fn get_workbook_rels_path(&self) -> String {
        let wb = self.get_workbook_path();
        let wb_dir = Path::new(&wb).parent().unwrap_or(Path::new("."));
        let wb_base = Path::new(&wb).file_name().unwrap_or_default();
        if wb_dir == Path::new(".") {
            return format!("_rels/{}", wb_base.to_string_lossy()) + ".rels";
        }
        format!(
            "{}/_rels/{}.rels",
            wb_dir.to_string_lossy(),
            wb_base.to_string_lossy()
        )
        .trim_start_matches('/')
        .to_string()
    }

    /// Convert a relative relationship target to an absolute package path.
    pub fn get_worksheet_path(&self, rel_target: &str) -> String {
        let wb_path = self.get_workbook_path();
        let wb_dir = Path::new(&wb_path).parent().unwrap_or(Path::new("."));
        let mut combined = wb_dir.to_path_buf();
        for c in rel_target.split('/') {
            if c == ".." {
                combined.pop();
            } else if !c.is_empty() && c != "." {
                combined.push(c);
            }
        }
        let mut s = combined.to_string_lossy().replace('\\', "/");
        s = s.trim_start_matches('/').to_string();
        if rel_target.starts_with('/') {
            s = Path::new(rel_target)
                .to_string_lossy()
                .replace('\\', "/")
                .trim_start_matches('/')
                .to_string();
        }
        s
    }

    /// Get the XML path for a worksheet by name.
    pub fn get_sheet_xml_path(&self, sheet: &str) -> Option<String> {
        let map = self.sheet_map.lock().unwrap();
        for (name, path) in map.iter() {
            if name.eq_ignore_ascii_case(sheet) {
                return Some(path.clone());
            }
        }
        None
    }

    /// Deserialize and return a worksheet by name.
    pub fn work_sheet_reader(&self, sheet: &str) -> Result<XlsxWorksheet> {
        crate::excelize::check_sheet_name(sheet)?;
        let name = self.get_sheet_xml_path(sheet).ok_or_else(|| {
            Box::new(crate::errors::ErrSheetNotExist {
                sheet_name: sheet.to_string(),
            }) as Box<dyn std::error::Error + Send + Sync>
        })?;
        if let Some(ws) = self.sheet.get(&name) {
            return Ok(ws.clone());
        }
        let data = self.apply_charset_transcoder(&self.read_bytes(&name))?;
        let data = namespace_strict_to_transitional(&data);
        let mut data = data;
        let ext_lst = extract_ext_lst(&data);
        if ext_lst.is_some() {
            remove_ext_lst(&mut data);
        }
        let mut ws: XlsxWorksheet = xml_from_reader(data.as_slice())?;
        if let Some(xml) = ext_lst {
            ws.ext_lst = Some(crate::xml::common::parse_ext_lst_content(&xml)?);
        }
        if !self.checked.contains_key(&name) {
            // Worksheet validation is intentionally minimal in this phase.
            self.checked.insert(name.clone(), true);
        }
        self.sheet.insert(name.clone(), ws.clone());
        Ok(ws)
    }

    // ------------------------------------------------------------------
    // Validation helpers
    // ------------------------------------------------------------------

    pub(crate) fn check_open_reader_options(&self) -> Result<()> {
        let mut opts = self.options.lock().unwrap();
        if opts.unzip_size_limit == 0 {
            opts.unzip_size_limit = opts.unzip_xml_size_limit.max(UNZIP_SIZE_LIMIT);
        }
        if opts.unzip_xml_size_limit == 0 {
            opts.unzip_xml_size_limit = opts.unzip_size_limit.min(STREAM_CHUNK_SIZE);
        }
        if opts.unzip_xml_size_limit > opts.unzip_size_limit {
            return Err(Box::new(ErrOptionsUnzipSizeLimit));
        }
        let patterns = [
            opts.short_date_pattern.clone(),
            opts.long_date_pattern.clone(),
            opts.long_time_pattern.clone(),
        ];
        drop(opts);
        self.check_date_time_pattern(&patterns)
    }

    fn check_date_time_pattern(&self, patterns: &[String]) -> Result<()> {
        for pattern in patterns {
            if !pattern.is_empty() && !numfmt::is_date_time_pattern(pattern) {
                return Err(Box::new(crate::errors::ErrUnsupportedNumberFormat));
            }
        }
        Ok(())
    }

    // ------------------------------------------------------------------
    // Writers
    // ------------------------------------------------------------------

    /// Serialize the workbook to an in-memory byte buffer.
    pub fn write_to_buffer(&self) -> Result<Vec<u8>> {
        let mut buf = Cursor::new(Vec::new());
        let factory = self.zip_writer_factory.lock().unwrap().clone();
        if let Some(factory) = factory {
            let mut zw = factory(&mut buf);
            self.write_to_zip(&mut *zw)?;
            zw.finish()?;
        } else {
            let mut zw = Box::new(zip::ZipWriter::new(&mut buf));
            self.write_to_zip(&mut *zw)?;
            zw.finish()?;
        }
        let mut inner = buf.into_inner();
        self.write_zip64_lfh(&mut inner)?;
        let opts = self.options.lock().unwrap().clone();
        if !opts.password.is_empty() {
            inner = crypt::encrypt(&inner, &opts)?;
        }
        Ok(inner)
    }

    fn write_to_zip(&self, zw: &mut dyn ZipWriter) -> Result<()> {
        self.calc_chain_writer();
        self.comments_writer();
        self.shared_strings_registrar();
        self.content_types_writer();
        self.drawings_writer();
        self.volatile_deps_writer();
        self.vml_drawing_writer();
        self.workbook_writer();
        self.work_sheet_writer();
        self.rels_writer();
        let _ = self.shared_strings_loader();
        self.shared_strings_writer();
        self.style_sheet_writer();
        self.theme_writer();

        // Write package parts in reverse alphabetical order (matches Go behavior).
        let mut files: Vec<String> = self.pkg.iter().map(|e| e.key().clone()).collect();
        files.sort_unstable_by(|a, b| b.cmp(a));
        for path in files {
            let opts = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
            zw.start_file(&path, opts)?;
            let content = self.read_xml(&path);
            zw.write_all(&content)?;
            if content.len() as u64 > u32::MAX as u64 {
                self.zip64_entries.lock().unwrap().push(path.clone());
            }
        }

        // Write any parts that only exist as temporary files.
        let mut temp_files: Vec<String> = self
            .temp_files
            .iter()
            .filter(|e| !self.pkg.contains_key(e.key()))
            .map(|e| e.key().clone())
            .collect();
        temp_files.sort_unstable_by(|a, b| b.cmp(a));
        for path in temp_files {
            let opts = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
            zw.start_file(&path, opts)?;
            let content = self.read_bytes(&path);
            zw.write_all(&content)?;
            if content.len() as u64 > u32::MAX as u64 {
                self.zip64_entries.lock().unwrap().push(path.clone());
            }
        }

        // Write worksheet parts produced by streaming writers directly from
        // their temporary files without loading them into memory.
        let mut streams: Vec<(String, PathBuf)> = self
            .streams
            .borrow_mut()
            .drain()
            .map(|(path, state)| (path, state.tmp_path))
            .collect();
        streams.sort_unstable_by(|a, b| b.0.cmp(&a.0));
        for (path, tmp_path) in streams {
            let opts = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
            zw.start_file(&path, opts)?;
            let mut file = fs::File::open(&tmp_path)?;
            let size = file.metadata()?.len();
            let mut buf = [0u8; 8192];
            loop {
                let n = file.read(&mut buf)?;
                if n == 0 {
                    break;
                }
                zw.write_all(&buf[..n])?;
            }
            if size > u32::MAX as u64 {
                self.zip64_entries.lock().unwrap().push(path.clone());
            }
            let _ = fs::remove_file(&tmp_path);
        }
        Ok(())
    }

    fn write_zip64_lfh(&self, buf: &mut [u8]) -> Result<()> {
        let entries = self.zip64_entries.lock().unwrap().clone();
        if entries.is_empty() {
            return Ok(());
        }
        let mut offset = 0usize;
        while offset < buf.len() {
            let window = &buf[offset..];
            let Some(idx) = find_subsequence(window, b"\x50\x4b\x03\x04") else {
                break;
            };
            let idx = idx + offset;
            if idx + 30 > buf.len() {
                break;
            }
            let filename_len = u16::from_le_bytes([buf[idx + 26], buf[idx + 27]]) as usize;
            if idx + 30 + filename_len > buf.len() {
                break;
            }
            let filename =
                std::str::from_utf8(&buf[idx + 30..idx + 30 + filename_len]).unwrap_or("");
            if in_str_slice(&entries, filename, true) != -1 {
                buf[idx + 4..idx + 6].copy_from_slice(&45u16.to_le_bytes());
            }
            offset = idx + 1;
        }
        Ok(())
    }

    // Calculation chain writer moved to `calc_chain.rs`.

    fn comments_writer(&self) {
        for entry in self.comments.iter() {
            let path = entry.key().clone();
            let comments = entry.value().clone();
            if let Ok(output) = quick_xml::se::to_string(&comments).map(|s| s.into_bytes()) {
                self.save_file_list(&path, &output);
            }
        }
    }

    fn content_types_writer(&self) {
        if let Some(ct) = self.content_types.lock().unwrap().clone() {
            if let Ok(mut output) = xml_to_string(&ct).map(|s| s.into_bytes()) {
                self.replace_namespace_bytes_if_needed(DEFAULT_XML_PATH_CONTENT_TYPES, &mut output);
                self.save_file_list(DEFAULT_XML_PATH_CONTENT_TYPES, &output);
            }
        }
    }

    fn drawings_writer(&self) {
        for entry in self.drawings.iter() {
            let path = entry.key().clone();
            let drawing = entry.value().clone();
            if let Ok(mut output) = xml_to_string(&drawing).map(|s| s.into_bytes()) {
                self.replace_namespace_bytes_if_needed(&path, &mut output);
                self.save_file_list(&path, &output);
            }
        }
    }

    // Volatile dependencies writer moved to `calc_chain.rs`.

    fn vml_drawing_writer(&self) {
        for entry in self.vml_drawing.iter() {
            let path = entry.key().clone();
            let drawing = entry.value().clone();
            let output = drawing.to_xml();
            self.pkg.insert(path, output);
        }
    }

    fn workbook_writer(&self) {
        if let Some(mut wb) = self.workbook.lock().unwrap().clone() {
            // If the workbook was read with the strict-namespace decoder, move
            // the decoded alternate content back to the serialized element so
            // the compatibility namespace is preserved on save.
            if let Some(decode) = wb.decode_alternate_content.take() {
                wb.alternate_content = Some(crate::xml::workbook::XlsxAlternateContent {
                    xmlns_mc: Some(
                        "http://schemas.openxmlformats.org/markup-compatibility/2006".to_string(),
                    ),
                    content: decode.content,
                });
            }
            if let Ok(mut output) = xml_to_string(&wb).map(|s| s.into_bytes()) {
                let path = self.get_workbook_path();
                self.replace_namespace_bytes_if_needed(&path, &mut output);
                self.save_file_list(&path, &output);
            }
        }
    }

    fn work_sheet_writer(&self) {
        for entry in self.sheet.iter() {
            let path = entry.key().clone();
            let sheet = entry.value().clone();
            if let Ok(mut output) = xml_to_string(&sheet).map(|s| s.into_bytes()) {
                self.replace_namespace_bytes_if_needed(&path, &mut output);
                strip_empty_attributes(&mut output);
                if let Some(ext_lst) = &sheet.ext_lst {
                    let ext_xml = crate::xml::common::serialize_ext_lst(ext_lst);
                    if !ext_xml.is_empty() {
                        inject_ext_lst(&mut output, &ext_xml);
                    }
                }
                self.save_file_list(&path, &output);
            }
        }
    }

    fn rels_writer(&self) {
        for entry in self.relationships.iter() {
            let path = entry.key().clone();
            let rels = entry.value().clone();
            if let Ok(mut output) = xml_to_string(&rels).map(|s| s.into_bytes()) {
                self.replace_namespace_bytes_if_needed(&path, &mut output);
                if !output.windows(6).any(|w| w == b"xmlns=".as_slice()) {
                    let _ = replace_root_namespace_attributes(
                        &mut output,
                        "xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\"",
                    );
                }
                strip_empty_attributes(&mut output);
                self.save_file_list(&path, &output);
            }
        }
    }

    fn shared_strings_loader(&self) -> Result<()> {
        if let Some(entry) = self.temp_files.get(DEFAULT_XML_PATH_SHARED_STRINGS) {
            let temp_path = entry.value().clone();
            drop(entry);
            let data = self.read_bytes(DEFAULT_XML_PATH_SHARED_STRINGS);
            if !data.is_empty() {
                self.pkg
                    .insert(DEFAULT_XML_PATH_SHARED_STRINGS.to_string(), data);
            }
            self.temp_files.remove(DEFAULT_XML_PATH_SHARED_STRINGS);
            let _ = fs::remove_file(temp_path);
            *self.shared_strings.lock().unwrap() = None;
            self.shared_strings_map.lock().unwrap().clear();
        }
        Ok(())
    }

    /// Ensure `[Content_Types].xml` and the workbook relationships reference
    /// the shared string table when it will be written to the package.
    fn shared_strings_registrar(&self) {
        let has_strings = self.shared_strings.lock().unwrap().is_some()
            || self.temp_files.contains_key(DEFAULT_XML_PATH_SHARED_STRINGS);
        if !has_strings {
            return;
        }
        if let Ok(mut ct) = self.content_types_reader() {
            let part_name = format!("/{DEFAULT_XML_PATH_SHARED_STRINGS}");
            let exists = ct.entries.iter().any(|e| {
                matches!(e, crate::xml::content_types::XlsxContentTypeEntry::Override(o) if o.part_name == part_name)
            });
            if !exists {
                ct.entries
                    .push(crate::xml::content_types::XlsxContentTypeEntry::Override(
                        crate::xml::content_types::XlsxOverride {
                            part_name,
                            content_type:
                                crate::constants::CONTENT_TYPE_SPREADSHEET_ML_SHARED_STRINGS
                                    .to_string(),
                        },
                    ));
                *self.content_types.lock().unwrap() = Some(ct);
            }
        }
        let rels_path = self.get_workbook_rels_path();
        let mut rels = self.rels_reader(&rels_path).unwrap_or_default().unwrap_or_default();
        crate::sheet::ensure_shared_strings_rel(&mut rels);
        self.relationships.insert(rels_path, rels);
    }

    fn shared_strings_writer(&self) {
        if let Some(sst) = self.shared_strings.lock().unwrap().clone() {
            if let Ok(output) = xml_to_string(&sst).map(|s| s.into_bytes()) {
                self.save_file_list(DEFAULT_XML_PATH_SHARED_STRINGS, &output);
            }
        }
    }

    fn style_sheet_writer(&self) {
        if let Some(st) = self.styles.lock().unwrap().clone() {
            if let Ok(mut output) = xml_to_string(&st).map(|s| s.into_bytes()) {
                self.replace_namespace_bytes_if_needed(DEFAULT_XML_PATH_STYLES, &mut output);
                strip_empty_attributes(&mut output);
                self.save_file_list(DEFAULT_XML_PATH_STYLES, &output);
            }
        }
    }

    fn theme_writer(&self) {
        if let Some(theme) = self.theme.lock().unwrap().clone() {
            let serialized = decode_theme_to_xlsx_theme(&theme);
            if let Ok(mut output) = xml_to_string(&serialized).map(|s| s.into_bytes()) {
                self.replace_namespace_bytes_if_needed(DEFAULT_XML_PATH_THEME, &mut output);
                self.save_file_list(DEFAULT_XML_PATH_THEME, &output);
            }
        }
    }

    // ------------------------------------------------------------------
    // Namespace helpers
    // ------------------------------------------------------------------

    pub(crate) fn replace_namespace_bytes_if_needed(&self, path: &str, content: &mut Vec<u8>) {
        if let Some(attrs) = self.xml_attr.get(path) {
            let _ = replace_root_namespace_attributes(content, attrs.value());
        }
    }

    /// Register a namespace attribute for the given XML part so that it is
    /// written back when the part is serialized.
    ///
    /// `ns` is the namespace URI; the prefix is looked up from the project's
    /// namespace dictionary. For known extension namespaces the `mc:Ignorable`
    /// attribute is also updated so Excel compatibility markup stays valid.
    pub fn add_name_spaces(&self, path: &str, ns: &str) {
        let prefix = match ns {
            crate::constants::SOURCE_RELATIONSHIP => "r",
            crate::constants::NAMESPACE_SPREADSHEET_X14 => "x14",
            crate::constants::NAMESPACE_SPREADSHEET_X15 => "x15",
            crate::constants::NAMESPACE_SPREADSHEET_EXCEL_2006_MAIN => "xm",
            crate::constants::NAMESPACE_DRAWING_ML_MAIN => "a",
            crate::constants::NAMESPACE_DRAWING_ML_CHART => "c",
            crate::constants::NAMESPACE_DRAWING_ML_SPREADSHEET => "xdr",
            crate::constants::NAMESPACE_DRAWING_ML_A14 => "a14",
            crate::constants::NAMESPACE_DRAWING_2016_SVG => "asvg",
            // Default spreadsheet namespace is already present on worksheet/workbook roots.
            crate::constants::NAMESPACE_SPREADSHEET => return,
            _ => return,
        };
        if prefix.is_empty() {
            return;
        }

        let attr_name = format!("xmlns:{prefix}");
        let mut attrs = self
            .xml_attr
            .get(path)
            .map(|a| a.clone())
            .unwrap_or_default();
        if !attrs.contains(&attr_name) {
            if !attrs.is_empty() {
                attrs.push(' ');
            }
            attrs.push_str(&format!("{attr_name}=\"{ns}\""));
        }

        // Ensure the markup-compatibility namespace is present and mark the
        // extension namespace as ignorable when appropriate.
        if self.needs_ignorable_prefix(prefix) {
            let mc_ns = "http://schemas.openxmlformats.org/markup-compatibility/2006";
            if !attrs.contains("xmlns:mc") {
                if !attrs.is_empty() && !attrs.ends_with(' ') {
                    attrs.push(' ');
                }
                attrs.push_str(&format!("xmlns:mc=\"{mc_ns}\""));
            }
            self.set_ignorable_name_space(path, prefix, &mut attrs);
        }

        self.xml_attr.insert(path.to_string(), attrs);
    }

    fn needs_ignorable_prefix(&self, prefix: &str) -> bool {
        const IGNORABLE_NS: &[&str] = &[
            "c14", "cdr14", "a14", "pic14", "x14", "xdr14", "x14ac", "dsp", "mso14", "dgm14",
            "x15", "x12ac", "x15ac", "xr", "xr2", "xr3", "xr4", "xr5", "xr6", "xr7", "xr8", "xr9",
            "xr10", "xr11", "xr12", "xr13", "xr14", "xr15", "x16", "x16r2", "mo", "mx", "mv", "o",
            "v",
        ];
        IGNORABLE_NS.contains(&prefix)
    }

    fn set_ignorable_name_space(&self, _path: &str, prefix: &str, attrs: &mut String) {
        let marker = "mc:Ignorable=\"";
        if let Some(start) = attrs.find(marker) {
            let value_start = start + marker.len();
            if let Some(end) = attrs[value_start..].find('"') {
                let value = &attrs[value_start..value_start + end];
                let parts: Vec<&str> = value.split_whitespace().collect();
                if !parts.contains(&prefix) {
                    let new_value = format!("{} {}", value, prefix);
                    attrs.replace_range(value_start..value_start + end, &new_value);
                }
                return;
            }
        }
        if !attrs.is_empty() && !attrs.ends_with(' ') {
            attrs.push(' ');
        }
        attrs.push_str(&format!("mc:Ignorable=\"{prefix}\""));
    }

    /// Register a namespace attribute for a worksheet XML part.
    pub(crate) fn add_sheet_name_space(&self, sheet: &str, ns: &str) {
        if let Some(path) = self.get_sheet_xml_path(sheet) {
            self.add_name_spaces(&path, ns);
        }
    }

    /// Return the sheet ID (1-based) for a worksheet name.
    pub fn get_sheet_id(&self, sheet: &str) -> i32 {
        if let Ok(wb) = self.workbook_reader() {
            for s in &wb.sheets.sheet {
                if s.name.as_deref().unwrap_or("").eq_ignore_ascii_case(sheet) {
                    return s.sheet_id.unwrap_or(0) as i32;
                }
            }
        }
        -1
    }

    /// Expand a 3D sheet range (e.g. `Sheet1:Sheet3`) into the ordered list of
    /// worksheet names between the two sheets inclusive.
    pub fn expand_3d_sheet_range(&self, sheet1: &str, sheet2: &str) -> Result<Vec<String>> {
        let mut idx1 = self.get_sheet_index(sheet1)?;
        let mut idx2 = self.get_sheet_index(sheet2)?;
        if idx1 > idx2 {
            std::mem::swap(&mut idx1, &mut idx2);
        }
        let list = self.get_sheet_list();
        Ok(list[(idx1 - 1) as usize..idx2 as usize].to_vec())
    }

    /// Clear the in-memory calc cache so the chain is rewritten on save.
    pub fn clear_calc_cache(&self) {
        let _ = self.calc_chain.lock().unwrap().take();
        self.calc_cache.lock().unwrap().clear();
        self.calc_raw_cache.lock().unwrap().clear();
        self.formula_arg_cache.lock().unwrap().clear();
    }

    /// Add a content type override for a numbered part.
    pub fn add_content_type_part(&self, id: i32, part: &str) -> Result<()> {
        match part {
            "comments" => self.set_content_type_part_vml_extensions()?,
            "drawings" => crate::sheet::set_content_type_part_image_extensions(self)?,
            _ => {}
        }
        let mut ct = self.content_types_reader()?;
        let content_type = match part {
            "table" => crate::constants::CONTENT_TYPE_SPREADSHEET_ML_TABLE,
            "pivotTable" => crate::constants::CONTENT_TYPE_SPREADSHEET_ML_PIVOT_TABLE,
            "pivotCache" => crate::constants::CONTENT_TYPE_SPREADSHEET_ML_PIVOT_CACHE_DEFINITION,
            "slicer" => crate::constants::CONTENT_TYPE_SLICER,
            "slicerCache" => crate::constants::CONTENT_TYPE_SLICER_CACHE,
            "drawings" => crate::constants::CONTENT_TYPE_DRAWING,
            "chart" => crate::constants::CONTENT_TYPE_DRAWING_ML,
            "chartsheet" => crate::constants::CONTENT_TYPE_SPREADSHEET_ML_CHARTSHEET,
            "comments" => crate::constants::CONTENT_TYPE_SPREADSHEET_ML_COMMENTS,
            _ => return Ok(()),
        };
        let part_name = match part {
            "table" => format!("/xl/tables/table{id}.xml"),
            "pivotTable" => format!("/xl/pivotTables/pivotTable{id}.xml"),
            "pivotCache" => format!("/xl/pivotCache/pivotCacheDefinition{id}.xml"),
            "slicer" => format!("/xl/slicers/slicer{id}.xml"),
            "slicerCache" => format!("/xl/slicerCaches/slicerCache{id}.xml"),
            "drawings" => format!("/xl/drawings/drawing{id}.xml"),
            "chart" => format!("/xl/charts/chart{id}.xml"),
            "chartsheet" => format!("/xl/chartsheets/sheet{id}.xml"),
            "comments" => format!("/xl/comments{id}.xml"),
            _ => return Ok(()),
        };
        for entry in &ct.entries {
            if let crate::xml::content_types::XlsxContentTypeEntry::Override(o) = entry {
                if o.part_name == part_name {
                    return Ok(());
                }
            }
        }
        ct.entries
            .push(crate::xml::content_types::XlsxContentTypeEntry::Override(
                crate::xml::content_types::XlsxOverride {
                    part_name,
                    content_type: content_type.to_string(),
                },
            ));
        *self.content_types.lock().unwrap() = Some(ct);
        self.set_content_type_part_rels_extensions()
    }

    /// Ensure `[Content_Types].xml` contains the default relationship content type.
    pub(crate) fn set_content_type_part_rels_extensions(&self) -> Result<()> {
        let mut ct = self.content_types_reader()?;
        let exists = ct.entries.iter().any(|e| {
            if let crate::xml::content_types::XlsxContentTypeEntry::Default(d) = e {
                d.extension == "rels"
            } else {
                false
            }
        });
        if !exists {
            ct.entries
                .push(crate::xml::content_types::XlsxContentTypeEntry::Default(
                    crate::xml::content_types::XlsxDefault {
                        extension: "rels".to_string(),
                        content_type: crate::constants::CONTENT_TYPE_RELATIONSHIPS.to_string(),
                    },
                ));
        }
        *self.content_types.lock().unwrap() = Some(ct);
        Ok(())
    }

    /// Ensure `[Content_Types].xml` contains the default VML content type.
    pub(crate) fn set_content_type_part_vml_extensions(&self) -> Result<()> {
        let mut ct = self.content_types_reader()?;
        let exists = ct.entries.iter().any(|e| {
            if let crate::xml::content_types::XlsxContentTypeEntry::Default(d) = e {
                d.extension == "vml"
            } else {
                false
            }
        });
        if !exists {
            ct.entries
                .push(crate::xml::content_types::XlsxContentTypeEntry::Default(
                    crate::xml::content_types::XlsxDefault {
                        extension: "vml".to_string(),
                        content_type: crate::constants::CONTENT_TYPE_VML.to_string(),
                    },
                ));
        }
        *self.content_types.lock().unwrap() = Some(ct);
        Ok(())
    }

    /// Remove a content type override by content type and part name prefix.
    pub fn remove_content_types_part(&self, content_type: &str, part_name: &str) -> Result<()> {
        let mut ct = self.content_types_reader()?;
        ct.entries.retain(|e| {
            if let crate::xml::content_types::XlsxContentTypeEntry::Override(o) = e {
                !(o.content_type == content_type && o.part_name == part_name)
            } else {
                true
            }
        });
        *self.content_types.lock().unwrap() = Some(ct);
        Ok(())
    }

    /// Return the relationship target for a worksheet relationship by rId.
    pub fn get_sheet_relationships_target_by_id(&self, sheet: &str, r_id: &str) -> String {
        let Some(sheet_xml_path) = self.get_sheet_xml_path(sheet) else {
            return String::new();
        };
        let sheet_rels = format!(
            "xl/worksheets/_rels/{}.rels",
            sheet_xml_path.trim_start_matches("xl/worksheets/")
        );
        if let Ok(Some(rels)) = self.rels_reader(&sheet_rels) {
            for rel in &rels.relationships {
                if rel.id == r_id {
                    return rel.target.clone();
                }
            }
        }
        String::new()
    }

    /// Add a legacy drawing reference to a worksheet.
    pub fn add_sheet_legacy_drawing(&self, sheet: &str, r_id: i32) -> Result<()> {
        let mut ws = self.work_sheet_reader(sheet)?;
        ws.legacy_drawing = Some(crate::xml::worksheet::XlsxLegacyDrawing {
            rid: Some(format!("rId{r_id}")),
        });
        if let Some(path) = self.get_sheet_xml_path(sheet) {
            self.sheet.insert(path, ws);
        }
        Ok(())
    }

    /// Add a legacy header/footer drawing reference to a worksheet.
    pub fn add_sheet_legacy_drawing_hf(&self, sheet: &str, r_id: i32) -> Result<()> {
        let mut ws = self.work_sheet_reader(sheet)?;
        ws.legacy_drawing_hf = Some(crate::xml::worksheet::XlsxLegacyDrawingHF {
            rid: Some(format!("rId{r_id}")),
        });
        if let Some(path) = self.get_sheet_xml_path(sheet) {
            self.sheet.insert(path, ws);
        }
        Ok(())
    }

    /// Count existing comments parts in the package.
    pub fn count_comments(&self) -> i32 {
        let mut count = 0;
        for entry in self.pkg.iter() {
            if entry.key().contains("xl/comments") {
                count += 1;
            }
        }
        for entry in self.comments.iter() {
            if entry.key().contains("xl/comments") && !self.pkg.contains_key(entry.key()) {
                count += 1;
            }
        }
        count
    }

    /// Count existing VML drawing parts in the package.
    pub fn count_vml_drawing(&self) -> i32 {
        let mut count = 0;
        for entry in self.pkg.iter() {
            if entry.key().contains("xl/drawings/vmlDrawing") {
                count += 1;
            }
        }
        for entry in self.vml_drawing.iter() {
            if entry.key().contains("xl/drawings/vmlDrawing") && !self.pkg.contains_key(entry.key())
            {
                count += 1;
            }
        }
        count
    }

    /// Lazy reader for comments parts.
    pub fn comments_reader(
        &self,
        path: &str,
    ) -> Result<Option<crate::xml::comments::XlsxComments>> {
        if let Some(cmts) = self.comments.get(path) {
            return Ok(Some(cmts.clone()));
        }
        if self.pkg.contains_key(path) || self.temp_files.contains_key(path) {
            let data = self.apply_charset_transcoder(&self.read_bytes(path))?;
            let data = crate::file::namespace_strict_to_transitional(&data);
            let mut cmts: crate::xml::comments::XlsxComments =
                quick_xml::de::from_reader(data.as_slice()).unwrap_or_default();
            for cmt in &cmts.comment_list.comment {
                cmts.cells.push(cmt.r#ref.clone());
            }
            self.comments.insert(path.to_string(), cmts.clone());
            return Ok(Some(cmts));
        }
        Ok(None)
    }

    /// Lazy reader for VML drawing parts.
    pub fn vml_drawing_reader(&self, path: &str) -> Result<Option<crate::xml::vml::VmlDrawing>> {
        if let Some(vml) = self.vml_drawing.get(path) {
            return Ok(Some(vml.clone()));
        }
        if self.pkg.contains_key(path) || self.temp_files.contains_key(path) {
            let data = self.apply_charset_transcoder(&self.read_bytes(path))?;
            let data = crate::file::namespace_strict_to_transitional(&data);
            let data = String::from_utf8_lossy(&data)
                .replace("<br>\r\n", "<br></br>\r\n")
                .into_bytes();
            let vml = crate::xml::vml::VmlDrawing::from_xml(&data).unwrap_or_default();
            self.vml_drawing.insert(path.to_string(), vml.clone());
            return Ok(Some(vml));
        }
        Ok(None)
    }

    /// Delete a worksheet relationship by rId.
    pub fn delete_sheet_relationships(&self, sheet: &str, r_id: &str) {
        let Some(sheet_xml_path) = self.get_sheet_xml_path(sheet) else {
            return;
        };
        let sheet_rels = format!(
            "xl/worksheets/_rels/{}.rels",
            sheet_xml_path.trim_start_matches("xl/worksheets/")
        );
        if let Ok(Some(mut rels)) = self.rels_reader(&sheet_rels) {
            rels.relationships.retain(|r| r.id != r_id);
            self.relationships.insert(sheet_rels, rels);
        }
    }

    /// Delete a workbook relationship by type and target, returning its rId.
    pub fn delete_workbook_rels(&self, rel_type: &str, target: &str) -> Result<String> {
        let rel_path = self.get_workbook_rels_path();
        let mut rels = self.rels_reader(&rel_path)?.unwrap_or_default();
        let mut r_id = String::new();
        rels.relationships.retain(|r| {
            if r.r#type == rel_type && (r.target == target || r.target == format!("/{target}")) {
                r_id = r.id.clone();
                false
            } else {
                true
            }
        });
        self.relationships.insert(rel_path, rels);
        Ok(r_id)
    }

    /// Add a drawing relationship reference to a worksheet.
    pub fn add_sheet_drawing(&self, sheet: &str, r_id: i32) -> Result<()> {
        let mut ws = self.work_sheet_reader(sheet)?;
        ws.drawing = Some(crate::xml::worksheet::XlsxDrawing {
            rid: Some(format!("rId{r_id}")),
        });
        self.sheet
            .insert(self.get_sheet_xml_path(sheet).unwrap_or_default(), ws);
        Ok(())
    }

    /// Count existing table parts in the package.
    pub fn count_tables(&self) -> i32 {
        let mut count = 0i32;
        for entry in self.pkg.iter() {
            let k = entry.key();
            if k.contains("xl/tables/tableSingleCells") {
                if let Ok(data) = self.apply_charset_transcoder(entry.value()) {
                    let data = namespace_strict_to_transitional(&data);
                    match xml_from_reader::<_, XlsxSingleXmlCells>(data.as_slice()) {
                        Ok(cells) => {
                            for cell in cells.single_xml_cell {
                                if count < cell.id as i32 {
                                    count = cell.id as i32;
                                }
                            }
                        }
                        Err(_) => count += 1,
                    }
                }
            }
            if k.contains("xl/tables/table") {
                if let Ok(data) = self.apply_charset_transcoder(entry.value()) {
                    let data = namespace_strict_to_transitional(&data);
                    match xml_from_reader::<_, XlsxTable>(data.as_slice()) {
                        Ok(t) => {
                            if count < t.id as i32 {
                                count = t.id as i32;
                            }
                        }
                        Err(_) => count += 1,
                    }
                }
            }
        }
        count
    }

    /// Count existing drawing parts in the package.
    pub fn count_drawings(&self) -> i32 {
        let mut count = 0;
        for entry in self.pkg.iter() {
            if entry.key().contains("xl/drawings/drawing") {
                count += 1;
            }
        }
        for entry in self.drawings.iter() {
            if entry.key().contains("xl/drawings/drawing") && !self.pkg.contains_key(entry.key()) {
                count += 1;
            }
        }
        count
    }

    /// Count existing chart parts in the package.
    pub fn count_charts(&self) -> i32 {
        let mut count = 0;
        for entry in self.pkg.iter() {
            if entry.key().contains("xl/charts/chart") {
                count += 1;
            }
        }
        count
    }

    /// Count existing pivot table parts in the package.
    pub fn count_pivot_tables(&self) -> i32 {
        let mut count = 0;
        for entry in self.pkg.iter() {
            if entry.key().contains("xl/pivotTables/pivotTable") {
                count += 1;
            }
        }
        count
    }

    /// Count existing pivot cache parts in the package.
    pub fn count_pivot_cache(&self) -> i32 {
        let mut count = 0;
        for entry in self.pkg.iter() {
            if entry.key().contains("xl/pivotCache/pivotCacheDefinition") {
                count += 1;
            }
        }
        count
    }

    /// Count existing slicer parts in the package.
    pub fn count_slicers(&self) -> i32 {
        let mut count = 0;
        for entry in self.pkg.iter() {
            if entry.key().contains("xl/slicers/slicer") {
                count += 1;
            }
        }
        count
    }

    /// Count existing slicer cache parts in the package.
    pub fn count_slicer_cache(&self) -> i32 {
        let mut count = 0;
        for entry in self.pkg.iter() {
            if entry.key().contains("xl/slicerCaches/slicerCache") {
                count += 1;
            }
        }
        count
    }

    /// Return all defined names in the workbook.
    pub fn get_defined_names(&self) -> Result<Vec<crate::xml::workbook::DefinedName>> {
        let mut out = Vec::new();
        if let Ok(wb) = self.workbook_reader() {
            if let Some(dns) = wb.defined_names {
                for dn in dns.defined_name {
                    out.push(crate::xml::workbook::DefinedName {
                        name: dn.name.clone().unwrap_or_default(),
                        comment: dn.comment.clone().unwrap_or_default(),
                        refers_to: dn.data.clone(),
                        scope: dn
                            .local_sheet_id
                            .map(|id| {
                                if let Ok(wb2) = self.workbook_reader() {
                                    if let Some(s) = wb2.sheets.sheet.get(id as usize) {
                                        return s.name.clone().unwrap_or_default();
                                    }
                                }
                                String::new()
                            })
                            .unwrap_or_else(|| "Workbook".to_string()),
                    });
                }
            }
        }
        Ok(out)
    }

    /// Add a defined name.
    pub fn set_defined_name(&self, dn: &crate::xml::workbook::DefinedName) -> Result<()> {
        let mut wb = self.workbook_reader()?;
        if wb.defined_names.is_none() {
            wb.defined_names = Some(crate::xml::workbook::XlsxDefinedNames::default());
        }
        let names = wb.defined_names.as_mut().unwrap();
        let local_sheet_id = if dn.scope.is_empty() || dn.scope == "Workbook" {
            None
        } else {
            Some((self.get_sheet_index(&dn.scope)? - 1) as i64)
        };
        for existing in &names.defined_name {
            if existing.name.as_deref().unwrap_or("") == dn.name
                && existing.local_sheet_id == local_sheet_id
            {
                return Err(Box::new(ErrDefinedNameDuplicate));
            }
        }
        names
            .defined_name
            .push(crate::xml::workbook::XlsxDefinedName {
                name: Some(dn.name.clone()),
                data: dn.refers_to.clone(),
                hidden: Some(false),
                local_sheet_id,
                ..Default::default()
            });
        *self.workbook.lock().unwrap() = Some(wb);
        Ok(())
    }

    /// Delete a defined name.
    pub fn delete_defined_name(&self, dn: &crate::xml::workbook::DefinedName) -> Result<()> {
        let mut wb = self.workbook_reader()?;
        let Some(names) = wb.defined_names.as_mut() else {
            return Err(Box::new(ErrDefinedNameScope));
        };
        let local_sheet_id = if dn.scope.is_empty() || dn.scope == "Workbook" {
            None
        } else {
            Some((self.get_sheet_index(&dn.scope)? - 1) as i64)
        };
        let before = names.defined_name.len();
        names.defined_name.retain(|e| {
            !(e.name.as_deref().unwrap_or("") == dn.name && e.local_sheet_id == local_sheet_id)
        });
        if names.defined_name.len() == before {
            return Err(Box::new(ErrDefinedNameScope));
        }
        if names.defined_name.is_empty() {
            wb.defined_names = None;
        }
        *self.workbook.lock().unwrap() = Some(wb);
        Ok(())
    }

    /// Return the reference a defined name resolves to.
    pub fn get_defined_name_ref_to(&self, name: &str, current_sheet: &str) -> String {
        let mut workbook_ref = String::new();
        let mut sheet_ref = String::new();
        if let Ok(names) = self.get_defined_names() {
            for dn in &names {
                if dn.name == name {
                    if dn.scope == "Workbook" {
                        workbook_ref = dn.refers_to.clone();
                    }
                    if dn.scope == current_sheet {
                        sheet_ref = dn.refers_to.clone();
                    }
                }
            }
        }
        if !sheet_ref.is_empty() {
            sheet_ref
        } else {
            workbook_ref
        }
    }

    /// Alias for [`Self::get_defined_names`], matching the Go `GetDefinedName`
    /// API surface.
    pub fn get_defined_name(&self) -> Result<Vec<crate::xml::workbook::DefinedName>> {
        self.get_defined_names()
    }
}

// ------------------------------------------------------------------
// Additional public File-level APIs ported from file.go / excelize.go
// ------------------------------------------------------------------

impl File {
    /// Write the workbook to any writer. Alias for [`Self::write_to`].
    pub fn write<W: Write>(&self, writer: W) -> Result<()> {
        self.write_to(writer)
    }

    /// Save to the original path, overriding the stored options.
    pub fn save_with_options(&mut self, opts: Options) -> Result<()> {
        {
            let mut o = self.options.lock().unwrap();
            *o = opts;
        }
        self.save()
    }

    /// Save the workbook to the given path, overriding the stored options.
    pub fn save_as_with_options(&mut self, path: &str, opts: Options) -> Result<()> {
        {
            let mut o = self.options.lock().unwrap();
            *o = opts;
        }
        self.save_as(path)
    }

    /// Clear cached linked values for formula cells so Excel recalculates them
    /// on open.
    ///
    /// Equivalent to Go `UpdateLinkedValue`.
    pub fn update_linked_value(&self) -> Result<()> {
        let mut wb = self.workbook_reader()?;
        wb.calc_pr = None;
        *self.workbook.lock().unwrap() = Some(wb);
        for name in self.get_sheet_list() {
            if let Ok(mut ws) = self.work_sheet_reader(&name) {
                let path = self.get_sheet_xml_path(&name).unwrap_or_default();
                let mut changed = false;
                for row in &mut ws.sheet_data.row {
                    for cell in &mut row.c {
                        if cell.f.is_some() && cell.v.is_some() {
                            cell.v = None;
                            cell.t = None;
                            changed = true;
                        }
                    }
                }
                if changed {
                    self.sheet.insert(path, ws);
                }
            }
        }
        Ok(())
    }

    /// Extract spreadsheet package parts from a `ZipArchive`.
    ///
    /// Equivalent to Go `ReadZipReader`.
    pub fn read_zip_reader<R: Read + Seek>(
        &self,
        archive: &mut ZipArchive<R>,
    ) -> Result<(HashMap<String, Vec<u8>>, i32)> {
        self.read_zip_archive(archive)
    }

    /// Set workbook properties.
    pub fn set_workbook_props(&self, opts: &WorkbookPropsOptions) -> Result<()> {
        let mut wb = self.workbook_reader()?;
        if wb.workbook_pr.is_none() {
            wb.workbook_pr = Some(XlsxWorkbookPr::default());
        }
        let pr = wb.workbook_pr.as_mut().unwrap();
        if let Some(v) = opts.date1904 {
            pr.date1904 = Some(v);
        }
        if let Some(v) = opts.filter_privacy {
            pr.filter_privacy = Some(v);
        }
        if let Some(ref v) = opts.code_name {
            pr.code_name = Some(v.clone());
        }
        *self.workbook.lock().unwrap() = Some(wb);
        Ok(())
    }

    /// Get workbook properties.
    pub fn get_workbook_props(&self) -> Result<WorkbookPropsOptions> {
        let mut opts = WorkbookPropsOptions::default();
        let wb = self.workbook_reader()?;
        if let Some(pr) = &wb.workbook_pr {
            opts.date1904 = pr.date1904;
            opts.filter_privacy = pr.filter_privacy;
            opts.code_name = pr.code_name.clone();
        }
        Ok(opts)
    }

    /// Set calculation properties.
    pub fn set_calc_props(&self, opts: &CalcPropsOptions) -> Result<()> {
        if let Some(ref mode) = opts.calc_mode {
            if in_str_slice(SUPPORTED_CALC_MODE, mode, true) == -1 {
                return Err(Box::new(ErrParameterInvalid));
            }
        }
        if let Some(ref mode) = opts.ref_mode {
            if in_str_slice(SUPPORTED_REF_MODE, mode, true) == -1 {
                return Err(Box::new(ErrParameterInvalid));
            }
        }

        let mut wb = self.workbook_reader()?;
        if wb.calc_pr.is_none() {
            wb.calc_pr = Some(XlsxCalcPr::default());
        }
        let pr = wb.calc_pr.as_mut().unwrap();
        if let Some(v) = opts.calc_completed {
            pr.calc_completed = Some(v);
        }
        if let Some(v) = opts.calc_on_save {
            pr.calc_on_save = Some(v);
        }
        if let Some(v) = opts.force_full_calc {
            pr.force_full_calc = Some(v);
        }
        if let Some(v) = opts.full_calc_on_load {
            pr.full_calc_on_load = Some(v);
        }
        if let Some(v) = opts.full_precision {
            pr.full_precision = Some(v);
        }
        if let Some(v) = opts.iterate {
            pr.iterate = Some(v);
        }
        if let Some(v) = opts.iterate_delta {
            pr.iterate_delta = Some(v);
        }
        if let Some(ref v) = opts.calc_mode {
            pr.calc_mode = Some(v.clone());
        }
        if let Some(ref v) = opts.ref_mode {
            pr.ref_mode = Some(v.clone());
        }
        if let Some(v) = opts.calc_id {
            pr.calc_id = Some(v as i64);
        }
        if let Some(v) = opts.concurrent_manual_count {
            pr.concurrent_manual_count = Some(v as i64);
        }
        if let Some(v) = opts.iterate_count {
            pr.iterate_count = Some(v as i64);
        }
        pr.concurrent_calc = opts.concurrent_calc;
        *self.workbook.lock().unwrap() = Some(wb);
        Ok(())
    }

    /// Get calculation properties.
    pub fn get_calc_props(&self) -> Result<CalcPropsOptions> {
        let mut opts = CalcPropsOptions::default();
        let wb = self.workbook_reader()?;
        if let Some(pr) = &wb.calc_pr {
            opts.calc_completed = pr.calc_completed;
            opts.calc_on_save = pr.calc_on_save;
            opts.force_full_calc = pr.force_full_calc;
            opts.full_calc_on_load = pr.full_calc_on_load;
            opts.full_precision = pr.full_precision;
            opts.iterate = pr.iterate;
            opts.iterate_delta = pr.iterate_delta;
            opts.calc_mode = pr.calc_mode.clone();
            opts.ref_mode = pr.ref_mode.clone();
            opts.calc_id = pr.calc_id.map(|v| v as u64);
            opts.concurrent_manual_count = pr.concurrent_manual_count.map(|v| v as u64);
            opts.iterate_count = pr.iterate_count.map(|v| v as u64);
            opts.concurrent_calc = pr.concurrent_calc;
        }
        Ok(opts)
    }

    /// Protect the workbook with optional password and lock settings.
    pub fn protect_workbook(&self, opts: &WorkbookProtectionOptions) -> Result<()> {
        let mut wb = self.workbook_reader()?;
        let mut protection = XlsxWorkbookProtection {
            lock_structure: Some(opts.lock_structure),
            lock_windows: Some(opts.lock_windows),
            ..Default::default()
        };
        if !opts.password.is_empty() {
            let algorithm_name = if opts.algorithm_name.is_empty() {
                "SHA-512"
            } else {
                &opts.algorithm_name
            };
            let (hash_value, salt_value) = crypt::gen_iso_passwd_hash(
                &opts.password,
                algorithm_name,
                "",
                WORKBOOK_PROTECTION_SPIN_COUNT,
            )?;
            protection.workbook_algorithm_name = Some(algorithm_name.to_string());
            protection.workbook_hash_value = Some(hash_value);
            protection.workbook_salt_value = Some(salt_value);
            protection.workbook_spin_count = Some(WORKBOOK_PROTECTION_SPIN_COUNT as i64);
        }
        wb.workbook_protection = Some(protection);
        *self.workbook.lock().unwrap() = Some(wb);
        Ok(())
    }

    /// Remove workbook protection, optionally verifying the password first.
    pub fn unprotect_workbook(&self, password: Option<&str>) -> Result<()> {
        let mut wb = self.workbook_reader()?;
        if let Some(pwd) = password {
            let protection = wb.workbook_protection.as_ref().ok_or_else(|| {
                Box::new(ErrUnprotectWorkbook) as Box<dyn std::error::Error + Send + Sync>
            })?;
            if let Some(ref algorithm_name) = protection.workbook_algorithm_name {
                let salt = protection.workbook_salt_value.as_deref().unwrap_or("");
                let spin_count = protection.workbook_spin_count.unwrap_or(0) as i32;
                let (hash_value, _) =
                    crypt::gen_iso_passwd_hash(pwd, algorithm_name, salt, spin_count)?;
                if protection.workbook_hash_value.as_deref().unwrap_or("") != hash_value {
                    return Err(Box::new(ErrUnprotectWorkbookPassword));
                }
            }
        }
        wb.workbook_protection = None;
        *self.workbook.lock().unwrap() = Some(wb);
        Ok(())
    }

    /// Add a VBA project binary to the workbook.
    ///
    /// The data must start with the OLE compound-file identifier. A valid
    /// `vbaProject.bin` can be embedded by reading it from disk and passing
    /// the bytes to this method. The workbook should be saved with an `.xlsm`
    /// or `.xltm` extension.
    pub fn add_vba_project(&self, data: &[u8]) -> Result<()> {
        if data.len() < 8 || &data[..8] != OLE_IDENTIFIER {
            return Err(Box::new(crate::errors::ErrAddVBAProject));
        }
        let rel_path = self.get_workbook_rels_path();
        let mut rels = self.rels_reader(&rel_path)?.unwrap_or_default();
        let mut existing = false;
        let mut r_id = 0;
        for rel in &rels.relationships {
            if rel.target == "vbaProject.bin" && rel.r#type == SOURCE_RELATIONSHIP_VBA_PROJECT {
                existing = true;
            }
            let id: i32 = rel.id.trim_start_matches("rId").parse().unwrap_or(0);
            if id > r_id {
                r_id = id;
            }
        }
        if !existing {
            r_id += 1;
            rels.relationships.push(XlsxRelationship {
                id: format!("rId{r_id}"),
                r#type: SOURCE_RELATIONSHIP_VBA_PROJECT.to_string(),
                target: "vbaProject.bin".to_string(),
                target_mode: None,
            });
            self.relationships.insert(rel_path, rels);
        }
        self.pkg
            .insert("xl/vbaProject.bin".to_string(), data.to_vec());
        Ok(())
    }

    /// Set the workbook content type and `.bin` default for macro workbooks.
    pub fn set_content_type_part_project_extensions(&self, content_type: &str) -> Result<()> {
        let mut ct = self.content_types_reader()?;
        let mut bin_ok = false;
        let mut entries = ct.entries.clone();
        for entry in &entries {
            if let crate::xml::content_types::XlsxContentTypeEntry::Default(d) = entry {
                if d.extension == "bin" {
                    bin_ok = true;
                }
            }
        }
        for entry in &mut entries {
            if let crate::xml::content_types::XlsxContentTypeEntry::Override(o) = entry {
                if o.part_name == "/xl/workbook.xml" {
                    o.content_type = content_type.to_string();
                }
            }
        }
        if !bin_ok {
            entries.push(crate::xml::content_types::XlsxContentTypeEntry::Default(
                XlsxDefault {
                    extension: "bin".to_string(),
                    content_type: CONTENT_TYPE_VBA.to_string(),
                },
            ));
        }
        ct.entries = entries;
        *self.content_types.lock().unwrap() = Some(ct);
        Ok(())
    }
}

// ------------------------------------------------------------------
// Free functions
// ------------------------------------------------------------------

fn normalize_options(mut opts: Options) -> Options {
    if opts.unzip_size_limit == 0 {
        opts.unzip_size_limit = UNZIP_SIZE_LIMIT;
    }
    if opts.unzip_xml_size_limit == 0 {
        opts.unzip_xml_size_limit = STREAM_CHUNK_SIZE;
    }
    if opts.culture_info == 0 && opts.short_date_pattern.is_empty() {
        opts.culture_info = CULTURE_NAME_UNKNOWN;
    }
    opts
}

/// Convert Strict Open XML namespaces to Transitional ones.
pub fn namespace_strict_to_transitional(content: &[u8]) -> Vec<u8> {
    if content.windows(13).all(|w| w != b"purl.oclc.org") {
        return content.to_vec();
    }
    let mut result = content.to_vec();
    let translations: &[(&[u8], &[u8])] = &[
        (
            STRICT_NAMESPACE_DOCUMENT_PROPERTIES_VARIANT_TYPES.as_bytes(),
            NAMESPACE_DOCUMENT_PROPERTIES_VARIANT_TYPES.as_bytes(),
        ),
        (
            STRICT_NAMESPACE_DRAWING_ML_MAIN.as_bytes(),
            NAMESPACE_DRAWING_ML_MAIN.as_bytes(),
        ),
        (
            STRICT_NAMESPACE_EXTENDED_PROPERTIES.as_bytes(),
            NAMESPACE_EXTENDED_PROPERTIES.as_bytes(),
        ),
        (
            STRICT_NAMESPACE_SPREADSHEET.as_bytes(),
            NAMESPACE_SPREADSHEET.as_bytes(),
        ),
        (
            STRICT_SOURCE_RELATIONSHIP.as_bytes(),
            SOURCE_RELATIONSHIP.as_bytes(),
        ),
        (
            STRICT_SOURCE_RELATIONSHIP_CHART.as_bytes(),
            SOURCE_RELATIONSHIP_CHART.as_bytes(),
        ),
        (
            STRICT_SOURCE_RELATIONSHIP_COMMENTS.as_bytes(),
            SOURCE_RELATIONSHIP_COMMENTS.as_bytes(),
        ),
        (
            STRICT_SOURCE_RELATIONSHIP_EXTEND_PROPERTIES.as_bytes(),
            SOURCE_RELATIONSHIP_EXTEND_PROPERTIES.as_bytes(),
        ),
        (
            STRICT_SOURCE_RELATIONSHIP_IMAGE.as_bytes(),
            SOURCE_RELATIONSHIP_IMAGE.as_bytes(),
        ),
        (
            STRICT_SOURCE_RELATIONSHIP_OFFICE_DOCUMENT.as_bytes(),
            SOURCE_RELATIONSHIP_OFFICE_DOCUMENT.as_bytes(),
        ),
    ];
    for (from, to) in translations {
        result = bytes_replace(&result, from, to);
    }
    result
}

fn bytes_replace(content: &[u8], from: &[u8], to: &[u8]) -> Vec<u8> {
    let mut result = Vec::with_capacity(content.len());
    let mut start = 0;
    while let Some(pos) = content[start..].windows(from.len()).position(|w| w == from) {
        let pos = start + pos;
        result.extend_from_slice(&content[start..pos]);
        result.extend_from_slice(to);
        start = pos + from.len();
    }
    result.extend_from_slice(&content[start..]);
    result
}

/// Detect the encoding named in an XML declaration, returning `None` when no
/// declaration is present. Works on raw bytes so that non-UTF-8 documents can
/// still be inspected.
fn detect_xml_encoding(content: &[u8]) -> Option<&str> {
    let start = content.windows(5).position(|w| w == b"<?xml")?;
    let rest = &content[start..];
    let end = rest.windows(2).position(|w| w == b"?>")? + 2;
    let decl = &rest[..end];
    let key = b"encoding=";
    let pos = decl.windows(key.len()).position(|w| w == key)?;
    let rest = &decl[pos + key.len()..];
    let quote = *rest.first()?;
    let close = rest[1..].iter().position(|&b| b == quote)?;
    std::str::from_utf8(&rest[1..1 + close]).ok()
}

/// Remove all occurrences of a single XML element (and its children) from a
/// UTF-8 document. Used as a deserialization workaround for elements that
/// contain arbitrary nested XML.
fn strip_xml_element(content: &[u8], name: &str) -> Vec<u8> {
    let s = String::from_utf8_lossy(content);
    let pattern = format!(
        r"(?s)<{}\b[^>]*>.*?</{}>",
        regex::escape(name),
        regex::escape(name)
    );
    if let Ok(re) = regex::Regex::new(&pattern) {
        return re.replace_all(&s, "").into_owned().into_bytes();
    }
    s.into_owned().into_bytes()
}

/// Extract the raw attribute string from the root element of an XML document.
pub(crate) fn extract_root_namespace_attributes(content: &[u8]) -> Option<String> {
    let s = std::str::from_utf8(content).ok()?;
    // Skip optional XML declaration and whitespace.
    let mut pos = 0usize;
    while pos < s.len() {
        let c = s[pos..].chars().next()?;
        if c == '<' {
            if s[pos..].starts_with("<?") {
                if let Some(end) = s[pos..].find("?>") {
                    pos += end + 2;
                    continue;
                }
                return None;
            }
            break;
        }
        pos += c.len_utf8();
    }
    if pos >= s.len() {
        return None;
    }
    let start = pos;
    let close = s[start..].find('>')?;
    let tag = &s[start..start + close + 1];
    // Tag name ends at first whitespace.
    let name_end = tag
        .find(|c: char| c.is_whitespace())
        .unwrap_or(tag.len() - 1);
    let after_name = &tag[name_end..tag.len() - 1];
    let trimmed = after_name.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

/// Replace the attributes of the root element with the captured namespace string.
pub(crate) fn replace_root_namespace_attributes(content: &mut Vec<u8>, attrs: &str) -> Result<()> {
    let s = std::str::from_utf8(content)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
    let mut pos = 0usize;
    while pos < s.len() {
        let c = s[pos..]
            .chars()
            .next()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "empty"))?;
        if c == '<' {
            if s[pos..].starts_with("<?") {
                if let Some(end) = s[pos..].find("?>") {
                    pos += end + 2;
                    continue;
                }
                return Err(Box::new(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "bad xml decl",
                )));
            }
            break;
        }
        pos += c.len_utf8();
    }
    let start = pos;
    let close = s[start..]
        .find('>')
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "no root close"))?;
    let tag = &s[start..start + close + 1];
    let name_end = tag
        .find(|c: char| c.is_whitespace())
        .unwrap_or(tag.len() - 1);
    let tag_name = &tag[1..name_end];
    let end_tag = start + close + 1;
    let new_start = format!("<{tag_name} {attrs}>");
    let tail = content.split_off(end_tag);
    content.truncate(start);
    content.extend_from_slice(new_start.as_bytes());
    content.extend_from_slice(&tail);
    Ok(())
}

fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

/// Remove attributes with empty values from serialized XML.
///
/// quick_xml emits `None` `Option` fields as `attr=""`, which cannot be
/// parsed back into boolean/integer fields. Stripping them lets round-trips
/// through `Option<T>` fields work until the XML types are updated to skip
/// `None` values explicitly.
pub(crate) fn strip_empty_attributes(content: &mut Vec<u8>) {
    static RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    let re = RE.get_or_init(|| regex::Regex::new(r#" ([a-zA-Z_][a-zA-Z0-9_:\-]*)="""#).unwrap());
    let s = String::from_utf8_lossy(content);
    let cleaned = re.replace_all(&s, "");
    *content = cleaned.into_owned().into_bytes();
}

fn decode_theme_to_xlsx_theme(theme: &DecodeTheme) -> XlsxTheme {
    fn convert_color(c: &crate::xml::theme::DecodeCtColor) -> XlsxCtColor {
        XlsxCtColor {
            scrgb_clr: c.scrgb_clr.clone(),
            srgb_clr: c.srgb_clr.clone(),
            hsl_clr: c.hsl_clr.clone(),
            sys_clr: c.sys_clr.clone().map(|s| XlsxSysClr {
                val: s.val,
                last_clr: s.last_clr,
            }),
            scheme_clr: c.scheme_clr.clone(),
            prst_clr: c.prst_clr.clone(),
        }
    }
    fn convert_font(c: &crate::xml::theme::DecodeFontCollection) -> XlsxFontCollection {
        XlsxFontCollection {
            latin: c.latin.clone(),
            ea: c.ea.clone(),
            cs: c.cs.clone(),
            font: c.font.clone(),
            ext_lst: c.ext_lst.clone(),
        }
    }
    XlsxTheme {
        xmlns_a: None,
        xmlns_r: None,
        name: theme.name.clone(),
        theme_elements: XlsxBaseStyles {
            clr_scheme: crate::xml::theme::XlsxColorScheme {
                name: theme.theme_elements.clr_scheme.name.clone(),
                dk1: convert_color(&theme.theme_elements.clr_scheme.dk1),
                lt1: convert_color(&theme.theme_elements.clr_scheme.lt1),
                dk2: convert_color(&theme.theme_elements.clr_scheme.dk2),
                lt2: convert_color(&theme.theme_elements.clr_scheme.lt2),
                accent1: convert_color(&theme.theme_elements.clr_scheme.accent1),
                accent2: convert_color(&theme.theme_elements.clr_scheme.accent2),
                accent3: convert_color(&theme.theme_elements.clr_scheme.accent3),
                accent4: convert_color(&theme.theme_elements.clr_scheme.accent4),
                accent5: convert_color(&theme.theme_elements.clr_scheme.accent5),
                accent6: convert_color(&theme.theme_elements.clr_scheme.accent6),
                hlink: convert_color(&theme.theme_elements.clr_scheme.hlink),
                fol_hlink: convert_color(&theme.theme_elements.clr_scheme.fol_hlink),
                ext_lst: theme.theme_elements.clr_scheme.ext_lst.clone(),
            },
            font_scheme: crate::xml::theme::XlsxFontScheme {
                name: theme.theme_elements.font_scheme.name.clone(),
                major_font: convert_font(&theme.theme_elements.font_scheme.major_font),
                minor_font: convert_font(&theme.theme_elements.font_scheme.minor_font),
                ext_lst: theme.theme_elements.font_scheme.ext_lst.clone(),
            },
            fmt_scheme: crate::xml::theme::XlsxStyleMatrix {
                name: theme.theme_elements.fmt_scheme.name.clone(),
                fill_style_lst: theme.theme_elements.fmt_scheme.fill_style_lst.clone(),
                ln_style_lst: theme.theme_elements.fmt_scheme.ln_style_lst.clone(),
                effect_style_lst: theme.theme_elements.fmt_scheme.effect_style_lst.clone(),
                bg_fill_style_lst: theme.theme_elements.fmt_scheme.bg_fill_style_lst.clone(),
            },
            ext_lst: theme.theme_elements.ext_lst.clone(),
        },
        object_defaults: theme.object_defaults.clone(),
        extra_clr_scheme_lst: theme.extra_clr_scheme_lst.clone(),
        cust_clr_lst: theme.cust_clr_lst.clone(),
        ext_lst: theme.ext_lst.clone(),
    }
}

// ------------------------------------------------------------------
// Trait-based helpers referenced by `excelize.rs`
// ------------------------------------------------------------------

/// Helper to apply the workbook content-type for macro/template files.
pub fn set_content_type_part_project_extensions(file: &File, content_type: &str) -> Result<()> {
    file.set_content_type_part_project_extensions(content_type)
}

/// Add a VBA project binary to the workbook.
pub fn add_vba_project(file: &File, data: &[u8]) -> Result<()> {
    file.add_vba_project(data)
}

// ------------------------------------------------------------------
// Extension list helpers
// ------------------------------------------------------------------

/// Extract the inner XML of the `<extLst>` element, if present.
fn extract_ext_lst(data: &[u8]) -> Option<String> {
    let s = String::from_utf8_lossy(data);
    let start_key = "<extLst";
    let start = s.find(start_key)?;
    let close_bracket = s[start..].find('>')? + start + 1;
    let end_key = "</extLst>";
    let end = s.find(end_key)? + end_key.len();
    Some(s[close_bracket..end - end_key.len()].to_string())
}

/// Remove the `<extLst>` element from raw worksheet XML so that serde can
/// deserialize the remainder.
fn remove_ext_lst(data: &mut Vec<u8>) {
    let s = String::from_utf8_lossy(data);
    let Some(start) = s.find("<extLst") else {
        return;
    };
    let Some(end) = s.find("</extLst>") else {
        return;
    };
    let end = end + "</extLst>".len();
    let mut result = s[..start].as_bytes().to_vec();
    result.extend_from_slice(s[end..].as_bytes());
    *data = result;
}

/// Inject the serialized `<extLst>` inner XML before the closing
/// `</worksheet>` tag.
fn inject_ext_lst(output: &mut Vec<u8>, ext_xml: &str) {
    let s = String::from_utf8_lossy(output);
    let Some(pos) = s.rfind("</worksheet>") else {
        return;
    };
    let mut result = s[..pos].as_bytes().to_vec();
    result.extend_from_slice(b"<extLst>");
    result.extend_from_slice(ext_xml.as_bytes());
    result.extend_from_slice(b"</extLst>");
    result.extend_from_slice(s[pos..].as_bytes());
    *output = result;
}

impl Drop for File {
    fn drop(&mut self) {
        // Best-effort cleanup of temporary files that may still be around if
        // the user did not call `close()` or if an error path left them behind.
        for entry in self.temp_files.iter() {
            let _ = fs::remove_file(entry.value());
        }
        self.temp_files.clear();
        for state in self.streams.borrow().values() {
            let _ = fs::remove_file(&state.tmp_path);
        }
        self.streams.borrow_mut().clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_file_round_trip() {
        let mut f = File::new_with_options(Options::default());
        let tmp = std::env::temp_dir().join("excelize_rust_test.xlsx");
        f.save_as(tmp.to_str().unwrap()).unwrap();

        let file = fs::File::open(&tmp).unwrap();
        let archive = ZipArchive::new(file).unwrap();
        let names: Vec<String> = archive.file_names().map(|s| s.to_string()).collect();
        assert!(names.contains(&"[Content_Types].xml".to_string()));
        assert!(names.contains(&"xl/workbook.xml".to_string()));
        assert!(names.contains(&"xl/worksheets/sheet1.xml".to_string()));
        assert!(names.contains(&"xl/styles.xml".to_string()));
        assert!(names.contains(&"xl/theme/theme1.xml".to_string()));
        drop(archive);
        let _ = fs::remove_file(&tmp);
    }

    #[test]
    fn save_and_reopen() {
        let mut f = File::new_with_options(Options::default());
        let tmp = std::env::temp_dir().join("excelize_rust_reopen_test.xlsx");
        f.save_as(tmp.to_str().unwrap()).unwrap();

        let f2 = File::open_file(tmp.to_str().unwrap(), Options::default()).unwrap();
        assert_eq!(*f2.sheet_count.lock().unwrap(), 1);
        assert_eq!(f2.get_sheet_list(), vec!["Sheet1"]);
        let _ = fs::remove_file(&tmp);
    }

    #[test]
    fn open_existing_xlsx() {
        // Open one of the fixtures shipped with the repository.
        let path = "test/Book1.xlsx";
        let f = File::open_file(path, Options::default()).unwrap();
        assert!(*f.sheet_count.lock().unwrap() >= 1);
        let list = f.get_sheet_list();
        assert!(!list.is_empty());
    }

    #[test]
    fn workbook_props_round_trip() {
        let f = File::new();
        let mut opts = WorkbookPropsOptions::default();
        opts.date1904 = Some(true);
        opts.filter_privacy = Some(false);
        opts.code_name = Some("Workbook1".to_string());
        f.set_workbook_props(&opts).unwrap();

        let got = f.get_workbook_props().unwrap();
        assert_eq!(got.date1904, Some(true));
        assert_eq!(got.filter_privacy, Some(false));
        assert_eq!(got.code_name, Some("Workbook1".to_string()));
    }

    #[test]
    fn calc_props_round_trip() {
        let f = File::new();
        let mut opts = CalcPropsOptions::default();
        opts.calc_mode = Some("manual".to_string());
        opts.ref_mode = Some("R1C1".to_string());
        opts.full_calc_on_load = Some(true);
        opts.calc_id = Some(152511);
        opts.iterate_count = Some(100);
        f.set_calc_props(&opts).unwrap();

        let got = f.get_calc_props().unwrap();
        assert_eq!(got.calc_mode, Some("manual".to_string()));
        assert_eq!(got.ref_mode, Some("R1C1".to_string()));
        assert_eq!(got.full_calc_on_load, Some(true));
        assert_eq!(got.calc_id, Some(152511));
        assert_eq!(got.iterate_count, Some(100));
    }

    #[test]
    fn calc_props_rejects_invalid_mode() {
        let f = File::new();
        let mut opts = CalcPropsOptions::default();
        opts.calc_mode = Some("invalid".to_string());
        assert!(f.set_calc_props(&opts).is_err());

        opts.calc_mode = None;
        opts.ref_mode = Some("B3".to_string());
        assert!(f.set_calc_props(&opts).is_err());
    }

    #[test]
    fn protect_workbook_round_trip() {
        let f = File::new();
        let opts = WorkbookProtectionOptions {
            password: "password".to_string(),
            lock_structure: true,
            lock_windows: false,
            ..Default::default()
        };
        f.protect_workbook(&opts).unwrap();
        let wb = f.workbook_reader().unwrap();
        let protection = wb.workbook_protection.as_ref().unwrap();
        assert_eq!(protection.lock_structure, Some(true));
        assert!(protection.workbook_hash_value.is_some());

        assert!(f.unprotect_workbook(Some("wrong")).is_err());
        f.unprotect_workbook(Some("password")).unwrap();
        assert!(f.workbook_reader().unwrap().workbook_protection.is_none());
    }

    #[test]
    fn add_vba_project() {
        let f = File::new();
        let mut sheet_opts = crate::sheet::SheetPropsOptions::default();
        sheet_opts.code_name = Some("Sheet1".to_string());
        f.set_sheet_props("Sheet1", &sheet_opts).unwrap();

        let bad = fs::read("test/Book1.xlsx").unwrap();
        assert!(f.add_vba_project(&bad).is_err());

        let data = fs::read("test/vbaProject.bin").unwrap();
        f.add_vba_project(&data).unwrap();
        // Adding the same VBA project again should be idempotent.
        f.add_vba_project(&data).unwrap();

        let rels = f
            .rels_reader("xl/_rels/workbook.xml.rels")
            .unwrap()
            .unwrap();
        let vba_rel = rels
            .relationships
            .iter()
            .find(|r| r.target == "vbaProject.bin" && r.r#type == SOURCE_RELATIONSHIP_VBA_PROJECT);
        assert!(vba_rel.is_some());

        let tmp = std::env::temp_dir().join("excelize_rust_vba.xlsm");
        let mut f2 = File::new();
        f2.set_sheet_props("Sheet1", &sheet_opts).unwrap();
        f2.add_vba_project(&data).unwrap();
        f2.save_as(tmp.to_str().unwrap()).unwrap();

        let f3 = File::open_file(tmp.to_str().unwrap(), Options::default()).unwrap();
        assert!(f3.pkg.contains_key("xl/vbaProject.bin"));
        let _ = fs::remove_file(&tmp);
    }

    #[test]
    fn charset_transcoder_is_used_for_non_utf8_encoding() {
        use crate::constants::DEFAULT_XML_PATH_CALC_CHAIN;
        use std::io::Read;
        use std::sync::{Arc, Mutex};

        let f = File::new();
        let xml = br#"<?xml version="1.0" encoding="X-TEST" standalone="yes"?>
<calcChain xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><c r="A1" i="1"/></calcChain>"#;
        f.pkg
            .insert(DEFAULT_XML_PATH_CALC_CHAIN.to_string(), xml.to_vec());
        *f.calc_chain.lock().unwrap() = None;

        let called = Arc::new(Mutex::new(String::new()));
        let called_clone = called.clone();
        f.charset_transcoder(move |charset, mut input| {
            *called_clone.lock().unwrap() = charset.to_string();
            let mut buf = Vec::new();
            input.read_to_end(&mut buf).unwrap();
            Ok(Box::new(Cursor::new(buf)) as Box<dyn Read>)
        });

        let cc = f.calc_chain_reader().unwrap();
        assert_eq!(cc.c.len(), 1);
        assert_eq!(cc.c[0].r, "A1");
        assert_eq!(called.lock().unwrap().as_str(), "X-TEST");
    }

    #[test]
    fn charset_transcoder_handles_invalid_utf8_bytes() {
        use crate::constants::DEFAULT_XML_PATH_CALC_CHAIN;
        use std::io::Read;
        use std::sync::{Arc, Mutex};

        let f = File::new();
        let mut xml = br#"<?xml version="1.0" encoding="X-BAD" standalone="yes"?>
<calcChain xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><c r="A1" i="1"/></calcChain>"#
            .to_vec();
        xml.push(0xFF); // invalid UTF-8 trailing byte
        f.pkg.insert(DEFAULT_XML_PATH_CALC_CHAIN.to_string(), xml);
        *f.calc_chain.lock().unwrap() = None;

        let called = Arc::new(Mutex::new(false));
        let called_clone = called.clone();
        f.charset_transcoder(move |_charset, mut input| {
            *called_clone.lock().unwrap() = true;
            let mut buf = Vec::new();
            input.read_to_end(&mut buf).unwrap();
            buf.pop(); // strip the invalid trailing byte
            Ok(Box::new(Cursor::new(buf)) as Box<dyn Read>)
        });

        let cc = f.calc_chain_reader().unwrap();
        assert_eq!(cc.c.len(), 1);
        assert_eq!(cc.c[0].r, "A1");
        assert!(*called.lock().unwrap());
    }

    #[test]
    fn set_zip_writer_uses_custom_factory() {
        use std::sync::{Arc, Mutex};

        struct MockZipWriter {
            calls: Arc<Mutex<Vec<String>>>,
        }

        impl ZipWriter for MockZipWriter {
            fn start_file(&mut self, name: &str, _options: SimpleFileOptions) -> Result<()> {
                self.calls.lock().unwrap().push(format!("start:{name}"));
                Ok(())
            }
            fn write_all(&mut self, _buf: &[u8]) -> Result<()> {
                self.calls.lock().unwrap().push("write".to_string());
                Ok(())
            }
            fn finish(self: Box<Self>) -> Result<()> {
                self.calls.lock().unwrap().push("finish".to_string());
                Ok(())
            }
        }

        let f = File::new();
        let calls = Arc::new(Mutex::new(Vec::new()));
        let calls_clone = calls.clone();
        f.set_zip_writer(move |_writer| {
            Box::new(MockZipWriter {
                calls: calls_clone.clone(),
            })
        });

        let _ = f.write_to_buffer().unwrap();
        let calls = calls.lock().unwrap();
        assert!(calls.iter().any(|c| c.starts_with("start:")));
        assert!(calls.contains(&"write".to_string()));
        assert!(calls.contains(&"finish".to_string()));
    }

    #[test]
    fn get_defined_name_alias() {
        let f = File::new();
        let names = f.get_defined_name().unwrap();
        assert!(names.is_empty());
        assert_eq!(f.get_defined_names().unwrap(), names);
    }

    #[test]
    fn update_linked_value_clears_formula_cache() {
        let f = File::new_with_options(Options::default());
        f.set_cell_int("Sheet1", "A1", 1).unwrap();
        f.set_cell_int("Sheet1", "A2", 2).unwrap();
        f.set_cell_formula("Sheet1", "A3", "A1+A2").unwrap();
        f.calc_cell_value("Sheet1", "A3").unwrap();
        f.update_linked_value().unwrap();
        assert!(f.workbook_reader().unwrap().calc_pr.is_none());
    }

    #[test]
    fn read_zip_reader_extracts_parts() {
        let mut f = File::new_with_options(Options::default());
        let tmp = std::env::temp_dir().join("excelize_rust_read_zip_reader.xlsx");
        f.save_as(tmp.to_str().unwrap()).unwrap();

        let file = fs::File::open(&tmp).unwrap();
        let mut archive = ZipArchive::new(file).unwrap();
        let f2 = File::new_with_options(Options::default());
        let (parts, count) = f2.read_zip_reader(&mut archive).unwrap();
        assert!(parts.contains_key(DEFAULT_XML_PATH_WORKBOOK));
        assert_eq!(count, 1);
        let _ = fs::remove_file(&tmp);
    }

    #[test]
    fn set_content_type_part_rels_extensions_adds_default() {
        let f = File::new();
        f.set_content_type_part_rels_extensions().unwrap();
        let ct = f.content_types_reader().unwrap();
        assert!(ct.entries.iter().any(|e| {
            if let crate::xml::content_types::XlsxContentTypeEntry::Default(d) = e {
                d.extension == "rels" && d.content_type == CONTENT_TYPE_RELATIONSHIPS
            } else {
                false
            }
        }));
    }

    #[test]
    fn add_content_type_part_adds_rels_default() {
        let f = File::new();
        f.add_content_type_part(1, "table").unwrap();
        let ct = f.content_types_reader().unwrap();
        assert!(ct.entries.iter().any(|e| {
            if let crate::xml::content_types::XlsxContentTypeEntry::Default(d) = e {
                d.extension == "rels"
            } else {
                false
            }
        }));
    }

    #[test]
    fn set_content_type_part_image_extensions_uses_defaults() {
        let f = File::new();
        crate::sheet::set_content_type_part_image_extensions(&f).unwrap();
        let ct = f.content_types_reader().unwrap();
        assert!(ct.entries.iter().any(|e| {
            if let crate::xml::content_types::XlsxContentTypeEntry::Default(d) = e {
                d.extension == "png" && d.content_type == "image/png"
            } else {
                false
            }
        }));
        assert!(ct.entries.iter().any(|e| {
            if let crate::xml::content_types::XlsxContentTypeEntry::Default(d) = e {
                d.extension == "jpeg" && d.content_type == "image/jpeg"
            } else {
                false
            }
        }));
        // Should not create Overrides with fake part names.
        assert!(!ct.entries.iter().any(|e| {
            if let crate::xml::content_types::XlsxContentTypeEntry::Override(o) = e {
                o.part_name.contains("image1")
            } else {
                false
            }
        }));
    }

    #[test]
    fn add_name_spaces_registers_namespace() {
        let f = File::new();
        let path = f.get_sheet_xml_path("Sheet1").unwrap();
        f.add_name_spaces(&path, SOURCE_RELATIONSHIP);
        let attrs = f.xml_attr.get(&path).map(|a| a.clone()).unwrap_or_default();
        assert!(attrs.contains("xmlns:r=\""));
        assert!(attrs.contains(SOURCE_RELATIONSHIP));
    }

    #[test]
    fn add_name_spaces_marks_extension_namespace_ignorable() {
        let f = File::new();
        let path = f.get_sheet_xml_path("Sheet1").unwrap();
        f.add_name_spaces(&path, NAMESPACE_SPREADSHEET_X14);
        let attrs = f.xml_attr.get(&path).map(|a| a.clone()).unwrap_or_default();
        assert!(attrs.contains("xmlns:x14=\""));
        assert!(attrs.contains("xmlns:mc=\""));
        assert!(attrs.contains("mc:Ignorable=\"x14\""));
    }

    #[test]
    fn add_sheet_name_space_resolves_path() {
        let f = File::new();
        f.add_sheet_name_space("Sheet1", SOURCE_RELATIONSHIP);
        let path = f.get_sheet_xml_path("Sheet1").unwrap();
        let attrs = f.xml_attr.get(&path).map(|a| a.clone()).unwrap_or_default();
        assert!(attrs.contains("xmlns:r=\""));
    }

    #[test]
    fn workbook_writer_preserves_alternate_content() {
        let f = File::new();
        let mut wb = f.workbook_reader().unwrap();
        wb.decode_alternate_content = Some(crate::xml::common::XlsxInnerXml {
            content: "<mc:Choice Requires=\"a14\" xmlns:a14=\"http://schemas.microsoft.com/office/drawing/2010/main\"><foo/></mc:Choice>".to_string(),
        });
        *f.workbook.lock().unwrap() = Some(wb);

        f.workbook_writer();

        let path = f.get_workbook_path();
        let bytes = f.read_xml(&path);
        let output = String::from_utf8_lossy(&bytes);
        assert!(output.contains("mc:AlternateContent"));
        assert!(output.contains("mc:Choice"));
    }
}
