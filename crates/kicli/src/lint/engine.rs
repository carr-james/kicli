//! Running every rule over every drawing, in one order.
//!
//! The engine holds the rules, asks each one about each drawing, and sorts what
//! comes back. Sorting happens here and once, because the order is part of the
//! output contract.
//!
//! The engine adds no judgement of its own. It decides nothing about what is
//! wrong with a drawing, so a rule is the only place a judgement lives.

use crate::lint::drawing::Drawing;
use crate::lint::finding::{Finding, RuleId, sort};
use crate::lint::registry;
use crate::lint::rule::{Findings, Rule};

/// The rules to run, and the order to report their findings in.
pub struct Engine {
    rules: Vec<&'static dyn Rule>,
}

impl Engine {
    /// Every rule the build registered.
    ///
    /// The registry is generated from the files under `src/lint/rules/`, so
    /// this is every rule the crate holds.
    #[must_use]
    pub fn of_every_rule() -> Self {
        Self::of(registry::all())
    }

    /// A named set of rules, which is what a test and a configured run use.
    ///
    /// The rules are held in code order, sorted by their codes. Two rules with
    /// one code would double count; `lint_rules_register_from_their_own_files`
    /// is what refuses that.
    #[must_use]
    pub fn of(rules: Vec<&'static dyn Rule>) -> Self {
        let mut rules = rules;
        rules.sort_by_key(|rule| rule.id());
        Self { rules }
    }

    /// The codes of the rules this engine runs, in order.
    #[must_use]
    pub fn codes(&self) -> Vec<RuleId> {
        self.rules.iter().map(|rule| rule.id()).collect()
    }

    /// How many rules this engine runs.
    #[must_use]
    pub fn len(&self) -> usize {
        self.rules.len()
    }

    /// Does this engine run no rules at all?
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }

    /// Every finding of every rule over one drawing, in report order.
    #[must_use]
    pub fn examine(&self, drawing: &Drawing<'_>) -> Vec<Finding> {
        let mut found = self.gather(drawing);
        sort(&mut found);
        found
    }

    /// Every finding of every rule over several drawings, in report order.
    ///
    /// The order is the same one a single drawing gets. A finding's sheet is
    /// the second term of it, so the drawings need no order of their own.
    #[must_use]
    pub fn examine_all<'a>(
        &self,
        drawings: impl IntoIterator<Item = &'a Drawing<'a>>,
    ) -> Vec<Finding> {
        let mut found: Vec<Finding> = drawings
            .into_iter()
            .flat_map(|drawing| self.gather(drawing))
            .collect();
        sort(&mut found);
        found
    }

    fn gather(&self, drawing: &Drawing<'_>) -> Vec<Finding> {
        let mut found = Vec::new();
        for rule in &self.rules {
            let mut collected = Findings::of(*rule, drawing.path());
            rule.examine(drawing, &mut collected);
            found.extend(collected.into_vec());
        }
        found
    }
}
