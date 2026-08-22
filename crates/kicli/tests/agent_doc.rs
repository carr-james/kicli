//! The agent document and the binary say the same thing.
//!
//! A command that is not written down is a command an agent cannot use, and a
//! document that describes a command the binary does not have is worse than no
//! document. This test reads both and compares them, so neither can drift.

use clap::{CommandFactory, Parser};
use kicli::cli::{Cli, ExitCode};
use kicli::model::{Schematic, SheetPath};
use kicli::view::delta::{Change, Delta, DeltaLine};
use kicli::view::snapshot::{ObjectKind, Snapshot};
use kicli_sexpr::Doc;
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

/// Every command line the document shows is one the binary accepts.
///
/// The synopsis blocks state a shape — `kicli wire delete <TARGET>` — and the
/// worked examples state a command someone can run. This test reads the second
/// kind and hands each one to the parser, so an example that names a flag the
/// binary does not have, or spells one the way an older build did, fails here
/// rather than in an agent's terminal.
///
/// The two kinds are told apart by their placeholders: a synopsis carries `<`
/// or the `|` that separates the forms of an end, and a runnable line carries
/// neither. That rule is why `kicli sch view | grep -E 'R99|SPY'` is out of
/// scope — it is a pipeline, and this test parses commands, not shells.
fn documented_commands(doc: &str) -> Vec<Vec<String>> {
    let mut found = Vec::new();
    let mut fenced = false;
    let mut joined = String::new();
    for line in doc.lines() {
        if line.trim_start().starts_with("```") {
            fenced = !fenced;
            joined.clear();
            continue;
        }
        if !fenced {
            continue;
        }
        // A command may be wrapped over several lines with a trailing `\`.
        let text = line.trim();
        joined.push_str(text.strip_suffix('\\').unwrap_or(text));
        if text.ends_with('\\') {
            joined.push(' ');
            continue;
        }
        let command = std::mem::take(&mut joined);
        if !command.starts_with("kicli ") || command.contains('<') || command.contains('|') {
            continue;
        }
        found.push(
            command
                .split_whitespace()
                .map(|word| word.trim_matches(['\'', '"']).to_owned())
                .collect(),
        );
    }
    found
}

