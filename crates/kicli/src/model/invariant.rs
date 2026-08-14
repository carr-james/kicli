//! The cheap checks every mutation re-runs, and reports.
//!
//! Constitution §5: every mutation is verified and reported. These four checks
//! are the verification. They are pure over the model and cost a walk of the
//! tree, which is what makes running them on every mutation affordable — and
//! running them on every mutation is what makes a mistake visible immediately
//! rather than the next time KiCad opens the file.
//!
//! A failure names which check failed. "Something is wrong" is not a report.

use std::collections::BTreeSet;
use std::fmt;

use kicli_sexpr::Doc;

use crate::geometry::{Iu, Point};
use crate::model::items::{Item, Schematic, SheetPath};

/// One of the four checks.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Invariant {
    /// The written bytes parse again.
    Reparses,
    /// Every identifier a file refers to belongs to an object of that file.
    ReferencesResolve,
    /// Wire ends, junctions, no-connects, bus entries, label anchors and sheet
    /// pins are on grid.
    ///
    /// A symbol's resolved pin positions are deliberately not here. They are the
    /// lint's jurisdiction, under the blocking rule `KI-GRID-001`, because a
    /// caller may put a symbol anchor off the grid with a flag that says so and
    /// a hard refusal here would make that flag useless. The boundary is: this
    /// check judges the standalone connectable geometry a mutation wrote, and
    /// the lint judges the drawing that results.
    GeometryOnGrid,
    /// No symbol carries instance data for a sheet path that is not there.
    InstancesResolve,
}

impl Invariant {
    /// Every check, in the order a report lists them.
    pub const ALL: &'static [Self] = &[
        Self::Reparses,
        Self::ReferencesResolve,
        Self::GeometryOnGrid,
        Self::InstancesResolve,
    ];

    /// The name a report uses.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Reparses => "reparses",
            Self::ReferencesResolve => "references-resolve",
            Self::GeometryOnGrid => "geometry-on-grid",
            Self::InstancesResolve => "instances-resolve",
        }
    }

    /// What the check is for, in one sentence.
    #[must_use]
    pub const fn meaning(self) -> &'static str {
        match self {
            Self::Reparses => "the file kicli wrote reads back as a schematic",
            Self::ReferencesResolve => "every identifier the file refers to is an object in it",
            Self::GeometryOnGrid => "the connection points this file draws sit on the grid",
            Self::InstancesResolve => "no symbol carries instance data for a sheet that is gone",
        }
    }
}

impl fmt::Display for Invariant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// What one check found.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Outcome {
    /// Which check.
    pub invariant: Invariant,
    /// What it objected to. Empty means it passed.
    pub faults: Vec<String>,
}

impl Outcome {
    /// Did the check pass?
    #[must_use]
    pub fn passed(&self) -> bool {
        self.faults.is_empty()
    }
}

/// The result of running every check.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Report {
    /// One outcome per check, in the order of [`Invariant::ALL`].
    pub outcomes: Vec<Outcome>,
}

impl Report {
    /// Did every check pass?
    #[must_use]
    pub fn passed(&self) -> bool {
        self.outcomes.iter().all(Outcome::passed)
    }

    /// The checks that failed.
    pub fn failures(&self) -> impl Iterator<Item = &Outcome> {
        self.outcomes.iter().filter(|outcome| !outcome.passed())
    }
}

/// Run every check over a document and the schematic read from it.
///
/// `grid` is the project's placement grid, which the configuration may change.
///
/// # Examples
///
/// ```
/// use kicli::model::{Schematic, check_invariants};
/// use kicli::geometry::GRID;
/// use kicli_sexpr::Doc;
///
/// let doc = Doc::parse("(kicad_sch\n\t(version 20260306)\n)\n").expect("parses");
/// let schematic = Schematic::read(&doc).expect("reads");
/// assert!(check_invariants(&doc, &schematic, GRID).passed());
/// ```
#[must_use]
pub fn check_invariants(doc: &Doc, schematic: &Schematic, grid: Iu) -> Report {
    Report {
        outcomes: vec![
            Outcome {
                invariant: Invariant::Reparses,
                faults: reparses(doc),
            },
            Outcome {
                invariant: Invariant::ReferencesResolve,
                faults: references_resolve(schematic),
            },
            Outcome {
                invariant: Invariant::GeometryOnGrid,
                faults: geometry_on_grid(schematic, grid),
            },
            Outcome {
                invariant: Invariant::InstancesResolve,
                faults: instances_resolve(schematic),
            },
        ],
    }
}

