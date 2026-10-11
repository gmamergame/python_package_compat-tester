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
