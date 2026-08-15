//! One implementation of "how many wire ends meet here", and three questions on
//! it.
//!
//! `edit::mark` refuses a junction where four wire ends already meet.
//! `edit::wire` reports a junction a delete left joining too few. `route` moves
//! a terminus that would bring the fourth. Those are three thresholds on one
//! measurement, and the measurement is [`wire_ends_at`]. A second
//! implementation of it is a second answer waiting to disagree with the first,
//! and the disagreement would show up as a router that draws exactly the shape
//! the junction verb refuses.
//!
//! **Every absence check below carries a presence control.** A sweep that read
//! no files, or looked for a name nothing has, would report a clean workspace
//! and mean nothing by it. So the canonical implementation must be found, each
//! named consumer's call must be found, and the shape the absence check forbids
//! must be found where it belongs.
//!
//! [`wire_ends_at`]: kicli::edit::mark

use std::path::{Path, PathBuf};

/// How the one implementation is written where it is defined.
const DEFINITION: &str = "fn wire_ends_at(";

/// How it is written where it is called.
const CALL: &str = "wire_ends_at(";

/// Where the one implementation lives.
const HOME: &str = "crates/kicli/src/edit/mark.rs";

/// Every module that must answer this question by calling the one
/// implementation, rather than by counting for itself.
///
/// The junction verb's own refusal, the wire verb's stranded-junction report,
/// and the router's four-way avoidance.
const CONSUMERS: [&str; 3] = [
    "crates/kicli/src/edit/mark.rs",
    "crates/kicli/src/edit/wire.rs",
    "crates/kicli/src/route/terminal.rs",
];

/// The shape a hand-rolled count of the ends at a point takes: a line's two
/// ends, each compared against the point.
///
/// A file offends only by carrying both, because either alone is an ordinary
/// comparison.
const SHAPE: [&str; 2] = [".from ==", ".to =="];

/// Every source of every crate.
fn sources(root: &Path, found: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            sources(&path, found);
        } else if path.extension().is_some_and(|end| end == "rs") {
            found.push(path);
        }
    }
}

fn workspace() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("the workspace root resolves")
}

/// Every source of the workspace's crates, sorted, with the text of each.
fn crate_sources() -> Vec<(String, String)> {
    let workspace = workspace();
    let mut files = Vec::new();
    sources(&workspace.join("crates"), &mut files);
    files.retain(|file| file.components().any(|part| part.as_os_str() == "src"));
    files.sort();
    // The control on the sweep itself: a walk that found nothing would report a
    // workspace with one implementation of everything.
    assert!(
        files.len() > 20,
        "the sources of this workspace were found, not {}",
        files.len()
    );
    files
        .iter()
        .map(|file| {
            let name = file
                .strip_prefix(&workspace)
                .unwrap_or(file)
                .display()
                .to_string();
            let text = std::fs::read_to_string(file).expect("a source reads");
            (name, text)
        })
        .collect()
}

/// The lines of a file that hold a fragment, outside its doc comments.
///
/// A rustdoc paragraph naming the function is prose about the rule, not a use
/// of it, and counting one would let a module claim to call what it only talks
/// about.
fn code_lines<'a>(text: &'a str, fragment: &str) -> Vec<&'a str> {
    text.lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .filter(|line| line.contains(fragment))
        .collect()
}

#[test]
fn the_four_end_count_is_implemented_once() {
    let sources = crate_sources();
    let defining: Vec<&String> = sources
        .iter()
        .filter(|(_, text)| !code_lines(text, DEFINITION).is_empty())
        .map(|(name, _)| name)
        .collect();

    // The presence control. A sweep for a name nothing carries would find one
    // implementation of it in exactly the same way it finds none.
    assert!(
        defining.iter().any(|name| name.replace('\\', "/") == HOME),
        "the one implementation is where it is documented to be: {defining:?}"
    );
    assert_eq!(
        defining.len(),
        1,
        "one implementation of the wire-end count, and these define it: {defining:?}"
    );
}

#[test]
fn the_mark_verb_and_the_router_call_the_same_one() {
    let sources = crate_sources();
    for consumer in CONSUMERS {
        let (_, text) = sources
            .iter()
            .find(|(name, _)| name.replace('\\', "/") == consumer)
            .unwrap_or_else(|| panic!("{consumer} is a source of this workspace"));
        let calls: Vec<&str> = code_lines(text, CALL)
            .into_iter()
            .filter(|line| !line.contains(DEFINITION))
            .collect();
        assert!(
            !calls.is_empty(),
            "{consumer} asks the one implementation rather than counting for itself"
        );
    }
}

#[test]
fn nothing_else_counts_the_ends_at_a_point_for_itself() {
    let sources = crate_sources();
    let carries = |text: &str| SHAPE.iter().all(|part| !code_lines(text, part).is_empty());

    // The presence control, first: the shape the sweep forbids elsewhere is
    // what the one implementation is written in, so a sweep looking for a shape
    // no code has cannot pass silently.
    let (_, home) = sources
        .iter()
        .find(|(name, _)| name.replace('\\', "/") == HOME)
        .expect("the one implementation is a source of this workspace");
    assert!(
        carries(home),
        "{HOME} is written in the shape this check forbids elsewhere"
    );

    let offenders: Vec<&String> = sources
        .iter()
        .filter(|(name, _)| name.replace('\\', "/") != HOME)
        .filter(|(_, text)| carries(text))
        .map(|(name, _)| name)
        .collect();
    assert!(
        offenders.is_empty(),
        "these count a point's wire ends themselves instead of asking {HOME}: {offenders:?}"
    );
}
