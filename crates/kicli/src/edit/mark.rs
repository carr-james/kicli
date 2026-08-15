//! Junctions and no-connects: the marks that decide what is joined.
//!
//! A junction makes a crossing a connection. A no-connect says a pin is
//! deliberately unconnected. Both marks are one point in the file, both change
//! what the netlist says, and both are refused when the drawing already says
//! something else.
//!
//! Two refusals live here. A junction where four wire ends already meet draws
//! as one dot that four wires run into, and a reader cannot tell which pair
//! the designer meant to join. A no-connect on a pin that something already
//! joins contradicts the drawing it sits in. Neither refusal writes a byte.
//!
//! Every command here takes the loaded hierarchy, because a mark is only
//! meaningful against the project it sits in, and hands the change to
//! [`crate::model::mutate`], which is the only path to disk.

use std::fmt;
use std::path::Path;

use kicli_sexpr::{Doc, NodeId, SexprError, quote};

use crate::connectivity::extract;
use crate::geometry::{Point, resolve_pins, snap_point};
use crate::model::hierarchy::{Hierarchy, LoadedFile};
use crate::model::items::{
    Item, LabelKind, Line, LineKind, ReadError, Refdes, Schematic, SheetPath, Uuid,
};
use crate::model::library::{definition_of, read_library};
use crate::model::mutate::{Mutation, MutationError, Target, commit, state_before};
use crate::view::snapshot::{Snapshot, SnapshotError, millimetres};

/// How many wire ends at one point make a junction a four-way junction.
const FOUR_WAY: usize = 4;

/// One end of a wire or bus segment, named by the segment it belongs to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WireEnd {
    /// The short form of the segment's identifier, as a delta prints it.
    pub handle: String,
    /// Where the segment's other end is.
    pub far: Point,
}

impl fmt::Display for WireEnd {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "wire {} to ({})", self.handle, millimetres(self.far))
    }
}

/// Which pin of which placed symbol a no-connect belongs to.
///
/// The reference designator is the one the symbol carries on the sheet path
/// being edited. A symbol on a sheet placed twice has two, and only the sheet
/// path decides which is meant.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PinAddress {
    /// The reference designator, such as `R11`.
    pub reference: Refdes,
    /// The pin number, as text: pins are named `A1` as often as `1`.
    pub number: String,
}

impl PinAddress {
    /// Name one pin of one symbol.
    ///
    /// # Examples
    ///
    /// ```
    /// use kicli::edit::mark::PinAddress;
    /// use kicli::model::Refdes;
    ///
    /// let pin = PinAddress::new(Refdes("R11".to_owned()), "2");
    /// assert_eq!(pin.to_string(), "R11.2");
    /// ```
    #[must_use]
    pub fn new(reference: Refdes, number: &str) -> Self {
        Self {
            reference,
            number: number.to_owned(),
        }
    }
}

impl fmt::Display for PinAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.reference.0, self.number)
    }
}

