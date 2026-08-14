//! The builder refuses a drawing it should not write, and writes one drawing.
//!
//! Both properties are about the instrument rather than about KiCad. A probe
//! that writes a coordinate KiCad would not write measures its own defect, and
//! a probe that writes differently on every run cannot be compared with itself.

use kicli_probe::Probe;
use std::path::{Path, PathBuf};

/// Where these tests write. `CARGO_TARGET_TMPDIR` is under `target/`.
fn scratch() -> PathBuf {
    Path::new(env!("CARGO_TARGET_TMPDIR")).join("drawing")
}

/// A drawing with one wire, whose far end the caller chooses.
fn with_wire_end(name: &'static str, x: &str) -> Probe {
    let mut probe = Probe::new(name, scratch());
    probe.place("R", "R1", ("50.8", "29.21"), &["1", "2"]);
    probe.wire(("50.8", "25.4"), (x, "25.4"));
    probe
}

#[test]
#[should_panic(expected = "not a number KiCad writes")]
fn a_number_kicad_would_not_write_is_refused() {
    // Five decimals. KiCad's own files carry four at most, and kicli's reader
    // rejects the value rather than rounding it.
    with_wire_end("refused-precision", "76.19999").write();
}

#[test]
fn a_number_kicad_writes_is_accepted() {
    // The control for the refusal above: the same drawing, one digit shorter.
    // Without it, a panic for any other reason would read as the refusal.
    let path = with_wire_end("accepted-precision", "76.1999").write();
    let text = std::fs::read_to_string(path).expect("the probe file reads");
    assert!(
        text.contains("76.1999"),
        "the wire reached the file: {text}"
    );
}

#[test]
fn two_runs_write_one_drawing() {
    // Identifiers come from a counter rather than a random source, so a probe
    // is comparable with itself and a test is repeatable.
    let first = with_wire_end("repeatable", "76.2").text();
    let second = with_wire_end("repeatable", "76.2").text();
    assert_eq!(first, second);
}
