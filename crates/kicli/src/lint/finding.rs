//! What a rule reports, and the order findings are written in.
//!
//! A finding names the rule, where the drawing is wrong, and which objects are
//! involved. It carries a suggested command as text. It never carries a way to
//! apply that command, because scoring reads and does not write.
//!
//! Every coordinate here is an [`Iu`](crate::geometry::Iu), so detection stays
//! integer arithmetic. Millimetres are a presentation unit at the command
//! boundary and appear nowhere in this module.

use std::cmp::Ordering;

use crate::geometry::Point;
use crate::model::items::{SheetPath, Uuid};

/// A rule's code, such as `KI-FLOW-001`.
///
/// The code is the rule's name in every output kicli writes. It is a compile
/// time constant, because a rule that could rename itself at run time would
/// make two reports of one drawing disagree.
///
/// # Examples
///
/// ```
/// use kicli::lint::RuleId;
/// assert!(RuleId("KI-FLOW-001").is_well_formed());
/// assert!(!RuleId("FLOW-1").is_well_formed());
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RuleId(pub &'static str);

impl RuleId {
    /// Does the code have the shape every published code has?
    ///
    /// The shape is `KI-`, a family of three to six capital letters, a hyphen,
    /// and three digits. It is derived from the published catalogue rather than
    /// chosen: every code there, from `KI-JCT-001` to `KI-GRID-001`, fits it.
    #[must_use]
    pub fn is_well_formed(self) -> bool {
        let Some(rest) = self.0.strip_prefix("KI-") else {
            return false;
        };
        let Some((family, number)) = rest.split_once('-') else {
            return false;
        };
        let family_fits = (3..=6).contains(&family.len())
            && family.chars().all(|letter| letter.is_ascii_uppercase());
        let number_fits = number.len() == 3 && number.chars().all(|digit| digit.is_ascii_digit());
        family_fits && number_fits
    }

    /// The family part of the code, such as `FLOW`.
    ///
    /// Returns the whole code when the code has no family, so a malformed code
    /// still groups with itself rather than with everything else.
    #[must_use]
    pub fn family(self) -> &'static str {
        self.0
            .strip_prefix("KI-")
            .and_then(|rest| rest.split_once('-'))
            .map_or(self.0, |(family, _)| family)
    }
}

impl std::fmt::Display for RuleId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}

/// How much a rule matters.
///
/// There are two tiers. Tier 1 blocks the gate and never moves the score. Tier
/// 2 moves the score and never blocks the gate. A third tier was cut from
/// scoring, so it is not a value this type can hold.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Tier {
    /// Blocking. The drawing fails the gate whatever it scores.
    One,
    /// Scored. The drawing loses points and still passes the gate.
    Two,
}

impl Tier {
    /// The number a report writes for this tier.
    ///
    /// # Examples
    ///
    /// ```
    /// use kicli::lint::Tier;
    /// assert_eq!(Tier::One.number(), 1);
    /// ```
    #[must_use]
    pub fn number(self) -> u8 {
        match self {
            Self::One => 1,
            Self::Two => 2,
        }
    }
}

/// How loudly a finding is reported.
///
/// The value is presentation. It never decides the gate, which reads the tier.
/// A project may lower a rule's severity without changing what the rule does.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Severity {
    /// The default of a Tier 1 rule.
    Error,
    /// The default of a Tier 2 rule.
    Warning,
}

impl Severity {
    /// The word a report writes for this severity.
    ///
    /// # Examples
    ///
    /// ```
    /// use kicli::lint::Severity;
    /// assert_eq!(Severity::Warning.word(), "warning");
    /// ```
    #[must_use]
    pub fn word(self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warning => "warning",
        }
    }
}

/// A penalty, in thousandths of a point.
///
/// The value is an integer because the score must be reproducible. A penalty
/// written as `3.0` is stored as `3_000`. Only the score's final exponential
/// uses floating point, and that step lives outside this module.
///
/// The type is unsigned, so a rule cannot award points for drawing well.
///
/// # Examples
///
/// ```
/// use kicli::lint::Penalty;
/// assert_eq!(Penalty::points(3).text(), "3.0");
/// assert_eq!(Penalty::thousandths(1_250).text(), "1.25");
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Penalty(u32);

impl Penalty {
    /// No penalty at all.
    pub const ZERO: Self = Self(0);

    /// A whole number of points.
    #[must_use]
    pub const fn points(points: u16) -> Self {
        Self(points as u32 * 1_000)
    }

    /// A penalty given directly in thousandths of a point.
    #[must_use]
    pub const fn thousandths(thousandths: u32) -> Self {
        Self(thousandths)
    }

    /// The value in thousandths of a point.
    #[must_use]
    pub const fn in_thousandths(self) -> u32 {
        self.0
    }

