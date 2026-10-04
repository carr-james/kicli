//! What KiCad's own electrical rule check said, in the form a rule reads it.
//!
//! KiCad 10's rule check implements 47 checks and kicli's lint engine
//! implements none of them (`spec/SPEC.md` §11.1). kicli runs the check, maps
//! what it reports into kicli's own findings, and adds only what the electrical
//! layer structurally cannot see: where things are *drawn*.
//!
//! # This is the seam, and it is data
//!
//! Everything in this module is a value. Nothing here starts a process, opens a
//! file, or knows that `kicad-cli` exists — [`ENGINEERING.md`'s dependency
//! direction](../../../../ENGINEERING.md) puts that knowledge in
//! [`crate::kicad`], which produces a [`RuleCheck`] and hands it in. A rule
//! reads one off the [`Drawing`] it is examining and never asks where it came
//! from.
//!
//! # Attribution, and why double counting is the thing to be careful about
//!
//! Two of KiCad's checks are deliberately re-published under kicli codes,
//! because KiCad's default severity for both is `IGNORE` and an untouched
//! project would silently pass: `four_way_junction` → `KI-JCT-001`, and
//! `single_global_label` → `KI-LBL-003` (§11.1). A third, `hier_label_mismatch`,
//! is what `KI-HIER-001` *is*.
//!
//! So one occurrence can have two reporters — KiCad's check and kicli's own
//! geometry — and a project that enables the severity would then be penalised
//! twice for one mistake. The mechanism that stops it is one question:
//!
//! ```
//! use kicli::lint::erc::{RuleCheck, delegate};
//! use kicli::lint::{Drawing, Findings, Penalty, Rule, RuleId, Tier};
//!
//! struct FourWayJunction;
//!
//! /// The check KiCad already implements, which this rule re-publishes.
//! const CHECK: &str = "four_way_junction";
//!
//! impl Rule for FourWayJunction {
//!     fn id(&self) -> RuleId {
//!         RuleId("KI-JCT-001")
//!     }
//!
//!     fn tier(&self) -> Tier {
//!         Tier::Two
//!     }
//!
//!     fn weight(&self) -> Penalty {
//!         Penalty::points(1)
//!     }
//!
//!     fn examine(&self, drawing: &Drawing<'_>, found: &mut Findings<'_>) {
//!         if drawing.rule_check().covers(CHECK) {
//!             // KiCad looked. Attribute what it found and add nothing.
//!             delegate(CHECK, drawing, found);
//!             return;
//!         }
//!         // KiCad did not look, so kicli must. (Its own geometry goes here.)
//!     }
//! }
//!
//! // A drawing nobody ran the rule check over reports that it was not covered,
//! // which is what sends a rule to its own detection rather than to silence.
//! assert!(!RuleCheck::NOT_RUN.covers(CHECK));
//! ```
//!
//! **A check kicli never ran is not a check that passed.** [`RuleCheck::covers`]
//! answers `false` for both "KiCad ignored it" and "nobody asked KiCad", so the
//! two cases that must fall through to kicli's own detection do, and only a
//! check KiCad actually ran suppresses it.
//!
//! # Severities are read-only
//!
//! §14.3. A check's severity lives in the project's `.kicad_pro`, under
//! `erc.rule_severities`, and `kicad-cli` can only filter which severities are
//! *reported*. **kicli relabels them for its own output and never edits
//! `.kicad_pro`.** [`KicadSeverity::relabelled`] is that relabelling, and
//! `kicli.toml`'s ERC severity mapping is therefore a **presentation** mapping:
//! it changes what kicli's report says, never what KiCad checks.

use crate::geometry::Point;
use crate::lint::drawing::Drawing;
use crate::lint::finding::Severity;
use crate::lint::rule::Findings;
use crate::model::items::{SheetPath, Uuid};

/// How loudly KiCad reported a violation.
///
/// This is KiCad's word, not kicli's. It is kept apart from [`Severity`]
/// because the two are set in different files by different people: KiCad's
/// comes from the project's `.kicad_pro`, which kicli never writes, and
/// kicli's is what kicli's own report says. [`Self::relabelled`] is the only
/// bridge, and it is presentation (§14.3).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum KicadSeverity {
    /// KiCad calls it an error.
    Error,
    /// KiCad calls it a warning.
    Warning,
    /// The project excluded this one occurrence.
    Excluded,
}

impl KicadSeverity {
    /// Read one of the words KiCad writes in its report.
    ///
    /// # Examples
    ///
    /// ```
    /// use kicli::lint::erc::KicadSeverity;
    /// assert_eq!(KicadSeverity::read("error"), Some(KicadSeverity::Error));
    /// assert_eq!(KicadSeverity::read("ignore"), None);
    /// ```
    #[must_use]
    pub fn read(word: &str) -> Option<Self> {
        match word {
            "error" => Some(Self::Error),
            "warning" => Some(Self::Warning),
            "exclusion" | "excluded" => Some(Self::Excluded),
            _ => None,
        }
    }

