//! Per-object content hashes, and the snapshot file that holds them.
//!
//! A snapshot is a map from object identifier to content hash. The hash covers
//! a canonical encoding of the object's own meaning and never its position in
//! the file, because KiCad reorders every item on save and a file hash would
//! make each save look like a total rewrite.
//!
//! Every object carries two hashes. `geometry` covers position, orientation
//! and size. `data` covers everything else. The two together let a comparison
//! say "moved" and "edited" in one pass.
//!
//! A field and a sheet pin are objects in their own right, because a user can
//! move and edit them on their own. They hash the offset from the item that
//! owns them, so moving that item reports one change and not six.

use crate::geometry::{Iu, Point};
use crate::model::items::{
    Field, Item, Label, LabelKind, Line, LineKind, Mirror, PointItem, Schematic, SheetItem,
    SheetPath, SheetPin, Symbol, TextItem, Uuid,
};
use kicli_sexpr::{AtomKind, Doc, Node, NodeId};
use sha2::{Digest, Sha256};
use std::fmt;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

/// The directory that holds the snapshot cache inside a project.
///
/// The directory is a cache and not an artefact. Keep it out of version
/// control.
pub const CACHE_DIRECTORY: &str = ".kicli";

/// A content hash: SHA-256 truncated to eight bytes.
///
/// Eight bytes are sixteen hexadecimal characters, which is what the snapshot
/// file writes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ContentHash([u8; 8]);

impl ContentHash {
    /// Read a hash back from the sixteen characters of a snapshot file.
    ///
    /// Returns `None` when the text is not sixteen hexadecimal characters.
    #[must_use]
    pub fn from_hex(text: &str) -> Option<Self> {
        if text.len() != 16 {
            return None;
        }
        let mut bytes = [0_u8; 8];
        for (index, byte) in bytes.iter_mut().enumerate() {
            *byte = u8::from_str_radix(text.get(index * 2..index * 2 + 2)?, 16).ok()?;
        }
        Some(Self(bytes))
    }
}

impl fmt::Display for ContentHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0 {
            write!(f, "{byte:02x}")?;
        }
        Ok(())
    }
}

/// What kind of object a snapshot line describes.
///
/// The order of the variants is the order a comparison prints them in, so it
/// is part of the output and not an implementation detail.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ObjectKind {
    /// A placed symbol.
    Symbol,
    /// A field of a symbol, a sheet, or a label.
    Field,
    /// A wire segment.
    Wire,
    /// A bus segment.
    Bus,
    /// A junction dot.
    Junction,
    /// A no-connect marker.
    NoConnect,
    /// A bus entry.
    BusEntry,
    /// A local label.
    Label,
    /// A global label.
    GlobalLabel,
    /// A hierarchical label.
    HierarchicalLabel,
    /// A netclass flag.
    NetclassFlag,
    /// Free text.
    Text,
    /// A text box.
    TextBox,
    /// A child sheet.
    Sheet,
    /// One pin on the border of a child sheet.
    SheetPin,
    /// An object kicli does not name, such as a graphic shape. The token is
    /// the one the file uses.
    Other(String),
}

impl ObjectKind {
    /// The word the snapshot file writes for this kind.
    #[must_use]
    pub fn token(&self) -> &str {
        match self {
            ObjectKind::Symbol => "symbol",
            ObjectKind::Field => "field",
            ObjectKind::Wire => "wire",
            ObjectKind::Bus => "bus",
            ObjectKind::Junction => "junction",
            ObjectKind::NoConnect => "no_connect",
            ObjectKind::BusEntry => "bus_entry",
            ObjectKind::Label => "label",
            ObjectKind::GlobalLabel => "global_label",
            ObjectKind::HierarchicalLabel => "hierarchical_label",
            ObjectKind::NetclassFlag => "netclass_flag",
            ObjectKind::Text => "text",
            ObjectKind::TextBox => "text_box",
            ObjectKind::Sheet => "sheet",
            ObjectKind::SheetPin => "sheet_pin",
            ObjectKind::Other(token) => token,
        }
    }

