//! The shape a route request answers in, against committed goldens.
//!
//! One golden per status, and one more for a terminal the router moved. Each
//! file holds the text form, then a `---` line, then the JSON form, so the two
//! halves of the contract for one case read as one document. A change to
//! either half is a change to the golden, and the golden change is part of the
//! change.
//!
//! **Only the `routed` golden comes from a drawing.** It is produced by
//! `edit::wire::draw` on a probe drawing, so the renderer is measured against
//! what the verb actually hands it rather than against a value written to suit
//! the renderer. The rest are constructed here, and say so where they are
//! constructed: nothing in this build produces them from a drawing. A proposal
//! comes from the label fallback, a blocked route and a moved terminal come
//! from the router, none of which is wired yet, and a refused `wire draw`
//! answers in the refusal convention — one sentence on standard error and a
//! row of the exit-code table — rather than in this contract.
//!
//! The goldens alone would not be a contract test. A golden refreshed after a
//! key was dropped passes as happily as one refreshed after a fix, so the key
//! set is asserted separately against a literal list.

use kicli::cli::edit::wire::contract::render;
use kicli::edit::mark::PinAddress;
use kicli::edit::wire::{End, Polyline, draw};
use kicli::geometry::{GRID, Iu, Point};
use kicli::model::items::Uuid;
use kicli::model::{Config, Hierarchy, Refdes, SheetPath, Target, WriteOptions};
use kicli::route::cost::{Cost, Tally};
use kicli::route::report::{Added, Adjusted, Adjustment, Crossing, LabelPair, Report, Status};
use kicli_probe::Probe;
use serde_json::Value;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

/// Where this binary writes the drawings it builds.
fn scratch() -> PathBuf {
    Path::new(env!("CARGO_TARGET_TMPDIR")).join("wire-contract")
}

/// Where the golden for one case sits.
fn golden_path(case: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join(format!("wire_contract_{case}.golden"))
}

/// The two forms the committed golden for one case holds.
fn golden(case: &str) -> (String, String) {
    let path = golden_path(case);
    let text =
        std::fs::read_to_string(&path).unwrap_or_else(|_| panic!("{} is readable", path.display()));
    let (before, after) = text
        .split_once("---\n")
        .unwrap_or_else(|| panic!("{} holds both forms", path.display()));
    (before.to_owned(), after.to_owned())
}

/// Compare one rendered report against the golden for its case.
fn matches_golden(case: &str, report: &Report) {
    let rendered = render(report, GRID);
    let printed = format!(
        "{}\n",
        serde_json::to_string_pretty(&rendered.json).expect("the report prints as JSON")
    );
    let (text, json) = golden(case);
    assert_eq!(
        without_generated_identifiers(&rendered.text),
        text,
        "the text form of {case}"
    );
    assert_eq!(
        without_generated_identifiers(&printed),
        json,
        "the JSON form of {case}"
    );
}

/// The identifiers a run generated, replaced by stable placeholders.
///
/// **A golden cannot assert which identifiers a run produced.**
/// `edit::wire::draw` derives each one by hashing a seed that starts with the
/// file's own **absolute path**, so the values are stable inside one checkout
/// and different in the next one. A golden that froze them would assert where
/// the repository is rather than what the contract promises — and it would
/// pass forever in the checkout that wrote it, which is the worst way for a
/// check to be wrong.
///
/// What the contract promises about `added.wires` is a count, an order and a
/// shape: one identifier per segment, in the order the segments were written.
/// All three survive this. Each **distinct** identifier becomes `<id-1>`,
/// `<id-2>` … numbered in first-appearance order, so a missing entry changes
/// the count and a swapped pair changes the order. The shape is asserted
/// separately, on the real values, by
/// [`what_was_added_is_named_by_identifier`] — normalising here would
/// otherwise hide it.
///
/// Nothing that is not identifier-shaped is touched, so every other byte of
/// both forms is still compared verbatim. The eight-character handles the
/// constructed reports carry are shorter than this shape and are left alone.
fn without_generated_identifiers(text: &str) -> String {
    /// The length of the identifiers KiCad writes, `8-4-4-4-12`.
    const WIDTH: usize = 36;

    let mut out = String::with_capacity(text.len());
    let mut seen: Vec<&str> = Vec::new();
    let mut at = 0;
    while at < text.len() {
        if let Some(found) = text
            .get(at..at + WIDTH)
            .filter(|slice| is_identifier(slice))
        {
            let position = seen
                .iter()
                .position(|had| *had == found)
                .unwrap_or_else(|| {
                    seen.push(found);
                    seen.len() - 1
                });
            let _ = write!(out, "<id-{}>", position + 1);
            at += WIDTH;
        } else {
            let next = text[at..]
                .chars()
                .next()
                .expect("at is a character boundary");
            out.push(next);
            at += next.len_utf8();
        }
    }
    out
}

