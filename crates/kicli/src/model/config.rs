//! The project-local `kicli.toml`.
//!
//! Unknown keys are an error rather than a warning. An agent that misspells a
//! key would otherwise get the default silently and never learn why its setting
//! did nothing.
//!
//! Every section is validated, including the ones no command reads yet, so a
//! typo in a routing weight is caught by the milestone that adds the file and
//! not by the one that finally reads it.

use crate::geometry::{GRID, Iu};
use crate::model::version::{FormatVersion, MAX_SCHEMATIC_VERSION};
use std::path::Path;
use toml::Value;

/// The name of the file, in the project directory.
pub const FILE_NAME: &str = "kicli.toml";

/// Why a configuration file could not be used.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// The file exists but does not read.
    #[error("cannot read {path}: {reason}")]
    Unreadable {
        /// The file that did not read.
        path: String,
        /// Why it did not read.
        reason: String,
    },
    /// The file is not valid TOML.
    #[error("{0} is not valid TOML: {1}")]
    NotToml(String, String),
    /// A key nothing reads. Almost always a typo.
    #[error("unknown key {key} in [{section}]")]
    UnknownKey {
        /// The section the key sits in.
        section: String,
        /// The key nothing reads.
        key: String,
    },
    /// A section nothing reads.
    #[error("unknown section [{0}]")]
    UnknownSection(String),
    /// A key holds the wrong kind of value.
    #[error("{section}.{key} must be {expected}")]
    WrongType {
        /// The section the key sits in.
        section: String,
        /// The key with the wrong value.
        key: String,
        /// What the key needs.
        expected: String,
    },
    /// A length is not one kicli understands.
    #[error("{section}.{key} is not a length: {value}")]
    NotALength {
        /// The section the key sits in.
        section: String,
        /// The key with the bad value.
        key: String,
        /// What was written.
        value: String,
    },
}

/// Grid settings.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Grid {
    /// The placement grid. KiCad's schematic default is 50 mil.
    pub step: Iu,
    /// Is field and graphic text exempt from the off-grid rule?
    ///
    /// It must be: KiCad's own autoplacement puts fields on arbitrary units, so
    /// a blanket rule would fail KiCad's own output.
    pub exempt_text: bool,
}

/// Format policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Formats {
    /// The newest stamp kicli will write. A file above it is refused.
    pub max_schematic_version: FormatVersion,
}

/// View settings.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct View {
    /// Above this size, a view emits an index and per-sheet summaries instead
    /// of the whole project.
    pub max_bytes: usize,
}

/// External tool locations.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Tools {
    /// Where `kicad-cli` is, when it is not on the path.
    pub kicad_cli_path: Option<String>,
}

/// Router settings, and the weights the score reads with it.
///
/// Every weight is a whole number, because the router's cost is an `i64` and
/// there is no floating point anywhere in it. A weight is a cost, so none of
/// them may be negative: a negative term would make a longer route cheaper and
/// would break the search's own arithmetic.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Routing {
    /// The distance above which a connection is proposed as a pair of labels
    /// rather than drawn as a wire.
    ///
    /// **One knob, read twice.** The router decides with it, and the long-wire
    /// style rule judges with it. A router that draws at one distance while the
    /// rule penalises at another argues with itself.
    pub label_threshold: Iu,
    /// The cost of one grid step of wire. The base unit.
    pub w_len: i64,
    /// The cost of one corner.
    ///
    /// The measured median segment is five grid steps, so a corner must cost
    /// more than a modest detour or the router zig-zags.
    pub w_turn: i64,
    /// The cost of crossing another net, which is the most visible defect.
    pub w_cross: i64,
    /// The cost of one grid step inside a label or text box.
    ///
    /// Routing through a label is nearly as bad as a crossing.
    pub w_text: i64,
    /// The cost of one grid step within one grid step of a symbol body.
    pub w_near: i64,
    /// How far outside the two terminals the router may look for a route.
    pub margin: Iu,
    /// How far outward a U-shaped route may reach.
    pub u_max: Iu,
}

