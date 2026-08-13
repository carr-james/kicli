//! Every call to an external KiCad binary goes through here.
//!
//! This module finds `kicad-cli`, checks that it is version 10, runs it, and
//! translates its exit codes into kicli's own. The two schemes give different
//! meanings to the same numbers, so a raw code must never reach a caller. The
//! process itself sits behind a trait, so tests can answer without running
//! anything.
//!
//! # `sch upgrade` is a project-level operation, never a file-level one
//!
//! kicli never runs `kicad-cli sch upgrade` on a user's file at all: it drops
//! bus aliases, which moved into the project file in KiCad 10.
//!
//! The rule is stricter still for a child sheet. Running `sch upgrade` on one
//! sheet of a hierarchy loads that file as its own root, so the sheet paths of
//! every placement fail to resolve, and KiCad prunes the instance data of all
//! but one of them on save. A sheet placed twice comes back with one reference
//! where it had two, and the loss is silent: the file still parses, still opens,
//! and now describes a different circuit. Any upgrade or canonicalisation runs
//! over a whole project, from its root sheet, or it does not run.
//!
//! Measured against KiCad 10.0.5 while building the connectivity fixtures.

mod discovery;
mod runner;

pub use discovery::{Discovery, ENVIRONMENT_VARIABLE, MACOS_INSTALL};
pub use runner::{Completed, Invocation, Runner, SystemRunner};

use std::path::{Path, PathBuf};

/// The binary kicli calls.
pub const PROGRAM: &str = "kicad-cli";

/// The major version kicli reads and writes.
///
/// Both the file format and the report schema move between major versions, so
/// a different major version is refused rather than tried.
pub const MAJOR_VERSION: u32 = 10;

/// What to tell a caller who has no usable `kicad-cli`.
pub const INSTALL_HINT: &str =
    "Install KiCad 10, or name the binary in kicli.toml as tools.kicad_cli_path.";

/// Why a `kicad-cli` call did not give kicli an answer.
///
/// The variants are the readings of `kicad-cli`'s exit codes, not the codes.
/// Translation happens once, where a code is read, so no later site can pass
/// one through by mistake.
#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
pub enum CliFailure {
    /// No place holds the binary.
    #[error("cannot find {program}. kicli looked in: {}. {hint}", searched.join(", "))]
    NotFound {
        /// The binary kicli looked for.
        program: String,
        /// Every place it looked, in order.
        searched: Vec<String>,
        /// How to install KiCad.
        hint: String,
    },

    /// The binary is there, and the operating system will not start it.
    #[error("cannot run {program}: {reason}. {hint}")]
    NotUsable {
        /// The binary kicli tried to start.
        program: String,
        /// What the operating system reported.
        reason: String,
        /// How to install KiCad.
        hint: String,
    },

    /// The binary is a major version kicli does not read.
    #[error(
        "{program} is version {found}. kicli needs major version {needed}. The file format and the report schema both move between major versions. {hint}"
    )]
    WrongVersion {
        /// The binary kicli asked.
        program: String,
        /// The version it reported.
        found: String,
        /// The major version kicli needs.
        needed: u32,
        /// How to install KiCad.
        hint: String,
    },

    /// `kicad-cli` reports that an input file is not valid.
    #[error("{command} reports that an input file is not valid. {message}")]
    BadInputFile {
        /// The command line kicli ran.
        command: String,
        /// What `kicad-cli` said about it.
        message: String,
    },

    /// `kicad-cli` did not complete the work.
    #[error("{command} did not complete. {message}")]
    Failed {
        /// The command line kicli ran.
        command: String,
        /// Why, as far as kicli can tell.
        message: String,
    },
}

/// The gateway to `kicad-cli`.
///
/// The runner is a type parameter, so a test drives the whole gateway with a
/// fake and never starts a process.
#[derive(Clone, Debug)]
pub struct KicadCli<R: Runner = SystemRunner> {
    /// The binary this gateway calls.
    program: PathBuf,
    /// How a command is run.
    runner: R,
}

impl KicadCli<SystemRunner> {
    /// Find the binary, and call it as a child process.
    ///
    /// # Errors
    ///
    /// Returns [`CliFailure::NotFound`] when no place holds the binary.
    pub fn locate(discovery: &Discovery) -> Result<Self, CliFailure> {
        Ok(Self::with_runner(discovery.locate()?, SystemRunner))
    }
}

impl<R: Runner> KicadCli<R> {
    /// Build a gateway over a given binary and a given runner.
    pub const fn with_runner(program: PathBuf, runner: R) -> Self {
        Self { program, runner }
    }

    /// The binary this gateway calls.
    #[must_use]
    pub fn program(&self) -> &Path {
        &self.program
    }

    /// Run `kicad-cli` and return what it wrote to standard output.
    ///
    /// # Errors
    ///
    /// Returns a [`CliFailure`] describing what `kicad-cli` reported. The raw
    /// exit code is read here and nowhere else.
    pub fn run(&self, arguments: &[&str]) -> Result<String, CliFailure> {
        let invocation = Invocation::new(self.program.clone(), arguments);
        let completed = self
            .runner
            .run(&invocation)
            .map_err(|error| CliFailure::NotUsable {
                program: self.program.display().to_string(),
                reason: error.to_string(),
                hint: INSTALL_HINT.to_owned(),
            })?;

        match read_status(&invocation, &completed) {
            Some(failure) => Err(failure),
            None => Ok(completed.stdout),
        }
    }

