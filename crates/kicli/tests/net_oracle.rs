//! kicli's net partition is KiCad's net partition.
//!
//! Connectivity is defined as whatever KiCad's netlister does, so this is the
//! gate rather than an opinion. The comparison is of the partition, the sets of
//! reference designator and pin number, and never of names: kicli names nets
//! its own way by design. It is of parsed content and never of bytes, because a
//! netlist carries the absolute path it was written from and the date.
//!
//! The default run reads the committed netlist. The regeneration run needs
//! `KICLI_TEST_KICAD_CLI` and writes a fresh one, so a stale oracle is caught
//! rather than trusted.

use kicli::connectivity::extract;
use kicli::model::Hierarchy;
use kicli_probe::oracle::{Kicad, Netlist, Partition, differences, kicli_partition};
use kicli_probe::scratch::Fixtures;
use std::path::{Path, PathBuf};

/// The committed fixtures this binary reads, and the scratch it writes in.
fn fixtures() -> Fixtures {
    Fixtures::new(env!("CARGO_TARGET_TMPDIR"), env!("CARGO_MANIFEST_DIR"))
}

/// The connectivity fixture directory.
fn nets_fixture(name: &str) -> PathBuf {
    fixtures().fixture("sch/nets").join(name)
}

/// The partition kicli reads out of a hierarchy on disk.
fn kicli_partition_of(root: &Path) -> Partition {
    let hierarchy = Hierarchy::load(root).expect("the hierarchy loads");
    kicli_partition(&extract(&hierarchy))
}

#[test]
fn netlist_partition_matches_kicad() {
    let committed =
        std::fs::read_to_string(nets_fixture("nets.netlist")).expect("the oracle is readable");
    let kicad = Netlist::parse(&committed).partition();
    let kicli = kicli_partition_of(&nets_fixture("nets.kicad_sch"));
    assert!(
        differences(&kicli, &kicad).is_none(),
        "{}",
        differences(&kicli, &kicad).unwrap_or_default()
    );
}

#[test]
fn netlist_oracle_is_current() {
    let Some(tool) = Kicad::found_or_skip("regenerate the oracle") else {
        return;
    };
    let root = nets_fixture("nets.kicad_sch");
    let copy = fixtures().copy_project("netlist-oracle", &root);
    let fresh = tool.netlist(
        &copy,
        &fixtures().scratch("netlist-fresh").join("nets.netlist"),
    );
    let committed =
        std::fs::read_to_string(nets_fixture("nets.netlist")).expect("the oracle is readable");

    let now = fresh.partition();
    assert!(
        differences(&Netlist::parse(&committed).partition(), &now).is_none(),
        "the committed netlist is stale: {}",
        differences(&Netlist::parse(&committed).partition(), &now).unwrap_or_default()
    );
    assert!(
        differences(&kicli_partition_of(&root), &now).is_none(),
        "{}",
        differences(&kicli_partition_of(&root), &now).unwrap_or_default()
    );
}

#[cfg(feature = "corpus")]
mod corpus {
    use super::{
        Hierarchy, Kicad, Path, PathBuf, differences, extract, fixtures, kicli_partition_of,
    };

