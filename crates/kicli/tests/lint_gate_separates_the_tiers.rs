//! The gate and the score are two answers, and neither one decides the other.
//!
//! Four claims, in two pairs. Each pair exists because one check over a drawing
//! carrying both kinds of finding passes just as happily when the
//! implementation has swapped them, and a swap is the mistake most worth
//! catching here.
//!
//! 1. A **blocking** finding does not move the raw penalty or the score.
//!    Its control is the same finding, with the same weight, in the scored
//!    tier: that one **does** move both. Without the control the first claim
//!    passes on a rule whose weight is nothing, which proves nothing at all.
//! 2. A **scored** finding does not fail the gate. Its control is the same
//!    finding from a rule that declared it saturates: that one **does** fail
//!    the gate. The two rules here report the same findings, on the same
//!    drawing, for the same weight. **The declaration is the only difference**,
//!    so nothing but the declaration can be deciding the verdict.
//!
//! # The rules this file runs, and why they are not the shipped ones
//!
//! No rule ships yet. These four are written here, as instruments, in the same
//! way `tests/specimen_rules/` holds instruments for the registry seam. Each is
//! a real integer test over real geometry, so none of them is a stub that
//! agrees with the code by construction.
//!
//! # What the fixture is worth
//!
//! `tests/fixtures/sch/score/high_and_blocked.kicad_sch` is a committed
//! drawing that scores well and fails the gate. It is a genuine two-rail
//! ladder: eight resistors, ten wires, four power symbols. It carries exactly
//! two blemishes, one of each kind, and KiCad's own electrical rule check
//! reports the blocking one and no other drawing fault.
//!
//! **The number it scores is produced by rules written in this file**, because
//! there are no others. The claim the fixture supports today is that the two
//! answers are computed and reported independently over a real file. It will be
//! worth more when the catalogue's own rules run over it.

use std::path::{Path, PathBuf};

use kicli::cli::ExitCode;
use kicli::geometry::Point;
use kicli::kicad::{Discovery, KicadCli};
use kicli::lint::gate::{Blocker, Counted, Report, Saturation};
use kicli::lint::score::{Density, Normaliser, RawPenalty};
use kicli::lint::{Drawing, Engine, Findings, Penalty, Rule, RuleId, Tier};
use kicli::model::items::{Item, LineKind};
use kicli::model::{Hierarchy, LoadedFile};
use kicli_probe::Probe;

/// The weight the two off-grid rules share.
///
/// It is large on purpose. A blocking rule that weighed nothing could not move
/// a score even if the scorer read it, so the first claim would hold for the
/// wrong reason.
const HEAVY: u16 = 50;

/// Every non-power symbol that sits off the connection grid, as a blocking
/// rule reports it.
fn off_grid(drawing: &Drawing<'_>, found: &mut Findings<'_>) {
    for symbol in drawing.schematic().symbols() {
        if !symbol.is_power() && !symbol.at.is_on_grid() {
            found.record(
                symbol.at,
                vec![symbol.uuid.clone()],
                "the symbol sits off the connection grid".to_owned(),
            );
        }
    }
}

/// Every wire that runs neither across nor down the page.
///
/// A slanted wire is the readability defect this file counts, because it is one
/// finding for one wire: the count and the wire count are then the same kind of
/// number, which is what a share of the sheet has to be.
fn slanted(drawing: &Drawing<'_>, found: &mut Findings<'_>) {
    for item in &drawing.schematic().items {
        let Item::Line(line) = item else { continue };
        if line.kind != LineKind::Wire {
            continue;
        }
        if line.from.x != line.to.x && line.from.y != line.to.y {
            found.record(
                line.from,
                vec![line.uuid.clone()],
                "the wire runs at an angle".to_owned(),
            );
        }
    }
}

/// Off-grid symbols, reported as blocking, with a weight the scorer must ignore.
struct BlockingOffGrid;

