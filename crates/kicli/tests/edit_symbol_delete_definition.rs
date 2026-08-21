//! `sym delete` says which way the two-way fork went.
//!
//! Deleting a placement leaves the sheet's embedded definition alone when
//! another placement still draws through it, and takes it when none does.
//! `AGENT.md` raises that fork in the reader's mind, so the report must answer
//! it — a silence there is the defect this file was written for (dogfood D5).
//!
//! Both arms are checked because a report that always says "kept" passes a
//! one-armed test. Each arm checks the file as well as the sentence, so the
//! note is a report of what happened rather than a constant.
//!
//! Everything here goes through the compiled binary, in both output forms,
//! because the sentence and the JSON key are what an agent actually meets.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use kicli_probe::scratch::Fixtures;
use serde_json::Value;

/// The committed fixtures this binary reads, and the scratch it writes in.
fn fixtures() -> Fixtures {
    Fixtures::new(env!("CARGO_TARGET_TMPDIR"), env!("CARGO_MANIFEST_DIR"))
}

/// A path no `kicad-cli` is at.
///
/// `sym delete` never needs KiCad. Pointing it at nothing keeps a machine with
/// KiCad installed giving the same answer as one without.
const NO_KICAD_CLI: &str = "/nonexistent/kicad-cli";

/// The two placements of `Test:GND` in `sch/nets`, in the order they go.
///
/// The first delete leaves one placement drawing the definition, so it stays.
/// The second leaves none, so it goes. One fixture, both arms.
const GROUNDS: [&str; 2] = ["#PWR01", "#PWR02"];

/// The library identifier both placements draw through.
const DEFINITION: &str = "Test:GND";

/// A scratch copy of the `sch/nets` project, and the commands run against it.
struct Project {
    directory: PathBuf,
}

impl Project {
    /// Copy the committed fixture into a scratch directory of its own.
    fn new(name: &str) -> Self {
        Self {
            directory: fixtures().scratch_directory(name, "sch/nets"),
        }
    }

    /// Run one `sym delete`, and fail loudly if the command refused.
    fn delete(&self, target: &str, arguments: &[&str]) -> Output {
        let run = Command::new(env!("CARGO_BIN_EXE_kicli"))
            .args(["sym", "delete", target])
            .args(arguments)
            .args(["--project", self.directory.to_str().expect("a text path")])
            .env("KICLI_KICAD_CLI", NO_KICAD_CLI)
            .output()
            .expect("the binary runs");
        assert_eq!(
            run.status.code(),
            Some(0),
            "deleting {target}: {}",
            text(&run.stderr)
        );
        run
    }

    /// Whether the sheet still embeds the definition the placements drew.
    fn embeds_definition(&self) -> bool {
        let source = std::fs::read_to_string(self.sheet()).expect("the sheet reads");
        source.contains(&format!("(symbol \"{DEFINITION}\""))
    }

    /// The sheet the placements sit on.
    fn sheet(&self) -> PathBuf {
        self.directory.join("nets.kicad_sch")
    }
}

/// One stream of a run, as text.
fn text(bytes: &[u8]) -> String {
    String::from_utf8(bytes.to_vec()).expect("the output is text")
}

/// The names of every note a JSON report carries.
fn note_names(report: &Value) -> Vec<String> {
    report["notes"]
        .as_array()
        .expect("a report carries a notes array")
        .iter()
        .map(|note| {
            note["name"]
                .as_str()
                .expect("a note carries a name")
                .to_owned()
        })
        .collect()
}

/// The message of the note carrying a name.
fn note_message(report: &Value, name: &str) -> String {
    report["notes"]
        .as_array()
        .expect("a report carries a notes array")
        .iter()
        .find(|note| note["name"].as_str() == Some(name))
        .and_then(|note| note["message"].as_str())
        .unwrap_or_else(|| panic!("the report carries a {name} note"))
        .to_owned()
}

#[test]
fn the_text_report_says_which_way_the_definition_went() {
    let project = Project::new("sym_delete_definition_text");

    // Arm one: another placement still draws the definition, so it stays.
    let kept = text(&project.delete(GROUNDS[0], &[]).stdout);
    assert!(
        project.embeds_definition(),
        "one placement is left, so the sheet still embeds {DEFINITION}"
    );
    assert!(
        kept.contains("note: definition-kept"),
        "the report names the fork it took: {kept}"
    );
    assert!(
        !kept.contains("definition-removed"),
        "the report takes one fork, not both: {kept}"
    );

    // Arm two: no placement is left, so the definition goes with the symbol.
    let gone = text(&project.delete(GROUNDS[1], &[]).stdout);
    assert!(
        !project.embeds_definition(),
        "no placement is left, so the sheet embeds {DEFINITION} no more"
    );
    assert!(
        gone.contains("note: definition-removed"),
        "the report names the fork it took: {gone}"
    );
    assert!(
        !gone.contains("definition-kept"),
        "the report takes one fork, not both: {gone}"
    );
}

#[test]
fn the_json_report_says_which_way_the_definition_went() {
    let project = Project::new("sym_delete_definition_json");
    let json = ["--output", "json"];

    // Arm one: another placement still draws the definition, so it stays.
    let kept: Value = serde_json::from_str(&text(&project.delete(GROUNDS[0], &json).stdout))
        .expect("the result is one JSON object");
    assert!(
        project.embeds_definition(),
        "one placement is left, so the sheet still embeds {DEFINITION}"
    );
    assert_eq!(
        note_names(&kept),
        vec!["definition-kept".to_owned()],
        "the machine form carries the fork too, and takes one of the two"
    );
    assert!(
        note_message(&kept, "definition-kept").contains(DEFINITION),
        "the note names the definition it is talking about"
    );

    // Arm two: no placement is left, so the definition goes with the symbol.
    let gone: Value = serde_json::from_str(&text(&project.delete(GROUNDS[1], &json).stdout))
        .expect("the result is one JSON object");
    assert!(
        !project.embeds_definition(),
        "no placement is left, so the sheet embeds {DEFINITION} no more"
    );
    assert_eq!(
        note_names(&gone),
        vec!["definition-removed".to_owned()],
        "the machine form carries the fork too, and takes one of the two"
    );
    assert!(
        note_message(&gone, "definition-removed").contains(DEFINITION),
        "the note names the definition it is talking about"
    );
}

/// The fixture this file rests on: two placements of one definition.
///
/// The arms above are only two arms while that holds. A fixture edited down to
/// one ground would turn arm one into a second copy of arm two, and both would
/// still pass, so the shape is checked rather than assumed.
#[test]
fn the_fixture_holds_exactly_two_placements_of_the_definition() {
    let sheet =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sch/nets/nets.kicad_sch");
    let source = std::fs::read_to_string(sheet).expect("the fixture reads");
    let placements = source
        .matches(&format!("(lib_id \"{DEFINITION}\")"))
        .count();
    assert_eq!(
        placements, 2,
        "the two arms need one definition drawn by exactly two placements"
    );
}
