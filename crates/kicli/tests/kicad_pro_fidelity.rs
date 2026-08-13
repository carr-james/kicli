//! Measure how close a JSON round trip comes to KiCad's own project files.
//!
//! This is a measurement, not a policy. It exists to answer one question: can a
//! project file be written back byte for byte by an order-preserving JSON
//! writer? Whether kicli then reformats or refuses is a separate decision.

use std::path::{Path, PathBuf};

/// How a re-serialised file differs from the original.
#[derive(Debug, PartialEq, Eq)]
enum Verdict {
    ByteIdentical,
    WhitespaceOnly,
    ReordersKeys,
    ReformatsNumbers,
    Other,
}

impl Verdict {
    fn label(&self) -> &'static str {
        match self {
            Self::ByteIdentical => "byte-identical",
            Self::WhitespaceOnly => "whitespace-only",
            Self::ReordersKeys => "reorders-keys",
            Self::ReformatsNumbers => "reformats-numbers",
            Self::Other => "other",
        }
    }
}

/// Re-serialise `text` and say how the result differs.
fn compare(text: &str) -> (Verdict, String) {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(text) else {
        return (Verdict::Other, String::new());
    };
    let mut written = serde_json::to_string_pretty(&value).unwrap_or_default();
    written.push('\n');

    if written == text {
        return (Verdict::ByteIdentical, written);
    }

    let strip = |s: &str| s.chars().filter(|c| !c.is_whitespace()).collect::<String>();
    if strip(&written) == strip(text) {
        return (Verdict::WhitespaceOnly, written);
    }

    // Compare the key order as it appears in each text.
    let keys = |s: &str| {
        s.lines()
            .filter_map(|line| line.trim().split_once("\": "))
            .map(|(key, _)| key.trim_start_matches('"').to_owned())
            .collect::<Vec<_>>()
    };
    let mut original_keys = keys(text);
    let mut written_keys = keys(&written);
    if original_keys != written_keys {
        original_keys.sort();
        written_keys.sort();
        if original_keys == written_keys {
            return (Verdict::ReordersKeys, written);
        }
    }

    (Verdict::ReformatsNumbers, written)
}

fn collect(dir: &Path, found: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect(&path, found);
        } else if path.extension().is_some_and(|e| e == "kicad_pro") {
            found.push(path);
        }
    }
}

#[test]
fn kicad_pro_fidelity_report() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    collect(&manifest.join("tests/fixtures"), &mut files);
    collect(&manifest.join("../../target/corpus/demos"), &mut files);
    files.sort();

    let mut lines = vec![
        "# `.kicad_pro` round-trip fidelity".to_owned(),
        String::new(),
        "Measured by `cargo test -p kicli kicad_pro_fidelity_report`. Each file is".to_owned(),
        "read with an order-preserving JSON reader and written back with".to_owned(),
        "`serde_json::to_string_pretty`, then compared byte for byte.".to_owned(),
        String::new(),
        "| verdict | files |".to_owned(),
        "|---|---|".to_owned(),
    ];

    let mut counts: std::collections::BTreeMap<&'static str, usize> =
        std::collections::BTreeMap::new();
    let mut examples: std::collections::BTreeMap<&'static str, String> =
        std::collections::BTreeMap::new();

    for path in &files {
        let Ok(text) = std::fs::read_to_string(path) else {
            continue;
        };
        let (verdict, written) = compare(&text);
        if verdict != Verdict::ByteIdentical {
            if let Some((before, after)) = text.lines().zip(written.lines()).find(|(a, b)| a != b) {
                println!(
                    "{}: first difference\n  kicad: {before}\n  kicli: {after}",
                    path.display()
                );
            }
        }
        *counts.entry(verdict.label()).or_default() += 1;
        examples.entry(verdict.label()).or_insert_with(|| {
            path.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned()
        });
    }

    for (label, count) in &counts {
        lines.push(format!("| {label} | {count} |"));
    }
    lines.push(String::new());
    lines.push(format!("Files measured: {}.", files.len()));
    lines.push(String::new());
    for (label, example) in &examples {
        lines.push(format!("- `{label}` first seen in `{example}`"));
    }
    lines.push(String::new());

    // The committed note records a measurement over KiCad's whole corpus. A run
    // without the corpus measures the fixtures alone, and writing that over the
    // note replaces forty files of evidence with four. Two lanes hit this in one
    // milestone. The report goes to the build directory, and only a run that
    // actually has the corpus offers to refresh the note.
    let report = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("kicad-pro-fidelity.md");
    std::fs::write(&report, lines.join("\n")).expect("report is written");
    println!("wrote {}", report.display());
    if cfg!(feature = "corpus") {
        println!(
            "the corpus was measured: copy that file over research/notes/kicad-pro-fidelity.md \
             to refresh the note"
        );
    }

    for (label, count) in &counts {
        println!("{label}: {count}");
    }
    assert!(
        !files.is_empty(),
        "there is at least the fixture project file"
    );
}
