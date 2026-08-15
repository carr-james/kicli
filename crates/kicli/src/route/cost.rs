//! What a route costs, kept as parts.
//!
//! ```text
//! cost = w_len·steps + w_turn·corners + w_cross·crossings
//!      + w_text·steps_in_text + w_near·steps_beside_a_body
//! ```
//!
//! **The breakdown is the point of the exercise.** An agent reads the parts to
//! decide whether to move a symbol instead of accepting a route, so the parts
//! are what is computed and the total is their sum. Nothing here holds a total
//! that the parts are reconstructed from, because the two would drift.
//!
//! Every term is an `i64` and every count a whole number. There is no floating
//! point below this line: two runs over one sheet must cost the same, on any
//! machine, forever.

use crate::geometry::Point;
use crate::model::config::Routing;
use crate::route::obstacles::Obstacles;
use crate::route::terminal::Heading;

/// What a path meets, counted before any weight is applied.
///
/// The counts are facts about a drawing. The weights that price them are
/// configuration, and the two are kept apart so that a report can say what a
/// route did as well as what it cost.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Tally {
    /// Grid steps walked.
    pub steps: u32,
    /// Places the path changes direction.
    pub corners: u32,
    /// Steps that cross another net's wire.
    pub crossings: u32,
    /// Steps inside a label or a text box.
    pub text_steps: u32,
    /// Steps within one grid step of a symbol body.
    pub near_steps: u32,
}

/// The five parts of a route's cost.
///
/// The field names are the ones the output contract writes, so a report is the
/// structure rather than a translation of it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Cost {
    /// What the wire's length costs.
    pub length: i64,
    /// What its corners cost.
    pub turns: i64,
    /// What its crossings cost.
    pub crossings: i64,
    /// What the steps through text cost.
    pub text: i64,
    /// What crowding the symbols costs.
    pub proximity: i64,
}

impl Cost {
    /// Price a tally with the configured weights.
    #[must_use]
    pub fn of(tally: Tally, weights: &Routing) -> Self {
        Self {
            length: weights.w_len * i64::from(tally.steps),
            turns: weights.w_turn * i64::from(tally.corners),
            crossings: weights.w_cross * i64::from(tally.crossings),
            text: weights.w_text * i64::from(tally.text_steps),
            proximity: weights.w_near * i64::from(tally.near_steps),
        }
    }

    /// The sum of the parts, which is the only place a total comes from.
    #[must_use]
    pub fn total(self) -> i64 {
        self.length + self.turns + self.crossings + self.text + self.proximity
    }

    /// The parts, named as a report names them, in the order it prints them.
    #[must_use]
    pub fn parts(self) -> [(&'static str, i64); 5] {
        [
            ("length", self.length),
            ("turns", self.turns),
            ("crossings", self.crossings),
            ("text", self.text),
            ("proximity", self.proximity),
        ]
    }
}

/// Why a path cannot be costed.
///
/// A path that cannot be walked has no cost, so this is one answer rather than
/// a cost with a flag beside it. A search reads [`Uncostable::Blocked`] as "this
/// candidate is out" and the rest as a bug in whatever produced the path.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum Uncostable {
    /// Fewer than two points, which draws nothing.
    #[error("a route needs two points, and this one has {0}")]
    TooShort(usize),
    /// Two points in a row that are the same point.
    #[error("the route stands still at {at}")]
    Stationary {
        /// The repeated point.
        at: Point,
    },
    /// A step that is along neither axis.
    #[error("the step from {from} to {to} is along neither axis")]
    Diagonal {
        /// Where the step starts.
        from: Point,
        /// Where it ends.
        to: Point,
    },
    /// A step that is not a whole number of grid steps.
    #[error("the step from {from} to {to} is not a whole number of grid steps")]
    OffGrid {
        /// Where the step starts.
        from: Point,
        /// Where it ends.
        to: Point,
    },
    /// Something the route may not pass through.
    #[error("{handle} blocks the route at {at}")]
    Blocked {
        /// What is in the way, as a report names it.
        handle: String,
        /// Where the route would have had to step.
        at: Point,
    },
}

