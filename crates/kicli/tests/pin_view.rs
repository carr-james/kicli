//! `sch pins` answers well enough to draw a wire without a failed attempt.
//!
//! **The completion criterion of this view is end-to-end and nothing less.**
//! The defect it exists to close (`tasks/dogfood.md`, run 1, defects 2 and 6)
//! was not that kicli computed a pin's position wrongly — `pin_positions.rs`
//! already gates that against KiCad's own rule check, 48 rows, no tolerance.
//! It was that an agent could not *ask*, so it guessed, and learned its guess
//! was wrong from a write command's refusal. A check that compared this view's
//! numbers against a fixture would restate what `pin_positions.rs` already
//! measures and would not test that at all.
//!
//! **What each side of the end-to-end check derives from**, because a check
//! whose two sides share an ancestor passes on a surface that is uniformly
//! wrong:
//!
//! - The **asking** side is the compiled binary's own standard output, parsed
//!   back out of the text exactly as an agent would parse it. Nothing of the
//!   library is called; the coordinate crosses a process boundary as characters.
//! - The **acting** side is a second run of the binary, `wire draw`, which
//!   resolves the pin through `edit::wire` and decides acceptance in
//!   `Tally::of_path` and `escapes_are_honoured`. **The view calls neither.**
//!   So the property under test — *the point the view printed is accepted* — is
//!   decided by code the view never runs.
//! - The two do share `geometry::pins::resolve_pins` for where the pin is. That
//!   shared ancestor is why the control below exists: the same command, with
//!   the printed token perturbed off the grid, must be refused. A check that
//!   passes for any token would pass on a view that printed nonsense.

use kicli_probe::scratch::Fixtures;
use kicli_probe::{Placed, Probe, millimetres};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// A path no `kicad-cli` is at, so a machine with KiCad gives the same answer
/// as one without.
const NO_KICAD_CLI: &str = "/nonexistent/kicad-cli";

/// The placement grid, in millimetres.
const GRID_MM: f64 = 1.27;

fn fixtures() -> Fixtures {
    Fixtures::new(env!("CARGO_TARGET_TMPDIR"), env!("CARGO_MANIFEST_DIR"))
}

fn scratch() -> PathBuf {
    Path::new(env!("CARGO_TARGET_TMPDIR")).join("pin-view")
}

/// Run the compiled binary, as a caller does.
fn kicli(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_kicli"))
        .args(args)
        .env("KICLI_KICAD_CLI", NO_KICAD_CLI)
        .output()
        .expect("the binary runs")
}

fn code(run: &Output) -> i32 {
    run.status.code().expect("the run ended by itself")
}

fn stdout(run: &Output) -> String {
    String::from_utf8(run.stdout.clone()).expect("stdout is text")
}

fn stderr(run: &Output) -> String {
    String::from_utf8(run.stderr.clone()).expect("stderr is text")
}

/// One `P` record of a printed answer, split into its words.
///
/// This is the parse an agent writes: filter on the record letter, split on
/// whitespace. Nothing of kicli is called.
fn record(printed: &str, number: &str) -> Vec<String> {
    printed
        .lines()
        .filter(|line| line.starts_with("P "))
        .map(|line| {
            line.split_whitespace()
                .map(ToOwned::to_owned)
                .collect::<Vec<String>>()
        })
        .find(|words| words.get(1).is_some_and(|had| had == number))
        .unwrap_or_else(|| panic!("pin {number} is in the answer:\n{printed}"))
}

/// The escape token of a record: the sixth word, `x,y` in millimetres.
fn escape_of(words: &[String]) -> String {
    words.get(6).expect("the record carries an escape").clone()
}

/// The connection point of a record: the fourth word.
fn at_of(words: &[String]) -> String {
    words.get(4).expect("the record carries a position").clone()
}

/// A point one grid step from the origin, written the way KiCad writes one.
fn steps(from: f64, count: f64) -> String {
    millimetres(from + count * GRID_MM)
}

