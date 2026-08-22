//! Turning findings into a number, with integer arithmetic and no other kind.
//!
//! A sheet's score falls as its findings accumulate. Each finding contributes
//! its rule's weight, divided by a normaliser that reads how crowded the sheet
//! is. The sum is the raw penalty. The score is a decaying function of the raw
//! penalty, on a scale of zero to one hundred.
//!
//! # Why there is no floating point here
//!
//! Two runs over one file must produce the same number, on any machine,
//! forever. A floating point expression can round two ways on two targets, and
//! a score that differs in the last place would report a drawing as changed
//! when nothing changed. The decaying function is therefore evaluated in fixed
//! point, from a series of integer terms, and rounded once at the end.
//!
//! The normalisers make that harder than it looks, because each one is a
//! reciprocal. They are held as exact ratios and applied once per family of
//! rules, so the whole calculation is a sum of integers over one denominator.
//! `cargo test --test the_linter_holds_no_floating_point` is the enforcement.

use crate::lint::drawing::Drawing;
use crate::lint::finding::{Finding, RuleId, Tier};
use crate::model::items::{Item, LineKind};

/// How many objects a sheet holds, which is what a normaliser divides by.
///
/// The symbol count excludes power symbols. A power symbol is a net name in
/// the shape of a part, so counting it would make a well labelled sheet look
/// crowded.
///
/// # Examples
///
/// ```
/// use kicli::lint::score::Density;
/// let density = Density::of_counts(4, 3);
/// assert_eq!(density.symbols(), 4);
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Density {
    symbols: u32,
    wires: u32,
}

impl Density {
    /// Count the objects of one drawing.
    #[must_use]
    pub fn of(drawing: &Drawing<'_>) -> Self {
        let mut density = Self::default();
        for item in &drawing.schematic().items {
            match item {
                Item::Symbol(symbol) if !symbol.is_power() => density.symbols += 1,
                Item::Line(line) if matches!(line.kind, LineKind::Wire) => density.wires += 1,
                _ => {}
            }
        }
        density
    }

    /// A density from counts a caller already has.
    #[must_use]
    pub const fn of_counts(symbols: u32, wires: u32) -> Self {
        Self { symbols, wires }
    }

    /// How many symbols the sheet holds, power symbols excluded.
    #[must_use]
    pub const fn symbols(self) -> u32 {
        self.symbols
    }

    /// How many wire segments the sheet holds.
    #[must_use]
    pub const fn wires(self) -> u32 {
        self.wires
    }
}

/// The sheet a normaliser measures against.
///
/// A sheet of twenty symbols, or of ten wire segments, is the reference. A
/// sheet below either count is not normalised at all, so a small sheet keeps
/// the full weight of every finding on it.
const REFERENCE_SYMBOLS: u32 = 20;

/// The wire count a crowded sheet is measured against.
const REFERENCE_WIRES: u32 = 10;

/// What a rule's weight is divided by before it enters the raw penalty.
///
/// A crossing on a sheet of two hundred wires says less about the drawing than
/// a crossing on a sheet of four, so the count a rule reports is divided by how
/// much of that kind of object the sheet holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Normaliser {
    /// Divide by the symbol count. Field, symbol and text rules take this.
    PerObject,
    /// Divide by the wire count. Crossing and dogleg rules take this.
    PerWire,
    /// Divide by nothing. Flow, layout and documentation rules take this.
    PerSheet,
}

/// The rule families that are divided by the symbol count.
const PER_OBJECT_FAMILIES: [&str; 3] = ["FLD", "SYM", "TXT"];

/// The rule families that are divided by the wire count.
const PER_WIRE_FAMILIES: [&str; 2] = ["XING", "RTE"];

impl Normaliser {
    /// Which normaliser a rule's findings take.
    ///
    /// The families named in the published catalogue decide it. Every other
    /// family divides by nothing, which is the strict choice: an unlisted rule
    /// keeps its full weight rather than quietly losing some of it.
    ///
    /// # Examples
    ///
    /// ```
    /// use kicli::lint::{RuleId, score::Normaliser};
    /// assert_eq!(Normaliser::of(RuleId("KI-XING-001")), Normaliser::PerWire);
    /// assert_eq!(Normaliser::of(RuleId("KI-FLOW-001")), Normaliser::PerSheet);
    /// ```
    #[must_use]
    pub fn of(rule: RuleId) -> Self {
        let family = rule.family();
        if PER_OBJECT_FAMILIES.contains(&family) {
            Self::PerObject
        } else if PER_WIRE_FAMILIES.contains(&family) {
            Self::PerWire
        } else {
            Self::PerSheet
        }
    }

