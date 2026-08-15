//! A\* over turn-aware states, for the routes no silhouette can draw.
//!
//! [`Shapes`](crate::route::shapes::Shapes) is the fast path and the common
//! one: half of all real segments are under 7 mm and a person draws them as an
//! I or an L. This is what answers when every silhouette meets something.
//!
//! **A state is `(x, y, dir)`.** A corner costs, so the cost of reaching a
//! point depends on the direction it was reached from, and a search over bare
//! points would price the same cell two ways and keep the wrong one.
//! Successors are expanded in the order `research/wire-routing.md` §4.3 fixes,
//! [`Heading::EVERY`], and the queue orders on the total order `(f, g, x, y,
//! dir)` — so **no tie is ever resolved by heap internals**. That is what makes
//! two runs over one sheet produce one route, on any machine, forever.
//!
//! **The heuristic is the Manhattan distance priced at `w_len`, plus `w_turn`
//! where a turn is unavoidable.** Every remaining step costs at least `w_len`
//! and there are at least that many of them; where the goal is off both the
//! current axis and the current heading, at least one corner is still to come.
//! Every term is non-negative, so the estimate never exceeds the truth and the
//! first goal state off the queue is the cheapest route there is.
//!
//! **The target-cell exception is repeated here, and it is a twin.** §3.2's
//! first row ends "except the grid point at a target pin", and
//! [`Tally::of_path`] excepts the two ends of the path it walks. This module
//! cannot inherit that: it queries [`Obstacles::entering`] a step at a time
//! rather than walking a finished path. So it excepts **its own goal cell, to
//! the same rule** — a block is excepted only where the step is the one that
//! ends the route, and what that step costs is still counted. The other end
//! needs no exception in either home: the walk never enters the first point,
//! and the search starts there rather than stepping onto it.
//!
//! Two homes for one rule drift silently, and the drift shows up as a route the
//! search finds and the cost model then refuses — which reads as a router bug
//! rather than as a rule in two places.
//! `tests/route_search.rs::the_search_and_the_walk_except_the_same_ends` is the
//! check that fails when they part.

use std::cmp::Reverse;
use std::collections::{BTreeMap, BinaryHeap};

use crate::geometry::Point;
use crate::model::config::Routing;
use crate::route::cost::{Cost, Tally};
use crate::route::obstacles::Obstacles;
use crate::route::terminal::{Heading, Terminal};
use crate::route::window::Cell;

/// One node of the search: a grid point, and the direction it was reached from.
///
/// The derived comparison is `(x, y, dir)`, because a [`Cell`] index grows with
/// the coordinate it counts and [`Heading`] is declared in the expansion order.
/// Nothing carries a tie-break beside it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct State {
    /// The grid point, as an index into the routing window.
    pub at: Cell,
    /// The direction the route was travelling when it arrived.
    pub dir: Heading,
}

/// One entry of the search's queue.
///
/// The derived comparison is the total order `(f, g, x, y, dir)` the search
/// promises. Two entries compare equal only when they are the same entry, so
/// the queue's own structure never decides which of two routes is found —
/// which is what `tests/route_search.rs::the_queue_order_is_total` measures.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Queued {
    /// The estimated cost of the whole route through this state.
    pub f: i64,
    /// What reaching this state has cost so far.
    pub g: i64,
    /// The state itself.
    pub state: State,
}

/// The route the search found, with what it costs.
///
/// The shape of a [`Candidate`](crate::route::shapes::Candidate), minus the
/// silhouette: A\* answers with a route rather than with a name for it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Route {
    /// The vertices, from the source terminal to the target terminal.
    pub path: Vec<Point>,
    /// What walking it meets.
    pub tally: Tally,
    /// What that costs, in parts.
    pub cost: Cost,
}

/// What one search request produced.
#[derive(Clone, Debug, Default)]
pub struct Search {
    route: Option<Route>,
    blocked_by: Vec<String>,
    expanded: u32,
}

