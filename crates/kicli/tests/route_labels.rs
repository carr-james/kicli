//! A connection too long to draw is proposed as a pair of labels.
//!
//! `research/wire-routing.md` §5.5: when the best path is longer than
//! `routing.label_threshold`, or nothing routes at all, the router **proposes**
//! paired labels. It does not act. `--auto-labels` is what acts, and the output
//! says which happened.
//!
//! There is no routing verb yet, so "routing" here is the composition a route
//! request drives, as `tests/route_determinism.rs` and `tests/route_four_way.rs`
//! drive it: settle the terminals against the drawing, read the sheet into the
//! router's lists, build the window and the obstacle map, ask the shapes, and
//! fall through to the search. The new step is the last one, and it is the step
//! under test.
//!
//! **The drawing.** `U1.1` faces up out of a symbol low on the page; `U2.2`
//! faces down out of one high on the far side. The two escape the opposite way
//! from each other on purpose: a proposal that stepped both labels the same way
//! would land one of them inside the pin it belongs to, and a drawing whose two
//! ends face the same way could not tell the two rules apart.
//!
//! The netlist oracle runs only with `KICLI_TEST_KICAD_CLI` set. Without it the
//! connectivity claim is kicli's own extractor; with it, KiCad is asked about
//! the file kicli wrote.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use kicli::connectivity::extract;
use kicli::edit::mark::PinAddress;
use kicli::edit::wire::{End, Polyline, draw};
use kicli::geometry::{GRID, Iu, Point, resolve_pins};
use kicli::model::config::Routing;
use kicli::model::items::{LabelKind, SheetPath};
use kicli::model::{
    Config, Hierarchy, LoadedFile, Refdes, Target, WriteOptions, definition_of, read_library,
};
use kicli::route::propose::Proposal;
use kicli::route::report::{Added, Report, Status};
use kicli::route::terminal::Approach;
use kicli::route::{Obstacles, Routed, Search, Shapes, SheetObjects, Terminal, Window};
use kicli_probe::oracle::{Kicad, differences, kicli_partition, net};
use kicli_probe::{Probe, pin, symbol};