impl Rule for BlockingOffGrid {
    fn id(&self) -> RuleId {
        RuleId("KI-GATE-001")
    }

    fn tier(&self) -> Tier {
        Tier::One
    }

    fn weight(&self) -> Penalty {
        Penalty::points(HEAVY)
    }

    fn examine(&self, drawing: &Drawing<'_>, found: &mut Findings<'_>) {
        off_grid(drawing, found);
    }
}

/// The same rule, the same weight, in the scored tier.
///
/// The control for the blocking claim. This one must move the score, or the
/// blocking claim is being made by a rule that could not have moved it anyway.
struct ScoredOffGrid;

impl Rule for ScoredOffGrid {
    fn id(&self) -> RuleId {
        RuleId("KI-GATE-004")
    }

    fn tier(&self) -> Tier {
        Tier::Two
    }

    fn weight(&self) -> Penalty {
        Penalty::points(HEAVY)
    }

    fn examine(&self, drawing: &Drawing<'_>, found: &mut Findings<'_>) {
        off_grid(drawing, found);
    }
}

/// Slanted wires, scored, and blocking once they cover enough of the sheet.
struct SaturatingSlant;

impl Rule for SaturatingSlant {
    fn id(&self) -> RuleId {
        RuleId("KI-GATE-002")
    }

    fn tier(&self) -> Tier {
        Tier::Two
    }

    fn weight(&self) -> Penalty {
        Penalty::points(1)
    }

    fn normaliser(&self) -> Normaliser {
        Normaliser::PerWire
    }

    fn saturation(&self) -> Saturation {
        Saturation::of(Counted::Wires)
    }

    fn examine(&self, drawing: &Drawing<'_>, found: &mut Findings<'_>) {
        slanted(drawing, found);
    }
}

/// The same rule, the same weight, the same normaliser, declaring no saturation.
///
/// The control for the saturation claim. It reports exactly what
/// [`SaturatingSlant`] reports, so a gate that failed on this one would be
/// failing on the findings rather than on the declaration.
struct PlainSlant;

impl Rule for PlainSlant {
    fn id(&self) -> RuleId {
        RuleId("KI-GATE-003")
    }

    fn tier(&self) -> Tier {
        Tier::Two
    }

    fn weight(&self) -> Penalty {
        Penalty::points(1)
    }

    fn normaliser(&self) -> Normaliser {
        Normaliser::PerWire
    }

    fn examine(&self, drawing: &Drawing<'_>, found: &mut Findings<'_>) {
        slanted(drawing, found);
    }
}

static BLOCKING_OFF_GRID: BlockingOffGrid = BlockingOffGrid;
static SCORED_OFF_GRID: ScoredOffGrid = ScoredOffGrid;
static SATURATING_SLANT: SaturatingSlant = SaturatingSlant;
static PLAIN_SLANT: PlainSlant = PlainSlant;

/// Where the drawings this binary writes go.
fn scratch() -> PathBuf {
    Path::new(env!("CARGO_TARGET_TMPDIR")).join("lint-gate")
}

/// The committed fixture tree.
fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

/// A drawing of `wires` wires, `slant` of which run at an angle.
///
/// **The probe name is the check's own**, because the tests of one binary run
/// in parallel and a probe writes to a path keyed by its name. Two checks
/// sharing a name write one file from two threads.
fn slanted_sheet(check: &str, wires: u32, slant: u32) -> PathBuf {
    let mut probe = Probe::new(check, scratch());
    for index in 0..wires {
        let y = kicli_probe::millimetres(25.4 + 2.54 * f64::from(index));
        let end_y = if index < slant {
            kicli_probe::millimetres(26.67 + 2.54 * f64::from(index))
        } else {
            y.clone()
        };
        probe.wire(("25.4", &y), ("50.8", &end_y));
    }
    probe.write()
}

