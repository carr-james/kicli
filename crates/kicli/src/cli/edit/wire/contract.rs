//! The shape every route request answers in, as text and as JSON.
//!
//! One renderer, four statuses. `routed` and `labels` are answers; `blocked`
//! and `invalid` are requests that could not be completed. The command layer
//! turns the status into a row of the exit-code table, and nothing below it
//! knows a number.
//!
//! **The cost breakdown is the point of the exercise.** An agent reads the
//! parts to decide whether to move a symbol instead of accepting a route, so
//! the parts are printed and never summarised away.
//!
//! Three rules shape the two forms.
//!
//! **The JSON carries every key at every status.** A caller parses one shape
//! whatever came back: a list that proposed nothing is empty, and a field that
//! has no value is null. Branching on which keys are present is the thing a
//! stable contract exists to avoid.
//!
//! **The text prints a line only when it has something to say.** Its reader is
//! a context budget. `adjusted` is the case the contract states outright — the
//! line is absent when nothing moved — and the same rule covers the crossings,
//! the proposal, the obstacles and the reason.
//!
//! **The two forms use one vocabulary.** The parts of the cost are named
//! `length`, `turns`, `crossings`, `text` and `proximity` in both, which is
//! what [`Cost::parts`] declares them to be. A text form with its own shorter
//! words would be a second vocabulary for one concept, and an agent that read
//! a name in one form and wrote it in the other would be wrong.

use serde_json::{Value, json};
use std::fmt::Write as _;

use crate::cli::output::Report as Rendered;
use crate::geometry::{Iu, Point};
use crate::route::cost::Cost;
use crate::route::report::{Adjusted, Crossing, LabelPair, Report};

/// Render a route report in both forms.
///
/// `grid` is the placement grid the route was measured on, which is what turns
/// a count of steps into a distance.
#[must_use]
pub fn render(report: &Report, grid: Iu) -> Rendered {
    Rendered {
        text: text(report, grid),
        json: to_json(report, grid),
    }
}

/// The text form.
///
/// **The joined net leads, ahead of the status line.** It is the answer to the
/// question `wire connect` was asked — *what are these two ends on now?* — and
/// it sat above the headline before it was a contract field, in the command
/// layer that prepended it. Moving it into the renderer without moving it on
/// the page keeps every byte an agent already reads, which is what
/// `AGENT.md`'s worked examples show.
fn text(report: &Report, grid: Iu) -> String {
    let mut out = String::new();
    if let Some(net) = &report.joined_net {
        let _ = writeln!(out, "joined: net {net}");
    }
    out.push_str(&headline(report, grid));
    if let Some(reason) = &report.reason {
        let _ = writeln!(out, "  reason: {reason}");
    }
    if report.segments() > 0 {
        let parts: Vec<String> = report
            .cost
            .parts()
            .iter()
            .map(|(name, value)| format!("{name} {value}"))
            .collect();
        let _ = writeln!(
            out,
            "  cost {} = {}",
            report.cost.total(),
            parts.join(" + ")
        );
    }
    detail(&mut out, report);
    out
}

/// The first line: the status, the two ends, and the route's own measurements.
///
/// A request that drew nothing has no measurements, so the line stops after the
/// two ends rather than printing three zeroes.
fn headline(report: &Report, grid: Iu) -> String {
    let mut out = format!("{} {} -> {}", report.status.token(), report.from, report.to);
    if report.segments() > 0 {
        let _ = write!(
            out,
            "   via {} segments, {} corners, {}mm",
            report.segments(),
            report.corners(),
            millimetres(report.length(grid))
        );
    }
    out.push('\n');
    out
}

