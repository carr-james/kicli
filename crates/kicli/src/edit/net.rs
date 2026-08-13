//! Net-level edits, which are edits to the labels that name a net.
//!
//! kicli does not write the project file, so a net's name is only what the
//! drawing says it is: a label, a sheet pin that meets one, or the `Value` of
//! the power symbols on it. Renaming a net therefore renames those, everywhere
//! the net reaches, and a net that carries none of them has no name to change.
//! That is the refusal this module exists for, and it points the caller at
//! `label add` rather than leaving it guessing.
//!
//! Every file the net reaches is checked before any file is written, so a
//! rename that cannot hold is refused before it starts. Each file is then
//! written atomically by [`crate::model::mutate`], which is the only path to
//! disk.

use std::fmt;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use kicli_sexpr::{NodeId, quote};

use crate::connectivity::{Net, extract};
use crate::geometry::Iu;
use crate::model::hierarchy::Hierarchy;
use crate::model::invariant::check_invariants;
use crate::model::items::{Item, LabelKind, ReadError, Schematic, SheetPath, Symbol, Uuid};
use crate::model::mutate::{Mutation, MutationError, Target, commit, state_before};
use crate::model::write::{WriteOptions, WriteRefusal, plan_write};
use crate::view::snapshot::{Snapshot, SnapshotError};

/// The project a rename writes into, and the rules it writes under.
#[derive(Clone, Copy, Debug)]
pub struct Scope<'a> {
    /// The project directory, which holds the snapshot cache.
    pub project: &'a Path,
    /// The placement grid.
    pub grid: Iu,
    /// What kicli may and may not write.
    pub options: WriteOptions,
}

/// What a rename changed.
#[derive(Clone, Debug)]
pub struct Renamed {
    /// The name the net had.
    pub from: String,
    /// The name the net has now.
    pub to: String,
    /// One report per file that was written.
    pub mutations: Vec<Mutation>,
    /// The labels whose text changed, by handle.
    pub labels: Vec<String>,
    /// The sheet pins whose name changed, by handle.
    pub sheet_pins: Vec<String>,
    /// The power symbols whose `Value` changed, by reference designator.
    pub power_symbols: Vec<String>,
}

impl Renamed {
    /// The text form: what the rename touched, then each file's own report.
    #[must_use]
    pub fn render(&self) -> String {
        let mut text = format!("net {} is now {}\n", self.from, self.to);
        for (what, items) in [
            ("labels", &self.labels),
            ("sheet pins", &self.sheet_pins),
            ("power symbols", &self.power_symbols),
        ] {
            if !items.is_empty() {
                let _ = writeln!(text, "{what}: {}", items.join(", "));
            }
        }
        for mutation in &self.mutations {
            let _ = writeln!(text, "-- {}", mutation.path.display());
            text.push_str(&mutation.render());
        }
        text
    }

    /// The JSON form, carrying the same content.
    #[must_use]
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "from": self.from,
            "to": self.to,
            "labels": self.labels,
            "sheet_pins": self.sheet_pins,
            "power_symbols": self.power_symbols,
            "files": self
                .mutations
                .iter()
                .map(Mutation::to_json)
                .collect::<Vec<_>>(),
        })
    }
}

