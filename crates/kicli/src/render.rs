//! Region cropping, annotation overlay, and rasterisation.
//!
//! This module crops a `kicad-cli` SVG by rewriting its `viewBox`. The SVG's
//! user units are millimetres and its origin is the page's top-left corner, so
//! a region in schematic coordinates needs no transform. The module appends one
//! annotation group, then rasterises to PNG. No render ever feeds the score.
