//! `KI-WIRE-001` — a wire crosses a symbol body.
//!
//! A wire drawn across a symbol's body reads as a connection that is not one,
//! and it hides the pins it is drawn over. It is the same family as
//! `KI-CONN-001`: the drawing means something other than what it shows, which
//! is the class `spec/SPEC.md` D3 names as motivating kicli.
//!
//! # The exclusion is the rule
//!
//! **A wire that legitimately ends on a pin can enter the body box**, because
//! the body box is the union of the symbol's graphics *and its pin segments*
//! (`geometry::symbol_box`), so a pin's connection point is inside that box
//! whenever some other contributor reaches further out along the pin's own
//! axis. A rule that tested only "does the wire meet the box" would fire on a
//! correct drawing. So the published definition has two halves and both are
//! load-bearing:
//!
//! > clip `seg(w)` to `body(s)`; finding if the clipped length > 0 **and
//! > neither endpoint of the clipped part is within 1 IU of a pin of `s`**.
//!
//! The two halves are **independently** falsifiable here, which is why the
//! checks are built the way they are:
//!
//! - a wire ending on an **inset** pin clips to a part of positive length, so
//!   only the proximity half suppresses it;
//! - a wire ending on a pin that sits **on** the box edge, and a wire that
//!   grazes a **corner**, clip to a single point, so only the length half
//!   suppresses them.
//!
//! # Where this under-reports, and it never over-reports
//!
//! Four gaps, all in the quiet direction, which is the one that costs least for
//! a blocking rule: a rule that blocked a build on a correct drawing would be
//! worse than one that stayed silent.
//!
//! 1. **A wire that enters at a pin and leaves through the far side of the body
//!    is not reported.** The published formula discards the whole finding when
//!    *either* endpoint of the clipped part is near a pin, and such a wire has
//!    one endpoint at the pin. The informal half of the published rule says
//!    *"excluding the ≤ 1 G stub at each end"* — excluding the **stub**, not the
//!    finding — so the formal restatement is weaker than the sentence it
//!    formalises. Recorded as a PROPOSED item in the task entry rather than
//!    silently repaired, because narrowing an exclusion is a value judgement
//!    and `tasks/M5/RULES.md` parks those.
//! 2. **Buses are not read.** The published knob is `wire.through_symbol` and
//!    the rule is named for a wire. A bus across a body is the same visual
//!    defect and is left to a rule that says so.
//! 3. **A symbol the file embeds no definition for draws no body**, so it is
//!    skipped. That is the same omission every geometry rule makes.
//! 4. **A wire of zero length is never reported.** It has no length to clip,
//!    whatever box it sits in.
//!
//! And one place it is deliberately *loud*: a wire drawn along the body box's
//! own edge clips to a part of positive length and is reported. The box is a
//! bounding box rather than an outline, so this covers a wire laid over the
//! symbol's outline and a wire laid tangent along its pin stubs. Both draw over
//! the symbol, and the published arithmetic reports both.
//!
//! # Integer geometry
//!
//! Constitution §4. The clip is Liang–Barsky with the parameter comparisons
//! done as cross multiplications, so no coordinate passes through a float. The
//! clipped endpoints are **rational** where a diagonal wire meets the box, and
//! they are never evaluated: the proximity test multiplies through by the
//! denominator and compares squared distances in `i128`.

use crate::geometry::{Iu, Point, Rect, resolve_pins, symbol_boxes};
use crate::lint::gate::{Counted, Saturation};
use crate::lint::{Drawing, Findings, Rule, RuleId, Tier};
use crate::model::items::{Item, Line, LineKind, Symbol, Uuid};

/// How near a pin an endpoint of the clipped part may be, in internal units.
///
/// The published definition says one internal unit, which is the smallest
/// distance a KiCad file can record: `geometry::UNITS_PER_MM` is ten thousand
/// and KiCad writes four decimals. A wire end and a pin that mean to meet are
/// therefore equal, and this is the slack the published rule states rather than
/// a tolerance this file chose.
const NEAR_A_PIN: i128 = 1;

/// A wire crosses a symbol body.
pub struct WireCrossesBody;

impl Rule for WireCrossesBody {
    fn id(&self) -> RuleId {
        RuleId("KI-WIRE-001")
    }

    fn tier(&self) -> Tier {
        Tier::One
    }

