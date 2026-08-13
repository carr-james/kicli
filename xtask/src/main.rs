//! Workspace automation for kicli.
//!
//! Run a task with `cargo xtask <task>`. The alias lives in
//! `.cargo/config.toml`.
//!
//! `check` runs every quality gate: formatting, lints, tests, documentation,
//! dependency licences, and that the run changed no file outside `target/`.
//! All gates must pass before a task is complete. A failing gate does not stop
//! the run, so one invocation reports every problem.

#![deny(unsafe_code)]
#![warn(clippy::pedantic)]

mod corpus;
mod text_metrics;

use std::process::{Command, ExitCode};

/// Exit code 2 reports a usage error: xtask did not understand the task name.
const EXIT_USAGE: u8 = 2;

/// One quality gate.
struct Gate {
    /// Short name printed in the summary.
    name: &'static str,
    /// Arguments passed to `cargo`.
    args: &'static [&'static str],
    /// Environment variables set for this gate only.
    env: &'static [(&'static str, &'static str)],
    /// Command that installs the missing tool, when the gate needs one.
    install_hint: Option<&'static str>,
}

/// The gates, in the order they run.
const GATES: &[Gate] = &[
    Gate {
        name: "fmt",
        args: &["fmt", "--check"],
        env: &[],
        install_hint: Some("rustup component add rustfmt"),
    },
    Gate {
        name: "clippy",
        args: &[
            "clippy",
            "--all-targets",
            "--all-features",
            "--",
            "-D",
            "warnings",
        ],
        env: &[],
        install_hint: Some("rustup component add clippy"),
    },
    Gate {
        name: "test",
        args: &["test"],
        env: &[],
        install_hint: None,
    },
    Gate {
        // Treat any rustdoc warning as an error, so "builds clean" is testable.
        name: "doc",
        args: &["doc", "--no-deps"],
        env: &[("RUSTDOCFLAGS", "-D warnings")],
        install_hint: None,
    },
    Gate {
        name: "deny",
        args: &["deny", "check"],
        env: &[],
        install_hint: Some("cargo install --locked cargo-deny"),
    },
];

fn main() -> ExitCode {
    let task = std::env::args().nth(1);

    match task.as_deref() {
        Some("check") => run_check(),
        Some("corpus") => corpus::run(std::env::args().any(|a| a == "--verify")),
        Some("text-metrics") => text_metrics::run(std::env::args().any(|a| a == "--verify")),
        Some(other) => {
            eprintln!("xtask: unknown task '{other}'.");
            usage();
            ExitCode::from(EXIT_USAGE)
        }
        None => {
            usage();
            ExitCode::from(EXIT_USAGE)
        }
    }
}

/// Print the list of tasks.
fn usage() {
    eprintln!("usage: cargo xtask <task>");
    eprintln!();
    eprintln!("tasks:");
    eprintln!("  check    Run every quality gate.");
    eprintln!("  corpus   Fetch KiCad's demo files into target/. --verify checks them.");
    eprintln!("  text-metrics  Derive the glyph advance table. --verify checks it.");
}

/// The name of the gate that checks the gates changed no tracked file.
const CLEAN: &str = "clean";

/// Run every gate. Report a summary. Fail if any gate failed.
fn run_check() -> ExitCode {
    let mut failed: Vec<&'static str> = Vec::new();
    let before = tree_state();

    for gate in GATES {
        println!("\n=== {} ===", gate.name);

        let mut command = Command::new("cargo");
        command.args(gate.args);
        for (key, value) in gate.env {
            command.env(key, value);
        }

        let outcome = match command.status() {
            Ok(status) => status.success(),
            Err(error) => {
                eprintln!("xtask: cannot run cargo: {error}");
                false
            }
        };

        if !outcome {
            failed.push(gate.name);
        }
    }

    println!("\n=== {CLEAN} ===");
    if !tree_is_unchanged(before.as_deref()) {
        failed.push(CLEAN);
    }

    report(&failed);

    if failed.is_empty() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

/// What git says about the working tree, or nothing when git cannot say.
///
/// The state is the porcelain status, which lists every file that differs from
/// the index or is not tracked. Files under `target/` are ignored, so a build
/// does not appear here.
fn tree_state() -> Option<String> {
    let output = Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).into_owned())
}

/// Did the gates leave the working tree as they found it?
///
/// A test writes its scratch files under `target/`. One that writes anywhere
/// else — a fixture rebuilt in place, a research note overwritten — changes the
/// repository as a side effect of running the suite, and the next reader cannot
/// tell that change from an edit somebody meant. Comparing the tree before and
/// after makes it a failed gate. The comparison is against the state at the
/// start of the run rather than against a clean tree, so uncommitted work in
/// progress is not itself a failure.
fn tree_is_unchanged(before: Option<&str>) -> bool {
    let Some(before) = before else {
        println!("skipped: git cannot report the working tree here");
        return true;
    };
    let Some(after) = tree_state() else {
        println!("skipped: git cannot report the working tree here");
        return true;
    };
    if before == after {
        println!("the gates changed no file outside target/");
        return true;
    }
    eprintln!("the gates changed the working tree. Before:");
    eprintln!("{before}");
    eprintln!("After:");
    eprintln!("{after}");
    eprintln!("A test must write its scratch files under target/.");
    false
}

/// Print the summary line for every gate.
fn report(failed: &[&str]) {
    println!("\n=== summary ===");

    let names = GATES.iter().map(|gate| gate.name).chain([CLEAN]);
    for name in names {
        let mark = if failed.contains(&name) {
            "FAIL"
        } else {
            "pass"
        };
        println!("  {mark}  {name}");
    }

    for gate in GATES {
        if let (true, Some(hint)) = (failed.contains(&gate.name), gate.install_hint) {
            println!("\nnote: if '{}' is not installed, run: {hint}", gate.name);
        }
    }

    if failed.is_empty() {
        println!("\nall gates passed");
    } else {
        println!("\n{} of {} gates failed", failed.len(), GATES.len() + 1);
    }
}
