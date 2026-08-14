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
use std::fmt::Write as _;

use kicli_sexpr::{Doc, NodeId, SexprError, quote};

use crate::geometry::{Angle, Iu, Point, Transform};
use crate::model::items::{LibId, Mirror, Refdes, Schematic, SheetPath, Symbol, Uuid};
use crate::model::library::read_library;

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

/// The header lists a schematic writes before its objects.
///
/// A new `lib_symbols` goes after the last of them, which is where KiCad puts
/// it.
const HEADER_TOKENS: [&str; 6] = [
    "version",
    "generator",
    "generator_version",
    "uuid",
    "paper",
    "title_block",
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

    /// The library definition is not a `(symbol "name" ...)` block.
    #[error("the library definition is not a (symbol \"name\" ...) block")]
    NotADefinition,

    /// A placement carries no instance data, so it has no reference anywhere.
    #[error("a placement needs instance data for every sheet path its sheet is on")]
    NoInstances,

    /// The source of new identifiers ran dry.
    #[error("the placement needs more identifiers than the source has")]
    NoIdentifier,

    /// The document holds no outermost list.
    #[error("this document has no outermost list, so it is not a schematic")]
    NoRoot,
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

/// One placement of a symbol, on one sheet path.
///
/// A sheet placed twice needs one of these per placement, because each carries
/// its own reference designator.
#[derive(Clone, Debug)]
pub struct Instance {
    /// The project the sheet path belongs to.
    pub project: String,
    /// The sheet path this reference applies to.
    pub path: SheetPath,
    /// The reference designator on that path.
    pub reference: Refdes,
    /// The unit of a multi-unit part on that path.
    pub unit: u32,
}

/// What to place, and where.
#[derive(Clone, Debug)]
pub struct Placement<'a> {
    /// The library identifier the placement records, such as `Device:R`.
    pub lib_id: &'a LibId,
    /// The `(symbol "R" ...)` block, as the library file writes it.
    ///
    /// The block is copied into the sheet's `lib_symbols` under the whole
    /// library identifier. KiCad draws that copy, so a placement without it
    /// draws as a placeholder.
    pub definition: &'a str,
    /// Where the anchor goes, before the grid snap.
    pub at: Point,
    /// The rotation to write.
    pub angle: Angle,
    /// The mirror to write, applied after the rotation.
    pub mirror: Option<Mirror>,
    /// Which unit of a multi-unit part to draw.
    pub unit: u32,
    /// Which body style to draw: 1 is normal, 2 is the De Morgan alternative.
    pub body_style: u32,
    /// The value to write, when it is not the library's own.
    pub value: Option<&'a str>,
    /// One entry per sheet path the sheet is placed on. It must not be empty.
    pub instances: &'a [Instance],
}

/// The grid helpers this module used to define.
///
/// They are geometry rather than symbol editing, and the extractor and the
/// label and marker editors ask the same questions, so they live in
/// [`crate::geometry::grid`]. They are re-exported here because the path was
/// public before they moved.
pub use crate::geometry::{snap, snap_point};

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
/// Only the symbol's own lines change: its `(at ...)`, its `(mirror ...)`, its
/// fields' `(at ...)`, and the `fields_autoplaced` flag when the fields turned.
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
/// Only the symbol's own lines change: its `(at ...)`, its `(mirror ...)`, its
/// fields' `(at ...)`, and the `fields_autoplaced` flag when the fields moved.
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

/// Delete a symbol, its instance data and its unused definition.
///
/// The instance data sits inside the symbol, so it goes with it. The embedded
/// definition stays when another placement still draws through it, and goes
/// when none does.
///
/// Only the symbol's own lines change, and the definition's when it goes too.
/// No other object of the file is touched.
///
/// # Errors
///
/// Returns [`EditError`] when the document holds no outermost list.
pub fn delete_symbol(
    doc: &mut Doc,
    schematic: &Schematic,
    symbol: &Symbol,
) -> Result<Edited, EditError> {
    let key = definition_key(symbol).to_owned();
    doc.remove(symbol.node);

    let still_drawn = schematic
        .symbols()
        .any(|other| other.node != symbol.node && definition_key(other) == key);
    if !still_drawn
        && let Some((_, entry)) = schematic
            .library_symbols
            .iter()
            .find(|(name, _)| *name == key)
    {
        doc.remove(*entry);
    }
    Ok(Edited {
        symbol: symbol.uuid.clone(),
        findings: Vec::new(),
    })
}

