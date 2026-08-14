//! A reference designator belongs to a sheet path, not to a symbol.
//!
//! The cached `Reference` property holds whichever sheet was loaded last. The
//! truth is the `instances` entry for one sheet path. A sheet placed twice has
//! two of them, and an edit that names one path must leave the other alone.

use kicli::edit::field::{FieldAddress, set_value};
use kicli::geometry::GRID;
use kicli::model::{Refdes, Schematic, SheetPath, Symbol, Target, Uuid, WriteOptions};
use kicli_sexpr::Doc;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

mod support;

/// The resistor of the child sheet, which the sheet's two placements name
/// `R100` and `R200`.
const RESISTOR: &str = "00000000-0000-4000-8000-03000000005c";

/// The first placement of the child sheet.
const CHANNEL_A: &str =
    "/00000000-0000-4000-8000-030000000000/00000000-0000-4000-8000-03a000000001";

/// The second placement of the same child sheet.
const CHANNEL_B: &str =
    "/00000000-0000-4000-8000-030000000000/00000000-0000-4000-8000-03b000000001";

/// Copy the fixture project into a scratch directory and hand back the copy.
fn scratch_copy(name: &str) -> PathBuf {
    support::scratch_directory(name, "sch/nets")
}

fn read(path: &Path) -> Doc {
    Doc::parse(&std::fs::read_to_string(path).expect("the file reads")).expect("it parses")
}

fn address(field: &str) -> FieldAddress {
    FieldAddress {
        owner: Uuid(RESISTOR.to_owned()),
        name: field.to_owned(),
    }
}

fn target<'a>(file: &'a Path, project: &'a Path, sheet_path: &'a SheetPath) -> Target<'a> {
    Target {
        path: file,
        project,
        sheet_path,
        grid: GRID,
        options: WriteOptions::default(),
    }
}

/// The resistor, as the written file now holds it.
fn resistor_of(file: &Path) -> Symbol {
    let doc = read(file);
    let schematic = Schematic::read(&doc).expect("the file reads as a schematic");
    schematic
        .symbols()
        .find(|symbol| symbol.uuid.0 == RESISTOR)
        .cloned()
        .expect("the resistor is still there")
}

fn reference_on(symbol: &Symbol, path: &str) -> String {
    symbol
        .reference_on(&SheetPath(path.to_owned()))
        .map_or_else(String::new, |reference| reference.0.clone())
}

#[test]
fn setting_a_reference_updates_the_path_that_was_asked_for() {
    let project = scratch_copy("edit_field_reference");
    let file = project.join("nets_channel.kicad_sch");
    let path = SheetPath(CHANNEL_A.to_owned());
    let mut doc = read(&file);

    let mutation = set_value(
        &mut doc,
        &target(&file, &project, &path),
        &address("Reference"),
        "R150",
        "2026-01-02T03:04:05Z",
    )
    .expect("the reference is written");

    assert!(
        mutation.invariants.passed(),
        "every invariant held: {:?}",
        mutation.invariants.failures().collect::<Vec<_>>()
    );

    let symbol = resistor_of(&file);
    assert_eq!(
        symbol.field("Reference").map(|field| field.value.as_str()),
        Some("R150"),
        "the cached property carries the new reference"
    );
    assert_eq!(
        reference_on(&symbol, CHANNEL_A),
        "R150",
        "and so does the instance entry for the path that was named"
    );
    assert_eq!(
        reference_on(&symbol, CHANNEL_B),
        "R200",
        "while the other placement keeps the reference it had"
    );

    // The cache and the truth move together, so a comparison reports both.
    let changed: Vec<String> = mutation
        .delta
        .lines
        .iter()
        .map(|line| line.handle.clone())
        .collect();
    assert!(
        changed.iter().any(|handle| handle.ends_with(".Reference")),
        "the field is reported as changed: {changed:?}"
    );
    assert_eq!(changed.len(), 2, "the symbol and its field: {changed:?}");
}

#[test]
fn setting_a_reference_needs_a_path_the_symbol_is_placed_on() {
    let project = scratch_copy("edit_field_reference_unplaced");
    let file = project.join("nets_channel.kicad_sch");
    let before = std::fs::read_to_string(&file).expect("the file reads");
    let path = SheetPath("/00000000-0000-4000-8000-030000000000/nowhere".to_owned());
    let mut doc = read(&file);

    let refused = set_value(
        &mut doc,
        &target(&file, &project, &path),
        &address("Reference"),
        "R150",
        "2026-01-02T03:04:05Z",
    );

    assert!(refused.is_err(), "a path the symbol is not on is refused");
    assert_eq!(
        std::fs::read_to_string(&file).expect("the file reads"),
        before,
        "and the file is untouched"
    );
}

