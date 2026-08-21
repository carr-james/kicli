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

/// A heading, its level, and everything under it until the next heading of the
/// same or a shallower level.
struct Section<'a> {
    level: usize,
    title: &'a str,
    body: String,
}

/// Split the document into sections, ignoring anything inside a fenced block.
///
/// The fence skipping is not decoration. `AGENT.md` prints view output inside
/// fences, and those lines start with `#` because that is how kicli comments a
/// view (`# scope project  sheets=3 ...`). Read naively, a view sample becomes
/// a top-level heading and truncates the section it sits in.
fn sections(doc: &str) -> Vec<Section<'_>> {
    let mut found: Vec<Section<'_>> = Vec::new();
    let mut fenced = false;
    for line in doc.lines() {
        if line.trim_start().starts_with("```") {
            fenced = !fenced;
        }
        let heading = if fenced || !line.starts_with('#') {
            None
        } else {
            let level = line.chars().take_while(|c| *c == '#').count();
            (level <= 6 && line[level..].starts_with(' ')).then_some(level)
        };
        match heading {
            Some(level) => found.push(Section {
                level,
                title: line[level..].trim(),
                body: String::new(),
            }),
            None => {
                // The line belongs to the open section and to every ancestor of
                // it, so prose under a `####` subheading still counts towards
                // the `###` command section that contains it.
                let mut open = usize::MAX;
                for section in found.iter_mut().rev() {
                    if section.level >= open {
                        continue;
                    }
                    section.body.push_str(line);
                    section.body.push('\n');
                    open = section.level;
                    if open == 1 {
                        break;
                    }
                }
            }
        }
    }
    found
}

/// The backtick-delimited spans of a heading, which is how `AGENT.md` names a
/// command. Delimiting matters: without it a heading for `kicli wire draw-arc`
/// would answer for `kicli wire draw`.
fn code_spans(title: &str) -> Vec<&str> {
    title.split('`').skip(1).step_by(2).collect()
}

/// A command is documented when a heading names it and that heading has
/// something under it.
///
/// **A mention is not documentation.** This check used to assert that the
/// command's name appeared anywhere in the file, and the whole `kicli wire
/// draw` section could be deleted without it noticing, because `[routing]`
/// prose elsewhere named the verb (C7).
///
/// "Documented" here means what `AGENT.md` already does for every command it
/// covers: the name appears as its own backticked span in a heading, and the
/// section that heading opens says something. A heading may name several
/// commands — `kicli field move`, `kicli field rotate` and `kicli field
/// justify` share one, and share a body — so the rule is one heading *per
/// command name*, not one section per command.
#[test]
fn agent_doc_covers_every_command() {
    let doc = agent_doc();
    let sections = sections(&doc);
    let command = Cli::command();

    // A heading with nothing under it documents nothing. Measured on the
    // document as it stands, the smallest real command section is `kicli sym
    // delete` at 135 characters of body; a heading with a single sentence under
    // it lands near 50. The floor sits between the two, so it catches a stub
    // without demanding a word count of a genuinely terse command.
    const SUBSTANCE: usize = 80;

    let mut checked = 0;
    for noun in command.get_subcommands() {
        for verb in noun.get_subcommands() {
            let name = format!("kicli {} {}", noun.get_name(), verb.get_name());
            let heading = sections
                .iter()
                .find(|section| code_spans(section.title).contains(&name.as_str()));
            let Some(section) = heading else {
                panic!(
                    "AGENT.md has no heading naming `{name}`. A mention in \
                     prose is not documentation, and the name has to be its own \
                     backticked span: give the command a heading of its own, or \
                     add it to the backticked list of a heading it shares."
                );
            };
            let substance = section.body.split_whitespace().map(str::len).sum::<usize>();
            assert!(
                substance >= SUBSTANCE,
                "AGENT.md's heading for `{name}` has only {substance} characters \
                 under it, which documents nothing"
            );
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

/// The routing settings are written down, with every key an agent may set.
///
/// A weight an agent cannot find is a weight it cannot tune, and the cost
/// breakdown a route prints is built from exactly these. The list is written
/// out rather than read from the parser, because a list the parser supplied
/// would agree with the parser however wrong the document was.
#[test]
fn agent_doc_covers_the_routing_settings() {
    let doc = agent_doc();
    assert!(doc.contains("[routing]"), "AGENT.md has no routing section");
    for key in [
        "label_threshold",
        "margin",
        "u_max",
        "w_len",
        "w_turn",
        "w_cross",
        "w_text",
        "w_near",
    ] {
        assert!(
            doc.contains(key),
            "AGENT.md does not document routing.{key}"
        );
    }
    // The one knob two things read. An agent that changed it expecting only
    // the router to move would be surprised by the style rules later.
    assert!(
        doc.contains("one knob read twice"),
        "AGENT.md does not say that label_threshold is shared"
    );
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

/// The `kicad-cli` wait is documented in both places it can happen.
///
/// `project info` and `project check` both call `cli::tools::probe`, which
/// prints the font-cache note and then blocks on `kicad-cli --version`.
/// `AGENT.md` described the wait under `project check` alone, so a reader who
/// ran `project info` met a pause of up to two minutes that the document had
/// told them nothing about (dogfood D6). One section carries the explanation
/// and the other points at it; both have to name the tool they run.
#[test]
fn agent_doc_warns_about_the_kicad_cli_wait_in_both_places() {
    let doc = agent_doc();
    let sections = sections(&doc);
    let body = |name: &str| {
        sections
            .iter()
            .find(|section| code_spans(section.title).contains(&name))
            .unwrap_or_else(|| panic!("AGENT.md has a section for `{name}`"))
            .body
            .clone()
    };

    let info = body("kicli project info");
    assert!(
        info.contains("kicad-cli") && info.contains("font cache"),
        "`project info` runs kicad-cli and blocks on it exactly as \
         `project check` does, so its section has to say so"
    );

    let check = body("kicli project check");
    assert!(
        check.contains("kicad-cli"),
        "`project check` runs kicad-cli, and its section has to name it"
    );
    assert!(
        check.contains("project info"),
        "`project check`'s section defers to `project info`'s for what the note \
         is, so it has to say where to look"
    );
}
