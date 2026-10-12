# CompatPy: Full Compatibility Roadmap

**Goal:** Make CompatPy a robust Python compatibility analyzer that
evaluates dependency requirements, environment markers, package
metadata, and dependency conflicts across Python versions.

This roadmap is intentionally comprehensive. Some items are already
implemented and should be audited with tests rather than rewritten
automatically.

---

-   [x] = the stated subtask is implemented and supported by current code or tests
-   [/] = partial implementation or partial verification; remaining scope is listed below
-   [ ] = not implemented, or no evidence is available to mark it complete

## How to execute this roadmap

-   **Use one branch per parent issue.** Make every focused commit for
    that parent issue on its branch; do not combine commits for separate
    parent issues on one branch.
-   **One unchecked leaf = one focused implementation change and one
    commit.** Do not combine sibling leaves just because they share a
    phase or module. A leaf may touch multiple files only when those
    edits are all required for that one observable behavior.
-   Each commit includes the implementation and its focused tests
    together. Do not create a separate test-only or documentation-only
    commit for the same behavior unless a leaf explicitly calls for a
    standalone audit, test harness, or documentation change.
-   Before starting a leaf, check its prerequisites and the current
    implementation. If implementation reveals more than one independent
    behavior, split the leaf first; do not broaden the commit silently.
-   Every implementation leaf must be small enough to review as one
    change. If its title or acceptance criteria contain independent
    behaviors joined by “and”, split them into sibling leaves before
    implementation unless both behaviors are inseparable parts of one
    externally visible contract.
-   For each leaf, the work description/PR must state: **scope** (what
    changes and what does not), **prerequisites**, **acceptance
    criteria**, **focused tests**, and **commit message**. If a behavior
    requires a product/policy choice not settled here, add a decision
    leaf first and do not hide the choice inside an implementation.
-   A task is complete only when its acceptance condition is observable,
    focused tests cover success and relevant failure/boundary cases, and
    applicable project checks pass. Update its checkbox and parent status
    in the same commit.
-   Never mark work complete based only on code being present, a test
    existing, or a command being attempted. Record the validation command
    and result in the commit/PR description. For Rust changes, run the
    narrowest relevant test while iterating and `cargo test` before
    completing the task; run `cargo fmt --check` when Rust formatting may
    have changed.
-   A checked item describes only the exact behavior in its wording.
    Parent items remain `[/]` until every child is complete; an existing
    implementation that is not integrated or fully verified stays
    partial.
-   When work uncovers additional scope, add unchecked sibling leaves
    before closing the parent. Do not silently expand the current commit
    or mark newly discovered work complete based on adjacent coverage.
-   Keep each commit independently reviewable and buildable. Do not leave
    an intermediate commit with failing tests, a knowingly broken CLI,
    or a success-shaped fallback for unsupported input.

---

## Phase 1: Requirements-file parsing

### Basic requirement parsing

-   [x] Parse and normalize package names.
    -   [x] Parse package-name fields and lowercase names.
    -   [x] Add one canonical package-name normalization helper that implements PEP 503 separator/case rules.
    -   [x] Use canonical names for dependency identity, duplicate detection, and PyPI lookup.
    -   [x] Test that `my_package`, `my.package`, `my-package`, and mixed-case spellings normalize identically.
    -   [x] Test that normalization does not alter extra names, URL fragments, or version specifiers.
-   [x] Parse `==`, `!=`, `<`, `<=`, `>`, `>=`, `~=`, and wildcard
    specifiers.
    -   [x] Cover common operators and wildcard forms with unit tests.
    -   [x] Validate version operands once with `pep440_rs` and return an error rather than coercing malformed versions.
    -   [x] Implement exact and ordered comparisons using parsed PEP 440 versions.
    -   [x] Implement `~=` bounds for two-component and three-or-more-component release segments.
    -   [x] Implement `==X.*` and `!=X.*` using PEP 440 prefix matching, including pre/post/local versions.
    -   [x] Add one table-driven operator test per operator with matching, non-matching, and boundary values.
    -   [x] Verify invalid operators, empty operands, and malformed comma-separated components return errors.
-   [x] Support requirements without version specifiers.
    -   [x] Parse a bare package name with no version clause and store the missing specifier as absent.
    -   [x] Treat an absent specifier as unrestricted when selecting compatible package releases.
    -   [x] Verify discovery retains both pinned and unpinned declarations from one requirements file.
-   [x] Support comma-separated specifiers.
    -   [x] Parse multiple comma-separated clauses as one combined specifier.
    -   [x] Require all supported clauses to match rather than accepting any single clause.
    -   [x] Cover a combined lower/upper-bound specifier in the version evaluator tests.
-   [x] Ignore blank lines and full-line comments.
    -   [x] Skip empty physical lines without creating dependencies.
    -   [x] Skip full-line `#` comments while continuing to discover following valid requirements.
-   [x] Handle inline comments without stripping valid URL fragments.
    -   [x] Strip a whitespace-introduced trailing comment from a requirement.
    -   [x] Preserve a URL fragment when `#` is part of a direct-reference URL.
    -   [x] Preserve the URL fragment and strip only a later whitespace-introduced comment.
-   [x] Handle whitespace and line-ending differences.
    -   [x] Trim leading/trailing spaces and tabs around requirement lines.
    -   [x] Produce equivalent discovered dependencies for LF and CRLF files.
    -   [x] Parse a file containing mixed LF and CRLF line endings.
-   [x] Handle supported line continuations.
    -   [x] Join a backslash-continued requirement into one logical requirement and combine its clauses.
    -   [x] Join multiple continuation lines using CRLF endings.
    -   [x] Ignore comments and blank lines correctly around continued requirements.
---
### Requirement syntax

-   [x] Parse extras, such as `requests[security]`.
    -   [x] Parse one extra and retain its name on the parsed requirement and discovered dependency.
    -   [x] Parse comma-separated extras, including whitespace after commas.
    -   [x] Retain extras when the requirement also has a version specifier and marker.
    -   [x] Reject empty extras, missing closing brackets, empty comma-separated entries, and trailing text.
    -   [x] Keep same-package declarations with different extras as distinct dependencies.
-   [/] Parse direct URL requirements.
    -   [x] Preserve package names and supported HTTP(S), file, and VCS direct-reference strings.
    -   [ ] Validate complete URL syntax and reject empty/malformed authorities, paths, or fragments.
    -   [ ] Preserve source-specific direct-reference fields and report unsupported schemes instead of silently skipping them.
-   [x] Handle Git/VCS references where supported.
    -   [x] Preserve Git direct references with revision, `egg`, and subdirectory fragments.
    -   [x] Preserve percent-encoded subdirectory fragments without decoding or rewriting them.
    -   [x] Discover short-form and long-form editable Git requirements when `#egg=` identifies the package.
    -   [x] Retain editable Git extras and combined `egg`/subdirectory fragments.
    -   [x] Discover supported Mercurial, Subversion, and Bazaar direct-reference schemes.
    -   [x] Cover Git revision identifiers in the tested branch, tag, and commit formats.
-   [/] Handle local paths and editable installs where supported.
    -   [x] Handle editable VCS references with valid `#egg=` metadata.
    -   [ ] Recognize local path requirement syntax separately from unsupported pip options.
    -   [ ] Resolve relative local paths against the containing requirements file.
    -   [ ] Read local project metadata without executing project code.
    -   [ ] Define and implement editable local project behavior when metadata is unavailable.
    -   [ ] Return a contextual unsupported/invalid-input error rather than silently dropping an unparseable local requirement.
    -   [ ] Test absolute, relative, missing, unreadable, and editable local paths in temporary projects.
-   [/] Recognize valid package-name punctuation.
    -   [x] Accept ASCII letters, digits, dot, underscore, and hyphen in the parser.
    -   [ ] Test leading/trailing punctuation and consecutive punctuation against the PEP 508 grammar.
    -   [ ] Test non-ASCII and whitespace behavior against the accepted grammar.
    -   [ ] Normalize valid punctuation variants only at package-identity boundaries.
-   [/] Support standard PEP 508 requirement syntax.
    -   [x] Parse common requirement names, extras, specifiers, direct URLs, and markers.
    -   [ ] Use a single PEP 508 parser as the source of truth for requirements and preserve parsed fields.
    -   [ ] Cover whitespace, extras lists, version lists, direct references, and marker clauses with table-driven tests.
    -   [ ] Cover URL quoting, fragments, and marker separators without corrupting direct references.
    -   [ ] Return a source- and line-aware diagnostic for every invalid requirement instead of silently skipping it.
-   [/] Report malformed requirements clearly.
    -   [x] The standalone requirement parser returns descriptive errors for malformed inputs.
    -   [ ] Preserve physical starting line and logical end line when joining continuations.
    -   [ ] Attach source path, line range, original text, and parser cause to discovery errors.
    -   [ ] Test malformed entries after valid entries and assert the error points to the malformed line.
    -   [ ] Ensure one malformed dependency makes analysis fail explicitly rather than returning partial success.
---
### File structure

-   [ ] Recognize `-r` / `--requirement` and `-c` / `--constraint`.
    -   [ ] Parse `-r path`, `--requirement path`, and `--requirement=path`.
    -   [ ] Parse `-c path`, `--constraint path`, and `--constraint=path`.
    -   [ ] Reject missing option values and unsupported option spellings with file/line diagnostics.
    -   [ ] Test all accepted spellings and malformed forms in parser unit tests.
-   [ ] Resolve nested paths relative to the containing file.
    -   [ ] Resolve relative include paths against the directory of the file containing the include.
    -   [ ] Preserve absolute include paths and normalize `.` / `..` segments safely.
    -   [ ] Test root-relative, nested-relative, absolute, and path-with-spaces includes.
-   [ ] Support nested requirements and constraint files.
    -   [ ] Recursively expand requirement includes in declaration order.
    -   [ ] Recursively collect constraint includes without making constraint-only entries installation roots.
    -   [ ] Preserve markers, extras, source path, line, and include chain for expanded entries.
    -   [ ] Test mixed nested `-r` and `-c` trees and repeated includes.
-   [ ] Detect recursive include loops.
    -   [ ] Track canonical paths on the active include stack to detect direct self-inclusion.
    -   [ ] Detect indirect cycles across multiple files and include types.
    -   [ ] Return the complete include chain in the cycle diagnostic.
    -   [ ] Test self-cycle, multi-file cycle, and a repeated non-cyclic include.
-   [ ] Report missing or unreadable included files.
    -   [ ] Report a missing include with the referring source and line.
    -   [ ] Report permission/read errors with the included path and operating-system cause.
    -   [ ] Verify no dependencies are returned as a success-shaped partial result after an include error.
-   [/] Keep comments and annotations separate from dependency
    semantics.
    -   [x] Treat full-line comments as non-dependency text.
    -   [ ] Preserve recognized `# via` and tool-generated annotations as provenance only.
    -   [ ] Keep unknown comment text out of dependency parsing while retaining source locations.
    -   [ ] Test inline URL fragments, inline comments, repeated annotations, and comments on continued lines.
