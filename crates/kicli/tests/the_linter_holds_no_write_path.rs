//! The linter reads. It never writes, and it never reaches outside the model.
//!
//! Scoring must never mutate a file, and a rule must know nothing of the
//! command line, the disk, or `kicad-cli`. A rule suggests a command as text
//! and stops. Nothing in the language enforces that, so this sweep does.
//!
//! **The sweep is a whitelist, and that is the point.** A list of forbidden
//! names is only as good as the author's memory of what writes: `std::fs::write`
//! is remembered and `std::fs::OpenOptions` is not. A list of *permitted*
//! module paths fails the other way. Forgetting an entry makes the sweep
//! stricter, never blinder, and the lane that needs the missing entry adds it
//! with a reason beside it.
//!
//! Three lists, and each is small enough to read:
//!
//! - which of this crate's modules the linter may name;
//! - which of the standard library's modules it may name, which is the
//!   arithmetic half and none of the input or output half;
//! - which other crates it may depend on at all.
//!
//! The sweep reads code rather than prose: comments are removed first, because
//! a paragraph explaining that the linter never writes would otherwise fail it.
//! String literals are **not** removed, which is the sweep's stated boundary.

use std::path::{Path, PathBuf};

/// The modules of this crate the linter may name.
///
/// `model::items` and `model::library` are the typed objects and the embedded
/// symbol cache. `model::write`, `model::write_file` and `model::mutate` are
/// the write path, and their absence from this list is the rule.
const CRATE_PATHS: [&str; 5] = [
    "geometry",
    "lint",
    "connectivity",
    "model::items",
    "model::library",
];

/// The standard library modules the linter may name.
///
/// Everything here computes. `fs`, `io`, `path`, `process`, `net`, `env`,
/// `time`, `thread`, `os` and `ffi` are not here, and that is the whole
/// prohibition: no disk, no other process, no clock, no environment.
const STD_MODULES: [&str; 20] = [
    "array",
    "borrow",
    "char",
    "cmp",
    "collections",
    "convert",
    "fmt",
    "hash",
    "iter",
    "marker",
    "mem",
    "num",
    "ops",
    "option",
    "primitive",
    "result",
    "slice",
    "str",
    "string",
    "vec",
];

/// The roots a `use` in the linter may name.
///
/// `kicli_sexpr` is the token layer and knows nothing of schematics or of the
/// disk. Every other dependency of this crate serves the command line, the
/// configuration file or the project file, and none of them belongs here.
const USE_ROOTS: [&str; 6] = ["crate", "std", "core", "self", "super", "kicli_sexpr"];

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

/// The text with its comments removed, so the sweep reads code.
fn code_of(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let bytes: Vec<char> = text.chars().collect();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == '/' && bytes.get(index + 1) == Some(&'/') {
            while index < bytes.len() && bytes[index] != '\n' {
                index += 1;
            }
        } else if bytes[index] == '/' && bytes.get(index + 1) == Some(&'*') {
            index += 2;
            while index < bytes.len()
                && !(bytes[index] == '*' && bytes.get(index + 1) == Some(&'/'))
            {
                index += 1;
            }
            index = (index + 2).min(bytes.len());
        } else {
            out.push(bytes[index]);
            index += 1;
        }
    }
    out
}

/// Read one path segment starting at `from`, and where it ended.
fn segment(letters: &[char], from: usize) -> (String, usize) {
    let mut end = from;
    while end < letters.len() && (letters[end].is_alphanumeric() || letters[end] == '_') {
        end += 1;
    }
    (letters[from..end].iter().collect(), end)
}

/// Every path in the code that begins with `root::`, to two segments deep.
fn paths_under(code: &str, root: &str) -> Vec<String> {
    let letters: Vec<char> = code.chars().collect();
    let opener = format!("{root}::");
    let mut found = Vec::new();
    for (start, _) in code.match_indices(&opener) {
        // Character indices, because the source may hold any character.
        let start = code[..start].chars().count() + opener.chars().count();
        let (first, after) = segment(&letters, start);
        let mut path = first;
        if letters.get(after) == Some(&':') && letters.get(after + 1) == Some(&':') {
            let (second, _) = segment(&letters, after + 2);
            path = format!("{path}::{second}");
        }
        found.push(path);
    }
    found
}

