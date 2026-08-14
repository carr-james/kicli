//! Integer geometry: transforms, pin positions, bounding boxes, and text extents.
//!
//! This module computes where things are drawn. All arithmetic uses integer
//! internal units of 100 nm, so no coordinate passes through a float and back.
//! The module knows nothing about the command surface, files on disk, or
//! `kicad-cli`.

pub mod font;
pub mod grid;
pub mod pins;
pub mod symbol_box;
pub mod text;
pub mod transform;

pub use grid::{on_segment, snap, snap_point};
pub use pins::{ResolvedPin, resolve_pins};
pub use symbol_box::{SymbolBoxes, symbol_boxes};
pub use text::{TextBox, TextStyle, text_box};
pub use transform::Transform;

use std::fmt;

/// Internal units per millimetre in a schematic.
///
/// A board file uses a different scale. Mixing the two is the bug behind
/// KiCad's own rule-check JSON exporter, which reports schematic coordinates
/// 100 times too small.
pub const UNITS_PER_MM: i32 = 10_000;

/// The schematic grid, 50 mil.
pub const GRID: Iu = Iu(12_700);

/// A length or coordinate in internal units of 100 nm.
///
/// The type exists so that a millimetre value and an internal-unit value cannot
/// be added by accident. Millimetres are a presentation unit at the command
/// boundary and appear nowhere else.
///
/// # Examples
///
/// ```
/// use kicli::geometry::{GRID, Iu};
/// assert_eq!(Iu::from_millimetres_text("1.27"), Some(Iu(12_700)));
/// assert!(GRID.is_on_grid());
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Iu(pub i32);

impl Iu {
    /// Read a millimetre reading as written in a KiCad file.
    ///
    /// Returns `None` when the text is not a number, or does not fit an `i32`
    /// of internal units.
    #[must_use]
    pub fn from_millimetres_text(text: &str) -> Option<Self> {
        kicli_sexpr::parse_iu(text).map(Self)
    }

    /// Is this coordinate on the schematic grid?
    #[must_use]
    pub fn is_on_grid(self) -> bool {
        self.0 % GRID.0 == 0
    }

    /// The value in millimetres, for display only.
    #[must_use]
    pub fn millimetres(self) -> f64 {
        f64::from(self.0) / f64::from(UNITS_PER_MM)
    }
}

impl std::ops::Add for Iu {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0)
    }
}

impl std::ops::Sub for Iu {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self(self.0 - other.0)
    }
}

impl std::ops::Neg for Iu {
    type Output = Self;

    fn neg(self) -> Self {
        Self(-self.0)
    }
}

impl fmt::Display for Iu {
    /// Write the value the way KiCad writes it.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&kicli_sexpr::fmt_iu(self.0))
    }
}

/// A point in schematic space, with Y increasing downwards.
///
/// Library space has Y increasing upwards. Library coordinates are negated as
/// they are read, so every point in this type is already schematic space.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Point {
    /// Distance from the page's left edge.
    pub x: Iu,
    /// Distance from the page's top edge.
    pub y: Iu,
}

impl Point {
    /// A point from raw internal units.
    #[must_use]
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x: Iu(x), y: Iu(y) }
    }

    /// Are both coordinates on the schematic grid?
    #[must_use]
    pub fn is_on_grid(self) -> bool {
        self.x.is_on_grid() && self.y.is_on_grid()
    }

    /// The point turned about another point.
    ///
    /// A positive angle turns anticlockwise on screen, because Y grows
    /// downwards. The four right angles are exact integer swaps, as they are in
    /// KiCad's own `RotatePoint` (`libs/kimath/src/trigo.cpp:295-330`). Any
    /// other angle goes through a sine and a cosine, and is rounded back to
    /// internal units; a schematic item never takes one.
    #[must_use]
    pub fn rotated(self, pivot: Self, angle: Angle) -> Self {
        let (dx, dy) = (self.x.0 - pivot.x.0, self.y.0 - pivot.y.0);
        let (x, y) = match angle.0.rem_euclid(360) {
            0 => (dx, dy),
            90 => (dy, -dx),
            180 => (-dx, -dy),
            270 => (-dy, dx),
            other => {
                let radians = f64::from(other).to_radians();
                let (sine, cosine) = radians.sin_cos();
                let (dx, dy) = (f64::from(dx), f64::from(dy));
                // A rotated coordinate stays on the page, so the narrowing is
                // safe and the rounding is the same one KiCad applies.
                #[allow(clippy::cast_possible_truncation)]
                {
                    (
                        (dy * sine + dx * cosine).round() as i32,
                        (dy * cosine - dx * sine).round() as i32,
                    )
                }
            }
        };
        Self {
            x: Iu(pivot.x.0 + x),
            y: Iu(pivot.y.0 + y),
        }
    }
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{},{}", self.x, self.y)
    }
}

