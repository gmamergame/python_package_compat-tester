use crate::package_name::normalize_package_name;

#[derive(Debug, PartialEq, Eq)]
pub struct ParsedRequirement {
    pub name: String,
    pub extras: Vec<String>,
    pub specifier: String,
    pub url: Option<String>,
}

pub fn parse_requirement(input: &str) -> Result<ParsedRequirement, String> {
    let input = input.trim();

    if input.is_empty() {
        return Err("Requirement cannot be empty".to_string());
    }

    let (name_and_extras, specifier, url) = match input.split_once('@') {
        Some((name, url)) => {
            let url = url.trim();

            let is_supported_url = ["https://", "http://", "file://"]
                .iter()
                .any(|prefix| url.starts_with(prefix));

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

            if !(is_supported_url || is_supported_vcs_url) {
                return Err("Invalid direct URL".to_string());
            }

            (name.trim(), String::new(), Some(url.to_owned()))
        }
        None => {
            let (name, specifier) = match input.find(['<', '>', '=', '!', '~']) {
                Some(index) => (&input[..index], input[index..].trim().to_string()),
                None => (input, String::new()),
            };

            (name, specifier, None)
        }
    };

    let (name, extras) = match name_and_extras.find('[') {
        Some(start) => {
            let end = name_and_extras
                .strip_suffix(']')
                .ok_or_else(|| "Extras must end with ']'".to_string())?;

            let name = &name_and_extras[..start];

            if end.len() <= start + 1 {
                return Err("Extras cannot be empty".to_string());
            }

            let extras_text = &end[start + 1..];

            let extras: Vec<String> = extras_text
                .split(',')
                .map(str::trim)
                .map(str::to_ascii_lowercase)
                .collect();

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
        None => (name_and_extras.to_string(), Vec::new()),
    };

    if name.is_empty()
        || !name
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || "._-".contains(character))
    {
        return Err("Invalid package name".to_string());
    }

    Ok(ParsedRequirement {
        name: normalize_package_name(&name),
        extras,
        specifier,
        url,
    })
}
