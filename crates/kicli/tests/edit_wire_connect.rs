//! `wire connect`: the router behind a verb, and the junctions it emits.
//!
//! `wire draw` takes the corners. This verb takes two ends and chooses the
//! path, so the questions here are the ones a chosen path raises: does the
//! drawing that comes out join what the report says it joins, does the route
//! reach a terminus that sits under a hard block, and does it put a dot exactly
//! where a dot is what makes the connection.
//!
//! **Three claims, and each one is measured against a drawing rather than
//! against the arithmetic that produced it.**
//!
//! *The route joins what it names.* The extractor is asked about the written
//! file, and the net the command **reported** is compared against the net the
//! extractor **found**. A report that is right about the connection and wrong
//! about its name is the defect that clause exists to catch.
//!
//! *The terminus may sit under a hard block.* A sheet pin sits on the border of
//! its own sheet body, which is a hard block, so without
//! `research/wire-routing.md` §3.2's target-cell exception the route is refused
//! at the very point it was asked to reach. This is where that rule is measured
//! on a drawing the tool reads back.
//!
//! *A junction is emitted where a junction is what joins.* The pair below is a
//! pair on purpose: a rule tested only where it fires is a rule half tested. The
//! interior arm's junction is measured by **removing it and re-measuring**, on
//! the same file kicli wrote — the control that separates "the junction is in
//! the file" from "the junction is what makes the connection".
//!
//! The netlist oracle runs only with `KICLI_TEST_KICAD_CLI` set. Without it the
//! connectivity claims here are kicli's own extractor; with it, KiCad is asked
//! about the file kicli wrote.

use kicli::connectivity::extract;
use kicli::model::Hierarchy;
use kicli_probe::drawing::{LabelKind, LabelShape, rectangle, symbol};
use kicli_probe::oracle::{Kicad, Partition, kicli_partition, net, skipped};
use kicli_probe::{Port, Probe};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// The uuid of the child sheet the port drawings place.
const CHILD: &str = "00000000-0000-4000-8000-cccccccccccc";

/// A path no `kicad-cli` is at.
///
/// Every run of the binary points discovery at it, so a machine with KiCad
/// installed gives the same answer as one without. The oracle below runs
/// `kicad-cli` itself, on the file the binary left behind.
const NO_KICAD_CLI: &str = "/nonexistent/kicad-cli";

/// Where this binary writes the drawings it builds.
fn scratch() -> PathBuf {
    Path::new(env!("CARGO_TARGET_TMPDIR")).join("wire-connect")
}

/// Run the compiled binary with the given arguments.
fn kicli(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_kicli"))
        .args(args)
        .env("KICLI_KICAD_CLI", NO_KICAD_CLI)
        .output()
        .expect("the binary runs")
}

/// One run of `wire connect`, as the JSON object it printed.
struct Connected {
    object: serde_json::Value,
    code: i32,
    stderr: String,
}

impl Connected {
    /// The object a successful run printed, or a panic naming the refusal.
    fn object(&self) -> &serde_json::Value {
        assert_eq!(self.code, 0, "the connection was refused: {}", self.stderr);
        &self.object
    }

    /// The route contract the run answered in.
    fn wire(&self) -> &serde_json::Value {
        &self.object()["wire"]
    }

    /// The net the command claims the two ends are now on.
    fn claimed_net(&self) -> Option<&str> {
        self.object()["net"].as_str()
    }

    /// The identifiers of the junctions the run wrote.
    fn junctions(&self) -> Vec<String> {
        self.wire()["added"]["junctions"]
            .as_array()
            .expect("the key is always there")
            .iter()
            .map(|value| value.as_str().unwrap_or_default().to_owned())
            .collect()
    }
}

/// Ask the binary to connect two ends of a written drawing.
fn connect(sheet: &Path, ends: &[&str]) -> Connected {
    let project = sheet
        .parent()
        .expect("the drawing sits in a directory")
        .to_str()
        .expect("the path is text")
        .to_owned();
    let mut arguments = vec!["wire", "connect"];
    arguments.extend_from_slice(ends);
    arguments.extend(["--output", "json", "-p", &project]);
    let run = kicli(&arguments);
    let code = run.status.code().expect("the run ended by itself");
    let stdout = String::from_utf8(run.stdout).expect("stdout is text");
    let stderr = String::from_utf8(run.stderr).expect("stderr is text");
    let object = if code == 0 {
        serde_json::from_str(&stdout).expect("one object on stdout")
    } else {
        serde_json::Value::Null
    };
    Connected {
        object,
        code,
        stderr,
    }
}

