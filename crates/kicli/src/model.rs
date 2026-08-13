//! Typed schematic objects over the s-expression tree.
//!
//! This module names the objects the rest of kicli works with: symbols, fields,
//! wires, labels, junctions, sheets, and sheet paths. It resolves a reference
//! designator through the symbol's `instances` list, not through the cached
//! `Reference` property. A symbol on a sheet that is instantiated twice has two
//! references, and only the instance list holds both.