    fn corpus_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/corpus/demos")
    }

    /// Every project root under a directory: a schematic beside a project file
    /// of the same name.
    fn roots(directory: &Path, found: &mut Vec<PathBuf>) {
        let Ok(entries) = std::fs::read_dir(directory) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                roots(&path, found);
            } else if path.extension().is_some_and(|end| end == "kicad_pro") {
                let schematic = path.with_extension("kicad_sch");
                if schematic.is_file() {
                    found.push(schematic);
                }
            }
        }
    }

    /// Every corpus hierarchy loads.
    ///
    /// kicli refuses a file carrying a measurement it cannot represent, which
    /// is the only honest answer for a value it would have to interpret. That
    /// refusal must never fall on a drawing KiCad itself wrote and kicli
    /// handles: `image` carries a placement and a scale at full float
    /// precision, and kicli copies both rather than reading them. This test is
    /// what keeps the list of objects kept verbatim complete — it failed on
    /// `jetson-agx-thor-baseboard` the first time the check was too broad.
    #[test]
    fn every_corpus_hierarchy_loads() {
        let mut projects = Vec::new();
        roots(&corpus_root(), &mut projects);
        projects.sort();
        if projects.is_empty() {
            eprintln!("skipped: the corpus is not there. Run `cargo xtask corpus` first.");
            return;
        }
        let mut refused = Vec::new();
        for root in &projects {
            if let Err(error) = Hierarchy::load(root) {
                refused.push(format!("{}: {error}", root.display()));
            }
        }
        assert!(
            refused.is_empty(),
            "kicli refused a drawing KiCad wrote:\n{}",
            refused.join("\n")
        );
        eprintln!("{} hierarchies loaded", projects.len());
    }

    /// No corpus hierarchy draws the one bundle shape kicli declines to join.
    ///
    /// kicli reports a bus that carries a vector bundle and a group bundle at
    /// once rather than reproducing KiCad's answer for it, which is degenerate.
    /// "Degenerate" is a claim about real drawings, so it is checked against
    /// them: if a demo ever draws one, the shape is not degenerate, the rule
    /// has to be implemented, and this test says so by failing.
    #[test]
    fn no_corpus_hierarchy_mixes_bundle_kinds() {
        let mut projects = Vec::new();
        roots(&corpus_root(), &mut projects);
        projects.sort();
        if projects.is_empty() {
            eprintln!("skipped: the corpus is not there. Run `cargo xtask corpus` first.");
            return;
        }

        let mut found = Vec::new();
        for root in &projects {
            let hierarchy = Hierarchy::load(root).expect("a corpus hierarchy loads");
            for warning in extract(&hierarchy).warnings() {
                found.push(format!(
                    "{}: {}",
                    root.file_stem().unwrap_or_default().to_string_lossy(),
                    warning.message()
                ));
            }
        }
        assert!(
            found.is_empty(),
            "a corpus hierarchy draws a bundle shape kicli only reports:\n{}",
            found.join("\n")
        );
        eprintln!(
            "{} hierarchies checked, none mixes bundle kinds",
            projects.len()
        );
    }

    /// kicli's partition equals KiCad's on every hierarchy of the demo corpus.
    ///
    /// All 35 match. This is the gate the extractor exists to pass, and it runs
    /// only when `KICLI_TEST_KICAD_CLI` is set, because it shells out to
    /// `kicad-cli` once per project.
    #[test]
    fn netlist_partition_matches_kicad_corpus() {
        let Some(tool) = Kicad::found_or_skip("compare against KiCad") else {
            return;
        };
        let mut projects = Vec::new();
        roots(&corpus_root(), &mut projects);
        projects.sort();
        if projects.is_empty() {
            eprintln!("skipped: the corpus is not there. Run `cargo xtask corpus` first.");
            return;
        }

        let mut passed = 0;
        let mut failures = Vec::new();
        let into = fixtures().scratch("corpus-netlist").join("corpus.netlist");
        for root in &projects {
            let name = root.file_stem().unwrap_or_default().to_string_lossy();
            let Some(netlist) = tool.try_netlist(root, &into) else {
                failures.push(format!("{name}: kicad-cli exported no netlist"));
                continue;
            };
            let kicad = netlist.partition();
            let kicli = kicli_partition_of(root);
            println!("{name}: {} nets", kicad.len());
            match differences(&kicli, &kicad) {
                None => passed += 1,
                Some(report) => failures.push(format!("{name}: {report}")),
            }
        }
        println!("hierarchies matched: {passed}/{}", projects.len());
        assert!(failures.is_empty(), "{}", failures.join("\n"));
    }
}
