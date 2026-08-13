//! The eight orientations a placed symbol can take.
//!
//! KiCad carries a 2x2 integer matrix per symbol and applies it as
//! `(x1*p.x + y1*p.y, x2*p.x + y2*p.y)`. Note the member naming: the rows are
//! `(x1, y1)` and `(x2, y2)`, **not** the columns. Transposing it swaps the 90
//! and 270 degree cases, which is the classic third-party bug, so the table
//! below is written out rather than derived.
//!
//! Ported from `libs/kimath/include/transform.h`,
//! `libs/kimath/src/transform.cpp` and `eeschema/sch_symbol.cpp` at tag 10.0.5.
//! KiCad is GPL-3.0-or-later, as is kicli.

use crate::geometry::{Angle, Iu, Point};
use crate::model::items::Mirror;

/// A 2x2 integer matrix, in KiCad's row-major member order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Transform {
    /// Row one, first column.
    pub x1: i32,
    /// Row one, second column.
    pub y1: i32,
    /// Row two, first column.
    pub x2: i32,
    /// Row two, second column.
    pub y2: i32,
}

impl Default for Transform {
    /// The identity, which is what a symbol at 0 degrees with no mirror gets.
    fn default() -> Self {
        Self {
            x1: 1,
            y1: 0,
            x2: 0,
            y2: 1,
        }
    }
}

impl Transform {
    /// The matrix a rotation alone gives.
    ///
    /// KiCad's parser refuses any angle but these four, so anything else falls
    /// back to the identity rather than inventing a fifth orientation.
    #[must_use]
    pub fn from_angle(angle: Angle) -> Self {
        match angle.0.rem_euclid(360) {
            90 => Self {
                x1: 0,
                y1: 1,
                x2: -1,
                y2: 0,
            },
            180 => Self {
                x1: -1,
                y1: 0,
                x2: 0,
                y2: -1,
            },
            270 => Self {
                x1: 0,
                y1: -1,
                x2: 1,
                y2: 0,
            },
            _ => Self::default(),
        }
    }

    /// The matrix of a symbol as the file writes it.
    ///
    /// `(at ... angle)` is always written before `(mirror ...)`, and the mirror
    /// composes as the **second** operand: `rotation.compose(mirror)`. The
    /// other order is the classic third-party bug. It agrees on all four
    /// unmirrored orientations and on both mirrors at 0 degrees, and swaps
    /// mirror X with mirror Y at 90 degrees, so it survives every symmetric
    /// test and fails only on an asymmetric part. KiCad's own rule-check output
    /// settles it, and the fixture beside this crate keeps it settled.
    #[must_use]
    pub fn from_file(angle: Angle, mirror: Option<Mirror>) -> Self {
        let rotation = Self::from_angle(angle);
        match mirror {
            None => rotation,
            Some(Mirror::X) => rotation.compose(Self {
                x1: 1,
                y1: 0,
                x2: 0,
                y2: -1,
            }),
            Some(Mirror::Y) => rotation.compose(Self {
                x1: -1,
                y1: 0,
                x2: 0,
                y2: 1,
            }),
        }
    }

    /// Compose two orientations, in the order `SCH_SYMBOL::SetOrientation`
    /// combines an incremental transform with the one already stored.
    #[must_use]
    pub fn compose(self, first: Self) -> Self {
        Self {
            x1: self.x1 * first.x1 + self.x2 * first.y1,
            y1: self.y1 * first.x1 + self.y2 * first.y1,
            x2: self.x1 * first.x2 + self.x2 * first.y2,
            y2: self.y1 * first.x2 + self.y2 * first.y2,
        }
    }

    /// Map a point in library space, Y already flipped to schematic sense.
    #[must_use]
    pub fn apply(self, point: Point) -> Point {
        Point {
            x: Iu(self.x1 * point.x.0 + self.y1 * point.y.0),
            y: Iu(self.x2 * point.x.0 + self.y2 * point.y.0),
        }
    }

    /// The `(angle, mirror)` pair KiCad itself would write for this matrix.
    ///
    /// The group has eight elements, so 180 and 270 degrees with a mirror
    /// reduce to a pair already in the table. Normalising means two symbols
    /// that look identical are described identically.
    #[must_use]
    pub fn to_file(self) -> (Angle, Option<Mirror>) {
        for &(angle, mirror) in &[
            (0, None),
            (90, None),
            (180, None),
            (270, None),
            (0, Some(Mirror::X)),
            (0, Some(Mirror::Y)),
            (90, Some(Mirror::X)),
            (90, Some(Mirror::Y)),
        ] {
            if Self::from_file(Angle(angle), mirror) == self {
                return (Angle(angle), mirror);
            }
        }
        (Angle(0), None)
    }
}

