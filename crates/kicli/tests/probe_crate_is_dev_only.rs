//! The probe harness reaches no shipped artefact.
//!
//! `kicli-probe` is a test instrument. It depends on `kicli`, and `kicli`
//! dev-depends on it, which cargo resolves because the edge back is dev-only.
//! The moment someone makes it a normal dependency the cycle becomes real and,
//! worse, a test helper ships inside the binary.
//!
//! The licence allowlist does not govern a dev-dependency, so `cargo deny`
//! cannot hold this line. The rule is executable here instead.

use std::path::{Path, PathBuf};

/// The crate whose edges are checked.
const HARNESS: &str = "kicli-probe";

/// Every `Cargo.toml` of the workspace, the manifest at the root included.
fn manifests(workspace: &Path) -> Vec<PathBuf> {
    let mut found = vec![workspace.join("Cargo.toml")];
    for member in [
        "crates/kicli",
        "crates/kicli-sexpr",
        "crates/kicli-probe",
        "xtask",
    ] {
        found.push(workspace.join(member).join("Cargo.toml"));
    }
    found
}

#[test]
fn no_crate_depends_on_the_probe_harness_outside_its_tests() {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("the workspace root resolves");

    let mut checked = 0;
    let mut offenders = Vec::new();
    for manifest in manifests(&workspace) {
        let text = std::fs::read_to_string(&manifest).expect("a manifest reads");
        let table: toml::Table = text.parse().expect("a manifest is TOML");
        checked += 1;
        for section in ["dependencies", "build-dependencies", "workspace"] {
            let Some(entries) = table.get(section).and_then(toml::Value::as_table) else {
                continue;
            };
            // The workspace table holds its own dependency list for inherited
            // versions, which is a shipping edge like any other.
            let names: Vec<&String> = if section == "workspace" {
                entries
                    .get("dependencies")
                    .and_then(toml::Value::as_table)
                    .map(|inherited| inherited.keys().collect())
                    .unwrap_or_default()
            } else {
                entries.keys().collect()
            };
            if names.iter().any(|name| name.as_str() == HARNESS) {
                offenders.push(format!(
                    "{}: [{section}]",
                    manifest
                        .strip_prefix(&workspace)
                        .unwrap_or(&manifest)
                        .display()
                ));
            }
        }
    }

    assert_eq!(checked, 5, "every manifest of the workspace was read");
    assert!(
        offenders.is_empty(),
        "{HARNESS} is a test instrument and belongs under [dev-dependencies] only: {offenders:?}"
    );
}

#[test]
fn the_probe_harness_is_a_dev_dependency_of_this_crate() {
    // The control for the check above. A test that reads no manifest, or reads
    // manifests that never name the harness at all, would pass while saying
    // nothing.
    let text = std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"))
        .expect("this crate's manifest reads");
    let table: toml::Table = text.parse().expect("the manifest is TOML");
    assert!(
        table["dev-dependencies"]
            .as_table()
            .expect("dev-dependencies is a table")
            .contains_key(HARNESS),
        "the tests of this crate use the harness"
    );
}
