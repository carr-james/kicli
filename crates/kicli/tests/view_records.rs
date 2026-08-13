//! The two structured views, against committed goldens.
//!
//! A view is an interface: an agent reads it, quotes lines of it back, and acts
//! on what it says. The goldens exist so that a change to the shape of a line
//! is a decision somebody takes, not a thing that happens.

use kicli::connectivity::extract;
use kicli::model::Hierarchy;
use kicli::view::connectivity::ViewOptions;
use kicli::view::{connectivity, layout};
use std::path::{Path, PathBuf};

fn fixture(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(relative)
}

fn golden(name: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join(name);
    std::fs::read_to_string(path).expect("the golden is readable")
}

fn nets_project() -> Hierarchy {
    Hierarchy::load(&fixture("sch/nets/nets.kicad_sch")).expect("the project loads")
}

#[test]
fn connectivity_view_matches_golden() {
    let hierarchy = nets_project();
    let nets = extract(&hierarchy);
    let view = connectivity::render(&hierarchy, &nets, &ViewOptions::default());
    assert_eq!(
        view,
        golden("view_connectivity.golden"),
        "the view is stable"
    );
}

#[test]
fn layout_view_matches_golden() {
    let hierarchy = nets_project();
    let view = layout::render(&hierarchy, &ViewOptions::default());
    assert_eq!(view, golden("view_layout.golden"), "the digest is stable");
}

#[test]
fn a_reference_sorts_by_number_not_by_text() {
    let hierarchy = nets_project();
    let nets = extract(&hierarchy);
    let view = connectivity::render(&hierarchy, &nets, &ViewOptions::default());
    let order: Vec<&str> = view
        .lines()
        .filter_map(|line| line.strip_prefix("S "))
        .filter_map(|line| line.split_whitespace().next())
        .collect();
    let two = order.iter().position(|r| *r == "R2").expect("R2 is listed");
    let ten = order
        .iter()
        .position(|r| *r == "R10")
        .expect("R10 is listed");
    assert!(two < ten, "R2 comes before R10, not after it: {order:?}");
}

#[test]
fn power_symbols_are_left_out_until_they_are_asked_for() {
    let hierarchy = nets_project();
    let nets = extract(&hierarchy);

    let plain = connectivity::render(&hierarchy, &nets, &ViewOptions::default());
    assert!(
        !plain.lines().any(|line| line.starts_with("S #PWR")),
        "a power symbol is a net-name carrier, not a part in the list"
    );
    assert!(
        plain.contains("N GND"),
        "and it appears where it belongs, as the name of a net"
    );

    let with_power = connectivity::render(
        &hierarchy,
        &nets,
        &ViewOptions {
            include_power: true,
            ..ViewOptions::default()
        },
    );
    let added: Vec<&str> = with_power
        .lines()
        .filter(|line| line.starts_with("S #PWR"))
        .collect();
    assert_eq!(added.len(), 4, "the four power symbols are the difference");
}

#[test]
fn identifiers_are_opt_in_and_change_nothing_else() {
    let hierarchy = nets_project();
    let nets = extract(&hierarchy);
    let plain = connectivity::render(&hierarchy, &nets, &ViewOptions::default());
    let with_uuids = connectivity::render(
        &hierarchy,
        &nets,
        &ViewOptions {
            uuids: true,
            ..ViewOptions::default()
        },
    );
    assert!(with_uuids.len() > plain.len(), "identifiers cost bytes");

    // Every symbol record gains a suffix, and nothing else about the line moves.
    for (plain_line, uuid_line) in plain
        .lines()
        .filter(|line| line.starts_with("S "))
        .zip(with_uuids.lines().filter(|line| line.starts_with("S ")))
    {
        let (head, tail) = uuid_line.rsplit_once(" @").expect("a record gains @uuid8");
        assert_eq!(head, plain_line);
        assert_eq!(tail.len(), 8, "eight characters, not the whole identifier");
    }
}