/// What kicli's extractor says about a written drawing.
fn kicli_nets(path: &Path) -> Partition {
    let hierarchy = Hierarchy::load(path).expect("the written drawing loads");
    kicli_partition(&extract(&hierarchy))
}

/// The net one pin is on, as kicli's extractor names it.
fn net_name_of(path: &Path, reference: &str, number: &str) -> String {
    let hierarchy = Hierarchy::load(path).expect("the written drawing loads");
    extract(&hierarchy)
        .net_of(reference, number)
        .unwrap_or_else(|| panic!("{reference}.{number} is on a net"))
        .name
        .clone()
}

/// What KiCad says about a file, when the environment asked for the tool.
fn oracle(path: &Path) -> Option<Partition> {
    Kicad::found().map(|kicad| kicad.netlist_beside(path).partition())
}

/// A partition as one comparable line per net.
fn lines(partition: &Partition) -> BTreeSet<String> {
    partition.iter().map(|pins| pins.join(",")).collect()
}

/// Two resistors to join, a name for one of them, and a net in the way.
///
/// R1.1 sits at `(50.8, 50.8)` and R2.1 at `(88.9, 50.8)`; a resistor's pin 1
/// is above its anchor and its body below, so both are left upwards. The stub
/// leaves R2.1 sideways, clear of the column the route comes down, so the wire
/// kicli draws lies on top of nothing.
///
/// **The third net is what makes "no other net changed" a live claim.** R5.1
/// and R6.1 are joined by a wire the route has to cross to get from one
/// resistor to the other. A crossing is allowed and costed, and it is
/// emphatically not a connection — so a drawing with nothing to cross would
/// let the oracle's control pass by having nothing to say.
fn two_resistors_one_named(name: &str) -> PathBuf {
    let mut probe = Probe::new(name, scratch());
    probe.place("R", "R1", ("50.8", "54.61"), &["1", "2"]);
    probe.place("R", "R2", ("88.9", "54.61"), &["1", "2"]);
    probe.wire(("88.9", "50.8"), ("101.6", "50.8"));
    probe.label_of_kind(LabelKind::Local, "SIG_A", ("101.6", "50.8"));
    probe.place("R", "R5", ("69.85", "34.29"), &["1", "2"]);
    probe.place("R", "R6", ("69.85", "67.31"), &["1", "2"]);
    probe.wire(("69.85", "30.48"), ("69.85", "63.5"));
    probe.write()
}

/// One resistor, and a wire of another net with a free end.
///
/// R1.1 is at `(50.8, 50.8)`. The wire runs from R3.1 at `(76.2, 38.1)` to a
/// free end at `(101.6, 38.1)`, so one drawing offers both arms of the junction
/// rule: `(88.9, 38.1)` is its interior and `(101.6, 38.1)` is its end. Both
/// arms join the same two pins, which is what makes the pair a pair.
fn a_wire_with_an_interior_and_an_end(name: &str) -> PathBuf {
    let mut probe = Probe::new(name, scratch());
    probe.place("R", "R1", ("50.8", "54.61"), &["1", "2"]);
    probe.place("R", "R3", ("76.2", "41.91"), &["1", "2"]);
    probe.wire(("76.2", "38.1"), ("101.6", "38.1"));
    probe.write()
}

/// A sheet with one port on its right edge, and a resistor beyond it.
///
/// The port is written on the edge its angle names, which is what the tool was
/// measured to require. Its connection point is on the sheet body's border, and
/// the body is a hard block — so the last step of any route to it is a step
/// into a block.
fn a_port_under_its_own_sheet_body(name: &str) -> PathBuf {
    let mut probe = Probe::new(name, scratch());
    let mut child = Probe::child_of(&probe);
    probe.sheet_of_size(
        CHILD,
        "child",
        ("101.6", "63.5"),
        ("25.4", "25.4"),
        &[Port {
            name: "OUT",
            at: ("127", "71.12"),
            angle: "0",
        }],
    );
    probe.place("R", "R1", ("139.7", "78.74"), &["1", "2"]);
    child.strand_of_kind(
        LabelKind::Hierarchical(LabelShape::Bidirectional),
        "RC1",
        "25.4",
        "29.21",
        "OUT",
    );
    let sheet = probe.write_all(&[&child]);
    // Two schematics in one directory are two candidate roots, and the loader
    // will not choose between them. The project file is what names the root,
    // and it is written here rather than in the drawing builder because it is
    // the command layer's requirement rather than the probe's.
    std::fs::write(
        sheet.with_extension("kicad_pro"),
        "{\n  \"board\": {},\n  \"meta\": { \"filename\": \"probe.kicad_pro\", \"version\": 3 },\n  \"schematic\": {}\n}\n",
    )
    .expect("the project file is writable");
    sheet
}