/// Every placement of a written or committed drawing, as the rules see it.
fn placements(hierarchy: &Hierarchy) -> Vec<Drawing<'_>> {
    hierarchy
        .placements
        .iter()
        .map(|placement| {
            let file: &LoadedFile = &hierarchy.files[placement.file];
            Drawing::read(&file.doc, &file.schematic, &placement.path)
        })
        .collect()
}

/// Score and gate one drawing with a named set of rules.
fn report(path: &Path, rules: Vec<&'static dyn Rule>) -> (Report, Density) {
    let hierarchy = Hierarchy::load(path).expect("the drawing loads");
    let drawings = placements(&hierarchy);
    let drawing = drawings.first().expect("the drawing has a sheet");
    let density = Density::of(drawing);
    let findings = Engine::of(rules).examine(drawing);
    (Report::of(&findings, density), density)
}

/// How many findings a named set of rules makes on a drawing.
fn count(path: &Path, rules: Vec<&'static dyn Rule>) -> usize {
    let hierarchy = Hierarchy::load(path).expect("the drawing loads");
    let drawings = placements(&hierarchy);
    let drawing = drawings.first().expect("the drawing has a sheet");
    Engine::of(rules).examine(drawing).len()
}

#[test]
fn a_blocking_finding_moves_neither_the_raw_penalty_nor_the_score() {
    let path = fixture("sch/score/high_and_blocked.kicad_sch");

    // The control on the instrument. A rule that found nothing would leave a
    // clean score and a passing gate, and every claim below would be vacuous.
    assert_eq!(
        count(&path, vec![&BLOCKING_OFF_GRID]),
        1,
        "the blocking rule found the off-grid symbol"
    );

    let (blocked, _) = report(&path, vec![&BLOCKING_OFF_GRID]);
    assert_eq!(blocked.score().raw(), RawPenalty::ZERO);
    assert_eq!(blocked.score().score(), 100);
    assert!(!blocked.gate().passes(), "and it still fails the gate");

    // The control the claim is worthless without: the same finding, the same
    // weight, scored instead of blocking. Fifty points of penalty leaves
    // nothing, so the tier is what the scorer read and not the weight.
    let (scored, _) = report(&path, vec![&SCORED_OFF_GRID]);
    assert_eq!(scored.score().raw(), RawPenalty::billionths(50_000_000_000));
    assert_eq!(scored.score().score(), 14);
}

#[test]
fn a_scored_finding_does_not_fail_the_gate() {
    let path = fixture("sch/score/high_and_blocked.kicad_sch");

    assert_eq!(
        count(&path, vec![&PLAIN_SLANT]),
        1,
        "the scored rule found the slanted wire"
    );

    let (scored, _) = report(&path, vec![&PLAIN_SLANT]);
    assert!(scored.gate().passes(), "{}", scored.text());
    assert!(
        scored.score().score() < 100,
        "and it did cost something: {}",
        scored.text()
    );
}

#[test]
fn a_rule_at_or_above_its_saturation_fails_the_gate() {
    // Six wires of ten run at an angle. Six of ten is above one half, and one
    // half is the share the rule declares.
    let path = slanted_sheet("saturated", 10, 6);
    let (saturated, density) = report(&path, vec![&SATURATING_SLANT]);
    assert_eq!(
        density.wires(),
        10,
        "the drawing holds the wires it was given"
    );
    assert_eq!(count(&path, vec![&SATURATING_SLANT]), 6);

    assert!(!saturated.gate().passes(), "{}", saturated.text());
    let blockers = saturated.gate().blockers();
    assert_eq!(blockers.len(), 1);
    assert_eq!(
        blockers[0],
        Blocker::Saturated {
            rule: RuleId("KI-GATE-002"),
            count: 6,
            total: 10,
            counted: Counted::Wires,
            share: (1, 2),
        }
    );
    // The reader is told which rule, how many, out of what, and against what
    // share. Nothing else explains a saturating failure.
    assert_eq!(
        saturated.text(),
        concat!(
            "score 79  gate fail  raw 6.0\n",
            "  saturated KI-GATE-002  wires 6 of 10 >= 1/2\n",
        )
    );
}