---
## Phase 2: Version specifiers and environment markers

### Version specifiers

-   [/] Audit exact versions, inequalities, and comma-separated specifiers.
    -   [x] Cover common comparisons and combined constraints with unit tests.
    -   [x] Compare equality and inequality for release tuples with omitted trailing zeros.
    -   [x] Test immediately-below, equal-to, and immediately-above boundaries for ordered operators.
    -   [x] Test comma-separated ranges where all components match, one fails, or one is malformed.
    -   [x] Compare expected outcomes against PEP 440 reference cases.
-   [/] Audit compatible-release operator `~=`.
    -   [x] Cover common compatible-release bounds in unit tests.
    -   [ ] Implement and test the upper bound for `~=1.4` (next major release).
    -   [ ] Implement and test the upper bound for `~=1.4.5` (next compatible minor release).
    -   [ ] Reject invalid one-component operands and non-release components with a clear error.
    -   [ ] Test pre-release and post-release candidate comparisons against compatible bounds.
-   [/] Audit wildcard equality `==3.12.*`.
    -   [x] Cover matching and non-matching release prefixes.
    -   [ ] Match every release tuple beginning with the normalized prefix and exclude adjacent prefixes.
    -   [ ] Define and test wildcard behavior for prerelease, postrelease, and local labels.
    -   [ ] Reject malformed wildcard placement and use with unsupported operators.
-   [/] Audit wildcard exclusion `!=3.12.*`.
    -   [x] Cover exclusion of matching and inclusion of non-matching prefixes.
    -   [ ] Apply the exact inverse of wildcard equality for valid operands.
    -   [ ] Test prerelease, postrelease, local labels, and malformed wildcard placement.
-   [/] Audit post-release ordering.
    -   [x] Cover common post-release ordering.
    -   [ ] Order a final release below its post-releases and compare serials numerically.
    -   [ ] Test post-releases with prerelease/dev components and normalized spellings.
    -   [ ] Reject malformed post-release serials instead of defaulting them to zero.
-   [ ] Support development releases, such as `1.0.dev1`.
    -   [ ] Parse `.devN` serials using the validated version parser.
    -   [ ] Order dev-only releases against alpha, beta, release candidate, and final releases.
    -   [ ] Compare dev releases against post releases and compare serials numerically.
    -   [ ] Test dev versions with equality, inequalities, and release selection.
-   [/] Support alpha and beta releases, such as `1.0a1` and `1.0b2`.
    -   [x] Parse and order common alpha/beta labels in current tests.
    -   [ ] Test accepted aliases and case/separator normalization.
    -   [ ] Test omitted serials, numeric serial ordering, and invalid suffix rejection.
    -   [ ] Test ordering against dev-only, later prerelease phases, final, and post releases.
-   [/] Support release candidates, such as `1.0rc1`.
    -   [x] Parse and order common release-candidate labels in current tests.
    -   [ ] Test accepted aliases and omitted/explicit serial behavior.
    -   [ ] Test numeric serial ordering and malformed suffix rejection.
    -   [ ] Test ordering against dev, final, and post releases.
-   [ ] Support epochs, such as `1!2.0`.
    -   [ ] Parse epoch serials and reject malformed or repeated separators.
    -   [ ] Compare epochs before release segments for every supported operator.
    -   [ ] Test omitted epoch equivalence to epoch zero and distinct nonzero epochs.
-   [ ] Support local versions, such as `1.0+vendor.1`.
    -   [ ] Parse local segments and normalize their case and separators.
    -   [ ] Compare numeric and lexical local segments using PEP 440 ordering.
    -   [ ] Implement local-version matching rules for equality and ordered specifiers.
    -   [ ] Test absent local labels, one label, and multi-segment labels.
-   [ ] Implement PEP 440 normalization and equivalence rules.
    -   [ ] Centralize semantic operations on `pep440_rs::Version` and `VersionSpecifier` where supported.
    -   [ ] Normalize release, epoch, pre/dev/post, and local components through the parser.
    -   [ ] Test case/separator aliases, implicit release zeros, and equivalent normalized strings.
    -   [ ] Remove custom permissive version parsing after all callers use validated PEP 440 types.
-   [/] Handle invalid version strings without panics.
    -   [x] Reject some malformed specifiers safely in current tests.
    -   [ ] Return contextual parse errors from release selection and `Requires-Python` evaluation.
    -   [ ] Remove default-to-zero parsing for invalid version components.
    -   [ ] Property-test malformed operands to prove no panic and no false-compatible result.
---
### Marker parsing

-   [/] Parse marker expressions after semicolons.
    -   [x] Parse marker syntax through `pep508_rs` and reject invalid expressions during discovery.
    -   [ ] Parse the complete PEP 508 requirement once and retain its typed marker expression.
    -   [ ] Preserve parser cause, source file, line, and original text instead of silently skipping.
    -   [ ] Test escaped strings, quotes, whitespace, nested grouping, and malformed syntax.
-   [/] Support `python_version` and `python_full_version`.
    -   [x] Evaluate both variables using candidate marker environments in tested library paths.
    -   [ ] Derive normalized `major.minor` for each candidate's `python_version`.
    -   [ ] Preserve the available patch version for `python_full_version` and document fallback behavior.
    -   [ ] Test 3.9.9/3.10.0 ordering, patch boundaries, and cloned candidate environments.
    -   [ ] Connect candidate-aware marker filtering to the CLI analysis flow.
-   [/] Support parentheses and boolean grouping.
    -   [x] Cover grouping and precedence with marker unit tests.
    -   [ ] Test nested parentheses at multiple depths and reject unbalanced delimiters.
    -   [ ] Test groups containing every comparison and membership operator.
    -   [ ] Verify discovery errors retain file and line context.
-   [/] Implement correct `and` / `or` precedence.
    -   [x] Cover representative precedence cases using the PEP 508 parser.
    -   [ ] Add truth-table cases distinguishing `a and b or c` from `a and (b or c)`.
    -   [ ] Test nested and repeated combinations at grouping boundaries.
-   [/] Support `==`, `!=`, `<`, `<=`, `>`, `>=`, `in`, and `not in` where valid.
    -   [x] Cover comparisons and membership operators in current marker tests.
    -   [ ] Add positive and negative cases for every comparison operator.
    -   [ ] Add `in`/`not in` substring and valid operand-order cases from PEP 508.
    -   [ ] Test version-typed operands and marker-specific comparison behavior.
    -   [ ] Assert invalid operator/operand forms return errors, not silent false results.
-   [/] Support the `extra` marker variable.
    -   [x] Add extra-aware evaluation helpers and tests for equality, inequality, and candidate Python versions.
    -   [ ] Parse requested extras from root requirements and explicitly selected optional groups.
    -   [ ] Normalize and validate active extras as `pep508_rs::ExtraName`.
    -   [ ] Evaluate base dependencies and each relevant active-extra context without losing either set.
    -   [ ] Add end-to-end no-extra, matching, non-matching, multiple-extra, and unrequested-extra cases.
    -   [ ] Pass requested extras through the end-to-end compatibility analysis.
-   [/] Support `implementation_name` and `implementation_version`.
    -   [x] Evaluate implementation markers through the marker environment.
    -   [ ] Add target implementation name/version to analysis options with documented defaults.
    -   [ ] Test CPython and a non-CPython target for both marker variables.
    -   [ ] Mark an unspecified target unknown instead of fabricating a value.
-   [/] Support `os_name`, `sys_platform`, `platform_system`, `platform_machine`, and `platform_python_implementation`.
    -   [x] Evaluate representative platform conditions through `pep508_rs`.
    -   [ ] Define a target-environment type carrying every standardized platform field.
    -   [ ] Add parameterized Windows/Linux/macOS tests for each field.
    -   [ ] Ensure cross-target analysis never substitutes the build host's fields.
    -   [ ] Wire target fields into end-to-end compatibility evaluation.
-   [/] Support other standardized PEP 508 marker variables.
    -   [x] Use the PEP 508 marker implementation rather than a hand-written expression grammar.
    -   [ ] Enumerate fields supported by the pinned `pep508_rs` version.
    -   [ ] Map each field from target input or a documented unknown/default policy.
    -   [ ] Add a conformance case showing each field can change evaluation.
-   [/] Handle unknown variables and invalid expressions safely.
    -   [x] Return marker parse/evaluation errors from the library helpers without panicking.
    -   [ ] Reject unknown identifiers at parse time and retain the parser's error cause.
    -   [ ] Propagate marker errors through discovery, analysis, and CLI with source locations.
    -   [ ] Test malformed markers after valid entries and ensure no partial-success report is emitted.
---
### Marker evaluation

-   [/] Evaluate markers independently for each candidate Python version.
    -   [x] Evaluate per-candidate markers in the library compatibility helper and cover with tests.
    -   [ ] Select candidate releases separately for each supported Python version.
    -   [ ] Evaluate each dependency marker in that candidate environment before applying release metadata.
    -   [x] Route CLI results through the marker-aware per-candidate helper.
    -   [ ] Add a CLI fixture where a dependency applies to only one Python candidate.
-   [/] Skip requirements whose markers evaluate false.
    -   [x] Exclude false-marker dependencies in the library compatibility helper.
    -   [ ] Ensure false-marker dependencies trigger no metadata request for that candidate.
    -   [ ] Test that a false branch with no release does not exclude the candidate.
    -   [ ] Verify this filtering is used for normal CLI runs.
-   [/] Include requirements whose markers evaluate true.
    -   [x] Retain true-marker dependencies in the library compatibility helper.
    -   [ ] Apply true-branch release metadata to the candidate result.
    -   [ ] Test that changing the marker context changes the result as expected.
    -   [ ] Verify true branches constrain the normal CLI result.
-   [/] Handle markers combining Python and operating-system conditions.
    -   [x] Cover combined candidate-Python/platform conditions in tests.
    -   [ ] Add explicit target platform options instead of assuming the build host.
    -   [ ] Test a condition true only for the matching Python-and-platform pair.
    -   [ ] Wire chosen target fields into user-facing analysis.
-   [/] Evaluate extra-specific markers in the relevant extras context.
    -   [x] Implement and test extra-aware evaluation helpers.
    -   [ ] Derive active extras from root requirements and selected project groups.
    -   [ ] Propagate extra context along transitive dependency edges.
    -   [ ] Verify unrequested optional dependencies do not constrain results.
    -   [ ] Connect requested extras and optional groups to compatibility evaluation.
-   [/] Test version boundaries such as Python 3.9.9 versus 3.10.0.
    -   [x] Cover full-version marker boundaries in unit tests.
    -   [ ] Test immediately-below, equal, and immediately-above `python_version` boundaries.
    -   [ ] Test patch boundaries for `python_full_version` with every comparison operator.
    -   [ ] Include two-digit minor versions to detect lexicographic comparison.
