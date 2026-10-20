use crate::package_name::normalize_package_name;
use pep440_rs::Version;
use pep508_rs::{MarkerEnvironment, Requirement, VerbatimUrl};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Dependency {
    pub name: String,
    pub extras: Vec<String>,
    pub specifier: Option<String>,
    pub url: Option<String>,
    pub marker: Option<String>,
    pub source: PathBuf,
}

pub fn dependency_applies_to_environment(
    dependency: &Dependency,
    environment: &MarkerEnvironment,
) -> Result<bool, String> {
    let Some(marker) = dependency.marker.as_deref() else {
        return Ok(true);
    };

    let requirement = format!("{}; {}", dependency.name, marker);
    let working_dir = dependency.source.parent().unwrap_or_else(|| Path::new("."));

    let parsed = Requirement::<VerbatimUrl>::parse(&requirement, working_dir)
        .map_err(|error| format!("Invalid environment marker: {error}"))?;

    Ok(parsed.evaluate_markers(environment, &[]))
}
#[allow(dead_code)]
pub fn dependency_applies_to_environment_with_extras(
    dependency: &Dependency,
    environment: &MarkerEnvironment,
    extras: &[pep508_rs::ExtraName],
) -> Result<bool, String> {
    let Some(marker) = dependency.marker.as_deref() else {
        return Ok(true);
    };

    let requirement = format!("{}; {}", dependency.name, marker);
    let working_dir = dependency.source.parent().unwrap_or_else(|| Path::new("."));

    let parsed = Requirement::<VerbatimUrl>::parse(&requirement, working_dir)
        .map_err(|error| format!("Invalid environment marker: {error}"))?;

    Ok(parsed.evaluate_markers(environment, extras))
}

#[allow(dead_code)]
pub fn dependency_applies_to_python_version_with_extras(
    dependency: &Dependency,
    environment: &MarkerEnvironment,
    python_version: &str,
    extras: &[pep508_rs::ExtraName],
) -> Result<bool, String> {
    let current_python_version = environment.python_version().to_string();

    let full_version = if python_version == current_python_version {
        environment.python_full_version().to_string()
    } else {
        format!("{python_version}.0")
    };

    let candidate_version = python_version
        .parse::<Version>()
        .map_err(|error| format!("Invalid candidate Python version: {error}"))?;

    let candidate_full_version = full_version
        .parse::<Version>()
        .map_err(|error| format!("Invalid candidate full Python version: {error}"))?;

    let candidate_environment = environment
        .clone()
        .with_python_version(candidate_version)
        .with_python_full_version(candidate_full_version);

    dependency_applies_to_environment_with_extras(dependency, &candidate_environment, extras)
}

pub fn dependency_applies_to_python_version(
    dependency: &Dependency,
    environment: &MarkerEnvironment,
    python_version: &str,
) -> Result<bool, String> {
    let current_python_version = environment.python_version().to_string();

    let full_version = if python_version == current_python_version {
        environment.python_full_version().to_string()
    } else {
        format!("{python_version}.0")
    };

    let candidate_version = python_version
        .parse::<Version>()
        .map_err(|error| format!("Invalid candidate Python version: {error}"))?;

    let candidate_full_version = full_version
        .parse::<Version>()
        .map_err(|error| format!("Invalid candidate full Python version: {error}"))?;

    let candidate_environment = environment
        .clone()
        .with_python_version(candidate_version)
        .with_python_full_version(candidate_full_version);

    dependency_applies_to_environment(dependency, &candidate_environment)
}

pub fn discover_dependencies(project_path: &Path) -> Result<Vec<Dependency>, String> {
    test_debug!(
        "discover_dependencies: project path '{}'",
        project_path.display()
    );
    let mut dependencies = Vec::new();

    let files = dependency_files(project_path)?;
    test_debug!(
        "discover_dependencies: found {} candidate file(s): {files:?}",
        files.len()
    );

    for path in files {
        test_debug!("discover_dependencies: reading '{}'", path.display());
        let contents = fs::read_to_string(&path)
            .map_err(|error| format!("failed to read '{}': {error}", path.display()))?;
        test_debug!(
            "discover_dependencies: contents of '{}':\n{}",
            path.display(),
            contents
        );

        if path.file_name().and_then(|name| name.to_str()) == Some("pyproject.toml") {
            test_debug!(
                "discover_dependencies: parsing '{}' as pyproject.toml",
                path.display()
            );
            discover_from_pyproject(&contents, &path, &mut dependencies)?;
        } else {
            test_debug!(
                "discover_dependencies: parsing '{}' as requirements",
                path.display()
            );
            discover_from_requirements(&contents, &path, &mut dependencies)?;
        }
    }

    dependencies.sort_by(|left, right| {
        left.name
            .cmp(&right.name)
            .then_with(|| right.specifier.cmp(&left.specifier))
            .then_with(|| left.marker.cmp(&right.marker))
            .then_with(|| left.source.cmp(&right.source))
    });
    test_debug!("discover_dependencies: final dependencies: {dependencies:#?}");
    Ok(dependencies)
}

