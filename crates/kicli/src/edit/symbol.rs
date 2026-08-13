//! Placed symbols: placing, moving, turning, mirroring and deleting.
//!
//! A command here changes the tree and says what it noticed. It writes no
//! bytes; [`crate::model::mutate::commit`] does that, after the invariants run.
//!
//! Three rules govern every command in this module.
//!
//! A symbol anchor carries connectable geometry, so it snaps to the grid. The
//! snap is exact integer arithmetic, and a half step rounds away from zero.
//! [`Options::off_grid`] overrides the snap and raises a finding of its own, so
//! a caller feels the exception rather than discovering it later.
//!
//! Fields move rigidly with their symbol. A turn or a mirror carries each field
//! position about the anchor by the same change the symbol itself took, and
//! leaves the field's own angle alone. [`Options::keep_field_positions`] opts
//! out.
//!
//! A command that sets a field position removes `fields_autoplaced`. KiCad
//! places the fields of a symbol that carries the flag again on its next open,
//! which would undo the work without saying so.

use std::fmt;

use kicli_sexpr::{Doc, NodeId, SexprError};

use crate::geometry::{Angle, Iu, Point, Transform};
use crate::model::items::{Mirror, Symbol, Uuid};

/// The eight orientations a placed symbol can take, as the file writes them.
///
/// The list is the search space for the orientation that undoes another.
const ORIENTATIONS: [(i32, Option<Mirror>); 8] = [
    (0, None),
    (90, None),
    (180, None),
    (270, None),
    (0, Some(Mirror::X)),
    (0, Some(Mirror::Y)),
    (90, Some(Mirror::X)),
    (90, Some(Mirror::Y)),
];

/// Why a symbol command did not happen.
#[derive(Debug, thiserror::Error)]
pub enum EditError {
    /// The list carries no `(at ...)`, so there is nothing to change.
    #[error("{0} carries no position, so kicli cannot move it")]
    NoPosition(String),

    /// The angle is not one of the four a schematic symbol can take.
    #[error("{0} is not a schematic angle: use 0, 90, 180 or 270")]
    NotARightAngle(i32),

    /// The text kicli built is not one well-formed s-expression.
    ///
    /// This is a kicli fault, not a caller's.
    #[error("kicli built s-expression text it cannot read back: {0}")]
    Fragment(#[from] SexprError),
}

/// How much freedom a symbol command has.
///
/// Both settings are exceptions to a rule, so both default to off.
#[derive(Clone, Copy, Debug, Default)]
pub struct Options {
    /// Place the anchor exactly where it was asked for, off the grid.
    ///
    /// The exception applies to a move and to a placement. A turn and a mirror
    /// leave the anchor where it is.
    pub off_grid: bool,
    /// Leave the fields where they are instead of carrying them.
    pub keep_field_positions: bool,
}

/// Where a move puts the anchor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Motion {
    /// Add an offset to the anchor it has now.
    By(Point),
    /// Set the anchor to a position.
    To(Point),
}

/// What a symbol command noticed about the grid.
///
/// A finding is not a failure. It is the part of the result a caller must feel,
/// because the command did something other than the plain reading of the
/// request.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Finding {
    /// The position asked for was off the grid, so kicli moved it to the grid.
    Snapped {
        /// The position the caller asked for.
        asked: Point,
        /// The position kicli used.
        placed: Point,
    },
    /// The caller overrode the grid rule, and the anchor is off the grid.
    OffGrid {
        /// The position kicli used.
        placed: Point,
    },
}

impl Finding {
    /// The name a report writes for this finding.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Finding::Snapped { .. } => "snapped-to-grid",
            Finding::OffGrid { .. } => "off-grid",
        }
    }
}

impl fmt::Display for Finding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Finding::Snapped { asked, placed } => {
                write!(f, "{asked} is off the grid, so kicli used {placed}")
            }
            Finding::OffGrid { placed } => {
                write!(f, "{placed} is off the grid, which the caller asked for")
            }
        }
    }
}