/// A copy of a written drawing with one junction record taken out.
///
/// The removal is done on **kicli's own output**, because the item whose causal
/// role is being measured is one kicli wrote rather than one the probe drew.
/// The copy goes in a directory of its own so that the reading of the original
/// is still there to compare against.
///
/// The record is taken whole, by bracket balance from the line that opens it,
/// so a record written over several lines goes as one.
fn without_the_junction(sheet: &Path, uuid: &str) -> PathBuf {
    let text = std::fs::read_to_string(sheet).expect("the written drawing reads");
    let balance = |line: &str| -> i32 {
        let count = |bracket: char| i32::try_from(line.matches(bracket).count()).unwrap_or(0);
        count('(') - count(')')
    };
    let source: Vec<&str> = text.lines().collect();
    let mut kept: Vec<&str> = Vec::new();
    let mut removed = 0;
    let mut index = 0;
    while index < source.len() {
        if !source[index].trim_start().starts_with("(junction") {
            kept.push(source[index]);
            index += 1;
            continue;
        }
        // The record runs from this line until its own brackets close, which
        // is how a record written over several lines is taken whole.
        let mut depth = 0;
        let mut last = index;
        while last < source.len() {
            depth += balance(source[last]);
            if depth <= 0 {
                break;
            }
            last += 1;
        }
        let last = last.min(source.len() - 1);
        if source[index..=last].join("\n").contains(uuid) {
            removed += 1;
        } else {
            kept.extend_from_slice(&source[index..=last]);
        }
        index = last + 1;
    }
    assert_eq!(removed, 1, "exactly one junction record was taken out");

    let from = sheet.parent().expect("the drawing sits in a directory");
    let directory = from.with_extension("without-junction");
    std::fs::create_dir_all(&directory).expect("the scratch directory is writable");
    // Every sibling schematic goes too, so a child sheet is still reachable.
    for entry in std::fs::read_dir(from).expect("the directory reads") {
        let sibling = entry.expect("an entry").path();
        if sibling.extension().is_some_and(|end| end == "kicad_sch") && sibling != sheet {
            std::fs::copy(
                &sibling,
                directory.join(sibling.file_name().expect("a file name")),
            )
            .expect("the sibling copies");
        }
    }
    let path = directory.join(sheet.file_name().expect("the drawing has a name"));
    std::fs::write(&path, format!("{}\n", kept.join("\n"))).expect("the copy is writable");
    path
}

/// Does the written drawing still hold this net?
fn after_holds(path: &Path, wanted: &[String]) -> bool {
    kicli_nets(path).contains(wanted)
}

#[test]
fn a_route_joins_the_two_pins_it_names() {
    let sheet = two_resistors_one_named("connect-two-pins");
    let before = kicli_nets(&sheet);
    assert!(
        !before.contains(&net(&["R1.1", "R2.1"])),
        "the two pins are not joined before the command runs: {before:?}"
    );

    let run = connect(&sheet, &["--from-pin", "R1.1", "--to-pin", "R2.1"]);
    assert_eq!(run.wire()["status"], "routed", "{}", run.object());
    assert!(
        run.wire()["segments"].as_u64().unwrap_or_default() > 0,
        "a routed answer draws a wire: {}",
        run.object()
    );
    // The route crosses the third net on its way, and a crossing is not a
    // connection: no junction is written for one.
    assert_eq!(
        run.wire()["crossings"].as_array().map(Vec::len),
        Some(1),
        "the route crosses the net in the way, and says so: {}",
        run.object()
    );
    assert!(
        run.junctions().is_empty(),
        "a crossing gets no dot: {}",
        run.object()
    );
    assert!(
        after_holds(&sheet, &net(&["R5.1", "R6.1"])),
        "the net the route crossed is still its own net"
    );

    // The extractor agrees about the file the command wrote.
    let after = kicli_nets(&sheet);
    assert!(
        after.contains(&net(&["R1.1", "R2.1"])),
        "the two pins the command named are not joined: {after:?}"
    );

    // And the net the command claimed is the net the extractor found. A report
    // that is right about the connection and wrong about its name is what this
    // clause is for, so the two are compared rather than each checked alone.
    let found = net_name_of(&sheet, "R1", "1");
    assert_eq!(
        run.claimed_net(),
        Some(found.as_str()),
        "the claimed net is not the extractor's: {}",
        run.object()
    );
    assert_eq!(
        found, "SIG_A",
        "the merged net keeps the name the drawing gives it"
    );
    assert_eq!(
        net_name_of(&sheet, "R2", "1"),
        found,
        "both ends are on the one net the command claimed"
    );
}

