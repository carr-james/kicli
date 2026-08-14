//! No test file builds its own copy of the probe harness.
//!
//! The harness was scattered before it was a crate: nine copies of the tool
//! lookup, five of the netlist export, three of the netlist reader, and a
//! scratch-copy module every writing test declared. A copy drifts, and a probe
//! built on a drifted instrument measures the drift.
//!
//! `kicli-sexpr` is out of scope on purpose. That crate depends on nothing of
//! ours, and a dev-dependency on the harness would pull `kicli` into its test
//! build, which is the boundary `ENGINEERING.md` draws.

use std::path::{Path, PathBuf};

/// A helper that belongs to the harness, and how it is written in a file.
const PROMOTED: [&str; 8] = [
    "fn kicad_cli(",
    "fn export_netlist(",
    "fn kicad_partition(",
    "fn kicad_nets(",
    "fn node_label(",
    "fn read_report(",
    "fn copy_project(",
    "struct Probe ",
];

/// Every test source of a crate.
fn sources(root: &Path, found: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            sources(&path, found);
        } else if path.extension().is_some_and(|end| end == "rs") {
            found.push(path);
        }
    }
}

fn workspace() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("the workspace root resolves")
}

#[test]
fn no_test_file_carries_its_own_copy_of_the_harness() {
    let workspace = workspace();
    let mut files = Vec::new();
    sources(&workspace.join("crates/kicli/tests"), &mut files);
    files.sort();
    assert!(files.len() > 20, "the tests of this crate were found");

    let mut offenders = Vec::new();
    for file in &files {
        if file.ends_with("probe_harness_has_one_home.rs") {
            continue;
        }
        let text = std::fs::read_to_string(file).expect("a test source reads");
        for helper in PROMOTED {
            if text.contains(helper) {
                offenders.push(format!(
                    "{}: {helper}",
                    file.strip_prefix(&workspace).unwrap_or(file).display()
                ));
            }
        }
        assert!(
            !text.contains("mod support;"),
            "{}: the scratch copy lives in kicli-probe",
            file.strip_prefix(&workspace).unwrap_or(file).display()
        );
    }
    assert!(
        offenders.is_empty(),
        "these belong to kicli-probe and are copied instead: {offenders:?}"
    );
}

#[test]
fn every_promoted_helper_is_in_the_harness() {
    // The control. A sweep for names that are nowhere would pass while the
    // copies it means to forbid sat under different names, so each name is
    // checked to be a thing the harness really has.
    let workspace = workspace();
    let mut files = Vec::new();
    sources(&workspace.join("crates/kicli-probe/src"), &mut files);
    let harness: String = files
        .iter()
        .map(|file| std::fs::read_to_string(file).expect("a harness source reads"))
        .collect();
    assert!(!harness.is_empty(), "the harness sources were found");

    // Each promoted helper, under the name the harness gives it.
    for wanted in [
        "fn found(",
        "fn try_netlist(",
        "fn partition(",
        "fn nets(",
        "fn parse(",
        "fn pin_positions(",
        "fn copy_project(",
        "struct Probe ",
    ] {
        assert!(harness.contains(wanted), "the harness carries {wanted}");
    }
}
