//! Typed schematic objects over the s-expression tree.
//!
//! This module names the objects the rest of kicli works with: symbols, fields,
//! wires, labels, junctions, sheets, and sheet paths. It resolves a reference
//! designator through the symbol's `instances` list, not through the cached
//! `Reference` property. A symbol on a sheet that is instantiated twice has two
//! references, and only the instance list holds both.
//!
//! The parts that exist so far are the ones the parser core needs: what a
//! file's version stamp changes about its tokens, what kicli will and will not
//! write, and the project file that holds bus aliases.

pub mod config;
pub mod hierarchy;
pub mod invariant;
pub mod items;
pub mod library;
pub mod project;
pub mod version;
pub mod write;
pub mod write_file;

pub use config::{Config, ConfigError, Formats, Grid, Ipc, Tools, View};
pub use hierarchy::{Hierarchy, LoadError, LoadedFile, Placement, Problem};
pub use invariant::{
    Invariant, Outcome, Report as InvariantReport, check_hierarchy, check_invariants,
};
pub use items::{
    Field, Item, Label, LabelKind, LibId, Line, LineKind, Mirror, PinInstance, PointItem,
    ReadError, Refdes, Schematic, SheetItem, SheetPath, SheetPin, Symbol, SymbolPlacement,
    TextItem, Uuid,
};
pub use library::{LibraryPin, LibrarySymbol, LibraryUnit, Shape, definition_of, read_library};
pub use project::{BusAlias, Project, ProjectError, read_project};
pub use version::{FormatVersion, MAX_SCHEMATIC_VERSION, PropertyOrder, pin_text};
pub use write::{WriteOptions, WritePlan, WriteRefusal, format_version, plan_write};
pub use write_file::{FileSystem, Sink, WriteError, Written, write_document, write_document_with};