/// What a symbol command did.
#[derive(Clone, Debug)]
pub struct Edited {
    /// The symbol the command addressed.
    pub symbol: Uuid,
    /// What the command noticed while it worked.
    pub findings: Vec<Finding>,
}

/// Round a coordinate to the nearest grid line, halves away from zero.
///
/// The arithmetic is exact integer arithmetic. A grid of zero is no grid, and
/// the value comes back unchanged.
///
/// # Examples
///
/// ```
/// use kicli::edit::symbol::snap;
/// use kicli::geometry::{GRID, Iu};
///
/// assert_eq!(snap(Iu(6_350), GRID), Iu(12_700), "a half step rounds away from zero");
/// assert_eq!(snap(Iu(-6_350), GRID), Iu(-12_700));
/// assert_eq!(snap(Iu(6_349), GRID), Iu(0));
/// ```
#[must_use]
pub fn snap(value: Iu, grid: Iu) -> Iu {
    if grid.0 == 0 {
        return value;
    }
    let step = i64::from(grid.0).abs();
    let size = i64::from(value.0).abs();
    let steps = (size + step / 2) / step;
    let rounded = steps * step * i64::from(value.0.signum());
    Iu(i32::try_from(rounded).unwrap_or(value.0))
}

/// Round both coordinates of a point to the nearest grid line.
///
/// # Examples
///
/// ```
/// use kicli::edit::symbol::snap_point;
/// use kicli::geometry::{GRID, Point};
///
/// assert_eq!(snap_point(Point::new(6_350, 0), GRID), Point::new(12_700, 0));
/// ```
#[must_use]
pub fn snap_point(point: Point, grid: Iu) -> Point {
    Point {
        x: snap(point.x, grid),
        y: snap(point.y, grid),
    }
}

/// Move a symbol, and carry its fields with it.
///
/// The anchor snaps to the grid unless [`Options::off_grid`] says otherwise.
/// Only the symbol's own lines change: its `(at ...)`, its fields' `(at ...)`,
/// and the `fields_autoplaced` flag when the fields moved.
///
/// # Errors
///
/// Returns [`EditError`] when the symbol or one of its fields carries no
/// position, or when kicli cannot read back the text it built.
pub fn move_symbol(
    doc: &mut Doc,
    symbol: &Symbol,
    motion: Motion,
    grid: Iu,
    options: Options,
) -> Result<Edited, EditError> {
    let asked = match motion {
        Motion::By(offset) => symbol.at + offset,
        Motion::To(place) => place,
    };
    let (placed, findings) = place_on_grid(asked, grid, options);
    let offset = placed - symbol.at;

    set_position(doc, symbol.node, placed)?;
    if !options.keep_field_positions {
        for field in &symbol.fields {
            set_position(doc, field.node, field.at + offset)?;
        }
        clear_autoplace(doc, symbol.node);
    }
    Ok(Edited {
        symbol: symbol.uuid.clone(),
        findings,
    })
}

/// Turn a symbol to an absolute angle, keeping the mirror it has.
///
/// The anchor does not move, so no grid question arises. The fields keep their
/// own angles and their positions turn about the anchor.
///
/// # Errors
///
/// Returns [`EditError`] when the angle is not 0, 90, 180 or 270, when the
/// symbol carries no position, or when kicli cannot read back the text it
/// built.
pub fn rotate_symbol(
    doc: &mut Doc,
    symbol: &Symbol,
    to: Angle,
    options: Options,
) -> Result<Edited, EditError> {
    if !matches!(to.0.rem_euclid(360), 0 | 90 | 180 | 270) {
        return Err(EditError::NotARightAngle(to.0));
    }
    reorient(
        doc,
        symbol,
        Transform::from_file(to, symbol.mirror),
        options,
    )
}

