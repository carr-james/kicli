//! Findings leave the engine in one order, and the order is the published one.
//!
//! The published order is `(rule, sheet, x, y, uuid)`. This check writes that
//! order out again, from the contract rather than from the engine, and compares.
//! A check that asked the engine for its own key would agree with any key the
//! engine happened to hold.
//!
//! An order is only measured by findings that contend. One finding sorts
//! correctly under every rule anybody could write, so the census below requires
//! each of the five terms to be the deciding term for some neighbouring pair,
//! and requires one pair to be decided by a *second* named object.

use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use kicli::lint::finding::sort;
use kicli::lint::{Drawing, Engine, Finding, Findings};
use kicli::model::{Hierarchy, LoadedFile};
use kicli_probe::{Probe, pin, rectangle, symbol};

/// The registry the build wrote from `tests/specimen_rules/`.
///
/// Each test binary uses a different part of the generated registry, so the
/// unused part is allowed rather than removed.
mod specimens {
    #![allow(dead_code)]

    include!(concat!(env!("OUT_DIR"), "/specimen_rules.rs"));
}

/// The published order, written out here rather than read from the engine.
///
/// Five terms: the rule's code, the sheet path, the x of the position, the y of
/// the position, and the objects the finding names. The fifth term is the whole
/// list, because a finding may name more than one object and two findings that
/// share their first object still need an order.
fn published_order(left: &Finding, right: &Finding) -> Ordering {
    left.rule
        .0
        .cmp(right.rule.0)
        .then_with(|| left.sheet.0.cmp(&right.sheet.0))
        .then_with(|| left.pos.x.0.cmp(&right.pos.x.0))
        .then_with(|| left.pos.y.0.cmp(&right.pos.y.0))
        .then_with(|| left.objects.cmp(&right.objects))
}

/// Which of the five terms separates two findings, counting from one.
///
/// Returns `None` when all five agree, which means the two are one finding.
fn deciding_term(left: &Finding, right: &Finding) -> Option<usize> {
    if left.rule != right.rule {
        return Some(1);
    }
    if left.sheet != right.sheet {
        return Some(2);
    }
    if left.pos.x != right.pos.x {
        return Some(3);
    }
    if left.pos.y != right.pos.y {
        return Some(4);
    }
    if left.objects != right.objects {
        return Some(5);
    }
    None
}

