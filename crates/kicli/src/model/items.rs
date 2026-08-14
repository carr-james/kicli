//! The objects a schematic file holds, read out of the token tree.
//!
//! Every object keeps the tree node it came from, so a later edit changes the
//! file rather than a copy of it. A token this module does not name is kept as
//! [`Item::Other`] with its head token, because KiCad's parser accepts more
//! tokens than any corpus exercises and dropping one silently would be the
//! failure this tool exists to prevent.

use crate::geometry::{Angle, Iu, Point};
use crate::model::version::{FormatVersion, pin_text};
use kicli_sexpr::{Doc, NodeId, SexprError};

/// An object's own identifier, as KiCad writes it.
///
/// KiCad calls it a KIID. It is unique within a file and is the handle every
/// other part of kicli addresses an object by.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Uuid(pub String);

/// A reference designator, such as `R12`.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Refdes(pub String);

/// A library identifier, such as `Device:R`.
///
/// The nickname is everything before the **first** colon. A symbol name may
/// itself contain a colon, so splitting at the last one is wrong.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LibId(pub String);

impl LibId {
    /// The library nickname, when the identifier carries one.
    #[must_use]
    pub fn nickname(&self) -> Option<&str> {
        self.0.split_once(':').map(|(nickname, _)| nickname)
    }

    /// The symbol name inside its library.
    #[must_use]
    pub fn symbol_name(&self) -> &str {
        self.0.split_once(':').map_or(&self.0, |(_, name)| name)
    }
}

/// A path through the sheet hierarchy: the root screen, then each sheet item.
///
/// KiCad writes it as `/<root uuid>/<sheet uuid>...`. It identifies one
/// *placement* of a sheet, so a sheet drawn once but placed twice has two.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SheetPath(pub String);

impl std::fmt::Display for SheetPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl SheetPath {
    /// The path of the root sheet of a hierarchy.
    #[must_use]
    pub fn root(root_screen: &Uuid) -> Self {
        Self(format!("/{}", root_screen.0))
    }

    /// This path with one more sheet item appended.
    #[must_use]
    pub fn child(&self, sheet_item: &Uuid) -> Self {
        Self(format!("{}/{}", self.0, sheet_item.0))
    }

    /// The uuids along the path, root screen first.
    pub fn segments(&self) -> impl Iterator<Item = &str> {
        self.0.split('/').filter(|segment| !segment.is_empty())
    }
}

/// Which way a symbol is flipped, after its rotation is applied.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mirror {
    /// Mirrored about the X axis.
    X,
    /// Mirrored about the Y axis.
    Y,
}

/// Which of the four label-like tokens an object is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LabelKind {
    /// A local label. It names a net within one sheet.
    Local,
    /// A global label. It names a net across the whole project.
    Global,
    /// A hierarchical label. It meets the parent sheet's like-named pin.
    Hierarchical,
    /// A netclass flag. It carries a netclass name, not a net name.
    NetclassFlag,
}

/// Whether a connection line carries one net or a bundle of them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LineKind {
    /// A wire: one net.
    Wire,
    /// A bus: a bundle, named with bracket notation.
    Bus,
}

/// A text field owned by a symbol, sheet, global label or netclass flag.
///
/// A field is movable text in its own right, with its own position, angle and
/// visibility, which is what makes field placement a first-class operation.
#[derive(Clone, Debug)]
pub struct Field {
    /// The field's name, such as `Reference`.
    pub name: String,
    /// The field's value.
    pub value: String,
    /// Where the text is drawn, in absolute schematic coordinates.
    pub at: Point,
    /// The text's own angle, independent of its owner's rotation.
    pub angle: Angle,
    /// Is the field hidden?
    pub hidden: bool,
    /// The justification tokens, as the file writes them.
    ///
    /// A centred edge writes no token, so an empty list means centred on both
    /// axes. The tokens decide where the text sits about its anchor, which
    /// makes them part of the field's placement.
    pub justify: Vec<String>,
    /// The `property` list this field was read from.
    pub node: NodeId,
}

/// One pin of a placed symbol, as the schematic records it.
///
/// The record carries the pin's number and its own uuid. Where the pin sits is
/// a question for the geometry module, which needs the library symbol as well.
#[derive(Clone, Debug)]
pub struct PinInstance {
    /// The pin number, as text: pins are named `A1` as often as `1`.
    pub number: String,
    /// The pin's own identifier, which the rule check reports.
    pub uuid: Uuid,
    /// The selected alternate pin function, when one is chosen.
    pub alternate: Option<String>,
    /// The `pin` list this was read from.
    pub node: NodeId,
}

