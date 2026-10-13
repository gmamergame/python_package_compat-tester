// CompatPy crate tests and public re-exports. The comments below explain intent;
// executable code and test expectations are otherwise left unchanged.
// Define a debug-printing macro that only produces output during tests.
macro_rules! test_debug {
    ($($argument:tt)*) => {{
        #[cfg(test)]
        {
            // Format and print the message with a recognizable prefix.
            eprintln!("[compat-debug] {}", format_args!($($argument)*));
        }
    }};
}

// Declare the internal modules that organize the application.
mod cli;
mod compatibility;
mod dependencies;
mod package_name;
mod pypi;
mod requirements;

// Re-export public types and functions so callers can access them
// directly through this crate.
pub use cli::{Command, VERSION, parse_args};
pub use compatibility::{
    compatible_python_versions, compatible_python_versions_for_environment, select_package_release,
};
pub use dependencies::{Dependency, dependency_applies_to_environment, discover_dependencies};
pub use pypi::{PackageMetadata, PackageRelease, fetch_pypi_metadata};
pub use requirements::{ParsedRequirement, parse_requirement};

// Import internal helpers for use by this module's unit tests only.
#[cfg(test)]
use compatibility::{compare_versions, package_version_satisfies};
#[cfg(test)]
use dependencies::strip_inline_comment;
#[cfg(test)]
use package_name::normalize_package_name;
#[cfg(test)]
use pypi::parse_pypi_metadata;

// Define the unit tests for this crate.
#[cfg(test)]
mod tests {

    use super::{
        Command, Dependency, PackageRelease, ParsedRequirement, dependency_applies_to_environment,
        discover_dependencies, normalize_package_name, parse_args, parse_requirement,
        select_package_release,
    };
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    // Check whether a version satisfies a requirement, treating invalid test input as a test error.
    fn package_version_satisfies(version: &str, requirement: &str) -> bool {
        super::package_version_satisfies(version, requirement)
            .expect("test specifier should be valid")
    }

    // Construct a realistic PEP 508 marker environment for a chosen Python version and platform.
    fn test_marker_environment(
        python_version: &str,
        sys_platform: &str,
    ) -> pep508_rs::MarkerEnvironment {
        // Construct the full Python version, e.g. "3.12" becomes "3.12.0".
        let python_full_version = format!("{python_version}.0");

        // Populate the environment fields used by PEP 508 marker evaluation.
        pep508_rs::MarkerEnvironment::try_from(pep508_rs::MarkerEnvironmentBuilder {
            implementation_name: "cpython",
            implementation_version: "3.12.0",
            // Windows uses "nt"; other platforms use "posix".
            os_name: if sys_platform == "win32" {
                "nt"
            } else {
                "posix"
            },
            platform_machine: "x86_64",
            platform_python_implementation: "CPython",
            platform_release: "",
            // Map the requested platform to its operating-system name.
            platform_system: if sys_platform == "win32" {
                "Windows"
            } else {
                "Linux"
            },
            platform_version: "",
            python_full_version: &python_full_version,
            python_version,
            sys_platform,
        })
        // Fail the test immediately if the environment is invalid.
        .unwrap()
    }

    // Verify that running without arguments selects the help command.

    #[test]
    fn help_is_the_default_command() {
        assert_eq!(parse_args(&[]), Ok(Command::Help));
    }

    // Verify that the version flag selects the version command.

    #[test]
    fn parses_version_flag() {
        assert_eq!(parse_args(&["--version".to_owned()]), Ok(Command::Version));
    }

    // Verify that unsupported command-line arguments are rejected.

    #[test]
    fn rejects_unexpected_arguments() {
        assert!(parse_args(&["unknown".to_owned()]).is_err());
    }

    // Verify that package names are normalized consistently.

    #[test]
    fn normalizes_package_names() {
        for name in ["my_package", "my.package", "my-package", "My_Package"] {
            assert_eq!(normalize_package_name(name), "my-package");
        }
        assert_eq!(normalize_package_name("My_Package.Name"), "my-package-name");
        assert_eq!(
            normalize_package_name("My--Package..Name"),
            "my-package-name"
        );
        assert_eq!(normalize_package_name("my.-_package"), "my-package");
    }

    // Verify parsing requirement with canonical name without changing other fields.

    #[test]
    fn parses_requirement_with_canonical_name_without_changing_other_fields() {
        assert_eq!(
            parse_requirement("My_Package.Name[Security_Extra]>=1.2.3").unwrap(),
            ParsedRequirement {
                name: "my-package-name".to_owned(),
                extras: vec!["security_extra".to_owned()],
                specifier: ">=1.2.3".to_owned(),
                url: None,
            }
        );

        assert_eq!(
            parse_requirement(
                "My_Package.Name @ https://example.com/pkg.whl#subdirectory=src&sha256=abc"
            )
            .unwrap()
            .url
            .as_deref(),
            Some("https://example.com/pkg.whl#subdirectory=src&sha256=abc")
        );
    }

    // Verify discovery of requirements txt.

    #[test]
    fn discovers_requirements_txt() {
        let project = temporary_project();
        fs::write(
            project.join("requirements.txt"),
            "# comment\nRequests>=2.0\nFlask==3.0\nrequests\n",
        )
        .unwrap();

        let dependencies = discover_dependencies(&project).unwrap();

        assert_eq!(
            dependencies,
            vec![
                Dependency {
                    name: "flask".to_owned(),
                    extras: vec![],
                    specifier: Some("==3.0".to_owned()),
                    url: None,
                    marker: None,
                    source: project.join("requirements.txt"),
                },
                Dependency {
                    name: "requests".to_owned(),
                    extras: vec![],
                    specifier: Some(">=2.0".to_owned()),
                    url: None,
                    marker: None,
                    source: project.join("requirements.txt"),
                },
                Dependency {
                    name: "requests".to_owned(),
                    extras: vec![],
                    specifier: None,
                    url: None,
                    marker: None,
                    source: project.join("requirements.txt"),
                },
            ]
        );
        fs::remove_dir_all(project).unwrap();
    }

    // Verify the expected behavior for skips editable git requirement without egg fragment.

    #[test]
    fn skips_editable_git_requirement_without_egg_fragment() {
        let project = temporary_project();
        fs::write(
            project.join("requirements.txt"),
            "-e git+https://github.com/example/repo.git@main\n",
        )
        .unwrap();

        let dependencies = discover_dependencies(&project).unwrap();

        assert!(
            dependencies.is_empty(),
            "editable Git URL without #egg= should be skipped"
        );

        fs::remove_dir_all(project).unwrap();
    }

    // Verify discovery of pyproject dependencies.

    #[test]
    fn discovers_pyproject_dependencies() {
        let project = temporary_project();

        fs::write(
            project.join("pyproject.toml"),
            r#"[project]
dependencies = [
    "requests>=2.0",
    "Flask",
]
"#,
        )
        .unwrap();

        let dependencies = discover_dependencies(&project).unwrap();

        assert_eq!(dependencies.len(), 2);

        assert_eq!(dependencies[0].name, "flask");
        assert_eq!(dependencies[0].specifier, None);
        assert_eq!(dependencies[0].marker, None);

        assert_eq!(dependencies[1].name, "requests");
        assert_eq!(dependencies[1].specifier, Some(">=2.0".to_owned()));
        assert_eq!(dependencies[1].marker, None);

        fs::remove_dir_all(project).unwrap();
    }

    // Verify parsing pypi metadata.

    #[test]
    fn parses_pypi_metadata() {
        let json = r#"
        {
            "info": {
                "name": "Example-Package",
                "version": "2.0.0",
                "requires_python": ">=3.9"
            },
            "releases": {
                "1.0.0": [
                    {
                        "requires_python": ">=3.7"
                    }
                ],
                "2.0.0": [
                    {
                        "requires_python": ">=3.9"
                    }
                ]
            }
        }
        "#;

        let metadata = super::parse_pypi_metadata(json).unwrap();

        assert_eq!(metadata.name, "example-package");
        assert_eq!(metadata.latest_version, "2.0.0");
        assert_eq!(metadata.requires_python, Some(">=3.9".to_owned()));

        assert_eq!(
            metadata.releases,
            vec![
                super::PackageRelease {
                    version: "1.0.0".to_owned(),
                    requires_python: Some(">=3.7".to_owned()),
                },
                super::PackageRelease {
                    version: "2.0.0".to_owned(),
                    requires_python: Some(">=3.9".to_owned()),
                },
            ]
        )
    }

    // Verify selection of exact package version.

    #[test]
    fn selects_exact_package_version() {
        let metadata = super::PackageMetadata {
            name: "uvicorn".to_owned(),
            latest_version: "0.54.0".to_owned(),
            requires_python: Some(">=3.10".to_owned()),
            releases: vec![
                super::PackageRelease {
                    version: "0.29.0".to_owned(),
                    requires_python: Some(">=3.8".to_owned()),
                },
                super::PackageRelease {
                    version: "0.54.0".to_owned(),
                    requires_python: Some(">=3.10".to_owned()),
                },
            ],
        };

        let release = select_package_release(&metadata, Some("==0.29.0"))
            .unwrap()
            .unwrap();

        assert_eq!(release.version, "0.29.0");
        assert_eq!(release.requires_python, Some(">=3.8".to_owned()));
    }

    // Verify selection of latest matching package version.

    #[test]
    fn selects_latest_matching_package_version() {
        let metadata = super::PackageMetadata {
            name: "example".to_owned(),
            latest_version: "2.0.0".to_owned(),
            requires_python: None,
            releases: vec![
                super::PackageRelease {
                    version: "1.0.0".to_owned(),
                    requires_python: None,
                },
                super::PackageRelease {
                    version: "1.5.0".to_owned(),
                    requires_python: None,
                },
                super::PackageRelease {
                    version: "2.0.0".to_owned(),
                    requires_python: None,
                },
            ],
        };

        let release = select_package_release(&metadata, Some(">=1.0,<2.0"))
            .unwrap()
            .unwrap();

        assert_eq!(release.version, "1.5.0");
    }

    // Verify selection of latest release without specifier.

    #[test]
    fn selects_latest_release_without_specifier() {
        let metadata = super::PackageMetadata {
            name: "example".to_owned(),
            latest_version: "2.0.0".to_owned(),
            requires_python: None,
            releases: vec![
                super::PackageRelease {
                    version: "1.0.0".to_owned(),
                    requires_python: None,
                },
                super::PackageRelease {
                    version: "2.0.0".to_owned(),
                    requires_python: None,
                },
            ],
        };

        let release = select_package_release(&metadata, None).unwrap().unwrap();

        assert_eq!(release.version, "2.0.0");
    }

    // Verify selection of stable exact match when prerelease also exists.

    #[test]
    fn selects_stable_exact_match_when_prerelease_also_exists() {
        let metadata = super::PackageMetadata {
            name: "example".to_owned(),
            latest_version: "3.9.0".to_owned(),
            requires_python: None,
            releases: vec![
                super::PackageRelease {
                    version: "3.9.0rc2".to_owned(),
                    requires_python: None,
                },
                super::PackageRelease {
                    version: "3.9.0".to_owned(),
                    requires_python: None,
                },
            ],
        };

        let release = select_package_release(&metadata, Some("==3.9.0"))
            .unwrap()
            .unwrap();

        assert_eq!(release.version, "3.9.0");
    }

    // Verify stable-release behavior for release beats prerelease.

