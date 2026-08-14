//! The router and the long-wire rule read one key, under one name.
//!
//! A router that proposes labels above one distance while the score penalises
//! long wires above another argues with itself. The resolution was to make them
//! one configuration key, `routing.label_threshold`, and one field. This test
//! is the enforcement: a second threshold, or a second name for this one,
//! fails here rather than in the milestone that finally reads both.
//!
//! `tasks/` is out of scope. A task file records what a name was corrected
//! from, which is how it explains the correction.

use std::path::{Path, PathBuf};

/// The one name a threshold may have.
const CANONICAL: &str = "label_threshold";

/// A name this rule has already refused once.
const REFUSED: &str = "distance_threshold";

/// Every source and document that describes or reads the configuration.
fn scanned(workspace: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    for tree in ["crates", "spec", "research"] {
        walk(&workspace.join(tree), &mut found);
    }
    found.push(workspace.join("AGENT.md"));
    found.sort();
    found
}

fn walk(root: &Path, found: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk(&path, found);
        } else if path
            .extension()
            .is_some_and(|end| end == "rs" || end == "md" || end == "toml")
        {
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

#[test]
fn no_second_name_for_the_threshold_survives() {
    let workspace = workspace();
    let files = scanned(&workspace);
    assert!(files.len() > 30, "the sources were found: {}", files.len());

    let mut carriers = Vec::new();
    let mut offenders = Vec::new();
    for file in &files {
        // This file names both, which is how it forbids one of them.
        if file.ends_with("the_label_threshold_has_one_name.rs") {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(file) else {
            continue;
        };
        let named = file
            .strip_prefix(&workspace)
            .unwrap_or(file)
            .display()
            .to_string();
        if text.contains(REFUSED) {
            offenders.push(named.clone());
        }
        if text.contains(CANONICAL) {
            carriers.push(named);
        }
    }

    // The control: the search must find the name that is there, or finding
    // nothing would prove nothing about the name that must not be.
    assert!(
        carriers.len() >= 3,
        "the canonical name was found where it belongs: {carriers:?}"
    );
    assert!(
        offenders.is_empty(),
        "{REFUSED} is a second name for {CANONICAL}: {offenders:?}"
    );
}

#[test]
fn the_configuration_holds_exactly_one_threshold() {
    let workspace = workspace();
    let mut sources = Vec::new();
    walk(&workspace.join("crates/kicli/src"), &mut sources);

    let mut fields = Vec::new();
    for file in &sources {
        if file.extension().is_some_and(|end| end != "rs") {
            continue;
        }
        let text = std::fs::read_to_string(file).expect("a source reads");
        for line in text.lines().map(str::trim) {
            // A struct field, as `pub label_threshold: Iu,`.
            let Some(rest) = line.strip_prefix("pub ") else {
                continue;
            };
            let Some((name, _)) = rest.split_once(':') else {
                continue;
            };
            if name.ends_with("threshold") {
                fields.push(name.to_owned());
            }
        }
    }
    assert_eq!(
        fields,
        vec![CANONICAL.to_owned()],
        "one threshold, read by the router and by the long-wire rule"
    );
}
