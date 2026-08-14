//! Where a route is allowed to look.
//!
//! The window is the bounding box of the two terminals, inflated by
//! `routing.margin` and clipped to the page. Everything outside it is off the
//! search: the page border is a hard limit, and a detour wider than the margin
//! is a route no reader would follow.
//!
//! The window also fixes the lattice. Every node is a grid point of the window,
//! addressed by an integer cell index, so a search never compares coordinates
//! for equality and never asks whether a point is on the grid twice.

use crate::geometry::{Iu, Point, Rect};

/// One grid point of a window, as integer indices from its top-left corner.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Cell {
    /// Steps to the right of the window's start corner.
    pub column: i32,
    /// Steps below it.
    pub row: i32,
}

/// The area a route may use, and the lattice over it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Window {
    area: Rect,
    grid: Iu,
}

impl Window {
    /// The window around two terminals.
    ///
    /// The bounding box of the two points, inflated by the margin, clipped to
    /// the page. A margin that would take the window off the page is not an
    /// error: the page wins, because a wire drawn outside it is not on the
    /// drawing.
    ///
    /// # Panics
    ///
    /// If the grid is zero. A lattice with no step has no nodes, and every
    /// caller reads the grid from a configuration that refuses zero.
    #[must_use]
    pub fn around(from: Point, to: Point, margin: Iu, page: Rect, grid: Iu) -> Self {
        assert!(grid.0 > 0, "a lattice needs a grid step");
        let span = Rect::new(from, to).inflate(margin);
        let area = Rect::new(
            Point::new(
                span.start().x.0.max(page.start().x.0),
                span.start().y.0.max(page.start().y.0),
            ),
            Point::new(
                span.end().x.0.min(page.end().x.0),
                span.end().y.0.min(page.end().y.0),
            ),
        );
        Self { area, grid }
    }

    /// The area the window covers, after the page clipped it.
    #[must_use]
    pub fn area(self) -> Rect {
        self.area
    }

    /// The grid step of the lattice.
    #[must_use]
    pub fn grid(self) -> Iu {
        self.grid
    }

    /// The cell a point sits on, when the point is in the window and on grid.
    ///
    /// A point off the grid names no cell. The lattice is exact rather than
    /// approximate — every wire endpoint in the demo corpus is on the grid — so
    /// rounding one here would invent a node that is not there.
    #[must_use]
    pub fn cell(self, at: Point) -> Option<Cell> {
        if !self.area.contains(at) {
            return None;
        }
        let start = self.area.start();
        let (dx, dy) = (at.x.0 - start.x.0, at.y.0 - start.y.0);
        if dx % self.grid.0 != 0 || dy % self.grid.0 != 0 {
            return None;
        }
        Some(Cell {
            column: dx / self.grid.0,
            row: dy / self.grid.0,
        })
    }

    /// Where a cell is on the drawing.
    #[must_use]
    pub fn point(self, cell: Cell) -> Point {
        let start = self.area.start();
        Point::new(
            start.x.0 + cell.column * self.grid.0,
            start.y.0 + cell.row * self.grid.0,
        )
    }

    /// Is the point inside the window?
    #[must_use]
    pub fn holds(self, at: Point) -> bool {
        self.area.contains(at)
    }
}