/// Why a net was not renamed.
///
/// Every variant is an operation error: the request is well formed and kicli
/// will not carry it out. No variant reaches the disk.
#[derive(Debug, thiserror::Error)]
pub enum NetError {
    /// The net carries no label, sheet pin or power value.
    #[error(
        "net {handle} has no label, so there is no name to change. \
         kicli reads a net's name from its labels only. \
         kicli does not write the project file. \
         Use `label add` to name the net."
    )]
    Unnamed {
        /// The handle kicli addresses the net by.
        handle: String,
    },

    /// No net of the project carries that name.
    #[error("no net of this project is called {name}.")]
    NoSuchNet {
        /// The name the caller asked for.
        name: String,
    },

    /// More than one net carries that name, so it identifies none of them.
    #[error(
        "{count} nets are called {name}, so the name does not identify one of them. \
         A local label names one net per placement of the sheet it is drawn on."
    )]
    AmbiguousNet {
        /// The name the caller asked for.
        name: String,
        /// How many nets carry it.
        count: usize,
    },

    /// Another net already carries the new name, so the rename would join them.
    #[error("another net is already called {name}. Renaming onto it would join the two nets.")]
    NameTaken {
        /// The name the caller asked for.
        name: String,
    },

    /// The new name is the old name.
    #[error("the net is already called {name}.")]
    SameName {
        /// The name the caller asked for.
        name: String,
    },

    /// The new name is empty.
    #[error("a net name cannot be empty.")]
    EmptyName,

    /// The net is named, but nothing in the files carries that text.
    #[error("net {name} is named by nothing kicli can edit in these files.")]
    NothingToRename {
        /// The name the caller asked for.
        name: String,
    },

    /// A file would not hold after the change, so no file was written.
    #[error("{path} would not hold after the rename, so nothing was written: {reason}")]
    WouldNotHold {
        /// The file that failed its check.
        path: PathBuf,
        /// Which check failed, and what it objected to.
        reason: String,
    },

    /// kicli refuses to write one of the files at all.
    #[error(transparent)]
    Refused(#[from] WriteRefusal),

    /// An edited file did not read back as a schematic.
    #[error(transparent)]
    Read(#[from] ReadError),

    /// The state to compare a change against could not be taken.
    #[error(transparent)]
    Snapshot(#[from] SnapshotError),

    /// A change did not survive its own checks, or could not be written.
    #[error(transparent)]
    Mutation(#[from] MutationError),
}

/// Rename a net, everywhere it reaches.
///
/// The net is addressed by the name kicli shows it under. Every label of the
/// net, every sheet pin that meets one, and the `Value` of every power symbol
/// on it change together, because changing some of them would split the net.
///
/// Each file is written atomically. A net whose labels are in more than one
/// file needs one write per file, so every file is checked first and the writes
/// follow only when all of them hold.
///
/// # Errors
///
/// Returns [`NetError::Unnamed`] when no label names the net, and points at
/// `label add`. Returns [`NetError::AmbiguousNet`] when two nets share the
/// name, and [`NetError::NameTaken`] when the new name would join this net to
/// another. None of them writes anything.
pub fn rename(
    hierarchy: &mut Hierarchy,
    from: &str,
    to: &str,
    scope: &Scope<'_>,
    taken: &str,
) -> Result<Renamed, NetError> {
    if to.is_empty() {
        return Err(NetError::EmptyName);
    }
    if from == to {
        return Err(NetError::SameName {
            name: to.to_owned(),
        });
    }

    let edits = {
        let nets = extract(hierarchy);
        let named: Vec<&Net> = nets.nets().iter().filter(|net| net.name == from).collect();
        let net = match named.as_slice() {
            [] => {
                return Err(NetError::NoSuchNet {
                    name: from.to_owned(),
                });
            }
            [only] => *only,
            many => {
                return Err(NetError::AmbiguousNet {
                    name: from.to_owned(),
                    count: many.len(),
                });
            }
        };
        if net.synthetic {
            return Err(NetError::Unnamed {
                handle: net.name.clone(),
            });
        }
        if nets.nets().iter().any(|other| other.name == to) {
            return Err(NetError::NameTaken {
                name: to.to_owned(),
            });
        }
        edits_for(hierarchy, net, from)
    };

    if edits.is_empty() {
        return Err(NetError::NothingToRename {
            name: from.to_owned(),
        });
    }
    apply(hierarchy, &edits, from, to, scope, taken)
}

/// Take the state of each file, change them all, check them all, write them all.
fn apply(
    hierarchy: &mut Hierarchy,
    edits: &[Edit],
    from: &str,
    to: &str,
    scope: &Scope<'_>,
    taken: &str,
) -> Result<Renamed, NetError> {
    let mut touched: Vec<usize> = edits.iter().map(|edit| edit.file).collect();
    touched.sort_unstable();
    touched.dedup();

    let mut before: Vec<Snapshot> = Vec::new();
    for &file in &touched {
        let loaded = &hierarchy.files[file];
        let sheet = sheet_path_of(hierarchy, file);
        before.push(state_before(&loaded.doc, &loaded.schematic, &sheet, taken)?);
    }

    let quoted = quote(to);
    for edit in edits {
        hierarchy.files[edit.file].doc.set_atom(edit.atom, &quoted);
    }

    // Every file is checked before any file is written, so a rename that
    // cannot hold is refused before it starts.
    for &file in &touched {
        let loaded = &hierarchy.files[file];
        let schematic = Schematic::read(&loaded.doc)?;
        plan_write(&loaded.doc, scope.options)?;
        let report = check_invariants(&loaded.doc, &schematic, scope.grid);
        if !report.passed() {
            return Err(NetError::WouldNotHold {
                path: loaded.path.clone(),
                reason: report
                    .failures()
                    .map(|outcome| format!("{}: {}", outcome.invariant, outcome.faults.join("; ")))
                    .collect::<Vec<String>>()
                    .join(" | "),
            });
        }
    }

    let mut mutations = Vec::new();
    for (index, &file) in touched.iter().enumerate() {
        let path = hierarchy.files[file].path.clone();
        let sheet = sheet_path_of(hierarchy, file);
        let target = Target {
            path: &path,
            project: scope.project,
            sheet_path: &sheet,
            grid: scope.grid,
            options: scope.options,
        };
        mutations.push(commit(
            &hierarchy.files[file].doc,
            &target,
            &before[index],
            taken,
        )?);
        hierarchy.files[file].schematic = Schematic::read(&hierarchy.files[file].doc)?;
    }

    Ok(Renamed {
        from: from.to_owned(),
        to: to.to_owned(),
        mutations,
        labels: handles(edits, What::Label),
        sheet_pins: handles(edits, What::SheetPin),
        power_symbols: handles(edits, What::PowerValue),
    })
}

/// What one edited atom belongs to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum What {
    /// A label of any kind but a netclass flag.
    Label,
    /// A pin on the border of a child sheet.
    SheetPin,
    /// The `Value` of a power symbol.
    PowerValue,
}

/// Where the text of a label or a sheet pin sits: right after the head token.
const TEXT: usize = 1;

/// Where the value of a field sits: after the head token and the field name.
const FIELD_VALUE: usize = 2;

/// One atom whose text the rename changes.
struct Edit {
    /// Which file of the hierarchy holds it.
    file: usize,
    /// The atom itself.
    atom: NodeId,
    /// What it belongs to.
    what: What,
    /// The name the report gives it.
    handle: String,
}

/// The handles of the edits of one kind, sorted.
///
/// Repeats are kept. Every edit is a different atom, and eight characters of a
/// uuid do not identify: a generated fixture gives every object the same eight,
/// so dropping repeats would report one label where two changed.
fn handles(edits: &[Edit], what: What) -> Vec<String> {
    let mut found: Vec<String> = edits
        .iter()
        .filter(|edit| edit.what == what)
        .map(|edit| edit.handle.clone())
        .collect();
    found.sort();
    found
}

/// Every atom that carries the net's name, in every file the net reaches.
fn edits_for(hierarchy: &Hierarchy, net: &Net, from: &str) -> Vec<Edit> {
    let powered: Vec<&Uuid> = net
        .pins
        .iter()
        .filter(|pin| pin.power)
        .map(|pin| &pin.symbol)
        .collect();
    let mut edits = Vec::new();

    for (file, loaded) in hierarchy.files.iter().enumerate() {
        let reaches = hierarchy
            .placements_of(file)
            .any(|placement| net.sheets.contains(&placement.path));
        if !reaches {
            continue;
        }
        let mut record = |node: NodeId, index: usize, what: What, handle: String| {
            if let Some(&atom) = loaded.doc.children(node).get(index) {
                edits.push(Edit {
                    file,
                    atom,
                    what,
                    handle,
                });
            }
        };

        for item in &loaded.schematic.items {
            match item {
                // A netclass flag carries a netclass name, not a net name.
                Item::Label(label)
                    if label.kind != LabelKind::NetclassFlag && label.text == from =>
                {
                    record(label.node, TEXT, What::Label, short(&label.uuid.0));
                }
                Item::Sheet(sheet) => {
                    for pin in &sheet.pins {
                        if pin.name == from {
                            let handle =
                                format!("{}.{}", sheet.name().unwrap_or("sheet"), pin.name);
                            record(pin.node, TEXT, What::SheetPin, handle);
                        }
                    }
                }
                Item::Symbol(symbol) if powered.contains(&&symbol.uuid) => {
                    let Some(value) = symbol.field("Value").filter(|field| field.value == from)
                    else {
                        continue;
                    };
                    let handle = reference_of(net, symbol);
                    record(value.node, FIELD_VALUE, What::PowerValue, handle);
                }
                _ => {}
            }
        }
    }
    edits
}

/// What a power symbol is called on the net it drives.
fn reference_of(net: &Net, symbol: &Symbol) -> String {
    net.pins
        .iter()
        .find(|pin| pin.symbol == symbol.uuid)
        .map_or_else(|| short(&symbol.uuid.0), |pin| pin.reference.0.clone())
}

/// A sheet path one file is drawn on, for the state a change is compared to.
fn sheet_path_of(hierarchy: &Hierarchy, file: usize) -> SheetPath {
    hierarchy
        .placements_of(file)
        .next()
        .map_or_else(|| SheetPath("/".to_owned()), |place| place.path.clone())
}

/// The first eight characters of an identifier, which is the handle a delta
/// prints.
fn short(uuid: &str) -> String {
    uuid.chars().take(8).collect()
}

impl fmt::Display for Renamed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.render())
    }
}
