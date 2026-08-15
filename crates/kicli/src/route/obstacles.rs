//! What a route meets on the way, and how each thing must be treated.
//!
//! The table below is `research/wire-routing.md` §3.2, as a closed set. Adding
//! a schematic object type is then a compile error at the one site that must
//! decide about it, which is the point of writing it as an enum rather than as
//! a chain of tests.
//!
//! | Feature | Treatment |
//! |---|---|
//! | symbol body, sheet body | hard block |
//! | another symbol's pin, and one step along its own direction | hard block |
//! | junction, no-connect | hard block |
//! | another net's wire, met along its own axis | hard block — it would render as a connection |
//! | another net's wire, met across it | allowed, costed |
//! | this net's wire | free, and entering it ends the route |
//! | a label or text box | allowed, costed |
//! | within one step of a symbol body | allowed, costed |
//!
//! **A wire's treatment depends on the heading.** Collinear and crossing are
//! not two kinds of wire; they are one wire met two ways. So a cell records the
//! axis a wire runs along, and the query is [`Obstacles::entering`] rather than
//! a test of the point alone.

use crate::geometry::{Point, Rect};
use crate::route::terminal::{Heading, Obstruction};
use crate::route::window::{Cell, Window};
use std::collections::BTreeMap;

/// The axis a wire segment runs along.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Axis {
    /// Along x, at one y.
    Horizontal,
    /// Along y, at one x.
    Vertical,
}

impl Axis {
    /// The axis a heading travels along.
    #[must_use]
    pub fn of(heading: Heading) -> Self {
        match heading {
            Heading::PlusX | Heading::MinusX => Self::Horizontal,
            Heading::PlusY | Heading::MinusY => Self::Vertical,
        }
    }
}

/// One thing a route may meet at a grid point.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Feature {
    /// The body box of a placed symbol.
    SymbolBody(String),
    /// The body box of a child sheet. Its pins are terminals, not obstacles.
    SheetBody(String),
    /// A pin of a symbol this route does not end on.
    ForeignPin(String),
    /// The step a foreign pin needs for its own escape, which a route may not
    /// take from it.
    PinHalo(String),
    /// A junction dot.
    Junction(String),
    /// A no-connect marker.
    NoConnect(String),
    /// A wire of another net.
    ForeignWire {
        /// What to call it in a report.
        handle: String,
        /// The axis it runs along.
        axis: Axis,
    },
    /// A wire of the net being routed.
    OwnWire(String),
    /// The bounding box of a label or a text item.
    TextBox(String),
    /// A grid point within one step of a symbol body.
    NearBody(String),
}

/// How a route may use one grid point.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Treatment {
    /// The route may not enter.
    Block,
    /// The route may pass, and pays for a crossing.
    Cross,
    /// The route may pass, and pays for a step through text.
    Text,
    /// The route may pass, and pays for crowding a symbol.
    Near,
    /// The route may enter, and ends there.
    Terminate,
}

impl Feature {
    /// What to call this in a report.
    #[must_use]
    pub fn handle(&self) -> &str {
        match self {
            Self::SymbolBody(handle)
            | Self::SheetBody(handle)
            | Self::ForeignPin(handle)
            | Self::PinHalo(handle)
            | Self::Junction(handle)
            | Self::NoConnect(handle)
            | Self::OwnWire(handle)
            | Self::TextBox(handle)
            | Self::NearBody(handle)
            | Self::ForeignWire { handle, .. } => handle,
        }
    }