fn dependency_files(project_path: &Path) -> Result<Vec<PathBuf>, String> {
    test_debug!("dependency_files: scanning '{}'", project_path.display());
    let mut files = Vec::new();

    let requirements = project_path.join("requirements.txt");
    if requirements.is_file() {
        test_debug!("dependency_files: found '{}'", requirements.display());
        files.push(requirements);
    } else {
        test_debug!("dependency_files: '{}' is absent", requirements.display());
    }

    let requirements_dir = project_path.join("requirements");
    if requirements_dir.is_dir() {
        test_debug!(
            "dependency_files: scanning directory '{}'",
            requirements_dir.display()
        );
        let entries = fs::read_dir(&requirements_dir)
            .map_err(|error| format!("failed to read '{}': {error}", requirements_dir.display()))?;

        for entry in entries {
            let entry = entry.map_err(|error| {
                format!(
                    "failed to inspect '{}': {error}",
                    requirements_dir.display()
                )
            })?;

            let path = entry.path();

            if path.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some("txt") {
                test_debug!(
                    "dependency_files: found requirements file '{}'",
                    path.display()
                );
                files.push(path);
            } else {
                test_debug!("dependency_files: ignoring entry '{}'", path.display());
            }
        }
    } else {
        test_debug!(
            "dependency_files: '{}' is absent",
            requirements_dir.display()
        );
    }

    let pyproject = project_path.join("pyproject.toml");
    if pyproject.is_file() {
        test_debug!("dependency_files: found '{}'", pyproject.display());
        files.push(pyproject);
    } else {
        test_debug!("dependency_files: '{}' is absent", pyproject.display());
    }

    files.sort();
    test_debug!("dependency_files: sorted candidate files: {files:?}");
    Ok(files)
}

pub(crate) fn strip_inline_comment(line: &str) -> &str {
    for (index, character) in line.char_indices() {
        if character == '#'
            && index > 0
            && line[..index]
                .chars()
                .next_back()
                .is_some_and(char::is_whitespace)
        {
            return line[..index].trim_end();
        }
    }

    line
}

fn normalize_editable_requirement(requirement: &str) -> Option<String> {
    let (_, fragment) = requirement.split_once('#')?;

    let egg = fragment
        .split('&')
        .find_map(|part| part.strip_prefix("egg="))?;

    let (package, extras) = match egg.split_once('[') {
        Some((package, extras)) => {
            let extras = extras.strip_suffix(']')?;
            (package, Some(extras))
        }
        None => (egg, None),
    };

    let package = package.trim();

    if package.is_empty()
        || !package
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || "._-".contains(character))
    {
        return None;
    }

    let extras = match extras {
        Some(extras) if !extras.trim().is_empty() => format!("[{}]", extras.trim()),
        Some(_) => return None,
        None => String::new(),
    };

    Some(format!("{package}{extras} @ {requirement}"))
}

fn parse_editable_requirement(line: &str) -> Option<&str> {
    line.strip_prefix("--editable ")
        .or_else(|| line.strip_prefix("-e "))
        .map(str::trim)
        .filter(|requirement| !requirement.is_empty())
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum RequirementLineKind<'a> {
    Editable(&'a str),
    LocalPath { path: &'a str, editable: bool },
    PipOption(&'a str),
    Package(&'a str),
}

fn is_local_path(requirement: &str) -> bool {
    let bytes = requirement.as_bytes();

    requirement.starts_with('.')
        || requirement.starts_with('/')
        || requirement.starts_with('\\')
        || (bytes.len() >= 3
            && bytes[0].is_ascii_alphabetic()
            && bytes[1] == b':'
            && (bytes[2] == b'/' || bytes[2] == b'\\'))
}

fn normalize_path(path: PathBuf) -> PathBuf {
    let mut normalized = PathBuf::new();

    for component in path.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                normalized.pop();
            }
            _ => normalized.push(component),
        }
    }

    normalized
}

pub(crate) fn resolve_local_path(requirement: &str, source: &Path) -> PathBuf {
    let path = Path::new(requirement);
    let path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        source.parent().unwrap_or_else(|| Path::new(".")).join(path)
    };

    normalize_path(path)
}