/// Inter-process settings.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ipc {
    /// How long the open-document probe may take in total.
    ///
    /// The probe must never slow the case where KiCad is not running, so this
    /// is a ceiling on the whole attempt rather than one connection.
    pub probe_timeout_ms: u64,
}

/// The whole configuration.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Config {
    /// Grid settings.
    pub grid: Grid,
    /// Format policy.
    pub formats: Formats,
    /// View settings.
    pub view: View,
    /// Router settings.
    pub routing: Routing,
    /// External tool locations.
    pub tools: Tools,
    /// Inter-process settings.
    pub ipc: Ipc,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            grid: Grid {
                step: GRID,
                exempt_text: true,
            },
            formats: Formats {
                max_schematic_version: MAX_SCHEMATIC_VERSION,
            },
            view: View { max_bytes: 32_768 },
            routing: Routing {
                label_threshold: Iu(300 * GRID.0),
                w_len: 1,
                w_turn: 6,
                w_cross: 20,
                w_text: 12,
                w_near: 2,
                margin: Iu(8 * GRID.0),
                u_max: Iu(6 * GRID.0),
            },
            tools: Tools::default(),
            ipc: Ipc {
                probe_timeout_ms: 250,
            },
        }
    }
}

/// The sections kicli knows, and the keys each one holds.
///
/// Sections later milestones read are listed here from the start, so a typo in
/// a routing weight is an error the moment the file is written rather than the
/// milestone that finally reads it.
const SECTIONS: &[(&str, &[&str])] = &[
    ("grid", &["step", "exempt_text"]),
    ("formats", &["max_schematic_version"]),
    ("view", &["max_bytes"]),
    (
        "libraries",
        &[
            "shared_path",
            "shared_nick",
            "symbols_dir",
            "footprints_dir",
            "models_dir",
        ],
    ),
    (
        "routing",
        &[
            "label_threshold",
            "w_len",
            "w_turn",
            "w_cross",
            "w_text",
            "w_near",
            "margin",
            "u_max",
        ],
    ),
    (
        "rules",
        &["default_tier2_enabled", "gate_on_tier1", "consume_erc"],
    ),
    ("render", &["max_px", "min_px_per_mm", "style", "cache"]),
    ("erc", &["severity_map"]),
    ("ipc", &["probe_timeout_ms"]),
    ("tools", &["kicad_cli_path"]),
];

/// The keys a per-rule table holds, as in `[rules."KI-XING-001"]`.
const RULE_KEYS: &[&str] = &["enabled", "weight", "free_allowance"];

impl Config {
    /// Read the configuration of a project directory.
    ///
    /// A directory with no `kicli.toml` yields the defaults.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError`] when the file does not read, is not TOML, or
    /// holds a key or section nothing reads.
    pub fn read(directory: &Path) -> Result<Self, ConfigError> {
        let path = directory.join(FILE_NAME);
        if !path.exists() {
            return Ok(Self::default());
        }
        let text = std::fs::read_to_string(&path).map_err(|error| ConfigError::Unreadable {
            path: path.display().to_string(),
            reason: error.to_string(),
        })?;
        Self::parse(&text).map_err(|error| match error {
            ConfigError::NotToml(_, reason) => {
                ConfigError::NotToml(path.display().to_string(), reason)
            }
            other => other,
        })
    }

    /// Read the configuration from text.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError`] when the text is not TOML, or holds a key or
    /// section nothing reads.
    ///
    /// # Examples
    ///
    /// ```
    /// use kicli::model::Config;
    ///
    /// let config = Config::parse("[grid]\nstep = \"25mil\"\n").expect("reads");
    /// assert_eq!(config.grid.step.0, 6_350);
    ///
    /// let typo = Config::parse("[grid]\nsetp = \"25mil\"\n");
    /// assert!(typo.is_err(), "a misspelled key is an error, not a default");
    /// ```
    pub fn parse(text: &str) -> Result<Self, ConfigError> {
        let table: toml::Table = text.parse().map_err(|error: toml::de::Error| {
            ConfigError::NotToml(String::new(), error.to_string())
        })?;
        check_keys(&table)?;

        let mut config = Self::default();
        config.read_grid(&table)?;
        config.read_routing(&table)?;
        config.read_limits(&table)?;
        Ok(config)
    }

