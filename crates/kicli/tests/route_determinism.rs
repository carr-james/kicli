//! One drawing, one pair of terminals, one answer — every time it is asked.
//!
//! There is no single verb that routes yet, so "routing" here is the
//! composition a route request drives: resolve the two terminals, read the
//! sheet into the router's lists, build the window and the obstacle map, ask
//! the shapes, and ask A\*. The answer of both engines is rendered to one
//! string, and that string is what must not move.
//!
//! Two things can make it move. A collection whose iteration order is not
//! fixed reaches a decision, and then two runs in one process differ — that is
//! the hundred-run arm. Or the order the file lists its items in reaches a
//! decision, and then the same drawing saved twice differs — that is the
//! shuffled arm. **KiCad reorders items when it saves**, so the file's order is
//! not a stable input, and the shuffled arm is the load-bearing half.
//!
//! The drawings are probe drawings rather than hand-built lists of rectangles:
//! a hand-made map would encode the same assumptions as the code that reads
//! one. The shuffle is applied to the written file, so every item keeps its own
//! identifier and only its position moves.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use kicli::geometry::{GRID, Point, resolve_pins};
use kicli::model::items::SheetPath;
use kicli::model::{Config, Hierarchy, LoadedFile, definition_of, read_library};
use kicli::route::{
    Candidate, Cost, Obstacles, Route, Routed, Search, Shapes, SheetObjects, Tally, Terminal,
    Window,
};
use kicli_probe::{Probe, pin, rectangle, symbol};
use kicli_sexpr::{Doc, NodeId};
use proptest::prelude::*;
use proptest::test_runner::{RngAlgorithm, RngSeed};

/// How many times one request is asked, in the hundred-run arm.
const RUNS: usize = 100;

/// The seed the shuffled arm runs on.
///
/// A property test whose seed comes from entropy makes a gate that fails on
/// somebody else's machine and passes on the next run, which is the shape of a
/// gate people learn to re-run rather than read. A fixed seed gives one
/// reproducible sample of the permutation space, and the case count is what
/// buys coverage inside it. The trade is deliberate: this gate is reproducible
/// first.
const SEED: u64 = 0x5EED_1F0A_2B3C_4D5E;

/// How many permutations the shuffled arm tries.
///
/// Each case routes every pair of the drawing it picks, so a case is a whole
/// sweep rather than one request. Sixty-four of them cost a few seconds and
/// cover both drawings many times over.
const PERMUTATIONS: u32 = 64;

/// Where this binary writes the drawings it builds.
fn scratch() -> PathBuf {
    Path::new(env!("CARGO_TARGET_TMPDIR")).join("route-determinism")
}

/// The pin numbers the probe symbol draws.
const PINS: [&str; 4] = ["1", "2", "3", "4"];

/// A symbol with a square body and one pin on each of its four edges.
///
/// The same instrument `tests/route_search.rs` measures A\* with: each pin's
/// angle points from its connection point towards the body, so pin 1 faces
/// west, 2 south, 3 east and 4 north.
fn quad() -> String {
    symbol(
        "QUAD",
        "U",
        false,
        &[(
            "1_1",
            vec![
                rectangle(("-2.54", "-2.54"), ("2.54", "2.54")),
                pin("passive", ("-3.81", "0"), "0", "1", "W"),
                pin("passive", ("0", "-3.81"), "90", "2", "S"),
                pin("passive", ("3.81", "0"), "180", "3", "E"),
                pin("passive", ("0", "3.81"), "270", "4", "N"),
            ],
        )],
    )
}

/// One drawing this check routes over, as the text of its file.
struct Drawing {
    name: &'static str,
    text: String,
}

