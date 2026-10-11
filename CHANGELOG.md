# Changelog

---

## 26.10.3 - 2026-10-11

### Added
- Evaluate PEP 508 environment markers for target Python versions, platforms, implementations, and active extras.
- Extend direct VCS references to Git, Mercurial, Subversion, and Bazaar over supported HTTP(S), SSH, and file transports.
- Normalize package names according to PEP 503 for requirement parsing, dependency identity, duplicate detection, and PyPI lookup.
- Support PEP 440 exact, ordered, compatible-release, and wildcard version specifiers.

### Breaking Changes
- `select_package_release` now returns `Result<Option<&PackageRelease>, String>` so invalid version specifiers can be reported. Callers must handle the new `Result`.

### Fixed
- Reject malformed version specifiers and operands with an error instead of silently coercing invalid version components.

### Maintenance
- Generate draft GitHub release notes from the matching version section in this changelog.
- Update the release workflow to use GitHub Actions' Node.js 24 runtime.

---

## 26.10.2 - 2026-10-10

### Added
- Support Git and VCS direct references, including editable VCS requirements.
- Support direct URL dependencies.
- Add Python 3.15 to the compatibility range.
- Support package extras.

### Fixed
- Fix `--version` and `--help` CLI flags.
- Expand requirements discovery and VCS tests.

---

## 26.10.1 - 2026-10-10

### Fixed
- Fix Linux ARM64 release builds and cross-compilation configuration.
- Improve OpenSSL configuration and diagnostics for ARM64 builds.
- Fix release workflow issues affecting binary builds.

### Changed
- Update GitHub Actions checkout to Node.js 24.

---

## 1.0.0 - 2026-10-08

### Added

- Discover Python dependencies from `requirements.txt`.
- Discover Python dependencies from `pyproject.toml`.
- Retrieve package metadata from PyPI.
- Resolve package versions against dependency specifiers.
- Determine Python versions supported by project dependencies.
- Exclude explicitly incompatible Python versions.
- Prefer stable package releases over prereleases.
- Recommend compatible Python versions.
