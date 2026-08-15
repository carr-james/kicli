//! When a connection is proposed as a pair of labels instead of drawn.
//!
//! `research/wire-routing.md` §5.5 gives two triggers: the best path is longer
//! than `routing.label_threshold`, or nothing routes between the two terminals
//! at all. Either way the answer is a **proposal**. This module writes nothing
//! and knows nothing of files. It answers with the name both labels carry, the
//! anchor each one goes on, and the sentence that says why — and a caller that
//! does not ask for the proposal to be performed leaves the drawing exactly as
//! it found it.
//!
//! **`routing.label_threshold` is one knob, read twice.** The router decides
//! with it and the long-wire style rule judges with it. The value is read from
//! the configuration on every request, so neither side holds one of its own.
//!
//! The decision is integer arithmetic, like the rest of the router: a length in
//! internal units against a threshold in internal units. The millimetre
//! readings in the sentence are the presentation of a decision already made.

use crate::geometry::{Iu, Point};
use crate::model::config::Routing;
use crate::route::report::{LabelPair, Report, Status};
use crate::route::terminal::Terminal;

/// How far along a terminal's own direction a proposed label's anchor sits.
///
/// Two grid steps, which `research/wire-routing.md` §5.5 fixes. One grid step
/// is the escape point, which is where a route may first turn, so a label there
/// would sit on the corner of every wire that leaves the pin. Two steps leave
/// the corner clear and still put the name beside the pin it names.
pub const REACH: i32 = 2;

/// Why a connection is proposed as a pair of labels rather than drawn.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Trigger {
    /// The best path is longer than the configured threshold.
    TooLong {
        /// How long the best path is.
        length: Iu,
        /// The boundary it is over, as the configuration gives it.
        threshold: Iu,
    },
    /// No route joins the two terminals at all.
    NoRoute,
}

impl Trigger {
    /// Is this connection one to propose as labels?
    ///
    /// `best` is the length of the cheapest route, and nothing when the search
    /// found none. A path exactly at the threshold is drawn: the threshold is
    /// the length a wire may still be.
    #[must_use]
    pub fn of(best: Option<Iu>, weights: &Routing) -> Option<Self> {
        match best {
            None => Some(Self::NoRoute),
            Some(length) if length.0 > weights.label_threshold.0 => Some(Self::TooLong {
                length,
                threshold: weights.label_threshold,
            }),
            Some(_) => None,
        }
    }

    /// The sentence a person reads, naming the numbers the decision rests on.
    ///
    /// A proposal made on length says both the length and the threshold, so a
    /// caller can see how far over it is and decide to move a symbol instead.
    #[must_use]
    pub fn reason(self, target: &str) -> String {
        match self {
            Self::TooLong { length, threshold } => format!(
                "path length {}mm is over the threshold {}mm",
                millimetres(length),
                millimetres(threshold)
            ),
            Self::NoRoute => format!("no route reaches {target}, so a pair of labels joins it"),
        }
    }
}

/// The pair of labels one connection is proposed as.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Proposal {
    /// Why the connection is proposed rather than drawn.
    pub trigger: Trigger,
    /// The name both labels carry, and where each one goes.
    pub labels: LabelPair,
}

impl Proposal {
    /// Consider one connection, and answer with the pair to propose.
    ///
    /// Nothing, when the best path is short enough to draw — which is the
    /// common answer and the one that leaves the drawing alone.
    #[must_use]
    pub fn of(
        source: &Terminal,
        target: &Terminal,
        best: Option<Iu>,
        name: &str,
        weights: &Routing,
        grid: Iu,
    ) -> Option<Self> {
        Some(Self {
            trigger: Trigger::of(best, weights)?,
            labels: LabelPair {
                name: name.to_owned(),
                at: [anchor(source, grid), anchor(target, grid)],
            },
        })
    }

    /// The answer a proposed connection makes, in the shape the contract prints.
    ///
    /// It carries no path and nothing added, because a proposal draws nothing.
    /// A caller that performs it fills in what the write added.
    #[must_use]
    pub fn report(&self, source: &Terminal, target: &Terminal) -> Report {
        let mut report = Report::of(Status::Labels, &source.name, &target.name);
        report.labels = Some(self.labels.clone());
        report.reason = Some(self.trigger.reason(&target.name));
        report
    }
}

/// Where a proposed label's anchor goes.
///
/// [`REACH`] grid steps along the terminal's own direction, which is the
/// direction a wire leaves it by. A terminal that fixes no direction — a point
/// on an existing net — is its own anchor: there is no direction to step along,
/// and the point already carries the net the label would name.
#[must_use]
pub fn anchor(terminal: &Terminal, grid: Iu) -> Point {
    match terminal.escape {
        Some(heading) => heading.step(terminal.at, Iu(grid.0.saturating_mul(REACH))),
        None => terminal.at,
    }
}

/// The name both labels carry.
///
/// The net's own name when the drawing gives it one. When it does not, the name
/// is made from the source end: `<reference>_<pin name>`, which says where the
/// signal comes from. A pin with no name of its own falls back to its number,
/// because `U1_` names nothing.
#[must_use]
pub fn label_name(net: Option<&str>, reference: &str, pin_name: &str, pin_number: &str) -> String {
    match net {
        Some(name) => name.to_owned(),
        None if pin_name.is_empty() => format!("{reference}_{pin_number}"),
        None => format!("{reference}_{pin_name}"),
    }
}

