//! A merge rule that lives only in the code is one the next reader argues with.
//!
//! The extractor names its rules, the specification names them beside what each
//! one does, and the research record names them beside the measurement each one
//! came from. This test holds the three lists together: same names, same count,
//! same order.

use kicli::connectivity::MERGE_RULES;
use std::path::{Path, PathBuf};

fn workspace() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The rule names of a numbered list, written `1. **name** — …`.
///
/// The list ends at the first line that is neither a numbered item nor its
/// indented continuation.
fn rule_names(text: &str, heading: &str) -> Vec<String> {
    let after = text
        .split_once(heading)
        .unwrap_or_else(|| panic!("the document carries the heading {heading}"))
        .1;
    let mut found = Vec::new();
    let mut started = false;
    for line in after.lines() {
        let numbered = line
            .split_once(". **")
            .filter(|(number, _)| number.chars().all(|digit| digit.is_ascii_digit()));
        if let Some((_, rest)) = numbered {
            started = true;
            let name = rest
                .split_once("**")
                .expect("a rule name is closed")
                .0
                .to_owned();
            found.push(name);
            continue;
        }
        let continued = line.trim().is_empty() || line.starts_with("   ");
        if started && !continued {
            break;
        }
    }
    found
}

#[test]
fn the_spec_and_the_extractor_name_the_same_rules() {
    let spec = std::fs::read_to_string(workspace().join("spec/SPEC.md")).expect("the spec reads");
    assert_eq!(
        rule_names(&spec, "### 7.1 View 1 — connectivity"),
        MERGE_RULES
    );
}

#[test]
fn the_research_record_and_the_extractor_name_the_same_rules() {
    let record = std::fs::read_to_string(workspace().join("research/representation.md"))
        .expect("the research record reads");
    assert_eq!(
        rule_names(&record, "### 3.2 Net construction rules"),
        MERGE_RULES
    );
}

#[test]
fn every_rule_has_a_note_carrying_its_evidence() {
    let notes = workspace().join("research/notes");
    let spec = std::fs::read_to_string(workspace().join("spec/SPEC.md")).expect("the spec reads");
    let list = spec
        .split_once("### 7.1 View 1 — connectivity")
        .expect("the spec carries the rule list")
        .1
        .split_once("**Connectivity is defined as")
        .expect("the rule list ends where the ruling begins")
        .0;
    let mut cited = 0;
    for reference in list.split("notes/").skip(1) {
        let Some((file, _)) = reference.split_once(".md") else {
            continue;
        };
        let path = notes.join(format!("{file}.md"));
        assert!(
            path.is_file(),
            "the spec cites {}, which is not there",
            path.display()
        );
        cited += 1;
    }
    assert!(
        cited >= MERGE_RULES.len(),
        "every rule cites the note its evidence lives in"
    );
}