    /// Read a kind back from a snapshot file.
    ///
    /// A word this module does not name becomes [`ObjectKind::Other`], so a
    /// snapshot written by a later version still reads.
    #[must_use]
    pub fn from_token(token: &str) -> Self {
        match token {
            "symbol" => ObjectKind::Symbol,
            "field" => ObjectKind::Field,
            "wire" => ObjectKind::Wire,
            "bus" => ObjectKind::Bus,
            "junction" => ObjectKind::Junction,
            "no_connect" => ObjectKind::NoConnect,
            "bus_entry" => ObjectKind::BusEntry,
            "label" => ObjectKind::Label,
            "global_label" => ObjectKind::GlobalLabel,
            "hierarchical_label" => ObjectKind::HierarchicalLabel,
            "netclass_flag" => ObjectKind::NetclassFlag,
            "text" => ObjectKind::Text,
            "text_box" => ObjectKind::TextBox,
            "sheet" => ObjectKind::Sheet,
            "sheet_pin" => ObjectKind::SheetPin,
            other => ObjectKind::Other(other.to_owned()),
        }
    }
}

/// What a comparison prints about an object, when the design is at hand.
///
/// A snapshot read from a file holds hashes only, so it has no detail. The
/// comparison then names an object by its identifier and reports that it
/// changed, without the old value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Detail {
    /// Where the object is drawn. A field and a sheet pin report the offset
    /// from the item that owns them.
    pub at: Option<Point>,
    /// A short description, such as a symbol's value and library identifier.
    pub summary: String,
    /// The text of a field, which a value change reports.
    pub value: Option<String>,
}

/// One object of a snapshot.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SnapshotObject {
    /// What identifies the object between two snapshots.
    ///
    /// It is the object's own identifier. A field has none of its own, so its
    /// key is the identifier of its owner, a full stop, and the field name.
    pub key: String,
    /// What kind of object this is.
    pub kind: ObjectKind,
    /// The name a comparison prints, such as a reference designator.
    pub handle: String,
    /// The hash of position, orientation and size.
    pub geometry: ContentHash,
    /// The hash of everything else.
    pub data: ContentHash,
    /// The key of the object this one belongs to, for a field or a sheet pin.
    pub owner: Option<String>,
    /// What a comparison prints, when the snapshot came from a design.
    pub detail: Option<Detail>,
}

/// The hashes of one sheet, and the header that says where they came from.
///
/// # Examples
///
/// ```
/// use kicli::model::{Schematic, SheetPath};
/// use kicli::view::snapshot::Snapshot;
/// use kicli_sexpr::Doc;
///
/// let source = concat!(
///     "(kicad_sch\n\t(version 20260306)\n\t(uuid \"a\")\n",
///     "\t(junction\n\t\t(at 0 0)\n\t\t(uuid \"j\")\n\t)\n)\n",
/// );
/// let doc = Doc::parse(source).expect("parses");
/// let schematic = Schematic::read(&doc).expect("reads");
/// let path = SheetPath::root(schematic.uuid.as_ref().expect("has a uuid"));
///
/// let taken = Snapshot::take("base", "2026-01-02T03:04:05Z", &path, &doc, &schematic)
///     .expect("takes");
/// assert_eq!(taken.objects.len(), 1);
/// assert!(taken.render().starts_with("snapshot base /a 2026-01-02T03:04:05Z kicli/"));
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Snapshot {
    /// The name the snapshot is filed under.
    pub name: String,
    /// The sheet path the objects belong to.
    pub sheet_path: SheetPath,
    /// When the snapshot was taken, as the caller reported it.
    pub taken: String,
    /// Which build of kicli took it.
    pub tool: String,
    /// The objects, sorted by key.
    pub objects: Vec<SnapshotObject>,
}

/// Why a snapshot could not be taken, written, or read.
#[derive(Debug, thiserror::Error)]
pub enum SnapshotError {
    /// A header value holds a space, a path separator, or nothing at all.
    #[error("the {field} cannot be {value:?}: it must be one word and not a path")]
    BadHeaderField {
        /// Which header value is wrong.
        field: &'static str,
        /// The value the caller offered.
        value: String,
    },
    /// The text is not a snapshot file.
    #[error("line {line} is not part of a snapshot: {reason}")]
    Malformed {
        /// The line number, counted from one.
        line: usize,
        /// What is wrong with the line.
        reason: String,
    },
    /// A snapshot file could not be written or read.
    #[error("cannot use {path}: {reason}")]
    Io {
        /// The file or directory that failed.
        path: PathBuf,
        /// What the operating system reported.
        reason: String,
    },
}