#[test]
fn the_printed_escape_point_is_accepted_by_wire_draw_first_time() {
    let project = fixtures().scratch_directory("pin_view_end_to_end", "sch/nets");
    let path = project.to_str().expect("the path is text");

    // Ask. R20.2 joins nothing, so it is the pin an agent reaches for.
    let asked = kicli(&["sch", "pins", "R20.2", "-p", path, "--quiet"]);
    assert_eq!(code(&asked), 0, "{}", stderr(&asked));
    let printed = stdout(&asked);
    let words = record(&printed, "2");
    let escape = escape_of(&words);
    let at = at_of(&words);
    assert!(
        words.last().is_some_and(|state| state == "free"),
        "the pin the check uses joins nothing: {printed}"
    );

    // Act, on the characters the first run printed and nothing else.
    let drawn = kicli(&[
        "wire",
        "draw",
        "--from-pin",
        "R20.2",
        "--to-at",
        &escape,
        "-p",
        path,
        "--quiet",
    ]);
    assert_eq!(
        code(&drawn),
        0,
        "the wire is accepted first time: {}",
        stderr(&drawn)
    );

    // And it is the wire that was asked for: the router's own report names the
    // pin's position and the printed escape point as the two ends.
    let report = stdout(&drawn);
    assert!(
        report.contains(&format!("routed R20.2 -> {escape}")),
        "the route ends where the view said it could: {report}"
    );
    let ends = report
        .lines()
        .find(|line| line.starts_with("+ W "))
        .expect("a wire was added");
    for end in [&at, &escape] {
        let normalised: Vec<&str> = end.split(',').collect();
        assert!(
            ends.contains(normalised[0]) && ends.contains(normalised[1]),
            "the drawn segment runs between the two points the view printed: {ends}"
        );
    }
}

#[test]
fn the_same_command_with_the_escape_point_moved_off_grid_is_refused() {
    // The control for the check above. Both runs are the same command on the
    // same drawing; only the token differs. Without it, that check would pass
    // on a view that printed any on-grid point at all.
    let project = fixtures().scratch_directory("pin_view_control", "sch/nets");
    let path = project.to_str().expect("the path is text");

    let asked = kicli(&["sch", "pins", "R20.2", "-p", path, "--quiet"]);
    let escape = escape_of(&record(&stdout(&asked), "2"));
    let (x, y) = escape.split_once(',').expect("the token is x,y");
    let moved = format!("{x},{}", millimetres(y.parse::<f64>().expect("mm") + 0.5));

    let drawn = kicli(&[
        "wire",
        "draw",
        "--from-pin",
        "R20.2",
        "--to-at",
        &moved,
        "-p",
        path,
        "--quiet",
    ]);
    assert_ne!(
        code(&drawn),
        0,
        "half a millimetre off the printed point is refused: {}",
        stdout(&drawn)
    );
    assert!(
        stderr(&drawn).contains("off the grid"),
        "and the refusal says why: {}",
        stderr(&drawn)
    );
}

/// Two resistors nose to nose, close enough that one pin's escape point lands
/// on the other placement.
///
/// Anchors are whole multiples of the grid, because the lattice is exact: a
/// terminal off the grid names no node and the question would not be asked.
fn nose_to_nose(name: &str) -> PathBuf {
    let mut probe = Probe::new(name, scratch());
    let anchor = 80.0 * GRID_MM;
    probe.place_symbol(&Placed::new(
        "R",
        "R1",
        (&millimetres(anchor), &millimetres(anchor)),
        &["1", "2"],
    ));
    // R1 pin 1 sits three grid steps above the anchor and escapes one further.
    // R2 is placed so that its own pin 2 lands exactly on that escape point.
    probe.place_symbol(&Placed::new(
        "R",
        "R2",
        (&millimetres(anchor), &steps(anchor, -7.0)),
        &["1", "2"],
    ));
    probe.write()
}

