//! The sheet tree of a project, loaded from its root file.
//!
//! One `.kicad_sch` is one drawing. A hierarchy is built by reference: a sheet
//! item names a child file, and the same file may be named twice. This module
//! walks that tree, gives every *placement* its own sheet path, and reads each
//! symbol's reference for the path it is on.
//!
//! Problems are data, not failures. A sheet naming a file that is not there, a
//! file that does not parse, and a cycle are all reported and the walk carries
//! on, because a health check must list every fault rather than stop at the
//! first one.

use crate::model::items::{Item, Schematic, SheetPath, Symbol, Uuid};
use kicli_sexpr::Doc;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// One schematic file, parsed once however many times it is placed.
pub struct LoadedFile {
    /// Where the file is on disk.
    pub path: PathBuf,
    /// The token tree, kept so an edit can reach the source.
    pub doc: Doc,
    /// The typed objects.
    pub schematic: Schematic,
}

/// One placement of a sheet in the hierarchy.
pub struct Placement {
    /// The sheet path of this placement, in KiCad's form.
    pub path: SheetPath,
    /// The sheet name, as drawn. The root sheet has none.
    pub name: Option<String>,
    /// The page number, as written.
    pub page: Option<String>,
    /// The file this placement draws.
    pub file: usize,
    /// The placement this one hangs from, if any.
    pub parent: Option<usize>,
}

/// Something wrong with the hierarchy, reported rather than raised.
#[derive(Debug, thiserror::Error)]
pub enum Problem {
    /// A sheet names a file that is not on disk.
    #[error("sheet {name} names {file}, which is not there")]
    MissingFile {
        /// The sheet name, as drawn.
        name: String,
        /// The file the sheet names.
        file: String,
        /// The sheet path of the placement that names it.
        path: SheetPath,
    },
    /// A sheet's file could not be read or parsed.
    #[error("sheet {name} names {file}, which does not read: {reason}")]
    Unreadable {
        /// The sheet name, as drawn.
        name: String,
        /// The file the sheet names.
        file: String,
        /// Why it did not read.
        reason: String,
        /// The sheet path of the placement that names it.
        path: SheetPath,
    },
    /// A sheet names a file already open above it in the tree.
    #[error("sheet {name} names {file}, which is already open above it")]
    Cycle {
        /// The sheet name, as drawn.
        name: String,
        /// The file the sheet names.
        file: String,
        /// The sheet path of the placement that names it.
        path: SheetPath,
    },
    /// A sheet item carries no `Sheetfile` field.
    #[error("a sheet at {path} names no file")]
    NoFile {
        /// The sheet path of the placement.
        path: SheetPath,
    },
}

/// Why a hierarchy could not be loaded at all.
#[derive(Debug, thiserror::Error)]
pub enum LoadError {
    /// The root file could not be read.
    #[error("cannot read {path}: {reason}")]
    Unreadable {
        /// The file that did not read.
        path: PathBuf,
        /// Why it did not read.
        reason: String,
    },
    /// The root file is not a schematic, or does not parse.
    #[error("cannot read {path} as a schematic: {reason}")]
    NotASchematic {
        /// The file that did not read.
        path: PathBuf,
        /// Why it did not read.
        reason: String,
    },
    /// The root file carries no uuid, so no sheet path can be built.
    #[error("{path} has no uuid, so its sheet paths cannot be built")]
    NoRootUuid {
        /// The file with no uuid.
        path: PathBuf,
    },
}

/// A project's sheet tree.
pub struct Hierarchy {
    /// Every file the tree reaches, each parsed once.
    pub files: Vec<LoadedFile>,
    /// Every placement, root first, then depth first in file order.
    pub placements: Vec<Placement>,
    /// Everything wrong that did not stop the walk.
    pub problems: Vec<Problem>,
}

impl Hierarchy {
    /// Load the tree that hangs from `root`.
    ///
    /// # Errors
    ///
    /// Returns [`LoadError`] only when the root file itself cannot be used.
    /// Every other fault is a [`Problem`] in the loaded tree.
    pub fn load(root: &Path) -> Result<Self, LoadError> {
        let file = read_file(root).map_err(|reason| LoadError::Unreadable {
            path: root.to_owned(),
            reason,
        })?;
        let root_uuid = file
            .schematic
            .uuid
            .clone()
            .ok_or_else(|| LoadError::NoRootUuid {
                path: root.to_owned(),
            })?;

        let mut tree = Self {
            files: vec![file],
            placements: vec![Placement {
                path: SheetPath::root(&root_uuid),
                name: None,
                page: None,
                file: 0,
                parent: None,
            }],
            problems: Vec::new(),
        };
        let mut by_path: BTreeMap<PathBuf, usize> = BTreeMap::new();
        by_path.insert(canonical(root), 0);
        tree.walk(0, &mut by_path);
        Ok(tree)
    }