#[test]
fn a_terminus_on_a_wire_interior_emits_a_junction() {
    let sheet = a_wire_with_an_interior_and_an_end("connect-interior");
    let before = kicli_nets(&sheet);
    assert!(
        before.contains(&net(&["R3.1"])),
        "the wire's net starts with R3.1 alone on it: {before:?}"
    );

    // (88.9, 38.1) is strictly inside the wire from (76.2, 38.1) to
    // (101.6, 38.1): a KiCad wire's connection points are its two ends and
    // nothing between them, so a route that ends there is joined to nothing
    // until a junction says otherwise.
    let run = connect(&sheet, &["--from-pin", "R1.1", "--to-at", "88.9,38.1"]);
    assert_eq!(run.wire()["status"], "routed", "{}", run.object());
    let junctions = run.junctions();
    assert_eq!(
        junctions.len(),
        1,
        "one junction, at the terminus: {}",
        run.object()
    );

    let joined = net(&["R1.1", "R3.1"]);
    let after = kicli_nets(&sheet);
    assert!(
        after.contains(&joined),
        "the route did not join R1.1 to the wire's net: {after:?}"
    );

    // The control. The junction's causal role is measured by taking it out of
    // the file kicli wrote and reading the drawing again — not by finding the
    // record and believing it did something.
    let without = without_the_junction(&sheet, &junctions[0]);
    let removed = kicli_nets(&without);
    assert!(
        !removed.contains(&joined),
        "the connection survives the junction's removal, so the junction is \
         not what makes it: {removed:?}"
    );
    assert!(
        removed.contains(&net(&["R3.1"])),
        "and the wire's own net is still there, so the removal took one record \
         rather than the drawing: {removed:?}"
    );

    // KiCad, asked about both files, when the tool was asked for.
    if let (Some(with), Some(cut)) = (oracle(&sheet), oracle(&without)) {
        assert!(
            with.contains(&joined),
            "KiCad does not report the join kicli made: {with:?}"
        );
        assert!(
            !cut.contains(&joined),
            "KiCad joins them without the junction, so the junction is not \
             what makes the connection: {cut:?}"
        );
    }
}

#[test]
fn a_terminus_at_an_endpoint_does_not() {
    let sheet = a_wire_with_an_interior_and_an_end("connect-endpoint");

    // (101.6, 38.1) is the free end of the same wire. Two wire ends that meet
    // are one conductor with no dot at all, and KiCad renders a corner there,
    // so a junction would draw something the drawing already says.
    let run = connect(&sheet, &["--from-pin", "R1.1", "--to-at", "101.6,38.1"]);
    assert_eq!(run.wire()["status"], "routed", "{}", run.object());
    assert!(
        run.junctions().is_empty(),
        "a terminus at an existing wire end needs no junction: {}",
        run.object()
    );
    let written = std::fs::read_to_string(&sheet).expect("the written drawing reads");
    assert!(
        !written.contains("(junction"),
        "and none reached the file: {written}"
    );

    // The other half of the pair: the connection is made anyway. Without this
    // the check would pass just as happily against a verb that connected
    // nothing at all.
    let joined = net(&["R1.1", "R3.1"]);
    let after = kicli_nets(&sheet);
    assert!(
        after.contains(&joined),
        "the route did not join R1.1 to the wire's net: {after:?}"
    );
    if let Some(kicad) = oracle(&sheet) {
        assert!(
            kicad.contains(&joined),
            "KiCad does not report the join kicli made without a junction: {kicad:?}"
        );
    }
}

#[test]
fn a_route_reaches_a_sheet_pin_under_its_own_body() {
    // The measurement carried in from the routing window (T7): the terminus
    // cell sits on the border of the sheet body, and a sheet body is a hard
    // block. Without §3.2's target-cell exception the route is refused at the
    // very point it was asked to reach.
    let sheet = a_port_under_its_own_sheet_body("connect-sheet-pin");
    let run = connect(&sheet, &["--from-pin", "R1.1", "--to-port", "OUT"]);
    assert_eq!(
        run.code, 0,
        "the route to the port was refused: {}",
        run.stderr
    );
    assert_eq!(run.wire()["status"], "routed", "{}", run.object());

    // The route really does end on the port, under the body.
    let path = run.wire()["path"].as_array().expect("a routed path");
    assert_eq!(
        path.last().expect("the path has an end"),
        &serde_json::json!([127.0, 71.12]),
        "the route ends on the port: {}",
        run.object()
    );

    // And the drawing that came out carries the connection through the sheet.
    let joined = net(&["R1.1", "RC1.1"]);
    let after = kicli_nets(&sheet);
    assert!(
        after.contains(&joined),
        "the route did not carry R1.1 into the child sheet: {after:?}"
    );
    if let Some(kicad) = oracle(&sheet) {
        assert!(
            kicad.contains(&joined),
            "KiCad does not report the join through the port: {kicad:?}"
        );
    }
}

