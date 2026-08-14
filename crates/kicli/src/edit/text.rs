//! Free text and text boxes: adding, moving, editing, resizing and deleting.
//!
//! Graphic text is a drawing and not a conductor. It has no anchor a net can
//! reach, so these commands never snap a position to the grid. A text box also
//! carries a size, which `resize` changes and no other command touches.

use kicli_sexpr::{Doc, NodeId, SexprError, quote};

use crate::edit::insert::{fresh_uuid, insertion_index};
use crate::geometry::{Angle, Point, Size};
use crate::model::items::{ReadError, Schematic, Uuid};
use crate::model::mutate::{Mutation, MutationError, Target, commit, state_before};
use crate::view::snapshot::SnapshotError;

/// The text size KiCad gives a new text item, in millimetres.
const DEFAULT_TEXT_SIZE: &str = "1.27";

/// The margin KiCad gives a new text box, in millimetres.
const DEFAULT_BOX_MARGIN: &str = "0.9525";

/// What a new text object says, and where it goes.
///
/// A request with a size makes a text box. A request without one makes free
/// text.
#[derive(Clone, Debug)]
pub struct NewText {
    /// The text to draw.
    pub text: String,
    /// Where the text is drawn.
    pub at: Point,
    /// The text angle.
    pub angle: Angle,
    /// The width and height of the box, for a text box.
    pub size: Option<Size>,
}

/// What a text command changed.
#[derive(Clone, Debug)]
pub struct TextChange {
    /// The identifier of the object the command made, changed or removed.
    pub uuid: Uuid,
    /// What the mutation touched, and what kicli checked afterwards.
    pub mutation: Mutation,
}