#[test]
fn a_pin_whose_escape_is_barred_says_what_bars_it_and_the_router_agrees() {
    let root = nose_to_nose("pin_view_barred");
    let path = root.parent().expect("the probe has a directory");
    let path = path.to_str().expect("the path is text");

    let asked = kicli(&["sch", "pins", "R1", "-p", path, "--quiet"]);
    assert_eq!(code(&asked), 0, "{}", stderr(&asked));
    let printed = stdout(&asked);
    let barred = record(&printed, "1");
    // Two things cover that cell — R2's body box and R2's own pin 2 — and
    // `Obstacles::entering` names the smallest handle of the ones that refuse,
    // so that one drawing gives one answer whatever order the file lists its
    // items in. The handle is read out of the record rather than written down
    // here, so the two sides of the comparison below cannot drift apart.
    let handle = barred
        .iter()
        .find_map(|word| word.strip_prefix("blocked="))
        .unwrap_or_else(|| panic!("the record names what is in the way:\n{printed}"))
        .to_owned();
    assert_eq!(handle, "R2", "the placement in the way: {printed}");

    // The control: the other pin of the same symbol is not barred, so the
    // check is measuring the drawing rather than reporting everything barred.
    let clear = record(&printed, "2");
    assert!(
        !clear.iter().any(|word| word.starts_with("blocked=")),
        "the pin facing the other way is clear: {printed}"
    );

    // And the router decides the same way about the same pin. This is the
    // whole claim: the view says so before the agent spends a write attempt
    // finding out. A route nothing routes between is not an error — kicli
    // answers with a proposed pair of labels and draws nothing — so what is
    // asserted is that no wire was drawn and that the obstruction named is the
    // one the view named.
    let routed = kicli(&[
        "wire",
        "connect",
        "--from-pin",
        "R1.1",
        "--to-pin",
        "R2.1",
        "-p",
        path,
        "--quiet",
    ]);
    let answer = stdout(&routed);
    assert!(
        answer.starts_with("labels "),
        "no wire was drawn between them: {answer}"
    );
    assert!(
        answer.contains(&format!("blocked by: {handle}")),
        "and the router names the obstruction the view named: {answer}"
    );
}

#[test]
fn an_off_grid_pin_is_named_off_grid_and_no_wire_may_start_there() {
    let mut probe = Probe::new("pin_view_off_grid", scratch());
    let anchor = 80.0 * GRID_MM;
    // Half a millimetre off the lattice, which is a drawing fault rather than
    // an arithmetic one: kicli refuses to move somebody's pin.
    probe.place_symbol(&Placed::new(
        "R",
        "R1",
        (&millimetres(anchor), &millimetres(anchor + 0.5)),
        &["1", "2"],
    ));
    let root = probe.write();
    let path = root.parent().expect("the probe has a directory");
    let path = path.to_str().expect("the path is text");

    let printed = stdout(&kicli(&["sch", "pins", "R1", "-p", path, "--quiet"]));
    let words = record(&printed, "1");
    assert!(
        words.iter().any(|word| word == "off-grid"),
        "the record says the pin is off the lattice: {printed}"
    );
    assert!(
        !words.iter().any(|word| word.starts_with("blocked=")),
        "and does not blame the page border for it: {printed}"
    );

    let escape = escape_of(&words);
    let drawn = kicli(&[
        "wire",
        "draw",
        "--from-pin",
        "R1.1",
        "--to-at",
        &escape,
        "-p",
        path,
        "--quiet",
    ]);
    assert_ne!(
        code(&drawn),
        0,
        "no wire starts at an off-grid pin: {}",
        stdout(&drawn)
    );
}

#[test]
fn a_pin_three_wire_ends_already_meet_at_is_called_crowded() {
    let mut probe = Probe::new("pin_view_crowded", scratch());
    let anchor = 80.0 * GRID_MM;
    probe.place_symbol(&Placed::new(
        "R",
        "R1",
        (&millimetres(anchor), &millimetres(anchor)),
        &["1", "2"],
    ));
    // R1 pin 1 is three grid steps above the anchor. Three wires end on it, so
    // a route's own end would be the fourth, which spec/SPEC.md §9 Q2 refuses.
    let pin_y = steps(anchor, -3.0);
    let at = (millimetres(anchor), pin_y);
    for far in [
        (steps(anchor, -2.0), at.1.clone()),
        (steps(anchor, 2.0), at.1.clone()),
        (at.0.clone(), steps(anchor, -5.0)),
    ] {
        probe.wire((&at.0, &at.1), (&far.0, &far.1));
    }
    let root = probe.write();
    let path = root.parent().expect("the probe has a directory");
    let path = path.to_str().expect("the path is text");

    let printed = stdout(&kicli(&["sch", "pins", "R1", "-p", path, "--quiet"]));
    assert!(
        record(&printed, "1").iter().any(|word| word == "crowded"),
        "three wire ends already meet at pin 1: {printed}"
    );
    assert!(
        !record(&printed, "2").iter().any(|word| word == "crowded"),
        "and nothing meets at pin 2, so the check reads the drawing: {printed}"
    );
}

