//! Where kicli looks for `kicad-cli`.
//!
//! Nothing is bundled and nothing is vendored. `kicad-cli` belongs to a full
//! KiCad install, so kicli finds the one on the machine or reports that there
//! is none.

use super::{CliFailure, INSTALL_HINT, PROGRAM};
use crate::model::Config;
use std::path::PathBuf;

/// The environment variable that names the binary directly.
pub const ENVIRONMENT_VARIABLE: &str = "KICLI_KICAD_CLI";

/// Where a KiCad install puts the binary on macOS.
pub const MACOS_INSTALL: &str = "/Applications/KiCad/KiCad.app/Contents/MacOS/kicad-cli";

/// The places kicli looks, in the order it looks in them.
const PLACES: &[&str] = &[
    "$KICLI_KICAD_CLI",
    "kicli.toml tools.kicad_cli_path",
    "PATH",
    MACOS_INSTALL,
];

/// What the caller knows about where the binary is.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Discovery {
    /// The value of `$KICLI_KICAD_CLI`, when it is set.
    ///
    /// It is an instruction rather than a hint: when it is set, kicli uses that
    /// binary or reports that there is none. Falling through to the rest of the
    /// order would hide a typed path that is wrong.
    pub environment: Option<String>,
    /// The `tools.kicad_cli_path` of the project's `kicli.toml`.
    pub configured: Option<String>,
}

impl Discovery {
    /// Read the environment, and take the rest from the project configuration.
    #[must_use]
    pub fn new(config: &Config) -> Self {
        Self {
            environment: std::env::var(ENVIRONMENT_VARIABLE).ok(),
            configured: config.tools.kicad_cli_path.clone(),
        }
    }

    /// The places kicli looks, in order, for an error message.
    #[must_use]
    pub const fn places() -> &'static [&'static str] {
        PLACES
    }

    /// Find the binary.
    ///
    /// # Errors
    ///
    /// Returns [`CliFailure::NotFound`] when no place holds it. The error names
    /// every place kicli looked and how to install KiCad.
    pub fn locate(&self) -> Result<PathBuf, CliFailure> {
        self.candidates()
            .into_iter()
            .find(|candidate| candidate.is_file())
            .ok_or_else(|| CliFailure::NotFound {
                program: PROGRAM.to_owned(),
                searched: PLACES.iter().map(|&place| place.to_owned()).collect(),
                hint: INSTALL_HINT.to_owned(),
            })
    }

    /// Every path that might hold the binary, in order.
    fn candidates(&self) -> Vec<PathBuf> {
        if let Some(named) = &self.environment {
            return vec![PathBuf::from(named)];
        }
        let mut found: Vec<PathBuf> = Vec::new();
        if let Some(configured) = &self.configured {
            found.push(PathBuf::from(configured));
        }
        found.extend(on_path());
        found.push(PathBuf::from(MACOS_INSTALL));
        found
    }
}

/// Every entry of `PATH`, with the program name joined on.
fn on_path() -> Vec<PathBuf> {
    let Some(path) = std::env::var_os("PATH") else {
        return Vec::new();
    };
    std::env::split_paths(&path)
        .filter(|entry| !entry.as_os_str().is_empty())
        .map(|entry| entry.join(PROGRAM))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{Discovery, MACOS_INSTALL};
    use std::path::PathBuf;

    #[test]
    fn a_named_binary_is_the_only_candidate() {
        let discovery = Discovery {
            environment: Some("/opt/kicad-cli".to_owned()),
            configured: Some("/ignored/kicad-cli".to_owned()),
        };
        assert_eq!(discovery.candidates(), [PathBuf::from("/opt/kicad-cli")]);
    }

    #[test]
    fn the_configured_path_comes_before_the_path_variable() {
        let discovery = Discovery {
            environment: None,
            configured: Some("/opt/kicad-cli".to_owned()),
        };
        let candidates = discovery.candidates();
        assert_eq!(candidates[0], PathBuf::from("/opt/kicad-cli"));
        assert_eq!(
            candidates.last().expect("there is a last candidate"),
            &PathBuf::from(MACOS_INSTALL),
            "the install location is the last resort"
        );
    }

    #[test]
    fn nothing_is_found_when_the_named_binary_is_not_there() {
        let discovery = Discovery {
            environment: Some("/nonexistent/kicad-cli".to_owned()),
            configured: None,
        };
        assert!(discovery.locate().is_err());
    }
}
