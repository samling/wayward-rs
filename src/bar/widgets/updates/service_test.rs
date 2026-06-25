use super::super::model::{UpdatePackage, UpdateSeverity};
use super::*;

#[test]
fn refreshing_snapshot_preserves_latest_packages() {
    let latest = UpdatesSnapshot {
        packages: vec![UpdatePackage {
            name: "linux".to_string(),
            old_version: "6.9.1.arch1-1".to_string(),
            new_version: "6.9.2.arch1-1".to_string(),
            severity: UpdateSeverity::Critical,
        }],
        last_error: None,
        refreshing: false,
    };

    let snapshot = refreshing_snapshot(Some(&latest));

    assert!(snapshot.refreshing);
    assert_eq!(snapshot.packages, latest.packages);
}
