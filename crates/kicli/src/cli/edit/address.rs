//! Turning what a caller typed into the object it names.
//!
//! A view prints a symbol by its reference designator and everything else by
//! the first eight characters of its identifier. Both are what a caller has to
//! hand, so both address an object here. A prefix that names more than one
//! object is refused with the whole list, because eight characters do not
//! always identify: a generated file can give every object the same eight.

use crate::cli::exit::ExitCode;
use crate::cli::output::Failure;
use crate::model::items::{Item, Schematic, SheetPath, Symbol, Uuid};

/// How much of an identifier a caller must type.
///
/// Eight characters are what the views print, so eight is the floor. Fewer
/// would let a single character address a whole sheet.
const HANDLE_LENGTH: usize = 8;

/// Find the symbol a caller named, by reference designator or by identifier.
///
/// The reference designator is the one the symbol carries on `sheet`. A symbol
/// on a sheet placed twice has two, and only the sheet path decides which is
/// meant.
///
/// # Errors
///
/// Returns [`ExitCode::Operation`] when nothing of that name is on the sheet,
/// and when an identifier prefix names more than one symbol.
pub fn symbol<'a>(
    schematic: &'a Schematic,
    sheet: &SheetPath,
    target: &str,
) -> Result<&'a Symbol, Failure> {
    if let Some(found) = schematic.symbols().find(|symbol| {
        symbol
            .reference_on(sheet)
            .is_some_and(|had| had.0 == target)
    }) {
        return Ok(found);
    }

    let matched: Vec<&Symbol> = schematic
        .symbols()
        .filter(|symbol| matches_handle(&symbol.uuid, target))
        .collect();
    match matched.as_slice() {
        [only] => Ok(only),
        [] => Err(nothing_named(target, "symbol")),
        many => {
            let found: Vec<String> = many.iter().map(|symbol| symbol.uuid.0.clone()).collect();
            Err(ambiguous(target, &found))
        }
    }
}

/// Find the object a caller named by identifier.
///
/// # Errors
///
/// Returns [`ExitCode::Operation`] when the sheet holds no such object, and
/// when a prefix names more than one.
pub fn item<'a>(schematic: &'a Schematic, target: &str) -> Result<&'a Item, Failure> {
    let matched: Vec<&Item> = schematic
        .items
        .iter()
        .filter(|item| item.uuid().is_some_and(|uuid| matches_handle(uuid, target)))
        .collect();
    match matched.as_slice() {
        [only] => Ok(only),
        [] => Err(nothing_named(target, "object")),
        many => {
            let found: Vec<String> = many
                .iter()
                .filter_map(|item| item.uuid().map(|uuid| uuid.0.clone()))
                .collect();
            Err(ambiguous(target, &found))
        }
    }
}

/// The identifier of the object a caller named.
///
/// # Errors
///
/// The same errors as [`item`].
pub fn uuid(schematic: &Schematic, target: &str) -> Result<Uuid, Failure> {
    item(schematic, target).map(|item| {
        item.uuid()
            .cloned()
            .unwrap_or_else(|| Uuid(target.to_owned()))
    })
}

/// The identifier of the object that owns a field.
///
/// A symbol answers to its reference designator, because that is what a view
/// prints for it. Every other object answers to its identifier.
///
/// # Errors
///
/// The same errors as [`symbol`] and [`item`].
pub fn owner(schematic: &Schematic, sheet: &SheetPath, target: &str) -> Result<Uuid, Failure> {
    if let Some(found) = schematic.symbols().find(|symbol| {
        symbol
            .reference_on(sheet)
            .is_some_and(|had| had.0 == target)
    }) {
        return Ok(found.uuid.clone());
    }
    uuid(schematic, target)
}

/// Is this identifier the one the caller typed, whole or by its handle?
fn matches_handle(uuid: &Uuid, target: &str) -> bool {
    if uuid.0 == target {
        return true;
    }
    target.len() >= HANDLE_LENGTH && uuid.0.starts_with(target)
}

