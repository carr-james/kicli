//! The routes a person draws, enumerated before anything searches.
//!
//! `research/wire-routing.md` §4.2 lists six silhouettes, and §7 fixes the order
//! they are tried in: I, L(horizontal first), L(vertical first), Z(vertical
//! middle, `m` ascending), Z(horizontal middle, `m` ascending), U(offset
//! ascending). Each candidate is validated against the obstacle map and costed;
//! the cheapest wins.
//!
//! **Shapes come before A\*** because A\* with a corner penalty finds *a*
//! minimal route and often not the one a person would draw, and because half of
//! all real segments are under 7 mm — the fast path is the common path rather
//! than an optimisation.
//!
//! **The target-cell exception is not here.** §3.2's first row ends "except the
//! grid point at a target pin", and that rule has one home: [`Tally::of_path`]
//! excepts the two ends of the path it walks. Every candidate below is costed
//! through it, so nothing in this module knows what a target is. A second
//! implementation of the rule is the divergence the cost model's note exists to
//! prevent.
//!
//! **A terminal the caller did not name blocks its own route.** The map is
//! built from [`Routed`](crate::route::sheet::Routed), and a pin left out of
//! `terminals` keeps its 1 G halo — which stands across the last step but one,
//! where no exception reaches. That is the caller's answer arriving late rather
//! than a hole here.

use crate::geometry::{Iu, Point};
use crate::model::config::Routing;
use crate::route::cost::{Cost, Tally, Uncostable};
use crate::route::obstacles::Obstacles;
use crate::route::terminal::{Heading, Terminal};

/// One silhouette of `research/wire-routing.md` §4.2.
///
/// The declaration order is the evaluation order, so the derived `Ord` is the
/// order §7 fixes and nothing has to carry an index beside it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Shape {
    /// One segment, when the two escape points share a coordinate.
    Straight,
    /// Two segments, the horizontal one first.
    LHorizontalFirst,
    /// Two segments, the vertical one first.
    LVerticalFirst,
    /// Three segments about a vertical middle line inside the span.
    ZVerticalMiddle,
    /// Three segments about a horizontal middle line inside the span.
    ZHorizontalMiddle,
    /// A middle line **outside** the span, reached outward one grid step at a
    /// time. It is what is left when both terminals face the same way.
    UOutside,
}

impl Shape {
    /// Every shape, in the order §4.2 evaluates them.
    ///
    /// The list is checked against an exhaustive match in this module's tests,
    /// so a new silhouette is a compile error there rather than a shape no
    /// coverage assertion ever misses.
    pub const EVERY: [Self; 6] = [
        Self::Straight,
        Self::LHorizontalFirst,
        Self::LVerticalFirst,
        Self::ZVerticalMiddle,
        Self::ZHorizontalMiddle,
        Self::UOutside,
    ];
}

/// One route the enumeration offered, with what it costs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Candidate {
    /// The silhouette that produced it.
    pub shape: Shape,
    /// The vertices, from the source terminal to the target terminal.
    pub path: Vec<Point>,
    /// What walking it meets.
    pub tally: Tally,
    /// What that costs, in parts.
    pub cost: Cost,
}

impl Candidate {
    /// The key `research/wire-routing.md` §7 breaks ties on.
    ///
    /// Cheapest first, then fewer corners, then shorter, then the
    /// lexicographically smallest vertex sequence. Two candidates that agree on
    /// all four are the same route drawn by two families, and then the order
    /// they were enumerated in is the only thing left to separate them — which
    /// [`Shapes::best`] does by keeping the first.
    fn rank(&self) -> (i64, u32, u32, &[Point]) {
        (
            self.cost.total(),
            self.tally.corners,
            self.tally.steps,
            &self.path,
        )
    }
}

/// Every candidate one route request produced.
#[derive(Clone, Debug, Default)]
pub struct Shapes {
    feasible: Vec<Candidate>,
    considered: u32,
    blocked_by: Vec<String>,
}

impl Shapes {
    /// Enumerate the shapes between two terminals, and cost the ones that fit.
    ///
    /// The escape rule is built in rather than checked: every candidate starts
    /// at the source's own point and passes through its escape point, and ends
    /// through the target's, so a route always leaves and arrives along the
    /// direction its terminal fixes. A candidate whose first leg then runs back
    /// against that escape turns on the spot, and is dropped rather than
    /// offered.
    ///
    /// Terminals that are not on one lattice produce candidates the walk
    /// refuses as off-grid, and the answer is then simply empty. The caller
    /// refuses an off-grid terminal before it asks — moving somebody's pin is
    /// not a routing decision — so that case does not arise from a request.
    #[must_use]
    pub fn of(
        source: &Terminal,
        target: &Terminal,
        obstacles: &Obstacles,
        weights: &Routing,
    ) -> Self {
        let grid = obstacles.window().grid();
        let mut shapes = Self::default();
        for (shape, outline) in outlines(
            source.escape_point(grid),
            target.escape_point(grid),
            grid,
            weights.u_max,
        ) {
            shapes.consider(shape, source.at, &outline, target.at, obstacles, weights);
        }
        shapes
    }

