//! The controls a conclusion needs, as code rather than as a convention.
//!
//! Two shapes recur. A causal claim — "this item is what joins them" — is
//! shown by removing the item and measuring again. A before-and-after claim —
//! "nothing else moved" — is only worth making when the before-reading carried
//! something to move.

use kicli_probe::oracle::{Change, with_and_without};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// Where these tests write. `CARGO_TARGET_TMPDIR` is under `target/`.
fn scratch() -> PathBuf {
    Path::new(env!("CARGO_TARGET_TMPDIR")).join("controls")
}

/// A reading, as a set of lines.
fn reading(lines: &[&str]) -> BTreeSet<String> {
    lines.iter().map(|line| (*line).to_owned()).collect()
}

#[test]
fn removing_the_cause_removes_the_merge() {
    // Two wires that cross, with a resistor pin on each of the four ends. A
    // junction at the crossing joins them; without it they are two nets, which
    // is what makes the junction the cause rather than the drawing.
    let (with, without) = with_and_without("crossing-junction", &scratch(), |probe, junction| {
        probe.wire(("50.8", "50.8"), ("101.6", "50.8"));
        probe.wire(("76.2", "25.4"), ("76.2", "76.2"));
        probe.place("R", "R1", ("50.8", "54.61"), &["1", "2"]);
        probe.place("R", "R2", ("101.6", "54.61"), &["1", "2"]);
        probe.place("R", "R3", ("76.2", "29.21"), &["1", "2"]);
        probe.place("R", "R4", ("76.2", "80.01"), &["1", "2"]);
        if junction {
            probe.junction(("76.2", "50.8"));
        }
    });

    assert!(
        with.contains(&kicli_probe::net(&["R1.1", "R2.1", "R3.1", "R4.1"])),
        "the junction joins the two wires: {with:?}"
    );
    assert!(
        without.contains(&kicli_probe::net(&["R1.1", "R2.1"]))
            && without.contains(&kicli_probe::net(&["R3.1", "R4.1"])),
        "and without it they are two nets: {without:?}"
    );
}

#[test]
#[should_panic(expected = "the before-reading is empty")]
fn a_comparison_with_an_empty_control_is_refused() {
    // "Nothing else moved" is true of a report that was never read, so the
    // comparison refuses the reading rather than the conclusion.
    let _ = Change::measured(BTreeSet::new(), reading(&["R1 pin 1"]));
}

#[test]
fn a_comparison_reads_what_moved() {
    // The control for the refusal above, and the shape a mutation oracle uses:
    // the subject's own entries are the point, and everything else must read
    // as it did.
    let change = Change::measured(
        reading(&["R1 pin 1", "R2 pin 1"]),
        reading(&["R1 pin 1", "R2 pin 1", "R900 pin 1"]),
    );
    assert_eq!(change.added(), reading(&["R900 pin 1"]));
    assert!(change.removed().is_empty());
    change.nothing_moved_but("R900");
}

#[test]
#[should_panic(expected = "does not name R900")]
fn a_comparison_reports_what_moved_and_should_not_have() {
    let change = Change::measured(
        reading(&["R1 pin 1", "R2 pin 1"]),
        reading(&["R1 pin 1", "R900 pin 1"]),
    );
    change.nothing_moved_but("R900");
}