#[test]
fn the_json_form_carries_what_the_terse_form_carries() {
    let project = fixtures().scratch_directory("pin_view_json", "sch/nets");
    let path = project.to_str().expect("the path is text");

    let text = stdout(&kicli(&["sch", "pins", "R20", "-p", path, "--quiet"]));
    let json: serde_json::Value = serde_json::from_str(&stdout(&kicli(&[
        "sch", "pins", "R20", "--output", "json", "-p", path, "--quiet",
    ])))
    .expect("the JSON form parses");

    let listed = json["pins"].as_array().expect("pins is a list");
    assert_eq!(
        listed.len(),
        text.lines().filter(|line| line.starts_with("P ")).count(),
        "the same records: {text}"
    );
    for record_json in listed {
        let number = record_json["number"].as_str().expect("a pin number");
        let words = record(&text, number);
        assert_eq!(record_json["at"], at_of(&words), "the same position");
        assert_eq!(record_json["escape"], escape_of(&words), "the same escape");
        let state: Vec<String> = record_json["state"]
            .as_array()
            .expect("state is a list")
            .iter()
            .map(|word| word.as_str().unwrap_or_default().to_owned())
            .collect();
        assert_eq!(state, words[7..], "the same state words");
    }
    assert_eq!(json["reference"], "R20");
    assert_eq!(json["scope"], "symbol");
}

#[test]
fn a_pin_already_on_a_net_names_it_and_free_lists_only_the_others() {
    // Written because the falsification table found nothing else watching it:
    // with `net` forced to `None` every other check in this file stayed green,
    // and a view that called every pin free would have shipped. The two halves
    // are one check on purpose — the state word and the filter are the same
    // fact read twice, and a break that flips one must not be able to hide in
    // the other.
    let project = fixtures().scratch_directory("pin_view_nets", "sch/nets");
    let path = project.to_str().expect("the path is text");

    let printed = stdout(&kicli(&["sch", "pins", "R20", "-p", path, "--quiet"]));
    let joined = record(&printed, "1");
    assert!(
        joined.iter().any(|word| word == "net=D0"),
        "pin 1 is on the net the connectivity view calls D0: {printed}"
    );
    let clear = record(&printed, "2");
    assert!(
        clear.iter().any(|word| word == "free"),
        "and pin 2 joins nothing, so the check reads the drawing: {printed}"
    );

    // `--free` is the second flood control, so it has to narrow.
    let narrowed = stdout(&kicli(&[
        "sch", "pins", "R20", "--free", "-p", path, "--quiet",
    ]));
    let listed: Vec<&str> = narrowed
        .lines()
        .filter(|line| line.starts_with("P "))
        .collect();
    assert_eq!(listed.len(), 1, "one of the two pins is free: {narrowed}");
    assert!(
        listed[0].starts_with("P 2 "),
        "and it is the one that joins nothing: {narrowed}"
    );
    assert!(
        narrowed.contains("1 of 2 pin(s) listed") && narrowed.contains("--free"),
        "a narrowed answer says what it left out and how to see it: {narrowed}"
    );
}

#[test]
fn a_pin_that_is_not_there_is_refused_with_the_ones_that_are() {
    let project = fixtures().scratch_directory("pin_view_no_such_pin", "sch/nets");
    let path = project.to_str().expect("the path is text");

    let run = kicli(&["sch", "pins", "R20.9", "-p", path, "--quiet"]);
    assert_ne!(code(&run), 0, "{}", stdout(&run));
    let said = stderr(&run);
    assert!(
        said.contains("no pin 9") && said.contains('1') && said.contains('2'),
        "the refusal lists the pins the symbol does have: {said}"
    );
    assert!(
        stdout(&run).is_empty(),
        "and nothing reaches stdout: {}",
        stdout(&run)
    );
}
