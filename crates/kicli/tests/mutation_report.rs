//! A mutation reports what it touched, and leaves the state to compare against.

use kicli::geometry::GRID;
use kicli::model::{LAST_WRITE, Schematic, SheetPath, Target, WriteOptions, commit, state_before};
use kicli::view::snapshot::Snapshot;
use kicli_sexpr::Doc;
use std::path::{Path, PathBuf};

const SHEET: &str = concat!(
    "(kicad_sch\n\t(version 20260306)\n\t(uuid \"root\")\n\t(paper \"A4\")\n",
    "\t(junction\n\t\t(at 25.4 25.4)\n\t\t(uuid \"j1\")\n\t)\n)\n"
);

fn scratch(name: &str) -> PathBuf {
    let directory = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory).expect("the directory is made");
    directory
}

/// Move the junction, and report it.
fn move_junction(doc: &mut Doc, to: &str) {
    let root = doc.root().expect("root");
    let junction = doc
        .children(root)
        .iter()
        .copied()
        .find(|&child| doc.head_is(child, "junction"))
        .expect("the sheet has a junction");
    let at = doc
        .children(junction)
        .iter()
        .copied()
        .find(|&child| doc.head_is(child, "at"))
        .expect("it has a position");
    let x = doc.children(at)[1];
    doc.set_atom(x, to);
}

#[test]
fn a_mutation_reports_what_it_touched() {
    let project = scratch("mutation_report");
    let file = project.join("board.kicad_sch");
    std::fs::write(&file, SHEET).expect("written");

    let mut doc = Doc::parse(SHEET).expect("parses");
    let schematic = Schematic::read(&doc).expect("reads");
    let path = SheetPath::root(schematic.uuid.as_ref().expect("has a uuid"));
    let before = state_before(&doc, &schematic, &path, "2026-01-02T03:04:05Z").expect("snapshots");

    move_junction(&mut doc, "38.1");
    let target = Target {
        path: &file,
        project: &project,
        sheet_path: &path,
        grid: GRID,
        options: WriteOptions::default(),
    };
    let mutation =
        commit(&doc, &target, &before, "2026-01-02T03:04:06Z").expect("the change is written");

    assert_eq!(mutation.delta.lines.len(), 1, "one object moved");
    assert!(
        mutation.invariants.passed(),
        "and every invariant held: {:?}",
        mutation.invariants.failures().collect::<Vec<_>>()
    );
    assert!(!mutation.reformatted);

    let text = mutation.render();
    assert!(text.contains("moved"), "{text}");
    assert!(text.contains("checked: every invariant passed"), "{text}");

    let json = mutation.to_json();
    assert_eq!(json["changed"].as_array().expect("a list").len(), 1);
    assert_eq!(json["reformatted"], false);
    assert_eq!(
        json["invariants"].as_array().expect("a list").len(),
        4,
        "all four checks are reported, not only the failures"
    );

    // The file on disk carries the change.
    let written = std::fs::read_to_string(&file).expect("reads");
    assert!(written.contains("(at 38.1 25.4)"), "{written}");
}

#[test]
fn the_last_write_snapshot_follows_every_mutation() {
    let project = scratch("last_write");
    let file = project.join("board.kicad_sch");
    std::fs::write(&file, SHEET).expect("written");

    let mut doc = Doc::parse(SHEET).expect("parses");
    let schematic = Schematic::read(&doc).expect("reads");
    let path = SheetPath::root(schematic.uuid.as_ref().expect("uuid"));

    // First mutation.
    let before = state_before(&doc, &schematic, &path, "t0").expect("snapshots");
    move_junction(&mut doc, "38.1");
    let target = Target {
        path: &file,
        project: &project,
        sheet_path: &path,
        grid: GRID,
        options: WriteOptions::default(),
    };
    commit(&doc, &target, &before, "t1").expect("writes");

    // Second mutation, compared against what the first left behind.
    let saved = Snapshot::read_in(&project, LAST_WRITE).expect("the snapshot is on disk");
    let mut doc = Doc::parse(&std::fs::read_to_string(&file).expect("reads")).expect("parses");
    move_junction(&mut doc, "50.8");
    let second = commit(&doc, &target, &saved, "t2").expect("writes");

    assert_eq!(
        second.delta.lines.len(),
        1,
        "the second change is reported against the first, not against the start"
    );
    assert!(
        second
            .delta
            .to_string()
            .contains("(38.10,25.40) -> (50.80,25.40)"),
        "and it names the move it made: {}",
        second.delta
    );
}

#[test]
fn a_change_that_breaks_an_invariant_is_not_written() {
    let project = scratch("refused_mutation");
    let file = project.join("board.kicad_sch");
    std::fs::write(&file, SHEET).expect("written");

    let mut doc = Doc::parse(SHEET).expect("parses");
    let schematic = Schematic::read(&doc).expect("reads");
    let path = SheetPath::root(schematic.uuid.as_ref().expect("uuid"));
    let before = state_before(&doc, &schematic, &path, "t0").expect("snapshots");

    // Off the grid, which is a blocking fault for connectable geometry.
    move_junction(&mut doc, "38.11");
    let refused = commit(
        &doc,
        &Target {
            path: &file,
            project: &project,
            sheet_path: &path,
            grid: GRID,
            options: WriteOptions::default(),
        },
        &before,
        "t1",
    );

    assert!(refused.is_err(), "an off-grid junction is not written");
    assert_eq!(
        std::fs::read_to_string(&file).expect("reads"),
        SHEET,
        "and the file is untouched"
    );
}
