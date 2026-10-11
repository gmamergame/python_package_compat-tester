use std::env;
use std::path::Path;
use std::process;

use compatpy::{
    VERSION, compatible_python_versions, discover_dependencies, fetch_pypi_metadata,
    select_package_release,
};

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.len() == 1 {
        match args[0].as_str() {
            "--version" | "-v" => {
                println!("compatpy {VERSION}");
                return;
            }
            "--help" | "-h" => {
                println!("Usage: compatpy <project-directory>");
                println!("       compatpy --help");
                println!("       compatpy --version");
                return;
            }
            _ => {}
        }
    }

    if args.len() != 1 {
        eprintln!("Usage: compatpy <project-directory>");
        process::exit(1);
    }

    let project_path = Path::new(&args[0]);

    if !project_path.exists() || !project_path.is_dir() {
        eprintln!(
            "Error: '{}' is not a valid directory.",
            project_path.display()
        );
        process::exit(1);
    }

    println!("Scanning");
    println!("{}", project_path.display());

    match discover_dependencies(project_path) {
        Ok(dependencies) => {
            if dependencies.is_empty() {
                println!("No dependencies found.");
                return;
            }

            println!();
            println!("Dependencies:");

            let mut selected_releases = Vec::new();

            for dependency in dependencies {
                println!(
                    "  {} | required: {} | ({})",
                    dependency.name,
                    dependency.specifier.as_deref().unwrap_or("any"),
                    dependency.source.display()
                );

                match fetch_pypi_metadata(&dependency.name) {
                    Ok(metadata) => {
                        match select_package_release(&metadata, dependency.specifier.as_deref()) {
                            Ok(Some(release)) => {
                                println!(
                                    "    PyPI: {} | requires Python: {}",
                                    release.version,
                                    release.requires_python.as_deref().unwrap_or("any")
                                );

                                selected_releases.push(release.clone());
                            }
                            Ok(None) => {
                                println!("    PyPI: no matching release");
                            }
                            Err(error) => {
                                eprintln!("Error: {error}");
                                process::exit(1);
                            }
                        }
                    }
                    Err(error) => {
                        println!("    PyPI: unavailable ({error})");
                    }
                }
            }

            println!();
            println!("compatible Python versions:");

            let compatible_versions = compatible_python_versions(&selected_releases);

            if compatible_versions.is_empty() {
                println!("  None");
            } else {
                for version in compatible_versions {
                    println!("  {version}");
                }
            }
        }
        Err(error) => {
            eprintln!("Error: {error}");
            process::exit(1);
        }
    }
}