#[test]
fn setting_a_value_touches_no_instance_data() {
    let project = scratch_copy("edit_field_value");
    let file = project.join("nets_channel.kicad_sch");
    let before = std::fs::read_to_string(&file).expect("the file reads");
    let path = SheetPath(CHANNEL_A.to_owned());
    let mut doc = read(&file);

    let mutation = set_value(
        &mut doc,
        &target(&file, &project, &path),
        &address("Value"),
        "22k",
        "2026-01-02T03:04:05Z",
    )
    .expect("the value is written");

    let symbol = resistor_of(&file);
    assert_eq!(
        symbol.field("Value").map(|field| field.value.as_str()),
        Some("22k")
    );
    assert_eq!(reference_on(&symbol, CHANNEL_A), "R100");
    assert_eq!(reference_on(&symbol, CHANNEL_B), "R200");
    assert_eq!(
        mutation.delta.lines.len(),
        1,
        "one object changed: {}",
        mutation.delta
    );

    let after = std::fs::read_to_string(&file).expect("the file reads");
    let block = instance_block(&before);
    assert!(
        block.iter().any(|line| line.contains("R200")),
        "the block under test holds the instance data: {block:?}"
    );
    assert_eq!(
        block,
        instance_block(&after),
        "the instance data is byte-identical"
    );
    assert_ne!(before, after, "and the value did change");
}

/// Every line of the file that belongs to an `instances` list.
///
/// The lists are indented four tabs deep and hold the paths and references. A
/// value edit must leave every one of those lines exactly as it found them.
fn instance_block(text: &str) -> Vec<&str> {
    let mut lines = Vec::new();
    let mut depth = 0_usize;
    for line in text.lines() {
        if line.trim_start().starts_with("(instances") {
            depth = 1;
        }
        if depth > 0 {
            lines.push(line);
            depth += line.matches('(').count();
            depth -= line.matches(')').count().min(depth);
            if depth <= 1 && line.trim_start().starts_with(')') {
                depth = 0;
            }
        }
    }
    lines
}

/// The `kicad-cli` to run, or nothing when the caller did not ask for it.
fn kicad_cli() -> Option<String> {
    std::env::var("KICLI_TEST_KICAD_CLI").ok()?;
    Some(std::env::var("KICLI_KICAD_CLI").unwrap_or_else(|_| "kicad-cli".to_owned()))
}

/// Ask KiCad for the netlist of a project it has never seen before.
///
/// The tool's own output is dropped: the first run on a machine prints
/// fontconfig warnings that say nothing about the netlist.
fn export_netlist(tool: &str, root: &Path, into: &Path) -> Option<String> {
    let status = Command::new(tool)
        .args(["sch", "export", "netlist", "-o"])
        .arg(into)
        .arg(root)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .ok()?;
    status
        .success()
        .then(|| std::fs::read_to_string(into).ok())?
}

/// What KiCad calls one symbol, on one sheet path.
///
/// A netlist writes the symbol's own identifier as `tstamps`, and the sheet
/// path it is on as the sheet's `tstamps`, which ends in a separator.
fn netlist_reference(netlist: &str, symbol: &str, sheet_path: &str) -> Option<String> {
    let doc = Doc::parse(netlist).expect("the netlist parses");
    let root = doc.root().expect("the netlist has a root list");
    let value = |list: kicli_sexpr::NodeId, head: &str| -> Option<String> {
        let child = doc
            .children(list)
            .iter()
            .copied()
            .find(|&child| doc.head_is(child, head))?;
        doc.children(child)
            .get(1)
            .and_then(|&atom| doc.atom_as_str(atom))
    };
    let leaf = sheet_path.rsplit('/').next().unwrap_or_default();
    for &components in doc.children(root) {
        if !doc.head_is(components, "components") {
            continue;
        }
        for &component in doc.children(components) {
            if !doc.head_is(component, "comp") || value(component, "tstamps")? != symbol {
                continue;
            }
            let sheet = doc
                .children(component)
                .iter()
                .copied()
                .find(|&child| doc.head_is(child, "sheetpath"))?;
            if value(sheet, "tstamps")?.contains(leaf) {
                return value(component, "ref");
            }
        }
    }
    None
}

#[test]
fn kicad_agrees_about_the_reference() {
    let Some(tool) = kicad_cli() else {
        eprintln!("skipped: set KICLI_TEST_KICAD_CLI to ask KiCad about the written file");
        return;
    };
    let project = scratch_copy("edit_field_reference_oracle");
    let file = project.join("nets_channel.kicad_sch");
    let path = SheetPath(CHANNEL_A.to_owned());
    let mut doc = read(&file);

    set_value(
        &mut doc,
        &target(&file, &project, &path),
        &address("Reference"),
        "R150",
        "2026-01-02T03:04:05Z",
    )
    .expect("the reference is written");

    let symbol = resistor_of(&file);
    assert_eq!(symbol.reference_on(&path), Some(&Refdes("R150".to_owned())));

    let netlist = export_netlist(
        &tool,
        &project.join("nets.kicad_sch"),
        &project.join("after.netlist"),
    )
    .expect("kicad-cli exported a netlist of the file kicli wrote");

    assert_eq!(
        netlist_reference(&netlist, RESISTOR, CHANNEL_A).as_deref(),
        Some("R150"),
        "KiCad names the symbol as kicli says it did, on the path that was edited"
    );
    assert_eq!(
        netlist_reference(&netlist, RESISTOR, CHANNEL_B).as_deref(),
        Some("R200"),
        "and as it did before, on the path that was not"
    );
}
