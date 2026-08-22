//! One sheet placement, in the form a rule reads it.
//!
//! A rule is handed a drawing and nothing else. The drawing carries the token
//! tree, the typed objects, the embedded library and the sheet path. It carries
//! no file name and no way to reach the disk, so a rule cannot read or write
//! one.
//!
//! The caller builds a drawing from a file it has already loaded. That keeps
//! the loading in the caller and the geometry here, which is the direction the
//! dependencies run everywhere else in this crate.

use kicli_sexpr::Doc;

use crate::model::items::{Schematic, SheetPath, Symbol};
use crate::model::library::{LibrarySymbol, definition_of, read_library};

/// What a rule examines.
///
/// The lifetime is the caller's loaded file. Reading a drawing copies the
/// library cache and nothing else.
pub struct Drawing<'a> {
    path: &'a SheetPath,
    doc: &'a Doc,
    schematic: &'a Schematic,
    library: Vec<LibrarySymbol>,
}

impl<'a> Drawing<'a> {
    /// Read one placement of one already loaded schematic.
    ///
    /// The sheet path picks the placement, because a reference designator
    /// belongs to a placement rather than to a symbol.
    #[must_use]
    pub fn read(doc: &'a Doc, schematic: &'a Schematic, path: &'a SheetPath) -> Self {
        let library = read_library(doc, &schematic.library_symbols, schematic.version);
        Self {
            path,
            doc,
            schematic,
            library,
        }
    }

    /// The sheet path of this placement.
    #[must_use]
    pub fn path(&self) -> &SheetPath {
        self.path
    }

    /// The token tree the drawing was read from.
    #[must_use]
    pub fn doc(&self) -> &Doc {
        self.doc
    }

    /// The typed objects of the drawing.
    #[must_use]
    pub fn schematic(&self) -> &Schematic {
        self.schematic
    }

    /// The library the file embeds.
    #[must_use]
    pub fn library(&self) -> &[LibrarySymbol] {
        &self.library
    }

    /// The definition a placed symbol draws, when the file embeds one.
    #[must_use]
    pub fn definition_of(&self, symbol: &Symbol) -> Option<&LibrarySymbol> {
        definition_of(&self.library, symbol)
    }
}