/// The lines that follow the cost: what the route met, moved, proposed and
/// added. Each one is written only when it has something to say.
fn detail(out: &mut String, report: &Report) {
    if !report.crossings.is_empty() {
        let _ = writeln!(
            out,
            "  crossings: {} ({})",
            report.crossings.len(),
            report
                .crossings
                .iter()
                .map(crossing_clause)
                .collect::<Vec<String>>()
                .join("; ")
        );
    }
    // Stated by the contract: the line is absent, not empty, when the router
    // put both terminals exactly where it was asked to.
    if !report.adjusted.is_empty() {
        let _ = writeln!(
            out,
            "  adjusted: {}",
            report
                .adjusted
                .iter()
                .map(adjusted_clause)
                .collect::<Vec<String>>()
                .join("; ")
        );
    }
    if let Some(labels) = &report.labels {
        let _ = writeln!(
            out,
            "  labels: {:?} at {} and {}",
            labels.name,
            position(labels.at[0]),
            position(labels.at[1])
        );
    }
    if !report.blocked_by.is_empty() {
        let _ = writeln!(out, "  blocked by: {}", report.blocked_by.join(", "));
    }
    let added = &report.added;
    if !added.wires.is_empty() || !added.junctions.is_empty() {
        let _ = writeln!(
            out,
            "  wires added: {}   junctions added: {}",
            added.wires.len(),
            added.junctions.len()
        );
    }
}

/// One crossing, as the text form names it.
///
/// The net is omitted when the caller did not attribute one, because whose a
/// wire is, is connectivity's answer and not the search's. The wire is always
/// there.
fn crossing_clause(crossing: &Crossing) -> String {
    match &crossing.net {
        Some(net) => format!(
            "net {net} at {} on wire {}",
            position(crossing.at),
            crossing.wire
        ),
        None => format!("at {} on wire {}", position(crossing.at), crossing.wire),
    }
}

/// One moved terminal, as the text form names it.
fn adjusted_clause(adjusted: &Adjusted) -> String {
    format!(
        "{} by {}mm ({})",
        adjusted.terminal,
        position(adjusted.by),
        adjusted.why.token()
    )
}

/// The JSON form.
fn to_json(report: &Report, grid: Iu) -> Value {
    json!({
        "status": report.status.token(),
        "from": report.from,
        "to": report.to,
        "path": report.path.iter().copied().map(pair).collect::<Vec<Value>>(),
        "segments": report.segments(),
        "corners": report.corners(),
        "length_mm": report.length(grid).millimetres(),
        "cost": cost(report.cost),
        "crossings": report
            .crossings
            .iter()
            .map(|crossing| json!({
                "wire": crossing.wire,
                "net": crossing.net,
                "at": pair(crossing.at),
            }))
            .collect::<Vec<Value>>(),
        "adjusted": report
            .adjusted
            .iter()
            .map(|adjusted| json!({
                "terminal": adjusted.terminal,
                "by": pair(adjusted.by),
                "why": adjusted.why.token(),
            }))
            .collect::<Vec<Value>>(),
        "added": {
            "wires": report.added.wires.iter().map(|uuid| uuid.0.clone()).collect::<Vec<String>>(),
            "junctions": report.added.junctions.iter().map(|uuid| uuid.0.clone()).collect::<Vec<String>>(),
        },
        "joined_net": report.joined_net,
        "labels": report.labels.as_ref().map(label_pair),
        "blocked_by": report.blocked_by,
        "reason": report.reason,
        "alternatives_considered": report.alternatives_considered,
    })
}

/// The cost, with the total beside the parts it is the sum of.
fn cost(cost: Cost) -> Value {
    let mut fields = serde_json::Map::new();
    fields.insert("total".to_owned(), cost.total().into());
    for (name, value) in cost.parts() {
        fields.insert(name.to_owned(), value.into());
    }
    Value::Object(fields)
}

/// A proposed pair of labels.
fn label_pair(labels: &LabelPair) -> Value {
    json!({
        "name": labels.name,
        "at": labels.at.iter().copied().map(pair).collect::<Vec<Value>>(),
    })
}

/// A point, as two millimetre numbers.
fn pair(point: Point) -> Value {
    json!([point.x.millimetres(), point.y.millimetres()])
}

/// One millimetre reading, to the two decimals the views print.
fn millimetres(length: Iu) -> String {
    format!("{:.2}", length.millimetres())
}

