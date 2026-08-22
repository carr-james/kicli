//! A crowded sheet is not punished for being crowded.
//!
//! A sheet with four symbols and one crossing is drawn worse than a sheet with
//! two hundred symbols and one crossing. The second sheet has more of
//! everything, so a rule that counted occurrences would report the two sheets
//! as equal and the crowded one as worse the moment it grew. The normalisers
//! exist for that sentence, and this is that sentence as a check.
//!
//! **Both drawings come from one generator with one parameter.** Everything
//! else is written by the same lines of code: the same symbol definition, the
//! same wire per symbol, the same two crossing wires at the same place, and the
//! same findings. Two fixtures that differed in more than the count would make
//! the comparison mean nothing, so the difference is engineered rather than
//! hoped for.
//!
//! The rules that report crossings and fields are not written yet, so the
//! findings here are made directly. What is measured from the file is the
//! density, which is the half of the calculation this check is about.

use std::path::{Path, PathBuf};

use kicli::geometry::Point;
use kicli::lint::score::{Density, Normaliser, RawPenalty, SheetScore, project_score};
use kicli::lint::{Drawing, Finding, Penalty, RuleId, Severity, Tier};
use kicli::model::items::SheetPath;
use kicli::model::{Hierarchy, LoadedFile};
use kicli_probe::{Probe, pin, power, rectangle, symbol};

/// How many symbols the sparse sheet holds.
const FEW: u32 = 4;

/// How many symbols the crowded sheet holds.
const MANY: u32 = 200;

/// The weight every finding in this file carries, in whole points.
const WEIGHT: u16 = 3;

/// Where the drawings this binary builds are written.
fn scratch() -> PathBuf {
    Path::new(env!("CARGO_TARGET_TMPDIR")).join("lint-density")
}

/// The pin numbers the specimen symbol draws.
const PINS: [&str; 2] = ["1", "2"];

