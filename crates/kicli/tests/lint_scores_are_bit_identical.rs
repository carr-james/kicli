//! Scoring an unchanged drawing twice gives the same number.
//!
//! The sort of the findings is one claim and the number is another. A score is
//! a decaying function of a sum, and a decay is where an author reaches for
//! floating point, so a score is exactly the kind of value that can differ in
//! its last place between two machines and round two ways.
//!
//! Two scores from one process share every ancestor, so agreement between them
//! is weak evidence. The load-bearing arm starts a **second process**. Each
//! child parses its own copy of the drawing, from its own directory, under its
//! own working directory, time zone and locale, and prints the report. What is
//! compared is the two children's bytes.
//!
//! Two things could make that comparison true for the wrong reason. Every sheet
//! could score the best there is, which is the same number for any drawing at
//! all; or the report could ignore the drawing. Both are refused: a sheet must
//! score below the best, and the same binary run over a **different** drawing
//! must print something different.

use std::path::{Path, PathBuf};
use std::process::Command;

use kicli::lint::score::{Density, Normaliser, SheetScore, project_score};
use kicli::lint::{Drawing, Engine, Finding, RuleId};
use kicli::model::{Hierarchy, LoadedFile};
use kicli_probe::{Probe, pin, rectangle, symbol};

/// The registry the build wrote from the specimen rule directory.
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
const DRAWING: &str = "KICLI_SCORE_DRAWING";

/// The name of the test a child process runs.
const CHILD_TEST: &str = "the_report_a_child_process_prints";

/// What brackets the report in a child's output.
const BEGIN: &str = "<<<BEGIN REPORT>>>";
const END: &str = "<<<END REPORT>>>";

/// A crossing rule's code, which divides by the wire count.
const CROSSING: RuleId = RuleId("KI-XING-001");

/// A field rule's code, which divides by the symbol count.
const FIELD: RuleId = RuleId("KI-FLD-001");

/// Every finding of the specimen rules, under three codes rather than one.
///
/// The specimen rules are one family, so their findings would all take one
/// normaliser and two thirds of the arithmetic would go unmeasured. Each
/// finding is therefore copied under a crossing code and a field code as well
/// as its own. The tier is copied with it, so a blocking finding stays
/// blocking whatever code it wears.
fn under_every_normaliser(found: &[Finding]) -> Vec<Finding> {
    let mut all = Vec::new();
    for finding in found {
        for code in [finding.rule, CROSSING, FIELD] {
            let mut copy = finding.clone();
            copy.rule = code;
            all.push(copy);
        }
    }
    all
}

/// The score report for one loaded drawing, as text.
fn report_for(path: &Path) -> String {
    let hierarchy = Hierarchy::load(path).expect("the drawing loads");
    let mut out = String::new();
    let mut sheets = Vec::new();
    for placement in &hierarchy.placements {
        let file: &LoadedFile = &hierarchy.files[placement.file];
        let drawing = Drawing::read(&file.doc, &file.schematic, &placement.path);
        let density = Density::of(&drawing);
        let found = under_every_normaliser(&Engine::of(specimens::all()).examine(&drawing));
        let scored = SheetScore::of(&found, density);
        out.push_str(&line(&placement.path.0, scored));
        sheets.push(scored);
    }
    out.push_str(&format!("project score={}\n", project_score(&sheets)));
    out
}

/// One sheet's line of the report, with every number that went into it.
fn line(path: &str, scored: SheetScore) -> String {
    format!(
        "sheet {path} symbols={} wires={} raw={} score={}\n",
        scored.density().symbols(),
        scored.density().wires(),
        scored.raw().text(),
        scored.score(),
    )
}

/// Run this test binary again, in a child process, over one drawing.
///
/// The child gets its own working directory, time zone and locale, because a
/// score that read any of them would be reproducible on one machine and
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
fn two_processes_score_the_same_bytes() {
    // Two copies of one drawing, in two directories with different names. The
    // probe writes the same bytes wherever it is told to write, and that is
    // asserted rather than assumed, because two different files would make the
    // comparison meaningless.
    let first = specimen_drawing("scored-first", 6);
    let second = specimen_drawing("scored-second", 6);
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

    // The control against an empty agreement. Every sheet scoring the best
    // there is would agree perfectly and measure nothing.
    assert!(!left.is_empty(), "the report holds sheets");
    assert!(
        left.lines().any(is_scored_below_the_best),
        "a sheet lost points, so the report is about this drawing:\n{left}"
    );

    assert_eq!(left, right, "two processes scored the same bytes");
}

#[test]
fn a_different_drawing_scores_different_bytes() {
    // The other control. A report that ignored the drawing would agree with
    // itself perfectly, which is exactly what the check above asserts.
    let sparse = specimen_drawing("scored-sparse", 3);
    let crowded = specimen_drawing("scored-crowded", 40);

    let small = child_report(&sparse, &scratch().join("contrast-sparse"), "UTC", "C");
    let large = child_report(&crowded, &scratch().join("contrast-crowded"), "UTC", "C");

    assert!(!small.is_empty(), "the sparse drawing reports something");
    assert!(!large.is_empty(), "the crowded drawing reports something");
    assert_ne!(small, large, "a different drawing scores differently");
}

#[test]
fn one_process_scores_the_same_bytes_every_time() {
    // The cheap arm. It catches a collection whose iteration order is not
    // fixed, which two runs in one process can differ on.
    let path = specimen_drawing("scored-repeated", 6);
    let first = report_for(&path);
    assert!(first.lines().any(is_scored_below_the_best));
    for run in 1..RUNS {
        assert_eq!(report_for(&path), first, "run {run} scored differently");
    }
}

#[test]
fn every_normaliser_takes_part_in_the_report() {
    // Without this the report could measure one third of the arithmetic and
    // still be perfectly reproducible.
    let codes: Vec<RuleId> = Engine::of(specimens::all()).codes();
    assert!(!codes.is_empty(), "there are specimen rules to score");
    assert_eq!(Normaliser::of(codes[0]), Normaliser::PerSheet);
    assert_eq!(Normaliser::of(CROSSING), Normaliser::PerWire);
    assert_eq!(Normaliser::of(FIELD), Normaliser::PerObject);
}

/// Does this line of the report show a sheet that lost points?
fn is_scored_below_the_best(line: &str) -> bool {
    line.starts_with("sheet ") && !line.ends_with("score=100")
}

/// Where the drawings this binary builds are written.
fn scratch() -> PathBuf {
    Path::new(env!("CARGO_TARGET_TMPDIR")).join("lint-score-determinism")
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

/// A sheet of a given number of symbols, each with a wire and a junction.
fn specimen_drawing(name: &str, symbols: u32) -> PathBuf {
    let mut probe = Probe::new(name, scratch());
    probe.define(pair());
    for index in 0..symbols {
        let column = index % 10;
        let row = index / 10;
        let x = 20 + column * 9;
        let y = 20 + row * 9;
        probe.place(
            "PAIR",
            &format!("U{}", index + 1),
            (&format!("{x}"), &format!("{y}")),
            &PINS,
        );
        probe.wire(
            (&format!("{}", x + 4), &format!("{y}")),
            (&format!("{}", x + 8), &format!("{y}")),
        );
        probe.junction((&format!("{}", x + 8), &format!("{y}")));
    }
    probe.write()
}
