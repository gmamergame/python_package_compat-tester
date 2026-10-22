pub(crate) fn is_valid_package_name(name: &str) -> bool {
    let mut characters = name.chars();

    let Some(first) = characters.next() else {
        return false;
    };
    if !first.is_ascii_alphanumeric() {
        return false;
    }

    let mut last = first;
    for character in characters {
        if !character.is_ascii_alphanumeric() && !matches!(character, '.' | '_' | '-') {
            return false;
        }
        last = character;
    }

    last.is_ascii_alphanumeric()
}

pub(crate) fn normalize_package_name(name: &str) -> String {
    let mut normalized = String::with_capacity(name.len());

    for character in name.trim().chars() {
        if matches!(character, '-' | '_' | '.') {
            if !normalized.ends_with('-') {
                normalized.push('-');
            }
        } else {
            normalized.push(character.to_ascii_lowercase());
        }
    }

    test_debug!("normalize_package_name: {name:?} -> {normalized:?}");
    normalized
}
