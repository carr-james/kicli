//! The whole loop, end to end: look, edit, verify.
//!
//! This is the only test that proves the milestone rather than a task. It does
//! what an agent does, through the compiled binary and nothing else: it reads a
//! view, places a symbol, moves that symbol's field, names a net with a label,
//! and reads the views back. Every step is checked against the JSON the command
//! returned, so the report and the file cannot drift apart.
//!
//! The oracle half runs `kicad-cli` and is off unless `KICLI_TEST_KICAD_CLI` is
//! set, so the default run needs no KiCad install.

use kicli_sexpr::Doc;
use serde_json::Value;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

/// A path no `kicad-cli` is at.
///
/// The commands under test never need KiCad. Pointing them at nothing keeps a
/// machine with KiCad installed giving the same answer as one without.
const NO_KICAD_CLI: &str = "/nonexistent/kicad-cli";

/// Where the new symbol goes: an empty part of the sheet, on the grid.
const PLACE_AT: &str = "190.5,101.6";

/// Where the new symbol's value goes. A field is exempt from the grid rule.
const FIELD_AT: &str = "193.04,104.14";

/// A point on the free stretch of the wire that carries `R12.2` and `R13.1`.
const LABEL_AT: &str = "30.48,88.9";

/// The pins of that wire's net, which the label joins.
const WIRE_PINS: [&str; 2] = ["R12.2", "R13.1"];

/// The project a run of the loop writes to.
struct Project {
    directory: PathBuf,
    root: PathBuf,
}

impl Project {
    /// Copy the committed fixture into a scratch directory of its own.
    fn new(name: &str) -> Self {
        let from = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sch/nets");
        let directory = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
        let _ = std::fs::remove_dir_all(&directory);
        std::fs::create_dir_all(&directory).expect("the scratch directory is made");
        for entry in std::fs::read_dir(&from).expect("the fixture reads") {
            let path = entry.expect("a directory entry reads").path();
            if path.is_file() {
                let file = path.file_name().expect("a file has a name");
                std::fs::copy(&path, directory.join(file)).expect("the copy is written");
            }
        }
        Self {
            root: directory.join("nets.kicad_sch"),
            directory,
        }
    }

    /// Run one kicli command against this project.
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
}

