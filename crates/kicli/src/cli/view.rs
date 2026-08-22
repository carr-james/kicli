//! The `sch view` command.
//!
//! It prints one of the structured views of a project. A view is what an agent
//! acts on, so the terse form is the default and JSON is printed only when it
//! is asked for.
//!
//! Two of the views describe the drawing as it is now. The third compares the
//! drawing with a state kicli saved, and answers a different question: what has
//! touched this file since kicli last wrote it? It is empty right after a
//! mutation, because the mutation reported its own changes.

use crate::cli::args::{Global, ViewArgs, ViewName};
use crate::cli::exit::ExitCode;
use crate::cli::locate::Loaded;
use crate::cli::output::{Failure, Report, Reporter};
use crate::connectivity::extract;
use crate::model::items::SheetPath;
use crate::model::mutate::LAST_WRITE;
use crate::view::connectivity::ViewOptions;
use crate::view::delta::Delta;
use crate::view::snapshot::Snapshot;
use crate::view::{Kind, Scope, connectivity, delta, layout, scope};
use std::fmt::Write as _;
use std::path::Path;

/// The name the delta gives the state of the file as it is now.
const CURRENT: &str = "current";

/// Print a view of the project.
///
/// # Errors
///
/// Returns a [`Failure`] when the project cannot be found or read, when the
/// sheet flag names a placement this project does not have, or when the delta
/// has no saved state to compare against.
pub fn view(global: &Global, args: &ViewArgs, reporter: &Reporter) -> Result<Report, Failure> {
    let ViewArgs {
        view: which,
        include_power,
        uuids,
        stats,
        against,
    } = args;
    let loaded = Loaded::for_command(global)?;

    if *which == ViewName::Delta {
        if *include_power || *uuids {
            reporter.note("--include-power and --uuids have no effect on the delta.");
        }
        let against = against.as_deref().unwrap_or(LAST_WRITE);
        return compare(global, &loaded, against, *stats);
    }
    if against.is_some() {
        reporter.note("--against has no effect on a view other than the delta.");
    }

    records(
        global,
        &loaded,
        *which,
        *include_power,
        *uuids,
        *stats,
        reporter,
    )
}

/// Print what the drawing holds now: the connectivity view or the layout digest.
fn records(
    global: &Global,
    loaded: &Loaded,
    which: ViewName,
    include_power: bool,
    uuids: bool,
    stats: bool,
    reporter: &Reporter,
) -> Result<Report, Failure> {
    let sheet = match &global.sheet {
        None => None,
        Some(path) => {
            let wanted = SheetPath(path.clone());
            if !loaded
                .hierarchy
                .placements
                .iter()
                .any(|placement| placement.path == wanted)
            {
                return Err(Failure::new(
                    ExitCode::Usage,
                    format!("no sheet of this project has the path {path}"),
                ));
            }
            Some(wanted)
        }
    };

    let options = ViewOptions {
        include_power,
        uuids,
        sheet,
    };
    let nets = extract(&loaded.hierarchy);
    let kind = match which {
        ViewName::Layout => Kind::Layout,
        // The delta is not a view of the drawing as it is, so it is answered
        // before this function is called.
        ViewName::Connectivity | ViewName::Delta => Kind::Connectivity,
    };
    let rendered = scope::render(
        kind,
        &loaded.hierarchy,
        &nets,
        &options,
        loaded.config.view.max_bytes,
    );

    if rendered.scope == Scope::IndexAndSummaries {
        reporter.note(&format!(
            "the whole project needs more than the {} byte budget, so this is the index",
            loaded.config.view.max_bytes
        ));
    }

    let mut json = match kind {
        Kind::Connectivity => connectivity::to_json(&loaded.hierarchy, &nets, &options),
        Kind::Layout => layout::to_json(&loaded.hierarchy, &options),
    };
    json["scope"] = rendered.scope.token().into();
    if stats {
        json["bytes"] = rendered.bytes.into();
    }

    let text = if stats {
        format!("{}# {} bytes\n", rendered.text, rendered.bytes)
    } else {
        rendered.text.clone()
    };
    Ok(Report { text, json })
}

/// Print what has touched the file since a saved state was taken.
fn compare(
    global: &Global,
    loaded: &Loaded,
    against: &str,
    stats: bool,
) -> Result<Report, Failure> {
    let saved = read_saved_state(&loaded.directory, against)?;
    let file = compared_file(loaded, &saved, global.sheet.as_deref())?;
    let file = &loaded.hierarchy.files[file];

    // The stamp belongs to the header of a snapshot file. This state is
    // compared and never written, so it carries the stamp of the state it is
    // compared against rather than a reading of a clock kicli never takes.
    let current = Snapshot::take(
        CURRENT,
        &saved.taken,
        &saved.sheet_path,
        &file.doc,
        &file.schematic,
    )
    .map_err(|error| Failure::new(ExitCode::File, error.to_string()))?;
    let difference = Delta::between(&saved, &current);

    let compared = compared_form(&saved);
    let (text, rendered_scope) = render(
        &difference,
        &saved.sheet_path,
        compared,
        loaded.config.view.max_bytes,
    );
    let mut json = to_json(&difference, &saved.sheet_path, compared, rendered_scope);

    let bytes = text.len();
    if stats {
        json["bytes"] = bytes.into();
        return Ok(Report {
            text: format!("{text}# {bytes} bytes\n"),
            json,
        });
    }
    Ok(Report { text, json })
}

