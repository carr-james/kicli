//! A specimen rule whose findings differ only in the objects they name.
//!
//! Every finding of this rule on one sheet sits at the same junction, so the
//! first four terms of the sort key agree and only the object list separates
//! them. One pair differs at the second object rather than the first, which is
//! the case a key holding a single identifier cannot order.

use kicli::lint::{Drawing, Findings, Penalty, Rule, RuleId, Tier};
use kicli::model::items::{Item, Uuid};

/// Every junction is reported three times, with three object lists.
pub struct EveryJunctionThrice;

impl Rule for EveryJunctionThrice {
    fn id(&self) -> RuleId {
        RuleId("KI-SPEC-004")
    }

    fn tier(&self) -> Tier {
        Tier::Two
    }

    fn weight(&self) -> Penalty {
        Penalty::points(1)
    }

    fn examine(&self, drawing: &Drawing<'_>, found: &mut Findings<'_>) {
        // The symbols the drawing holds, by identifier, so the second object of
        // a pair is a real object rather than an invented one.
        let mut symbols: Vec<Uuid> = drawing
            .schematic()
            .symbols()
            .map(|symbol| symbol.uuid.clone())
            .collect();
        symbols.sort();

        for item in &drawing.schematic().items {
            let Item::Junction(junction) = item else {
                continue;
            };
            found.record(
                junction.at,
                vec![junction.uuid.clone()],
                "the junction is drawn".to_owned(),
            );
            for symbol in symbols.iter().take(2) {
                found.record(
                    junction.at,
                    vec![junction.uuid.clone(), symbol.clone()],
                    "the junction is near a symbol".to_owned(),
                );
            }
        }
    }
}

/// The rules this file declares.
pub static RULES: &[&'static dyn Rule] = &[&EveryJunctionThrice];
