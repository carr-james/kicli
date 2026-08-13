//! Library resolution, the parts catalogue, and vendoring.
//!
//! This module resolves a `lib_id` through the project and global library
//! tables. It vendors parts into a project, or up into the shared library.
//! Vendoring must also rewrite the copy embedded in each schematic, because
//! KiCad draws the embedded copy and not the library file.
