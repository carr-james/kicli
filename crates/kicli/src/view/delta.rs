//! What changed between two snapshots.
//!
//! The delta reports added, removed, moved and edited objects, ordered so that
//! the same pair of states always produces the same bytes.
//!
//! A line names the record the change belongs to, so a reader knows which view
//! to read again. A symbol that moved changes a layout record and prints `L`.
//! The same symbol renamed changes a connectivity record and prints `S`.
//!
//! The state a comparison sees decides how much it can say. A snapshot taken
//! from a design carries positions and values, so a line reports the old and
//! the new. A snapshot read from a file carries hashes only, so a line reports
//! which object changed and how, and nothing more.

use crate::view::snapshot::{Detail, ObjectKind, Snapshot, SnapshotObject, millimetres};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

/// What happened to one object.
///
/// The order of the variants breaks a tie between two lines that name one
/// object, so it is part of the output.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Change {
    /// The object is in the second state only.
    Added,
    /// The object is in the first state only.
    Removed,
    /// The object is in both states, in two places.
    Moved,
    /// The object is in both states, with two contents.
    Edited,
}

impl Change {
    /// Every variant, in the order they are declared.
    ///
    /// Exposed for `crates/kicli/tests/agent_doc.rs`, which reads the record
    /// examples out of `AGENT.md` and needs **the set of marks this writer can
    /// produce** rather than a set a test author remembered. A list of `+`, `-`
    /// and `~` spelled out in the test would be that author's vocabulary
    /// wearing a reference; this one is the enum's.
    ///
    /// The bound: adding a variant here is not enforced by the compiler. What
    /// is enforced is that a new variant makes [`Change::mark`]'s match
    /// non-exhaustive, so the author of one has to open this impl block, and
    /// the unit test below fails if a mark in it is missing or repeated.
    pub const ALL: [Change; 4] = [
        Change::Added,
        Change::Removed,
        Change::Moved,
        Change::Edited,
    ];

    /// The mark that starts the line.
    #[must_use]
    pub fn mark(self) -> char {
        match self {
            Change::Added => '+',
            Change::Removed => '-',
            Change::Moved | Change::Edited => '~',
        }
    }
}

/// One reported change.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeltaLine {
    /// What happened.
    pub change: Change,
    /// What kind of object it happened to.
    pub kind: ObjectKind,
    /// The record letter, which names the view the change shows up in.
    pub record: char,
    /// The name of the object.
    pub handle: String,
    /// What changed, such as two positions or two values.
    pub detail: String,
}

impl fmt::Display for DeltaLine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {}", self.change.mark(), self.record, self.handle)?;
        if self.detail.is_empty() {
            return Ok(());
        }
        // A change to an object that stays gets two spaces, so the reader can
        // tell the description of a new object from the report of an edit.
        match self.change {
            Change::Added | Change::Removed => write!(f, " {}", self.detail),
            Change::Moved | Change::Edited => write!(f, "  {}", self.detail),
        }
    }
}

/// Everything that changed between two states, and how much did not.
///
/// # Examples
///
/// ```
/// use kicli::model::{Schematic, SheetPath};
/// use kicli::view::delta::Delta;
/// use kicli::view::snapshot::Snapshot;
/// use kicli_sexpr::Doc;
///
/// fn take(name: &str, source: &str) -> Snapshot {
///     let doc = Doc::parse(source).expect("parses");
///     let schematic = Schematic::read(&doc).expect("reads");
///     let path = SheetPath::root(schematic.uuid.as_ref().expect("has a uuid"));
///     Snapshot::take(name, "2026-01-02T03:04:05Z", &path, &doc, &schematic).expect("takes")
/// }
///
/// let head = "(kicad_sch\n\t(version 20260306)\n\t(uuid \"a\")\n";
/// let before = take("base", &format!("{head}\t(junction\n\t\t(at 0 0)\n\t\t(uuid \"j\")\n\t)\n)\n"));
/// let after = take("current", &format!("{head}\t(junction\n\t\t(at 2.54 0)\n\t\t(uuid \"j\")\n\t)\n)\n"));
///
/// assert_eq!(
///     Delta::between(&before, &after).to_string(),
///     "delta base -> current\n~ W j  moved  (0.00,0.00) -> (2.54,0.00)\n= 0 objects unchanged\n",
/// );
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Delta {
    /// The name of the first state.
    pub from: String,
    /// The name of the second state.
    pub to: String,
    /// The changes, ordered by kind, then by name, then by what happened.
    pub lines: Vec<DeltaLine>,
    /// How many objects are in both states and did not change.
    pub unchanged: usize,
}