    /// The word KiCad writes for this severity.
    #[must_use]
    pub const fn word(self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warning => "warning",
            Self::Excluded => "exclusion",
        }
    }

    /// The kicli severity a report writes for this one.
    ///
    /// **Presentation only.** kicli cannot change what KiCad checks or how
    /// loudly KiCad checks it — that is `.kicad_pro`'s, and kicli does not
    /// write `.kicad_pro` (§14.3). This decides what kicli's own report says
    /// and nothing else. An excluded occurrence is relabelled a warning rather
    /// than dropped, because the project excluded it from KiCad's verdict and
    /// not from kicli's view of how the drawing reads.
    ///
    /// # Examples
    ///
    /// ```
    /// use kicli::lint::Severity;
    /// use kicli::lint::erc::KicadSeverity;
    /// assert_eq!(KicadSeverity::Error.relabelled(), Severity::Error);
    /// assert_eq!(KicadSeverity::Excluded.relabelled(), Severity::Warning);
    /// ```
    #[must_use]
    pub const fn relabelled(self) -> Severity {
        match self {
            Self::Error => Severity::Error,
            Self::Warning | Self::Excluded => Severity::Warning,
        }
    }
}

/// One object a violation names.
///
/// The identifier is the whole reason kicli reads the JSON report rather than
/// the text one: it joins straight to kicli's own object handles, and the text
/// report carries no identifier at all (`research/kicad-cli.md` §3.1).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Item {
    /// The object KiCad names, when the report carries an identifier for it.
    pub uuid: Option<Uuid>,
    /// What KiCad calls the object, such as `Symbol R1 Pin 1 [Passive, Line]`.
    pub description: String,
    /// Where KiCad says it is.
    pub at: Point,
}

/// One violation KiCad's rule check reported.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Violation {
    /// The check's key, such as `four_way_junction`.
    pub check: String,
    /// How loudly KiCad reported it.
    pub severity: KicadSeverity,
    /// What KiCad said about it, in one sentence.
    pub description: String,
    /// The sheet placement it is on.
    pub sheet: SheetPath,
    /// The objects it names, in the order KiCad named them.
    pub items: Vec<Item>,
}

impl Violation {
    /// Where a reader should look, which is the first object KiCad named.
    ///
    /// A violation with no objects at all is about the sheet, so it has no
    /// place on the sheet to point at and takes the origin.
    #[must_use]
    pub fn at(&self) -> Point {
        self.items.first().map_or(Point::default(), |item| item.at)
    }

    /// The identifiers of the objects it names, in KiCad's order.
    #[must_use]
    pub fn objects(&self) -> Vec<Uuid> {
        self.items
            .iter()
            .filter_map(|item| item.uuid.clone())
            .collect()
    }
}

/// Everything KiCad's rule check said about one project.
///
/// The empty value is [`Self::NOT_RUN`], and it means **nobody asked KiCad** —
/// which is a different answer from *KiCad found nothing*. Keeping the two
/// apart is what stops a rule from falling silent on a machine with no KiCad
/// install: see [`Self::covers`].
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuleCheck {
    violations: Vec<Violation>,
    ignored: Vec<String>,
    ran: bool,
}

/// A rule check nobody ran, for a [`Drawing`] that was given none.
static NOT_RUN: RuleCheck = RuleCheck::NOT_RUN;

impl RuleCheck {
    /// A rule check that never ran.
    pub const NOT_RUN: Self = Self {
        violations: Vec::new(),
        ignored: Vec::new(),
        ran: false,
    };

    /// A rule check that ran, with what it reported and what it skipped.
    ///
    /// `ignored` holds the keys of the checks the project turned off, which is
    /// what KiCad writes under `ignored_checks`.
    #[must_use]
    pub fn ran(violations: Vec<Violation>, ignored: Vec<String>) -> Self {
        Self {
            violations,
            ignored,
            ran: true,
        }
    }

    /// Did KiCad's rule check run at all?
    #[must_use]
    pub const fn has_run(&self) -> bool {
        self.ran
    }

    /// Every violation, in the order KiCad reported them.
    #[must_use]
    pub fn violations(&self) -> &[Violation] {
        &self.violations
    }

    /// The keys of the checks the project turned off.
    #[must_use]
    pub fn ignored(&self) -> &[String] {
        &self.ignored
    }

