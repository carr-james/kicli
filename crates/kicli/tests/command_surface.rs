//! The command surface: flags, help, exit codes, and the shape of an error.
//!
//! These tests run the compiled binary, because the exit code and the split
//! between stdout and stderr are part of the contract, and neither is visible
//! from inside the library.

use clap::CommandFactory;
use kicli::cli::{Cli, ExitCode};
use kicli::route::Status;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use kicli_probe::Probe;
use kicli_probe::scratch::Fixtures;

/// The committed fixtures this binary reads, and the scratch it writes in.
fn fixtures() -> Fixtures {
    Fixtures::new(env!("CARGO_TARGET_TMPDIR"), env!("CARGO_MANIFEST_DIR"))
}

/// A path no `kicad-cli` is at.
///
/// Every test here points discovery at it, so a run never starts KiCad and a
/// machine with KiCad installed gives the same answer as one without.
const NO_KICAD_CLI: &str = "/nonexistent/kicad-cli";

/// Run the binary with the given arguments.
fn kicli(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_kicli"))
        .args(args)
        .env("KICLI_KICAD_CLI", NO_KICAD_CLI)
        .output()
        .expect("the binary runs")
}

/// The exit code of a run.
fn code(run: &Output) -> i32 {
    run.status.code().expect("the run ended by itself")
}

/// Everything a run wrote to standard output.
fn stdout(run: &Output) -> String {
    String::from_utf8(run.stdout.clone()).expect("stdout is text")
}

/// Everything a run wrote to standard error.
fn stderr(run: &Output) -> String {
    String::from_utf8(run.stderr.clone()).expect("stderr is text")
}

#[test]
fn version_prints_and_succeeds() {
    let run = kicli(&["--version"]);
    assert_eq!(code(&run), 0);
    assert!(
        stdout(&run).contains(kicli::version()),
        "the version is on stdout: {:?}",
        stdout(&run)
    );
    assert!(stderr(&run).is_empty(), "nothing goes to stderr");
}

#[test]
fn an_unknown_verb_is_a_usage_error_on_stderr() {
    let run = kicli(&["project", "explode"]);
    assert_eq!(code(&run), 2, "usage errors exit 2");
    assert!(stdout(&run).is_empty(), "stdout stays clean");
    let message = stderr(&run);
    assert!(
        message.contains("explode"),
        "the error names the verb: {message:?}"
    );
    assert!(
        message.to_lowercase().contains("usage"),
        "the error carries usage: {message:?}"
    );
}

#[test]
fn an_unknown_noun_is_a_usage_error() {
    let run = kicli(&["schematic", "view"]);
    assert_eq!(code(&run), 2);
    assert!(stdout(&run).is_empty());
}

#[test]
fn a_usage_error_in_json_is_one_object_on_stderr() {
    for arguments in [
        vec!["--output", "json", "project", "explode"],
        vec!["--output=json", "project", "explode"],
        vec!["project", "explode", "--output", "json"],
    ] {
        let run = kicli(&arguments);
        assert_eq!(code(&run), 2, "{arguments:?} exits 2");
        assert!(stdout(&run).is_empty(), "{arguments:?} keeps stdout clean");

        let reported: serde_json::Value =
            serde_json::from_str(&stderr(&run)).expect("stderr is one JSON object");
        assert_eq!(reported["error"]["kind"], "usage");
        assert_eq!(reported["error"]["exit_code"], 2);
        assert!(
            reported["error"]["message"]
                .as_str()
                .expect("the message is text")
                .contains("explode"),
            "the JSON error names the verb: {reported}"
        );
    }
}

#[test]
fn help_lists_the_global_flags_and_hides_the_variant_flag() {
    let run = kicli(&["--help"]);
    assert_eq!(code(&run), 0, "help succeeds");
    let help = stdout(&run);
    for flag in ["--output", "--project", "--sheet", "--quiet", "--version"] {
        assert!(help.contains(flag), "{flag} is documented: {help}");
    }
    assert!(
        !help.contains("--variant"),
        "the variant flag is hidden: {help}"
    );
}