-   [/] Compare version values semantically, not lexicographically.
    -   [x] Delegate marker comparisons to `pep508_rs`.
    -   [ ] Ensure package-version operations use validated PEP 440 version objects.
    -   [ ] Keep PEP 508 marker comparison semantics distinct from package specifier semantics.
    -   [ ] Add regression cases for 3.9/3.10, trailing zeros, prereleases, and local versions.
---
## Phase 3: Constraints and dependency sources

### Requirements and constraints

-   [ ] Read `-r` / `--requirement` includes.
    -   [ ] Load one direct `-r` include and retain included source provenance.
    -   [ ] Recursively load nested `-r` includes in declared order.
    -   [ ] Test duplicate includes and define whether repeated declarations are preserved or coalesced.
-   [ ] Read `-c` / `--constraint` includes.
    -   [ ] Load one direct `-c` include into a constraint collection, not the requested dependency roots.
    -   [ ] Recursively load nested `-c` includes and preserve their origin.
    -   [ ] Test that a package appearing only in a constraint file is not requested/installed by itself.
-   [ ] Apply constraints without treating them as ordinary requested
    dependencies.
    -   [ ] Apply a matching constraint's version specifier to a requested dependency.
    -   [ ] Keep a constraint's marker separate and evaluate it for each candidate.
    -   [ ] Leave a requested dependency unchanged when no active constraint matches its normalized name.
    -   [ ] Test that constraint-only packages do not trigger PyPI queries.
-   [ ] Combine multiple constraints for the same package.
    -   [ ] Intersect compatible lower-bound constraints.
    -   [ ] Intersect compatible upper-bound and exact-pin constraints.
    -   [ ] Detect an empty intersection and report the minimal conflicting constraint set.
    -   [ ] Preserve separate intersections for distinct marker branches.
-   [ ] Detect contradictory requirements and constraints.
    -   [ ] Detect contradictory direct requirements before network lookup.
    -   [ ] Detect a direct requirement/constraint contradiction before release selection.
    -   [ ] Detect transitive contradictions during graph resolution.
    -   [ ] Include normalized package name, original specifiers, sources, and line numbers in diagnostics.
-   [ ] Apply environment markers correctly to requirements and
    constraints.
    -   [ ] Evaluate requirement markers before adding a requirement to one candidate's constraint set.
    -   [ ] Evaluate constraint markers before applying the constraint to that candidate.
    -   [ ] Test a marker-false constraint, marker-true constraint, and mutually exclusive branches.
-   [/] Handle repeated package entries.
    -   [x] Retain/sort discovered repeated entries rather than failing on duplicates.
    -   [ ] Define identity equality across normalized name, extras, direct URL, and marker.
    -   [ ] Merge exact duplicate declarations without discarding their source records.
    -   [ ] Intersect compatible version constraints while keeping marker-distinct entries separate.
    -   [ ] Test same-name entries with different extras, markers, URLs, and source files.
-   [/] Preserve provenance: source file and line number for each rule.
    -   [x] Retain the source file path on discovered dependencies.
    -   [ ] Record logical line start/end for backslash-continued requirements.
    -   [ ] Preserve include-chain provenance when entries come from nested files.
    -   [ ] Carry all origins through merges, constraints, resolution, and user-visible diagnostics.
    -   [ ] Test a merged/conflicting dependency and assert every contributing location is reported.
---
### Project metadata

-   [x] Read `pyproject.toml` project dependencies.
    -   [x] Read string entries from the PEP 621 `[project].dependencies` array in a temporary project.
    -   [x] Parse package names and version specifiers from those entries.
    -   [x] Normalize discovered names and retain the `pyproject.toml` source path.
    -   [x] Verify unpinned and version-constrained entries are both represented in the result.
-   [ ] Read optional-dependency groups from `pyproject.toml`.
    -   [ ] Parse and validate the `project.optional-dependencies` table and each PEP 508 entry.
    -   [ ] Add a library option to select optional groups explicitly.
    -   [ ] Add a CLI option to select one or more groups and reject unknown group names.
    -   [ ] Activate dependencies from selected groups only and test default exclusion.
    -   [ ] Test a selected group's marker and extra behavior in the compatibility result.
-   [ ] Read PEP 621 `requires-python`.
    -   [ ] Parse the `[project].requires-python` field as a validated PEP 440 specifier set.
    -   [ ] Reject malformed values with `pyproject.toml` context.
    -   [ ] Intersect the project range with candidate versions before dependency checks.
    -   [ ] Report project-excluded candidates separately from dependency-excluded candidates.
    -   [ ] Test exact, lower/upper bounded, disjoint, missing, and malformed values.
-   [ ] Read `setup.cfg` metadata where practical.
    -   [ ] Define a narrow supported `metadata` / `options` field list before implementation.
    -   [ ] Parse supported `install_requires` and `python_requires` fields without interpolation surprises.
    -   [ ] Report unsupported dynamic/configuration syntax rather than silently treating it as absent.
    -   [ ] Test representative valid, missing, and malformed `setup.cfg` files.
-   [ ] Read `setup.py` metadata where practical without executing
    arbitrary project code.
    -   [ ] Document the deliberately supported static syntax subset.
    -   [ ] Extract literal `install_requires` and `python_requires` values only when statically safe.
    -   [ ] Return an explicit unsupported/dynamic-metadata result when values require execution.
    -   [ ] Add a test proving analysis never imports or executes a fixture's `setup.py`.
-   [ ] Read Poetry dependency declarations where practical.
    -   [ ] Document supported Poetry table forms and Poetry-version assumptions.
    -   [ ] Parse normal and grouped dependencies, extras, markers, and Python constraints separately.
    -   [ ] Translate supported declarations to the shared dependency model without losing source information.
    -   [ ] Add fixtures for valid syntax, unsupported syntax, and malformed TOML.
-   [ ] Read PDM and other common project metadata where practical.
    -   [ ] Select one ecosystem and documented metadata format per implementation task.
    -   [ ] Define field-by-field mapping to the common model before adding a parser.
    -   [ ] Add valid, invalid, unsupported, and conflicting-source fixtures for each ecosystem.
-   [/] Support `requirements.in` and generated `requirements.txt`
    workflows.
    -   [ ] Discover `.in` files with deterministic source ordering.
    -   [ ] Define whether source `.in` and compiled `.txt` files are both analyzed or one is authoritative.
    -   [ ] Prevent accidental duplicate constraints when both input and output files are present.
    -   [ ] Test common pip-tools annotations without parsing them as requirements.
-   [/] Clearly report which sources were analyzed or skipped.
    -   [x] Discover a limited set of known files and log scan decisions in debug output.
    -   [ ] Return a source-discovery report listing analyzed files in deterministic order.
    -   [ ] List recognized-but-unsupported files/options with a reason.
    -   [ ] Distinguish absent optional files from present but unreadable files.
    -   [ ] Add CLI output tests for analyzed and skipped source reporting.
---
### Package identity

-   [/] Normalize package names according to Python packaging rules.
    -   [x] Lowercase names and normalize some separators for PyPI queries.
    -   [ ] Normalize parsed dependency names at model construction.
    -   [ ] Normalize PyPI lookup keys using the exact same helper.
    -   [ ] Preserve the user's original spelling for display and diagnostics.
    -   [ ] Test equivalent names across requirement, metadata, merge, and output paths.
-   [ ] Merge equivalent names such as `my_package` and `my-package`.
    -   [ ] Compare normalized identity while retaining original declarations.
    -   [ ] Combine compatible constraints from equivalent spellings.
    -   [ ] Report conflicts across equivalent spellings using every original source.
    -   [ ] Verify names that are not equivalent remain distinct.
-   [/] Distinguish extras from separate package names.
    -   [x] Store extras separately from the base package name on parsed dependencies.
    -   [ ] Include extras in requirement request identity without changing canonical base package identity.
    -   [ ] Normalize and deduplicate extras according to PEP 508 rules.
    -   [ ] Propagate extras only along the dependency edge that requested them.
    -   [ ] Test that `pkg[foo]` and package `pkg-foo` are never conflated.
-   [ ] Merge duplicate dependencies from multiple files appropriately.
    -   [ ] Merge exact duplicates while retaining all provenance.
    -   [ ] Intersect distinct compatible specifiers for the same active marker context.
    -   [ ] Keep requirements with different markers, extras, or direct references distinct.
    -   [ ] Test duplicate merging and conflict reporting across two source files.
-   [/] Track direct-reference URLs and indexes where relevant.
    -   [x] Preserve direct URLs on parsed dependency records.
    -   [ ] Keep direct-reference URL as part of requirement identity and never send it to PyPI name lookup.
    -   [ ] Add an explicit index/source configuration model and precedence rules.
    -   [ ] Route metadata requests to the selected source and retain source provenance.
    -   [ ] Test direct URLs and alternate index configuration without relying on live network.
-   [/] Avoid silently merging genuinely different direct references.
    -   [x] Keep URL text on dependency records and avoid current URL-based merging.
    -   [ ] Treat distinct URLs for one canonical package as distinct declarations until explicit policy applies.
    -   [ ] Report conflicting direct references with both URLs and all source locations.
    -   [ ] Test equivalent and distinct URL spellings under the chosen URL normalization policy.
---
## Phase 4: Python version discovery and project ranges

### Candidate versions

-   [ ] Discover Python versions from project metadata when specified.
    -   [ ] Parse a project's `requires-python` constraint into a validated specifier set.
    -   [ ] Intersect the configured candidate set with the project constraint.
    -   [ ] Report when the project range yields no candidates before dependency analysis.
    -   [ ] Test lower-only, upper-only, bounded, exact, disjoint, and empty ranges.
-   [ ] Support a configurable candidate-version range.
    -   [ ] Add a typed library option for minimum/maximum Python minor versions.
    -   [ ] Add CLI flags for a bounded minor-version range and validate malformed values.
    -   [ ] Add a separately specified patch-version mode without changing the default minor mode.
    -   [ ] Test inclusive endpoints, reversed ranges, unsupported versions, and defaults.
-   [ ] Include newly released Python versions when reliable metadata is
    available.
    -   [ ] Select and document a maintained source for Python release status/versions.
    -   [ ] Add refresh/update behavior with a deterministic checked-in or cached fallback.
    -   [ ] Exclude unreleased Python versions by default and expose a deliberate prerelease option.
    -   [ ] Test source failure, stale data, and addition of a newly released minor version.
-   [/] Distinguish released versions from prereleases.
    -   [x] Prefer stable package releases over prereleases in current selection tests.
    -   [ ] Model Python candidate release status independently from package version prerelease status.
    -   [ ] Add an explicit policy option for Python prerelease candidates.
    -   [ ] Test stable package/pre-release Python and pre-release package/stable Python independently.
-   [/] Define a policy for minor-version versus patch-version
    evaluation.
    -   [x] Evaluate minor-version candidates and preserve the current interpreter's full version in marker helpers.
    -   [ ] Document minor mode as a representative patch policy and specify how `.0` is synthesized.
    -   [ ] Support patch-specific candidates only when release data or user input supplies a patch.
    -   [ ] Test patch marker behavior when exact patch metadata is known and when it is unknown.