/// The text of a comparison, and how much of it the budget allowed.
///
/// The lines are printed when they fit. Otherwise the counts stand in for them,
/// as the index stands in for the records of a view that does not fit.
fn render(difference: &Delta, sheet: &SheetPath, compared: &str, budget: usize) -> (String, Scope) {
    let whole = format!(
        "{}\n{}",
        header(difference, sheet, compared, Scope::OneSheet),
        lines(difference)
    );
    let summary = format!(
        "{}  full={}B budget={budget}B\n# raise view.max_bytes to see the lines\n{}{}",
        header(difference, sheet, compared, Scope::SheetSummary),
        whole.len(),
        counts(difference),
        unchanged(difference),
    );

    // The fallback exists to spend fewer bytes. A comparison of two or three
    // lines does not reach the size of its own summary, so falling back would
    // cost more than it saves and the lines are printed instead.
    if whole.len() <= budget || summary.len() >= whole.len() {
        return (whole, Scope::OneSheet);
    }
    (summary, Scope::SheetSummary)
}

/// The JSON twin, carrying what the text carries.
fn to_json(
    difference: &Delta,
    sheet: &SheetPath,
    compared: &str,
    scope: Scope,
) -> serde_json::Value {
    serde_json::json!({
        "from": difference.from,
        "to": difference.to,
        "sheet": sheet.0,
        "scope": scope.token(),
        "compared": compared,
        "changed": difference
            .lines
            .iter()
            .map(|line| serde_json::json!({
                "change": line.change.mark().to_string(),
                "record": line.record.to_string(),
                "handle": line.handle,
                "detail": line.detail,
            }))
            .collect::<Vec<_>>(),
        "unchanged": difference.unchanged,
    })
}

/// The first line, without its newline: what was compared, and how closely.
///
/// The summary form writes more on the same line, so the line is ended by
/// whoever prints it.
fn header(difference: &Delta, sheet: &SheetPath, compared: &str, scope: Scope) -> String {
    format!(
        "# delta {} -> {}  scope={}  sheet={}  compared={compared}",
        difference.from,
        difference.to,
        scope.token(),
        sheet.0,
    )
}

/// One line per changed object, then the count of what did not change.
fn lines(difference: &Delta) -> String {
    let mut text = String::new();
    for line in &difference.lines {
        let _ = writeln!(text, "{line}");
    }
    text.push_str(&unchanged(difference));
    text
}

/// How many objects both states hold unchanged.
fn unchanged(difference: &Delta) -> String {
    format!("= {} objects unchanged\n", difference.unchanged)
}

/// What changed, by kind, for a comparison too large to print in full.
fn counts(difference: &Delta) -> String {
    let mark = |wanted: delta::Change| {
        difference
            .lines
            .iter()
            .filter(|line| line.change == wanted)
            .count()
    };
    format!(
        "# added={} removed={} moved={} edited={}\n",
        mark(delta::Change::Added),
        mark(delta::Change::Removed),
        mark(delta::Change::Moved),
        mark(delta::Change::Edited),
    )
}

/// How much the saved state can say about what changed.
///
/// A snapshot file carries a display column beside its hashes, so a comparison
/// against it prints the old position and the old value. A file written before
/// that column existed carries hashes and names only, and then a line can say
/// that an object changed but not what it was. The reader is told which of the
/// two it is holding, so it never has to assume.
fn compared_form(saved: &Snapshot) -> &'static str {
    if saved.objects.iter().all(|object| object.detail.is_some()) {
        "values"
    } else {
        "hashes"
    }
}

/// Read the saved state a comparison is made against.
///
/// A state that is absent and a state that will not read are different
/// failures. A project kicli has never written has no state at all, which is a
/// well-formed request kicli cannot complete. A state that is there and does
/// not parse is a file error.
fn read_saved_state(project: &Path, name: &str) -> Result<Snapshot, Failure> {
    if !Snapshot::path_in(project, name).is_file() {
        return Err(Failure::new(ExitCode::Operation, no_saved_state(name)));
    }
    Snapshot::read_in(project, name)
        .map_err(|error| Failure::new(ExitCode::File, error.to_string()))
}

/// What to say when there is no state to compare against.
fn no_saved_state(name: &str) -> String {
    if name == LAST_WRITE {
        return format!(
            "this project has no {LAST_WRITE} state, so kicli has nothing to compare it with. \
             Every command that writes leaves one behind."
        );
    }
    format!(
        "this project has no saved state named {name}. \
         The default is {LAST_WRITE}, which every command that writes leaves behind."
    )
}

/// Which file of the project the saved state describes.
///
/// A state covers one sheet, because a mutation writes one file. `--sheet` may
/// name that sheet and no other, so a caller cannot be answered about a sheet
/// the state does not hold.
fn compared_file(loaded: &Loaded, saved: &Snapshot, sheet: Option<&str>) -> Result<usize, Failure> {
    if let Some(asked) = sheet {
        if asked != saved.sheet_path.0 {
            return Err(Failure::new(
                ExitCode::Operation,
                format!(
                    "the saved state {} covers the sheet path {}, not {asked}.",
                    saved.name, saved.sheet_path.0
                ),
            ));
        }
    }

    loaded
        .hierarchy
        .placements
        .iter()
        .find(|placement| placement.path == saved.sheet_path)
        .map(|placement| placement.file)
        .ok_or_else(|| {
            Failure::new(
                ExitCode::Operation,
                format!(
                    "the saved state {} covers the sheet path {}, which this project no longer has.",
                    saved.name, saved.sheet_path.0
                ),
            )
        })
}