    #[test]
    fn stable_release_beats_prerelease() {
        let metadata = super::PackageMetadata {
            name: "example".to_owned(),
            latest_version: "4.0.0".to_owned(),
            requires_python: None,
            releases: vec![
                super::PackageRelease {
                    version: "3.9.0".to_owned(),
                    requires_python: None,
                },
                super::PackageRelease {
                    version: "3.9.0rc2".to_owned(),
                    requires_python: None,
                },
                super::PackageRelease {
                    version: "4.0.0a1".to_owned(),
                    requires_python: None,
                },
                super::PackageRelease {
                    version: "4.0.0".to_owned(),
                    requires_python: None,
                },
            ],
        };

        let release = select_package_release(&metadata, Some(">=3.8.0"))
            .unwrap()
            .unwrap();

        assert_eq!(release.version, "4.0.0");
    }

    // Verify the expected behavior for prerelease versions are ordered correctly.

    #[test]
    fn prerelease_versions_are_ordered_correctly() {
        assert_eq!(
            super::compare_versions("3.9.0a1", "3.9.0b1"),
            std::cmp::Ordering::Less
        );

        assert_eq!(
            super::compare_versions("3.9.0b1", "3.9.0rc1"),
            std::cmp::Ordering::Less
        );

        assert_eq!(
            super::compare_versions("3.9.0rc1", "3.9.0"),
            std::cmp::Ordering::Less
        );
    }

    // Verify release behavior for candidate aliases and omitted serial normalize.

    #[test]
    fn release_candidate_aliases_and_omitted_serial_normalize() {
        let equivalent_versions = [
            ("1.0rc1", "1.0c1"),
            ("1.0rc1", "1.0pre1"),
            ("1.0rc1", "1.0preview1"),
            ("1.0rc1", "1.0-Preview-1"),
            ("1.0rc1", "1.0_PRE_1"),
            ("1.0rc1", "1.0RC1"),
        ];

        for (canonical, alias) in equivalent_versions {
            let parsed = alias
                .parse::<pep440_rs::Version>()
                .unwrap_or_else(|error| panic!("failed to parse alias {alias:?}: {error}"));
            assert_eq!(parsed.to_string(), canonical, "alias {alias:?}");
            assert_eq!(
                super::compare_versions(alias, canonical),
                std::cmp::Ordering::Equal,
                "alias {alias:?} should normalize to {canonical:?}"
            );
        }

        let omitted_serial = "1.0rc".parse::<pep440_rs::Version>().unwrap();
        assert_eq!(omitted_serial.pre().unwrap().number, 0);
        assert_eq!(
            super::compare_versions("1.0rc", "1.0rc0"),
            std::cmp::Ordering::Equal
        );
    }

    // Verify release behavior for candidate serials order numerically and reject invalid suffixes.

    #[test]
    fn release_candidate_serials_order_numerically_and_reject_invalid_suffixes() {
        use super::compare_versions;
        use std::cmp::Ordering;

        assert_eq!(compare_versions("1.0rc2", "1.0rc10"), Ordering::Less);
        assert_eq!(compare_versions("1.0rc10", "1.0rc2"), Ordering::Greater);

        for invalid in ["1.0rc.invalid", "1.0rc2junk"] {
            assert!(
                invalid.parse::<pep440_rs::Version>().is_err(),
                "expected malformed RC version {invalid:?} to fail parsing"
            );
        }

        let metadata = super::PackageMetadata {
            name: "example".to_owned(),
            latest_version: "1.0rc.invalid".to_owned(),
            requires_python: None,
            releases: vec![PackageRelease {
                version: "1.0rc.invalid".to_owned(),
                requires_python: None,
            }],
        };
        let error = select_package_release(&metadata, None)
            .expect_err("malformed RC suffix should be rejected");
        assert!(error.contains("Invalid package version '1.0rc.invalid'"));
    }

    // Verify release behavior for candidates order after dev and before final and post.

    #[test]
    fn release_candidates_order_after_dev_and_before_final_and_post() {
        use super::compare_versions;
        use std::cmp::Ordering;

        let ordered_versions = ["1.0.dev10", "1.0rc1", "1.0rc10", "1.0", "1.0.post1"];

        for pair in ordered_versions.windows(2) {
            assert_eq!(
                compare_versions(pair[0], pair[1]),
                Ordering::Less,
                "expected {:?} to sort before {:?}",
                pair[0],
                pair[1]
            );
        }
    }

    // Verify alpha/beta release behavior for beta aliases and separators normalize equivalently.

    #[test]
    fn alpha_beta_aliases_and_separators_normalize_equivalently() {
        let equivalent_versions = [
            ("1.0a1", "1.0alpha1"),
            ("1.0a1", "1.0-alpha-1"),
            ("1.0a1", "1.0_ALPHA_1"),
            ("1.0a1", "1.0ALPHA1"),
            ("1.0b2", "1.0beta2"),
            ("1.0b2", "1.0-beta-2"),
            ("1.0b2", "1.0_BETA_2"),
            ("1.0b2", "1.0BETA2"),
        ];

        for (canonical, alias) in equivalent_versions {
            let parsed = alias
                .parse::<pep440_rs::Version>()
                .unwrap_or_else(|error| panic!("failed to parse alias {alias:?}: {error}"));
            assert_eq!(parsed.to_string(), canonical, "alias {alias:?}");
            assert_eq!(
                super::compare_versions(alias, canonical),
                std::cmp::Ordering::Equal,
                "alias {alias:?} should normalize to {canonical:?}"
            );
        }
    }

    // Verify alpha/beta release behavior for beta serials default to zero and reject invalid suffixes.

    #[test]
    fn alpha_beta_serials_default_to_zero_and_reject_invalid_suffixes() {
        use super::compare_versions;
        use std::cmp::Ordering;

        for (without_serial, explicit_zero) in [("1.0a", "1.0a0"), ("1.0b", "1.0b0")] {
            let parsed = without_serial.parse::<pep440_rs::Version>().unwrap();
            assert_eq!(parsed.pre().unwrap().number, 0);
            assert_eq!(
                compare_versions(without_serial, explicit_zero),
                Ordering::Equal
            );
        }

        assert_eq!(compare_versions("1.0a2", "1.0a10"), Ordering::Less);
        assert_eq!(compare_versions("1.0b2", "1.0b10"), Ordering::Less);

        for invalid in ["1.0a.invalid", "1.0b2junk"] {
            assert!(
                invalid.parse::<pep440_rs::Version>().is_err(),
                "expected invalid prerelease suffix {invalid:?} to fail parsing"
            );
        }

        let metadata = super::PackageMetadata {
            name: "example".to_owned(),
            latest_version: "1.0a.invalid".to_owned(),
            requires_python: None,
            releases: vec![PackageRelease {
                version: "1.0a.invalid".to_owned(),
                requires_python: None,
            }],
        };
        let error = select_package_release(&metadata, None)
            .expect_err("malformed alpha suffix should be rejected");
        assert!(error.contains("Invalid package version '1.0a.invalid'"));
    }

    // Verify alpha/beta release behavior for beta versions order across development final and post releases.

    #[test]
    fn alpha_beta_versions_order_across_development_final_and_post_releases() {
        use super::compare_versions;
        use std::cmp::Ordering;

        let ordered_versions = [
            "1.0.dev1",
            "1.0a1",
            "1.0a2",
            "1.0b1",
            "1.0b2",
            "1.0rc1",
            "1.0",
            "1.0.post1",
        ];

        for pair in ordered_versions.windows(2) {
            assert_eq!(
                compare_versions(pair[0], pair[1]),
                Ordering::Less,
                "expected {:?} to sort before {:?}",
                pair[0],
                pair[1]
            );
        }
    }

    // Verify development-release behavior for release serials are parsed by pep440.

    #[test]
    fn development_release_serials_are_parsed_by_pep440() {
        let dev_without_serial = "1.0.dev".parse::<pep440_rs::Version>().unwrap();
        let dev_zero = "1.0.dev0".parse::<pep440_rs::Version>().unwrap();
        let dev_one = "1.0.dev1".parse::<pep440_rs::Version>().unwrap();
        let dev_ten = "1.0.dev10".parse::<pep440_rs::Version>().unwrap();

        assert_eq!(dev_without_serial.dev(), Some(0));
        assert_eq!(dev_zero.dev(), Some(0));
        assert_eq!(dev_one.dev(), Some(1));
        assert_eq!(dev_ten.dev(), Some(10));
        assert!("1.0.devinvalid".parse::<pep440_rs::Version>().is_err());
    }

    // Verify development-release behavior for releases order before alpha beta rc and final.

    #[test]
    fn development_releases_order_before_alpha_beta_rc_and_final() {
        use super::compare_versions;
        use std::cmp::Ordering;

        let ordered_versions = ["1.0.dev1", "1.0.dev2", "1.0a1", "1.0b1", "1.0rc1", "1.0"];

        for pair in ordered_versions.windows(2) {
            assert_eq!(
                compare_versions(pair[0], pair[1]),
                Ordering::Less,
                "expected {:?} to sort before {:?}",
                pair[0],
                pair[1]
            );
        }

        assert_eq!(compare_versions("1.0.dev10", "1.0.dev2"), Ordering::Greater);
    }

    // Verify development-release behavior for releases order against post releases numerically.

    #[test]
    fn development_releases_order_against_post_releases_numerically() {
        use super::compare_versions;
        use std::cmp::Ordering;

        let ordered_versions = [
            "1.0.dev2",
            "1.0.dev10",
            "1.0",
            "1.0.post1.dev2",
            "1.0.post1.dev10",
            "1.0.post1",
            "1.0.post2.dev1",
            "1.0.post2",
        ];

        for pair in ordered_versions.windows(2) {
            assert_eq!(
                compare_versions(pair[0], pair[1]),
                Ordering::Less,
                "expected {:?} to sort before {:?}",
                pair[0],
                pair[1]
            );
        }

        assert_eq!(
            compare_versions("1.0.post1.dev10", "1.0.post1.dev2"),
            Ordering::Greater
        );
        assert_eq!(
            compare_versions("1.0.post10", "1.0.post2"),
            Ordering::Greater
        );
    }

    // Verify development-release behavior for versions match specifiers and select latest release.

    #[test]
    fn development_versions_match_specifiers_and_select_latest_release() {
        assert!(package_version_satisfies("1.0.dev1", "==1.0.dev1"));
        assert!(!package_version_satisfies("1.0.dev2", "==1.0.dev1"));
        assert!(package_version_satisfies("1.0.dev2", ">1.0.dev1"));
        assert!(!package_version_satisfies("1.0.dev1", ">=1.0.dev2"));
        assert!(package_version_satisfies("1.0.dev10", "<=1.0.dev10"));

        let metadata = super::PackageMetadata {
            name: "example".to_owned(),
            latest_version: "1.0".to_owned(),
            requires_python: None,
            releases: vec![
                PackageRelease {
                    version: "1.0.dev2".to_owned(),
                    requires_python: None,
                },
                PackageRelease {
                    version: "1.0.dev10".to_owned(),
                    requires_python: None,
                },
                PackageRelease {
                    version: "1.0.dev1".to_owned(),
                    requires_python: None,
                },
            ],
        };

        let selected = select_package_release(&metadata, Some(">=1.0.dev2,<=1.0.dev10"))
            .unwrap()
            .expect("matching development release should be selected");
        assert_eq!(selected.version, "1.0.dev10");
    }

    // Verify preference for stable release over prerelease.

