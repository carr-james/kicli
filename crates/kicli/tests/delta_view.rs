//! The delta names what changed between two states, in a fixed order.
//!
//! The same pair of states must always produce the same bytes, so an agent can
//! compare two runs without reading the whole design again. The tests below
//! build a pair of sheets that differ by one move, one field edit, one added
//! symbol and one removed symbol.

use kicli::model::{Schematic, SheetPath};
use kicli::view::delta::Delta;
use kicli::view::snapshot::Snapshot;
use kicli_sexpr::Doc;

/// The root screen uuid both states share.
const ROOT: &str = "10000000-0000-4000-8000-000000000000";

/// A timestamp the test supplies, so no run reads the clock.
const TAKEN: &str = "2026-01-02T03:04:05Z";

/// One placed resistor, with the two fields KiCad always writes.
fn symbol(uuid: &str, reference: &str, value: &str, x: f64, y: f64) -> String {
    format!(
        "\t(symbol\n\
         \t\t(lib_id \"Test:R\")\n\
         \t\t(at {x:.4} {y:.4} 0)\n\
         \t\t(unit 1)\n\
         \t\t(uuid \"{uuid}\")\n\
         \t\t(property \"Reference\" \"{reference}\"\n\t\t\t(at {rx:.4} {y:.4} 90)\n\t\t)\n\
         \t\t(property \"Value\" \"{value}\"\n\t\t\t(at {x:.4} {vy:.4} 90)\n\t\t)\n\
         \t\t(instances\n\t\t\t(project \"\"\n\t\t\t\t(path \"/{ROOT}\"\n\
         \t\t\t\t\t(reference \"{reference}\")\n\t\t\t\t\t(unit 1)\n\t\t\t\t)\n\t\t\t)\n\t\t)\n\
         \t)\n",
        rx = x + 2.032,
        vy = y + 2.54,
    )
}

/// A sheet holding the symbols it is given.
fn sheet(symbols: &[String]) -> String {
    format!(
        "(kicad_sch\n\t(version 20260306)\n\t(uuid \"{ROOT}\")\n\t(paper \"A4\")\n{}\
         )\n",
        symbols.concat()
    )
}

fn snapshot(name: &str, source: &str) -> Snapshot {
    let doc = Doc::parse(source).expect("the sheet parses");
    let schematic = Schematic::read(&doc).expect("the sheet reads as a schematic");
    let path = SheetPath::root(schematic.uuid.as_ref().expect("the sheet has a uuid"));
    Snapshot::take(name, TAKEN, &path, &doc, &schematic).expect("the snapshot is taken")
}

const R1: &str = "11111111-1111-4111-8111-111111111111";
const R2: &str = "22222222-2222-4222-8222-222222222222";
const R7: &str = "77777777-7777-4777-8777-777777777777";
const R42: &str = "42424242-4242-4242-8242-424242424242";

fn base() -> String {
    sheet(&[
        symbol(R1, "R1", "10k", 50.8, 50.8),
        symbol(R2, "R2", "1k", 63.5, 50.8),
        symbol(R7, "R7", "4k7", 76.2, 50.8),
    ])
}

/// The base sheet with R1 moved, R2 revalued, R7 removed and R42 added.
fn changed() -> String {
    sheet(&[
        symbol(R1, "R1", "10k", 50.8, 63.5),
        symbol(R2, "R2", "2k2", 63.5, 50.8),
        symbol(R42, "R42", "10k", 88.9, 50.8),
    ])
}

#[test]
fn delta_distinguishes_moved_from_edited() {
    let before = snapshot("base", &base());
    let after = snapshot("current", &changed());
    let delta = Delta::between(&before, &after);

    assert_eq!(
        delta.to_string(),
        concat!(
            "delta base -> current\n",
            "~ L R1  moved  (50.80,50.80) -> (50.80,63.50)\n",
            "+ S R42 10k Test:R\n",
            "- S R7 4k7 Test:R\n",
            "~ S R2.Value  \"1k\" -> \"2k2\"\n",
            "= 4 objects unchanged\n",
        )
    );
    assert_eq!(delta.unchanged, 4);
    assert_eq!(delta.lines.len(), 4);
}

#[test]
fn the_same_pair_of_states_gives_the_same_bytes() {
    let before = snapshot("base", &base());
    let after = snapshot("current", &changed());
    assert_eq!(
        Delta::between(&before, &after).to_string(),
        Delta::between(&before, &after).to_string()
    );
}

#[test]
fn a_state_compared_with_itself_reports_only_the_count() {
    let taken = snapshot("base", &base());
    let delta = Delta::between(&taken, &taken);

    assert!(delta.lines.is_empty());
    assert_eq!(
        delta.to_string(),
        "delta base -> base\n= 9 objects unchanged\n"
    );
}

#[test]
fn the_fields_of_an_added_symbol_are_not_reported_twice() {
    let before = snapshot("base", &sheet(&[symbol(R1, "R1", "10k", 50.8, 50.8)]));
    let after = snapshot(
        "current",
        &sheet(&[
            symbol(R1, "R1", "10k", 50.8, 50.8),
            symbol(R42, "R42", "10k", 88.9, 50.8),
        ]),
    );
    let delta = Delta::between(&before, &after);

    assert_eq!(
        delta.to_string(),
        "delta base -> current\n+ S R42 10k Test:R\n= 3 objects unchanged\n"
    );
}

#[test]
fn a_delta_against_a_saved_state_reads_like_one_against_a_design() {
    // The file carries the display column as well as the hashes, so a
    // comparison against a saved state says the same thing as a comparison
    // against the design it came from. A delta that could only say "something
    // changed" would make the implicit snapshot after every mutation useless
    // for the one question it exists to answer.
    let before = Snapshot::parse(&snapshot("base", &base()).render()).expect("the file parses");
    let after = snapshot("current", &changed());

    let from_file = Delta::between(&before, &after).to_string();
    let from_design = Delta::between(&snapshot("base", &base()), &after).to_string();
    assert_eq!(
        from_file, from_design,
        "the file loses nothing a reader needs"
    );
    assert!(
        from_file.contains("- S R7 4k7 Test:R"),
        "a removed object is named, not just its identifier: {from_file}"
    );
}
