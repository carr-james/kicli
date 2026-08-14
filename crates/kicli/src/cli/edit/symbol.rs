//! The `sym` noun: placing, moving, turning, mirroring and deleting a symbol.
//!
//! A placement must carry its definition, because KiCad draws the copy embedded
//! in the sheet and not the library file. This build has no library table, so
//! the definition comes from a `.kicad_sym` file the caller names, or from the
//! copy the project already holds. Resolving a bare library identifier through
//! the configured library paths is later work, and the refusal says so rather
//! than writing a placement that draws as a placeholder.

use std::path::Path;

use kicli_sexpr::{Doc, Node, NodeId};
use serde_json::json;

use crate::cli::args::{Axis, Global, MoveArgs, PlaceArgs, SymVerb};
use crate::cli::edit::{Editing, Note, address, code_for, code_for_snapshot, report};
use crate::cli::exit::ExitCode;
use crate::cli::output::{Failure, Report};
use crate::edit::insert::Identifiers;
use crate::edit::symbol::{
    EditError, Edited, Finding, Instance, Motion, Options, Placement, delete_symbol, mirror_symbol,
    move_symbol, place_symbol, rotate_symbol,
};
use crate::geometry::Iu;
use crate::model::items::{LibId, Refdes, Symbol};
use crate::model::mutate::Mutation;

/// Run one verb of the `sym` noun.
///
/// # Errors
///
/// Returns a [`Failure`] carrying the row of the exit-code table the command
/// ended on.
pub fn run(global: &Global, verb: &SymVerb) -> Result<Report, Failure> {
    match verb {
        SymVerb::Place(args) => place(global, args),

        SymVerb::Move(args) => {
            let motion = motion_of(args)?;
            let options = Options {
                off_grid: args.off_grid,
                keep_field_positions: args.keep_field_positions,
            };
            change(global, &args.target, |doc, symbol, grid| {
                move_symbol(doc, symbol, motion, grid, options)
            })
        }

        SymVerb::Rotate {
            target,
            to,
            keep_field_positions,
        } => {
            let options = turning(*keep_field_positions);
            let to = to.angle();
            change(global, target, |doc, symbol, _| {
                rotate_symbol(doc, symbol, to, options)
            })
        }

        SymVerb::Mirror {
            target,
            axis,
            keep_field_positions,
        } => {
            let options = turning(*keep_field_positions);
            let axis = axis.mirror();
            change(global, target, |doc, symbol, _| {
                mirror_symbol(doc, symbol, axis, options)
            })
        }

        SymVerb::Delete { target } => delete(global, target),

        SymVerb::SetField {
            target,
            name,
            value,
        } => super::field::set_field(global, target, name, value),
    }
}

/// Where a move puts the anchor.
fn motion_of(args: &MoveArgs) -> Result<Motion, Failure> {
    match (args.to, args.by) {
        (Some(place), _) => Ok(Motion::To(place.point())),
        (None, Some(offset)) => Ok(Motion::By(offset.point())),
        // The argument parser requires one of the two.
        (None, None) => Err(Failure::new(ExitCode::Usage, "a move needs --to or --by.")),
    }
}

/// The options a turn takes. A turn leaves the anchor where it is.
const fn turning(keep_field_positions: bool) -> Options {
    Options {
        off_grid: false,
        keep_field_positions,
    }
}

/// One change to one placed symbol, written and reported.
fn change(
    global: &Global,
    target: &str,
    edit: impl FnOnce(&mut Doc, &Symbol, Iu) -> Result<Edited, EditError>,
) -> Result<Report, Failure> {
    let mut editing = Editing::open(global)?;
    let symbol = address::symbol(editing.schematic(), editing.place.sheet_path(), target)?.clone();
    let grid = editing.place.grid();

    let before = editing
        .state()
        .map_err(|error| Failure::new(code_for_snapshot(&error), error.to_string()))?;
    let edited = edit(editing.doc(), &symbol, grid).map_err(|error| refused(&error))?;
    let mutation = editing
        .commit(&before)
        .map_err(|error| Failure::new(code_for(&error), error.to_string()))?;

    Ok(report(
        &mutation,
        Some(("symbol", json!({ "uuid": edited.symbol.0 }))),
        &notes_of(&edited.findings),
    ))
}