/// Is this text one identifier of the shape KiCad writes?
fn is_identifier(slice: &str) -> bool {
    let mut groups = slice.split('-');
    for width in [8, 4, 4, 4, 12] {
        let Some(group) = groups.next() else {
            return false;
        };
        if group.len() != width || !group.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return false;
        }
    }
    groups.next().is_none()
}

/// A point from two millimetre readings, as a KiCad file writes them.
fn at(x: &str, y: &str) -> Point {
    let read = |text: &str| {
        Iu::from_millimetres_text(text)
            .unwrap_or_else(|| panic!("{text} is a millimetre reading"))
            .0
    };
    Point::new(read(x), read(y))
}

/// A wire drawn between two resistors, and the report the verb answered with.
///
/// This is the one report here that a drawing produced. Every number in the
/// `routed` golden comes from the walk over that drawing.
fn a_drawn_route(name: &str) -> Report {
    let mut probe = Probe::new(name, scratch());
    probe.place("R", "R1", ("50.8", "54.61"), &["1", "2"]);
    probe.place("R", "R2", ("76.2", "54.61"), &["1", "2"]);
    let path = probe.write();

    let mut hierarchy = Hierarchy::load(&path).expect("the drawing loads");
    let sheet: SheetPath = hierarchy.placements[0].path.clone();
    let project = path.parent().expect("the drawing sits in a directory");
    let request = Polyline {
        from: End::Pin(PinAddress::new(Refdes("R1".to_owned()), "1")),
        to: End::Pin(PinAddress::new(Refdes("R2".to_owned()), "1")),
        via: vec![at("50.8", "45.72"), at("76.2", "45.72")],
    };
    let target = Target {
        path: &path,
        project,
        sheet_path: &sheet,
        grid: GRID,
        options: WriteOptions::default(),
    };
    draw(
        &mut hierarchy,
        &request,
        &Config::default().routing,
        &target,
        "after",
    )
    .expect("the wire draws")
    .report
}

/// A proposal, constructed.
///
/// **Nothing in this build produces one.** The label fallback is what proposes
/// a pair of labels instead of a long or blocked wire, and it is not wired yet.
/// This value exists to measure the rendering, and it is not evidence that any
/// drawing has ever produced a proposal.
fn a_constructed_proposal() -> Report {
    let mut report = Report::of(Status::Labels, "U1.14", "U7.3");
    report.labels = Some(LabelPair {
        name: "SPI_SCK".to_owned(),
        at: [at("25.4", "50.8"), at("76.2", "50.8")],
    });
    report.reason = Some("path length 447.04mm is over the threshold 381.00mm".to_owned());
    report.alternatives_considered = 7;
    report
}

/// A route with no way through, constructed.
///
/// **Nothing in this build produces one.** The search reports it, and the verb
/// that drives the search is not wired yet. A refused `wire draw` is a refusal
/// in the command layer's own convention and never reaches this renderer.
fn a_constructed_blockage() -> Report {
    let mut report = Report::of(Status::Blocked, "U1.14", "R7.1");
    report.blocked_by = vec!["U1".to_owned(), "da5aa983".to_owned()];
    report.reason = Some("no route reaches R7.1 without passing through U1".to_owned());
    report.alternatives_considered = 12;
    report
}

/// A request no drawing can hold, constructed.
///
/// **Nothing in this build produces one either**, for the same reason: a
/// `wire draw` given a diagonal refuses in the command layer's convention.
fn a_constructed_refusal() -> Report {
    let mut report = Report::of(Status::Invalid, "U1.14", "R7.1");
    report.reason = Some("the segment from (50.8,50.8) to (76.2,63.5) is diagonal".to_owned());
    report
}

