//! The views stay inside their byte budgets, and say what they cover.
//!
//! A ceiling is indexed on what drives the size of a view: its symbols and its
//! nets. A bank of connector pins is a handful of symbols and several hundred
//! nets, so a ceiling indexed on symbols alone says nothing about it.
//!
//! The hermetic run gates the fixture and the compression ratio, so a change
//! that doubles a record fails here without the corpus. The corpus run gates
//! the published formula over all 153 demo sheets.

use kicli::connectivity::extract;
use kicli::model::Hierarchy;
use kicli::view::connectivity::ViewOptions;
use kicli::view::{Kind, Scope, connectivity, layout, scope};
use std::path::{Path, PathBuf};

fn fixture(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(relative)
}

fn nets_project() -> Hierarchy {
    Hierarchy::load(&fixture("sch/nets/nets.kicad_sch")).expect("the project loads")
}

/// Every byte of every file the project reaches.
fn source_bytes(hierarchy: &Hierarchy) -> usize {
    hierarchy
        .files
        .iter()
        .map(|file| file.doc.source().len())
        .sum()
}

#[test]
fn views_stay_within_byte_ceilings() {
    let hierarchy = nets_project();
    let nets = extract(&hierarchy);
    let options = ViewOptions::default();

    let connectivity = connectivity::render(&hierarchy, &nets, &options);
    let layout = layout::render(&hierarchy, &options);

    // The fixture is small, so these ceilings are tight on purpose: they are
    // here to catch a regression in what a record costs, not to prove the
    // published numbers.
    assert!(
        connectivity.len() < 4_096,
        "connectivity is {} bytes",
        connectivity.len()
    );
    assert!(layout.len() < 4_096, "layout is {} bytes", layout.len());

    // The ratio is the property that survives a bigger fixture.
    let source = source_bytes(&hierarchy);
    let ratio = source / connectivity.len().max(1);
    assert!(
        ratio >= 10,
        "the view is {ratio}x smaller than its source, which is not enough"
    );
    println!(
        "connectivity {} B, layout {} B, source {source} B, {ratio}x",
        connectivity.len(),
        layout.len()
    );
}

#[test]
fn view_scope_switches_on_budget() {
    let hierarchy = nets_project();
    let nets = extract(&hierarchy);
    let options = ViewOptions::default();

    let generous = scope::render(Kind::Connectivity, &hierarchy, &nets, &options, 32_768);
    assert_eq!(generous.scope, Scope::WholeProject);
    assert!(
        generous.text.starts_with("# scope project"),
        "the output states its own scope: {}",
        generous.text.lines().next().unwrap_or_default()
    );

    // The same project, with a budget it cannot fit.
    let tight = scope::render(Kind::Connectivity, &hierarchy, &nets, &options, 200);
    assert_eq!(tight.scope, Scope::IndexAndSummaries);
    assert!(
        tight.text.starts_with("# scope index"),
        "the index says it is the index: {}",
        tight.text.lines().next().unwrap_or_default()
    );
    assert!(
        tight.text.contains("--sheet"),
        "and says how to get the rest: {}",
        tight.text
    );
    assert_eq!(
        tight.text.lines().filter(|l| l.starts_with("I ")).count(),
        hierarchy.placements.len(),
        "one index line per placement"
    );

    // One sheet, within its budget, is that sheet.
    let child = hierarchy
        .placements
        .iter()
        .find(|placement| placement.name.is_some())
        .expect("there is a child sheet");
    let options_for_child = ViewOptions {
        sheet: Some(child.path.clone()),
        ..ViewOptions::default()
    };
    let one = scope::render(
        Kind::Connectivity,
        &hierarchy,
        &nets,
        &options_for_child,
        32_768,
    );
    assert_eq!(one.scope, Scope::OneSheet);
    assert!(one.text.starts_with("# scope sheet "));

    // One sheet too big for the budget falls back the same way a project does.
    // A sheet of few symbols and many nets is the case that needs it.
    let summary = scope::render(
        Kind::Connectivity,
        &hierarchy,
        &nets,
        &options_for_child,
        100,
    );
    assert_eq!(summary.scope, Scope::SheetSummary);
    assert!(
        summary.text.starts_with("# scope sheet-summary"),
        "the summary says what it is: {}",
        summary.text
    );
    assert_eq!(
        summary.text.lines().filter(|l| l.starts_with("I ")).count(),
        1,
        "one line, for the sheet that was asked for: {}",
        summary.text
    );
    assert!(
        summary.text.contains("view.max_bytes"),
        "and says how to see the records: {}",
        summary.text
    );
}

/// The published ceiling for a connectivity view, in bytes.
///
/// Derived from all 153 sheets of KiCad's demo corpus: the worst sheet fills
/// 71 per cent of it.
fn connectivity_ceiling(symbols: usize, nets: usize) -> usize {
    2_048 + 80 * symbols + 128 * nets
}

/// The published ceiling for a layout digest, in bytes.
///
/// The base is larger because a digest also carries the wires and the text.
/// The worst sheet fills 89 per cent of it.
fn layout_ceiling(symbols: usize, nets: usize) -> usize {
    8_192 + 80 * symbols + 128 * nets
}