/// Nothing of this sheet answers to that name.
fn nothing_named(target: &str, kind: &str) -> Failure {
    Failure::new(
        ExitCode::Operation,
        format!(
            "this sheet has no {kind} called {target}. \
             Name a reference designator, or at least {HANDLE_LENGTH} characters of an identifier. \
             Run sch view --uuids to list them."
        ),
    )
}

/// More than one object answers to that name.
fn ambiguous(target: &str, found: &[String]) -> Failure {
    Failure::new(
        ExitCode::Operation,
        format!(
            "{target} names {} objects of this sheet: {}. Name more of the identifier.",
            found.len(),
            found.join(", ")
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::{item, owner, symbol};
    use crate::cli::exit::ExitCode;
    use crate::model::items::{Schematic, SheetPath};
    use kicli_sexpr::Doc;

    /// Two junctions whose identifiers share their first eight characters, and
    /// one symbol placed on the root sheet.
    const SOURCE: &str = concat!(
        "(kicad_sch (version 20260306) (uuid \"aaaaaaaa-0000-4000-8000-000000000000\")\n",
        "  (symbol (lib_id \"Device:R\") (at 0 0 0) (unit 1)\n",
        "    (uuid \"bbbbbbbb-0000-4000-8000-000000000001\")\n",
        "    (property \"Reference\" \"R1\" (at 0 0 0))\n",
        "    (instances (project \"p\" (path \"/aaaaaaaa-0000-4000-8000-000000000000\"\n",
        "      (reference \"R1\") (unit 1)))))\n",
        "  (junction (at 0 0) (uuid \"cccccccc-0000-4000-8000-000000000002\"))\n",
        "  (junction (at 0 0) (uuid \"cccccccc-0000-4000-8000-000000000003\"))\n",
        ")\n",
    );

    fn read() -> (Schematic, SheetPath) {
        let doc = Doc::parse(SOURCE).expect("the source parses");
        let schematic = Schematic::read(&doc).expect("the source is a schematic");
        let path = SheetPath::root(schematic.uuid.as_ref().expect("the file has a uuid"));
        (schematic, path)
    }

    #[test]
    fn a_symbol_answers_to_its_reference_and_to_its_identifier() {
        let (schematic, path) = read();
        assert_eq!(
            symbol(&schematic, &path, "R1")
                .expect("the reference finds it")
                .uuid
                .0,
            "bbbbbbbb-0000-4000-8000-000000000001"
        );
        assert!(
            symbol(&schematic, &path, "bbbbbbbb").is_ok(),
            "so does the handle"
        );
        assert!(
            symbol(&schematic, &path, "R2").is_err(),
            "and nothing else does"
        );
    }

    #[test]
    fn a_handle_that_names_two_objects_is_refused_with_both() {
        let (schematic, _) = read();
        let failure = item(&schematic, "cccccccc").expect_err("two junctions share it");
        assert_eq!(failure.code, ExitCode::Operation);
        assert!(
            failure.message.contains("000000000002") && failure.message.contains("000000000003"),
            "the refusal lists them: {}",
            failure.message
        );
        assert!(
            item(&schematic, "cccccccc-0000-4000-8000-000000000002").is_ok(),
            "the whole identifier still names one"
        );
    }

    #[test]
    fn a_short_prefix_names_nothing() {
        let (schematic, _) = read();
        let failure = item(&schematic, "ccc").expect_err("three characters do not identify");
        assert!(
            failure.message.contains("8 characters"),
            "the refusal says how many: {}",
            failure.message
        );
    }

    #[test]
    fn a_field_owner_is_a_symbol_or_any_other_object() {
        let (schematic, path) = read();
        assert_eq!(
            owner(&schematic, &path, "R1")
                .expect("a symbol owns fields")
                .0,
            "bbbbbbbb-0000-4000-8000-000000000001"
        );
        assert!(
            owner(&schematic, &path, "cccccccc-0000-4000-8000-000000000002").is_ok(),
            "and so does anything else, by its identifier"
        );
    }
}
