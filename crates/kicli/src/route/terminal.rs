//! Where a route starts and finishes, and which way it must leave.
//!
//! A terminal is a symbol pin, a sheet pin, or a point on an existing net. The
//! first two fix the direction a wire may leave in; the third does not, because
//! a point on a wire can be met from any side.
//!
//! **The escape rule.** A route must step at least one grid step along the
//! terminal's own direction before it turns, at both ends. A wire that
//! approaches a pin from the side reads wrong, and on a pin whose graphic
//! carries a marker — a clock, an inversion bubble — it draws over the marker.
//! This is a hard constraint and not a cost: a terminal that cannot escape is
//! reported blocked, naming what blocked it.
//!
//! **The four-way rule.** A route's own end is a wire end. A terminus that
//! already carries three of them would take the fourth, and `spec/SPEC.md` §9
//! Q2 rules that refused and offset by one grid step, reporting the adjustment.
//! [`Approach`] is where that happens, before the search runs: it answers with
//! the terminals a route may really use and with the record of what moved. The
//! count it decides on is `edit::mark::wire_ends_at`, which is the one
//! implementation of "the wire ends at this point" and is also what the
//! junction verb's own refusal reads. `tests/the_four_way_rule_has_one_home.rs`
//! is the check that fails when a second one appears.

use crate::edit::mark::wire_ends_at;
use crate::geometry::pins::ResolvedPin;
use crate::geometry::{Angle, Iu, Point, Rect};
use crate::model::items::{Schematic, SheetPin};
use crate::route::report::{Adjusted, Adjustment};

/// Which way a route runs from a point.
///
/// The names are the axes rather than the compass, because the schematic's Y
/// grows downwards and "up" would have to be explained at every use.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Heading {
    /// Towards larger x.
    PlusX,
    /// Towards smaller x.
    MinusX,
    /// Towards larger y, which is down the page.
    PlusY,
    /// Towards smaller y, which is up the page.
    MinusY,
}

impl Heading {
    /// Every heading, in the order `research/wire-routing.md` §4.3 expands
    /// them: `+x, −x, +y, −y`.
    ///
    /// One order in the router rather than two. The search expands its
    /// successors in it, the U-shaped candidates take their four sides in it,
    /// and it is the order the derived comparison gives — which is what settles
    /// a tie in the search's queue, so the two must agree. The test below holds
    /// them together.
    pub const EVERY: [Self; 4] = [Self::PlusX, Self::MinusX, Self::PlusY, Self::MinusY];

    /// The heading an angle in schematic sense names.
    ///
    /// Schematic sense is the one [`ResolvedPin::direction`] uses: 0 towards
    /// larger x, 90 towards larger y, 180 and 270 the reverses. An angle off
    /// the quarter turn names no heading.
    #[must_use]
    pub fn from_schematic_angle(angle: Angle) -> Option<Self> {
        match angle.0.rem_euclid(360) {
            0 => Some(Self::PlusX),
            90 => Some(Self::PlusY),
            180 => Some(Self::MinusX),
            270 => Some(Self::MinusY),
            _ => None,
        }
    }

    /// The heading from one point to another.
    ///
    /// A pair of points that is diagonal, or that is the same point twice,
    /// names no heading.
    #[must_use]
    pub fn between(from: Point, to: Point) -> Option<Self> {
        match (to.x.0 - from.x.0, to.y.0 - from.y.0) {
            (0, 0) => None,
            (0, dy) if dy > 0 => Some(Self::PlusY),
            (0, _) => Some(Self::MinusY),
            (dx, 0) if dx > 0 => Some(Self::PlusX),
            (_, 0) => Some(Self::MinusX),
            _ => None,
        }
    }

    /// The opposite heading.
    #[must_use]
    pub fn reversed(self) -> Self {
        match self {
            Self::PlusX => Self::MinusX,
            Self::MinusX => Self::PlusX,
            Self::PlusY => Self::MinusY,
            Self::MinusY => Self::PlusY,
        }
    }

    /// One step of the given distance from a point.
    #[must_use]
    pub fn step(self, from: Point, distance: Iu) -> Point {
        match self {
            Self::PlusX => Point::new(from.x.0 + distance.0, from.y.0),
            Self::MinusX => Point::new(from.x.0 - distance.0, from.y.0),
            Self::PlusY => Point::new(from.x.0, from.y.0 + distance.0),
            Self::MinusY => Point::new(from.x.0, from.y.0 - distance.0),
        }
    }
}

/// One end of a route.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Terminal {
    /// Where the route must start or finish.
    pub at: Point,
    /// Which way a route must leave, when the terminal fixes one.
    ///
    /// A point on an existing net fixes none: a wire may meet it from any
    /// side.
    pub escape: Option<Heading>,
    /// What to call this terminal in a report, such as `R12.1`.
    pub name: String,
}

