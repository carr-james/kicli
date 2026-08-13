//! Integer geometry: transforms, pin positions, bounding boxes, and text extents.
//!
//! This module computes where things are drawn. All arithmetic uses integer
//! internal units of 100 nm, so no coordinate passes through a float and back.
//! The module knows nothing about the command surface, files on disk, or
//! `kicad-cli`.

pub mod transform;

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
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{},{}", self.x, self.y)
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