/// Mirror a symbol about an axis through its own anchor.
///
/// `Mirror::X` reflects about the horizontal line through the anchor, which is
/// what the file's `(mirror x)` means. The written orientation is the
/// normalised one, so 180 degrees with a mirror comes out as 0 degrees with the
/// other mirror, exactly as KiCad's own editor writes it.
///
/// # Errors
///
/// Returns [`EditError`] when the symbol carries no position, or when kicli
/// cannot read back the text it built.
pub fn mirror_symbol(
    doc: &mut Doc,
    symbol: &Symbol,
    axis: Mirror,
    options: Options,
) -> Result<Edited, EditError> {
    let current = Transform::from_file(symbol.angle, symbol.mirror);
    let reflection = Transform::from_file(Angle(0), Some(axis));
    reorient(doc, symbol, current.compose(reflection), options)
}

/// Decide where the anchor lands, and say so when it is not where it was asked.
fn place_on_grid(asked: Point, grid: Iu, options: Options) -> (Point, Vec<Finding>) {
    let on_grid = grid.0 != 0 && asked.x.0 % grid.0 == 0 && asked.y.0 % grid.0 == 0;
    if options.off_grid {
        let findings = if on_grid {
            Vec::new()
        } else {
            vec![Finding::OffGrid { placed: asked }]
        };
        return (asked, findings);
    }
    let placed = snap_point(asked, grid);
    let findings = if placed == asked {
        Vec::new()
    } else {
        vec![Finding::Snapped { asked, placed }]
    };
    (placed, findings)
}

/// Write a new orientation, and carry the fields by the same change.
fn reorient(
    doc: &mut Doc,
    symbol: &Symbol,
    after: Transform,
    options: Options,
) -> Result<Edited, EditError> {
    let before = Transform::from_file(symbol.angle, symbol.mirror);
    let (angle, mirror) = after.to_file();
    set_position_and_angle(doc, symbol.node, symbol.at, angle)?;
    set_mirror(doc, symbol.node, mirror)?;

    if !options.keep_field_positions {
        // The fields move rigidly, so their offsets take exactly the change the
        // symbol's own orientation took.
        let change = inverse(before).compose(after);
        for field in &symbol.fields {
            let offset = change.apply(field.at - symbol.at);
            set_position(doc, field.node, symbol.at + offset)?;
        }
        clear_autoplace(doc, symbol.node);
    }
    Ok(Edited {
        symbol: symbol.uuid.clone(),
        findings: Vec::new(),
    })
}

/// The orientation that undoes another.
///
/// The eight orientations form a group, so the inverse is one of them and the
/// search is exact. `Transform` carries no inverse of its own.
fn inverse(transform: Transform) -> Transform {
    ORIENTATIONS
        .iter()
        .map(|&(angle, mirror)| Transform::from_file(Angle(angle), mirror))
        .find(|&candidate| transform.compose(candidate) == Transform::default())
        .unwrap_or_default()
}

/// The `(at ...)` of a list, when it has one.
fn at_of(doc: &Doc, owner: NodeId) -> Option<NodeId> {
    doc.children(owner)
        .iter()
        .copied()
        .find(|&child| doc.head_is(child, "at"))
}

/// Write a position, keeping any angle the list already carries.
fn set_position(doc: &mut Doc, owner: NodeId, point: Point) -> Result<(), EditError> {
    let at = at_of(doc, owner).ok_or_else(|| EditError::NoPosition(name_of(doc, owner)))?;
    let angle = doc
        .children(at)
        .get(3)
        .and_then(|&id| doc.atom_text(id))
        .map(str::to_owned);
    replace_at(doc, owner, at, point, angle.as_deref())
}

/// Write a position and an angle.
fn set_position_and_angle(
    doc: &mut Doc,
    owner: NodeId,
    point: Point,
    angle: Angle,
) -> Result<(), EditError> {
    let at = at_of(doc, owner).ok_or_else(|| EditError::NoPosition(name_of(doc, owner)))?;
    replace_at(
        doc,
        owner,
        at,
        point,
        Some(&angle.0.rem_euclid(360).to_string()),
    )
}

/// Put a fresh `(at ...)` where the old one was.
fn replace_at(
    doc: &mut Doc,
    owner: NodeId,
    at: NodeId,
    point: Point,
    angle: Option<&str>,
) -> Result<(), EditError> {
    let text = match angle {
        Some(angle) => format!("(at {} {} {angle})", point.x, point.y),
        None => format!("(at {} {})", point.x, point.y),
    };
    let fresh = doc.add_fragment(&text)?;
    let index = doc
        .children(owner)
        .iter()
        .position(|&child| child == at)
        .unwrap_or(0);
    doc.insert_child(owner, index, fresh);
    doc.remove(at);
    Ok(())
}