    /// The value as a report writes it.
    ///
    /// The text keeps at least one decimal place and drops trailing zeros
    /// beyond that. The conversion is integer division, so it rounds the same
    /// way on every machine.
    #[must_use]
    pub fn text(self) -> String {
        let whole = self.0 / 1_000;
        let mut fraction = format!("{:03}", self.0 % 1_000);
        while fraction.len() > 1 && fraction.ends_with('0') {
            fraction.pop();
        }
        format!("{whole}.{fraction}")
    }
}

/// One thing a rule found wrong with a drawing.
///
/// The fields are the published finding record. `fix` is a suggested command
/// and nothing more: kicli never mutates while it scores, so no part of this
/// type can apply it.
///
/// `penalty` is the rule's weight for one occurrence, before the density
/// normaliser is applied. The scorer applies the normaliser; the rule does not
/// know how many symbols the sheet holds.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Finding {
    /// Which rule found it.
    pub rule: RuleId,
    /// Which tier that rule is in.
    pub tier: Tier,
    /// How loudly to report it.
    pub severity: Severity,
    /// The sheet placement the drawing is on.
    pub sheet: SheetPath,
    /// Where on the sheet to look.
    pub pos: Point,
    /// The objects involved, in the order the rule names them.
    pub objects: Vec<Uuid>,
    /// What is wrong, in one sentence.
    pub message: String,
    /// A command that would put it right, when one exists.
    pub fix: Option<String>,
    /// The weight of one occurrence, before normalisation.
    pub penalty: Penalty,
}

impl Finding {
    /// The key findings are sorted by.
    ///
    /// The published order is `(rule, sheet, x, y, uuid)`. The fifth term here
    /// is the whole object list rather than one identifier, because a finding
    /// may name several objects. Two findings that agreed on the first four
    /// terms and on their first object would otherwise have no order at all.
    ///
    /// The list is compared in the order the rule wrote it, so a rule that
    /// names its objects in a different order on two runs is a bug in that
    /// rule. `cargo test --test lint_findings_sort_by_their_key` measures the
    /// order this key produces.
    #[must_use]
    pub fn key(&self) -> (RuleId, &SheetPath, Point, &[Uuid]) {
        (self.rule, &self.sheet, self.pos, &self.objects)
    }
}

impl Ord for Finding {
    /// Order by the published key, then by everything else.
    ///
    /// The tail after the key exists so that the order is total and agrees with
    /// equality. Two findings that share the key are the same finding, so the
    /// tail is never expected to decide anything. `lint_findings_sort_by_their_key`
    /// asserts that no two findings of one drawing share a key, which is what
    /// keeps that expectation honest.
    fn cmp(&self, other: &Self) -> Ordering {
        self.key().cmp(&other.key()).then_with(|| {
            (
                self.tier,
                self.severity,
                &self.message,
                &self.fix,
                self.penalty,
            )
                .cmp(&(
                    other.tier,
                    other.severity,
                    &other.message,
                    &other.fix,
                    other.penalty,
                ))
        })
    }
}

impl PartialOrd for Finding {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Put findings in the order a report writes them.
///
/// Sorting happens once, on the way out of the engine. Nothing downstream may
/// re-order, because the published order is part of the output contract.
pub fn sort(findings: &mut [Finding]) {
    findings.sort();
}

#[cfg(test)]
mod tests {
    use super::{Penalty, RuleId, Severity, Tier};

    #[test]
    fn a_published_code_is_well_formed() {
        // Every family length the catalogue uses.
        for code in ["KI-JCT-001", "KI-FLOW-001", "KI-GRID-001", "KI-CONN-001"] {
            assert!(RuleId(code).is_well_formed(), "{code}");
        }
    }

    #[test]
    fn a_code_of_the_wrong_shape_is_refused() {
        for code in [
            "KI-FLOW-01",
            "KI-flow-001",
            "KI--001",
            "XX-FLOW-001",
            "KI-FLOW-001A",
            "KI-VERYLONG-001",
            "KI-FLOW",
            "",
        ] {
            assert!(!RuleId(code).is_well_formed(), "{code}");
        }
    }

    #[test]
    fn a_code_names_its_family() {
        assert_eq!(RuleId("KI-FLOW-001").family(), "FLOW");
        assert_eq!(RuleId("nonsense").family(), "nonsense");
    }

    #[test]
    fn a_penalty_writes_the_shortest_exact_text() {
        assert_eq!(Penalty::ZERO.text(), "0.0");
        assert_eq!(Penalty::points(3).text(), "3.0");
        assert_eq!(Penalty::thousandths(1_500).text(), "1.5");
        assert_eq!(Penalty::thousandths(1_250).text(), "1.25");
        assert_eq!(Penalty::thousandths(1_005).text(), "1.005");
        assert_eq!(Penalty::thousandths(12_345).text(), "12.345");
    }

    #[test]
    fn a_tier_and_a_severity_write_what_the_record_shows() {
        assert_eq!(Tier::One.number(), 1);
        assert_eq!(Tier::Two.number(), 2);
        assert_eq!(Severity::Error.word(), "error");
        assert_eq!(Severity::Warning.word(), "warning");
    }
}