impl Search {
    /// Search for the cheapest route between two terminals.
    ///
    /// The escape rule is built in at both ends: the first step leaves along
    /// the source's own direction, and only a state that arrives along the
    /// target's counts as reaching it. A terminal that fixes no direction — a
    /// point on an existing net — may be left and met any way.
    ///
    /// A terminal off the lattice names no cell and the answer is then simply
    /// empty. The caller refuses an off-grid terminal before it asks, because
    /// moving somebody's pin is not a routing decision.
    #[must_use]
    pub fn of(
        source: &Terminal,
        target: &Terminal,
        obstacles: &Obstacles,
        weights: &Routing,
    ) -> Self {
        let mut found = Self::default();
        let window = obstacles.window();
        let (Some(start), Some(goal)) = (window.cell(source.at), window.cell(target.at)) else {
            return found;
        };
        if start == goal {
            return found;
        }
        let field = Field {
            obstacles,
            weights,
            goal,
            arrival: target.escape.map(Heading::reversed),
        };

        let mut frontier = Frontier::default();
        let openings: &[Heading] = match &source.escape {
            Some(heading) => std::slice::from_ref(heading),
            None => &Heading::EVERY,
        };
        for &heading in openings {
            if let Some((state, cost)) = found.advance(source.at, heading, false, &field) {
                frontier.admit(state, cost, None, &field);
            }
        }

        while let Some(queued) = frontier.pop() {
            if frontier.is_stale(&queued) {
                continue;
            }
            found.expanded = found.expanded.saturating_add(1);
            if field.reaches(queued.state) {
                found.route = Some(route_of(source.at, &frontier.trace(queued.state), &field));
                break;
            }
            let from = field.obstacles.window().point(queued.state.at);
            for &heading in &Heading::EVERY {
                // Stepping back the way it came would draw a wire over the
                // route's own last segment, which reads as a connection rather
                // than as a corner. It is not a route, so it is not expanded.
                if heading == queued.state.dir.reversed() {
                    continue;
                }
                let turning = heading != queued.state.dir;
                if let Some((state, cost)) = found.advance(from, heading, turning, &field) {
                    frontier.admit(state, queued.g + cost, Some(queued.state), &field);
                }
            }
        }
        found
    }

    /// The route, when the search found one.
    #[must_use]
    pub fn route(&self) -> Option<&Route> {
        self.route.as_ref()
    }

    /// What stood in the way, named once each, in the order first met.
    ///
    /// A failure is never a bare failure: an agent given a list of handles can
    /// move a symbol or reroute a net, where an agent told "no route" can only
    /// guess. The list is what refused a step, so it is filled whether or not a
    /// route was found in the end; it is the answer to a refusal, and a caller
    /// holding a route has no use for it.
    ///
    /// The window's own edge is not in the list. It bounds the search rather
    /// than standing in a drawing, and a handle that names no object is a line
    /// an agent cannot act on.
    #[must_use]
    pub fn blocked_by(&self) -> &[String] {
        &self.blocked_by
    }

    /// How many states the search took off its queue.
    ///
    /// What a report carries as `alternatives_considered` when the search
    /// rather than the shapes produced the route: the states it looked at, not
    /// the ones it could use.
    #[must_use]
    pub fn expanded(&self) -> u32 {
        self.expanded
    }

    /// One step onto the next grid point, and what it costs.
    ///
    /// Nothing, when the step leaves the window or when something refuses it —
    /// and **the goal cell is excepted from a refusal exactly where the step is
    /// the one that ends the route**, which is [`Tally::of_path`]'s rule in the
    /// other of its two homes. A step into the goal cell that does not end the
    /// route is a route passing through its own target, and the walk refuses
    /// that too.
    fn advance(
        &mut self,
        from: Point,
        heading: Heading,
        turning: bool,
        field: &Field,
    ) -> Option<(State, i64)> {
        let window = field.obstacles.window();
        let to = heading.step(from, window.grid());
        // A point outside the window names no cell. `from` is on the lattice
        // and the step is exactly one grid step, so nothing else can miss it.
        let at = window.cell(to)?;
        let state = State { at, dir: heading };
        let verdict = field.obstacles.entering(to, heading);
        if let Some(handle) = verdict.blocked_by {
            if !field.reaches(state) {
                if !self.blocked_by.contains(&handle) {
                    self.blocked_by.push(handle);
                }
                return None;
            }
        }
        let mut cost = field.weights.w_len
            + field.weights.w_cross * i64::from(verdict.crossings)
            + field.weights.w_text * i64::from(verdict.text_steps)
            + field.weights.w_near * i64::from(verdict.near_steps);
        if turning {
            cost += field.weights.w_turn;
        }
        Some((state, cost))
    }
}