impl std::ops::Add for Point {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl std::ops::Sub for Point {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

/// An axis-aligned box in schematic space.
///
/// The corners are ordered: `start` is the top-left and `end` the
/// bottom-right, because Y grows downwards. Every constructor normalises, so a
/// box never has a negative side.
///
/// # Examples
///
/// ```
/// use kicli::geometry::{Iu, Point, Rect};
/// let rect = Rect::new(Point::new(10, 20), Point::new(0, 0));
/// assert_eq!(rect.start(), Point::new(0, 0));
/// assert_eq!(rect.width(), Iu(10));
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Rect {
    /// The corner with the smaller coordinates.
    start: Point,
    /// The corner with the larger coordinates.
    end: Point,
}

impl Rect {
    /// The box spanned by two opposite corners, in any order.
    #[must_use]
    pub fn new(one: Point, other: Point) -> Self {
        Self {
            start: Point {
                x: one.x.min(other.x),
                y: one.y.min(other.y),
            },
            end: Point {
                x: one.x.max(other.x),
                y: one.y.max(other.y),
            },
        }
    }

    /// The box at a corner with a size, which may be negative on either axis.
    #[must_use]
    pub fn from_origin(origin: Point, size: Size) -> Self {
        Self::new(
            origin,
            Point {
                x: origin.x + size.x,
                y: origin.y + size.y,
            },
        )
    }

    /// The box around a single point, which has no size.
    #[must_use]
    pub fn around(point: Point) -> Self {
        Self {
            start: point,
            end: point,
        }
    }

    /// The corner with the smaller coordinates.
    #[must_use]
    pub fn start(self) -> Point {
        self.start
    }

    /// The corner with the larger coordinates.
    #[must_use]
    pub fn end(self) -> Point {
        self.end
    }

    /// How wide the box is.
    #[must_use]
    pub fn width(self) -> Iu {
        self.end.x - self.start.x
    }

    /// How tall the box is.
    #[must_use]
    pub fn height(self) -> Iu {
        self.end.y - self.start.y
    }

    /// The two sides together.
    #[must_use]
    pub fn size(self) -> Size {
        Size {
            x: self.width(),
            y: self.height(),
        }
    }

    /// The middle of the box.
    ///
    /// An odd side leaves the centre half a unit out. The result is rounded
    /// towards the start corner, which is what integer division does.
    #[must_use]
    pub fn centre(self) -> Point {
        Point {
            x: Iu(self.start.x.0 + self.width().0 / 2),
            y: Iu(self.start.y.0 + self.height().0 / 2),
        }
    }

    /// The smallest box holding both.
    #[must_use]
    pub fn union(self, other: Self) -> Self {
        Self {
            start: Point {
                x: self.start.x.min(other.start.x),
                y: self.start.y.min(other.start.y),
            },
            end: Point {
                x: self.end.x.max(other.end.x),
                y: self.end.y.max(other.end.y),
            },
        }
    }

    /// The box moved by an offset.
    #[must_use]
    pub fn offset(self, by: Point) -> Self {
        Self {
            start: self.start + by,
            end: self.end + by,
        }
    }

    /// The box grown by the same amount on all four sides.
    #[must_use]
    pub fn inflate(self, by: Iu) -> Self {
        Self::new(
            Point {
                x: self.start.x - by,
                y: self.start.y - by,
            },
            Point {
                x: self.end.x + by,
                y: self.end.y + by,
            },
        )
    }

    /// Is the point inside the box, edges included?
    #[must_use]
    pub fn contains(self, point: Point) -> bool {
        point.x >= self.start.x
            && point.x <= self.end.x
            && point.y >= self.start.y
            && point.y <= self.end.y
    }

