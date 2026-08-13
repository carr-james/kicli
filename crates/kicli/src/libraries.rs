//! Library resolution, the parts catalogue, and vendoring.
//!
//! This module resolves a `lib_id` through the project and global library
//! tables. It vendors parts into a project or up into the shared submodule.
//! Vendoring must also rewrite the embedded `lib_symbols` cache. See
//! `spec/SPEC.md` §10.