/// A drawn route whose second terminal the router had to move, constructed.
///
/// The drawn route above moves nothing, because taking the corners a caller
/// gave never has to. Four-way avoidance is what moves a terminal, and it lives
/// in the search. So the moved case is built here, and it gets a golden of its
/// own because a field with only an empty golden has never been seen rendered.
fn a_route_that_moved_a_terminal(name: &str) -> Report {
    let mut report = a_drawn_route(name);
    report.adjusted = vec![Adjusted {
        terminal: report.to.clone(),
        by: at("0", "1.27"),
        why: Adjustment::FourWayJunction,
    }];
    report.reason = Some("R2.1 would have made a fourth wire end meet at one point".to_owned());
    report
}

#[test]
fn a_drawn_route_matches_the_golden() {
    matches_golden("routed", &a_drawn_route("routed-golden"));
}

#[test]
fn a_moved_terminal_matches_the_golden() {
    matches_golden(
        "routed_adjusted",
        &a_route_that_moved_a_terminal("routed-adjusted-golden"),
    );
}

#[test]
fn a_proposal_matches_the_golden() {
    matches_golden("labels", &a_constructed_proposal());
}

#[test]
fn a_blocked_route_matches_the_golden() {
    matches_golden("blocked", &a_constructed_blockage());
}

#[test]
fn an_invalid_request_matches_the_golden() {
    matches_golden("invalid", &a_constructed_refusal());
}

/// Every key of the contract, at the top level of the object.
///
/// The list is written out rather than derived, because a list derived from the
/// renderer would agree with the renderer however wrong the renderer was.
const KEYS: [&str; 15] = [
    "added",
    "adjusted",
    "alternatives_considered",
    "blocked_by",
    "corners",
    "cost",
    "crossings",
    "from",
    "labels",
    "length_mm",
    "path",
    "reason",
    "segments",
    "status",
    "to",
];

/// The parts of the cost, and the total they sum to.
const COST_KEYS: [&str; 6] = ["crossings", "length", "proximity", "text", "total", "turns"];

/// The names of an object's keys, sorted.
fn keys(value: &Value) -> Vec<&str> {
    let mut found: Vec<&str> = value
        .as_object()
        .unwrap_or_else(|| panic!("{value} is an object"))
        .keys()
        .map(String::as_str)
        .collect();
    found.sort_unstable();
    found
}

#[test]
fn every_status_answers_with_the_same_key_set() {
    // One shape, four statuses. A caller that had to ask which keys were there
    // before reading them would be parsing rather than reading a contract.
    for report in [
        a_drawn_route("routed-keys"),
        a_constructed_proposal(),
        a_constructed_blockage(),
        a_constructed_refusal(),
    ] {
        let json = render(&report, GRID).json;
        assert_eq!(
            keys(&json),
            KEYS,
            "the key set of {}",
            report.status.token()
        );
        assert_eq!(
            keys(&json["cost"]),
            COST_KEYS,
            "the cost of {}",
            report.status.token()
        );
        assert_eq!(
            keys(&json["added"]),
            ["junctions", "wires"],
            "what {} added",
            report.status.token()
        );
    }
}

#[test]
fn the_lists_carry_the_keys_their_entries_are_read_by() {
    // The three nested shapes, each on a report that holds one. An empty list
    // proves nothing about the entries it would have held.
    let mut report = a_drawn_route("routed-lists");
    report.crossings = vec![Crossing {
        at: at("152.4", "88.9"),
        wire: "da5aa983".to_owned(),
        net: Some("GND".to_owned()),
    }];
    report.adjusted = vec![Adjusted {
        terminal: "R7.1".to_owned(),
        by: at("0", "1.27"),
        why: Adjustment::FourWayJunction,
    }];
    let json = render(&report, GRID).json;
    assert_eq!(keys(&json["crossings"][0]), ["at", "net", "wire"]);
    assert_eq!(keys(&json["adjusted"][0]), ["by", "terminal", "why"]);

    let proposal = render(&a_constructed_proposal(), GRID).json;
    assert_eq!(keys(&proposal["labels"]), ["at", "name"]);
}

