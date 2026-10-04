//! `KI-CONN-001` — a pin touches a wire but is not connected to it.
//!
//! A pin whose connection point lands on a wire's *interior* with no junction
//! there reads, on screen and to a reviewer, exactly like a connection. KiCad
//! 10.0.5's netlister does not merge it, so the board is wired differently from
//! the way the schematic reads. Measured in both directions against KiCad
//! 10.0.5 in `research/notes/pin-on-wire-interior.md`: two clusters identical
//! but for a junction, and only the junctioned one is one net.
//!
//! The tier has published support, which is rare in this catalogue —
//! `lane-t5` measured 24 of 28 tiers as asserted rather than argued. Olin
//! Lathrop, on the junction dot: *"It's a rule. We don't care whether you think
//! it's silly or not. That's how it's done."*
//!
//! KiCad's own 47 rule checks have nothing for this. The netlister reports two
//! nets, which from its point of view is not a violation, so there is no
//! overlap with ERC and the check is kicli's to make.
//!
//! # The test, and why it is the extractor's answer rather than this file's
//!
//! For every pin connection point `p` and wire segment `w` where `p` lies on
//! the interior of `w`: a finding when `p` and `w` are in **different nets
//! after union-find**. Geometric coincidence without electrical merge is the
//! whole test.
//!
//! It is deliberately *not* "there is no junction at `p`". A pin sitting
//! mid-wire may still be on the wire's net by another route — a ground pin on a
//! wire that a label or another power symbol has already named `GND` is the
//! ordinary case — and firing there would block a build on a correct drawing.
//! The same phrasing is what makes the rule catch the label-plus-pin case
//! (`research/notes/label-on-wire-interior.md`) with no code written for it: a
//! label sharing a mid-wire anchor with a pin joins the pin and leaves the wire
//! out, so the pin's net is not the wire's net, which is already the test.
//!
//! # Which side each half of the comparison comes from
//!
//! Both sides are the extractor's, and they are two **different questions** put
//! to one partition rather than one answer compared with itself:
//!
//! - the pin's net is the net the partition lists that pin on;
//! - the wire's net is the net of a pin sitting at an **end** of the wire, or
//!   of a wire joined to it end to end. A wire's connection points are its ends
//!   (`SCH_LINE::GetConnectionPoints`), so a pin at an end is on the wire's net
//!   by the extractor's first merge rule, exactly and without inference.
//!
//! A break that collapsed the partition would put both sides on one net and the
//! rule would stop reporting; a break that shattered it would put every pin on
//! its own net and the rule would report everywhere. Neither moves the two
//! sides together, which is what a comparison of a thing with itself does.
//!
//! # Where this under-reports, and never over-reports
//!
//! The wire's net is only known when the extractor lists a pin at an end of the
//! wire's end-to-end chain. Where it does not — a wire whose chain reaches no
//! pin, or a **sheet pin**, which the partition does not list at all — the rule
//! reports **nothing**. Every gap is therefore a missed finding and never a
//! false one, which is the direction that costs least: a pin at a wire's end is
//! an ordinary connection, and a rule that blocked a build on one would be
//! worse than a rule that stayed quiet.

use crate::connectivity::Nets;
use crate::geometry::{Point, on_segment, resolve_pins};
use crate::lint::gate::Saturation;
use crate::lint::{Drawing, Findings, Rule, RuleId, Tier};
use crate::model::items::{Item, Line, LineKind, Refdes, SheetPath, Uuid};
use std::collections::BTreeMap;

/// A pin of this sheet placement, with its position and its net.
struct Located {
    /// Where the pin connects.
    at: Point,
    /// Which net of the partition the pin is on.
    net: usize,
    /// The pin as a netlist writes it, such as `R11.1`.
    label: String,
    /// What a finding names: the pin instance, or its symbol when the
    /// placement records no instance for the pin.
    object: Uuid,
}

/// A pin touches a wire but is not connected to it.
pub struct PinTouchesWire;

impl Rule for PinTouchesWire {
    fn id(&self) -> RuleId {
        RuleId("KI-CONN-001")
    }

    fn tier(&self) -> Tier {
        Tier::One
    }

    /// This rule never saturates, and the declaration is honest rather than
    /// empty.
    ///
    /// Two reasons, and the first is the one that matters. **A blocking rule
    /// fails the gate on its first finding**, so [`crate::lint::gate::Gate::of`]
    /// never asks a Tier 1 rule what share it covered: the share could be one
    /// pin in a thousand or every pin on the sheet and the verdict is the same
    /// failure. Saturation exists because a *normalised* rule's cost is capped
    /// however often it fires, and a blocking rule has no cost to cap.
    ///
    /// The second is a gap worth reporting rather than papering over. What this
    /// rule counts is **pin connection points**, and [`crate::lint::gate::Counted`]
    /// offers symbols, wires or nothing. Declaring wires here would be a false
    /// declaration of the denominator, so it declares nothing and says why.
    fn saturation(&self) -> Saturation {
        Saturation::NEVER
    }