/// The emitted bytes read back as the same tree.
fn reparses(doc: &Doc) -> Vec<String> {
    let written = doc.emit();
    match Doc::parse(&written) {
        Err(error) => vec![format!("the file does not parse: {error}")],
        Ok(reread) if !doc.structurally_eq(&reread) => {
            vec!["the file reads back as different tokens".to_owned()]
        }
        Ok(_) => Vec::new(),
    }
}

/// Every identifier the file uses names one object, once.
///
/// A duplicated identifier is how a half-finished copy shows up: KiCad keys its
/// own tables on these, so two objects sharing one identifier means one of them
/// is invisible to the editor.
///
/// Whether a symbol's *sheet path* resolves is a question about the hierarchy,
/// not about one file — a child sheet's paths start with the root file's
/// identifier, which the child cannot know. [`check_hierarchy`] asks that one.
fn references_resolve(schematic: &Schematic) -> Vec<String> {
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut faults = Vec::new();
    let mut claim = |uuid: &str, what: &str, faults: &mut Vec<String>| {
        if uuid.is_empty() {
            faults.push(format!("{what} has no identifier"));
        } else if !seen.insert(uuid.to_owned()) {
            faults.push(format!(
                "{uuid} names more than one object, including {what}"
            ));
        }
    };

    for item in &schematic.items {
        if let Some(uuid) = item.uuid() {
            claim(&uuid.0, "an object", &mut faults);
        }
        match item {
            Item::Symbol(symbol) => {
                for pin in &symbol.pins {
                    claim(&pin.uuid.0, "a symbol pin", &mut faults);
                }
            }
            Item::Sheet(sheet) => {
                for pin in &sheet.pins {
                    claim(&pin.uuid.0, "a sheet pin", &mut faults);
                }
            }
            _ => {}
        }
    }
    faults
}

/// The connection points this file draws sit on the grid.
///
/// Wire and bus ends, junctions, no-connects, bus entries, label anchors and
/// sheet pins. Two things are outside it, for different reasons.
///
/// Field and graphic text are exempt because KiCad's own autoplacement puts them
/// on arbitrary units, so a blanket rule would fail KiCad's own output.
///
/// A symbol's resolved pin positions are the lint's, under `KI-GRID-001`. A
/// caller may put a symbol anchor off the grid with a flag that says so, and a
/// refusal here would make that flag useless. This check judges the geometry a
/// mutation wrote; the lint judges the drawing that results.
fn geometry_on_grid(schematic: &Schematic, grid: Iu) -> Vec<String> {
    let on_grid = |point: Point| -> bool {
        grid.0 != 0 && point.x.0 % grid.0 == 0 && point.y.0 % grid.0 == 0
    };
    let mut faults = Vec::new();
    for item in &schematic.items {
        match item {
            Item::Line(line) => {
                for (name, end) in [("start", line.from), ("end", line.to)] {
                    if !on_grid(end) {
                        faults.push(format!(
                            "wire {} has its {name} off grid at {end}",
                            line.uuid.0
                        ));
                    }
                }
            }
            Item::Junction(point) | Item::NoConnect(point) | Item::BusEntry(point) => {
                if !on_grid(point.at) {
                    faults.push(format!("{} is off grid at {}", point.uuid.0, point.at));
                }
            }
            Item::Label(label) => {
                if !on_grid(label.at) {
                    faults.push(format!("label {} is off grid at {}", label.text, label.at));
                }
            }
            Item::Sheet(sheet) => {
                for pin in &sheet.pins {
                    if !on_grid(pin.at) {
                        faults.push(format!("sheet pin {} is off grid at {}", pin.name, pin.at));
                    }
                }
            }
            _ => {}
        }
    }
    faults
}