/// Where this binary writes the drawings it builds.
fn scratch() -> PathBuf {
    Path::new(env!("CARGO_TARGET_TMPDIR")).join("route-labels")
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

/// A two-pin symbol whose pins carry names.
///
/// The probe's own resistor names neither of its pins, and the name a proposal
/// falls back to is `<reference>_<pin name>`. A drawing that could not tell that
/// rule from `<reference>_<pin number>` would not be measuring it.
fn named_symbol() -> String {
    symbol(
        "U",
        "U",
        false,
        &[(
            "1_1",
            vec![
                pin("passive", ("0", "3.81"), "270", "1", "SCK"),
                pin("passive", ("0", "-3.81"), "90", "2", "GND"),
            ],
        )],
    )
}

/// Two symbols, far enough apart that the best route is over the threshold.
///
/// `U1.1` is the top pin of the lower symbol and leaves upwards. `U2.2` is the
/// bottom pin of the upper symbol and leaves downwards.
fn far_apart(name: &str) -> Probe {
    let mut probe = Probe::new(name, scratch());
    probe.define(named_symbol());
    probe.place("U", "U1", ("50.8", "105.41"), &["1", "2"]);
    probe.place("U", "U2", ("152.4", "76.2"), &["1", "2"]);
    probe
}

/// The same, with the source pin's net already carrying a name.
///
/// A label on a pin's own anchor names that pin's net, which is the name the
/// pair must take. Without this drawing the rule that a net's own name wins is
/// measured on no drawing at all.
fn far_apart_and_named(name: &str, net: &str) -> Probe {
    let mut probe = far_apart(name);
    probe.label_of_kind(
        kicli_probe::drawing::LabelKind::Local,
        net,
        ("50.8", "101.6"),
    );
    probe
}

/// Two symbols side by side, near enough that the best route is drawn.
fn side_by_side(name: &str) -> Probe {
    let mut probe = Probe::new(name, scratch());
    probe.define(named_symbol());
    probe.place("U", "U1", ("50.8", "105.41"), &["1", "2"]);
    probe.place("U", "U2", ("76.2", "105.41"), &["1", "2"]);
    probe
}

/// Load a written drawing as a project rooted at it.
fn loaded(path: &Path) -> Hierarchy {
    Hierarchy::load(path).expect("the drawing loads")
}

/// The root placement of a loaded drawing, and the file it draws.
fn root(hierarchy: &Hierarchy) -> (&LoadedFile, &SheetPath) {
    let placement = hierarchy
        .placements
        .first()
        .expect("the root sheet is placed");
    (&hierarchy.files[placement.file], &placement.path)
}

/// One pin of one placed symbol, as a terminal.
fn pin_of(hierarchy: &Hierarchy, reference: &str, number: &str) -> Terminal {
    let (file, path) = root(hierarchy);
    let schematic = &file.schematic;
    let library = read_library(&file.doc, &schematic.library_symbols, schematic.version);
    for symbol in schematic.symbols() {
        if symbol.reference_on(path).map(|found| found.0.as_str()) != Some(reference) {
            continue;
        }
        let definition = definition_of(&library, symbol).expect("the definition is embedded");
        for resolved in resolve_pins(&symbol.drawn_on(path), definition) {
            if resolved.number == number {
                return Terminal::of_pin(reference, &resolved);
            }
        }
    }
    panic!("{reference}.{number} is on this drawing");
}

/// What one route request answers, and the terminals it used.
///
/// The composition in full, shapes first and the search behind them, which is
/// the order `research/wire-routing.md` §7 fixes. A request that neither can
/// answer comes back `blocked`, which is the proposal's other trigger.
fn route(
    hierarchy: &Hierarchy,
    source: &Terminal,
    target: &Terminal,
    weights: &Routing,
) -> (Report, Approach) {
    let (file, path) = root(hierarchy);
    let approach = Approach::of(source, target, &file.schematic, GRID);
    let named = [approach.source.name.clone(), approach.target.name.clone()];
    let objects = SheetObjects::read(
        file,
        path,
        &Routed {
            wires: &[],
            terminals: &named,
        },
    );
    let window = Window::around(
        approach.source.at,
        approach.target.at,
        weights.margin,
        objects.page(),
        GRID,
    );
    let obstacles = Obstacles::build(window, &objects.geometry());

    let mut report = Report::of(Status::Routed, &approach.source.name, &approach.target.name);
    let shapes = Shapes::of(&approach.source, &approach.target, &obstacles, weights);
    if let Some(best) = shapes.best() {
        report.path.clone_from(&best.path);
        report.tally = best.tally;
        report.cost = best.cost;
        report.alternatives_considered = shapes.considered();
        return (report, approach);
    }
    let search = Search::of(&approach.source, &approach.target, &obstacles, weights);
    match search.route() {
        Some(found) => {
            report.path.clone_from(&found.path);
            report.tally = found.tally;
            report.cost = found.cost;
            report.alternatives_considered = search.expanded();
        }
        None => {
            report.status = Status::Blocked;
            report.blocked_by = search.blocked_by().to_vec();
        }
    }
    (report, approach)
}

/// How long the best path is, or nothing when there is no path.
fn best_length(report: &Report) -> Option<Iu> {
    (report.status == Status::Routed).then(|| report.length(GRID))
}

/// The target a request writes through: the file, its directory, its sheet.
fn written_to<'a>(path: &'a Path, project: &'a Path, sheet: &'a SheetPath) -> Target<'a> {
    Target {
        path,
        project,
        sheet_path: sheet,
        grid: GRID,
        options: WriteOptions::default(),
    }
}

/// Run the compiled binary with the given arguments.
///
/// Discovery is pointed at a path no `kicad-cli` is at, so a machine with KiCad
/// installed answers exactly as one without.
fn kicli(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_kicli"))
        .args(args)
        .env("KICLI_KICAD_CLI", "/nonexistent/kicad-cli")
        .output()
        .expect("the binary runs")
}

/// Everything a run wrote to standard output.
fn stdout(run: &Output) -> String {
    String::from_utf8(run.stdout.clone()).expect("stdout is text")
}

/// Everything a run wrote to standard error.
fn stderr(run: &Output) -> String {
    String::from_utf8(run.stderr.clone()).expect("stderr is text")
}