#[test]
fn the_variant_flag_is_accepted_and_says_it_does_nothing() {
    let project = fixtures().fixture("project/healthy");
    let project = project.to_str().expect("the path is text");

    let run = kicli(&["project", "info", "--variant", "assembled", "-p", project]);
    assert_ne!(
        code(&run),
        2,
        "the flag is not a usage error: {}",
        stderr(&run)
    );
    assert!(
        stderr(&run).contains("--variant"),
        "the run says the flag has no effect: {:?}",
        stderr(&run)
    );

    let quiet = kicli(&[
        "project",
        "info",
        "--quiet",
        "--variant",
        "assembled",
        "-p",
        project,
    ]);
    assert_ne!(code(&quiet), 2);
    assert!(
        !stderr(&quiet).contains("--variant"),
        "--quiet suppresses the note"
    );
}

/// Every code in the table has exactly one name, and every name one code.
#[test]
fn every_exit_code_has_exactly_one_name() {
    let table = ExitCode::ALL;
    assert_eq!(table.len(), 7, "the table has seven rows");

    let mut numbers: Vec<u8> = table.iter().map(|entry| entry.code()).collect();
    numbers.sort_unstable();
    assert_eq!(numbers, [0, 1, 2, 3, 4, 5, 6], "the codes are 0 to 6");

    let mut names: Vec<&str> = table.iter().map(|entry| entry.name()).collect();
    names.sort_unstable();
    names.dedup();
    assert_eq!(names.len(), table.len(), "no two rows share a name");

    for entry in table {
        assert!(
            !entry.meaning().is_empty(),
            "{} states its meaning",
            entry.name()
        );
    }
}

/// The table is the only place a kicli exit code is a number.
///
/// The failure this guards against is a second site that knows an integer: a
/// `process::exit(4)` somewhere else makes the table advisory rather than true.
#[test]
fn only_the_table_turns_an_exit_code_into_a_number() {
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let table = source_root.join("cli/exit.rs");

    for file in rust_sources(&source_root) {
        if file == table {
            continue;
        }
        let text = std::fs::read_to_string(&file).expect("the source reads");
        for line in code_lines(&text) {
            assert!(
                !line.contains("process::exit"),
                "{} leaves through process::exit: {line}",
                file.display()
            );
            assert!(
                !line.contains("ExitCode::from("),
                "{} builds an exit code from a number: {line}",
                file.display()
            );
        }
    }
}

/// Every noun and verb the binary has, as a caller types them.
fn every_verb() -> Vec<(String, String)> {
    let command = Cli::command();
    let mut pairs = Vec::new();
    for noun in command.get_subcommands() {
        for verb in noun.get_subcommands() {
            pairs.push((noun.get_name().to_owned(), verb.get_name().to_owned()));
        }
    }
    pairs
}

/// Every verb of every noun answers for itself.
///
/// `--help` is the cheapest proof that a verb is wired: the argument parser has
/// to build the whole subcommand to print it, so a verb whose flags do not
/// declare cleanly fails here rather than in the field.
#[test]
fn every_verb_parses() {
    let verbs = every_verb();
    assert!(
        verbs.len() >= 28,
        "the binary has the read verbs and the mutation verbs: {verbs:?}"
    );

    for (noun, verb) in &verbs {
        let run = kicli(&[noun, verb, "--help"]);
        assert_eq!(
            code(&run),
            0,
            "kicli {noun} {verb} --help: {}",
            stderr(&run)
        );
        assert!(
            stdout(&run).contains(verb.as_str()),
            "the help names the verb: {}",
            stdout(&run)
        );
    }
}

/// The mutation nouns, which the milestone exists for.
#[test]
fn every_mutation_noun_is_on_the_surface() {
    let nouns: Vec<String> = every_verb().into_iter().map(|(noun, _)| noun).collect();
    for noun in [
        "sym",
        "field",
        "text",
        "label",
        "junction",
        "noconnect",
        "net",
        "wire",
    ] {
        assert!(nouns.iter().any(|had| had == noun), "kicli {noun} is wired");
    }
}