/// What one placement of a symbol is called, on one sheet path.
#[derive(Clone, Debug)]
pub struct SymbolPlacement {
    /// The project the path belongs to.
    pub project: String,
    /// The sheet path this reference applies to.
    pub path: SheetPath,
    /// The reference designator on that path.
    pub reference: Refdes,
    /// The unit of a multi-unit part on that path.
    pub unit: u32,
}

/// A placed symbol.
#[derive(Clone, Debug)]
pub struct Symbol {
    /// The symbol's identifier.
    pub uuid: Uuid,
    /// The library identifier the symbol was placed from.
    pub lib_id: LibId,
    /// The key into the file's embedded library cache, when it differs from
    /// `lib_id`. A symbol edited in place gets a uniquified cache entry, and
    /// resolving by `lib_id` alone then finds the wrong definition.
    pub lib_name: Option<String>,
    /// The symbol anchor.
    pub at: Point,
    /// The rotation written in the file: 0, 90, 180 or 270.
    pub angle: Angle,
    /// The mirror written in the file, applied after the rotation.
    pub mirror: Option<Mirror>,
    /// Which unit of a multi-unit part this is.
    pub unit: u32,
    /// Which body style: 1 is normal, 2 is the De Morgan alternative.
    pub body_style: u32,
    /// Is the part marked do-not-populate?
    pub dnp: bool,
    /// The symbol's fields, in file order.
    pub fields: Vec<Field>,
    /// The symbol's pins, in file order.
    pub pins: Vec<PinInstance>,
    /// One entry per sheet path this symbol appears on.
    pub placements: Vec<SymbolPlacement>,
    /// The `symbol` list this was read from.
    pub node: NodeId,
}

impl Symbol {
    /// The value of a field, by name.
    #[must_use]
    pub fn field(&self, name: &str) -> Option<&Field> {
        self.fields.iter().find(|field| field.name == name)
    }

    /// The reference designator for one sheet path.
    ///
    /// The cached `Reference` field holds the value for whichever sheet was
    /// loaded last, so it is never the answer on its own. A symbol on a sheet
    /// placed twice has two references, and only the placement list holds both.
    #[must_use]
    pub fn reference_on(&self, path: &SheetPath) -> Option<&Refdes> {
        self.placements
            .iter()
            .find(|placement| &placement.path == path)
            .map(|placement| &placement.reference)
    }

    /// Is this a power symbol?
    ///
    /// Power symbols carry the net name in their `Value` field and their
    /// reference starts with `#`. They are net-name carriers, not parts.
    #[must_use]
    pub fn is_power(&self) -> bool {
        self.field("Reference")
            .is_some_and(|field| field.value.starts_with('#'))
    }
}

/// A wire or bus segment. KiCad splits polylines, so a segment has two ends.
#[derive(Clone, Debug)]
pub struct Line {
    /// The segment's identifier.
    pub uuid: Uuid,
    /// One net, or a bundle.
    pub kind: LineKind,
    /// One end.
    pub from: Point,
    /// The other end.
    pub to: Point,
    /// The `wire` or `bus` list this was read from.
    pub node: NodeId,
}

/// A junction, no-connect, or bus entry: an item defined by one point.
#[derive(Clone, Debug)]
pub struct PointItem {
    /// The item's identifier.
    pub uuid: Uuid,
    /// Where it sits.
    pub at: Point,
    /// The list this was read from.
    pub node: NodeId,
}

/// A label of any of the four kinds.
#[derive(Clone, Debug)]
pub struct Label {
    /// The label's identifier.
    pub uuid: Uuid,
    /// Which kind of label this is.
    pub kind: LabelKind,
    /// The text, which for the first three kinds is the net name.
    pub text: String,
    /// The label anchor, which is the point that connects.
    pub at: Point,
    /// The text angle.
    pub angle: Angle,
    /// The port direction, for global and hierarchical labels.
    pub shape: Option<String>,
    /// Fields the label owns, such as a global label's intersheet references.
    pub fields: Vec<Field>,
    /// The list this was read from.
    pub node: NodeId,
}