    /// How a route travelling this way must treat the feature.
    ///
    /// Every arm but the wire answers the same for any heading. A wire of
    /// another net met along its own axis would draw on top of it and read as
    /// a connection, so it blocks; met across, it is a crossing and is costed.
    #[must_use]
    pub fn treatment(&self, heading: Heading) -> Treatment {
        match self {
            Self::SymbolBody(_)
            | Self::SheetBody(_)
            | Self::ForeignPin(_)
            | Self::PinHalo(_)
            | Self::Junction(_)
            | Self::NoConnect(_) => Treatment::Block,
            Self::ForeignWire { axis, .. } => {
                if *axis == Axis::of(heading) {
                    Treatment::Block
                } else {
                    Treatment::Cross
                }
            }
            Self::OwnWire(_) => Treatment::Terminate,
            Self::TextBox(_) => Treatment::Text,
            Self::NearBody(_) => Treatment::Near,
        }
    }
}

/// What a route pays, or is refused, for one step onto a grid point.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Verdict {
    /// What refuses the step, when something does.
    pub blocked_by: Option<String>,
    /// The wire the step lands on, which ends the route.
    pub terminates_on: Option<String>,
    /// Crossings of another net's wire.
    pub crossings: u32,
    /// Steps inside a label or text box.
    pub text_steps: u32,
    /// Steps within one grid step of a symbol body.
    pub near_steps: u32,
}

impl Verdict {
    /// May a route take this step?
    #[must_use]
    pub fn is_allowed(&self) -> bool {
        self.blocked_by.is_none()
    }
}

/// A wire already on the sheet.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Segment {
    /// What to call it in a report.
    pub handle: String,
    /// One end.
    pub from: Point,
    /// The other.
    pub to: Point,
    /// Is it part of the net being routed?
    pub own_net: bool,
}

/// A pin the route must keep away from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PinObstacle {
    /// What to call it in a report, such as `R7.1`.
    pub handle: String,
    /// Where it connects.
    pub at: Point,
    /// Which way its own escape runs, when it has one.
    pub escape: Option<Heading>,
}

/// The geometry of one sheet, as the router needs it.
///
/// The caller sorts the sheet's objects into these lists, because deciding
/// which wires belong to the net being routed is connectivity's work and not
/// the search's. The search then knows nothing of files or nets.
#[derive(Clone, Copy, Debug, Default)]
pub struct SheetGeometry<'a> {
    /// The body boxes of placed symbols.
    pub symbol_bodies: &'a [Obstruction],
    /// The body boxes of child sheets.
    pub sheet_bodies: &'a [Obstruction],
    /// Pins of symbols the route does not end on.
    pub pins: &'a [PinObstacle],
    /// Junction dots.
    pub junctions: &'a [Obstruction],
    /// No-connect markers.
    pub no_connects: &'a [Obstruction],
    /// Wires and buses already drawn.
    pub segments: &'a [Segment],
    /// The boxes of labels and text.
    pub texts: &'a [Obstruction],
}

/// Every feature of a sheet, keyed by the grid point it sits on.
///
/// Built once per route request in one pass over the objects, so a query is a
/// map lookup rather than a walk. The map is ordered, because an unordered one
/// would let iteration order reach a decision.
#[derive(Clone, Debug)]
pub struct Obstacles {
    window: Window,
    cells: BTreeMap<Cell, Vec<Feature>>,
}

impl Obstacles {
    /// Sort a sheet's geometry onto the lattice of a window.
    #[must_use]
    pub fn build(window: Window, sheet: &SheetGeometry) -> Self {
        let mut map = Self {
            window,
            cells: BTreeMap::new(),
        };
        for body in sheet.symbol_bodies {
            map.fill(body.area, Feature::SymbolBody, &body.handle);
            map.fill_ring(body.area, &body.handle);
        }
        for body in sheet.sheet_bodies {
            map.fill(body.area, Feature::SheetBody, &body.handle);
        }
        for text in sheet.texts {
            map.fill(text.area, Feature::TextBox, &text.handle);
        }
        for mark in sheet.junctions {
            map.fill(mark.area, Feature::Junction, &mark.handle);
        }
        for mark in sheet.no_connects {
            map.fill(mark.area, Feature::NoConnect, &mark.handle);
        }
        for pin in sheet.pins {
            map.add(pin.at, Feature::ForeignPin(pin.handle.clone()));
            if let Some(heading) = pin.escape {
                let halo = heading.step(pin.at, window.grid());
                map.add(halo, Feature::PinHalo(pin.handle.clone()));
            }
        }
        for segment in sheet.segments {
            map.lay(segment);
        }
        map
    }