/// Place a symbol from a library, with its definition and its instance data.
///
/// The definition is copied into the sheet's `lib_symbols` under the whole
/// library identifier, because that copy is what KiCad draws. The placement
/// gets one `path` entry per [`Instance`], so a sheet placed twice gets two
/// reference designators.
///
/// `identifiers` supplies the new symbol's identifier and one per pin. A test
/// passes a counter, so a placement is reproducible.
///
/// # Errors
///
/// Returns [`EditError`] when the request carries no instance data, when the
/// definition is not a symbol block, when the identifiers run out, or when
/// kicli cannot read back the text it built.
pub fn place_symbol(
    doc: &mut Doc,
    schematic: &Schematic,
    request: &Placement<'_>,
    grid: Iu,
    options: Options,
    identifiers: &mut dyn Iterator<Item = Uuid>,
) -> Result<Edited, EditError> {
    let first = request.instances.first().ok_or(EditError::NoInstances)?;
    let (at, findings) = place_on_grid(request.at, grid, options);
    let uuid = identifiers.next().ok_or(EditError::NoIdentifier)?;

    embed_definition(doc, schematic, request)?;

    // A second copy of the definition, kept out of the tree. Its properties
    // become the placement's fields and its pins name the placement's pins.
    let template = doc.add_fragment(request.definition)?;
    if !doc.head_is(template, "symbol") {
        return Err(EditError::NotADefinition);
    }
    let definition = read_library(
        doc,
        &[(request.lib_id.0.clone(), template)],
        schematic.version,
    )
    .into_iter()
    .next()
    .ok_or(EditError::NotADefinition)?;

    let node = doc.add_fragment(&placement_text(doc, template, request, at, &uuid))?;
    take_fields(doc, template, node, request, at, &first.reference)?;

    for pin in definition.pins_for(request.unit, request.body_style) {
        let pin_uuid = identifiers.next().ok_or(EditError::NoIdentifier)?;
        let fragment = doc.add_fragment(&format!(
            "(pin {} (uuid {}))",
            quote(&pin.number),
            quote(&pin_uuid.0)
        ))?;
        doc.push_child(node, fragment);
    }

    let instances = doc.add_fragment(&instances_text(request.instances))?;
    doc.push_child(node, instances);

    let root = doc.root().ok_or(EditError::NoRoot)?;
    let index = doc
        .children(root)
        .iter()
        .position(|&child| {
            doc.head_is(child, "sheet_instances") || doc.head_is(child, "embedded_fonts")
        })
        .unwrap_or_else(|| doc.children(root).len());
    doc.insert_child(root, index, node);

    Ok(Edited {
        symbol: uuid,
        findings,
    })
}

/// Move the definition's properties onto the placement, as its fields.
///
/// A library property position is relative to the anchor and Y-up, exactly like
/// a library pin's, so it takes the same flip and the same orientation matrix.
fn take_fields(
    doc: &mut Doc,
    template: NodeId,
    node: NodeId,
    request: &Placement<'_>,
    at: Point,
    reference: &Refdes,
) -> Result<(), EditError> {
    let transform = Transform::from_file(request.angle, request.mirror);
    for property in doc.children(template).to_vec() {
        if !doc.head_is(property, "property") {
            continue;
        }
        doc.remove(property);
        let name = doc
            .children(property)
            .get(1)
            .and_then(|&id| doc.atom_as_str(id))
            .unwrap_or_default();
        let template_at = position_of(doc, property).unwrap_or_default();
        let offset = transform.apply(Point {
            x: template_at.x,
            y: -template_at.y,
        });
        set_position(doc, property, at + offset)?;
        if name == "Reference" {
            set_value(doc, property, &reference.0);
        } else if name == "Value"
            && let Some(value) = request.value
        {
            set_value(doc, property, value);
        }
        doc.push_child(node, property);
    }
    Ok(())
}

