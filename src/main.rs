use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process;

use compatpy::{
    PackageMetadata, VERSION, compatible_python_versions,
    compatible_python_versions_for_environment, discover_dependencies, fetch_pypi_metadata,
    select_package_release,
};
use pep508_rs::{MarkerEnvironment, MarkerEnvironmentBuilder};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct TargetEnvironment {
    implementation_name: String,
    implementation_version: String,
    os_name: String,
    platform_machine: String,
    platform_python_implementation: String,
    platform_release: String,
    platform_system: String,
    platform_version: String,
    python_full_version: String,
    python_version: String,
    sys_platform: String,
}

impl TargetEnvironment {
    fn marker_environment(&self) -> Result<MarkerEnvironment, String> {
        MarkerEnvironment::try_from(MarkerEnvironmentBuilder {
            implementation_name: &self.implementation_name,
            implementation_version: &self.implementation_version,
            os_name: &self.os_name,
            platform_machine: &self.platform_machine,
            platform_python_implementation: &self.platform_python_implementation,
            platform_release: &self.platform_release,
            platform_system: &self.platform_system,
            platform_version: &self.platform_version,
            python_full_version: &self.python_full_version,
            python_version: &self.python_version,
            sys_platform: &self.sys_platform,
        })
        .map_err(|error| format!("Invalid target environment: {error}"))
    }
}

struct CliOptions {
    project_path: PathBuf,
    target_environment_path: Option<PathBuf>,
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.len() == 1 {
        match args[0].as_str() {
            "--version" | "-v" => {
                println!("compatpy {VERSION}");
                return;
            }
            "--help" | "-h" => {
                print_usage(&mut io::stdout().lock()).expect("failed to write usage");
                return;
            }
            _ => {}
        }
    }

    let status = run(
        &args,
        &mut io::stdout().lock(),
        &mut io::stderr().lock(),
        fetch_pypi_metadata,
    );

    if status != 0 {
        process::exit(status);
    }
}

fn run<F>(
    args: &[String],
    stdout: &mut impl Write,
    stderr: &mut impl Write,
    mut fetch_metadata: F,
) -> i32
where
    F: FnMut(&str) -> Result<PackageMetadata, String>,
{
    let options = match parse_options(args) {
        Ok(options) => options,
        Err(error) => {
            let _ = writeln!(stderr, "Error: {error}");
            let _ = print_usage(stderr);
            return 1;
        }
    };

    let project_path = &options.project_path;
    if !project_path.is_dir() {
        let _ = writeln!(
            stderr,
            "Error: '{}' is not a valid directory.",
            project_path.display()
        );
        return 1;
    }

    let _ = writeln!(stdout, "Scanning\n{}", project_path.display());

    let dependencies = match discover_dependencies(project_path) {
        Ok(dependencies) => dependencies,
        Err(error) => {
            let _ = writeln!(stderr, "Error: {error}");
            return 1;
        }
    };

    if dependencies.is_empty() {
        let _ = writeln!(stdout, "No dependencies found.");
        return 0;
    }

    let has_markers = dependencies
        .iter()
        .any(|dependency| dependency.marker.is_some());
    let target_environment = match options.target_environment_path {
        Some(path) => {
            let contents = match fs::read(&path) {
                Ok(contents) => contents,
                Err(error) => {
                    let _ = writeln!(
                        stderr,
                        "Error: failed to read target environment '{}': {error}",
                        path.display()
                    );
                    return 1;
                }
            };

            let target: TargetEnvironment = match serde_json::from_slice(&contents) {
                Ok(target) => target,
                Err(error) => {
                    let _ = writeln!(
                        stderr,
                        "Error: invalid target environment file '{}': {error}",
                        path.display()
                    );
                    return 1;
                }
            };

            let marker_environment = match target.marker_environment() {
                Ok(environment) => environment,
                Err(error) => {
                    let _ = writeln!(stderr, "Error: {error}");
                    return 1;
                }
            };

            Some((target, marker_environment))
        }
        None if has_markers => {
            let _ = writeln!(
                stderr,
                "Error: dependency markers require --target-environment <json-file> with all PEP 508 environment fields."
            );
            return 1;
        }
        None => None,
    };

    if let Some((target, _)) = &target_environment {
        match serde_json::to_string(target) {
            Ok(json) => {
                let _ = writeln!(stdout, "Target environment: {json}");
            }
            Err(error) => {
                let _ = writeln!(
                    stderr,
                    "Error: failed to display target environment: {error}"
                );
                return 1;
            }
        }
    } else {
        let _ = writeln!(
            stdout,
            "Target environment: not required (no dependency markers)"
        );
    }

    let _ = writeln!(stdout, "\nDependencies:");
    let mut selected_releases = Vec::with_capacity(dependencies.len());

    for dependency in &dependencies {
        let _ = writeln!(
            stdout,
            "  {} | required: {} | ({})",
            dependency.name,
            dependency.specifier.as_deref().unwrap_or("any"),
            dependency.source.display()
        );

        match fetch_metadata(&dependency.name) {
            Ok(metadata) => {
                match select_package_release(&metadata, dependency.specifier.as_deref()) {
                    Ok(Some(release)) => {
                        let _ = writeln!(
                            stdout,
                            "    PyPI: {} | requires Python: {}",
                            release.version,
                            release.requires_python.as_deref().unwrap_or("any")
                        );
                        selected_releases.push(Some(release.clone()));
                    }
                    Ok(None) => {
                        let _ = writeln!(stdout, "    PyPI: no matching release");
                        selected_releases.push(None);
                    }
                    Err(error) => {
                        let _ = writeln!(stderr, "Error: {error}");
                        return 1;
                    }
                }
            }
            Err(error) => {
                let _ = writeln!(stdout, "    PyPI: unavailable ({error})");
                selected_releases.push(None);
            }
        }
    }

    let compatible_versions = match target_environment {
        Some((_, environment)) if has_markers => compatible_python_versions_for_environment(
            &dependencies,
            &selected_releases,
            &environment,
        ),
        _ => Ok(compatible_python_versions(
            &selected_releases.into_iter().flatten().collect::<Vec<_>>(),
        )),
    };

    match compatible_versions {
        Ok(versions) if versions.is_empty() => {
            let _ = writeln!(stdout, "\ncompatible Python versions:");
            let _ = writeln!(stdout, "  None");
            0
        }
        Ok(versions) => {
            let _ = writeln!(stdout, "\ncompatible Python versions:");
            for version in versions {
                let _ = writeln!(stdout, "  {version}");
            }
            0
        }
        Err(error) => {
            let _ = writeln!(stderr, "Error: compatibility analysis failed: {error}");
            1
        }
    }
}