#[test]
fn kicad_agrees_about_the_route() {
    let Some(kicad) = Kicad::found_or_skip("ask KiCad about the routes kicli wrote") else {
        return;
    };

    // Two writes, because the second carries a junction and the first does
    // not. Each is measured the same way: the join kicli reported is in KiCad's
    // netlist, and nothing that is not part of that join has moved.
    for (name, ends, expected) in [
        (
            "oracle-two-pins",
            vec!["--from-pin", "R1.1", "--to-pin", "R2.1"],
            net(&["R1.1", "R2.1"]),
        ),
        (
            "oracle-interior",
            vec!["--from-pin", "R1.1", "--to-at", "88.9,38.1"],
            net(&["R1.1", "R3.1"]),
        ),
    ] {
        let sheet = if name == "oracle-two-pins" {
            two_resistors_one_named(name)
        } else {
            a_wire_with_an_interior_and_an_end(name)
        };
        let before = lines(&kicad.netlist_beside(&sheet).partition());
        assert!(
            !before.is_empty(),
            "{name}: the before-reading is empty, so a comparison says nothing"
        );

        let run = connect(&sheet, &ends);
        assert_eq!(run.wire()["status"], "routed", "{name}: {}", run.object());
        let after = lines(&kicad.netlist_beside(&sheet).partition());

        // The join kicli reported is the join KiCad reports.
        assert!(
            after.contains(&expected.join(",")),
            "{name}: KiCad does not carry the join kicli made: {after:?}"
        );

        // The control: nothing else changed. A write that connects the two ends
        // by fusing half the sheet satisfies the clause above and fails this
        // one. Only nets whose pins are all part of the join may move.
        let touched = |line: &String| -> bool {
            line.split(',')
                .all(|pin| expected.iter().any(|had| had == pin))
        };
        let gone: Vec<&String> = before.difference(&after).filter(|l| !touched(l)).collect();
        let fresh: Vec<&String> = after.difference(&before).filter(|l| !touched(l)).collect();
        assert!(
            gone.is_empty() && fresh.is_empty(),
            "{name}: a net that is no part of the join moved. \
             gone: {gone:?}, new: {fresh:?}"
        );
    }
}

/// The pair-arm nobody runs by hand: a proposal writes nothing at all.
///
/// A connection longer than `routing.label_threshold` is an answer rather than
/// a failure, so the run succeeds — and the drawing must be byte for byte what
/// it was, because the proposal is the thing kicli decided **not** to draw.
#[test]
fn a_connection_over_the_threshold_is_proposed_and_not_drawn() {
    let sheet = two_resistors_one_named("connect-proposed");
    let before = std::fs::read_to_string(&sheet).expect("the drawing reads");

    // The default threshold is 300 grid steps, which is 381 mm, so the drawing
    // above is far under it. The knob is what moves the boundary onto it.
    let project = sheet.parent().expect("the drawing sits in a directory");
    std::fs::write(
        project.join("kicli.toml"),
        "[routing]\nlabel_threshold = \"1G\"\n",
    )
    .expect("the settings file is writable");

    let run = connect(&sheet, &["--from-pin", "R1.1", "--to-pin", "R2.1"]);
    assert_eq!(run.wire()["status"], "labels", "{}", run.object());
    assert_eq!(
        run.claimed_net(),
        None,
        "nothing was written, so nothing was joined: {}",
        run.object()
    );
    assert_eq!(
        before,
        std::fs::read_to_string(&sheet).expect("the drawing reads"),
        "a proposal wrote to the file anyway"
    );
    if let Some(reason) = run.wire()["reason"].as_str() {
        assert!(
            reason.contains("threshold"),
            "the proposal says why: {reason}"
        );
    }
}

/// A skipped oracle says so, so a silent run is not read as a passing one.
#[test]
fn the_oracle_says_when_it_did_not_run() {
    if Kicad::found().is_none() {
        skipped("ask KiCad about the routes kicli wrote");
    }
}

/// The project file two schematics in one directory need.
///
/// The loader will not choose between two candidate roots, and the project
/// file is what names the root. It is written here rather than in the drawing
/// builder because it is the command layer's requirement rather than the
/// probe's.
const PROJECT: &str = concat!(
    "{\n  \"board\": {},\n",
    "  \"meta\": { \"filename\": \"probe.kicad_pro\", \"version\": 3 },\n",
    "  \"schematic\": {}\n}\n",
);