    #[test]
    fn prefers_stable_release_over_prerelease() {
        let metadata = super::PackageMetadata {
            name: "example".to_owned(),
            latest_version: "3.1.0rc0".to_owned(),
            requires_python: None,
            releases: vec![
                super::PackageRelease {
                    version: "2.9.0".to_owned(),
                    requires_python: None,
                },
                super::PackageRelease {
                    version: "3.0.0".to_owned(),
                    requires_python: None,
                },
                super::PackageRelease {
                    version: "3.1.0rc0".to_owned(),
                    requires_python: None,
                },
            ],
        };

        let release = select_package_release(&metadata, Some(">=2.0.0"))
            .unwrap()
            .unwrap();

        assert_eq!(release.version, "3.0.0");
    }

    // Verify discovery of compatible python versions.

    #[test]
    fn finds_compatible_python_versions() {
        let release1 = super::PackageRelease {
            version: "1.0.0".to_owned(),
            requires_python: Some(">=3.10".to_owned()),
        };

        let release2 = super::PackageRelease {
            version: "2.0.0".to_owned(),
            requires_python: Some(">=3.11".to_owned()),
        };

        let release3 = super::PackageRelease {
            version: "3.0.0".to_owned(),
            requires_python: Some(">=3.12".to_owned()),
        };

        let releases = vec![release1, release2, release3];

        let compatible = super::compatible_python_versions(&releases);

        assert_eq!(
            compatible,
            vec![
                "3.12".to_owned(),
                "3.13".to_owned(),
                "3.14".to_owned(),
                "3.15".to_owned()
            ]
        );
    }

    // Verify exclusion of specific python versions.

    #[test]
    fn excludes_specific_python_versions() {
        let release = super::PackageRelease {
            version: "1.0.0".to_owned(),
            requires_python: Some(">=3.8, !=3.9".to_owned()),
        };

        let releases = vec![release];

        let compatible = super::compatible_python_versions(&releases);

        assert_eq!(
            compatible,
            vec![
                "3.8".to_owned(),
                "3.10".to_owned(),
                "3.11".to_owned(),
                "3.12".to_owned(),
                "3.13".to_owned(),
                "3.14".to_owned(),
                "3.15".to_owned(),
            ]
        );
    }

    // Verify support for compatible release operator.

    #[test]
    fn supports_compatible_release_operator() {
        assert!(package_version_satisfies("3.12.0", "~=3.12.0"));
        assert!(package_version_satisfies("3.12.5", "~=3.12.0"));
        assert!(!package_version_satisfies("3.11.9", "~=3.12.0"));
        assert!(!package_version_satisfies("3.13.0", "~=3.12.0"));

        assert!(package_version_satisfies("3.12.0", "~=3.12"));
        assert!(package_version_satisfies("3.14.0", "~=3.12"));
        assert!(!package_version_satisfies("2.9.9", "~=3.12"));
        assert!(!package_version_satisfies("4.0.0", "~=3.12"));

        assert!(package_version_satisfies("1.4.5.0", "~=1.4.5.0"));
        assert!(package_version_satisfies("1.4.5.9", "~=1.4.5.0"));
        assert!(!package_version_satisfies("1.4.6.0", "~=1.4.5.0"));
    }

    // Verify support for equal version wildcard.

    #[test]
    fn supports_equal_version_wildcard() {
        assert!(package_version_satisfies("3.12.0", "==3.12.*"));
        assert!(package_version_satisfies("3.12.5", "==3.12.*"));
        assert!(package_version_satisfies("3.12", "==3.12.*"));
        assert!(package_version_satisfies("3.12.0a1", "==3.12.*"));
        assert!(package_version_satisfies("3.12.0.post1", "==3.12.*"));
        assert!(package_version_satisfies("3.12.0+local", "==3.12.*"));
        assert!(!package_version_satisfies("3.13.0", "==3.12.*"));
        assert!(!package_version_satisfies("3.120.0", "==3.12.*"));
        assert!(!package_version_satisfies("1!3.12.0", "==3.12.*"));
        assert!(package_version_satisfies("1!3.12.0", "==1!3.12.*"));
    }

    // Verify equality behavior for version wildcard matches normalized release prefixes.

    #[test]
    fn equal_version_wildcard_matches_normalized_release_prefixes() {
        let matching_versions = [
            "3.12",
            "3.12.0",
            "3.12.0.0",
            "3.12.1",
            "3.12.1.0.7",
            "3.12.10",
        ];
        let non_matching_versions = ["3.11.99", "3.120", "3.120.0", "3.13", "4.12.0"];

        for version in matching_versions {
            assert!(
                package_version_satisfies(version, "==3.12.*"),
                "expected {version:?} to match ==3.12.*"
            );
        }

        for version in non_matching_versions {
            assert!(
                !package_version_satisfies(version, "==3.12.*"),
                "expected adjacent prefix {version:?} not to match ==3.12.*"
            );
        }

        assert!(package_version_satisfies("3.012.5", "==3.012.*"));
    }

    // Verify equality behavior for version wildcard matches release labels by public prefix.

    #[test]
    fn equal_version_wildcard_matches_release_labels_by_public_prefix() {
        let matching_versions = [
            "3.12.0.dev1",
            "3.12.0a1",
            "3.12.0b1",
            "3.12.0rc1",
            "3.12.0.post1",
            "3.12.1+vendor.1",
            "3.12.1rc1+vendor.1",
        ];
        let non_matching_versions = [
            "3.11.99.post1",
            "3.120.0a1",
            "3.13.0.post1",
            "3.13.0+vendor.1",
        ];

        for version in matching_versions {
            assert!(
                package_version_satisfies(version, "==3.12.*"),
                "expected labeled version {version:?} to match ==3.12.*"
            );
        }

        for version in non_matching_versions {
            assert!(
                !package_version_satisfies(version, "==3.12.*"),
                "expected labeled version {version:?} not to match ==3.12.*"
            );
        }
    }

    // Verify support for not equal version wildcard.

    #[test]
    fn supports_not_equal_version_wildcard() {
        assert!(package_version_satisfies("3.11.9", "!=3.12.*"));
        assert!(!package_version_satisfies("3.12.0", "!=3.12.*"));
        assert!(!package_version_satisfies("3.12.5", "!=3.12.*"));
        assert!(!package_version_satisfies("3.12", "!=3.12.*"));
        assert!(!package_version_satisfies("3.12.0a1", "!=3.12.*"));
        assert!(!package_version_satisfies("3.12.0.post1", "!=3.12.*"));
        assert!(!package_version_satisfies("3.12.0+local", "!=3.12.*"));
        assert!(package_version_satisfies("3.13.0", "!=3.12.*"));
        assert!(package_version_satisfies("3.120.0", "!=3.12.*"));
        assert!(package_version_satisfies("1!3.12.0", "!=3.12.*"));
        assert!(!package_version_satisfies("1!3.12.0", "!=1!3.12.*"));
    }

    // Verify the expected behavior for not equal version wildcard is inverse for valid operands.

    #[test]
    fn not_equal_version_wildcard_is_inverse_for_valid_operands() {
        let cases = [
            ("3.12", "3.12.*", true),
            ("3.12.0", "3.12.*", true),
            ("3.12.0.0", "3.12.*", true),
            ("3.12.1.0.7", "3.12.*", true),
            ("3.120.0", "3.12.*", false),
            ("3.13.0", "3.12.*", false),
            ("1!3.12.0", "3.12.*", false),
            ("1!3.12.0", "1!3.12.*", true),
            ("3.012.5", "3.012.*", true),
        ];

        for (version, pattern, equality_matches) in cases {
            let equality = package_version_satisfies(version, &format!("=={pattern}"));
            let inequality = package_version_satisfies(version, &format!("!={pattern}"));
            assert_eq!(
                equality, equality_matches,
                "unexpected equality result for {version:?} =={pattern}"
            );
            assert_eq!(
                inequality, !equality,
                "!= wildcard was not the inverse for {version:?} and {pattern}"
            );
        }
    }

    // Verify the expected behavior for not equal version wildcard excludes labels and rejects malformed patterns.

    #[test]
    fn not_equal_version_wildcard_excludes_labels_and_rejects_malformed_patterns() {
        let cases = [
            ("3.12.0.dev1", false),
            ("3.12.0a1", false),
            ("3.12.0b1", false),
            ("3.12.0rc1", false),
            ("3.12.0.post1", false),
            ("3.12.1+vendor.1", false),
            ("3.12.1rc1+vendor.1", false),
            ("3.11.99.post1", true),
            ("3.120.0a1", true),
            ("3.13.0.post1", true),
            ("3.13.0+vendor.1", true),
        ];

        for (version, expected) in cases {
            assert_eq!(
                package_version_satisfies(version, "!=3.12.*"),
                expected,
                "specifier !=3.12.* with candidate {version:?}"
            );
        }

        let malformed_specifiers = ["!=3.*.12", "!=3.12.*.1", "!=*.12", "!=3.12.**", "!=3.12*"];

        for specifier in malformed_specifiers {
            let error = super::package_version_satisfies("3.12.5", specifier)
                .expect_err("malformed exclusion wildcard should be rejected");
            assert!(
                error.contains(specifier),
                "expected error to identify {specifier:?}, got {error:?}"
            );
        }
    }

    // Verify the expected behavior for post releases are ordered correctly.

    #[test]
    fn post_releases_are_ordered_correctly() {
        use super::compare_versions;
        use std::cmp::Ordering;

        assert_eq!(compare_versions("1.0", "1.0.post1"), Ordering::Less);
        assert_eq!(compare_versions("1.0", "1.0.post0"), Ordering::Less);
        assert_eq!(compare_versions("1.0.post1", "1.0.post2"), Ordering::Less);
        assert_eq!(compare_versions("1.0.post2", "1.0.post10"), Ordering::Less);
        assert_eq!(
            compare_versions("1.0.post10", "1.0.post2"),
            Ordering::Greater
        );
        assert_eq!(
            compare_versions("1.0.post2", "1.0.post1"),
            Ordering::Greater
        );
        assert_eq!(compare_versions("1.0rc1", "1.0"), Ordering::Less);
    }

    // Verify the expected behavior for post releases order with pre dev and normalized spellings.

    #[test]
    fn post_releases_order_with_pre_dev_and_normalized_spellings() {
        use super::compare_versions;
        use std::cmp::Ordering;

        assert_eq!(compare_versions("1.0a1.post1", "1.0a1"), Ordering::Greater);
        assert_eq!(
            compare_versions("1.0a1.post1", "1.0a1.post2"),
            Ordering::Less
        );
        assert_eq!(
            compare_versions("1.0.post1.dev1", "1.0.post1.dev2"),
            Ordering::Less
        );
        assert_eq!(
            compare_versions("1.0.post1.dev2", "1.0.post1"),
            Ordering::Less
        );
        assert_eq!(compare_versions("1.0-r1", "1.0.post1"), Ordering::Equal);
        assert_eq!(compare_versions("1.0_rev2", "1.0.post2"), Ordering::Equal);
        assert_eq!(compare_versions("1.0-3", "1.0.post3"), Ordering::Equal);
    }

    // Verify rejection of malformed postrelease serials.