    /// The exact ratio this normaliser multiplies a weight by.
    ///
    /// The ratio is the reference count over the sheet's own count, and never
    /// more than one. It is returned as a pair rather than evaluated, because
    /// evaluating it here would need a fraction.
    #[must_use]
    pub const fn ratio(self, density: Density) -> (u32, u32) {
        match self {
            Self::PerObject => (
                REFERENCE_SYMBOLS,
                if density.symbols > REFERENCE_SYMBOLS {
                    density.symbols
                } else {
                    REFERENCE_SYMBOLS
                },
            ),
            Self::PerWire => (
                REFERENCE_WIRES,
                if density.wires > REFERENCE_WIRES {
                    density.wires
                } else {
                    REFERENCE_WIRES
                },
            ),
            Self::PerSheet => (1, 1),
        }
    }

    /// Every normaliser, in a fixed order.
    #[must_use]
    pub const fn every() -> [Self; 3] {
        [Self::PerObject, Self::PerWire, Self::PerSheet]
    }
}

/// How many billionths of a point there are in a thousandth of one.
const BILLIONTHS_IN_A_THOUSANDTH: u128 = 1_000_000;

/// How many billionths of a point there are in a point.
const BILLIONTHS_IN_A_POINT: u128 = 1_000_000_000;

/// The summed weight of a sheet's findings, in billionths of a point.
///
/// The unit is finer than a rule's own weight because a normaliser divides.
/// Dividing once for each family of rules, rather than once for each finding,
/// keeps the whole sum exact to within one billionth of a point.
///
/// # Examples
///
/// ```
/// use kicli::lint::score::RawPenalty;
/// assert_eq!(RawPenalty::ZERO.text(), "0.0");
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct RawPenalty(u128);

impl RawPenalty {
    /// No penalty at all, which scores one hundred.
    pub const ZERO: Self = Self(0);

    /// The raw penalty of one sheet's findings.
    ///
    /// Blocking findings are skipped **here**, in the sum, rather than by the
    /// caller. A caller that filtered would leave the next caller free not to,
    /// and a blocking finding must never move the score whoever asks.
    #[must_use]
    pub fn of(findings: &[Finding], density: Density) -> Self {
        let mut total = 0;
        for normaliser in Normaliser::every() {
            let weight = Self::weight_of(findings, normaliser);
            let (numerator, denominator) = normaliser.ratio(density);
            total += weight * BILLIONTHS_IN_A_THOUSANDTH * u128::from(numerator)
                / u128::from(denominator);
        }
        Self(total)
    }

    /// A raw penalty given directly in billionths of a point.
    #[must_use]
    pub const fn billionths(billionths: u128) -> Self {
        Self(billionths)
    }

    /// The value in billionths of a point.
    #[must_use]
    pub const fn in_billionths(self) -> u128 {
        self.0
    }

    /// The value as a report writes it.
    ///
    /// The text keeps at least one decimal place and drops trailing zeros
    /// beyond that. The conversion is integer division, so it rounds the same
    /// way on every machine.
    #[must_use]
    pub fn text(self) -> String {
        let whole = self.0 / BILLIONTHS_IN_A_POINT;
        let mut fraction = format!("{:09}", self.0 % BILLIONTHS_IN_A_POINT);
        while fraction.len() > 1 && fraction.ends_with('0') {
            fraction.pop();
        }
        format!("{whole}.{fraction}")
    }

    /// The summed weight, in thousandths of a point, of the scored findings
    /// that take one normaliser.
    fn weight_of(findings: &[Finding], normaliser: Normaliser) -> u128 {
        findings
            .iter()
            .filter(|finding| finding.tier == Tier::Two)
            .filter(|finding| Normaliser::of(finding.rule) == normaliser)
            .map(|finding| u128::from(finding.penalty.in_thousandths()))
            .sum()
    }
}