/// A position, as the two-decimal pair the views print.
fn position(point: Point) -> String {
    format!("{},{}", millimetres(point.x), millimetres(point.y))
}

#[cfg(test)]
mod tests {
    use super::{render, to_json};
    use crate::geometry::{GRID, Point};
    use crate::model::Config;
    use crate::route::cost::{Cost, Tally};
    use crate::route::report::{Adjusted, Adjustment, Crossing, Report, Status};

    /// The worked example of the output contract, built by hand.
    fn worked_example() -> Report {
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
        report
    }

    #[test]
    fn a_route_that_moved_nothing_prints_no_adjusted_line() {
        let rendered = render(&worked_example(), GRID);
        assert!(
            !rendered.text.contains("adjusted"),
            "the line is absent, not empty: {}",
            rendered.text
        );
        assert_eq!(
            rendered.json["adjusted"],
            serde_json::json!([]),
            "the key is there and empty, so one parse covers both cases"
        );
    }

    #[test]
    fn a_moved_terminal_names_itself_and_the_displacement() {
        let mut report = worked_example();
        report.adjusted = vec![Adjusted {
            terminal: "R7.1".to_owned(),
            by: Point::new(0, 12_700),
            why: Adjustment::FourWayJunction,
        }];
        let rendered = render(&report, GRID);
        assert!(
            rendered
                .text
                .contains("  adjusted: R7.1 by 0.00,1.27mm (four-way)\n"),
            "{}",
            rendered.text
        );
        assert_eq!(rendered.json["adjusted"][0]["terminal"], "R7.1");
        assert_eq!(rendered.json["adjusted"][0]["why"], "four-way");
    }

    #[test]
    fn a_crossing_with_no_net_prints_the_wire_alone() {
        let mut report = worked_example();
        report.crossings[0].net = None;
        let rendered = render(&report, GRID);
        assert!(
            rendered
                .text
                .contains("  crossings: 1 (at 152.40,88.90 on wire da5aa983)\n"),
            "{}",
            rendered.text
        );
        assert_eq!(
            rendered.json["crossings"][0]["net"],
            serde_json::Value::Null,
            "the key stays, so a caller never branches on its absence"
        );
    }

    #[test]
    fn a_route_that_joined_nothing_prints_no_line_and_still_carries_the_key() {
        let rendered = render(&worked_example(), GRID);
        assert!(
            !rendered.text.contains("joined:"),
            "the text line is absent when there is nothing to say: {}",
            rendered.text
        );
        // Presence and null are asserted apart, because a reader that only
        // asks for the value cannot tell a null from a dropped key — and
        // every-key-at-every-status is the rule being checked.
        assert!(
            rendered
                .json
                .as_object()
                .expect("the contract is an object")
                .contains_key("joined_net"),
            "the key is there at every status: {}",
            rendered.json
        );
        assert_eq!(
            rendered.json["joined_net"],
            serde_json::Value::Null,
            "and it is null rather than absent"
        );
    }

    #[test]
    fn a_joined_net_leads_the_text_and_names_itself_in_json() {
        let mut report = worked_example();
        report.joined_net = Some("SIG_A".to_owned());
        let rendered = render(&report, GRID);
        // The line sits above the status line, which is where the command
        // layer put it before it was a contract field. `AGENT.md` shows that
        // order, so the whole first line is asserted rather than a substring.
        assert!(
            rendered.text.starts_with(
                "joined: net SIG_A
routed U1.14 -> R7.1"
            ),
            "{}",
            rendered.text
        );
        assert_eq!(rendered.json["joined_net"], "SIG_A");
    }

    #[test]
    fn the_total_is_the_sum_of_the_parts_it_is_printed_beside() {
        let json = to_json(&worked_example(), GRID);
        let parts: i64 = ["length", "turns", "crossings", "text", "proximity"]
            .iter()
            .map(|name| json["cost"][name].as_i64().expect("a whole number"))
            .sum();
        assert_eq!(json["cost"]["total"].as_i64(), Some(parts));
        assert_eq!(json["cost"]["total"], 62, "the contract's worked example");
    }
}