pub(crate) fn classify_requirement_line(line: &str) -> RequirementLineKind<'_> {
    if let Some(editable) = parse_editable_requirement(line) {
        if is_local_path(editable) {
            RequirementLineKind::LocalPath {
                path: editable,
                editable: true,
            }
        } else {
            RequirementLineKind::Editable(editable)
        }
    } else if is_local_path(line) {
        RequirementLineKind::LocalPath {
            path: line,
            editable: false,
        }
    } else if line.starts_with('-') {
        RequirementLineKind::PipOption(line)
    } else {
        RequirementLineKind::Package(line)
    }
}

fn discover_from_requirements(
    contents: &str,
    source: &Path,
    dependencies: &mut Vec<Dependency>,
) -> Result<(), String> {
    let mut logical_line = String::new();
    let mut logical_line_number = None;

    for (line_index, physical_line) in contents.lines().enumerate() {
        let line = physical_line.trim();

        // Skip blank lines and comments when not inside a continuation.
        if logical_line.is_empty() && (line.is_empty() || line.starts_with('#')) {
            test_debug!("requirements: skipping line {line:?}");
            continue;
        }

        if logical_line.is_empty() {
            logical_line_number = Some(line_index + 1);
        }

        // A trailing backslash continues the requirement on the next line.
        if let Some(without_backslash) = line.strip_suffix('\\') {
            if !logical_line.is_empty() {
                logical_line.push(' ');
            }

            logical_line.push_str(without_backslash.trim_end());
            continue;
        }

        if !logical_line.is_empty() {
            logical_line.push(' ');
        }

        logical_line.push_str(line);

        let requirement = strip_inline_comment(logical_line.trim()).trim();

        if requirement.is_empty() {
            test_debug!("requirements: skipping empty logical line");
        } else {
            match classify_requirement_line(requirement) {
                RequirementLineKind::Editable(editable) => {
                    test_debug!("requirements: parsing editable requirement {editable:?}");

                    if let Some(normalized) = normalize_editable_requirement(editable) {
                        add_requirement(&normalized, source, logical_line_number, dependencies)?;
                    } else {
                        test_debug!(
                            "requirements: skipping editable requirement without a valid egg fragment"
                        );
                    }
                }
                RequirementLineKind::LocalPath { path, editable } => {
                    if editable {
                        let resolved = resolve_local_path(path, source);
                        test_debug!(
                            "requirements: recognized editable local path requirement {path:?} as '{}'",
                            resolved.display()
                        );
                        discover_editable_local_project_metadata(
                            &resolved,
                            source,
                            logical_line_number,
                            dependencies,
                        );
                    } else {
                        test_debug!("requirements: recognized local path requirement {path:?}");
                    }
                }
                RequirementLineKind::PipOption(_option) => {
                    test_debug!("requirements: skipping unsupported option {_option:?}");
                }
                RequirementLineKind::Package(package) => {
                    test_debug!("requirements: parsing logical line {package:?}");
                    add_requirement(package, source, logical_line_number, dependencies)?;
                }
            }
        }

        logical_line.clear();
        logical_line_number = None;
    }

    // Process any remaining content, even if the file ends with a backslash.
    if !logical_line.trim().is_empty() {
        let requirement = strip_inline_comment(logical_line.trim()).trim();

        if !requirement.is_empty() {
            match classify_requirement_line(requirement) {
                RequirementLineKind::Editable(editable) => {
                    if let Some(normalized) = normalize_editable_requirement(editable) {
                        add_requirement(&normalized, source, logical_line_number, dependencies)?;
                    }
                }
                RequirementLineKind::Package(package) => {
                    add_requirement(package, source, logical_line_number, dependencies)?;
                }
                RequirementLineKind::LocalPath { path, editable } => {
                    if editable {
                        let resolved = resolve_local_path(path, source);
                        test_debug!(
                            "requirements: recognized editable local path requirement {path:?} as '{}'",
                            resolved.display()
                        );
                        discover_editable_local_project_metadata(
                            &resolved,
                            source,
                            logical_line_number,
                            dependencies,
                        );
                    } else {
                        test_debug!("requirements: recognized local path requirement {path:?}");
                    }
                }
                RequirementLineKind::PipOption(_option) => {
                    test_debug!("requirements: skipping unsupported option {_option:?}");
                }
            }
        }
    }

    Ok(())
}