fn parse_options(args: &[String]) -> Result<CliOptions, String> {
    let mut project_path = None;
    let mut target_environment_path = None;
    let mut index = 0;

    while index < args.len() {
        match args[index].as_str() {
            "--target-environment" => {
                if target_environment_path.is_some() {
                    return Err("--target-environment may only be specified once".to_owned());
                }
                index += 1;
                let Some(path) = args.get(index) else {
                    return Err("--target-environment requires a JSON file path".to_owned());
                };
                target_environment_path = Some(PathBuf::from(path));
            }
            argument if argument.starts_with('-') => {
                return Err(format!("unexpected argument '{argument}'"));
            }
            _ => {
                if project_path.is_some() {
                    return Err("only one project directory may be specified".to_owned());
                }
                project_path = Some(PathBuf::from(&args[index]));
            }
        }
        index += 1;
    }

    Ok(CliOptions {
        project_path: project_path.ok_or_else(|| "a project directory is required".to_owned())?,
        target_environment_path,
    })
}

fn print_usage(writer: &mut impl Write) -> io::Result<()> {
    writeln!(
        writer,
        "Usage: compatpy <project-directory> [--target-environment <json-file>]"
    )?;
    writeln!(writer, "       compatpy --help")?;
    writeln!(writer, "       compatpy --version")
}

#[cfg(test)]
mod tests {
    use super::*;
    use compatpy::PackageRelease;
    use std::path::Path;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_project(requirements: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = env::temp_dir().join(format!("compatpy-cli-{unique}"));
        fs::create_dir_all(&path).unwrap();
        fs::write(path.join("requirements.txt"), requirements).unwrap();
        path
    }

    fn target_environment_file(project: &Path, contents: &str) -> PathBuf {
        let path = project.join("target.json");
        fs::write(&path, contents).unwrap();
        path
    }

