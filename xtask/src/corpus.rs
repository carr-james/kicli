//! Fetch KiCad's own project files as an external test corpus.
//!
//! KiCad's demos and regression data never enter this repository. This task
//! clones them at a pinned tag into `target/`, which is not tracked, and
//! canonicalises the demo schematics through KiCad's own writer. Fetching beats
//! copying here: the repository stays small, the in-repo fixtures stay
//! purpose-built, and the corpus is pinned to a KiCad tag rather than to a copy
//! that ages in the tree. Tests that use the corpus are feature-gated, so the
//! default test run stays hermetic and needs no network.

use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

/// The KiCad release the corpus is pinned to.
///
/// Format facts drift between releases, so an unpinned corpus would make test
/// results depend on when they ran.
pub const PINNED_TAG: &str = "10.0.5";

/// The schematic format stamp that release writes.
pub const EXPECTED_STAMP: &str = "20260306";

/// Schematic count the canonicalised demo tree is expected to hold.
const EXPECTED_SCHEMATICS: usize = 115;

/// Library-table count the demo tree is expected to hold.
const MINIMUM_LIBRARY_TABLES: usize = 36;

/// Where the corpus lives, relative to the workspace root.
fn corpus_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../target/corpus")
}

/// Run the fetch, or the verification when `--verify` is given.
pub fn run(verify_only: bool) -> ExitCode {
    let root = corpus_root();

    if verify_only {
        return verify(&root);
    }

    if let Err(reason) = fetch(&root) {
        eprintln!("corpus: {reason}");
        eprintln!("corpus: skipping. Corpus tests will not run.");
        return ExitCode::SUCCESS;
    }
    ExitCode::SUCCESS
}

/// Clone KiCad at the pinned tag, then canonicalise the demo schematics.
fn fetch(root: &Path) -> Result<(), String> {
    if which("git").is_none() {
        return Err("git is not on PATH".to_owned());
    }

    let checkout = root.join("kicad");
    if checkout.join(".git").is_dir() {
        println!("corpus: already fetched at {}", checkout.display());
    } else {
        std::fs::create_dir_all(root)
            .map_err(|e| format!("cannot create {}: {e}", root.display()))?;
        println!("corpus: cloning KiCad at tag {PINNED_TAG} (this is a large download)");
        let status = Command::new("git")
            .args([
                "clone",
                "--depth",
                "1",
                "--branch",
                PINNED_TAG,
                "https://gitlab.com/kicad/code/kicad.git",
            ])
            .arg(&checkout)
            .status()
            .map_err(|e| format!("cannot run git: {e}"))?;
        if !status.success() {
            return Err("git clone failed; is the network reachable?".to_owned());
        }
    }

    canonicalise(root, &checkout)
}

/// Copy the demo tree and rewrite every schematic through KiCad's own writer.
///
/// `sch upgrade` loses bus aliases, so this runs on the copy in `target/` and
/// never on a file anybody keeps.
fn canonicalise(root: &Path, checkout: &Path) -> Result<(), String> {
    let demos = root.join("demos");
    if demos.is_dir() {
        println!("corpus: demos already canonicalised");
        return Ok(());
    }

    let Some(kicad_cli) = which("kicad-cli") else {
        return Err("kicad-cli is not on PATH".to_owned());
    };

    copy_tree(&checkout.join("demos"), &demos)
        .map_err(|e| format!("cannot copy the demo tree: {e}"))?;

    // The regression data is deliberately old and awkward. It is never
    // canonicalised: it exists to prove the semantic round trip holds for files
    // KiCad would not write today.
    copy_tree(&checkout.join("qa/data"), &root.join("qa"))
        .map_err(|e| format!("cannot copy the regression data: {e}"))?;

    // Some demo files were written by an older KiCad and never re-saved, so
    // they are not in the current canonical form. Rewriting them through
    // KiCad's own writer is what makes "our output matches KiCad's" testable.
    for (extension, subcommand) in [("kicad_sch", "sch"), ("kicad_pcb", "pcb")] {
        let mut files = Vec::new();
        collect(&demos, extension, &mut files);
        println!("corpus: canonicalising {} {subcommand} files", files.len());
        for path in &files {
            let status = Command::new(&kicad_cli)
                .args([subcommand, "upgrade", "--force"])
                .arg(path)
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status()
                .map_err(|e| format!("cannot run kicad-cli: {e}"))?;
            if !status.success() {
                println!("corpus: kicad-cli declined {}", path.display());
            }
        }
    }
    Ok(())
}

/// Check the corpus holds what the tests assume.
fn verify(root: &Path) -> ExitCode {
    let demos = root.join("demos");
    if !demos.is_dir() {
        eprintln!("corpus: not fetched. Run `cargo xtask corpus` first.");
        return ExitCode::FAILURE;
    }

    let mut schematics = Vec::new();
    collect(&demos, "kicad_sch", &mut schematics);

    let mut tables = Vec::new();
    collect_named(&demos, "sym-lib-table", &mut tables);
    collect_named(&demos, "fp-lib-table", &mut tables);

    let mut wrong_stamp = Vec::new();
    for path in &schematics {
        let text = std::fs::read_to_string(path).unwrap_or_default();
        if !text.contains(&format!("(version {EXPECTED_STAMP})")) {
            wrong_stamp.push(path.clone());
        }
    }

    println!("corpus: {} schematics", schematics.len());
    println!("corpus: {} library tables", tables.len());
    println!(
        "corpus: {} schematics not at the pinned stamp",
        wrong_stamp.len()
    );

    let mut ok = true;
    if schematics.len() != EXPECTED_SCHEMATICS {
        eprintln!(
            "corpus: expected {EXPECTED_SCHEMATICS} schematics, found {}",
            schematics.len()
        );
        ok = false;
    }
    if tables.len() < MINIMUM_LIBRARY_TABLES {
        eprintln!(
            "corpus: expected at least {MINIMUM_LIBRARY_TABLES} library tables, found {}",
            tables.len()
        );
        ok = false;
    }
    if !wrong_stamp.is_empty() {
        eprintln!("corpus: some schematics are not at the pinned stamp");
        ok = false;
    }

    if ok {
        println!("corpus: verified");
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

/// Find an executable on `PATH`.
fn which(program: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path)
        .map(|dir| dir.join(program))
        .find(|candidate| candidate.is_file())
}

/// Collect every file under `dir` with the given extension.
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

/// Collect every file under `dir` with the given exact name.
fn collect_named(dir: &Path, name: &str, found: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_named(&path, name, found);
        } else if path.file_name().is_some_and(|n| n == name) {
            found.push(path);
        }
    }
}

/// Copy a directory tree.
fn copy_tree(from: &Path, to: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(to)?;
    for entry in std::fs::read_dir(from)? {
        let entry = entry?;
        let target = to.join(entry.file_name());
        if entry.path().is_dir() {
            copy_tree(&entry.path(), &target)?;
        } else {
            std::fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}
