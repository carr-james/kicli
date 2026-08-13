//! Net extraction from geometry and names.
//!
//! This module builds the net partition with union-find over wire endpoints,
//! segment interiors, pins, labels, and power-symbol values. The name-based
//! merges are mandatory. Geometry alone splits one ground net into many,
//! because power symbols connect by value and not by wire.