-   [/] Correctly handle versions such as 3.9 through 3.14.
    -   [x] Include these and newer minor candidates in the current hardcoded candidate set.
    -   [ ] Assert every default candidate is valid and monotonically ordered.
    -   [ ] Test behavior for a version older/newer than the current static list.
    -   [ ] Keep the supported range and update process documented with the candidate source.
-   [ ] Avoid relying forever on an obsolete hardcoded version list.
    -   [ ] Define an interface for candidate-version sources before replacing the current array.
    -   [ ] Implement a maintained metadata source with explicit refresh and failure semantics.
    -   [ ] Retain a bounded deterministic fallback and report when it is used.
    -   [ ] Test source updates and fallback results without network access.
---
### Project-level restrictions

-   [ ] Read `requires-python` from project metadata.
    -   [ ] Read PEP 621 `project.requires-python` and validate it as PEP 440.
    -   [ ] Read supported `setup.cfg`/ecosystem fields only after their parsers are implemented.
    -   [ ] Return source-specific parse errors for malformed project Python constraints.
-   [ ] Intersect it with dependency compatibility restrictions.
    -   [ ] Apply project-level restrictions before package resolution for each candidate.
    -   [ ] Keep dependency `Requires-Python` restrictions separate in result reasons.
    -   [ ] Test a candidate excluded by project metadata and another excluded by a dependency.
-   [/] Handle lower and upper bounds correctly.
    -   [x] Evaluate common lower/upper `Requires-Python` bounds against candidates.
    -   [ ] Evaluate strict/inclusive lower and upper bounds at exact boundaries.
    -   [ ] Support wildcard, prerelease, and epoch bounds according to PEP 440.
    -   [ ] Replace custom version comparisons with validated PEP 440 matching.
-   [ ] Represent disjoint compatible ranges if needed.
    -   [ ] Store a specifier set without reducing it to a single minimum/maximum interval.
    -   [ ] Evaluate exclusions and unions for each candidate independently.
    -   [ ] Test ranges such as `>=3.8,<3.10` plus `>=3.12` and explicit exclusions.
-   [ ] Explain why a Python version was excluded.
    -   [ ] Record each rule that rejects a candidate (project, marker, package release, or constraint).
    -   [ ] Render a stable machine-readable reason code and human-readable explanation.
    -   [ ] Test candidates rejected by one rule and candidates rejected by multiple rules.
-   [ ] Distinguish explicit project restrictions from dependency
    conflicts.
    -   [ ] Assign distinct result reasons to project range, marker, release metadata, and solver conflict.
    -   [ ] Preserve the original metadata expression for the explanation.
    -   [ ] Test that identical excluded versions with different causes produce different reason codes.
---
### Results

-   [/] Report compatible candidate Python versions.
    -   [x] Print candidate versions surviving selected release metadata in the CLI.
    -   [ ] Return a typed per-candidate result before formatting CLI output.
    -   [ ] Include only candidates for which all active dependency constraints are satisfied.
    -   [ ] Include active markers, selected releases, and evidence used for each compatible result.
    -   [ ] Test exact human-readable output for a fully compatible fixture.
-   [ ] Report incompatible candidates separately.
    -   [ ] Return incompatible candidates separately from compatible and unknown candidates.
    -   [ ] Attach one or more structured failure reasons to each incompatible candidate.
    -   [ ] Test that no-candidate and partially-compatible projects report all candidate groups.
-   [/] Define behavior when no candidate is compatible.
    -   [x] Print `None` when the current candidate computation returns no compatible versions.
    -   [ ] Return incompatible only when evidence proves every candidate fails.
    -   [ ] Return unknown when missing metadata prevents a conclusion for any potentially compatible candidate.
    -   [ ] Define exit status and output for fully incompatible versus fully unknown analyses.
-   [/] Define behavior when metadata is incomplete.
    -   [x] Print PyPI lookup failures as unavailable for individual dependencies.
    -   [ ] Represent missing, malformed, and network-unavailable metadata as distinct unknown causes.
    -   [ ] Do not treat unknown metadata as compatible or as proven incompatible.
    -   [ ] Report which candidates/dependencies are affected and what data is missing.
-   [ ] Explain the scope of the result: declared compatibility versus
    guaranteed runtime success.
    -   [ ] Label every user-facing result as metadata/declaration compatibility.
    -   [ ] List unverified installability factors such as platform wheels, build backends, and runtime behavior.
    -   [ ] Add README/CLI tests ensuring no wording claims guaranteed installation or execution.
---
## Phase 5: PyPI and package release metadata

### Release metadata

-   [/] Retrieve package release metadata from PyPI.
    -   [x] Parse PyPI JSON fixtures into package/release metadata in unit tests.
    -   [ ] Add mock-HTTP tests for request construction, successful fetch, non-success status, invalid JSON, and body-read failure.
    -   [ ] Verify timeouts and network errors propagate without turning into compatibility success.
    -   [ ] Test package-not-found, non-success status, invalid JSON, and truncated body responses through a mockable HTTP client.
    -   [ ] Include normalized package name and endpoint context in every fetch/parse error.
-   [/] Read each release's `Requires-Python` field.
    -   [x] Parse one available `requires_python` value for each release version in PyPI JSON.
    -   [ ] Preserve all file-level `requires_python` values for a release rather than selecting the first silently.
    -   [ ] Define release-level aggregation when wheel/sdist files disagree or omit the field.
    -   [ ] Test empty releases, mixed missing/present values, and conflicting file metadata.
-   [/] Reject releases whose declared Python range excludes the
    candidate.
    -   [x] Apply selected release `Requires-Python` bounds when building the aggregate candidate list.
    -   [ ] Parse each release's `Requires-Python` into a validated PEP 440 specifier set.
    -   [ ] Filter release candidates against the target Python before choosing the newest candidate.
    -   [ ] Fall back to an older matching release when the newest release excludes the target.
    -   [ ] Test no matching version, matching version with unsupported Python, and older compatible fallback.
-   [/] Handle missing or malformed metadata.
    -   [x] Return errors for malformed JSON and unavailable HTTP responses.
    -   [ ] Distinguish missing `Requires-Python` from unrestricted only when policy explicitly allows that inference.
    -   [ ] Treat malformed `Requires-Python` as unknown/error, never as unrestricted.
    -   [ ] Preserve missing/malformed/network failure causes in typed results.
    -   [ ] Test each metadata failure class through selection and CLI reporting.
-   [ ] Handle yanked releases appropriately.
    -   [ ] Parse yanked state and yank reason from each release file's metadata.
    -   [ ] Exclude yanked releases by default while allowing only the explicitly specified pin/override policy.
    -   [ ] Explain yanked fallback or rejection in selection results.
    -   [ ] Test yanked latest, yanked exact pin, and non-yanked older fallback.
-   [/] Define prerelease and development-release selection policy.
    -   [x] Prefer stable matching package versions, falling back to prereleases if needed.
    -   [ ] Specify when a user constraint explicitly permits prereleases.
    -   [ ] Apply the same documented policy to alpha, beta, rc, and dev releases.
    -   [ ] Test stable preferred, explicit prerelease pin, and no-stable-match cases.
-   [ ] Cache metadata to reduce network requests.
    -   [ ] Deduplicate identical package/source requests within one analysis run.
    -   [ ] Define persistent cache path, schema version, and atomic-write behavior.
    -   [ ] Define freshness/expiry and stale-data use policies.
    -   [ ] Test cache hit, miss, expiry, malformed cache, and concurrent-safe writes.
-   [/] Handle network failures, rate limits, and timeouts.
    -   [x] Return contextual errors for request and HTTP failures.
    -   [ ] Configure finite connect and total request timeouts.
    -   [ ] Retry only transient failures with bounded attempts and backoff.
    -   [ ] Respect `Retry-After` for rate-limit responses and bound wait time.
    -   [ ] Make network operations cancellable where supported and propagate cancellation.
    -   [ ] Test timeout, retry success, retry exhaustion, rate limit, and permanent HTTP failure.
-   [ ] Support offline analysis from available cache.
    -   [ ] Add an explicit offline mode that performs no network request.
    -   [ ] Use valid fresh cache entries and report stale entries according to policy.
    -   [ ] Return an actionable unknown/error when offline cache data is absent or corrupt.
    -   [ ] Test offline success, miss, expired data, and proof that no HTTP call was made.
-   [ ] Support configurable indexes or sources where feasible.
    -   [ ] Define CLI/library source configuration and precedence against defaults.
    -   [ ] Validate and normalize source URLs and reject unsafe/unsupported schemes.
    -   [ ] Implement metadata parsing per documented source protocol.
    -   [ ] Test custom source selection with a local mock server and no credentials in logs.
---
### Candidate releases

-   [/] Find releases satisfying all applicable specifiers.
    -   [x] Select the newest stable release satisfying the current single specifier.
    -   [ ] Intersect all direct/constraint specifier sets before querying candidate releases.
    -   [ ] Compare release versions using complete PEP 440 semantics.
    -   [ ] Return a structured no-match cause distinguishing invalid range from empty result.
    -   [ ] Test exact pin, bounded range, exclusions, wildcard, prerelease, and no match.
-   [ ] Combine release metadata with marker evaluation.
    -   [ ] Evaluate dependency marker for the target Python/platform/extra before release selection.
    -   [ ] Filter candidate releases by their own `Requires-Python` before selecting.
    -   [ ] Test a conditional dependency where one candidate is excluded and another is not.
-   [/] Do not treat a matching version number as proof of Python
    compatibility.
    -   [x] Check selected releases' declared `Requires-Python` bounds in the aggregate candidate calculation.
    -   [ ] Filter/select releases independently for each target Python candidate.
    -   [ ] Require explicit evidence of Python support for a compatible classification.
    -   [ ] Propagate missing/malformed `Requires-Python` as unknown according to documented policy.
    -   [ ] Test a newer incompatible release with an older compatible fallback.
-   [/] Detect packages with no compatible release.
    -   [x] Return no selection when no fetched release matches the supplied specifier.
    -   [ ] Distinguish no version satisfying the package range from all matching versions excluding Python.
    -   [ ] Distinguish both proven failures from unknown caused by missing metadata.
    -   [ ] Test all three cases and verify their separate reason codes.
-   [ ] Explain whether failure comes from a version range or Python
    support.
    -   [ ] Attribute each failure to the requested version range, Python support, marker, or missing metadata.
-   [ ] Define behavior for direct URLs and local packages with
    unavailable metadata.
    -   [ ] Use direct-reference metadata when safely available; do not query PyPI by package name instead.
    -   [ ] Mark unavailable local/direct-reference metadata unknown, not compatible.
    -   [ ] Include reference and origin in the diagnostic while redacting credentials.
---
### Metadata reliability

