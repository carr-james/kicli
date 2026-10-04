//! Consuming KiCad's rule check: the gateway, the exit codes, and attribution.
//!
//! No test here starts KiCad. The process seam is a trait, so a fake runner
//! answers with the reports `kicad-cli` 10.0.5 actually wrote — the committed
//! fixtures under `tests/fixtures/erc/` — and the whole of kicli's reading runs
//! on a machine that has never seen a KiCad install.
//!
//! # The three claims
//!
//! 1. **`kicad` owns the invocation.** The rule check is asked for through
//!    [`kicli::kicad::erc::rule_check`], which is where discovery, the version
//!    check and the §6.2 exit-code translation already live. `lint` receives a
//!    value and never learns that a process ran.
//! 2. **Absence of `kicad-cli` is exit 6, and *present and broken* is not.**
//!    Those are different rows of §6.2's table and the same mistake in a
//!    careless test, so they are asserted apart.
//! 3. **kicli attributes KiCad's checks and does not double count them**
//!    (§11.1). The mechanism is one question — did KiCad run this check on this
//!    project — and this file measures both of its answers on one drawing.

use std::path::{Path, PathBuf};

use kicli::cli::ExitCode;
use kicli::geometry::Point;
use kicli::kicad::erc::rule_check;
use kicli::kicad::{CliFailure, Completed, Discovery, Invocation, KicadCli, Runner};
use kicli::lint::erc::{RuleCheck, delegate};
use kicli::lint::{Drawing, Engine, Findings, Penalty, Rule, RuleId, Tier};
use kicli::model::items::Item;
use kicli::model::{Hierarchy, LoadedFile};

/// The KiCad check `KI-JCT-001` re-publishes (`spec/SPEC.md` §11.1).
const FOUR_WAY: &str = "four_way_junction";

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("{} reads: {error}", path.display()))
}

/// A scratch directory of this check's own.
///
/// **Named for the check that calls it**, because the checks of one binary run
/// in parallel and two sharing a path would write one directory from two
/// threads.
fn scratch(check: &str) -> PathBuf {
    let path = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(check);
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(&path).expect("the scratch directory is made");
    path
}

/// A runner that writes the committed reports where the command asked for them.
///
/// It answers `kicad-cli`'s success code and does exactly what `kicad-cli` does
/// with `-o`: writes the report and prints nothing useful. What it writes is
/// KiCad 10.0.5's own bytes, so the reading under test is the real one.
struct Reports {
    json: String,
    text: String,
}

impl Reports {
    fn of(stem: &str) -> Self {
        Self {
            json: read(&fixture(&format!("{stem}.erc.json"))),
            text: read(&fixture(&format!("{stem}.erc.txt"))),
        }
    }
}

impl Runner for Reports {
    fn run(&self, invocation: &Invocation) -> Result<Completed, std::io::Error> {
        let arguments = &invocation.arguments;
        let json = arguments.iter().any(|argument| argument == "json");
        let into = arguments
            .iter()
            .position(|argument| argument == "-o")
            .and_then(|at| arguments.get(at + 1))
            .expect("the command names an output file");
        std::fs::write(into, if json { &self.json } else { &self.text })?;
        Ok(Completed {
            code: Some(0),
            stdout: String::new(),
            stderr: String::new(),
        })
    }
}

/// A runner that reports one of `kicad-cli`'s own exit codes and writes nothing.
struct Refuses(i32);

impl Runner for Refuses {
    fn run(&self, _invocation: &Invocation) -> Result<Completed, std::io::Error> {
        Ok(Completed {
            code: Some(self.0),
            stdout: String::new(),
            stderr: String::new(),
        })
    }
}

/// A binary the operating system will not start.
struct WillNotStart;

impl Runner for WillNotStart {
    fn run(&self, _invocation: &Invocation) -> Result<Completed, std::io::Error> {
        Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "the file is not executable",
        ))
    }
}

/// A specimen rule that KiCad's own check duplicates.
///
/// It is written here rather than shipped because `KI-JCT-001`'s geometry is a
/// later task's. What this file measures is the **mechanism** around it: one
/// question, two answers, one finding either way.
///
/// **Its own detection reads the schematic; its delegation reads KiCad's JSON.**
/// The two counts therefore have no common ancestor below the drawing itself,
/// which is what stops the equality below from being one number compared with
/// itself.
struct JunctionRule;

impl Rule for JunctionRule {
    fn id(&self) -> RuleId {
        RuleId("KI-JCT-001")
    }

    fn tier(&self) -> Tier {
        Tier::Two
    }

    fn weight(&self) -> Penalty {
        Penalty::points(1)
    }

    fn examine(&self, drawing: &Drawing<'_>, found: &mut Findings<'_>) {
        if drawing.rule_check().covers(FOUR_WAY) {
            delegate(FOUR_WAY, drawing, found);
            return;
        }
        for item in &drawing.schematic().items {
            let Item::Junction(junction) = item else {
                continue;
            };
            found.record(
                junction.at,
                vec![junction.uuid.clone()],
                "four connection points are joined here".to_owned(),
            );
        }
    }
}