    /// This rule counts wires, and the declaration is true even though it
    /// cannot change an outcome today.
    ///
    /// **What it counts is exact.** The rule reports at most once per wire
    /// segment — see [`WireCrossesBody::examine`] — and
    /// [`Counted::Wires`] is the wire segments of the sheet. So the count and
    /// the denominator are the same objects, and `count <= total` always. A
    /// sheet on which every wire crosses a body gives the whole share, which is
    /// the shape `tasks/M5/RULES.md` records the saturation property for.
    ///
    /// **It changes no outcome while this rule is Tier 1**, and that is worth
    /// stating plainly rather than leaving a reader to find it in
    /// [`crate::lint::gate::Gate::of`]: a blocking rule fails the gate on its
    /// first finding, so the share is never asked for. `KI-CONN-001`'s author
    /// reached the same conclusion and declared [`Saturation::NEVER`] on the
    /// strength of it.
    ///
    /// This file declares the share anyway, and the difference between the two
    /// cases is the honest reason. `KI-CONN-001` counts **pin connection
    /// points**, which [`Counted`] cannot name, so declaring a denominator
    /// there would have been a false declaration. Here the denominator is
    /// exactly right, so `NEVER` would be the false one — and the declaration
    /// becomes live the moment a tier review moves this rule, which is a thing
    /// Phase 3 is chartered to do.
    fn saturation(&self) -> Saturation {
        Saturation::of(Counted::Wires)
    }

    /// One finding per wire, naming the first body it crosses in file order.
    ///
    /// **One per wire, not one per (wire, symbol) pair.** The repair is to
    /// re-route that one wire, and a wire drawn across three symbols needs it
    /// once; Constitution §6 says a view that floods is wrong whatever it
    /// contains. It is also what makes [`WireCrossesBody::saturation`]'s
    /// denominator exact rather than a bound.
    ///
    /// No fix command is offered. The repair is a route, and choosing a route
    /// is a layout decision — the same line `KI-CONN-001` draws when it
    /// declines to suggest moving a symbol.
    fn examine(&self, drawing: &Drawing<'_>, found: &mut Findings<'_>) {
        let bodies = bodies_of(drawing);
        for wire in wires_of(drawing) {
            let crossed: Vec<&Placement> = bodies
                .iter()
                .filter(|body| crosses(wire.from, wire.to, body))
                .collect();
            let Some(first) = crossed.first() else {
                continue;
            };
            let others = crossed.len() - 1;
            let also = if others == 0 {
                String::new()
            } else if others == 1 {
                " and 1 other".to_owned()
            } else {
                format!(" and {others} others")
            };
            found.record(
                first.body.centre(),
                vec![wire.uuid.clone(), first.object.clone()],
                format!("this wire crosses the body of {}{also}", first.name),
            );
        }
    }
}

/// One placed symbol, as this rule reads it.
struct Placement {
    /// The body box in schematic coordinates: the graphics and the pin
    /// segments, and no text. `spec/SPEC.md` §8's two-box model — the **body**
    /// box, as `KI-OVL-001` uses, and not the full box `KI-TXT-001` uses.
    body: Rect,
    /// Every pin connection point of this placement, in schematic coordinates.
    pins: Vec<Point>,
    /// What a finding calls the symbol: its reference designator on this sheet,
    /// or its library name when the placement records no instance here.
    name: String,
    /// What a finding names: the symbol.
    object: Uuid,
}

/// The wire segments of one drawing, in file order.
///
/// Buses are left out, for the reason the module header gives.
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

/// Every symbol of this placement that draws a body, in file order.
///
/// **Hidden pins are kept.** The published exclusion says *"a pin of `s`"*, and
/// a hidden power pin is a pin of `s`: it connects. The body box leaves hidden
/// pins out, so a hidden pin can sit inside the box and suppress a crossing
/// that passes within one internal unit of it. That is a missed finding rather
/// than a false one, which is the direction this rule errs in everywhere.
fn bodies_of(drawing: &Drawing<'_>) -> Vec<Placement> {
    let sheet = drawing.path();
    let mut bodies = Vec::new();
    for item in &drawing.schematic().items {
        let Item::Symbol(symbol) = item else {
            continue;
        };
        let Some(definition) = drawing.definition_of(symbol) else {
            continue;
        };
        let drawn: Symbol = symbol.drawn_on(sheet);
        bodies.push(Placement {
            body: symbol_boxes(drawing.doc(), &drawn, definition).body,
            pins: resolve_pins(&drawn, definition)
                .into_iter()
                .map(|pin| pin.position)
                .collect(),
            name: symbol.reference_on(sheet).map_or_else(
                || symbol.lib_id.symbol_name().to_owned(),
                |reference| reference.0.clone(),
            ),
            object: symbol.uuid.clone(),
        });
    }
    bodies
}