    #[test]
    fn rejects_malformed_postrelease_serials() {
        let release = PackageRelease {
            version: "1.0".to_owned(),
            requires_python: Some("<=3.8.postinvalid".to_owned()),
        };
        assert!(super::compatible_python_versions(&[release]).is_empty());

        let metadata = super::PackageMetadata {
            name: "example".to_owned(),
            latest_version: "1.0.postinvalid".to_owned(),
            requires_python: None,
            releases: vec![PackageRelease {
                version: "1.0.postinvalid".to_owned(),
                requires_python: None,
            }],
        };
        let error = select_package_release(&metadata, None)
            .expect_err("malformed release serial should be rejected");
        assert!(
            error.contains("Invalid package version '1.0.postinvalid'"),
            "expected error to identify malformed version, got {error:?}"
        );
    }

    // Verify support for basic comparison operators.

    #[test]
    fn supports_basic_comparison_operators() {
        assert!(package_version_satisfies("2.0", "==2.0"));
        assert!(!package_version_satisfies("2.1", "==2.0"));

        assert!(package_version_satisfies("2.1", "!=2.0"));
        assert!(!package_version_satisfies("2.0", "!=2.0"));

        assert!(package_version_satisfies("1.9", "<2.0"));
        assert!(!package_version_satisfies("2.0", "<2.0"));

        assert!(package_version_satisfies("2.0", "<=2.0"));
        assert!(!package_version_satisfies("2.1", "<=2.0"));

        assert!(package_version_satisfies("2.1", ">2.0"));
        assert!(!package_version_satisfies("2.0", ">2.0"));

        assert!(package_version_satisfies("2.0", ">=2.0"));
        assert!(!package_version_satisfies("1.9", ">=2.0"));
    }

    // Verify the expected behavior for equality and inequality ignore omitted trailing release zeros.

    #[test]
    fn equality_and_inequality_ignore_omitted_trailing_release_zeros() {
        let cases = [
            ("1.2", "==1.2.0", true),
            ("1.2.0", "==1.2", true),
            ("1.2.0.0", "==1.2", true),
            ("1.2", "!=1.2.0", false),
            ("1.2.0", "!=1.2", false),
            ("1.2.0.0", "!=1.2", false),
            ("1.2.1", "==1.2", false),
            ("1.2.1", "!=1.2", true),
        ];

        for (version, specifier, expected) in cases {
            assert_eq!(
                package_version_satisfies(version, specifier),
                expected,
                "specifier {specifier:?} with candidate {version:?}"
            );
        }
    }

    // Verify ordering behavior for specifiers cover below equal and above boundaries.

    #[test]
    fn ordered_specifiers_cover_below_equal_and_above_boundaries() {
        let cases = [
            ("<2.0.0", [true, false, false]),
            ("<=2.0.0", [true, true, false]),
            (">2.0.0", [false, false, true]),
            (">=2.0.0", [false, true, true]),
        ];
        let candidates = ["1.9.9", "2.0.0", "2.0.1"];

        for (specifier, expected_results) in cases {
            for (candidate, expected) in candidates.into_iter().zip(expected_results) {
                assert_eq!(
                    package_version_satisfies(candidate, specifier),
                    expected,
                    "specifier {specifier:?} with candidate {candidate:?}"
                );
            }
        }
    }

    // Verify the expected behavior for exact and ordered specifiers use pep440 versions.

    #[test]
    fn exact_and_ordered_specifiers_use_pep440_versions() {
        assert!(package_version_satisfies("1!1.0", ">2.0"));
        assert!(package_version_satisfies("1.0.post2", ">1.0.post1"));
        assert!(package_version_satisfies("1.0+local", "==1.0"));
        assert!(!package_version_satisfies("1.0+local", "!=1.0"));
    }

    // Verify version behavior for operators match nonmatch and boundary cases.

    #[test]
    fn version_operators_match_nonmatch_and_boundary_cases() {
        let cases = [
            (
                "==1.2.3",
                [("1.2.3", true), ("1.2.4", false), ("1.2.3+local", true)],
            ),
            (
                "!=1.2.3",
                [("1.2.4", true), ("1.2.3", false), ("1.2.3+local", false)],
            ),
            (
                "<1.2.3",
                [("1.2.2", true), ("1.2.3", false), ("1.2.4", false)],
            ),
            (
                "<=1.2.3",
                [("1.2.2", true), ("1.2.4", false), ("1.2.3", true)],
            ),
            (
                ">1.2.3",
                [("1.2.4", true), ("1.2.3", false), ("1.2.2", false)],
            ),
            (
                ">=1.2.3",
                [("1.2.4", true), ("1.2.2", false), ("1.2.3", true)],
            ),
            (
                "~=1.2.3",
                [("1.2.4", true), ("1.3.0", false), ("1.2.3", true)],
            ),
            (
                "==1.2.*",
                [("1.2.4", true), ("1.3.0", false), ("1.2.0a1", true)],
            ),
            (
                "!=1.2.*",
                [("1.3.0", true), ("1.2.4", false), ("1.2.0.post1", false)],
            ),
        ];

        for (specifier, candidates) in cases {
            for (candidate, expected) in candidates {
                assert_eq!(
                    package_version_satisfies(candidate, specifier),
                    expected,
                    "specifier {specifier:?} with candidate {candidate:?}"
                );
            }
        }
    }

    // Verify support for combined version specifiers.

    #[test]
    fn supports_combined_version_specifiers() {
        assert!(package_version_satisfies("2.5.0", ">=2.0,<3.0"));
        assert!(package_version_satisfies("2.5.0", ">=2.0,!=2.4.0,<3.0"));
        assert!(!package_version_satisfies("3.0.0", ">=2.0,<3.0"));
        assert!(!package_version_satisfies("2.4.0", ">=2.0,!=2.4.0,<3.0"));
    }

    // Verify comma-separated constraint behavior for separated ranges require every component to match.

    #[test]
    fn comma_separated_ranges_require_every_component_to_match() {
        assert!(package_version_satisfies("2.5.0", ">=2.0,<3.0"));
        assert!(!package_version_satisfies("1.9.9", ">=2.0,<3.0"));
        assert!(!package_version_satisfies("3.0.0", ">=2.0,<3.0"));
        assert!(super::package_version_satisfies("2.5.0", ">=2.0,<<3.0").is_err());
    }

    // Verify the expected behavior for package specifiers match pep440 reference examples.

    #[test]
    fn package_specifiers_match_pep440_reference_examples() {
        let cases = [
            ("1.1", "==1.1", true),
            ("1.1", "==1.1.0", true),
            ("1.1", "==1.1.dev1", false),
            ("1.1", "==1.1a1", false),
            ("1.1", "==1.1.post1", false),
            ("1.1", "==1.1.*", true),
            ("1.1.post1", "==1.1", false),
            ("1.1.post1", "==1.1.post1", true),
            ("1.1.post1", "==1.1.*", true),
            ("1.1.post1", "!=1.1", true),
            ("1.1.post1", "!=1.1.post1", false),
            ("1.1.post1", "!=1.1.*", false),
            ("1.7.1", ">1.7", true),
            ("1.7.0.post1", ">1.7", false),
            ("1.7.1", ">1.7.post2", true),
            ("1.7.0.post3", ">1.7.post2", true),
            ("1.7.0", ">1.7.post2", false),
        ];

        for (version, specifier, expected) in cases {
            assert_eq!(
                package_version_satisfies(version, specifier),
                expected,
                "PEP 440 case {specifier:?} with candidate {version:?}"
            );
        }
    }

    // Verify the expected behavior for compatible release operator respects release prefix.

    #[test]
    fn compatible_release_operator_respects_release_prefix() {
        assert!(package_version_satisfies("1.4.5", "~=1.4.5"));
        assert!(package_version_satisfies("1.4.9", "~=1.4.5"));
        assert!(!package_version_satisfies("1.5.0", "~=1.4.5"));

        assert!(package_version_satisfies("1.9.0", "~=1.4"));
        assert!(!package_version_satisfies("2.0.0", "~=1.4"));
    }

    // Verify the expected behavior for compatible release two segment upper bound is next major.

    #[test]
    fn compatible_release_two_segment_upper_bound_is_next_major() {
        let cases = [
            ("1.4", true),
            ("1.4.0", true),
            ("1.9.9", true),
            ("1.99.0", true),
            ("2.0", false),
            ("2.0.0", false),
            ("2.1.0", false),
        ];

        for (version, expected) in cases {
            assert_eq!(
                package_version_satisfies(version, "~=1.4"),
                expected,
                "specifier ~=1.4 with candidate {version:?}"
            );
        }
    }

    // Verify the expected behavior for compatible release three segment upper bound is next minor.

    #[test]
    fn compatible_release_three_segment_upper_bound_is_next_minor() {
        let cases = [
            ("1.4.4", false),
            ("1.4.5", true),
            ("1.4.5.0", true),
            ("1.4.9", true),
            ("1.4.99.99", true),
            ("1.5", false),
            ("1.5.0", false),
            ("1.5.1", false),
        ];

        for (version, expected) in cases {
            assert_eq!(
                package_version_satisfies(version, "~=1.4.5"),
                expected,
                "specifier ~=1.4.5 with candidate {version:?}"
            );
        }
    }

    // Verify the expected behavior for compatible release rejects invalid operands with clear errors.

    #[test]
    fn compatible_release_rejects_invalid_operands_with_clear_errors() {
        let invalid_specifiers = ["~=1", "~=1.*", "~=1.4.*", "~=1.4+local"];

        for specifier in invalid_specifiers {
            let error = super::package_version_satisfies("1.4.5", specifier)
                .expect_err("invalid compatible-release operand should be rejected");
            assert!(
                error.contains("Invalid package version specifier"),
                "expected a clear specifier error, got {error:?}"
            );
            assert!(
                error.contains(specifier),
                "expected the error to identify {specifier:?}, got {error:?}"
            );
        }
    }

    // Verify the expected behavior for compatible release compares pre and post release candidates.

    #[test]
    fn compatible_release_compares_pre_and_post_release_candidates() {
        let cases = [
            ("1.4.5.dev1", false),
            ("1.4.5a1", false),
            ("1.4.5rc1", false),
            ("1.4.5", true),
            ("1.4.5.post1", true),
            ("1.4.6a1", true),
            ("1.4.6.post1", true),
            ("1.5.0a1", false),
            ("1.5.0.post1", false),
        ];

        for (version, expected) in cases {
            assert_eq!(
                package_version_satisfies(version, "~=1.4.5"),
                expected,
                "specifier ~=1.4.5 with candidate {version:?}"
            );
        }
    }

    // Verify rejection of invalid version specifiers.

    #[test]
    fn rejects_invalid_version_specifiers() {
        assert!(super::package_version_satisfies("2.0.0", "=>2.0.0").is_err());
        assert!(super::package_version_satisfies("2.0.0", "><2.0.0").is_err());
        assert!(super::package_version_satisfies("2.0.0", "").is_err());
        assert!(super::package_version_satisfies("2.0.0", "==").is_err());
        assert!(super::package_version_satisfies("2.0.0", ">=1.0,,<3.0").is_err());
        assert!(super::package_version_satisfies("2.0.0", ">=1.0,").is_err());
        assert!(super::package_version_satisfies("2.0.0", ">=1.0,<").is_err());
    }

    // Verify rejection of malformed and unsupported version wildcards.

