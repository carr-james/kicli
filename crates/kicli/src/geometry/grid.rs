//! Where a point may sit, and whether it sits on a line.
//!
//! Both questions are asked by the extractor and by every mutation, so they are
//! answered once here rather than three times each. All arithmetic is exact
//! integer arithmetic: a point is on a segment or it is not, and a coordinate is
//! on the grid or it is not. Neither question has a tolerance, because KiCad's
//! own connectivity has none.

use super::{Iu, Point};

/// Round a coordinate to the nearest grid line.
///
/// A half step rounds away from zero, which is what KiCad's own snap does, so a
/// point exactly between two lines does not depend on its sign. A grid of zero
/// is no grid, and leaves the value alone.
///
/// # Examples
///
/// ```
/// use kicli::geometry::{GRID, Iu, snap};
///
/// assert_eq!(snap(Iu(6_350), GRID), Iu(12_700), "a half step rounds away from zero");
/// assert_eq!(snap(Iu(-6_350), GRID), Iu(-12_700));
/// assert_eq!(snap(Iu(6_349), GRID), Iu(0));
/// ```
#[must_use]
pub fn snap(value: Iu, grid: Iu) -> Iu {
    if grid.0 == 0 {
        return value;
    }
    let step = i64::from(grid.0).abs();
    let size = i64::from(value.0).abs();
    let steps = (size + step / 2) / step;
    let rounded = steps * step * i64::from(value.0.signum());
    Iu(i32::try_from(rounded).unwrap_or(value.0))
}

/// Round both coordinates of a point to the nearest grid line.
///
/// # Examples
///
/// ```
/// use kicli::geometry::{GRID, Point, snap_point};
///
/// assert_eq!(snap_point(Point::new(6_350, 0), GRID), Point::new(12_700, 0));
/// ```
#[must_use]
pub fn snap_point(point: Point, grid: Iu) -> Point {
    Point {
        x: snap(point.x, grid),
        y: snap(point.y, grid),
    }
}

/// Is a point on a segment, ends included?
///
/// Exact integer arithmetic in 64 bits: the cross product decides whether the
/// point is on the line, and the dot product whether it is between the ends.
///
/// # Examples
///
/// ```
/// use kicli::geometry::{Point, on_segment};
///
/// let (from, to) = (Point::new(0, 0), Point::new(100, 0));
/// assert!(on_segment(from, to, Point::new(50, 0)));
/// assert!(on_segment(from, to, from), "an end is on the segment");
/// assert!(!on_segment(from, to, Point::new(101, 0)));
/// assert!(!on_segment(from, to, Point::new(50, 1)));
/// ```
#[must_use]
pub fn on_segment(from: Point, to: Point, point: Point) -> bool {
    let (ax, ay) = (i64::from(from.x.0), i64::from(from.y.0));
    let (bx, by) = (i64::from(to.x.0), i64::from(to.y.0));
    let (px, py) = (i64::from(point.x.0), i64::from(point.y.0));
    let (dx, dy) = (bx - ax, by - ay);
    if dx * (py - ay) - dy * (px - ax) != 0 {
        return false;
    }
    let along = dx * (px - ax) + dy * (py - ay);
    along >= 0 && along <= dx * dx + dy * dy
}

#[cfg(test)]
mod tests {
    use super::{on_segment, snap};
    use crate::geometry::{GRID, Iu, Point};

    #[test]
    fn a_half_step_rounds_away_from_zero() {
        assert_eq!(snap(Iu(6_350), GRID), Iu(12_700));
        assert_eq!(snap(Iu(-6_350), GRID), Iu(-12_700));
        assert_eq!(snap(Iu(6_349), GRID), Iu(0));
        assert_eq!(snap(Iu(-6_349), GRID), Iu(0));
    }

    #[test]
    fn no_grid_leaves_a_value_alone() {
        assert_eq!(snap(Iu(1), Iu(0)), Iu(1));
    }

    #[test]
    fn a_point_is_on_a_segment_or_it_is_not() {
        let from = Point::new(0, 0);
        let to = Point::new(100, 0);
        assert!(on_segment(from, to, Point::new(50, 0)));
        assert!(on_segment(from, to, from));
        assert!(on_segment(from, to, to));
        assert!(!on_segment(from, to, Point::new(101, 0)));
        assert!(!on_segment(from, to, Point::new(-1, 0)));
        assert!(!on_segment(from, to, Point::new(50, 1)));
        // A diagonal segment is measured the same way.
        let corner = Point::new(100, 100);
        assert!(on_segment(from, corner, Point::new(37, 37)));
        assert!(!on_segment(from, corner, Point::new(37, 38)));
    }
}