/// Free text, or a text box.
#[derive(Clone, Debug)]
pub struct TextItem {
    /// The item's identifier.
    pub uuid: Uuid,
    /// The text as written, with escapes resolved.
    pub text: String,
    /// Where the text is drawn.
    pub at: Point,
    /// The text angle.
    pub angle: Angle,
    /// Does the item draw a box around the text?
    pub boxed: bool,
    /// The width and height of a text box. Free text has none.
    ///
    /// A box that cannot be resized is not parity with the editor, and a
    /// snapshot that cannot see the size reports a resize as no change at all.
    pub size: Option<(Iu, Iu)>,
    /// The list this was read from.
    pub node: NodeId,
}

/// One pin on the border of a sheet symbol.
#[derive(Clone, Debug)]
pub struct SheetPin {
    /// The pin's identifier.
    pub uuid: Uuid,
    /// The pin name, which must match a hierarchical label in the child sheet.
    pub name: String,
    /// The direction, written as a positional token rather than a `shape`
    /// list, unlike a hierarchical label's.
    pub direction: String,
    /// Where the pin sits on the border.
    pub at: Point,
    /// The pin's angle.
    pub angle: Angle,
    /// The `pin` list this was read from.
    pub node: NodeId,
}

/// Where one placement of a sheet sits in the page order.
#[derive(Clone, Debug)]
pub struct SheetPage {
    /// The project the path belongs to.
    pub project: String,
    /// The path of the **parent** sheet. A sheet item's own uuid is not part of
    /// it, unlike a symbol's placement path.
    pub path: SheetPath,
    /// The page number, as written. It is text: pages are numbered `1`, `2` and
    /// also `A4` in some projects.
    pub page: String,
}

/// A sheet item: a reference from this file to a child file.
#[derive(Clone, Debug)]
pub struct SheetItem {
    /// The sheet item's identifier, which forms part of a sheet path.
    pub uuid: Uuid,
    /// The sheet's top-left corner.
    pub at: Point,
    /// The sheet's width and height.
    pub size: (Iu, Iu),
    /// The sheet's fields, which include `Sheetname` and `Sheetfile`.
    pub fields: Vec<Field>,
    /// The sheet's pins.
    pub pins: Vec<SheetPin>,
    /// One entry per placement of this sheet, giving its page number.
    pub pages: Vec<SheetPage>,
    /// The `sheet` list this was read from.
    pub node: NodeId,
}

impl SheetItem {
    /// The sheet's name, as drawn.
    #[must_use]
    pub fn name(&self) -> Option<&str> {
        self.field_value("Sheetname")
    }

    /// The child file this sheet names, relative to this file's directory.
    #[must_use]
    pub fn file(&self) -> Option<&str> {
        self.field_value("Sheetfile")
    }

    fn field_value(&self, name: &str) -> Option<&str> {
        self.fields
            .iter()
            .find(|field| field.name == name)
            .map(|field| field.value.as_str())
    }
}

/// One object of a schematic file.
#[derive(Clone, Debug)]
pub enum Item {
    /// A placed symbol.
    Symbol(Symbol),
    /// A wire or bus segment.
    Line(Line),
    /// A junction.
    Junction(PointItem),
    /// A no-connect marker.
    NoConnect(PointItem),
    /// A bus entry.
    BusEntry(PointItem),
    /// A label of any kind.
    Label(Label),
    /// Free text or a text box.
    Text(TextItem),
    /// A child sheet.
    Sheet(SheetItem),
    /// Anything else: graphics, images, tables, and tokens kicli does not name.
    ///
    /// The variant carries the head token so a caller can report what it saw,
    /// and the node so the object still round-trips.
    Other {
        /// The list's head token, such as `polyline`.
        token: String,
        /// The object's identifier, when it has one.
        uuid: Option<Uuid>,
        /// The list this was read from.
        node: NodeId,
    },
}

impl Item {
    /// The object's identifier, when it has one.
    #[must_use]
    pub fn uuid(&self) -> Option<&Uuid> {
        match self {
            Item::Symbol(symbol) => Some(&symbol.uuid),
            Item::Line(line) => Some(&line.uuid),
            Item::Junction(item) | Item::NoConnect(item) | Item::BusEntry(item) => Some(&item.uuid),
            Item::Label(label) => Some(&label.uuid),
            Item::Text(text) => Some(&text.uuid),
            Item::Sheet(sheet) => Some(&sheet.uuid),
            Item::Other { uuid, .. } => uuid.as_ref(),
        }
    }