    /// Read the two grid settings.
    fn read_grid(&mut self, table: &toml::Table) -> Result<(), ConfigError> {
        let Some(grid) = table.get("grid").and_then(Value::as_table) else {
            return Ok(());
        };
        if let Some(step) = grid.get("step") {
            self.grid.step = length("grid", "step", step)?;
        }
        if let Some(exempt) = grid.get("exempt_text") {
            self.grid.exempt_text = boolean("grid", "exempt_text", exempt)?;
        }
        Ok(())
    }

    /// Read the router's distances and weights.
    fn read_routing(&mut self, table: &toml::Table) -> Result<(), ConfigError> {
        let routing = &mut self.routing;
        for (key, field) in [
            ("label_threshold", &mut routing.label_threshold),
            ("margin", &mut routing.margin),
            ("u_max", &mut routing.u_max),
        ] {
            let Some(value) = section_value(table, "routing", key) else {
                continue;
            };
            *field = distance("routing", key, value)?;
        }
        for (key, field) in [
            ("w_len", &mut routing.w_len),
            ("w_turn", &mut routing.w_turn),
            ("w_cross", &mut routing.w_cross),
            ("w_text", &mut routing.w_text),
            ("w_near", &mut routing.w_near),
        ] {
            let Some(value) = section_value(table, "routing", key) else {
                continue;
            };
            *field = weight("routing", key, value)?;
        }
        Ok(())
    }

    /// Read the ceilings, budgets and tool locations.
    fn read_limits(&mut self, table: &toml::Table) -> Result<(), ConfigError> {
        if let Some(stamp) = section_value(table, "formats", "max_schematic_version") {
            let stamp = integer("formats", "max_schematic_version", stamp)?;
            let stamp = u32::try_from(stamp).map_err(|_| ConfigError::WrongType {
                section: "formats".to_owned(),
                key: "max_schematic_version".to_owned(),
                expected: "a date stamp such as 20260306".to_owned(),
            })?;
            self.formats.max_schematic_version = FormatVersion::new(stamp);
        }
        if let Some(max) = section_value(table, "view", "max_bytes") {
            let max = integer("view", "max_bytes", max)?;
            self.view.max_bytes = usize::try_from(max).map_err(|_| ConfigError::WrongType {
                section: "view".to_owned(),
                key: "max_bytes".to_owned(),
                expected: "a size in bytes".to_owned(),
            })?;
        }
        if let Some(path) = section_value(table, "tools", "kicad_cli_path") {
            self.tools.kicad_cli_path = Some(string("tools", "kicad_cli_path", path)?);
        }
        if let Some(timeout) = section_value(table, "ipc", "probe_timeout_ms") {
            let timeout = integer("ipc", "probe_timeout_ms", timeout)?;
            self.ipc.probe_timeout_ms =
                u64::try_from(timeout).map_err(|_| ConfigError::WrongType {
                    section: "ipc".to_owned(),
                    key: "probe_timeout_ms".to_owned(),
                    expected: "a count of milliseconds".to_owned(),
                })?;
        }
        Ok(())
    }
}

/// One value of one section, when the file has it.
fn section_value<'a>(table: &'a toml::Table, section: &str, key: &str) -> Option<&'a Value> {
    table.get(section)?.as_table()?.get(key)
}

/// Refuse any section or key nothing reads.
fn check_keys(table: &toml::Table) -> Result<(), ConfigError> {
    for (name, value) in table {
        let Some((_, keys)) = SECTIONS.iter().find(|(section, _)| section == name) else {
            return Err(ConfigError::UnknownSection(name.clone()));
        };
        let Some(section) = value.as_table() else {
            return Err(ConfigError::WrongType {
                section: name.clone(),
                key: String::new(),
                expected: "a section".to_owned(),
            });
        };
        for key in section.keys() {
            if keys.contains(&key.as_str()) {
                continue;
            }
            // [rules."KI-XING-001"] holds one table per rule.
            if name == "rules" {
                check_rule_table(name, key, &section[key])?;
                continue;
            }
            return Err(ConfigError::UnknownKey {
                section: name.clone(),
                key: key.clone(),
            });
        }
    }
    Ok(())
}

