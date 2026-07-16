//! VML drawing part (`xl/drawings/vmlDrawingN.vml`).
//!
//! Ported from Go `vmlDrawing.go`. The VML format mixes namespaced elements
//! and arbitrary inner XML, so this module uses a small manual parser/serializer
//! built on `quick_xml` rather than full serde mapping.

use quick_xml::events::{BytesStart, BytesText, Event};
use quick_xml::name::QName;
use quick_xml::{Reader, Writer};

// ------------------------------------------------------------------
// Public data model
// ------------------------------------------------------------------

/// Root element of a VML drawing part.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct VmlDrawing {
    pub xmlns_v: String,
    pub xmlns_o: String,
    pub xmlns_x: String,
    pub xmlns_mv: Option<String>,
    pub shape_layout: Option<VmlShapeLayout>,
    pub shape_type: Option<VmlShapeType>,
    pub shape: Vec<VmlShape>,
}

/// `<o:shapelayout>` element.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct VmlShapeLayout {
    pub ext: String,
    pub idmap: Option<VmlIdmap>,
}

/// `<o:idmap>` element.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct VmlIdmap {
    pub ext: String,
    pub data: i32,
}

/// `<v:shapetype>` element.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct VmlShapeType {
    pub id: String,
    pub coord_size: String,
    pub spt: i32,
    pub prefer_relative: Option<String>,
    pub path: String,
    pub filled: Option<String>,
    pub stroked: Option<String>,
    pub stroke: Option<VmlStroke>,
    pub formulas: Option<VmlFormulas>,
    pub v_path: Option<VmlPath>,
    pub lock: Option<VmlLock>,
}

/// `<v:stroke>` element.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct VmlStroke {
    pub join_style: String,
}

/// `<v:path>` element.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct VmlPath {
    pub extrusion_ok: Option<String>,
    pub gradient_shape_ok: Option<String>,
    pub connect_type: String,
}

/// `<v:formulas>` / `<v:f>` elements.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct VmlFormulas {
    pub formula: Vec<VmlFormula>,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct VmlFormula {
    pub equation: String,
}

/// `<o:lock>` element.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct VmlLock {
    pub ext: String,
    pub rotation: Option<String>,
    pub aspect_ratio: Option<String>,
}

/// `<v:imagedata>` element.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct VmlImageData {
    pub id: Option<String>,
    pub src: Option<String>,
    pub rel_id: Option<String>,
    pub title: Option<String>,
    pub crop_top: Option<String>,
    pub crop_left: Option<String>,
    pub crop_bottom: Option<String>,
    pub crop_right: Option<String>,
    pub gain: Option<String>,
    pub black_level: Option<String>,
    pub gamma: Option<String>,
    pub grayscale: Option<String>,
    pub bilevel: Option<String>,
}

/// `<v:shape>` element. The inner XML is stored as an opaque string so that
/// namespaced child elements (`v:fill`, `x:ClientData`, etc.) round-trip
/// without loss.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct VmlShape {
    pub id: String,
    pub spid: Option<String>,
    pub shape_type: String,
    pub style: String,
    pub button: Option<String>,
    pub filled: Option<String>,
    pub fill_color: Option<String>,
    pub inset_mode: Option<String>,
    pub stroked: Option<String>,
    pub stroke_color: Option<String>,
    pub inner_xml: String,
}

// ------------------------------------------------------------------
// Serialization
// ------------------------------------------------------------------

impl VmlLock {
    pub(crate) fn write_xml<W: std::io::Write>(&self, w: &mut Writer<W>) {
        let mut e = BytesStart::new("o:lock");
        if !self.ext.is_empty() {
            e.push_attribute(("v:ext", self.ext.as_str()));
        }
        if let Some(v) = &self.rotation {
            e.push_attribute(("rotation", v.as_str()));
        }
        if let Some(v) = &self.aspect_ratio {
            e.push_attribute(("aspectratio", v.as_str()));
        }
        w.write_event(Event::Empty(e)).ok();
    }

    pub(crate) fn to_xml_string(&self) -> String {
        let mut buf = Vec::new();
        self.write_xml(&mut Writer::new(&mut buf));
        String::from_utf8_lossy(&buf).to_string()
    }
}