/// Delete a symbol, its instance data and its unused definition.
fn delete(global: &Global, target: &str) -> Result<Report, Failure> {
    let mut editing = Editing::open(global)?;
    let symbol = address::symbol(editing.schematic(), editing.place.sheet_path(), target)?.clone();
    let schematic = editing.schematic().clone();

    let before = editing
        .state()
        .map_err(|error| Failure::new(code_for_snapshot(&error), error.to_string()))?;
    let edited =
        delete_symbol(editing.doc(), &schematic, &symbol).map_err(|error| refused(&error))?;
    let mutation = editing
        .commit(&before)
        .map_err(|error| Failure::new(code_for(&error), error.to_string()))?;

    Ok(report(
        &mutation,
        Some(("symbol", json!({ "uuid": edited.symbol.0 }))),
        &[],
    ))
}

/// Place a symbol, with its definition and its instance data.
fn place(global: &Global, args: &PlaceArgs) -> Result<Report, Failure> {
    let mut editing = Editing::open(global)?;
    let lib_id = LibId(args.lib_id.clone());
    let reference = Refdes(args.reference.clone());
    let definition = definition_text(&editing, &lib_id, args.from.as_deref())?;
    let instances = instances_of(&editing, &reference, args.unit)?;
    let schematic = editing.schematic().clone();
    let grid = editing.place.grid();

    let placement = Placement {
        lib_id: &lib_id,
        definition: &definition,
        at: args.at.point(),
        angle: args.angle.angle(),
        mirror: args.mirror.map(Axis::mirror),
        unit: args.unit,
        body_style: args.body_style,
        value: args.value.as_deref(),
        instances: &instances,
    };
    let options = Options {
        off_grid: args.off_grid,
        keep_field_positions: false,
    };
    let seed = format!(
        "place {} {} {} {}",
        lib_id.0,
        reference.0,
        args.at.point(),
        editing.taken
    );
    let mut identifiers = Identifiers::for_document(editing.doc(), &seed);

    let before = editing
        .state()
        .map_err(|error| Failure::new(code_for_snapshot(&error), error.to_string()))?;
    let edited = place_symbol(
        editing.doc(),
        &schematic,
        &placement,
        grid,
        options,
        &mut identifiers,
    )
    .map_err(|error| refused(&error))?;
    let mutation = editing
        .commit(&before)
        .map_err(|error| Failure::new(code_for(&error), error.to_string()))?;

    Ok(placed(&mutation, &edited, &lib_id, &reference, &instances))
}

/// The report of a placement: what changed, and how to address what was made.
fn placed(
    mutation: &Mutation,
    edited: &Edited,
    lib_id: &LibId,
    reference: &Refdes,
    instances: &[Instance],
) -> Report {
    report(
        mutation,
        Some((
            "symbol",
            json!({
                "uuid": edited.symbol.0,
                "reference": reference.0,
                "lib_id": lib_id.0,
                "sheet_paths": instances
                    .iter()
                    .map(|instance| instance.path.0.clone())
                    .collect::<Vec<String>>(),
            }),
        )),
        &notes_of(&edited.findings),
    )
}

/// One instance entry per sheet path this file is drawn on.
///
/// A sheet placed twice gets two references, which is where most tools get this
/// wrong. Both start with the reference the caller asked for; `sym set-field`
/// changes one path's without touching the other.
fn instances_of(
    editing: &Editing,
    reference: &Refdes,
    unit: u32,
) -> Result<Vec<Instance>, Failure> {
    let project = project_name(editing);
    let instances: Vec<Instance> = editing
        .loaded
        .hierarchy
        .placements_of(editing.file)
        .map(|placement| Instance {
            project: project.clone(),
            path: placement.path.clone(),
            reference: reference.clone(),
            unit,
        })
        .collect();

    if instances.is_empty() {
        return Err(Failure::new(
            ExitCode::Operation,
            "this file is on no sheet path, so a placement would have no reference.",
        ));
    }
    Ok(instances)
}

