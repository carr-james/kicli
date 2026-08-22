//! The arms of `cargo xtask check`, their verdicts, and the summary.
//!
//! An arm is one named piece of the gate. Most arms run on every invocation.
//! Two run only when the caller asks for them. The summary lists EVERY arm
//! either way, so a reader can answer "what would a full run have been?" from
//! the output alone.
//!
//! An arm has three verdicts, not two. `pass` and `FAIL` say what a run
//! measured. `skip` says the arm did not run, names the reason, and names the
//! exact command that runs it. Without the third verdict an arm that did not
//! run is reported by silence, and silence reads as success.

use std::fmt::Write as _;

/// What one arm did.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Verdict {
    /// The arm ran and its command succeeded.
    Pass,
    /// The arm ran and its command failed.
    Fail,
    /// The arm did not run.
    Skip {
        /// Why the arm did not run.
        why: String,
        /// The command that runs it.
        run: String,
    },
}

/// When an arm runs.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Run {
    /// On every invocation.
    Always,
    /// Only when `--corpus` is given.
    OnCorpusFlag,
    /// On the corpus run, which carries this arm's environment variable.
    RidesCorpus,
    /// On every invocation, comparing the working tree before and after.
    /// This arm runs no cargo command.
    Tree,
}

/// One arm of the gate.
pub struct Arm {
    /// Short name printed in the summary.
    pub name: &'static str,
    /// The command this arm runs, written as a reader would type it.
    ///
    /// For the five arms `ENGINEERING.md` lists, this string is the exact line
    /// that document holds. `every_documented_gate_is_an_arm` compares them.
    pub command: &'static str,
    /// Arguments passed to `cargo`. Empty when the arm runs no cargo command.
    pub args: &'static [&'static str],
    /// Environment variables set for this arm only.
    pub env: &'static [(&'static str, &'static str)],
    /// Command that installs the missing tool, when the arm needs one.
    pub install_hint: Option<&'static str>,
    /// The Cargo feature this arm turns on, when it turns one on.
    pub feature: Option<&'static str>,
    /// The test environment variable this arm sets, when it sets one.
    pub env_var: Option<&'static str>,
    /// When this arm runs.
    pub run: Run,
}

/// The name of the arm that checks the gates changed no tracked file.
pub const CLEAN: &str = "clean";

/// The name of the arm that runs the corpus tests.
pub const CORPUS: &str = "corpus";

/// The name of the arm that asks KiCad itself.
pub const ORACLE: &str = "kicad-cli";

/// The flag that asks for the two opt-in arms.
pub const CORPUS_FLAG: &str = "--corpus";

/// Every arm, in the order it runs and the order the summary prints it.
///
/// This table is the whole enumeration. Nothing else lists the arms, so the
/// summary cannot drift from what the run performs.
pub const ARMS: &[Arm] = &[
    Arm {
        name: "fmt",
        command: "cargo fmt --check",
        args: &["fmt", "--check"],
        env: &[],
        install_hint: Some("rustup component add rustfmt"),
        feature: None,
        env_var: None,
        run: Run::Always,
    },
    Arm {
        name: "clippy",
        command: "cargo clippy --all-targets --all-features -- -D warnings",
        args: &[
            "clippy",
            "--all-targets",
            "--all-features",
            "--",
            "-D",
            "warnings",
        ],
        env: &[],
        install_hint: Some("rustup component add clippy"),
        feature: None,
        env_var: None,
        run: Run::Always,
    },
    Arm {
        name: "test",
        command: "cargo test",
        args: &["test"],
        env: &[],
        install_hint: None,
        feature: None,
        env_var: None,
        run: Run::Always,
    },
    Arm {
        // Treat any rustdoc warning as an error, so "builds clean" is testable.
        name: "doc",
        command: "cargo doc --no-deps",
        args: &["doc", "--no-deps"],
        env: &[("RUSTDOCFLAGS", "-D warnings")],
        install_hint: None,
        feature: None,
        env_var: None,
        run: Run::Always,
    },
    Arm {
        name: "deny",
        command: "cargo deny check",
        args: &["deny", "check"],
        env: &[],
        install_hint: Some("cargo install --locked cargo-deny"),
        feature: None,
        env_var: None,
        run: Run::Always,
    },
    Arm {
        // The only checks that measure kicli against KiCad's own files. They
        // are compiled out without the feature, so a run without them reports
        // no ignored test and says nothing. That silence is what the third
        // verdict exists to break.
        name: CORPUS,
        command: "cargo test --features corpus",
        args: &["test", "--features", "corpus"],
        env: &[],
        install_hint: None,
        feature: Some("corpus"),
        env_var: None,
        run: Run::OnCorpusFlag,
    },
    Arm {
        // This arm cannot be made unconditional: kicad-cli genuinely may be
        // absent. What it can do is say so.
        name: ORACLE,
        command: "KICLI_TEST_KICAD_CLI=1 cargo test --features corpus",
        args: &[],
        env: &[],
        install_hint: None,
        feature: None,
        env_var: Some("KICLI_TEST_KICAD_CLI"),
        run: Run::RidesCorpus,
    },
    Arm {
        name: CLEAN,
        command: "git status --porcelain, before the arms and after them",
        args: &[],
        env: &[],
        install_hint: None,
        feature: None,
        env_var: None,
        run: Run::Tree,
    },
];

