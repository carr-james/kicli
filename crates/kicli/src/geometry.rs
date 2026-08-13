//! Integer geometry: transforms, pin positions, bounding boxes, and text extents.
//!
//! This module computes where things are drawn. All arithmetic uses integer
//! internal units of 100 nm. The module knows nothing about the CLI, files on
//! disk, or `kicad-cli`. See `spec/SPEC.md` §8 and `research/geometry.md`.
