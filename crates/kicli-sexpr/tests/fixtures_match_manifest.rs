//! The fixture manifest and the fixture tree agree.
//!
//! A fixture nobody records is a fixture nobody understands, and a record with
//! no file is a stale claim. This test keeps the two in step. It reads the
//! manifest with the standard library only, so fixtures cost the workspace no
//! dependency.
//!
//! `kicli` carries a copy of this test for its own fixture root. The two roots
//! are independent, and sharing the code would make one crate read the other
//! crate's directory.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// One manifest record.
struct Record {
    version: String,
    mode: String,
    canonical: bool,
    provenance: String,
}

/// Read the manifest into records keyed by relative path.
fn read_manifest(root: &Path) -> BTreeMap<String, Record> {
    let text = std::fs::read_to_string(root.join("MANIFEST")).expect("manifest is readable");
    let mut records = BTreeMap::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let fields: Vec<&str> = line.split_whitespace().collect();
        assert_eq!(fields.len(), 5, "record needs five fields: {line}");
        let canonical = match fields[3] {
            "yes" => true,
            "no" => false,
            other => panic!("canonical is yes or no, not {other}: {line}"),
        };
        records.insert(
            fields[0].to_owned(),
            Record {
                version: fields[1].to_owned(),
                mode: fields[2].to_owned(),
                canonical,
                provenance: fields[4].to_owned(),
            },
        );
    }
    records
}

/// Collect every fixture under `root`, as paths relative to it.
///
/// Manifests are records, not fixtures, so they are skipped.
fn collect_fixtures(root: &Path, dir: &Path, found: &mut Vec<String>) {
    for entry in std::fs::read_dir(dir).expect("fixture directory is readable") {
        let path = entry.expect("directory entry is readable").path();
        if path.is_dir() {
            collect_fixtures(root, &path, found);
            continue;
        }
        let name = path.file_name().unwrap_or_default().to_string_lossy();
        if name == "MANIFEST" || name.ends_with(".manifest") {
            continue;
        }
        let relative = path
            .strip_prefix(root)
            .expect("fixture sits under the root")
            .to_string_lossy()
            .into_owned();
        found.push(relative);
    }
}

/// Find the `(version N)` stamp, when the file has one.
fn version_stamp(text: &str) -> Option<String> {
    let start = text.find("(version ")? + "(version ".len();
    let rest = &text[start..];
    let end = rest.find(')')?;
    Some(rest[..end].trim().to_owned())
}

fn fixture_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

#[test]
fn fixtures_match_manifest() {
    let root = fixture_root();
    let records = read_manifest(&root);

    let mut found = Vec::new();
    collect_fixtures(&root, &root, &mut found);
    found.sort();

    let recorded: Vec<String> = records.keys().cloned().collect();
    assert_eq!(
        found, recorded,
        "every fixture is recorded, and the reverse"
    );

    for (relative, record) in &records {
        let bytes = std::fs::read(root.join(relative)).expect("fixture is readable");
        let text = String::from_utf8(bytes.clone()).expect("fixture is UTF-8");

        if record.canonical {
            assert!(
                text.ends_with('\n') && !text.ends_with("\n\n"),
                "{relative} is canonical, so it ends in exactly one newline"
            );
        }

        if record.version != "-" {
            assert_eq!(
                version_stamp(&text).as_deref(),
                Some(record.version.as_str()),
                "{relative} carries the version the manifest records"
            );
        }

        assert!(
            matches!(
                record.mode.as_str(),
                "normal" | "compact" | "library-table" | "json"
            ),
            "{relative} has a known mode, not {}",
            record.mode
        );
        assert!(
            matches!(record.provenance.as_str(), "kicad-cli" | "hand"),
            "{relative} has a known provenance, not {}",
            record.provenance
        );
    }
}