    /// The route to draw, when any candidate fits.
    #[must_use]
    pub fn best(&self) -> Option<&Candidate> {
        let mut best: Option<&Candidate> = None;
        for candidate in &self.feasible {
            // Strictly better, so an exact tie leaves the earlier candidate in
            // place. That is where the fixed enumeration order does its work.
            if best.is_none_or(|held| candidate.rank() < held.rank()) {
                best = Some(candidate);
            }
        }
        best
    }

    /// Every candidate that fits, in the order the shapes were tried.
    #[must_use]
    pub fn feasible(&self) -> &[Candidate] {
        &self.feasible
    }

    /// How many candidates were tried, feasible or not.
    ///
    /// This is the report's `alternatives_considered`: what the router looked
    /// at, not what it could use.
    #[must_use]
    pub fn considered(&self) -> u32 {
        self.considered
    }

    /// What stood in the way, named once each, in the order first met.
    #[must_use]
    pub fn blocked_by(&self) -> &[String] {
        &self.blocked_by
    }

    /// Cost one outline, and file it as feasible or as a refusal.
    fn consider(
        &mut self,
        shape: Shape,
        start: Point,
        outline: &[Point],
        finish: Point,
        obstacles: &Obstacles,
        weights: &Routing,
    ) {
        self.considered = self.considered.saturating_add(1);
        let mut points = Vec::with_capacity(outline.len() + 2);
        points.push(start);
        points.extend_from_slice(outline);
        points.push(finish);
        let Some(path) = polyline(&points) else {
            return;
        };
        match Tally::of_path(&path, obstacles) {
            Ok(tally) => {
                let cost = Cost::of(tally, weights);
                self.feasible.push(Candidate {
                    shape,
                    path,
                    tally,
                    cost,
                });
            }
            Err(Uncostable::Blocked { handle, .. }) => {
                if !self.blocked_by.contains(&handle) {
                    self.blocked_by.push(handle);
                }
            }
            // Anything else is an outline this module built badly rather than a
            // drawing that refused it. It is dropped, and the shape then
            // reaches no drawing at all, which is what the coverage assertion
            // in `tests/route_shapes.rs` is watching for.
            Err(_) => {}
        }
    }
}

/// The outlines of every shape, between two escape points.
fn outlines(from: Point, to: Point, grid: Iu, u_max: Iu) -> Vec<(Shape, Vec<Point>)> {
    let mut all = Vec::new();
    if from.x == to.x || from.y == to.y {
        all.push((Shape::Straight, vec![from, to]));
    }
    all.push((
        Shape::LHorizontalFirst,
        vec![from, Point::new(to.x.0, from.y.0), to],
    ));
    all.push((
        Shape::LVerticalFirst,
        vec![from, Point::new(from.x.0, to.y.0), to],
    ));
    for middle in lines(from.x, to.x, grid) {
        all.push((
            Shape::ZVerticalMiddle,
            vec![
                from,
                Point::new(middle.0, from.y.0),
                Point::new(middle.0, to.y.0),
                to,
            ],
        ));
    }
    for middle in lines(from.y, to.y, grid) {
        all.push((
            Shape::ZHorizontalMiddle,
            vec![
                from,
                Point::new(from.x.0, middle.0),
                Point::new(to.x.0, middle.0),
                to,
            ],
        ));
    }
    // The offset ascends, and within one offset the four sides are taken in the
    // order §4.3 fixes for the search's own expansion — one order in the
    // router rather than two.
    let offsets = if grid.0 > 0 { u_max.0 / grid.0 } else { 0 };
    for offset in 1..=offsets {
        let reach = Iu(offset * grid.0);
        for side in Heading::EVERY {
            all.push((Shape::UOutside, detour(from, to, side, reach)));
        }
    }
    all
}

/// The grid lines from one coordinate to another, inclusive of both.
fn lines(one: Iu, other: Iu, grid: Iu) -> Vec<Iu> {
    let (low, high) = (one.0.min(other.0), one.0.max(other.0));
    let mut found = Vec::new();
    if grid.0 <= 0 {
        return found;
    }
    let mut at = low;
    while at <= high {
        found.push(Iu(at));
        at += grid.0;
    }
    found
}

