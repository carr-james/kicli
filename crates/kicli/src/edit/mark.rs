//! Junctions and no-connects: the marks that decide what is joined.
//!
//! A junction makes a crossing a connection. Both marks are one point in the
//! file, both change what the netlist says, and both are refused when the
//! drawing already says something else. A junction where four wire ends
//! already meet is the refusal this module exists for: it draws as one dot
//! that four wires run into, and a reader cannot tell which pair the designer
//! meant to join.
//!
//! Every command here takes the loaded hierarchy, because a mark is only
//! meaningful against the project it sits in, and hands the change to
//! [`crate::model::mutate`], which is the only path to disk.

use std::fmt;
use std::path::Path;

use kicli_sexpr::{Doc, SexprError, quote};

use crate::geometry::{Iu, Point};
use crate::model::hierarchy::Hierarchy;
use crate::model::items::{Line, ReadError, Schematic, Uuid};
use crate::model::mutate::{Mutation, MutationError, Target, commit, state_before};
use crate::view::snapshot::{Snapshot, SnapshotError, millimetres};

/// How many wire ends at one point make a junction a four-way junction.
const FOUR_WAY: usize = 4;

/// One end of a wire or bus segment, named by the segment it belongs to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WireEnd {
    /// The short form of the segment's identifier, as a delta prints it.
    pub handle: String,
    /// Where the segment's other end is.
    pub far: Point,
}

impl fmt::Display for WireEnd {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "wire {} to ({})", self.handle, millimetres(self.far))
    }
}