/// Every verb of the `wire` noun is on the surface and answers for itself.
#[test]
fn every_wire_verb_parses() {
    let verbs: Vec<String> = every_verb()
        .into_iter()
        .filter(|(noun, _)| noun == "wire")
        .map(|(_, verb)| verb)
        .collect();
    assert_eq!(
        verbs,
        ["draw", "connect", "delete"],
        "the wire noun's verbs"
    );

    // Each end of a drawn wire is addressed the way that kind of end is
    // addressed everywhere else, and the parser has to build all three forms.
    for verb in &verbs {
        let run = kicli(&["wire", verb, "--help"]);
        assert_eq!(code(&run), 0, "kicli wire {verb} --help: {}", stderr(&run));
    }
    let help = stdout(&kicli(&["wire", "draw", "--help"]));
    for flag in [
        "--from-pin",
        "--from-port",
        "--from-at",
        "--to-pin",
        "--to-port",
        "--to-at",
        "--via",
        "--auto-labels",
    ] {
        assert!(help.contains(flag), "the help names {flag}: {help}");
    }

    // The routed verb addresses its ends the same way and chooses the path
    // itself, so it takes every form of an end and no `--via` at all. The far
    // end takes one form more: a whole net, which is not one point and which
    // only a verb that chooses the path can be given.
    let help = stdout(&kicli(&["wire", "connect", "--help"]));
    for flag in [
        "--from-pin",
        "--from-port",
        "--from-at",
        "--to-pin",
        "--to-port",
        "--to-at",
        "--to-net",
        "--auto-labels",
    ] {
        assert!(help.contains(flag), "the help names {flag}: {help}");
    }
    assert!(
        !help.contains("--via"),
        "the corners are the router's to choose: {help}"
    );
    // And the net form is the far end's alone. A route leaves a point it was
    // told to leave; which point of a net it leaves is not a question the
    // escape rule can answer.
    assert!(
        !help.contains("--from-net"),
        "a route leaves one end, not a net: {help}"
    );
}

/// A drawn wire needs both of its ends, and no form of one is a usage error.
#[test]
fn a_wire_with_an_end_missing_is_a_usage_error() {
    let project = fixtures().scratch_directory("surface_wire_ends", "sch/nets");
    let path = project.to_str().expect("the path is text");
    for arguments in [
        vec!["wire", "draw", "--to-pin", "R2.1"],
        vec!["wire", "draw", "--from-pin", "R1.1"],
        vec!["wire", "draw"],
    ] {
        let mut arguments = arguments;
        arguments.extend(["-p", path]);
        let run = kicli(&arguments);
        assert_eq!(
            code(&run),
            i32::from(ExitCode::Usage.code()),
            "{arguments:?}: {}",
            stderr(&run)
        );
    }
}

