//! The `project` commands, run as the compiled binary.
//!
//! Discovery is pointed at a path no binary is at, so a machine with KiCad
//! installed gives the same answer as one without and the text form is stable.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// A path no `kicad-cli` is at.
const NO_KICAD_CLI: &str = "/nonexistent/kicad-cli";

/// Run the binary against one of this crate's fixture projects.
fn run(project: &str, args: &[&str]) -> Output {
    let directory = fixture(project);
    Command::new(env!("CARGO_BIN_EXE_kicli"))
        .args(args)
        .arg("--project")
        .arg(&directory)
        .env("KICLI_KICAD_CLI", NO_KICAD_CLI)
        .output()
        .expect("the binary runs")
}

/// One of this crate's fixture directories.
fn fixture(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(relative)
}

/// The committed text a command must reproduce.
fn golden(name: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join(name);
    std::fs::read_to_string(path).expect("the golden file is readable")
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

/// The JSON a run wrote to standard output.
fn json(run: &Output) -> serde_json::Value {
    serde_json::from_str(&stdout(run)).expect("stdout is one JSON object")
}

#[test]
fn project_info_reports_the_tree() {
    let text = run("project/healthy", &["project", "info", "--quiet"]);
    assert_eq!(code(&text), 0, "{}", stderr(&text));
    assert_eq!(
        stdout(&text),
        golden("project_info_healthy.golden"),
        "the text form is stable"
    );

    let machine = run(
        "project/healthy",
        &["project", "info", "--quiet", "--output", "json"],
    );
    assert_eq!(code(&machine), 0, "{}", stderr(&machine));
    let reported = json(&machine);

    assert_eq!(reported["project"]["name"], "healthy");
    assert_eq!(reported["project"]["root"], "healthy.kicad_sch");
    assert_eq!(reported["project"]["file"], "healthy.kicad_pro");

    // The same sheet paths, page numbers and counts as the text form.
    let sheets = reported["sheets"].as_array().expect("sheets is a list");
    assert_eq!(sheets.len(), 2);
    for sheet in sheets {
        let path = sheet["path"].as_str().expect("a sheet path");
        assert!(stdout(&text).contains(path), "the text form carries {path}");
        assert!(path.starts_with("/00000000-0000-4000-8000-050000000000"));
    }
    assert_eq!(sheets[0]["page"], "1");
    assert_eq!(sheets[1]["page"], "2");
    assert_eq!(sheets[1]["name"], "stage");
    assert_eq!(sheets[1]["file"], "stage.kicad_sch");
    assert_eq!(sheets[0]["symbols"], 0);
    assert_eq!(sheets[0]["power_symbols"], 0);

    // The same format stamps as the text form.
    let files = reported["files"].as_array().expect("files is a list");
    assert_eq!(files.len(), 2);
    for file in files {
        assert_eq!(file["stamp"], 20_260_306);
        assert_eq!(file["layout"], "normal");
        assert_eq!(file["canonical"], true);
    }

    // Bus aliases are project-file data in this format version, so the report
    // carries them as a list of their own.
    assert!(reported["bus_aliases"].is_array());

    assert_eq!(reported["kicad_cli"]["found"], false);
    assert_eq!(reported["project"]["faults"], 0);
}

#[test]
fn project_info_reports_a_project_that_is_broken() {
    let text = run("project/broken", &["project", "info", "--quiet"]);
    assert_eq!(code(&text), 0, "a fault is data, not failure");
    assert_eq!(stdout(&text), golden("project_info_broken.golden"));
}

#[test]
fn project_info_restricts_itself_to_one_sheet() {
    let whole = json(&run(
        "project/healthy",
        &["project", "info", "--quiet", "--output", "json"],
    ));
    let child = whole["sheets"][1]["path"].as_str().expect("a sheet path");

    let one = run(
        "project/healthy",
        &[
            "project", "info", "--quiet", "--output", "json", "--sheet", child,
        ],
    );
    assert_eq!(code(&one), 0, "{}", stderr(&one));
    let reported = json(&one);
    let sheets = reported["sheets"].as_array().expect("sheets is a list");
    assert_eq!(sheets.len(), 1);
    assert_eq!(sheets[0]["file"], "stage.kicad_sch");

    let files = reported["files"].as_array().expect("files is a list");
    assert_eq!(files.len(), 1, "only the sheet's own file is reported");
}

#[test]
fn a_sheet_path_that_is_not_in_the_tree_is_a_usage_error() {
    let run = run(
        "project/healthy",
        &["project", "info", "--quiet", "--sheet", "/nowhere"],
    );
    assert_eq!(code(&run), 2);
    assert!(stdout(&run).is_empty());
    assert!(
        stderr(&run).contains("/nowhere"),
        "the error names the path: {}",
        stderr(&run)
    );
}

#[test]
fn a_directory_with_no_project_is_an_operation_error() {
    let run = run("sch", &["project", "info", "--quiet"]);
    assert_eq!(code(&run), 1);
    assert!(stdout(&run).is_empty());
}