impl Delta {
    /// Compare two states.
    ///
    /// An object that belongs to an added or a removed object is not reported
    /// on its own. Adding a symbol reports one line, not one for the symbol
    /// and one for each of its fields.
    #[must_use]
    pub fn between(from: &Snapshot, to: &Snapshot) -> Self {
        let old = index(from);
        let new = index(to);
        let mut reports: Vec<Report<'_>> = Vec::new();
        let mut unchanged = 0;
        // The objects that arrived or left whole, whose parts say nothing.
        let mut whole: BTreeSet<&str> = BTreeSet::new();

        for (key, object) in &old {
            match new.get(key) {
                Some(later) if later.kind == object.kind => {
                    let moved = object.geometry != later.geometry;
                    let edited = object.data != later.data;
                    if moved {
                        reports.push((Change::Moved, object, later));
                    }
                    if edited {
                        reports.push((Change::Edited, object, later));
                    }
                    if !moved && !edited {
                        unchanged += 1;
                    }
                }
                Some(_) | None => {
                    whole.insert(key);
                    reports.push((Change::Removed, object, object));
                }
            }
        }
        for (key, object) in &new {
            if old.get(key).is_none_or(|before| before.kind != object.kind) {
                whole.insert(key);
                reports.push((Change::Added, object, object));
            }
        }

        reports.retain(|report| !belongs_to(report, &whole));
        let mut lines: Vec<DeltaLine> = reports
            .iter()
            .map(|(change, before, after)| line(*change, before, after))
            .collect();
        lines.sort_by(|left, right| sort_key(left).cmp(&sort_key(right)));

        Self {
            from: from.name.clone(),
            to: to.name.clone(),
            lines,
            unchanged,
        }
    }
}

/// One change, with the object on each side of it.
type Report<'a> = (Change, &'a SnapshotObject, &'a SnapshotObject);

/// Does this change belong to an object that arrived or left whole?
fn belongs_to(report: &Report<'_>, whole: &BTreeSet<&str>) -> bool {
    let (change, before, after) = report;
    let object = if *change == Change::Removed {
        before
    } else {
        after
    };
    object
        .owner
        .as_deref()
        .is_some_and(|owner| whole.contains(owner))
}

impl fmt::Display for Delta {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "delta {} -> {}", self.from, self.to)?;
        for line in &self.lines {
            writeln!(f, "{line}")?;
        }
        writeln!(f, "= {} objects unchanged", self.unchanged)
    }
}

/// The objects of a snapshot, by key.
fn index(snapshot: &Snapshot) -> BTreeMap<&str, &SnapshotObject> {
    snapshot
        .objects
        .iter()
        .map(|object| (object.key.as_str(), object))
        .collect()
}

/// What orders one line against another.
fn sort_key(line: &DeltaLine) -> (&ObjectKind, &str, Change) {
    (&line.kind, line.handle.as_str(), line.change)
}

/// Build one line.
///
/// The name comes from the second state where there is one, because that is
/// what the design is called now.
fn line(change: Change, before: &SnapshotObject, after: &SnapshotObject) -> DeltaLine {
    let named = if change == Change::Removed {
        before
    } else {
        after
    };
    DeltaLine {
        change,
        kind: named.kind.clone(),
        record: record_of(&named.kind, change),
        handle: named.handle.clone(),
        detail: detail(change, before, after),
    }
}

/// The record letter for one kind of change.
///
/// The letter names the view record the change shows up in: `S` for the
/// connectivity record of a symbol, `L` for its placement, `F` for a field
/// offset, `T` for text, `W` for wires and what sits on them, `H` for a child
/// sheet and `P` for its pins.
fn record_of(kind: &ObjectKind, change: Change) -> char {
    let moved = change == Change::Moved;
    match kind {
        ObjectKind::Symbol => {
            if moved {
                'L'
            } else {
                'S'
            }
        }
        ObjectKind::Field => {
            if moved {
                'F'
            } else {
                'S'
            }
        }
        ObjectKind::Wire
        | ObjectKind::Bus
        | ObjectKind::Junction
        | ObjectKind::NoConnect
        | ObjectKind::BusEntry => 'W',
        ObjectKind::Label
        | ObjectKind::GlobalLabel
        | ObjectKind::HierarchicalLabel
        | ObjectKind::NetclassFlag
        | ObjectKind::Text
        | ObjectKind::TextBox => 'T',
        ObjectKind::Sheet => 'H',
        ObjectKind::SheetPin => 'P',
        ObjectKind::Other(_) => 'O',
    }
}

