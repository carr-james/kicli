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
    run_with_tool(project, args, Path::new(NO_KICAD_CLI))
}

/// Run the binary with discovery pointed at a chosen binary.
fn run_with_tool(project: &str, args: &[&str], tool: &Path) -> Output {
    let directory = fixture(project);
    Command::new(env!("CARGO_BIN_EXE_kicli"))
        .args(args)
        .arg("--project")
        .arg(&directory)
        .env("KICLI_KICAD_CLI", tool)
        .output()
        .expect("the binary runs")
}

/// A stand-in for `kicad-cli` that reports one version and does nothing else.
///
/// The health check asks the binary its version, and a real KiCad install is
/// neither present on every machine nor quick on a cold one. A stand-in keeps
/// the check hermetic while the discovery and version paths still run.
#[cfg(unix)]
fn stand_in(version: &str) -> PathBuf {
    use std::os::unix::fs::PermissionsExt;

    let directory = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("kicad-cli-{version}"));
    std::fs::create_dir_all(&directory).expect("the directory is made");
    let program = directory.join("kicad-cli");
    std::fs::write(&program, format!("#!/bin/sh\necho {version}\n")).expect("the file is written");
    std::fs::set_permissions(&program, std::fs::Permissions::from_mode(0o755))
        .expect("the file is made runnable");
    program
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

#[cfg(unix)]
#[test]
fn project_check_finds_each_fault() {
    let tool = stand_in("10.0.5");
    let text = run_with_tool("project/broken", &["project", "check", "--quiet"], &tool);
    assert_eq!(code(&text), 0, "findings are data, not failure");

    let machine = run_with_tool(
        "project/broken",
        &["project", "check", "--quiet", "--output", "json"],
        &tool,
    );
    assert_eq!(code(&machine), 0);
    let reported = json(&machine);
    let findings = reported["findings"].as_array().expect("findings is a list");

    // The three planted faults, each in its own file and each a different kind.
    let planted = [
        ("sheet-file-missing", "broken.kicad_sch"),
        ("version-ceiling", "future.kicad_sch"),
        ("refuse-to-write", "commented.kicad_sch"),
    ];
    for (kind, file) in planted {
        let found = findings
            .iter()
            .find(|finding| finding["kind"] == kind)
            .unwrap_or_else(|| panic!("{kind} is reported: {reported}"));
        assert_eq!(found["file"], file, "{kind} names its file");
        assert!(
            !found["message"]
                .as_str()
                .expect("the message is text")
                .is_empty(),
            "{kind} says what is wrong"
        );
    }

    let kinds: Vec<&str> = findings
        .iter()
        .map(|finding| finding["kind"].as_str().expect("a kind"))
        .collect();
    assert_eq!(
        kinds.len(),
        3,
        "no fault beyond the three planted: {kinds:?}"
    );

    // The text form carries the same findings.
    let printed = stdout(&text);
    for (kind, file) in planted {
        assert!(printed.contains(kind), "the text form names {kind}");
        assert!(printed.contains(file), "the text form names {file}");
    }
    assert_eq!(printed, golden("project_check_broken.golden"));
}

#[cfg(unix)]
#[test]
fn project_check_passes_a_healthy_project() {
    let tool = stand_in("10.0.5");
    let text = run_with_tool("project/healthy", &["project", "check", "--quiet"], &tool);
    assert_eq!(code(&text), 0, "{}", stderr(&text));
    assert_eq!(stdout(&text), golden("project_check_healthy.golden"));

    let reported = json(&run_with_tool(
        "project/healthy",
        &["project", "check", "--quiet", "--output", "json"],
        &tool,
    ));
    assert_eq!(
        reported["findings"].as_array().expect("a list").len(),
        0,
        "a healthy project has nothing to report: {reported}"
    );
    assert_eq!(reported["checked"]["files"], 2);
    assert_eq!(reported["checked"]["sheets"], 2);
}

