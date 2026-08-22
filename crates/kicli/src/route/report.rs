//! What a route request answers, in the shape the output contract prints.
//!
//! The type is frozen before the search that fills it and the verbs that print
//! it are written, so both are built against one shape rather than against each
//! other. Every field of `research/wire-routing.md` §8 is here, and nothing that
//! can be worked out from another field is stored: the segment count is the
//! path, the length is the steps walked, and the total is the sum of the cost's
//! parts. A stored derivative is a second answer waiting to disagree with the
//! first.
//!
//! The module knows nothing of exit codes. `Status` says whether a request
//! succeeded and the command layer maps that to a number, because nothing below
//! the command layer may depend on it.
//!
//! **Two fields are the caller's to fill, not the search's.** `Crossing::net`
//! and [`Report::joined_net`] are both connectivity's answers, and the search
//! never learns a net name. They are declared here because the output contract
//! prints them; they are attributed at the seam that already sorts a wire into
//! the route's own net or another's.

use crate::geometry::{Iu, Point};
use crate::model::items::Uuid;
use crate::route::cost::{Cost, Tally};

/// How a route request ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    /// A route was found, and drawn if the caller asked for it.
    Routed,
    /// A pair of labels is proposed instead of a wire.
    Labels,
    /// No route exists, and the report names what stood in the way.
    Blocked,
    /// The request itself was not drawable: a diagonal, an off-grid vertex.
    Invalid,
}

impl Status {
    /// The word the output contract writes.
    #[must_use]
    pub fn token(self) -> &'static str {
        match self {
            Self::Routed => "routed",
            Self::Labels => "labels",
            Self::Blocked => "blocked",
            Self::Invalid => "invalid",
        }
    }

    /// Did the request fail?
    ///
    /// A proposal is a **result**, not a failure: the router was asked what to
    /// do and it answered. Only `blocked` and `invalid` are well-formed
    /// requests that could not be completed.
    #[must_use]
    pub fn is_failure(self) -> bool {
        matches!(self, Self::Blocked | Self::Invalid)
    }
}

/// One crossing of another net, and where it happens.
///
/// The wire is named by its own handle, because that is what the map knows. The
/// **net** it belongs to is connectivity's answer, on the same seam that sorts
/// a wire into the route's own or another's, so the caller attaches it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Crossing {
    /// Where the route crosses.
    pub at: Point,
    /// The wire crossed, as a report names it.
    pub wire: String,
    /// The net that wire carries, when the caller knows it.
    pub net: Option<String>,
}

/// The pair of labels a route proposes instead of a long or blocked wire.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LabelPair {
    /// The name both labels carry.
    pub name: String,
    /// Where each label goes, in the order the terminals were given.
    pub at: [Point; 2],
}

/// What a write added to the drawing.
///
/// Empty until something is written, which is what a proposal reports.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Added {
    /// The wire segments written, one record per segment.
    pub wires: Vec<Uuid>,
    /// The junctions written.
    pub junctions: Vec<Uuid>,
}

/// Why the router put a terminal somewhere other than where it was asked to.
///
/// A closed set, deliberately. An agent decides what to do next by matching on
/// this, so a new reason is a new variant and a compile error at every match —
/// which is the point. Free text would push the decision back onto a parser.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Adjustment {
    /// Terminating where it was asked would have made a fourth wire end meet at
    /// one point, which `spec/SPEC.md` §9 Q2 rules is refused and offset by 1 G.
    FourWayJunction,
}

impl Adjustment {
    /// The word the output contract prints for this reason.
    #[must_use]
    pub fn token(self) -> &'static str {
        match self {
            Self::FourWayJunction => "four-way",
        }
    }
}

/// One terminal the router moved, and what it moved it by.
///
/// The point it moved **to** is not stored: it is the corresponding end of
/// `Report::path`, and a stored derivative is a second answer waiting to
/// disagree with the first. `by` is a displacement rather than a position —
/// `Rect::offset` uses `Point` the same way.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Adjusted {
    /// Which terminal moved, naming itself exactly as `from` or `to` does.
    pub terminal: String,
    /// How far it moved, and in which direction.
    pub by: Point,
    /// Why it moved.
    pub why: Adjustment,
}

/// The answer to one route request.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Report {
    /// How the request ended.
    pub status: Status,
    /// The terminal the route starts at, as it names itself.
    pub from: String,
    /// The terminal it was asked to reach.
    pub to: String,
    /// The vertices of the route, which is empty unless the status is `routed`.
    pub path: Vec<Point>,
    /// What the path met.
    pub tally: Tally,
    /// What that cost, in parts.
    pub cost: Cost,
    /// The crossings, in the order the route makes them.
    pub crossings: Vec<Crossing>,
    /// What was written.
    pub added: Added,
    /// The net the two ends are on now, when the request was to join them.
    ///
    /// **Read back out of the written file, never predicted**, so it is what
    /// the drawing says rather than what the arithmetic that produced it
    /// expected. It is therefore not derivable from any other field here, and
    /// the module's no-stored-derivative rule still holds.
    ///
    /// `None` when nothing was joined: a proposal that wrote no wire, a
    /// connection between ends that name no pin, and a request that was never
    /// asked to join anything — `wire draw` takes the corners it was given and
    /// reports no net.
    pub joined_net: Option<String>,
    /// The labels proposed, when the status is `labels`.
    pub labels: Option<LabelPair>,
    /// What stood in the way, when the status is `blocked`.
    pub blocked_by: Vec<String>,
    /// The terminals the router moved, empty when it moved none.
    ///
    /// Structured because an agent acts on it: an adjusted terminus means the
    /// wire does not end where the request said, and the agent may want to move
    /// a symbol instead of accepting that. `reason` says the same thing in
    /// English for a person, and carries no load this field does not.
    pub adjusted: Vec<Adjusted>,
    /// One sentence for a person, naming the numbers a decision rests on.
    ///
    /// A proposal says both the length and the threshold; a refusal says what
    /// it refused. A route that simply worked needs none.
    pub reason: Option<String>,
    /// How many candidates were tried before this one was chosen.
    pub alternatives_considered: u32,
}