impl VmlImageData {
    pub(crate) fn write_xml<W: std::io::Write>(&self, w: &mut Writer<W>) {
        let mut e = BytesStart::new("v:imagedata");
        if let Some(v) = &self.id {
            e.push_attribute(("id", v.as_str()));
        }
        if let Some(v) = &self.src {
            e.push_attribute(("src", v.as_str()));
        }
        if let Some(v) = &self.rel_id {
            e.push_attribute(("o:relid", v.as_str()));
        }
        if let Some(v) = &self.title {
            e.push_attribute(("o:title", v.as_str()));
        }
        if let Some(v) = &self.crop_top {
            e.push_attribute(("croptop", v.as_str()));
        }
        if let Some(v) = &self.crop_left {
            e.push_attribute(("cropleft", v.as_str()));
        }
        if let Some(v) = &self.crop_bottom {
            e.push_attribute(("cropbottom", v.as_str()));
        }
        if let Some(v) = &self.crop_right {
            e.push_attribute(("cropright", v.as_str()));
        }
        if let Some(v) = &self.gain {
            e.push_attribute(("gain", v.as_str()));
        }
        if let Some(v) = &self.black_level {
            e.push_attribute(("blacklevel", v.as_str()));
        }
        if let Some(v) = &self.gamma {
            e.push_attribute(("gamma", v.as_str()));
        }
        if let Some(v) = &self.grayscale {
            e.push_attribute(("grayscale", v.as_str()));
        }
        if let Some(v) = &self.bilevel {
            e.push_attribute(("bilevel", v.as_str()));
        }
        w.write_event(Event::Empty(e)).ok();
    }

    pub(crate) fn to_xml_string(&self) -> String {
        let mut buf = Vec::new();
        self.write_xml(&mut Writer::new(&mut buf));
        String::from_utf8_lossy(&buf).to_string()
    }
}

impl VmlDrawing {
    /// Serialize the VML drawing to bytes including the XML header.
    pub fn to_xml(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(crate::constants::XML_HEADER.as_bytes());
        out.push(b'\n');
        let mut w = Writer::new_with_indent(&mut out, b' ', 2);

        let mut root = BytesStart::new("xml");
        root.push_attribute(("xmlns:v", self.xmlns_v.as_str()));
        root.push_attribute(("xmlns:o", self.xmlns_o.as_str()));
        root.push_attribute(("xmlns:x", self.xmlns_x.as_str()));
        if let Some(ns) = &self.xmlns_mv {
            root.push_attribute(("xmlns:mv", ns.as_str()));
        }
        w.write_event(Event::Start(root)).ok();

        if let Some(layout) = &self.shape_layout {
            layout.write_xml(&mut w);
        }
        if let Some(st) = &self.shape_type {
            st.write_xml(&mut w);
        }
        for shape in &self.shape {
            shape.write_xml(&mut w);
        }

        w.write_event(Event::End(QName(b"xml").into())).ok();
        out
    }

    /// Parse a VML drawing from raw bytes.
    pub fn from_xml(data: &[u8]) -> Result<Self, quick_xml::Error> {
        let mut reader = Reader::from_reader(data);
        reader.config_mut().trim_text(true);
        let mut buf = Vec::new();
        let mut drawing = VmlDrawing::default();
        let mut depth = 0usize;

        loop {
            let event = reader.read_event_into(&mut buf)?;
            match event {
                Event::Start(e) => {
                    let name = e.name();
                    let local = local_name(name.as_ref());
                    if local == b"xml" && depth == 0 {
                        parse_root_attrs(&e, &mut drawing);
                        depth += 1;
                    } else if local == b"shapelayout" {
                        drawing.shape_layout = Some(parse_shape_layout(&e, &mut reader)?);
                    } else if local == b"shapetype" {
                        drawing.shape_type = Some(parse_shape_type(&e, &mut reader)?);
                    } else if local == b"shape" {
                        drawing.shape.push(parse_shape(&e, &mut reader)?);
                    } else {
                        depth += 1;
                    }
                }
                Event::Empty(e) => {
                    let name = e.name();
                    let local = local_name(name.as_ref());
                    if local == b"xml" && depth == 0 {
                        parse_root_attrs(&e, &mut drawing);
                    } else if local == b"shapelayout" {
                        drawing.shape_layout = Some(parse_shape_layout(&e, &mut reader)?);
                    } else if local == b"shapetype" {
                        drawing.shape_type = Some(parse_shape_type(&e, &mut reader)?);
                    } else if local == b"shape" {
                        let mut shape = parse_shape_attrs(&e);
                        shape.inner_xml = String::new();
                        drawing.shape.push(shape);
                    }
                }
                Event::End(e) => {
                    let name = e.name();
                    if local_name(name.as_ref()) == b"xml" {
                        depth = depth.saturating_sub(1);
                    }
                }
                Event::Eof => break,
                _ => {}
            }
            buf.clear();
        }
        Ok(drawing)
    }
}

