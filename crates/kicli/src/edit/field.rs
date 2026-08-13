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

use kicli_sexpr::{Doc, NodeId, SexprError, quote};

use crate::model::items::{Field, Item, ReadError, Schematic, Uuid};
use crate::model::mutate::{Mutation, MutationError, Target, commit, state_before};
use crate::view::snapshot::SnapshotError;

/// The field whose truth lives in the instance data, not in the property.
const REFERENCE: &str = "Reference";

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
