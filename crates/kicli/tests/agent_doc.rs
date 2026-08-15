//! The agent document and the binary say the same thing.
//!
//! A command that is not written down is a command an agent cannot use, and a
//! document that describes a command the binary does not have is worse than no
//! document. This test reads both and compares them, so neither can drift.

use clap::CommandFactory;
use kicli::cli::{Cli, ExitCode};
use std::path::Path;

fn agent_doc() -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../AGENT.md");
    std::fs::read_to_string(path).expect("AGENT.md sits at the root of the repository")
}

#[test]
fn agent_doc_covers_every_command() {
    let doc = agent_doc();
    let command = Cli::command();

    let mut checked = 0;
    for noun in command.get_subcommands() {
        for verb in noun.get_subcommands() {
            let name = format!("kicli {} {}", noun.get_name(), verb.get_name());
            assert!(doc.contains(&name), "AGENT.md does not document `{name}`");
            checked += 1;
        }
    }
    assert!(checked >= 3, "the binary has commands to document");
}

#[test]
fn agent_doc_covers_every_global_flag() {
    let doc = agent_doc();
    let command = Cli::command();

    for argument in command.get_arguments() {
        let Some(long) = argument.get_long() else {
            continue;
        };
        // A hidden flag is deliberately not advertised. `--variant` is accepted
        // so that a caller's script does not break, and does nothing.
        if argument.is_hide_set() {
            assert!(
                !doc.contains(&format!("`--{long}`")),
                "--{long} is hidden, so AGENT.md should not advertise it"
            );
            continue;
        }
        assert!(
            doc.contains(&format!("--{long}")),
            "AGENT.md does not document --{long}"
        );
    }
}

#[test]
fn agent_doc_covers_every_verb_flag() {
    let doc = agent_doc();
    let command = Cli::command();

    for noun in command.get_subcommands() {
        for verb in noun.get_subcommands() {
            for argument in verb.get_arguments() {
                let Some(long) = argument.get_long() else {
                    continue;
                };
                if argument.is_hide_set() || long == "help" {
                    continue;
                }
                assert!(
                    doc.contains(&format!("--{long}")),
                    "AGENT.md does not document --{long} of `{} {}`",
                    noun.get_name(),
                    verb.get_name()
                );
            }
        }
    }
}

#[test]
fn agent_doc_carries_the_whole_exit_code_table() {
    let doc = agent_doc();
    for code in ExitCode::ALL {
        let row = format!("| {} | {} |", code.code(), code.name());
        assert!(
            doc.contains(&row),
            "AGENT.md is missing the row for exit code {} ({})",
            code.code(),
            code.name()
        );
        assert!(
            doc.contains(code.meaning()),
            "AGENT.md does not say what exit code {} means",
            code.code()
        );
    }
}

#[test]
fn agent_doc_states_what_the_spec_requires_it_to_state() {
    let doc = agent_doc();

    // The translation table, because an agent that reads a kicad-cli code in
    // kicli's table would be misled.
    assert!(
        doc.contains("ERR_INVALID_INPUT_FILE"),
        "the translation table"
    );
    assert!(
        doc.contains("translated, never passed through")
            || doc.contains("translated") && doc.contains("never lets one"),
        "and the rule that goes with it"
    );

    // The delta and the result of a mutation answer different questions. An
    // agent told otherwise goes looking for a command that replays its own
    // edits, and there is none.
    assert!(
        doc.contains("kicli sch view --view delta"),
        "the command that answers the second question"
    );
    assert!(
        doc.contains("since kicli last wrote it"),
        "the question it answers"
    );
    assert!(
        doc.contains("that command already reported it"),
        "and the question it does not"
    );

    // The views are the truth an agent acts on.
    assert!(
        doc.contains("This is what you act on"),
        "the views are the data, not a picture of it"
    );

    // The licence, and the recommendation for people who need a permissive one.
    assert!(doc.contains("GPL-3.0-or-later"), "kicli's licence");
    assert!(
        doc.contains("kicad-tools"),
        "the recommendation for Python users"
    );
}