/// A source pin, and one net drawn in two places on one sheet.
///
/// `SIG` is one net drawn as two vertical strands: R3's, near the source, and
/// R4's, far to the right. Both carry a local label of the same text, so the
/// two strands are one net that a route can reach at two very different costs.
///
/// `near` and `far` choose which strands are drawn, so the same builder makes
/// the drawing under test and the two controls it is measured against. A
/// control is what turns "it connected to the near one" into "the near one is
/// the cheaper of two the router really had".
fn a_net_in_two_places(name: &str, near: bool, far: bool) -> PathBuf {
    let mut probe = Probe::new(name, scratch());
    probe.place("R", "R1", ("76.2", "54.61"), &["1", "2"]);
    if near {
        probe.place("R", "R3", ("69.85", "46.99"), &["1", "2"]);
        probe.wire(("69.85", "43.18"), ("69.85", "33.02"));
        probe.label_of_kind(LabelKind::Local, "SIG", ("69.85", "33.02"));
    }
    if far {
        probe.place("R", "R4", ("114.3", "46.99"), &["1", "2"]);
        probe.wire(("114.3", "43.18"), ("114.3", "33.02"));
        probe.label_of_kind(LabelKind::Local, "SIG", ("114.3", "33.02"));
    }
    probe.write()
}

/// One named net and one the drawing does not name, on one sheet.
fn a_named_net_and_an_unnamed_one(name: &str) -> PathBuf {
    let mut probe = Probe::new(name, scratch());
    probe.place("R", "R1", ("76.2", "54.61"), &["1", "2"]);
    probe.place("R", "R3", ("69.85", "46.99"), &["1", "2"]);
    probe.wire(("69.85", "43.18"), ("69.85", "33.02"));
    probe.label_of_kind(LabelKind::Local, "SIG", ("69.85", "33.02"));
    probe.place("R", "R5", ("88.9", "46.99"), &["1", "2"]);
    probe.wire(("88.9", "43.18"), ("88.9", "33.02"));
    probe.write()
}

/// The same label text on two sheets, which is two nets of one name.
///
/// A local label names one net **per sheet**, so `SIG` on the root and `SIG`
/// on the child are two different conductors that answer to one word.
fn one_name_on_two_sheets(name: &str) -> PathBuf {
    let mut probe = Probe::new(name, scratch());
    let mut child = Probe::child_of(&probe);
    probe.sheet_of_size(CHILD, "child", ("139.7", "63.5"), ("25.4", "25.4"), &[]);
    probe.place("R", "R1", ("76.2", "54.61"), &["1", "2"]);
    probe.place("R", "R3", ("69.85", "46.99"), &["1", "2"]);
    probe.wire(("69.85", "43.18"), ("69.85", "33.02"));
    probe.label_of_kind(LabelKind::Local, "SIG", ("69.85", "33.02"));
    child.named_strand("RC1", "38.1", "41.91", "SIG");
    let sheet = probe.write_all(&[&child]);
    std::fs::write(sheet.with_extension("kicad_pro"), PROJECT)
        .expect("the project file is writable");
    sheet
}

/// The name kicli's extractor gives the net one pin is on.
fn net_handle_of(path: &Path, reference: &str, number: &str) -> String {
    net_name_of(path, reference, number)
}

/// The total cost of the route a run reported.
fn cost_of(run: &Connected) -> i64 {
    run.wire()["cost"]["total"]
        .as_i64()
        .unwrap_or_else(|| panic!("a routed answer carries a total cost: {}", run.object()))
}

/// The terminal a run says it reached.
fn reached(run: &Connected) -> String {
    run.wire()["to"]
        .as_str()
        .unwrap_or_else(|| panic!("every answer names the end it reached: {}", run.object()))
        .to_owned()
}