#[test]
fn a_rule_below_its_saturation_passes_the_gate_and_still_scores() {
    // Four wires of ten. Four of ten is below one half.
    let path = slanted_sheet("below", 10, 4);
    let (below, density) = report(&path, vec![&SATURATING_SLANT]);
    assert_eq!(density.wires(), 10);
    assert_eq!(count(&path, vec![&SATURATING_SLANT]), 4);

    assert!(below.gate().passes(), "{}", below.text());
    assert_eq!(
        below.score().raw(),
        RawPenalty::billionths(4_000_000_000),
        "four findings of a point each, on a sheet at the reference wire count"
    );
    assert_eq!(below.score().score(), 85);

    // The pair. The same drawing, the same findings, the same weight: only the
    // declaration differs, and only the verdict changes.
    let (declared, _) = report(&path, vec![&PLAIN_SLANT]);
    assert_eq!(declared.score().raw(), below.score().raw());
    assert!(declared.gate().passes());

    let saturating = slanted_sheet("below-control", 10, 5);
    let (control, _) = report(&saturating, vec![&SATURATING_SLANT]);
    assert!(
        !control.gate().passes(),
        "one more slanted wire reaches the share: {}",
        control.text()
    );
    let (unsaturating, _) = report(&saturating, vec![&PLAIN_SLANT]);
    assert!(
        unsaturating.gate().passes(),
        "and the rule that declared nothing still passes on it: {}",
        unsaturating.text()
    );
}

#[test]
fn the_fixture_scores_high_and_fails_the_gate_in_one_output() {
    let path = fixture("sch/score/high_and_blocked.kicad_sch");
    let (both, density) = report(&path, vec![&BLOCKING_OFF_GRID, &PLAIN_SLANT]);

    // What the drawing is, measured from the drawing rather than asserted.
    assert_eq!((density.symbols(), density.wires()), (8, 10));

    assert_eq!(
        both.text(),
        concat!(
            "score 96  gate fail  raw 1.0\n",
            "  blocking  KI-GATE-001  findings 1\n",
        )
    );

    // Both facts, from one output, without a footnote: the number is high and
    // the verdict is fail.
    assert!(both.score().score() >= 90);
    assert!(!both.gate().passes());
}

#[test]
fn the_gate_decides_the_exit_code_and_a_missing_tool_decides_its_own() {
    let path = fixture("sch/score/high_and_blocked.kicad_sch");
    let (blocked, _) = report(&path, vec![&BLOCKING_OFF_GRID]);
    let (clean, _) = report(&path, vec![&PLAIN_SLANT]);

    // Findings are data. A drawing that reported a scored finding and passed
    // the gate leaves nothing but success.
    assert_eq!(ExitCode::for_gate(clean.gate()), ExitCode::Success);
    assert_eq!(ExitCode::for_gate(clean.gate()).code(), 0);

    // A gate failure is its own code, and the drawing scored 100 while it
    // happened.
    assert_eq!(blocked.score().score(), 100);
    assert_eq!(ExitCode::for_gate(blocked.gate()), ExitCode::Gate);
    assert_eq!(ExitCode::for_gate(blocked.gate()).code(), 5);

    // An absent kicad-cli is the tool row, because half of the blocking tier
    // is the electrical rule check's.
    let nowhere = Discovery {
        environment: Some("/nonexistent/kicad-cli".to_owned()),
        configured: None,
    };
    let failure = KicadCli::locate(&nowhere).expect_err("nothing is at that path");
    assert_eq!(ExitCode::for_tool_failure(&failure), ExitCode::Tool);
    assert_eq!(ExitCode::for_tool_failure(&failure).code(), 6);
}

