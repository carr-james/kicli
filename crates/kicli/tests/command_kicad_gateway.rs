//! The external-binary gateway: discovery, the version check, and translation.
//!
//! No test here starts KiCad. The process runner is a trait, so a fake answers
//! with the exit code under test and the whole translation is exercised on a
//! machine that has never seen `kicad-cli`.

use kicli::cli::ExitCode;
use kicli::kicad::{CliFailure, Completed, Discovery, Invocation, KicadCli, Runner};
use std::path::{Path, PathBuf};

/// A runner that answers with a fixed exit code and a fixed version.
struct Fake {
    /// What `kicad-cli` would have reported.
    code: i32,
    /// What `version --format plain` would have printed.
    version: &'static str,
}

impl Fake {
    fn reporting(code: i32) -> Self {
        Self {
            code,
            version: "10.0.5",
        }
    }
}

impl Runner for Fake {
    fn run(&self, _invocation: &Invocation) -> Result<Completed, std::io::Error> {
        Ok(Completed {
            code: Some(self.code),
            stdout: format!("{}\n", self.version),
            stderr: String::new(),
        })
    }
}

/// A gateway over a fake runner, at a program name that is never started.
fn gateway(runner: Fake) -> KicadCli<Fake> {
    KicadCli::with_runner(PathBuf::from("kicad-cli"), runner)
}

#[test]
fn kicad_cli_codes_are_translated() {
    // The left column is kicad-cli's, the right is kicli's. The two schemes
    // give different meanings to the same numbers, so every row must move.
    let table = [
        (0, None),
        (1, Some(ExitCode::Operation)),
        (2, Some(ExitCode::Operation)),
        (3, Some(ExitCode::File)),
        (5, Some(ExitCode::Operation)),
        (6, Some(ExitCode::Operation)),
    ];

    for (reported, expected) in table {
        let outcome = gateway(Fake::reporting(reported)).run(&["sch", "erc", "sheet.kicad_sch"]);
        match (outcome, expected) {
            (Ok(_), None) => {}
            (Err(failure), Some(code)) => {
                assert_eq!(
                    ExitCode::for_tool_failure(&failure),
                    code,
                    "kicad-cli {reported} became the wrong kicli code: {failure}"
                );
            }
            (outcome, expected) => {
                panic!("kicad-cli {reported} gave {outcome:?}, expected {expected:?}");
            }
        }
    }
}

#[test]
fn no_kicli_code_repeats_a_kicad_cli_code() {
    // The two collisions that would mislead an agent most: 5 means rule
    // violations to kicad-cli and a gate failure to kicli; 6 means a failed job
    // to kicad-cli and a missing tool to kicli.
    for reported in [5, 6] {
        let failure = gateway(Fake::reporting(reported))
            .run(&["sch", "erc", "sheet.kicad_sch"])
            .expect_err("the run failed");
        let code = ExitCode::for_tool_failure(&failure);
        assert_ne!(
            i32::from(code.code()),
            reported,
            "kicad-cli {reported} passed through unchanged"
        );
    }
}

#[test]
fn a_missing_binary_is_reported_with_an_install_hint() {
    let discovery = Discovery {
        environment: Some("/nonexistent/kicad-cli".to_owned()),
        configured: None,
    };
    let failure = discovery.locate().expect_err("nothing is there");
    assert_eq!(ExitCode::for_tool_failure(&failure), ExitCode::Tool);

    let message = failure.to_string();
    assert!(
        message.contains("kicad-cli"),
        "the error names the binary: {message}"
    );
    assert!(
        message.contains("KiCad 10"),
        "the error carries an install hint: {message}"
    );
    assert!(
        matches!(failure, CliFailure::NotFound { .. }),
        "the error says the binary is not there"
    );
}

#[test]
fn a_binary_of_the_wrong_major_version_is_refused() {
    let older = Fake {
        code: 0,
        version: "9.0.1",
    };
    let failure = gateway(older).version().expect_err("version 9 is refused");
    assert_eq!(ExitCode::for_tool_failure(&failure), ExitCode::Tool);
    let message = failure.to_string();
    assert!(
        message.contains("9.0.1") && message.contains("10"),
        "the error names both versions: {message}"
    );

    let current = gateway(Fake::reporting(0))
        .version()
        .expect("version 10 is accepted");
    assert_eq!(current, "10.0.5");
}

#[test]
fn discovery_looks_in_the_documented_order() {
    let places = Discovery::places();
    assert_eq!(
        places,
        [
            "$KICLI_KICAD_CLI",
            "kicli.toml tools.kicad_cli_path",
            "PATH",
            "/Applications/KiCad/KiCad.app/Contents/MacOS/kicad-cli",
        ]
    );
}

/// kicli never runs `sch upgrade`, so no source file may name it as an
/// argument. The rule is easy to break by accident and silent when broken: the
/// upgraded file still parses, still opens, and describes a different circuit.
#[test]
fn no_code_path_invokes_sch_upgrade() {
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    for file in rust_sources(&source_root) {
        let text = std::fs::read_to_string(&file).expect("the source reads");
        for line in text.lines().map(str::trim) {
            if line.starts_with("//") {
                continue;
            }
            assert!(
                !line.contains("\"upgrade\""),
                "{} names upgrade as an argument: {line}",
                file.display()
            );
        }
    }
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