/// Why a text command did not happen.
#[derive(Debug, thiserror::Error)]
pub enum TextError {
    /// The file holds no object with that identifier.
    #[error("this sheet has no object with the identifier {0}")]
    NotFound(String),
    /// The object is there, but it is not text.
    #[error("{0} is a {1}, and this command works on text")]
    NotText(String, String),
    /// The object is free text, which has no size.
    #[error("{0} is free text, so it has no size; only a text box can be resized")]
    NotABox(String),
    /// The object is text, but it lacks the list this command edits.
    #[error("{0} has no {1} list, so kicli will not guess where to write one")]
    Malformed(String, String),
    /// The file could not be read as a schematic.
    #[error(transparent)]
    Read(#[from] ReadError),
    /// The change was refused, or the file could not be written.
    #[error(transparent)]
    Mutation(#[from] MutationError),
    /// The state to compare against could not be taken.
    #[error(transparent)]
    Snapshot(#[from] SnapshotError),
    /// The new object could not be built.
    #[error(transparent)]
    Fragment(#[from] SexprError),
}

/// Add free text, or a text box, to a sheet.
///
/// The position is written as it was given. Graphic text is exempt from the
/// grid rule, because KiCad's own output places text on arbitrary units.
///
/// # Errors
///
/// Returns [`TextError`] when the file is not a schematic, or when the change
/// does not survive the invariants. Nothing is written unless it does.
pub fn add(
    doc: &mut Doc,
    target: &Target<'_>,
    request: &NewText,
    taken: &str,
) -> Result<TextChange, TextError> {
    let schematic = Schematic::read(doc)?;
    let root = doc.root().ok_or(ReadError::Empty)?;
    let before = state_before(doc, &schematic, target.sheet_path, taken)?;

    let uuid = fresh_uuid(
        doc,
        &format!(
            "{} text {} {} {}",
            target.path.display(),
            request.text,
            request.at,
            request.angle
        ),
    );
    let fragment = doc.add_fragment(&fragment_of(request, &uuid))?;
    let index = insertion_index(doc, root);
    doc.insert_child(root, index, fragment);

    let mutation = commit(doc, target, &before, taken)?;
    Ok(TextChange { uuid, mutation })
}

/// Move text to a position.
///
/// # Errors
///
/// Returns [`TextError`] when the identifier names no text, when the object
/// carries no position, or when the change does not survive the invariants.
pub fn move_to(
    doc: &mut Doc,
    target: &Target<'_>,
    uuid: &Uuid,
    to: Point,
    taken: &str,
) -> Result<TextChange, TextError> {
    change(doc, target, uuid, taken, |doc, node| {
        let at = list_of(doc, node, "at")
            .ok_or_else(|| TextError::Malformed(uuid.0.clone(), "at".to_owned()))?;
        let (Some(&x), Some(&y)) = (doc.children(at).get(1), doc.children(at).get(2)) else {
            return Err(TextError::Malformed(uuid.0.clone(), "at".to_owned()));
        };
        doc.set_atom(x, &to.x.to_string());
        doc.set_atom(y, &to.y.to_string());
        Ok(())
    })
}

/// Replace what the text says.
///
/// # Errors
///
/// Returns [`TextError`] when the identifier names no text, or when the change
/// does not survive the invariants.
pub fn edit(
    doc: &mut Doc,
    target: &Target<'_>,
    uuid: &Uuid,
    text: &str,
    taken: &str,
) -> Result<TextChange, TextError> {
    change(doc, target, uuid, taken, |doc, node| {
        let value = *doc
            .children(node)
            .get(1)
            .ok_or_else(|| TextError::Malformed(uuid.0.clone(), "text".to_owned()))?;
        doc.set_atom(value, &quote(text));
        Ok(())
    })
}

/// Give a text box a new width and height.
///
/// # Errors
///
/// Returns [`TextError`] when the identifier names free text rather than a
/// box, or when the change does not survive the invariants.
pub fn resize(
    doc: &mut Doc,
    target: &Target<'_>,
    uuid: &Uuid,
    size: Size,
    taken: &str,
) -> Result<TextChange, TextError> {
    change(doc, target, uuid, taken, |doc, node| {
        let list = list_of(doc, node, "size").ok_or_else(|| TextError::NotABox(uuid.0.clone()))?;
        let (Some(&width), Some(&height)) = (doc.children(list).get(1), doc.children(list).get(2))
        else {
            return Err(TextError::Malformed(uuid.0.clone(), "size".to_owned()));
        };
        doc.set_atom(width, &size.x.to_string());
        doc.set_atom(height, &size.y.to_string());
        Ok(())
    })
}

/// Take text off a sheet.
///
/// # Errors
///
/// Returns [`TextError`] when the identifier names no text, or when the change
/// does not survive the invariants.
pub fn delete(
    doc: &mut Doc,
    target: &Target<'_>,
    uuid: &Uuid,
    taken: &str,
) -> Result<TextChange, TextError> {
    change(doc, target, uuid, taken, |doc, node| {
        doc.remove(node);
        Ok(())
    })
}

/// Run one edit over an existing text object, then write and report it.
fn change(
    doc: &mut Doc,
    target: &Target<'_>,
    uuid: &Uuid,
    taken: &str,
    edit: impl FnOnce(&mut Doc, NodeId) -> Result<(), TextError>,
) -> Result<TextChange, TextError> {
    let schematic = Schematic::read(doc)?;
    let node = locate(doc, uuid)?;
    let before = state_before(doc, &schematic, target.sheet_path, taken)?;
    edit(doc, node)?;
    let mutation = commit(doc, target, &before, taken)?;
    Ok(TextChange {
        uuid: uuid.clone(),
        mutation,
    })
}

/// The `text` or `text_box` list an identifier names.
fn locate(doc: &Doc, uuid: &Uuid) -> Result<NodeId, TextError> {
    let node = doc
        .uuid_index()
        .get(&uuid.0)
        .copied()
        .ok_or_else(|| TextError::NotFound(uuid.0.clone()))?;
    match doc.head(node) {
        Some("text" | "text_box") => Ok(node),
        Some(other) => Err(TextError::NotText(uuid.0.clone(), other.to_owned())),
        None => Err(TextError::NotFound(uuid.0.clone())),
    }
}

/// The named child list of a list.
fn list_of(doc: &Doc, node: NodeId, head: &str) -> Option<NodeId> {
    doc.children(node)
        .iter()
        .copied()
        .find(|&child| doc.head_is(child, head))
}

/// The text of a new object, in the shape KiCad writes.
fn fragment_of(request: &NewText, uuid: &Uuid) -> String {
    let text = quote(&request.text);
    let uuid = quote(&uuid.0);
    let (x, y, angle) = (request.at.x, request.at.y, request.angle);
    let Some(size) = request.size else {
        return format!(
            "(text {text} (exclude_from_sim no) (at {x} {y} {angle}) \
             (effects (font (size {DEFAULT_TEXT_SIZE} {DEFAULT_TEXT_SIZE}))) (uuid {uuid}))"
        );
    };
    let margin = DEFAULT_BOX_MARGIN;
    format!(
        "(text_box {text} (exclude_from_sim no) (at {x} {y} {angle}) \
         (size {} {}) (margins {margin} {margin} {margin} {margin}) \
         (stroke (width 0) (type default)) (fill (type none)) \
         (effects (font (size {DEFAULT_TEXT_SIZE} {DEFAULT_TEXT_SIZE})) (justify left top)) \
         (uuid {uuid}))",
        size.x, size.y
    )
}

#[cfg(test)]
mod tests {
    use super::{NewText, fragment_of};
    use crate::geometry::{Angle, Point, Size};
    use crate::model::items::Uuid;
    use kicli_sexpr::Doc;

    fn request() -> NewText {
        NewText {
            text: "say \"hello\"".to_owned(),
            at: Point::new(254_000, 508_000),
            angle: Angle(90),
            size: None,
        }
    }

    #[test]
    fn a_new_text_carries_its_escapes_into_the_file() {
        let fragment = fragment_of(&request(), &Uuid("abc".to_owned()));
        let doc = Doc::parse(&fragment).expect("the fragment parses");
        let root = doc.root().expect("it has a root");
        assert!(doc.head_is(root, "text"));
        assert_eq!(
            doc.atom_as_str(doc.children(root)[1]).as_deref(),
            Some("say \"hello\"")
        );
        assert!(fragment.contains("(at 25.4 50.8 90)"), "{fragment}");
    }

    #[test]
    fn a_new_box_carries_its_size() {
        let request = NewText {
            size: Some(Size::new(508_000, 254_000)),
            ..request()
        };
        let fragment = fragment_of(&request, &Uuid("abc".to_owned()));
        assert!(fragment.starts_with("(text_box "), "{fragment}");
        assert!(fragment.contains("(size 50.8 25.4)"), "{fragment}");
    }
}
