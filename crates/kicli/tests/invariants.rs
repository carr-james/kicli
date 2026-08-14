//! The four checks catch what they are named for, and nothing else.

use kicli::geometry::GRID;
use kicli::model::{Invariant, Schematic, check_invariants};
use kicli_sexpr::Doc;
use std::path::{Path, PathBuf};

fn fixture_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

/// Which checks a source fails.
fn failures(source: &str) -> Vec<Invariant> {
    let doc = Doc::parse(source).expect("the source parses");
    let schematic = Schematic::read(&doc).expect("it reads as a schematic");
    check_invariants(&doc, &schematic, GRID)
        .failures()
        .map(|outcome| outcome.invariant)
        .collect()
}

/// A sheet with one symbol, built around a body that a test replaces.
fn sheet_with(symbol: &str) -> String {
    format!(
        concat!(
            "(kicad_sch\n\t(version 20260306)\n\t(uuid \"root\")\n",
            "\t(lib_symbols\n\t\t(symbol \"Test:R\"\n\t\t\t(symbol \"R_1_1\"\n",
            "\t\t\t\t(pin passive line\n\t\t\t\t\t(at 0 3.81 270)\n\t\t\t\t\t(length 1.27)\n",
            "\t\t\t\t\t(name \"\")\n\t\t\t\t\t(number \"1\")\n\t\t\t\t)\n\t\t\t)\n\t\t)\n\t)\n",
            "{}\n)\n"
        ),
        symbol
    )
}

const GOOD_SYMBOL: &str = concat!(
    "\t(symbol\n\t\t(lib_id \"Test:R\")\n\t\t(at 50.8 50.8 0)\n\t\t(uuid \"s1\")\n",
    "\t\t(property \"Reference\" \"R1\" (at 50.8 50.8 0))\n",
    "\t\t(pin \"1\" (uuid \"p1\"))\n",
    "\t\t(instances (project \"t\" (path \"/root\" (reference \"R1\") (unit 1))))\n\t)"
);

#[test]
fn invariants_catch_what_they_are_named_for() {
    // A clean sheet passes every check.
    assert!(failures(&sheet_with(GOOD_SYMBOL)).is_empty());

    // Off-grid connectable geometry.
    let off_grid = sheet_with(&format!(
        "{GOOD_SYMBOL}\n\t(junction\n\t\t(at 25.41 25.4)\n\t\t(uuid \"j1\")\n\t)"
    ));
    assert_eq!(failures(&off_grid), [Invariant::GeometryOnGrid]);

    // Two objects sharing one identifier: one of them is invisible to KiCad.
    let doubled_uuid = sheet_with(&format!(
        "{GOOD_SYMBOL}\n\t(junction\n\t\t(at 25.4 25.4)\n\t\t(uuid \"s1\")\n\t)"
    ));
    assert_eq!(failures(&doubled_uuid), [Invariant::ReferencesResolve]);

    // A symbol with no instance data at all has no reference anywhere.
    let no_instances = sheet_with(&GOOD_SYMBOL.replace(
        "\t\t(instances (project \"t\" (path \"/root\" (reference \"R1\") (unit 1))))\n",
        "",
    ));
    assert_eq!(failures(&no_instances), [Invariant::InstancesResolve]);

    // The same sheet path twice, which KiCad prunes silently on its next save.
    let doubled = sheet_with(&GOOD_SYMBOL.replace(
        "(path \"/root\" (reference \"R1\") (unit 1))",
        "(path \"/root\" (reference \"R1\") (unit 1)) (path \"/root\" (reference \"R9\") (unit 1))",
    ));
    assert_eq!(failures(&doubled), [Invariant::InstancesResolve]);
}

#[test]
fn a_failure_says_which_check_and_what_it_found() {
    let doc = Doc::parse(&sheet_with(&format!(
        "{GOOD_SYMBOL}\n\t(junction\n\t\t(at 25.41 25.4)\n\t\t(uuid \"j1\")\n\t)"
    )))
    .expect("parses");
    let schematic = Schematic::read(&doc).expect("reads");
    let report = check_invariants(&doc, &schematic, GRID);

    let failure = report.failures().next().expect("one check failed");
    assert_eq!(failure.invariant.name(), "geometry-on-grid");
    assert!(!failure.invariant.meaning().is_empty());
    assert!(
        failure.faults[0].contains("j1") && failure.faults[0].contains("off grid"),
        "the fault names the object and what is wrong: {:?}",
        failure.faults
    );
}

#[test]
fn instance_data_for_a_placement_that_is_gone_is_caught() {
    // A single file cannot know whether its sheet paths resolve: a child
    // sheet's paths start with the root file's identifier. The hierarchy can.
    use kicli::model::{Hierarchy, check_hierarchy};

    let healthy = Hierarchy::load(&fixture_root().join("sch/nets/nets.kicad_sch")).expect("loads");
    assert!(
        check_hierarchy(&healthy).passed(),
        "a project whose instance data resolves"
    );

    let scratch = Path::new(env!("CARGO_TARGET_TMPDIR")).join("orphaned_instances");
    let _ = std::fs::remove_dir_all(&scratch);
    std::fs::create_dir_all(&scratch).expect("the directory is made");
    for name in ["nets.kicad_sch", "nets_channel.kicad_sch", "nets.kicad_pro"] {
        std::fs::copy(
            fixture_root().join("sch/nets").join(name),
            scratch.join(name),
        )
        .expect("the fixture copies");
    }
    // Point one placement's instance data at a sheet that is not there.
    let child = scratch.join("nets_channel.kicad_sch");
    let text = std::fs::read_to_string(&child).expect("reads");
    std::fs::write(
        &child,
        text.replace(
            "00000000-0000-4000-8000-03b000000001",
            "00000000-0000-4000-8000-0deadbeef001",
        ),
    )
    .expect("writes");

    let broken = Hierarchy::load(&scratch.join("nets.kicad_sch")).expect("loads");
    let outcome = check_hierarchy(&broken);
    assert!(!outcome.passed(), "the orphaned instance data is found");
    assert!(
        outcome.faults[0].contains("0deadbeef001"),
        "and the fault names the path: {:?}",
        outcome.faults
    );
}

#[test]
fn invariants_pass_on_every_fixture() {
    // Every committed schematic is clean before anything mutates it, so a
    // failure after a mutation belongs to the mutation.
    let root = fixture_root();
    let manifest = std::fs::read_to_string(root.join("MANIFEST")).expect("manifest reads");
    let mut checked = 0;
    for line in manifest.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let path = line.split_whitespace().next().unwrap_or_default();
        if !path.ends_with(".kicad_sch") {
            continue;
        }
        // The broken project exists to be wrong; it is checked by project check.
        if path.contains("project/broken") || path.contains("project/cycle") {
            continue;
        }
        // This one exists to be refused, and unreadable_numbers checks that it
        // is. It never reaches the invariant check because it never loads.
        if path.contains("unreadable_coordinate") {
            continue;
        }
        let source = std::fs::read_to_string(root.join(path)).expect("fixture reads");
        let doc = Doc::parse(&source).expect("fixture parses");
        let schematic = Schematic::read(&doc).expect("fixture reads");
        let report = check_invariants(&doc, &schematic, GRID);
        assert!(
            report.passed(),
            "{path} is not clean: {:?}",
            report.failures().collect::<Vec<_>>()
        );
        checked += 1;
    }
    assert!(checked >= 8, "the fixture tree was actually walked");
}
