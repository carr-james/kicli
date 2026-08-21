//! Every object of every committed fixture answers to a handle of its own.
//!
//! A report prints the first eight characters of an identifier so an agent can
//! type them back at a command ([`kicli::model::Uuid::short`]). That only works
//! where eight characters tell two objects apart. The committed fixtures were
//! generated with the variation in the **last** field of the identifier, so
//! every object of every fixture answered to the same handle `00000000` — and
//! **no committed fixture could exercise a command addressed by a handle at
//! all**. The M4 probe-handle chore (C5) measured it, fixed the probe crate,
//! and named the fixtures as a known second half; the M4 dogfood run (D1) is
//! the cost arriving, an agent unable to use `sch view --uuids` for anything.
//!
//! This file is the guard that the second half stays done. Two claims, and the
//! second is the one that matters:
//!
//! 1. **arithmetic** — across the committed fixture tree, the number of
//!    distinct handles equals the number of identifiers, so no two objects
//!    share one;
//! 2. **capability** — a fixture object is actually *addressed* by its handle
//!    and the command finds it. A count that is right while nothing uses it
//!    proves the arithmetic and not the capability.
//!
//! # The controls, and why they are not optional
//!
//! A sweep that reads no files reports every file clean. So the arithmetic
//! claim carries a presence control — identifiers were found, files were read,
//! and a **named** fixture is among what was read with the count it is known
//! to carry. An equality between two zeroes is not evidence.
//!
//! # What this file does not cut
//!
//! It shortens no string itself. Every handle here comes from
//! [`kicli::model::Uuid::short`], which is the one rule
//! (`cargo test --test the_handle_has_one_name`). A private copy of the
//! arithmetic here would be a second place the rule could drift, in the very
//! file that exists to defend it.

use kicli::cli::edit::address;
use kicli::model::{Schematic, Uuid};
use kicli_sexpr::Doc;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// The committed fixture tree this sweep reads.
fn fixture_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

/// The character counts of the five groups KiCad writes an identifier in.
///
/// Transcribed from the shape KiCad itself writes — `8-4-4-4-12` — rather than
/// from a regular expression, because the workspace carries no regex crate and
/// a hand-rolled matcher that agrees with a written-down shape is checkable by
/// a reader.
const GROUPS: [usize; 5] = [8, 4, 4, 4, 12];

/// A fixture named in the presence control, and how many identifiers it holds.
///
/// **The control that stops this sweep passing while blind.** A walk that found
/// the wrong root, or read nothing, reports a clean tree just as loudly as a
/// correct one does. Naming a file and its count means an empty read fails
/// rather than passes. The counts are the measurement D1 recorded, and a
/// deliberate edit to a fixture is expected to move them — that is a decision
/// somebody takes, which is the same reason the goldens exist.
const NAMED: [(&str, usize); 3] = [
    ("sch/nets/nets.kicad_sch", 117),
    ("sch/routing/calibration.kicad_sch", 240),
    ("text/calibration.kicad_sch", 873),
];

/// Is this character one an identifier is written from?
fn is_identifier_character(c: char) -> bool {
    c == '-' || c.is_ascii_hexdigit()
}

/// Does this run of characters have the shape KiCad writes an identifier in?
fn is_identifier(run: &str) -> bool {
    let groups: Vec<&str> = run.split('-').collect();
    groups.len() == GROUPS.len()
        && groups
            .iter()
            .zip(GROUPS)
            .all(|(group, want)| group.chars().count() == want)
}

/// Every identifier-shaped string in a file, in the order it appears.
///
/// Identifiers reach a fixture by more routes than the `(uuid …)` atom: a
/// sheet instance `path`, a netlist `tstamps` field and an ERC report's JSON
/// all name objects, and a handle that collides there collides just as badly.
/// So the scan is over the text rather than over the parsed model, and it is
/// deliberately the wider of the two questions.
fn identifiers(text: &str) -> Vec<String> {
    let characters: Vec<char> = text.chars().collect();
    let mut found = Vec::new();
    let mut at = 0;
    while at < characters.len() {
        if !is_identifier_character(characters[at]) {
            at += 1;
            continue;
        }
        let start = at;
        while at < characters.len() && is_identifier_character(characters[at]) {
            at += 1;
        }
        let run: String = characters[start..at].iter().collect();
        if is_identifier(&run) {
            found.push(run);
        }
    }
    found
}

/// Every `(uuid "…")` atom in a file, in the order it appears.
///
/// This is the narrower question, and it is the one D1 states: a `uuid` atom
/// is an object declaring its own identity, so one atom is one object.
fn uuid_atoms(text: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut rest = text;
    while let Some(at) = rest.find("(uuid \"") {
        rest = &rest[at + "(uuid \"".len()..];
        let Some(end) = rest.find('"') else {
            break;
        };
        found.push(rest[..end].to_owned());
        rest = &rest[end..];
    }
    found
}

/// Every readable file of the fixture tree, sorted, with its text.
///
/// Sorted rather than in the order the filesystem hands back, so a failure
/// names the same first offender on every machine and every run.
fn fixture_files() -> Vec<(String, String)> {
    let root = fixture_root();
    let mut paths = Vec::new();
    walk(&root, &mut paths);
    paths.sort();
    paths
        .into_iter()
        .filter_map(|path| {
            let named = path
                .strip_prefix(&root)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            std::fs::read_to_string(&path)
                .ok()
                .map(|text| (named, text))
        })
        .collect()
}

fn walk(directory: &Path, found: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(directory) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk(&path, found);
        } else {
            found.push(path);
        }
    }
}