    fn examine(&self, drawing: &Drawing<'_>, found: &mut Findings<'_>) {
        // No partition, no answer. A rule that guessed here would guess in the
        // blocking direction, which is the expensive one.
        let Some(nets) = drawing.nets() else {
            return;
        };

        let wires = wires_of(drawing);
        let pins = pins_of(drawing, nets);
        let chains = chains_of(&wires);
        let chain_nets = chain_nets(&wires, &chains, &pins);

        for pin in &pins {
            let Some((wire, wire_net)) = wire_it_is_not_on(&wires, &chain_nets, pin) else {
                continue;
            };
            found.record_with_fix(
                pin.at,
                vec![pin.object.clone(), wire.uuid.clone()],
                format!(
                    "pin {} touches this wire but is on net {}, not {}",
                    pin.label,
                    nets.nets()[pin.net].name,
                    nets.nets()[wire_net].name
                ),
                format!("kicli junction add --at {}", pin.at),
            );
        }
    }
}

/// The wire segments of one drawing, in file order.
///
/// Buses are left out. A bus carries a bundle and never joins a single net, so
/// a pin on a bus's interior is a different drawing mistake from this one.
fn wires_of<'a>(drawing: &'a Drawing<'a>) -> Vec<&'a Line> {
    drawing
        .schematic()
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Line(line) if matches!(line.kind, LineKind::Wire) => Some(line),
            _ => None,
        })
        .collect()
}

/// Every pin this placement draws that the partition lists, in file order.
///
/// A symbol with no library definition draws no pins, and a symbol with no
/// instance record on this sheet path has no reference designator, so the
/// partition cannot name its pins. Both are skipped, and both are the same
/// omission the netlist itself makes.
fn pins_of(drawing: &Drawing<'_>, nets: &Nets) -> Vec<Located> {
    let index = net_index(nets);
    let sheet = drawing.path();
    let mut located = Vec::new();
    for item in &drawing.schematic().items {
        let Item::Symbol(symbol) = item else {
            continue;
        };
        let Some(definition) = drawing.definition_of(symbol) else {
            continue;
        };
        let Some(reference) = symbol.reference_on(sheet) else {
            continue;
        };
        for pin in resolve_pins(&symbol.drawn_on(sheet), definition) {
            let key = (sheet.clone(), reference.clone(), pin.number.clone());
            let Some(&net) = index.get(&key) else {
                continue;
            };
            located.push(Located {
                at: pin.position,
                net,
                label: format!("{}.{}", reference.0, pin.number),
                object: pin.uuid.clone().unwrap_or_else(|| symbol.uuid.clone()),
            });
        }
    }
    located
}

/// Which net each listed pin is on, by sheet, reference designator and number.
fn net_index(nets: &Nets) -> BTreeMap<(SheetPath, Refdes, String), usize> {
    let mut index = BTreeMap::new();
    for (place, net) in nets.nets().iter().enumerate() {
        for pin in &net.pins {
            index.insert(
                (pin.sheet.clone(), pin.reference.clone(), pin.number.clone()),
                place,
            );
        }
    }
    index
}

/// Group the wires that meet end to end, giving each group's index per wire.
///
/// Two wire segments that share an end share a connection point, so they are
/// one conductor by the extractor's first merge rule. Nothing else is grouped:
/// this walk is used only to **find** a pin the extractor has already put on
/// the wire's net, so a group that is smaller than the true net loses a finding
/// and a group that claimed too much would invent one. Sharing an end is the
/// only relation certain enough to be used that way.
fn chains_of(wires: &[&Line]) -> Vec<usize> {
    let mut chain: Vec<usize> = (0..wires.len()).collect();
    // Repeat until nothing moves. The relation is symmetric and the group
    // count only ever falls, so this settles in at most `wires.len()` passes.
    let mut moved = true;
    while moved {
        moved = false;
        for (one, first) in wires.iter().enumerate() {
            for (two, second) in wires.iter().enumerate().skip(one + 1) {
                if !meet(first, second) {
                    continue;
                }
                let (low, high) = (chain[one].min(chain[two]), chain[one].max(chain[two]));
                if low == high {
                    continue;
                }
                for group in &mut chain {
                    if *group == high {
                        *group = low;
                    }
                }
                moved = true;
            }
        }
    }
    chain
}

/// Do two wire segments share an end?
fn meet(one: &Line, two: &Line) -> bool {
    [one.from, one.to]
        .iter()
        .any(|end| *end == two.from || *end == two.to)
}