static JUNCTION_RULE: JunctionRule = JunctionRule;

/// The rule under test, as the engine takes it.
fn rules() -> Vec<&'static dyn Rule> {
    vec![&JUNCTION_RULE]
}

/// Load the canary fixture and examine it with a given rule-check reading.
fn findings_over_the_canary(reading: &RuleCheck) -> Vec<kicli::lint::Finding> {
    let hierarchy =
        Hierarchy::load(&fixture("erc/canary/canary.kicad_sch")).expect("the drawing loads");
    let placement = hierarchy
        .placements
        .first()
        .expect("the drawing has a sheet");
    let file: &LoadedFile = &hierarchy.files[placement.file];
    let drawing =
        Drawing::read(&file.doc, &file.schematic, &placement.path).with_rule_check(reading);
    Engine::of(rules()).examine(&drawing)
}

/// What KiCad said about the canary fixture, read through the gateway.
fn canary_reading(check: &str) -> RuleCheck {
    let gateway =
        KicadCli::with_runner(PathBuf::from("kicad-cli"), Reports::of("erc/canary/canary"));
    rule_check(
        &gateway,
        &fixture("erc/canary/canary.kicad_sch"),
        &scratch(check),
    )
    .expect("the committed reports are read")
}

#[test]
fn the_gateway_reads_both_reports_of_one_run() {
    let reading = canary_reading("erc-gateway-both-reports");

    assert!(
        reading.has_run(),
        "a reading that ran is not a reading that did not"
    );
    assert_eq!(reading.violations().len(), 11);

    // The identifiers are the reason the JSON is read at all: the text report
    // carries none, and a finding kicli cannot join to a handle is a finding an
    // agent cannot act on.
    let named: usize = reading
        .violations()
        .iter()
        .flat_map(|violation| &violation.items)
        .filter(|item| item.uuid.is_some())
        .count();
    assert_eq!(named, 13, "every item carries the object's identifier");

    // And the coordinates are the corrected ones, not the hundred-times-small
    // ones. R1 pin 1 sits at 25.40, 21.59 mm.
    let pin = reading
        .violations()
        .iter()
        .flat_map(|violation| &violation.items)
        .find(|item| item.description.starts_with("Symbol R1 Pin 1"))
        .expect("the report names R1 pin 1");
    assert_eq!(pin.at, Point::new(254_000, 215_900));
}

#[test]
fn the_project_severities_are_read_and_never_written() {
    // §14.3. A check's severity lives in `.kicad_pro`, and kicli reads what
    // KiCad reported under it and changes nothing. The whole gateway runs over
    // a copy of the fixture project here, and the copy is compared byte for
    // byte afterwards.
    let directory = scratch("erc-gateway-severities-read-only");
    let mut before = Vec::new();
    for name in ["canary.kicad_sch", "canary.kicad_pro"] {
        let text = read(&fixture(&format!("erc/canary/{name}")));
        std::fs::write(directory.join(name), &text).expect("the copy is written");
        before.push((name, text));
    }

    let gateway =
        KicadCli::with_runner(PathBuf::from("kicad-cli"), Reports::of("erc/canary/canary"));
    let reading = rule_check(&gateway, &directory.join("canary.kicad_sch"), &directory)
        .expect("the rule check is read");

    for (name, text) in &before {
        assert_eq!(
            &read(&directory.join(name)),
            text,
            "{name} is byte for byte what it was before the rule check"
        );
    }

    // The presence control on that claim: the project file really does turn a
    // check on, and kicli really does see it. Without this the comparison above
    // would hold just as well over a project file nothing read.
    assert!(
        reading.covers(FOUR_WAY),
        "the project's own severity turned this check on"
    );
    assert!(
        !reading.covers("simulation_model_issue"),
        "and KiCad's default left this one off"
    );
}

