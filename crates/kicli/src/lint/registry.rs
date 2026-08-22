//! Every rule the crate holds, listed by the build rather than by hand.
//!
//! The build script reads `src/lint/rules/`, writes one module declaration per
//! file, and writes the list below. A rule is therefore a new file and nothing
//! else: no module list to edit, and no shared registry for two authors to
//! collide in.
//!
//! Each rule file declares `pub static RULES`, a slice of the rules that file
//! holds. One file may hold a family of rules that share a definition.
//!
//! `cargo test --test lint_rules_register_from_their_own_files` reads the same
//! directory at run time and compares. A hand-written list would fail it the
//! first time a file was added.

include!(concat!(env!("OUT_DIR"), "/lint_rules.rs"));