-   [/] Record metadata provenance.
    -   [x] Normalize and retain the package name in parsed metadata.
    -   [ ] Record request URL and configured index/source on each metadata response.
    -   [ ] Record release version and the exact metadata file(s) supplying its fields.
    -   [ ] Carry provenance into selected-release and per-candidate results.
    -   [ ] Redact URL credentials/tokens in logs, diagnostics, snapshots, and JSON.
-   [ ] Record cache/fetch timestamps where useful.
    -   [ ] Record fetch time, cache-write time, and source timestamp when provided.
    -   [ ] Use a timezone-aware stable timestamp format in the cache schema.
    -   [ ] Test serialization, reload, expiry calculations, and clock-boundary behavior.
-   [/] Do not treat missing metadata as proof of compatibility.
    -   [x] Failed PyPI queries do not produce a successfully parsed release.
    -   [ ] Preserve missing, malformed, unavailable, and stale metadata as typed unknown causes.
    -   [ ] Propagate unknown causes to every affected Python candidate.
    -   [ ] Prevent output formatting from dropping or converting unknown states.
    -   [ ] Test partial metadata availability across multiple dependencies.
-   [ ] Distinguish known incompatibility from unknown compatibility.
    -   [ ] Define decision rules for proven compatible, proven incompatible, unknown, and analysis error.
    -   [ ] Ensure a known hard conflict remains incompatible even if unrelated metadata is unknown.
    -   [ ] Ensure an unknown dependency prevents a candidate from being reported compatible.
    -   [ ] Expose cause-specific unknown results in human and machine output.
-   [ ] Consider platform-specific wheels and source distributions as a
    later capability.
    -   [ ] Keep current declared-metadata compatibility separate from artifact installability.
    -   [ ] Track wheel/sdist evaluation as a separate feature with target tags and build assumptions.
    -   [ ] Do not mark a package installable without a matching artifact or a defined source-build proof.
---
## Phase 6: Dependency conflicts and resolution

### Conflict detection

-   [ ] Detect incompatible version ranges for the same package.
    -   [ ] Define a normalized package/constraint model that can represent every supported PEP 440 specifier.
    -   [ ] Detect an empty intersection for exact pins and simple lower/upper bounds without network access.
    -   [ ] Detect exclusions, wildcard constraints, and prerelease intersections using PEP 440 matching.
    -   [ ] Report the minimal set of conflicting constraints with source locations.
    -   [ ] Test pairwise and multi-way intersections, including a non-conflicting adjacent boundary.
-   [ ] Detect contradictions between project requirements and
    constraints.
    -   [ ] Intersect one requested requirement with one active constraint before PyPI lookup.
    -   [ ] Intersect multiple requirement and constraint sources for the same candidate.
    -   [ ] Preserve URL, extras, and marker identity while detecting contradictions.
    -   [ ] Test same-package conflicts across separate files and a satisfiable equivalent case.
-   [ ] Distinguish missing metadata from a real conflict.
    -   [ ] Introduce distinct `Conflict`, `UnknownMetadata`, and `Resolved` outcomes in the model.
    -   [ ] Ensure lack of releases/network data never emits a constraint-conflict reason.
    -   [ ] Test that the same requirement returns distinct outcomes for empty intersection and unavailable metadata.
-   [/] Report the origins of conflicting requirements.
    -   [x] Retain a source path on each parsed dependency.
    -   [ ] Retain source lines for every requirement, constraint, and transitive edge.
    -   [ ] Carry provenance through normalization and constraint intersection.
    -   [ ] Print all contributing origins in deterministic order in conflict diagnostics.
    -   [ ] Test that merging never loses the earlier declaration's origin.
-   [ ] Handle marker-dependent requirements for the same package.
    -   [ ] Evaluate requirement markers before building the candidate-specific constraint set.
    -   [ ] Include only requirements active in the candidate and selected target environment.
    -   [ ] Recompute active constraints per Python candidate rather than reusing a global intersection.
    -   [ ] Test same-package dependencies on opposite marker branches.
-   [ ] Avoid false conflicts between mutually exclusive environment
    branches.
    -   [ ] Retain marker predicates with each constraint until target evaluation.
    -   [ ] Prove mutually exclusive branches are not intersected in one environment.
    -   [ ] Test Python-version, platform, and extra branches that are mutually exclusive.
---
### Transitive dependencies

-   [ ] Read dependencies declared by candidate package releases.
    -   [ ] Extend the metadata model with each distribution's `Requires-Dist` declarations.
    -   [ ] Parse each declaration as a PEP 508 requirement with source release provenance.
    -   [ ] Select artifact/release metadata consistently when declarations vary across files.
    -   [ ] Test valid, malformed, missing, and empty `Requires-Dist` metadata.
-   [ ] Evaluate dependency markers recursively.
    -   [ ] Add a resolver queue that expands one selected package's active dependencies.
    -   [ ] Evaluate every transitive marker against the target Python/platform/extra context.
    -   [ ] Add a fixture with one active and one inactive transitive edge.
    -   [ ] Preserve the full parent-to-child chain in errors and provenance.
-   [ ] Propagate Python restrictions through the dependency tree.
    -   [ ] Apply each transitive release's `Requires-Python` to the current candidate.
    -   [ ] Propagate active marker branches to the appropriate dependency edges.
    -   [ ] Recompute graph compatibility for every candidate Python version.
    -   [ ] Test a root package compatible alone but made incompatible by a transitive package.
-   [ ] Combine transitive version constraints.
    -   [ ] Accumulate all active transitive constraints by canonical package name.
    -   [ ] Intersect transitive constraints with direct requirements and constraints.
    -   [ ] Preserve every parent edge and declaration origin after accumulation.
    -   [ ] Test compatible transitive intersections and a transitive conflict.
-   [ ] Detect dependency cycles safely.
    -   [ ] Track resolving/resolved nodes separately from version-specific resolution state.
    -   [ ] Terminate self-cycle and multi-node cycles without infinite recursion.
    -   [ ] Define whether cycles are accepted as graph edges or reported as diagnostics.
    -   [ ] Test a self-cycle, multi-package cycle, and repeated acyclic shared dependency.
-   [ ] Handle extras and their additional dependencies.
    -   [ ] Read extras requested on each direct and transitive dependency edge.
    -   [ ] Activate only matching `extra`-conditioned `Requires-Dist` declarations.
    -   [ ] Merge repeated requests for the same package's extras without losing base requirements.
    -   [ ] Test matching, non-matching, multiple, and recursively requested extras.
-   [ ] Include optional dependencies only when requested or configured.
    -   [ ] Exclude every optional group by default.
    -   [ ] Activate only explicitly named groups from project metadata.
    -   [ ] Reject unknown group names with a source-aware diagnostic.
    -   [ ] Test default exclusion and each independently selected group.
---
### Resolution strategy

-   [/] Choose candidate releases using a documented strategy, preferring the newest compatible release that satisfies all applicable constraints.
    -   [x] Prefer the newest stable matching release, with prerelease fallback, in the current single-package selector.
    -   [ ] Document candidate ordering, stable/prerelease policy, yanked handling, and tie-breaking.
    -   [ ] Filter releases by package specifiers, `Requires-Python`, and yanked policy before ordering.
    -   [ ] Select separately for each Python candidate and active dependency graph.
    -   [ ] Test deterministic selection with shuffled metadata and equivalent input ordering.
-   [ ] Backtrack when a selected release creates a conflict.
    -   [ ] Represent unresolved package candidate sets and record each selection decision.
    -   [ ] Backtrack to the next candidate when a downstream constraint invalidates a selection.
    -   [ ] Stop when candidates are exhausted and report the smallest useful conflict explanation.
    -   [ ] Test a fixture requiring one backtrack, one unsatisfiable graph, and one no-backtrack graph.
-   [/] Do not assume the newest release is always correct.
    -   [x] Apply the requested version specifier before selecting the newest matching release.
    -   [ ] Filter by Python support before considering a release newest-compatible.
    -   [ ] Revisit earlier choices when transitive constraints invalidate a selected release.
    -   [ ] Test that an older release wins when the newest violates a transitive constraint.
-   [/] Define prerelease selection behavior.
    -   [x] Prefer stable matches and fall back to a prerelease if no stable match exists.
    -   [ ] Define explicit prerelease opt-in and constraint-triggered prerelease behavior.
    -   [ ] Apply policy consistently across alpha, beta, rc, and dev versions.
    -   [ ] Test stable available, explicit prerelease, dev-only, and prerelease-excluded cases.
-   [ ] Report unresolved dependency sets clearly.
    -   [ ] List each unresolved canonical package, its active constraints, and origin chain.
    -   [ ] Assign stable reason codes for no release, Python exclusion, conflict, and unavailable metadata.
    -   [ ] Keep diagnostics deterministic and test exact structured and human output.
-   [/] Keep results deterministic for unchanged inputs and metadata.
    -   [x] Sort discovered dependencies and release metadata before processing.
    -   [ ] Define tie-breakers for equal versions, duplicate metadata, and multiple conflicts.
    -   [ ] Sort graph expansions and diagnostics independently of hash-map iteration order.
    -   [ ] Run the same fixture repeatedly with shuffled input metadata and assert byte-identical results.
---
## Phase 7: Real-world regression testing

### Regression fixtures

-   [/] Test the supplied uv-generated requirements file.
    -   [x] Exercise representative generated requirements syntax in parser tests.
    -   [ ] Add the actual supplied file as a fixture and record its source/license provenance.
    -   [ ] Assert parsed names, specifiers, markers, extras, and comments against a reviewed manifest.
    -   [ ] Add an offline end-to-end fixture with captured metadata and expected candidate results.
    -   [ ] Document intentional differences from the source uv export/lock.
-   [ ] Test repeated `# via` annotations.
    -   [ ] Verify one and repeated annotations never create dependency entries.
    -   [ ] Preserve parent/tool provenance when annotations contain recognized relationships.
    -   [ ] Test annotations with inline comments, continuations, and duplicate dependencies.
-   [ ] Test multiple Python marker branches.
    -   [ ] Add two mutually exclusive Python-version branches and assert each matching candidate.
    -   [ ] Add a Python-plus-platform branch and assert unmatched candidates are unaffected.
    -   [ ] Assert inactive branches do not trigger metadata lookups.
-   [ ] Test separate common constraint files.
    -   [ ] Add two files constraining one requested package and assert their intersection.
    -   [ ] Add a package found only in a constraints file and assert it is not a root dependency.
    -   [ ] Add contradictory constraints and assert both source locations appear in the diagnostic.
-   [ ] Test nested `-r` and `-c` references.
    -   [ ] Test direct and nested relative includes from distinct directories.
    -   [ ] Test mixed `-r`/`-c` nesting and preserve declaration order/provenance.
    -   [ ] Test self-cycle, multi-file cycle, missing file, and unreadable file.
    -   [ ] Assert include errors report the full referring-file chain.
-   [/] Test exact pins mixed with bounded ranges.
    -   [x] Cover combined specifiers and exact selection in unit tests.
    -   [ ] Test an exact pin that satisfies a range declared in another file.
    -   [ ] Test an exact pin that contradicts a range and reports both locations.
    -   [ ] Repeat the conflict case with one dependency marker inactive.