/// The skip verdict for an arm nobody asked for.
///
/// The reason names what the arm would have turned on, so the reader learns
/// which tests did not run and not only that an arm did not.
#[must_use]
pub fn not_requested(arm: &Arm) -> Verdict {
    Verdict::Skip {
        why: format!("{CORPUS_FLAG} was not given, so {} stayed off", off(arm)),
        run: format!("cargo xtask check {CORPUS_FLAG}"),
    }
}

/// The skip verdict for the oracle arm when `kicad-cli` is not there.
#[must_use]
pub fn no_kicad_cli(arm: &Arm) -> Verdict {
    Verdict::Skip {
        why: format!(
            "kicad-cli is not on PATH and KICLI_KICAD_CLI does not name it, \
             so {} stayed off",
            off(arm)
        ),
        run: format!("cargo xtask check {CORPUS_FLAG}, with kicad-cli installed"),
    }
}

/// What an arm turns on, named as a reader would look for it.
fn off(arm: &Arm) -> String {
    if let Some(feature) = arm.feature {
        format!("the `{feature}` feature")
    } else if let Some(variable) = arm.env_var {
        variable.to_owned()
    } else {
        format!("the {} arm", arm.name)
    }
}

/// How many arms ended each way.
#[derive(Default)]
struct Tally {
    /// Arms that ran and passed.
    passed: usize,
    /// Arms that ran and failed.
    failed: usize,
    /// Arms that did not run.
    skipped: usize,
    /// Arms the run recorded no verdict for. Always zero, or a bug.
    unrecorded: usize,
}

