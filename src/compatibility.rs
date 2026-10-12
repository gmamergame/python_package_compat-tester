use crate::dependencies::{Dependency, dependency_applies_to_python_version};
use crate::pypi::PackageMetadata;
use crate::pypi::PackageRelease;
use pep508_rs::MarkerEnvironment;

pub fn select_package_release<'a>(
    metadata: &'a PackageMetadata,
    specifier: Option<&str>,
) -> Result<Option<&'a PackageRelease>, String> {
    let validated_specifier = specifier
        .map(|specifier| parse_package_specifier(specifier).map(|parsed| (specifier, parsed)))
        .transpose()?;

    for release in &metadata.releases {
        release
            .version
            .parse::<pep440_rs::Version>()
            .map_err(|error| format!("Invalid package version '{}': {error}", release.version))?;
    }

    let matching = metadata.releases.iter().filter(|release| {
        validated_specifier
            .as_ref()
            .map(|(_, specifiers)| {
                package_version_satisfies_validated(&release.version, specifiers)
            })
            .unwrap_or(true)
    });

    let stable = matching
        .clone()
        .filter(|release| !is_prerelease(&release.version))
        .max_by(|left, right| compare_versions(&left.version, &right.version));

    Ok(stable
        .or_else(|| matching.max_by(|left, right| compare_versions(&left.version, &right.version))))
}

pub fn compatible_python_versions(releases: &[PackageRelease]) -> Vec<String> {
    let mut compatible_versions = vec![
        "3.7".to_owned(),
        "3.8".to_owned(),
        "3.9".to_owned(),
        "3.10".to_owned(),
        "3.11".to_owned(),
        "3.12".to_owned(),
        "3.13".to_owned(),
        "3.14".to_owned(),
        "3.15".to_owned(),
    ];

    for release in releases {
        if let Some(requirement) = &release.requires_python {
            compatible_versions
                .retain(|python_version| python_version_satisfies(python_version, requirement));
        }

        if compatible_versions.is_empty() {
            break;
        }
    }

    compatible_versions
}

pub fn compatible_python_versions_for_environment(
    dependencies: &[Dependency],
    releases: &[Option<PackageRelease>],
    environment: &MarkerEnvironment,
) -> Result<Vec<String>, String> {
    if dependencies.len() != releases.len() {
        return Err("Each dependency must have a corresponding selected release".to_owned());
    }

    let candidates = [
        "3.7", "3.8", "3.9", "3.10", "3.11", "3.12", "3.13", "3.14", "3.15",
    ];

    let mut compatible_versions = Vec::new();

    for candidate in candidates {
        let mut applicable_releases = Vec::new();
        let mut missing_release = false;

        for (dependency, release) in dependencies.iter().zip(releases) {
            if !dependency_applies_to_python_version(dependency, environment, candidate)? {
                continue;
            }

            match release {
                Some(release) => applicable_releases.push(release.clone()),
                None => {
                    missing_release = true;
                    break;
                }
            }
        }

        if missing_release {
            continue;
        }

        if compatible_python_versions(&applicable_releases)
            .iter()
            .any(|version| version == candidate)
        {
            compatible_versions.push(candidate.to_owned());
        }
    }

    Ok(compatible_versions)
}

fn python_version_satisfies(version: &str, requirement: &str) -> bool {
    requirement
        .split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .all(|part| satisfies_python_specifier(version, part))
}

fn satisfies_python_specifier(version: &str, specifier: &str) -> bool {
    let operators = [">=", "<=", "==", "!=", ">", "<"];

    let (operator, required) = operators
        .iter()
        .find_map(|operator| {
            specifier
                .strip_prefix(operator)
                .map(|required| (*operator, required.trim()))
        })
        .unwrap_or(("==", specifier.trim()));

    let (Ok(version), Ok(required)) = (
        version.parse::<pep440_rs::Version>(),
        required.parse::<pep440_rs::Version>(),
    ) else {
        return false;
    };

    let comparison = version.cmp(&required);

    match operator {
        ">=" => comparison != std::cmp::Ordering::Less,
        "<=" => comparison != std::cmp::Ordering::Greater,
        "==" => comparison == std::cmp::Ordering::Equal,
        "!=" => comparison != std::cmp::Ordering::Equal,
        ">" => comparison == std::cmp::Ordering::Greater,
        "<" => comparison == std::cmp::Ordering::Less,
        _ => false,
    }
}

pub(crate) fn is_prerelease(version: &str) -> bool {
    version
        .parse::<pep440_rs::Version>()
        .is_ok_and(|parsed| parsed.any_prerelease())
}

pub(crate) fn compare_versions(left: &str, right: &str) -> std::cmp::Ordering {
    match (
        left.parse::<pep440_rs::Version>(),
        right.parse::<pep440_rs::Version>(),
    ) {
        (Ok(left), Ok(right)) => left.cmp(&right),
        _ => left.cmp(right),
    }
}

#[cfg(test)]
pub(crate) fn package_version_satisfies(version: &str, requirement: &str) -> Result<bool, String> {
    let specifiers = parse_package_specifier(requirement)?;
    Ok(package_version_satisfies_validated(version, &specifiers))
}

fn parse_package_specifier(requirement: &str) -> Result<pep440_rs::VersionSpecifiers, String> {
    if requirement.trim().is_empty() {
        return Err("Version specifier cannot be empty".to_owned());
    }

    requirement
        .parse::<pep440_rs::VersionSpecifiers>()
        .map_err(|error| format!("Invalid package version specifier '{requirement}': {error}"))
}

fn package_version_satisfies_validated(
    version: &str,
    specifiers: &pep440_rs::VersionSpecifiers,
) -> bool {
    let Ok(parsed_version) = version.parse::<pep440_rs::Version>() else {
        return false;
    };

    specifiers
        .iter()
        .all(|specifier| specifier.contains(&parsed_version))
}
