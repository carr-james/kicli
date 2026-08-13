//! The command surface: flags, help, exit codes, and the shape of an error.
//!
//! These tests run the compiled binary, because the exit code and the split
//! between stdout and stderr are part of the contract, and neither is visible
//! from inside the library.

use kicli::cli::ExitCode;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

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
    let project = fixture("project/healthy");
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

/// One of this crate's fixture directories.
fn fixture(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(relative)
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