impl Snapshot {
    /// Hash every object of one sheet.
    ///
    /// `taken` is a timestamp the caller supplies. kicli never reads a clock
    /// of its own, so two runs over one design produce the same file.
    ///
    /// # Errors
    ///
    /// Returns [`SnapshotError::BadHeaderField`] when the name or the
    /// timestamp holds whitespace, holds a path separator, or is empty. The
    /// name becomes a file name, so a name that could leave the cache
    /// directory is refused.
    pub fn take(
        name: &str,
        taken: &str,
        sheet_path: &SheetPath,
        doc: &Doc,
        schematic: &Schematic,
    ) -> Result<Self, SnapshotError> {
        check_word("name", name)?;
        check_word("timestamp", taken)?;
        check_word("sheet path", &sheet_path.0)?;

        let mut objects = Vec::new();
        for item in &schematic.items {
            push_item(&mut objects, doc, item, sheet_path);
        }
        objects.sort_by(|left, right| left.key.cmp(&right.key));
        number_repeated_keys(&mut objects);

        Ok(Self {
            name: name.to_owned(),
            sheet_path: sheet_path.clone(),
            taken: taken.to_owned(),
            tool: format!("kicli/{}", crate::version()),
            objects,
        })
    }

    /// The object with one key.
    #[must_use]
    pub fn object(&self, key: &str) -> Option<&SnapshotObject> {
        self.objects
            .binary_search_by(|object| object.key.as_str().cmp(key))
            .ok()
            .map(|index| &self.objects[index])
    }

    /// The text of the snapshot file.
    ///
    /// The header names the snapshot, its sheet path, its timestamp and the
    /// build that took it. One line follows per object.
    #[must_use]
    pub fn render(&self) -> String {
        let mut text = format!(
            "snapshot {} {} {} {}\n",
            self.name, self.sheet_path, self.taken, self.tool
        );
        for object in &self.objects {
            let _ = writeln!(
                text,
                "{} {} {} {}",
                object.key,
                object.kind.token(),
                object.geometry,
                object.data
            );
        }
        text
    }

    /// Read a snapshot file.
    ///
    /// The file holds hashes only, so every object comes back without its
    /// [`Detail`]. A comparison against it names objects by their identifiers.
    ///
    /// # Errors
    ///
    /// Returns [`SnapshotError::Malformed`] when the header or an object line
    /// does not have the shape this module writes.
    pub fn parse(text: &str) -> Result<Self, SnapshotError> {
        let mut lines = text.lines().enumerate();
        let (_, header) = lines.next().ok_or_else(|| SnapshotError::Malformed {
            line: 1,
            reason: "the file is empty".to_owned(),
        })?;
        let header: Vec<&str> = header.split_whitespace().collect();
        if header.len() != 5 || header[0] != "snapshot" {
            return Err(SnapshotError::Malformed {
                line: 1,
                reason: "the header needs a name, a sheet path, a stamp and a tool".to_owned(),
            });
        }

        let mut objects = Vec::new();
        for (index, line) in lines {
            if line.trim().is_empty() {
                continue;
            }
            objects.push(read_object_line(line, index + 1)?);
        }
        objects.sort_by(|left, right| left.key.cmp(&right.key));

        Ok(Self {
            name: header[1].to_owned(),
            sheet_path: SheetPath(header[2].to_owned()),
            taken: header[3].to_owned(),
            tool: header[4].to_owned(),
            objects,
        })
    }

    /// Where a snapshot of one name lives inside a project directory.
    #[must_use]
    pub fn path_in(project: &Path, name: &str) -> PathBuf {
        project
            .join(CACHE_DIRECTORY)
            .join("snapshots")
            .join(format!("{name}.snap"))
    }

