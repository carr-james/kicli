//! The process seam.
//!
//! Starting a process is the one thing in this module a test cannot do
//! cheaply, so it sits behind a trait. A fake runner answers with the exit code
//! and the output a test wants, and every other rule in the module — discovery,
//! the version check, the translation — runs unchanged over the answer.

use std::path::PathBuf;
use std::process::Command;

/// One external command, ready to run.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Invocation {
    /// The binary to start.
    pub program: PathBuf,
    /// Its arguments, in order.
    pub arguments: Vec<String>,
}

impl Invocation {
    /// Build an invocation.
    #[must_use]
    pub fn new(program: PathBuf, arguments: &[&str]) -> Self {
        Self {
            program,
            arguments: arguments
                .iter()
                .map(|&argument| argument.to_owned())
                .collect(),
        }
    }

    /// The command line, for an error message.
    ///
    /// # Examples
    ///
    /// ```
    /// use kicli::kicad::Invocation;
    /// use std::path::PathBuf;
    ///
    /// let call = Invocation::new(PathBuf::from("kicad-cli"), &["version"]);
    /// assert_eq!(call.command_line(), "kicad-cli version");
    /// ```
    #[must_use]
    pub fn command_line(&self) -> String {
        let mut line = self.program.display().to_string();
        for argument in &self.arguments {
            line.push(' ');
            line.push_str(argument);
        }
        line
    }
}

/// What running a command produced.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Completed {
    /// The exit code, or `None` when a signal ended the process.
    pub code: Option<i32>,
    /// Everything the command wrote to standard output.
    pub stdout: String,
    /// Everything the command wrote to standard error.
    pub stderr: String,
}

/// Runs an external command and captures what it produced.
pub trait Runner {
    /// Run one command to completion.
    ///
    /// # Errors
    ///
    /// Returns the error the operating system reported when the process could
    /// not be started at all.
    fn run(&self, invocation: &Invocation) -> Result<Completed, std::io::Error>;
}

/// Runs commands as child processes of this one.
#[derive(Clone, Copy, Debug, Default)]
pub struct SystemRunner;

impl Runner for SystemRunner {
    fn run(&self, invocation: &Invocation) -> Result<Completed, std::io::Error> {
        let output = Command::new(&invocation.program)
            .args(&invocation.arguments)
            .output()?;
        Ok(Completed {
            code: output.status.code(),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        })
    }
}
