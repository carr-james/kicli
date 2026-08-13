//! Region cropping, annotation overlay, and rasterisation.
//!
//! This module crops a `kicad-cli` SVG by rewriting its `viewBox`. It appends
//! one annotation group. It rasterises to PNG. Renders are passive output. No
//! render ever feeds the score. See `spec/SPEC.md` §12.