/// The raw penalty, in points, that divides the score by `e`.
///
/// **This is a starting point, not a measured value.** The constant is frozen
/// last, by requiring known good sheets to land near one hundred and heavily
/// degraded sheets to land near a third of that. Nothing in this module may be
/// tuned to make one drawing read nicely.
const DECAY_POINTS: u128 = 25;

/// The best a sheet can score.
const BEST: u128 = 100;

/// One, in the fixed point arithmetic the decay is evaluated in.
///
/// Seventeen digits leave the answer correct to far more places than a score of
/// zero to one hundred can show, and leave the series below clear of the range
/// of a `u128`.
const ONE: u128 = 100_000_000_000_000_000;

/// How many terms of the series are summed before it is certainly exhausted.
const TERMS: u128 = 64;

/// The raw penalty, in units of the decay constant, above which nothing is left.
///
/// Six decay constants leave a quarter of one point of score, which rounds to
/// nothing. The decay never reaches zero, so the cut is where the rounding
/// makes it unobservable rather than where the arithmetic ends.
const EXHAUSTED: u128 = 6;

/// One sheet's score, and the numbers it was made from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SheetScore {
    density: Density,
    raw: RawPenalty,
    score: u32,
}

impl SheetScore {
    /// Score one sheet's findings.
    #[must_use]
    pub fn of(findings: &[Finding], density: Density) -> Self {
        let raw = RawPenalty::of(findings, density);
        Self {
            density,
            raw,
            score: score_of(raw),
        }
    }

    /// Score a sheet from a raw penalty a caller already has.
    #[must_use]
    pub fn of_raw(raw: RawPenalty, density: Density) -> Self {
        Self {
            density,
            raw,
            score: score_of(raw),
        }
    }

    /// How crowded the sheet is.
    #[must_use]
    pub const fn density(self) -> Density {
        self.density
    }

    /// The summed weight of the sheet's scored findings.
    #[must_use]
    pub const fn raw(self) -> RawPenalty {
        self.raw
    }

    /// The sheet's score, from zero to one hundred.
    #[must_use]
    pub const fn score(self) -> u32 {
        self.score
    }
}

/// The score a raw penalty leaves, from zero to one hundred.
///
/// The value is `BEST` times a decay of the raw penalty, rounded to the nearest
/// whole number. The decay is evaluated in fixed point, so the answer is the
/// same on every machine.
///
/// # Examples
///
/// ```
/// use kicli::lint::score::{RawPenalty, score_of};
/// assert_eq!(score_of(RawPenalty::ZERO), 100);
/// ```
#[must_use]
pub fn score_of(raw: RawPenalty) -> u32 {
    let billionths = raw.in_billionths();
    if billionths >= EXHAUSTED * DECAY_POINTS * BILLIONTHS_IN_A_POINT {
        return 0;
    }
    let exponent = billionths * ONE / (DECAY_POINTS * BILLIONTHS_IN_A_POINT);
    let grown = grown_by(exponent);
    let scaled = BEST * ONE * ONE / grown;
    rounded(scaled, ONE)
}

/// The project's score: the symbol count weighted mean of its sheets.
///
/// A crowded sheet decides more of the answer than an empty one. A project of
/// sheets that hold no symbols at all is averaged evenly, because there is
/// nothing to weigh it by. A project of no sheets has nothing drawn wrong.
///
/// # Examples
///
/// ```
/// use kicli::lint::score::{Density, RawPenalty, SheetScore, project_score};
/// let small = SheetScore::of_raw(RawPenalty::billionths(22_900_000_000), Density::of_counts(10, 0));
/// let large = SheetScore::of_raw(RawPenalty::billionths(1_300_000_000), Density::of_counts(200, 0));
/// assert_eq!(small.score(), 40);
/// assert_eq!(large.score(), 95);
/// assert_eq!(project_score(&[small, large]), 92);
/// ```
#[must_use]
pub fn project_score(sheets: &[SheetScore]) -> u32 {
    if sheets.is_empty() {
        return u32::try_from(BEST).unwrap_or(u32::MAX);
    }
    let symbols = |sheet: &SheetScore| u128::from(sheet.density().symbols());
    let total: u128 = sheets.iter().map(symbols).sum();
    if total == 0 {
        let sum: u128 = sheets.iter().map(|sheet| u128::from(sheet.score())).sum();
        return rounded(sum, sheets.len() as u128);
    }
    let sum: u128 = sheets
        .iter()
        .map(|sheet| u128::from(sheet.score()) * symbols(sheet))
        .sum();
    rounded(sum, total)
}