/// What one line says after the name.
fn detail(change: Change, before: &SnapshotObject, after: &SnapshotObject) -> String {
    match change {
        Change::Added => summary(after.detail.as_ref()),
        Change::Removed => summary(before.detail.as_ref()),
        Change::Moved => move_detail(before.detail.as_ref(), after.detail.as_ref()),
        Change::Edited => edit_detail(before.detail.as_ref(), after.detail.as_ref()),
    }
}

/// The short description of an object, when the state carries one.
fn summary(detail: Option<&Detail>) -> String {
    detail.map_or_else(String::new, |detail| detail.summary.clone())
}

/// Two positions, when both states know them.
fn move_detail(before: Option<&Detail>, after: Option<&Detail>) -> String {
    match (before.and_then(|d| d.at), after.and_then(|d| d.at)) {
        (Some(from), Some(to)) => {
            format!("moved  ({}) -> ({})", millimetres(from), millimetres(to))
        }
        _ => "moved".to_owned(),
    }
}

/// Two values, when both states know them and they differ.
fn edit_detail(before: Option<&Detail>, after: Option<&Detail>) -> String {
    match (
        before.and_then(|d| d.value.as_deref()),
        after.and_then(|d| d.value.as_deref()),
    ) {
        (Some(old), Some(new)) if old != new => format!("{old:?} -> {new:?}"),
        _ => "edited".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::{Change, Delta, DeltaLine, record_of};
    use crate::model::items::SheetPath;
    use crate::view::snapshot::{ObjectKind, Snapshot};

    fn empty(name: &str) -> Snapshot {
        Snapshot {
            name: name.to_owned(),
            sheet_path: SheetPath("/a".to_owned()),
            taken: "2026-01-02T03:04:05Z".to_owned(),
            tool: "kicli/0.0.0".to_owned(),
            objects: Vec::new(),
        }
    }

    #[test]
    fn two_empty_states_report_nothing() {
        let delta = Delta::between(&empty("base"), &empty("current"));
        assert_eq!(
            delta.to_string(),
            "delta base -> current\n= 0 objects unchanged\n"
        );
    }

    #[test]
    fn a_moved_symbol_reports_a_placement_and_an_edited_one_a_connection() {
        assert_eq!(record_of(&ObjectKind::Symbol, Change::Moved), 'L');
        assert_eq!(record_of(&ObjectKind::Symbol, Change::Edited), 'S');
        assert_eq!(record_of(&ObjectKind::Symbol, Change::Added), 'S');
        assert_eq!(record_of(&ObjectKind::Field, Change::Moved), 'F');
        assert_eq!(record_of(&ObjectKind::Field, Change::Edited), 'S');
    }

    /// `ALL` is the enum, not a subset of it.
    ///
    /// A duplicate would hide a missing variant behind the right length, so the
    /// marks are checked for distinctness as well as for count.
    #[test]
    fn every_change_is_listed_once_with_a_mark_of_its_own() {
        let mut marks: Vec<char> = Change::ALL.iter().map(|change| change.mark()).collect();
        assert_eq!(marks.len(), Change::ALL.len());
        marks.sort_unstable();
        marks.dedup();
        assert_eq!(
            marks,
            vec!['+', '-', '~'],
            "a moved and an edited object share the `~` mark, and nothing else \
             shares one"
        );
        for change in Change::ALL {
            assert_eq!(
                Change::ALL.iter().filter(|other| **other == change).count(),
                1,
                "{change:?} appears once in ALL"
            );
        }
    }

    #[test]
    fn a_line_with_nothing_to_add_ends_at_its_name() {
        let line = DeltaLine {
            change: Change::Removed,
            kind: ObjectKind::Symbol,
            record: 'S',
            handle: "R7".to_owned(),
            detail: String::new(),
        };
        assert_eq!(line.to_string(), "- S R7");
    }
}