    #[test]
    fn rejects_malformed_and_unsupported_version_wildcards() {
        let invalid_specifiers = [
            "==3.*.12",
            "==3.12.*.1",
            "==*.12",
            "==3.12.**",
            "==3.12*",
            "<3.12.*",
            "<=3.12.*",
            ">3.12.*",
            ">=3.12.*",
            "~=3.12.*",
            "===3.12.*",
        ];

        for specifier in invalid_specifiers {
            let error = super::package_version_satisfies("3.12.5", specifier)
                .expect_err("malformed or unsupported wildcard should be rejected");
            assert!(
                error.contains(specifier),
                "expected error to identify {specifier:?}, got {error:?}"
            );
        }
    }

    // Verify rejection of malformed package version operands.

    #[test]
    fn rejects_malformed_package_version_operands() {
        assert!(super::package_version_satisfies("2.0.0", "==2.invalid").is_err());

        let metadata = super::PackageMetadata {
            name: "example".to_owned(),
            latest_version: "2.0.0".to_owned(),
            requires_python: None,
            releases: vec![super::PackageRelease {
                version: "2.0.0".to_owned(),
                requires_python: None,
            }],
        };

        assert!(select_package_release(&metadata, Some(">=2.invalid")).is_err());
    }

    // Verify removal of inline requirement comments.

    #[test]
    fn strips_inline_requirement_comments() {
        assert_eq!(
            super::strip_inline_comment("requests>=2.0  # minimum version"),
            "requests>=2.0"
        );

        assert_eq!(
            super::strip_inline_comment("mypackage @ https://example.com/pkg.whl#sha256=abc123"),
            "mypackage @ https://example.com/pkg.whl#sha256=abc123"
        );

        assert_eq!(
            super::strip_inline_comment(
                "mypackage @ https://example.com/pkg.whl#sha256=abc123  # comment"
            ),
            "mypackage @ https://example.com/pkg.whl#sha256=abc123"
        );
    }

    // Verify handling of whitespace in requirements.

    #[test]
    fn handles_whitespace_in_requirements() {
        let project = temporary_project();
        let requirements = project.join("requirements.txt");

        fs::write(
            &requirements,
            "  requests>=2.0  \n\tflask==3.0\t\n\n   # comment\n",
        )
        .unwrap();

        let dependencies = discover_dependencies(&project).unwrap();

        assert_eq!(dependencies.len(), 2);
        assert_eq!(dependencies[0].name, "flask");
        assert_eq!(dependencies[0].specifier.as_deref(), Some("==3.0"));
        assert_eq!(dependencies[1].name, "requests");
        assert_eq!(dependencies[1].specifier.as_deref(), Some(">=2.0"));

        fs::remove_dir_all(project).unwrap();
    }

    // Verify handling of crlf line endings equivalently to lf.

    #[test]
    fn handles_crlf_line_endings_equivalently_to_lf() {
        let project = temporary_project();
        let requirements = project.join("requirements.txt");

        fs::write(&requirements, "requests>=2.0\nflask==3.0\n").unwrap();
        let lf_dependencies = discover_dependencies(&project).unwrap();

        fs::write(&requirements, "requests>=2.0\r\nflask==3.0\r\n").unwrap();
        let crlf_dependencies = discover_dependencies(&project).unwrap();

        assert_eq!(crlf_dependencies, lf_dependencies);

        fs::remove_dir_all(project).unwrap();
    }

    // Verify handling of mixed line endings.

    #[test]
    fn handles_mixed_line_endings() {
        let project = temporary_project();
        let requirements = project.join("requirements.txt");

        fs::write(
            &requirements,
            "requests>=2.0\r\nflask==3.0\nuvicorn>=0.30\r\n",
        )
        .unwrap();

        let dependencies = discover_dependencies(&project).unwrap();

        assert_eq!(dependencies.len(), 3);
        assert_eq!(
            dependencies
                .iter()
                .map(|dependency| dependency.name.as_str())
                .collect::<Vec<_>>(),
            vec!["flask", "requests", "uvicorn"]
        );

        fs::remove_dir_all(project).unwrap();
    }

    // Verify handling of backslash line continuations.

    #[test]
    fn handles_backslash_line_continuations() {
        let project = temporary_project();
        let requirements = project.join("requirements.txt");

        fs::write(&requirements, "requests>=2.0,\\\n    <3.0\nflask==3.0\n").unwrap();

        let dependencies = discover_dependencies(&project).unwrap();

        assert_eq!(dependencies.len(), 2);

        let requests = dependencies
            .iter()
            .find(|dependency| dependency.name == "requests")
            .unwrap();

        assert_eq!(requests.specifier.as_deref(), Some(">=2.0, <3.0"));

        fs::remove_dir_all(project).unwrap();
    }

    // Verify handling of multiple continuation lines and crlf.

    #[test]
    fn handles_multiple_continuation_lines_and_crlf() {
        let project = temporary_project();
        let requirements = project.join("requirements.txt");

        fs::write(
            &requirements,
            "requests>=2.0,\\\r\n    <3.0,\\\r\n    !=2.5.0\r\n",
        )
        .unwrap();

        let dependencies = discover_dependencies(&project).unwrap();

        assert_eq!(dependencies.len(), 1);
        assert_eq!(
            dependencies[0].specifier.as_deref(),
            Some(">=2.0, <3.0, !=2.5.0")
        );

        fs::remove_dir_all(project).unwrap();
    }

    // Verify handling of comments after continuation lines.

    #[test]
    fn handles_comments_after_continuation_lines() {
        let project = temporary_project();
        let requirements = project.join("requirements.txt");

        fs::write(
            &requirements,
            "# comment\nrequests>=2.0,\\\n    <3.0  # upper bound\n\nflask==3.0\n",
        )
        .unwrap();

        let dependencies = discover_dependencies(&project).unwrap();

        assert_eq!(dependencies.len(), 2);

        let requests = dependencies
            .iter()
            .find(|dependency| dependency.name == "requests")
            .unwrap();

        assert_eq!(requests.specifier.as_deref(), Some(">=2.0, <3.0"));

        fs::remove_dir_all(project).unwrap();
    }

    // Verify parsing requirement without extras.

    #[test]
    fn parses_requirement_without_extras() {
        assert_eq!(
            parse_requirement("requests").unwrap(),
            ParsedRequirement {
                name: "requests".to_string(),
                extras: vec![],
                specifier: String::new(),
                url: None,
            }
        );
    }

    // Verify parsing requirement with one extra.

    #[test]
    fn parses_requirement_with_one_extra() {
        assert_eq!(
            parse_requirement("requests[security]").unwrap(),
            ParsedRequirement {
                name: "requests".to_string(),
                extras: vec!["security".to_string()],
                specifier: String::new(),
                url: None,
            }
        );
    }

    // Verify parsing requirement with multiple extras.

    #[test]
    fn parses_requirement_with_multiple_extras() {
        assert_eq!(
            parse_requirement("requests[security,socks]").unwrap(),
            ParsedRequirement {
                name: "requests".to_string(),
                extras: vec!["security".to_string(), "socks".to_string(),],
                specifier: String::new(),
                url: None,
            }
        );
    }

    // Verify parsing extras with version specifier.

    #[test]
    fn parses_extras_with_version_specifier() {
        assert_eq!(
            parse_requirement("requests[security]>=2.31.0").unwrap(),
            ParsedRequirement {
                name: "requests".to_string(),
                extras: vec!["security".to_string()],
                specifier: ">=2.31.0".to_string(),
                url: None,
            }
        );
    }

    // Verify parsing extras with whitespace.

    #[test]
    fn parses_extras_with_whitespace() {
        assert_eq!(
            parse_requirement("requests[security, socks]").unwrap(),
            ParsedRequirement {
                name: "requests".to_string(),
                extras: vec!["security".to_string(), "socks".to_string(),],
                specifier: String::new(),
                url: None,
            }
        );
    }

    // Verify rejection of empty extras.

    #[test]
    fn rejects_empty_extras() {
        assert!(parse_requirement("requests[]").is_err());
    }

    // Verify rejection of missing closing bracket.

    #[test]
    fn rejects_missing_closing_bracket() {
        assert!(parse_requirement("requests[security").is_err());
    }

    // Verify rejection of empty extra between commas.

    #[test]
    fn rejects_empty_extra_between_commas() {
        assert!(parse_requirement("requests[security,,socks]").is_err());
    }

    // Verify rejection of empty requirement.

    #[test]
    fn rejects_empty_requirement() {
        assert!(parse_requirement("").is_err());
    }

    // Verify discovery of requirement with one extra.

    #[test]
    fn discovers_requirement_with_one_extra() {
        let project = temporary_project();
        let requirements = project.join("requirements.txt");

        fs::write(&requirements, "requests[security]\n").unwrap();

        let dependencies = discover_dependencies(&project).unwrap();

        assert_eq!(dependencies.len(), 1);
        assert_eq!(dependencies[0].name, "requests");
        assert_eq!(dependencies[0].extras, vec!["security"]);
        assert_eq!(dependencies[0].specifier, None);
        assert_eq!(dependencies[0].marker, None);

        fs::remove_dir_all(project).unwrap();
    }

    // Verify discovery of requirement with multiple extras and specifier.

    #[test]
    fn discovers_requirement_with_multiple_extras_and_specifier() {
        let project = temporary_project();
        let requirements = project.join("requirements.txt");

        fs::write(&requirements, "requests[security,socks]>=2.31.0\n").unwrap();

        let dependencies = discover_dependencies(&project).unwrap();

        assert_eq!(dependencies.len(), 1);
        assert_eq!(dependencies[0].name, "requests");
        assert_eq!(dependencies[0].extras, vec!["security", "socks"]);
        assert_eq!(dependencies[0].specifier.as_deref(), Some(">=2.31.0"));

        fs::remove_dir_all(project).unwrap();
    }

    // Verify discovery of requirement with extra and environment marker.

    #[test]
    fn discovers_requirement_with_extra_and_environment_marker() {
        let project = temporary_project();
        let requirements = project.join("requirements.txt");

        fs::write(
            &requirements,
            "requests[security]>=2.31.0; python_version >= \"3.10\"\n",
        )
        .unwrap();

        let dependencies = discover_dependencies(&project).unwrap();

        assert_eq!(dependencies.len(), 1);
        assert_eq!(dependencies[0].name, "requests");
        assert_eq!(dependencies[0].extras, vec!["security"]);
        assert_eq!(dependencies[0].specifier.as_deref(), Some(">=2.31.0"));
        assert_eq!(
            dependencies[0].marker.as_deref(),
            Some("python_version >= \"3.10\"")
        );

        fs::remove_dir_all(project).unwrap();
    }

    // Verify the expected behavior for keeps dependencies with different extras distinct.

    #[test]
    fn keeps_dependencies_with_different_extras_distinct() {
        let project = temporary_project();
        let requirements = project.join("requirements.txt");

        fs::write(&requirements, "requests[security]\nrequests[socks]\n").unwrap();

        let dependencies = discover_dependencies(&project).unwrap();

        assert_eq!(dependencies.len(), 2);
        assert!(dependencies.iter().any(|dependency| {
            dependency.name == "requests" && dependency.extras == vec!["security"]
        }));
        assert!(dependencies.iter().any(|dependency| {
            dependency.name == "requests" && dependency.extras == vec!["socks"]
        }));

        fs::remove_dir_all(project).unwrap();
    }

    // Verify the expected behavior for canonical names define dependency identity and duplicate detection.

