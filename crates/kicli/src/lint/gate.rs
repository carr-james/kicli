//! Whether a drawing passes the gate, and why it does not.
//!
//! The gate and the score answer two different questions about one drawing.
//! The score says how well it is drawn. The gate says whether it may be built.
//! A drawing may score well and still fail, so the two are read together and
//! never one instead of the other.
//!
//! # The two ways a drawing fails
//!
//! A **blocking** finding fails the gate on its first occurrence, and it never
//! moves the score. That is the published rule for the blocking tier.
//!
//! A **saturating** rule fails the gate because it covered enough of the sheet.
//! A scored rule is normalised, so what it can cost is capped whatever the
//! drawing holds: a sheet on which every wire crosses another loses the same at
//! ten wires as at ten thousand. The cap is right for the number and wrong for
//! the verdict, because a sheet where every wire crosses another cannot be
//! read. The verdict is therefore taken separately, from how much of the sheet
//! the rule covered rather than from what the rule cost.
//!
//! Nothing here moves a weight and nothing here changes the formula. A rule
//! declares what it counts and how much of it is too much; this module reads
//! that declaration and answers pass or fail.

use std::fmt::Write as _;

use crate::lint::finding::{Finding, RuleId, Tier};
use crate::lint::score::{Density, SheetScore};

/// What a rule counts, which is what its saturation is measured against.
///
/// The denominator is the rule's own knowledge. A rule that reports one finding
/// for each symbol counts symbols; a rule that reports one for each crossing
/// counts wires; a rule that reports at most once about the whole sheet counts
/// nothing, because there is no share of a sheet to cover.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Counted {
    /// The non-power symbols of the sheet.
    Symbols,
    /// The wire segments of the sheet.
    Wires,
    /// Nothing countable. A rule that counts nothing never saturates.
    Nothing,
}

impl Counted {
    /// How many of these the sheet holds.
    ///
    /// # Examples
    ///
    /// ```
    /// use kicli::lint::gate::Counted;
    /// use kicli::lint::score::Density;
    /// assert_eq!(Counted::Wires.total(Density::of_counts(4, 9)), 9);
    /// ```
    #[must_use]
    pub const fn total(self, density: Density) -> u32 {
        match self {
            Self::Symbols => density.symbols(),
            Self::Wires => density.wires(),
            Self::Nothing => 0,
        }
    }

    /// The word a report writes for this denominator.
    ///
    /// # Examples
    ///
    /// ```
    /// use kicli::lint::gate::Counted;
    /// assert_eq!(Counted::Symbols.word(), "symbols");
    /// ```
    #[must_use]
    pub const fn word(self) -> &'static str {
        match self {
            Self::Symbols => "symbols",
            Self::Wires => "wires",
            Self::Nothing => "nothing",
        }
    }
}

/// The share of what a rule counts that makes the drawing fail the gate.
///
/// **The default share is a starting point, not a measured value.** Half is the
/// coarsest line that holds without a measurement: a normalised rule that has
/// fired on half of what it counts is already within a factor of two of a
/// ceiling it can never exceed, and the ceiling does not fall as the drawing
/// grows. The value is expected to move once real drawings are scored, and
/// moving it is one edit here.
///
/// # Examples
///
/// ```
/// use kicli::lint::gate::{Counted, Saturation};
/// use kicli::lint::score::Density;
///
/// let crossings = Saturation::of(Counted::Wires);
/// let sheet = Density::of_counts(0, 10);
/// assert!(!crossings.is_reached(4, sheet));
/// assert!(crossings.is_reached(5, sheet));
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Saturation {
    counted: Counted,
    numerator: u32,
    denominator: u32,
}

/// The share of its objects a rule covers before the drawing fails the gate.
///
/// This is the starting point named in [`Saturation`]'s documentation. It is
/// one number and one place, so a measurement moves it here and nowhere else.
const SHARE: (u32, u32) = (1, 2);