#[test]
fn a_name_that_two_nets_share_is_qualified_by_its_sheet() {
    // The child sheet is placed twice, and each placement has its own net
    // reaching a hierarchical label called IN. Two nets cannot answer to one
    // name in a view whose names are addresses.
    let hierarchy = nets_project();
    let nets = extract(&hierarchy);
    let view = connectivity::render(&hierarchy, &nets, &ViewOptions::default());

    assert!(view.contains("N channel_a/IN"), "{view}");
    assert!(view.contains("N channel_b/IN"), "{view}");

    let names: Vec<&str> = view
        .lines()
        .filter_map(|line| line.strip_prefix("N "))
        .filter_map(|line| line.split(':').next())
        .map(|name| name.trim_end_matches('*'))
        .map(|name| name.split('=').next().unwrap_or(name))
        .collect();
    let mut sorted = names.clone();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(sorted.len(), names.len(), "every name addresses one net");
}

#[test]
fn a_net_drawn_on_more_than_one_sheet_is_marked() {
    let hierarchy = nets_project();
    let nets = extract(&hierarchy);
    let view = connectivity::render(&hierarchy, &nets, &ViewOptions::default());
    let ground = view
        .lines()
        .find(|line| line.starts_with("N GND"))
        .expect("ground is a net");
    assert!(
        ground.starts_with("N GND*"),
        "ground reaches the child sheets, so it is marked: {ground}"
    );
    let local = view
        .lines()
        .find(|line| line.starts_with("N NET_A"))
        .expect("NET_A is a net");
    assert!(
        !local.starts_with("N NET_A*"),
        "NET_A is drawn on one sheet only: {local}"
    );
}

#[test]
fn a_field_is_listed_only_when_it_has_moved_off_its_library_position() {
    // The connectivity fixture places every field at its symbol's anchor. Its
    // library puts the reference beside the body, so every symbol has exactly
    // one moved field, and the fields that sit where the library puts them are
    // not listed at all.
    let hierarchy = nets_project();
    let view = layout::render(&hierarchy, &ViewOptions::default());
    let moved: Vec<&str> = view.lines().filter(|line| line.starts_with("F ")).collect();
    assert!(
        moved.iter().all(|line| line.contains(".Reference ")),
        "only the reference moved: {moved:?}"
    );
    assert_eq!(
        moved.len(),
        view.lines().filter(|line| line.starts_with("L ")).count(),
        "one per listed symbol, and no line for a field left where it belongs"
    );

    // The geometry fixture's library puts every field at the anchor, and so
    // does the placement, so that view lists no moved field at all.
    let plain = Hierarchy::load(&fixture("geometry/orientations.kicad_sch")).expect("loads");
    let plain_view = layout::render(&plain, &ViewOptions::default());
    assert!(
        !plain_view.lines().any(|line| line.starts_with("F ")),
        "nothing is listed when nothing has moved: {plain_view}"
    );
}

#[test]
fn the_wire_summary_counts_what_it_says_it_counts() {
    let hierarchy = nets_project();
    let view = layout::render(&hierarchy, &ViewOptions::default());
    let summary = view
        .lines()
        .find(|line| line.starts_with("W "))
        .expect("the root sheet has a wire summary");
    // The fixture plants two crossings: one without a junction and one with.
    // The junction one is a join at a shared point, so only the bare crossing
    // is counted.
    assert_eq!(
        summary, "W 15 segments, 2 junctions, 1 crossings",
        "the planted crossing is counted and the junctioned one is not"
    );
}

#[test]
fn a_view_of_one_sheet_says_so_and_covers_only_that_sheet() {
    let hierarchy = nets_project();
    let nets = extract(&hierarchy);
    let child = hierarchy
        .placements
        .iter()
        .find(|placement| placement.name.as_deref() == Some("channel_b"))
        .expect("the second placement is there");

    let options = ViewOptions {
        sheet: Some(child.path.clone()),
        ..ViewOptions::default()
    };
    let view = connectivity::render(&hierarchy, &nets, &options);
    assert!(
        view.starts_with(&format!("# scope sheet {}", child.path.0)),
        "the view names its own scope: {view}"
    );
    assert!(view.contains("S R200"), "the placement's own symbols");
    assert!(!view.contains("S R100"), "and not the other placement's");
    assert!(
        view.contains("N GND*"),
        "a net that leaves the sheet is still marked"
    );
}

#[test]
fn two_runs_of_a_view_agree_byte_for_byte() {
    let hierarchy = nets_project();
    let nets = extract(&hierarchy);
    let options = ViewOptions::default();
    assert_eq!(
        connectivity::render(&hierarchy, &nets, &options),
        connectivity::render(&hierarchy, &nets, &options)
    );
    assert_eq!(
        layout::render(&hierarchy, &options),
        layout::render(&hierarchy, &options)
    );
}