-   [ ] Test extras and optional dependencies.
    -   [ ] Test direct requested extras through marker evaluation and release selection.
    -   [ ] Test selected and unselected optional groups independently.
    -   [ ] Test transitive extra activation and extra-specific `Requires-Dist`.
    -   [ ] Assert unrequested groups/extras do not cause metadata requests.
-   [/] Test old and new package-version formats.
    -   [x] Cover common modern release, prerelease, and post-release strings.
    -   [ ] Add separate cases for epochs, local versions, and development releases.
    -   [ ] Add case/separator aliases and implicit-zero normalization cases.
    -   [ ] Compare expected results against a pinned reference implementation.
---
### Edge cases

-   [/] Test Python-version boundaries.
    -   [x] Cover representative full-version marker boundaries.
    -   [ ] Add table-driven inclusive/exclusive lower and upper boundary tests.
    -   [ ] Add 3.9/3.10 and multi-digit-minor cases.
    -   [ ] Add patch-level and prerelease Python boundaries.
    -   [ ] Verify each expected result against reference semantics.
-   [/] Test marker combinations using `and`, `or`, and parentheses.
    -   [x] Cover representative grouping, precedence, and membership cases.
    -   [ ] Add truth-table tests for `and`, `or`, precedence, and nested parentheses.
    -   [ ] Add each standardized marker variable and operator to a conformance matrix.
    -   [ ] Add CLI tests proving marker evaluation changes candidate results.
-   [ ] Test conflicting dependencies and diagnostic output.
    -   [ ] Test contradictory direct requirements and assert exact ranges and source lines.
    -   [ ] Test requirement-versus-constraint contradiction across files.
    -   [ ] Test a transitive conflict and assert its dependency chain.
    -   [ ] Assert conflict is distinct from unavailable metadata in output and exit status.
-   [ ] Test packages without a release for a candidate Python version.
    -   [ ] Test no release satisfies the package version specifier.
    -   [ ] Test releases exist but every release excludes the target Python.
    -   [ ] Test unavailable metadata and assert unknown, not incompatible.
    -   [ ] Test fallback to an older release compatible with the candidate.
-   [ ] Test empty files and projects without dependencies.
    -   [ ] Test empty requirements files and empty included requirements.
    -   [ ] Test empty PEP 621 dependency arrays and absent dependency declarations.
    -   [ ] Define and assert the result/exit behavior for a project with no dependencies.
-   [/] Test malformed files and unreadable paths.
    -   [x] Cover malformed requirement entries and path/error behavior in unit tests.
    -   [ ] Test malformed TOML, PEP 508 entries, markers, and version specifiers separately.
    -   [ ] Test missing/unreadable root and included files with source-aware errors.
    -   [ ] Assert parse/I/O failure cannot produce a partial-success result.
-   [/] Test duplicate and normalized package names.
    -   [x] Cover duplicate entries with distinct extras.
    -   [ ] Test case/separator-equivalent names across discovery, merge, and lookup.
    -   [ ] Test compatible and contradictory duplicate constraints across files.
    -   [ ] Test marker-, extra-, and URL-distinct entries remain separate.
-   [ ] Test platform markers and unavailable metadata.
    -   [ ] Add Windows/Linux/macOS target fixtures independent of the host platform.
    -   [ ] Test network-unavailable, missing-field, malformed-field, and stale-cache outcomes.
    -   [ ] Assert unknown is never reported as compatible or proven incompatible.
---
### Reference validation

-   [ ] Compare selected outcomes with pip or uv where feasible.
    -   [ ] Pin reference tool/version and record invocation/environment in test documentation.
    -   [ ] Compare PEP 440 normalization and matching on a curated version corpus.
    -   [ ] Compare PEP 508 requirement and marker parsing on a curated syntax corpus.
    -   [ ] Compare release choices only where policies align and document deliberate divergences.
    -   [ ] Keep differential tests reproducible and offline after capturing fixtures.
-   [/] Create fixtures with known expected outcomes.
    -   [x] Use inline unit fixtures with explicit expected parse and selection results.
    -   [ ] Add a minimal fixture per behavior with only the required project files.
    -   [ ] Check in deterministic PyPI metadata fixtures with source/capture provenance.
    -   [ ] Add at least one multi-source fixture covering full project analysis.
-   [ ] Document deliberate differences from pip or uv.
    -   [ ] Record intentional policy differences and unsupported resolver behavior in user documentation.
-   [/] Prevent regressions with automated tests.
    -   [x] Cover current parsing, marker, and selection behavior in Rust unit tests.
    -   [ ] Add integration coverage for discovery, includes, constraints, resolver, and CLI output/exit behavior.
    -   [ ] Require a regression test in the same commit for every bug fix.
    -   [ ] Keep default tests deterministic, network-free, and isolated from user cache/home state.
---
## Phase 8: Diagnostics, CLI, performance, and resilience

### Explanations

-   [ ] Identify the package responsible for excluding a Python version.
    -   [ ] Preserve one structured cause for each failed package/candidate pair.
    -   [ ] Rank/display all independent causes without hiding secondary failures.
    -   [ ] Test single-package and multi-package candidate exclusions.
-   [/] Show the relevant version specifier and marker.
    -   [x] Print a dependency's version specifier in the current CLI output.
    -   [ ] Print the original normalized version constraint in candidate failure explanations.
    -   [ ] Print marker text and whether it applied in the target environment.
    -   [ ] Escape/control output so package metadata cannot forge diagnostic lines.
-   [/] Show source file and line number where possible.
    -   [x] Print the dependency source path.
    -   [ ] Attach start/end line numbers to each logical requirement during parsing.
    -   [ ] Preserve source locations through include expansion, merge, and resolution.
    -   [ ] Print file and line for every direct and transitive conflict cause.
    -   [ ] Test CRLF and continued requirements report their original physical line range.
-   [ ] Explain conflicts between requirements.
    -   [ ] Render normalized package name and every conflicting specifier.
    -   [ ] Render each source location and the marker context that activated it.
    -   [ ] Provide a minimal conflict summary and optional verbose dependency chain.
    -   [ ] Snapshot-test stable conflict output.
-   [ ] Distinguish compatible, incompatible, and unknown outcomes.
    -   [ ] Define a typed result enum for compatible, incompatible, unknown, and analysis-error states.
    -   [ ] Specify how mixed known/unknown dependencies determine a candidate result.
    -   [ ] Render each status distinctly in human and JSON output.
    -   [ ] Test all status transitions and ensure unknown never becomes compatible.
-   [ ] Warn about missing or stale metadata.
    -   [ ] Attach structured warnings for absent `Requires-Python` and other required metadata.
    -   [ ] Add fetched/cached timestamps to metadata provenance.
    -   [ ] Define stale-cache policy and report age/source of stale data.
    -   [ ] Test that warnings do not downgrade a proven conflict or upgrade unknown to compatible.
-   [ ] Summarize why each candidate passed or failed.
    -   [ ] Accumulate active requirements, selected releases, skipped markers, and failures per candidate.
    -   [ ] Generate deterministic human-readable summaries from the structured result.
    -   [ ] Include a verbose explanation mode that exposes all evidence without changing the result.
    -   [ ] Test candidate explanations against a fixture with pass, fail, skipped, and unknown causes.
---
### CLI behavior

-   [/] Keep the existing command-line interface predictable.
    -   [x] Provide help/version flags and reject unexpected argument counts.
    -   [ ] Specify command grammar, defaults, option precedence, and backward compatibility.
    -   [ ] Snapshot help/version output and test exit behavior for each result state.
    -   [ ] Reject contradictory/duplicate options with a usage diagnostic.
-   [/] Validate project-directory arguments.
    -   [x] Require the single positional argument to exist and be a directory.
    -   [ ] Resolve relative and symlinked project paths to a stable analysis root.
    -   [ ] Check directory and discovered input readability and report OS errors.
    -   [ ] Test path with spaces, symlink, nonexistent path, file path, and permission failure where supported.
-   [/] Report invalid paths clearly.
    -   [x] Print an error naming an invalid project directory.
    -   [ ] Include the operation (open/read/resolve) and target path in each I/O error.
    -   [ ] Avoid exposing stack traces or unrelated environment data in normal mode.
    -   [ ] Test root-project and nested-include path failures.
-   [/] Define exit codes for success, incompatibility, and analysis
    errors.
    -   [x] Return nonzero for invalid arguments/paths and analysis errors.
    -   [ ] Reserve and document stable numeric codes for compatible, incompatible, unknown, and usage/error.
    -   [ ] Ensure OS/process failure codes do not collide with documented analysis states.
    -   [ ] Test each exit code through the compiled CLI binary.
-   [ ] Keep stdout suitable for piping.
    -   [ ] Define which stdout content is stable and which is presentation-only.
    -   [ ] Send warnings, progress, and diagnostics to stderr unless selected output format requires them.
    -   [ ] Test quiet/non-TTY invocation and ensure stdout contains no progress noise.
-   [/] Send diagnostics to stderr where appropriate.
    -   [x] Send top-level discovery errors to stderr.
    -   [ ] Send progress and warning diagnostics to stderr in human mode.
    -   [ ] Include package/network failures in structured result output without mislabeling them as stdout prose.
    -   [ ] Test stdout/stderr separately for success, warning, and fatal failure.
-   [ ] Consider machine-readable JSON output.
    -   [ ] Define a versioned JSON schema for inputs, dependencies, candidates, statuses, provenance, and reasons.
    -   [ ] Add a JSON output flag with deterministic key/value and array ordering.
    -   [ ] Keep diagnostics on stderr and emit exactly one JSON document on stdout.
    -   [ ] Validate output against checked-in schema and golden tests.
-   [/] Consider verbose/debug output.
    -   [x] Provide compile-time debug logging through the existing debug macro.
    -   [ ] Replace compile-time-only visibility with a documented runtime verbosity option.
    -   [ ] Define log levels and ensure secrets/credentials are redacted.
    -   [ ] Keep debug messages off machine-readable stdout and test level filtering.
---
### Performance and resilience

-   [ ] Avoid fetching identical package metadata repeatedly.
    -   [ ] Deduplicate requests by canonical package name and configured source within one run.
    -   [ ] Share request results safely between candidate-version analyses.
    -   [ ] Test request counts with a mock client and ensure distinct sources are not conflated.
-   [ ] Cache repeated marker and version calculations where useful.
    -   [ ] Establish CPU/allocation baselines before introducing caches.
    -   [ ] Cache only immutable parsed specifiers/markers or pure evaluations with complete environment keys.
    -   [ ] Bound memory and define cache invalidation for configuration changes.
    -   [ ] Benchmark before/after and retain a cache only if it materially improves the measured workload.
-   [/] Handle large requirements files efficiently.
    -   [x] Process requirement files line-by-line without retaining all physical lines separately.
    -   [ ] Add synthetic large-file benchmark cases with duplicate and marker-heavy requirements.
    -   [ ] Bound include recursion, file size, dependency count, and resolver work with actionable errors.
    -   [ ] Measure peak memory and runtime against a documented baseline.
