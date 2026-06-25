use super::*;

#[test]
fn version_text_shows_old_and_new_versions() {
    let package = UpdatePackage {
        name: "linux".to_string(),
        old_version: "6.9.1.arch1-1".to_string(),
        new_version: "6.9.2.arch1-1".to_string(),
        severity: UpdateSeverity::Normal,
    };

    assert_eq!(version_text(&package), "6.9.1.arch1-1 -> 6.9.2.arch1-1");
}