/// Check one `[rules."KI-..."]` table.
fn check_rule_table(section: &str, name: &str, value: &Value) -> Result<(), ConfigError> {
    let Some(table) = value.as_table() else {
        return Err(ConfigError::UnknownKey {
            section: section.to_owned(),
            key: name.to_owned(),
        });
    };
    if !name.starts_with("KI-") {
        return Err(ConfigError::UnknownKey {
            section: section.to_owned(),
            key: name.to_owned(),
        });
    }
    for key in table.keys() {
        if !RULE_KEYS.contains(&key.as_str()) {
            return Err(ConfigError::UnknownKey {
                section: format!("{section}.{name}"),
                key: key.clone(),
            });
        }
    }
    Ok(())
}

/// Read a length, in millimetres, mils, or whole grid steps.
///
/// A bare number is millimetres, matching how KiCad writes coordinates.
fn length(section: &str, key: &str, value: &Value) -> Result<Iu, ConfigError> {
    let text = match value {
        Value::String(text) => text.clone(),
        Value::Integer(number) => number.to_string(),
        Value::Float(number) => number.to_string(),
        _ => {
            return Err(ConfigError::WrongType {
                section: section.to_owned(),
                key: key.to_owned(),
                expected: "a length such as \"50mil\", \"1.27mm\" or \"8G\"".to_owned(),
            });
        }
    };
    let not_a_length = || ConfigError::NotALength {
        section: section.to_owned(),
        key: key.to_owned(),
        value: text.clone(),
    };
    let trimmed = text.trim();
    let (number, scale) = if let Some(rest) = trimmed.strip_suffix("mil") {
        (rest, Scale::Mils)
    } else if let Some(rest) = trimmed.strip_suffix("mm") {
        (rest, Scale::Millimetres)
    } else if let Some(rest) = trimmed.strip_suffix('G') {
        (rest, Scale::GridSteps)
    } else {
        (trimmed, Scale::Millimetres)
    };
    let number: f64 = number.trim().parse().map_err(|_| not_a_length())?;
    let units = match scale {
        // A mil is 25.4 micrometres, which is 254 internal units exactly.
        Scale::Mils => number * 254.0,
        Scale::Millimetres => number * f64::from(crate::geometry::UNITS_PER_MM),
        Scale::GridSteps => number * f64::from(GRID.0),
    };
    if !units.is_finite() || units.abs() > f64::from(i32::MAX) {
        return Err(not_a_length());
    }
    #[allow(clippy::cast_possible_truncation)] // bounded by the check above
    Ok(Iu(units.round() as i32))
}

/// Read a length that may not be negative.
///
/// A negative routing distance describes nothing: a window cannot be inflated
/// inwards, and a threshold below zero would propose labels for every route.
fn distance(section: &str, key: &str, value: &Value) -> Result<Iu, ConfigError> {
    let measured = length(section, key, value)?;
    if measured.0 < 0 {
        return Err(ConfigError::WrongType {
            section: section.to_owned(),
            key: key.to_owned(),
            expected: "a distance of zero or more".to_owned(),
        });
    }
    Ok(measured)
}

/// Read a cost weight: a whole number, never negative.
///
/// A negative weight would make a longer route cheaper than a shorter one and
/// would break the search's own arithmetic, so it is refused where it is read
/// rather than found later as a route nobody can explain.
fn weight(section: &str, key: &str, value: &Value) -> Result<i64, ConfigError> {
    let number = integer(section, key, value)?;
    if number < 0 {
        return Err(ConfigError::WrongType {
            section: section.to_owned(),
            key: key.to_owned(),
            expected: "a cost of zero or more".to_owned(),
        });
    }
    Ok(number)
}