    /// Write the snapshot into the project's cache directory.
    ///
    /// Returns the file it wrote.
    ///
    /// # Errors
    ///
    /// Returns [`SnapshotError::Io`] when the directory cannot be made or the
    /// file cannot be written.
    pub fn write_in(&self, project: &Path) -> Result<PathBuf, SnapshotError> {
        let file = Self::path_in(project, &self.name);
        let directory = file.parent().unwrap_or(project).to_owned();
        std::fs::create_dir_all(&directory).map_err(|error| SnapshotError::Io {
            path: directory,
            reason: error.to_string(),
        })?;
        std::fs::write(&file, self.render()).map_err(|error| SnapshotError::Io {
            path: file.clone(),
            reason: error.to_string(),
        })?;
        Ok(file)
    }

    /// Read a snapshot of one name from the project's cache directory.
    ///
    /// # Errors
    ///
    /// Returns [`SnapshotError::Io`] when the file cannot be read, and
    /// [`SnapshotError::Malformed`] when it is not a snapshot.
    pub fn read_in(project: &Path, name: &str) -> Result<Self, SnapshotError> {
        let file = Self::path_in(project, name);
        let text = std::fs::read_to_string(&file).map_err(|error| SnapshotError::Io {
            path: file,
            reason: error.to_string(),
        })?;
        Self::parse(&text)
    }
}

/// Read one object line of a snapshot file.
///
/// The line is split from the right, because a field name may hold a space
/// and the key comes first.
fn read_object_line(line: &str, number: usize) -> Result<SnapshotObject, SnapshotError> {
    let malformed = |reason: &str| SnapshotError::Malformed {
        line: number,
        reason: reason.to_owned(),
    };
    let mut parts = line.rsplitn(4, ' ');
    let data = parts.next().ok_or_else(|| malformed("no data hash"))?;
    let geometry = parts.next().ok_or_else(|| malformed("no geometry hash"))?;
    let kind = parts.next().ok_or_else(|| malformed("no kind"))?;
    let key = parts.next().ok_or_else(|| malformed("no key"))?;

    Ok(SnapshotObject {
        key: key.to_owned(),
        kind: ObjectKind::from_token(kind),
        handle: handle_of_key(key),
        geometry: ContentHash::from_hex(geometry)
            .ok_or_else(|| malformed("the geometry hash is not sixteen hex characters"))?,
        data: ContentHash::from_hex(data)
            .ok_or_else(|| malformed("the data hash is not sixteen hex characters"))?,
        owner: key.split_once('.').map(|(owner, _)| owner.to_owned()),
        detail: None,
    })
}

/// The name to print for an object read from a file.
///
/// The file holds identifiers, so the handle is the short form of the
/// identifier, and the field name when the key names one.
fn handle_of_key(key: &str) -> String {
    match key.split_once('.') {
        Some((owner, field)) => format!("{}.{field}", short(owner)),
        None => short(key),
    }
}

/// Give repeated keys an index, so no two objects share one.
///
/// An object with no identifier of its own is keyed by its content. Two such
/// objects that are identical then share a key. They are interchangeable, so
/// numbering them in key order stays deterministic.
fn number_repeated_keys(objects: &mut [SnapshotObject]) {
    let mut start = 0;
    while start < objects.len() {
        let mut end = start + 1;
        while end < objects.len() && objects[end].key == objects[start].key {
            end += 1;
        }
        if end - start > 1 {
            for (index, object) in objects[start..end].iter_mut().enumerate() {
                let _ = write!(object.key, "#{index}");
            }
        }
        start = end;
    }
}

/// The first eight characters of an identifier.
fn short(uuid: &str) -> String {
    uuid.chars().take(8).collect()
}

/// Is a header value one word that names no path?
fn check_word(field: &'static str, value: &str) -> Result<(), SnapshotError> {
    let usable = !value.is_empty()
        && !value.chars().any(|c| c.is_whitespace() || c.is_control())
        && !value.contains("..");
    let path_free = field != "name" || !value.contains(['/', '\\']);
    if usable && path_free {
        return Ok(());
    }
    Err(SnapshotError::BadHeaderField {
        field,
        value: value.to_owned(),
    })
}

/// A point in millimetres, at the precision the views print.
pub(crate) fn millimetres(point: Point) -> String {
    format!("{:.2},{:.2}", point.x.millimetres(), point.y.millimetres())
}

