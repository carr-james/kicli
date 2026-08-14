//! The command surface: flags, help, exit codes, and the shape of an error.
//!
//! These tests run the compiled binary, because the exit code and the split
//! between stdout and stderr are part of the contract, and neither is visible
//! from inside the library.

use clap::CommandFactory;
use kicli::cli::{Cli, ExitCode};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

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
        verbs.len() >= 26,
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
    ] {
        assert!(nouns.iter().any(|had| had == noun), "kicli {noun} is wired");
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

    let cases: [(&str, &Path, Vec<&str>, ExitCode); 8] = [
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