/// The net of each group of wires, when a listed pin sits at one of its ends.
///
/// The lowest net index wins where several pins answer. They should all give
/// one answer — they are all on one conductor — so the choice settles nothing
/// but determinism.
fn chain_nets(wires: &[&Line], chains: &[usize], pins: &[Located]) -> Vec<Option<usize>> {
    let mut nets: Vec<Option<usize>> = vec![None; wires.len()];
    for (index, wire) in wires.iter().enumerate() {
        for pin in pins {
            if pin.at != wire.from && pin.at != wire.to {
                continue;
            }
            let group = chains[index];
            for (other, slot) in nets.iter_mut().enumerate() {
                if chains[other] != group {
                    continue;
                }
                *slot = Some(slot.map_or(pin.net, |held| held.min(pin.net)));
            }
        }
    }
    nets
}

/// The first wire in file order whose interior this pin lands on **and whose
/// net the pin is not on**, with that net.
///
/// The two conditions are asked together on purpose. A pin can land on the
/// interior of two wires at once — at an unjunctioned crossing, say — and
/// asking only for the first wire it crosses would let a wire it *is* connected
/// to hide one it is not, purely by file order.
///
/// One finding per pin, not one per wire. The repair is a single junction at
/// the pin's connection point, and that junction joins **every** line passing
/// through the point, so a second finding at the same place would name the same
/// repair twice — and Constitution §6 says a view that floods is wrong whatever
/// it contains.
fn wire_it_is_not_on<'a>(
    wires: &[&'a Line],
    chain_nets: &[Option<usize>],
    pin: &Located,
) -> Option<(&'a Line, usize)> {
    wires
        .iter()
        .enumerate()
        .find_map(|(index, wire)| match chain_nets[index] {
            Some(net) if net != pin.net && is_interior(wire.from, wire.to, pin.at) => {
                Some((*wire, net))
            }
            _ => None,
        })
}

/// Does this point lie on the wire's interior?
///
/// Exact integer arithmetic, and the endpoint exclusion is exact too: a point
/// **at** an end is an ordinary connection and is excluded, and a point one
/// internal unit along from an end is interior and is not. Every coordinate in
/// a schematic is an integer number of internal units, so "not within 1 IU of
/// either endpoint" and "not equal to either endpoint" are the same exclusion —
/// there is no point between them to disagree about. The published rule's own
/// falsification obligation states the two cases that way: *"a pin exactly at
/// an endpoint (no finding), and one 1 IU inside the interior (finding)."*
///
/// A wire of zero length has no interior at all, and falls out of the same two
/// tests rather than needing one of its own.
fn is_interior(from: Point, to: Point, at: Point) -> bool {
    at != from && at != to && on_segment(from, to, at)
}

/// The rules this file declares.
pub static RULES: &[&'static dyn Rule] = &[&PinTouchesWire];

#[cfg(test)]
mod tests {
    use super::is_interior;
    use crate::geometry::Point;

    /// The boundary, from both sides, at the level the claim is made.
    ///
    /// The end-to-end checks in `tests/lint_pin_on_wire.rs` cannot see this
    /// one: a pin at a wire's end is on that wire's net by the extractor's
    /// first merge rule, so the net comparison suppresses it whether or not the
    /// exclusion is there, and a break that deletes the exclusion leaves every
    /// integration check green. Measured rather than supposed — that break was
    /// run. The exclusion states the published rule's own definition and covers
    /// the case where the two mechanisms could disagree, so it is checked here,
    /// where deleting it is visible.
    #[test]
    fn an_end_is_not_the_interior_and_one_unit_along_from_it_is() {
        let (from, to) = (Point::new(0, 0), Point::new(10_000, 0));
        assert!(!is_interior(from, to, from), "a pin at an end connects");
        assert!(!is_interior(from, to, to), "and so does one at the other");
        assert!(
            is_interior(from, to, Point::new(1, 0)),
            "one internal unit along from an end is the interior"
        );
        assert!(
            is_interior(from, to, Point::new(9_999, 0)),
            "and so is one internal unit short of the other end"
        );
        assert!(is_interior(from, to, Point::new(5_000, 0)));
    }

    #[test]
    fn a_point_off_the_wire_is_not_on_its_interior() {
        let (from, to) = (Point::new(0, 0), Point::new(10_000, 0));
        assert!(!is_interior(from, to, Point::new(5_000, 1)), "one unit off");
        assert!(!is_interior(from, to, Point::new(-1, 0)), "one unit before");
        assert!(!is_interior(from, to, Point::new(10_001, 0)), "and past it");
    }

    #[test]
    fn a_wire_of_no_length_has_no_interior() {
        let at = Point::new(5_000, 5_000);
        assert!(!is_interior(at, at, at));
    }

    #[test]
    fn a_diagonal_wire_has_an_interior_too() {
        let (from, to) = (Point::new(0, 0), Point::new(10_000, 10_000));
        assert!(is_interior(from, to, Point::new(1, 1)));
        assert!(!is_interior(from, to, Point::new(1, 2)), "and it is exact");
    }
}
