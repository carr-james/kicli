//! The kicli command-line binary.
//!
//! This binary is a thin shell over the library. It parses arguments and
//! renders results. The command surface is specified in `spec/SPEC.md` §6.

#![deny(unsafe_code)]
#![warn(clippy::pedantic)]

use std::process::ExitCode;

/// Exit code 2 means a usage error. See `spec/SPEC.md` §6.1.
const EXIT_USAGE: u8 = 2;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.iter().any(|arg| arg == "--version") {
        println!("kicli {}", kicli::version());
        return ExitCode::SUCCESS;
    }

    eprintln!(
        "kicli {}: no commands are implemented yet.",
        kicli::version()
    );
    eprintln!("Milestone M1 builds the parser core. See tasks/M1.md.");
    ExitCode::from(EXIT_USAGE)
}