    #[test]
    fn canonical_names_define_dependency_identity_and_duplicate_detection() {
        let project = temporary_project();
        let requirements = project.join("requirements.txt");

        fs::write(
            &requirements,
            "my_package\nmy.package\nMY-PACKAGE\nmy_package>=1.0\n",
        )
        .unwrap();

        let dependencies = discover_dependencies(&project).unwrap();

        assert_eq!(dependencies.len(), 2);
        assert!(
            dependencies
                .iter()
                .all(|dependency| dependency.name == "my-package")
        );
        assert_eq!(
            dependencies
                .iter()
                .filter(|dependency| dependency.specifier.is_none())
                .count(),
            1
        );
        assert!(
            dependencies
                .iter()
                .any(|dependency| dependency.specifier.as_deref() == Some(">=1.0"))
        );

        fs::remove_dir_all(project).unwrap();
    }

    // Verify the expected behavior for ignores malformed extras.

    #[test]
    fn ignores_malformed_extras() {
        let project = temporary_project();
        let requirements = project.join("requirements.txt");

        fs::write(
            &requirements,
            "requests[]\nrequests[security\nrequests[security,,socks]\nrequests[security]junk\n",
        )
        .unwrap();

        let dependencies = discover_dependencies(&project).unwrap();

        assert!(dependencies.is_empty());

        fs::remove_dir_all(project).unwrap();
    }

    // Verify parsing requirement with direct url.

    #[test]
    fn parses_requirement_with_direct_url() {
        assert_eq!(
            parse_requirement("mypackage @ https://example.com/pkg.whl#sha256=abc123").unwrap(),
            ParsedRequirement {
                name: "mypackage".to_owned(),
                extras: vec![],
                specifier: String::new(),
                url: Some("https://example.com/pkg.whl#sha256=abc123".to_owned()),
            }
        );
    }

    // Verify parsing requirement with extras and direct url.

    #[test]
    fn parses_requirement_with_extras_and_direct_url() {
        assert_eq!(
            parse_requirement("mypackage[security,socks] @ git+https://example.com/repo.git")
                .unwrap(),
            ParsedRequirement {
                name: "mypackage".to_owned(),
                extras: vec!["security".to_owned(), "socks".to_owned()],
                specifier: String::new(),
                url: Some("git+https://example.com/repo.git".to_owned()),
            }
        );
    }

    // Verify rejection of requirement with invalid direct url.

    #[test]
    fn rejects_requirement_with_invalid_direct_url() {
        assert!(parse_requirement("mypackage @ not-a-url").is_err());
    }

    #[test]
    fn rejects_direct_url_with_missing_authority() {
        assert!(parse_requirement("mypackage @ https://").is_err())
    }

    // Verify discovery of requirement with direct url and marker.

    #[test]
    fn discovers_requirement_with_direct_url_and_marker() {
        let project = temporary_project();
        let requirements = project.join("requirements.txt");

        fs::write(
        &requirements,
        "mypackage[security] @ https://example.com/pkg.whl#sha256=abc123; python_version >= \"3.10\"\n",
    )
    .unwrap();

        let dependencies = discover_dependencies(&project).unwrap();

        assert_eq!(dependencies.len(), 1);
        assert_eq!(dependencies[0].name, "mypackage");
        assert_eq!(dependencies[0].extras, vec!["security"]);
        assert_eq!(
            dependencies[0].url.as_deref(),
            Some("https://example.com/pkg.whl#sha256=abc123")
        );
        assert_eq!(dependencies[0].specifier, None);
        assert_eq!(
            dependencies[0].marker.as_deref(),
            Some("python_version >= \"3.10\"")
        );

        fs::remove_dir_all(project).unwrap();
    }

    // Verify parsing vcs reference with subdirectory fragment.

    #[test]
    fn parses_vcs_reference_with_subdirectory_fragment() {
        let input =
            "mypackage @ git+https://github.com/example/repo.git@main#subdirectory=python_pkg";

        let parsed = parse_requirement(input).unwrap();

        assert_eq!(parsed.name, "mypackage");
        assert_eq!(
            parsed.url.as_deref(),
            Some("git+https://github.com/example/repo.git@main#subdirectory=python_pkg")
        );
        assert!(parsed.specifier.is_empty());
    }

    // Verify the expected behavior for preserves vcs reference with revision and egg fragment.

    #[test]
    fn preserves_vcs_reference_with_revision_and_egg_fragment() {
        let input = "mypackage @ git+https://github.com/example/repo.git@v1.2.3#egg=mypackage";

        let parsed = parse_requirement(input).unwrap();

        assert_eq!(parsed.name, "mypackage");
        assert_eq!(
            parsed.url.as_deref(),
            Some("git+https://github.com/example/repo.git@v1.2.3#egg=mypackage")
        );
        assert!(parsed.specifier.is_empty());
    }

    // Verify the expected behavior for preserves encoded subdirectory fragment.

    #[test]
    fn preserves_encoded_subdirectory_fragment() {
        let input = "mypackage @ git+https://github.com/example/repo.git@main#subdirectory=packages%2Fpython_pkg";

        let parsed = parse_requirement(input).unwrap();

        assert_eq!(
            parsed.url.as_deref(),
            Some("git+https://github.com/example/repo.git@main#subdirectory=packages%2Fpython_pkg")
        );
    }

    // Verify discovery of vcs reference with subdirectory fragment.

    #[test]
    fn discovers_vcs_reference_with_subdirectory_fragment() {
        let project = temporary_project();
        let requirements = project.join("requirements.txt");

        fs::write(
            &requirements,
            "mypackage @ git+https://github.com/example/repo.git@main#subdirectory=python_pkg\n",
        )
        .unwrap();

        let dependencies = discover_dependencies(&project).unwrap();

        assert_eq!(dependencies.len(), 1);
        assert_eq!(dependencies[0].name, "mypackage");
        assert_eq!(
            dependencies[0].url.as_deref(),
            Some("git+https://github.com/example/repo.git@main#subdirectory=python_pkg")
        );

        fs::remove_dir_all(project).unwrap();
    }

    // Verify discovery of editable git requirement.

    #[test]
    fn discovers_editable_git_requirement() {
        let project = temporary_project();
        let requirements = project.join("requirements.txt");

        fs::write(
            &requirements,
            "-e git+https://github.com/example/repo.git@main#egg=mypackage\n",
        )
        .unwrap();

        let dependencies = discover_dependencies(&project).unwrap();

        assert_eq!(dependencies.len(), 1);
        assert_eq!(dependencies[0].name, "mypackage");
        assert_eq!(
            dependencies[0].url.as_deref(),
            Some("git+https://github.com/example/repo.git@main#egg=mypackage")
        );

        fs::remove_dir_all(project).unwrap();
    }

    // Verify discovery of long form editable git requirement.

    #[test]
    fn discovers_long_form_editable_git_requirement() {
        let project = temporary_project();
        let requirements = project.join("requirements.txt");

        fs::write(
            &requirements,
            "--editable git+https://github.com/example/repo.git@main#egg=mypackage\n",
        )
        .unwrap();

        let dependencies = discover_dependencies(&project).unwrap();

        assert_eq!(dependencies.len(), 1);
        assert_eq!(dependencies[0].name, "mypackage");
        assert_eq!(
            dependencies[0].url.as_deref(),
            Some("git+https://github.com/example/repo.git@main#egg=mypackage")
        );

        fs::remove_dir_all(project).unwrap();
    }

    // Verify discovery of editable git requirement with extras.

    #[test]
    fn discovers_editable_git_requirement_with_extras() {
        let project = temporary_project();
        let requirements = project.join("requirements.txt");

        fs::write(
            &requirements,
            "-e git+https://github.com/example/repo.git@main#egg=mypackage[security]\n",
        )
        .unwrap();

        let dependencies = discover_dependencies(&project).unwrap();

        assert_eq!(dependencies.len(), 1);
        assert_eq!(dependencies[0].name, "mypackage");
        assert_eq!(dependencies[0].extras, vec!["security"]);
        assert_eq!(
            dependencies[0].url.as_deref(),
            Some("git+https://github.com/example/repo.git@main#egg=mypackage[security]")
        );

        fs::remove_dir_all(project).unwrap();
    }

    // Verify discovery of editable git requirement with subdirectory.

    #[test]
    fn discovers_editable_git_requirement_with_subdirectory() {
        let project = temporary_project();
        let requirements = project.join("requirements.txt");

        fs::write(
        &requirements,
        "-e git+https://github.com/example/repo.git@main#egg=mypackage&subdirectory=python_pkg\n",
    )
    .unwrap();

        let dependencies = discover_dependencies(&project).unwrap();

        assert_eq!(dependencies.len(), 1);
        assert_eq!(dependencies[0].name, "mypackage");
        assert_eq!(
            dependencies[0].url.as_deref(),
            Some(
                "git+https://github.com/example/repo.git@main#egg=mypackage&subdirectory=python_pkg"
            )
        );

        fs::remove_dir_all(project).unwrap();
    }

    // Verify discovery of mercurial vcs reference.

    #[test]
    fn discovers_mercurial_vcs_reference() {
        let project = temporary_project();
        fs::write(
            project.join("requirements.txt"),
            "mypackage @ hg+https://example.com/repo\n",
        )
        .unwrap();

        let dependencies = discover_dependencies(&project).unwrap();

        assert_eq!(dependencies.len(), 1);
        assert_eq!(dependencies[0].name, "mypackage");
        assert_eq!(
            dependencies[0].url.as_deref(),
            Some("hg+https://example.com/repo")
        );

        fs::remove_dir_all(project).unwrap();
    }

    // Verify discovery of subversion vcs reference.

    #[test]
    fn discovers_subversion_vcs_reference() {
        let project = temporary_project();
        fs::write(
            project.join("requirements.txt"),
            "mypackage @ svn+https://example.com/repo\n",
        )
        .unwrap();

        let dependencies = discover_dependencies(&project).unwrap();

        assert_eq!(dependencies.len(), 1);
        assert_eq!(dependencies[0].name, "mypackage");
        assert_eq!(
            dependencies[0].url.as_deref(),
            Some("svn+https://example.com/repo")
        );

        fs::remove_dir_all(project).unwrap();
    }

    // Verify discovery of bazaar vcs reference.

    #[test]
    fn discovers_bazaar_vcs_reference() {
        let project = temporary_project();
        fs::write(
            project.join("requirements.txt"),
            "mypackage @ bzr+https://example.com/repo\n",
        )
        .unwrap();

        let dependencies = discover_dependencies(&project).unwrap();

        assert_eq!(dependencies.len(), 1);
        assert_eq!(dependencies[0].name, "mypackage");
        assert_eq!(
            dependencies[0].url.as_deref(),
            Some("bzr+https://example.com/repo")
        );

        fs::remove_dir_all(project).unwrap();
    }

    // Verify discovery of git references with different revision formats.

    #[test]
    fn discovers_git_references_with_different_revision_formats() {
        let project = temporary_project();
        fs::write(
            project.join("requirements.txt"),
            concat!(
                "branchpkg @ git+https://example.com/branch.git@main\n",
                "tagpkg @ git+https://example.com/tag.git@v1.2.3\n",
                "commitpkg @ git+https://example.com/commit.git@0123456789abcdef\n",
            ),
        )
        .unwrap();

        let dependencies = discover_dependencies(&project).unwrap();

        assert_eq!(dependencies.len(), 3);
        assert!(dependencies.iter().any(|d| d.name == "branchpkg"));
        assert!(dependencies.iter().any(|d| d.name == "tagpkg"));
        assert!(dependencies.iter().any(|d| d.name == "commitpkg"));

        fs::remove_dir_all(project).unwrap();
    }