/// A drawn wire answers in the route contract and in the mutation result.
///
/// The two say different things and both are needed: the contract says what
/// the route is and what it cost, and the mutation result says what the file
/// now holds and what kicli checked afterwards.
#[test]
fn a_drawn_wire_answers_in_the_contract_and_in_the_mutation_result() {
    // A fresh drawing per run. The wire the first run draws would block the
    // second, which is the verb working rather than the test failing.
    let two_resistors = |name: &str| {
        let mut probe = Probe::new(
            name,
            Path::new(env!("CARGO_TARGET_TMPDIR")).join("wire-surface"),
        );
        probe.place("R", "R1", ("50.8", "54.61"), &["1", "2"]);
        probe.place("R", "R2", ("76.2", "54.61"), &["1", "2"]);
        probe
            .write()
            .parent()
            .expect("the drawing sits in a directory")
            .to_owned()
    };
    fn request(project: &str) -> Vec<&str> {
        vec![
            "wire",
            "draw",
            "--from-pin",
            "R1.1",
            "--to-pin",
            "R2.1",
            "--via",
            "50.8,45.72",
            "--via",
            "76.2,45.72",
            "-p",
            project,
        ]
    }

    let text_project = two_resistors("surface_wire_text");
    let run = kicli(&request(text_project.to_str().expect("the path is text")));
    assert_eq!(code(&run), 0, "a drawn route succeeds: {}", stderr(&run));

    let printed = stdout(&run);
    assert!(
        printed.starts_with("routed R1.1 -> R2.1   via 3 segments, 2 corners, 35.56mm\n"),
        "the contract answers first: {printed}"
    );
    assert!(
        printed.contains("checked: every invariant passed"),
        "and the mutation result follows it: {printed}"
    );

    let json_project = two_resistors("surface_wire_json");
    let mut json = request(json_project.to_str().expect("the path is text"));
    json.extend(["--output", "json"]);
    let run = kicli(&json);
    assert_eq!(code(&run), 0, "and so does the JSON twin: {}", stderr(&run));
    let object: serde_json::Value =
        serde_json::from_str(&stdout(&run)).expect("one object on stdout");
    assert_eq!(
        object["wire"]["status"], "routed",
        "the noun's key carries the contract: {object}"
    );
    // `wire draw` was not asked to join anything, and the contract carries
    // every key at every status — so the joined net is present and null. The
    // text assertion above is the other half: no null, no line.
    assert!(
        object["wire"]
            .as_object()
            .expect("the contract is an object")
            .contains_key("joined_net"),
        "the key is there even where there is no net to name: {object}"
    );
    assert_eq!(
        object["wire"]["joined_net"],
        serde_json::Value::Null,
        "wire draw reports no net: {object}"
    );
    assert_eq!(object["wire"]["cost"]["total"], 44);
    assert!(
        object["invariants"].is_array(),
        "beside what kicli checked: {object}"
    );
}

/// The command layer resolves the segment a delete names, not the library.
///
/// `cli::edit::address` is the one resolver on the agent-facing path. It is
/// judged over the segments alone, so a handle a symbol or a junction happens
/// to share does not make the request ambiguous — a handle shared with a
/// junction says nothing about which wire was meant. When it is genuinely
/// ambiguous the refusal is the command layer's own, listing what it matched
/// rather than choosing one of them.
#[test]
fn a_deleted_segment_is_named_through_the_command_layer_resolver() {
    // Two wires meeting at a junction. The probe gives every object its own
    // handle, so a shared one has to be written into the drawing's text — the
    // one thing here that is not the harness's own bytes, and only because no
    // probe drawing can reach this behaviour.
    let two_wires_and_a_junction = |name: &str, share_with_the_junction: bool| {
        let mut probe = Probe::new(
            name,
            Path::new(env!("CARGO_TARGET_TMPDIR")).join("wire-surface"),
        );
        probe.wire(("50.8", "50.8"), ("63.5", "50.8"));
        probe.wire(("63.5", "50.8"), ("76.2", "50.8"));
        probe.junction(("63.5", "50.8"));
        let sheet = probe.write();
        let written = std::fs::read_to_string(&sheet).expect("the drawing reads");
        let borrower = if share_with_the_junction {
            "01000003-0000-4000-8001-000000000003"
        } else {
            "01000002-0000-4000-8001-000000000002"
        };
        std::fs::write(
            &sheet,
            written.replace(borrower, "01000001-0000-4000-8001-00000000000f"),
        )
        .expect("the drawing is rewritten");
        sheet
    };

    // A handle a junction shares names one segment, and a delete can act on
    // nothing but a segment. It is not ambiguous.
    let sheet = two_wires_and_a_junction("surface_wire_delete_shared", true);
    let project = sheet.parent().expect("the drawing sits in a directory");
    let run = kicli(&[
        "wire",
        "delete",
        "01000001",
        "-p",
        project.to_str().expect("the path is text"),
    ]);
    assert_eq!(
        code(&run),
        0,
        "a handle a junction shares still names one segment: {}",
        stderr(&run)
    );
    let after = std::fs::read_to_string(&sheet).expect("the drawing reads");
    assert!(
        !after.contains("01000001-0000-4000-8001-000000000001"),
        "the named segment is gone: {after}"
    );

    // The junction the removal left joining one end is reported and left in
    // the file. Removing it is a second decision, and it is the caller's.
    let printed = stdout(&run);
    assert!(
        printed.contains("note: stranded-junction"),
        "the caller is told what the removal left behind: {printed}"
    );
    assert!(
        after.contains("(junction"),
        "and the junction is still there: {after}"
    );

    // A handle two segments share is genuinely ambiguous, and the refusal is
    // the command layer's own: it lists what it matched rather than choosing.
    let sheet = two_wires_and_a_junction("surface_wire_delete_ambiguous", false);
    let project = sheet.parent().expect("the drawing sits in a directory");
    let before = std::fs::read_to_string(&sheet).expect("the drawing reads");
    let run = kicli(&[
        "wire",
        "delete",
        "01000001",
        "-p",
        project.to_str().expect("the path is text"),
    ]);
    assert_eq!(
        code(&run),
        i32::from(ExitCode::Operation.code()),
        "an ambiguous handle is refused: {}",
        stderr(&run)
    );
    let refusal = stderr(&run);
    assert!(
        refusal.contains("names 2 objects of this sheet"),
        "the command layer's own resolver answered: {refusal}"
    );
    assert!(
        refusal.contains("000000000001") && refusal.contains("00000000000f"),
        "and it lists both: {refusal}"
    );
    assert_eq!(
        before,
        std::fs::read_to_string(&sheet).expect("the drawing reads"),
        "a refused delete writes nothing"
    );
}