/// The handle of an identifier, from the one rule and never from a copy.
fn handle(identifier: &str) -> String {
    Uuid(identifier.to_owned()).short().to_owned()
}

#[test]
fn every_committed_fixture_object_answers_to_a_handle_of_its_own() {
    let files = fixture_files();

    // The presence control, before any conclusion is drawn from an equality.
    // A sweep that read nothing would find no collision and report success.
    assert!(
        files.len() >= 20,
        "the fixture tree was read: {} files under {}",
        files.len(),
        fixture_root().display()
    );
    let counts: BTreeMap<&str, usize> = files
        .iter()
        .map(|(named, text)| (named.as_str(), identifiers(text).len()))
        .collect();
    for (named, want) in NAMED {
        assert_eq!(
            counts.get(named).copied(),
            Some(want),
            "{named} was read and holds the identifiers D1 measured; the tree \
             this sweep actually read is {counts:#?}"
        );
    }

    // Claim 1, as D1 states it: one `uuid` atom is one object.
    let atoms: Vec<(String, String)> = files
        .iter()
        .flat_map(|(named, text)| {
            uuid_atoms(text)
                .into_iter()
                .map(move |atom| (named.clone(), atom))
        })
        .collect();
    assert!(
        atoms.len() >= 300,
        "the fixture tree carries uuid atoms: {}",
        atoms.len()
    );
    let atom_handles: BTreeSet<String> = atoms.iter().map(|(_, atom)| handle(atom)).collect();
    assert_eq!(
        atom_handles.len(),
        atoms.len(),
        "every uuid atom of the committed fixture tree must answer to a handle \
         of its own, or no fixture can exercise a command addressed by one. \
         {} atoms share {} handles. The sharers are: {:#?}",
        atoms.len(),
        atom_handles.len(),
        sharers(
            atoms
                .iter()
                .map(|(named, atom)| (named.clone(), atom.clone()))
        )
    );

    // The wider question. An identifier also reaches a fixture through a sheet
    // instance `path`, a netlist `tstamps` and an ERC report, and a collision
    // there is the same defect wearing different clothes.
    let named_everywhere: Vec<(String, String)> = files
        .iter()
        .flat_map(|(named, text)| {
            identifiers(text)
                .into_iter()
                .map(move |found| (named.clone(), found))
        })
        .collect();
    let distinct: BTreeSet<&String> = named_everywhere.iter().map(|(_, id)| id).collect();
    let handles: BTreeSet<String> = distinct.iter().map(|id| handle(id)).collect();
    assert!(
        distinct.len() >= 1000,
        "the fixture tree carries identifiers: {}",
        distinct.len()
    );
    assert_eq!(
        handles.len(),
        distinct.len(),
        "every identifier the committed fixture tree names — in a uuid atom, a \
         sheet path, a netlist or an oracle — must answer to a handle of its \
         own. {} identifiers share {} handles. The sharers are: {:#?}",
        distinct.len(),
        handles.len(),
        sharers(named_everywhere.into_iter())
    );
}

/// The identifiers that share a handle, grouped by the handle they share.
///
/// What a failure prints. A count says the tree is broken; this says where.
fn sharers(
    found: impl Iterator<Item = (String, String)>,
) -> BTreeMap<String, BTreeSet<(String, String)>> {
    let mut by_handle: BTreeMap<String, BTreeSet<(String, String)>> = BTreeMap::new();
    for (named, identifier) in found {
        by_handle
            .entry(handle(&identifier))
            .or_default()
            .insert((identifier, named));
    }
    by_handle.retain(|_, sharing| sharing.len() > 1);
    by_handle
}

/// Read a committed fixture into the typed model.
fn read(relative: &str) -> (Doc, Schematic) {
    let path = fixture_root().join(relative);
    let source = std::fs::read_to_string(&path).expect("the fixture is readable");
    let doc = Doc::parse(&source).expect("the fixture parses");
    let schematic = Schematic::read(&doc).expect("the fixture reads as a schematic");
    (doc, schematic)
}

#[test]
fn a_fixture_object_is_addressed_by_its_handle_and_found() {
    // Two fixtures, because the defect this guards was found in a multi-sheet
    // case after a single-sheet control had waved it through (C5's own first
    // pass).
    for fixture in ["sch/nets/nets.kicad_sch", "sch/item_zoo.kicad_sch"] {
        let (_doc, schematic) = read(fixture);

        let addressable: Vec<&Uuid> = schematic
            .items
            .iter()
            .filter_map(kicli::model::Item::uuid)
            .collect();

        // The control. Addressing every object of an empty list succeeds
        // vacuously, which is the shape of a check that proves nothing.
        assert!(
            addressable.len() >= 10,
            "{fixture} holds objects that carry an identifier: {}",
            addressable.len()
        );

        for uuid in &addressable {
            let found = address::item(&schematic, uuid.short()).unwrap_or_else(|failure| {
                panic!(
                    "{fixture}: {} is what a view prints for {}, and it must be \
                     what a command accepts back. It was refused: {failure:?}",
                    uuid.short(),
                    uuid.0
                )
            });
            assert_eq!(
                found.uuid(),
                Some(*uuid),
                "{fixture}: the handle {} found a different object",
                uuid.short()
            );
        }
    }
}

#[test]
fn a_handle_no_fixture_object_carries_is_refused() {
    // The other half of the capability: addressing succeeds because the object
    // is there, not because the resolver accepts anything it is handed.
    let (_doc, schematic) = read("sch/nets/nets.kicad_sch");
    assert!(
        address::item(&schematic, "ffffffff").is_err(),
        "a handle no object of the fixture carries is refused"
    );
}