/// The root crate of every `use` in the code.
fn use_roots(code: &str) -> Vec<String> {
    let mut found = Vec::new();
    for line in code.lines() {
        let line = line.trim();
        let Some(rest) = line.strip_prefix("use ") else {
            continue;
        };
        let letters: Vec<char> = rest.chars().collect();
        let (root, _) = segment(&letters, 0);
        if !root.is_empty() {
            found.push(root);
        }
    }
    found
}

/// Everything in one source that the linter may not name.
fn offences(text: &str) -> Vec<String> {
    let code = code_of(text);
    let mut offences = Vec::new();

    for path in paths_under(&code, "crate") {
        let allowed = CRATE_PATHS
            .iter()
            .any(|permitted| path == *permitted || path.starts_with(&format!("{permitted}::")));
        if !allowed {
            offences.push(format!("crate::{path}"));
        }
    }
    for path in paths_under(&code, "std") {
        let module = path.split("::").next().unwrap_or_default().to_owned();
        if !STD_MODULES.contains(&module.as_str()) {
            offences.push(format!("std::{module}"));
        }
    }
    for root in use_roots(&code) {
        if !USE_ROOTS.contains(&root.as_str()) {
            offences.push(format!("use {root}"));
        }
    }

    offences.sort();
    offences.dedup();
    offences
}

#[test]
fn the_linter_names_nothing_that_writes_or_reaches_outside() {
    let sources = linter_sources();
    assert!(sources.len() >= 5, "the linter's sources were found");

    let mut offenders = Vec::new();
    for source in &sources {
        let text = std::fs::read_to_string(source).expect("a linter source reads");
        for offence in offences(&text) {
            offenders.push(format!(
                "{}: {offence}",
                source.file_name().unwrap_or_default().to_string_lossy()
            ));
        }
    }
    assert!(
        offenders.is_empty(),
        "the linter reads and never writes: {offenders:?}"
    );
}

#[test]
fn the_sweep_can_see_what_it_is_looking_for() {
    // The control. A sweep that read nothing, or parsed nothing, would pass the
    // check above while the linter opened a file.
    let sources = linter_sources();
    let text: String = sources
        .iter()
        .map(|source| std::fs::read_to_string(source).expect("a linter source reads"))
        .collect();
    assert!(!text.is_empty(), "the linter's sources were read");
    let code = code_of(&text);
    assert!(
        !paths_under(&code, "crate").is_empty(),
        "the sweep found paths into this crate, so it is reading Rust"
    );
    assert!(
        !use_roots(&code).is_empty(),
        "the sweep found use statements, so it is reading Rust"
    );
}

#[test]
fn the_sweep_refuses_every_way_out_it_knows() {
    // Each of the three lists, exercised on source the linter must not hold.
    for forbidden in [
        "use crate::model::mutate::commit;",
        "use crate::model::write_file::write_document;",
        "use crate::model::write::plan_write;",
        "use crate::cli::Command;",
        "use crate::kicad::Erc;",
        "use std::fs::OpenOptions;",
        "use std::process::Command;",
        "use std::path::PathBuf;",
        "use std::time::Instant;",
        "use std::env::var;",
        "use serde_json::Value;",
        "use clap::Parser;",
        "fn write(&self) { std::fs::write(\"a\", \"b\"); }",
        "let plan = crate::model::write::plan_write();",
    ] {
        assert!(
            !offences(forbidden).is_empty(),
            "the sweep refuses: {forbidden}"
        );
    }

    // And it permits what the linter really holds.
    for allowed in [
        "use crate::geometry::Point;",
        "use crate::model::items::{SheetPath, Uuid};",
        "use crate::model::library::read_library;",
        "use crate::lint::finding::Finding;",
        "use std::cmp::Ordering;",
        "use std::collections::BTreeMap;",
        "use kicli_sexpr::Doc;",
    ] {
        assert!(offences(allowed).is_empty(), "the sweep permits: {allowed}");
    }

    // A comment is prose, not code. The sweep must read past it.
    assert!(offences("// this never calls std::fs::write\n").is_empty());
    assert!(offences("/* crate::model::mutate is forbidden */\n").is_empty());
    // And it must not read past the code that follows one.
    assert!(!offences("// harmless\nuse std::fs::write;\n").is_empty());
}
