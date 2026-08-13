//! Field text: position, angle, justification and visibility.
//!
//! A field is text that belongs to another object. A symbol owns `Reference`,
//! `Value` and the rest; a sheet owns `Sheetname` and `Sheetfile`; a global
//! label owns `Intersheetrefs`; a netclass flag owns `Netclass` and
//! `Component Class`. This module changes any of them, whoever owns it.
//!
//! One field is different. A symbol's `Reference` property is the cached value
//! for whichever sheet was loaded last, and the truth is the `instances` entry
//! for one sheet path. A symbol on a sheet placed twice has two references.
//! Setting one moves the cache and the named path's entry together, and leaves
//! every other path alone.

use kicli_sexpr::{Doc, NodeId, SexprError, fmt_iu, quote};

use crate::geometry::{Angle, Point};
use crate::model::items::{Field, Item, ReadError, Schematic, Uuid};
use crate::model::mutate::{Mutation, MutationError, Target, commit, state_before};
use crate::model::version::PropertyOrder;
use crate::view::snapshot::SnapshotError;

/// The field whose truth lives in the instance data, not in the property.
const REFERENCE: &str = "Reference";

/// The words a `justify` list uses for alignment.
///
/// Any other word in the list belongs to something else and is kept.
const ALIGNMENT: [&str; 4] = ["left", "right", "top", "bottom"];

/// Which field, and on which object.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FieldAddress {
    /// The identifier of the object that owns the field.
    pub owner: Uuid,
    /// The field name, such as `Reference` or `Sheetname`.
    pub name: String,
}

/// Where a field sits in the token tree.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Located {
    /// The list of the object that owns the field.
    pub owner: NodeId,
    /// The `property` list of the field itself.
    pub property: NodeId,
}

/// Why a field command did not happen.
#[derive(Debug, thiserror::Error)]
pub enum FieldError {
    /// The file holds no object with that identifier.
    #[error("this sheet holds no object with the identifier {0}")]
    NoSuchOwner(String),

    /// The object holds no field of that name.
    #[error("{owner} has no field named {name}")]
    NoSuchField {
        /// The identifier of the object that was asked.
        owner: String,
        /// The field name that was asked for.
        name: String,
    },

    /// The symbol is not placed on the sheet path the caller named.
    #[error("{owner} is not placed on the sheet path {path}, so it has no reference there")]
    NoSuchPlacement {
        /// The identifier of the symbol.
        owner: String,
        /// The sheet path the caller named.
        path: String,
    },

    /// The property does not hold a name and a value.
    #[error("the field {0} is not a complete property, so kicli will not change it")]
    Malformed(String),

