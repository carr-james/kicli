//! The whole wiring loop, end to end, against KiCad.
//!
//! Every other check in this milestone proves one mechanism. This one does
//! what an agent does with wires, in one sequence, through the compiled binary
//! and nothing else: it **reads a view**, **connects two pins**, **connects a
//! pin to a net**, **draws one explicit polyline**, and then **asks KiCad what
//! it made**. Four writes, then the oracle.
//!
//! # The four clauses, and why each one is load-bearing
//!
//! *The netlist carries exactly the partition kicli reported.* Not "the pins
//! are connected": **exactly the partition, net for net and pin for pin, both
//! directions**. A write that joins what was asked and also fuses something
//! else satisfies the weaker reading and fails this one. It is asserted twice,
//! at two levels — the view an agent reads, and the extractor the commands
//! answer from — and the expectation for the view is derived from **KiCad's
//! own netlist** rather than from kicli, so the control does not share an
//! ancestor with the thing it checks.
//!
//! *The before-partition is non-empty.* The anti-vacuity control. A loop test
//! that compares an empty partition against an empty partition passes while
//! proving nothing, so the drawing is asserted to carry connectivity before
//! any command runs, and to **not** carry the joins the loop is about to make.
//!
//! *KiCad's rule check reports no fault that does not name the wires just
//! drawn.* Not "no faults" — this drawing has pre-existing ones, and so does
//! any real sheet. No **new** fault, and every fault that moves in either
//! direction names a symbol one of the four writes touched.
//!
//! *P1 and P2 hold on the file the four writes produced.* Constitution §1, on
//! the end state rather than on each step.
//!
//! # What is deliberately not here
//!
//! No identifier is quoted. Identifiers are a SHA-256 of a seed built from the
//! request and the document it lands in, so an assertion that named one would
//! have to be rewritten whenever anything about either moved. Counts and
//! positions are asserted instead. Until the identifier-seed chore (C8) the
//! seed carried the document's **absolute path** as well, and a quoted
//! identifier would have passed in the worktree it was written in and failed
//! everywhere else — the sharper version of the same reason.
//!
//! The oracle half runs only with `KICLI_TEST_KICAD_CLI` set, so the default
//! run needs no KiCad install — and [`the_oracle_says_when_it_did_not_run`]
//! makes a silent run visible rather than letting it read as a passing one.

use kicli::connectivity::extract;
use kicli::model::Hierarchy;
use kicli_probe::Probe;
use kicli_probe::drawing::LabelKind;
use kicli_probe::oracle::{
    Change, Kicad, NamedNet, Netlist, Partition, differences, kicli_partition, net, skipped,
};
use kicli_sexpr::Doc;
use serde_json::Value;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

/// Where this binary writes the drawings it builds.
fn scratch() -> PathBuf {
    Path::new(env!("CARGO_TARGET_TMPDIR")).join("wire-loop")
}

/// A path no `kicad-cli` is at.
///
/// Every run of the binary points discovery at it, so a machine with KiCad
/// installed gives the same answer as one without. The oracle below runs
/// `kicad-cli` itself, on the file the binary left behind.
const NO_KICAD_CLI: &str = "/nonexistent/kicad-cli";

/// Everything the four writes name: the four symbols, and the net.
///
/// KiCad's rule check names symbols, pins and labels rather than wire
/// identifiers, so this is the form "the wires just drawn" takes in a report.
/// A finding about `R1` is about a wire this loop drew; one about `R5` is not.
///
/// **`SIG` is here because the loop measured that it had to be.** The second
/// write joins a second pin to `SIG`, and KiCad's answer to that is to stop
/// reporting `isolated_pin_label` about `SIG`'s **label** — an item that names
/// no symbol at all. A clause that recognised only symbols would have read a
/// direct consequence of the write as a fault about something the loop never
/// touched.
const TOUCHED: [&str; 5] = ["R1", "R2", "R3", "R4", "SIG"];

/// The sheet a run of the loop writes to.
struct Project {
    directory: PathBuf,
    root: PathBuf,
}

