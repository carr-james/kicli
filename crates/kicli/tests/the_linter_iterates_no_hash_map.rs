//! The linter's collections are ordered, and that is checked rather than
//! intended.
//!
//! A hash map hands its contents back in an order that depends on a seed the
//! standard library picks per map, so two runs of one program can walk the same
//! map two ways. A rule that walked one would report its findings in an order
//! the drawing does not decide, and re-scoring an unchanged file would stop
//! being bit-identical. `BTreeMap` and a sorted vector cost nothing at the
//! sizes a sheet holds, and they cannot do that.
//!
//! The sweep bans the names rather than the iteration, because a lookup-only
//! hash map is one refactor away from being walked, and no reader of a diff
//! reliably notices that refactor. Clippy has no lint for this, so the sweep is
//! the enforcement.

use std::path::{Path, PathBuf};

/// The collections the linter may not name.
const FORBIDDEN: [&str; 2] = ["HashMap", "HashSet"];

/// A type every linter source names, which the sweep expects to find.
const PRESENT: &str = "Finding";

/// Every source file of the linter, including the rule files.
fn linter_sources() -> Vec<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/lint");
    let mut found = vec![root.with_extension("rs")];
    collect(&root, &mut found);
    found.sort();
    found
}

/// Add every `.rs` file under a directory, however deep.
fn collect(dir: &Path, found: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect(&path, found);
        } else if path.extension().is_some_and(|end| end == "rs") {
            found.push(path);
        }
    }
}

/// Is this a whole word of the source, rather than part of a longer name?
fn names_type(text: &str, wanted: &str) -> bool {
    text.match_indices(wanted).any(|(start, _)| {
        let before = text[..start].chars().next_back();
        let after = text[start + wanted.len()..].chars().next();
        let part_of_a_name =
            |letter: Option<char>| letter.is_some_and(|c| c.is_alphanumeric() || c == '_');
        !part_of_a_name(before) && !part_of_a_name(after)
    })
}

#[test]
fn no_unordered_collection_appears_under_the_linter() {
    let sources = linter_sources();
    assert!(sources.len() >= 5, "the linter's sources were found");

    let mut offenders = Vec::new();
    for source in &sources {
        let text = std::fs::read_to_string(source).expect("a linter source reads");
        for forbidden in FORBIDDEN {
            if names_type(&text, forbidden) {
                offenders.push(format!(
                    "{}: {forbidden}",
                    source.file_name().unwrap_or_default().to_string_lossy()
                ));
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "the linter's collections must be ordered: {offenders:?}"
    );
}

#[test]
fn the_sweep_can_see_what_it_is_looking_for() {
    // The control. A sweep that read nothing, or that matched nothing it was
    // given, would pass the check above while a hash map sat in a rule.
    let sources = linter_sources();
    let text: String = sources
        .iter()
        .map(|source| std::fs::read_to_string(source).expect("a linter source reads"))
        .collect();
    assert!(!text.is_empty(), "the linter's sources were read");
    assert!(
        names_type(&text, PRESENT),
        "the linter names {PRESENT}, so the sweep is reading the linter"
    );

    // And the matcher itself: it finds a collection where one is named, and is
    // not fooled by a name that merely contains the letters.
    assert!(names_type("use std::collections::HashMap;", "HashMap"));
    assert!(names_type("let seen: HashSet<Uuid> = ...", "HashSet"));
    assert!(names_type("(HashMap::new())", "HashMap"));
    assert!(!names_type("struct HashMapOfFindings;", "HashMap"));
    assert!(!names_type("self.my_HashMap_thing", "HashMap"));
}