/// Each status of a route report names one row of the exit-code table.
///
/// The mapping is the command layer's and nothing below it knows a number. A
/// proposal is a **result**: the router was asked what to do and it answered,
/// so `labels` exits 0 beside `routed`. Only a well-formed request that could
/// not be completed leaves a non-zero code.
#[test]
fn every_route_status_exits_the_code_its_row_names() {
    for (status, expected) in [
        (Status::Routed, ExitCode::Success),
        (Status::Labels, ExitCode::Success),
        (Status::Blocked, ExitCode::Operation),
        (Status::Invalid, ExitCode::Operation),
    ] {
        assert_eq!(
            ExitCode::for_route(status),
            expected,
            "{} exits {} ({})",
            status.token(),
            expected.code(),
            expected.name()
        );
    }
}

/// A project a test may write to, in a scratch directory of its own.
fn scratch_project(name: &str, files: &[(&str, &str)]) -> PathBuf {
    let directory = fixtures().scratch(name);
    for (file, source) in files {
        std::fs::write(directory.join(file), source).expect("the sheet is written");
    }
    directory
}

/// Four wire ends meeting at one point, and nothing else.
const CROSSROADS: &str = concat!(
    "(kicad_sch\n\t(version 20260306)\n\t(uuid \"root\")\n\t(paper \"A4\")\n",
    "\t(wire\n\t\t(pts\n\t\t\t(xy 38.1 50.8) (xy 50.8 50.8)\n\t\t)\n\t\t(uuid \"wire0\")\n\t)\n",
    "\t(wire\n\t\t(pts\n\t\t\t(xy 50.8 50.8) (xy 63.5 50.8)\n\t\t)\n\t\t(uuid \"wire1\")\n\t)\n",
    "\t(wire\n\t\t(pts\n\t\t\t(xy 50.8 38.1) (xy 50.8 50.8)\n\t\t)\n\t\t(uuid \"wire2\")\n\t)\n",
    "\t(wire\n\t\t(pts\n\t\t\t(xy 50.8 50.8) (xy 50.8 63.5)\n\t\t)\n\t\t(uuid \"wire3\")\n\t)\n)\n",
);