impl Project {
    /// Build the loop's drawing through the probe harness.
    fn new(name: &str) -> Self {
        let root = the_sheet_an_agent_meets(name);
        Self {
            directory: root
                .parent()
                .expect("the drawing sits in a directory")
                .to_path_buf(),
            root,
        }
    }

    /// Run one kicli command against this sheet.
    fn run(&self, arguments: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_kicli"))
            .args(arguments)
            .args(["--project", self.directory.to_str().expect("a text path")])
            .env("KICLI_KICAD_CLI", NO_KICAD_CLI)
            .output()
            .expect("the binary runs")
    }

    /// Run one command and read its JSON result.
    fn json(&self, arguments: &[&str]) -> Value {
        let mut arguments = arguments.to_vec();
        arguments.extend(["--output", "json"]);
        let run = self.run(&arguments);
        assert_eq!(
            run.status.code(),
            Some(0),
            "{arguments:?}: {}",
            text(&run.stderr)
        );
        serde_json::from_str(&text(&run.stdout)).expect("the result is one JSON object")
    }

    /// Run one command and read its text result.
    fn text(&self, arguments: &[&str]) -> String {
        let run = self.run(arguments);
        assert_eq!(
            run.status.code(),
            Some(0),
            "{arguments:?}: {}",
            text(&run.stderr)
        );
        text(&run.stdout)
    }

    /// The partition kicli's extractor reads out of the written file.
    fn extracted(&self) -> Partition {
        let hierarchy = Hierarchy::load(&self.root).expect("the written drawing loads");
        kicli_partition(&extract(&hierarchy))
    }

    /// The net one pin is on, as kicli's extractor names it.
    fn net_of(&self, reference: &str, number: &str) -> String {
        let hierarchy = Hierarchy::load(&self.root).expect("the written drawing loads");
        extract(&hierarchy)
            .net_of(reference, number)
            .unwrap_or_else(|| panic!("{reference}.{number} is on a net"))
            .name
            .clone()
    }
}

/// Bytes a command wrote, as text.
fn text(bytes: &[u8]) -> String {
    String::from_utf8(bytes.to_vec()).expect("kicli writes text")
}

/// The sheet the loop starts from, built through the probe harness.
///
/// A hand-built fixture encodes the same assumptions as the code that reads
/// it, so the drawing is built rather than committed.
///
/// Six resistors, and nothing else. **No power symbol and no child sheet**,
/// because `kicli_partition` drops a power pin and a netlist does not, and a
/// clause that says "exactly" cannot afford a normalisation it did not need.
///
/// ```text
///                      R5.1 (69.85, 30.48)
///   R1.1 ------------- | ------------- R2.1        SIG ---------------- R4.1
///  (50.8, 50.8)        |         (88.9, 50.8)   (114.3, 38.1)     (139.7, 38.1)
///   R1.2               |              R2.2                 R3.1
///                      |                              (127, 50.8)
///                      R6.1 (69.85, 63.5)
/// ```
///
/// **`R5.1`–`R6.1` is the control that makes "no other net changed" a live
/// claim.** The wire at `x = 69.85` runs between the two pins the first write
/// joins, so the route has to **cross** it — and a crossing is allowed, costed,
/// and emphatically not a connection. A drawing with nothing to cross would let
/// that control pass by having nothing to say.
///
/// `SIG` is the net the second write joins, drawn as a horizontal wire with
/// `R4.1` at one end and its label at the other. `R3.1` stands under its
/// **middle**, so the nearest point of the net is the wire's **interior** — and
/// an interior terminus is where a junction is what makes the connection.
fn the_sheet_an_agent_meets(name: &str) -> PathBuf {
    let mut probe = Probe::new(name, scratch());

    // The two pins the first write joins. A resistor's pin 1 is above its
    // anchor and its body below, so both are left upwards.
    probe.place("R", "R1", ("50.8", "54.61"), &["1", "2"]);
    probe.place("R", "R2", ("88.9", "54.61"), &["1", "2"]);

    // The net in the way, which the route between them has to cross.
    probe.place("R", "R5", ("69.85", "34.29"), &["1", "2"]);
    probe.place("R", "R6", ("69.85", "67.31"), &["1", "2"]);
    probe.wire(("69.85", "30.48"), ("69.85", "63.5"));

    // The named net the second write joins, and the pin that joins it.
    probe.place("R", "R3", ("127", "54.61"), &["1", "2"]);
    probe.place("R", "R4", ("139.7", "41.91"), &["1", "2"]);
    probe.wire(("114.3", "38.1"), ("139.7", "38.1"));
    probe.label_of_kind(LabelKind::Local, "SIG", ("114.3", "38.1"));

    probe.write()
}