#[test]
fn a_displacement_is_a_displacement_and_not_a_position() {
    // The contract stores what a terminal moved BY. Where it ended up is the
    // path's own end, and the point the caller asked for is that end less the
    // displacement. A renderer that printed the position instead would pass a
    // golden and mislead every caller.
    let mut report = a_drawn_route("routed-displacement");
    let terminus = *report.path.last().expect("the route has an end");
    report.adjusted = vec![Adjusted {
        terminal: report.to.clone(),
        by: at("0", "1.27"),
        why: Adjustment::FourWayJunction,
    }];
    let json = render(&report, GRID).json;
    assert_eq!(
        json["adjusted"][0]["by"],
        serde_json::json!([0.0, 1.27]),
        "the displacement, not {terminus}"
    );
    assert_ne!(
        json["adjusted"][0]["by"],
        json["path"][report.path.len() - 1],
        "and it is not the terminus in disguise"
    );
}

#[test]
fn what_was_added_is_named_by_identifier() {
    // A proposal added nothing, and a drawn route names every record it wrote,
    // because those are the handles a caller addresses next.
    let proposal = render(&a_constructed_proposal(), GRID).json;
    assert_eq!(proposal["added"]["wires"], serde_json::json!([]));

    let drawn = a_drawn_route("routed-added");
    let json = render(&drawn, GRID).json;
    let written = json["added"]["wires"]
        .as_array()
        .expect("a list of identifiers");
    assert_eq!(
        written.len(),
        drawn.segments(),
        "one record per segment, which is what a KiCad wire is"
    );
    // The golden compares these under placeholders, because their value is a
    // hash of the file's own path. **The shape is asserted here, on the real
    // values**, so normalising in the golden hides nothing.
    for uuid in written {
        assert!(
            uuid.as_str().is_some_and(is_identifier),
            "{uuid} is an identifier of the shape KiCad writes"
        );
    }
    let distinct: std::collections::BTreeSet<&str> =
        written.iter().filter_map(Value::as_str).collect();
    assert_eq!(
        distinct.len(),
        written.len(),
        "no two segments of one wire share an identifier"
    );
}

#[test]
fn a_report_that_wrote_nothing_prints_no_added_line() {
    let text = render(&a_constructed_proposal(), GRID).text;
    assert!(
        !text.contains("wires added"),
        "a proposal proposes; it does not write: {text}"
    );
    let drawn = render(&a_drawn_route("routed-no-added"), GRID).text;
    assert!(
        drawn.contains("wires added: "),
        "and a route that wrote says what it wrote: {drawn}"
    );
}

#[test]
fn the_status_word_starts_the_first_line_of_every_form() {
    // An agent greps for it, and a person reads it first. Both need it in the
    // same place whatever came back.
    for report in [
        a_drawn_route("routed-status-word"),
        a_constructed_proposal(),
        a_constructed_blockage(),
        a_constructed_refusal(),
    ] {
        let rendered = render(&report, GRID);
        let first = rendered.text.lines().next().expect("there is a first line");
        assert!(
            first.starts_with(report.status.token()),
            "{first:?} starts with {:?}",
            report.status.token()
        );
        assert_eq!(rendered.json["status"], report.status.token());
    }
}

#[test]
fn an_empty_report_still_names_both_ends() {
    // The degenerate case: a status and two names, and nothing else to say.
    let report = Report::of(Status::Blocked, "U1.14", "R7.1");
    let rendered = render(&report, GRID);
    assert_eq!(rendered.text, "blocked U1.14 -> R7.1\n");
    assert_eq!(rendered.json["from"], "U1.14");
    assert_eq!(rendered.json["to"], "R7.1");
    assert_eq!(rendered.json["length_mm"], 0.0);
}

#[test]
fn a_route_reports_its_cost_in_parts_and_the_total_is_their_sum() {
    // The point of the whole exercise: an agent reads the parts to decide
    // whether to move a symbol instead of accepting the route.
    let mut report = Report::of(Status::Routed, "U1.14", "R7.1");
    report.path = vec![at("139.7", "88.9"), at("152.4", "88.9")];
    report.tally = Tally {
        steps: 30,
        corners: 2,
        crossings: 1,
        ..Tally::default()
    };
    report.cost = Cost::of(report.tally, &Config::default().routing);
    report.added = Added {
        wires: vec![Uuid("00000000-0000-4000-8000-000000000001".to_owned())],
        junctions: Vec::new(),
    };
    let rendered = render(&report, GRID);
    assert!(
        rendered
            .text
            .contains("  cost 62 = length 30 + turns 12 + crossings 20 + text 0 + proximity 0\n"),
        "{}",
        rendered.text
    );
    assert_eq!(rendered.json["cost"]["total"], 62);
}
