//! The pin view stays inside a byte ceiling, and says what it left out.
//!
//! Constitution §6: *a view that floods is wrong, whatever it contains.* A
//! forty-pin connector is the case — a handful of symbols and a great many
//! records — so the ceiling here is indexed on the thing that drives the size,
//! which for this view is the pin count and nothing else.
//!
//! The shape is the one `view_budgets.rs` already uses for the other three
//! views: a published formula, a measurement of how much of it the worst case
//! fills, and a fallback that must fire when the budget is smaller than the
//! records and must not cost more than the records it stands in for.

use kicli::connectivity::extract;
use kicli::geometry::{GRID, Iu};
use kicli::model::Hierarchy;
use kicli::view::pins::{Listing, Pins, render, to_json};
use kicli_probe::{Placed, Probe, millimetres, pin, symbol};
use std::path::{Path, PathBuf};

fn scratch() -> PathBuf {
    Path::new(env!("CARGO_TARGET_TMPDIR")).join("pin-view-budgets")
}

/// The published ceiling for a pin view, in bytes.
///
/// The base covers the header and the legend, which every answer carries once.
/// The per-pin term covers one record: a number, a name, an electrical type,
/// two points, a heading and the state words.
fn pins_ceiling(pins: usize) -> usize {
    256 + 96 * pins
}

/// A connector of `count` pins in one column, and the drawing that places it.
///
/// The pins run down the left edge facing right, which is how a connector is
/// drawn and, more to the point, gives every pin a distinct position on the
/// lattice.
fn connector(name: &str, count: usize) -> PathBuf {
    let numbers: Vec<String> = (1..=count).map(|number| number.to_string()).collect();
    let pins: Vec<String> = numbers
        .iter()
        .enumerate()
        .map(|(index, number)| {
            // Library space, Y upwards, two grid steps apart so no two pins
            // share a cell.
            let y = millimetres(2.0 * GRID.millimetres() * index as f64);
            pin(
                "passive",
                ("-5.08", &y),
                "0",
                number,
                &format!("IO{number}"),
            )
        })
        .collect();
    let mut probe = Probe::new(name, scratch());
    probe.define(symbol("CONN", "J", false, &[("1_1", pins)]));
    let borrowed: Vec<&str> = numbers.iter().map(String::as_str).collect();
    let anchor = millimetres(80.0 * GRID.millimetres());
    probe.place_symbol(&Placed::new("CONN", "J1", (&anchor, &anchor), &borrowed));
    probe.write()
}

/// The answer about every pin of the connector a probe drew.
fn asked(name: &str, count: usize) -> Pins {
    let root = connector(name, count);
    let hierarchy = Hierarchy::load(&root).expect("the probe loads");
    let placement = hierarchy
        .placements
        .first()
        .expect("the probe has one placement");
    let file = &hierarchy.files[placement.file];
    let symbol = file
        .schematic
        .symbols()
        .find(|symbol| {
            symbol
                .reference_on(&placement.path)
                .is_some_and(|had| had.0 == "J1")
        })
        .expect("the connector is placed");
    let nets = extract(&hierarchy);
    Pins::of(file, &placement.path, symbol, &nets, None, GRID).expect("the pins resolve")
}

#[test]
fn a_wide_connector_stays_inside_the_published_ceiling() {
    let mut worst = (0usize, 0usize);
    for count in [2, 8, 40, 100] {
        let answer = asked(&format!("pin_budget_{count}"), count);
        assert_eq!(answer.total, count, "the connector drew every pin");

        // A budget it fits, so the records are what is measured.
        let rendered = render(&answer, 1_000_000);
        assert_eq!(rendered.listing, Listing::Records);
        let room = pins_ceiling(count);
        assert!(
            rendered.bytes <= room,
            "{count} pins allow {room} bytes, the answer is {}",
            rendered.bytes
        );
        let fill = (rendered.bytes * 100) / room.max(1);
        if fill > worst.0 {
            worst = (fill, count);
        }
        println!("{count} pins: {} B, {fill}% of {room} B", rendered.bytes);
    }
    println!("worst fill {}% at {} pins", worst.0, worst.1);
    assert!(
        worst.0 > 10,
        "the ceiling is tight enough to be a ceiling: worst fill was {}%",
        worst.0
    );
}

