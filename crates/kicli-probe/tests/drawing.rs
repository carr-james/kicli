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

#[test]
fn a_probe_drawing_yields_distinct_handles_for_each_object() {
    // A probe drawing with multiple objects should have distinct eight-character
    // handles, so that `Uuid::short` can distinguish them in commands addressed
    // by handle. This control asserts that the count of distinct handles equals
    // the count of objects, so a generator that varied nothing cannot pass.
    let mut probe = Probe::new("multi-object", scratch());
    probe.place("R", "R1", ("50.8", "29.21"), &["1", "2"]);
    probe.place("R", "R2", ("76.2", "29.21"), &["1", "2"]);
    probe.place("R", "R3", ("50.8", "50.8"), &["1", "2"]);
    probe.wire(("50.8", "25.4"), ("76.2", "25.4"));
    probe.junction(("50.8", "25.4"));

    let text = probe.text();
    // Extract all UUIDs from the drawing
    let mut handles = Vec::new();
    for line in text.lines() {
        if let Some(start) = line.find("(uuid \"") {
            let uuid_start = start + 7;
            if let Some(end) = line[uuid_start..].find('"') {
                let uuid = &line[uuid_start..uuid_start + end];
                let handle = uuid.chars().take(8).collect::<String>();
                handles.push(handle);
            }
        }
    }

    // We must have at least some objects with UUIDs
    let object_count = handles.len();
    assert!(
        object_count > 0,
        "expected at least one UUID in the drawing"
    );

    // All handles must be distinct. The control that fails if the generator
    // produces no variation: count of distinct handles equals count of objects.
    let unique_count = {
        use std::collections::HashSet;
        let set: HashSet<_> = handles.iter().cloned().collect();
        set.len()
    };
    assert_eq!(
        unique_count, object_count,
        "not all handles are distinct: {object_count} objects, {unique_count} unique handles. \
         Handles: {:?}",
        handles
    );
}

#[test]
fn sibling_probes_of_different_series_have_no_colliding_handles() {
    // Probe::named_child_of creates siblings with different series values. Each
    // starts its own next_uuid counter at zero. If series is not encoded in the
    // leading digits, siblings collide: left's first object and right's first
    // object both get handle 00000001. This control asserts that across a parent
    // and two named children, all handles are distinct: count of distinct handles
    // equals count of objects across all three drawings.
    let mut parent = Probe::new("multi-sheet", scratch());
    parent.place("R", "R1", ("50.8", "29.21"), &["1", "2"]);

    let mut left =
        Probe::named_child_of(&parent, "left", "aaaaaaaa-aaaa-4000-aaaa-aaaaaaaaaaaa", 2);
    left.place("R", "R1", ("50.8", "29.21"), &["1", "2"]);
    left.place("R", "R2", ("76.2", "29.21"), &["1", "2"]);

    let mut right =
        Probe::named_child_of(&parent, "right", "bbbbbbbb-bbbb-4000-bbbb-bbbbbbbbbbbb", 3);
    right.place("R", "R1", ("50.8", "29.21"), &["1", "2"]);
    right.place("R", "R2", ("76.2", "29.21"), &["1", "2"]);

    // Extract handles from all three drawings
    let extract_handles = |text: &str| -> Vec<String> {
        let mut handles = Vec::new();
        for line in text.lines() {
            if let Some(start) = line.find("(uuid \"") {
                let uuid_start = start + 7;
                if let Some(end) = line[uuid_start..].find('"') {
                    let uuid = &line[uuid_start..uuid_start + end];
                    let handle = uuid.chars().take(8).collect::<String>();
                    handles.push(handle);
                }
            }
        }
        handles
    };

    let mut all_handles = Vec::new();
    all_handles.extend(extract_handles(&parent.text()));
    all_handles.extend(extract_handles(&left.text()));
    all_handles.extend(extract_handles(&right.text()));

    let object_count = all_handles.len();
    assert!(
        object_count > 0,
        "expected at least one UUID across the drawings"
    );

    // All handles must be distinct across all three drawings
    let unique_count = {
        use std::collections::HashSet;
        let set: HashSet<_> = all_handles.iter().cloned().collect();
        set.len()
    };
    assert_eq!(
        unique_count, object_count,
        "handles collided across sibling probes: {object_count} objects, {unique_count} unique handles. \
         Handles: {:?}",
        all_handles
    );
}
