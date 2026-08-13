//! Every call to an external KiCad binary goes through here.
//!
//! This module finds `kicad-cli`, checks that it is version 10, runs it, and
//! translates its exit codes into kicli's own. The two schemes give different
//! meanings to the same numbers, so a raw code must never reach a caller. The
//! process itself sits behind a trait, so tests can answer without running
//! anything. kicli never runs `sch upgrade` on a user's file: it destroys bus
//! aliases.
