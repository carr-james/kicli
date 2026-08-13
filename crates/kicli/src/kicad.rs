//! Every call to an external KiCad binary goes through here.
//!
//! This module finds `kicad-cli`, checks that it is version 10, runs it, and
//! translates its exit codes into kicli's own. The two schemes give different
//! meanings to the same numbers, so a raw code must never reach a caller. The
//! process itself sits behind a trait, so tests can answer without running
//! anything.
//!
//! # `sch upgrade` is a project-level operation, never a file-level one
//!
//! kicli never runs `kicad-cli sch upgrade` on a user's file at all: it drops
//! bus aliases, which moved into the project file in KiCad 10.
//!
//! The rule is stricter still for a child sheet. Running `sch upgrade` on one
//! sheet of a hierarchy loads that file as its own root, so the sheet paths of
//! every placement fail to resolve, and KiCad prunes the instance data of all
//! but one of them on save. A sheet placed twice comes back with one reference
//! where it had two, and the loss is silent: the file still parses, still opens,
//! and now describes a different circuit. Any upgrade or canonicalisation runs
//! over a whole project, from its root sheet, or it does not run.
//!
//! Measured against KiCad 10.0.5 while building the connectivity fixtures.