// ------------------------------------------------------------------
// Helper implementations
// ------------------------------------------------------------------

impl VmlShapeLayout {
    fn write_xml<W: std::io::Write>(&self, w: &mut Writer<W>) {
        let mut start = BytesStart::new("o:shapelayout");
        if !self.ext.is_empty() {
            start.push_attribute(("v:ext", self.ext.as_str()));
        }
        w.write_event(Event::Start(start)).ok();
        if let Some(idmap) = &self.idmap {
            let mut e = BytesStart::new("o:idmap");
            if !idmap.ext.is_empty() {
                e.push_attribute(("v:ext", idmap.ext.as_str()));
            }
            e.push_attribute(("data", idmap.data.to_string().as_str()));
            w.write_event(Event::Empty(e)).ok();
        }
        w.write_event(Event::End(QName(b"o:shapelayout").into()))
            .ok();
    }
}

impl VmlShapeType {
    fn write_xml<W: std::io::Write>(&self, w: &mut Writer<W>) {
        let mut start = BytesStart::new("v:shapetype");
        if !self.id.is_empty() {
            start.push_attribute(("id", self.id.as_str()));
        }
        if !self.coord_size.is_empty() {
            start.push_attribute(("coordsize", self.coord_size.as_str()));
        }
        if self.spt != 0 {
            start.push_attribute(("o:spt", self.spt.to_string().as_str()));
        }
        if let Some(v) = &self.prefer_relative {
            start.push_attribute(("o:preferrelative", v.as_str()));
        }
        if !self.path.is_empty() {
            start.push_attribute(("path", self.path.as_str()));
        }
        if let Some(v) = &self.filled {
            start.push_attribute(("filled", v.as_str()));
        }
        if let Some(v) = &self.stroked {
            start.push_attribute(("stroked", v.as_str()));
        }
        w.write_event(Event::Start(start)).ok();
        if let Some(stroke) = &self.stroke {
            let mut e = BytesStart::new("v:stroke");
            e.push_attribute(("joinstyle", stroke.join_style.as_str()));
            w.write_event(Event::Empty(e)).ok();
        }
        if let Some(formulas) = &self.formulas {
            w.write_event(Event::Start(BytesStart::new("v:formulas")))
                .ok();
            for f in &formulas.formula {
                let mut e = BytesStart::new("v:f");
                e.push_attribute(("eqn", f.equation.as_str()));
                w.write_event(Event::Empty(e)).ok();
            }
            w.write_event(Event::End(QName(b"v:formulas").into())).ok();
        }
        if let Some(path) = &self.v_path {
            let mut e = BytesStart::new("v:path");
            if let Some(v) = &path.extrusion_ok {
                e.push_attribute(("o:extrusionok", v.as_str()));
            }
            if let Some(v) = &path.gradient_shape_ok {
                e.push_attribute(("gradientshapeok", v.as_str()));
            }
            e.push_attribute(("o:connecttype", path.connect_type.as_str()));
            w.write_event(Event::Empty(e)).ok();
        }
        if let Some(lock) = &self.lock {
            lock.write_xml(w);
        }
        w.write_event(Event::End(QName(b"v:shapetype").into())).ok();
    }
}

impl VmlShape {
    fn write_xml<W: std::io::Write>(&self, w: &mut Writer<W>) {
        let mut start = BytesStart::new("v:shape");
        if !self.id.is_empty() {
            start.push_attribute(("id", self.id.as_str()));
        }
        if !self.shape_type.is_empty() {
            start.push_attribute(("type", self.shape_type.as_str()));
        }
        if !self.style.is_empty() {
            start.push_attribute(("style", self.style.as_str()));
        }
        if let Some(v) = &self.spid {
            start.push_attribute(("o:spid", v.as_str()));
        }
        if let Some(v) = &self.button {
            start.push_attribute(("o:button", v.as_str()));
        }
        if let Some(v) = &self.filled {
            start.push_attribute(("filled", v.as_str()));
        }
        if let Some(v) = &self.fill_color {
            start.push_attribute(("fillcolor", v.as_str()));
        }
        if let Some(v) = &self.inset_mode {
            start.push_attribute(("o:insetmode", v.as_str()));
        }
        if let Some(v) = &self.stroked {
            start.push_attribute(("stroked", v.as_str()));
        }
        if let Some(v) = &self.stroke_color {
            start.push_attribute(("strokecolor", v.as_str()));
        }
        w.write_event(Event::Start(start)).ok();
        if !self.inner_xml.is_empty() {
            w.write_event(Event::Text(BytesText::from_escaped(
                self.inner_xml.as_str(),
            )))
            .ok();
        }
        w.write_event(Event::End(QName(b"v:shape").into())).ok();
    }
}