/// No symbol carries instance data for a sheet path this file cannot be on.
///
/// A symbol on a sheet placed twice has two entries. A symbol with an entry for
/// a placement that has been deleted is orphaned data, which KiCad prunes
/// silently on its next save — taking the reference with it.
fn instances_resolve(schematic: &Schematic) -> Vec<String> {
    let mut faults = Vec::new();
    for item in &schematic.items {
        let Item::Symbol(symbol) = item else { continue };
        if symbol.placements.is_empty() {
            faults.push(format!(
                "symbol {} carries no instance data, so it has no reference anywhere",
                symbol.uuid.0
            ));
            continue;
        }
        let mut seen: BTreeSet<&SheetPath> = BTreeSet::new();
        for placement in &symbol.placements {
            if !seen.insert(&placement.path) {
                faults.push(format!(
                    "symbol {} has two entries for sheet path {}",
                    symbol.uuid.0, placement.path.0
                ));
            }
            if placement.reference.0.is_empty() {
                faults.push(format!(
                    "symbol {} has an entry for {} with no reference",
                    symbol.uuid.0, placement.path.0
                ));
            }
        }
    }
    faults
}

/// The instance data of a whole project resolves against its sheet tree.
///
/// This is the check a single file cannot make. A symbol's sheet path names the
/// root screen and then each sheet item above it; an entry for a placement that
/// no longer exists is orphaned data, which KiCad prunes on its next save,
/// taking the reference with it.
#[must_use]
pub fn check_hierarchy(hierarchy: &crate::model::Hierarchy) -> Outcome {
    let known: BTreeSet<&SheetPath> = hierarchy
        .placements
        .iter()
        .map(|placement| &placement.path)
        .collect();

    let mut faults = Vec::new();
    for file in &hierarchy.files {
        for item in &file.schematic.items {
            let Item::Symbol(symbol) = item else { continue };
            for placement in &symbol.placements {
                if !known.contains(&placement.path) {
                    faults.push(format!(
                        "{}: symbol {} carries instance data for {}, which is not a placement of this project",
                        file.path.display(),
                        symbol.uuid.0,
                        placement.path.0
                    ));
                }
            }
        }
    }
    Outcome {
        invariant: Invariant::InstancesResolve,
        faults,
    }
}

#[cfg(test)]
mod tests {
    use super::{Invariant, check_invariants};
    use crate::geometry::GRID;
    use crate::model::Schematic;
    use kicli_sexpr::Doc;

    #[test]
    fn every_check_has_a_name_and_a_meaning() {
        for invariant in Invariant::ALL {
            assert!(!invariant.name().is_empty());
            assert!(!invariant.meaning().is_empty());
        }
        assert_eq!(Invariant::ALL.len(), 4, "Constitution names four checks");
    }

    #[test]
    fn an_off_grid_wire_end_is_named_by_its_own_check() {
        let source = concat!(
            "(kicad_sch\n\t(version 20260306)\n",
            "\t(wire\n\t\t(pts\n\t\t\t(xy 25.4 25.4) (xy 25.41 25.4)\n\t\t)\n",
            "\t\t(uuid \"w1\")\n\t)\n)\n"
        );
        let doc = Doc::parse(source).expect("parses");
        let schematic = Schematic::read(&doc).expect("reads");
        let report = check_invariants(&doc, &schematic, GRID);

        assert!(!report.passed());
        let failures: Vec<Invariant> = report.failures().map(|outcome| outcome.invariant).collect();
        assert_eq!(failures, [Invariant::GeometryOnGrid], "one check, not four");
        assert!(
            report.failures().next().expect("a failure").faults[0].contains("off grid"),
            "and it says what it found"
        );
    }
}
