//! The two net counts an agent meets are held to the relationship the document
//! states.
//!
//! `project info` prints `nets N`; the connectivity view prints `nets=N` and a
//! `N pin(s) join nothing` tally. A dogfood run worked the relationship out
//! unaided (defect D4), which is the good outcome of a bad situation: nothing
//! said what either number counted, and nothing held them to it. A documented
//! relationship that nothing tests is a comment.

use kicli::connectivity::{Nets, extract};
use kicli::model::Hierarchy;
use kicli::view::connectivity;
use kicli::view::connectivity::ViewOptions;
use std::path::{Path, PathBuf};

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

/// Every schematic under `tests/fixtures`, sorted so a failure names the same
/// file on every machine.
fn every_schematic() -> Vec<PathBuf> {
    let mut found = Vec::new();
    let mut pending = vec![fixtures()];
    while let Some(directory) = pending.pop() {
        let Ok(entries) = std::fs::read_dir(&directory) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|kind| kind == "kicad_sch") {
                found.push(path);
            }
        }
    }
    found.sort();
    found
}

/// What `project info` prints: every net of the whole project.
fn project_info_count(nets: &Nets) -> usize {
    nets.nets().len()
}

/// What the connectivity view prints, read back out of the view's own text
/// rather than recomputed. Recomputing would compare kicli against a second
/// implementation of kicli and agree with itself.
fn view_counts(view: &str) -> (usize, usize) {
    let listed = view.lines().filter(|line| line.starts_with("N ")).count();
    let tallied = view
        .lines()
        .find(|line| line.contains("pin(s) join nothing"))
        .and_then(|line| line.trim_start_matches("# ").split_whitespace().next())
        .map_or(0, |count| count.parse().expect("the tally is a number"));
    (listed, tallied)
}

/// A net the view shows no pin of, and so does not list or tally: every pin is
/// a power pin and `--include-power` is off, or every pin is on another sheet.
fn hidden(nets: &Nets, options: &ViewOptions) -> usize {
    nets.nets()
        .iter()
        .filter(|net| {
            !net.pins
                .iter()
                .any(|pin| options.include_power || !pin.power)
        })
        .count()
}

#[test]
fn the_whole_project_view_accounts_for_every_net() {
    let mut projects = 0;
    for path in every_schematic() {
        let Ok(hierarchy) = Hierarchy::load(&path) else {
            continue;
        };
        let nets = extract(&hierarchy);
        let total = project_info_count(&nets);
        if total == 0 {
            continue;
        }
        let options = ViewOptions::default();
        let (listed, tallied) = view_counts(&connectivity::render(&hierarchy, &nets, &options));
        assert_eq!(
            listed + tallied + hidden(&nets, &options),
            total,
            "the connectivity view of {} lists {listed}, tallies {tallied} and \
             hides {}, which does not account for the {total} nets \
             `project info` reports",
            path.display(),
            hidden(&nets, &options)
        );
        projects += 1;
    }
    assert!(
        projects >= 5,
        "only {projects} fixture projects have nets, which is too few to have \
         checked anything"
    );
}

#[test]
fn a_per_sheet_view_does_not_account_for_the_project() {
    // The document says a per-sheet view is not meant to add up to the project
    // figure. If it always did, that sentence would be wrong and this test
    // would be the thing that noticed.
    let hierarchy = Hierarchy::load(&fixtures().join("sch/nets/nets.kicad_sch"))
        .expect("the nets fixture loads");
    let nets = extract(&hierarchy);
    let root = hierarchy
        .placements
        .first()
        .expect("the fixture has a root placement")
        .path
        .clone();
    let options = ViewOptions {
        sheet: Some(root),
        ..ViewOptions::default()
    };
    let (listed, tallied) = view_counts(&connectivity::render(&hierarchy, &nets, &options));
    assert!(
        listed + tallied < project_info_count(&nets),
        "the root sheet accounts for {} of {} nets, so the per-sheet and \
         project figures no longer differ and the document is wrong",
        listed + tallied,
        project_info_count(&nets)
    );
}
