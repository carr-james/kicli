//! A specimen rule that reports every placed symbol.
//!
//! The rule exists so the seam can be measured. It reports something at a
//! position that differs from symbol to symbol, so findings of one rule on one
//! sheet contend on the third and fourth terms of the sort key.

use kicli::lint::{Drawing, Findings, Penalty, Rule, RuleId, Tier};

/// Every placed symbol is a finding.
pub struct EverySymbol;

impl Rule for EverySymbol {
    fn id(&self) -> RuleId {
        RuleId("KI-SPEC-001")
    }

    fn tier(&self) -> Tier {
        Tier::Two
    }

    fn weight(&self) -> Penalty {
        Penalty::points(2)
    }

    fn examine(&self, drawing: &Drawing<'_>, found: &mut Findings<'_>) {
        for symbol in drawing.schematic().symbols() {
            found.record_with_fix(
                symbol.at,
                vec![symbol.uuid.clone()],
                format!("symbol {} is placed", symbol.lib_id.0),
                format!("kicli sym show {}", symbol.uuid.short()),
            );
        }
    }
}

/// The rules this file declares.
pub static RULES: &[&'static dyn Rule] = &[&EverySymbol];