#[test]
fn connecting_to_a_net_takes_the_nearest_terminal() {
    let both = a_net_in_two_places("connect-net-two-places", true, true);
    let near_only = a_net_in_two_places("connect-net-near-only", true, false);
    let far_only = a_net_in_two_places("connect-net-far-only", false, true);
    let ask = ["--from-pin", "R1.1", "--to-net", "SIG"];

    let run = connect(&both, &ask);
    let near = connect(&near_only, &ask);
    let far = connect(&far_only, &ask);
    for (name, one) in [("both", &run), ("near", &near), ("far", &far)] {
        assert_eq!(one.wire()["status"], "routed", "{name}: {}", one.object());
    }

    // Both strands are live: each on its own is routed, so the drawing under
    // test really did offer the router two places to join the net.
    assert!(
        cost_of(&near) < cost_of(&far),
        "the two strands cost the same to reach, so nothing is being chosen: \
         near {}, far {}",
        cost_of(&near),
        cost_of(&far)
    );

    // The answer is the cheaper of the two, measured against the two controls
    // rather than against a number written here.
    assert_eq!(
        cost_of(&run),
        cost_of(&near),
        "the route to the net is not the route to the near strand: {}",
        run.object()
    );
    assert!(
        cost_of(&run) < cost_of(&far),
        "the route to the net costs what the far strand costs: {}",
        run.object()
    );
    assert_eq!(
        run.wire()["path"],
        near.wire()["path"],
        "the route to the net is not the route to the near strand: {}",
        run.object()
    );

    // And the answer says which point of the net it joined, rather than only
    // that it joined one.
    assert_eq!(
        reached(&run),
        reached(&near),
        "the answer names a different point than the near strand's own run"
    );
    assert!(
        reached(&run).starts_with("SIG@"),
        "the answer does not name the net and the point it joined: {}",
        reached(&run)
    );
    assert_ne!(
        reached(&run),
        reached(&far),
        "the answer names the far strand's point"
    );

    // The drawing that came out carries the join, which is the claim the
    // report is making about the file.
    let after = kicli_nets(&both);
    assert!(
        after.contains(&net(&["R1.1", "R3.1", "R4.1"])),
        "the route did not join R1.1 to the net: {after:?}"
    );
    assert_eq!(
        run.claimed_net(),
        Some("SIG"),
        "the claimed net is not the one that was asked for: {}",
        run.object()
    );
    if let Some(kicad) = oracle(&both) {
        assert!(
            kicad.contains(&net(&["R1.1", "R3.1", "R4.1"])),
            "KiCad does not report the join kicli made to the net: {kicad:?}"
        );
    }
}

#[test]
fn a_net_is_addressed_by_name_or_by_handle() {
    // By name. `SIG` is what the drawing calls the net, and it is what a view
    // prints for it.
    let by_name = a_named_net_and_an_unnamed_one("connect-net-by-name");
    let named = connect(&by_name, &["--from-pin", "R1.1", "--to-net", "SIG"]);
    assert_eq!(named.wire()["status"], "routed", "{}", named.object());
    assert!(
        reached(&named).starts_with("SIG@"),
        "the answer does not name the net it joined: {}",
        reached(&named)
    );
    assert!(
        kicli_nets(&by_name).contains(&net(&["R1.1", "R3.1"])),
        "the route did not join R1.1 to SIG"
    );

    // By handle. The drawing names R5's net nothing at all, so kicli gives it
    // a handle — and the handle is read from the drawing rather than written
    // here, because which handle a net gets depends on the whole drawing.
    let by_handle = a_named_net_and_an_unnamed_one("connect-net-by-handle");
    let handle = net_handle_of(&by_handle, "R5", "1");
    assert!(
        handle.starts_with("#n"),
        "the drawing names R5's net after all: {handle}"
    );
    let unnamed = connect(&by_handle, &["--from-pin", "R1.1", "--to-net", &handle]);
    assert_eq!(unnamed.wire()["status"], "routed", "{}", unnamed.object());
    assert!(
        reached(&unnamed).starts_with(&format!("{handle}@")),
        "the answer does not name the net it joined: {}",
        reached(&unnamed)
    );
    assert!(
        kicli_nets(&by_handle).contains(&net(&["R1.1", "R5.1"])),
        "the route did not join R1.1 to the unnamed net"
    );

    // A name two nets answer to is refused, with both of them listed. This is
    // the arm an agent meets: a local label names one net per sheet, so the
    // same word on two sheets is two conductors.
    let ambiguous = one_name_on_two_sheets("connect-net-ambiguous");
    let before = std::fs::read_to_string(&ambiguous).expect("the drawing reads");
    let refused = connect(&ambiguous, &["--from-pin", "R1.1", "--to-net", "SIG"]);
    assert_ne!(refused.code, 0, "an ambiguous name was not refused");
    assert!(
        refused.stderr.contains("R3.1") && refused.stderr.contains("RC1.1"),
        "the refusal does not list the candidates: {}",
        refused.stderr
    );
    assert!(
        refused.stderr.contains("2 nets"),
        "the refusal does not say how many nets answer to the name: {}",
        refused.stderr
    );
    assert_eq!(
        before,
        std::fs::read_to_string(&ambiguous).expect("the drawing reads"),
        "a refusal wrote to the file"
    );
}