    /// The tree node the object was read from.
    #[must_use]
    pub fn node(&self) -> NodeId {
        match self {
            Item::Symbol(symbol) => symbol.node,
            Item::Line(line) => line.node,
            Item::Junction(item) | Item::NoConnect(item) | Item::BusEntry(item) => item.node,
            Item::Label(label) => label.node,
            Item::Text(text) => text.node,
            Item::Sheet(sheet) => sheet.node,
            Item::Other { node, .. } => *node,
        }
    }
}

/// One schematic file, read into typed objects.
///
/// # Examples
///
/// ```
/// use kicli_sexpr::Doc;
/// use kicli::model::Schematic;
///
/// let source = "(kicad_sch\n\t(version 20260306)\n\t(paper \"A4\")\n)\n";
/// let doc = Doc::parse(source).expect("parses");
/// let schematic = Schematic::read(&doc).expect("reads");
/// assert_eq!(schematic.paper.as_deref(), Some("A4"));
/// assert!(schematic.items.is_empty());
/// ```
#[derive(Clone, Debug)]
pub struct Schematic {
    /// The file's format stamp, which decides what some tokens mean.
    pub version: FormatVersion,
    /// This screen's own identifier, which starts every sheet path.
    pub uuid: Option<Uuid>,
    /// The page size, as written.
    pub paper: Option<String>,
    /// The objects of the file, in file order.
    pub items: Vec<Item>,
    /// The embedded library cache, keyed the way the file keys it.
    pub library_symbols: Vec<(String, NodeId)>,
}

/// Objects kicli copies rather than reads, so their numbers are not its own.
///
/// KiCad's `image` carries a placement and a scale at full float precision,
/// which no internal unit holds exactly. kicli never interprets either: the
/// object is kept as [`Item::Other`] and written back from the bytes it came
/// from, so refusing the file would refuse a drawing kicli handles correctly.
/// Every other object in KiCad's demo corpus measures in units kicli holds.
const KEPT_VERBATIM: &[&str] = &["image"];

/// Why a schematic could not be read.
#[derive(Debug, thiserror::Error)]
pub enum ReadError {
    /// The file's outermost list is not `kicad_sch`.
    #[error("this is not a schematic: the file starts with {0}")]
    NotASchematic(String),
    /// The file has no outermost list at all.
    #[error("this file is empty")]
    Empty,
    /// The file carries no `(version ...)` stamp, so its tokens cannot be read.
    #[error("this schematic has no version stamp")]
    NoVersion,
    /// A measurement in the file is not one kicli can represent exactly.
    ///
    /// Reading it as zero would move the item somewhere its author never put
    /// it, and a later write would save that move as though it were meant.
    #[error("{0}")]
    BadNumber(#[from] SexprError),
}

impl Schematic {
    /// Read a parsed file into typed objects.
    ///
    /// # Errors
    ///
    /// Returns [`ReadError`] when the document is not a schematic, or carries
    /// no version stamp, or carries a measurement kicli cannot represent
    /// exactly. Objects the module does not name are kept as [`Item::Other`]
    /// rather than refused.
    pub fn read(doc: &Doc) -> Result<Self, ReadError> {
        let root = doc.root().ok_or(ReadError::Empty)?;
        match doc.head(root) {
            Some("kicad_sch") => {}
            Some(other) => return Err(ReadError::NotASchematic(other.to_owned())),
            None => return Err(ReadError::Empty),
        }
        // Every measurement is checked before any of them is read. Below this
        // line a reader may treat a missing number as absent, because an
        // unreadable one has already stopped the load.
        doc.check_measurements(KEPT_VERBATIM)?;

        let version = FormatVersion::new(
            child_atom(doc, root, "version")
                .and_then(|text| text.parse().ok())
                .ok_or(ReadError::NoVersion)?,
        );

        let mut schematic = Self {
            version,
            uuid: child_atom_string(doc, root, "uuid").map(Uuid),
            paper: child_atom_string(doc, root, "paper"),
            items: Vec::new(),
            library_symbols: Vec::new(),
        };

        for &child in doc.children(root) {
            let Some(head) = doc.head(child) else {
                continue;
            };
            match head {
                // Header lists, already read above.
                "version" | "generator" | "generator_version" | "uuid" | "paper"
                | "title_block" | "sheet_instances" | "embedded_fonts" => {}
                "lib_symbols" => {
                    for &entry in doc.children(child) {
                        if let Some(name) =
                            first_atom(doc, entry).and_then(|id| doc.atom_as_str(id))
                        {
                            schematic.library_symbols.push((name, entry));
                        }
                    }
                }
                _ => schematic.items.push(read_item(doc, child, head, version)),
            }
        }
        Ok(schematic)
    }