/// The document's worked examples parse, flags and all.
#[test]
fn agent_doc_examples_are_commands_the_binary_accepts() {
    let doc = agent_doc();
    let examples = documented_commands(&doc);

    // A rule that stopped matching would otherwise pass by finding nothing.
    // Measured on the document as it stands: 12 runnable lines.
    assert!(
        examples.len() >= 8,
        "only {} runnable example(s) found in AGENT.md, so the rule that \
         recognises one has stopped working",
        examples.len()
    );

    for words in &examples {
        if let Err(why) = Cli::try_parse_from(words) {
            panic!(
                "AGENT.md shows a command the binary does not accept:\n  {}\n{why}",
                words.join(" ")
            );
        }
    }
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

// ---------------------------------------------------------------------------
// The worked examples are measured output.
//
// `tasks/M5/RULES.md`, "Worked examples in `AGENT.md` are measured output".
// ---------------------------------------------------------------------------

/// How many record lines the document holds today, measured.
///
/// A floor rather than an equality: adding a worked example must not fail the
/// build, and **removing** one must. The number is the count this test found on
/// the document at the commit that added it, so a rule that stopped recognising
/// a line cannot pass by finding nothing.
const RECORD_EXAMPLES: usize = 20;

/// How many of those this test rebuilds through the tool's own pipeline.
const REBUILT_EXAMPLES: usize = 14;

/// The root sheet uuid every rebuilt sheet uses.
const ROOT_SHEET: &str = "aa000000-0000-4000-8000-00000000000f";

/// A timestamp the test supplies, so no run reads the clock.
const TAKEN_AT: &str = "2026-01-02T03:04:05Z";

/// Every record line `AGENT.md` shows inside a fenced block, with its line
/// number and with a shell comment marker taken off the front.
///
/// A candidate is a line whose first character is a mark **the writer can
/// produce** — the set comes from [`Change::ALL`], not from a list spelled out
/// here — followed by a space, one upper-case letter, and either a space or the
/// end of the line. That is [`DeltaLine`]'s own frame; anything else in a fence
/// is a shell command, a JSON body or prose and is not this test's business.
///
/// The session walkthrough shows each command's answer as a shell comment
/// under the command — `# + T da5aa983 "SPY"` — so a leading `# ` is stripped
/// before the frame is looked for. Without that, four worked examples sat in
/// the document unchecked because of two characters.
fn record_examples(doc: &str) -> Vec<(usize, &str)> {
    let mut found = Vec::new();
    let mut fenced = false;
    for (index, raw) in doc.lines().enumerate() {
        if raw.trim_start().starts_with("```") {
            fenced = !fenced;
            continue;
        }
        if !fenced {
            continue;
        }
        let line = raw.strip_prefix("# ").unwrap_or(raw);
        let mut characters = line.chars();
        let Some(mark) = characters.next() else {
            continue;
        };
        if !Change::ALL.iter().any(|change| change.mark() == mark) {
            continue;
        }
        if characters.next() != Some(' ') {
            continue;
        }
        if !characters.next().is_some_and(|c| c.is_ascii_uppercase()) {
            continue;
        }
        if !matches!(characters.next(), None | Some(' ')) {
            continue;
        }
        found.push((index + 1, line));
    }
    found
}

/// What the writer puts between a handle and a detail, asked of the writer.
///
/// [`DeltaLine`] separates the two with one space after a `+` or a `-` and with
/// two after a `~`, so that a reader can tell the description of a new object
/// from the report of an edit. This function measures that rather than
/// restating it: it emits a line with a one-character handle and a
/// one-character detail and reads back what came out between them.
fn separator(change: Change) -> String {
    let probe = DeltaLine {
        change,
        // `Display` never reads the kind. It is here because the struct has the
        // field, and the record letter — which `Display` does read — is its own
        // field beside it.
        kind: ObjectKind::Wire,
        record: 'Z',
        handle: "H".to_owned(),
        detail: "D".to_owned(),
    };
    let emitted = probe.to_string();
    let head = format!("{} Z H", change.mark());
    emitted
        .strip_prefix(&head)
        .and_then(|rest| rest.strip_suffix('D'))
        .unwrap_or_else(|| {
            panic!("DeltaLine no longer emits `<mark> <record> <handle><sep><detail>`: {emitted}")
        })
        .to_owned()
}

/// Take a documented line apart into the pieces the writer puts together.
///
/// **Nothing here decides what a record line may look like.** The function
/// proposes one decomposition — the mark, the record letter, the first
/// whitespace-free token as the handle, the rest as the detail — hands it back
/// to [`DeltaLine`]'s own `Display`, and succeeds only when what comes out is
/// the documented line byte for byte. A grammar written beside the writer would
/// be this test author's vocabulary wearing a reference; a round trip through
/// the writer is the writer's.
///
/// Two properties of the writer make the decomposition unambiguous, and both
/// are asserted rather than assumed:
///
/// * a handle carries no space — it is a uuid's first eight characters, a
///   reference designator, or a reference and a field name joined by a full
///   stop;
/// * a detail never begins with whitespace — all four producers in `detail()`
///   start with a value, the word `moved`, or a quote.
///
/// The second is what gives the separator rule teeth: without it a detail would
/// swallow a stray space and a `+` line with two spaces would pass.
fn decompose(text: &str) -> Result<(Change, char, String, String), String> {
    let mut characters = text.chars();
    let mark = characters.next().ok_or("the line is empty")?;
    let change = Change::ALL
        .into_iter()
        .find(|change| change.mark() == mark)
        .ok_or_else(|| format!("`{mark}` is not a mark any Change produces"))?;
    if characters.next() != Some(' ') {
        return Err("the mark is not followed by a single space".to_owned());
    }
    let record = characters.next().ok_or("the line stops after the mark")?;
    let rest = match characters.next() {
        None => "",
        Some(' ') => characters.as_str(),
        Some(other) => return Err(format!("`{other}` follows the record letter, not a space")),
    };

    let handle = rest.split(' ').next().unwrap_or("").to_owned();
    if handle.is_empty() {
        return Err("the record letter is followed by a space and no handle".to_owned());
    }
    let after = &rest[handle.len()..];
    let wanted = separator(change);
    let detail = if after.is_empty() {
        String::new()
    } else {
        after
            .strip_prefix(wanted.as_str())
            .ok_or_else(|| {
                format!(
                    "the writer puts {} space(s) between a `{mark}` line's handle \
                     and its detail, and this line has `{after}`",
                    wanted.len()
                )
            })?
            .to_owned()
    };
    if detail.starts_with(char::is_whitespace) {
        return Err(format!(
            "the detail `{detail}` begins with whitespace, which no producer in \
             delta.rs's `detail()` can emit"
        ));
    }

    let rebuilt = DeltaLine {
        change,
        kind: ObjectKind::Wire,
        record,
        handle: handle.clone(),
        detail: detail.clone(),
    };
    if rebuilt.to_string() != text {
        return Err(format!(
            "the writer emits `{rebuilt}` for these pieces, not `{text}`"
        ));
    }
    Ok((change, record, handle, detail))
}

/// Every numeric literal in a string, in order, at most one point each.
///
/// `..` separates the two ends of a segment summary, so a scanner that let a
/// number carry two points would read `50.80..63.50` as one.
fn numbers(text: &str) -> Vec<&str> {
    let bytes = text.as_bytes();
    let mut found = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        let start = index;
        if bytes[index] == b'-' {
            index += 1;
        }
        let digits = index;
        let mut point = false;
        while index < bytes.len() {
            match bytes[index] {
                b'0'..=b'9' => index += 1,
                b'.' if !point => {
                    point = true;
                    index += 1;
                }
                _ => break,
            }
        }
        if index > digits && bytes[digits].is_ascii_digit() {
            found.push(text[start..index].trim_end_matches('.'));
        } else {
            index = start + 1;
        }
    }
    found
}