    /// The file could not be read as a schematic.
    #[error(transparent)]
    Read(#[from] ReadError),

    /// The state to compare against could not be taken.
    #[error(transparent)]
    Snapshot(#[from] SnapshotError),

    /// The change was made but could not be written.
    #[error(transparent)]
    Write(#[from] MutationError),

    /// kicli built a token that does not parse. This is a kicli fault.
    #[error("kicli built a token it cannot read back: {0}")]
    Fragment(#[from] SexprError),
}

/// Find a field in the tree.
///
/// # Errors
///
/// Returns [`FieldError::NoSuchOwner`] when the sheet holds no object with that
/// identifier, and [`FieldError::NoSuchField`] when the object owns no field of
/// that name.
pub fn locate(schematic: &Schematic, address: &FieldAddress) -> Result<Located, FieldError> {
    let item = schematic
        .items
        .iter()
        .find(|item| item.uuid() == Some(&address.owner))
        .ok_or_else(|| FieldError::NoSuchOwner(address.owner.0.clone()))?;

    let property = fields_of(item)
        .iter()
        .find(|field| field.name == address.name)
        .map(|field| field.node)
        .ok_or_else(|| FieldError::NoSuchField {
            owner: address.owner.0.clone(),
            name: address.name.clone(),
        })?;

    Ok(Located {
        owner: item.node(),
        property,
    })
}

/// Set the text of a field, on any object that owns fields.
///
/// A `Reference` on a symbol also moves the `instances` entry for
/// `target.sheet_path`. No other sheet path is touched, so the other placements
/// of a twice-placed sheet keep the references they had.
///
/// `taken` is the timestamp to record, supplied by the caller so that a run is
/// repeatable.
///
/// # Errors
///
/// Returns [`FieldError`] when the field is not there, when the symbol is not
/// placed on the named sheet path, or when the write does not happen. Nothing
/// is written unless every invariant holds.
pub fn set_value(
    doc: &mut Doc,
    target: &Target<'_>,
    address: &FieldAddress,
    value: &str,
    taken: &str,
) -> Result<Mutation, FieldError> {
    change(doc, target, address, taken, |doc, located| {
        set_property_value(doc, located.property, &address.name, value)?;
        if address.name == REFERENCE && doc.head_is(located.owner, "symbol") {
            set_instance_reference(doc, located.owner, target, value)?;
        }
        Ok(())
    })
}

/// Put a field at a position.
///
/// A field's `(at ...)` is absolute schematic space, so the position is a
/// direct edit and needs no transform. Fields are exempt from the grid rule:
/// KiCad's own autoplacement lands them on arbitrary units, so snapping them
/// would fight the editor.
///
/// # Errors
///
/// Returns [`FieldError`] when the field is not there, or when the write does
/// not happen.
pub fn move_to(
    doc: &mut Doc,
    target: &Target<'_>,
    address: &FieldAddress,
    to: Point,
    taken: &str,
) -> Result<Mutation, FieldError> {
    change(doc, target, address, taken, |doc, located| {
        set_at(doc, located.property, Some(to), None)?;
        clear_autoplace(doc, located.owner);
        Ok(())
    })
}

/// Turn a field to an angle.
///
/// The angle is the text's own, and stays what it is when the owner turns.
///
/// # Errors
///
/// Returns [`FieldError`] when the field is not there, or when the write does
/// not happen.
pub fn rotate_to(
    doc: &mut Doc,
    target: &Target<'_>,
    address: &FieldAddress,
    angle: Angle,
    taken: &str,
) -> Result<Mutation, FieldError> {
    change(doc, target, address, taken, |doc, located| {
        set_at(doc, located.property, None, Some(angle))?;
        clear_autoplace(doc, located.owner);
        Ok(())
    })
}

/// Set which part of a field's text sits at its position.
///
/// # Errors
///
/// Returns [`FieldError`] when the field is not there, or when the write does
/// not happen.
pub fn justify(
    doc: &mut Doc,
    target: &Target<'_>,
    address: &FieldAddress,
    justification: Justification,
    taken: &str,
) -> Result<Mutation, FieldError> {
    change(doc, target, address, taken, |doc, located| {
        set_justification(doc, located.property, justification)?;
        clear_autoplace(doc, located.owner);
        Ok(())
    })
}

/// Where a field's text sits, left to right, about its position.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Horizontal {
    /// The text starts at the position.
    Left,
    /// The text is centred on the position.
    #[default]
    Center,
    /// The text ends at the position.
    Right,
}

impl Horizontal {
    /// Every value, in the order left to right.
    pub const ALL: [Self; 3] = [Self::Left, Self::Center, Self::Right];

    /// The token KiCad writes, when it writes one.
    const fn token(self) -> Option<&'static str> {
        match self {
            Self::Left => Some("left"),
            Self::Center => None,
            Self::Right => Some("right"),
        }
    }
}

/// Where a field's text sits, top to bottom, about its position.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Vertical {
    /// The text hangs below the position.
    Top,
    /// The text is centred on the position.
    #[default]
    Center,
    /// The text stands above the position.
    Bottom,
}

impl Vertical {
    /// Every value, in the order top to bottom.
    pub const ALL: [Self; 3] = [Self::Top, Self::Center, Self::Bottom];

    /// The token KiCad writes, when it writes one.
    const fn token(self) -> Option<&'static str> {
        match self {
            Self::Top => Some("top"),
            Self::Center => None,
            Self::Bottom => Some("bottom"),
        }
    }
}

/// Which part of a field's text sits at its position.
///
/// KiCad writes a token only for an edge that is not centred, and no `justify`
/// list at all when both are. Every one of KiCad's own demo files follows that
/// rule, and none of them writes the word `center`.
///
/// # Examples
///
/// ```
/// use kicli::edit::field::{Horizontal, Justification, Vertical};
/// use kicli_sexpr::Doc;
///
/// let source = "(property \"Value\" \"10k\"\n\t(effects\n\t\t(justify left bottom)\n\t)\n)\n";
/// let doc = Doc::parse(source).expect("parses");
/// let property = doc.root().expect("has a root");
/// assert_eq!(
///     Justification::read(&doc, property),
///     Justification { horizontal: Horizontal::Left, vertical: Vertical::Bottom },
/// );
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Justification {
    /// Where the text sits from left to right.
    pub horizontal: Horizontal,
    /// Where the text sits from top to bottom.
    pub vertical: Vertical,
}

impl Justification {
    /// Read the justification of a `property` list.
    ///
    /// A missing token means centred, which is what KiCad means by leaving it
    /// out.
    #[must_use]
    pub fn read(doc: &Doc, property: NodeId) -> Self {
        let mut found = Self::default();
        let Some(effects) = child_of(doc, property, "effects") else {
            return found;
        };
        let Some(list) = child_of(doc, effects, "justify") else {
            return found;
        };
        for &atom in doc.children(list).iter().skip(1) {
            match doc.atom_text(atom) {
                Some("left") => found.horizontal = Horizontal::Left,
                Some("right") => found.horizontal = Horizontal::Right,
                Some("top") => found.vertical = Vertical::Top,
                Some("bottom") => found.vertical = Vertical::Bottom,
                _ => {}
            }
        }
        found
    }

