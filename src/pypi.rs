use crate::package_name::normalize_package_name;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageMetadata {
    pub name: String,
    pub latest_version: String,
    pub requires_python: Option<String>,
    pub releases: Vec<PackageRelease>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageRelease {
    pub version: String,
    pub requires_python: Option<String>,
}

pub fn fetch_pypi_metadata(package_name: &str) -> Result<PackageMetadata, String> {
    let normalized_name = normalize_package_name(package_name);

    let url = format!("https://pypi.org/pypi/{normalized_name}/json");

    test_debug!("fetch_pypi_metadata: requesting '{url}'");

    let response = reqwest::blocking::get(&url)
        .map_err(|error| format!("failed to query PyPI for '{normalized_name}': {error}"))?;

    if !response.status().is_success() {
        return Err(format!(
            "PyPI returned HTTP {} for package '{normalized_name}'",
            response.status()
        ));
    }

    let body = response
        .text()
        .map_err(|error| format!("failed to read PyPI response: {error}"))?;

    parse_pypi_metadata(&body)
}

#[derive(Debug, serde::Deserialize)]
struct PyPiResponse {
    info: PyPiInfo,
    releases: std::collections::HashMap<String, Vec<PyPiReleaseFile>>,
}

#[derive(Debug, serde::Deserialize)]
struct PyPiInfo {
    name: String,
    version: String,
    requires_python: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct PyPiReleaseFile {
    requires_python: Option<String>,
}

pub(crate) fn parse_pypi_metadata(contents: &str) -> Result<PackageMetadata, String> {
    let metadata: PyPiResponse = serde_json::from_str(contents)
        .map_err(|error| format!("failed to parse PyPI metadata: {error}"))?;

    let mut releases = metadata
        .releases
        .into_iter()
        .map(|(version, files)| {
            let requires_python = files
                .into_iter()
                .filter_map(|file| file.requires_python)
                .next();

            PackageRelease {
                version,
                requires_python,
            }
        })
        .collect::<Vec<_>>();

    releases.sort_by(|left, right| left.version.cmp(&right.version));

    Ok(PackageMetadata {
        name: normalize_package_name(&metadata.info.name),
        latest_version: metadata.info.version,
        requires_python: metadata.info.requires_python,
        releases,
    })
}
