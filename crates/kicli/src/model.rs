//! Typed schematic objects over the s-expression tree.
//!
//! This module names the objects the rest of kicli works with: symbols, fields,
//! wires, labels, junctions, sheets, and sheet paths. It resolves a reference
//! designator through `instances`, not through the cached property. See
//! `spec/SPEC.md` §5.5.
