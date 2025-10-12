use std::collections::HashSet;
use std::env;
use std::path::{Path, PathBuf};

/// Normalize and deduplicate file extensions provided by the user.
pub(super) fn sanitize_extensions(values: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut cleaned = Vec::new();
    for value in values {
        let normalized = value.trim().trim_start_matches('.').to_ascii_lowercase();
        if normalized.is_empty() {
            continue;
        }
        if seen.insert(normalized.clone()) {
            cleaned.push(normalized);
        }
    }
    cleaned
}

/// Remove empty headers and trim whitespace from the provided values.
pub(super) fn sanitize_headers(headers: Vec<String>) -> Vec<String> {
    headers
        .into_iter()
        .map(|header| header.trim().to_string())
        .filter(|header| !header.is_empty())
        .collect()
}

/// Determine a sensible default title for the UI given the resolved root path.
pub(super) fn default_title_for(root: &Path) -> String {
    fn shorten(path: &Path) -> String {
        if let Some(home_os) = env::var_os("HOME") {
            let home = PathBuf::from(home_os);
            if let Ok(rel) = path.strip_prefix(&home) {
                if rel.components().next().is_none() {
                    return "~".to_string();
                }
                let sep = std::path::MAIN_SEPARATOR;
                return format!("~{}{}", sep, rel.display());
            }
        }
        path.display().to_string()
    }

    shorten(root)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn extensions_are_cleaned_and_deduplicated() {
        let cleaned =
            sanitize_extensions(vec![" .RS ".into(), "rs".into(), "".into(), ".Txt".into()]);
        assert_eq!(cleaned, vec!["rs", "txt"]);
    }

    #[test]
    fn headers_are_trimmed_and_filtered() {
        let headers = sanitize_headers(vec![" foo ".into(), "".into(), "bar".into()]);
        assert_eq!(headers, vec!["foo", "bar"]);
    }

    #[test]
    fn default_title_prefers_home_relative_paths() {
        let dir = tempdir().unwrap();
        let home = dir.path();
        let old_home = env::var_os("HOME");
        // SAFETY: Adjusting the HOME environment variable for the duration of this test.
        unsafe {
            env::set_var("HOME", home.as_os_str());
        }
        let inside = home.join("projects/foo");

        let title = default_title_for(&inside);

        assert!(title.starts_with("~"));

        if let Some(value) = old_home {
            // SAFETY: Restoring previous HOME value captured at the start of the test.
            unsafe {
                env::set_var("HOME", value);
            }
        } else {
            unsafe {
                env::remove_var("HOME");
            }
        }
    }
}