impl Report {
    /// A report of a request that produced no route.
    #[must_use]
    pub fn of(status: Status, from: &str, to: &str) -> Self {
        Self {
            status,
            from: from.to_owned(),
            to: to.to_owned(),
            path: Vec::new(),
            tally: Tally::default(),
            cost: Cost::default(),
            crossings: Vec::new(),
            added: Added::default(),
            joined_net: None,
            labels: None,
            blocked_by: Vec::new(),
            adjusted: Vec::new(),
            reason: None,
            alternatives_considered: 0,
        }
    }

    /// How many wire records the route draws.
    ///
    /// One per segment, because a KiCad wire is always two points. A path of
    /// one point or none draws nothing.
    #[must_use]
    pub fn segments(&self) -> usize {
        self.path.len().saturating_sub(1)
    }

    /// How many corners the route turns.
    #[must_use]
    pub fn corners(&self) -> u32 {
        self.tally.corners
    }

    /// How long the route is, as a distance rather than as a step count.
    #[must_use]
    pub fn length(&self, grid: Iu) -> Iu {
        Iu(i32::try_from(self.tally.steps)
            .unwrap_or(i32::MAX)
            .saturating_mul(grid.0))
    }
}

#[cfg(test)]
mod tests {
    use super::{Added, Adjusted, Adjustment, Crossing, LabelPair, Report, Status};
    use crate::geometry::{GRID, Iu, Point};
    use crate::model::Config;
    use crate::route::cost::{Cost, Tally};

    #[test]
    fn a_proposal_is_a_result_and_a_refusal_is_not() {
        assert!(!Status::Routed.is_failure());
        assert!(!Status::Labels.is_failure(), "a proposal is an answer");
        assert!(Status::Blocked.is_failure());
        assert!(Status::Invalid.is_failure());
        for (status, token) in [
            (Status::Routed, "routed"),
            (Status::Labels, "labels"),
            (Status::Blocked, "blocked"),
            (Status::Invalid, "invalid"),
        ] {
            assert_eq!(status.token(), token);
        }
    }

    #[test]
    fn what_can_be_worked_out_is_not_stored() {
        let mut report = Report::of(Status::Routed, "U1.14", "R7.1");
        report.path = vec![
            Point::new(1_397_000, 889_000),
            Point::new(1_524_000, 889_000),
            Point::new(1_524_000, 1_016_000),
            Point::new(1_651_000, 1_016_000),
        ];
        report.tally = Tally {
            steps: 30,
            corners: 2,
            crossings: 1,
            ..Tally::default()
        };
        report.cost = Cost::of(report.tally, &Config::default().routing);
        report.crossings = vec![Crossing {
            at: Point::new(1_524_000, 889_000),
            wire: "da5aa983".to_owned(),
            net: Some("GND".to_owned()),
        }];
        report.alternatives_considered = 7;

        // The contract's worked example, and every number in it comes from one
        // place: three segments, two corners, 38.10 mm, cost 62.
        assert_eq!(report.segments(), 3);
        assert_eq!(report.corners(), 2);
        assert_eq!(report.length(GRID), Iu(381_000));
        assert_eq!(report.cost.total(), 62);
        assert_eq!(report.status.token(), "routed");
    }

    #[test]
    fn a_report_that_wrote_nothing_says_so() {
        let mut report = Report::of(Status::Labels, "U1.14", "U7.3");
        report.labels = Some(LabelPair {
            name: "SPI_SCK".to_owned(),
            at: [Point::new(0, 0), Point::new(12_700, 0)],
        });
        report.reason = Some("path length 447.04mm is over the threshold 381.00mm".to_owned());
        assert_eq!(report.added, Added::default());
        assert_eq!(report.segments(), 0, "a proposal draws no wire");
        assert_eq!(report.cost.total(), 0);
        assert_eq!(
            report.joined_net, None,
            "a report that wrote no wire joined no net"
        );
    }

    #[test]
    fn a_route_that_moved_nothing_reports_no_adjustment() {
        let report = Report::of(Status::Routed, "U1.14", "R7.1");
        assert!(
            report.adjusted.is_empty(),
            "an empty collection, not an absent one"
        );
    }

    #[test]
    fn an_adjusted_terminal_says_which_by_how_much_and_why() {
        let mut report = Report::of(Status::Routed, "U1.14", "R7.1");
        report.path = vec![
            Point::new(1_397_000, 889_000),
            Point::new(1_524_000, 889_000),
        ];
        report.adjusted = vec![Adjusted {
            terminal: "R7.1".to_owned(),
            by: Point::new(0, 12_700),
            why: Adjustment::FourWayJunction,
        }];

        let moved = &report.adjusted[0];
        assert_eq!(
            moved.terminal, report.to,
            "it names the terminal as `to` does"
        );
        assert_eq!(moved.why.token(), "four-way");

        // Where it ended up is the path's own end, so it is not stored twice:
        // the point the caller asked for is that end less the displacement.
        assert_eq!(
            report.path[1] - moved.by,
            Point::new(1_524_000, 876_300),
            "the requested point is the terminus less the offset, not a stored field"
        );
    }
}
