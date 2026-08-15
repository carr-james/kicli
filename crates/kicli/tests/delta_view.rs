//! The delta names what changed between two states, in a fixed order.
//!
//! The same pair of states must always produce the same bytes, so an agent can
//! compare two runs without reading the whole design again. The tests below
//! build a pair of sheets that differ by one move, one field edit, one added
//! symbol and one removed symbol.
//!
//! The last tests run the compiled binary, because the question the printed
//! delta answers — what has touched this file since kicli last wrote it? — can
//! only be asked of a file on disk that something else has touched.

use kicli::model::{Schematic, SheetPath};
use kicli::view::delta::Delta;
use kicli::view::snapshot::Snapshot;
use kicli_probe::scratch::Fixtures;
use kicli_sexpr::Doc;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// The root screen uuid both states share.
const ROOT: &str = "10000000-0000-4000-8000-000000000000";

/// A timestamp the test supplies, so no run reads the clock.
const TAKEN: &str = "2026-01-02T03:04:05Z";

/// One placed resistor, with the two fields KiCad always writes.
fn symbol(uuid: &str, reference: &str, value: &str, x: f64, y: f64) -> String {
    format!(
        "\t(symbol\n\
         \t\t(lib_id \"Test:R\")\n\
         \t\t(at {x:.4} {y:.4} 0)\n\
         \t\t(unit 1)\n\
         \t\t(uuid \"{uuid}\")\n\
         \t\t(property \"Reference\" \"{reference}\"\n\t\t\t(at {rx:.4} {y:.4} 90)\n\t\t)\n\
         \t\t(property \"Value\" \"{value}\"\n\t\t\t(at {x:.4} {vy:.4} 90)\n\t\t)\n\
         \t\t(instances\n\t\t\t(project \"\"\n\t\t\t\t(path \"/{ROOT}\"\n\
         \t\t\t\t\t(reference \"{reference}\")\n\t\t\t\t\t(unit 1)\n\t\t\t\t)\n\t\t\t)\n\t\t)\n\
         \t)\n",
        rx = x + 2.032,
        vy = y + 2.54,
    )
}

/// A sheet holding the symbols it is given.
fn sheet(symbols: &[String]) -> String {
    format!(
        "(kicad_sch\n\t(version 20260306)\n\t(uuid \"{ROOT}\")\n\t(paper \"A4\")\n{}\
         )\n",
        symbols.concat()
    )
}

fn snapshot(name: &str, source: &str) -> Snapshot {
    let doc = Doc::parse(source).expect("the sheet parses");
    let schematic = Schematic::read(&doc).expect("the sheet reads as a schematic");
    let path = SheetPath::root(schematic.uuid.as_ref().expect("the sheet has a uuid"));
    Snapshot::take(name, TAKEN, &path, &doc, &schematic).expect("the snapshot is taken")
}

const R1: &str = "11111111-1111-4111-8111-111111111111";
const R2: &str = "22222222-2222-4222-8222-222222222222";
const R7: &str = "77777777-7777-4777-8777-777777777777";
const R42: &str = "42424242-4242-4242-8242-424242424242";

fn base() -> String {
    sheet(&[
        symbol(R1, "R1", "10k", 50.8, 50.8),
        symbol(R2, "R2", "1k", 63.5, 50.8),
        symbol(R7, "R7", "4k7", 76.2, 50.8),
    ])
}

/// The base sheet with R1 moved, R2 revalued, R7 removed and R42 added.
fn changed() -> String {
    sheet(&[
        symbol(R1, "R1", "10k", 50.8, 63.5),
        symbol(R2, "R2", "2k2", 63.5, 50.8),
        symbol(R42, "R42", "10k", 88.9, 50.8),
    ])
}

#[test]
fn delta_distinguishes_moved_from_edited() {
    let before = snapshot("base", &base());
    let after = snapshot("current", &changed());
    let delta = Delta::between(&before, &after);

    assert_eq!(
        delta.to_string(),
        concat!(
            "delta base -> current\n",
            "~ L R1  moved  (50.80,50.80) -> (50.80,63.50)\n",
            "+ S R42 10k Test:R\n",
            "- S R7 4k7 Test:R\n",
            "~ S R2.Value  \"1k\" -> \"2k2\"\n",
            "= 4 objects unchanged\n",
        )
    );
    assert_eq!(delta.unchanged, 4);
    assert_eq!(delta.lines.len(), 4);
}

#[test]
fn the_same_pair_of_states_gives_the_same_bytes() {
    let before = snapshot("base", &base());
    let after = snapshot("current", &changed());
    assert_eq!(
        Delta::between(&before, &after).to_string(),
        Delta::between(&before, &after).to_string()
    );
}

#[test]
fn a_state_compared_with_itself_reports_only_the_count() {
    let taken = snapshot("base", &base());
    let delta = Delta::between(&taken, &taken);

    assert!(delta.lines.is_empty());
    assert_eq!(
        delta.to_string(),
        "delta base -> base\n= 9 objects unchanged\n"
    );
}