/// A middle line one reach outside the span, on one of the four sides.
fn detour(from: Point, to: Point, side: Heading, reach: Iu) -> Vec<Point> {
    match side {
        Heading::PlusX | Heading::MinusX => {
            let line = if side == Heading::PlusX {
                from.x.0.max(to.x.0) + reach.0
            } else {
                from.x.0.min(to.x.0) - reach.0
            };
            vec![
                from,
                Point::new(line, from.y.0),
                Point::new(line, to.y.0),
                to,
            ]
        }
        Heading::PlusY | Heading::MinusY => {
            let line = if side == Heading::PlusY {
                from.y.0.max(to.y.0) + reach.0
            } else {
                from.y.0.min(to.y.0) - reach.0
            };
            vec![
                from,
                Point::new(from.x.0, line),
                Point::new(to.x.0, line),
                to,
            ]
        }
    }
}

/// The vertices a wire is drawn from, or nothing when the points are not one.
///
/// Three things happen here. A point repeated because a free coordinate landed
/// on a terminal's own is dropped; a vertex in the middle of a straight run is
/// dropped, because a reader sees one wire there and the file should hold one
/// record; and a route that turns back the way it came, or that meets a vertex
/// it has already been to, is refused outright — it would draw a wire over its
/// own last segment, which reads as a connection rather than as a corner.
fn polyline(points: &[Point]) -> Option<Vec<Point>> {
    let mut walked: Vec<Point> = Vec::with_capacity(points.len());
    for &point in points {
        if walked.last() != Some(&point) {
            walked.push(point);
        }
    }
    let (first, last) = (*walked.first()?, *walked.last()?);
    if walked.len() < 2 {
        return None;
    }
    let mut vertices = vec![first];
    let mut heading = Heading::between(walked[0], walked[1])?;
    for pair in walked.windows(2).skip(1) {
        let next = Heading::between(pair[0], pair[1])?;
        if next == heading {
            continue;
        }
        if next == heading.reversed() {
            return None;
        }
        vertices.push(pair[0]);
        heading = next;
    }
    vertices.push(last);
    for (index, vertex) in vertices.iter().enumerate() {
        if vertices[index + 1..].contains(vertex) {
            return None;
        }
    }
    Some(vertices)
}

#[cfg(test)]
mod tests {
    //! The two rules of [`polyline`] that no drawing can exercise.
    //!
    //! A route only ever turns back on itself at one of its two stubs — the
    //! outline between the escape points turns from one axis to the other and
    //! never reverses — and a terminal's own cell is always covered by the body
    //! it belongs to: a symbol's body box reaches its pins' connection points,
    //! and a sheet's covers the border its ports sit on. So the leg that would
    //! reverse always steps through the terminal's own cell first, where the
    //! map blocks it, and every candidate these two rules refuse is refused by
    //! the drawing as well. **Measured on 2026-08-15**: with both rules taken
    //! out, all five checks in `tests/route_shapes.rs` still pass.
    //!
    //! That is the condition T7 attached to its exception — hand-stated input
    //! is permitted **only where no drawable request can distinguish the
    //! behaviour**. The rules are kept rather than deleted because what makes
    //! them unreachable is another module's invariant rather than this one's,
    //! and they are measured here because the alternative is not measuring them
    //! at all.

    use super::{Candidate, Shape, Shapes, polyline};
    use crate::geometry::{GRID, Point};
    use crate::model::Config;
    use crate::route::cost::{Cost, Tally};

    /// A point a whole number of grid steps from the origin.
    fn at(column: i32, row: i32) -> Point {
        Point::new(column * GRID.0, row * GRID.0)
    }

    #[test]
    fn a_route_that_draws_over_itself_is_not_a_route() {
        // The straight run of three points is one wire record, so the vertex in
        // the middle of it goes.
        assert_eq!(
            polyline(&[at(0, 0), at(1, 0), at(4, 0)]),
            Some(vec![at(0, 0), at(4, 0)])
        );
        // A repeated point is a free coordinate that landed on a terminal's own.
        assert_eq!(
            polyline(&[at(0, 0), at(0, 0), at(0, 3)]),
            Some(vec![at(0, 0), at(0, 3)])
        );
        // The control: a corner is kept, so the collapse above is not simply
        // dropping everything between the ends.
        assert_eq!(
            polyline(&[at(0, 0), at(2, 0), at(2, 3)]),
            Some(vec![at(0, 0), at(2, 0), at(2, 3)])
        );

        // A stub that turns back the way it came would draw a wire over its own
        // last segment, which reads as a connection rather than as a corner.
        assert_eq!(polyline(&[at(0, 0), at(2, 0), at(1, 0)]), None);
        // And a vertex met twice is the same fault a step further out.
        assert_eq!(
            polyline(&[at(0, 0), at(2, 0), at(2, 2), at(0, 2), at(0, 0)]),
            None
        );

        // Nothing a wire cannot be drawn from: a diagonal, and a list too short
        // to hold a segment.
        assert_eq!(polyline(&[at(0, 0), at(1, 1)]), None);
        assert_eq!(polyline(&[at(0, 0), at(0, 0)]), None);
        assert_eq!(polyline(&[at(0, 0)]), None);
        assert_eq!(polyline(&[]), None);
    }

