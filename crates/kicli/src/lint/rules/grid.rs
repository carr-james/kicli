//! `KI-GRID-001` — connectable geometry off the connection grid.
//!
//! KiCad joins two objects when their connection points are the *same* point.
//! There is no tolerance in that test, so a pin one internal unit off the grid
//! the wires were drawn on does not connect, and nothing on screen says so: the
//! drawing reads as wired and the netlist is not. That is why the rule blocks
//! rather than scores (Constitution §7).
//!
//! # What is connectable
//!
//! Symbol pins, wire endpoints, junctions, no-connect markers, label anchors
//! and sheet pin positions. Each is a point KiCad's connectivity graph reads.
//!
//! # What is exempt, and why the exemption is not a softening
//!
//! Field and graphic text positions. `research/geometry.md` Contradiction 2:
//! **KiCad's own field autoplacement lands fields on arbitrary internal
//! units**, so a rule over every position in the file would fire on a file
//! KiCad had just written. A text position joins nothing, so nothing is lost
//! by leaving it alone. The exemption is held by
//! `cargo test --test lint_grid_covers_every_connectable_class`, not by this
//! paragraph.
//!
//! # Arithmetic
//!
//! Integer modulus, through [`crate::geometry::Point::is_on_grid`]. There is no tolerance and no
//! rounding: a tolerance would make the rule's own boundary a judgement, and
//! KiCad's connectivity has none to copy (Constitution §4).

use crate::geometry::resolve_pins;
use crate::lint::drawing::Drawing;
use crate::lint::finding::{RuleId, Tier};
use crate::lint::rule::{Findings, Rule};
use crate::model::items::{Item, LineKind};

/// Every connectable point that is not on the connection grid.
pub struct ConnectableGeometryOffGrid;

impl Rule for ConnectableGeometryOffGrid {
    fn id(&self) -> RuleId {
        RuleId("KI-GRID-001")
    }

    fn tier(&self) -> Tier {
        Tier::One
    }

    fn examine(&self, drawing: &Drawing<'_>, found: &mut Findings<'_>) {
        for item in &drawing.schematic().items {
            match item {
                Item::Symbol(symbol) => {
                    // A pin's position is the placement's anchor plus the
                    // library offset through the orientation matrix. The rule
                    // reads the resolved point, because an anchor on the grid
                    // says nothing about where the pins landed.
                    let Some(definition) = drawing.definition_of(symbol) else {
                        continue;
                    };
                    for pin in resolve_pins(symbol, definition) {
                        if pin.position.is_on_grid() {
                            continue;
                        }
                        let mut objects = vec![symbol.uuid.clone()];
                        objects.extend(pin.uuid.clone());
                        found.record(pin.position, objects, message("pin"));
                    }
                }
                Item::Line(line) if line.kind == LineKind::Wire => {
                    // Both ends, separately. A wire drawn from a good point to
                    // a bad one connects at one end and not the other, which
                    // is the case a per-wire finding would report as one
                    // problem and leave half of.
                    for end in [line.from, line.to] {
                        if !end.is_on_grid() {
                            found.record(end, vec![line.uuid.clone()], message("wire endpoint"));
                        }
                    }
                }
                Item::Junction(junction) if !junction.at.is_on_grid() => {
                    found.record(
                        junction.at,
                        vec![junction.uuid.clone()],
                        message("junction"),
                    );
                }
                Item::NoConnect(marker) if !marker.at.is_on_grid() => {
                    found.record(
                        marker.at,
                        vec![marker.uuid.clone()],
                        message("no-connect marker"),
                    );
                }
                Item::Label(label) if !label.at.is_on_grid() => {
                    found.record(label.at, vec![label.uuid.clone()], message("label anchor"));
                }
                Item::Sheet(sheet) => {
                    for pin in &sheet.pins {
                        if pin.at.is_on_grid() {
                            continue;
                        }
                        found.record(
                            pin.at,
                            vec![sheet.uuid.clone(), pin.uuid.clone()],
                            message("sheet pin"),
                        );
                    }
                }
                _ => {}
            }
        }
    }
}

/// What one finding says, for a connectable point of the named kind.
///
/// The position is already a field of the finding, so the message carries the
/// kind and nothing else. A reader who needs the coordinate has it.
fn message(what: &str) -> String {
    format!("the {what} sits off the connection grid")
}

/// The one instance of the rule, held for the registry.
static OFF_GRID: ConnectableGeometryOffGrid = ConnectableGeometryOffGrid;

/// The rules this file declares.
pub static RULES: &[&'static dyn Rule] = &[&OFF_GRID];