/// A quotient rounded to the nearest whole number, halves away from zero.
fn rounded(numerator: u128, denominator: u128) -> u32 {
    let whole = (numerator * 2 + denominator) / (denominator * 2);
    u32::try_from(whole).unwrap_or(u32::MAX)
}

/// The growth factor of an exponent, in fixed point.
///
/// The exponent arrives scaled by `ONE` and no larger than `EXHAUSTED` times
/// it. The answer is the sum of the exponential series, whose terms are all
/// positive: an alternating series would lose most of its digits to
/// cancellation, so the decay is taken as the reciprocal of this instead.
fn grown_by(exponent: u128) -> u128 {
    let mut term = ONE;
    let mut total = ONE;
    for step in 1..=TERMS {
        term = term * exponent / (ONE * step);
        if term == 0 {
            break;
        }
        total += term;
    }
    total
}

#[cfg(test)]
mod tests {
    use super::{Density, Normaliser, RawPenalty, SheetScore, project_score, score_of};
    use crate::lint::finding::{Finding, Penalty, RuleId, Severity, Tier};
    use crate::model::items::SheetPath;

    /// One finding of a named rule, with a weight in whole points.
    fn finding(rule: &'static str, tier: Tier, points: u16) -> Finding {
        Finding {
            rule: RuleId(rule),
            tier,
            severity: Severity::Warning,
            sheet: SheetPath(String::new()),
            pos: crate::geometry::Point::default(),
            objects: Vec::new(),
            message: String::new(),
            fix: None,
            penalty: Penalty::points(points),
        }
    }

    #[test]
    fn a_clean_sheet_scores_the_best_there_is() {
        assert_eq!(score_of(RawPenalty::ZERO), 100);
    }

    #[test]
    fn the_decay_matches_the_published_function() {
        // Values of 100*exp(-raw/25), rounded. A whole decay constant of
        // penalty leaves 100/e, which is 36.788.
        let published: [(u32, u32); 6] = [(1, 96), (5, 82), (25, 37), (50, 14), (100, 2), (149, 0)];
        for (points, expected) in published {
            let raw = RawPenalty::billionths(u128::from(points) * 1_000_000_000);
            assert_eq!(score_of(raw), expected, "{points} points");
        }
    }

    #[test]
    fn the_decay_never_rises() {
        let mut last = 101;
        for points in 0_u32..200 {
            let raw = RawPenalty::billionths(u128::from(points) * 1_000_000_000);
            let score = score_of(raw);
            assert!(score <= last, "{points} points scored {score} after {last}");
            last = score;
        }
        assert_eq!(last, 0, "a heavy enough penalty leaves nothing");
    }

    #[test]
    fn a_blocking_finding_never_moves_the_score() {
        let sheet = Density::of_counts(4, 4);
        let blocking = [finding("KI-GRID-001", Tier::One, 50)];
        assert_eq!(RawPenalty::of(&blocking, sheet), RawPenalty::ZERO);
        assert_eq!(SheetScore::of(&blocking, sheet).score(), 100);
    }

    #[test]
    fn a_crossing_weighs_less_on_a_crowded_sheet() {
        let crossing = [finding("KI-XING-001", Tier::Two, 3)];
        let sparse = SheetScore::of(&crossing, Density::of_counts(4, 4));
        let crowded = SheetScore::of(&crossing, Density::of_counts(200, 200));
        assert!(sparse.score() < crowded.score());
    }

    #[test]
    fn a_sheet_below_the_reference_is_not_normalised() {
        let crossing = [finding("KI-XING-001", Tier::Two, 3)];
        let tiny = RawPenalty::of(&crossing, Density::of_counts(1, 1));
        let reference = RawPenalty::of(&crossing, Density::of_counts(20, 10));
        assert_eq!(tiny, reference);
        assert_eq!(tiny, RawPenalty::billionths(3_000_000_000));
    }

