//! Where a result goes, and what it looks like when it gets there.
//!
//! Results go to standard output. Progress notes and errors go to standard
//! error, so a caller that reads JSON from standard output always reads JSON
//! and nothing else. Text is the default form and JSON is its twin: the same
//! content, chosen with `--output json`.

use super::args::{Global, OutputFormat};
use super::exit::ExitCode;
use serde_json::{Value, json};

/// A command that could not be completed.
#[derive(Clone, Debug, thiserror::Error)]
#[error("{message}")]
pub struct Failure {
    /// Which row of the exit-code table this is.
    pub code: ExitCode,
    /// What went wrong, in one sentence.
    pub message: String,
}

impl Failure {
    /// Build a failure from a code and a message.
    #[must_use]
    pub fn new(code: ExitCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    /// The JSON form: one object, so a caller can parse it whole.
    #[must_use]
    pub fn to_json(&self) -> Value {
        json!({
            "error": {
                "kind": self.code.name(),
                "exit_code": self.code.code(),
                "message": self.message,
            }
        })
    }
}

/// The result of a command, in both forms.
pub struct Report {
    /// The text form. It ends with a newline.
    pub text: String,
    /// The JSON form, carrying the same content.
    pub json: Value,
}

/// Writes results, notes and errors in the form the caller asked for.
pub struct Reporter {
    /// Text or JSON.
    format: OutputFormat,
    /// Are progress notes suppressed?
    quiet: bool,
}

impl Reporter {
    /// Build a reporter from the global flags.
    #[must_use]
    pub fn new(global: &Global) -> Self {
        Self {
            format: global.output,
            quiet: global.quiet,
        }
    }

    /// The form results are written in.
    #[must_use]
    pub const fn format(&self) -> OutputFormat {
        self.format
    }

    /// Print a progress note on standard error.
    ///
    /// A note says what kicli is about to do. `--quiet` drops it. Notes never
    /// go to standard output, whatever the form, because a caller reading JSON
    /// must not have to filter it out.
    pub fn note(&self, message: &str) {
        if !self.quiet {
            eprintln!("kicli: {message}");
        }
    }

    /// Print a result on standard output.
    pub fn result(&self, report: &Report) {
        match self.format {
            OutputFormat::Text => print!("{}", report.text),
            OutputFormat::Json => println!("{}", report.json),
        }
    }

    /// Print a failure on standard error, and return its exit code.
    #[must_use]
    pub fn failure(&self, failure: &Failure) -> ExitCode {
        match self.format {
            OutputFormat::Text => eprintln!("kicli: {}", failure.message),
            OutputFormat::Json => eprintln!("{}", failure.to_json()),
        }
        failure.code
    }
}

#[cfg(test)]
mod tests {
    use super::{ExitCode, Failure};

    #[test]
    fn a_failure_reports_its_code_by_name_and_by_number() {
        let failure = Failure::new(ExitCode::File, "cannot read root.kicad_sch");
        let reported = failure.to_json();
        assert_eq!(reported["error"]["kind"], "file");
        assert_eq!(reported["error"]["exit_code"], 4);
        assert_eq!(reported["error"]["message"], "cannot read root.kicad_sch");
    }
}