    /// The version of the binary, checked against the major version kicli reads.
    ///
    /// # Errors
    ///
    /// Returns [`CliFailure::WrongVersion`] when the major version is not the
    /// one kicli reads, and a [`CliFailure`] from the call itself otherwise.
    pub fn version(&self) -> Result<String, CliFailure> {
        let reported = self.run(&["version", "--format", "plain"])?;
        let found = reported.trim().lines().next().unwrap_or("").trim();

        if major_version(found) == Some(MAJOR_VERSION) {
            return Ok(found.to_owned());
        }
        Err(CliFailure::WrongVersion {
            program: self.program.display().to_string(),
            found: found.to_owned(),
            needed: MAJOR_VERSION,
            hint: INSTALL_HINT.to_owned(),
        })
    }
}

/// The major version of a version string such as `10.0.5`.
fn major_version(reported: &str) -> Option<u32> {
    reported.split('.').next()?.trim().parse().ok()
}

/// Read one of `kicad-cli`'s exit codes.
///
/// `kicad-cli` uses `0 OK, 1 ERR_ARGS, 2 ERR_UNKNOWN, 3 ERR_INVALID_INPUT_FILE,
/// 5 ERR_RC_VIOLATIONS, 6 ERR_JOBS_RUN_FAILED`, per
/// `include/cli/exit_codes.h` at tag 10.0.5. kicli gives four of those numbers
/// a different meaning, so this function turns each one into a reading and the
/// number is not carried any further.
fn read_status(invocation: &Invocation, completed: &Completed) -> Option<CliFailure> {
    let command = invocation.command_line();
    let said = detail(&completed.stderr);

    let message = match completed.code {
        Some(0) => return None,
        // kicli built the command line, so bad arguments are a kicli fault.
        Some(1) => {
            format!("kicli built a command line {PROGRAM} rejected. This is a kicli bug.{said}")
        }
        Some(2) => format!("{PROGRAM} reports an error it does not name.{said}"),
        Some(3) => {
            return Some(CliFailure::BadInputFile {
                command,
                message: format!("Check that the file exists and that it parses.{said}"),
            });
        }
        // kicli reads the report instead of asking for this code, so seeing it
        // means the command line carried --exit-code-violations by mistake.
        Some(5) => {
            format!("{PROGRAM} reports rule violations, which kicli never asks it for.{said}")
        }
        Some(6) => format!("{PROGRAM} could not run the job.{said}"),
        Some(other) => format!("{PROGRAM} reports {other}, which kicli does not know.{said}"),
        None => format!("A signal ended {PROGRAM}.{said}"),
    };
    Some(CliFailure::Failed { command, message })
}

/// What the command wrote to standard error, when it is worth repeating.
///
/// `kicad-cli` writes fontconfig warnings to standard error on every run, so a
/// bare copy of the stream is noise. Only the last line is kept.
fn detail(stderr: &str) -> String {
    let last = stderr
        .lines()
        .map(str::trim)
        .rfind(|line| !line.is_empty() && !line.starts_with("Fontconfig"));
    match last {
        Some(line) => format!(" It said: {line}"),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::{CliFailure, Completed, Invocation, major_version, read_status};
    use std::path::PathBuf;

    fn completed(code: Option<i32>) -> Completed {
        Completed {
            code,
            stdout: String::new(),
            stderr: String::new(),
        }
    }

    fn call() -> Invocation {
        Invocation::new(PathBuf::from("kicad-cli"), &["sch", "erc", "a.kicad_sch"])
    }

    #[test]
    fn success_is_not_a_failure() {
        assert!(read_status(&call(), &completed(Some(0))).is_none());
    }

    #[test]
    fn only_an_invalid_input_file_is_a_file_fault() {
        assert!(matches!(
            read_status(&call(), &completed(Some(3))),
            Some(CliFailure::BadInputFile { .. })
        ));
        for other in [1, 2, 5, 6, 42] {
            assert!(
                matches!(
                    read_status(&call(), &completed(Some(other))),
                    Some(CliFailure::Failed { .. })
                ),
                "{other} is read as a failed run"
            );
        }
    }

    #[test]
    fn a_signal_is_a_failure_too() {
        assert!(matches!(
            read_status(&call(), &completed(None)),
            Some(CliFailure::Failed { .. })
        ));
    }

    #[test]
    fn font_cache_warnings_are_not_repeated() {
        let noisy = Completed {
            code: Some(2),
            stdout: String::new(),
            stderr: "Fontconfig warning: 49-sansserif.conf\n".to_owned(),
        };
        let failure = read_status(&call(), &noisy).expect("a failure");
        assert!(!failure.to_string().contains("Fontconfig"), "{failure}");
    }

    #[test]
    fn the_major_version_is_the_first_number() {
        assert_eq!(major_version("10.0.5"), Some(10));
        assert_eq!(major_version("9.0.1"), Some(9));
        assert_eq!(major_version(""), None);
        assert_eq!(major_version("nightly"), None);
    }
}
