//! The `project` noun: what a project is.
//!
//! `project info` answers the first question an agent asks about a design it
//! has never seen: which files it is made of, how the sheets hang together, what
//! is on each of them, and whether the external tools are there.

use super::args::Global;
use super::exit::ExitCode;
use super::locate::Loaded;
use super::output::{Failure, Report, Reporter};
use super::tools::{ToolStatus, probe};
use crate::connectivity::{Nets, extract};
use crate::model::{Placement, SheetPath};
use kicli_sexpr::{Doc, FormatMode};
use serde_json::{Value, json};
use std::fmt::Write as _;

/// What kicli does not answer yet, named rather than passed over in silence.
///
/// The net count left this list when the extractor landed. An empty list is
/// printed as no section at all, rather than as a heading with nothing under
/// it, so a reader is never told there is a gap when there is none.
const NOT_COVERED: &[&str] = &[];

/// One sheet placement, as the report names it.
struct SheetRecord {
    /// The page number, as written.
    page: Option<String>,
    /// The sheet name, as drawn. The root sheet has none.
    name: Option<String>,
    /// The file this placement draws, relative to the project directory.
    file: String,
    /// The sheet path, in KiCad's form.
    path: String,
    /// How many symbols the file holds.
    symbols: usize,
    /// How many of those are power symbols.
    power_symbols: usize,
}

/// One file of the project, as the report names it.
struct FileRecord {
    /// The file, relative to the project directory.
    file: String,
    /// The format stamp in `(version ...)`.
    stamp: u32,
    /// Which layout KiCad writes this kind of file in.
    layout: &'static str,
    /// Is the file already in that layout?
    canonical: bool,
}

/// Run `project info`.
///
/// # Errors
///
/// Returns a [`Failure`] when the project does not read, or when `--sheet`
/// names a path the tree does not hold.
pub fn info(global: &Global, reporter: &Reporter) -> Result<Report, Failure> {
    let loaded = Loaded::for_command(global)?;

    let wanted = chosen_placements(&loaded, global.sheet.as_deref())?;
    let sheets = sheet_records(&loaded, &wanted);
    let files = file_records(&loaded, &wanted);
    let status = probe(reporter, &loaded.config);

    // The extractor answers this now, so the report says how many nets the
    // drawing has rather than that kicli cannot tell.
    let nets = extract(&loaded.hierarchy);

    Ok(Report {
        text: as_text(&loaded, &sheets, &files, &status, &nets),
        json: as_json(&loaded, &sheets, &files, &status, &nets),
    })
}

/// The placements the caller asked about, by index.
fn chosen_placements(loaded: &Loaded, sheet: Option<&str>) -> Result<Vec<usize>, Failure> {
    let Some(wanted) = sheet else {
        return Ok((0..loaded.hierarchy.placements.len()).collect());
    };

    let wanted = SheetPath(wanted.to_owned());
    let found: Vec<usize> = loaded
        .hierarchy
        .placements
        .iter()
        .enumerate()
        .filter(|(_, placement)| placement.path == wanted)
        .map(|(index, _)| index)
        .collect();

    if found.is_empty() {
        return Err(Failure::new(
            ExitCode::Usage,
            format!(
                "{} is not a sheet path of this project. Run project info to list them.",
                wanted.0
            ),
        ));
    }
    Ok(found)
}

/// One record per chosen placement, in tree order.
fn sheet_records(loaded: &Loaded, wanted: &[usize]) -> Vec<SheetRecord> {
    wanted
        .iter()
        .map(|&index| {
            let placement = &loaded.hierarchy.placements[index];
            let file = &loaded.hierarchy.files[placement.file];
            let symbols: Vec<_> = file.schematic.symbols().collect();
            SheetRecord {
                page: page_of(loaded, placement),
                name: placement.name.clone(),
                file: loaded.shorten(&file.path),
                path: placement.path.0.clone(),
                symbols: symbols.len(),
                power_symbols: symbols.iter().filter(|symbol| symbol.is_power()).count(),
            }
        })
        .collect()
}

/// The page number of one placement.
///
/// A child sheet records its page under the path of its parent. The root sheet
/// records its own under `sheet_instances`, which is a list of the file rather
/// than an item of it.
fn page_of(loaded: &Loaded, placement: &Placement) -> Option<String> {
    match &placement.page {
        Some(page) => Some(page.clone()),
        None => root_page(&loaded.hierarchy.files[placement.file].doc),
    }
}

/// The page number a file records for itself as a root.
fn root_page(doc: &Doc) -> Option<String> {
    let root = doc.root()?;
    let instances = child_named(doc, root, "sheet_instances")?;
    let path = child_named(doc, instances, "path")?;
    let page = child_named(doc, path, "page")?;
    doc.atom_as_str(*doc.children(page).get(1)?)
}

/// The first child list with a given head token.
fn child_named(doc: &Doc, node: kicli_sexpr::NodeId, token: &str) -> Option<kicli_sexpr::NodeId> {
    doc.children(node)
        .iter()
        .copied()
        .find(|&child| doc.head_is(child, token))
}

