//! Deterministic style rules and the readability score.
//!
//! This module scores how a schematic is drawn. It layers on KiCad's own
//! electrical rule check and repeats none of those checks, because the
//! electrical layer already owns them. Detection uses integer geometry only, so
//! two runs over one file always agree.