-   [/] Handle interrupted network requests gracefully.
    -   [x] Return request failures as errors instead of panicking.
    -   [ ] Add finite connect/read/overall request timeouts.
    -   [ ] Propagate user cancellation and avoid starting subsequent metadata requests.
    -   [ ] Define bounded retry policy for transient errors and honor capped `Retry-After`.
    -   [ ] Report partial metadata failure as unknown, not successful compatibility.
-   [/] Avoid panics on user-controlled input.
    -   [x] Validate common malformed requirement and marker inputs with error-returning paths.
    -   [ ] Remove permissive parse-to-zero/default branches from version and metadata parsing.
    -   [ ] Audit all `unwrap`, `expect`, indexing, and unchecked conversion paths reachable from project files.
    -   [ ] Fuzz requirement, marker, version, TOML, and PyPI JSON boundaries for panic-free behavior.
    -   [ ] Ensure invalid input always returns an error/unknown state rather than false compatibility.
-   [/] Keep errors contextual and actionable.
    -   [x] Include package/path context in many discovery and PyPI errors.
    -   [ ] Define typed error categories for invalid input, unsupported syntax, I/O, network, and resolution.
    -   [ ] Preserve source line and include/dependency chain context when wrapping errors.
    -   [ ] Add a next-action hint where a user can resolve the issue.
    -   [ ] Test error categories and messages without asserting unstable OS-specific wording.
---
## Phase 9: Rust code quality and release process

### Code structure

-   [/] Separate parsing from evaluation.
    -   [x] Keep requirement discovery and compatibility selection in distinct modules.
    -   [ ] Define typed boundaries for raw source, parsed requirement, normalized dependency, and resolution input.
    -   [ ] Move recursive file traversal behind a source-loading interface.
    -   [ ] Keep pure marker/specifier evaluation free of file-system and network access.
    -   [ ] Add boundary tests proving the refactor preserves CLI behavior.
-   [/] Separate version parsing from comparison.
    -   [x] Keep version parsing and comparison helpers separate from requirement scanning.
    -   [ ] Route package version parse, compare, and specifier matching through one PEP 440 abstraction.
    -   [ ] Remove custom parsing only after every caller and test uses the shared abstraction.
    -   [ ] Keep Python candidate version granularity explicit (minor or patch).
-   [ ] Separate marker parsing from evaluation.
    -   [ ] Introduce a marker type constructible only after successful PEP 508 parsing.
    -   [ ] Parse each marker once and evaluate against multiple candidate environments.
    -   [ ] Retain original marker text alongside the validated representation for diagnostics.
    -   [ ] Test parse-once/evaluate-many and error propagation.
-   [/] Separate file loading from dependency resolution.
    -   [x] Keep file discovery/parsing in `dependencies.rs` and candidate calculations in `compatibility.rs`.
    -   [ ] Add a loader that yields requested dependencies, constraints, and source diagnostics separately.
    -   [ ] Keep nested-include expansion in the loader, not in the resolver.
    -   [ ] Make resolution operate on parsed data through an injected metadata provider.
-   [/] Use explicit types for requirements, markers, constraints, and
    results.
    -   [x] Model dependencies, package releases, and marker environments with explicit types.
    -   [ ] Introduce separate `Requirement` and `Constraint` types over shared parsed syntax.
    -   [ ] Introduce typed release-selection results and source provenance.
    -   [ ] Introduce per-candidate Compatible/Incompatible/Unknown results with typed reasons.
    -   [ ] Remove ambiguous `Option`/empty-vector result encodings after callers migrate.
-   [/] Keep public APIs stable where practical.
    -   [x] Expose library functions separately from the binary entry point.
    -   [ ] Document current guarantees and identify experimental public APIs.
    -   [ ] Prefer additive options/result types or intentional versioned breaks over silent signature changes.
    -   [ ] Add compile-tested rustdoc examples for supported library usage.
    -   [ ] Review semver impact for every public API change.
-   [ ] Document tricky PEP 440 and PEP 508 behavior.
    -   [ ] Document PEP 440 normalization, release ordering, and operator examples.
    -   [ ] Document PEP 508 marker variables/operators and target-environment inputs.
    -   [ ] Document how extras and optional groups activate dependencies.
    -   [ ] Link each documented rule to an automated regression test.
---
### Quality gates

-   [/] Run `cargo fmt`.
    -   [ ] Run `cargo fmt --check` for every Rust implementation commit.
    -   [ ] Format before commit and avoid unrelated formatting-only edits.
-   [x] Run `cargo test`.
    -   [x] Run the Rust unit test suite; the recorded run passed all 92 unit tests.
    -   [x] Run the Rust doc-test phase; the recorded run completed with no doc tests present.
-   [/] Run `cargo clippy` and address relevant warnings.
    -   [ ] Run `cargo clippy --all-targets --all-features -- -D warnings` or document justified exceptions.
    -   [ ] Fix warnings introduced by the change, including unused extra-aware helpers after integration.
    -   [ ] Record unrelated baseline warnings rather than expanding an atomic implementation commit.
-   [/] Add unit tests for every new behavior.
    -   [x] Cover current parser, marker, and selection functionality with Rust unit tests.
    -   [ ] Each leaf implementation includes focused success, invalid-input, and relevant boundary tests.
    -   [ ] Keep tests offline unless the leaf explicitly adds network behavior.
    -   [ ] Add bug-regression assertions in the same commit as the fix.
-   [ ] Add integration tests using temporary project directories.
    -   [ ] Test public discovery against temporary single-file projects and assert typed parsed data.
    -   [ ] Test nested includes and constraints with temporary multi-file projects.
    -   [ ] Test the compiled CLI while asserting stdout, stderr, and exit status independently.
    -   [ ] Isolate test outcomes from current directory, environment, user cache, and network.
-   [/] Test Windows and Unix path behavior.
    -   [x] Current suite runs on Windows and uses `PathBuf`-based temporary paths.
    -   [ ] Add Linux and Windows CI jobs running the core Rust suite.
    -   [ ] Test separators, drive/UNC paths, spaces, symlinks, and case sensitivity where supported.
    -   [ ] Add macOS CI only if declared supported; otherwise explicitly document the gap.
-   [/] Avoid unnecessary dependencies and excessive network calls.
    -   [x] Keep current tests independent of live network requests using metadata fixtures.
    -   [ ] Inject a mockable metadata provider and verify request counts.
    -   [ ] Deduplicate same-package/source requests across candidate calculations.
    -   [ ] Document rationale, feature use, MSRV impact, and alternatives for each new dependency.
-   [/] Review panic-prone and error-handling paths.
    -   [x] Return errors for common malformed inputs and I/O/network failures.
    -   [ ] Remove silent skip/default behavior for malformed project dependency entries.
    -   [ ] Preserve source, line, operation, and cause in propagated errors.
    -   [ ] Add regressions for each identified panic or swallowed error path.
---
### Release readiness

-   [/] Update README examples and supported-feature documentation.
    -   [x] Document basic CLI usage and supported project files.
    -   [ ] Map each README feature claim to an implemented end-to-end behavior and test.
    -   [ ] Remove or qualify claims that only have helper-level or partial support.
    -   [ ] Add examples only for verified behavior and state metadata-only limitations.
-   [/] Update `CHANGELOG.md`.
    -   [ ] Verify version/date/headings against actual release history.
    -   [ ] Include only user-visible changes that ship in the target version.
    -   [ ] Keep incomplete roadmap work in Unreleased without presenting it as shipped.
-   [/] Review Cargo.toml metadata and dependency versions.
    -   [ ] Validate crate name, description, license, repository, keywords, categories, and readme.
    -   [ ] Confirm declared MSRV against CI and all dependency MSRVs.
    -   [ ] Review direct dependencies, enabled features, and lockfile changes.
    -   [ ] Build and test using the declared minimum Rust toolchain.
-   [/] Check license and attribution files.
    -   [x] A project license file is present.
    -   [ ] Verify source and binary artifacts include the project license.
    -   [ ] Determine and satisfy third-party dependency notice obligations.
    -   [ ] Verify generated/bundled files have clear licensing provenance.
-   [/] Build release binaries for supported targets.
    -   [ ] Enumerate release workflow targets and document each target's toolchain.
    -   [ ] Build each declared target in CI from the release commit.
    -   [ ] Smoke-test version/help and a deterministic local fixture on runnable targets.
    -   [ ] Verify artifact names, checksums, and uploaded files against the release manifest.
-   [/] Verify `cargo package` and publishing readiness.
    -   [ ] Run `cargo package --list` and review the exact included file set.
    -   [ ] Run `cargo package --verify` and build/test the packaged crate.
    -   [ ] Confirm readme/license/source files are present and local-only files absent.
    -   [ ] Validate version/tag and publishing metadata without publishing.
-   [/] Create a Git commit for every successful implementation or fix.
    -   [ ] Keep one focused roadmap leaf per commit and reference its task title in the commit body.
    -   [ ] Record focused validation commands and results in the commit/PR description.
    -   [ ] Exclude unrelated formatting, generated files, and environment artifacts.
    -   [ ] Verify every commit is buildable/testable before starting the next leaf.
-   [/] Release a new version only when the project is ready.
    -   [ ] Identify release-blocking roadmap items and record explicit waivers for any deferred items.
    -   [ ] Run tests, formatting, Clippy, target builds, packaging verification, and smoke tests.
    -   [ ] Review changelog, README claims, version/tag, artifacts, and rollback procedure.
    -   [ ] Publish only after checks pass and record post-release verification.
---
## Phase 10: Advanced capabilities

### Platform and interpreter support

-   [/] Evaluate operating-system-specific markers.
    -   [x] Evaluate representative OS/platform marker expressions in library tests.
    -   [ ] Accept explicit target OS values in library options and CLI.
    -   [ ] Map target OS consistently to all dependent standardized marker fields.
    -   [ ] Test Windows, Linux, and macOS outcomes independently of the build host.
-   [ ] Distinguish CPython from alternative implementations where
    relevant.
    -   [ ] Accept target implementation name/version and document any default.
    -   [ ] Test CPython, PyPy, and an unspecified implementation for relevant markers.
    -   [ ] Return unknown for unsupported implementation metadata rather than assuming CPython.
-   [ ] Consider architecture-specific wheel availability.
    -   [ ] Accept target architecture and define the supported architecture identifiers.
    -   [ ] Parse wheel tags and compare them with target architecture/interpreter tags.
    -   [ ] Keep source-distribution/build feasibility separate from wheel compatibility.
    -   [ ] Test matching, mismatched, universal, and unavailable artifact cases.
-   [ ] Consider ABI and platform tags.
    -   [ ] Parse wheel Python/ABI/platform tags using a maintained standards implementation.
    -   [ ] Define target tag generation for supported interpreter/platform combinations.
    -   [ ] Report the exact tag mismatch that rejects a wheel.
    -   [ ] Test `abi3`, pure-Python, platform-specific, and incompatible wheels.