impl Terminal {
    /// The terminal a placed symbol's pin makes.
    ///
    /// A pin's stored angle points from its connection point **towards the
    /// symbol body**: `Device:R` draws pin 1 at `(0, 3.81)` with angle 270 and
    /// length 1.27, whose root therefore lands on the body rectangle's top
    /// edge at `y = 2.54`. So a wire leaves along the reverse of that angle.
    /// See `SCH_PIN::GetPinRoot` in `eeschema/sch_pin.cpp`, with the file
    /// angle mapped to an orientation at
    /// `sch_io_kicad_sexpr_parser.cpp:1673` (0 right, 90 up, 180 left,
    /// 270 down).
    #[must_use]
    pub fn of_pin(reference: &str, pin: &ResolvedPin) -> Self {
        Self {
            at: pin.position,
            escape: Heading::from_schematic_angle(pin.direction).map(Heading::reversed),
            name: format!("{reference}.{}", pin.number),
        }
    }

    /// The terminal a sheet pin makes.
    ///
    /// A sheet pin's angle names the **edge it sits on** rather than the
    /// direction of its graphic, and it counts anticlockwise from the right
    /// with y upwards: 0 is the right edge, 90 the top, 180 the left, 270 the
    /// bottom (`sch_io_kicad_sexpr_parser.cpp:2440`, `parseSchSheetPin`). A
    /// wire leaves outwards, away from the sheet body, so the heading is the
    /// angle reflected into the schematic's y-down sense.
    ///
    /// **Measured against the running tool, 2026-08-15, kicad-cli 10.0.5**
    /// (`tests/edit_wire_sheet_pin.rs`), which is what this rule rests on now:
    /// it was read from KiCad's parser until then and is no longer only that.
    /// One drawing, four ports, one stub wire leaving each port outwards to a
    /// resistor pin, built three times over the same written port positions.
    /// With each angle naming the edge its port is written on, all four stubs
    /// carry through to the child sheet. With each angle naming the opposite
    /// edge — same positions, same wires — all four stubs read `unconnected`.
    /// With those same reflected angles and each stub moved to the far edge,
    /// the along-edge coordinate kept, all four join again exactly as in the
    /// first drawing: so KiCad puts the port's **connection point** on the edge
    /// the angle names rather than where the file wrote it, which the second
    /// drawing alone could not tell from a port that merely refused an
    /// approach. A symbol-pin net in the same drawing is joined in all three,
    /// so the instrument was working when it reported the break.
    #[must_use]
    pub fn of_sheet_pin(pin: &SheetPin) -> Self {
        let escape = match pin.angle.0.rem_euclid(360) {
            0 => Some(Heading::PlusX),
            90 => Some(Heading::MinusY),
            180 => Some(Heading::MinusX),
            270 => Some(Heading::PlusY),
            _ => None,
        };
        Self {
            at: pin.at,
            escape,
            name: pin.name.clone(),
        }
    }

    /// The terminal a point on an existing net makes.
    ///
    /// It fixes no direction: a route may meet a wire from any side, and which
    /// side is cheapest is the cost model's business rather than a rule.
    #[must_use]
    pub fn of_point(at: Point, name: &str) -> Self {
        Self {
            at,
            escape: None,
            name: name.to_owned(),
        }
    }

    /// The point a route must reach before it may turn.
    ///
    /// A terminal that fixes no direction escapes to itself.
    #[must_use]
    pub fn escape_point(&self, grid: Iu) -> Point {
        match self.escape {
            Some(heading) => heading.step(self.at, grid),
            None => self.at,
        }
    }

    /// Is this terminal on the placement grid?
    ///
    /// A lattice route can only start where the lattice has a node. An
    /// off-grid pin is a drawing fault the lint reports; the router refuses it
    /// rather than snapping, because moving somebody's pin is not a routing
    /// decision.
    #[must_use]
    pub fn is_on_grid(&self, grid: Iu) -> bool {
        grid.0 != 0 && self.at.x.0 % grid.0 == 0 && self.at.y.0 % grid.0 == 0
    }
}

/// Something a route may not pass through, and what to call it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Obstruction {
    /// What to call it in a report: a reference designator or an identifier.
    pub handle: String,
    /// The area it occupies.
    pub area: Rect,
}

/// Why a route cannot leave a terminal.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{terminal} cannot escape: {handle} is in the way at {at}")]
pub struct BlockedEscape {
    /// The terminal that is boxed in.
    pub terminal: String,
    /// What is in the way.
    pub handle: String,
    /// Where the route would have had to step.
    pub at: Point,
}

/// The escape point of a terminal, or what blocks it.
///
/// The check is the escape point alone. Everything past it is the search's
/// business, and a terminal whose first step is blocked has no route at all —
/// so this answer is worth having before any search runs.
///
/// # Errors
///
/// Returns [`BlockedEscape`] naming the first obstruction that covers the
/// escape point, in the order the caller gave them.
pub fn escape(
    terminal: &Terminal,
    grid: Iu,
    obstructions: &[Obstruction],
) -> Result<Point, BlockedEscape> {
    let at = terminal.escape_point(grid);
    if terminal.escape.is_none() {
        return Ok(at);
    }
    match obstructions
        .iter()
        .find(|obstruction| obstruction.area.contains(at))
    {
        Some(blocker) => Err(BlockedEscape {
            terminal: terminal.name.clone(),
            handle: blocker.handle.clone(),
            at,
        }),
        None => Ok(at),
    }
}

