//! A rule is a new file, and adding one edits nothing.
//!
//! This is the seam the whole lint engine stands on. It has two arms, and each
//! is blind without the other.
//!
//! The **directory arm** reads the rule directory at run time and compares it
//! with the list the build generated. A hand-written list passes a count of
//! files and fails this, because the first file added to the directory would
//! not be in it.
//!
//! The **engine arm** runs the registered rules over a real drawing and reads
//! what comes back. A registration that compiles and never runs passes the
//! directory arm and fails this one.
//!
//! The instrument is the specimen directory, `tests/specimen_rules/`, which
//! holds rules built for this measurement. The crate's own rule directory is
//! checked by the same code and is empty until the first rule is written, so at
//! that point it proves the mechanism tolerates an empty directory and nothing
//! more.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use kicli::lint::{Drawing, Engine, Finding, RuleId};
use kicli::model::{Hierarchy, LoadedFile};
use kicli_probe::{Probe, pin, rectangle, symbol};

/// The registry the build wrote from `tests/specimen_rules/`.
mod specimens {
    include!(concat!(env!("OUT_DIR"), "/specimen_rules.rs"));
}

/// Where the crate's own rules live, and where the specimens live.
fn rule_directory(under: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(under)
}

/// The module names of the rule files in a directory, read now rather than at
/// build time.
///
/// The whole point of the check is that this list and the generated one are
/// derived separately. Nothing here may call the generator.
fn files_on_disk(dir: &Path) -> Vec<String> {
    if !dir.is_dir() {
        return Vec::new();
    }
    let mut found: Vec<String> = std::fs::read_dir(dir)
        .expect("the rule directory is readable")
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|end| end == "rs"))
        .map(|path| {
            path.file_stem()
                .expect("a rule file has a name")
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    found.sort();
    found
}

#[test]
fn the_registry_holds_exactly_the_files_the_directory_holds() {
    let dir = rule_directory("tests/specimen_rules");
    let on_disk = files_on_disk(&dir);

    // The control. A directory that read as empty would agree with an empty
    // registry, and the check would be watching nothing.
    assert!(
        on_disk.len() >= 3,
        "the specimen directory holds rule files: {on_disk:?}"
    );

    assert_eq!(
        specimens::files(),
        on_disk,
        "the generated registry names the files the directory holds"
    );
}

#[test]
fn every_registered_file_declares_at_least_one_rule() {
    for (file, rules) in specimens::BY_FILE {
        assert!(!rules.is_empty(), "{file} declares a rule");
    }
    assert!(
        specimens::all().len() > specimens::BY_FILE.len(),
        "some file declares more than one rule, so a family shares a file"
    );
}

#[test]
fn every_registered_rule_runs_and_its_findings_reach_the_output() {
    let path = specimen_drawing("registered");
    let hierarchy = loaded(&path);
    let drawings = drawings(&hierarchy);
    assert_eq!(drawings.len(), 2, "the specimen drawing has two placements");

    let engine = Engine::of(specimens::all());
    let findings = engine.examine_all(&drawings);

    // The control. An engine that answered nothing would make every set
    // comparison below trivially true.
    assert!(!findings.is_empty(), "the engine found something");

    let reported: BTreeSet<RuleId> = findings.iter().map(|finding| finding.rule).collect();
    let registered: BTreeSet<RuleId> = engine.codes().into_iter().collect();
    assert_eq!(
        reported, registered,
        "every registered rule reported, and nothing else did"
    );
}

#[test]
fn a_rule_code_is_unique_and_published_in_shape() {
    let mut seen = BTreeSet::new();
    for rule in specimens::all().iter().chain(&kicli::lint::registry::all()) {
        let id = rule.id();
        assert!(
            id.is_well_formed(),
            "{id} has the shape of a published code"
        );
        assert!(seen.insert(id), "{id} is registered twice");
    }
}

#[test]
fn the_crate_rules_are_listed_with_everything_they_report() {
    // The seam check's window. Adding one file under `src/lint/rules/` must
    // make a rule appear here and make its findings appear below it, without
    // any other file being touched. Run it with `--nocapture` and read it.
    //
    // Nothing about the content is asserted, because what a real rule reports
    // is that rule's business. What is asserted is that the engine runs
    // exactly what the registry holds.
    let path = specimen_drawing("crate-rules");
    let hierarchy = loaded(&path);
    let sheets = drawings(&hierarchy);
    let engine = Engine::of_every_rule();

    println!("registered files: {:?}", kicli::lint::registry::files());
    println!("registered rules: {:?}", engine.codes());
    for finding in engine.examine_all(&sheets) {
        println!("finding: {}", line(&finding));
    }

    assert_eq!(engine.codes().len(), kicli::lint::registry::all().len());
}

#[test]
fn the_crate_registry_holds_exactly_the_files_its_directory_holds() {
    // The same comparison over the crate's own rules. It is vacuous while the
    // directory is empty, and it is what starts measuring the moment a rule is
    // written. The specimen arms above are the non-vacuous instrument.
    let dir = rule_directory("src/lint/rules");
    assert_eq!(kicli::lint::registry::files(), files_on_disk(&dir));

    let engine = Engine::of_every_rule();
    assert_eq!(engine.len(), kicli::lint::registry::all().len());
}

/// Where the drawings this binary builds are written.
fn scratch() -> PathBuf {
    Path::new(env!("CARGO_TARGET_TMPDIR")).join("lint-register")
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

/// One finding, on one line, for the seam check to read.
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