impl Tally {
    /// Walk a path over a map, counting what each step meets.
    ///
    /// The points are the polyline's **vertices**, as a report prints them. The
    /// walk expands them into grid steps, because a cost is charged per step
    /// and an obstacle is met at a point.
    ///
    /// **The two ends are the route's own.** The first point is where the route
    /// starts, so it is never entered. The last is the terminal it was asked to
    /// reach, and a terminal sits on its own pin — inside its own symbol's body
    /// box, and on the border of the sheet it belongs to — so a block there is
    /// the route arriving rather than the route colliding. That is
    /// `research/wire-routing.md` §3.2's "except the grid point at a target
    /// pin", and it lives here so that no caller has to remember it. What the
    /// last step **costs** is still counted: another net's wire through the
    /// target pin is a real crossing.
    ///
    /// # Errors
    ///
    /// Returns [`Uncostable`] for a path that is too short, stands still, runs
    /// diagonally, misses the lattice, or is blocked at any point but its last.
    pub fn of_path(vertices: &[Point], obstacles: &Obstacles) -> Result<Self, Uncostable> {
        if vertices.len() < 2 {
            return Err(Uncostable::TooShort(vertices.len()));
        }
        let grid = obstacles.window().grid();
        let mut tally = Self::default();
        let mut last_heading: Option<Heading> = None;

        for (index, pair) in vertices.windows(2).enumerate() {
            let (from, to) = (pair[0], pair[1]);
            let heading = Heading::between(from, to).ok_or_else(|| {
                if from == to {
                    Uncostable::Stationary { at: from }
                } else {
                    Uncostable::Diagonal { from, to }
                }
            })?;
            let span = (to.x.0 - from.x.0).abs() + (to.y.0 - from.y.0).abs();
            if span % grid.0 != 0 {
                return Err(Uncostable::OffGrid { from, to });
            }
            if last_heading.is_some_and(|last| last != heading) {
                tally.corners += 1;
            }
            last_heading = Some(heading);

            let steps = span / grid.0;
            let ends_the_route = index + 2 == vertices.len();
            for step in 1..=steps {
                let at = heading.step(from, crate::geometry::Iu(step * grid.0));
                let verdict = obstacles.entering(at, heading);
                let arriving = ends_the_route && step == steps;
                if let Some(handle) = verdict.blocked_by {
                    if !arriving {
                        return Err(Uncostable::Blocked { handle, at });
                    }
                }
                tally.steps += 1;
                tally.crossings += verdict.crossings;
                tally.text_steps += verdict.text_steps;
                tally.near_steps += verdict.near_steps;
            }
        }
        Ok(tally)
    }
}

#[cfg(test)]
mod tests {
    use super::{Cost, Tally};
    use crate::model::Config;

    #[test]
    fn a_total_is_the_sum_of_the_parts_and_nothing_else() {
        let weights = Config::default().routing;
        let tally = Tally {
            steps: 30,
            corners: 2,
            crossings: 1,
            text_steps: 0,
            near_steps: 0,
        };
        let cost = Cost::of(tally, &weights);
        assert_eq!(cost.length, 30);
        assert_eq!(cost.turns, 12);
        assert_eq!(cost.crossings, 20);
        assert_eq!(cost.total(), 62);
        assert_eq!(
            cost.total(),
            cost.parts().iter().map(|&(_, part)| part).sum::<i64>()
        );
    }

    #[test]
    fn the_weights_keep_their_documented_order() {
        // The rationale sentences of the weight table, made executable. A
        // weight is a break-even against a detour: how far out of its way a
        // route may go rather than accept the thing the weight prices. So each
        // comparison is one term against the length term, with everything else
        // held at nothing.
        let weights = Config::default().routing;
        let priced = |what: fn(&mut Tally)| {
            let mut tally = Tally::default();
            what(&mut tally);
            Cost::of(tally, &weights).total()
        };
        let detour = |steps: u32| {
            Cost::of(
                Tally {
                    steps,
                    ..Tally::default()
                },
                &weights,
            )
            .total()
        };

        // A crossing is worth a detour of twenty grid steps, which is what
        // "detour up to 20 grid steps to avoid one crossing" means: the break
        // even is **at** twenty, so a crossing beats nineteen steps and loses
        // to twenty-one.
        let crossing = priced(|tally| tally.crossings = 1);
        assert!(crossing > detour(19), "a crossing is worth 20 steps");
        assert!(crossing < detour(21), "and no more than 20");
        assert!(
            crossing < priced(|tally| tally.crossings = 2),
            "two crossings cost more than one"
        );

        // A corner is worth a detour of six, so it beats the measured median
        // segment of five and loses to seven. A router whose corner cost less
        // than a modest detour zig-zags.
        let corner = priced(|tally| tally.corners = 1);
        assert!(corner > detour(5), "a corner is worth more than 5 steps");
        assert!(corner < detour(7), "and less than 7");

        // A step through a label is nearly as bad as a crossing, and worse
        // than a corner.
        let text = priced(|tally| tally.text_steps = 1);
        assert!(text < crossing, "text is not as bad as a crossing");
        assert!(text > corner, "and it is worse than a corner");

        // Crowding a symbol is a preference and not a rule: dearer than a
        // plain step, far cheaper than a corner.
        let near = priced(|tally| tally.near_steps = 1);
        assert!(near > detour(1), "a crowded step costs more than a step");
        assert!(near < corner, "and much less than a corner");
    }

    #[test]
    fn a_route_that_meets_nothing_costs_its_length() {
        let weights = Config::default().routing;
        let straight = Tally {
            steps: 5,
            ..Tally::default()
        };
        assert_eq!(Cost::of(straight, &weights).total(), 5);
        assert_eq!(Cost::of(Tally::default(), &weights).total(), 0);
    }
}