/// A sheet that already breaks the on-grid invariant.
///
/// Nothing a command does to this file can hold, so the invariant pass refuses
/// the write. It is the shortest route to a verification failure through the
/// command surface.
const OFF_GRID: &str = concat!(
    "(kicad_sch\n\t(version 20260306)\n\t(uuid \"root\")\n\t(paper \"A4\")\n",
    "\t(junction\n\t\t(at 25.41 25.4)\n\t\t(uuid \"j1\")\n\t)\n)\n",
);

/// Every refusal exits with the row of the table its kind names.
///
/// The three the milestone gates on are here — the four-way junction, the
/// no-connect on a connected pin and the rename of an unnamed net — with one
/// case for each of the other rows a mutation can end on.
#[test]
fn every_refusal_exits_with_the_code_its_row_names() {
    let nets = fixtures().scratch_directory("surface_refusals_nets", "sch/nets");
    let crossroads = scratch_project(
        "surface_refusals_crossroads",
        &[("board.kicad_sch", CROSSROADS)],
    );
    let off_grid = scratch_project(
        "surface_refusals_off_grid",
        &[("board.kicad_sch", OFF_GRID)],
    );
    let future = fixtures().scratch("surface_refusals_future");
    let _copy = fixtures().copy_file(&future, "sch/future_version.kicad_sch");

    let cases: [(&str, &Path, Vec<&str>, ExitCode); 11] = [
        (
            // The route contract's `invalid`: a request no drawing can hold.
            "a wire vertex off the placement grid",
            &nets,
            vec![
                "wire",
                "draw",
                "--from-at",
                "25.4,25.4",
                "--to-at",
                "25.4,26.035",
            ],
            ExitCode::Operation,
        ),
        (
            // The route contract's `blocked`: a way that is barred.
            "a wire with something in its way",
            &nets,
            vec!["wire", "draw", "--from-pin", "R1.1", "--to-pin", "R2.1"],
            ExitCode::Operation,
        ),
        (
            "a wire segment this sheet does not hold",
            &nets,
            vec!["wire", "delete", "deadbeef"],
            ExitCode::Operation,
        ),
        (
            "a junction where four wire ends already meet",
            &crossroads,
            vec!["junction", "add", "--at", "50.8,50.8"],
            ExitCode::Operation,
        ),
        (
            "a no-connect on a pin something already joins",
            &nets,
            vec!["noconnect", "add", "--pin", "R12.2"],
            ExitCode::Operation,
        ),
        (
            "a rename of a net no label names",
            &nets,
            vec!["net", "rename", "#n1", "--to", "SPY"],
            ExitCode::Operation,
        ),
        (
            "an object this sheet does not hold",
            &nets,
            vec!["sym", "move", "R999", "--by", "0,0"],
            ExitCode::Operation,
        ),
        (
            "a flag the parser does not know",
            &nets,
            vec!["sym", "move", "R1", "--sideways", "1"],
            ExitCode::Usage,
        ),
        (
            "an angle that is not a quarter turn",
            &nets,
            vec!["sym", "rotate", "R1", "--to", "45"],
            ExitCode::Usage,
        ),
        (
            "a change that would not hold its own invariants",
            &off_grid,
            vec!["text", "add", "--text", "hello", "--at", "10,10"],
            ExitCode::Verification,
        ),
        (
            "a file kicli refuses to write at all",
            &future,
            vec!["text", "add", "--text", "hello", "--at", "10,10"],
            ExitCode::File,
        ),
    ];

    for (what, project, arguments, expected) in cases {
        let before = files_of(project);
        let mut args = arguments.clone();
        args.push("-p");
        let project_text = project.to_str().expect("the path is text");
        args.push(project_text);

        let run = kicli(&args);
        assert_eq!(
            code(&run),
            i32::from(expected.code()),
            "{what} exits {} ({}): {}",
            expected.code(),
            expected.name(),
            stderr(&run)
        );
        assert!(
            stdout(&run).is_empty(),
            "{what} writes no result to stdout: {}",
            stdout(&run)
        );
        assert_eq!(before, files_of(project), "{what} wrote nothing");
    }
}

