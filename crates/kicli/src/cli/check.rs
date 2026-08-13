//! The `project check` health check.
//!
//! Findings are data. A project with faults is still a project kicli read, so
//! the command reports what it found and succeeds. It fails only when a file
//! will not read at all.
//!
//! What the check cannot yet answer, it names. A check that silently passes on
//! a question it never asked teaches an agent that the answer was yes.

use super::args::Global;
use super::exit::ExitCode;
use super::locate::Loaded;
use super::output::{Failure, Report, Reporter};
use super::tools::{ToolStatus, probe};
use crate::model::{
    LoadedFile, Problem, SheetPath, Symbol, WriteOptions, WriteRefusal, plan_write,
};
use kicli_sexpr::{Doc, changed_line_count};
use serde_json::{Value, json};
use std::collections::BTreeSet;
use std::fmt::Write as _;

/// What this check does not answer, named rather than passed over in silence.
const NOT_COVERED: &[&str] =
    &["library resolution: lib_id values, Footprint fields, model paths and library nicknames"];

/// One thing the check found.
struct Finding {
    /// A short, stable name for what kind of fault this is.
    kind: &'static str,
    /// The file the fault is in, relative to the project directory.
    file: String,
    /// The sheet path it is on, when the fault has one.
    sheet_path: Option<String>,
    /// What is wrong, in one sentence.
    message: String,
}

impl Finding {
    /// Build a finding with no sheet path.
    fn in_file(kind: &'static str, file: String, message: String) -> Self {
        Self {
            kind,
            file,
            sheet_path: None,
            message,
        }
    }
}

/// Run `project check`.
///
/// # Errors
///
/// Returns [`ExitCode::Usage`] when `--sheet` is given, and [`ExitCode::File`]
/// when a file the project names will not read at all.
pub fn check(global: &Global, reporter: &Reporter) -> Result<Report, Failure> {
    if global.sheet.is_some() {
        return Err(Failure::new(
            ExitCode::Usage,
            "project check reads the whole project. --sheet is not accepted here.".to_owned(),
        ));
    }
    let loaded = Loaded::for_command(global)?;
    let status = probe(reporter, &loaded.config);

    // The order is fixed, so two runs over one project print the same lines:
    // the sheet tree first, then each file in load order, then the tools.
    let mut findings = tree_findings(&loaded);
    for (index, file) in loaded.hierarchy.files.iter().enumerate() {
        findings.extend(file_findings(&loaded, file));
        findings.extend(instance_findings(&loaded, index));
    }
    findings.extend(tool_findings(&status));

    Ok(Report {
        text: as_text(&loaded, &findings, &status),
        json: as_json(&loaded, &findings, &status),
    })
}

/// Faults in the sheet tree: a missing file, a file that will not read, a
/// cycle, or a sheet that names nothing.
fn tree_findings(loaded: &Loaded) -> Vec<Finding> {
    loaded
        .hierarchy
        .problems
        .iter()
        .map(|problem| {
            let (kind, path) = match problem {
                Problem::MissingFile { path, .. } => ("sheet-file-missing", path),
                Problem::Unreadable { path, .. } => ("sheet-file-unreadable", path),
                Problem::Cycle { path, .. } => ("sheet-cycle", path),
                Problem::NoFile { path } => ("sheet-file-unnamed", path),
            };
            Finding {
                kind,
                file: referring_file(loaded, path),
                sheet_path: Some(path.0.clone()),
                message: problem.to_string(),
            }
        })
        .collect()
}

/// The file that holds the sheet item at a sheet path.
///
/// The fault is in the drawing that names the child, not in the child, so the
/// report points at the parent. A path whose parent is not in the tree falls
/// back to the root, which is the only file that can hold it.
fn referring_file(loaded: &Loaded, path: &SheetPath) -> String {
    let parent = path
        .0
        .rsplit_once('/')
        .map(|(above, _)| SheetPath(above.to_owned()));

    let placement = parent.and_then(|parent| {
        loaded
            .hierarchy
            .placements
            .iter()
            .find(|placement| placement.path == parent)
    });

    let file = placement.map_or(0, |placement| placement.file);
    loaded.shorten(&loaded.hierarchy.files[file].path)
}

/// Faults in one file: it does not round-trip, its stamp is above the ceiling,
/// or kicli would refuse to write it.
fn file_findings(loaded: &Loaded, file: &LoadedFile) -> Vec<Finding> {
    let name = loaded.shorten(&file.path);
    let mut found = round_trip_findings(&name, &file.doc);

    let stamp = file.schematic.version;
    let ceiling = loaded.config.formats.max_schematic_version;
    if stamp > ceiling {
        found.push(Finding::in_file(
            "version-ceiling",
            name.clone(),
            format!(
                "the format stamp {} is above the ceiling {}, so kicli will not write this file",
                stamp.stamp(),
                ceiling.stamp()
            ),
        ));
    }

    // The ceiling is raised to this file's own stamp, so a stamp fault does not
    // hide a comment fault. The two are reported separately or not at all.
    let options = WriteOptions {
        allow_comment_loss: false,
        max_version: stamp.max(ceiling),
    };
    if let Err(refusal) = plan_write(&file.doc, options) {
        found.push(refusal_finding(&name, &refusal));
    }
    found
}