/// The drawings, built through the probe harness.
///
/// Two of them, because they defeat the router in different ways, and each is
/// kept to two symbols because every pair of every terminal is routed a hundred
/// times: a third symbol is a third of a minute on every commit for coverage
/// the marks already give.
///
/// The **walled** drawing is where the three answers come from. A no-connect
/// marker on one pin's escape cell refuses every request from that pin; a
/// column of junctions between the two symbols refuses every silhouette, so A\*
/// runs and finds the way round; every other request the shapes answer.
///
/// The **crowded** drawing is where the shuffle bites. Its two symbols overlap,
/// so the cells they share are refused by both of them and the report has to
/// choose a name — the case where naming whichever object the file listed first
/// would make the answer depend on the file's order. It also holds an item of
/// every kind the shuffle can move.
fn drawings() -> Vec<Drawing> {
    let mut walled = Probe::new("walled", scratch());
    walled.define(quad());
    walled.place("QUAD", "U1", ("76.2", "88.9"), &PINS);
    walled.place("QUAD", "U2", ("101.6", "88.9"), &PINS);
    // On U1.3's escape cell, which is the step that pin must take first.
    walled.no_connect(("81.28", "88.9"));
    // A wall of junctions, with the way round at both ends of it.
    for step in 0..11 {
        let y = kicli_probe::millimetres(82.55 + f64::from(step) * 1.27);
        walled.junction(("88.9", &y));
    }

    let mut crowded = Probe::new("crowded", scratch());
    crowded.define(quad());
    crowded.place("QUAD", "U1", ("88.9", "88.9"), &PINS);
    crowded.place("QUAD", "U2", ("92.71", "88.9"), &PINS);
    // A wire of another net: it blocks along its own axis and is crossed the
    // other way, at a price. The junction and the marker sit on its ends.
    crowded.wire(("81.28", "82.55"), ("81.28", "95.25"));
    crowded.junction(("81.28", "95.25"));
    crowded.no_connect(("81.28", "82.55"));
    // Text and a label, which cost a step rather than blocking one.
    crowded.free_text("keep clear", ("85.09", "82.55"));
    crowded.label_of_kind("label", "input", "SENSE", ("85.09", "95.25"));

    vec![
        Drawing {
            name: "walled",
            text: walled.text(),
        },
        Drawing {
            name: "crowded",
            text: crowded.text(),
        },
    ]
}

/// Write a file text under this binary's scratch directory, and load it.
fn loaded(drawing: &str, file: &str, text: &str) -> Hierarchy {
    let directory = scratch().join(drawing);
    std::fs::create_dir_all(&directory).expect("the scratch directory is writable");
    let path = directory.join(format!("{file}.kicad_sch"));
    std::fs::write(&path, text).expect("the drawing is writable");
    Hierarchy::load(&path).expect("the drawing loads")
}

/// The root placement of a loaded drawing, and the file it draws.
fn root(hierarchy: &Hierarchy) -> (&LoadedFile, &SheetPath) {
    let placement = hierarchy
        .placements
        .first()
        .expect("the root sheet is placed");
    (&hierarchy.files[placement.file], &placement.path)
}

/// Every pin of every placed symbol, as a terminal, ordered by name.
///
/// The order is the name's rather than the file's, so that a shuffled file
/// offers the same pairs in the same order and the comparison is about the
/// route rather than about which pair was asked for.
fn terminals(hierarchy: &Hierarchy) -> Vec<Terminal> {
    let (file, path) = root(hierarchy);
    let schematic = &file.schematic;
    let library = read_library(&file.doc, &schematic.library_symbols, schematic.version);
    let mut found = Vec::new();
    for symbol in schematic.symbols() {
        let Some(reference) = symbol.reference_on(path) else {
            continue;
        };
        let Some(definition) = definition_of(&library, symbol) else {
            continue;
        };
        for resolved in resolve_pins(&symbol.drawn_on(path), definition) {
            found.push(Terminal::of_pin(&reference.0, &resolved));
        }
    }
    found.sort_by(|left, right| left.name.cmp(&right.name));
    found
}

/// Every ordered pair of distinct terminals.
fn pairs(terminals: &[Terminal]) -> Vec<(&Terminal, &Terminal)> {
    let mut all = Vec::new();
    for source in terminals {
        for target in terminals {
            if source.name != target.name {
                all.push((source, target));
            }
        }
    }
    all
}

/// What one route request answers, rendered so that two answers compare byte
/// for byte.
///
/// This is the composition, in full: the sheet is read into the router's lists,
/// the window is built around the two terminals, the map is filled, and both
/// engines are asked. Every number either engine reports is in the string —
/// the winning shape and its vertices, the tally, the five parts of the cost
/// and their total, how many alternatives were looked at, and the handles that
/// refused a step.
fn answer(hierarchy: &Hierarchy, source: &Terminal, target: &Terminal) -> String {
    let (file, path) = root(hierarchy);
    let named = [source.name.clone(), target.name.clone()];
    let objects = SheetObjects::read(
        file,
        path,
        &Routed {
            wires: &[],
            terminals: &named,
        },
    );
    let routing = Config::default().routing;
    let window = Window::around(source.at, target.at, routing.margin, objects.page(), GRID);
    let obstacles = Obstacles::build(window, &objects.geometry());
    let shapes = Shapes::of(source, target, &obstacles, &routing);
    // The fall-through the composition makes: the shapes are the fast path, and
    // A* is asked only when no silhouette fits.
    let search = shapes
        .best()
        .is_none()
        .then(|| Search::of(source, target, &obstacles, &routing));

    let mut out = String::new();
    let write = &mut out;
    writeln!(write, "{} -> {}", source.name, target.name).expect("a string is writable");
    writeln!(
        write,
        "window {} .. {} grid {}",
        window.area().start(),
        window.area().end(),
        window.grid()
    )
    .expect("a string is writable");
    writeln!(
        write,
        "shapes considered {} blocked-by {:?}",
        shapes.considered(),
        shapes.blocked_by()
    )
    .expect("a string is writable");
    match shapes.best() {
        Some(best) => writeln!(write, "best {}", candidate(best)),
        None => writeln!(write, "best none"),
    }
    .expect("a string is writable");
    for feasible in shapes.feasible() {
        writeln!(write, "feasible {}", candidate(feasible)).expect("a string is writable");
    }
    match &search {
        Some(search) => {
            writeln!(
                write,
                "search expanded {} blocked-by {:?}",
                search.expanded(),
                search.blocked_by()
            )
            .expect("a string is writable");
            match search.route() {
                Some(route) => writeln!(write, "route {}", found(route)),
                None => writeln!(write, "route none"),
            }
        }
        None => writeln!(write, "search not asked"),
    }
    .expect("a string is writable");
    out
}