/// Everything one step is measured against, so that one step is one call.
struct Field<'a> {
    obstacles: &'a Obstacles,
    weights: &'a Routing,
    goal: Cell,
    arrival: Option<Heading>,
}

impl Field<'_> {
    /// Does this state end the route?
    ///
    /// The escape rule at the target end: a route arrives along the terminal's
    /// own direction, so a state on the goal cell that got there any other way
    /// has not arrived. A terminal that fixes no direction may be met from any
    /// side.
    fn reaches(&self, state: State) -> bool {
        state.at == self.goal && self.arrival.is_none_or(|heading| state.dir == heading)
    }

    /// What the rest of the route must cost at least.
    fn heuristic(&self, state: State) -> i64 {
        let dx = i64::from(self.goal.column - state.at.column);
        let dy = i64::from(self.goal.row - state.at.row);
        let mut estimate = self.weights.w_len * (dx.abs() + dy.abs());
        if turns_again(state.dir, dx, dy) {
            estimate += self.weights.w_turn;
        }
        estimate
    }
}

/// Must a route from here to there turn at least once more?
///
/// It must when the goal is off both axes, and when the one axis it is on is
/// not the one being travelled. Answering "no" where the truth is "yes" only
/// loses the corner from the estimate, which keeps it under the truth.
fn turns_again(dir: Heading, dx: i64, dy: i64) -> bool {
    if dx != 0 && dy != 0 {
        return true;
    }
    let needed = match (dx, dy) {
        (0, 0) => return false,
        (dx, _) if dx > 0 => Heading::PlusX,
        (dx, _) if dx < 0 => Heading::MinusX,
        (_, dy) if dy > 0 => Heading::PlusY,
        _ => Heading::MinusY,
    };
    needed != dir
}

/// The open set, the best cost known for each state, and where each came from.
///
/// The two maps are ordered, because an unordered one would let iteration order
/// reach a decision.
#[derive(Default)]
struct Frontier {
    queue: BinaryHeap<Reverse<Queued>>,
    best: BTreeMap<State, i64>,
    came: BTreeMap<State, State>,
}

impl Frontier {
    /// Offer a state at a cost, and keep it if that beats what is known.
    ///
    /// A state is only ever pushed at a strictly smaller `g` than before, so no
    /// two entries in the queue are the same entry and the total order decides
    /// every pop.
    ///
    /// The improvement branch is a guard rather than a rule the answers rest
    /// on: the heuristic is consistent, so the first time a state comes off the
    /// queue it is already as cheap as it will get. Measured, 2026-08-15 —
    /// refusing every re-offer outright changes no answer this milestone's
    /// checks can see. It is kept because consistency is a property of the
    /// weights, which are configuration.
    fn admit(&mut self, state: State, g: i64, from: Option<State>, field: &Field) {
        if self.best.get(&state).is_some_and(|&held| held <= g) {
            return;
        }
        self.best.insert(state, g);
        match from {
            Some(previous) => {
                self.came.insert(state, previous);
            }
            None => {
                self.came.remove(&state);
            }
        }
        self.queue.push(Reverse(Queued {
            f: g + field.heuristic(state),
            g,
            state,
        }));
    }

    /// The cheapest entry the queue holds.
    fn pop(&mut self) -> Option<Queued> {
        self.queue.pop().map(|Reverse(queued)| queued)
    }

    /// Was this entry left behind by a cheaper way to the same state?
    ///
    /// Expanding it again would relax nothing, so it is skipped rather than
    /// answered differently.
    fn is_stale(&self, queued: &Queued) -> bool {
        self.best
            .get(&queued.state)
            .is_some_and(|&held| held < queued.g)
    }

    /// The states of the route that ends here, in the order it walks them.
    fn trace(&self, last: State) -> Vec<State> {
        let mut states = vec![last];
        let mut here = last;
        while let Some(&previous) = self.came.get(&here) {
            states.push(previous);
            here = previous;
        }
        states.reverse();
        states
    }
}

