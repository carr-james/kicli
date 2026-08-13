//! The kicli command-line binary.
//!
//! This binary is a thin shell over the library. It parses arguments and
//! renders results.

#![deny(unsafe_code)]
#![warn(clippy::pedantic)]

use std::process::ExitCode;

/// Exit code 2 reports a usage error: kicli did not understand the arguments.
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
    eprintln!("The only supported flag is --version.");
    ExitCode::from(EXIT_USAGE)
}