/// Does this wire cross this body, as the published rule defines crossing?
///
/// The three refusals are each a separate statement about the drawing, and each
/// is checked on its own in the tests below:
///
/// - a wire of no length has no clipped length, whatever box holds it;
/// - a clipped part of **zero** length is not a crossing, so a corner graze and
///   a wire ending on a pin that sits on the box edge both pass;
/// - a clipped part whose **endpoint** is within one internal unit of a pin of
///   this symbol is the legitimate stub, and that is the exclusion.
fn crosses(from: Point, to: Point, body: &Placement) -> bool {
    if from == to {
        return false;
    }
    let Some((lo, hi)) = clip(from, to, body.body) else {
        return false;
    };
    if !lo.is_before(hi) {
        return false;
    }
    !body
        .pins
        .iter()
        .any(|pin| near(from, to, lo, *pin) || near(from, to, hi, *pin))
}

/// A position along a segment, as an exact fraction of its length.
///
/// Where a diagonal wire meets a box the position is **not** an integer
/// coordinate, so it is held as a fraction and never evaluated. The denominator
/// is kept positive, which is what makes every comparison below a plain cross
/// multiplication.
#[derive(Clone, Copy)]
struct Along {
    numerator: i64,
    denominator: i64,
}

impl Along {
    /// The start of the segment.
    const START: Self = Self {
        numerator: 0,
        denominator: 1,
    };

    /// The end of the segment.
    const END: Self = Self {
        numerator: 1,
        denominator: 1,
    };

    /// A position from a numerator and a denominator of either sign.
    ///
    /// A negative denominator is negated along with its numerator, which names
    /// the same position with the sign convention the comparisons rely on.
    fn of(numerator: i64, denominator: i64) -> Self {
        if denominator < 0 {
            Self {
                numerator: -numerator,
                denominator: -denominator,
            }
        } else {
            Self {
                numerator,
                denominator,
            }
        }
    }

    /// Is this position strictly nearer the start than that one?
    ///
    /// An exact cross multiplication in `i128`. Both denominators are positive,
    /// so the inequality keeps its direction.
    fn is_before(self, other: Self) -> bool {
        i128::from(self.numerator) * i128::from(other.denominator)
            < i128::from(other.numerator) * i128::from(self.denominator)
    }
}

/// Clip a segment to a box, as a pair of positions along the segment.
///
/// Liang–Barsky, in integers. Each of the four edges gives an inequality
/// `p · t <= q`; `p == 0` means the segment is parallel to that edge, which
/// either rules it out altogether or says nothing. A negative `p` moves the
/// near end forward and a positive one moves the far end back.
///
/// Returns nothing when the segment misses the box. Returns a pair that may be
/// equal, which is a segment touching the box at exactly one point: the caller
/// decides what that means, because the published rule does.
fn clip(from: Point, to: Point, box_of: Rect) -> Option<(Along, Along)> {
    let span = |one: Iu, other: Iu| i64::from(other.0) - i64::from(one.0);
    let dx = span(from.x, to.x);
    let dy = span(from.y, to.y);
    let edges = [
        (-dx, span(box_of.start().x, from.x)),
        (dx, span(from.x, box_of.end().x)),
        (-dy, span(box_of.start().y, from.y)),
        (dy, span(from.y, box_of.end().y)),
    ];

    let mut near_end = Along::START;
    let mut far_end = Along::END;
    for (p, q) in edges {
        if p == 0 {
            // Parallel to this edge: outside it for every t, or inside for
            // every t. There is nothing in between to clip.
            if q < 0 {
                return None;
            }
            continue;
        }
        let at = Along::of(q, p);
        if p < 0 {
            if near_end.is_before(at) {
                near_end = at;
            }
        } else if at.is_before(far_end) {
            far_end = at;
        }
    }

    if far_end.is_before(near_end) {
        None
    } else {
        Some((near_end, far_end))
    }
}

