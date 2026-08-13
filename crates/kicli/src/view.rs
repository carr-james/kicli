//! Compact representations of a schematic, for agents with a context budget.
//!
//! This module emits the three views: connectivity, layout, and delta. Terse
//! text is the default and JSON is its twin, because the same content costs
//! more than twice as much as JSON. A view states its own scope, so a reader
//! always knows whether it covers one sheet or the whole project. Renders live
//! in [`crate::render`] and are pictures; the views here are what an agent
//! acts on.