/// The route a chain of states draws, counted as it is walked.
///
/// The tally is the search's own count over its own states, rather than a
/// second reading of [`Tally::of_path`]: the two are compared by the check that
/// holds the twins together, and a tally copied from the other home would make
/// that check agree with itself.
///
/// A vertex is written where the direction changes, so the path holds one point
/// per corner and no point in the middle of a straight run — a reader sees one
/// wire there and the file should hold one record.
fn route_of(start: Point, states: &[State], field: &Field) -> Route {
    let window = field.obstacles.window();
    let mut path = vec![start];
    let mut tally = Tally::default();
    let mut heading: Option<Heading> = None;
    for (index, state) in states.iter().enumerate() {
        let at = window.point(state.at);
        let verdict = field.obstacles.entering(at, state.dir);
        if heading.is_some_and(|last| last != state.dir) {
            tally.corners += 1;
            // The corner is where the step before this one ended. A direction
            // can only change after a first step, so there is always one.
            if let Some(previous) = index.checked_sub(1).and_then(|before| states.get(before)) {
                path.push(window.point(previous.at));
            }
        }
        tally.steps += 1;
        tally.crossings += verdict.crossings;
        tally.text_steps += verdict.text_steps;
        tally.near_steps += verdict.near_steps;
        heading = Some(state.dir);
    }
    if let Some(last) = states.last() {
        path.push(window.point(last.at));
    }
    Route {
        path,
        tally,
        cost: Cost::of(tally, field.weights),
    }
}

#[cfg(test)]
mod tests {
    //! The answers of this module that no drawing can ask for.
    //!
    //! A request always names two terminals of a loaded sheet, and the caller
    //! refuses an off-grid terminal before it asks — so neither a terminal
    //! outside the window nor a request whose two ends are one point can be
    //! drawn. Neither can a route that carries on **through** its own target:
    //! a terminal's own cell is covered by its own body, so a route that did
    //! not stop there would step into the body next and be refused anyway.
    //! Hand-stated input is permitted **only where no drawable request can
    //! distinguish the behaviour** (ruled, M4 T7), which is the condition each
    //! test here meets, and the reason the rules they measure are kept: what
    //! makes them unreachable is another module's invariant rather than this
    //! one's.

    use super::{Field, Search, State, turns_again};
    use crate::geometry::{GRID, Point, Rect};
    use crate::model::Config;
    use crate::route::obstacles::{Obstacles, SheetGeometry};
    use crate::route::terminal::{Heading, Obstruction, Terminal};
    use crate::route::window::Window;

    /// An empty map over a window around two points.
    fn empty(from: Point, to: Point) -> Obstacles {
        let page = Rect::new(Point::default(), Point::new(80 * GRID.0, 80 * GRID.0));
        let window = Window::around(from, to, Config::default().routing.margin, page, GRID);
        Obstacles::build(window, &SheetGeometry::default())
    }

    #[test]
    fn the_goal_cell_is_excepted_for_arriving_and_not_for_passing_through() {
        // The exception is narrowed to the step that **ends** the route, which
        // is the rule `Tally::of_path` keeps: the walk excepts the last point
        // of the path and refuses a block anywhere before it. A route that
        // entered its own target and carried on would be a block the walk
        // refuses and the search allowed — the divergence in the direction no
        // drawing can produce.
        let weights = Config::default().routing;
        let here = Point::new(20 * GRID.0, 20 * GRID.0);
        let there = Point::new(24 * GRID.0, 20 * GRID.0);
        let page = Rect::new(Point::default(), Point::new(80 * GRID.0, 80 * GRID.0));
        let window = Window::around(here, there, weights.margin, page, GRID);
        // A body over the target, as a symbol's box covers its own pin.
        let body = [Obstruction {
            handle: "U1".to_owned(),
            area: Rect::new(there, Point::new(there.x.0 + GRID.0, there.y.0)),
        }];
        let obstacles = Obstacles::build(
            window,
            &SheetGeometry {
                symbol_bodies: &body,
                ..SheetGeometry::default()
            },
        );
        let goal = window.cell(there).expect("the window holds the target");
        let field = Field {
            obstacles: &obstacles,
            weights: &weights,
            goal,
            arrival: Some(Heading::PlusX),
        };

        // Arriving: the step that ends the route is excepted, and it is still
        // costed — one grid step, at the length weight.
        let mut search = Search::default();
        let before = Point::new(there.x.0 - GRID.0, there.y.0);
        let (state, cost) = search
            .advance(before, Heading::PlusX, false, &field)
            .expect("the route arrives at the cell its own body covers");
        assert_eq!(
            state,
            State {
                at: goal,
                dir: Heading::PlusX
            }
        );
        assert_eq!(cost, weights.w_len);
        assert!(search.blocked_by().is_empty());

        // Passing through: the same cell entered any other way is not the end
        // of the route, so the block stands and names what stands there.
        let above = Point::new(there.x.0, there.y.0 - GRID.0);
        assert!(
            search
                .advance(above, Heading::PlusY, true, &field)
                .is_none(),
            "a route carried on through its own target"
        );
        assert_eq!(search.blocked_by(), ["U1"]);
    }

