//! What a rule is, and how a rule reports.
//!
//! A rule is a pure function over one drawing. It reads geometry and writes
//! findings into a collector. The collector stamps the rule's own identity onto
//! every finding, so a rule cannot report under another rule's code.
//!
//! Rules are an open set, so they are trait objects. Each rule is a value with
//! no state, held in a `static` in the file that declares it.

use crate::geometry::Point;
use crate::lint::drawing::Drawing;
use crate::lint::finding::{Finding, Penalty, RuleId, Severity, Tier};
use crate::model::items::{SheetPath, Uuid};

/// One deterministic check over a drawing.
///
/// Implement this once per rule, in one file under `src/lint/rules/`. Declare
/// the rule in that file's `RULES` slice. Nothing else registers it.
///
/// # Examples
///
/// ```
/// use kicli::lint::{Drawing, Findings, Penalty, Rule, RuleId, Tier};
///
/// struct EveryPageIsWrong;
///
/// impl Rule for EveryPageIsWrong {
///     fn id(&self) -> RuleId {
///         RuleId("KI-DEMO-001")
///     }
///
///     fn tier(&self) -> Tier {
///         Tier::Two
///     }
///
///     fn weight(&self) -> Penalty {
///         Penalty::points(1)
///     }
///
///     fn examine(&self, _drawing: &Drawing<'_>, found: &mut Findings<'_>) {
///         found.record(Default::default(), Vec::new(), "the page is wrong".to_owned());
///     }
/// }
/// ```
pub trait Rule: Sync {
    /// The rule's published code.
    fn id(&self) -> RuleId;

    /// Which tier the rule is in.
    fn tier(&self) -> Tier;

    /// The weight of one occurrence, before normalisation.
    ///
    /// Tier 1 rules do not move the score, so they keep the default of nothing.
    fn weight(&self) -> Penalty {
        Penalty::ZERO
    }

    /// How loudly the rule reports.
    ///
    /// The default follows the tier. Override it only when the rule reports
    /// something quieter than its tier suggests.
    fn severity(&self) -> Severity {
        match self.tier() {
            Tier::One => Severity::Error,
            Tier::Two => Severity::Warning,
        }
    }

    /// Look at one drawing and record what is wrong with it.
    ///
    /// The method reads and never writes. It sees one sheet placement at a
    /// time, because a finding names one sheet.
    fn examine(&self, drawing: &Drawing<'_>, found: &mut Findings<'_>);
}

/// Where a rule puts what it finds.
///
/// The collector holds the rule's identity and the sheet under examination.
/// A rule supplies only the position, the objects, the message and the optional
/// command. Everything else is stamped, so it cannot be got wrong.
pub struct Findings<'a> {
    rule: RuleId,
    tier: Tier,
    severity: Severity,
    weight: Penalty,
    sheet: &'a SheetPath,
    found: Vec<Finding>,
}

impl<'a> Findings<'a> {
    /// A collector for one rule looking at one sheet.
    #[must_use]
    pub fn of(rule: &dyn Rule, sheet: &'a SheetPath) -> Self {
        Self {
            rule: rule.id(),
            tier: rule.tier(),
            severity: rule.severity(),
            weight: rule.weight(),
            sheet,
            found: Vec::new(),
        }
    }

    /// Record one finding.
    pub fn record(&mut self, at: Point, objects: Vec<Uuid>, message: String) {
        self.push(at, objects, message, None);
    }

    /// Record one finding, with a command the caller could run to fix it.
    ///
    /// The command is text. Nothing in this module runs it, because scoring
    /// never mutates a file.
    pub fn record_with_fix(&mut self, at: Point, objects: Vec<Uuid>, message: String, fix: String) {
        self.push(at, objects, message, Some(fix));
    }

    /// How many findings the rule has recorded so far.
    #[must_use]
    pub fn len(&self) -> usize {
        self.found.len()
    }

    /// Has the rule recorded nothing?
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.found.is_empty()
    }

    /// Take what the rule recorded, in the order it recorded it.
    #[must_use]
    pub fn into_vec(self) -> Vec<Finding> {
        self.found
    }

    fn push(&mut self, at: Point, objects: Vec<Uuid>, message: String, fix: Option<String>) {
        self.found.push(Finding {
            rule: self.rule,
            tier: self.tier,
            severity: self.severity,
            sheet: self.sheet.clone(),
            pos: at,
            objects,
            message,
            fix,
            penalty: self.weight,
        });
    }
}