/// Every refusal in JSON is one object naming its row of the table.
#[test]
fn a_refusal_in_json_names_its_row_of_the_table() {
    let crossroads = scratch_project("surface_refusal_json", &[("board.kicad_sch", CROSSROADS)]);
    let run = kicli(&[
        "junction",
        "add",
        "--at",
        "50.8,50.8",
        "-p",
        crossroads.to_str().expect("the path is text"),
        "--output",
        "json",
    ]);

    assert_eq!(code(&run), 1);
    let reported: serde_json::Value =
        serde_json::from_str(&stderr(&run)).expect("stderr is one JSON object");
    assert_eq!(reported["error"]["kind"], "operation");
    assert_eq!(reported["error"]["exit_code"], 1);
    assert!(
        reported["error"]["message"]
            .as_str()
            .expect("the message is text")
            .contains("four wire ends meet"),
        "the refusal says what it refused: {reported}"
    );
}

/// A mutation reports the invariants it ran, in both forms.
#[test]
fn a_mutation_reports_the_invariants_it_ran() {
    let project = fixtures().scratch_directory("surface_mutation_report", "sch/nets");
    let path = project.to_str().expect("the path is text");

    let run = kicli(&[
        "label",
        "add",
        "--text",
        "SPY",
        "--at",
        "30.48,88.9",
        "-p",
        path,
        "--output",
        "json",
    ]);
    assert_eq!(code(&run), 0, "{}", stderr(&run));

    let reported: serde_json::Value =
        serde_json::from_str(&stdout(&run)).expect("stdout is one JSON object");
    let invariants = reported["invariants"]
        .as_array()
        .expect("the report lists the invariants");
    assert_eq!(invariants.len(), 4, "all four ran: {reported}");
    assert!(
        invariants.iter().all(|check| check["passed"] == true),
        "and all four passed: {reported}"
    );
    assert_eq!(reported["reformatted"], false);

    let text = kicli(&[
        "label",
        "add",
        "--text",
        "SPY2",
        "--at",
        "30.48,80.01",
        "-p",
        path,
    ]);
    assert_eq!(code(&text), 0, "{}", stderr(&text));
    assert!(
        stdout(&text).contains("checked: every invariant passed"),
        "the text form says so too: {}",
        stdout(&text)
    );
}

/// Every file of a directory, by name and content.
fn files_of(directory: &Path) -> Vec<(String, Vec<u8>)> {
    let mut found: Vec<(String, Vec<u8>)> = std::fs::read_dir(directory)
        .expect("the directory reads")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .map(|path| {
            (
                path.file_name()
                    .expect("a file has a name")
                    .to_string_lossy()
                    .into_owned(),
                std::fs::read(&path).expect("the file reads"),
            )
        })
        .collect();
    found.sort();
    found
}

/// Every Rust source file under a directory.
fn rust_sources(root: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(&directory).expect("the directory reads") {
            let path = entry.expect("the entry reads").path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|kind| kind == "rs") {
                found.push(path);
            }
        }
    }
    found
}

/// The lines of a source file that are not comments.
fn code_lines(text: &str) -> impl Iterator<Item = &str> {
    text.lines()
        .map(str::trim)
        .filter(|line| !line.starts_with("//"))
}

/// The value names a positional argument may carry.
///
/// Each one names an object that is already in the drawing: `TARGET` and
/// `OWNER` are handles, and `FROM` is the net name a rename starts from.
const HANDLES: [&str; 3] = ["TARGET", "OWNER", "FROM"];

/// The verbs that make a new object, which therefore address none.
const MAKERS: [&str; 6] = [
    "sym place",
    "text add",
    "label add",
    "junction add",
    "noconnect add",
    "wire draw",
];

