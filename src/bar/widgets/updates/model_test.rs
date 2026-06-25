use super::*;

#[test]
fn parses_checkupdates_rows() {
    let packages = parse_checkupdates_output(
        "linux 6.9.1.arch1-1 -> 6.9.2.arch1-1\nmesa 24.0.1-1 -> 24.0.2-1\n",
    );

    assert_eq!(
        packages,
        vec![
            UpdatePackage {
                name: "linux".to_string(),
                old_version: "6.9.1.arch1-1".to_string(),
                new_version: "6.9.2.arch1-1".to_string(),
                severity: UpdateSeverity::Normal,
            },
            UpdatePackage {
                name: "mesa".to_string(),
                old_version: "24.0.1-1".to_string(),
                new_version: "24.0.2-1".to_string(),
                severity: UpdateSeverity::Normal,
            },
        ]
    );
}

#[test]
fn classifies_package_severity_by_glob() {
    let matcher =
        UpdateSeverityMatcher::new(&["linux*".to_string()], &["mesa".to_string()]).unwrap();

    assert_eq!(matcher.severity_for("linux"), UpdateSeverity::Critical);
    assert_eq!(
        matcher.severity_for("linux-headers"),
        UpdateSeverity::Critical
    );
    assert_eq!(matcher.severity_for("mesa"), UpdateSeverity::Warning);
    assert_eq!(matcher.severity_for("glibc"), UpdateSeverity::Normal);
}

#[test]
fn sorts_by_severity_and_preserves_group_order() {
    let mut packages = vec![
        package("first-normal", UpdateSeverity::Normal),
        package("first-critical", UpdateSeverity::Critical),
        package("first-warning", UpdateSeverity::Warning),
        package("second-critical", UpdateSeverity::Critical),
        package("second-normal", UpdateSeverity::Normal),
    ];

    sort_packages_by_severity(&mut packages);

    let names = packages
        .into_iter()
        .map(|package| package.name)
        .collect::<Vec<_>>();

    assert_eq!(
        names,
        vec![
            "first-critical",
            "second-critical",
            "first-warning",
            "first-normal",
            "second-normal",
        ]
    );
}

fn package(name: &str, severity: UpdateSeverity) -> UpdatePackage {
    UpdatePackage {
        name: name.to_string(),
        old_version: "1-1".to_string(),
        new_version: "1-2".to_string(),
        severity,
    }
}