    #[test]
    fn an_unlisted_family_keeps_its_whole_weight() {
        // The strict default. A rule the catalogue does not place is divided
        // by nothing, on every density.
        let unlisted = [finding("KI-JCT-001", Tier::Two, 3)];
        for density in [Density::of_counts(4, 4), Density::of_counts(200, 200)] {
            assert_eq!(
                RawPenalty::of(&unlisted, density),
                RawPenalty::billionths(3_000_000_000)
            );
        }
        assert_eq!(Normaliser::of(RuleId("KI-JCT-001")), Normaliser::PerSheet);
    }

    #[test]
    fn each_family_takes_the_normaliser_the_catalogue_gives_it() {
        for code in ["KI-FLD-001", "KI-SYM-001", "KI-TXT-002"] {
            assert_eq!(
                Normaliser::of(RuleId(code)),
                Normaliser::PerObject,
                "{code}"
            );
        }
        for code in ["KI-XING-001", "KI-RTE-001"] {
            assert_eq!(Normaliser::of(RuleId(code)), Normaliser::PerWire, "{code}");
        }
        for code in ["KI-FLOW-001", "KI-LAY-001", "KI-DOC-001"] {
            assert_eq!(Normaliser::of(RuleId(code)), Normaliser::PerSheet, "{code}");
        }
    }

    #[test]
    fn a_normalised_rule_cannot_take_off_more_than_its_ceiling() {
        // The normaliser bounds what one rule can cost, whatever the sheet
        // holds. A sheet where every wire crosses another loses the same at
        // ten thousand wires as at ten, so growing a bad drawing does not make
        // it score worse. The ceiling is the reference count times the weight.
        for wires in [10_u32, 200, 10_000] {
            let every = vec![finding("KI-XING-001", Tier::Two, 1); wires as usize];
            let scored = SheetScore::of(&every, Density::of_counts(wires, wires));
            assert_eq!(
                scored.raw(),
                RawPenalty::billionths(10_000_000_000),
                "{wires}"
            );
            assert_eq!(scored.score(), 67, "{wires} wires all crossing");
        }
        for symbols in [20_u32, 200, 10_000] {
            let every = vec![finding("KI-FLD-001", Tier::Two, 1); symbols as usize];
            let scored = SheetScore::of(&every, Density::of_counts(symbols, 0));
            assert_eq!(
                scored.raw(),
                RawPenalty::billionths(20_000_000_000),
                "{symbols}"
            );
            assert_eq!(scored.score(), 45, "{symbols} symbols all wrong");
        }
    }

    #[test]
    fn the_raw_penalty_writes_the_shortest_exact_text() {
        assert_eq!(RawPenalty::ZERO.text(), "0.0");
        assert_eq!(RawPenalty::billionths(3_000_000_000).text(), "3.0");
        assert_eq!(RawPenalty::billionths(1_500_000_000).text(), "1.5");
        assert_eq!(RawPenalty::billionths(300_000_000).text(), "0.3");
        assert_eq!(RawPenalty::billionths(1).text(), "0.000000001");
    }

    #[test]
    fn a_project_weighs_each_sheet_by_its_symbols() {
        let small = SheetScore::of_raw(
            RawPenalty::billionths(22_900_000_000),
            Density::of_counts(10, 0),
        );
        let large = SheetScore::of_raw(
            RawPenalty::billionths(1_300_000_000),
            Density::of_counts(200, 0),
        );
        assert_eq!((small.score(), large.score()), (40, 95));
        assert_eq!(project_score(&[small, large]), 92);
        // The unweighted mean of the same two sheets, which this is not.
        assert_ne!(project_score(&[small, large]), 68);
    }

    #[test]
    fn a_project_of_empty_sheets_is_averaged_evenly() {
        let one = SheetScore::of_raw(RawPenalty::billionths(1_300_000_000), Density::default());
        let two = SheetScore::of_raw(RawPenalty::billionths(22_900_000_000), Density::default());
        assert_eq!(project_score(&[one, two]), 68);
        assert_eq!(project_score(&[]), 100);
    }
}
