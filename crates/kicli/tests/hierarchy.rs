//! A project's sheet tree loads, with one reference per placement.

use kicli::model::{Hierarchy, Problem};
use std::path::{Path, PathBuf};

fn fixture(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(relative)
}

#[test]
fn sheet_paths_carry_their_own_references() {
    let tree = Hierarchy::load(&fixture("sch/multi_instance/multi_instance.kicad_sch"))
        .expect("the root loads");
    assert!(tree.problems.is_empty(), "{:?}", tree.problems);

    // One file drawn once, placed twice: two placements, two files loaded
    // (the root and the child), and two references for the one symbol.
    assert_eq!(tree.files.len(), 2, "the child file is parsed once");
    assert_eq!(tree.placements.len(), 3, "the root and two placements");

    let mut references: Vec<String> = tree
        .references()
        .into_iter()
        .map(|(_path, _symbol, reference)| reference)
        .collect();
    references.sort();
    assert_eq!(references, ["R201", "R301"]);

    let mut names: Vec<&str> = tree
        .placements
        .iter()
        .filter_map(|placement| placement.name.as_deref())
        .collect();
    names.sort_unstable();
    assert_eq!(names, ["channel_a", "channel_b"]);

    let pages: Vec<&str> = tree
        .placements
        .iter()
        .filter_map(|placement| placement.page.as_deref())
        .collect();
    assert_eq!(pages, ["2", "3"], "each placement carries its own page");
}

#[test]
fn a_sheet_placed_twice_gives_every_symbol_two_references() {
    let tree = Hierarchy::load(&fixture("sch/nets/nets.kicad_sch")).expect("the root loads");
    assert!(tree.problems.is_empty(), "{:?}", tree.problems);
    assert_eq!(tree.files.len(), 2);

    let references = tree.references();
    for expected in ["R100", "R101", "R200", "R201"] {
        assert!(
            references.iter().any(|(_, _, name)| name == expected),
            "{expected} is one of the references in the tree"
        );
    }

    // The same symbol object answers for both paths, and its two answers
    // differ. That is the property the cached Reference field cannot hold.
    let (first_path, symbol, first) = references
        .iter()
        .find(|(_, _, name)| name == "R100")
        .expect("R100 is in the tree");
    let other = references
        .iter()
        .find(|(path, other, _)| path != first_path && other.uuid == symbol.uuid)
        .expect("the same symbol appears on the other placement");
    assert_eq!(first, "R100");
    assert_eq!(other.2, "R200");
}

#[test]
fn sheet_cycle_is_reported_not_recursed() {
    let tree = Hierarchy::load(&fixture("project/cycle/cycle.kicad_sch")).expect("the root loads");
    assert_eq!(tree.placements.len(), 2, "the walk stops at the cycle");
    assert_eq!(tree.problems.len(), 1, "{:?}", tree.problems);
    match &tree.problems[0] {
        Problem::Cycle { file, .. } => assert_eq!(file, "cycle.kicad_sch"),
        other => panic!("expected a cycle, got {other:?}"),
    }
}

#[test]
fn a_missing_child_file_is_a_problem_and_not_a_failure() {
    let tree =
        Hierarchy::load(&fixture("project/broken/broken.kicad_sch")).expect("the root loads");

    // Three sheets: one names a file that is not there, one has a stamp above
    // the ceiling but still reads, one carries a comment and still reads.
    let missing: Vec<&Problem> = tree
        .problems
        .iter()
        .filter(|problem| matches!(problem, Problem::MissingFile { .. }))
        .collect();
    assert_eq!(missing.len(), 1, "{:?}", tree.problems);
    assert!(
        format!("{}", missing[0]).contains("absent.kicad_sch"),
        "the report names the file: {}",
        missing[0]
    );
    assert_eq!(
        tree.placements.len(),
        3,
        "the two readable sheets still load"
    );
}

#[test]
fn a_healthy_project_loads_without_problems() {
    let tree =
        Hierarchy::load(&fixture("project/healthy/healthy.kicad_sch")).expect("the root loads");
    assert!(tree.problems.is_empty(), "{:?}", tree.problems);
    assert_eq!(tree.files.len(), 2);
    assert_eq!(tree.placements.len(), 2);
    assert_eq!(tree.placements[1].name.as_deref(), Some("stage"));
    assert_eq!(tree.placements[1].page.as_deref(), Some("2"));
}
