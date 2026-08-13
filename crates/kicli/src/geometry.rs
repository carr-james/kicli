//! Integer geometry: transforms, pin positions, bounding boxes, and text extents.
//!
//! This module computes where things are drawn. All arithmetic uses integer
//! internal units of 100 nm, so no coordinate passes through a float and back.
//! The module knows nothing about the command surface, files on disk, or
//! `kicad-cli`.