#[test]
fn a_rule_that_declares_nothing_takes_the_catalogue_default_and_never_blocks() {
    // The two defaults, read off a rule rather than off the table. A rule that
    // says nothing about what it counts cannot fail the gate however often it
    // fires, and a rule that says nothing about its normaliser is divided by
    // what its family is divided by.
    struct SaysNothing;
    impl Rule for SaysNothing {
        fn id(&self) -> RuleId {
            RuleId("KI-XING-001")
        }
        fn tier(&self) -> Tier {
            Tier::Two
        }
        fn weight(&self) -> Penalty {
            Penalty::points(1)
        }
        fn examine(&self, _drawing: &Drawing<'_>, found: &mut Findings<'_>) {
            found.record(Point::default(), Vec::new(), "wrong".to_owned());
        }
    }

    assert_eq!(SaysNothing.normaliser(), Normaliser::PerWire);
    assert_eq!(SaysNothing.saturation(), Saturation::NEVER);
    assert_eq!(SaysNothing.saturation().counts(), Counted::Nothing);
    assert!(
        !SaysNothing
            .saturation()
            .is_reached(1_000, Density::of_counts(1, 1)),
        "a rule that counts nothing never saturates"
    );
}

/// One finding of the saturating rule, built without running the rule.
///
/// The hand-built instrument. It reaches drawing sizes a probe would take
/// minutes to write, and it is checked against the probe at the two sizes both
/// can reach.
fn slant_finding() -> kicli::lint::Finding {
    kicli::lint::Finding {
        rule: SATURATING_SLANT.id(),
        tier: SATURATING_SLANT.tier(),
        severity: SATURATING_SLANT.severity(),
        sheet: kicli::model::SheetPath(String::new()),
        pos: Point::default(),
        objects: Vec::new(),
        message: String::new(),
        fix: None,
        penalty: SATURATING_SLANT.weight(),
        normaliser: SATURATING_SLANT.normaliser(),
        saturation: SATURATING_SLANT.saturation(),
    }
}

#[test]
fn a_sheet_that_is_wrong_all_over_keeps_its_score_and_stops_passing() {
    // The measurement this mechanism was built to answer. A normalised rule
    // that fires on every object it counts costs the same at ten objects as at
    // ten thousand, so its score does not fall as the drawing grows. The score
    // is unchanged by anything here. The verdict is not.
    for wires in [10_u32, 200, 10_000] {
        let findings = vec![slant_finding(); wires as usize];
        let density = Density::of_counts(0, wires);
        let all_wrong = Report::of(&findings, density);

        assert_eq!(
            all_wrong.score().raw(),
            RawPenalty::billionths(10_000_000_000),
            "{wires} wires: the ceiling does not move"
        );
        assert_eq!(all_wrong.score().score(), 67, "{wires} wires");
        assert!(
            !all_wrong.gate().passes(),
            "{wires} wires: {}",
            all_wrong.text()
        );

        // A drawing of the same size with one slanted wire in ten is under the
        // share, so growing a bad drawing is what fails, not growing a drawing.
        let few = vec![slant_finding(); (wires / 10).max(1) as usize];
        assert!(
            Report::of(&few, density).gate().passes(),
            "{wires} wires, a tenth of them slanted"
        );
    }
}

#[test]
fn the_hand_built_findings_agree_with_the_rule_run_over_a_drawing() {
    // The control on the instrument above. Two sizes a probe can reach are
    // measured both ways, so the hand-built findings are not free to describe
    // a drawing the rule would never produce.
    for wires in [10_u32, 200] {
        let path = slanted_sheet(&format!("agree-{wires}"), wires, wires);
        let (drawn, density) = report(&path, vec![&SATURATING_SLANT]);
        assert_eq!(density.wires(), wires);

        let built = Report::of(
            &vec![slant_finding(); wires as usize],
            Density::of_counts(0, wires),
        );
        assert_eq!(drawn.score().raw(), built.score().raw(), "{wires} wires");
        assert_eq!(drawn.text(), built.text(), "{wires} wires");
    }
}
