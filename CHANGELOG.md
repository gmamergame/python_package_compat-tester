# Changelog

---

## 26.10.4 - 2026-10-20

### Added
- Evaluate environment markers for target Python versions, platforms,
  implementations, and active extras through the CLI compatibility flow.
- Support PEP 440 development, alpha, beta, release-candidate, post-release,
  compatible-release, wildcard, and combined version-specifier cases.
- Recognize local path requirements separately from unsupported pip options.
- Resolve relative local paths against the containing requirements file.
- Read dependencies from local `pyproject.toml` metadata without executing project code.
- Support editable local projects with contextual warnings when metadata is unavailable or invalid.
- Validate direct-reference URLs and supported direct-reference schemes.

### Changed
- Route compatibility analysis through marker-aware dependency evaluation.
- Improve package-version parsing and comparison behavior using validated PEP 440
  semantics, including prerelease and post-release ordering.
- Expand dependency and direct-reference parsing diagnostics and preserve
  requirements-file context for local project warnings.

### Fixed
- Reject malformed direct URLs, unsupported direct-reference schemes, missing
  URL authorities, invalid percent-encoding, and malformed version operands
  instead of silently accepting or coercing them.
- Correct compatibility handling for omitted release components, wildcard
  labels, compatible-release bounds, and prerelease/post-release comparisons.

### Tests
- Add comprehensive coverage for environment markers and PEP 440 version
  normalization, ordering, boundaries, aliases, and invalid inputs.
- Add coverage for absolute, relative, missing, unreadable, and editable local
  requirements.

### Maintenance
- Update project documentation and roadmap coverage for the expanded
  compatibility and requirements parsing behavior.
- Add repository-wide editor and line-ending configuration.

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