/// Why a mark command did not happen.
///
/// Every variant is an operation error: the request is well formed and kicli
/// will not carry it out. No variant reaches the disk, so a refusal leaves the
/// file exactly as it was.
#[derive(Debug, thiserror::Error)]
pub enum MarkError {
    /// Four or more wire ends already meet at the point.
    #[error(
        "four wire ends meet at ({}): {}. A junction there is a four-way junction. \
         That is a defect, and KiCad's own rule check ignores it by default. \
         Move one wire end by one grid step. Then add the junction.",
        millimetres(*.at),
        listed(.ends)
    )]
    FourWayJunction {
        /// Where the wire ends meet.
        at: Point,
        /// The wire ends that meet there.
        ends: Vec<WireEnd>,
    },

    /// A junction is already drawn at the point.
    #[error("a junction is already drawn at ({}).", millimetres(*.at))]
    JunctionExists {
        /// Where the junction is.
        at: Point,
    },

    /// No junction is drawn at the point.
    #[error("no junction is drawn at ({}).", millimetres(*.at))]
    NoJunctionThere {
        /// Where kicli looked.
        at: Point,
    },

    /// Something already joins the pin, so it is not unconnected.
    #[error(
        "{} joins {pin} to {}. A no-connect says a pin has no connection. \
         kicli will not contradict the drawing. Remove the connection first.",
        net_phrase(.net.as_deref()),
        listed(.joined)
    )]
    PinConnected {
        /// The pin the caller named.
        pin: PinAddress,
        /// The net the pin is on, when the pin is listed on one.
        net: Option<String>,
        /// What the pin is joined to.
        joined: Vec<String>,
    },

    /// The pin already carries a no-connect.
    #[error("{pin} already carries a no-connect.")]
    NoConnectExists {
        /// The pin the caller named.
        pin: PinAddress,
    },

    /// The pin carries no no-connect.
    #[error("{pin} carries no no-connect.")]
    NoNoConnectThere {
        /// The pin the caller named.
        pin: PinAddress,
    },

    /// The sheet holds no symbol of that name on this sheet path.
    #[error("this sheet has no symbol called {reference} on sheet path {path}.")]
    NoSuchSymbol {
        /// The reference designator the caller named.
        reference: String,
        /// The sheet path kicli looked on.
        path: String,
    },

    /// The symbol has no pin of that number.
    #[error("{pin} does not exist: the symbol has no pin of that number.")]
    NoSuchPin {
        /// The pin the caller named.
        pin: PinAddress,
    },

    /// The symbol's definition is not embedded, so its pins have no positions.
    #[error(
        "the definition of {reference} is not in this file, so kicli cannot place its pins. \
         The symbol was placed from {lib_id}."
    )]
    NoDefinition {
        /// The reference designator of the symbol.
        reference: String,
        /// The library identifier the symbol was placed from.
        lib_id: String,
    },

    /// The file to edit is not part of the loaded hierarchy.
    #[error("{path} is not one of the files of this project.")]
    UnknownFile {
        /// The file the caller asked for.
        path: String,
    },

    /// The file holds no outermost list, so nothing can be added to it.
    #[error("this file is empty, so kicli cannot add anything to it.")]
    Empty,

    /// A fragment kicli built did not parse.
    #[error(transparent)]
    Sexpr(#[from] SexprError),

    /// The edited file did not read back as a schematic.
    #[error(transparent)]
    Read(#[from] ReadError),

    /// The state to compare the change against could not be taken.
    #[error(transparent)]
    Snapshot(#[from] SnapshotError),

    /// The change did not survive its own checks, or could not be written.
    #[error(transparent)]
    Mutation(#[from] MutationError),
}

/// Add a junction at a point.
///
/// The point is snapped to `target.grid`, because a junction is connectable
/// geometry and off-grid connectable geometry is a blocking fault. `uuid` is
/// the identifier the new object carries; the caller supplies it, so that two
/// runs of one command over one design produce one file.
///
/// # Errors
///
/// Returns [`MarkError::FourWayJunction`] when four wire ends already meet at
/// the point, and [`MarkError::JunctionExists`] when a junction is drawn there
/// already. Neither writes anything.
pub fn add_junction(
    hierarchy: &mut Hierarchy,
    at: Point,
    uuid: &Uuid,
    target: &Target<'_>,
    taken: &str,
) -> Result<Mutation, MarkError> {
    let file = file_of(hierarchy, target.path)?;
    let at = snap_point(at, target.grid);
    let schematic = &hierarchy.files[file].schematic;

    if schematic.junctions().any(|junction| junction.at == at) {
        return Err(MarkError::JunctionExists { at });
    }
    let ends = wire_ends_at(schematic, at);
    if ends.len() >= FOUR_WAY {
        return Err(MarkError::FourWayJunction { at, ends });
    }

    let fragment = format!(
        "(junction (at {} {}) (diameter 0) (color 0 0 0 0) (uuid {}))",
        at.x,
        at.y,
        quote(&uuid.0)
    );
    write_change(hierarchy, file, target, taken, |doc| {
        add_item(doc, &fragment)
    })
}

/// Delete the junction at a point.
///
/// The point is snapped to `target.grid`, so a caller addresses the junction
/// the same way it added one.
///
/// # Errors
///
/// Returns [`MarkError::NoJunctionThere`] when no junction is drawn at the
/// point.
pub fn delete_junction(
    hierarchy: &mut Hierarchy,
    at: Point,
    target: &Target<'_>,
    taken: &str,
) -> Result<Mutation, MarkError> {
    let file = file_of(hierarchy, target.path)?;
    let at = snap_point(at, target.grid);
    let node = hierarchy.files[file]
        .schematic
        .junctions()
        .find(|junction| junction.at == at)
        .map(|junction| junction.node)
        .ok_or(MarkError::NoJunctionThere { at })?;

    write_change(hierarchy, file, target, taken, |doc| {
        doc.remove(node);
        Ok(())
    })
}

/// Add a no-connect to a pin.
///
/// The marker lands on the pin's own connection point, which is where KiCad
/// draws it and where the netlister looks for it. The point is not snapped: a
/// pin that is off the grid is a fault of the drawing, and the invariants
/// refuse the write rather than moving the marker away from its pin.
///
/// # Errors
///
/// Returns [`MarkError::PinConnected`] when something already joins the pin,
/// and [`MarkError::NoConnectExists`] when the pin carries a marker already.
/// Neither writes anything. Returns [`MarkError::NoSuchSymbol`],
/// [`MarkError::NoSuchPin`] or [`MarkError::NoDefinition`] when the pin cannot
/// be found or cannot be placed.
pub fn add_no_connect(
    hierarchy: &mut Hierarchy,
    pin: &PinAddress,
    uuid: &Uuid,
    target: &Target<'_>,
    taken: &str,
) -> Result<Mutation, MarkError> {
    let file = file_of(hierarchy, target.path)?;
    let at = pin_position(&hierarchy.files[file], target.sheet_path, pin)?;

    if no_connect_at(&hierarchy.files[file].schematic, at).is_some() {
        return Err(MarkError::NoConnectExists { pin: pin.clone() });
    }
    let (net, joined) = joins_of(hierarchy, file, at, pin);
    if !joined.is_empty() {
        return Err(MarkError::PinConnected {
            pin: pin.clone(),
            net,
            joined,
        });
    }

    let fragment = format!(
        "(no_connect (at {} {}) (uuid {}))",
        at.x,
        at.y,
        quote(&uuid.0)
    );
    write_change(hierarchy, file, target, taken, |doc| {
        add_item(doc, &fragment)
    })
}

/// Delete the no-connect on a pin.
///
/// # Errors
///
/// Returns [`MarkError::NoNoConnectThere`] when the pin carries no marker, and
/// the same pin-lookup errors as [`add_no_connect`].
pub fn delete_no_connect(
    hierarchy: &mut Hierarchy,
    pin: &PinAddress,
    target: &Target<'_>,
    taken: &str,
) -> Result<Mutation, MarkError> {
    let file = file_of(hierarchy, target.path)?;
    let at = pin_position(&hierarchy.files[file], target.sheet_path, pin)?;
    let node = no_connect_at(&hierarchy.files[file].schematic, at)
        .ok_or_else(|| MarkError::NoNoConnectThere { pin: pin.clone() })?;

    write_change(hierarchy, file, target, taken, |doc| {
        doc.remove(node);
        Ok(())
    })
}

/// Where one pin of one placed symbol connects.
///
/// A junction is addressed by a point, and a caller who means "the point where
/// `R11.2` connects" should not have to work it out. The answer is a property of
/// the sheet path: a symbol on a sheet placed twice draws a different unit on
/// each placement, so each has its own pin positions.
///
/// # Errors
///
/// Returns [`MarkError::UnknownFile`] when the target names no file of this
/// project, and [`MarkError::NoSuchSymbol`], [`MarkError::NoSuchPin`] or
/// [`MarkError::NoDefinition`] when the pin cannot be found or cannot be placed.
pub fn pin_point(
    hierarchy: &Hierarchy,
    target: &Target<'_>,
    pin: &PinAddress,
) -> Result<Point, MarkError> {
    let file = file_of(hierarchy, target.path)?;
    pin_position(&hierarchy.files[file], target.sheet_path, pin)
}

/// Where one pin of one placed symbol connects.
fn pin_position(
    loaded: &LoadedFile,
    sheet: &SheetPath,
    pin: &PinAddress,
) -> Result<Point, MarkError> {
    let symbol = loaded
        .schematic
        .symbols()
        .find(|symbol| symbol.reference_on(sheet) == Some(&pin.reference))
        .ok_or_else(|| MarkError::NoSuchSymbol {
            reference: pin.reference.0.clone(),
            path: sheet.0.clone(),
        })?;
    let library = read_library(
        &loaded.doc,
        &loaded.schematic.library_symbols,
        loaded.schematic.version,
    );
    let definition = definition_of(&library, symbol).ok_or_else(|| MarkError::NoDefinition {
        reference: pin.reference.0.clone(),
        lib_id: symbol.lib_id.0.clone(),
    })?;
    // The unit is a property of the sheet path, not of the cache beside the
    // lib_id. A sheet placed twice draws a different unit on each placement,
    // and resolving from the cache would put the marker on the other one's pin.
    resolve_pins(&symbol.drawn_on(sheet), definition)
        .into_iter()
        .find(|resolved| resolved.number == pin.number)
        .map(|resolved| resolved.position)
        .ok_or_else(|| MarkError::NoSuchPin { pin: pin.clone() })
}

/// The no-connect marker at one point, when there is one.
fn no_connect_at(schematic: &Schematic, at: Point) -> Option<NodeId> {
    schematic.items.iter().find_map(|item| match item {
        Item::NoConnect(marker) if marker.at == at => Some(marker.node),
        _ => None,
    })
}

/// What joins a pin to the rest of the drawing, and the net it is on.
///
/// Two sources answer the question, because one alone would mislead. The net
/// partition names the other pins, wherever in the project they are. The sheet
/// itself names the wires, junctions, labels and sheet pins that meet the pin,
/// which a pin with a stub wire and no other pin would otherwise hide.
fn joins_of(
    hierarchy: &Hierarchy,
    file: usize,
    at: Point,
    pin: &PinAddress,
) -> (Option<String>, Vec<String>) {
    let nets = extract(hierarchy);
    let mut joined = Vec::new();
    let net = nets.net_of(&pin.reference.0, &pin.number).map(|net| {
        for other in &net.pins {
            if other.reference != pin.reference || other.number != pin.number {
                joined.push(other.label());
            }
        }
        net.name.clone()
    });

    for item in &hierarchy.files[file].schematic.items {
        match item {
            Item::Line(line) if line.from == at || line.to == at => {
                joined.push(format!("{} {}", token_of(line.kind), short(&line.uuid.0)));
            }
            Item::Junction(junction) if junction.at == at => {
                joined.push(format!("junction {}", short(&junction.uuid.0)));
            }
            // A netclass flag carries a netclass name, not a net name, so it
            // joins nothing and says nothing about a connection.
            Item::Label(label) if label.at == at && label.kind != LabelKind::NetclassFlag => {
                joined.push(format!("label {:?}", label.text));
            }
            Item::Sheet(sheet) => {
                for sheet_pin in &sheet.pins {
                    if sheet_pin.at == at {
                        joined.push(format!("sheet pin {:?}", sheet_pin.name));
                    }
                }
            }
            _ => {}
        }
    }
    (net, joined)
}

/// The word a file uses for one kind of connection line.
fn token_of(kind: LineKind) -> &'static str {
    match kind {
        LineKind::Wire => "wire",
        LineKind::Bus => "bus",
    }
}

/// The phrase that names what joins a pin, when the net is known.
fn net_phrase(net: Option<&str>) -> String {
    match net {
        Some(name) => format!("net {name}"),
        None => "the drawing".to_owned(),
    }
}

/// The wire and bus ends that meet at one point.
///
/// A segment's body is not an end. A pin or a wire end on another segment's
/// interior does not join it, so the interior is not part of this count.
///
/// Visible to the rest of the crate because two questions rest on it and must
/// rest on the same answer: whether a junction being added is a four-way one,
/// and whether a junction a deleted wire left behind still joins enough ends
/// to be doing anything. A second implementation of "the ends at this point"
/// is a second answer waiting to disagree with the first.
pub(crate) fn wire_ends_at(schematic: &Schematic, at: Point) -> Vec<WireEnd> {
    let mut ends = Vec::new();
    for line in schematic.lines() {
        if line.from == at {
            ends.push(end_of(line, line.to));
        }
        if line.to == at {
            ends.push(end_of(line, line.from));
        }
    }
    ends
}

fn end_of(line: &Line, far: Point) -> WireEnd {
    WireEnd {
        handle: short(&line.uuid.0),
        far,
    }
}

/// Put a new object into a file, before its `sheet_instances` list.
///
/// KiCad reads the objects of a sheet in any order and sorts them on save. The
/// position chosen here keeps the trailing lists at the end, so the change
/// reads as one insertion rather than as a move of everything after it.
fn add_item(doc: &mut Doc, fragment: &str) -> Result<(), MarkError> {
    let root = doc.root().ok_or(MarkError::Empty)?;
    let node = doc.add_fragment(fragment)?;
    let before = doc
        .children(root)
        .iter()
        .position(|&child| doc.head_is(child, "sheet_instances"));
    match before {
        Some(index) => doc.insert_child(root, index, node),
        None => doc.push_child(root, node),
    }
    Ok(())
}

/// Change one file of a hierarchy, then check it, write it, and report it.
///
/// The state to compare against is taken before the change, and the file's
/// typed objects are read again after it, so a second command on the same
/// hierarchy sees what the first one did.
fn write_change(
    hierarchy: &mut Hierarchy,
    file: usize,
    target: &Target<'_>,
    taken: &str,
    change: impl FnOnce(&mut Doc) -> Result<(), MarkError>,
) -> Result<Mutation, MarkError> {
    let loaded = &mut hierarchy.files[file];
    let before: Snapshot = state_before(&loaded.doc, &loaded.schematic, target.sheet_path, taken)?;
    change(&mut loaded.doc)?;
    let mutation = commit(&loaded.doc, target, &before, taken)?;
    loaded.schematic = Schematic::read(&loaded.doc)?;
    Ok(mutation)
}

/// Which file of the hierarchy the target names.
fn file_of(hierarchy: &Hierarchy, path: &Path) -> Result<usize, MarkError> {
    let wanted = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_owned());
    hierarchy
        .files
        .iter()
        .position(|file| {
            std::fs::canonicalize(&file.path).unwrap_or_else(|_| file.path.clone()) == wanted
        })
        .ok_or_else(|| MarkError::UnknownFile {
            path: path.display().to_string(),
        })
}

/// The first eight characters of an identifier, which is the handle a delta
/// prints.
fn short(uuid: &str) -> String {
    uuid.chars().take(8).collect()
}

/// A comma-separated list, for an error message.
fn listed<T: fmt::Display>(items: &[T]) -> String {
    items
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<String>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use super::{listed, short};

    #[test]
    fn a_handle_is_the_first_eight_characters() {
        assert_eq!(short("0123456789abcdef"), "01234567");
        assert_eq!(short("short"), "short");
    }

    #[test]
    fn a_list_reads_as_prose() {
        assert_eq!(listed(&["one", "two"]), "one, two");
        assert_eq!(listed::<&str>(&[]), "");
    }
}