/// The summary text for a run.
///
/// The text holds one line per arm in `ARMS`, whatever the run did. An arm the
/// run recorded no verdict for is printed as a bug rather than left out: a
/// missing line is the failure this whole module exists to prevent.
#[must_use]
pub fn summary(outcomes: &[(&'static str, Verdict)]) -> String {
    let mut text = String::from("=== summary ===\n");
    let tally = write_arms(&mut text, outcomes);
    write_hints(&mut text, outcomes);
    text.push('\n');
    write_headline(&mut text, &tally);
    text
}

/// Write one block per arm. Count how each arm ended.
fn write_arms(text: &mut String, outcomes: &[(&'static str, Verdict)]) -> Tally {
    let width = ARMS.iter().map(|arm| arm.name.len()).max().unwrap_or(0);
    let mut tally = Tally::default();
    for arm in ARMS {
        match found_verdict(outcomes, arm.name) {
            Some(Verdict::Pass) => {
                tally.passed += 1;
                let _ = writeln!(text, "  pass  {:width$}  {}", arm.name, arm.command);
            }
            Some(Verdict::Fail) => {
                tally.failed += 1;
                let _ = writeln!(text, "  FAIL  {:width$}  {}", arm.name, arm.command);
            }
            Some(Verdict::Skip { why, run }) => {
                tally.skipped += 1;
                let _ = writeln!(text, "  skip  {:width$}  {}", arm.name, arm.command);
                let _ = writeln!(text, "        {:width$}  did not run: {why}", "");
                let _ = writeln!(text, "        {:width$}  runs with: {run}", "");
            }
            None => {
                tally.unrecorded += 1;
                let _ = writeln!(text, "  BUG   {:width$}  {}", arm.name, arm.command);
                let _ = writeln!(
                    text,
                    "        {:width$}  the run recorded no verdict for this arm",
                    ""
                );
            }
        }
    }
    tally
}

/// Write the install hint for every arm that failed and has one.
fn write_hints(text: &mut String, outcomes: &[(&'static str, Verdict)]) {
    for arm in ARMS {
        let failed = matches!(found_verdict(outcomes, arm.name), Some(Verdict::Fail));
        if let (true, Some(hint)) = (failed, arm.install_hint) {
            let _ = writeln!(
                text,
                "\nnote: if '{}' is not installed, run: {hint}",
                arm.name
            );
        }
    }
}

/// Write the one sentence a reader takes away.
///
/// The sentence is false when an arm did not run. "All gates passed" over a
/// run with a skipped arm is true of the arms that ran and silent about the
/// rest, and silence reads as success.
fn write_headline(text: &mut String, tally: &Tally) {
    let total = ARMS.len();
    let Tally {
        passed,
        failed,
        skipped,
        unrecorded,
    } = *tally;
    if failed > 0 || unrecorded > 0 {
        let _ = writeln!(
            text,
            "FAILED: {failed} of {total} arms failed. {passed} passed, {skipped} skipped."
        );
    } else if skipped > 0 {
        let _ = writeln!(
            text,
            "INCOMPLETE: {skipped} of {total} arms did not run. {passed} passed, 0 failed."
        );
        let _ = writeln!(
            text,
            "This was not a full run. The skip lines above say what runs each one."
        );
    } else {
        let _ = writeln!(
            text,
            "COMPLETE: all {total} arms passed. This was a full run."
        );
    }
}

/// The verdict recorded for an arm, or nothing.
fn found_verdict<'a>(outcomes: &'a [(&'static str, Verdict)], name: &str) -> Option<&'a Verdict> {
    outcomes
        .iter()
        .find(|(recorded, _)| *recorded == name)
        .map(|(_, verdict)| verdict)
}

#[cfg(test)]
mod tests {
    use super::{ARMS, Run, Verdict, no_kicad_cli, not_requested, summary};
    use std::path::{Path, PathBuf};

    /// The workspace root, from this crate's manifest.
    fn workspace() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
    }

    /// The arm of this name.
    fn named(name: &str) -> &'static super::Arm {
        ARMS.iter()
            .find(|arm| arm.name == name)
            .expect("the arm is enumerated")
    }

    /// Every arm passing, as the run would record it.
    fn all_passing() -> Vec<(&'static str, Verdict)> {
        ARMS.iter().map(|arm| (arm.name, Verdict::Pass)).collect()
    }

    // --- The enumeration, checked against sources nobody here wrote. ---
    //
    // These three checks each derive their expectation from a file this module
    // does not own: the engineering standards, the crate manifests, and the
    // test sources. Removing an arm from ARMS makes one of them fail. That is
    // the point: an enumeration nothing counts is a list that drifts.
    //
    // The boundary, stated rather than implied: the `clean` arm has no such
    // independent source, because the working-tree comparison is xtask's own
    // invention and no other file names it. `every_kind_of_arm_is_enumerated`
    // covers it instead, through the `Run` enum the compiler maintains.

    /// The gate lines `ENGINEERING.md` lists, with comments stripped.
    fn documented_gates() -> Vec<String> {
        let text = std::fs::read_to_string(workspace().join("ENGINEERING.md"))
            .expect("ENGINEERING.md is readable from the workspace root");
        let mut lines = Vec::new();
        let mut inside = false;
        let mut seen_heading = false;
        for line in text.lines() {
            if line.starts_with("## Machine-enforced gates") {
                seen_heading = true;
                continue;
            }
            if !seen_heading {
                continue;
            }
            if line.starts_with("```") {
                if inside {
                    break;
                }
                inside = true;
                continue;
            }
            if inside {
                let command = line.split('#').next().unwrap_or("").trim();
                if !command.is_empty() {
                    lines.push(command.to_owned());
                }
            }
        }
        lines
    }

    #[test]
    fn every_documented_gate_is_an_arm() {
        let documented = documented_gates();
        // Presence control: a parser that read nothing must not pass.
        assert!(
            documented.len() >= 5,
            "read {} gate lines from ENGINEERING.md; the parser found nothing to check",
            documented.len()
        );
        let commands: Vec<&str> = ARMS.iter().map(|arm| arm.command).collect();
        for gate in &documented {
            assert!(
                commands.contains(&gate.as_str()),
                "ENGINEERING.md lists `{gate}` and no arm runs it. Arms run: {commands:?}"
            );
        }
    }

    /// Every feature name declared by a crate in this workspace.
    fn declared_features() -> Vec<String> {
        let mut found = Vec::new();
        let crates = workspace().join("crates");
        let entries = std::fs::read_dir(&crates).expect("the crates directory is readable");
        for entry in entries.flatten() {
            let manifest = entry.path().join("Cargo.toml");
            let Ok(text) = std::fs::read_to_string(&manifest) else {
                continue;
            };
            let mut inside = false;
            for line in text.lines() {
                let line = line.trim();
                if line.starts_with('[') {
                    inside = line == "[features]";
                    continue;
                }
                if !inside || line.starts_with('#') || line.is_empty() {
                    continue;
                }
                if let Some((name, _)) = line.split_once('=') {
                    let name = name.trim();
                    if name != "default" && !found.iter().any(|held| held == name) {
                        found.push(name.to_owned());
                    }
                }
            }
        }
        found
    }

    #[test]
    fn every_declared_feature_is_an_arm() {
        let features = declared_features();
        // Presence control: a reader that found no manifest must not pass.
        assert!(
            !features.is_empty(),
            "read no feature names from crates/*/Cargo.toml"
        );
        let claimed: Vec<&str> = ARMS.iter().filter_map(|arm| arm.feature).collect();
        for feature in &features {
            assert!(
                claimed.contains(&feature.as_str()),
                "crates declare the feature `{feature}` and no arm turns it on. \
                 A feature nothing runs is a set of tests the gate cannot see."
            );
        }
    }

    /// Every `KICLI_TEST_*` variable the workspace sources read.
    fn test_environment_variables() -> Vec<String> {
        let mut found = Vec::new();
        let mut files = Vec::new();
        collect_rust(&workspace().join("crates"), &mut files);
        for file in files {
            let Ok(text) = std::fs::read_to_string(&file) else {
                continue;
            };
            let mut rest = text.as_str();
            while let Some(at) = rest.find("KICLI_TEST_") {
                rest = &rest[at..];
                let end = rest
                    .find(|c: char| !(c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_'))
                    .unwrap_or(rest.len());
                let name = rest[..end].to_owned();
                if !found.contains(&name) {
                    found.push(name);
                }
                rest = &rest[end..];
            }
        }
        found
    }

    /// Every `.rs` file under a directory.
    fn collect_rust(directory: &Path, found: &mut Vec<PathBuf>) {
        let Ok(entries) = std::fs::read_dir(directory) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_rust(&path, found);
            } else if path.extension().is_some_and(|end| end == "rs") {
                found.push(path);
            }
        }
    }

    #[test]
    fn every_test_environment_variable_is_an_arm() {
        let variables = test_environment_variables();
        // Presence control: a sweep that matched no file must not pass.
        assert!(
            !variables.is_empty(),
            "read no KICLI_TEST_* names from crates/**/*.rs"
        );
        let claimed: Vec<&str> = ARMS.iter().filter_map(|arm| arm.env_var).collect();
        for variable in &variables {
            assert!(
                claimed.contains(&variable.as_str()),
                "the sources read `{variable}` and no arm sets it. \
                 An environment the gate never supplies is a set of checks \
                 that silently do nothing."
            );
        }
    }

    #[test]
    fn every_kind_of_arm_is_enumerated() {
        // The match is exhaustive, so a new Run variant must be listed here,
        // and the assertion then demands an arm that performs it.
        for kind in [Run::Always, Run::OnCorpusFlag, Run::RidesCorpus, Run::Tree] {
            let label = match kind {
                Run::Always => "Always",
                Run::OnCorpusFlag => "OnCorpusFlag",
                Run::RidesCorpus => "RidesCorpus",
                Run::Tree => "Tree",
            };
            assert!(
                ARMS.iter().any(|arm| arm.run == kind),
                "the run can perform a {label} arm and no arm of that kind is enumerated"
            );
        }
    }

    // --- The summary. ---

    #[test]
    fn each_arm_reports_its_own_verdict() {
        // A substring match for "skip" passes when EVERY line says skip. This
        // check cannot: each arm gets a different verdict, and each line is
        // read back against the verdict that arm was given. A summary that
        // ignored its input, or blurred one arm's verdict into another's,
        // fails here.
        let mut outcomes: Vec<(&'static str, Verdict)> = Vec::new();
        for (index, arm) in ARMS.iter().enumerate() {
            let verdict = match index % 3 {
                0 => Verdict::Pass,
                1 => Verdict::Fail,
                _ => not_requested(arm),
            };
            outcomes.push((arm.name, verdict));
        }

        let text = summary(&outcomes);
        for (name, verdict) in &outcomes {
            let mark = match verdict {
                Verdict::Pass => "pass",
                Verdict::Fail => "FAIL",
                Verdict::Skip { .. } => "skip",
            };
            let line = text
                .lines()
                .find(|line| line.split_whitespace().nth(1) == Some(name))
                .unwrap_or_else(|| panic!("the summary holds a line for `{name}`:\n{text}"));
            assert_eq!(
                line.split_whitespace().next(),
                Some(mark),
                "arm `{name}` was given {verdict:?} and its line reads `{line}`"
            );
        }
    }

    #[test]
    fn every_arm_appears_exactly_once() {
        let text = summary(&all_passing());
        for arm in ARMS {
            let count = text
                .lines()
                .filter(|line| line.split_whitespace().nth(1) == Some(arm.name))
                .count();
            assert_eq!(count, 1, "arm `{}` has {count} lines in:\n{text}", arm.name);
        }
    }

    #[test]
    fn a_full_run_is_the_only_run_called_complete() {
        let text = summary(&all_passing());
        assert!(text.contains("COMPLETE: all"), "{text}");
        assert!(!text.contains("INCOMPLETE"), "{text}");
        assert!(!text.contains("FAILED"), "{text}");
    }

    #[test]
    fn one_skipped_arm_makes_the_run_incomplete() {
        // The headline must be FALSE when an arm did not run, not merely
        // silent about it. One arm skipped is enough.
        for skipped in ARMS {
            let mut outcomes = all_passing();
            for entry in &mut outcomes {
                if entry.0 == skipped.name {
                    entry.1 = not_requested(skipped);
                }
            }
            let text = summary(&outcomes);
            assert!(
                text.contains("INCOMPLETE: 1 of"),
                "arm `{}` was skipped and the summary does not say so:\n{text}",
                skipped.name
            );
            assert!(
                !text.contains("COMPLETE: all"),
                "arm `{}` was skipped and the summary still calls the run complete:\n{text}",
                skipped.name
            );
        }
    }

    #[test]
    fn one_failed_arm_makes_the_run_failed() {
        for broken in ARMS {
            let mut outcomes = all_passing();
            for entry in &mut outcomes {
                if entry.0 == broken.name {
                    entry.1 = Verdict::Fail;
                }
            }
            let text = summary(&outcomes);
            assert!(
                text.contains("FAILED: 1 of"),
                "arm `{}` failed and the summary does not say so:\n{text}",
                broken.name
            );
            assert!(!text.contains("COMPLETE: all"), "{text}");
        }
    }

    #[test]
    fn a_skip_line_names_its_reason_and_the_command_that_runs_it() {
        let corpus = named(super::CORPUS);
        let oracle = named(super::ORACLE);
        let mut outcomes = all_passing();
        for entry in &mut outcomes {
            if entry.0 == super::CORPUS {
                entry.1 = not_requested(corpus);
            }
            if entry.0 == super::ORACLE {
                entry.1 = no_kicad_cli(oracle);
            }
        }
        let text = summary(&outcomes);

        assert!(
            text.contains("did not run: --corpus was not given"),
            "the corpus skip does not name its reason:\n{text}"
        );
        assert!(
            text.contains("runs with: cargo xtask check --corpus"),
            "the corpus skip does not name the command that runs it:\n{text}"
        );
        assert!(
            text.contains("did not run: kicad-cli is not on PATH"),
            "the oracle skip does not name its reason:\n{text}"
        );
        // The two skips must not be interchangeable. A reader has to be able
        // to tell "nobody asked" from "the tool is missing".
        assert_ne!(
            not_requested(oracle),
            no_kicad_cli(oracle),
            "the two skip reasons are the same text"
        );
    }

    #[test]
    fn a_skipped_arm_still_states_what_a_full_run_would_have_been() {
        // The finding this module answers: nothing anywhere said what a full
        // run consisted of. A skipped arm still prints its command.
        let corpus = named(super::CORPUS);
        let mut outcomes = all_passing();
        for entry in &mut outcomes {
            if entry.0 == super::CORPUS {
                entry.1 = not_requested(corpus);
            }
        }
        let text = summary(&outcomes);
        assert!(
            text.contains(corpus.command),
            "the skipped corpus arm does not print `{}`:\n{text}",
            corpus.command
        );
    }

    #[test]
    fn an_arm_with_no_verdict_is_reported_as_a_bug() {
        // The run builds one outcome per arm, so this cannot happen today.
        // The summary still refuses to leave a line out, because leaving a
        // line out is the exact defect.
        let outcomes: Vec<(&'static str, Verdict)> = all_passing()
            .into_iter()
            .filter(|(name, _)| *name != super::CLEAN)
            .collect();
        let text = summary(&outcomes);
        assert!(text.contains("BUG"), "{text}");
        assert!(text.contains("FAILED:"), "{text}");
    }
}