/// How long a path is, as the vertices walk it.
///
/// Each leg is counted along its own axis, so a pair of ends with no corners
/// between them measures the Manhattan distance — which no orthogonal route can
/// beat, and is therefore the right length to judge a connection nobody has
/// drawn a path for yet.
#[must_use]
pub fn walked(vertices: &[Point]) -> Iu {
    let mut length: i32 = 0;
    for pair in vertices.windows(2) {
        let leg = (pair[1].x.0 - pair[0].x.0).abs() + (pair[1].y.0 - pair[0].y.0).abs();
        length = length.saturating_add(leg);
    }
    Iu(length)
}

/// One length in millimetres, to the two decimals every view prints.
fn millimetres(length: Iu) -> String {
    format!("{:.2}", length.millimetres())
}

#[cfg(test)]
mod tests {
    use super::{Proposal, Trigger, anchor, label_name, walked};
    use crate::geometry::{GRID, Iu, Point};
    use crate::model::Config;
    use crate::route::terminal::{Heading, Terminal};

    /// A point a whole number of grid steps from the origin.
    fn at(column: i32, row: i32) -> Point {
        Point::new(column * GRID.0, row * GRID.0)
    }

    /// A terminal that leaves the given way, at a stated point.
    fn terminal(at: Point, escape: Option<Heading>, name: &str) -> Terminal {
        Terminal {
            at,
            escape,
            name: name.to_owned(),
        }
    }

    #[test]
    fn the_threshold_is_the_length_a_wire_may_still_be() {
        let weights = Config::default().routing;
        let threshold = weights.label_threshold;
        assert_eq!(threshold, Iu(30 * GRID.0), "the configured default");

        // Exactly at the boundary is drawn, one unit over it is proposed. The
        // boundary itself is the case a comparison written the other way round
        // would answer differently, so it is the one worth stating.
        assert_eq!(Trigger::of(Some(threshold), &weights), None);
        assert_eq!(
            Trigger::of(Some(Iu(threshold.0 + 1)), &weights),
            Some(Trigger::TooLong {
                length: Iu(threshold.0 + 1),
                threshold,
            })
        );
        assert_eq!(Trigger::of(Some(Iu(0)), &weights), None);

        // A search that found nothing proposes whatever the length would have
        // been, because there is no length.
        assert_eq!(Trigger::of(None, &weights), Some(Trigger::NoRoute));

        // And the value is read rather than held: a threshold of one grid step
        // moves the boundary onto a route the default draws.
        let mut lowered = weights;
        lowered.label_threshold = GRID;
        assert!(Trigger::of(Some(Iu(4 * GRID.0)), &lowered).is_some());
        assert_eq!(Trigger::of(Some(Iu(4 * GRID.0)), &weights), None);
    }

    #[test]
    fn a_proposal_names_both_numbers_the_decision_rests_on() {
        let weights = Config::default().routing;
        let source = terminal(at(4, 40), Some(Heading::MinusY), "U1.1");
        let target = terminal(at(120, 40), Some(Heading::PlusY), "U2.2");
        let proposal = Proposal::of(
            &source,
            &target,
            Some(Iu(116 * GRID.0)),
            "SPI_SCK",
            &weights,
            GRID,
        )
        .expect("a route that long is proposed as labels");

        let reason = proposal.trigger.reason(&target.name);
        assert!(reason.contains("147.32mm"), "the length: {reason}");
        assert!(reason.contains("38.10mm"), "and the threshold: {reason}");

        let report = proposal.report(&source, &target);
        assert_eq!(report.status.token(), "labels");
        assert!(report.path.is_empty(), "a proposal draws no wire");
        assert_eq!(report.segments(), 0);
        assert_eq!(report.added, crate::route::report::Added::default());
        let pair = report.labels.as_ref().expect("the pair is proposed");
        assert_eq!(pair.name, "SPI_SCK");
        assert_eq!(pair.at, [at(4, 38), at(120, 42)]);
    }

    #[test]
    fn an_anchor_steps_along_the_terminal_it_belongs_to() {
        // Two grid steps along the way a wire leaves, at each of the four
        // headings, so a sign that was inverted for one is caught.
        for (heading, expected) in [
            (Heading::PlusX, at(12, 10)),
            (Heading::MinusX, at(8, 10)),
            (Heading::PlusY, at(10, 12)),
            (Heading::MinusY, at(10, 8)),
        ] {
            let pin = terminal(at(10, 10), Some(heading), "U1.1");
            assert_eq!(anchor(&pin, GRID), expected, "{heading:?}");
            assert!(anchor(&pin, GRID).is_on_grid(), "{heading:?} is on grid");
        }
        // A point on an existing net fixes no direction and is its own anchor.
        let point = terminal(at(10, 10), None, "GND");
        assert_eq!(anchor(&point, GRID), at(10, 10));
    }

    #[test]
    fn a_name_comes_from_the_net_or_from_the_source_pin() {
        assert_eq!(label_name(Some("SPI_SCK"), "U1", "SCK", "14"), "SPI_SCK");
        assert_eq!(label_name(None, "U1", "SCK", "14"), "U1_SCK");
        // A pin with no name of its own is named by its number instead.
        assert_eq!(label_name(None, "R7", "", "1"), "R7_1");
    }

    #[test]
    fn a_length_is_the_legs_walked_and_never_the_line_between() {
        // Two ends and no corner measure the Manhattan distance, which is the
        // shortest any orthogonal route could be.
        assert_eq!(walked(&[at(0, 0), at(3, 4)]), Iu(7 * GRID.0));
        // Corners are walked leg by leg, so a detour is longer than the pair.
        assert_eq!(
            walked(&[at(0, 0), at(0, 6), at(3, 6), at(3, 4)]),
            Iu(11 * GRID.0)
        );
        assert_eq!(walked(&[at(0, 0)]), Iu(0));
        assert_eq!(walked(&[]), Iu(0));
    }
}