    /// The tokens KiCad writes for this pair.
    fn tokens(self) -> Vec<&'static str> {
        [self.horizontal.token(), self.vertical.token()]
            .into_iter()
            .flatten()
            .collect()
    }
}

/// Read the file, apply one change to one field, then write and report it.
///
/// Every command of this module goes through here, so every one of them runs
/// the invariants and leaves the `@last-write` state behind.
fn change<F>(
    doc: &mut Doc,
    target: &Target<'_>,
    address: &FieldAddress,
    taken: &str,
    apply: F,
) -> Result<Mutation, FieldError>
where
    F: FnOnce(&mut Doc, Located) -> Result<(), FieldError>,
{
    let schematic = Schematic::read(doc)?;
    let located = locate(&schematic, address)?;
    let before = state_before(doc, &schematic, target.sheet_path, taken)?;
    apply(doc, located)?;
    Ok(commit(doc, target, &before, taken)?)
}

/// The fields of an object, or nothing when it owns none.
fn fields_of(item: &Item) -> &[Field] {
    match item {
        Item::Symbol(symbol) => &symbol.fields,
        Item::Label(label) => &label.fields,
        Item::Sheet(sheet) => &sheet.fields,
        _ => &[],
    }
}

/// Put new text in a property's value, quoted the way KiCad quotes it.
fn set_property_value(
    doc: &mut Doc,
    property: NodeId,
    name: &str,
    value: &str,
) -> Result<(), FieldError> {
    let atom = doc
        .children(property)
        .get(2)
        .copied()
        .ok_or_else(|| FieldError::Malformed(name.to_owned()))?;
    doc.set_atom(atom, &quote(value));
    Ok(())
}

/// Put a reference in the `instances` entry for one sheet path.
///
/// Only an entry whose path is the one the caller named changes. A symbol on a
/// sheet placed twice keeps its other reference, which is the case most tools
/// get wrong.
fn set_instance_reference(
    doc: &mut Doc,
    symbol: NodeId,
    target: &Target<'_>,
    value: &str,
) -> Result<(), FieldError> {
    let mut entries = Vec::new();
    for &instances in doc.children(symbol) {
        if !doc.head_is(instances, "instances") {
            continue;
        }
        for &project in doc.children(instances) {
            if !doc.head_is(project, "project") {
                continue;
            }
            for &path in doc.children(project) {
                if doc.head_is(path, "path")
                    && first_atom(doc, path).as_deref() == Some(target.sheet_path.0.as_str())
                {
                    entries.push(path);
                }
            }
        }
    }

    if entries.is_empty() {
        return Err(FieldError::NoSuchPlacement {
            owner: child_text(doc, symbol, "uuid").unwrap_or_default(),
            path: target.sheet_path.0.clone(),
        });
    }

    for path in entries {
        if let Some(reference) = child_of(doc, path, "reference") {
            let atom = doc
                .children(reference)
                .get(1)
                .copied()
                .ok_or_else(|| FieldError::Malformed(REFERENCE.to_owned()))?;
            doc.set_atom(atom, &quote(value));
        } else {
            // A path entry without a reference is instance data KiCad would
            // fill in on its next save. kicli fills it in now, so the file
            // says what it means.
            let fragment = doc.add_fragment(&format!("(reference {})", quote(value)))?;
            doc.insert_child(path, 2, fragment);
        }
    }
    Ok(())
}

/// Set a field's position, its angle, or both.
///
/// The whole `(at x y angle)` list is written again from what it held, so a
/// field that carried no angle gets the `0` KiCad would write for it.
fn set_at(
    doc: &mut Doc,
    property: NodeId,
    position: Option<Point>,
    angle: Option<Angle>,
) -> Result<(), FieldError> {
    let current = child_of(doc, property, "at");
    let mut at = current.map_or_else(Point::default, |list| point_of(doc, list));
    let mut turn = current.map_or_else(Angle::default, |list| angle_of(doc, list));
    if let Some(wanted) = position {
        at = wanted;
    }
    if let Some(wanted) = angle {
        turn = wanted;
    }

    let fragment = doc.add_fragment(&format!(
        "(at {} {} {})",
        fmt_iu(at.x.0),
        fmt_iu(at.y.0),
        turn.0
    ))?;
    replace_child(doc, property, "at", current, fragment);
    Ok(())
}