/// Is the point at this position along the segment within one unit of `pin`?
///
/// Exact, and with no division. The offset from the pin to the point at
/// `numerator / denominator` along the segment is multiplied through by the
/// denominator, and the squared length of that is compared with the squared
/// slack scaled the same way. Every product is taken in `i128`: a coordinate
/// fits a schematic page, so the numerators and denominators here are bounded
/// by the page diagonal and the squares by its fourth power.
fn near(from: Point, to: Point, at: Along, pin: Point) -> bool {
    let span = |one: Iu, other: Iu| i128::from(other.0) - i128::from(one.0);
    let denominator = i128::from(at.denominator);
    let numerator = i128::from(at.numerator);
    let offset = |start: Iu, end: Iu, target: Iu| {
        denominator * span(target, start) + numerator * span(start, end)
    };
    let x = offset(from.x, to.x, pin.x);
    let y = offset(from.y, to.y, pin.y);
    let slack = NEAR_A_PIN * denominator;
    x * x + y * y <= slack * slack
}

/// The rules this file declares.
pub static RULES: &[&'static dyn Rule] = &[&WireCrossesBody];

#[cfg(test)]
mod tests {
    use super::{Along, Placement, clip, crosses, near};
    use crate::geometry::{Point, Rect};
    use crate::model::items::Uuid;

    /// A ten-by-ten box with its near corner at the origin.
    fn box_of() -> Rect {
        Rect::new(Point::new(0, 0), Point::new(10_000, 10_000))
    }

    /// A body with that box and the pins given.
    fn body(pins: &[Point]) -> Placement {
        Placement {
            body: box_of(),
            pins: pins.to_vec(),
            name: "U1".to_owned(),
            object: Uuid("u1".to_owned()),
        }
    }

    /// The clipped part of a segment, as a pair of fractions.
    fn clipped(from: Point, to: Point) -> Option<((i64, i64), (i64, i64))> {
        clip(from, to, box_of()).map(|(lo, hi)| {
            (
                (lo.numerator, lo.denominator),
                (hi.numerator, hi.denominator),
            )
        })
    }

    #[test]
    fn a_segment_that_misses_the_box_clips_to_nothing() {
        assert_eq!(
            clipped(Point::new(-5, 20_000), Point::new(15_000, 20_000)),
            None
        );
        assert_eq!(clipped(Point::new(-5_000, 0), Point::new(-1, 0)), None);
        assert_eq!(
            clipped(Point::new(20_000, 0), Point::new(30_000, 10_000)),
            None
        );
    }

    #[test]
    fn a_segment_through_the_box_clips_to_the_part_inside_it() {
        // Entering at a fifth of the way along and leaving at three fifths.
        assert_eq!(
            clipped(Point::new(-5_000, 5_000), Point::new(20_000, 5_000)),
            Some(((5_000, 25_000), (15_000, 25_000)))
        );
        // A segment wholly inside keeps both of its own ends.
        assert_eq!(
            clipped(Point::new(1, 1), Point::new(9_999, 9_999)),
            Some(((0, 1), (1, 1)))
        );
    }

    #[test]
    fn a_segment_touching_one_point_of_the_box_clips_to_a_pair_that_is_equal() {
        // The corner graze. A diagonal through the far corner and no further.
        let graze = clipped(Point::new(20_000, 0), Point::new(0, 20_000))
            .expect("the diagonal reaches the corner");
        assert!(!Along::of(graze.0.0, graze.0.1).is_before(Along::of(graze.1.0, graze.1.1)));
        // And one along an edge, ending exactly on it.
        let edge =
            clipped(Point::new(-5_000, 0), Point::new(0, 0)).expect("the segment reaches the edge");
        assert!(!Along::of(edge.0.0, edge.0.1).is_before(Along::of(edge.1.0, edge.1.1)));
    }

    #[test]
    fn the_clip_is_exact_on_a_diagonal_that_divides_by_nothing_whole() {
        // dx is 3 and dy is 7, so neither crossing is a round fraction and a
        // float would be the only way to get one wrong.
        let from = Point::new(-1_000, 2_000);
        let to = Point::new(2_000, 9_000);
        let (lo, hi) = clip(from, to, box_of()).expect("the segment reaches the box");
        // x reaches 0 at a third of the way along.
        assert_eq!((lo.numerator, lo.denominator), (1_000, 3_000));
        // and the far end is the segment's own, which is inside the box.
        assert_eq!((hi.numerator, hi.denominator), (1, 1));
    }

    #[test]
    fn an_endpoint_is_near_a_pin_at_one_unit_and_not_at_two() {
        let (from, to) = (Point::new(-10_000, 5_000), Point::new(10_000, 5_000));
        let end = Along::END;
        assert!(
            near(from, to, end, Point::new(10_000, 5_000)),
            "exactly on it"
        );
        assert!(
            near(from, to, end, Point::new(10_001, 5_000)),
            "one unit along"
        );
        assert!(
            near(from, to, end, Point::new(10_000, 4_999)),
            "one unit across"
        );
        assert!(
            !near(from, to, end, Point::new(10_002, 5_000)),
            "two units is not within one"
        );
        assert!(
            !near(from, to, end, Point::new(10_001, 5_001)),
            "a unit on each axis is further than one unit"
        );
    }

    #[test]
    fn a_rational_endpoint_is_measured_without_being_evaluated() {
        // The segment enters the box at x = 0, which is two sevenths of the way
        // along: y there is 3_000 + 2/7 * 7_000, which is 5_000 exactly, and
        // the fraction is not one a float holds exactly.
        let from = Point::new(-2_000, 3_000);
        let to = Point::new(5_000, 10_000);
        let (lo, _) = clip(from, to, box_of()).expect("the segment reaches the box");
        assert_eq!((lo.numerator, lo.denominator), (2_000, 7_000));
        assert!(near(from, to, lo, Point::new(0, 5_000)), "the entry point");
        assert!(!near(from, to, lo, Point::new(0, 5_002)));
    }

    #[test]
    fn a_wire_through_the_body_away_from_every_pin_is_a_crossing() {
        let pins = body(&[Point::new(0, 10_000), Point::new(10_000, 0)]);
        assert!(crosses(
            Point::new(-5_000, 5_000),
            Point::new(15_000, 5_000),
            &pins
        ));
    }

    #[test]
    fn a_wire_ending_on_an_inset_pin_is_not_a_crossing() {
        // The pin sits inside the box, so the clipped part has real length and
        // only the proximity half of the rule can suppress it. This is the one
        // check that proves the exclusion exists.
        let pin = Point::new(4_000, 5_000);
        let pins = body(&[pin]);
        assert!(
            !crosses(Point::new(-5_000, 5_000), pin, &pins),
            "a wire that ends on a pin of this symbol is a connection"
        );
        // The same wire, one unit further on, has left the pin behind.
        assert!(crosses(
            Point::new(-5_000, 5_000),
            Point::new(4_002, 5_000),
            &pins
        ));
    }

    #[test]
    fn the_exclusion_reads_this_symbols_pins_and_no_others() {
        // "a pin of s". The clipped part leaves this body at 10_000,5_000. A
        // pin there belonging to ANOTHER symbol is not in this body's list and
        // does not excuse the crossing; the same point in this body's own list
        // does. The contrast is the check — what suppresses a finding is whose
        // pin it is, and nothing else about the geometry moves between the two
        // halves.
        let leaving = Point::new(10_000, 5_000);
        let wire = (Point::new(-5_000, 5_000), Point::new(15_000, 5_000));
        assert!(
            crosses(wire.0, wire.1, &body(&[Point::new(0, 10_000)])),
            "another symbol's pin does not excuse this crossing"
        );
        assert!(
            !crosses(wire.0, wire.1, &body(&[leaving])),
            "and this symbol's own pin at the same place does"
        );
    }

    #[test]
    fn a_wire_of_no_length_is_never_a_crossing() {
        let inside = Point::new(5_000, 5_000);
        assert!(!crosses(inside, inside, &body(&[])));
    }

    #[test]
    fn a_wire_that_touches_the_body_at_one_point_is_not_a_crossing() {
        // Neither of these is near a pin, so the length half of the rule is the
        // only thing that can suppress them.
        let bare = body(&[]);
        assert!(
            !crosses(Point::new(20_000, 0), Point::new(0, 20_000), &bare),
            "a corner graze has no clipped length"
        );
        assert!(
            !crosses(Point::new(-5_000, 0), Point::new(0, 0), &bare),
            "a wire ending on the box edge has no clipped length"
        );
    }

    #[test]
    fn a_wire_along_the_body_edge_is_a_crossing() {
        // The recorded decision, executable: the body box is a bounding box,
        // and a wire laid along its edge is drawn over the symbol.
        assert!(crosses(
            Point::new(-5_000, 0),
            Point::new(15_000, 0),
            &body(&[])
        ));
    }

    #[test]
    fn the_known_under_report_is_observed_rather_than_unobserved() {
        // Gap 1 of the module header, as a check. A wire that enters at a pin
        // and leaves through the far side of the body is NOT reported, because
        // the published formula discards the finding when either endpoint of
        // the clipped part is near a pin. When the PROPOSED narrowing in the
        // task entry is ruled on, this check goes red and names the decision.
        let pin = Point::new(0, 5_000);
        assert!(!crosses(pin, Point::new(15_000, 5_000), &body(&[pin])));
    }
}