#[test]
fn the_fields_of_an_added_symbol_are_not_reported_twice() {
    let before = snapshot("base", &sheet(&[symbol(R1, "R1", "10k", 50.8, 50.8)]));
    let after = snapshot(
        "current",
        &sheet(&[
            symbol(R1, "R1", "10k", 50.8, 50.8),
            symbol(R42, "R42", "10k", 88.9, 50.8),
        ]),
    );
    let delta = Delta::between(&before, &after);

    assert_eq!(
        delta.to_string(),
        "delta base -> current\n+ S R42 10k Test:R\n= 3 objects unchanged\n"
    );
}

#[test]
fn a_delta_against_a_saved_state_reads_like_one_against_a_design() {
    // The file carries the display column as well as the hashes, so a
    // comparison against a saved state says the same thing as a comparison
    // against the design it came from. A delta that could only say "something
    // changed" would make the implicit snapshot after every mutation useless
    // for the one question it exists to answer.
    let before = Snapshot::parse(&snapshot("base", &base()).render()).expect("the file parses");
    let after = snapshot("current", &changed());

    let from_file = Delta::between(&before, &after).to_string();
    let from_design = Delta::between(&snapshot("base", &base()), &after).to_string();
    assert_eq!(
        from_file, from_design,
        "the file loses nothing a reader needs"
    );
    assert!(
        from_file.contains("- S R7 4k7 Test:R"),
        "a removed object is named, not just its identifier: {from_file}"
    );
}

/// The committed fixtures this binary reads, and the scratch it writes in.
fn fixtures() -> Fixtures {
    Fixtures::new(env!("CARGO_TARGET_TMPDIR"), env!("CARGO_MANIFEST_DIR"))
}