/// Ask the binary to join two pins, writing the pair rather than a wire.
fn auto_labels(project: &Path, from: &str, to: &str) -> Output {
    kicli(&[
        "wire",
        "draw",
        "--from-pin",
        from,
        "--to-pin",
        to,
        "--auto-labels",
        "-p",
        project.to_str().expect("the path is text"),
    ])
}

#[test]
fn a_long_route_is_proposed_as_labels_and_not_drawn() {
    let path = far_apart("proposed").write();
    let before = std::fs::read(&path).expect("the drawing reads");
    let hierarchy = loaded(&path);
    let source = pin_of(&hierarchy, "U1", "1");
    let target = pin_of(&hierarchy, "U2", "2");
    let weights = Config::default().routing;

    // A route exists. It is not the route that is refused — it is the drawing
    // this connection would make, and the proposal is the better one.
    let (routed, approach) = route(&hierarchy, &source, &target, &weights);
    assert_eq!(routed.status, Status::Routed, "{:?}", routed.blocked_by);
    let length = routed.length(GRID);
    assert_eq!(length, Iu(1_231_900), "the best path is 123.19mm");
    assert!(
        length.0 > weights.label_threshold.0,
        "which is over the threshold of {}",
        weights.label_threshold.0
    );

    let proposal = Proposal::of(
        &approach.source,
        &approach.target,
        best_length(&routed),
        "U1_SCK",
        &weights,
        GRID,
    )
    .expect("a path that long is proposed as labels");
    let report = proposal.report(&approach.source, &approach.target);

    assert_eq!(report.status.token(), "labels");
    assert_eq!(report.from, "U1.1");
    assert_eq!(report.to, "U2.2");
    let reason = report.reason.as_deref().expect("a proposal says why");
    assert!(reason.contains("123.19mm"), "the length: {reason}");
    assert!(reason.contains("38.10mm"), "and the threshold: {reason}");

    // It proposes. Nothing is drawn and nothing is added.
    assert!(report.path.is_empty(), "a proposal draws no wire");
    assert_eq!(report.segments(), 0);
    assert_eq!(report.added, Added::default());
    let pair = report.labels.as_ref().expect("the pair is proposed");
    assert_eq!(pair.name, "U1_SCK");
    assert_eq!(
        pair.at,
        [at("50.8", "99.06"), at("152.4", "82.55")],
        "two grid steps along each pin's own direction"
    );
    for anchor in pair.at {
        assert!(anchor.is_on_grid(), "{anchor} is on the grid");
    }

    // And the file is exactly as it was found.
    assert_eq!(
        std::fs::read(&path).expect("the drawing reads"),
        before,
        "a proposal writes nothing"
    );

    // The control on that comparison, and on the threshold itself. The same
    // reading over a drawing that WAS written to differs — so the assertion
    // above watches the file rather than agreeing with itself — and a shorter
    // connection on the same symbols is drawn rather than proposed.
    let near = side_by_side("proposed-control").write();
    let untouched = std::fs::read(&near).expect("the drawing reads");
    let hierarchy = loaded(&near);
    let (short, approach) = route(
        &hierarchy,
        &pin_of(&hierarchy, "U1", "1"),
        &pin_of(&hierarchy, "U2", "1"),
        &weights,
    );
    assert_eq!(short.status, Status::Routed);
    assert!(short.length(GRID).0 < weights.label_threshold.0);
    assert!(
        Proposal::of(
            &approach.source,
            &approach.target,
            best_length(&short),
            "U1_SCK",
            &weights,
            GRID,
        )
        .is_none(),
        "a connection under the threshold is drawn, not proposed"
    );

    let project = near.parent().expect("the drawing sits in a directory");
    let sheet = hierarchy.placements[0].path.clone();
    let mut writing = loaded(&near);
    draw(
        &mut writing,
        &Polyline {
            from: End::Pin(PinAddress::new(Refdes("U1".to_owned()), "1")),
            to: End::Pin(PinAddress::new(Refdes("U2".to_owned()), "1")),
            via: short.path[1..short.path.len() - 1].to_vec(),
        },
        &weights,
        &written_to(&near, project, &sheet),
        "after",
    )
    .expect("the route kicli chose is drawable");
    assert_ne!(
        std::fs::read(&near).expect("the drawing reads"),
        untouched,
        "a drawing that was written to does not read back the same"
    );
}

