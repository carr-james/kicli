//! Two specimen rules that report the same wire at the same place.
//!
//! The file holds a family, because one file may hold several rules that share
//! a definition. The two rules report at identical positions, so their findings
//! contend on the first term of the sort key and on nothing else.

use kicli::lint::{Drawing, Findings, Penalty, Rule, RuleId, Tier};
use kicli::model::items::{Item, Line};

/// Every wire segment of a drawing, in file order.
fn wires<'a>(drawing: &'a Drawing<'a>) -> Vec<&'a Line> {
    drawing
        .schematic()
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Line(line) => Some(line),
            _ => None,
        })
        .collect()
}

/// Every wire is a blocking finding.
pub struct EveryWireBlocks;

impl Rule for EveryWireBlocks {
    fn id(&self) -> RuleId {
        RuleId("KI-SPEC-002")
    }

    fn tier(&self) -> Tier {
        Tier::One
    }

    fn examine(&self, drawing: &Drawing<'_>, found: &mut Findings<'_>) {
        for line in wires(drawing) {
            found.record(
                line.from,
                vec![line.uuid.clone()],
                "the segment is drawn".to_owned(),
            );
        }
    }
}

/// Every wire is a scored finding, at the very same place.
pub struct EveryWireScores;

impl Rule for EveryWireScores {
    fn id(&self) -> RuleId {
        RuleId("KI-SPEC-003")
    }

    fn tier(&self) -> Tier {
        Tier::Two
    }

    fn weight(&self) -> Penalty {
        Penalty::thousandths(1_500)
    }

    fn examine(&self, drawing: &Drawing<'_>, found: &mut Findings<'_>) {
        for line in wires(drawing) {
            found.record(
                line.from,
                vec![line.uuid.clone()],
                "the segment is drawn".to_owned(),
            );
        }
    }
}

/// The rules this file declares.
pub static RULES: &[&'static dyn Rule] = &[&EveryWireBlocks, &EveryWireScores];