    /// Follow every sheet of one placement, depth first.
    fn walk(&mut self, placement: usize, by_path: &mut BTreeMap<PathBuf, usize>) {
        let directory = self.files[self.placements[placement].file]
            .path
            .parent()
            .unwrap_or(Path::new("."))
            .to_owned();

        for sheet in self.child_sheets(placement) {
            let path = self.placements[placement].path.child(&sheet.uuid);
            let Some(file) = sheet.file else {
                self.problems.push(Problem::NoFile { path });
                continue;
            };
            let on_disk = canonical(&directory.join(&file));
            let name = sheet.name.clone().unwrap_or_else(|| file.clone());

            let Some(index) = self.resolve(placement, &on_disk, name, file, &path, by_path) else {
                continue;
            };
            self.placements.push(Placement {
                path,
                name: sheet.name,
                page: sheet.page,
                file: index,
                parent: Some(placement),
            });
            let child = self.placements.len() - 1;
            self.walk(child, by_path);
        }
    }

    /// The sheet items of one placement's file, with its page numbers resolved.
    fn child_sheets(&self, placement: usize) -> Vec<SheetRef> {
        let parent = &self.placements[placement].path;
        self.files[self.placements[placement].file]
            .schematic
            .items
            .iter()
            .filter_map(|item| match item {
                Item::Sheet(sheet) => Some(SheetRef {
                    uuid: sheet.uuid.clone(),
                    name: sheet.name().map(str::to_owned),
                    file: sheet.file().map(str::to_owned),
                    page: page_of(sheet, parent),
                }),
                _ => None,
            })
            .collect()
    }

    /// Find or load the file a sheet names, recording why when it cannot.
    fn resolve(
        &mut self,
        placement: usize,
        on_disk: &Path,
        name: String,
        file: String,
        path: &SheetPath,
        by_path: &mut BTreeMap<PathBuf, usize>,
    ) -> Option<usize> {
        if self.is_ancestor(placement, on_disk) {
            self.problems.push(Problem::Cycle {
                name,
                file,
                path: path.clone(),
            });
            return None;
        }
        if let Some(&known) = by_path.get(on_disk) {
            return Some(known);
        }
        if !on_disk.exists() {
            self.problems.push(Problem::MissingFile {
                name,
                file,
                path: path.clone(),
            });
            return None;
        }
        match read_file(on_disk) {
            Ok(loaded) => {
                self.files.push(loaded);
                let index = self.files.len() - 1;
                by_path.insert(on_disk.to_owned(), index);
                Some(index)
            }
            Err(reason) => {
                self.problems.push(Problem::Unreadable {
                    name,
                    file,
                    reason,
                    path: path.clone(),
                });
                None
            }
        }
    }

    /// Is `file` already open at or above `placement`?
    fn is_ancestor(&self, placement: usize, file: &Path) -> bool {
        let mut current = Some(placement);
        while let Some(index) = current {
            if canonical(&self.files[self.placements[index].file].path) == file {
                return true;
            }
            current = self.placements[index].parent;
        }
        false
    }

    /// The reference of every symbol placement in the tree.
    #[must_use]
    ///
    /// A symbol on a sheet placed twice appears twice, with a different
    /// reference each time. The cached `Reference` field is never consulted.
    pub fn references(&self) -> Vec<(SheetPath, &Symbol, String)> {
        let mut found = Vec::new();
        for placement in &self.placements {
            for symbol in self.files[placement.file].schematic.symbols() {
                if let Some(reference) = symbol.reference_on(&placement.path) {
                    found.push((placement.path.clone(), symbol, reference.0.clone()));
                }
            }
        }
        found
    }

    /// The placements of one file, by its index.
    pub fn placements_of(&self, file: usize) -> impl Iterator<Item = &Placement> {
        self.placements
            .iter()
            .filter(move |placement| placement.file == file)
    }
}

/// One sheet item of a file, as the walk needs it.
struct SheetRef {
    uuid: Uuid,
    name: Option<String>,
    file: Option<String>,
    page: Option<String>,
}

/// The page number a sheet item records for one parent path.
///
/// A sheet item's page is filed under the path of its **parent**, without the
/// sheet's own uuid, unlike a symbol's reference.
fn page_of(sheet: &crate::model::items::SheetItem, parent: &SheetPath) -> Option<String> {
    sheet
        .pages
        .iter()
        .find(|page| &page.path == parent)
        .map(|page| page.page.clone())
}

/// Read and parse one schematic file.
fn read_file(path: &Path) -> Result<LoadedFile, String> {
    let source = std::fs::read_to_string(path).map_err(|error| error.to_string())?;
    let doc = Doc::parse(&source).map_err(|error| error.to_string())?;
    let schematic = Schematic::read(&doc).map_err(|error| error.to_string())?;
    Ok(LoadedFile {
        path: path.to_owned(),
        doc,
        schematic,
    })
}

/// A path with `.` and `..` resolved where possible, for identity comparison.
fn canonical(path: &Path) -> PathBuf {
    std::fs::canonicalize(path).unwrap_or_else(|_| path.to_owned())
}