#[test]
fn an_absent_binary_is_exit_six_and_a_broken_run_is_not() {
    // Two rows of §6.2's table that a careless test would merge. Both are asked
    // through the same gateway function, so the difference is the code's and
    // not the test's.
    let nowhere = Discovery {
        environment: Some("/nonexistent/kicad-cli".to_owned()),
        configured: None,
    };
    let absent = KicadCli::locate(&nowhere).expect_err("nothing is there");
    assert!(matches!(absent, CliFailure::NotFound { .. }));
    assert_eq!(ExitCode::for_tool_failure(&absent), ExitCode::Tool);
    assert_eq!(ExitCode::Tool.code(), 6);
    let message = absent.to_string();
    assert!(
        message.contains("kicad-cli") && message.contains("KiCad 10"),
        "the error names the binary and how to install it: {message}"
    );

    // Present, and the operating system will not start it: still exit 6, and
    // still not the same reading.
    let unusable = rule_check(
        &KicadCli::with_runner(PathBuf::from("kicad-cli"), WillNotStart),
        &fixture("erc/canary/canary.kicad_sch"),
        &scratch("erc-gateway-will-not-start"),
    )
    .expect_err("a binary that will not start is a failure");
    assert!(matches!(unusable, CliFailure::NotUsable { .. }));
    assert_eq!(ExitCode::for_tool_failure(&unusable), ExitCode::Tool);

    // Present, started, and unhappy: these are NOT exit 6. `kicad-cli` 3 is a
    // bad input file, which is kicli 4; everything else it reports is kicli 1.
    let directory = scratch("erc-gateway-broken-run");
    for (reported, expected) in [
        (1, ExitCode::Operation),
        (2, ExitCode::Operation),
        (3, ExitCode::File),
        (5, ExitCode::Operation),
        (6, ExitCode::Operation),
    ] {
        let failure = rule_check(
            &KicadCli::with_runner(PathBuf::from("kicad-cli"), Refuses(reported)),
            &fixture("erc/canary/canary.kicad_sch"),
            &directory,
        )
        .expect_err("a refused run is a failure");
        let code = ExitCode::for_tool_failure(&failure);
        assert_eq!(
            code, expected,
            "kicad-cli {reported} became the wrong kicli code: {failure}"
        );
        assert_ne!(
            code,
            ExitCode::Tool,
            "kicad-cli {reported} means the tool ran, so it is not a missing tool"
        );
    }
}

#[test]
fn a_report_kicli_cannot_read_is_a_refusal_and_not_a_coordinate() {
    // A report that arrives corrupt must never become a position. It becomes a
    // structured failure, translated through the one exit-code table.
    let directory = scratch("erc-gateway-unreadable-report");
    let broken = Reports {
        json: "{\"sheets\": []}".to_owned(),
        text: read(&fixture("erc/canary/canary.erc.txt")),
    };
    let failure = rule_check(
        &KicadCli::with_runner(PathBuf::from("kicad-cli"), broken),
        &fixture("erc/canary/canary.kicad_sch"),
        &directory,
    )
    .expect_err("a report that does not match its twin is refused");
    assert_eq!(ExitCode::for_tool_failure(&failure), ExitCode::Operation);
    assert!(
        failure.to_string().contains("do not describe the same run"),
        "the refusal says what is wrong: {failure}"
    );
}

#[test]
fn a_delegating_rule_reports_once_however_the_project_is_configured() {
    // §11.1's non-double-counting, as the two configurations of one drawing.
    //
    // The two sides have no common ancestor below the fixture: the delegated
    // finding is built from the JSON `kicad-cli` wrote, and the geometric one
    // from kicli's own parse of the schematic. A break in either moves one and
    // not the other.
    let covered = canary_reading("erc-gateway-delegation-covered");
    assert!(covered.covers(FOUR_WAY));
    let attributed = findings_over_the_canary(&covered);

    // The same drawing, with the project's severity for this check turned off,
    // which is KiCad's own default (§11.1: an untouched project silently
    // passes). Everything else about the reading is unchanged.
    let ignored = RuleCheck::ran(covered.violations().to_vec(), vec![FOUR_WAY.to_owned()]);
    assert!(!ignored.covers(FOUR_WAY));
    let detected = findings_over_the_canary(&ignored);

    // One occurrence, one finding, either way — not two, which is the failure
    // being guarded against, and not none, which would make the claim vacuous.
    assert_eq!(attributed.len(), 1, "{attributed:?}");
    assert_eq!(detected.len(), 1, "{detected:?}");
    assert_eq!(
        covered.of_check(FOUR_WAY, &attributed[0].sheet).count(),
        1,
        "KiCad reported the occurrence once"
    );

    // And the two findings really did come from different places, so the
    // equality above is not two readings of one number.
    assert!(
        attributed[0].message.contains("reported by KiCad's ERC"),
        "the attributed finding says who found it: {}",
        attributed[0].message
    );
    assert!(
        !detected[0].message.contains("reported by KiCad's ERC"),
        "the geometric finding does not: {}",
        detected[0].message
    );
    assert_ne!(
        attributed[0].objects, detected[0].objects,
        "KiCad names the wires it joined; kicli names the junction it drew"
    );

    // Both are reported under kicli's own code, which is the attribution half.
    for finding in [&attributed[0], &detected[0]] {
        assert_eq!(finding.rule, RuleId("KI-JCT-001"));
    }
}

#[test]
fn a_rule_check_nobody_ran_sends_a_rule_to_its_own_detection() {
    // The case that would fall silent if `covers` folded "ignored" together
    // with "never asked": a machine with no KiCad install must still get
    // kicli's own finding, not nothing.
    let findings = findings_over_the_canary(&RuleCheck::NOT_RUN);
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(!findings[0].message.contains("reported by KiCad's ERC"));
}
