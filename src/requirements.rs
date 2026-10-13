use crate::package_name::normalize_package_name;

// Represents a parsed package requirement.
#[derive(Debug, PartialEq, Eq)]
pub struct ParsedRequirement {
    pub name: String,        // Normalized package name
    pub extras: Vec<String>, // Optional extras, e.g. ["security"]
    pub specifier: String,   // Version constraint, e.g. ">=1.0"
    pub url: Option<String>, // Optional direct URL
}

// Parse a requirement string or return an error if invalid.
pub fn parse_requirement(input: &str) -> Result<ParsedRequirement, String> {
    // Remove surrounding whitespace.
    let input = input.trim();

    if input.is_empty() {
        return Err("Requirement cannot be empty".to_string());
    }

    // Split into package name, version specifier, and optional URL.
    let (name_and_extras, specifier, url) = match input.split_once('@') {
        // Handle direct URL requirements.
        Some((name, url)) => {
            let url = url.trim();

            // Supported standard URL schemes.
            let is_supported_url = ["https://", "http://", "file://"]
                .iter()
                .any(|prefix| url.starts_with(prefix));

            // Supported version-control URL schemes.
            let is_supported_vcs_url = [
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
            .any(|prefix| url.starts_with(prefix));

            // Reject unsupported URL schemes.
            if !(is_supported_url || is_supported_vcs_url) {
                return Err("Invalid direct URL".to_string());
            }

            (name.trim(), String::new(), Some(url.to_owned()))
        }
        // Handle regular requirements with optional version constraints.
        None => {
            let (name, specifier) = match input.find(['<', '>', '=', '!', '~']) {
                // Split at the first version operator.
                Some(index) => (&input[..index], input[index..].trim().to_string()),
                None => (input, String::new()),
            };

            (name, specifier, None)
        }
    };

    // Extract optional extras enclosed in square brackets.
    let (name, extras) = match name_and_extras.find('[') {
        Some(start) => {
            // Require a closing bracket.
            let end = name_and_extras
                .strip_suffix(']')
                .ok_or_else(|| "Extras must end with ']'".to_string())?;

            let name = &name_and_extras[..start];

            // Reject empty extras, e.g. "package[]".
            if end.len() <= start + 1 {
                return Err("Extras cannot be empty".to_string());
            }

            // Split extras by commas, trim whitespace, and lowercase them.
            let extras_text = &end[start + 1..];
            let extras: Vec<String> = extras_text
                .split(',')
                .map(str::trim)
                .map(str::to_ascii_lowercase)
                .collect();

            // Allow only ASCII letters, digits, periods, underscores, and hyphens.
            if extras.iter().any(|extra| {
                extra.is_empty()
                    || !extra.chars().all(|character| {
                        character.is_ascii_alphanumeric() || "._-".contains(character)
                    })
            }) {
                return Err("Invalid extra name".to_string());
            }

            (name.to_string(), extras)
        }
        // No brackets means no extras.
        None => (name_and_extras.to_string(), Vec::new()),
    };

    // Validate the package name.
    if name.is_empty()
        || !name
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || "._-".contains(character))
    {
        return Err("Invalid package name".to_string());
    }

    // Return the parsed requirement with a normalized package name.
    Ok(ParsedRequirement {
        name: normalize_package_name(&name),
        extras,
        specifier,
        url,
    })
}
