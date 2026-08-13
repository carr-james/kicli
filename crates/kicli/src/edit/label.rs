//! Labels: local, global and hierarchical.
//!
//! A label anchor is connectable geometry, so every command here snaps it to
//! the grid and says that it did. Adding a label is therefore a change to the
//! netlist as well as to the drawing, and the report names the net the label
//! joined.
//!
//! What a label joins is measured behaviour, not arithmetic. A label on a
//! wire's interior joins that wire, unless a pin, another label, a sheet pin
//! or a no-connect shares its anchor; a label where two or more wires meet
//! joins all of them. The shared-anchor case draws as a connection and is not
//! one, so the commands here warn about it rather than leaving a caller to
//! find out from a netlist.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::Path;

use kicli_sexpr::{Doc, NodeId, SexprError, quote};

use crate::connectivity::{Net, NetPin, Nets, extract};
use crate::edit::text::{fresh_uuid, insertion_index};
use crate::geometry::{Angle, Iu, Point, resolve_pins};
use crate::model::hierarchy::{Hierarchy, LoadError};
use crate::model::items::{Item, LabelKind, ReadError, Schematic, Uuid};
use crate::model::library::{definition_of, read_library};
use crate::model::mutate::{Mutation, MutationError, Target, commit, state_before};
use crate::model::version::FormatVersion;
use crate::view::snapshot::SnapshotError;

/// The text size KiCad gives a new label, in millimetres.
const DEFAULT_TEXT_SIZE: &str = "1.27";

/// The direction a port faces.
///
/// A global or hierarchical label carries one. A local label does not.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PortShape {
    /// The port takes a signal in.
    Input,
    /// The port sends a signal out.
    Output,
    /// The port does both.
    Bidirectional,
    /// The port is driven by more than one source.
    TriState,
    /// The port has no direction.
    #[default]
    Passive,
}

impl PortShape {
    /// The token KiCad writes for this direction.
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            PortShape::Input => "input",
            PortShape::Output => "output",
            PortShape::Bidirectional => "bidirectional",
            PortShape::TriState => "tri_state",
            PortShape::Passive => "passive",
        }
    }
}

/// What a new label says, and where its anchor goes.
#[derive(Clone, Debug)]
pub struct NewLabel {
    /// Which kind of label to make.
    pub kind: LabelKind,
    /// The net name the label carries.
    pub text: String,
    /// The anchor, which is the point that connects. It is snapped to the
    /// grid.
    pub at: Point,
    /// The text angle: 0, 90, 180 or 270.
    pub angle: Angle,
    /// The port direction. A local label ignores it.
    pub shape: PortShape,
}

/// One net, as a report names it.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct NetSummary {
    /// The name kicli shows for the net.
    pub name: String,
    /// The pins of the net, sorted.
    pub pins: Vec<String>,
}

/// What a label command did to the nets of a project.
///
/// A net is identified by its pins, because the name is the part a label
/// changes. A net whose only change is its synthetic handle is left out: those
/// handles are assigned across the whole design, so an unrelated net gaining a
/// name renumbers them.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NetChange {
    /// The nets as they stand after the write.
    pub after: Vec<NetSummary>,
    /// The nets those replaced, as they stood before it.
    pub before: Vec<NetSummary>,
}

impl NetChange {
    /// Did no net with a pin change?
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.after.is_empty() && self.before.is_empty()
    }

    /// The text form: one line per net, in the vocabulary of the net view.
    #[must_use]
    pub fn render(&self) -> String {
        if self.is_empty() {
            return "no net with a pin changed\n".to_owned();
        }
        let mut text = String::new();
        for net in &self.after {
            let was: Vec<&str> = self
                .before
                .iter()
                .filter(|old| old.pins.iter().any(|pin| net.pins.contains(pin)))
                .map(|old| old.name.as_str())
                .collect();
            let _ = write!(text, "net {}: {}", net.name, net.pins.join(" "));
            if !was.is_empty() {
                let _ = write!(text, " (was {})", was.join(" "));
            }
            text.push('\n');
        }
        for net in &self.before {
            if self
                .after
                .iter()
                .any(|new| new.pins.iter().any(|pin| net.pins.contains(pin)))
            {
                continue;
            }
            let _ = writeln!(text, "net {} is gone: {}", net.name, net.pins.join(" "));
        }
        text
    }
}