#[cfg(test)]
mod tests {
    use super::Transform;
    use crate::geometry::{Angle, Point};
    use crate::model::items::Mirror;

    /// The table of research/geometry.md 2.3, written out rather than derived.
    const ORIENTATIONS: [(i32, Option<Mirror>, [i32; 4]); 8] = [
        (0, None, [1, 0, 0, 1]),
        (90, None, [0, 1, -1, 0]),
        (180, None, [-1, 0, 0, -1]),
        (270, None, [0, -1, 1, 0]),
        (0, Some(Mirror::X), [1, 0, 0, -1]),
        (0, Some(Mirror::Y), [-1, 0, 0, 1]),
        (90, Some(Mirror::X), [0, 1, 1, 0]),
        (90, Some(Mirror::Y), [0, -1, -1, 0]),
    ];

    #[test]
    fn the_matrix_matches_kicads_table_row_by_row() {
        for (angle, mirror, [x1, y1, x2, y2]) in ORIENTATIONS {
            let built = Transform::from_file(Angle(angle), mirror);
            assert_eq!(
                (built.x1, built.y1, built.x2, built.y2),
                (x1, y1, x2, y2),
                "{angle} degrees, mirror {mirror:?}"
            );
        }
    }

    #[test]
    fn a_transposed_matrix_would_not_pass() {
        // 90 and 270 degrees are transposes of each other. If the rows and
        // columns were swapped, these two would be equal, and every rotated
        // symbol would be placed as its own mirror image.
        let ninety = Transform::from_angle(Angle(90));
        let two_seventy = Transform::from_angle(Angle(270));
        assert_ne!(ninety, two_seventy);
        assert_eq!(
            (ninety.y1, ninety.x2),
            (-two_seventy.y1, -two_seventy.x2),
            "the two differ in the off-diagonal, which is what transposing hides"
        );
    }

    #[test]
    fn the_group_has_exactly_eight_elements() {
        let mut seen = Vec::new();
        for angle in [0, 90, 180, 270] {
            for mirror in [None, Some(Mirror::X), Some(Mirror::Y)] {
                let built = Transform::from_file(Angle(angle), mirror);
                if !seen.contains(&built) {
                    seen.push(built);
                }
            }
        }
        assert_eq!(seen.len(), 8, "twelve written forms, eight orientations");

        // Composing any two elements lands inside the group.
        for &a in &seen {
            for &b in &seen {
                assert!(seen.contains(&a.compose(b)), "the group is closed");
            }
        }
    }

    #[test]
    fn a_written_orientation_survives_a_round_trip() {
        for (angle, mirror, _) in ORIENTATIONS {
            let built = Transform::from_file(Angle(angle), mirror);
            let (written_angle, written_mirror) = built.to_file();
            assert_eq!((written_angle.0, written_mirror), (angle, mirror));
            assert_eq!(
                Transform::from_file(written_angle, written_mirror),
                built,
                "writing then reading gives the same matrix"
            );
        }
    }

    #[test]
    fn the_redundant_forms_normalise_to_the_eight() {
        // 180 with a mirror is 0 with the other mirror, and 270 with a mirror
        // is 90 with the other. KiCad's editor writes the normalised form.
        let cases = [
            (180, Mirror::X, 0, Mirror::Y),
            (180, Mirror::Y, 0, Mirror::X),
            (270, Mirror::X, 90, Mirror::Y),
            (270, Mirror::Y, 90, Mirror::X),
        ];
        for (angle, mirror, expected_angle, expected_mirror) in cases {
            let built = Transform::from_file(Angle(angle), Some(mirror));
            assert_eq!(
                built.to_file(),
                (Angle(expected_angle), Some(expected_mirror)),
                "{angle} degrees with mirror {mirror:?}"
            );
        }
    }

    #[test]
    fn a_point_maps_the_way_the_table_says() {
        let point = Point::new(30_000, 10_000);
        // (px, py) -> (py, -px) at 90 degrees.
        assert_eq!(
            Transform::from_angle(Angle(90)).apply(point),
            Point::new(10_000, -30_000)
        );
        // (px, py) -> (py, px) at 90 degrees with mirror x.
        assert_eq!(
            Transform::from_file(Angle(90), Some(Mirror::X)).apply(point),
            Point::new(10_000, 30_000)
        );
    }
}