/// Why a mark command did not happen.
///
/// Every variant is an operation error: the request is well formed and kicli
/// will not carry it out. No variant reaches the disk, so a refusal leaves the
/// file exactly as it was.
#[derive(Debug, thiserror::Error)]
pub enum MarkError {
    /// Four or more wire ends already meet at the point.
    #[error(
        "four wire ends meet at ({}): {}. A junction there is a four-way junction. \
         That is a defect, and KiCad's own rule check ignores it by default. \
         Move one wire end by one grid step. Then add the junction.",
        millimetres(*.at),
        listed(.ends)
    )]
    FourWayJunction {
        /// Where the wire ends meet.
        at: Point,
        /// The wire ends that meet there.
        ends: Vec<WireEnd>,
    },

    /// A junction is already drawn at the point.
    #[error("a junction is already drawn at ({}).", millimetres(*.at))]
    JunctionExists {
        /// Where the junction is.
        at: Point,
    },

    /// No junction is drawn at the point.
    #[error("no junction is drawn at ({}).", millimetres(*.at))]
    NoJunctionThere {
        /// Where kicli looked.
        at: Point,
    },

    /// The file to edit is not part of the loaded hierarchy.
    #[error("{path} is not one of the files of this project.")]
    UnknownFile {
        /// The file the caller asked for.
        path: String,
    },

    /// The file holds no outermost list, so nothing can be added to it.
    #[error("this file is empty, so kicli cannot add anything to it.")]
    Empty,

    /// A fragment kicli built did not parse.
    #[error(transparent)]
    Sexpr(#[from] SexprError),

    /// The edited file did not read back as a schematic.
    #[error(transparent)]
    Read(#[from] ReadError),

    /// The state to compare the change against could not be taken.
    #[error(transparent)]
    Snapshot(#[from] SnapshotError),

    /// The change did not survive its own checks, or could not be written.
    #[error(transparent)]
    Mutation(#[from] MutationError),
}

/// Add a junction at a point.
///
/// The point is snapped to `target.grid`, because a junction is connectable
/// geometry and off-grid connectable geometry is a blocking fault. `uuid` is
/// the identifier the new object carries; the caller supplies it, so that two
/// runs of one command over one design produce one file.
///
/// # Errors
///
/// Returns [`MarkError::FourWayJunction`] when four wire ends already meet at
/// the point, and [`MarkError::JunctionExists`] when a junction is drawn there
/// already. Neither writes anything.
pub fn add_junction(
    hierarchy: &mut Hierarchy,
    at: Point,
    uuid: &Uuid,
    target: &Target<'_>,
    taken: &str,
) -> Result<Mutation, MarkError> {
    let file = file_of(hierarchy, target.path)?;
    let at = snapped(at, target.grid);
    let schematic = &hierarchy.files[file].schematic;

    if schematic.junctions().any(|junction| junction.at == at) {
        return Err(MarkError::JunctionExists { at });
    }
    let ends = wire_ends_at(schematic, at);
    if ends.len() >= FOUR_WAY {
        return Err(MarkError::FourWayJunction { at, ends });
    }

    let fragment = format!(
        "(junction (at {} {}) (diameter 0) (color 0 0 0 0) (uuid {}))",
        at.x,
        at.y,
        quote(&uuid.0)
    );
    write_change(hierarchy, file, target, taken, |doc| {
        add_item(doc, &fragment)
    })
}

/// Delete the junction at a point.
///
/// The point is snapped to `target.grid`, so a caller addresses the junction
/// the same way it added one.
///
/// # Errors
///
/// Returns [`MarkError::NoJunctionThere`] when no junction is drawn at the
/// point.
pub fn delete_junction(
    hierarchy: &mut Hierarchy,
    at: Point,
    target: &Target<'_>,
    taken: &str,
) -> Result<Mutation, MarkError> {
    let file = file_of(hierarchy, target.path)?;
    let at = snapped(at, target.grid);
    let node = hierarchy.files[file]
        .schematic
        .junctions()
        .find(|junction| junction.at == at)
        .map(|junction| junction.node)
        .ok_or(MarkError::NoJunctionThere { at })?;

    write_change(hierarchy, file, target, taken, |doc| {
        doc.remove(node);
        Ok(())
    })
}

/// The wire and bus ends that meet at one point.
///
/// A segment's body is not an end. A pin or a wire end on another segment's
/// interior does not join it, so the interior is not part of this count.
fn wire_ends_at(schematic: &Schematic, at: Point) -> Vec<WireEnd> {
    let mut ends = Vec::new();
    for line in schematic.lines() {
        if line.from == at {
            ends.push(end_of(line, line.to));
        }
        if line.to == at {
            ends.push(end_of(line, line.from));
        }
    }
    ends
}

fn end_of(line: &Line, far: Point) -> WireEnd {
    WireEnd {
        handle: short(&line.uuid.0),
        far,
    }
}

/// Put a new object into a file, before its `sheet_instances` list.
///
/// KiCad reads the objects of a sheet in any order and sorts them on save. The
/// position chosen here keeps the trailing lists at the end, so the change
/// reads as one insertion rather than as a move of everything after it.
fn add_item(doc: &mut Doc, fragment: &str) -> Result<(), MarkError> {
    let root = doc.root().ok_or(MarkError::Empty)?;
    let node = doc.add_fragment(fragment)?;
    let before = doc
        .children(root)
        .iter()
        .position(|&child| doc.head_is(child, "sheet_instances"));
    match before {
        Some(index) => doc.insert_child(root, index, node),
        None => doc.push_child(root, node),
    }
    Ok(())
}

/// Change one file of a hierarchy, then check it, write it, and report it.
///
/// The state to compare against is taken before the change, and the file's
/// typed objects are read again after it, so a second command on the same
/// hierarchy sees what the first one did.
fn write_change(
    hierarchy: &mut Hierarchy,
    file: usize,
    target: &Target<'_>,
    taken: &str,
    change: impl FnOnce(&mut Doc) -> Result<(), MarkError>,
) -> Result<Mutation, MarkError> {
    let loaded = &mut hierarchy.files[file];
    let before: Snapshot = state_before(&loaded.doc, &loaded.schematic, target.sheet_path, taken)?;
    change(&mut loaded.doc)?;
    let mutation = commit(&loaded.doc, target, &before, taken)?;
    loaded.schematic = Schematic::read(&loaded.doc)?;
    Ok(mutation)
}

/// Which file of the hierarchy the target names.
fn file_of(hierarchy: &Hierarchy, path: &Path) -> Result<usize, MarkError> {
    let wanted = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_owned());
    hierarchy
        .files
        .iter()
        .position(|file| {
            std::fs::canonicalize(&file.path).unwrap_or_else(|_| file.path.clone()) == wanted
        })
        .ok_or_else(|| MarkError::UnknownFile {
            path: path.display().to_string(),
        })
}

/// The nearest grid point, rounding half away from zero.
fn snapped(point: Point, grid: Iu) -> Point {
    Point {
        x: snap(point.x, grid),
        y: snap(point.y, grid),
    }
}

/// One coordinate on the grid, by exact integer arithmetic.
fn snap(value: Iu, grid: Iu) -> Iu {
    if grid.0 <= 0 {
        return value;
    }
    let step = i64::from(grid.0);
    let raw = i64::from(value.0);
    let half = step / 2;
    let steps = if raw >= 0 {
        (raw + half) / step
    } else {
        (raw - half) / step
    };
    Iu(i32::try_from(steps * step).unwrap_or(value.0))
}

/// The first eight characters of an identifier, which is the handle a delta
/// prints.
fn short(uuid: &str) -> String {
    uuid.chars().take(8).collect()
}

/// A comma-separated list, for an error message.
fn listed<T: fmt::Display>(items: &[T]) -> String {
    items
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<String>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use super::{Iu, Point, listed, short, snap, snapped};
    use crate::geometry::GRID;

    #[test]
    fn a_point_snaps_to_the_nearest_grid_step() {
        assert_eq!(snap(Iu(0), GRID), Iu(0));
        assert_eq!(snap(Iu(12_700), GRID), Iu(12_700));
        // Half a step rounds away from zero, in both directions.
        assert_eq!(snap(Iu(6_350), GRID), Iu(12_700));
        assert_eq!(snap(Iu(-6_350), GRID), Iu(-12_700));
        assert_eq!(snap(Iu(6_349), GRID), Iu(0));
        assert_eq!(snap(Iu(-6_349), GRID), Iu(0));
        assert_eq!(
            snapped(Point::new(6_350, -6_350), GRID),
            Point::new(12_700, -12_700)
        );
    }

    #[test]
    fn a_grid_of_nothing_leaves_a_point_alone() {
        assert_eq!(snap(Iu(3), Iu(0)), Iu(3));
    }

    #[test]
    fn a_handle_is_the_first_eight_characters() {
        assert_eq!(short("0123456789abcdef"), "01234567");
        assert_eq!(short("short"), "short");
    }

    #[test]
    fn a_list_reads_as_prose() {
        assert_eq!(listed(&["one", "two"]), "one, two");
        assert_eq!(listed::<&str>(&[]), "");
    }
}