/// A symbol that is nothing but a body, for a route to go around.
///
/// A route may not pass through a symbol body, and a body with no pin is the
/// simplest wall a probe can draw: it blocks and it joins nothing. The
/// rectangle is centred on the symbol's own position, so where the wall is
/// placed is where its middle goes.
fn wall(name: &'static str, half_width: &str, half_height: &str) -> String {
    symbol(
        name,
        "W",
        false,
        &[(
            "1_1",
            vec![rectangle(
                (&format!("-{half_width}"), &format!("-{half_height}")),
                (half_width, half_height),
            )],
        )],
    )
}

/// A source pin in a pocket, and one net drawn on both sides of the way out.
///
/// Three walls stand around R1.1 on three sides, so no silhouette reaches
/// anything: every route out turns at least four times, and the enumeration
/// offers at most three corners. What answers is the search.
///
/// The net is drawn twice. `SIG` above the pocket's lid is the **nearer** of
/// the two in a straight line and the dearer to reach, because a route to it
/// leaves the pocket downwards and climbs back over everything. `SIG` below
/// and to the right is further away and cheaper: the way out passes it.
fn a_net_on_both_sides_of_a_pocket(name: &str, over: bool, past: bool) -> PathBuf {
    let mut probe = Probe::new(name, scratch());
    probe.define(wall("LID", "6.35", "1.27"));
    probe.define(wall("JAMB", "0.635", "3.81"));
    probe.place("LID", "W1", ("76.2", "46.99"), &[]);
    probe.place("JAMB", "W2", ("69.85", "52.07"), &[]);
    probe.place("JAMB", "W3", ("82.55", "52.07"), &[]);

    probe.place("R", "R1", ("76.2", "54.61"), &["1", "2"]);

    // Over the lid: near in a straight line, and walled off.
    if over {
        probe.place("R", "R3", ("73.66", "39.37"), &["1", "2"]);
        probe.wire(("73.66", "43.18"), ("78.74", "43.18"));
        probe.label_of_kind(LabelKind::Local, "SIG", ("78.74", "43.18"));
    }

    // Past the way out: further away, and on the road.
    if past {
        probe.place("R", "R4", ("88.9", "64.77"), &["1", "2"]);
        probe.wire(("88.9", "60.96"), ("96.52", "60.96"));
        probe.label_of_kind(LabelKind::Local, "SIG", ("96.52", "60.96"));
    }
    probe.write()
}

/// The search is given every terminal of the net at once, not one at a time.
///
/// The pocket is what makes this measurable. No silhouette fits, so the answer
/// comes from A\*; and the terminal nearest in a straight line is not the one
/// the cheapest route reaches, so an estimate measured to one chosen terminal
/// answers differently from an estimate measured to the whole set.
#[test]
fn the_search_reaches_the_cheapest_terminal_of_the_net() {
    let both = a_net_on_both_sides_of_a_pocket("connect-net-pocket", true, true);
    let over_only = a_net_on_both_sides_of_a_pocket("connect-net-pocket-over", true, false);
    let past_only = a_net_on_both_sides_of_a_pocket("connect-net-pocket-past", false, true);
    let ask = ["--from-pin", "R1.1", "--to-net", "SIG"];

    let run = connect(&both, &ask);
    let over = connect(&over_only, &ask);
    let past = connect(&past_only, &ask);
    for (name, one) in [("both", &run), ("over", &over), ("past", &past)] {
        assert_eq!(one.wire()["status"], "routed", "{name}: {}", one.object());
        // Four corners at least, which is more than any silhouette draws: every
        // one of these routes came from the search rather than the fast path.
        assert!(
            one.wire()["corners"].as_u64().unwrap_or_default() >= 4,
            "{name}: a silhouette answered, so the search is not what is being \
             measured: {}",
            one.object()
        );
    }

    // The strand over the lid is the nearer of the two in a straight line, and
    // the dearer to reach. Both halves are measured rather than asserted.
    assert!(
        cost_of(&past) < cost_of(&over),
        "the two strands cost the same to reach, so nothing is being chosen: \
         over {}, past {}",
        cost_of(&over),
        cost_of(&past)
    );

    assert_eq!(
        cost_of(&run),
        cost_of(&past),
        "the search did not reach the cheapest terminal: {}",
        run.object()
    );
    assert_eq!(
        reached(&run),
        reached(&past),
        "the search reached a different terminal than the cheap strand's own run"
    );
    assert_ne!(
        reached(&run),
        reached(&over),
        "the search reached the near strand rather than the cheap one"
    );

    let after = kicli_nets(&both);
    assert!(
        after.contains(&net(&["R1.1", "R3.2", "R4.1"])),
        "the route did not join R1.1 to the net: {after:?}"
    );
}