impl Saturation {
    /// A rule that never saturates, whatever it reports.
    pub const NEVER: Self = Self {
        counted: Counted::Nothing,
        numerator: SHARE.0,
        denominator: SHARE.1,
    };

    /// A rule that saturates at the standard share of what it counts.
    #[must_use]
    pub const fn of(counted: Counted) -> Self {
        Self {
            counted,
            numerator: SHARE.0,
            denominator: SHARE.1,
        }
    }

    /// A rule that saturates at a share of its own.
    ///
    /// A denominator of zero says nothing, so it is read as one.
    #[must_use]
    pub const fn share_of(counted: Counted, numerator: u32, denominator: u32) -> Self {
        Self {
            counted,
            numerator,
            denominator: if denominator == 0 { 1 } else { denominator },
        }
    }

    /// What the rule counts.
    #[must_use]
    pub const fn counts(self) -> Counted {
        self.counted
    }

    /// The share, as a numerator over a denominator.
    #[must_use]
    pub const fn share(self) -> (u32, u32) {
        (self.numerator, self.denominator)
    }

    /// Has a rule that reported this many times covered enough of the sheet?
    ///
    /// The comparison is a cross multiplication, so it is exact and holds no
    /// division. A rule that reported nothing never saturates, and neither does
    /// a rule on a sheet that holds none of what the rule counts.
    #[must_use]
    pub fn is_reached(self, count: u32, density: Density) -> bool {
        let total = self.counted.total(density);
        if count == 0 || total == 0 {
            return false;
        }
        u64::from(count) * u64::from(self.denominator)
            >= u64::from(self.numerator) * u64::from(total)
    }
}

/// Why one rule fails a drawing's gate.
///
/// The two reasons are kept apart because they send a reader to different
/// places. A blocking finding names one thing to put right. A saturating rule
/// says the drawing is wrong all over, and the count against the denominator is
/// the whole explanation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Blocker {
    /// The rule blocks, so one finding is enough.
    Blocking {
        /// The rule that reported.
        rule: RuleId,
        /// How many times it reported.
        count: u32,
    },
    /// The rule is scored, and it covered enough of the sheet to block anyway.
    Saturated {
        /// The rule that reported.
        rule: RuleId,
        /// How many times it reported.
        count: u32,
        /// How many of the objects it counts the sheet holds.
        total: u32,
        /// What the rule counts.
        counted: Counted,
        /// The share the rule declared, as a numerator over a denominator.
        share: (u32, u32),
    },
}

impl Blocker {
    /// The rule this blocker names.
    #[must_use]
    pub const fn rule(&self) -> RuleId {
        match self {
            Self::Blocking { rule, .. } | Self::Saturated { rule, .. } => *rule,
        }
    }

    /// The word that says which kind of failure this is.
    ///
    /// # Examples
    ///
    /// ```
    /// use kicli::lint::gate::Blocker;
    /// use kicli::lint::RuleId;
    /// let one = Blocker::Blocking { rule: RuleId("KI-ERC-001"), count: 1 };
    /// assert_eq!(one.word(), "blocking");
    /// ```
    #[must_use]
    pub const fn word(&self) -> &'static str {
        match self {
            Self::Blocking { .. } => "blocking",
            Self::Saturated { .. } => "saturated",
        }
    }

    /// One line saying why this rule fails the gate.
    ///
    /// A saturating rule writes the count, the denominator and the share,
    /// because that is the whole explanation and it costs one line.
    #[must_use]
    pub fn line(&self) -> String {
        match self {
            Self::Blocking { rule, count } => format!("blocking  {rule}  findings {count}"),
            Self::Saturated {
                rule,
                count,
                total,
                counted,
                share,
            } => {
                let (numerator, denominator) = *share;
                format!(
                    "saturated {rule}  {} {count} of {total} >= {numerator}/{denominator}",
                    counted.word()
                )
            }
        }
    }
}

/// Whether a drawing passes the gate, and every reason it does not.
///
/// The reasons are in rule order, which is the order findings are reported in.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Gate {
    blockers: Vec<Blocker>,
}