/// What every specimen rule reports, in the order the rules report it.
///
/// This is the engine's input rather than its output. It is built here, through
/// the same public interface a rule is called by, so the sorted answer has
/// something unsorted to be compared against.
fn as_reported(drawings: &[Drawing<'_>]) -> Vec<Finding> {
    let mut found = Vec::new();
    for drawing in drawings {
        for rule in specimens::all() {
            let mut collected = Findings::of(rule, drawing.path());
            rule.examine(drawing, &mut collected);
            found.extend(collected.into_vec());
        }
    }
    found
}

#[test]
fn the_engine_writes_findings_in_the_published_order() {
    let path = specimen_drawing("sorted");
    let hierarchy = loaded(&path);
    let drawings = drawings(&hierarchy);
    let findings = Engine::of(specimens::all()).examine_all(&drawings);

    assert!(
        findings.len() > 20,
        "the drawing gives the order something to do: {}",
        findings.len()
    );

    for pair in findings.windows(2) {
        assert_eq!(
            published_order(&pair[0], &pair[1]),
            Ordering::Less,
            "{}\ncomes before\n{}",
            line(&pair[0]),
            line(&pair[1])
        );
    }
}

#[test]
fn every_term_of_the_key_decides_some_pair() {
    let path = specimen_drawing("contending");
    let hierarchy = loaded(&path);
    let drawings = drawings(&hierarchy);
    let findings = Engine::of(specimens::all()).examine_all(&drawings);

    let mut decided = [0_usize; 5];
    let mut by_a_later_object = 0_usize;
    for pair in findings.windows(2) {
        let term = deciding_term(&pair[0], &pair[1]).expect("neighbours are not one finding");
        decided[term - 1] += 1;
        if term == 5 {
            let shared = pair[0]
                .objects
                .iter()
                .zip(&pair[1].objects)
                .take_while(|(left, right)| left == right)
                .count();
            if shared >= 1 {
                by_a_later_object += 1;
            }
        }
    }

    for (index, count) in decided.iter().enumerate() {
        assert!(
            *count > 0,
            "term {} of the key decides some neighbouring pair: {decided:?}",
            index + 1
        );
    }
    assert!(
        by_a_later_object > 0,
        "some pair is separated by an object after the first: {decided:?}"
    );
}

#[test]
fn no_two_findings_share_the_published_key() {
    let path = specimen_drawing("distinct");
    let hierarchy = loaded(&path);
    let drawings = drawings(&hierarchy);
    let findings = Engine::of(specimens::all()).examine_all(&drawings);

    let mut seen = BTreeSet::new();
    for finding in &findings {
        assert!(
            seen.insert(finding.key()),
            "two findings share a key, so they are one finding: {}",
            line(finding)
        );
    }
}

#[test]
fn the_sort_is_what_puts_them_in_order() {
    let path = specimen_drawing("unsorted");
    let hierarchy = loaded(&path);
    let drawings = drawings(&hierarchy);

    let reported = as_reported(&drawings);
    let sorted = Engine::of(specimens::all()).examine_all(&drawings);

    // The control against a sort that does nothing: the rules report in an
    // order that is not the published one, so agreement afterwards is the
    // sort's work rather than an accident of how the rules were called.
    assert_ne!(
        report(&reported),
        report(&sorted),
        "the reported order is not already the published order"
    );

    let mut put_in_order = reported;
    sort(&mut put_in_order);
    assert_eq!(report(&put_in_order), report(&sorted));
}

/// Where the drawings this binary builds are written.
fn scratch() -> PathBuf {
    Path::new(env!("CARGO_TARGET_TMPDIR")).join("lint-sort")
}

/// The pin numbers the specimen symbol draws.
const PINS: [&str; 2] = ["1", "2"];

/// A symbol with a square body and a pin on two of its edges.
fn pair() -> String {
    symbol(
        "PAIR",
        "U",
        false,
        &[(
            "1_1",
            vec![
                rectangle(("-2.54", "-2.54"), ("2.54", "2.54")),
                pin("passive", ("-3.81", "0"), "0", "1", "W"),
                pin("passive", ("3.81", "0"), "180", "2", "E"),
            ],
        )],
    )
}

/// A hierarchy of two sheets, each holding symbols, wires and a junction.
///
/// Both sheets carry the same kinds of object at different places, so the sort
/// key has something to separate on in every one of its terms: two rules over
/// one object, one rule over objects at two positions, and one rule naming
/// different objects at one position.
fn specimen_drawing(name: &str) -> PathBuf {
    let mut root = Probe::new(name, scratch());
    let mut child = Probe::child_of(&root);

    root.define(pair());
    root.place("PAIR", "U1", ("76.2", "88.9"), &PINS);
    root.place("PAIR", "U2", ("101.6", "88.9"), &PINS);
    // Directly below U1: same x, different y, so the fourth term of the sort
    // key has a pair to decide.
    root.place("PAIR", "U5", ("76.2", "114.3"), &PINS);
    root.wire(("80.01", "88.9"), ("88.9", "88.9"));
    root.wire(("88.9", "88.9"), ("97.79", "88.9"));
    root.wire(("88.9", "88.9"), ("88.9", "101.6"));
    root.wire(("88.9", "101.6"), ("101.6", "101.6"));
    root.junction(("88.9", "88.9"));
    root.sheet_of_size(
        "00000000-0000-4000-8000-cccccccccccc",
        "child",
        ("127", "63.5"),
        ("25.4", "25.4"),
        &[],
    );

    child.define(pair());
    child.place("PAIR", "U3", ("63.5", "63.5"), &PINS);
    child.place("PAIR", "U4", ("88.9", "63.5"), &PINS);
    child.wire(("67.31", "63.5"), ("76.2", "63.5"));
    child.wire(("76.2", "63.5"), ("85.09", "63.5"));
    child.junction(("76.2", "63.5"));

    root.write_all(&[&child])
}

/// Load a written drawing as the project rooted at it.
fn loaded(path: &Path) -> Hierarchy {
    Hierarchy::load(path).expect("the specimen drawing loads")
}

/// Every placement of a loaded hierarchy, as the rules see it.
fn drawings(hierarchy: &Hierarchy) -> Vec<Drawing<'_>> {
    hierarchy
        .placements
        .iter()
        .map(|placement| {
            let file: &LoadedFile = &hierarchy.files[placement.file];
            Drawing::read(&file.doc, &file.schematic, &placement.path)
        })
        .collect()
}

/// One finding, written so that two reports compare byte for byte.
///
/// Every field of the record is in the line, including the ones the sort key
/// does not read. A report that dropped a field would compare equal while the
/// findings differed.
fn line(finding: &Finding) -> String {
    let objects: Vec<&str> = finding
        .objects
        .iter()
        .map(|object| object.0.as_str())
        .collect();
    format!(
        "{} tier={} {} {} {},{} [{}] {:?} fix={:?} penalty={}",
        finding.rule,
        finding.tier.number(),
        finding.severity.word(),
        finding.sheet,
        finding.pos.x.0,
        finding.pos.y.0,
        objects.join(" "),
        finding.message,
        finding.fix,
        finding.penalty.text(),
    )
}

/// A whole report, one finding to a line.
fn report(findings: &[Finding]) -> String {
    let mut out = String::new();
    for finding in findings {
        out.push_str(&line(finding));
        out.push('\n');
    }
    out
}
