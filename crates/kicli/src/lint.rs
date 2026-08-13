//! Deterministic style rules and the readability score.
//!
//! This module implements the Tier 1 and Tier 2 rules in
//! `research/style-rules.md` §4. It layers on KiCad's ERC and duplicates none of
//! ERC's 47 checks. Detection uses integer geometry only. See `spec/SPEC.md`
//! §11.
