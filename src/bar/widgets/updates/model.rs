use globset::{Glob, GlobSet, GlobSetBuilder};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum UpdateSeverity {
    Critical,
    Warning,
    Normal,
}

impl UpdateSeverity {
    fn sort_rank(self) -> u8 {
        match self {
            Self::Critical => 0,
            Self::Warning => 1,
            Self::Normal => 2,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct UpdatePackage {
    pub(crate) name: String,
    pub(crate) old_version: String,
    pub(crate) new_version: String,
    pub(crate) severity: UpdateSeverity,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct UpdatesSnapshot {
    pub(crate) packages: Vec<UpdatePackage>,
    pub(crate) last_error: Option<String>,
    pub(crate) refreshing: bool,
}

#[derive(Debug)]
pub(super) struct UpdateSeverityMatcher {
    critical: GlobSet,
    warning: GlobSet,
}

impl UpdateSeverityMatcher {
    pub(super) fn new(
        critical_patterns: &[String],
        warning_patterns: &[String],
    ) -> Result<Self, globset::Error> {
        Ok(Self {
            critical: build_glob_set(critical_patterns)?,
            warning: build_glob_set(warning_patterns)?,
        })
    }

    pub(super) fn severity_for(&self, package_name: &str) -> UpdateSeverity {
        if self.critical.is_match(package_name) {
            UpdateSeverity::Critical
        } else if self.warning.is_match(package_name) {
            UpdateSeverity::Warning
        } else {
            UpdateSeverity::Normal
        }
    }

    pub(super) fn apply(&self, packages: &mut [UpdatePackage]) {
        for package in packages {
            package.severity = self.severity_for(&package.name)
        }
    }
}

pub(super) fn parse_checkupdates_output(output: &str) -> Vec<UpdatePackage> {
    output.lines().filter_map(parse_checkupdates_line).collect()
}

fn parse_checkupdates_line(line: &str) -> Option<UpdatePackage> {
    let mut fields = line.split_whitespace();

    let name = fields.next()?;
    let old_version = fields.next()?;

    if fields.next()? != "->" {
        return None;
    }

    let new_version = fields.next()?;

    Some(UpdatePackage {
        name: name.to_string(),
        old_version: old_version.to_string(),
        new_version: new_version.to_string(),
        severity: UpdateSeverity::Normal,
    })
}

pub(super) fn sort_packages_by_severity(packages: &mut [UpdatePackage]) {
    packages.sort_by_key(|package| package.severity.sort_rank());
}

fn build_glob_set(patterns: &[String]) -> Result<GlobSet, globset::Error> {
    let mut builder = GlobSetBuilder::new();

    for pattern in patterns {
        builder.add(Glob::new(pattern)?);
    }

    builder.build()
}

#[cfg(test)]
mod tests {
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
}