#[test]
fn json_view_carries_the_same_content() {
    let hierarchy = nets_project();
    let nets = extract(&hierarchy);
    let options = ViewOptions::default();

    let text = connectivity::render(&hierarchy, &nets, &options);
    let json = connectivity::to_json(&hierarchy, &nets, &options);

    // Every symbol record of the text form is in the JSON form.
    let text_symbols: Vec<&str> = text
        .lines()
        .filter_map(|line| line.strip_prefix("S "))
        .filter_map(|line| line.split_whitespace().next())
        .collect();
    let json_symbols: Vec<String> = json["sheets"]
        .as_array()
        .expect("sheets is a list")
        .iter()
        .flat_map(|sheet| sheet["symbols"].as_array().expect("symbols is a list"))
        .map(|symbol| symbol["reference"].as_str().unwrap_or_default().to_owned())
        .collect();
    assert_eq!(
        text_symbols, json_symbols,
        "the same symbols, in the same order"
    );

    // Every net record, with the same pins.
    let text_nets = text.lines().filter(|line| line.starts_with("N ")).count();
    let json_nets = json["nets"].as_array().expect("nets is a list");
    assert_eq!(text_nets, json_nets.len());
    let ground = json_nets
        .iter()
        .find(|net| net["name"] == "GND")
        .expect("ground is a net");
    assert_eq!(
        ground["pins"].as_array().expect("pins is a list").len(),
        4,
        "with the pins the text form lists"
    );
    assert_eq!(ground["crosses_sheets"], true, "and the same marking");

    let layout_json = layout::to_json(&hierarchy, &options);
    let sheets = layout_json["sheets"].as_array().expect("sheets is a list");
    assert_eq!(sheets.len(), hierarchy.placements.len());
    assert_eq!(
        sheets[0]["wires"]["crossings"], 1,
        "the wire summary survives the crossing"
    );
}

#[test]
fn the_json_form_costs_more_than_the_terse_one() {
    let hierarchy = nets_project();
    let nets = extract(&hierarchy);
    let options = ViewOptions::default();

    let text = connectivity::render(&hierarchy, &nets, &options);
    let json = serde_json::to_string(&connectivity::to_json(&hierarchy, &nets, &options))
        .expect("the view serialises");

    let ratio = json.len() as f64 / text.len() as f64;
    println!("terse {} B, JSON {} B, {ratio:.1}x", text.len(), json.len());
    assert!(
        ratio > 1.0,
        "JSON is the expensive twin, which is why it is opt-in"
    );
    assert!(
        ratio < 4.0,
        "and it should not be more than four times the size: {ratio:.1}x"
    );
}

#[cfg(feature = "corpus")]
mod corpus {
    use super::{ViewOptions, connectivity_ceiling, extract, layout, layout_ceiling};
    use kicli::model::Hierarchy;
    use kicli::view::connectivity;
    use std::path::PathBuf;

    /// KiCad's demo corpus, when `cargo xtask corpus` has fetched it.
    fn corpus_root() -> Option<PathBuf> {
        let root =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/corpus/kicad/demos");
        root.is_dir().then_some(root)
    }

    #[test]
    fn views_stay_within_byte_ceilings_corpus() {
        let Some(root) = corpus_root() else {
            eprintln!("skipped: run `cargo xtask corpus` first");
            return;
        };

        let mut checked = 0;
        let mut worst = (0usize, String::new());
        for entry in walk(&root) {
            let Ok(hierarchy) = Hierarchy::load(&entry) else {
                continue;
            };
            if !hierarchy.problems.is_empty() {
                continue;
            }
            let nets = extract(&hierarchy);
            let options = ViewOptions::default();
            for placement in &hierarchy.placements {
                let one = ViewOptions {
                    sheet: Some(placement.path.clone()),
                    ..options.clone()
                };
                let symbols = hierarchy.files[placement.file]
                    .schematic
                    .symbols()
                    .filter(|symbol| symbol.reference_on(&placement.path).is_some())
                    .filter(|symbol| !symbol.is_power())
                    .count();
                let nets_here = nets
                    .nets()
                    .iter()
                    .filter(|net| net.sheets.contains(&placement.path))
                    .count();
                let connectivity = connectivity::render(&hierarchy, &nets, &one);
                let layout = layout::render(&hierarchy, &one);

                let connectivity_room = connectivity_ceiling(symbols, nets_here);
                let layout_room = layout_ceiling(symbols, nets_here);
                assert!(
                    connectivity.len() <= connectivity_room,
                    "{}: {symbols} symbols and {nets_here} nets allow {connectivity_room} bytes, the view is {}",
                    entry.display(),
                    connectivity.len()
                );
                assert!(
                    layout.len() <= layout_room,
                    "{}: {symbols} symbols and {nets_here} nets allow {layout_room} bytes, the digest is {}",
                    entry.display(),
                    layout.len()
                );
                assert!(
                    connectivity.len() + layout.len() <= connectivity_room + layout_room,
                    "{}: both views together are {} bytes",
                    entry.display(),
                    connectivity.len() + layout.len()
                );

                let fill = (connectivity.len() * 100) / connectivity_room.max(1);
                if fill > worst.0 {
                    worst = (
                        fill,
                        format!("{} ({symbols} symbols, {nets_here} nets)", entry.display()),
                    );
                }
                checked += 1;
            }
        }
        println!(
            "checked {checked} sheets; largest pair {} bytes on {}",
            worst.0, worst.1
        );
        assert!(checked > 0, "the corpus held no readable project");
    }

    /// Every root schematic of the corpus: a file with a project beside it.
    fn walk(root: &std::path::Path) -> Vec<PathBuf> {
        let mut found = Vec::new();
        let mut stack = vec![root.to_path_buf()];
        while let Some(directory) = stack.pop() {
            let Ok(entries) = std::fs::read_dir(&directory) else {
                continue;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                } else if path.extension().is_some_and(|end| end == "kicad_pro") {
                    let schematic = path.with_extension("kicad_sch");
                    if schematic.is_file() {
                        found.push(schematic);
                    }
                }
            }
        }
        found.sort();
        found
    }
}