/// The nets a connectivity view lists, as one sorted pin list per net.
///
/// The record is `N <name>[=<kicad name>]: <pins>`, so the pins are the tail.
fn view_partition(view: &str) -> Partition {
    view.lines()
        .filter_map(|line| line.strip_prefix("N "))
        .filter_map(|line| line.split_once(": "))
        .map(|(_, pins)| {
            let mut sorted: Vec<String> = pins.split_whitespace().map(str::to_owned).collect();
            sorted.sort();
            sorted
        })
        .collect()
}

/// KiCad's partition, as the connectivity view reports the same drawing.
///
/// **The expectation is derived from KiCad's own netlist, not from kicli.** The
/// view drops the one-pin nets that join nothing — `sch erc` lists them, and
/// carrying them here would cost a fifth of the view to say nothing — and KiCad
/// is the authority on which nets those are, because it is KiCad that calls
/// them `unconnected-…`. So the filter reads KiCad's names rather than
/// restating kicli's rule.
fn as_a_view_would_list_it(netlist: &Netlist) -> Partition {
    netlist
        .nets()
        .iter()
        .filter(|net| !net.pins.is_empty())
        .filter(|net| !(net.pins.len() == 1 && net.name.starts_with("unconnected-")))
        .map(|net: &NamedNet| net.pins.clone())
        .collect()
}

/// The route contract a `wire` verb answered in.
fn wire(result: &Value) -> &Value {
    &result["wire"]
}

/// The net a `wire connect` claims it joined, as the route contract holds it.
///
/// **Presence before value.** The contract carries the key at every status, so
/// a run that dropped it would be a contract break — and `as_str()` cannot
/// tell an absent key from a null one, so the key is asked for first.
fn joined_net(result: &Value) -> Option<&str> {
    let contract = wire(result).as_object().expect("the contract is an object");
    assert!(
        contract.contains_key("joined_net"),
        "the contract dropped the joined net instead of nulling it: {result}"
    );
    contract["joined_net"].as_str()
}

/// The identifiers of the junctions one run wrote.
fn junctions(result: &Value) -> Vec<String> {
    wire(result)["added"]["junctions"]
        .as_array()
        .expect("the key is always there")
        .iter()
        .map(|value| value.as_str().unwrap_or_default().to_owned())
        .collect()
}

/// Every invariant of a mutation result passed, and all four ran.
fn invariants_passed(result: &Value) {
    let checks = result["invariants"]
        .as_array()
        .expect("the result lists the invariants");
    assert_eq!(checks.len(), 4, "all four ran: {result}");
    for check in checks {
        assert_eq!(
            check["passed"],
            true,
            "{} passed: {result}",
            check["name"].as_str().unwrap_or("?")
        );
    }
}

/// Does this rule-check item name something one of the four writes touched?
///
/// An item that names none of them — a dangling wire endpoint, say, which
/// names no symbol and no label — is a fault about something the loop did not
/// touch, and that is what the clause forbids.
///
/// The trailing space is a boundary: `Symbol R1 Pin 2` and `Symbol R1 [R]` are
/// both `R1`, and `Symbol R10 Pin 1` is not.
fn names_a_wire_the_loop_drew(item: &str) -> bool {
    TOUCHED.iter().any(|name| {
        item.contains(&format!("Symbol {name} ")) || item.contains(&format!("Label '{name}'"))
    })
}

