//! Workspace automation for kicli.
//!
//! Run a task with `cargo xtask <task>`. The alias lives in
//! `.cargo/config.toml`.
//!
//! `check` runs the quality gates: formatting, lints, tests, documentation,
//! dependency licences, and that the run changed no file outside `target/`.
//! All gates must pass before a task is complete. A failing gate does not stop
//! the run, so one invocation reports every problem.
//!
//! Two arms are opt-in, because they cost minutes rather than seconds:
//! the corpus tests and the `kicad-cli` oracle. `--corpus` runs them. The
//! ENUMERATION is not opt-in: the summary lists every arm on every run, and an
//! arm that did not run reports `skip` with the command that runs it. See
//! `gate` for why the third verdict exists.

#![deny(unsafe_code)]
#![warn(clippy::pedantic)]

mod corpus;
mod gate;
mod text_metrics;

use gate::{ARMS, CORPUS_FLAG, Run, Verdict};
use std::process::{Command, ExitCode};

/// Exit code 2 reports a usage error: xtask did not understand the task name.
const EXIT_USAGE: u8 = 2;

fn main() -> ExitCode {
    let task = std::env::args().nth(1);

    match task.as_deref() {
        Some("check") => run_check(std::env::args().any(|a| a == CORPUS_FLAG)),
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
    eprintln!("  check    Run the quality gates. Every arm is listed, run or not.");
    eprintln!("           --corpus also runs the corpus tests and the kicad-cli");
    eprintln!("           oracle. Fetch the corpus first with `cargo xtask corpus`.");
    eprintln!("  corpus   Fetch KiCad's demo files into target/. --verify checks them.");
    eprintln!("  text-metrics  Derive the glyph advance table. --verify checks it.");
}

/// Run the arms. Report every arm. Fail if any arm failed.
///
/// `corpus_requested` is the `--corpus` flag. It turns on the two arms that
/// measure kicli against KiCad's own files. Without it those arms are still
/// listed, and still say what would run them.
fn run_check(corpus_requested: bool) -> ExitCode {
    if let Some(refusal) = refuse_without_a_corpus(corpus_requested) {
        return refusal;
    }

    let oracle = kicad_cli();
    let before = tree_state();
    let mut outcomes: Vec<(&'static str, Verdict)> = Vec::new();
    let mut corpus_ran: Option<Verdict> = None;

    for arm in ARMS {
        if let Some(skipped) = gate::skipped_before_running(arm, corpus_requested, oracle.is_some())
        {
            outcomes.push((arm.name, skipped));
            continue;
        }
        let verdict = match arm.run {
            Run::Always => {
                header(arm.name);
                verdict_of(cargo(arm.args, arm.env, &[]))
            }
            Run::OnCorpusFlag => {
                header(arm.name);
                let verdict = run_corpus(arm, oracle.is_some());
                corpus_ran = Some(verdict.clone());
                verdict
            }
            // The oracle arm has no command of its own. It is the corpus run
            // carrying KICLI_TEST_KICAD_CLI, so it reports what that run
            // reported.
            Run::RidesCorpus => corpus_ran.clone().unwrap_or(Verdict::Fail),
            Run::Tree => {
                header(arm.name);
                verdict_of(tree_is_unchanged(before.as_deref()))
            }
        };
        outcomes.push((arm.name, verdict));
    }

    println!();
    print!("{}", gate::summary(&outcomes));

    if outcomes
        .iter()
        .any(|(_, verdict)| *verdict == Verdict::Fail)
    {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

/// Refuse the run when `--corpus` asks for a corpus that is not there.
///
/// A flag that asks for the corpus and silently gets no corpus is the defect
/// this summary exists to prevent, wearing a flag: every corpus test finds no
/// files, prints one line and returns, and the arm passes on nothing.
fn refuse_without_a_corpus(corpus_requested: bool) -> Option<ExitCode> {
    if !corpus_requested {
        return None;
    }
    let reason = corpus::fetched(&corpus::root()).err()?;
    eprintln!("xtask: {CORPUS_FLAG} asks for the corpus and {reason}.");
    eprintln!("xtask: run `cargo xtask corpus` first, then run this again.");
    eprintln!("xtask: no arm ran.");
    Some(ExitCode::FAILURE)
}

/// Run the corpus arm, with the oracle environment when KiCad is there.
fn run_corpus(arm: &gate::Arm, oracle: bool) -> Verdict {
    let extra: &[(&str, &str)] = if oracle {
        &[("KICLI_TEST_KICAD_CLI", "1")]
    } else {
        &[]
    };
    verdict_of(cargo(arm.args, arm.env, extra))
}

/// Print the banner that separates one arm's output from the next.
fn header(name: &str) {
    println!("\n=== {name} ===");
}

/// Run one cargo command. Say whether it succeeded.
fn cargo(args: &[&str], env: &[(&'static str, &'static str)], extra_env: &[(&str, &str)]) -> bool {
    let mut command = Command::new("cargo");
    command.args(args);
    for (key, value) in env {
        command.env(key, value);
    }
    for (key, value) in extra_env {
        command.env(key, value);
    }
    match command.status() {
        Ok(status) => status.success(),
        Err(error) => {
            eprintln!("xtask: cannot run cargo: {error}");
            false
        }
    }
}

/// The verdict for an arm that ran.
fn verdict_of(succeeded: bool) -> Verdict {
    if succeeded {
        Verdict::Pass
    } else {
        Verdict::Fail
    }
}

/// The `kicad-cli` this machine offers, or nothing.
///
/// `KICLI_KICAD_CLI` names the binary when it is not `kicad-cli` on the path,
/// which is the same rule the test oracle follows.
fn kicad_cli() -> Option<std::path::PathBuf> {
    let named = std::env::var("KICLI_KICAD_CLI").unwrap_or_else(|_| "kicad-cli".to_owned());
    corpus::which(&named)
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