fn discover_from_pyproject(
    contents: &str,
    source: &Path,
    dependencies: &mut Vec<Dependency>,
) -> Result<(), String> {
    let document: toml::Value = toml::from_str(contents)
        .map_err(|error| format!("failed to parse '{}': {error}", source.display()))?;
    test_debug!("pyproject: parsed document: {document:#?}");

    if let Some(array) = document
        .get("project")
        .and_then(|project| project.get("dependencies"))
        .and_then(toml::Value::as_array)
    {
        test_debug!(
            "pyproject: found {} project dependency item(s)",
            array.len()
        );
        for dependency in array.iter().filter_map(toml::Value::as_str) {
            test_debug!("pyproject: parsing dependency {dependency:?}");
            add_requirement(dependency, source, None, dependencies)?;
        }
    } else {
        test_debug!("pyproject: no project.dependencies array found");
    }

    Ok(())
}

fn discover_editable_local_project_metadata(
    project_path: &Path,
    source: &Path,
    line_number: Option<usize>,
    dependencies: &mut Vec<Dependency>,
) {
    let pyproject = project_path.join("pyproject.toml");
    if !pyproject.is_file() {
        eprintln!(
            "{}",
            local_requirement_diagnostic(
                source,
                line_number,
                project_path,
                "no pyproject.toml found"
            )
        );
        return;
    }

    let contents = match fs::read_to_string(&pyproject) {
        Ok(contents) => contents,
        Err(error) => {
            eprintln!(
                "{}",
                local_requirement_diagnostic(
                    source,
                    line_number,
                    project_path,
                    &format!("failed to read metadata: {error}")
                )
            );
            return;
        }
    };

    if let Err(error) = discover_from_pyproject(&contents, &pyproject, dependencies) {
        eprintln!(
            "{}",
            local_requirement_diagnostic(
                source,
                line_number,
                project_path,
                &format!("invalid metadata: {error}")
            )
        );
    }
}

pub(crate) fn local_requirement_diagnostic(
    source: &Path,
    line_number: Option<usize>,
    project_path: &Path,
    cause: &str,
) -> String {
    let location = line_number
        .map(|line| format!("{}:{line}", source.display()))
        .unwrap_or_else(|| source.display().to_string());
    format!(
        "warning: {location}: skipping editable local requirement '{}': {cause}",
        project_path.display()
    )
}

fn has_valid_percent_encoding(value: &str) -> bool {
    let bytes = value.as_bytes();
    let mut index = 0;

    while index < bytes.len() {
        if bytes[index] == b'%' {
            if index + 2 >= bytes.len()
                || !bytes[index + 1].is_ascii_hexdigit()
                || !bytes[index + 2].is_ascii_hexdigit()
            {
                return false;
            }

            index += 3;
        } else {
            index += 1;
        }
    }

    true
}

fn is_supported_direct_url(url: &str) -> bool {
    [
        "https://",
        "http://",
        "file://",
        "git+https://",
        "git+http://",
        "git+ssh://",
        "git+file://",
        "hg+https://",
        "hg+http://",
        "hg+ssh://",
        "hg+file://",
        "svn+https://",
        "svn+http://",
        "svn+ssh://",
        "svn+file://",
        "bzr+https://",
        "bzr+http://",
        "bzr+ssh://",
        "bzr+file://",
    ]
    .iter()
    .any(|prefix| url.starts_with(prefix))
}

