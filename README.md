# CompatPy

Find Python versions compatible with your project's dependencies.

CompatPy analyzes a Python project, checks its dependencies against PyPI metadata, and determines which Python versions are compatible with the dependency requirements.

## Features

- Discover dependencies from `requirements.txt`
- Discover dependencies from `pyproject.toml`
- Normalize dependency names using Python packaging conventions
- Parse common version constraints and specifier combinations
- Retrieve package metadata from PyPI
- Evaluate Python-version compatibility for dependency requirements
- Handle stable and prerelease package versions
- Support Python version exclusions
- Recommend compatible Python versions
- Provide a simple command-line interface

## Current status

CompatPy is a focused compatibility checker for Python projects. It is useful for common dependency and Python-version checks, but it is not intended to be treated as a full dependency solver or a complete replacement for a production environment resolver.

Useful today:
- common dependency declarations in `requirements.txt`
- common dependency declarations in `pyproject.toml`
- standard specifier parsing for many requirements
- compatibility checks using published package metadata from PyPI

Known limitations:
- some advanced requirement forms are still partially supported or under active development
- direct URL and some VCS-style requirements may be limited or unevenly handled
- complex dependency graphs and edge-case metadata scenarios may not be fully covered
- results depend on the dependency metadata available from the packages being analyzed

## Installation

### Download a release asset

Pre-built release assets are published on the [GitHub Releases](https://github.com/oluxid/CompatPy/releases) page.

The release workflow builds and uploads binaries for:

- Linux x86_64
- Linux ARM64
- Windows x86_64
- Windows ARM64
- macOS x86_64
- macOS ARM64

It also builds a Windows MSI installer for the x86_64 release.

Typical asset names look like:

```text
compatpy-x86_64-unknown-linux-gnu.tar.gz
compatpy-aarch64-unknown-linux-gnu.tar.gz
compatpy-x86_64-pc-windows-msvc.zip
compatpy-aarch64-pc-windows-msvc.zip
compatpy-x86_64-apple-darwin.tar.gz
compatpy-aarch64-apple-darwin.tar.gz
compatpy-<version>.msi
```

For example, on Windows x86_64 you can download the zip archive or the MSI installer, then run the binary from PowerShell:

```powershell
Expand-Archive compatpy-x86_64-pc-windows-msvc.zip -DestinationPath .\release
.\release\compatpy.exe "C:\path\to\your\project"
```

Alternatively, install the MSI and run `compatpy` directly from PowerShell.

### From source

If you have Rust installed:

```powershell
cargo install --git https://github.com/oluxid/CompatPy.git
```

Or clone the repository and build it locally:

```powershell
git clone https://github.com/oluxid/CompatPy.git
cd CompatPy
cargo build --release
```

The compiled executable will be located in:

```text
target/release/
```

## Usage

Pass the root directory of a Python project:

```powershell
compatpy "C:\path\to\your\project"
```

For requirements with environment markers, provide a JSON file containing
every standardized PEP 508 target-environment field. CompatPy does not infer
target values from the machine running the analysis:

```powershell
compatpy "C:\path\to\your\project" --target-environment .\target.json
```

The JSON file must include `implementation_name`,
`implementation_version`, `os_name`, `platform_machine`,
`platform_python_implementation`, `platform_release`, `platform_system`,
`platform_version`, `python_full_version`, `python_version`, and
`sys_platform`. For example:

```json
{
  "implementation_name": "cpython",
  "implementation_version": "3.12.0",
  "os_name": "posix",
  "platform_machine": "x86_64",
  "platform_python_implementation": "CPython",
  "platform_release": "6.8.0",
  "platform_system": "Linux",
  "platform_version": "generic",
  "python_full_version": "3.12.0",
  "python_version": "3.12",
  "sys_platform": "linux"
}
```

When running a downloaded portable binary directly on Windows:

```powershell
.\release\compatpy.exe "C:\path\to\your\project"
```

CompatPy will discover the project's dependencies, inspect their package metadata, and recommend Python versions that satisfy the declared requirements.

### Help

```powershell
compatpy --help
```

### Version

```powershell
compatpy --version
```

## Supported project files

| File               | Dependency discovery |
| ------------------ | -------------------- |
| `requirements.txt` | Yes                  |
| `pyproject.toml`   | Yes                  |

## How it works

CompatPy follows a simple process:

```text
Python project
      │
      ▼
Discover dependencies
      │
      ▼
Retrieve package metadata
      │
      ▼
Resolve dependency versions
      │
      ▼
Analyze Python compatibility
      │
      ▼
Recommend compatible Python versions
```

The result is based on the compatibility information available from the project's dependencies and their PyPI metadata.

## Requirements

### Running a release binary

No Rust installation is required when using a pre-built release binary.

An internet connection is required when CompatPy needs to retrieve package metadata from PyPI.

### Building from source

Building CompatPy from source requires:

- Rust 1.85 or newer
- Cargo
- Internet access for retrieving dependencies and PyPI metadata

## Development

Clone the repository:

```powershell
git clone https://github.com/oluxid/CompatPy.git
cd CompatPy
```

Run the test suite:

```powershell
cargo test
```

Build a release binary:

```powershell
cargo build --release
```

Format the code:

```powershell
cargo fmt
```

## Contributing

Want to help improve CompatPy? Pick an item from the roadmap and help move it forward!

Check the current implementation before starting work.
For larger changes, open an issue to discuss the proposed approach first.
Keep changes focused and include tests where appropriate.
Update the roadmap when a feature is implemented and verified.

Not every roadmap item is a commitment or a release blocker. Contributions, bug reports, tests, and suggestions are all welcome.

## License

CompatPy is licensed under the MIT License.

See [`LICENSE`](LICENSE) for the full license text.