#[test]
fn auto_labels_writes_the_pair_and_says_so() {
    let path = far_apart("performed").write();
    let project = path.parent().expect("the drawing sits in a directory");

    // Before: the two pins are on nets of their own.
    let apart = extract(&loaded(&path));
    assert!(
        apart
            .net_of("U1", "1")
            .is_none_or(|net| net.pins.iter().all(|pin| pin.label() != "U2.2")),
        "the two pins start unjoined"
    );

    let run = auto_labels(project, "U1.1", "U2.2");
    assert_eq!(
        run.status.code(),
        Some(0),
        "a proposal is a result: {}",
        stderr(&run)
    );
    let printed = stdout(&run);
    assert!(
        printed.starts_with("labels U1.1 -> U2.2\n"),
        "the status word comes first: {printed}"
    );
    assert!(
        printed.contains("  reason: path length 123.19mm is over the threshold 38.10mm\n"),
        "and it says why: {printed}"
    );
    assert!(
        printed.contains("  labels: \"U1_SCK\" at 50.80,99.06 and 152.40,82.55\n"),
        "and where the pair went: {printed}"
    );
    assert!(
        printed.contains("checked: every invariant passed"),
        "the mutation result follows it: {printed}"
    );

    // The drawing now holds the pair, each label two grid steps along its own
    // pin's direction and on the grid.
    let after = loaded(&path);
    let (file, _) = root(&after);
    let labels: Vec<(&str, Point)> = file
        .schematic
        .labels()
        .map(|label| {
            assert_eq!(
                label.kind,
                LabelKind::Local,
                "a local label names one sheet"
            );
            (label.text.as_str(), label.at)
        })
        .collect();
    assert_eq!(
        labels,
        vec![
            ("U1_SCK", at("50.8", "99.06")),
            ("U1_SCK", at("152.4", "82.55")),
        ],
        "one label per end, both carrying the name the net had none of"
    );
    for (_, anchor) in &labels {
        assert!(anchor.is_on_grid(), "{anchor} is on the grid");
    }

    // Two stubs and no wire between the two ends. The stub is what makes each
    // label its own pin's: a label standing off a pin with nothing between them
    // names a net that pin is not on.
    let wires: Vec<(Point, Point)> = file
        .schematic
        .lines()
        .map(|line| (line.from, line.to))
        .collect();
    assert_eq!(
        wires,
        vec![
            (at("50.8", "101.6"), at("50.8", "99.06")),
            (at("152.4", "80.01"), at("152.4", "82.55")),
        ],
        "a stub from each pin to its own label, and nothing joining the two"
    );

    // And the extractor joins the two pins, which is the point of the pair.
    let joined = extract(&after);
    let net = joined
        .net_of("U1", "1")
        .expect("the routed pin is on a net");
    assert_eq!(net.name, "U1_SCK", "the net carries the name kicli wrote");
    assert!(
        net.pins.iter().any(|pin| pin.label() == "U2.2"),
        "the pair joins the two pins: {:?}",
        net.pins.iter().map(|pin| pin.label()).collect::<Vec<_>>()
    );
}

#[test]
fn a_named_net_keeps_its_name() {
    // The other half of the naming rule. `U1_SCK` is what a proposal falls
    // back to; a net the drawing already names keeps that name, because a pair
    // that renamed it would split the net it was asked to join.
    let path = far_apart_and_named("performed-named", "SPI_SCK").write();
    let project = path.parent().expect("the drawing sits in a directory");
    let named = extract(&loaded(&path));
    let before = named.net_of("U1", "1").expect("the pin is on a net");
    assert_eq!(before.name, "SPI_SCK", "the drawing names this net already");

    let run = auto_labels(project, "U1.1", "U2.2");
    assert_eq!(
        run.status.code(),
        Some(0),
        "the pair is written: {}",
        stderr(&run)
    );
    assert!(
        stdout(&run).contains("  labels: \"SPI_SCK\" at 50.80,99.06 and 152.40,82.55\n"),
        "the pair takes the name the net had: {}",
        stdout(&run)
    );

    let joined = extract(&loaded(&path));
    let net = joined.net_of("U1", "1").expect("the pin is still on a net");
    assert_eq!(net.name, "SPI_SCK");
    assert!(
        net.pins.iter().any(|pin| pin.label() == "U2.2"),
        "and it now reaches the other pin: {:?}",
        net.pins.iter().map(|pin| pin.label()).collect::<Vec<_>>()
    );
}