    // Verify discovery of editable git requirement with reordered fragments.

    #[test]
    fn discovers_editable_git_requirement_with_reordered_fragments() {
        let project = temporary_project();
        fs::write(
            project.join("requirements.txt"),
            "-e git+https://example.com/repo.git#subdirectory=python_pkg&egg=mypackage\n",
        )
        .unwrap();

        let dependencies = discover_dependencies(&project).unwrap();

        assert_eq!(dependencies.len(), 1);
        assert_eq!(dependencies[0].name, "mypackage");

        fs::remove_dir_all(project).unwrap();
    }

    // Verify the expected behavior for skips editable git requirement with malformed egg extras.

    #[test]
    fn skips_editable_git_requirement_with_malformed_egg_extras() {
        let project = temporary_project();
        fs::write(
            project.join("requirements.txt"),
            "-e git+https://example.com/repo.git#egg=mypackage[security\n",
        )
        .unwrap();

        let dependencies = discover_dependencies(&project).unwrap();

        assert!(dependencies.is_empty());

        fs::remove_dir_all(project).unwrap();
    }

    // Verify the expected behavior for skips editable local path without package metadata support.

    #[test]
    fn skips_editable_local_path_without_package_metadata_support() {
        let project = temporary_project();
        fs::write(project.join("requirements.txt"), "-e .\n-e ./my_package\n").unwrap();

        let dependencies = discover_dependencies(&project).unwrap();

        // Documents current behavior: local editable paths aren't
        // normalized into named dependencies by the current helper.
        assert!(dependencies.is_empty());

        fs::remove_dir_all(project).unwrap();
    }

    // Verify discovery of requirements with line continuation.

    #[test]
    fn discovers_requirements_with_line_continuation() {
        let project = temporary_project();
        fs::write(
            project.join("requirements.txt"),
            "mypackage>=1.0,\\\n<2.0\n",
        )
        .unwrap();

        let dependencies = discover_dependencies(&project).unwrap();

        assert_eq!(dependencies.len(), 1);
        assert_eq!(dependencies[0].name, "mypackage");
        assert_eq!(dependencies[0].specifier.as_deref(), Some(">=1.0, <2.0"));

        fs::remove_dir_all(project).unwrap();
    }

    // Verify the expected behavior for skips requirements options without skipping valid dependencies.

    #[test]
    fn skips_requirements_options_without_skipping_valid_dependencies() {
        let project = temporary_project();

        fs::write(
            project.join("requirements.txt"),
            "--index-url https://packages.example.com/simple\n\
         requests>=2.0\n\
         --extra-index-url https://backup.example.com/simple\n\
         flask==3.0\n",
        )
        .unwrap();

        let dependencies = discover_dependencies(&project).unwrap();

        assert_eq!(dependencies.len(), 2);
        assert!(
            dependencies
                .iter()
                .any(|d| { d.name == "requests" && d.specifier.as_deref() == Some(">=2.0") })
        );
        assert!(
            dependencies
                .iter()
                .any(|d| { d.name == "flask" && d.specifier.as_deref() == Some("==3.0") })
        );

        fs::remove_dir_all(project).unwrap();
    }

    // Verify handling of editable requirements with requirement without trailing newline is handled.

    #[test]
    fn editable_requirement_without_trailing_newline_is_handled() {
        let project = temporary_project();
        fs::write(
            project.join("requirements.txt"),
            "-e git+https://example.com/repo.git@main#egg=mypackage",
        )
        .unwrap();

        let dependencies = discover_dependencies(&project).unwrap();

        assert_eq!(dependencies.len(), 1);
        assert_eq!(dependencies[0].name, "mypackage");

        fs::remove_dir_all(project).unwrap();
    }

    // Verify parsing supported vcs direct urls.

    #[test]
    fn parses_supported_vcs_direct_urls() {
        for (vcs, transport) in [
            ("git", "https"),
            ("git", "http"),
            ("git", "ssh"),
            ("git", "file"),
            ("hg", "https"),
            ("hg", "http"),
            ("hg", "ssh"),
            ("hg", "file"),
            ("svn", "https"),
            ("svn", "http"),
            ("svn", "ssh"),
            ("svn", "file"),
            ("bzr", "https"),
            ("bzr", "http"),
            ("bzr", "ssh"),
            ("bzr", "file"),
        ] {
            let input = format!("mypackage @ {vcs}+{transport}://example.com/repo");

            let parsed = parse_requirement(&input)
                .unwrap_or_else(|error| panic!("Failed to parse {input}: {error}"));

            assert_eq!(parsed.name, "mypackage");
            assert_eq!(
                parsed.url.as_deref(),
                Some(format!("{vcs}+{transport}://example.com/repo").as_str())
            );
        }
    }

    // Verify rejection of unsupported vcs direct urls.

    #[test]
    fn rejects_unsupported_vcs_direct_urls() {
        for url in [
            "cvs+https://example.com/repo",
            "git+ftp://example.com/repo",
            "hg+ftp://example.com/repo",
            "svn+ftp://example.com/repo",
            "bzr+ftp://example.com/repo",
        ] {
            let input = format!("mypackage @ {url}");
            assert!(
                parse_requirement(&input).is_err(),
                "Expected unsupported URL to be rejected: {input}"
            );
        }
    }
    // Verify the expected behavior for accepts valid compound environment markers.
    #[test]
    fn accepts_valid_compound_environment_markers() {
        let project = temporary_project();

        fs::write(
            project.join("requirements.txt"),
            "requests>=2.0; python_version >= \"3.10\" and sys_platform == \"win32\"\n",
        )
        .unwrap();

        let dependencies = discover_dependencies(&project).unwrap();

        assert_eq!(dependencies.len(), 1);
        assert_eq!(dependencies[0].name, "requests");
        assert_eq!(
            dependencies[0].marker.as_deref(),
            Some("python_version >= \"3.10\" and sys_platform == \"win32\"")
        );

        fs::remove_dir_all(project).unwrap();
    }

    // Verify the expected behavior for reports invalid requirements environment markers with source location.

    #[test]
    fn reports_invalid_requirements_environment_markers_with_source_location() {
        let project = temporary_project();

        fs::write(
            project.join("requirements.txt"),
            "requests>=2.0; python_version >>> \"3.10\"\nflask==3.0\n",
        )
        .unwrap();

        let error = discover_dependencies(&project).unwrap_err();

        assert!(error.contains("requirements.txt:1"));
        assert!(error.contains("python_version >>>"));

        fs::remove_dir_all(project).unwrap();
    }

    // Verify dependency-marker behavior for marker matches python version.

    #[test]
    fn dependency_marker_matches_python_version() {
        let dependency = Dependency {
            name: "requests".to_owned(),
            extras: vec![],
            specifier: None,
            url: None,
            marker: Some("python_version >= \"3.10\"".to_owned()),
            source: std::path::PathBuf::from("requirements.txt"),
        };

        let environment = test_marker_environment("3.12", "linux");

        assert!(dependency_applies_to_environment(&dependency, &environment).unwrap());
    }

    // Verify dependency-marker behavior for marker rejects mismatched python version.

    #[test]
    fn dependency_marker_rejects_mismatched_python_version() {
        let dependency = Dependency {
            name: "requests".to_owned(),
            extras: vec![],
            specifier: None,
            url: None,
            marker: Some("python_version >= \"3.10\"".to_owned()),
            source: std::path::PathBuf::from("requirements.txt"),
        };

        let environment = test_marker_environment("3.9", "linux");

        assert!(!dependency_applies_to_environment(&dependency, &environment).unwrap());
    }

    // Verify dependency-marker behavior for marker checks platform and compound conditions.

    #[test]
    fn dependency_marker_checks_platform_and_compound_conditions() {
        let dependency = Dependency {
            name: "pywin32".to_owned(),
            extras: vec![],
            specifier: None,
            url: None,
            marker: Some("python_version >= \"3.10\" and sys_platform == \"win32\"".to_owned()),
            source: std::path::PathBuf::from("requirements.txt"),
        };

        let windows = test_marker_environment("3.12", "win32");
        let linux = test_marker_environment("3.12", "linux");

        assert!(dependency_applies_to_environment(&dependency, &windows).unwrap());
        assert!(!dependency_applies_to_environment(&dependency, &linux).unwrap());
    }

    // Verify dependency-marker behavior for without marker applies to every environment.

    #[test]
    fn dependency_without_marker_applies_to_every_environment() {
        let dependency = Dependency {
            name: "requests".to_owned(),
            extras: vec![],
            specifier: None,
            url: None,
            marker: None,
            source: std::path::PathBuf::from("requirements.txt"),
        };

        let environment = test_marker_environment("3.9", "linux");

        assert!(dependency_applies_to_environment(&dependency, &environment).unwrap());
    }

    // Verify the expected behavior for compatibility ignores dependencies for other platforms.

    #[test]
    fn compatibility_ignores_dependencies_for_other_platforms() {
        let dependency = Dependency {
            name: "pywin32".to_owned(),
            extras: vec![],
            specifier: None,
            url: None,
            marker: Some("sys_platform == \"win32\"".to_owned()),
            source: std::path::PathBuf::from("requirements.txt"),
        };

        let release = PackageRelease {
            version: "1.0.0".to_owned(),
            requires_python: Some(">=3.12".to_owned()),
        };

        let linux = test_marker_environment("3.12", "linux");

        let compatible = super::compatible_python_versions_for_environment(
            &[dependency],
            &[Some(release)],
            &linux,
        )
        .unwrap();

        assert_eq!(
            compatible,
            vec![
                "3.7".to_owned(),
                "3.8".to_owned(),
                "3.9".to_owned(),
                "3.10".to_owned(),
                "3.11".to_owned(),
                "3.12".to_owned(),
                "3.13".to_owned(),
                "3.14".to_owned(),
                "3.15".to_owned(),
            ]
        );
    }

    // Verify dependency-marker behavior for marker respects parentheses and boolean precedence.

    #[test]
    fn dependency_marker_respects_parentheses_and_boolean_precedence() {
        let dependency = Dependency {
            name: "example".to_owned(),
            extras: vec![],
            specifier: None,
            url: None,
            marker: Some(
                "(python_version >= \"3.10\" and sys_platform == \"win32\") \
             or python_version == \"3.9\""
                    .to_owned(),
            ),
            source: std::path::PathBuf::from("requirements.txt"),
        };

        let windows = test_marker_environment("3.12", "win32");
        let linux = test_marker_environment("3.12", "linux");
        let python39 = test_marker_environment("3.9", "linux");

        assert!(dependency_applies_to_environment(&dependency, &windows).unwrap());
        assert!(!dependency_applies_to_environment(&dependency, &linux).unwrap());
        assert!(dependency_applies_to_environment(&dependency, &python39).unwrap());
    }

    // Verify dependency-marker behavior for marker supports in and not in.

    #[test]
    fn dependency_marker_supports_in_and_not_in() {
        let included = Dependency {
            name: "example".to_owned(),
            extras: vec![],
            specifier: None,
            url: None,
            marker: Some("sys_platform in \"linux win32\"".to_owned()),
            source: std::path::PathBuf::from("requirements.txt"),
        };

        let excluded = Dependency {
            marker: Some("sys_platform not in \"linux darwin\"".to_owned()),
            ..included.clone()
        };

        let linux = test_marker_environment("3.12", "linux");
        let windows = test_marker_environment("3.12", "win32");

        assert!(dependency_applies_to_environment(&included, &linux).unwrap());
        assert!(dependency_applies_to_environment(&included, &windows).unwrap());

        assert!(!dependency_applies_to_environment(&excluded, &linux).unwrap());
        assert!(dependency_applies_to_environment(&excluded, &windows).unwrap());
    }