    fn full_target_json() -> &'static str {
        r#"{
            "implementation_name": "cpython",
            "implementation_version": "3.12.0",
            "os_name": "posix",
            "platform_machine": "x86_64",
            "platform_python_implementation": "CPython",
            "platform_release": "test-release",
            "platform_system": "Linux",
            "platform_version": "test-version",
            "python_full_version": "3.12.0",
            "python_version": "3.12",
            "sys_platform": "linux"
        }"#
    }

    fn metadata(requires_python: Option<&str>) -> PackageMetadata {
        PackageMetadata {
            name: "example".to_owned(),
            latest_version: "1.0.0".to_owned(),
            requires_python: None,
            releases: vec![PackageRelease {
                version: "1.0.0".to_owned(),
                requires_python: requires_python.map(str::to_owned),
            }],
        }
    }

    #[test]
    fn marker_analysis_reports_target_and_filters_candidate_releases() {
        let project = temp_project("example; python_version < \"3.10\"\n");
        let target = target_environment_file(&project, full_target_json());
        let args = vec![
            project.display().to_string(),
            "--target-environment".to_owned(),
            target.display().to_string(),
        ];
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        let status = run(&args, &mut stdout, &mut stderr, |_| {
            Ok(metadata(Some(">=3.9")))
        });

        let stdout = String::from_utf8(stdout).unwrap();
        let stderr = String::from_utf8(stderr).unwrap();
        assert_eq!(status, 0, "{stderr}");
        assert!(stderr.is_empty());
        assert!(stdout.contains("Target environment:"));
        assert!(stdout.contains("\"sys_platform\":\"linux\""));
        assert!(stdout.contains("  3.9\n"));
        assert!(!stdout.contains("  3.8\n"));

        fs::remove_dir_all(project).unwrap();
    }

    #[test]
    fn marker_false_dependency_without_release_does_not_exclude_candidates() {
        let project = temp_project("example; python_version < \"3.10\"\n");
        let target = target_environment_file(&project, full_target_json());
        let args = vec![
            project.display().to_string(),
            "--target-environment".to_owned(),
            target.display().to_string(),
        ];
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        let status = run(&args, &mut stdout, &mut stderr, |_| {
            Err("fixture metadata unavailable".to_owned())
        });

        let stdout = String::from_utf8(stdout).unwrap();
        assert_eq!(status, 0, "{}", String::from_utf8_lossy(&stderr));
        assert!(stdout.contains("  3.10\n"));
        assert!(!stdout.contains("  3.9\n"));

        fs::remove_dir_all(project).unwrap();
    }

    #[test]
    fn marker_true_dependency_without_release_has_no_compatible_candidates() {
        let project = temp_project("example; python_version >= \"3.7\"\n");
        let target = target_environment_file(&project, full_target_json());
        let args = vec![
            project.display().to_string(),
            "--target-environment".to_owned(),
            target.display().to_string(),
        ];
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        let status = run(&args, &mut stdout, &mut stderr, |_| {
            Err("fixture metadata unavailable".to_owned())
        });

        let stdout = String::from_utf8(stdout).unwrap();
        assert_eq!(status, 0, "{}", String::from_utf8_lossy(&stderr));
        assert!(stdout.contains("compatible Python versions:\n  None"));

        fs::remove_dir_all(project).unwrap();
    }

    #[test]
    fn invalid_marker_returns_cli_error_without_compatibility_output() {
        let project = temp_project("example; python_version >>> \"3.10\"\n");
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        let status = run(
            &[project.display().to_string()],
            &mut stdout,
            &mut stderr,
            |_| panic!("invalid marker must fail before metadata lookup"),
        );

        let stdout = String::from_utf8(stdout).unwrap();
        let stderr = String::from_utf8(stderr).unwrap();
        assert_eq!(status, 1);
        assert!(stderr.contains("requirements.txt"));
        assert!(stderr.contains("python_version >>>"));
        assert!(!stdout.contains("compatible Python versions:"));

        fs::remove_dir_all(project).unwrap();
    }

    #[test]
    fn marker_analysis_requires_complete_explicit_target_environment() {
        let project = temp_project("example; sys_platform == \"linux\"\n");
        let target = target_environment_file(&project, r#"{"sys_platform":"linux"}"#);
        let args = vec![
            project.display().to_string(),
            "--target-environment".to_owned(),
            target.display().to_string(),
        ];
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        let status = run(&args, &mut stdout, &mut stderr, |_| {
            panic!("invalid target environment must fail before metadata lookup")
        });

        let stdout = String::from_utf8(stdout).unwrap();
        let stderr = String::from_utf8(stderr).unwrap();
        assert_eq!(status, 1);
        assert!(stderr.contains("invalid target environment file"));
        assert!(!stdout.contains("compatible Python versions:"));

        fs::remove_dir_all(project).unwrap();
    }

    #[test]
    fn marker_dependencies_require_an_explicit_target_file() {
        let project = temp_project("example; sys_platform == \"linux\"\n");
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        let status = run(
            &[project.display().to_string()],
            &mut stdout,
            &mut stderr,
            |_| panic!("missing target environment must fail before metadata lookup"),
        );

        assert_eq!(status, 1);
        assert!(
            String::from_utf8(stderr)
                .unwrap()
                .contains("dependency markers require --target-environment")
        );
        assert!(
            !String::from_utf8(stdout)
                .unwrap()
                .contains("compatible Python versions:")
        );

        fs::remove_dir_all(project).unwrap();
    }

    #[test]
    fn marker_free_dependencies_keep_existing_cli_analysis() {
        let project = temp_project("example>=1\n");
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        let status = run(
            &[project.display().to_string()],
            &mut stdout,
            &mut stderr,
            |_| Ok(metadata(Some(">=3.12"))),
        );

        let stdout = String::from_utf8(stdout).unwrap();
        assert_eq!(status, 0, "{}", String::from_utf8_lossy(&stderr));
        assert!(stdout.contains("Target environment: not required"));
        assert!(stdout.contains("  3.12\n"));
        assert!(!stdout.contains("  3.11\n"));

        fs::remove_dir_all(project).unwrap();
    }
}