/// Bytes a command wrote, as text.
fn text(bytes: &[u8]) -> String {
    String::from_utf8(bytes.to_vec()).expect("kicli writes text")
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

/// The handles the delta of a mutation result names.
fn changed(result: &Value) -> Vec<(String, String)> {
    result["changed"]
        .as_array()
        .expect("the result lists what changed")
        .iter()
        .map(|line| {
            (
                line["change"].as_str().unwrap_or_default().to_owned(),
                line["handle"].as_str().unwrap_or_default().to_owned(),
            )
        })
        .collect()
}

#[test]
fn an_agent_can_look_edit_and_verify() {
    let project = Project::new("mutation_loop");

    // Look. The view is the data an agent acts on.
    let before = project.text(&["sch", "view"]);
    assert!(!before.contains("R900"), "the sheet has no R900 yet");
    assert!(!before.contains("N LOOP"), "and no net called LOOP");
    let source = std::fs::read_to_string(&project.root).expect("the root reads");

    // Edit, once. Place a symbol through the definition the project embeds.
    let placed = project.json(&[
        "sym",
        "place",
        "--lib-id",
        "Test:R",
        "--at",
        PLACE_AT,
        "--reference",
        "R900",
        "--value",
        "4k7",
    ]);
    invariants_passed(&placed);
    assert_eq!(
        changed(&placed),
        [("+".to_owned(), "R900".to_owned())],
        "the delta names the symbol and nothing else: {placed}"
    );
    assert_eq!(placed["symbol"]["reference"], "R900");
    assert_eq!(
        placed["symbol"]["sheet_paths"]
            .as_array()
            .expect("the placement names its sheet paths")
            .len(),
        1,
        "the root sheet is placed once"
    );
    assert_eq!(placed["reformatted"], false, "the fixture is canonical");
    assert!(
        placed["notes"]
            .as_array()
            .expect("the result carries its notes")
            .is_empty(),
        "the anchor was already on the grid: {placed}"
    );

    // Edit, twice. Move the new symbol's value away from its anchor.
    let moved = project.json(&["field", "move", "R900", "--name", "Value", "--to", FIELD_AT]);
    invariants_passed(&moved);
    assert_eq!(
        changed(&moved),
        [("~".to_owned(), "R900.Value".to_owned())],
        "the delta names the field and nothing else: {moved}"
    );
    assert_eq!(moved["field"]["name"], "Value");

    // Edit, three times. A label is a connectivity change, so the report says
    // which net it joined.
    let labelled = project.json(&["label", "add", "--text", "LOOP", "--at", LABEL_AT]);
    invariants_passed(&labelled);
    let lines = changed(&labelled);
    assert_eq!(lines.len(), 1, "one object was added: {labelled}");
    assert_eq!(lines[0].0, "+");
    let joined = &labelled["nets"]["after"]
        .as_array()
        .expect("the report names the nets after the write")[0];
    assert_eq!(joined["name"], "LOOP");
    assert_eq!(
        joined["pins"]
            .as_array()
            .expect("a net has pins")
            .iter()
            .map(|pin| pin.as_str().unwrap_or_default().to_owned())
            .collect::<Vec<String>>(),
        WIRE_PINS,
        "the label joined the wire's own net: {labelled}"
    );

    // Verify. The views say what the three deltas said they would.
    let after = project.text(&["sch", "view"]);
    assert!(
        after.contains("S R900 4k7 R"),
        "the connectivity view lists the new symbol: {after}"
    );
    // The record is `N <handle>[=<kicad name>]: <pins>`, so the name is a
    // prefix and the pins are the tail.
    assert!(
        after
            .lines()
            .any(|line| line.starts_with("N LOOP") && line.ends_with(&WIRE_PINS.join(" "))),
        "and the net the label made: {after}"
    );

    let layout = project.text(&["sch", "view", "--view", "layout"]);
    assert!(
        layout.contains("L R900 190.50 101.60 0 -"),
        "the layout digest draws it where it was asked for: {layout}"
    );
    // An `F` record carries the offset from the anchor, and only for a field
    // that has moved off the position its library gives it.
    assert!(
        layout.contains("F R900.Value 2.54 2.54"),
        "with the field the offset from the anchor it was moved to: {layout}"
    );

    // The round-trip properties still hold on the file three writes produced.
    let written = std::fs::read_to_string(&project.root).expect("the root reads");
    assert_ne!(written, source, "the loop changed the file");
    let doc = Doc::parse(&written).expect("the written file parses");
    assert!(doc.is_canonical(), "kicli wrote KiCad's own layout");
    assert_eq!(doc.emit(), written, "and it emits its own bytes again");
    let again = Doc::parse(&doc.emit()).expect("the emitted bytes parse");
    assert!(
        doc.structurally_eq(&again),
        "the tree survives an emit and a re-parse"
    );
}

#[test]
fn kicad_reads_what_the_loop_wrote() {
    let Some(tool) = kicad_cli() else {
        eprintln!("skipped: set KICLI_TEST_KICAD_CLI to run kicad-cli");
        return;
    };
    let project = Project::new("mutation_loop_oracle");

    // What KiCad says about the drawing before kicli touches it. The control
    // for the comparison below: a report parser that read nothing would make
    // "nothing else moved" true and meaningless.
    let before = rule_check(&tool, &project, "before.txt");
    assert!(
        !before.is_empty(),
        "the rule check reported items on the fixture as it stands"
    );

    project.json(&[
        "sym",
        "place",
        "--lib-id",
        "Test:R",
        "--at",
        PLACE_AT,
        "--reference",
        "R900",
        "--value",
        "4k7",
    ]);
    project.json(&["field", "move", "R900", "--name", "Value", "--to", FIELD_AT]);
    project.json(&["label", "add", "--text", "LOOP", "--at", LABEL_AT]);

    // KiCad reads the file kicli wrote, and reports the new symbol's pins.
    let after = rule_check(&tool, &project, "after.txt");
    assert!(
        after.iter().any(|item| item.contains("Symbol R900 Pin")),
        "the rule check found the placed symbol: {after:?}"
    );

    // And nothing else moved. Every item that does not name the new symbol is
    // exactly the item the drawing had before the loop ran.
    let mine = |items: &BTreeSet<String>| -> BTreeSet<String> {
        items
            .iter()
            .filter(|item| !item.contains("R900"))
            .cloned()
            .collect()
    };
    assert_eq!(
        mine(&after),
        mine(&before),
        "the loop raised no fault of its own"
    );

    // KiCad's netlist carries the net the label report claimed.
    let netlist = export_netlist(&tool, &project).expect("kicad-cli exported a netlist");
    let named: Vec<Vec<String>> = kicad_nets(&netlist)
        .into_iter()
        .filter(|(name, _)| name.ends_with("LOOP"))
        .map(|(_, pins)| pins)
        .collect();
    assert_eq!(named.len(), 1, "one net is called LOOP: {netlist}");
    assert_eq!(named[0], WIRE_PINS, "with the pins the report named");
}

/// The `kicad-cli` binary, when the environment asks for the live tests.
fn kicad_cli() -> Option<String> {
    std::env::var("KICLI_TEST_KICAD_CLI").ok()?;
    Some(std::env::var("KICLI_KICAD_CLI").unwrap_or_else(|_| "kicad-cli".to_owned()))
}

/// Run KiCad's own rule check, and read the items it reported.
///
/// An item line reads `@(25.40 mm, 21.59 mm): Symbol R1 Pin 1 [Passive, Line]`.
/// The counts and headings around them move whenever anything is added, so the
/// items are what a comparison can be made of.
fn rule_check(tool: &str, project: &Project, into: &str) -> BTreeSet<String> {
    let report = project.directory.join(into);
    let status = Command::new(tool)
        .current_dir(&project.directory)
        .args([
            "sch",
            "erc",
            "--format",
            "report",
            "--units",
            "mm",
            "--severity-all",
            "-o",
        ])
        .arg(&report)
        .arg(project.root.file_name().expect("the root has a name"))
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .expect("kicad-cli runs");
    assert!(status.success(), "KiCad read the file kicli wrote");

    std::fs::read_to_string(&report)
        .expect("the report reads")
        .lines()
        .map(str::trim)
        .filter(|line| line.starts_with("@("))
        .map(str::to_owned)
        .collect()
}

/// Export KiCad's own netlist of the project.
fn export_netlist(tool: &str, project: &Project) -> Option<String> {
    let into = project.directory.join("after.net");
    let status = Command::new(tool)
        .args(["sch", "export", "netlist", "-o"])
        .arg(&into)
        .arg(&project.root)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .ok()?;
    status
        .success()
        .then(|| std::fs::read_to_string(&into).ok())?
}

/// The nets KiCad reports, as a name and a sorted pin list each.
fn kicad_nets(text: &str) -> Vec<(String, Vec<String>)> {
    let doc = Doc::parse(text).expect("the netlist parses");
    let root = doc.root().expect("the netlist has a root list");
    let mut found = Vec::new();
    for &child in doc.children(root) {
        if !doc.head_is(child, "nets") {
            continue;
        }
        for &net in doc.children(child) {
            if !doc.head_is(net, "net") {
                continue;
            }
            let name = atom_of(&doc, net, "name").unwrap_or_default();
            let mut pins: Vec<String> = doc
                .children(net)
                .iter()
                .filter(|&&node| doc.head_is(node, "node"))
                .filter_map(|&node| {
                    Some(format!(
                        "{}.{}",
                        atom_of(&doc, node, "ref")?,
                        atom_of(&doc, node, "pin")?
                    ))
                })
                .collect();
            pins.sort();
            found.push((name, pins));
        }
    }
    assert!(!found.is_empty(), "the netlist reported no nets at all");
    found
}

/// The first value of a named child list.
fn atom_of(doc: &Doc, node: kicli_sexpr::NodeId, head: &str) -> Option<String> {
    let list = doc
        .children(node)
        .iter()
        .copied()
        .find(|&child| doc.head_is(child, head))?;
    doc.children(list)
        .get(1)
        .and_then(|&id| doc.atom_as_str(id))
}
