use super::builtins::BUILT_IN_DEFINITIONS;
use super::types::{
    AliasConflict, Theme, ThemeDefinition, ThemeDescriptor, ThemeRegistration,
    ThemeRegistrationReport,
};
use std::collections::{BTreeMap, HashMap};
use std::sync::{OnceLock, RwLock, RwLockReadGuard, RwLockWriteGuard};

#[derive(Debug)]
struct ThemeEntry {
    display_name: String,
    theme: Theme,
    aliases: Vec<String>,
}

impl ThemeEntry {
    fn new(name: String, theme: Theme) -> Self {
        Self {
            display_name: name,
            theme,
            aliases: Vec::new(),
        }
    }
}

#[derive(Debug, Default)]
struct ThemeRegistry {
    canonical: BTreeMap<String, ThemeEntry>,
    aliases: HashMap<String, String>,
}

impl ThemeRegistry {
    fn register(&mut self, registration: ThemeRegistration, report: &mut ThemeRegistrationReport) {
        let ThemeRegistration {
            name,
            theme,
            aliases,
        } = registration;

        let normalized = normalize_name(&name);
        let mut removed_aliases = Vec::new();

        match self.canonical.get_mut(&normalized) {
            Some(entry) => {
                report.replaced.push(entry.display_name.clone());
                removed_aliases = std::mem::take(&mut entry.aliases);
                entry.display_name = name.clone();
                entry.theme = theme;
            }
            None => {
                self.canonical
                    .insert(normalized.clone(), ThemeEntry::new(name.clone(), theme));
                report.inserted.push(name.clone());
            }
        }

        for alias in removed_aliases {
            self.aliases.remove(&normalize_name(&alias));
        }

        for alias in aliases {
            let alias_normalized = normalize_name(&alias);

            if alias_normalized == normalized {
                continue;
            }

            match self.aliases.get(&alias_normalized) {
                Some(existing) if existing != &normalized => {
                    report.alias_conflicts.push(AliasConflict {
                        alias,
                        existing: existing.clone(),
                        attempted: normalized.clone(),
                    });
                }
                _ => {
                    self.aliases
                        .insert(alias_normalized.clone(), normalized.clone());

                    if let Some(entry) = self.canonical.get_mut(&normalized)
                        && !entry
                            .aliases
                            .iter()
                            .any(|existing| existing.eq_ignore_ascii_case(&alias))
                    {
                        entry.aliases.push(alias);
                    }
                }
            }
        }

        if let Some(entry) = self.canonical.get_mut(&normalized) {
            entry
                .aliases
                .sort_unstable_by_key(|a| a.to_ascii_lowercase());
        }
    }

    fn get(&self, name: &str) -> Option<Theme> {
        let normalized = normalize_name(name);

        if let Some(entry) = self.canonical.get(&normalized) {
            return Some(entry.theme);
        }

        let target = self.aliases.get(&normalized)?;
        self.canonical.get(target).map(|entry| entry.theme)
    }

    fn names(&self) -> Vec<String> {
        self.canonical
            .values()
            .map(|entry| entry.display_name.clone())
            .collect()
    }

    fn descriptors(&self) -> Vec<ThemeDescriptor> {
        self.canonical
            .values()
            .map(|entry| ThemeDescriptor {
                name: entry.display_name.clone(),
                aliases: entry.aliases.clone(),
                theme: entry.theme,
            })
            .collect()
    }
}

static REGISTRY: OnceLock<RwLock<ThemeRegistry>> = OnceLock::new();

fn registry_lock() -> &'static RwLock<ThemeRegistry> {
    REGISTRY.get_or_init(|| {
        let mut registry = ThemeRegistry::default();
        let mut report = ThemeRegistrationReport::default();

        for definition in BUILT_IN_DEFINITIONS {
            registry.register(definition.to_registration(), &mut report);
        }

        debug_assert!(report.replaced.is_empty(), "duplicate built-in theme names");
        debug_assert!(
            report.alias_conflicts.is_empty(),
            "conflicting built-in theme aliases"
        );

        RwLock::new(registry)
    })
}

fn read_registry() -> RwLockReadGuard<'static, ThemeRegistry> {
    registry_lock()
        .read()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn write_registry() -> RwLockWriteGuard<'static, ThemeRegistry> {
    registry_lock()
        .write()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn normalize_name(name: &str) -> String {
    name.trim().to_ascii_lowercase()
}

/// Register an additional collection of themes at runtime.
#[must_use]
pub fn register_additional<I>(registrations: I) -> ThemeRegistrationReport
where
    I: IntoIterator<Item = ThemeRegistration>,
{
    let mut report = ThemeRegistrationReport::default();
    let mut registry = write_registry();

    for registration in registrations {
        registry.register(registration, &mut report);
    }

    report
}

/// Register additional static theme definitions.
#[must_use]
pub fn register_definitions(definitions: &[ThemeDefinition]) -> ThemeRegistrationReport {
    register_additional(definitions.iter().copied().map(ThemeRegistration::from))
}

/// Lookup a Theme by case-insensitive name.
#[must_use]
pub fn by_name(name: &str) -> Option<Theme> {
    read_registry().get(name)
}

/// Return the canonical theme names known to the UI.
#[must_use]
pub fn names() -> Vec<String> {
    let mut names = read_registry().names();
    names.sort_unstable_by_key(|a| a.to_ascii_lowercase());
    names
}

/// Produce detailed descriptors for every known theme.
#[must_use]
pub fn descriptors() -> Vec<ThemeDescriptor> {
    let mut descriptors = read_registry().descriptors();
    descriptors.sort_unstable_by(|a, b| {
        a.name
            .to_ascii_lowercase()
            .cmp(&b.name.to_ascii_lowercase())
    });
    descriptors
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::style::{Color, Style};

    fn sample_theme() -> Theme {
        Theme {
            header: Style::new().bg(Color::Blue),
            row_highlight: Style::new().bg(Color::Cyan),
            prompt: Style::new().fg(Color::White),
            empty: Style::new().fg(Color::DarkGray),
            highlight: Style::new().fg(Color::Yellow),
        }
    }

    #[test]
    fn builtin_themes_are_registered() {
        let names = names();
        assert!(names.iter().any(|name| name == "slate"));
        assert!(by_name("slate").is_some());
    }

    #[test]
    fn registering_additional_theme_adds_aliases() {
        let report = register_additional([ThemeRegistration::new("test-theme", sample_theme())
            .aliases(["Test Theme", "test_theme"])]);
        assert!(report.alias_conflicts.is_empty());

        assert!(names().iter().any(|name| name == "test-theme"));
        assert!(by_name("test theme").is_some());
        assert!(by_name("TEST_THEME").is_some());

        let descriptors = descriptors();
        let descriptor = descriptors
            .into_iter()
            .find(|descriptor| descriptor.name == "test-theme")
            .expect("descriptor should exist");

        assert!(
            descriptor
                .aliases
                .iter()
                .any(|alias| alias.eq_ignore_ascii_case("test theme"))
        );
    }

    #[test]
    fn names_are_sorted_case_insensitively() {
        let sorted = names();
        let mut manual = sorted.clone();
        manual.sort_unstable_by_key(|a| a.to_ascii_lowercase());
        assert_eq!(sorted, manual);
    }
}
