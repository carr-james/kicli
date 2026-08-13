//! Reading bus aliases and ERC severities out of a project file.

use kicli::model::read_project;
use std::path::Path;

#[test]
fn kicad_pro_read() {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/project/minimal.kicad_pro");
    let text = std::fs::read_to_string(path).expect("fixture is readable");
    let project = read_project(&text).expect("reads");

    assert_eq!(project.bus_aliases.len(), 1);
    assert_eq!(project.bus_aliases[0].name, "DPHY");
    assert_eq!(
        project.bus_aliases[0].members,
        ["D0_N", "D0_P", "CLK_N", "CLK_P"]
    );

    assert_eq!(
        project
            .erc_severities
            .get("four_way_junction")
            .map(String::as_str),
        Some("ignore")
    );
    assert_eq!(
        project
            .erc_severities
            .get("pin_not_connected")
            .map(String::as_str),
        Some("error")
    );
}
