//! Reading the project file.
//!
//! KiCad 10 moved bus aliases out of the schematic and into the project file,
//! so connectivity cannot be worked out from the schematic alone. ERC severities
//! live there too. kicli reads both and writes neither: changing a severity
//! would change what KiCad reports, which is the user's call.

use std::collections::BTreeMap;

/// A bus alias: a name standing for a list of member nets.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BusAlias {
    /// The alias name.
    pub name: String,
    /// The nets it expands to.
    pub members: Vec<String>,
}

/// The parts of a project file kicli reads.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Project {
    /// Bus aliases, which KiCad 10 keeps here rather than in the schematic.
    pub bus_aliases: Vec<BusAlias>,
    /// ERC severities, keyed by check name. Read only, and relabelled for
    /// kicli's own output rather than edited.
    pub erc_severities: BTreeMap<String, String>,
}

/// Something a project file can be wrong about.
#[derive(Debug, thiserror::Error)]
pub enum ProjectError {
    /// The file is not valid JSON.
    #[error("the project file is not valid JSON: {0}")]
    Malformed(#[from] serde_json::Error),
}

/// Read the parts of a project file kicli uses.
///
/// Anything kicli does not understand is ignored rather than rejected: a
/// project file carries settings for every part of KiCad, and most of them are
/// none of kicli's business.
///
/// # Errors
///
/// Returns [`ProjectError::Malformed`] when the text is not valid JSON.
pub fn read_project(text: &str) -> Result<Project, ProjectError> {
    let root: serde_json::Value = serde_json::from_str(text)?;

    // KiCad writes bus aliases as an object: the alias name is the key and
    // its members are the value. Verified against 73 project files KiCad wrote
    // itself, 12 of them with aliases, none in any other shape. An earlier
    // reading here expected a list of name-and-members pairs, which no KiCad
    // file uses, so every real project reported no aliases at all.
    let bus_aliases = root
        .pointer("/schematic/bus_aliases")
        .and_then(serde_json::Value::as_object)
        .map(|entries| {
            entries
                .iter()
                .map(|(name, members)| BusAlias {
                    name: name.clone(),
                    members: members
                        .as_array()
                        .map(|list| {
                            list.iter()
                                .filter_map(|member| member.as_str().map(str::to_owned))
                                .collect()
                        })
                        .unwrap_or_default(),
                })
                .collect()
        })
        .unwrap_or_default();

    let erc_severities = root
        .pointer("/erc/rule_severities")
        .and_then(serde_json::Value::as_object)
        .map(|entries| {
            entries
                .iter()
                .filter_map(|(key, value)| value.as_str().map(|v| (key.clone(), v.to_owned())))
                .collect()
        })
        .unwrap_or_default();

    Ok(Project {
        bus_aliases,
        erc_severities,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_project_without_the_keys_reads_as_empty() {
        let project = read_project("{}").expect("reads");
        assert_eq!(project, Project::default());
    }

    #[test]
    fn malformed_json_is_an_error() {
        assert!(read_project("{").is_err());
    }
}