/// Run the binary in one project.
fn kicli(project: &Path, arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_kicli"))
        .args(arguments)
        .arg("-p")
        .arg(project)
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

/// The lines of a printed delta that name a changed object.
fn changes(printed: &str) -> Vec<&str> {
    printed
        .lines()
        .filter(|line| line.starts_with(['+', '-', '~']))
        .collect()
}

/// A project holding the base sheet, which kicli has written once.
///
/// The write is what leaves the saved state behind, so this is the state a
/// delta compares against.
fn project_kicli_has_written(name: &str) -> PathBuf {
    let directory = fixtures().scratch(name);
    std::fs::write(directory.join("board.kicad_sch"), base()).expect("the sheet is written");

    let run = kicli(
        &directory,
        &["label", "add", "--text", "SPY", "--at", "30.48,88.9"],
    );
    assert_eq!(code(&run), 0, "the write succeeds: {}", stderr(&run));
    directory
}

/// Change one value in the file, the way a person editing in KiCad does.
fn edit_the_file_behind_kiclis_back(project: &Path) {
    let path = project.join("board.kicad_sch");
    let source = std::fs::read_to_string(&path).expect("the sheet reads");
    let edited = source.replace("\"10k\"", "\"22k\"");
    assert_ne!(edited, source, "the outside edit changed something");
    std::fs::write(&path, edited).expect("the sheet is written");
}

#[test]
fn a_delta_right_after_a_mutation_reports_nothing_changed() {
    let project = project_kicli_has_written("delta_after_a_write");

    let run = kicli(&project, &["sch", "view", "--view", "delta"]);
    assert_eq!(code(&run), 0, "{}", stderr(&run));

    let printed = stdout(&run);
    assert!(
        changes(&printed).is_empty(),
        "the mutation's own result reported its changes, so this reports none: {printed}"
    );
    assert!(
        printed.contains("objects unchanged"),
        "and it says how much it compared: {printed}"
    );
}

#[test]
fn a_delta_reports_an_edit_made_outside_kicli() {
    // This is the control for the test above. Both read one project through
    // one command, so an implementation that reported nothing whatever the
    // state of the file would pass that test and fail this one.
    let project = project_kicli_has_written("delta_outside_edit");
    edit_the_file_behind_kiclis_back(&project);

    let run = kicli(&project, &["sch", "view", "--view", "delta"]);
    assert_eq!(code(&run), 0, "{}", stderr(&run));

    let printed = stdout(&run);
    assert_eq!(
        changes(&printed),
        vec!["~ S R1.Value  \"10k\" -> \"22k\""],
        "exactly the object the outside edit touched: {printed}"
    );
}

#[test]
fn a_delta_says_which_state_it_compared_and_how_much_that_state_holds() {
    let project = project_kicli_has_written("delta_states_its_form");

    let printed = stdout(&kicli(&project, &["sch", "view", "--view", "delta"]));
    let header = printed.lines().next().expect("the delta has a header");
    for fact in [
        "delta @last-write -> current",
        "scope=sheet",
        "compared=values",
    ] {
        assert!(header.contains(fact), "the header states {fact}: {header}");
    }
}

/// A state that holds only hashes says so, and is still read.
///
/// A snapshot written before the display column existed carries four columns.
/// A comparison against it can name the object that changed and no more, and
/// the caller is told that rather than left to assume it.
#[test]
fn a_delta_against_a_state_of_hashes_alone_says_so() {
    let project = project_kicli_has_written("delta_hashes_only");
    let saved = project.join(".kicli/snapshots/@last-write.snap");
    let text = std::fs::read_to_string(&saved).expect("the saved state reads");
    let stripped: String = text
        .lines()
        .map(|line| match line.starts_with("snapshot ") {
            true => format!("{line}\n"),
            false => format!(
                "{}\n",
                line.split(' ').take(4).collect::<Vec<_>>().join(" ")
            ),
        })
        .collect();
    assert_ne!(stripped, text, "the display column was there to remove");
    std::fs::write(&saved, stripped).expect("the saved state is written");
    edit_the_file_behind_kiclis_back(&project);

    let printed = stdout(&kicli(&project, &["sch", "view", "--view", "delta"]));
    let header = printed.lines().next().expect("the delta has a header");
    assert!(
        header.contains("compared=hashes"),
        "the header states what the state holds: {header}"
    );
    assert_eq!(
        changes(&printed),
        vec!["~ S R1.Value  edited"],
        "the object is named, and the old value is not there to print: {printed}"
    );
}

/// A comparison too large for the budget falls back, and says that it did.
#[test]
fn a_delta_larger_than_the_budget_falls_back_to_a_summary() {
    let project = project_kicli_has_written("delta_over_budget");
    // A re-layout in the editor: several objects moved, one gone, one new.
    std::fs::write(project.join("board.kicad_sch"), changed()).expect("the sheet is written");

    let full = stdout(&kicli(&project, &["sch", "view", "--view", "delta"]));
    assert!(!changes(&full).is_empty(), "there is something to print");

    let budget = full.len() - 1;
    std::fs::write(
        project.join("kicli.toml"),
        format!("[view]\nmax_bytes = {budget}\n"),
    )
    .expect("the configuration is written");

    let printed = stdout(&kicli(&project, &["sch", "view", "--view", "delta"]));
    assert!(
        printed.len() < full.len(),
        "the summary costs less: {printed}"
    );
    assert!(
        changes(&printed).is_empty(),
        "it prints no lines at all: {printed}"
    );
    for fact in [
        "scope=sheet-summary",
        &format!("full={}B budget={budget}B", full.len()),
        "raise view.max_bytes",
        "# added=",
    ] {
        assert!(printed.contains(fact), "it states {fact}: {printed}");
    }
}

/// The text form and the JSON form carry the same content.
#[test]
fn the_delta_json_twin_carries_what_the_text_carries() {
    let project = project_kicli_has_written("delta_json_twin");
    edit_the_file_behind_kiclis_back(&project);

    let printed = stdout(&kicli(&project, &["sch", "view", "--view", "delta"]));
    let run = kicli(
        &project,
        &["sch", "view", "--view", "delta", "--output", "json"],
    );
    assert_eq!(code(&run), 0, "{}", stderr(&run));
    let reported: serde_json::Value =
        serde_json::from_str(&stdout(&run)).expect("stdout is one JSON object");

    let header = printed.lines().next().expect("the delta has a header");
    for (key, value) in [
        ("from", "@last-write"),
        ("to", "current"),
        ("scope", "sheet"),
        ("compared", "values"),
    ] {
        assert_eq!(reported[key], value, "the JSON says {key}: {reported}");
        assert!(header.contains(value), "and so does the text: {header}");
    }

    let changed = reported["changed"]
        .as_array()
        .expect("the JSON lists what changed");
    let rebuilt: Vec<String> = changed.iter().map(text_line).collect();
    assert_eq!(
        changes(&printed),
        rebuilt,
        "the same lines, in the same order: {reported}"
    );

    let unchanged = reported["unchanged"].as_u64().expect("a count");
    assert!(
        printed.contains(&format!("= {unchanged} objects unchanged")),
        "and the same count: {printed}"
    );
}

/// One JSON change, written as the text form writes it.
///
/// A change to an object that stays gets two spaces, so a reader can tell the
/// description of a new object from the report of an edit.
fn text_line(change: &serde_json::Value) -> String {
    let field = |name: &str| {
        change[name]
            .as_str()
            .unwrap_or_else(|| panic!("a change carries {name}: {change}"))
            .to_owned()
    };
    let (mark, detail) = (field("change"), field("detail"));
    let named = format!("{mark} {} {}", field("record"), field("handle"));
    if detail.is_empty() {
        named
    } else if mark == "+" || mark == "-" {
        format!("{named} {detail}")
    } else {
        format!("{named}  {detail}")
    }
}