/// Set which part of a field's text sits at its position.
///
/// A token that is not an alignment word is kept, because it belongs to
/// something this command was not asked about.
fn set_justification(
    doc: &mut Doc,
    property: NodeId,
    justification: Justification,
) -> Result<(), FieldError> {
    let effects = effects_of(doc, property)?;
    let current = child_of(doc, effects, "justify");
    let kept: Vec<String> = current
        .map(|list| {
            doc.children(list)
                .iter()
                .skip(1)
                .filter_map(|&atom| doc.atom_text(atom))
                .filter(|token| !ALIGNMENT.contains(token))
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default();

    let mut tokens: Vec<&str> = justification.tokens();
    tokens.extend(kept.iter().map(String::as_str));
    if tokens.is_empty() {
        // KiCad writes no list at all when the text is centred both ways.
        if let Some(list) = current {
            doc.remove(list);
        }
        return Ok(());
    }

    let fragment = doc.add_fragment(&format!("(justify {})", tokens.join(" ")))?;
    replace_child(doc, effects, "justify", current, fragment);
    Ok(())
}

/// The `effects` list of a property, made if the property has none.
fn effects_of(doc: &mut Doc, property: NodeId) -> Result<NodeId, FieldError> {
    if let Some(effects) = child_of(doc, property, "effects") {
        return Ok(effects);
    }
    let fragment = doc.add_fragment("(effects)")?;
    insert_in_order(doc, property, "effects", fragment);
    Ok(fragment)
}

/// Put a list where the one it replaces was, or where its token belongs.
fn replace_child(
    doc: &mut Doc,
    parent: NodeId,
    token: &str,
    current: Option<NodeId>,
    fragment: NodeId,
) {
    let Some(list) = current else {
        insert_in_order(doc, parent, token, fragment);
        return;
    };
    let index = doc.children(parent).iter().position(|&child| child == list);
    doc.remove(list);
    match index {
        Some(index) => doc.insert_child(parent, index, fragment),
        None => doc.push_child(parent, fragment),
    }
}

/// Put a list among a parent's children, in the order KiCad writes them.
///
/// A property on a placed object writes `at`, `hide`, `show_name`,
/// `do_not_autoplace`, `effects`. An `effects` list writes `font`, `justify`,
/// and then `hide` in a file old enough to keep `hide` there.
fn insert_in_order(doc: &mut Doc, parent: NodeId, token: &str, child: NodeId) {
    let order: &[&str] = if doc.head_is(parent, "effects") {
        &["font", "justify", "hide"]
    } else {
        PropertyOrder::Instance.tokens()
    };
    let rank = |token: &str| order.iter().position(|known| *known == token);

    let Some(mine) = rank(token) else {
        doc.push_child(parent, child);
        return;
    };
    let before = doc
        .children(parent)
        .iter()
        .position(|&sibling| doc.head(sibling).and_then(rank).is_some_and(|at| at > mine));
    match before {
        Some(index) => doc.insert_child(parent, index, child),
        None => doc.push_child(parent, child),
    }
}

/// Take the autoplace flag off the object that owns a field.
///
/// KiCad places the fields of an object that carries the flag again on its next
/// open. Leaving it would discard the position kicli just set.
fn clear_autoplace(doc: &mut Doc, owner: NodeId) -> bool {
    match child_of(doc, owner, "fields_autoplaced") {
        Some(flag) => doc.remove(flag),
        None => false,
    }
}

/// The first two numbers of an `(at ...)` list.
fn point_of(doc: &Doc, list: NodeId) -> Point {
    let values = doc.children(list);
    let number = |index: usize| {
        values
            .get(index)
            .and_then(|&atom| doc.atom_as_iu(atom))
            .unwrap_or_default()
    };
    Point::new(number(1), number(2))
}

/// The third number of an `(at ...)` list, which is the angle.
fn angle_of(doc: &Doc, list: NodeId) -> Angle {
    doc.children(list)
        .get(3)
        .and_then(|&atom| doc.atom_text(atom))
        .and_then(Angle::from_text)
        .unwrap_or_default()
}

/// The first child list with this head token.
fn child_of(doc: &Doc, node: NodeId, head: &str) -> Option<NodeId> {
    doc.children(node)
        .iter()
        .copied()
        .find(|&child| doc.head_is(child, head))
}

/// The unquoted text of the first value of a named child list.
fn child_text(doc: &Doc, node: NodeId, head: &str) -> Option<String> {
    let child = child_of(doc, node, head)?;
    doc.children(child)
        .get(1)
        .and_then(|&atom| doc.atom_as_str(atom))
}

/// The unquoted text of a list's first value.
fn first_atom(doc: &Doc, node: NodeId) -> Option<String> {
    doc.children(node)
        .get(1)
        .and_then(|&atom| doc.atom_as_str(atom))
}
