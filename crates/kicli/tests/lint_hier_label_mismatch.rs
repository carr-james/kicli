//! `KI-HIER-001` reports KiCad's `hier_label_mismatch` and finds nothing itself.
//!
//! # Where each side of every comparison comes from
//!
//! This rule is a delegation, so the trap `opening-1` named — *"a check
//! asserting that the reported net equals the extractor's net is a
//! degenerate-equality candidate"* — is live in a sharper form here: the rule's
//! only input is KiCad's report, so a check that took its expectation from the
//! same report through the same reader would compare one parse with itself.
//!
//! It is avoided the same way the fixture's other oracles are. The
//! **expectation** is read out of `erc/hier/hier.erc.txt` — the *text* report
//! `kicad-cli` 10.0.5 wrote, in KiCad's own sentences, by the line-oriented
//! reader in [`mismatches_kicad_reported`] below. The **actual** is kicli's
//! whole pipeline: `kicad::erc::read_reports` parsing the *JSON* report for the
//! violations and their objects, the engine running every registered rule, and
//! [`delegate`](kicli::lint::erc::delegate) attributing them. The two share the
//! fixture's bytes and nothing above them, and only one of them is the code
//! under test.
//!
//! # What KiCad says about this fixture, measured before anything was asserted
//!
//! `kicad-cli sch erc` on `erc/hier/hier.kicad_sch`, KiCad 10.0.5, reports
//! `hier_label_mismatch` **twice** and on two different sheets:
//!
//! - on the root sheet, *"Sheet pin BUSY has no matching hierarchical label
//!   inside the sheet"*;
//! - on `/mismatched/`, *"Hierarchical label WRONG has no matching sheet pin in
//!   the parent sheet"*.
//!
//! The `matched` sheet carries the pin `READY` and the label `READY`, and KiCad
//! reports **nothing** about it. That is the presence control for an absence
//! claim, and it is intrinsic rather than a second fixture: both cases are in
//! one project and one ERC run, so a run that had stopped reading after the
//! first sheet could not produce this pair of answers.
//!
//! Both are reported at severity `error` over a `.kicad_pro` that sets no
//! severity at all, and the key is absent from the report's `ignored_checks`.
//! **That is the source for this rule's tier**: Tier 1 is kicli agreeing with
//! KiCad's own default rather than asserting over it.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use kicli::cli::ExitCode;
use kicli::kicad::erc::{read_reports, rule_check};
use kicli::kicad::{CliFailure, Completed, Discovery, Invocation, KicadCli, Runner};
use kicli::lint::erc::RuleCheck;
use kicli::lint::{
    Blocker, Density, Drawing, Engine, Finding, Findings, Penalty, RawPenalty, Report, Rule,
    RuleId, Tier,
};
use kicli::model::Hierarchy;
use kicli::model::items::SheetPath;
use kicli_probe::oracle::Kicad;
use kicli_probe::scratch::Fixtures;

/// The rule under test.
const RULE: RuleId = RuleId("KI-HIER-001");

/// The KiCad check it delegates to.
const CHECK: &str = "hier_label_mismatch";

/// The fixture's root sheet, which is the whole project.
const ROOT: &str = "erc/hier/hier.kicad_sch";

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("{} reads: {error}", path.display()))
}

/// Every `hier_label_mismatch` sentence of the committed **text** report.
///
/// This is the oracle, and it is deliberately a different reader from kicli's.
/// A text report's violation line reads
/// `[hier_label_mismatch]: Sheet pin BUSY has no matching …`, so the key is the
/// bracketed prefix and the sentence is the rest. Nothing kicli parsed enters
/// it, and nothing here knows what a sheet path is.
fn mismatches_kicad_reported() -> BTreeSet<String> {
    let opener = format!("[{CHECK}]: ");
    read(&fixture("erc/hier/hier.erc.txt"))
        .lines()
        .map(str::trim)
        .filter_map(|line| line.strip_prefix(&opener).map(str::to_owned))
        .collect()
}

/// What KiCad said about the fixture, read the way kicli reads it.
///
/// `read_reports` is the whole of kicli's understanding of the two reports, so
/// this needs no KiCad install: the bytes are committed and `kicad-cli` wrote
/// them.
fn committed_reading() -> RuleCheck {
    read_reports(
        &read(&fixture("erc/hier/hier.erc.json")),
        &read(&fixture("erc/hier/hier.erc.txt")),
    )
    .expect("the committed pair of reports is read")
}

