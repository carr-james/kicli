//! The kicli command-line binary.
//!
//! This binary is a thin shell over the library. It parses arguments and
//! renders results. The exit code it returns is the library's answer, never a
//! number this file knows.

#![deny(unsafe_code)]
#![warn(clippy::pedantic)]

fn main() -> std::process::ExitCode {
    kicli::cli::run().into()
}