/// KiCad's own layout of the file kicli wrote, measured rather than assumed.
///
/// `kicad-cli sch upgrade --force` re-saves a schematic the way KiCad saves
/// one, on a **copy**, so the file under test is left as kicli wrote it.
///
/// **This is not byte-identity and cannot be.** KiCad reorders the items and
/// renames the project, which is why SPEC's P4 — "what KiCad would write" — is
/// informational rather than a gate. What the re-save is good for is the
/// **layout**: the prefixes KiCad indents its lines with. Those come from the
/// tool in this run rather than from a constant written here, and — the point —
/// they do not come from kicli's own prettifier.
///
/// # Panics
///
/// If the copy cannot be made, or if `kicad-cli` will not re-save it.
fn as_kicad_would_lay_it_out(written: &Path, into: &Path) -> String {
    std::fs::create_dir_all(into).expect("the scratch directory is writable");
    let copy = into.join("resaved.kicad_sch");
    std::fs::copy(written, &copy).expect("the written file copies");
    let binary = std::env::var("KICLI_KICAD_CLI").unwrap_or_else(|_| "kicad-cli".to_owned());
    let status = Command::new(binary)
        .args(["sch", "upgrade", "--force"])
        .arg(&copy)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .expect("kicad-cli runs");
    assert!(status.success(), "kicad-cli re-saved the file kicli wrote");
    std::fs::read_to_string(&copy).expect("the re-saved file reads")
}

/// The distinct indentations a file's lines carry.
///
/// A set rather than a count: KiCad's re-save moves one record to a depth of
/// its own, and the claim here is about the **alphabet and the depths** a
/// layout uses, not about how many lines sit at each.
fn indentations(text: &str) -> BTreeSet<String> {
    text.lines()
        .map(|line| {
            line.chars()
                .take_while(|character| character.is_whitespace())
                .collect()
        })
        .collect()
}

/// The four nets the loop is about, as they stand after the four writes.
fn the_nets_the_loop_makes() -> [Vec<String>; 4] {
    [
        net(&["R1.1", "R2.1"]),
        net(&["R3.1", "R4.1"]),
        net(&["R1.2", "R2.2"]),
        // The control: untouched, and it is in the way of the first route.
        net(&["R5.1", "R6.1"]),
    ]
}