/// `yes` or `no`, as KiCad writes a flag.
fn yes_no(flag: bool) -> &'static str {
    if flag { "yes" } else { "no" }
}

/// The bytes one hash covers.
///
/// Every value carries a tag and a length, so no two different objects can
/// produce the same bytes. The encoding starts with the object's kind, so a
/// junction and a no-connect at one point hash differently.
struct Encoding {
    hasher: Sha256,
}

impl Encoding {
    fn new(kind: &str) -> Self {
        let mut encoding = Self {
            hasher: Sha256::new(),
        };
        encoding.text("kind", kind);
        encoding
    }

    fn text(&mut self, tag: &str, value: &str) {
        self.hasher.update(tag.as_bytes());
        self.hasher.update(b"=");
        self.hasher.update(value.len().to_string().as_bytes());
        self.hasher.update(b":");
        self.hasher.update(value.as_bytes());
        self.hasher.update(b"\n");
    }

    fn number(&mut self, tag: &str, value: i64) {
        self.hasher.update(tag.as_bytes());
        self.hasher.update(b"=");
        self.hasher.update(value.to_string().as_bytes());
        self.hasher.update(b"\n");
    }

    fn point(&mut self, tag: &str, value: Point) {
        self.text("point", tag);
        self.number("x", value.x.0.into());
        self.number("y", value.y.0.into());
    }

    fn finish(self) -> ContentHash {
        let digest = self.hasher.finalize();
        let mut bytes = [0_u8; 8];
        bytes.copy_from_slice(&digest[..8]);
        ContentHash(bytes)
    }
}

/// Hash one object and everything it owns.
fn push_item(objects: &mut Vec<SnapshotObject>, doc: &Doc, item: &Item, path: &SheetPath) {
    match item {
        Item::Symbol(symbol) => push_symbol(objects, symbol, path),
        Item::Line(line) => objects.push(line_object(line)),
        Item::Junction(point) => objects.push(point_object(point, &ObjectKind::Junction)),
        Item::NoConnect(point) => objects.push(point_object(point, &ObjectKind::NoConnect)),
        Item::BusEntry(point) => objects.push(point_object(point, &ObjectKind::BusEntry)),
        Item::Label(label) => push_label(objects, label),
        Item::Text(text) => objects.push(text_object(text)),
        Item::Sheet(sheet) => push_sheet(objects, sheet),
        Item::Other { token, uuid, node } => {
            objects.push(other_object(doc, token, uuid.as_ref(), *node));
        }
    }
}

/// Hash a placed symbol, then its fields.
fn push_symbol(objects: &mut Vec<SnapshotObject>, symbol: &Symbol, path: &SheetPath) {
    let handle = symbol
        .reference_on(path)
        .map_or_else(|| short(&symbol.uuid.0), |reference| reference.0.clone());

    let mut geometry = Encoding::new("symbol");
    geometry.point("at", symbol.at);
    geometry.number("angle", symbol.angle.0.into());
    geometry.text(
        "mirror",
        match symbol.mirror {
            Some(Mirror::X) => "x",
            Some(Mirror::Y) => "y",
            None => "-",
        },
    );

    let value = symbol
        .field("Value")
        .map_or("", |field| field.value.as_str());
    let summary = if value.is_empty() {
        symbol.lib_id.0.clone()
    } else {
        format!("{value} {}", symbol.lib_id.0)
    };

    objects.push(SnapshotObject {
        key: symbol.uuid.0.clone(),
        kind: ObjectKind::Symbol,
        handle: handle.clone(),
        geometry: geometry.finish(),
        data: symbol_data(symbol),
        owner: None,
        detail: Some(Detail {
            at: Some(symbol.at),
            summary,
            value: None,
        }),
    });
    push_fields(objects, &symbol.uuid.0, &handle, symbol.at, &symbol.fields);
}