/// What a label command changed, to the file and to the nets.
#[derive(Clone, Debug)]
pub struct LabelChange {
    /// The label's identifier.
    pub uuid: Uuid,
    /// Where the anchor ended up.
    pub at: Point,
    /// Where the caller asked for it.
    pub requested: Point,
    /// What the mutation touched, and what kicli checked afterwards.
    pub mutation: Mutation,
    /// What the change did to the nets.
    pub nets: NetChange,
    /// What the caller must know about the anchor.
    pub notes: Vec<String>,
}

impl LabelChange {
    /// Did the anchor move onto the grid?
    #[must_use]
    pub fn snapped(&self) -> bool {
        self.at != self.requested
    }

    /// The text form: what moved, what it did to the nets, and the notes.
    #[must_use]
    pub fn render(&self) -> String {
        let mut text = self.mutation.render();
        text.push_str(&self.nets.render());
        for note in &self.notes {
            let _ = writeln!(text, "note: {note}");
        }
        text
    }

    /// The JSON form, carrying the same content.
    #[must_use]
    pub fn to_json(&self) -> serde_json::Value {
        let mut value = self.mutation.to_json();
        let summaries = |nets: &[NetSummary]| -> serde_json::Value {
            nets.iter()
                .map(|net| serde_json::json!({ "name": net.name, "pins": net.pins }))
                .collect::<Vec<_>>()
                .into()
        };
        if let Some(fields) = value.as_object_mut() {
            fields.insert(
                "label".to_owned(),
                serde_json::json!({
                    "uuid": self.uuid.0,
                    "at": self.at.to_string(),
                    "requested": self.requested.to_string(),
                    "snapped": self.snapped(),
                }),
            );
            fields.insert(
                "nets".to_owned(),
                serde_json::json!({
                    "after": summaries(&self.nets.after),
                    "before": summaries(&self.nets.before),
                }),
            );
            fields.insert("notes".to_owned(), serde_json::json!(self.notes));
        }
        value
    }
}