/// A sheet holding one item, written the way KiCad writes one.
fn one_item_sheet(item: &str) -> String {
    format!(
        "(kicad_sch\n\t(version 20260306)\n\t(uuid \"{ROOT_SHEET}\")\n\t(paper \"A4\")\n{item})\n"
    )
}

/// The snapshot of one sheet.
fn taken(name: &str, source: &str) -> Snapshot {
    let doc = Doc::parse(source).expect("the sheet parses");
    let schematic = Schematic::read(&doc).expect("the sheet reads as a schematic");
    let path = SheetPath::root(schematic.uuid.as_ref().expect("the sheet has a uuid"));
    Snapshot::take(name, TAKEN_AT, &path, &doc, &schematic).expect("the snapshot is taken")
}

/// The line the tool prints when one item arrives, or leaves.
///
/// This is the whole stack an agent's own run goes through: the reader, the
/// model, the snapshot and the comparison. Nothing about the printed line is
/// predicted here.
fn line_for(change: Change, item: &str) -> String {
    let empty = taken("base", &one_item_sheet(""));
    let full = taken("current", &one_item_sheet(item));
    let delta = match change {
        Change::Added => Delta::between(&empty, &full),
        Change::Removed => Delta::between(&full, &empty),
        other => panic!("only an addition or a removal is rebuilt from one item, not {other:?}"),
    };
    assert_eq!(
        delta.lines.len(),
        1,
        "one item makes one line, and this made {}: {delta}",
        delta.lines.len()
    );
    delta.lines[0].to_string()
}

/// A uuid whose first eight characters are the documented handle.
fn uuid_for(handle: &str) -> String {
    format!("{handle}-0000-4000-8000-000000000001")
}

/// The document's record examples are lines the writer can emit.
///
/// # What this covers
///
/// Every line inside a fenced block that carries [`DeltaLine`]'s frame: the
/// mark, the record letter, the handle, and the spacing between them. Each one
/// is taken apart and handed back to the writer, and the writer has to produce
/// the documented bytes. A mark no `Change` makes, a missing space, the one-
/// space separator on a `~` line or the two-space separator on a `+` line all
/// fail here.
///
/// # What it does not cover, and why the boundary sits there
///
/// **The content of the handle and of the detail.** This test knows the frame
/// only, so `+ W abcd1234 (50.80,50.80) -> (63.50,50.80)` — the shape of
/// dogfood defect D3 — passes it. That defect is caught one test down, in
/// [`agent_doc_wire_and_text_examples_are_what_the_writer_emits`], which
/// rebuilds the object and compares the whole line. Splitting the two is
/// deliberate: the frame belongs to every record kind, and the content can only
/// be rebuilt for the kinds a documented line carries enough to rebuild.
///
/// **The record letter.** `record_of` decides it from an `ObjectKind`, and a
/// test that listed the kinds to ask it about would be listing them from
/// memory. The letter *is* checked wherever a line is rebuilt, because there it
/// comes out of `record_of` on a real comparison.
#[test]
fn agent_doc_record_examples_are_lines_the_writer_can_emit() {
    let doc = agent_doc();
    let examples = record_examples(&doc);

    assert!(
        examples.len() >= RECORD_EXAMPLES,
        "AGENT.md holds {} record example line(s) and held {RECORD_EXAMPLES} when \
         this test was written, so either an example was deleted or the rule \
         that recognises one has stopped working",
        examples.len()
    );

    for (number, text) in &examples {
        if let Err(why) = decompose(text) {
            panic!(
                "AGENT.md line {number} is not a line kicli's delta writer can \
                 emit:\n  {text}\n{why}"
            );
        }
    }
}