-   [/] Distinguish metadata-level compatibility from installability on
    a specific machine.
    -   [x] This roadmap states that metadata compatibility does not guarantee installation or runtime success.
    -   [ ] State the same limitation explicitly in the README's user-facing explanation of results.
---
### Project ecosystems

-   [/] Improve support for common `pyproject.toml` configurations.
    -   [x] Read PEP 621 `project.dependencies`.
    -   [ ] Implement optional groups and `requires-python` as independently verifiable changes.
    -   [ ] Define precedence when `pyproject.toml` coexists with requirements files.
    -   [ ] Report unsupported tool/build-system sections instead of treating them as analyzed.
-   [ ] Improve Poetry, PDM, and other ecosystem-specific parsing.
    -   [ ] Create one work item per ecosystem and per supported metadata feature.
    -   [ ] Document tool/version formats and parsing boundaries before implementation.
    -   [ ] Test source provenance and compare parsed declarations with that tool's output.
-   [ ] Handle documented lockfile formats.
    -   [ ] Select one lockfile format/schema version per implementation change.
    -   [ ] Parse package identity, pin, source, dependencies, extras, and marker fields.
    -   [ ] Reject unsupported schema versions with an explicit error.
    -   [ ] Test sanitized real-world fixtures and ensure lockfiles are not mistaken for requirements.
-   [ ] Explain differences between ecosystem resolution rules and
    standard requirements syntax.
    -   [ ] Document mapping from each supported ecosystem to the internal dependency model.
    -   [ ] List unsupported features and resolution/prerelease/source-policy differences.
    -   [ ] Add examples whose expected outcomes are covered by conformance tests.
---
### Developer integrations

-   [ ] Add JSON output for CI pipelines.
    -   [ ] Version/publish the JSON schema and document compatibility guarantees.
    -   [ ] Include inputs, candidate statuses, package decisions, reasons, and provenance.
    -   [ ] Validate output against the schema and test additive evolution.
    -   [ ] Add a CI example that checks statuses without parsing human-readable output.
-   [/] Provide stable exit codes for automation.
    -   [x] Use nonzero exits for invalid arguments, invalid paths, and top-level analysis errors.
    -   [ ] Finalize codes for compatible, incompatible, unknown, and analysis/usage error.
    -   [ ] Keep process exit status consistent with the selected output mode.
    -   [ ] Test all codes on Windows and Unix command invocation.
-   [ ] Consider GitHub Actions documentation.
    -   [ ] Add a minimal workflow using a pinned release and local fixture.
    -   [ ] Document permissions, inputs, caching, outputs, and unknown/failure semantics.
    -   [ ] Validate workflow syntax and ensure examples contain no credentials.
-   [/] Consider a library API alongside the CLI.
    -   [x] Expose dependency discovery, metadata, selection, and compatibility functions from the library.
    -   [ ] Define public options/results/errors without exposing internal resolver implementation details.
    -   [ ] Add rustdoc examples and compile them in tests.
    -   [ ] Document API stability and semver policy.
-   [ ] Consider reports comparing Python versions.
    -   [ ] Define a structured comparison row with candidate status, selected releases, and reasons.
    -   [ ] Add deterministic human-readable table output with narrow-terminal fallback.
    -   [ ] Add JSON representation using the same structured results.
    -   [ ] Test no-color, non-TTY, and machine-readable modes.
-   [ ] Consider analysis of a specific target environment.
    -   [ ] Define complete target options for Python, OS, architecture, implementation, and extras.
    -   [ ] Add CLI parsing and validation for each target option.
    -   [ ] Use the same target model for markers, release metadata, and optional wheel analysis.
    -   [ ] Report missing target fields as unknown rather than silently substituting host values.

------------------------------------------------------------------------

## Recommended implementation order

The numbered entries below are milestones, **not commit-sized tasks**.
For every milestone, implement and commit one unchecked leaf at a time.
Where a leaf depends on shared models or infrastructure, complete the
prerequisite leaf first; do not combine the prerequisite and feature in
one oversized commit. Every change should leave the main branch
buildable and its tests passing.

1.  **Audit version semantics:** test `~=`, wildcard specifiers,
    post-releases, prereleases, development releases, epochs, and local
    versions.
    -   [x] Cover common compatible, wildcard, post-release, alpha/beta, and release-candidate behavior.
    -   [ ] Centralize package comparisons on validated `pep440_rs` values and test current behavior parity.
    -   [ ] Implement development-release parsing/order as one change, with focused tests.
    -   [ ] Implement epoch parsing/order as one change, with focused tests.
    -   [ ] Implement local-version parsing/matching as one change, with focused tests.
    -   [ ] Implement compatible-release and wildcard edge cases as separate operator changes.
    -   [ ] Add malformed-input and normalization regressions; remove custom parser only after parity is demonstrated.
2.  **Complete marker integration:** parse the expression after `;`
    and evaluate it for each candidate Python version and requested extra.
    -   [x] Use PEP 508 marker parsing/evaluation in library helpers and cover representative cases.
    -   [ ] Build one explicit analysis target/environment model and test its defaults.
    -   [ ] Route candidate Python markers through compatibility selection and prove false branches are skipped.
    -   [ ] Route active extras through analysis and prove extra-specific branches are selected.
    -   [ ] Wire analysis results to the CLI and test stdout, stderr, and exit behavior for marker branches.
3.  **Implement requirements and constraint includes:** support `-r` and
    `-c`, relative paths, and correct constraint semantics.
    -   [ ] Add typed source/line provenance before recursive expansion.
    -   [ ] Implement direct `-r` loading and its path/error tests.
    -   [ ] Implement direct `-c` loading while keeping constraint-only packages out of roots.
    -   [ ] Add relative nested includes and cycle detection as separate commits.
    -   [ ] Apply one constraint intersection, then multiple/contradictory constraints in separate commits.
4.  **Integrate PyPI release metadata:** use `Requires-Python` to
    improve release selection.
    -   [x] Fetch PyPI JSON and parse release versions and available `Requires-Python` metadata.
    -   [ ] Introduce a mockable metadata provider before changing network behavior.
    -   [ ] Validate `Requires-Python` and select the newest compatible release for one candidate.
    -   [ ] Add older-release fallback and unknown-metadata outcomes separately.
    -   [ ] Add yanked/prerelease policies as separate selection changes.
5.  **Analyze transitive dependencies:** build the dependency graph,
    propagate constraints, and identify conflicts.
    -   [ ] Parse one selected release's `Requires-Dist` data and test provenance.
    -   [ ] Expand one dependency level and evaluate its markers before recursive traversal.
    -   [ ] Add cycle-safe traversal independently from version backtracking.
    -   [ ] Add direct/transitive constraint intersection and conflict explanation.
    -   [ ] Add extras propagation, then backtracking, as distinct resolver changes.
6.  **Test against real projects:** use the supplied uv-generated
    requirements file as a regression fixture and compare with
    established tools.
    -   [x] Cover representative requirements syntax in focused unit tests.
    -   [ ] Check in the supplied fixture and verify its parser output.
    -   [ ] Capture deterministic metadata and verify expected end-to-end results offline.
    -   [ ] Add pinned-reference differential cases and document expected divergences.
7.  **Improve reporting and reliability:** diagnostics, caching, error
    handling, and automation-friendly output.
    -   [x] Report common top-level input and lookup errors.
    -   [ ] Add structured per-candidate compatible/incompatible/unknown results before formatting.
    -   [ ] Add deterministic human explanations from those structured results.
    -   [ ] Add JSON schema/output separately from exit-code changes.
    -   [ ] Add request deduplication before persistent cache, then offline cache behavior.
    -   [ ] Add bounded network timeouts/retries as separate changes with mock-server tests.

## Rules for every implementation

-   Select exactly one unchecked leaf as the change scope; do not bundle
    another leaf in the same commit.
-   Include tests for the leaf's acceptance criteria in that same
    implementation commit. A standalone test/harness commit is permitted
    only when it is itself a listed leaf and leaves the build green.
-   Run the focused test first, then `cargo test` and `cargo fmt --check`
    before marking a Rust implementation leaf complete. Run Clippy when
    the change affects lintable Rust code or explicitly requires it.
-   Fix regressions caused by the change before committing; record
    unrelated baseline failures instead of silently broadening scope.
-   Commit after every successful leaf with a message naming the one
    behavior changed. Include the focused/full validation commands and
    their results in the commit or PR description.
-   Update the leaf and parent statuses in that commit only after its
    acceptance criteria and validation pass.
-   Keep the real-world requirements file as a long-term regression
    test.
-   Never claim full compatibility when only declared metadata has been
    checked.

## Important scope distinction

CompatPy can determine whether declared requirements and package
metadata **indicate** compatibility. Guaranteeing that every dependency
installs and runs on a given Python version is a larger task involving
build systems, wheels, operating systems, and potentially actual
installations.

## Next single change

**Task:** Route candidate-specific environment-marker evaluation through
the CLI compatibility result.

**Scope for this commit**

-   Keep the current dependency discovery and selected-release strategy;
    do not add nested includes, constraint solving, transitive resolution,
    persistent caching, JSON output, or wheel analysis in this change.
-   Supply a deterministic marker environment through an explicit
    analysis input. Do not silently construct a target by mixing values
    from different hosts or inventing missing marker values.
-   For each candidate Python version, evaluate each dependency marker
    before including that dependency's selected release in the candidate
    compatibility calculation.
-   Preserve current behavior for dependencies with no marker.
-   Propagate invalid marker/environment errors as analysis errors; do not
    print a successful compatibility list after a failed evaluation.

**Acceptance criteria**

-   A dependency whose marker is true for a candidate affects that
    candidate's compatibility result.
-   A dependency whose marker is false for a candidate does not affect
    that candidate's result.
-   A marker-false dependency with no selected release does not exclude
    that candidate.
-   A marker-true dependency with no selected release is not reported as
    compatible.
-   Results are deterministic for the same dependencies, releases, and
    marker environment.
-   The CLI's output clearly states which target environment was used;
    if the CLI cannot provide a complete environment, return an explicit
    error/unknown result rather than guessed values.

**Focused tests in the same commit**

-   Unit test the analysis entry point with a controlled marker
    environment and true/false branches across at least two Python
    candidates.
-   Test a false-marker dependency with `None` release and a true-marker
    dependency with `None` release.
-   Test a dependency without a marker to verify backward-compatible
    inclusion.
-   Test invalid marker/environment input returns an error and no
    success-shaped result.
-   Add a CLI integration test asserting stdout, stderr, and exit code
    for a successful marker analysis and an invalid marker analysis.
-   Keep tests offline by injecting fixture releases/metadata; make no
    live PyPI request.

**Commit gate:** run the focused integration/unit tests, `cargo test`,
`cargo fmt --check`, and relevant Clippy checks. Only then mark the
matching marker-evaluation leaves complete. Implement nested includes,
constraints, and candidate-specific release selection in later commits.