/// Why a label command did not happen.
#[derive(Debug, thiserror::Error)]
pub enum LabelError {
    /// The file holds no object with that identifier.
    #[error("this sheet has no object with the identifier {0}")]
    NotFound(String),
    /// The object is there, but it is not a label.
    #[error("{0} is a {1}, and this command works on labels")]
    NotALabel(String, String),
    /// The object is a label, but it lacks the list this command edits.
    #[error("{0} has no {1} list, so kicli will not guess where to write one")]
    Malformed(String, String),
    /// A netclass flag is not a net name.
    #[error(
        "a netclass flag carries a netclass name and not a net name, so this command does not make one"
    )]
    NetclassFlag,
    /// The file could not be read as a schematic.
    #[error(transparent)]
    Read(#[from] ReadError),
    /// The sheet tree could not be walked, so the nets are unknown.
    #[error(transparent)]
    Load(#[from] LoadError),
    /// The change was refused, or the file could not be written.
    #[error(transparent)]
    Mutation(#[from] MutationError),
    /// The state to compare against could not be taken.
    #[error(transparent)]
    Snapshot(#[from] SnapshotError),
    /// The new label could not be built.
    #[error(transparent)]
    Fragment(#[from] SexprError),
}

/// Add a label to a sheet.
///
/// `root` is the root schematic of the project. The nets before and after the
/// write are read from it, so the report names what the label joined rather
/// than what it was meant to join. Pass the edited file itself when it is the
/// root.
///
/// The anchor is snapped to `target.grid`. A hierarchical label is the child
/// half of a sheet port: the parent's sheet pin is a separate object, and the
/// notes say that kicli did not add it.
///
/// # Errors
///
/// Returns [`LabelError`] when the kind is a netclass flag, when the project
/// cannot be walked, or when the change does not survive the invariants.
/// Nothing is written unless every invariant holds.
pub fn add(
    doc: &mut Doc,
    target: &Target<'_>,
    root: &Path,
    request: &NewLabel,
    taken: &str,
) -> Result<LabelChange, LabelError> {
    if request.kind == LabelKind::NetclassFlag {
        return Err(LabelError::NetclassFlag);
    }
    let schematic = Schematic::read(doc)?;
    let sheet = doc.root().ok_or(ReadError::Empty)?;
    let at = snapped(request.at, target.grid);

    let mut notes = grid_note(request.at, at);
    notes.extend(anchor_notes(doc, &schematic, at, None));
    if request.kind == LabelKind::Hierarchical {
        notes.push(
            "This label is the child half of a sheet port. kicli does not add the parent's sheet pin."
                .to_owned(),
        );
    }

    let before_nets = nets_of(root)?;
    let before = state_before(doc, &schematic, target.sheet_path, taken)?;

    let uuid = fresh_uuid(
        doc,
        &format!(
            "{} label {} {} {}",
            target.path.display(),
            request.text,
            at,
            request.angle
        ),
    );
    let fragment = doc.add_fragment(&fragment_of(request, at, &uuid, schematic.version))?;
    let index = insertion_index(doc, sheet);
    doc.insert_child(sheet, index, fragment);

    let mutation = commit(doc, target, &before, taken)?;
    Ok(LabelChange {
        uuid,
        at,
        requested: request.at,
        mutation,
        nets: net_change(&before_nets, &nets_of(root)?),
        notes,
    })
}

/// Move a label's anchor to a position.
///
/// The anchor is snapped to `target.grid`, and the report says what the move
/// did to the nets.
///
/// # Errors
///
/// Returns [`LabelError`] when the identifier names no label, when the project
/// cannot be walked, or when the change does not survive the invariants.
pub fn move_to(
    doc: &mut Doc,
    target: &Target<'_>,
    root: &Path,
    uuid: &Uuid,
    to: Point,
    taken: &str,
) -> Result<LabelChange, LabelError> {
    let at = snapped(to, target.grid);
    let mut notes = grid_note(to, at);
    let plan = Plan {
        target,
        root,
        uuid,
        at,
        requested: to,
        taken,
    };
    change(doc, &plan, |doc, node| {
        let list = list_of(doc, node, "at")
            .ok_or_else(|| LabelError::Malformed(uuid.0.clone(), "at".to_owned()))?;
        let (Some(&x), Some(&y)) = (doc.children(list).get(1), doc.children(list).get(2)) else {
            return Err(LabelError::Malformed(uuid.0.clone(), "at".to_owned()));
        };
        doc.set_atom(x, &at.x.to_string());
        doc.set_atom(y, &at.y.to_string());
        Ok(())
    })
    .map(|mut change| {
        notes.append(&mut change.notes);
        change.notes = notes;
        change
    })
}

/// Take a label off a sheet.
///
/// # Errors
///
/// Returns [`LabelError`] when the identifier names no label, when the project
/// cannot be walked, or when the change does not survive the invariants.
pub fn delete(
    doc: &mut Doc,
    target: &Target<'_>,
    root: &Path,
    uuid: &Uuid,
    taken: &str,
) -> Result<LabelChange, LabelError> {
    let at = anchor_of(doc, uuid)?;
    let plan = Plan {
        target,
        root,
        uuid,
        at,
        requested: at,
        taken,
    };
    change(doc, &plan, |doc, node| {
        doc.remove(node);
        Ok(())
    })
}

/// What a command over an existing label needs to know.
struct Plan<'a> {
    /// Where the write lands, and under what rules.
    target: &'a Target<'a>,
    /// The root schematic, which the nets are read from.
    root: &'a Path,
    /// The label to change.
    uuid: &'a Uuid,
    /// Where the anchor ends up.
    at: Point,
    /// Where the caller asked for it.
    requested: Point,
    /// The timestamp to record.
    taken: &'a str,
}

/// Run one edit over an existing label, then write and report it.
fn change(
    doc: &mut Doc,
    plan: &Plan<'_>,
    edit: impl FnOnce(&mut Doc, NodeId) -> Result<(), LabelError>,
) -> Result<LabelChange, LabelError> {
    let schematic = Schematic::read(doc)?;
    let node = locate(doc, plan.uuid)?;
    let notes = anchor_notes(doc, &schematic, plan.at, Some(plan.uuid));
    let before_nets = nets_of(plan.root)?;
    let before = state_before(doc, &schematic, plan.target.sheet_path, plan.taken)?;
    edit(doc, node)?;
    let mutation = commit(doc, plan.target, &before, plan.taken)?;
    Ok(LabelChange {
        uuid: plan.uuid.clone(),
        at: plan.at,
        requested: plan.requested,
        mutation,
        nets: net_change(&before_nets, &nets_of(plan.root)?),
        notes,
    })
}

/// The label list an identifier names.
fn locate(doc: &Doc, uuid: &Uuid) -> Result<NodeId, LabelError> {
    let node = doc
        .uuid_index()
        .get(&uuid.0)
        .copied()
        .ok_or_else(|| LabelError::NotFound(uuid.0.clone()))?;
    match doc.head(node) {
        Some("label" | "global_label" | "hierarchical_label" | "netclass_flag") => Ok(node),
        Some(other) => Err(LabelError::NotALabel(uuid.0.clone(), other.to_owned())),
        None => Err(LabelError::NotFound(uuid.0.clone())),
    }
}

/// Where a label sits now.
fn anchor_of(doc: &Doc, uuid: &Uuid) -> Result<Point, LabelError> {
    let node = locate(doc, uuid)?;
    let list = list_of(doc, node, "at")
        .ok_or_else(|| LabelError::Malformed(uuid.0.clone(), "at".to_owned()))?;
    let values = doc.children(list);
    let (Some(x), Some(y)) = (
        values.get(1).and_then(|&id| doc.atom_as_iu(id)),
        values.get(2).and_then(|&id| doc.atom_as_iu(id)),
    ) else {
        return Err(LabelError::Malformed(uuid.0.clone(), "at".to_owned()));
    };
    Ok(Point::new(x, y))
}

/// The named child list of a list.
fn list_of(doc: &Doc, node: NodeId, head: &str) -> Option<NodeId> {
    doc.children(node)
        .iter()
        .copied()
        .find(|&child| doc.head_is(child, head))
}

/// The nets of a project as it stands on disk.
fn nets_of(root: &Path) -> Result<Nets, LabelError> {
    Ok(extract(&Hierarchy::load(root)?))
}

/// A position on the grid: round half away from zero, in integers.
fn snapped(point: Point, grid: Iu) -> Point {
    Point {
        x: snap(point.x, grid),
        y: snap(point.y, grid),
    }
}

/// One coordinate on the grid.
fn snap(value: Iu, grid: Iu) -> Iu {
    let step = i64::from(grid.0).abs();
    if step == 0 {
        return value;
    }
    let raw = i64::from(value.0);
    let half = step / 2;
    let steps = if raw >= 0 {
        (raw + half) / step
    } else {
        -((-raw + half) / step)
    };
    Iu(i32::try_from(steps * step).unwrap_or(value.0))
}

/// The note a snap leaves, when the anchor moved.
fn grid_note(requested: Point, at: Point) -> Vec<String> {
    if requested == at {
        return Vec::new();
    }
    vec![format!(
        "The anchor moved onto the grid, from {requested} to {at}."
    )]
}

/// What the caller must know about an anchor before the label lands on it.
///
/// The rules are KiCad's, measured against its own netlister: a label on two
/// or more lines joins all of them, a label on one line joins it only while
/// nothing else shares the anchor, and a label on no line joins nothing.
fn anchor_notes(doc: &Doc, schematic: &Schematic, at: Point, ignore: Option<&Uuid>) -> Vec<String> {
    let lines = schematic
        .lines()
        .filter(|line| on_segment(line.from, line.to, at))
        .count();
    let blocker = blocker_at(doc, schematic, at, ignore);
    match (lines, blocker) {
        (0, _) => {
            vec!["No wire passes through this anchor. The label names nothing yet.".to_owned()]
        }
        (1, Some(what)) => vec![format!(
            "{what} shares this anchor. The label does not join the wire."
        )],
        (1, None) => Vec::new(),
        (_, _) => {
            vec!["Two or more wires meet at this anchor. The label joins all of them.".to_owned()]
        }
    }
}

/// What sits at a point and stops a label joining the wire under it.
fn blocker_at(
    doc: &Doc,
    schematic: &Schematic,
    at: Point,
    ignore: Option<&Uuid>,
) -> Option<&'static str> {
    if pin_at(doc, schematic, at) {
        return Some("A pin");
    }
    for item in &schematic.items {
        match item {
            Item::Label(label) if label.at == at && Some(&label.uuid) != ignore => {
                return Some("Another label");
            }
            Item::NoConnect(mark) if mark.at == at => return Some("A no-connect"),
            Item::Sheet(sheet) if sheet.pins.iter().any(|pin| pin.at == at) => {
                return Some("A sheet pin");
            }
            _ => {}
        }
    }
    None
}

/// Is a symbol pin at a point?
fn pin_at(doc: &Doc, schematic: &Schematic, at: Point) -> bool {
    let library = read_library(doc, &schematic.library_symbols, schematic.version);
    schematic.symbols().any(|symbol| {
        definition_of(&library, symbol).is_some_and(|definition| {
            resolve_pins(symbol, definition)
                .iter()
                .any(|pin| pin.position == at)
        })
    })
}

/// Is a point on a segment, ends included?
///
/// Exact integer arithmetic in 64 bits: the cross product decides whether the
/// point is on the line, and the dot product whether it is between the ends.
fn on_segment(from: Point, to: Point, point: Point) -> bool {
    let (ax, ay) = (i64::from(from.x.0), i64::from(from.y.0));
    let (bx, by) = (i64::from(to.x.0), i64::from(to.y.0));
    let (px, py) = (i64::from(point.x.0), i64::from(point.y.0));
    let (dx, dy) = (bx - ax, by - ay);
    if dx * (py - ay) - dy * (px - ax) != 0 {
        return false;
    }
    let along = dx * (px - ax) + dy * (py - ay);
    along >= 0 && along <= dx * dx + dy * dy
}

/// What changed between two readings of a project's nets.
fn net_change(before: &Nets, after: &Nets) -> NetChange {
    let keyed: BTreeMap<Vec<String>, &Net> = before
        .nets()
        .iter()
        .map(|net| (pins_of(net), net))
        .collect();

    let mut changed: Vec<&Net> = Vec::new();
    for net in after.nets() {
        match keyed.get(&pins_of(net)) {
            // A synthetic handle is assigned across the whole design, so an
            // unrelated net gaining a name renumbers this one. That is not
            // news about this net.
            Some(old) if old.name == net.name || (old.synthetic && net.synthetic) => {}
            _ => changed.push(net),
        }
    }

    let mut was: Vec<NetSummary> = Vec::new();
    for net in &changed {
        let pins = pins_of(net);
        for old in before.nets() {
            if old.pins.iter().any(|pin| pins.contains(&pin.label()))
                && !(old.name == net.name && pins_of(old) == pins)
            {
                was.push(summary(old));
            }
        }
    }
    // A net that lost every pin it had is gone, and the report says so.
    let names: Vec<String> = changed.iter().map(|net| net.name.clone()).collect();
    for old in before.nets() {
        let pins = pins_of(old);
        if !after.nets().iter().any(|net| pins_of(net) == pins) && !names.contains(&old.name) {
            was.push(summary(old));
        }
    }
    was.sort();
    was.dedup();

    let mut after_summaries: Vec<NetSummary> = changed.iter().map(|net| summary(net)).collect();
    after_summaries.sort();
    NetChange {
        after: after_summaries,
        before: was,
    }
}

/// The pins of a net, sorted, as a report names them.
fn pins_of(net: &Net) -> Vec<String> {
    let mut pins: Vec<String> = net.pins.iter().map(NetPin::label).collect();
    pins.sort();
    pins
}

/// One net, as a report names it.
fn summary(net: &Net) -> NetSummary {
    NetSummary {
        name: net.name.clone(),
        pins: pins_of(net),
    }
}

/// The text of a new label, in the shape KiCad writes.
fn fragment_of(request: &NewLabel, at: Point, uuid: &Uuid, version: FormatVersion) -> String {
    let head = match request.kind {
        LabelKind::Global => "global_label",
        LabelKind::Hierarchical => "hierarchical_label",
        _ => "label",
    };
    let shape = match request.kind {
        LabelKind::Global | LabelKind::Hierarchical => {
            format!("(shape {}) ", request.shape.token())
        }
        _ => String::new(),
    };
    let text = quote(&request.text);
    let identifier = quote(&uuid.0);
    let (x, y, angle) = (at.x, at.y, request.angle);
    let justify = justification(request.angle);
    let intersheet = if request.kind == LabelKind::Global {
        intersheet_refs(at, version)
    } else {
        String::new()
    };
    format!(
        "({head} {text} {shape}(at {x} {y} {angle}) \
         (effects (font (size {DEFAULT_TEXT_SIZE} {DEFAULT_TEXT_SIZE})) (justify {justify})) \
         (uuid {identifier}){intersheet})"
    )
}

/// Which way a label's text runs from its anchor.
///
/// KiCad writes `left bottom` for a label that reads to the right, and
/// `right bottom` for one turned to read back the other way. The corpus has
/// both: `(at … 0)` with `left bottom`, `(at … 180)` with `right bottom`.
fn justification(angle: Angle) -> &'static str {
    match angle.0.rem_euclid(360) {
        180 | 270 => "right bottom",
        _ => "left bottom",
    }
}

/// The field a global label carries for its intersheet references.
///
/// The `hide` token moved between formats: it sits inside `effects` below
/// stamp 20251028 and beside `show_name` above it. Writing the wrong one makes
/// the field visible in the editor.
fn intersheet_refs(at: Point, version: FormatVersion) -> String {
    let (x, y) = (at.x, at.y);
    let font = format!("(font (size {DEFAULT_TEXT_SIZE} {DEFAULT_TEXT_SIZE}))");
    if version.hide_lives_in_effects() {
        return format!(
            " (property \"Intersheetrefs\" \"${{INTERSHEET_REFS}}\" (at {x} {y} 0) \
             (show_name no) (do_not_autoplace no) (effects {font} (hide yes)))"
        );
    }
    format!(
        " (property \"Intersheetrefs\" \"${{INTERSHEET_REFS}}\" (at {x} {y} 0) \
         (hide yes) (show_name no) (do_not_autoplace no) (effects {font}))"
    )
}

#[cfg(test)]
mod tests {
    use super::{
        Angle, Iu, LabelKind, NewLabel, Point, PortShape, Uuid, fragment_of, justification, snapped,
    };
    use crate::geometry::GRID;
    use crate::model::version::FormatVersion;
    use kicli_sexpr::Doc;

    fn request(kind: LabelKind) -> NewLabel {
        NewLabel {
            kind,
            text: "NET_A".to_owned(),
            at: Point::new(254_000, 508_000),
            angle: Angle(0),
            shape: PortShape::Input,
        }
    }

    fn fragment(kind: LabelKind) -> String {
        fragment_of(
            &request(kind),
            Point::new(254_000, 508_000),
            &Uuid("abc".to_owned()),
            FormatVersion::new(20_260_306),
        )
    }

    #[test]
    fn a_local_label_carries_no_shape() {
        let text = fragment(LabelKind::Local);
        assert!(
            text.starts_with("(label \"NET_A\" (at 25.4 50.8 0)"),
            "{text}"
        );
        assert!(!text.contains("shape"), "{text}");
        assert!(Doc::parse(&text).is_ok());
    }

    #[test]
    fn a_hierarchical_label_carries_its_direction() {
        let text = fragment(LabelKind::Hierarchical);
        assert!(
            text.starts_with("(hierarchical_label \"NET_A\" (shape input)"),
            "{text}"
        );
    }

    #[test]
    fn a_global_label_carries_its_intersheet_field() {
        let text = fragment(LabelKind::Global);
        assert!(text.contains("(property \"Intersheetrefs\""), "{text}");
        // The current format writes the token beside show_name.
        assert!(text.contains("(hide yes) (show_name no)"), "{text}");
        let doc = Doc::parse(&text).expect("the fragment parses");
        let root = doc.root().expect("it has a root");
        assert!(doc.head_is(root, "global_label"));
    }

    #[test]
    fn an_older_format_hides_the_field_inside_the_effects() {
        let text = fragment_of(
            &request(LabelKind::Global),
            Point::new(0, 0),
            &Uuid("abc".to_owned()),
            FormatVersion::new(20_250_114),
        );
        assert!(text.contains("(hide yes)))"), "{text}");
        assert!(!text.contains("(hide yes) (show_name no)"), "{text}");
    }

    #[test]
    fn an_anchor_rounds_half_away_from_zero() {
        assert_eq!(
            snapped(Point::new(304_900, 0), GRID),
            Point::new(304_800, 0)
        );
        assert_eq!(snapped(Point::new(6_350, 0), GRID), Point::new(12_700, 0));
        assert_eq!(snapped(Point::new(-6_350, 0), GRID), Point::new(-12_700, 0));
        // A grid of zero divides nothing.
        assert_eq!(snapped(Point::new(1, 2), Iu(0)), Point::new(1, 2));
    }

    #[test]
    fn text_runs_away_from_its_anchor() {
        assert_eq!(justification(Angle(0)), "left bottom");
        assert_eq!(justification(Angle(90)), "left bottom");
        assert_eq!(justification(Angle(180)), "right bottom");
        assert_eq!(justification(Angle(270)), "right bottom");
    }
}