/// Hash what a placed symbol is, apart from where it sits.
///
/// The pins and the placements are sorted, because their order in the file
/// carries no meaning and KiCad is free to change it.
fn symbol_data(symbol: &Symbol) -> ContentHash {
    let mut data = Encoding::new("symbol");
    data.text("lib_id", &symbol.lib_id.0);
    data.text("lib_name", symbol.lib_name.as_deref().unwrap_or(""));
    data.number("unit", symbol.unit.into());
    data.number("body_style", symbol.body_style.into());
    data.text("dnp", yes_no(symbol.dnp));

    let mut pins: Vec<(&str, &str)> = symbol
        .pins
        .iter()
        .map(|pin| (pin.number.as_str(), pin.alternate.as_deref().unwrap_or("")))
        .collect();
    pins.sort_unstable();
    for (number, alternate) in pins {
        data.text("pin", number);
        data.text("alternate", alternate);
    }

    let mut places: Vec<(&str, &str, u32)> = symbol
        .placements
        .iter()
        .map(|place| {
            (
                place.path.0.as_str(),
                place.reference.0.as_str(),
                place.unit,
            )
        })
        .collect();
    places.sort_unstable();
    for (place, reference, unit) in places {
        data.text("path", place);
        data.text("reference", reference);
        data.number("unit", unit.into());
    }
    data.finish()
}

/// Hash the fields of one item.
fn push_fields(
    objects: &mut Vec<SnapshotObject>,
    owner_key: &str,
    owner_handle: &str,
    anchor: Point,
    fields: &[Field],
) {
    for field in fields {
        let offset = offset_from(anchor, field.at);
        let mut geometry = Encoding::new("field");
        geometry.point("offset", offset);
        geometry.number("angle", field.angle.0.into());

        let mut data = Encoding::new("field");
        data.text("name", &field.name);
        data.text("value", &field.value);
        data.text("hidden", yes_no(field.hidden));

        objects.push(SnapshotObject {
            key: format!("{owner_key}.{}", field.name),
            kind: ObjectKind::Field,
            handle: format!("{owner_handle}.{}", field.name),
            geometry: geometry.finish(),
            data: data.finish(),
            owner: Some(owner_key.to_owned()),
            detail: Some(Detail {
                at: Some(offset),
                summary: format!("{:?}", field.value),
                value: Some(field.value.clone()),
            }),
        });
    }
}

/// Hash a wire or bus segment.
///
/// The two ends are sorted, because a segment drawn the other way round is the
/// same segment and KiCad may write either order.
fn line_object(line: &Line) -> SnapshotObject {
    let (first, second) = if line.from <= line.to {
        (line.from, line.to)
    } else {
        (line.to, line.from)
    };
    let kind = match line.kind {
        LineKind::Wire => ObjectKind::Wire,
        LineKind::Bus => ObjectKind::Bus,
    };

    let mut geometry = Encoding::new(kind.token());
    geometry.point("end", first);
    geometry.point("end", second);

    SnapshotObject {
        key: line.uuid.0.clone(),
        handle: short(&line.uuid.0),
        geometry: geometry.finish(),
        data: Encoding::new(kind.token()).finish(),
        kind,
        owner: None,
        detail: Some(Detail {
            at: Some(first),
            summary: format!("{}..{}", millimetres(first), millimetres(second)),
            value: None,
        }),
    }
}

/// Hash a junction, a no-connect, or a bus entry.
fn point_object(item: &PointItem, kind: &ObjectKind) -> SnapshotObject {
    let mut geometry = Encoding::new(kind.token());
    geometry.point("at", item.at);

    SnapshotObject {
        key: item.uuid.0.clone(),
        kind: kind.clone(),
        handle: short(&item.uuid.0),
        geometry: geometry.finish(),
        data: Encoding::new(kind.token()).finish(),
        owner: None,
        detail: Some(Detail {
            at: Some(item.at),
            summary: millimetres(item.at),
            value: None,
        }),
    }
}

/// Hash a label, then the fields it owns.
fn push_label(objects: &mut Vec<SnapshotObject>, label: &Label) {
    let kind = match label.kind {
        LabelKind::Local => ObjectKind::Label,
        LabelKind::Global => ObjectKind::GlobalLabel,
        LabelKind::Hierarchical => ObjectKind::HierarchicalLabel,
        LabelKind::NetclassFlag => ObjectKind::NetclassFlag,
    };

    let mut geometry = Encoding::new(kind.token());
    geometry.point("at", label.at);
    geometry.number("angle", label.angle.0.into());

    let mut data = Encoding::new(kind.token());
    data.text("text", &label.text);
    data.text("shape", label.shape.as_deref().unwrap_or(""));

    let handle = short(&label.uuid.0);
    objects.push(SnapshotObject {
        key: label.uuid.0.clone(),
        kind,
        handle: handle.clone(),
        geometry: geometry.finish(),
        data: data.finish(),
        owner: None,
        detail: Some(Detail {
            at: Some(label.at),
            summary: format!("{:?}", label.text),
            value: Some(label.text.clone()),
        }),
    });
    push_fields(objects, &label.uuid.0, &handle, label.at, &label.fields);
}