/// Every positional argument of the surface, as `noun verb` and its value name.
fn positionals(command: &clap::Command, path: &str, found: &mut Vec<(String, String)>) {
    for sub in command.get_subcommands() {
        let here = if path.is_empty() {
            sub.get_name().to_owned()
        } else {
            format!("{path} {}", sub.get_name())
        };
        for argument in sub
            .get_arguments()
            .filter(|argument| argument.is_positional())
        {
            let name = argument
                .get_value_names()
                .and_then(|names| names.first().map(ToString::to_string))
                .unwrap_or_else(|| argument.get_id().to_string().to_uppercase());
            found.push((here.clone(), name));
        }
        positionals(sub, &here, found);
    }
}

#[test]
fn a_positional_argument_always_names_something_that_exists() {
    // The rule the whole surface follows: a positional is a handle for an
    // object already in the drawing, and everything a command makes or sets is
    // a named flag. It is what makes `label delete NAME` and `label add
    // --text NAME` different shapes: one addresses, the other creates.
    let mut found = Vec::new();
    positionals(&Cli::command(), "", &mut found);
    assert!(
        found.len() >= 10,
        "the surface has positionals to check: {found:?}"
    );

    let strangers: Vec<&(String, String)> = found
        .iter()
        .filter(|(_, name)| !HANDLES.contains(&name.as_str()))
        .collect();
    assert!(
        strangers.is_empty(),
        "a positional must name an existing object, one of {HANDLES:?}: {strangers:?}"
    );
}

/// The delta view is on the surface, and its help states what it answers.
///
/// The words matter. An agent that reads this help must not go looking for a
/// command that replays its own edits, because the specification itself once
/// expected the delta to be that.
#[test]
fn the_delta_view_help_says_which_question_it_answers() {
    let run = kicli(&["sch", "view", "--help"]);
    assert_eq!(code(&run), 0, "{}", stderr(&run));

    let help = stdout(&run);
    for words in [
        "delta",
        "--against",
        "since kicli last wrote it",
        "empty right after a mutation, by design",
    ] {
        assert!(help.contains(words), "the help says {words:?}: {help}");
    }
}

/// A project kicli has never written has no state to compare against.
///
/// That is a refusal naming the state it looked for. An empty delta would say
/// "nothing has touched this file", which nothing measured.
#[test]
fn a_delta_with_no_saved_state_exits_the_code_its_row_names() {
    let project = fixtures().scratch_directory("surface_delta_no_state", "sch/nets");
    let path = project.to_str().expect("the path is text");

    for (arguments, named) in [
        (vec!["sch", "view", "--view", "delta"], "@last-write"),
        (
            vec!["sch", "view", "--view", "delta", "--against", "review"],
            "review",
        ),
    ] {
        let mut arguments = arguments;
        arguments.extend(["-p", path]);
        let run = kicli(&arguments);

        assert_eq!(
            code(&run),
            i32::from(ExitCode::Operation.code()),
            "{arguments:?} exits {} ({}): {}",
            ExitCode::Operation.code(),
            ExitCode::Operation.name(),
            stderr(&run)
        );
        assert!(
            stdout(&run).is_empty(),
            "no result reaches stdout: {}",
            stdout(&run)
        );
        assert!(
            stderr(&run).contains(named),
            "the refusal names the state it looked for: {}",
            stderr(&run)
        );
    }
}

#[test]
fn a_verb_that_makes_an_object_takes_no_positional() {
    // The other half of the rule, and the control for the check above: a sweep
    // that found no positionals at all would pass it.
    let mut found = Vec::new();
    positionals(&Cli::command(), "", &mut found);
    let addressing: Vec<&String> = found.iter().map(|(verb, _)| verb).collect();
    assert!(
        addressing.iter().any(|verb| *verb == "label delete"),
        "the sweep reached the verbs that address an object: {addressing:?}"
    );

    for maker in MAKERS {
        assert!(
            !addressing.iter().any(|verb| *verb == maker),
            "{maker} makes an object, so its name is a flag and not a positional"
        );
    }
}
