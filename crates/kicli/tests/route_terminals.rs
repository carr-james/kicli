//! A route leaves a terminal along the terminal's own direction.
//!
//! The rule is stated against the symbol's **body**, not against kicli's
//! arithmetic: the escape point must land outside the body box, and the step
//! the other way must land on it. A test that only restated the code's own sign
//! would pass just as happily with the sign inverted, and every route would
//! then leave its pins through the symbol.
//!
//! The probe symbol carries `Device:R`'s own numbers — a pin 3.81 mm out with a
//! length of 1.27 mm, whose root lands exactly on the body edge — so the
//! convention under test is the one KiCad's own library draws.

use kicli::geometry::{GRID, Iu, Point, Rect, resolve_pins, symbol_boxes};
use kicli::model::{Hierarchy, definition_of, read_library};
use kicli::route::{Heading, Obstruction, Terminal, terminal::escape};
use kicli_probe::{Placed, Probe, pin, rectangle, symbol};
use std::path::{Path, PathBuf};

/// Where this binary writes the drawings it builds.
fn scratch() -> PathBuf {
    Path::new(env!("CARGO_TARGET_TMPDIR")).join("route-terminals")
}

/// Half the width of the probe symbol's square body, in millimetres.
const BODY: &str = "2.54";

/// How far a pin's connection point sits from the body's centre.
const REACH: &str = "3.81";

/// A symbol with a square body and one pin on each of its four edges.
///
/// Each pin's angle points from its connection point towards the body, which
/// is the convention `Device:R` draws: pin 1 at `(0, 3.81)` angle 270, length
/// 1.27, root on the body edge at 2.54.
fn quad() -> String {
    symbol(
        "QUAD",
        "U",
        false,
        &[(
            "1_1",
            vec![
                rectangle(("-2.54", "-2.54"), (BODY, BODY)),
                // On the left edge, pointing right, into the body.
                pin("passive", ("-3.81", "0"), "0", "1", "W"),
                // On the bottom edge of the library drawing, pointing up.
                pin("passive", ("0", "-3.81"), "90", "2", "S"),
                // On the right edge, pointing left.
                pin("passive", (REACH, "0"), "180", "3", "E"),
                // On the top edge, pointing down.
                pin("passive", ("0", REACH), "270", "4", "N"),
            ],
        )],
    )
}

/// Every pin of one placement, with the body box it belongs to.
fn placement(name: &'static str, mirror: Option<&str>) -> (Rect, Vec<(String, Terminal)>) {
    let mut probe = Probe::new(name, scratch());
    probe.define(quad());
    let mut placed = Placed::new("QUAD", "U1", ("101.6", "101.6"), &["1", "2", "3", "4"]);
    placed.mirror = mirror;
    probe.place_symbol(&placed);

    let path = probe.write();
    let hierarchy = Hierarchy::load(&path).expect("the probe loads");
    let file = hierarchy.files.first().expect("the probe has one sheet");
    let schematic = &file.schematic;
    let library = read_library(&file.doc, &schematic.library_symbols, schematic.version);
    let symbol = schematic
        .symbols()
        .next()
        .expect("the probe places a symbol");
    let definition = definition_of(&library, symbol).expect("the definition is embedded");

    let boxes = symbol_boxes(&file.doc, symbol, definition);
    let terminals = resolve_pins(symbol, definition)
        .iter()
        .map(|pin| (pin.name.clone(), Terminal::of_pin("U1", pin)))
        .collect();
    (boxes.body, terminals)
}

#[test]
fn a_route_leaves_a_pin_along_its_own_direction() {
    for mirror in [None, Some("x"), Some("y")] {
        let name = match mirror {
            None => "upright",
            Some("x") => "mirrored-x",
            _ => "mirrored-y",
        };
        let (body, terminals) = placement(
            match mirror {
                None => "escape-upright",
                Some("x") => "escape-mirrored-x",
                _ => "escape-mirrored-y",
            },
            mirror,
        );
        assert_eq!(terminals.len(), 4, "{name}: four pins");

        for (pin, terminal) in &terminals {
            let escape = terminal.escape_point(GRID);
            let heading = terminal.escape.expect("a pin fixes a direction");

            // One grid step, and no more.
            let moved = (escape.x.0 - terminal.at.x.0).abs() + (escape.y.0 - terminal.at.y.0).abs();
            assert_eq!(moved, GRID.0, "{name} {pin}: one grid step");

            // Away from the body, which is what makes the sign right.
            assert!(
                !body.contains(escape),
                "{name} {pin}: the escape point {escape} is inside the body {body:?}"
            );
            // And the step the other way lands on the body, so the pin really
            // does point at the symbol it belongs to.
            let inwards = heading.reversed().step(terminal.at, GRID);
            assert!(
                body.contains(inwards),
                "{name} {pin}: the pin root {inwards} is off the body {body:?}"
            );
        }
    }
}