    /// The placed symbols, in file order.
    pub fn symbols(&self) -> impl Iterator<Item = &Symbol> {
        self.items.iter().filter_map(|item| match item {
            Item::Symbol(symbol) => Some(symbol),
            _ => None,
        })
    }

    /// The wire and bus segments, in file order.
    pub fn lines(&self) -> impl Iterator<Item = &Line> {
        self.items.iter().filter_map(|item| match item {
            Item::Line(line) => Some(line),
            _ => None,
        })
    }

    /// The labels of every kind, in file order.
    pub fn labels(&self) -> impl Iterator<Item = &Label> {
        self.items.iter().filter_map(|item| match item {
            Item::Label(label) => Some(label),
            _ => None,
        })
    }

    /// The child sheets, in file order.
    pub fn sheets(&self) -> impl Iterator<Item = &SheetItem> {
        self.items.iter().filter_map(|item| match item {
            Item::Sheet(sheet) => Some(sheet),
            _ => None,
        })
    }

    /// The junctions, in file order.
    pub fn junctions(&self) -> impl Iterator<Item = &PointItem> {
        self.items.iter().filter_map(|item| match item {
            Item::Junction(junction) => Some(junction),
            _ => None,
        })
    }
}

/// Read one top-level object.
fn read_item(doc: &Doc, node: NodeId, head: &str, version: FormatVersion) -> Item {
    let uuid = child_atom_string(doc, node, "uuid").map(Uuid);
    match head {
        "symbol" => read_symbol(doc, node, uuid, version),
        "wire" | "bus" => read_line(doc, node, uuid, head),
        "junction" | "no_connect" | "bus_entry" => {
            let item = PointItem {
                uuid: uuid.unwrap_or_else(|| Uuid(String::new())),
                at: child_point(doc, node).unwrap_or_default(),
                node,
            };
            match head {
                "junction" => Item::Junction(item),
                "no_connect" => Item::NoConnect(item),
                _ => Item::BusEntry(item),
            }
        }
        "label" | "global_label" | "hierarchical_label" | "netclass_flag" => {
            read_label(doc, node, uuid, head, version)
        }
        "text" | "text_box" => Item::Text(TextItem {
            uuid: uuid.unwrap_or_else(|| Uuid(String::new())),
            text: first_atom(doc, node)
                .and_then(|id| doc.atom_as_str(id))
                .unwrap_or_default(),
            at: child_point(doc, node).unwrap_or_default(),
            angle: child_angle(doc, node).unwrap_or_default(),
            boxed: head == "text_box",
            size: child_size(doc, node),
            node,
        }),
        "sheet" => read_sheet(doc, node, uuid, version),
        _ => Item::Other {
            token: head.to_owned(),
            uuid,
            node,
        },
    }
}

fn read_symbol(doc: &Doc, node: NodeId, uuid: Option<Uuid>, version: FormatVersion) -> Item {
    let mut symbol = Symbol {
        uuid: uuid.unwrap_or_else(|| Uuid(String::new())),
        lib_id: LibId(child_atom_string(doc, node, "lib_id").unwrap_or_default()),
        lib_name: child_atom_string(doc, node, "lib_name"),
        at: child_point(doc, node).unwrap_or_default(),
        angle: child_angle(doc, node).unwrap_or_default(),
        mirror: match child_atom(doc, node, "mirror") {
            Some("x") => Some(Mirror::X),
            Some("y") => Some(Mirror::Y),
            _ => None,
        },
        unit: child_number(doc, node, "unit").unwrap_or(1),
        body_style: child_number(doc, node, "body_style").unwrap_or(1),
        dnp: child_atom(doc, node, "dnp") == Some("yes"),
        fields: read_fields(doc, node, version),
        pins: Vec::new(),
        placements: Vec::new(),
        node,
    };

    for &child in doc.children(node) {
        match doc.head(child) {
            Some("pin") => {
                let Some(number) = first_atom(doc, child).and_then(|id| doc.atom_as_str(id)) else {
                    continue;
                };
                symbol.pins.push(PinInstance {
                    number: pin_text(&number, version).to_owned(),
                    uuid: Uuid(child_atom_string(doc, child, "uuid").unwrap_or_default()),
                    alternate: child_atom_string(doc, child, "alternate"),
                    node: child,
                });
            }
            Some("instances") => symbol.placements = read_placements(doc, child),
            _ => {}
        }
    }
    Item::Symbol(symbol)
}

fn read_placements(doc: &Doc, instances: NodeId) -> Vec<SymbolPlacement> {
    let mut placements = Vec::new();
    for &project_node in doc.children(instances) {
        if !doc.head_is(project_node, "project") {
            continue;
        }
        let project = first_atom(doc, project_node)
            .and_then(|id| doc.atom_as_str(id))
            .unwrap_or_default();
        for &path_node in doc.children(project_node) {
            if !doc.head_is(path_node, "path") {
                continue;
            }
            let Some(path) = first_atom(doc, path_node).and_then(|id| doc.atom_as_str(id)) else {
                continue;
            };
            placements.push(SymbolPlacement {
                project: project.clone(),
                path: SheetPath(path),
                reference: Refdes(
                    child_atom_string(doc, path_node, "reference").unwrap_or_default(),
                ),
                unit: child_number(doc, path_node, "unit").unwrap_or(1),
            });
        }
    }
    placements
}

fn read_line(doc: &Doc, node: NodeId, uuid: Option<Uuid>, head: &str) -> Item {
    let mut ends = Vec::new();
    for &child in doc.children(node) {
        if !doc.head_is(child, "pts") {
            continue;
        }
        for &point in doc.children(child) {
            if doc.head_is(point, "xy") {
                if let Some(at) = point_of(doc, point) {
                    ends.push(at);
                }
            }
        }
    }
    Item::Line(Line {
        uuid: uuid.unwrap_or_else(|| Uuid(String::new())),
        kind: if head == "bus" {
            LineKind::Bus
        } else {
            LineKind::Wire
        },
        from: ends.first().copied().unwrap_or_default(),
        to: ends.last().copied().unwrap_or_default(),
        node,
    })
}

fn read_label(
    doc: &Doc,
    node: NodeId,
    uuid: Option<Uuid>,
    head: &str,
    version: FormatVersion,
) -> Item {
    let kind = match head {
        "global_label" => LabelKind::Global,
        "hierarchical_label" => LabelKind::Hierarchical,
        "netclass_flag" => LabelKind::NetclassFlag,
        _ => LabelKind::Local,
    };
    Item::Label(Label {
        uuid: uuid.unwrap_or_else(|| Uuid(String::new())),
        kind,
        text: first_atom(doc, node)
            .and_then(|id| doc.atom_as_str(id))
            .unwrap_or_default(),
        at: child_point(doc, node).unwrap_or_default(),
        angle: child_angle(doc, node).unwrap_or_default(),
        shape: child_atom_string(doc, node, "shape"),
        fields: read_fields(doc, node, version),
        node,
    })
}

fn read_sheet(doc: &Doc, node: NodeId, uuid: Option<Uuid>, version: FormatVersion) -> Item {
    // A sheet symbol with no size is drawn as a point, which is what KiCad
    // shows for one. A size that is there and unreadable never reaches this:
    // the file is refused when it is read.
    let size = child_size(doc, node).unwrap_or_default();

    let mut pins = Vec::new();
    for &child in doc.children(node) {
        if !doc.head_is(child, "pin") {
            continue;
        }
        let values = doc.children(child);
        pins.push(SheetPin {
            uuid: Uuid(child_atom_string(doc, child, "uuid").unwrap_or_default()),
            name: values
                .get(1)
                .and_then(|&id| doc.atom_as_str(id))
                .unwrap_or_default(),
            // The direction is a bare token in position two, unlike a
            // hierarchical label's, which is a (shape ...) list.
            direction: values
                .get(2)
                .and_then(|&id| doc.atom_text(id))
                .unwrap_or_default()
                .to_owned(),
            at: child_point(doc, child).unwrap_or_default(),
            angle: child_angle(doc, child).unwrap_or_default(),
            node: child,
        });
    }

    Item::Sheet(SheetItem {
        uuid: uuid.unwrap_or_else(|| Uuid(String::new())),
        at: child_point(doc, node).unwrap_or_default(),
        size,
        fields: read_fields(doc, node, version),
        pins,
        pages: read_sheet_pages(doc, node),
        node,
    })
}

/// Read a sheet item's page number, one per placement of the sheet.
fn read_sheet_pages(doc: &Doc, node: NodeId) -> Vec<SheetPage> {
    let mut pages = Vec::new();
    for &child in doc.children(node) {
        if !doc.head_is(child, "instances") {
            continue;
        }
        for &project_node in doc.children(child) {
            if !doc.head_is(project_node, "project") {
                continue;
            }
            let project = first_atom(doc, project_node)
                .and_then(|id| doc.atom_as_str(id))
                .unwrap_or_default();
            for &path_node in doc.children(project_node) {
                if !doc.head_is(path_node, "path") {
                    continue;
                }
                let Some(path) = first_atom(doc, path_node).and_then(|id| doc.atom_as_str(id))
                else {
                    continue;
                };
                pages.push(SheetPage {
                    project: project.clone(),
                    path: SheetPath(path),
                    page: child_atom_string(doc, path_node, "page").unwrap_or_default(),
                });
            }
        }
    }
    pages
}

/// Read every `property` child of a list as a field.
///
/// A library symbol's defaults are read the same way as a placement's, so the
/// library reader shares this rather than keeping a second copy.
pub(crate) fn read_fields_of(doc: &Doc, node: NodeId, version: FormatVersion) -> Vec<Field> {
    read_fields(doc, node, version)
}

/// Read every `property` child as a field.
fn read_fields(doc: &Doc, node: NodeId, version: FormatVersion) -> Vec<Field> {
    let mut fields = Vec::new();
    for &child in doc.children(node) {
        if !doc.head_is(child, "property") {
            continue;
        }
        let values = doc.children(child);
        let (Some(name), Some(value)) = (
            values.get(1).and_then(|&id| doc.atom_as_str(id)),
            values.get(2).and_then(|&id| doc.atom_as_str(id)),
        ) else {
            continue;
        };
        fields.push(Field {
            name,
            value,
            at: child_point(doc, child).unwrap_or_default(),
            angle: child_angle(doc, child).unwrap_or_default(),
            hidden: is_hidden(doc, child, version),
            justify: justification(doc, child),
            node: child,
        });
    }
    fields
}

/// The justification tokens of a field, as written.
fn justification(doc: &Doc, field: NodeId) -> Vec<String> {
    for &child in doc.children(field) {
        if !doc.head_is(child, "effects") {
            continue;
        }
        for &effect in doc.children(child) {
            if !doc.head_is(effect, "justify") {
                continue;
            }
            return doc
                .children(effect)
                .iter()
                .skip(1)
                .filter_map(|&id| doc.atom_text(id).map(str::to_owned))
                .collect();
        }
    }
    Vec::new()
}

/// Is a field hidden?
///
/// The token moved: it sits inside `effects` in files written before stamp
/// 20251028 and beside `show_name` after it. Reading only one place makes every
/// hidden field of an older file look visible.
fn is_hidden(doc: &Doc, field: NodeId, version: FormatVersion) -> bool {
    if version.hide_lives_in_effects() {
        for &child in doc.children(field) {
            if doc.head_is(child, "effects") {
                for &effect in doc.children(child) {
                    if doc.head_is(effect, "hide") || doc.atom_text(effect) == Some("hide") {
                        return true;
                    }
                }
            }
        }
        return false;
    }
    child_atom(doc, field, "hide") == Some("yes")
}

/// The `(size w h)` of a list, when it has one.
fn child_size(doc: &Doc, node: NodeId) -> Option<(Iu, Iu)> {
    let size = doc
        .children(node)
        .iter()
        .copied()
        .find(|&child| doc.head_is(child, "size"))?;
    let values = doc.children(size);
    Some((
        Iu(doc.atom_as_iu(*values.get(1)?)?),
        Iu(doc.atom_as_iu(*values.get(2)?)?),
    ))
}

/// The first child that is an atom after the head token.
fn first_atom(doc: &Doc, node: NodeId) -> Option<NodeId> {
    doc.children(node)
        .iter()
        .skip(1)
        .copied()
        .find(|&child| doc.atom_text(child).is_some())
}

/// The `(at x y [angle])` of a list, as a point.
fn child_point(doc: &Doc, node: NodeId) -> Option<Point> {
    let at = doc
        .children(node)
        .iter()
        .copied()
        .find(|&child| doc.head_is(child, "at"))?;
    point_of(doc, at)
}

/// The first two numbers of a list, as a point.
fn point_of(doc: &Doc, list: NodeId) -> Option<Point> {
    let values = doc.children(list);
    Some(Point {
        x: Iu(doc.atom_as_iu(*values.get(1)?)?),
        y: Iu(doc.atom_as_iu(*values.get(2)?)?),
    })
}

/// The angle of an `(at x y angle)`, when it has one.
fn child_angle(doc: &Doc, node: NodeId) -> Option<Angle> {
    let at = doc
        .children(node)
        .iter()
        .copied()
        .find(|&child| doc.head_is(child, "at"))?;
    let third = *doc.children(at).get(3)?;
    Angle::from_text(doc.atom_text(third)?)
}

/// The first value of a named child list, as source text.
fn child_atom<'a>(doc: &'a Doc, node: NodeId, head: &str) -> Option<&'a str> {
    let child = doc
        .children(node)
        .iter()
        .copied()
        .find(|&child| doc.head_is(child, head))?;
    doc.atom_text(*doc.children(child).get(1)?)
}