/// A refusal to write, as a finding.
fn refusal_finding(name: &str, refusal: &WriteRefusal) -> Finding {
    Finding::in_file("refuse-to-write", name.to_owned(), refusal.to_string())
}

/// Does a file come back out as it went in?
///
/// A file in KiCad's own layout must come back byte for byte. Every file must
/// come back with the same tokens and the same shape.
fn round_trip_findings(name: &str, doc: &Doc) -> Vec<Finding> {
    let mut found = Vec::new();
    let emitted = doc.emit();

    if doc.is_canonical() && emitted != doc.source() {
        found.push(Finding::in_file(
            "round-trip",
            name.to_owned(),
            format!(
                "the file is in KiCad's own layout, and writing it back changes {} line(s)",
                changed_line_count(doc.source(), &emitted)
            ),
        ));
    }

    let same_tokens = Doc::parse(&emitted).is_ok_and(|again| again.structurally_eq(doc));
    if !same_tokens {
        found.push(Finding::in_file(
            "round-trip",
            name.to_owned(),
            "the file does not read back as itself".to_owned(),
        ));
    }
    found
}

/// Does every symbol of a file carry instance data for the paths it is on?
///
/// A reference designator lives in the instance list, one entry per sheet path.
/// A path with no entry has no reference, and an entry for a path the sheet is
/// not on is data KiCad will drop the next time it saves.
fn instance_findings(loaded: &Loaded, file: usize) -> Vec<Finding> {
    let name = loaded.shorten(&loaded.hierarchy.files[file].path);
    let drawn: BTreeSet<&SheetPath> = loaded
        .hierarchy
        .placements_of(file)
        .map(|placement| &placement.path)
        .collect();

    let mut found = Vec::new();
    for symbol in loaded.hierarchy.files[file].schematic.symbols() {
        let listed: BTreeSet<&SheetPath> = symbol
            .placements
            .iter()
            .map(|placement| &placement.path)
            .collect();

        for path in drawn.difference(&listed) {
            found.push(Finding {
                kind: "instance-missing",
                file: name.clone(),
                sheet_path: Some(path.0.clone()),
                message: format!(
                    "symbol {} has no instance for this sheet path, so it has no reference there",
                    label(symbol)
                ),
            });
        }

        // Only this project's own entries are judged. A sheet shared with
        // another project carries that project's paths too, and they are not
        // this project's business.
        for placement in &symbol.placements {
            if placement.project == loaded.name && !drawn.contains(&placement.path) {
                found.push(Finding {
                    kind: "instance-orphan",
                    file: name.clone(),
                    sheet_path: Some(placement.path.0.clone()),
                    message: format!(
                        "symbol {} has an instance for a sheet path it is not drawn on",
                        label(symbol)
                    ),
                });
            }
        }
    }
    found
}

/// How a report names one symbol.
fn label(symbol: &Symbol) -> String {
    symbol
        .field("Reference")
        .map_or_else(|| symbol.uuid.0.clone(), |field| field.value.clone())
}

/// Is `kicad-cli` there, and of a version kicli reads?
fn tool_findings(status: &ToolStatus) -> Vec<Finding> {
    match &status.problem {
        None => Vec::new(),
        Some(problem) => vec![Finding::in_file(
            "kicad-cli",
            String::new(),
            problem.to_string(),
        )],
    }
}

/// The text form.
fn as_text(loaded: &Loaded, findings: &[Finding], status: &ToolStatus) -> String {
    let mut text = String::new();

    let _ = writeln!(text, "project {}", loaded.name);
    let _ = writeln!(text, "  root       {}", loaded.shorten(&loaded.root));
    let _ = writeln!(
        text,
        "  kicad-cli  {}",
        status
            .version
            .clone()
            .unwrap_or_else(|| "not usable, and named below".to_owned())
    );
    let _ = writeln!(
        text,
        "  checked    {} file(s), {} sheet(s)",
        loaded.hierarchy.files.len(),
        loaded.hierarchy.placements.len()
    );

    let _ = writeln!(text, "\nfindings {}", findings.len());
    let width = findings
        .iter()
        .map(|finding| finding.kind.len())
        .max()
        .unwrap_or(0);
    for finding in findings {
        let _ = writeln!(
            text,
            "  {:width$}  {}  {}",
            finding.kind,
            if finding.file.is_empty() {
                "-"
            } else {
                &finding.file
            },
            finding.message
        );
    }

    let _ = writeln!(text, "\nnot covered");
    for item in NOT_COVERED {
        let _ = writeln!(text, "  {item}");
    }
    text
}

/// The JSON form, carrying the same content.
fn as_json(loaded: &Loaded, findings: &[Finding], status: &ToolStatus) -> Value {
    json!({
        "project": {
            "name": loaded.name,
            "file": loaded.project_file.as_ref().map(|path| loaded.shorten(path)),
            "root": loaded.shorten(&loaded.root),
        },
        "kicad_cli": status.to_json(),
        "checked": {
            "files": loaded.hierarchy.files.len(),
            "sheets": loaded.hierarchy.placements.len(),
        },
        "findings": findings
            .iter()
            .map(|finding| json!({
                "kind": finding.kind,
                "file": finding.file,
                "sheet_path": finding.sheet_path,
                "message": finding.message,
            }))
            .collect::<Vec<Value>>(),
        "not_covered": NOT_COVERED,
    })
}