    /// Did KiCad actually run this check on this project?
    ///
    /// This is the question a rule that duplicates a KiCad check must ask
    /// before it looks for itself. It answers `false` when the project ignored
    /// the check **and** when the rule check never ran, because both leave the
    /// occurrence unreported by KiCad and so leave it to kicli.
    ///
    /// # Examples
    ///
    /// ```
    /// use kicli::lint::erc::RuleCheck;
    ///
    /// let ran = RuleCheck::ran(Vec::new(), vec!["four_way_junction".to_owned()]);
    /// assert!(ran.covers("single_global_label"), "KiCad looked at this one");
    /// assert!(!ran.covers("four_way_junction"), "the project turned it off");
    /// assert!(!RuleCheck::NOT_RUN.covers("single_global_label"), "nobody looked");
    /// ```
    #[must_use]
    pub fn covers(&self, check: &str) -> bool {
        self.ran && !self.ignored.iter().any(|key| key == check)
    }

    /// The violations of one check, on one sheet placement.
    pub fn of_check<'a>(
        &'a self,
        check: &'a str,
        sheet: &'a SheetPath,
    ) -> impl Iterator<Item = &'a Violation> {
        self.violations
            .iter()
            .filter(move |violation| violation.check == check && &violation.sheet == sheet)
    }
}

/// Record KiCad's own violations of `check` under this rule's code.
///
/// This is the attribution half of §11.1: kicli says which of its rules the
/// finding belongs to, and KiCad's check is what actually found it. The message
/// stays KiCad's, so a reader can tell that the electrical layer reported it.
///
/// Returns how many findings were recorded, so a caller that wants to know
/// whether KiCad had anything to say does not have to count twice.
///
/// **A rule calls this instead of looking, never as well as looking.** See the
/// module documentation for the shape.
pub fn delegate(check: &str, drawing: &Drawing<'_>, found: &mut Findings<'_>) -> usize {
    let mut recorded = 0;
    for violation in drawing.rule_check().of_check(check, drawing.path()) {
        found.record(
            violation.at(),
            violation.objects(),
            format!(
                "{} ({check}, reported by KiCad's ERC)",
                violation.description
            ),
        );
        recorded += 1;
    }
    recorded
}

/// The rule check a drawing was given, or one that never ran.
pub(crate) fn not_run() -> &'static RuleCheck {
    &NOT_RUN
}

#[cfg(test)]
mod tests {
    use super::{Item, KicadSeverity, RuleCheck, Violation};
    use crate::geometry::Point;
    use crate::lint::Severity;
    use crate::model::items::{SheetPath, Uuid};

    fn violation(check: &str, sheet: &str) -> Violation {
        Violation {
            check: check.to_owned(),
            severity: KicadSeverity::Warning,
            description: "something".to_owned(),
            sheet: SheetPath(sheet.to_owned()),
            items: vec![Item {
                uuid: Some(Uuid("31000010-0000-4000-8000-000000000001".to_owned())),
                description: "Symbol R1".to_owned(),
                at: Point::new(254_000, 254_000),
            }],
        }
    }

    #[test]
    fn a_check_nobody_ran_is_not_a_check_that_passed() {
        // The two false answers are the point: a rule that read them as "KiCad
        // has this covered" would fall silent on a machine with no KiCad.
        assert!(!RuleCheck::NOT_RUN.covers("four_way_junction"));
        let ignored = RuleCheck::ran(Vec::new(), vec!["four_way_junction".to_owned()]);
        assert!(!ignored.covers("four_way_junction"));
        assert!(ignored.covers("single_global_label"));
    }

    #[test]
    fn violations_are_read_by_check_and_by_sheet() {
        let check = RuleCheck::ran(
            vec![
                violation("four_way_junction", "/a"),
                violation("four_way_junction", "/b"),
                violation("single_global_label", "/a"),
            ],
            Vec::new(),
        );
        let sheet = SheetPath("/a".to_owned());
        assert_eq!(check.of_check("four_way_junction", &sheet).count(), 1);
        assert_eq!(check.of_check("single_global_label", &sheet).count(), 1);
        assert_eq!(check.of_check("pin_not_connected", &sheet).count(), 0);
    }

    #[test]
    fn a_violation_points_at_its_first_object() {
        let one = violation("four_way_junction", "/a");
        assert_eq!(one.at(), Point::new(254_000, 254_000));
        assert_eq!(one.objects().len(), 1);

        let mut sheet_wide = one;
        sheet_wide.items.clear();
        assert_eq!(sheet_wide.at(), Point::default());
        assert!(sheet_wide.objects().is_empty());
    }

    #[test]
    fn kicad_severities_are_relabelled_and_never_written_back() {
        assert_eq!(KicadSeverity::Error.relabelled(), Severity::Error);
        assert_eq!(KicadSeverity::Warning.relabelled(), Severity::Warning);
        assert_eq!(KicadSeverity::Excluded.relabelled(), Severity::Warning);
        for word in ["error", "warning", "exclusion"] {
            let read = KicadSeverity::read(word).expect("a word KiCad writes");
            assert_eq!(read.word(), word);
        }
        assert_eq!(KicadSeverity::read("ignore"), None);
    }
}