/// One candidate shape, on one line.
fn candidate(candidate: &Candidate) -> String {
    format!(
        "{:?} {} {}",
        candidate.shape,
        walk(&candidate.path),
        charged(candidate.tally, candidate.cost)
    )
}

/// One route A\* found, on one line.
fn found(route: &Route) -> String {
    format!("{} {}", walk(&route.path), charged(route.tally, route.cost))
}

/// The vertices of a path.
fn walk(path: &[Point]) -> String {
    let points: Vec<String> = path.iter().map(ToString::to_string).collect();
    format!("[{}]", points.join(" "))
}

/// What a path meets and what that costs.
fn charged(tally: Tally, cost: Cost) -> String {
    let parts: Vec<String> = cost
        .parts()
        .iter()
        .map(|(name, value)| format!("{name}={value}"))
        .collect();
    format!(
        "steps={} corners={} crossings={} text={} near={} | {} | total={}",
        tally.steps,
        tally.corners,
        tally.crossings,
        tally.text_steps,
        tally.near_steps,
        parts.join(" "),
        cost.total()
    )
}

/// What every answer over one drawing says, in one string.
fn every_answer(hierarchy: &Hierarchy, terminals: &[Terminal]) -> String {
    let mut all = String::new();
    for (source, target) in pairs(terminals) {
        all.push_str(&answer(hierarchy, source, target));
    }
    all
}

/// The heads of the items KiCad reorders when it saves a sheet.
///
/// Everything else a file holds — the version, the generator, the paper, the
/// library, the instance table — is a header rather than an item, and stays
/// where it is.
const ITEM_HEADS: [&str; 9] = [
    "symbol",
    "wire",
    "bus",
    "junction",
    "no_connect",
    "label",
    "global_label",
    "hierarchical_label",
    "text",
];

/// Which children of the root are drawable items.
fn item_slots(doc: &Doc, children: &[NodeId]) -> Vec<usize> {
    children
        .iter()
        .enumerate()
        .filter(|&(_, &child)| {
            doc.head(child)
                .is_some_and(|head| ITEM_HEADS.contains(&head))
        })
        .map(|(index, _)| index)
        .collect()
}

/// How many drawable items a file text holds.
fn item_count(text: &str) -> usize {
    let doc = Doc::parse(text).expect("the drawing parses");
    let root = doc.root().expect("the drawing has a root");
    item_slots(&doc, doc.children(root)).len()
}

/// The same file, with its items in the order given.
///
/// Every item keeps its own record and its own identifier: only the position
/// moves, which is exactly what KiCad's own save does. The header records keep
/// their places, so the result is a file KiCad writes rather than an
/// arrangement no tool produces.
fn reordered(text: &str, order: &[usize]) -> String {
    let mut doc = Doc::parse(text).expect("the drawing parses");
    let root = doc.root().expect("the drawing has a root");
    let children: Vec<NodeId> = doc.children(root).to_vec();
    let slots = item_slots(&doc, &children);
    assert_eq!(order.len(), slots.len(), "the order names every item");

    let mut wanted = children.clone();
    for (&slot, &pick) in slots.iter().zip(order) {
        wanted[slot] = children[slots[pick]];
    }
    for &child in &children {
        assert!(doc.remove(child), "a child of the root is removable");
    }
    for &child in &wanted {
        doc.push_child(root, child);
    }
    doc.emit()
}

/// A permutation of the positions `0..count`.
fn permutation(count: usize) -> impl Strategy<Value = Vec<usize>> {
    Just((0..count).collect::<Vec<usize>>()).prop_shuffle()
}