// ------------------------------------------------------------------
// Parsing helpers
// ------------------------------------------------------------------

fn local_name(name: &[u8]) -> &[u8] {
    if let Some(pos) = name.iter().rposition(|&b| b == b':') {
        &name[pos + 1..]
    } else {
        name
    }
}

fn parse_root_attrs(e: &BytesStart<'_>, drawing: &mut VmlDrawing) {
    for attr in e.attributes().flatten() {
        let key = String::from_utf8_lossy(attr.key.as_ref());
        let value = String::from_utf8_lossy(&attr.value).to_string();
        if key == "xmlns:v" {
            drawing.xmlns_v = value;
        } else if key == "xmlns:o" {
            drawing.xmlns_o = value;
        } else if key == "xmlns:x" {
            drawing.xmlns_x = value;
        } else if key == "xmlns:mv" {
            drawing.xmlns_mv = Some(value);
        }
    }
}

fn parse_shape_layout<R: std::io::BufRead>(
    e: &BytesStart<'_>,
    reader: &mut Reader<R>,
) -> Result<VmlShapeLayout, quick_xml::Error> {
    let mut layout = VmlShapeLayout::default();
    for attr in e.attributes().flatten() {
        let key = String::from_utf8_lossy(attr.key.as_ref());
        if key.ends_with(":ext") || key == "ext" {
            layout.ext = String::from_utf8_lossy(&attr.value).to_string();
        }
    }
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Start(e) if local_name(e.name().as_ref()) == b"idmap" => {
                let mut idmap = VmlIdmap::default();
                for attr in e.attributes().flatten() {
                    let key = String::from_utf8_lossy(attr.key.as_ref());
                    if key.ends_with(":ext") || key == "ext" {
                        idmap.ext = String::from_utf8_lossy(&attr.value).to_string();
                    } else if key == "data" {
                        idmap.data = String::from_utf8_lossy(&attr.value).parse().unwrap_or(0);
                    }
                }
                layout.idmap = Some(idmap);
            }
            Event::End(e) if local_name(e.name().as_ref()) == b"shapelayout" => break,
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }
    Ok(layout)
}

fn parse_shape_type<R: std::io::BufRead>(
    e: &BytesStart<'_>,
    reader: &mut Reader<R>,
) -> Result<VmlShapeType, quick_xml::Error> {
    let mut st = VmlShapeType::default();
    for attr in e.attributes().flatten() {
        let key = String::from_utf8_lossy(attr.key.as_ref());
        let value = String::from_utf8_lossy(&attr.value).to_string();
        if key == "id" {
            st.id = value;
        } else if key == "coordsize" {
            st.coord_size = value;
        } else if key.ends_with(":spt") || key == "spt" {
            st.spt = value.parse().unwrap_or(0);
        } else if key.ends_with(":preferrelative") || key == "preferrelative" {
            st.prefer_relative = Some(value);
        } else if key == "path" {
            st.path = value;
        } else if key == "filled" {
            st.filled = Some(value);
        } else if key == "stroked" {
            st.stroked = Some(value);
        }
    }

    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Empty(e) => {
                let name = e.name();
                let local = local_name(name.as_ref());
                parse_shape_type_child(local, &e, &mut st);
            }
            Event::Start(e) => {
                let name = e.name();
                let local = local_name(name.as_ref());
                if local == b"formulas" {
                    st.formulas = Some(parse_formulas(reader)?);
                } else {
                    parse_shape_type_child(local, &e, &mut st);
                }
            }
            Event::End(e) => {
                let name = e.name();
                if local_name(name.as_ref()) == b"shapetype" {
                    break;
                }
            }
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }
    Ok(st)
}