    /// What a route travelling this way pays for stepping onto a point.
    ///
    /// A point outside the window is refused: the window is already clipped to
    /// the page, so its edge is the page border.
    #[must_use]
    pub fn entering(&self, at: Point, heading: Heading) -> Verdict {
        let Some(cell) = self.window.cell(at) else {
            return Verdict {
                blocked_by: Some("page border".to_owned()),
                ..Verdict::default()
            };
        };
        let mut verdict = Verdict::default();
        for feature in self.features(cell) {
            match feature.treatment(heading) {
                Treatment::Block => {
                    verdict
                        .blocked_by
                        .get_or_insert_with(|| feature.handle().to_owned());
                }
                Treatment::Cross => verdict.crossings += 1,
                Treatment::Text => verdict.text_steps += 1,
                Treatment::Near => verdict.near_steps += 1,
                Treatment::Terminate => {
                    verdict
                        .terminates_on
                        .get_or_insert_with(|| feature.handle().to_owned());
                }
            }
        }
        verdict
    }

    /// Everything on one cell, in the order it was laid down.
    #[must_use]
    pub fn features(&self, cell: Cell) -> &[Feature] {
        self.cells.get(&cell).map_or(&[], Vec::as_slice)
    }

    /// The window this map covers.
    #[must_use]
    pub fn window(&self) -> Window {
        self.window
    }

    /// Put a feature on the cell a point names, when the window holds it.
    fn add(&mut self, at: Point, feature: Feature) {
        if let Some(cell) = self.window.cell(at) {
            self.cells.entry(cell).or_default().push(feature);
        }
    }

    /// Put a feature on every cell of an area.
    fn fill(&mut self, area: Rect, feature: impl Fn(String) -> Feature, handle: &str) {
        for at in self.points_in(area) {
            self.add(at, feature(handle.to_owned()));
        }
    }

    /// Mark the ring one grid step outside an area as crowded.
    ///
    /// Breathing room around a symbol is a preference and not a rule, so it is
    /// costed rather than blocked.
    fn fill_ring(&mut self, area: Rect, handle: &str) {
        let grid = self.window.grid();
        for at in self.points_in(area.inflate(grid)) {
            if !area.contains(at) {
                self.add(at, Feature::NearBody(handle.to_owned()));
            }
        }
    }

    /// Put a segment on every cell it passes through.
    ///
    /// A diagonal segment is left out. The corpus holds a handful and the
    /// lattice has no way to describe one, so pretending otherwise would put a
    /// wire on cells it does not touch.
    fn lay(&mut self, segment: &Segment) {
        let grid = self.window.grid();
        let (from, to) = (segment.from, segment.to);
        let axis = if from.y == to.y {
            Axis::Horizontal
        } else if from.x == to.x {
            Axis::Vertical
        } else {
            return;
        };
        let steps = match axis {
            Axis::Horizontal => (to.x.0 - from.x.0).abs() / grid.0,
            Axis::Vertical => (to.y.0 - from.y.0).abs() / grid.0,
        };
        let heading = match axis {
            Axis::Horizontal if to.x.0 >= from.x.0 => Heading::PlusX,
            Axis::Horizontal => Heading::MinusX,
            Axis::Vertical if to.y.0 >= from.y.0 => Heading::PlusY,
            Axis::Vertical => Heading::MinusY,
        };
        for step in 0..=steps {
            let at = heading.step(from, crate::geometry::Iu(step * grid.0));
            let feature = if segment.own_net {
                Feature::OwnWire(segment.handle.clone())
            } else {
                Feature::ForeignWire {
                    handle: segment.handle.clone(),
                    axis,
                }
            };
            self.add(at, feature);
        }
    }

