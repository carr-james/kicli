//! Finding the project a command was pointed at.
//!
//! A project is a directory. The `.kicad_pro` file names it, and the root sheet
//! is the `.kicad_sch` beside it with the same stem. A directory with no
//! project file still reads when it holds exactly one schematic, because a
//! single loose sheet is a common way to start.

use super::args::Global;
use super::exit::ExitCode;
use super::output::Failure;
use crate::model::{Config, Hierarchy, Project, read_project};
use std::path::{Path, PathBuf};

/// One project, read from disk.
pub struct Loaded {
    /// The directory the project lives in.
    pub directory: PathBuf,
    /// The same directory with `.`, `..` and symbolic links resolved.
    ///
    /// The sheet tree resolves child files the same way, so this is the form a
    /// loaded file's path can be measured against.
    resolved: PathBuf,
    /// The project name, which is the stem of the root file.
    pub name: String,
    /// The `.kicad_pro` file, when the project has one.
    pub project_file: Option<PathBuf>,
    /// The root `.kicad_sch`.
    pub root: PathBuf,
    /// The project's `kicli.toml`, or the defaults.
    pub config: Config,
    /// The sheet tree.
    pub hierarchy: Hierarchy,
    /// What the project file says, when there is one.
    pub project: Option<Project>,
}

impl Loaded {
    /// Read the project a command was pointed at.
    ///
    /// `--project` names the directory. Without it, the working directory is
    /// the project, which is what a person standing in one expects.
    ///
    /// # Errors
    ///
    /// Returns a [`Failure`] when the directory holds no project kicli can
    /// name, or when a file it does name will not read.
    pub fn for_command(global: &Global) -> Result<Self, Failure> {
        let directory = match &global.project {
            Some(named) => named.clone(),
            None => std::env::current_dir().map_err(|error| {
                Failure::new(
                    ExitCode::File,
                    format!("cannot read the working directory: {error}"),
                )
            })?,
        };
        Self::read(&directory)
    }

    /// Read the project in a directory.
    ///
    /// # Errors
    ///
    /// Returns [`ExitCode::Operation`] when the directory holds no project kicli
    /// can name, and [`ExitCode::File`] when a file it does name will not read.
    pub fn read(directory: &Path) -> Result<Self, Failure> {
        if !directory.is_dir() {
            return Err(Failure::new(
                ExitCode::File,
                format!("{} is not a directory.", directory.display()),
            ));
        }

        let (project_file, root) = roots(directory)?;
        let name = stem(&root);

        let config = Config::read(directory)
            .map_err(|error| Failure::new(ExitCode::File, error.to_string()))?;
        let hierarchy = Hierarchy::load(&root)
            .map_err(|error| Failure::new(ExitCode::File, error.to_string()))?;
        let project = match &project_file {
            Some(path) => Some(read_file(path)?),
            None => None,
        };

        Ok(Self {
            directory: directory.to_owned(),
            resolved: std::fs::canonicalize(directory).unwrap_or_else(|_| directory.to_owned()),
            name,
            project_file,
            root,
            config,
            hierarchy,
            project,
        })
    }

    /// The path of a file, as short as the project directory allows.
    ///
    /// A report names files the way a person reading the directory would. A
    /// file outside the project keeps its whole path, because shortening it
    /// would hide where it is.
    #[must_use]
    pub fn shorten(&self, path: &Path) -> String {
        for directory in [&self.directory, &self.resolved] {
            if let Ok(inside) = path.strip_prefix(directory) {
                return inside.display().to_string();
            }
        }
        path.display().to_string()
    }
}

/// The project file and root sheet of a directory.
fn roots(directory: &Path) -> Result<(Option<PathBuf>, PathBuf), Failure> {
    let projects = with_extension(directory, "kicad_pro")?;
    let schematics = with_extension(directory, "kicad_sch")?;

    match projects.len() {
        1 => {
            let project = projects[0].clone();
            let root = project.with_extension("kicad_sch");
            if root.is_file() {
                return Ok((Some(project), root));
            }
            Err(Failure::new(
                ExitCode::File,
                format!(
                    "{} names no root sheet. kicli expects {} beside it.",
                    project.display(),
                    root.display()
                ),
            ))
        }
        0 if schematics.is_empty() => Err(Failure::new(
            ExitCode::Operation,
            format!("{} holds no KiCad project.", directory.display()),
        )),
        0 => match root_without_a_project_file(directory, &schematics) {
            Some(root) => Ok((None, root)),
            None => Err(Failure::new(
                ExitCode::Operation,
                format!(
                    "{} holds {} schematics and no project file, so kicli cannot tell which is the root.",
                    directory.display(),
                    schematics.len()
                ),
            )),
        },
        found => Err(Failure::new(
            ExitCode::Operation,
            format!(
                "{} holds {found} project files. kicli reads one project at a time.",
                directory.display()
            ),
        )),
    }
}

/// The root sheet of a directory that holds no project file.
///
/// One schematic is the root. Several are a choice, and the only convention
/// that settles it is KiCad's own: a project directory is named after its
/// project, so `channel/channel.kicad_sch` is the root and its siblings are its
/// children. Without that match kicli asks rather than guesses.
fn root_without_a_project_file(directory: &Path, schematics: &[PathBuf]) -> Option<PathBuf> {
    if let [only] = schematics {
        return Some(only.clone());
    }
    let named = directory.file_name()?;
    schematics
        .iter()
        .find(|path| path.file_stem() == Some(named))
        .cloned()
}

/// Every file of a directory with one extension, sorted by name.
fn with_extension(directory: &Path, extension: &str) -> Result<Vec<PathBuf>, Failure> {
    let entries = std::fs::read_dir(directory).map_err(|error| {
        Failure::new(
            ExitCode::File,
            format!("cannot read {}: {error}", directory.display()),
        )
    })?;

    let mut found: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|kind| kind == extension))
        .collect();
    found.sort();
    Ok(found)
}

/// Read and parse a `.kicad_pro`.
fn read_file(path: &Path) -> Result<Project, Failure> {
    let text = std::fs::read_to_string(path).map_err(|error| {
        Failure::new(
            ExitCode::File,
            format!("cannot read {}: {error}", path.display()),
        )
    })?;
    read_project(&text).map_err(|error| {
        Failure::new(
            ExitCode::File,
            format!("cannot read {}: {error}", path.display()),
        )
    })
}

/// The file name of a path, without its extension.
fn stem(path: &Path) -> String {
    path.file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned()
}