impl Gate {
    /// Read the gate from one sheet's findings.
    ///
    /// Each rule is answered once. A blocking rule that also saturated is
    /// reported as blocking, because it already fails on its first finding and
    /// two reasons for one rule would say the same thing twice.
    #[must_use]
    pub fn of(findings: &[Finding], density: Density) -> Self {
        let mut seen: Vec<Reported> = Vec::new();
        for finding in findings {
            match seen.iter_mut().find(|rule| rule.id == finding.rule) {
                Some(rule) => rule.count += 1,
                None => seen.push(Reported {
                    id: finding.rule,
                    tier: finding.tier,
                    saturation: finding.saturation,
                    count: 1,
                }),
            }
        }
        seen.sort_by_key(|rule| rule.id);

        let mut blockers = Vec::new();
        for rule in seen {
            if rule.tier == Tier::One {
                blockers.push(Blocker::Blocking {
                    rule: rule.id,
                    count: rule.count,
                });
            } else if rule.saturation.is_reached(rule.count, density) {
                blockers.push(Blocker::Saturated {
                    rule: rule.id,
                    count: rule.count,
                    total: rule.saturation.counts().total(density),
                    counted: rule.saturation.counts(),
                    share: rule.saturation.share(),
                });
            }
        }
        Self { blockers }
    }

    /// Does the drawing pass?
    #[must_use]
    pub fn passes(&self) -> bool {
        self.blockers.is_empty()
    }

    /// The word a report writes for this verdict.
    ///
    /// # Examples
    ///
    /// ```
    /// use kicli::lint::gate::Gate;
    /// assert_eq!(Gate::default().word(), "pass");
    /// ```
    #[must_use]
    pub fn word(&self) -> &'static str {
        if self.passes() { "pass" } else { "fail" }
    }

    /// Every reason the drawing fails, in rule order.
    #[must_use]
    pub fn blockers(&self) -> &[Blocker] {
        &self.blockers
    }
}

/// One rule's showing on one sheet, while the gate is being read.
struct Reported {
    id: RuleId,
    tier: Tier,
    saturation: Saturation,
    count: u32,
}

/// One sheet's score and one sheet's gate, in one answer.
///
/// The two are held together because reading either alone misleads. A drawing
/// that scores well and fails the gate is the case the tiers exist for, and a
/// reader who had to ask twice would let the failure hide behind the number.
///
/// # Examples
///
/// ```
/// use kicli::lint::gate::Report;
/// use kicli::lint::score::Density;
///
/// let clean = Report::of(&[], Density::of_counts(4, 4));
/// assert_eq!(clean.text(), "score 100  gate pass  raw 0.0\n");
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Report {
    score: SheetScore,
    gate: Gate,
}

impl Report {
    /// Score one sheet and read its gate, from one set of findings.
    #[must_use]
    pub fn of(findings: &[Finding], density: Density) -> Self {
        Self {
            score: SheetScore::of(findings, density),
            gate: Gate::of(findings, density),
        }
    }

    /// The sheet's score, and the numbers it was made from.
    #[must_use]
    pub const fn score(&self) -> SheetScore {
        self.score
    }

    /// The sheet's verdict, and every reason for it.
    #[must_use]
    pub const fn gate(&self) -> &Gate {
        &self.gate
    }

    /// The compact form, which carries both answers.
    ///
    /// The first line holds the number and the verdict together, so neither can
    /// be read without the other. Each following line names one rule that fails
    /// the gate. A drawing that passes writes one line and nothing more.
    #[must_use]
    pub fn text(&self) -> String {
        let mut text = String::new();
        let _ = writeln!(
            text,
            "score {}  gate {}  raw {}",
            self.score.score(),
            self.gate.word(),
            self.score.raw().text()
        );
        for blocker in self.gate.blockers() {
            let _ = writeln!(text, "  {}", blocker.line());
        }
        text
    }
}
