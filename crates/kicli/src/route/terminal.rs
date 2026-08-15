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

use crate::geometry::pins::ResolvedPin;
use crate::geometry::{Angle, Iu, Point, Rect};
use crate::model::items::SheetPin;

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
