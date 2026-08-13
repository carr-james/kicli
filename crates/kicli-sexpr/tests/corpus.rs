//! The same properties, against KiCad's own files.
//!
//! These files live in `target/`, fetched by `cargo xtask corpus`.
//! They are the strongest available evidence that the prettifier port matches
//! KiCad, because KiCad wrote every byte of them.
//!
//! The whole file is behind the `corpus` feature, so the normal test run needs
//! no network and no KiCad install.
#![cfg(feature = "corpus")]

use kicli_sexpr::{Doc, FormatMode, flatten, lex, parse_iu, prettify};
use std::path::{Path, PathBuf};

fn corpus_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/corpus")
}

/// Every file under `dir` with the given extension.
fn collect(dir: &Path, extension: &str, found: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect(&path, extension, found);
        } else if path.extension().is_some_and(|e| e == extension) {
            found.push(path);
        }
    }
}

fn demo_files(extension: &str) -> Vec<PathBuf> {
    let mut found = Vec::new();
    collect(&corpus_root().join("demos"), extension, &mut found);
    found.sort();
    found
}

fn regression_files(extension: &str) -> Vec<PathBuf> {
    let mut found = Vec::new();
    collect(&corpus_root().join("qa"), extension, &mut found);
    found.sort();
    found
}

fn require_corpus(files: &[PathBuf]) {
    assert!(
        !files.is_empty(),
        "the corpus is missing. Run `cargo xtask corpus` first."
    );
}

#[test]
fn prettify_reproduces_kicad_layout_corpus() {
    let schematics = demo_files("kicad_sch");
    require_corpus(&schematics);

    let mut passed = 0;
    let mut failures = Vec::new();
    for path in &schematics {
        let Ok(source) = std::fs::read_to_string(path) else {
            continue;
        };
        if prettify(&flatten(&source), FormatMode::Normal) == source {
            passed += 1;
        } else {
            failures.push(path.clone());
        }
    }
    println!("schematics reproduced: {passed}/{}", schematics.len());
    assert!(
        failures.is_empty(),
        "these schematics were not reproduced: {failures:#?}"
    );
    assert!(
        passed >= 115,
        "expected at least 115 schematics, reproduced {passed}"
    );

    // Boards use the same writer, so one prettifier serves both. They only
    // reproduce once `pcb upgrade` has run over them: one demo board still
    // carried `(generator_version "9.0")` and packed two sibling lists onto one
    // line as `)(`, which KiCad 9 wrote and KiCad 10 does not.
    let boards = demo_files("kicad_pcb");
    let mut board_failures = Vec::new();
    for path in &boards {
        let Ok(source) = std::fs::read_to_string(path) else {
            continue;
        };
        if prettify(&flatten(&source), FormatMode::Normal) != source {
            board_failures.push(path.clone());
        }
    }
    println!(
        "boards reproduced: {}/{}",
        boards.len() - board_failures.len(),
        boards.len()
    );
    assert!(
        board_failures.is_empty(),
        "these boards were not reproduced: {board_failures:#?}"
    );
    assert!(boards.len() >= 6, "expected at least 6 boards");

    // And the shipped symbol library, when KiCad is installed.
    let shipped =
        Path::new("/Applications/KiCad/KiCad.app/Contents/SharedSupport/symbols/Device.kicad_sym");
    if let Ok(source) = std::fs::read_to_string(shipped) {
        assert_eq!(
            prettify(&flatten(&source), FormatMode::Normal),
            source,
            "the shipped symbol library is reproduced"
        );
        println!("shipped symbol library reproduced");
    } else {
        println!("shipped symbol library not found; skipped");
    }
}

#[test]
fn emit_reproduces_input_bytes_corpus() {
    let schematics = demo_files("kicad_sch");
    require_corpus(&schematics);

    let mut failures = Vec::new();
    for path in &schematics {
        let Ok(source) = std::fs::read_to_string(path) else {
            continue;
        };
        let doc = Doc::parse(&source).expect("corpus file parses");
        if doc.emit() != source {
            failures.push(path.clone());
        }
    }
    println!(
        "schematics round-tripped byte for byte: {}/{}",
        schematics.len() - failures.len(),
        schematics.len()
    );
    assert!(
        failures.is_empty(),
        "these files did not round-trip: {failures:#?}"
    );
}

