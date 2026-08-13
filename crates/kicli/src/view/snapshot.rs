//! Per-object content hashes, and the snapshot file that holds them.
//!
//! A snapshot is a map from object identifier to content hash. The hash covers
//! a canonical encoding of the object's own meaning and never its position in
//! the file, because KiCad reorders every item on save and a file hash would
//! make each save look like a total rewrite.