/// One record per file the chosen placements draw, in load order.
fn file_records(loaded: &Loaded, wanted: &[usize]) -> Vec<FileRecord> {
    let mut used: Vec<usize> = wanted
        .iter()
        .map(|&index| loaded.hierarchy.placements[index].file)
        .collect();
    used.sort_unstable();
    used.dedup();

    used.into_iter()
        .map(|index| {
            let file = &loaded.hierarchy.files[index];
            FileRecord {
                file: loaded.shorten(&file.path),
                stamp: file.schematic.version.stamp(),
                layout: layout_name(file.doc.mode()),
                canonical: file.doc.is_canonical(),
            }
        })
        .collect()
}

/// The name of a layout, as the fixture manifest writes it.
const fn layout_name(mode: FormatMode) -> &'static str {
    match mode {
        FormatMode::Normal => "normal",
        FormatMode::CompactTextProperties => "compact",
        FormatMode::LibraryTable => "library-table",
    }
}

/// The text form.
fn as_text(
    loaded: &Loaded,
    sheets: &[SheetRecord],
    files: &[FileRecord],
    status: &ToolStatus,
    nets: &Nets,
) -> String {
    let mut text = String::new();

    let _ = writeln!(text, "project {}", loaded.name);
    let _ = writeln!(
        text,
        "  file       {}",
        loaded
            .project_file
            .as_ref()
            .map_or_else(|| "none".to_owned(), |path| loaded.shorten(path))
    );
    let _ = writeln!(text, "  root       {}", loaded.shorten(&loaded.root));
    let _ = writeln!(text, "  kicad-cli  {}", status.summary());
    // A sheet that did not load is missing from the tree below. Saying how many
    // stops a reader taking a short tree for a small project.
    let faults = loaded.hierarchy.problems.len();
    if faults > 0 {
        let _ = writeln!(
            text,
            "  faults     {faults}, which project check names one by one"
        );
    }
    for alias in bus_aliases(loaded) {
        let _ = writeln!(text, "  alias      {} = {}", alias.0, alias.1.join(" "));
    }

    write_nets(&mut text, nets);

    let _ = writeln!(text, "\nsheets {}", sheets.len());
    for sheet in sheets {
        let _ = write!(
            text,
            "  page {}  symbols {}  power {}",
            sheet.page.as_deref().unwrap_or("-"),
            sheet.symbols,
            sheet.power_symbols
        );
        if let Some(name) = &sheet.name {
            let _ = write!(text, "  name {name}");
        }
        let _ = writeln!(text, "  file {}  path {}", sheet.file, sheet.path);
    }

    let _ = writeln!(text, "\nfiles {}", files.len());
    for file in files {
        let _ = writeln!(
            text,
            "  {}  stamp {}  layout {}  canonical {}",
            file.file,
            file.stamp,
            file.layout,
            yes_or_no(file.canonical)
        );
    }

    if !NOT_COVERED.is_empty() {
        let _ = writeln!(text, "\nnot covered");
        for item in NOT_COVERED {
            let _ = writeln!(text, "  {item}");
        }
    }
    text
}

/// Write the net count, and any warning that qualifies it.
///
/// A warning belongs beside the count it is about, not in a section of its own
/// at the end where a reader who has the number already has stopped.
fn write_nets(text: &mut String, nets: &Nets) {
    let _ = writeln!(text, "  nets       {}", nets.nets().len());
    for warning in nets.warnings() {
        let _ = writeln!(
            text,
            "  warning    {} {}: {}",
            warning.kind.code(),
            warning.sheet.0,
            warning.message()
        );
    }
}

/// The JSON form, carrying the same content.
fn as_json(
    loaded: &Loaded,
    sheets: &[SheetRecord],
    files: &[FileRecord],
    status: &ToolStatus,
    nets: &Nets,
) -> Value {
    json!({
        "project": {
            "name": loaded.name,
            "file": loaded.project_file.as_ref().map(|path| loaded.shorten(path)),
            "root": loaded.shorten(&loaded.root),
            "faults": loaded.hierarchy.problems.len(),
            "nets": nets.nets().len(),
        },
        "kicad_cli": status.to_json(),
        "bus_aliases": bus_aliases(loaded)
            .into_iter()
            .map(|(name, members)| json!({ "name": name, "members": members }))
            .collect::<Vec<Value>>(),
        "sheets": sheets
            .iter()
            .map(|sheet| json!({
                "page": sheet.page,
                "name": sheet.name,
                "file": sheet.file,
                "path": sheet.path,
                "symbols": sheet.symbols,
                "power_symbols": sheet.power_symbols,
            }))
            .collect::<Vec<Value>>(),
        "files": files
            .iter()
            .map(|file| json!({
                "file": file.file,
                "stamp": file.stamp,
                "layout": file.layout,
                "canonical": file.canonical,
            }))
            .collect::<Vec<Value>>(),
        "not_covered": NOT_COVERED,
        "warnings": nets
            .warnings()
            .iter()
            .map(|warning| json!({
                "code": warning.kind.code(),
                "sheet": warning.sheet.0,
                "message": warning.message(),
            }))
            .collect::<Vec<Value>>(),
    })
}

/// The project file's bus aliases, by name.
fn bus_aliases(loaded: &Loaded) -> Vec<(String, Vec<String>)> {
    loaded.project.as_ref().map_or_else(Vec::new, |project| {
        project
            .bus_aliases
            .iter()
            .map(|alias| (alias.name.clone(), alias.members.clone()))
            .collect()
    })
}

/// A flag, written the way the report writes flags.
const fn yes_or_no(flag: bool) -> &'static str {
    if flag { "yes" } else { "no" }
}