#[test]
fn reparse_preserves_tree_corpus() {
    // The regression data is deliberately awkward and holds older formats, so
    // it is the honest test of the property that must hold for every input.
    let mut files = demo_files("kicad_sch");
    files.extend(regression_files("kicad_sch"));
    files.extend(demo_files("kicad_sym"));
    files.extend(regression_files("kicad_sym"));
    require_corpus(&files);

    let mut checked = 0;
    let mut failures = Vec::new();
    for path in &files {
        let Ok(source) = std::fs::read_to_string(path) else {
            continue;
        };
        let Ok(first) = Doc::parse(&source) else {
            // A file that does not parse is not a round-trip failure. The
            // regression set holds files KiCad itself rejects.
            continue;
        };
        checked += 1;
        let written = first.emit();
        match Doc::parse(&written) {
            Ok(second) if first.structurally_eq(&second) => {}
            _ => failures.push(path.clone()),
        }
    }
    println!(
        "files whose tree survived a write: {}/{checked}",
        checked - failures.len()
    );
    assert!(checked > 300, "the regression set was included");
    assert!(
        failures.is_empty(),
        "these files lost their shape: {failures:#?}"
    );
}

#[test]
fn fmt_iu_reproduces_corpus_numbers() {
    let schematics = demo_files("kicad_sch");
    require_corpus(&schematics);

    let mut checked = 0usize;
    let mut skipped = 0usize;
    for path in &schematics {
        let Ok(source) = std::fs::read_to_string(path) else {
            continue;
        };
        for token in lex(&source).expect("corpus file lexes") {
            let text = token.text(&source);
            if token.kind != kicli_sexpr::TokenKind::Bare {
                continue;
            }
            let looks_numeric = text
                .strip_prefix('-')
                .unwrap_or(text)
                .starts_with(|c: char| c.is_ascii_digit());
            if !looks_numeric {
                continue;
            }
            if let Some(units) = parse_iu(text) {
                assert_eq!(
                    kicli_sexpr::fmt_iu(units),
                    text,
                    "{} re-formats {text} unchanged",
                    path.display()
                );
                checked += 1;
            } else {
                // Version stamps and counts are integers too large for a
                // coordinate, and angles can carry more decimals.
                skipped += 1;
            }
        }
    }
    println!("numeric atoms re-formatted unchanged: {checked} (skipped {skipped})");
    assert!(
        checked > 100_000,
        "the corpus supplied plenty of coordinates"
    );
}

/// KiCad's own writer agrees with ours.
///
/// This is informational: it tracks drift from KiCad's canonical form rather
/// than gating a merge, because "what KiCad would write" is a different
/// property from "what we read".
#[test]
fn output_matches_kicad_writer() {
    if std::env::var_os("KICLI_TEST_KICAD_CLI").is_none() {
        println!("set KICLI_TEST_KICAD_CLI=1 to run the KiCad writer comparison");
        return;
    }

    let schematics = demo_files("kicad_sch");
    require_corpus(&schematics);

    let scratch = corpus_root().join("oracle");
    std::fs::create_dir_all(&scratch).expect("scratch directory");

    let mut agreed = 0;
    let mut compared = 0;
    for path in schematics.iter().take(40) {
        let Ok(source) = std::fs::read_to_string(path) else {
            continue;
        };
        let doc = Doc::parse(&source).expect("parses");
        let target = scratch.join(path.file_name().expect("has a name"));
        std::fs::write(&target, doc.emit()).expect("writes");

        let status = std::process::Command::new("kicad-cli")
            .args(["sch", "upgrade", "--force"])
            .arg(&target)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
        if status.is_ok_and(|s| s.success()) {
            compared += 1;
            if std::fs::read_to_string(&target).is_ok_and(|after| after == doc.emit()) {
                agreed += 1;
            }
        }
    }
    println!("KiCad left our output unchanged in {agreed}/{compared} files");
}
