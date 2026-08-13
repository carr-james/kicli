//! Asking `kicad-cli` who it is, and saying so before the wait.
//!
//! The first KiCad run on a machine builds the font cache and can take over two
//! minutes. An agent that sees no output for two minutes decides kicli has
//! hung, so kicli says what it is about to do before it blocks.

use super::output::Reporter;
use crate::kicad::{CliFailure, Discovery, KicadCli};
use crate::model::Config;
use serde_json::{Value, json};
use std::path::PathBuf;

/// What kicli found when it looked for `kicad-cli`.
pub struct ToolStatus {
    /// The binary, when a place held one.
    pub program: Option<PathBuf>,
    /// The version it reported, when it is one kicli reads.
    pub version: Option<String>,
    /// What stopped kicli using it, when something did.
    pub problem: Option<CliFailure>,
}

impl ToolStatus {
    /// Is `kicad-cli` there, and of a version kicli reads?
    #[must_use]
    pub const fn is_usable(&self) -> bool {
        self.problem.is_none()
    }

    /// One line saying what the state is.
    #[must_use]
    pub fn summary(&self) -> String {
        let Some(program) = &self.program else {
            return "not found".to_owned();
        };
        let place = program.display();

        match (&self.problem, &self.version) {
            (None, Some(version)) => format!("{version} at {place}"),
            (Some(CliFailure::WrongVersion { found, .. }), _) => {
                format!("{found} at {place}, which kicli does not read")
            }
            _ => format!("not usable at {place}"),
        }
    }

    /// The JSON form.
    #[must_use]
    pub fn to_json(&self) -> Value {
        json!({
            "found": self.program.is_some(),
            "usable": self.is_usable(),
            "program": self.program.as_ref().map(|path| path.display().to_string()),
            "version": self.version,
            "problem": self.problem.as_ref().map(ToString::to_string),
        })
    }
}

/// Find `kicad-cli` and ask its version, saying so first.
#[must_use]
pub fn probe(reporter: &Reporter, config: &Config) -> ToolStatus {
    let discovery = Discovery::new(config);
    let program = match discovery.locate() {
        Ok(program) => program,
        Err(problem) => {
            return ToolStatus {
                program: None,
                version: None,
                problem: Some(problem),
            };
        }
    };

    reporter.note(&format!(
        "asking {} its version. The first KiCad run on a machine builds the font cache. It can take over 120 seconds.",
        program.display()
    ));

    let gateway = KicadCli::locate(&discovery);
    let asked = gateway.and_then(|gateway| gateway.version());
    match asked {
        Ok(version) => ToolStatus {
            program: Some(program),
            version: Some(version),
            problem: None,
        },
        Err(problem) => ToolStatus {
            program: Some(program),
            version: None,
            problem: Some(problem),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::ToolStatus;
    use crate::kicad::CliFailure;
    use std::path::PathBuf;

    #[test]
    fn a_usable_binary_reports_its_version_and_its_place() {
        let status = ToolStatus {
            program: Some(PathBuf::from("/opt/kicad-cli")),
            version: Some("10.0.5".to_owned()),
            problem: None,
        };
        assert!(status.is_usable());
        assert_eq!(status.summary(), "10.0.5 at /opt/kicad-cli");
    }

    #[test]
    fn a_binary_of_the_wrong_version_is_named_with_its_version() {
        let status = ToolStatus {
            program: Some(PathBuf::from("/opt/kicad-cli")),
            version: None,
            problem: Some(CliFailure::WrongVersion {
                program: "/opt/kicad-cli".to_owned(),
                found: "9.0.1".to_owned(),
                needed: 10,
                hint: String::new(),
            }),
        };
        assert!(!status.is_usable());
        assert!(status.summary().starts_with("9.0.1 at /opt/kicad-cli"));
    }
}