fn parse_shape_type_child(local: &[u8], e: &BytesStart<'_>, st: &mut VmlShapeType) {
    if local == b"stroke" {
        let mut stroke = VmlStroke::default();
        for attr in e.attributes().flatten() {
            let key = String::from_utf8_lossy(attr.key.as_ref());
            if key == "joinstyle" {
                stroke.join_style = String::from_utf8_lossy(&attr.value).to_string();
            }
        }
        st.stroke = Some(stroke);
    } else if local == b"path" {
        let mut path = VmlPath {
            connect_type: String::new(),
            ..Default::default()
        };
        for attr in e.attributes().flatten() {
            let key = String::from_utf8_lossy(attr.key.as_ref());
            if key.ends_with(":extrusionok") || key == "extrusionok" {
                path.extrusion_ok = Some(String::from_utf8_lossy(&attr.value).to_string());
            } else if key == "gradientshapeok" {
                path.gradient_shape_ok = Some(String::from_utf8_lossy(&attr.value).to_string());
            } else if key.ends_with(":connecttype") || key == "connecttype" {
                path.connect_type = String::from_utf8_lossy(&attr.value).to_string();
            }
        }
        st.v_path = Some(path);
    } else if local == b"lock" {
        let mut lock = VmlLock::default();
        for attr in e.attributes().flatten() {
            let key = String::from_utf8_lossy(attr.key.as_ref());
            let value = Some(String::from_utf8_lossy(&attr.value).to_string());
            if key.ends_with(":ext") || key == "ext" {
                lock.ext = value.unwrap_or_default();
            } else if key == "rotation" {
                lock.rotation = value;
            } else if key == "aspectratio" {
                lock.aspect_ratio = value;
            }
        }
        st.lock = Some(lock);
    }
}

fn parse_formulas<R: std::io::BufRead>(
    reader: &mut Reader<R>,
) -> Result<VmlFormulas, quick_xml::Error> {
    let mut formulas = VmlFormulas::default();
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Empty(e) | Event::Start(e) if local_name(e.name().as_ref()) == b"f" => {
                for attr in e.attributes().flatten() {
                    let key = String::from_utf8_lossy(attr.key.as_ref());
                    if key == "eqn" {
                        formulas.formula.push(VmlFormula {
                            equation: String::from_utf8_lossy(&attr.value).to_string(),
                        });
                    }
                }
            }
            Event::End(e) if local_name(e.name().as_ref()) == b"formulas" => break,
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }
    Ok(formulas)
}

fn parse_shape_attrs(e: &BytesStart<'_>) -> VmlShape {
    let mut shape = VmlShape::default();
    for attr in e.attributes().flatten() {
        let key = String::from_utf8_lossy(attr.key.as_ref());
        let value = String::from_utf8_lossy(&attr.value).to_string();
        if key == "id" {
            shape.id = value;
        } else if key == "type" {
            shape.shape_type = value;
        } else if key == "style" {
            shape.style = value;
        } else if key.ends_with(":spid") || key == "spid" {
            shape.spid = Some(value);
        } else if key.ends_with(":button") || key == "button" {
            shape.button = Some(value);
        } else if key == "filled" {
            shape.filled = Some(value);
        } else if key == "fillcolor" {
            shape.fill_color = Some(value);
        } else if key.ends_with(":insetmode") || key == "insetmode" {
            shape.inset_mode = Some(value);
        } else if key == "stroked" {
            shape.stroked = Some(value);
        } else if key == "strokecolor" {
            shape.stroke_color = Some(value);
        }
    }
    shape
}

fn parse_shape<R: std::io::BufRead>(
    e: &BytesStart<'_>,
    reader: &mut Reader<R>,
) -> Result<VmlShape, quick_xml::Error> {
    let mut shape = parse_shape_attrs(e);
    let mut inner = Vec::new();
    let mut inner_writer = Writer::new(&mut inner);
    let mut depth = 1usize;
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Start(ref se) => {
                inner_writer.write_event(Event::Start(se.clone())).ok();
                if local_name(se.name().as_ref()) == b"shape" {
                    depth += 1;
                }
            }
            Event::Empty(ref see) => {
                inner_writer.write_event(Event::Empty(see.clone())).ok();
            }
            Event::Text(ref te) => {
                inner_writer.write_event(Event::Text(te.clone())).ok();
            }
            Event::End(ref ee) => {
                if local_name(ee.name().as_ref()) == b"shape" {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                inner_writer.write_event(Event::End(ee.clone())).ok();
            }
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }
    shape.inner_xml = String::from_utf8_lossy(&inner).trim().to_string();
    Ok(shape)
}
