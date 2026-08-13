//! Net extraction from geometry and names.
//!
//! This module builds the net partition with union-find over wire endpoints,
//! segment interiors, pins, labels, and power-symbol values. The name-based
//! merges are mandatory. Geometry alone gives the wrong answer. See
//! `spec/SPEC.md` §7.1.