/// How many wire ends must already meet at a point for a route's own end to be
/// the fourth.
///
/// Three. [`crate::edit::mark`] refuses a junction on the ends that **are**
/// there; this asks about the ends there **would be** once the route arrives,
/// so it is that boundary less the one end the route brings. Neither constant
/// is written in terms of the other — they are two thresholds on one
/// measurement, approached from opposite directions — and both read the count
/// from [`wire_ends_at`], which is the single implementation of it.
const CROWDED: usize = 3;

/// Where a route will really begin and end, and which terminals moved.
///
/// A route is asked for between two terminals and may not be drawable between
/// exactly those two points: ending on a point that already carries enough wire
/// ends for the route's own to be the fourth would draw the junction
/// `spec/SPEC.md` §9 Q2 refuses. So the terminals are settled against the
/// drawing before the search sees them, and what moved is reported rather than
/// left for the reader to notice.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Approach {
    /// The terminal the route leaves, after any adjustment.
    pub source: Terminal,
    /// The terminal it arrives at, after any adjustment.
    pub target: Terminal,
    /// The terminals that moved, in the order the two were given.
    ///
    /// Empty when neither moved, which is the common answer. This is what a
    /// caller copies into [`Report::adjusted`], so an agent reads which end
    /// moved and by how far without comparing coordinates.
    ///
    /// [`Report::adjusted`]: crate::route::report::Report::adjusted
    pub adjusted: Vec<Adjusted>,
}

impl Approach {
    /// The terminals a route may use on this drawing.
    ///
    /// Both ends are asked, because both ends of a route are wire ends: a
    /// source standing on a crowded point brings the fourth just as a target
    /// does.
    #[must_use]
    pub fn of(source: &Terminal, target: &Terminal, schematic: &Schematic, grid: Iu) -> Self {
        let mut adjusted = Vec::new();
        let moved_source = settle(source, schematic, grid, &mut adjusted);
        let moved_target = settle(target, schematic, grid, &mut adjusted);
        Self {
            source: moved_source,
            target: moved_target,
            adjusted,
        }
    }
}

/// One terminal, moved off a four-way point if it stands on one, and recorded.
///
/// The move and the record are made together and from one answer, because a
/// route that moved without saying so and a route that said so without moving
/// are two different defects and both are silent.
fn settle(
    terminal: &Terminal,
    schematic: &Schematic,
    grid: Iu,
    adjusted: &mut Vec<Adjusted>,
) -> Terminal {
    let Some(at) = clear_of_four_way(terminal.at, schematic, grid) else {
        return terminal.clone();
    };
    adjusted.push(Adjusted {
        terminal: terminal.name.clone(),
        by: at - terminal.at,
        why: Adjustment::FourWayJunction,
    });
    // The terminal is now a point on a wire rather than the pin or port it was
    // asked for, and a point on a wire may be met from any side. Its name is
    // kept, because the report names the end that moved as it names `from` and
    // `to`.
    Terminal::of_point(at, &terminal.name)
}

/// Where a terminus goes instead, when ending where it was asked would make a
/// fourth wire end meet at one point.
///
/// Nothing, when the point is not crowded, which is the common answer.
///
/// The step is one grid step **along a wire that already meets the point**, so
/// the terminus stays on the thing the route was drawn to reach: a step into
/// empty space would move the wire off its own net, which is a worse drawing
/// than the one being avoided. The direction is taken in [`Heading::EVERY`]
/// order rather than in the order the file happens to list its wires, because
/// KiCad reorders items when it saves and a decision made on file order gives
/// one drawing two answers. A step that would land on another crowded point is
/// passed over for the next direction.
fn clear_of_four_way(at: Point, schematic: &Schematic, grid: Iu) -> Option<Point> {
    if wire_ends_at(schematic, at).len() < CROWDED {
        return None;
    }
    let along: Vec<Heading> = wire_ends_at(schematic, at)
        .iter()
        .filter_map(|end| Heading::between(at, end.far))
        .collect();
    Heading::EVERY
        .into_iter()
        .filter(|heading| along.contains(heading))
        .map(|heading| heading.step(at, grid))
        .find(|step| wire_ends_at(schematic, *step).len() < CROWDED)
}

#[cfg(test)]
mod tests {
    use super::Heading;

    #[test]
    fn every_heading_is_in_the_expansion_order() {
        for heading in Heading::EVERY {
            // The match is exhaustive, so a fifth heading is a compile error
            // here, and its arm is what puts it in the list.
            let place = match heading {
                Heading::PlusX => 0_usize,
                Heading::MinusX => 1,
                Heading::PlusY => 2,
                Heading::MinusY => 3,
            };
            assert_eq!(Heading::EVERY[place], heading);
        }
        // And the list is the order the derived comparison gives, which is the
        // last rung of the search's queue order.
        let mut sorted = Heading::EVERY;
        sorted.sort_unstable();
        assert_eq!(sorted, Heading::EVERY);
    }
}