/// The first value of a named child list, unquoted.
fn child_atom_string(doc: &Doc, node: NodeId, head: &str) -> Option<String> {
    let child = doc
        .children(node)
        .iter()
        .copied()
        .find(|&child| doc.head_is(child, head))?;
    doc.children(child)
        .get(1)
        .and_then(|&id| doc.atom_as_str(id))
}

/// The first value of a named child list, as a whole number.
///
/// `None` covers both an absent list and one holding something that is not a
/// count. A count is not a measurement, so [`Doc::check_measurements`] does not
/// look at it; the callers each say what an absent one means, and every one of
/// them means "the KiCad default" rather than zero.
fn child_number(doc: &Doc, node: NodeId, head: &str) -> Option<u32> {
    child_atom(doc, node, head)?.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::{LibId, Schematic, SheetPath, Uuid};
    use kicli_sexpr::Doc;

    fn schematic(source: &str) -> Schematic {
        let doc = Doc::parse(source).expect("parses");
        Schematic::read(&doc).expect("reads")
    }

    #[test]
    fn a_library_identifier_splits_at_the_first_colon() {
        let id = LibId("Device:R".to_owned());
        assert_eq!(id.nickname(), Some("Device"));
        assert_eq!(id.symbol_name(), "R");
        // A symbol name may contain a colon of its own.
        let odd = LibId("Lib:Part:1".to_owned());
        assert_eq!(odd.nickname(), Some("Lib"));
        assert_eq!(odd.symbol_name(), "Part:1");
        assert_eq!(LibId("R".to_owned()).nickname(), None);
    }

    #[test]
    fn a_sheet_path_grows_by_one_sheet_at_a_time() {
        let root = SheetPath::root(&Uuid("aaaa".to_owned()));
        assert_eq!(root.0, "/aaaa");
        let child = root.child(&Uuid("bbbb".to_owned()));
        assert_eq!(child.0, "/aaaa/bbbb");
        assert_eq!(child.segments().collect::<Vec<_>>(), ["aaaa", "bbbb"]);
    }

    #[test]
    fn a_file_that_is_not_a_schematic_is_refused() {
        let doc = Doc::parse("(kicad_symbol_lib\n\t(version 20251024)\n)\n").expect("parses");
        assert!(Schematic::read(&doc).is_err());
    }

    #[test]
    fn an_unknown_token_keeps_its_name_and_its_node() {
        let source = concat!(
            "(kicad_sch\n\t(version 20260306)\n",
            "\t(rule_area\n\t\t(uuid \"1234\")\n\t)\n)\n"
        );
        let read = schematic(source);
        assert_eq!(read.items.len(), 1);
        match &read.items[0] {
            super::Item::Other { token, uuid, .. } => {
                assert_eq!(token, "rule_area");
                assert_eq!(uuid.as_ref().map(|id| id.0.as_str()), Some("1234"));
            }
            other => panic!("expected an unnamed item, got {other:?}"),
        }
    }
}