    /// Two candidates offered in the order given, and the one that is chosen.
    ///
    /// The tallies are stated rather than walked, because the rung being
    /// measured is the comparison and not the arithmetic that feeds it.
    fn chosen(first: (Tally, &[Point]), second: (Tally, &[Point])) -> Vec<Point> {
        let weights = Config::default().routing;
        let candidate = |shape, (tally, path): (Tally, &[Point])| Candidate {
            shape,
            path: path.to_vec(),
            tally,
            cost: Cost::of(tally, &weights),
        };
        let shapes = Shapes {
            feasible: vec![
                candidate(Shape::LHorizontalFirst, first),
                candidate(Shape::LVerticalFirst, second),
            ],
            considered: 2,
            blocked_by: Vec::new(),
        };
        match shapes.best() {
            Some(best) => best.path.clone(),
            None => Vec::new(),
        }
    }

    #[test]
    fn a_tie_falls_through_the_rungs_of_the_documented_chain() {
        // §7 breaks a tie by fewer corners, then by shorter, then by the
        // lexicographically smallest vertex sequence. Two of those rungs are
        // reached only where two candidates cost the same and differ in what
        // makes up the cost, which none of the six drawings in
        // `tests/route_shapes.rs` does — measured, 2026-08-15: a cost tie there
        // is a tie in the corners and the steps as well. So the rungs are
        // measured here, on stated tallies, under the condition T7 attached to
        // its exception.
        let straight = &[Point::new(0, 0), at(12, 0)][..];
        let bent = &[Point::new(0, 0), at(6, 0), at(6, 6)][..];
        let tally = |steps, corners, near_steps| Tally {
            steps,
            corners,
            near_steps,
            ..Tally::default()
        };

        // Twelve plain steps and six steps with a corner both cost twelve. The
        // corner rung is above the length rung, so the longer route wins.
        assert_eq!(
            chosen((tally(12, 0, 0), straight), (tally(6, 1, 0), bent)),
            straight.to_vec(),
            "fewer corners beats shorter"
        );
        // And the same pair the other way round, so the answer is the rule
        // rather than the order they were offered in.
        assert_eq!(
            chosen((tally(6, 1, 0), bent), (tally(12, 0, 0), straight)),
            straight.to_vec()
        );

        // Ten steps past one crowded body and twelve plain ones, each with one
        // corner, both cost eighteen. Now the length rung decides.
        let short = &[Point::new(0, 0), at(5, 0), at(5, 5)][..];
        assert_eq!(
            chosen((tally(12, 1, 0), bent), (tally(10, 1, 1), short)),
            short.to_vec(),
            "shorter beats longer once the corners agree"
        );

        // Two routes that agree on all three: the smaller vertex sequence wins,
        // whichever order they were offered in.
        let low = &[Point::new(0, 0), at(3, 0), at(3, 6)][..];
        let high = &[Point::new(0, 0), at(4, 0), at(4, 6)][..];
        assert_eq!(chosen((tally(9, 1, 0), high), (tally(9, 1, 0), low)), low);
        assert_eq!(chosen((tally(9, 1, 0), low), (tally(9, 1, 0), high)), low);

        // And two that agree on the vertices too — the same route drawn by two
        // families — leave the enumeration order as the only thing left, which
        // keeps the first.
        assert_eq!(chosen((tally(9, 1, 0), low), (tally(9, 1, 0), low)), low);
    }

    #[test]
    fn every_shape_is_in_the_evaluation_order() {
        for shape in Shape::EVERY {
            // The match is exhaustive, so a new silhouette is a compile error
            // here, and its arm is what puts it in the list.
            let place = match shape {
                Shape::Straight => 0_usize,
                Shape::LHorizontalFirst => 1,
                Shape::LVerticalFirst => 2,
                Shape::ZVerticalMiddle => 3,
                Shape::ZHorizontalMiddle => 4,
                Shape::UOutside => 5,
            };
            assert_eq!(Shape::EVERY[place], shape);
        }
        // And the list is the order the derived comparison gives, which is what
        // a tie between two families is settled by.
        let mut sorted = Shape::EVERY;
        sorted.sort_unstable();
        assert_eq!(sorted, Shape::EVERY);
    }
}
