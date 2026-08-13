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

pub mod hierarchy;
pub mod items;
pub mod project;
pub mod version;
pub mod write;

pub use hierarchy::{Hierarchy, LoadError, LoadedFile, Placement, Problem};
pub use items::{
    Field, Item, Label, LabelKind, LibId, Line, LineKind, Mirror, PinInstance, PointItem,
    ReadError, Refdes, Schematic, SheetItem, SheetPath, SheetPin, Symbol, SymbolPlacement,
    TextItem, Uuid,
};
pub use project::{BusAlias, Project, ProjectError, read_project};
pub use version::{FormatVersion, MAX_SCHEMATIC_VERSION, PropertyOrder, pin_text};
pub use write::{WriteOptions, WritePlan, WriteRefusal, format_version, plan_write};