/// Hash free text or a text box.
fn text_object(text: &TextItem) -> SnapshotObject {
    let kind = if text.boxed {
        ObjectKind::TextBox
    } else {
        ObjectKind::Text
    };

    let mut geometry = Encoding::new(kind.token());
    geometry.point("at", text.at);
    geometry.number("angle", text.angle.0.into());

    let mut data = Encoding::new(kind.token());
    data.text("text", &text.text);

    SnapshotObject {
        key: text.uuid.0.clone(),
        kind,
        handle: short(&text.uuid.0),
        geometry: geometry.finish(),
        data: data.finish(),
        owner: None,
        detail: Some(Detail {
            at: Some(text.at),
            summary: format!("{:?}", text.text),
            value: Some(text.text.clone()),
        }),
    }
}

/// Hash a child sheet, then its fields and its pins.
fn push_sheet(objects: &mut Vec<SnapshotObject>, sheet: &SheetItem) {
    let mut geometry = Encoding::new("sheet");
    geometry.point("at", sheet.at);
    geometry.number("width", sheet.size.0.0.into());
    geometry.number("height", sheet.size.1.0.into());

    let mut data = Encoding::new("sheet");
    let mut pages: Vec<(&str, &str)> = sheet
        .pages
        .iter()
        .map(|page| (page.path.0.as_str(), page.page.as_str()))
        .collect();
    pages.sort_unstable();
    for (path, page) in pages {
        data.text("path", path);
        data.text("page", page);
    }

    let handle = sheet
        .name()
        .map_or_else(|| short(&sheet.uuid.0), str::to_owned);
    objects.push(SnapshotObject {
        key: sheet.uuid.0.clone(),
        kind: ObjectKind::Sheet,
        handle: handle.clone(),
        geometry: geometry.finish(),
        data: data.finish(),
        owner: None,
        detail: Some(Detail {
            at: Some(sheet.at),
            summary: sheet.file().unwrap_or_default().to_owned(),
            value: None,
        }),
    });
    push_fields(objects, &sheet.uuid.0, &handle, sheet.at, &sheet.fields);
    for pin in &sheet.pins {
        objects.push(sheet_pin_object(pin, &sheet.uuid, &handle, sheet.at));
    }
}

/// Hash one pin on the border of a child sheet.
fn sheet_pin_object(
    pin: &SheetPin,
    sheet_key: &Uuid,
    sheet_handle: &str,
    anchor: Point,
) -> SnapshotObject {
    let offset = offset_from(anchor, pin.at);
    let mut geometry = Encoding::new("sheet_pin");
    geometry.point("offset", offset);
    geometry.number("angle", pin.angle.0.into());

    let mut data = Encoding::new("sheet_pin");
    data.text("name", &pin.name);
    data.text("direction", &pin.direction);

    SnapshotObject {
        key: pin.uuid.0.clone(),
        kind: ObjectKind::SheetPin,
        handle: format!("{sheet_handle}.{}", pin.name),
        geometry: geometry.finish(),
        data: data.finish(),
        owner: Some(sheet_key.0.clone()),
        detail: Some(Detail {
            at: Some(offset),
            summary: format!("{} ({})", pin.name, pin.direction),
            value: None,
        }),
    }
}