/// Every `KI-HIER-001` finding of the fixture, given a rule-check reading.
///
/// The drawings are built the way a scoring command will build them: one per
/// placement, each carrying the reading. Every registered rule runs, so a
/// finding reported under another rule's code would not be counted here.
fn findings_with(reading: &RuleCheck) -> Vec<Finding> {
    let hierarchy = Hierarchy::load(&fixture(ROOT)).expect("the hierarchy loads");
    let drawings: Vec<Drawing<'_>> = hierarchy
        .placements
        .iter()
        .map(|placement| {
            let file = &hierarchy.files[placement.file];
            Drawing::read(&file.doc, &file.schematic, &placement.path).with_rule_check(reading)
        })
        .collect();
    Engine::of_every_rule()
        .examine_all(&drawings)
        .into_iter()
        .filter(|finding| finding.rule == RULE)
        .collect()
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

/// A scored rule of the same shape, used only as a control on the scorer.
///
/// It exists so that "this rule moves no score" is a measurement of the tier
/// rather than of a scorer that moves for nothing. Its weight is three points
/// and its tier is two, and that is the only way it differs.
struct ScoredTwin;

impl Rule for ScoredTwin {
    fn id(&self) -> RuleId {
        RuleId("KI-TWIN-001")
    }

    fn tier(&self) -> Tier {
        Tier::Two
    }

    fn weight(&self) -> Penalty {
        Penalty::points(3)
    }

    fn examine(&self, _drawing: &Drawing<'_>, _found: &mut Findings<'_>) {}
}

#[test]
fn kicad_reports_the_mismatch_twice_and_says_nothing_about_the_matched_pair() {
    // The fixture's own claim, taken from KiCad's words and not from kicli's.
    // `ENGINEERING.md`: fixture expectations are verified against KiCad, never
    // hand-asserted — and `the_committed_reports_are_current` below is the
    // environment-gated half that keeps these bytes KiCad's.
    let reported = mismatches_kicad_reported();
    assert_eq!(
        reported,
        BTreeSet::from([
            "Hierarchical label WRONG has no matching sheet pin in the parent sheet".to_owned(),
            "Sheet pin BUSY has no matching hierarchical label inside the sheet".to_owned(),
        ]),
        "KiCad reports both directions of the mismatch and nothing else"
    );

    // The presence control for the absence claim: the matched pair really is in
    // the same drawing and the same run, so the silence about it is a verdict
    // rather than a sheet KiCad never opened.
    let text = read(&fixture("erc/hier/hier.erc.txt"));
    assert!(
        text.contains("Hierarchical Label 'READY'")
            && text.contains("Hierarchical Sheet Pin 'READY'"),
        "KiCad saw the matched pin and the matched label: {text}"
    );
    for sentence in &reported {
        assert!(
            !sentence.contains("READY"),
            "no mismatch is reported about the matched pair: {sentence}"
        );
    }

    // And the severity is KiCad's default over a project that sets none, which
    // is this rule's whole argument for Tier 1.
    assert!(
        read(&fixture("erc/hier/hier.kicad_pro")).contains("\"rule_severities\": {}"),
        "the project file sets no severity for anything"
    );
    let reading = committed_reading();
    let errors = reading
        .violations()
        .iter()
        .filter(|violation| violation.check == CHECK)
        .filter(|violation| violation.severity == kicli::lint::KicadSeverity::Error)
        .count();
    assert_eq!(errors, 2, "KiCad calls both of them errors by default");
    assert!(
        !reading.ignored().contains(&CHECK.to_owned()),
        "and it did not skip the check: {:?}",
        reading.ignored()
    );
}

#[test]
fn the_rule_reports_kicads_findings_on_the_sheets_kicad_named() {
    let findings = findings_with(&committed_reading());

    // One finding per sentence KiCad wrote, and the sentence is kept, so a
    // reader can tell which tool found it (§11.1's attribution half).
    let messages: Vec<&str> = findings
        .iter()
        .map(|finding| finding.message.as_str())
        .collect();
    assert_eq!(findings.len(), 2, "{messages:?}");
    for sentence in mismatches_kicad_reported() {
        assert!(
            messages.iter().any(|message| message.contains(&sentence)),
            "KiCad's own words survive into the finding: {sentence} in {messages:?}"
        );
    }
    for finding in &findings {
        assert_eq!(finding.rule, RULE, "reported under kicli's own code");
        assert_eq!(finding.tier, Tier::One);
        assert!(
            finding.message.contains("reported by KiCad's ERC"),
            "the finding says who found it: {}",
            finding.message
        );
        assert_eq!(finding.objects.len(), 1, "KiCad named one object");
        assert!(finding.fix.is_none(), "the repair is a drawing decision");
    }

    // The two findings land on two different sheet placements, because KiCad
    // reported them on two different sheets: the sheet pin is the parent's
    // mistake and the hierarchical label is the child's. A delegation that
    // ignored the sheet would put both on every placement.
    let sheets: BTreeSet<&SheetPath> = findings.iter().map(|finding| &finding.sheet).collect();
    assert_eq!(sheets.len(), 2, "{sheets:?}");
    let hierarchy = Hierarchy::load(&fixture(ROOT)).expect("the hierarchy loads");
    assert_eq!(
        hierarchy.placements.len(),
        3,
        "the root and its two child placements"
    );
    let matched = hierarchy
        .placements
        .iter()
        .map(|placement| &placement.path)
        .find(|path| path.0.ends_with("33000002-0000-4000-8000-000000000001"))
        .expect("the matched child is placed");
    assert!(
        !sheets.contains(&matched),
        "nothing is reported on the sheet whose pin and label agree: {sheets:?}"
    );
}

#[test]
fn it_fails_the_gate_and_moves_no_score() {
    // T4's first direction, exercised by the first delegated rule. The sheet
    // that carries a finding fails, and its score is still a hundred.
    let findings = findings_with(&committed_reading());
    let density = Density::of_counts(0, 0);
    let report = Report::of(&findings, density);

    assert!(!report.gate().passes(), "{}", report.text());
    assert_eq!(report.gate().word(), "fail");
    assert_eq!(report.score().score(), 100, "a blocker is not a cost");
    assert_eq!(report.score().raw(), RawPenalty::ZERO);
    assert_eq!(
        report.gate().blockers(),
        [Blocker::Blocking {
            rule: RULE,
            count: 2
        }],
        "it blocks as a tier, not as a saturation"
    );

    // The control on the half that reports zero. A scorer that moved for
    // nothing would make the assertion above say nothing, so the same scorer is
    // asked about a finding of a scored rule at the same density.
    let twin = ScoredTwin;
    let sheet = SheetPath("/".to_owned());
    let mut collected = Findings::of(&twin, &sheet);
    collected.record(Default::default(), Vec::new(), "a cost".to_owned());
    let scored = collected.into_vec();
    assert_ne!(
        RawPenalty::of(&scored, density),
        RawPenalty::ZERO,
        "the scorer does move for a scored finding, so the zero above is the tier's"
    );

    // And the clean direction: the same drawing with nothing reported passes.
    assert!(Report::of(&[], density).gate().passes());
}

#[test]
fn a_broken_drawing_passes_the_gate_when_nobody_ran_the_rule_check() {
    // **The hazard, executable, and the worst output this milestone can
    // produce.** These are the same bytes KiCad calls two errors. With no
    // reading attached, `KI-HIER-001` reports nothing and the gate says pass —
    // because this rule has no geometry of its own to fall through to, and
    // `RuleCheck::covers` answers `false` for "nobody ran it" exactly as it
    // does for "the project turned it off".
    //
    // The silence is right for a rule: "the drawing was never examined" is not
    // a fact about the drawing, and §6.1 already says what the absence of
    // `kicad-cli` is — a structured error and exit 6, not a finding. What the
    // check records is that **the repair cannot live in the finding stream**,
    // and therefore belongs to the `sch score` surface.
    let density = Density::of_counts(0, 0);

    let unexamined = findings_with(&RuleCheck::NOT_RUN);
    assert!(unexamined.is_empty(), "{unexamined:?}");
    assert!(
        Report::of(&unexamined, density).gate().passes(),
        "a drawing KiCad calls an error passes the gate when nobody asked KiCad"
    );

    // The discriminator is the point. "Nobody ran it" and "KiCad ran it and
    // found nothing" produce byte-identical findings and byte-identical
    // verdicts, so no reader of the output can tell them apart...
    let clean = RuleCheck::ran(Vec::new(), Vec::new());
    assert_eq!(findings_with(&clean), unexamined);
    assert_eq!(
        Report::of(&findings_with(&clean), density).text(),
        Report::of(&unexamined, density).text()
    );

    // ...and the fact that tells them apart is on the reading, where the `sch
    // score` surface can reach it and this rule cannot.
    assert!(clean.has_run());
    assert!(!RuleCheck::NOT_RUN.has_run());

    // The third silence, which is a different situation again: KiCad ran, and
    // the project turned this check off. kicli's answer is the same silence,
    // and for this rule that is the only answer available — there is no own
    // detector for the occurrence to fall through to.
    let reading = committed_reading();
    assert_eq!(
        findings_with(&reading).len(),
        2,
        "the control: it does fire"
    );
    let turned_off = RuleCheck::ran(reading.violations().to_vec(), vec![CHECK.to_owned()]);
    assert!(!turned_off.covers(CHECK));
    assert!(findings_with(&turned_off).is_empty());
    assert!(
        turned_off.has_run(),
        "and this one is not an absent tool, which is why the surface needs both questions"
    );
}

#[test]
fn an_absent_kicad_cli_is_exit_six_and_a_broken_run_is_not() {
    // §6.2's two rows that a careless test merges, asked on the path this rule
    // depends on: `sch score --gate` over this fixture cannot answer without
    // `kicad-cli`, so the two ways it can fail to answer must stay apart.
    let nowhere = Discovery {
        environment: Some("/nonexistent/kicad-cli".to_owned()),
        configured: None,
    };
    let absent = KicadCli::locate(&nowhere).expect_err("nothing is there");
    assert!(matches!(absent, CliFailure::NotFound { .. }));
    assert_eq!(ExitCode::for_tool_failure(&absent), ExitCode::Tool);
    assert_eq!(ExitCode::Tool.code(), 6);

    // Present, started, and unhappy. `kicad-cli` 3 is a bad input file, which
    // is kicli 4; the rest is kicli 1. None of them is 6, because the tool was
    // there.
    let directory = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("hier-broken-run");
    std::fs::create_dir_all(&directory).expect("the scratch directory is made");
    for (reported, expected) in [
        (1, ExitCode::Operation),
        (3, ExitCode::File),
        (6, ExitCode::Operation),
    ] {
        let failure = rule_check(
            &KicadCli::with_runner(PathBuf::from("kicad-cli"), Refuses(reported)),
            &fixture(ROOT),
            &directory,
        )
        .expect_err("a refused run is a failure");
        assert_eq!(
            ExitCode::for_tool_failure(&failure),
            expected,
            "kicad-cli {reported} became the wrong kicli code: {failure}"
        );
        assert_ne!(
            ExitCode::for_tool_failure(&failure),
            ExitCode::Tool,
            "kicad-cli {reported} means the tool ran, so it is not a missing tool"
        );
    }
}

#[test]
fn the_committed_reports_are_current() {
    // The environment-gated half, and the reason this fixture's expectations
    // are KiCad's rather than this lane's. It runs the real binary through
    // kicli's own gateway and compares what KiCad writes today against the
    // bytes beside the fixture.
    //
    // A lane worktree's green here counts toward no tick (`CLAUDE.md`); the
    // measurement it makes is what the entry records.
    let Some(_asked) = Kicad::found_or_skip("regenerate the hier ERC reports") else {
        return;
    };
    let fixtures = Fixtures::new(env!("CARGO_TARGET_TMPDIR"), env!("CARGO_MANIFEST_DIR"));
    let cli = KicadCli::locate(&Discovery::new(&kicli::model::Config::default()))
        .expect("kicad-cli is on this machine");
    let version = cli.version().expect("kicad-cli is a version kicli reads");
    println!("measured against kicad-cli {version}");

    // KiCad writes a .kicad_prl beside any project it opens, so the fixture is
    // copied out and the tool runs on the copy. The fixture tree stays exactly
    // as committed.
    let directory = fixtures.scratch_directory("hier-currency", "erc/hier");
    rule_check(&cli, &directory.join("hier.kicad_sch"), &directory)
        .expect("the fresh reports read");

    for (fresh, committed) in [
        ("kicli-erc.json", "erc/hier/hier.erc.json"),
        ("kicli-erc.txt", "erc/hier/hier.erc.txt"),
    ] {
        let dated = |text: &str| -> Vec<String> {
            text.lines()
                .map(str::trim_end)
                .filter(|line| {
                    let line = line.trim_start();
                    !line.starts_with("\"date\":") && !line.starts_with("ERC report (")
                })
                .map(str::to_owned)
                .collect()
        };
        assert_eq!(
            dated(&read(&fixture(committed))),
            dated(&read(&directory.join(fresh))),
            "{committed} is what kicad-cli {version} writes today"
        );
    }
}