/// The project name the file's instance data is filed under.
///
/// KiCad keys instance data by project name, so a new placement must use the
/// name the file's other placements use. A file with no symbol yet has none, and
/// the project's own name is what KiCad would write.
fn project_name(editing: &Editing) -> String {
    editing
        .schematic()
        .symbols()
        .find_map(|symbol| symbol.placements.first())
        .map_or_else(
            || editing.loaded.name.clone(),
            |first| first.project.clone(),
        )
}

/// The definition a placement embeds, as a library file writes it.
fn definition_text(
    editing: &Editing,
    lib_id: &LibId,
    from: Option<&Path>,
) -> Result<String, Failure> {
    if let Some(file) = from {
        return from_library_file(file, lib_id);
    }
    for loaded in &editing.loaded.hierarchy.files {
        if let Some((_, node)) = loaded
            .schematic
            .library_symbols
            .iter()
            .find(|(name, _)| *name == lib_id.0)
            && let Some(text) = list_text(&loaded.doc, *node)
        {
            return Ok(text.to_owned());
        }
    }
    Err(Failure::new(
        ExitCode::Operation,
        format!(
            "no file of this project embeds {}, so kicli cannot draw it. \
             Name a .kicad_sym file with --from. \
             kicli does not resolve a library identifier through the library tables in this build.",
            lib_id.0
        ),
    ))
}

/// One definition, read out of a `.kicad_sym` file.
fn from_library_file(file: &Path, lib_id: &LibId) -> Result<String, Failure> {
    let source = std::fs::read_to_string(file).map_err(|error| unreadable(file, &error))?;
    let doc = Doc::parse(&source).map_err(|error| unreadable(file, &error))?;
    let root = doc.root().ok_or_else(|| {
        Failure::new(
            ExitCode::File,
            format!("{} holds no library.", file.display()),
        )
    })?;

    let wanted = lib_id.symbol_name();
    let mut names = Vec::new();
    for &child in doc.children(root) {
        if !doc.head_is(child, "symbol") {
            continue;
        }
        let Some(name) = doc
            .children(child)
            .get(1)
            .and_then(|&id| doc.atom_as_str(id))
        else {
            continue;
        };
        if name == wanted || name == lib_id.0 {
            return list_text(&doc, child).map(str::to_owned).ok_or_else(|| {
                Failure::new(
                    ExitCode::File,
                    format!("{name} in {} is not a list.", file.display()),
                )
            });
        }
        names.push(name);
    }

    Err(Failure::new(
        ExitCode::Operation,
        format!(
            "{} holds no symbol called {wanted}. It holds: {}.",
            file.display(),
            names.join(", ")
        ),
    ))
}

/// A library file that will not read.
fn unreadable(file: &Path, reason: &dyn std::fmt::Display) -> Failure {
    Failure::new(
        ExitCode::File,
        format!("cannot read {}: {reason}", file.display()),
    )
}

/// The source text of one list, exactly as the file writes it.
fn list_text(doc: &Doc, node: NodeId) -> Option<&str> {
    match doc.node(node) {
        Node::List { span, .. } => doc.source().get(span.clone()),
        _ => None,
    }
}

/// A grid finding, as a report writes it.
fn notes_of(findings: &[Finding]) -> Vec<Note> {
    findings
        .iter()
        .map(|finding| Note::new(finding.name(), finding.to_string()))
        .collect()
}

/// Which row of the table a refused symbol command is.
fn refused(error: &EditError) -> Failure {
    let code = match error {
        // The argument parser accepts four angles, so this is unreachable from
        // the command line and is a usage error wherever it does arrive.
        EditError::NotARightAngle(_) => ExitCode::Usage,
        _ => ExitCode::Operation,
    };
    Failure::new(code, error.to_string())
}