/// Which unit a length was written in.
enum Scale {
    Mils,
    Millimetres,
    GridSteps,
}

fn boolean(section: &str, key: &str, value: &Value) -> Result<bool, ConfigError> {
    value.as_bool().ok_or_else(|| ConfigError::WrongType {
        section: section.to_owned(),
        key: key.to_owned(),
        expected: "true or false".to_owned(),
    })
}

fn integer(section: &str, key: &str, value: &Value) -> Result<i64, ConfigError> {
    value.as_integer().ok_or_else(|| ConfigError::WrongType {
        section: section.to_owned(),
        key: key.to_owned(),
        expected: "a whole number".to_owned(),
    })
}

fn string(section: &str, key: &str, value: &Value) -> Result<String, ConfigError> {
    value
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| ConfigError::WrongType {
            section: section.to_owned(),
            key: key.to_owned(),
            expected: "text".to_owned(),
        })
}

#[cfg(test)]
mod tests {
    use super::{Config, ConfigError};
    use crate::geometry::{GRID, Iu};

    #[test]
    fn an_absent_file_gives_the_documented_defaults() {
        let config = Config::read(std::path::Path::new("/nonexistent")).expect("defaults");
        assert_eq!(config, Config::default());
        assert_eq!(config.grid.step, Iu(12_700));
        assert_eq!(config.view.max_bytes, 32_768);
        assert_eq!(config.ipc.probe_timeout_ms, 250);
        assert!(config.grid.exempt_text);
    }

    #[test]
    fn lengths_read_in_mils_millimetres_and_grid_steps() {
        let config = Config::parse("[grid]\nstep = \"50mil\"\n").expect("reads");
        assert_eq!(config.grid.step, Iu(12_700));
        let config = Config::parse("[grid]\nstep = \"1.27mm\"\n").expect("reads");
        assert_eq!(config.grid.step, Iu(12_700));
        let config = Config::parse("[grid]\nstep = \"2G\"\n").expect("reads");
        assert_eq!(config.grid.step, Iu(25_400));
        let config = Config::parse("[grid]\nstep = 1.27\n").expect("reads");
        assert_eq!(config.grid.step, Iu(12_700));
    }

    #[test]
    fn a_misspelled_key_names_itself_and_its_section() {
        let error = Config::parse("[grid]\nsetp = \"50mil\"\n").expect_err("is an error");
        match error {
            ConfigError::UnknownKey { section, key } => {
                assert_eq!(section, "grid");
                assert_eq!(key, "setp");
            }
            other => panic!("expected an unknown key, got {other}"),
        }
    }

    #[test]
    fn a_section_nothing_reads_is_an_error() {
        let error = Config::parse("[grod]\nstep = \"50mil\"\n").expect_err("is an error");
        assert!(matches!(error, ConfigError::UnknownSection(name) if name == "grod"));
    }

    #[test]
    fn config_reads_every_routing_key() {
        // Each key parses to the value the specification documents.
        let defaults = Config::default().routing;
        // 300 grid steps and 381 mm are the same length. The two forms are
        // asserted against each other because they once were not: `Iu(381_000)`
        // is internal units, and a gloss that read that constant as millimetres
        // put the documented default a factor of ten from the code's.
        assert_eq!(defaults.label_threshold, Iu(300 * GRID.0), "300 grid steps");
        assert_eq!(
            defaults.label_threshold,
            Iu::from_millimetres_text("381").expect("381 mm is a length"),
            "which is 381 mm exactly, the gloss spec/SPEC.md \u{00a7}9 publishes"
        );
        assert_eq!(defaults.margin, Iu(101_600), "8 grid steps");
        assert_eq!(defaults.u_max, Iu(76_200), "6 grid steps");
        assert_eq!(
            (
                defaults.w_len,
                defaults.w_turn,
                defaults.w_cross,
                defaults.w_text,
                defaults.w_near
            ),
            (1, 6, 20, 12, 2)
        );

        let config = Config::parse(concat!(
            "[routing]\n",
            "label_threshold = \"20G\"\n",
            "margin = \"25.4mm\"\n",
            "u_max = \"400mil\"\n",
            "w_len = 2\nw_turn = 9\nw_cross = 30\nw_text = 15\nw_near = 0\n",
        ))
        .expect("every routing key reads");
        // A distance reads in grid steps, millimetres or mils alike.
        assert_eq!(config.routing.label_threshold, Iu(254_000));
        assert_eq!(config.routing.margin, Iu(254_000));
        assert_eq!(config.routing.u_max, Iu(101_600));
        assert_eq!(config.routing.w_len, 2);
        assert_eq!(config.routing.w_turn, 9);
        assert_eq!(config.routing.w_cross, 30);
        assert_eq!(config.routing.w_text, 15);
        assert_eq!(config.routing.w_near, 0);

        // A misspelled key is still an error rather than a silent default.
        let error =
            Config::parse("[routing]\nlabel_treshold = \"20G\"\n").expect_err("a typo is an error");
        assert!(matches!(error, ConfigError::UnknownKey { key, .. } if key == "label_treshold"));
    }