/// The document's wire and text examples are what the writer actually emits.
///
/// Each one is rebuilt: a sheet is written holding exactly the object the line
/// describes, a snapshot is taken of it and of the same sheet without it, and
/// the comparison's own line has to equal the documented line byte for byte.
/// The separator between two ends, the two decimal places, the sorting of the
/// ends and the record letter all come out of the tool.
///
/// # What this covers, and what it does not
///
/// **Added and removed `W` and `T` lines only** — a wire or bus segment, a
/// junction, no-connect or bus entry, a label of any kind, free text or a text
/// box. The boundary is not the record letter: it is whether the documented
/// line carries **enough to rebuild the object**. It does for these, and it
/// does not for the rest:
///
/// * `S` — a symbol's summary is `<value> <lib_id>`, and both may hold a space,
///   so splitting one is a grammar rather than a reconstruction;
/// * `L`, `F`, and every `~` line — the detail is a pair of states, and
///   rebuilding the pair needs the object to exist in both, which one sheet
///   with one item cannot express;
/// * `H`, `P` — a child sheet and its pins need a second file.
///
/// Those lines keep the frame check above and nothing more. Naming where this
/// stops is the point: extending it a kind at a time has no fixed point, and a
/// matcher one kind behind reads as covering what it does not.
///
/// **The five `T` kinds are indistinguishable.** A local, global or
/// hierarchical label, a netclass flag, free text and a text box all print `T`
/// with the text quoted, so a `T` line is rebuilt as a local label and the test
/// cannot say which kind the document meant. The same holds for `W`: a bus
/// segment prints exactly as a wire does.
///
/// **Agreement is not provenance.** This test asserts the document and the tool
/// say the same thing. It cannot tell a block regenerated from a real run from
/// one hand-written to be self-consistent — that is what `RULES.md` asks of the
/// person editing the block, and it is not decidable from the bytes.
#[test]
fn agent_doc_wire_and_text_examples_are_what_the_writer_emits() {
    let doc = agent_doc();
    let mut rebuilt = 0;

    for (number, text) in record_examples(&doc) {
        let (change, record, handle, detail) =
            decompose(text).unwrap_or_else(|why| panic!("AGENT.md line {number}: {text}\n{why}"));
        if !matches!(change, Change::Added | Change::Removed) {
            continue;
        }
        if !matches!(record, 'W' | 'T') {
            continue;
        }
        assert_eq!(
            handle.chars().count(),
            8,
            "AGENT.md line {number} names the handle `{handle}`, and a handle a \
             `W` or `T` line carries is a uuid's first eight characters"
        );
        let uuid = uuid_for(&handle);

        let item = if record == 'W' {
            let ends = numbers(&detail);
            match ends.as_slice() {
                [x1, y1, x2, y2] => format!(
                    "\t(wire\n\t\t(pts\n\t\t\t(xy {x1} {y1})\n\t\t\t(xy {x2} {y2})\n\t\t)\n\
                     \t\t(uuid \"{uuid}\")\n\t)\n"
                ),
                [x, y] => format!("\t(junction\n\t\t(at {x} {y})\n\t\t(uuid \"{uuid}\")\n\t)\n"),
                other => panic!(
                    "AGENT.md line {number} is a `W` line whose detail holds {} \
                     number(s):\n  {text}\nA segment holds four and a point item \
                     two, so this is neither.",
                    other.len()
                ),
            }
        } else {
            let quoted = detail
                .strip_prefix('"')
                .and_then(|rest| rest.strip_suffix('"'))
                .unwrap_or_else(|| {
                    panic!(
                        "AGENT.md line {number} is a `T` line whose detail is not \
                         a quoted string:\n  {text}"
                    )
                });
            assert!(
                !quoted.contains(['"', '\\']),
                "AGENT.md line {number} holds a quote or a backslash inside its \
                 text, which this test cannot write back into a sheet:\n  {text}"
            );
            format!("\t(label \"{quoted}\"\n\t\t(at 50.8 50.8 0)\n\t\t(uuid \"{uuid}\")\n\t)\n")
        };

        let emitted = line_for(change, &item);
        assert_eq!(
            emitted, *text,
            "AGENT.md line {number} shows a record kicli does not write.\n  \
             document: {text}\n  kicli:    {emitted}"
        );
        rebuilt += 1;
    }

    assert!(
        rebuilt >= REBUILT_EXAMPLES,
        "{rebuilt} example(s) were rebuilt through the writer and \
         {REBUILT_EXAMPLES} were when this test was written, so either an \
         example was deleted or this test has stopped reaching them"
    );
}