#[test]
fn an_agent_wires_a_sheet_and_kicad_agrees() {
    let project = Project::new("wire-loop");
    let kicad = Kicad::found();

    // ---- 1. Read a view. This is the data an agent acts on. ----------------
    let before = project.text(&["sch", "view"]);
    let before_partition = view_partition(&before);

    // **The anti-vacuity control.** A loop over an empty drawing would compare
    // nothing against nothing and pass.
    assert!(
        !before_partition.is_empty(),
        "the drawing carries no connectivity before the loop runs, so every \
         comparison below would say nothing: {before}"
    );
    assert!(
        before_partition.contains(&net(&["R5.1", "R6.1"])),
        "the net the first route has to cross is not there: {before}"
    );
    assert!(
        before_partition.contains(&net(&["R4.1"])),
        "SIG is not there for the second write to join: {before}"
    );
    // And the joins the loop is about to make are not already made.
    for joined in [
        net(&["R1.1", "R2.1"]),
        net(&["R3.1", "R4.1"]),
        net(&["R1.2", "R2.2"]),
    ] {
        assert!(
            !before_partition.contains(&joined),
            "{joined:?} is joined before the loop ran: {before}"
        );
    }

    // What KiCad says about the drawing before kicli touches it. The rule
    // check's before-reading is the other anti-vacuity control, and `Change`
    // refuses an empty one.
    let faults_before = kicad
        .as_ref()
        .map(|tool| tool.rule_check_into(&project.root, &project.directory.join("before.txt")))
        .map(|report| report.items());
    let netlist_before = kicad
        .as_ref()
        .map(|tool| tool.netlist(&project.root, &project.directory.join("before.net")));
    if let Some(netlist) = &netlist_before {
        assert_eq!(
            before_partition,
            as_a_view_would_list_it(netlist),
            "the view and KiCad disagree about the drawing before any write"
        );
    }

    let source = std::fs::read_to_string(&project.root).expect("the root reads");

    // ---- 2. Connect two pins. ---------------------------------------------
    let pins = project.json(&["wire", "connect", "--from-pin", "R1.1", "--to-pin", "R2.1"]);
    invariants_passed(&pins);
    assert_eq!(wire(&pins)["status"], "routed", "{pins}");
    assert!(
        wire(&pins)["segments"].as_u64().unwrap_or_default() > 0,
        "a routed answer draws a wire: {pins}"
    );
    // The route crosses the net in the way, and a crossing is not a connection.
    assert_eq!(
        wire(&pins)["crossings"].as_array().map(Vec::len),
        Some(1),
        "the route did not cross the net between the two pins: {pins}"
    );
    assert!(
        junctions(&pins).is_empty(),
        "a crossing gets no dot: {pins}"
    );
    let named = project.net_of("R1", "1");
    assert_eq!(
        joined_net(&pins),
        Some(named.as_str()),
        "the net the command claimed is not the net the file holds: {pins}"
    );

    // ---- 3. Connect a pin to a net. ---------------------------------------
    let to_net = project.json(&["wire", "connect", "--from-pin", "R3.1", "--to-net", "SIG"]);
    invariants_passed(&to_net);
    assert_eq!(wire(&to_net)["status"], "routed", "{to_net}");
    // The nearest point of SIG is the interior of its wire, and an interior
    // terminus is joined to nothing until a junction says so.
    assert_eq!(
        junctions(&to_net).len(),
        1,
        "the route to the net wrote no junction, so it did not end on the \
         interior of the net's wire: {to_net}"
    );
    assert!(
        wire(&to_net)["to"]
            .as_str()
            .unwrap_or_default()
            .starts_with("SIG@"),
        "the answer does not say which point of SIG it joined: {to_net}"
    );
    assert_eq!(
        joined_net(&to_net),
        Some("SIG"),
        "the connection did not land on the net it was asked for: {to_net}"
    );

    // ---- 4. Draw one explicit polyline. -----------------------------------
    // kicli does no searching here: the corners are the caller's. Pin 2 of a
    // resistor is below its body, so both ends are left downwards.
    let drawn = project.json(&[
        "wire",
        "draw",
        "--from-pin",
        "R1.2",
        "--via",
        "50.8,76.2",
        "--via",
        "88.9,76.2",
        "--to-pin",
        "R2.2",
    ]);
    invariants_passed(&drawn);
    assert_eq!(wire(&drawn)["status"], "routed", "{drawn}");
    assert_eq!(
        wire(&drawn)["path"],
        serde_json::json!([[50.8, 58.42], [50.8, 76.2], [88.9, 76.2], [88.9, 58.42]]),
        "the polyline did not go through the corners it was given: {drawn}"
    );
    assert_eq!(
        wire(&drawn)["segments"],
        3,
        "one segment per leg of the polyline: {drawn}"
    );

    // ---- 5. Ask what was made. --------------------------------------------
    let written = std::fs::read_to_string(&project.root).expect("the root reads");
    assert_ne!(written, source, "the loop changed the file");

    let after = project.text(&["sch", "view"]);
    let after_partition = view_partition(&after);
    for joined in the_nets_the_loop_makes() {
        assert!(
            after_partition.contains(&joined),
            "the view does not carry {joined:?} after the loop: {after}"
        );
    }
    // The three writes are the only nets that moved. This is the "exactly"
    // clause at the view's level, and it is what a write that fused half the
    // sheet would fail.
    let moved: Vec<&Vec<String>> = after_partition
        .symmetric_difference(&before_partition)
        .collect();
    assert_eq!(
        moved.len(),
        4,
        "the loop moved more than the three nets it made: {moved:?}"
    );

    // kicli's extractor is asked about the file the commands wrote, rather
    // than about the arithmetic that produced it.
    let extracted = project.extracted();
    for joined in the_nets_the_loop_makes() {
        assert!(
            extracted.contains(&joined),
            "the written file does not hold {joined:?}: {extracted:?}"
        );
    }

    // **The oracle.** KiCad's netlist of the file kicli wrote carries exactly
    // the partition kicli reported — net for net and pin for pin, in both
    // directions.
    if let (Some(tool), Some(faults_before)) = (&kicad, faults_before) {
        let netlist = tool.netlist(&project.root, &project.directory.join("after.net"));
        let kicad_partition = netlist.partition();

        // The rule check first, because a fault it reports is the thing this
        // loop would most like to have caught, and an assertion that never
        // runs is one that never fails.
        let faults = tool.rule_check_into(&project.root, &project.directory.join("after.txt"));
        let change = Change::measured(faults_before, faults.items());
        let new_faults: Vec<String> = change
            .added()
            .iter()
            .filter(|item| !names_a_wire_the_loop_drew(item))
            .cloned()
            .collect();
        assert!(
            new_faults.is_empty(),
            "KiCad reports a fault that names none of the wires the loop drew: \
             {new_faults:?}"
        );
        // And the faults that went away are the ones about the pins the loop
        // connected. A write that silenced a finding elsewhere is as much a
        // change as one that raised it.
        let silenced: Vec<String> = change
            .removed()
            .iter()
            .filter(|item| !names_a_wire_the_loop_drew(item))
            .cloned()
            .collect();
        assert!(
            silenced.is_empty(),
            "the loop silenced a fault about something it never touched: {silenced:?}"
        );

        // The exactness clause, at the extractor's level: every net, both ways.
        assert!(
            differences(&extracted, &kicad_partition).is_none(),
            "kicli and KiCad do not report the same partition of the written \
             file:\n{}",
            differences(&extracted, &kicad_partition).unwrap_or_default()
        );
        // And at the view's level, against an expectation derived from KiCad's
        // own netlist rather than from kicli's rule.
        assert_eq!(
            after_partition,
            as_a_view_would_list_it(&netlist),
            "the view an agent reads is not the netlist KiCad exports"
        );
        for joined in the_nets_the_loop_makes() {
            assert!(
                kicad_partition.contains(&joined),
                "KiCad does not carry {joined:?}: {kicad_partition:?}"
            );
        }

        // **P1's layout half, against a control KiCad supplied.**
        // `doc.is_canonical()` below is kicli's opinion of KiCad's layout, and
        // `doc.emit()` produces that layout — both through one prettifier, so a
        // break in it moves the claim and the control together and neither
        // notices. This clause asks the tool instead: the indentations kicli
        // wrote are the indentations KiCad writes.
        let resaved = as_kicad_would_lay_it_out(&project.root, &project.directory.join("resaved"));
        let kicad_layout = indentations(&resaved);
        assert!(
            kicad_layout.len() > 1,
            "the control read no indentation at all, so it would agree with \
             anything"
        );
        assert_eq!(
            indentations(&written),
            kicad_layout,
            "kicli does not indent a file the way KiCad indents one"
        );
    }

    // ---- P1 and P2, on the file the four writes produced. ------------------
    let doc = Doc::parse(&written).expect("the written file parses");
    assert!(doc.is_canonical(), "kicli wrote KiCad's own layout");
    assert_eq!(doc.emit(), written, "and it emits its own bytes again");
    let again = Doc::parse(&doc.emit()).expect("the emitted bytes parse");
    assert!(
        doc.structurally_eq(&again),
        "the tree survives an emit and a re-parse"
    );
}

/// A skipped oracle says so, so a silent run is not read as a passing one.
#[test]
fn the_oracle_says_when_it_did_not_run() {
    if Kicad::found().is_none() {
        skipped("ask KiCad about the file the whole loop wrote");
    }
}