/// Hash an object kicli does not name.
///
/// kicli cannot tell which tokens of such an object are its position, so the
/// whole of it counts as data. A graphic that moves therefore reports as
/// edited rather than moved.
fn other_object(doc: &Doc, token: &str, uuid: Option<&Uuid>, node: NodeId) -> SnapshotObject {
    let mut data = Encoding::new(token);
    encode_tree(&mut data, doc, node);
    let data = data.finish();

    // An object with no identifier of its own is keyed by what it holds. That
    // is the only handle available, and it keeps the object in the snapshot
    // rather than dropping it.
    let key = uuid.map_or_else(|| format!("{token}@{data}"), |uuid| uuid.0.clone());

    SnapshotObject {
        key: key.clone(),
        kind: ObjectKind::Other(token.to_owned()),
        handle: short(&key),
        geometry: Encoding::new(token).finish(),
        data,
        owner: None,
        detail: Some(Detail {
            at: None,
            summary: token.to_owned(),
            value: None,
        }),
    }
}

/// Encode one subtree, in file order.
///
/// A number is encoded as internal units, so a file that writes `1.270`
/// instead of `1.27` hashes the same.
fn encode_tree(encoding: &mut Encoding, doc: &Doc, node: NodeId) {
    match doc.node(node) {
        Node::List { .. } => {
            encoding.text("list", "");
            for &child in doc.children(node) {
                encode_tree(encoding, doc, child);
            }
            encoding.text("end", "");
        }
        Node::Atom {
            kind: AtomKind::Bare,
            ..
        } => {
            let text = doc.atom_text(node).unwrap_or_default();
            match kicli_sexpr::parse_iu(text) {
                Some(units) => encoding.number("number", units.into()),
                None => encoding.text("token", text),
            }
        }
        Node::Atom { .. } => encoding.text("string", &doc.atom_as_str(node).unwrap_or_default()),
        // KiCad drops comments on save, so they are not content.
        Node::Comment { .. } => {}
    }
}

/// Where a point sits relative to the item that owns it.
fn offset_from(anchor: Point, at: Point) -> Point {
    Point {
        x: Iu(at.x.0.saturating_sub(anchor.x.0)),
        y: Iu(at.y.0.saturating_sub(anchor.y.0)),
    }
}

#[cfg(test)]
mod tests {
    use super::{ContentHash, Encoding, ObjectKind, check_word, handle_of_key, millimetres};
    use crate::geometry::Point;

    #[test]
    fn a_hash_is_sixteen_hexadecimal_characters() {
        let hash = Encoding::new("symbol").finish();
        let text = hash.to_string();
        assert_eq!(text.len(), 16);
        assert!(text.chars().all(|c| c.is_ascii_hexdigit()));
        assert_eq!(ContentHash::from_hex(&text), Some(hash));
        assert_eq!(ContentHash::from_hex("nope"), None);
    }

    #[test]
    fn the_kind_starts_the_encoding() {
        assert_ne!(
            Encoding::new("junction").finish(),
            Encoding::new("no_connect").finish()
        );
    }

    #[test]
    fn a_tagged_value_cannot_be_confused_with_its_neighbour() {
        let mut split = Encoding::new("field");
        split.text("name", "ab");
        split.text("value", "c");
        let mut joined = Encoding::new("field");
        joined.text("name", "a");
        joined.text("value", "bc");
        assert_ne!(split.finish(), joined.finish());
    }

    #[test]
    fn a_kind_word_survives_a_write_and_a_read() {
        for kind in [
            ObjectKind::Symbol,
            ObjectKind::Field,
            ObjectKind::SheetPin,
            ObjectKind::Other("polyline".to_owned()),
        ] {
            assert_eq!(ObjectKind::from_token(kind.token()), kind);
        }
    }

    #[test]
    fn a_name_that_could_leave_the_cache_is_refused() {
        assert!(check_word("name", "last-write").is_ok());
        assert!(check_word("name", "../escape").is_err());
        assert!(check_word("name", "two words").is_err());
        assert!(check_word("name", "").is_err());
        // A sheet path holds slashes and is not a file name.
        assert!(check_word("sheet path", "/root/child").is_ok());
    }

    #[test]
    fn a_key_read_from_a_file_shortens_to_a_handle() {
        assert_eq!(handle_of_key("0123456789abcdef"), "01234567");
        assert_eq!(handle_of_key("0123456789abcdef.Value"), "01234567.Value");
    }

    #[test]
    fn a_point_prints_in_millimetres() {
        assert_eq!(
            millimetres(Point::new(1_270_000, -889_000)),
            "127.00,-88.90"
        );
    }
}