/// The configuration both proptest arms run under.
fn config() -> ProptestConfig {
    ProptestConfig {
        cases: PERMUTATIONS,
        // A fixed seed, chosen so the gate answers the same on every machine.
        rng_algorithm: RngAlgorithm::ChaCha,
        rng_seed: RngSeed::Fixed(SEED),
        // Shrinking stays on: a permutation that breaks the router shrinks
        // towards the file's own order, which names the item that moved.
        // Persistence is off, because a fixed seed already replays the case
        // and a check that writes a file into the source tree when it fails is
        // a check that edits the repository.
        failure_persistence: None,
        ..ProptestConfig::default()
    }
}

#[test]
fn routing_is_identical_across_a_hundred_runs() {
    let mut by_shapes = 0_usize;
    let mut by_search = 0_usize;
    let mut refused = 0_usize;

    for drawing in drawings() {
        let hierarchy = loaded(drawing.name, "runs", &drawing.text);
        let terminals = terminals(&hierarchy);
        assert!(
            terminals.len() >= 8,
            "{} offers terminals to pair: {}",
            drawing.name,
            terminals.len()
        );

        for (source, target) in pairs(&terminals) {
            let first = answer(&hierarchy, source, target);
            // The controls. A hundred identical refusals would pass this check
            // while the router answered nothing at all, so what each answer was
            // is counted and the counts are asserted at the end.
            if first.contains("\nbest none\n") {
                if first.contains("\nroute none\n") {
                    refused += 1;
                } else {
                    by_search += 1;
                }
            } else {
                by_shapes += 1;
            }

            for run in 1..RUNS {
                let again = answer(&hierarchy, source, target);
                assert_eq!(
                    again, first,
                    "{} run {run} answered differently for {} -> {}",
                    drawing.name, source.name, target.name
                );
            }
        }
    }

    assert!(
        by_shapes > 0,
        "some request produced a route from the shapes"
    );
    assert!(
        by_search > 0,
        "some request fell through to A* and it routed"
    );
    assert!(
        refused > 0,
        "some request was refused, so refusals compare too"
    );
}

/// What one drawing answers, in the order its own file was written.
struct Baseline {
    /// The terminals the drawing offers, by name.
    names: Vec<String>,
    /// What every pair of them answers.
    answers: String,
}

/// The baselines, held once for the whole binary.
///
/// The shuffled arm compares every case against these, and re-routing the
/// unshuffled drawing once per case would measure nothing the hundred-run arm
/// has not already measured.
fn baselines() -> &'static [Baseline] {
    static HELD: OnceLock<Vec<Baseline>> = OnceLock::new();
    HELD.get_or_init(|| {
        drawings()
            .iter()
            .map(|drawing| {
                let hierarchy = loaded(drawing.name, "baseline", &drawing.text);
                let terminals = terminals(&hierarchy);
                Baseline {
                    names: terminals
                        .iter()
                        .map(|terminal| terminal.name.clone())
                        .collect(),
                    answers: every_answer(&hierarchy, &terminals),
                }
            })
            .collect()
    })
}

/// One drawing, and a permutation of the items its file holds.
fn shuffled_drawing() -> impl Strategy<Value = (usize, Vec<usize>)> {
    let counts: Vec<usize> = drawings()
        .iter()
        .map(|drawing| item_count(&drawing.text))
        .collect();
    (0..counts.len()).prop_flat_map(move |which| (Just(which), permutation(counts[which])))
}

proptest! {
    #![proptest_config(config())]

    /// The file's order is not an input the answer may depend on.
    #[test]
    fn routing_is_identical_across_a_shuffled_item_order(
        (which, order) in shuffled_drawing(),
    ) {
        let drawing = &drawings()[which];
        let moved = reordered(&drawing.text, &order);

        // The controls, before anything is concluded. The shuffle must keep
        // every token of the file, and it must actually move something
        // whenever the permutation is not the file's own order.
        let tokens = |text: &str| Doc::parse(text).expect("the drawing parses").token_count();
        prop_assert_eq!(
            tokens(&moved),
            tokens(&drawing.text),
            "the shuffle kept every token"
        );
        let identity: Vec<usize> = (0..order.len()).collect();
        if order != identity {
            prop_assert_ne!(&moved, &drawing.text, "the shuffle moved an item");
        }

        let shuffled = loaded(drawing.name, "shuffled", &moved);
        let after = terminals(&shuffled);
        let baseline = &baselines()[which];
        // And the shuffled file must still offer the same terminals, or the
        // two answers would be answers to different questions.
        let names: Vec<String> = after.iter().map(|terminal| terminal.name.clone()).collect();
        prop_assert_eq!(&names, &baseline.names, "the shuffled file holds the same terminals");

        prop_assert_eq!(every_answer(&shuffled, &after), baseline.answers.clone());
    }
}