#[test]
fn every_pin_of_a_placement_escapes_its_own_way() {
    // The four pins face four ways, so a rule that answered the same heading
    // for all of them would pass the check above on a symmetrical body.
    let (_, terminals) = placement("escape-headings", None);
    let mut headings: Vec<Heading> = terminals
        .iter()
        .map(|(_, terminal)| terminal.escape.expect("a pin fixes a direction"))
        .collect();
    headings.sort();
    headings.dedup();
    assert_eq!(headings.len(), 4, "four pins, four headings");
}

#[test]
fn a_point_on_a_net_may_be_met_from_any_side() {
    let terminal = Terminal::of_point(Point::new(101_600, 101_600), "#n3");
    assert_eq!(terminal.escape, None);
    assert_eq!(terminal.escape_point(GRID), terminal.at);
    // With no direction to be blocked in, nothing can block the escape.
    let everywhere = Obstruction {
        handle: "R1".to_owned(),
        area: Rect::new(Point::new(0, 0), Point::new(1_000_000, 1_000_000)),
    };
    assert!(escape(&terminal, GRID, &[everywhere]).is_ok());
}

#[test]
fn a_blocked_escape_is_reported_and_not_routed_around() {
    let (_, terminals) = placement("escape-blocked", None);
    let (pin, terminal) = &terminals[0];
    let at = terminal.escape_point(GRID);

    // A body sitting exactly where the route must first step.
    let blocker = Obstruction {
        handle: "R99".to_owned(),
        area: Rect::new(
            Point::new(at.x.0 - GRID.0, at.y.0 - GRID.0),
            Point::new(at.x.0 + GRID.0, at.y.0 + GRID.0),
        ),
    };
    let refused = escape(terminal, GRID, std::slice::from_ref(&blocker))
        .expect_err("the escape point is covered");
    assert_eq!(refused.handle, "R99", "the report names what blocked it");
    assert_eq!(refused.terminal, format!("U1.{}", pin_number(pin)));
    assert_eq!(refused.at, at);
    assert!(refused.to_string().contains("R99"), "{refused}");

    // The control: the same terminal with the obstruction moved one step
    // further out escapes, so the refusal is about the escape point and not
    // about the terminal.
    let clear = Obstruction {
        area: blocker.area.offset(Point::new(
            (at.x.0 - terminal.at.x.0) * 3,
            (at.y.0 - terminal.at.y.0) * 3,
        )),
        ..blocker
    };
    assert!(escape(terminal, GRID, &[clear]).is_ok());
}

#[test]
fn a_sheet_pin_leaves_the_edge_it_sits_on() {
    // A sheet pin's angle names its edge, counting anticlockwise from the
    // right with y upwards, and a wire leaves outwards from that edge. The
    // mapping is KiCad's own parser: 0 right, 90 top, 180 left, 270 bottom.
    // Established from source rather than measured; a probe against the tool
    // confirms it when a route first terminates on one.
    for (angle, heading) in [
        ("0", Heading::PlusX),
        ("90", Heading::MinusY),
        ("180", Heading::MinusX),
        ("270", Heading::PlusY),
    ] {
        let mut probe = Probe::new(&format!("sheet-pin-{angle}"), scratch());
        let child = Probe::child_of(&probe);
        probe.sheet_named(
            "00000000-0000-4000-8000-cccccccccccc",
            "child",
            "IN",
            ("101.6", "101.6"),
            angle,
        );
        let path = probe.write_all(&[&child]);
        let hierarchy = Hierarchy::load(&path).expect("the probe loads");
        let file = hierarchy.files.first().expect("the probe has a root sheet");
        let sheet = file
            .schematic
            .sheets()
            .next()
            .expect("the probe draws a sheet");
        let pin = sheet.pins.first().expect("the sheet has a port");

        let terminal = Terminal::of_sheet_pin(pin);
        assert_eq!(terminal.escape, Some(heading), "angle {angle}");
        assert_eq!(terminal.name, "IN");
        assert_eq!(terminal.at, Point::new(1_016_000, 1_016_000));
    }
}

/// The pin number of the pin whose name a placement reports.
fn pin_number(name: &str) -> &str {
    match name {
        "W" => "1",
        "S" => "2",
        "E" => "3",
        _ => "4",
    }
}

#[test]
fn a_terminal_off_the_grid_is_seen_before_it_is_routed() {
    let on = Terminal::of_point(Point::new(2 * GRID.0, 3 * GRID.0), "#n1");
    assert!(on.is_on_grid(GRID));
    let off = Terminal::of_point(Point::new(2 * GRID.0 + 1, 3 * GRID.0), "#n2");
    assert!(!off.is_on_grid(GRID));
    // A grid of zero divides nothing, and must not panic.
    assert!(!on.is_on_grid(Iu(0)));
}