    /// Every grid point of the window inside an area.
    ///
    /// The points are the **window's** lattice, not the page's. A body box is
    /// under no obligation to land on the grid, so the first point is the box's
    /// start corner rounded up to the lattice, and the walk stops at the last
    /// point the box holds.
    fn points_in(&self, area: Rect) -> Vec<Point> {
        let grid = self.window.grid().0;
        let window = self.window.area();
        let start = Point::new(
            area.start().x.0.max(window.start().x.0),
            area.start().y.0.max(window.start().y.0),
        );
        let end = Point::new(
            area.end().x.0.min(window.end().x.0),
            area.end().y.0.min(window.end().y.0),
        );
        let first = self.window.point(Cell { column: 0, row: 0 });
        let align = |value: i32, origin: i32| -> i32 {
            let offset = value - origin;
            origin + offset.div_euclid(grid) * grid + i32::from(offset.rem_euclid(grid) != 0) * grid
        };
        let mut found = Vec::new();
        let mut y = align(start.y.0, first.y.0);
        while y <= end.y.0 {
            let mut x = align(start.x.0, first.x.0);
            while x <= end.x.0 {
                found.push(Point::new(x, y));
                x += grid;
            }
            y += grid;
        }
        found
    }
}

#[cfg(test)]
mod tests {
    //! The one rule of this module that no drawing can exercise.
    //!
    //! `points_in` walks the **window's** lattice rather than the page's. Every
    //! window a route request can build starts on the grid, because a terminal
    //! off the grid is refused and the page starts at the origin, so on any
    //! real drawing the two alignments agree and neither is measured. They stop
    //! agreeing the moment a window starts off the grid, and [`Window::cell`]
    //! already defines the lattice that way, so a body sorted onto the page's
    //! grid instead would name no cell at all and land nowhere.
    //!
    //! Everything else here is measured on probe drawings, where the geometry
    //! is KiCad's own rather than this test's.

    use super::{Feature, Obstacles, SheetGeometry};
    use crate::geometry::{GRID, Iu, Point, Rect};
    use crate::route::terminal::Obstruction;
    use crate::route::window::Window;

    #[test]
    fn the_lattice_belongs_to_the_window() {
        // A window whose corner misses the grid by a third of a step.
        let step = GRID.0;
        let corner = Point::new(step / 3, step / 3);
        let page = Rect::new(Point::default(), Point::new(40 * step, 40 * step));
        let window = Window::around(corner, Point::new(20 * step, 20 * step), Iu(0), page, GRID);
        assert_eq!(window.area().start(), corner);
        assert!(
            !corner.is_on_grid(),
            "the window starts off the page's grid"
        );

        let body = [Obstruction {
            handle: "U1".to_owned(),
            area: Rect::new(
                Point::new(corner.x.0 + step, corner.y.0 + step),
                Point::new(corner.x.0 + 3 * step, corner.y.0 + 3 * step),
            ),
        }];
        let map = Obstacles::build(
            window,
            &SheetGeometry {
                symbol_bodies: &body,
                ..SheetGeometry::default()
            },
        );

        // Nine points of the window's own lattice, none of them on the page's.
        let mut marked = Vec::new();
        for column in 0..=20 {
            for row in 0..=20 {
                let cell = super::Cell { column, row };
                if map
                    .features(cell)
                    .iter()
                    .any(|feature| matches!(feature, Feature::SymbolBody(_)))
                {
                    marked.push(window.point(cell));
                }
            }
        }
        assert_eq!(marked.len(), 9, "{marked:?}");
        for point in &marked {
            assert!(body[0].area.contains(*point), "{point} is outside the body");
            assert!(!point.is_on_grid(), "{point} is on the page's grid");
        }
    }
}