/// Write the mirror, or take it away.
///
/// KiCad writes `(mirror ...)` straight after the position, so a fresh one goes
/// there and the diff stays where the reader expects it.
fn set_mirror(doc: &mut Doc, owner: NodeId, mirror: Option<Mirror>) -> Result<(), EditError> {
    if let Some(existing) = doc
        .children(owner)
        .iter()
        .copied()
        .find(|&child| doc.head_is(child, "mirror"))
    {
        doc.remove(existing);
    }
    let Some(axis) = mirror else {
        return Ok(());
    };
    let fresh = doc.add_fragment(&format!("(mirror {})", axis_token(axis)))?;
    let index = doc
        .children(owner)
        .iter()
        .position(|&child| doc.head_is(child, "at"))
        .map_or(1, |position| position + 1);
    doc.insert_child(owner, index, fresh);
    Ok(())
}

/// The token a file writes for a mirror axis.
const fn axis_token(axis: Mirror) -> &'static str {
    match axis {
        Mirror::X => "x",
        Mirror::Y => "y",
    }
}

/// Take away the flag that lets KiCad place the fields again.
///
/// The flag is `(fields_autoplaced ...)` in current files and a bare token in
/// older ones. An absent flag reads as off in every version, so removing it is
/// the one form that means the same everywhere.
fn clear_autoplace(doc: &mut Doc, owner: NodeId) {
    for child in doc.children(owner).to_vec() {
        if doc.head_is(child, "fields_autoplaced")
            || doc.atom_text(child) == Some("fields_autoplaced")
        {
            doc.remove(child);
        }
    }
}

/// Something to call a list in an error message.
fn name_of(doc: &Doc, owner: NodeId) -> String {
    doc.head(owner).unwrap_or("this object").to_owned()
}

#[cfg(test)]
mod tests {
    use super::{ORIENTATIONS, inverse, snap, snap_point};
    use crate::geometry::{Angle, GRID, Iu, Point, Transform};
    use crate::model::items::Mirror;

    #[test]
    fn a_half_grid_step_rounds_away_from_zero() {
        assert_eq!(snap(Iu(6_350), GRID), Iu(12_700));
        assert_eq!(snap(Iu(-6_350), GRID), Iu(-12_700));
        assert_eq!(snap(Iu(6_349), GRID), Iu(0));
        assert_eq!(snap(Iu(-6_349), GRID), Iu(0));
        assert_eq!(snap(Iu(0), GRID), Iu(0));
        assert_eq!(snap(Iu(12_700), GRID), Iu(12_700), "a grid line stays put");
        assert_eq!(snap(Iu(1), Iu(0)), Iu(1), "no grid means no snap");
        assert_eq!(
            snap_point(Point::new(-6_350, 6_350), GRID),
            Point::new(-12_700, 12_700)
        );
    }

    #[test]
    fn every_orientation_has_an_inverse_in_the_group() {
        for (angle, mirror) in ORIENTATIONS {
            let transform = Transform::from_file(Angle(angle), mirror);
            assert_eq!(
                transform.compose(inverse(transform)),
                Transform::default(),
                "{angle} degrees, mirror {mirror:?}"
            );
        }
    }

    #[test]
    fn a_turn_of_a_mirrored_symbol_carries_its_fields_the_other_way() {
        // A mirror reverses the sense of a turn. A field offset therefore takes
        // the change of the whole orientation, not the change of the angle.
        let before = Transform::from_file(Angle(0), Some(Mirror::Y));
        let after = Transform::from_file(Angle(90), Some(Mirror::Y));
        let change = inverse(before).compose(after);
        let offset = Point::new(20_320, 0);
        assert_eq!(
            change.apply(offset),
            offset.rotated(Point::default(), Angle(270)),
            "the field turns against the angle the file records"
        );
    }
}