#[test]
fn the_threshold_is_the_configured_one() {
    // One drawing, read twice. The only difference between the two answers is
    // the `kicli.toml` beside it, which is what makes this a check on the knob
    // rather than on the geometry.
    let path = side_by_side("configured").write();
    let project = path.parent().expect("the drawing sits in a directory");
    // A scratch directory outlives one run, and the file this check writes in
    // its second arm would still be there in the first arm of the next one.
    // Measured: without this the check passes once and fails on every rerun.
    let settings_file = project.join("kicli.toml");
    let _ = std::fs::remove_file(&settings_file);
    let untouched = std::fs::read(&path).expect("the drawing reads");
    let hierarchy = loaded(&path);
    let source = pin_of(&hierarchy, "U1", "1");
    let target = pin_of(&hierarchy, "U2", "1");

    let settings = |weights: &Routing| {
        let (routed, approach) = route(&hierarchy, &source, &target, weights);
        assert_eq!(routed.status, Status::Routed, "{:?}", routed.blocked_by);
        assert_eq!(routed.length(GRID), Iu(304_800), "the best path is 30.48mm");
        Proposal::of(
            &approach.source,
            &approach.target,
            best_length(&routed),
            "U1_SCK",
            weights,
            GRID,
        )
    };

    let default = Config::read(project).expect("a directory with no kicli.toml reads as defaults");
    assert_eq!(
        default.routing.label_threshold,
        Iu(30 * GRID.0),
        "the documented default"
    );
    assert!(
        settings(&default.routing).is_none(),
        "30.48mm is under 38.10mm, so it is drawn"
    );

    std::fs::write(&settings_file, "[routing]\nlabel_threshold = \"10G\"\n")
        .expect("the project directory is writable");
    let lowered = Config::read(project).expect("the file kicli just wrote reads");
    assert_eq!(
        lowered.routing.label_threshold,
        Iu(10 * GRID.0),
        "the file moved the knob"
    );
    let proposal = settings(&lowered.routing).expect("30.48mm is over 12.70mm, so it is proposed");
    let reason = proposal.trigger.reason("U2.1");
    assert!(reason.contains("30.48mm"), "the length: {reason}");
    assert!(
        reason.contains("12.70mm"),
        "and the threshold the file gave: {reason}"
    );

    // The drawing never changed. Both answers are about the same file.
    assert_eq!(
        std::fs::read(&path).expect("the drawing reads"),
        untouched,
        "the same drawing gave both answers"
    );
}

#[test]
fn the_written_pair_joins_the_pins_kicad_reads() {
    // The oracle. Two labels and two stubs are a connection kicli's extractor
    // reads; whether KiCad reads it is the question that matters, and it is not
    // a question about kicli's arithmetic.
    let Some(kicad) = Kicad::found_or_skip("ask KiCad about the labels kicli wrote") else {
        return;
    };

    let path = far_apart("performed-oracle").write();
    let project = path.parent().expect("the drawing sits in a directory");

    // The control comes first, on the drawing before anything was written: the
    // two pins are apart, and KiCad says so. Without it, a KiCad that joined
    // everything would pass the measurement below.
    let apart = kicad
        .netlist(&path, &path.with_extension("before.net"))
        .partition();
    assert!(
        !apart.contains(&net(&["U1.1", "U2.2"])),
        "KiCad reads no connection before kicli wrote one: {apart:?}"
    );
    assert_eq!(
        differences(&kicli_partition(&extract(&loaded(&path))), &apart),
        None,
        "kicli and KiCad partition the untouched drawing the same way"
    );

    let run = auto_labels(project, "U1.1", "U2.2");
    assert_eq!(
        run.status.code(),
        Some(0),
        "the pair is written: {}",
        stderr(&run)
    );

    let joined = kicad.netlist_beside(&path).partition();
    assert!(
        joined.contains(&net(&["U1.1", "U2.2"])),
        "KiCad joins the two pins the pair names: {joined:?}"
    );
    assert_eq!(
        differences(&kicli_partition(&extract(&loaded(&path))), &joined),
        None,
        "kicli and KiCad partition the written drawing the same way"
    );
}