    /// The four corners, from the start corner, clockwise on screen.
    #[must_use]
    pub fn corners(self) -> [Point; 4] {
        [
            self.start,
            Point {
                x: self.end.x,
                y: self.start.y,
            },
            self.end,
            Point {
                x: self.start.x,
                y: self.end.y,
            },
        ]
    }

    /// The box under a symbol orientation.
    ///
    /// Only the two corners are transformed, then the result is normalised.
    /// That is correct for the eight orientations a symbol can take and for
    /// nothing else, which is exactly what KiCad relies on in
    /// `TRANSFORM::TransformCoordinate` (`libs/kimath/src/transform.cpp:50-56`).
    #[must_use]
    pub fn transformed(self, transform: Transform) -> Self {
        Self::new(transform.apply(self.start), transform.apply(self.end))
    }
}

impl fmt::Display for Rect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}..{}", self.start, self.end)
    }
}

/// A width and a height in internal units.
///
/// A size is not a point: adding two sizes is meaningful and adding two points
/// is not. Text carries its size as a pair of internal units, one per axis,
/// because KiCad lets the two differ.
///
/// # Examples
///
/// ```
/// use kicli::geometry::{Iu, Size};
/// let size = Size::new(12_700, 12_700);
/// assert_eq!(size.x, Iu(12_700));
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Size {
    /// The width.
    pub x: Iu,
    /// The height.
    pub y: Iu,
}

impl Size {
    /// A size from raw internal units.
    #[must_use]
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x: Iu(x), y: Iu(y) }
    }

    /// The smaller of the two sides, ignoring sign.
    ///
    /// KiCad derives a pen width from this side.
    #[must_use]
    pub fn smaller_side(self) -> Iu {
        Iu(self.x.0.abs().min(self.y.0.abs()))
    }
}

impl fmt::Display for Size {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}x{}", self.x, self.y)
    }
}

/// A text or symbol angle, in whole degrees.
///
/// KiCad writes 0, 90, 180 or 270 for a placed symbol and refuses anything
/// else. Text angles take the same four values in a schematic.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Angle(pub i32);

impl Angle {
    /// Read an angle as written in a KiCad file.
    ///
    /// The file may write `90` or `90.0`. Anything with a fractional part is
    /// rounded to the nearest degree, which loses nothing: schematic items are
    /// axis-aligned.
    #[must_use]
    pub fn from_text(text: &str) -> Option<Self> {
        if let Ok(whole) = text.parse::<i32>() {
            return Some(Self(whole));
        }
        let value: f64 = text.parse().ok()?;
        #[allow(clippy::cast_possible_truncation)] // an angle is a small number
        Some(Self(value.round() as i32))
    }
}

impl fmt::Display for Angle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::{Angle, GRID, Iu, Point};

    #[test]
    fn a_millimetre_reading_becomes_integer_units() {
        assert_eq!(Iu::from_millimetres_text("1.27"), Some(Iu(12_700)));
        assert_eq!(Iu::from_millimetres_text("-3.81"), Some(Iu(-38_100)));
        assert_eq!(Iu::from_millimetres_text("0"), Some(Iu(0)));
        assert_eq!(Iu::from_millimetres_text("nonsense"), None);
    }

    #[test]
    fn the_grid_test_is_exact_integer_arithmetic() {
        assert!(GRID.is_on_grid());
        assert!(Iu(0).is_on_grid());
        assert!(Iu(-12_700).is_on_grid());
        assert!(!Iu(12_701).is_on_grid());
        assert!(Point::new(12_700, 25_400).is_on_grid());
        assert!(!Point::new(12_700, 25_401).is_on_grid());
    }

    #[test]
    fn an_angle_reads_whole_or_fractional_text() {
        assert_eq!(Angle::from_text("90"), Some(Angle(90)));
        assert_eq!(Angle::from_text("90.0"), Some(Angle(90)));
        assert_eq!(Angle::from_text("270"), Some(Angle(270)));
        assert_eq!(Angle::from_text(""), None);
    }

    #[test]
    fn a_point_writes_the_way_kicad_does() {
        assert_eq!(Point::new(12_700, -38_100).to_string(), "1.27,-3.81");
    }
}
