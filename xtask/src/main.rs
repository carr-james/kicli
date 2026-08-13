//! Workspace automation for kicli.
//!
//! Run a task with `cargo xtask <task>`. The alias lives in
//! `.cargo/config.toml`.
//!
//! `check` runs every quality gate: formatting, lints, tests, documentation,
//! and dependency licences. All gates must pass before a task is complete. A
//! failing gate does not stop the run, so one invocation reports every problem.

#![deny(unsafe_code)]
#![warn(clippy::pedantic)]

mod corpus;

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
}

/// Run every gate. Report a summary. Fail if any gate failed.
fn run_check() -> ExitCode {
    let mut failed: Vec<&Gate> = Vec::new();

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
            failed.push(gate);
        }
    }

    report(&failed);

    if failed.is_empty() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

/// Print the summary line for every gate.
fn report(failed: &[&Gate]) {
    println!("\n=== summary ===");

    for gate in GATES {
        let ok = !failed.iter().any(|other| other.name == gate.name);
        let mark = if ok { "pass" } else { "FAIL" };
        println!("  {mark}  {}", gate.name);
    }

    for gate in failed {
        if let Some(hint) = gate.install_hint {
            println!("\nnote: if '{}' is not installed, run: {hint}", gate.name);
        }
    }

    if failed.is_empty() {
        println!("\nall gates passed");
    } else {
        println!("\n{} of {} gates failed", failed.len(), GATES.len());
    }
}