fn add_requirement(
    requirement: &str,
    source: &Path,
    line_number: Option<usize>,
    dependencies: &mut Vec<Dependency>,
) -> Result<(), String> {
    let original_requirement = requirement;
    test_debug!(
        "add_requirement: input {requirement:?} from '{}'",
        source.display()
    );

    let (requirement, marker) = requirement
        .split_once(';')
        .map_or((requirement, None), |(requirement, marker)| {
            (requirement, Some(marker.trim().to_owned()))
        });

    if let Some(marker) = marker.as_deref() {
        let marker_requirement = format!("compatpy-marker-check; {marker}");
        let working_dir = source.parent().unwrap_or_else(|| Path::new("."));

        let marker_result =
            Requirement::<pep508_rs::VerbatimUrl>::parse(&marker_requirement, working_dir);

        if let Err(error) = marker_result {
            let location = line_number
                .map(|line| format!("{}:{line}", source.display()))
                .unwrap_or_else(|| source.display().to_string());
            return Err(format!(
                "{location}: invalid environment marker in requirement {original_requirement:?}: {error}"
            ));
        }
    }

    let requirement = requirement.trim();

    let (name_and_extras, url) = match requirement.split_once('@') {
        Some((name, url)) => {
            let url = url.trim();

            if !has_valid_percent_encoding(url) {
                return Err(format!("Invalid percent-encoding in direct URL: {url:?}"));
            }

            if !is_supported_direct_url(url) {
                let location = line_number
                    .map(|line| format!("{}:{line}", source.display()))
                    .unwrap_or_else(|| source.display().to_string());

                return Err(format!(
                    "{location}: unsupported direct URL scheme in requirement {original_requirement:?}: {url:?}"
                ));
            }

            (name.trim(), Some(url.to_owned()))
        }

        None => (requirement, None),
    };

    let (name, extras, specifier) = if let Some(start) = name_and_extras.find('[') {
        let Some(end) = name_and_extras.find(']') else {
            test_debug!("add_requirement: ignoring extras without closing bracket");
            return Ok(());
        };

        let name = name_and_extras[..start].trim();
        let extras_text = &name_and_extras[start + 1..end];
        let remainder = name_and_extras[end + 1..].trim();

        if extras_text.trim().is_empty() {
            test_debug!("add_requirement: ignoring empty extras");
            return Ok(());
        }

        let extras: Vec<String> = extras_text
            .split(',')
            .map(str::trim)
            .map(str::to_ascii_lowercase)
            .collect();

        if extras.iter().any(|extra| {
            extra.is_empty()
                || !extra
                    .chars()
                    .all(|character| character.is_ascii_alphanumeric() || "._-".contains(character))
        }) {
            test_debug!("add_requirement: ignoring invalid extras");
            return Ok(());
        }

        if !remainder.is_empty()
            && ![">=", "<=", "==", "!=", "~=", ">", "<", "=", "!", "~", "^"]
                .iter()
                .any(|operator| remainder.starts_with(operator))
        {
            test_debug!("add_requirement: ignoring invalid text after extras");
            return Ok(());
        }

        let specifier = if url.is_none() {
            (!remainder.is_empty()).then(|| remainder.to_owned())
        } else {
            if !remainder.is_empty() {
                test_debug!("add_requirement: ignoring specifier after direct URL");
                return Ok(());
            }
            None
        };

        (name, extras, specifier)
    } else {
        let name_end = name_and_extras
            .find(['<', '>', '=', '!', '~', '^'])
            .unwrap_or(name_and_extras.len());

        let name = name_and_extras[..name_end].trim();
        let remainder = name_and_extras[name_end..].trim();

        if !remainder.is_empty()
            && ![">=", "<=", "==", "!=", "~=", ">", "<", "=", "!", "~", "^"]
                .iter()
                .any(|operator| remainder.starts_with(operator))
        {
            test_debug!("add_requirement: ignoring invalid specifier");
            return Ok(());
        }

        let specifier = if url.is_none() {
            (!remainder.is_empty()).then(|| remainder.to_owned())
        } else {
            if !remainder.is_empty() {
                test_debug!("add_requirement: ignoring specifier after direct URL");
                return Ok(());
            }
            None
        };

        (name, Vec::new(), specifier)
    };

    if name.is_empty()
        || !name
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || "._-".contains(character))
    {
        test_debug!("add_requirement: ignoring invalid package name");
        return Ok(());
    }

    let dependency = Dependency {
        name: normalize_package_name(name),
        extras,
        specifier,
        url,
        marker,
        source: source.to_path_buf(),
    };

    test_debug!("add_requirement: normalized dependency {dependency:#?}");

    if !dependencies.contains(&dependency) {
        dependencies.push(dependency);
    }
    Ok(())
}