/// A symbol with a square body and a pin on two of its edges.
fn part() -> String {
    symbol(
        "PART",
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

/// One sheet holding a given number of symbols, and nothing else that varies.
///
/// Each symbol gets one wire beside it, so a sheet with more symbols has more
/// wires, as a real drawing does. Two further wires cross each other in the
/// corner of every sheet this generator writes, at the same place and with the
/// same length, whatever the count is.
fn sheet_of(name: &str, symbols: u32) -> PathBuf {
    let mut probe = Probe::new(name, scratch());
    probe.define(part());
    for index in 0..symbols {
        let column = index % 20;
        let row = index / 20;
        let x = format!("{}", 20 + column * 8);
        let y = format!("{}", 20 + row * 8);
        probe.place("PART", &format!("U{}", index + 1), (&x, &y), &PINS);
        let wire_y = format!("{}", 24 + row * 8);
        probe.wire(
            (&x, &wire_y),
            (&format!("{}", 20 + column * 8 + 4), &wire_y),
        );
    }
    // The crossing every sheet holds: two wires over one another, no junction.
    probe.wire(("5", "6"), ("9", "6"));
    probe.wire(("7", "4"), ("7", "8"));
    // One bundle, on every sheet. A bundle is not a wire, and the wire counts
    // asserted below are the check that it is not counted as one.
    probe.bus(("5", "12"), ("9", "12"));
    probe.write()
}

/// The density of the root sheet of a written drawing.
fn density_of(path: &Path) -> Density {
    let hierarchy = Hierarchy::load(path).expect("the drawing loads");
    let placement = hierarchy
        .placements
        .first()
        .expect("the drawing has a sheet");
    let file: &LoadedFile = &hierarchy.files[placement.file];
    Density::of(&Drawing::read(&file.doc, &file.schematic, &placement.path))
}

/// One finding of a named rule, weighing the same as every other here.
fn finding(rule: &'static str, tier: Tier) -> Finding {
    Finding {
        rule: RuleId(rule),
        tier,
        severity: Severity::Warning,
        sheet: SheetPath(String::new()),
        pos: Point::default(),
        objects: Vec::new(),
        message: "the drawing is wrong here".to_owned(),
        fix: None,
        penalty: Penalty::points(WEIGHT),
    }
}

/// The two densities this file compares, sparse first.
///
/// Each check writes its own pair of drawings. One pair shared between them
/// would be written and read by several threads at once, because the tests of
/// one binary run in parallel, and a half written file reads as a broken one.
fn the_two_sheets(check: &str) -> (Density, Density) {
    let sparse = density_of(&sheet_of(&format!("density-few-{check}"), FEW));
    let crowded = density_of(&sheet_of(&format!("density-many-{check}"), MANY));

    // The control on the generator. The two sheets differ in the count they
    // were given, and the wire count follows it, as a real drawing's does.
    assert_eq!(sparse.symbols(), FEW);
    assert_eq!(crowded.symbols(), MANY);
    // One wire for each symbol and the two that cross. The bundle each sheet
    // also holds is not among them.
    assert_eq!(sparse.wires(), FEW + 2);
    assert_eq!(crowded.wires(), MANY + 2);

    (sparse, crowded)
}

/// A sheet holding parts and power symbols in equal number.
fn sheet_with_power(name: &str, parts: u32) -> PathBuf {
    let mut probe = Probe::new(name, scratch());
    probe.define(part());
    probe.define(power("VCC"));
    for index in 0..parts {
        let x = format!("{}", 20 + index * 8);
        probe.place("PART", &format!("U{}", index + 1), (&x, "20"), &PINS);
        probe.place("VCC", &format!("#PWR{:02}", index + 1), (&x, "30"), &["1"]);
    }
    probe.write()
}

#[test]
fn a_power_symbol_does_not_make_a_sheet_look_crowded() {
    // A power symbol is a net name drawn in the shape of a part. Counting it
    // would divide every field finding by the number of ground symbols, so a
    // sheet that labels its rails well would be penalised less for its fields.
    let parts = 6;
    let density = density_of(&sheet_with_power("density-power", parts));
    assert_eq!(
        density.symbols(),
        parts,
        "the power symbols were not counted"
    );

    // The control that says the power symbols really are on the sheet: a
    // count that included them would read twice as many.
    let hierarchy = Hierarchy::load(&sheet_with_power("density-power-control", parts))
        .expect("the drawing loads");
    let placement = hierarchy
        .placements
        .first()
        .expect("the drawing has a sheet");
    let file: &LoadedFile = &hierarchy.files[placement.file];
    let placed = file.schematic.symbols().count();
    let carrying_power = file.schematic.symbols().filter(|s| s.is_power()).count();
    assert_eq!(placed, (parts * 2) as usize, "both kinds were placed");
    assert_eq!(carrying_power, parts as usize, "half of them carry power");
}

#[test]
fn a_sparse_sheet_with_one_crossing_scores_worse_than_a_crowded_one() {
    let (sparse, crowded) = the_two_sheets("crossing");
    let crossing = [finding("KI-XING-001", Tier::Two)];
    assert_eq!(crossing.len(), 1, "one crossing on each sheet");

    let small = SheetScore::of(&crossing, sparse);
    let large = SheetScore::of(&crossing, crowded);

    assert!(
        small.score() < large.score(),
        "the sparse sheet scored {} and the crowded one {}",
        small.score(),
        large.score()
    );
    // The anti-vacuity arm. Two sheets that scored the same would satisfy no
    // inequality, and two that differed for another reason would satisfy this
    // one by accident.
    assert_ne!(small.score(), large.score());
    assert_eq!((small.score(), large.score()), (89, 99));
}

#[test]
fn a_sparse_sheet_with_one_stray_field_scores_worse_than_a_crowded_one() {
    // The other normaliser, on the same pair of sheets. This one divides by
    // the symbol count rather than the wire count.
    let (sparse, crowded) = the_two_sheets("field");
    let stray = [finding("KI-FLD-001", Tier::Two)];
    assert_eq!(Normaliser::of(stray[0].rule), Normaliser::PerObject);

    let small = SheetScore::of(&stray, sparse);
    let large = SheetScore::of(&stray, crowded);
    assert!(small.score() < large.score());
    assert_eq!((small.score(), large.score()), (89, 99));
}

#[test]
fn with_no_normaliser_the_two_sheets_score_the_same() {
    // The control that says the difference above came from the normaliser and
    // from nothing else about the two drawings. The same weight, on the same
    // two sheets, under a rule that divides by nothing.
    let (sparse, crowded) = the_two_sheets("flow");
    let flow = [finding("KI-FLOW-001", Tier::Two)];
    assert_eq!(Normaliser::of(flow[0].rule), Normaliser::PerSheet);

    let small = SheetScore::of(&flow, sparse);
    let large = SheetScore::of(&flow, crowded);
    assert_eq!(small.raw(), large.raw());
    assert_eq!(small.score(), large.score());
    assert_eq!(small.raw(), RawPenalty::billionths(3_000_000_000));
}

#[test]
fn a_blocking_finding_leaves_both_sheets_untouched() {
    // The blocking tier is skipped in the sum, not by the caller. This hands
    // the scorer a heavy blocking finding directly, which a caller that
    // filtered first would never do.
    let (sparse, crowded) = the_two_sheets("blocking");
    let blocking = [finding("KI-GRID-001", Tier::One)];
    for density in [sparse, crowded] {
        let scored = SheetScore::of(&blocking, density);
        assert_eq!(scored.raw(), RawPenalty::ZERO);
        assert_eq!(scored.score(), 100);
    }
}

#[test]
fn a_project_leans_towards_the_sheet_that_holds_the_symbols() {
    let (sparse, crowded) = the_two_sheets("project");
    let crossing = [finding("KI-XING-001", Tier::Two)];
    let sheets = [
        SheetScore::of(&crossing, sparse),
        SheetScore::of(&crossing, crowded),
    ];
    assert_eq!((sheets[0].score(), sheets[1].score()), (89, 99));

    // Two hundred symbols against four, so the project sits at the crowded
    // sheet. The unweighted mean of the same two sheets is 94, which this is
    // not, and the two sheets scored together as one would be neither.
    assert_eq!(project_score(&sheets), 99);
    assert_ne!(project_score(&sheets), 94);
}
