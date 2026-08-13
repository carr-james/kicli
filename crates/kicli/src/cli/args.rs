//! The `kicli <noun> <verb> [flags] [args]` surface.
//!
//! One type per noun, one variant per verb. The global flags are declared once
//! and accepted before or after the verb, so an agent does not have to remember
//! where a flag goes.

use clap::{Args, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

/// The whole command line.
#[derive(Debug, Parser)]
#[command(
    name = "kicli",
    version,
    about = "Read and edit KiCad 10 schematics from the command line.",
    long_about = None,
    disable_help_subcommand = true
)]
pub struct Cli {
    /// The flags every command accepts.
    #[command(flatten)]
    pub global: Global,

    /// The noun, and its verb.
    #[command(subcommand)]
    pub command: Command,
}

/// The flags every command accepts.
#[derive(Args, Clone, Debug)]
pub struct Global {
    /// Print results as text or as JSON.
    #[arg(long, global = true, value_enum, default_value_t = OutputFormat::Text)]
    pub output: OutputFormat,

    /// The project directory. The default is the working directory.
    #[arg(long, short = 'p', global = true, value_name = "DIR")]
    pub project: Option<PathBuf>,

    /// One sheet path, to read instead of the whole project.
    #[arg(long, global = true, value_name = "PATH")]
    pub sheet: Option<String>,

    /// Print results only. Suppress progress notes.
    #[arg(long, short = 'q', global = true)]
    pub quiet: bool,

    // A variant selects one set of field overrides. kicli keeps variant data
    // through a round trip and does not act on it, so the flag is accepted and
    // hidden rather than advertised.
    /// A design variant. Accepted, and without effect in this version.
    #[arg(long, global = true, hide = true, value_name = "NAME")]
    pub variant: Option<String>,
}

/// How a result is printed.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    /// Terse lines, for a person and for an agent's context budget.
    #[default]
    Text,
    /// One JSON object, carrying the same content.
    Json,
}

/// The nouns.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Read a whole project.
    Project {
        /// What to report about it.
        #[command(subcommand)]
        verb: ProjectVerb,
    },
    /// Read the schematics.
    Sch {
        /// What to do with them.
        #[command(subcommand)]
        verb: SchVerb,
    },
}

/// The verbs of the `sch` noun.
#[derive(Clone, Copy, Debug, Subcommand)]
pub enum SchVerb {
    /// Print a compact view of the drawing.
    View {
        /// Which view to print.
        #[arg(long, value_enum, default_value_t = ViewName::Connectivity)]
        view: ViewName,

        /// List power symbols, which are otherwise left out as noise.
        #[arg(long)]
        include_power: bool,

        /// Add the first eight characters of each object's identifier.
        #[arg(long)]
        uuids: bool,

        /// Report the size of the view in bytes.
        #[arg(long)]
        stats: bool,
    },
}

/// The views `sch view` can print.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, ValueEnum)]
pub enum ViewName {
    /// What is joined to what.
    #[default]
    Connectivity,
    /// Where things are drawn.
    Layout,
}

/// The verbs of the `project` noun.
#[derive(Clone, Copy, Debug, Subcommand)]
pub enum ProjectVerb {
    /// Report what the project is.
    Info,
    /// Report what is wrong with the project.
    Check,
}
