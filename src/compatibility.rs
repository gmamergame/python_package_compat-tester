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

    let comparison = compare_versions(version, required);

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
    let parsed = parse_package_version(version);
    parsed.prerelease.is_some()
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct ParsedPackageVersion {
    release: Vec<u64>,
    prerelease: Option<Prerelease>,
    postrelease: Option<u64>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Prerelease {
    Alpha(u64),
    Beta(u64),
    Rc(u64),
}

pub(crate) fn compare_versions(left: &str, right: &str) -> std::cmp::Ordering {
    let left_version = parse_package_version(left);
    let right_version = parse_package_version(right);

    match left_version.release.cmp(&right_version.release) {
        std::cmp::Ordering::Equal => {
            match (&left_version.prerelease, &right_version.prerelease) {
                (None, None) => {}
                (None, Some(_)) => return std::cmp::Ordering::Greater,
                (Some(_), None) => return std::cmp::Ordering::Less,
                (Some(left), Some(right)) => {
                    let ordering = left.cmp(right);

                    if ordering != std::cmp::Ordering::Equal {
                        return ordering;
                    }
                }
            }

            left_version.postrelease.cmp(&right_version.postrelease)
        }
        ordering => ordering,
    }
}

fn parse_package_version(version: &str) -> ParsedPackageVersion {
    let version = version.trim().to_ascii_lowercase();

    let (base_version, postrelease) = if let Some(index) = version.find(".post") {
        let post_number = version[index + 5..].parse::<u64>().unwrap_or(0);

        (&version[..index], Some(post_number))
    } else {
        (version.as_str(), None)
    };

    let (release_part, prerelease) = if let Some(index) = base_version.find("rc") {
        (
            &base_version[..index],
            Some(Prerelease::Rc(
                base_version[index + 2..].parse::<u64>().unwrap_or(0),
            )),
        )
    } else if let Some(index) = base_version.find('a') {
        (
            &base_version[..index],
            Some(Prerelease::Alpha(
                base_version[index + 1..].parse::<u64>().unwrap_or(0),
            )),
        )
    } else if let Some(index) = base_version.find('b') {
        (
            &base_version[..index],
            Some(Prerelease::Beta(
                base_version[index + 1..].parse::<u64>().unwrap_or(0),
            )),
        )
    } else {
        (base_version, None)
    };

    let mut release = release_part
        .split('.')
        .map(|part| part.parse::<u64>().unwrap_or(0))
        .collect::<Vec<_>>();

    while release.len() < 3 {
        release.push(0);
    }

    ParsedPackageVersion {
        release,
        prerelease,
        postrelease,
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
        .all(|specifier| match specifier.operator() {
            pep440_rs::Operator::EqualStar | pep440_rs::Operator::NotEqualStar => {
                satisfies_package_specifier(version, &specifier.to_string())
            }
            _ => specifier.contains(&parsed_version),
        })
}

fn satisfies_package_specifier(version: &str, specifier: &str) -> bool {
    let operators = ["~=", ">=", "<=", "==", "!=", ">", "<"];

    let Some((operator, required)) = operators.iter().find_map(|operator| {
        specifier
            .strip_prefix(operator)
            .map(|required| (*operator, required.trim()))
    }) else {
        return false;
    };

    if required.is_empty() {
        return false;
    }

    if required.starts_with('<')
        || required.starts_with('>')
        || required.starts_with('=')
        || required.starts_with('!')
        || required.starts_with('~')
    {
        return false;
    }

    if (operator == "==" || operator == "!=") && required.ends_with(".*") {
        let prefix = &required[..required.len() - 2];

        let matches = version == prefix
            || version
                .strip_prefix(prefix)
                .is_some_and(|suffix| suffix.starts_with('.'));

        return if operator == "==" { matches } else { !matches };
    }
    let comparison = compare_versions(version, required);

    match operator {
        "~=" => {
            let parts: Vec<&str> = required.split('.').collect();

            if parts.len() < 2 || parts.iter().any(|part| part.parse::<u64>().is_err()) {
                return false;
            }

            let mut upper_parts: Vec<u64> = parts[..parts.len() - 1]
                .iter()
                .map(|part| part.parse::<u64>().unwrap())
                .collect();

            let Some(last) = upper_parts.last_mut() else {
                return false;
            };

            let Some(next) = last.checked_add(1) else {
                return false;
            };

            *last = next;
            upper_parts.push(0);

            let upper_bound = upper_parts
                .iter()
                .map(u64::to_string)
                .collect::<Vec<_>>()
                .join(".");

            comparison != std::cmp::Ordering::Less
                && compare_versions(version, &upper_bound) == std::cmp::Ordering::Less
        }
        ">=" => comparison != std::cmp::Ordering::Less,
        "<=" => comparison != std::cmp::Ordering::Greater,
        "==" => comparison == std::cmp::Ordering::Equal,
        "!=" => comparison != std::cmp::Ordering::Equal,
        ">" => comparison == std::cmp::Ordering::Greater,
        "<" => comparison == std::cmp::Ordering::Less,
        _ => false,
    }
}