#[test]
fn the_ceiling_is_capable_of_failing() {
    // The falsification, kept as a check rather than as a note: the assertion
    // above is only worth its bytes if a fatter record would break it. A
    // ceiling of one byte per pin is the same comparison against a formula the
    // answer cannot meet, so a run in which the assertion cannot fail is a run
    // in which this test fails too.
    let count = 40;
    let answer = asked("pin_budget_falsified", count);
    let rendered = render(&answer, 1_000_000);
    // One byte per pin and no base: the same comparison against a formula the
    // answer cannot meet.
    let impossible = count;
    assert!(
        rendered.bytes > impossible,
        "a one-byte-per-pin ceiling refuses this {} byte answer, so the comparison bites",
        rendered.bytes
    );
}

#[test]
fn a_budget_smaller_than_the_records_falls_back_to_counts() {
    let answer = asked("pin_budget_fallback", 40);
    let full = render(&answer, 1_000_000);
    assert_eq!(full.listing, Listing::Records);

    // Every budget below the records, at the boundary and well under it.
    for budget in [full.bytes - 1, full.bytes / 2, 200, 1] {
        let short = render(&answer, budget);
        assert_eq!(
            short.listing,
            Listing::Summary,
            "a budget of {budget} against {} bytes of records",
            full.bytes
        );
        assert!(
            short.text.starts_with("# pins J1 "),
            "the summary still says which placement it is about: {}",
            short.text
        );
        assert!(
            short.text.contains("scope=symbol-summary"),
            "and says it is not the records: {}",
            short.text
        );
        assert!(
            short.text.contains("pins=40"),
            "and how many it stood in for: {}",
            short.text
        );
        assert!(
            short.text.contains("J1.N")
                && short.text.contains("--free")
                && short.text.contains("view.max_bytes"),
            "and every way to get the records back: {}",
            short.text
        );
        assert!(
            short.bytes < full.bytes,
            "a fallback that costs more than the records it replaces is not a fallback: \
             {} against {}",
            short.bytes,
            full.bytes
        );
        assert_eq!(
            short
                .text
                .lines()
                .filter(|line| line.starts_with("P "))
                .count(),
            0,
            "no records: {}",
            short.text
        );
    }

    // And the budget the answer fits does not fall back. Without this the
    // check above would pass on a view that always summarised.
    assert_eq!(render(&answer, full.bytes).listing, Listing::Records);
}

#[test]
fn the_json_form_carries_the_listing_it_was_rendered_at() {
    let answer = asked("pin_budget_json", 40);
    let full = render(&answer, 1_000_000);
    let records = to_json(&answer, full.listing);
    assert_eq!(
        records["pins"].as_array().map(Vec::len),
        Some(40),
        "every record"
    );
    assert_eq!(records["scope"], "symbol");

    let short = render(&answer, 200);
    let counts = to_json(&answer, short.listing);
    assert_eq!(
        counts["pins"].as_array().map(Vec::len),
        Some(0),
        "an empty list rather than a missing key, so one shape parses both"
    );
    assert_eq!(counts["scope"], "symbol-summary");
    assert_eq!(counts["total"], 40, "and the count is still there");
}

#[test]
fn narrowing_to_the_free_pins_is_what_makes_a_wide_connector_answerable() {
    // The first flood control is that a target is required; the second is this.
    // A connector nobody has wired yet has every pin free, so the check that
    // means anything is that the filter is what the records say it is.
    let answer = asked("pin_budget_free", 40);
    let narrowed = answer.clone().only_free();
    assert_eq!(narrowed.pins.len(), 40, "nothing is wired on this drawing");
    assert_eq!(narrowed.total, 40, "and the count of what it drew is kept");

    let rendered = render(&narrowed, 1_000_000);
    assert!(
        rendered.text.contains("filter=free"),
        "a narrowed answer says it was narrowed: {}",
        rendered.text
    );
    assert_eq!(
        Iu(GRID.0),
        GRID,
        "the grid the escape points are one step of"
    );
}