#[cfg(unix)]
#[test]
fn project_check_reports_a_sheet_that_names_a_file_above_it() {
    let tool = stand_in("10.0.5");
    let reported = json(&run_with_tool(
        "project/cycle",
        &["project", "check", "--quiet", "--output", "json"],
        &tool,
    ));
    let kinds: Vec<&str> = reported["findings"]
        .as_array()
        .expect("a list")
        .iter()
        .map(|finding| finding["kind"].as_str().expect("a kind"))
        .collect();
    assert!(kinds.contains(&"sheet-cycle"), "{reported}");
}

#[test]
fn project_check_names_a_missing_kicad_cli() {
    let reported = json(&run(
        "project/healthy",
        &["project", "check", "--quiet", "--output", "json"],
    ));
    let tool: Vec<&serde_json::Value> = reported["findings"]
        .as_array()
        .expect("a list")
        .iter()
        .filter(|finding| finding["kind"] == "kicad-cli")
        .collect();
    assert_eq!(tool.len(), 1, "{reported}");
    assert!(
        tool[0]["message"]
            .as_str()
            .expect("the message is text")
            .contains("KiCad 10"),
        "the finding carries an install hint: {}",
        tool[0]["message"]
    );
}

#[cfg(unix)]
#[test]
fn project_check_refuses_a_binary_of_the_wrong_version() {
    let tool = stand_in("9.0.1");
    let reported = json(&run_with_tool(
        "project/healthy",
        &["project", "check", "--quiet", "--output", "json"],
        &tool,
    ));
    let named: Vec<&serde_json::Value> = reported["findings"]
        .as_array()
        .expect("a list")
        .iter()
        .filter(|finding| finding["kind"] == "kicad-cli")
        .collect();
    assert_eq!(named.len(), 1, "{reported}");
    assert!(
        named[0]["message"]
            .as_str()
            .expect("the message is text")
            .contains("9.0.1"),
        "the finding names the version it found"
    );
}

#[test]
fn project_check_names_the_checks_it_does_not_make() {
    let run = run("project/healthy", &["project", "check", "--quiet"]);
    let printed = stdout(&run);
    assert!(
        printed.contains("not covered"),
        "the check says what it leaves out: {printed}"
    );
    assert!(
        printed.contains("library"),
        "library resolution is named as not covered: {printed}"
    );
}

#[test]
fn project_check_reads_the_whole_project() {
    let run = run(
        "project/healthy",
        &["project", "check", "--quiet", "--sheet", "/anything"],
    );
    assert_eq!(code(&run), 2, "a health check is not restricted to a sheet");
    assert!(stdout(&run).is_empty());
    assert!(
        stderr(&run).contains("--sheet"),
        "the error names the flag: {}",
        stderr(&run)
    );
}

/// The warming step prints before the health check blocks on `kicad-cli`.
///
/// The first KiCad run on a machine builds the font cache and can take over two
/// minutes. This one uses the real binary, so it is skipped unless asked for.
#[test]
fn project_check_announces_warming() {
    if std::env::var("KICLI_TEST_KICAD_CLI").is_err() {
        eprintln!("skipped: set KICLI_TEST_KICAD_CLI=1 to run this against a real kicad-cli");
        return;
    }

    let found = which_kicad_cli().expect("kicad-cli is on PATH");
    let run = run_with_tool("project/healthy", &["project", "check"], &found);
    assert_eq!(code(&run), 0, "{}", stderr(&run));

    let said = stderr(&run);
    assert!(
        said.contains("font cache") && said.contains("120"),
        "the run says why it may be slow: {said}"
    );
    assert!(
        stdout(&run).contains("kicad-cli  10."),
        "the report carries the version it found: {}",
        stdout(&run)
    );
}

/// The `kicad-cli` on `PATH`, when there is one.
fn which_kicad_cli() -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path)
        .map(|entry| entry.join("kicad-cli"))
        .find(|candidate| candidate.is_file())
}
