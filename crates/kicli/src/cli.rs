//! Command surface, output formatting, and exit codes.
//!
//! This module parses arguments and renders results as text or JSON. It owns
//! the exit-code table. It translates `kicad-cli`'s exit codes into kicli's own,
//! because the two schemes give different meanings to the same numbers. It
//! depends on the other modules. No other module depends on it.

mod args;
mod check;
mod exit;
mod locate;
mod output;
mod project;
mod tools;
mod view;

pub use args::{Cli, Command, Global, OutputFormat, ProjectVerb, SchVerb};
pub use exit::ExitCode;
pub use output::{Failure, Report, Reporter};

use clap::Parser;
use std::ffi::OsString;

/// Run kicli with the arguments of this process.
///
/// The returned code is the one the caller reads. Errors are already printed.
#[must_use]
pub fn run() -> ExitCode {
    run_with(std::env::args_os())
}

/// Run kicli with a given argument list, the program name first.
///
/// # Examples
///
/// ```
/// use kicli::cli::{ExitCode, run_with};
///
/// assert_eq!(run_with(["kicli", "--version"]), ExitCode::Success);
/// ```
#[must_use]
pub fn run_with<I, T>(arguments: I) -> ExitCode
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let arguments: Vec<OsString> = arguments.into_iter().map(Into::into).collect();

    let parsed = match Cli::try_parse_from(&arguments) {
        Ok(parsed) => parsed,
        Err(error) => return report_parse_error(&error, &arguments),
    };

    let reporter = Reporter::new(&parsed.global);
    if parsed.global.variant.is_some() {
        reporter.note("--variant is accepted and has no effect in this version.");
    }

    match dispatch(&parsed, &reporter) {
        Ok(report) => {
            reporter.result(&report);
            ExitCode::Success
        }
        Err(failure) => reporter.failure(&failure),
    }
}

/// Send one parsed command to the code that answers it.
fn dispatch(parsed: &Cli, reporter: &Reporter) -> Result<Report, Failure> {
    match parsed.command {
        Command::Project {
            verb: ProjectVerb::Info,
        } => project::info(&parsed.global, reporter),
        Command::Project {
            verb: ProjectVerb::Check,
        } => check::check(&parsed.global, reporter),
        Command::Sch {
            verb:
                SchVerb::View {
                    view: which,
                    include_power,
                    uuids,
                    stats,
                },
        } => view::view(&parsed.global, which, include_power, uuids, stats, reporter),
    }
}

/// Print what the argument parser said, in the form the caller asked for.
///
/// Help and the version are results, not errors: they go to standard output and
/// the run succeeds. Everything else is a usage error on standard error, and
/// standard output stays empty so a caller reading JSON reads only the error.
fn report_parse_error(error: &clap::Error, arguments: &[OsString]) -> ExitCode {
    if !error.use_stderr() {
        print!("{error}");
        return ExitCode::Success;
    }

    let failure = Failure::new(ExitCode::Usage, error.to_string().trim_end().to_owned());
    if json_was_asked_for(arguments) {
        eprintln!("{}", failure.to_json());
    } else {
        eprint!("{error}");
    }
    failure.code
}

/// Did the caller ask for JSON?
///
/// The argument parser failed, so the flags are not available in typed form.
/// The raw list is read instead, because an error in the form the caller cannot
/// parse is as good as no error at all.
fn json_was_asked_for(arguments: &[OsString]) -> bool {
    let mut wants = false;
    let mut expecting_value = false;
    for argument in arguments {
        let Some(text) = argument.to_str() else {
            continue;
        };
        if expecting_value {
            wants = text == "json";
            expecting_value = false;
        } else if text == "--output" {
            expecting_value = true;
        } else if let Some(value) = text.strip_prefix("--output=") {
            wants = value == "json";
        }
    }
    wants
}

#[cfg(test)]
mod tests {
    use super::{ExitCode, json_was_asked_for, run_with};
    use std::ffi::OsString;

    fn words(line: &str) -> Vec<OsString> {
        line.split_whitespace().map(OsString::from).collect()
    }

    #[test]
    fn the_output_flag_is_read_from_raw_arguments() {
        assert!(json_was_asked_for(&words("kicli --output json project")));
        assert!(json_was_asked_for(&words("kicli --output=json project")));
        assert!(json_was_asked_for(&words("kicli project --output json")));
        assert!(!json_was_asked_for(&words("kicli --output text project")));
        assert!(!json_was_asked_for(&words("kicli project")));
        assert!(!json_was_asked_for(&words("kicli --output")));
    }

    #[test]
    fn an_unknown_noun_is_a_usage_error() {
        assert_eq!(run_with(["kicli", "schematic"]), ExitCode::Usage);
    }

    #[test]
    fn the_version_flag_succeeds() {
        assert_eq!(run_with(["kicli", "--version"]), ExitCode::Success);
    }
}