    #[test]
    fn a_cost_that_would_break_the_search_is_refused() {
        // A negative weight makes a longer route cheaper. A negative distance
        // describes nothing. Both are refused where they are read.
        for text in [
            "[routing]\nw_turn = -1\n",
            "[routing]\nmargin = \"-8G\"\n",
            "[routing]\nlabel_threshold = \"-1mm\"\n",
        ] {
            let error = Config::parse(text).expect_err("a negative cost is an error");
            assert!(
                matches!(&error, ConfigError::WrongType { expected, .. } if expected.contains("zero or more")),
                "{error}"
            );
        }
        // The control: the same keys at zero are accepted.
        let config = Config::parse("[routing]\nw_turn = 0\nmargin = \"0mm\"\n")
            .expect("zero is a cost like any other");
        assert_eq!(config.routing.w_turn, 0);
        assert_eq!(config.routing.margin, Iu(0));
    }

    #[test]
    fn later_milestones_sections_validate_now() {
        // Nothing reads these yet. A typo in one is still an error today.
        Config::parse(concat!(
            "[routing]\nw_turn = 6\nmargin = \"8G\"\n",
            "[rules]\ngate_on_tier1 = true\n",
            "[rules.\"KI-XING-001\"]\nenabled = true\nweight = 1.0\nfree_allowance = 2\n",
            "[libraries]\nshared_path = \"../shared\"\n",
            "[render]\nmax_px = 1600\n",
        ))
        .expect("a complete file reads");

        let error = Config::parse("[routing]\nw_trun = 6\n").expect_err("a typo is an error");
        assert!(matches!(error, ConfigError::UnknownKey { key, .. } if key == "w_trun"));

        let error = Config::parse("[rules.\"KI-XING-001\"]\nwieght = 1.0\n")
            .expect_err("a typo inside a rule table is an error");
        assert!(matches!(error, ConfigError::UnknownKey { key, .. } if key == "wieght"));

        let error =
            Config::parse("[rules.\"XING\"]\nweight = 1.0\n").expect_err("a rule id is checked");
        assert!(matches!(error, ConfigError::UnknownKey { key, .. } if key == "XING"));
    }

    #[test]
    fn a_value_of_the_wrong_kind_says_what_it_needs() {
        let error = Config::parse("[view]\nmax_bytes = \"lots\"\n").expect_err("is an error");
        assert!(matches!(error, ConfigError::WrongType { key, .. } if key == "max_bytes"));
        let error = Config::parse("[grid]\nstep = \"50 furlongs\"\n").expect_err("is an error");
        assert!(matches!(error, ConfigError::NotALength { key, .. } if key == "step"));
    }

    #[test]
    fn the_version_ceiling_can_be_raised() {
        let config = Config::parse("[formats]\nmax_schematic_version = 20260803\n").expect("reads");
        assert_eq!(config.formats.max_schematic_version.stamp(), 20_260_803);
    }
}
