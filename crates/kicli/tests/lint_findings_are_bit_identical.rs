//! Examining an unchanged drawing twice gives the same bytes.
//!
//! Two runs in one process share every ancestor: one load of one file, one heap
//! layout, one process-wide hash seed. Agreement between them is weak evidence,
//! so the load-bearing arm here starts a **second process**. Each child parses
//! its own copy of the drawing, from its own directory, under its own working
//! directory, time zone and locale, and prints the report. What is compared is
//! the two children's bytes.
//!
//! Two things could make that comparison true for the wrong reason. The report
//! could be empty, or the report could ignore the drawing. Both are refused:
//! the report must hold findings from every specimen rule, and the same binary
//! run over a **different** drawing must print something different.

use std::path::{Path, PathBuf};
use std::process::Command;

use kicli::lint::{Drawing, Engine, Finding};
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

/// How many times one report is asked for inside one process.
const RUNS: usize = 50;

/// The drawing a child process is asked to report on.
const DRAWING: &str = "KICLI_LINT_DRAWING";

/// The name of the test a child process runs.
const CHILD_TEST: &str = "the_report_a_child_process_prints";

/// What brackets the report in a child's output.
const BEGIN: &str = "<<<BEGIN REPORT>>>";
const END: &str = "<<<END REPORT>>>";

/// The report for one loaded drawing, as text.
fn report_for(path: &Path) -> String {
    let hierarchy = loaded(path);
    let drawings = drawings(&hierarchy);
    report(&Engine::of(specimens::all()).examine_all(&drawings))
}

/// Run this test binary again, in a child process, over one drawing.
///
/// The child gets its own working directory, time zone and locale, because a
/// report that read any of them would be reproducible on one machine and
/// nowhere else.
fn child_report(drawing: &Path, working_directory: &Path, zone: &str, locale: &str) -> String {
    std::fs::create_dir_all(working_directory).expect("the working directory is writable");
    let binary = std::env::current_exe().expect("this test binary has a path");
    let output = Command::new(binary)
        .args(["--exact", "--ignored", "--nocapture", CHILD_TEST])
        .env(DRAWING, drawing)
        .env("TZ", zone)
        .env("LC_ALL", locale)
        .current_dir(working_directory)
        .output()
        .expect("the child process runs");
    let text = String::from_utf8(output.stdout).expect("the child prints text");
    assert!(
        output.status.success(),
        "the child process succeeded: {text}"
    );
    let start = text.find(BEGIN).expect("the child printed a report") + BEGIN.len();
    let end = text.find(END).expect("the child closed its report");
    text[start..end].to_owned()
}

#[test]
#[ignore = "run by the process-boundary arm, in a child process"]
fn the_report_a_child_process_prints() {
    let drawing = std::env::var(DRAWING).expect("a child process is told which drawing to read");
    println!("{BEGIN}");
    print!("{}", report_for(Path::new(&drawing)));
    println!("{END}");
}

#[test]
fn two_processes_report_the_same_bytes() {
    // Two copies of one drawing, in two directories with different names. The
    // probe writes the same bytes wherever it is told to write, and that is
    // asserted rather than assumed, because two different files would make the
    // comparison meaningless.
    let first = specimen_drawing("identical-first");
    let second = specimen_drawing("identical-second");
    assert_ne!(first, second, "the two copies are at different paths");
    assert_eq!(
        std::fs::read(&first).expect("the first copy reads"),
        std::fs::read(&second).expect("the second copy reads"),
        "the two copies hold the same bytes"
    );

    let left = child_report(&first, &scratch().join("run-left"), "UTC", "C");
    let right = child_report(
        &second,
        &scratch().join("run-right"),
        "Pacific/Auckland",
        "en_GB.UTF-8",
    );

    // The control against an empty agreement. A report of nothing is identical
    // to a report of nothing.
    let codes = Engine::of(specimens::all()).codes();
    assert!(!codes.is_empty(), "there are specimen rules to report");
    for code in codes {
        assert!(
            left.contains(code.0),
            "the report holds a finding of {code}:\n{left}"
        );
    }

    assert_eq!(left, right, "two processes reported the same bytes");
}

#[test]
fn a_different_drawing_reports_different_bytes() {
    // The other control. A report that ignored the drawing would agree with
    // itself perfectly, which is exactly what the check above asserts.
    let root = specimen_drawing("contrast");
    let only_the_child = root
        .parent()
        .expect("the drawing sits in a directory")
        .join("child.kicad_sch");
    assert!(only_the_child.is_file(), "the child sheet was written");

    let whole = child_report(&root, &scratch().join("contrast-whole"), "UTC", "C");
    let part = child_report(
        &only_the_child,
        &scratch().join("contrast-part"),
        "UTC",
        "C",
    );

    assert!(!whole.is_empty(), "the whole drawing reports something");
    assert!(!part.is_empty(), "the child sheet reports something");
    assert_ne!(whole, part, "a different drawing reports differently");
}

#[test]
fn one_process_reports_the_same_bytes_every_time() {
    // The cheap arm. It catches a collection whose iteration order is not
    // fixed, which two runs in one process can differ on.
    let path = specimen_drawing("repeated");
    let first = report_for(&path);
    assert!(!first.is_empty(), "the report holds findings");
    for run in 1..RUNS {
        assert_eq!(report_for(&path), first, "run {run} reported differently");
    }
}

/// Where the drawings this binary builds are written.
fn scratch() -> PathBuf {
    Path::new(env!("CARGO_TARGET_TMPDIR")).join("lint-determinism")
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