    // Verify dependency-marker behavior for marker checks python full version boundaries.

    #[test]
    fn dependency_marker_checks_python_full_version_boundaries() {
        let dependency = Dependency {
            name: "example".to_owned(),
            extras: vec![],
            specifier: None,
            url: None,
            marker: Some("python_full_version >= \"3.12.0\"".to_owned()),
            source: std::path::PathBuf::from("requirements.txt"),
        };

        let python312 = test_marker_environment("3.12", "linux");
        let python39 = test_marker_environment("3.9", "linux");

        assert!(dependency_applies_to_environment(&dependency, &python312).unwrap());
        assert!(!dependency_applies_to_environment(&dependency, &python39).unwrap());
    }

    // Verify dependency-marker behavior for marker returns error for invalid expression.

    #[test]
    fn dependency_marker_returns_error_for_invalid_expression() {
        let dependency = Dependency {
            name: "example".to_owned(),
            extras: vec![],
            specifier: None,
            url: None,
            marker: Some("python_version >>> \"3.10\"".to_owned()),
            source: std::path::PathBuf::from("requirements.txt"),
        };

        let environment = test_marker_environment("3.12", "linux");

        assert!(dependency_applies_to_environment(&dependency, &environment).is_err());
    }

    // Verify the expected behavior for compatibility evaluates markers for each candidate python version.

    #[test]
    fn compatibility_evaluates_markers_for_each_candidate_python_version() {
        let dependency = Dependency {
            name: "example".to_owned(),
            extras: vec![],
            specifier: None,
            url: None,
            marker: Some("python_version < \"3.10\"".to_owned()),
            source: std::path::PathBuf::from("requirements.txt"),
        };

        let release = PackageRelease {
            version: "2.0.0".to_owned(),
            requires_python: Some(">=3.10".to_owned()),
        };

        let environment = test_marker_environment("3.12", "linux");

        let compatible = super::compatible_python_versions_for_environment(
            &[dependency],
            &[Some(release)],
            &environment,
        )
        .unwrap();

        assert_eq!(
            compatible,
            vec![
                "3.10".to_owned(),
                "3.11".to_owned(),
                "3.12".to_owned(),
                "3.13".to_owned(),
                "3.14".to_owned(),
                "3.15".to_owned(),
            ]
        );
    }

    // Verify the expected behavior for compatibility evaluates markers for each candidate python version second.

    #[test]
    fn compatibility_evaluates_markers_for_each_candidate_python_version_second() {
        let dependency = Dependency {
            name: "example".to_owned(),
            extras: vec![],
            specifier: None,
            url: None,
            marker: Some("python_version < \"3.10\"".to_owned()),
            source: std::path::PathBuf::from("requirements.txt"),
        };

        let release = PackageRelease {
            version: "2.0.0".to_owned(),
            requires_python: Some(">=3.10".to_owned()),
        };

        let environment = test_marker_environment("3.12", "linux");

        let compatible = crate::compatibility::compatible_python_versions_for_environment(
            &[dependency],
            &[Some(release)],
            &environment,
        )
        .unwrap();

        assert_eq!(compatible, ["3.10", "3.11", "3.12", "3.13", "3.14", "3.15"]);
    }

    // Verify dependency-marker behavior for marker evaluates candidate python full version.

    #[test]
    fn dependency_marker_evaluates_candidate_python_full_version() {
        let dependency = Dependency {
            name: "example".to_owned(),
            extras: vec![],
            specifier: None,
            url: None,
            marker: Some("python_full_version < \"3.10.1\"".to_owned()),
            source: std::path::PathBuf::from("requirements.txt"),
        };

        let environment = test_marker_environment("3.12", "linux");

        assert!(
            crate::dependencies::dependency_applies_to_python_version(
                &dependency,
                &environment,
                "3.10",
            )
            .unwrap()
        );
    }

    // Verify dependency-marker behavior for marker combines candidate python and platform.

    #[test]
    fn dependency_marker_combines_candidate_python_and_platform() {
        let dependency = Dependency {
            name: "example".to_owned(),
            extras: vec![],
            specifier: None,
            url: None,
            marker: Some("python_version >= \"3.10\" and sys_platform == \"win32\"".to_owned()),
            source: std::path::PathBuf::from("requirements.txt"),
        };

        let windows = test_marker_environment("3.12", "win32");
        let linux = test_marker_environment("3.12", "linux");

        assert!(
            crate::dependencies::dependency_applies_to_python_version(
                &dependency,
                &windows,
                "3.10",
            )
            .unwrap()
        );

        assert!(!crate::dependencies::dependency_applies_to_python_version(
            &dependency,
            &linux,
            "3.10",
        )
        .unwrap());

        assert!(
            !crate::dependencies::dependency_applies_to_python_version(
                &dependency,
                &windows,
                "3.9",
            )
            .unwrap()
        );
    }

    // Verify the expected behavior for missing release is ignored when dependency marker does not apply.

    #[test]
    fn missing_release_is_ignored_when_dependency_marker_does_not_apply() {
        let dependency = Dependency {
            name: "optional-example".to_owned(),
            extras: vec![],
            specifier: None,
            url: None,
            marker: Some("python_version < \"3.10\"".to_owned()),
            source: std::path::PathBuf::from("requirements.txt"),
        };

        let environment = test_marker_environment("3.12", "linux");

        let compatible = crate::compatibility::compatible_python_versions_for_environment(
            &[dependency],
            &[None],
            &environment,
        )
        .unwrap();

        assert_eq!(compatible, ["3.10", "3.11", "3.12", "3.13", "3.14", "3.15"]);
    }

    // Verify dependency-marker behavior for marker matches active extra.

    #[test]
    fn dependency_marker_matches_active_extra() {
        let dependency = Dependency {
            name: "requests".to_owned(),
            extras: vec![],
            specifier: None,
            url: None,
            marker: Some("extra == \"security\"".to_owned()),
            source: std::path::PathBuf::from("requirements.txt"),
        };

        let environment = test_marker_environment("3.12", "linux");
        let security = ["security".parse::<pep508_rs::ExtraName>().unwrap()];
        let docs = ["docs".parse::<pep508_rs::ExtraName>().unwrap()];

        assert!(
            super::dependencies::dependency_applies_to_environment_with_extras(
                &dependency,
                &environment,
                &security,
            )
            .unwrap()
        );

        assert!(
            !super::dependencies::dependency_applies_to_environment_with_extras(
                &dependency,
                &environment,
                &docs,
            )
            .unwrap()
        );
    }

    // Verify dependency-marker behavior for marker not equal respects active extra.

    #[test]
    fn dependency_marker_not_equal_respects_active_extra() {
        let dependency = Dependency {
            name: "requests".to_owned(),
            extras: vec![],
            specifier: None,
            url: None,
            marker: Some("extra != \"security\"".to_owned()),
            source: std::path::PathBuf::from("requirements.txt"),
        };

        let environment = test_marker_environment("3.12", "linux");
        let security = ["security".parse::<pep508_rs::ExtraName>().unwrap()];
        let docs = ["docs".parse::<pep508_rs::ExtraName>().unwrap()];

        assert!(
            !super::dependencies::dependency_applies_to_environment_with_extras(
                &dependency,
                &environment,
                &security,
            )
            .unwrap()
        );

        assert!(
            super::dependencies::dependency_applies_to_environment_with_extras(
                &dependency,
                &environment,
                &docs,
            )
            .unwrap()
        );
    }

    // Verify the expected behavior for candidate python version evaluation preserves active extra.

    #[test]
    fn candidate_python_version_evaluation_preserves_active_extra() {
        let dependency = Dependency {
            name: "requests".to_owned(),
            extras: vec![],
            specifier: None,
            url: None,
            marker: Some("python_version >= \"3.10\" and extra == \"security\"".to_owned()),
            source: std::path::PathBuf::from("requirements.txt"),
        };

        let environment = test_marker_environment("3.12", "linux");
        let security = ["security".parse::<pep508_rs::ExtraName>().unwrap()];
        let docs = ["docs".parse::<pep508_rs::ExtraName>().unwrap()];

        assert!(
            super::dependencies::dependency_applies_to_python_version_with_extras(
                &dependency,
                &environment,
                "3.12",
                &security,
            )
            .unwrap()
        );

        assert!(
            !super::dependencies::dependency_applies_to_python_version_with_extras(
                &dependency,
                &environment,
                "3.12",
                &docs,
            )
            .unwrap()
        );

        assert!(
            !super::dependencies::dependency_applies_to_python_version_with_extras(
                &dependency,
                &environment,
                "3.9",
                &security,
            )
            .unwrap()
        );
    }

    // Verify dependency-marker behavior for marker respects boolean precedence.

    #[test]
    fn dependency_marker_respects_boolean_precedence() {
        let dependency = Dependency {
        name: "example".to_owned(),
        extras: vec![],
        specifier: None,
        url: None,
        marker: Some(
            "python_version < \"3.10\" or python_version >= \"3.12\" and sys_platform == \"win32\""
                .to_owned(),
        ),
        source: std::path::PathBuf::from("requirements.txt"),
    };

        let python39 = test_marker_environment("3.9", "linux");
        let python311_windows = test_marker_environment("3.11", "win32");
        let python312_windows = test_marker_environment("3.12", "win32");
        let python312_linux = test_marker_environment("3.12", "linux");

        assert!(dependency_applies_to_environment(&dependency, &python39).unwrap());
        assert!(!dependency_applies_to_environment(&dependency, &python311_windows).unwrap());
        assert!(dependency_applies_to_environment(&dependency, &python312_windows).unwrap());
        assert!(!dependency_applies_to_environment(&dependency, &python312_linux).unwrap());
    }

    // Verify dependency-marker behavior for marker handles python version equality.

    #[test]
    fn dependency_marker_handles_python_version_equality() {
        let dependency = Dependency {
            name: "example".to_owned(),
            extras: vec![],
            specifier: None,
            url: None,
            marker: Some("python_version == \"3.12\"".to_owned()),
            source: std::path::PathBuf::from("requirements.txt"),
        };

        let python312 = test_marker_environment("3.12", "linux");
        let python313 = test_marker_environment("3.13", "linux");

        assert!(dependency_applies_to_environment(&dependency, &python312).unwrap());
        assert!(!dependency_applies_to_environment(&dependency, &python313).unwrap());
    }

    // Verify dependency-marker behavior for marker handles platform and implementation.

    #[test]
    fn dependency_marker_handles_platform_and_implementation() {
        let dependency = Dependency {
            name: "example".to_owned(),
            extras: vec![],
            specifier: None,
            url: None,
            marker: Some(
                "sys_platform == \"linux\" and implementation_name == \"cpython\"".to_owned(),
            ),
            source: std::path::PathBuf::from("requirements.txt"),
        };

        let linux = test_marker_environment("3.12", "linux");

        assert!(dependency_applies_to_environment(&dependency, &linux).unwrap());
    }

    // Create a fresh temporary project directory for tests that read files from disk.
    fn temporary_project() -> std::path::PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        let path = std::env::temp_dir().join(format!("compatpy-{timestamp}"));

        fs::create_dir_all(&path).unwrap();
        path
    }
}
