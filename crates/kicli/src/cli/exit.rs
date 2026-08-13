//! The exit-code table.
//!
//! This is the only place in kicli where an exit code is a number. Every other
//! site names a row of the table. A caller reads the number, so a second site
//! that knew one would be free to disagree with this one, and an agent that
//! looked the number up would be misled.

/// What a kicli run reports to whoever started it.
///
/// Findings are data, not failure. A command that reports problems in a project
/// it read successfully exits [`ExitCode::Success`].
///
/// # Examples
///
/// ```
/// use kicli::cli::ExitCode;
///
/// assert_eq!(ExitCode::Success.code(), 0);
/// assert_eq!(ExitCode::Usage.name(), "usage");
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ExitCode {
    /// The command did what was asked.
    Success,
    /// A well-formed request that could not be completed.
    Operation,
    /// Bad flags or arguments.
    Usage,
    /// A mutation was verified, failed, and was rolled back. No file changed.
    Verification,
    /// A file could not be read or parsed, or kicli refused to write one.
    File,
    /// A gate found the findings it was told to fail on.
    Gate,
    /// A required external tool is missing or is the wrong version.
    Tool,
}

impl ExitCode {
    /// Every row of the table, in code order.
    pub const ALL: &'static [Self] = &[
        Self::Success,
        Self::Operation,
        Self::Usage,
        Self::Verification,
        Self::File,
        Self::Gate,
        Self::Tool,
    ];

    /// The number a caller reads.
    #[must_use]
    pub const fn code(self) -> u8 {
        match self {
            Self::Success => 0,
            Self::Operation => 1,
            Self::Usage => 2,
            Self::Verification => 3,
            Self::File => 4,
            Self::Gate => 5,
            Self::Tool => 6,
        }
    }

    /// The short name kicli uses for this code in its own output.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::Operation => "operation",
            Self::Usage => "usage",
            Self::Verification => "verification",
            Self::File => "file",
            Self::Gate => "gate",
            Self::Tool => "tool",
        }
    }

    /// One sentence saying what the code means.
    #[must_use]
    pub const fn meaning(self) -> &'static str {
        match self {
            Self::Success => "the command did what was asked; findings are data, not failure",
            Self::Operation => "a well-formed request that kicli could not complete",
            Self::Usage => "kicli did not understand the flags or the arguments",
            Self::Verification => "a mutation failed its own checks and was rolled back",
            Self::File => "a file did not read or parse, or kicli refused to write it",
            Self::Gate => "a gate found the findings it was told to fail on",
            Self::Tool => "a required external tool is missing or is the wrong version",
        }
    }

    /// Did the command succeed?
    #[must_use]
    pub const fn is_success(self) -> bool {
        matches!(self, Self::Success)
    }
}

impl From<ExitCode> for std::process::ExitCode {
    fn from(code: ExitCode) -> Self {
        Self::from(code.code())
    }
}

#[cfg(test)]
mod tests {
    use super::ExitCode;

    #[test]
    fn the_table_holds_one_row_for_each_code() {
        let mut codes: Vec<u8> = ExitCode::ALL.iter().map(|row| row.code()).collect();
        codes.sort_unstable();
        codes.dedup();
        assert_eq!(codes.len(), ExitCode::ALL.len());
    }

    #[test]
    fn only_success_is_success() {
        for row in ExitCode::ALL {
            assert_eq!(row.is_success(), row.code() == 0, "{}", row.name());
        }
    }
}