/// The cache key a placement draws through.
fn definition_key(symbol: &Symbol) -> &str {
    symbol.lib_name.as_deref().unwrap_or(&symbol.lib_id.0)
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

/// The position a list's `(at ...)` records.
fn position_of(doc: &Doc, owner: NodeId) -> Option<Point> {
    let at = at_of(doc, owner)?;
    let values = doc.children(at);
    Some(Point {
        x: Iu(doc.atom_as_iu(*values.get(1)?)?),
        y: Iu(doc.atom_as_iu(*values.get(2)?)?),
    })
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

/// Replace a property's value.
fn set_value(doc: &mut Doc, property: NodeId, value: &str) {
    let Some(&atom) = doc.children(property).get(2) else {
        return;
    };
    if doc.atom_text(atom).is_some() {
        doc.set_atom(atom, &quote(value));
    }
}

/// Something to call a list in an error message.
fn name_of(doc: &Doc, owner: NodeId) -> String {
    doc.head(owner).unwrap_or("this object").to_owned()
}

/// Copy the definition into the sheet's embedded library, once.
fn embed_definition(
    doc: &mut Doc,
    schematic: &Schematic,
    request: &Placement<'_>,
) -> Result<(), EditError> {
    if schematic
        .library_symbols
        .iter()
        .any(|(name, _)| *name == request.lib_id.0)
    {
        return Ok(());
    }
    let entry = doc.add_fragment(request.definition)?;
    if !doc.head_is(entry, "symbol") {
        return Err(EditError::NotADefinition);
    }
    // A library file keys the definition by the symbol name alone. A schematic
    // keys it by the whole library identifier.
    let &name = doc
        .children(entry)
        .get(1)
        .ok_or(EditError::NotADefinition)?;
    if doc.atom_text(name).is_none() {
        return Err(EditError::NotADefinition);
    }
    doc.set_atom(name, &quote(&request.lib_id.0));

    let cache = library_cache(doc)?;
    doc.push_child(cache, entry);
    Ok(())
}

/// The file's `lib_symbols`, made when the file has none.
fn library_cache(doc: &mut Doc) -> Result<NodeId, EditError> {
    let root = doc.root().ok_or(EditError::NoRoot)?;
    if let Some(&cache) = doc
        .children(root)
        .iter()
        .find(|&&child| doc.head_is(child, "lib_symbols"))
    {
        return Ok(cache);
    }
    let fresh = doc.add_fragment("(lib_symbols)")?;
    let index = doc
        .children(root)
        .iter()
        .rposition(|&child| {
            doc.head(child)
                .is_some_and(|head| HEADER_TOKENS.contains(&head))
        })
        .map_or(1, |position| position + 1);
    doc.insert_child(root, index, fresh);
    Ok(fresh)
}

/// The placement's own lists, up to and including its identifier.
///
/// The three yes-or-no flags are copied from the definition, so a file written
/// against an older format does not gain a token its version does not know.
fn placement_text(
    doc: &Doc,
    template: NodeId,
    request: &Placement<'_>,
    at: Point,
    uuid: &Uuid,
) -> String {
    let mut text = String::new();
    let _ = write!(text, "(symbol (lib_id {})", quote(&request.lib_id.0));
    let _ = write!(
        text,
        " (at {} {} {})",
        at.x,
        at.y,
        request.angle.0.rem_euclid(360)
    );
    if let Some(axis) = request.mirror {
        let _ = write!(text, " (mirror {})", axis_token(axis));
    }
    let _ = write!(text, " (unit {})", request.unit);
    let _ = write!(text, " (body_style {})", request.body_style);
    for flag in ["exclude_from_sim", "in_bom", "on_board", "in_pos_files"] {
        if let Some(value) = flag_of(doc, template, flag) {
            let _ = write!(text, " ({flag} {value})");
        }
    }
    let _ = write!(text, " (dnp no) (uuid {}))", quote(&uuid.0));
    text
}

/// The value of a yes-or-no list of a definition, when it carries one.
fn flag_of<'a>(doc: &'a Doc, node: NodeId, head: &str) -> Option<&'a str> {
    let list = doc
        .children(node)
        .iter()
        .copied()
        .find(|&child| doc.head_is(child, head))?;
    doc.atom_text(*doc.children(list).get(1)?)
}

/// The `(instances ...)` of a placement, one path per placement of its sheet.
fn instances_text(instances: &[Instance]) -> String {
    let mut projects: Vec<&str> = Vec::new();
    for instance in instances {
        if !projects.contains(&instance.project.as_str()) {
            projects.push(&instance.project);
        }
    }
    let mut text = String::from("(instances");
    for project in projects {
        let _ = write!(text, " (project {}", quote(project));
        for instance in instances
            .iter()
            .filter(|instance| instance.project == project)
        {
            let _ = write!(
                text,
                " (path {} (reference {}) (unit {}))",
                quote(&instance.path.0),
                quote(&instance.reference.0),
                instance.unit
            );
        }
        text.push(')');
    }
    text.push(')');
    text
}

#[cfg(test)]
mod tests {
    use super::{Instance, ORIENTATIONS, instances_text, inverse, snap, snap_point};
    use crate::geometry::{Angle, GRID, Iu, Point, Transform};
    use crate::model::items::{Mirror, Refdes, SheetPath};

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

    #[test]
    fn a_twice_placed_sheet_gets_two_paths_under_one_project() {
        let instances = [
            Instance {
                project: "board".to_owned(),
                path: SheetPath("/a/b".to_owned()),
                reference: Refdes("R1".to_owned()),
                unit: 1,
            },
            Instance {
                project: "board".to_owned(),
                path: SheetPath("/a/c".to_owned()),
                reference: Refdes("R2".to_owned()),
                unit: 1,
            },
        ];
        let text = instances_text(&instances);
        assert_eq!(text.matches("(project ").count(), 1);
        assert_eq!(text.matches("(path ").count(), 2);
        assert!(text.contains(r#"(reference "R1")"#), "{text}");
        assert!(text.contains(r#"(reference "R2")"#), "{text}");
    }
}