    #[test]
    fn a_request_the_lattice_cannot_hold_has_no_route() {
        let weights = Config::default().routing;
        let here = Point::new(20 * GRID.0, 20 * GRID.0);
        let there = Point::new(24 * GRID.0, 20 * GRID.0);
        let map = empty(here, there);

        // A route from a point to itself draws nothing.
        let one = Terminal::of_point(here, "NET");
        let none = Search::of(&one, &one, &map, &weights);
        assert!(none.route().is_none());
        assert!(none.blocked_by().is_empty(), "and it blames nothing");

        // A terminal the window does not hold names no cell.
        let outside = Terminal::of_point(Point::new(60 * GRID.0, 60 * GRID.0), "AWAY");
        assert!(map.window().cell(outside.at).is_none());
        assert!(Search::of(&one, &outside, &map, &weights).route().is_none());
        assert!(Search::of(&outside, &one, &map, &weights).route().is_none());

        // The control: the same request between two points the window holds is
        // routed, so what refused the three above is the lattice and not a
        // search that refuses everything.
        let other = Terminal::of_point(there, "NET");
        let route = Search::of(&one, &other, &map, &weights)
            .route()
            .expect("an empty window routes a straight run")
            .clone();
        assert_eq!(route.path, vec![here, there]);
        assert_eq!(route.tally.steps, 4);
        assert_eq!(route.tally.corners, 0);
    }

    #[test]
    fn a_route_over_an_empty_window_turns_once_and_says_so() {
        // Four steps across and three down over an empty window: the route a
        // person draws turns once, and the path holds one vertex for that
        // corner and none in the middle of either run.
        //
        // **Measured, 2026-08-15: this does not watch the corner cost.** With
        // `w_turn` taken out of both the step and the estimate, the search
        // still answers with the one-cornered route — the queue's total order
        // settles the tie the corner cost was meant to settle. What the check
        // does watch is the tally and the vertices agreeing with each other,
        // which is what the two breaks recorded against it fire on.
        let weights = Config::default().routing;
        let here = Point::new(20 * GRID.0, 20 * GRID.0);
        let there = Point::new(24 * GRID.0, 23 * GRID.0);
        let map = empty(here, there);
        let route = Search::of(
            &Terminal::of_point(here, "NET"),
            &Terminal::of_point(there, "NET"),
            &map,
            &weights,
        )
        .route()
        .expect("an empty window routes a corner")
        .clone();
        assert_eq!(route.tally.steps, 7);
        assert_eq!(route.tally.corners, 1, "{:?}", route.path);
        assert_eq!(
            route.path.len(),
            3,
            "one vertex per corner: {:?}",
            route.path
        );
        assert_eq!(route.cost.total(), 7 + weights.w_turn);
    }

    #[test]
    fn a_corner_still_to_come_is_in_the_estimate() {
        // The heuristic's second term, at the four ways it can be reached. It
        // is measured here because a drawing can only show the route it
        // produced, never the estimate that ordered the queue.
        assert!(turns_again(Heading::PlusX, 3, 2), "off both axes");
        assert!(turns_again(Heading::PlusY, -3, 0), "on the other axis");
        assert!(turns_again(Heading::MinusX, 3, 0), "the wrong way along it");
        assert!(!turns_again(Heading::PlusX, 3, 0), "straight on");
        assert!(!turns_again(Heading::PlusX, 0, 0), "already there");
    }
}
