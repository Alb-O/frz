use std::collections::HashMap;
use std::fmt;

use crate::plugins::{SearchMode, builtin::{FACETS_MODE, FILES_MODE}};
use crate::types::{SearchData, UiConfig};

#[derive(Debug, Clone)]
struct IndexEntry {
    total: Option<usize>,
    indexed: usize,
}

impl IndexEntry {
    fn new(total: Option<usize>) -> Self {
        Self { total, indexed: 0 }
    }

    fn record(&mut self, count: usize) {
        self.indexed = self.indexed.max(count);
    }

    fn set_total(&mut self, total: Option<usize>) {
        if let Some(total) = total {
            let adjusted = total.max(self.indexed);
            self.total = Some(adjusted);
        } else {
            self.total = None;
        }
    }

    fn is_complete(&self) -> bool {
        match self.total {
            Some(0) => true,
            Some(total) => self.indexed >= total,
            None => false,
        }
    }
}

/// Tracks indexing progress for one or more plugins contributing to the UI.
#[derive(Debug, Clone)]
pub struct IndexProgress {
    entries: HashMap<SearchMode, IndexEntry>,
    order: Vec<SearchMode>,
    complete: bool,
}

impl IndexProgress {
    fn ensure_entry(&mut self, mode: SearchMode) -> &mut IndexEntry {
        let order = &mut self.order;
        self.entries.entry(mode).or_insert_with(|| {
            if !order.contains(&mode) {
                order.push(mode);
            }
            IndexEntry::new(None)
        })
    }

    #[must_use]
    pub fn for_modes<I>(modes: I) -> Self
    where
        I: IntoIterator<Item = SearchMode>,
    {
        let mut progress = Self {
            entries: HashMap::new(),
            order: Vec::new(),
            complete: false,
        };
        for mode in modes {
            progress.ensure_entry(mode);
        }
        progress
    }

    #[must_use]
    pub fn from_plugins<I>(data: &SearchData, modes: I) -> Self
    where
        I: IntoIterator<Item = SearchMode>,
    {
        let modes_vec: Vec<SearchMode> = modes.into_iter().collect();
        let mut progress = Self::for_modes(modes_vec.clone());
        for mode in &modes_vec {
            let len = (mode.behavior().dataset_len)(data);
            progress.set_total_for_mode(*mode, Some(len));
            progress.record_indexed_for_mode(*mode, len);
        }
        progress.update_completion();
        progress
    }

    #[must_use]
    pub fn new(total_facets: usize, total_files: usize) -> Self {
        let mut progress = Self::for_modes([FACETS_MODE, FILES_MODE]);
        progress.set_totals(Some(total_facets), Some(total_files));
        progress
    }

    #[must_use]
    pub fn with_unknown_totals() -> Self {
        Self::for_modes([FACETS_MODE, FILES_MODE])
    }

    pub fn set_totals(&mut self, total_facets: Option<usize>, total_files: Option<usize>) {
        self.set_total_for_mode(FACETS_MODE, total_facets);
        self.set_total_for_mode(FILES_MODE, total_files);
        self.update_completion();
    }

    pub fn record_indexed(&mut self, facets: usize, files: usize) {
        self.record_indexed_for_mode(FACETS_MODE, facets);
        self.record_indexed_for_mode(FILES_MODE, files);
        self.update_completion();
    }

    pub fn mark_complete(&mut self) {
        self.complete = true;
    }

    pub fn status<I>(&self, ui: &UiConfig, modes: I) -> (String, bool)
    where
        I: IntoIterator<Item = SearchMode>,
    {
        let mut parts = Vec::new();
        for mode in modes.into_iter() {
            if let Some(entry) = self.entries.get(&mode) {
                let label = mode.count_label(ui);
                let progress = self.format_progress(entry);
                parts.push(format!("Indexed {}: {}", label, progress));
            }
        }
        if parts.is_empty() {
            for mode in &self.order {
                if let Some(entry) = self.entries.get(mode) {
                    let label = mode.count_label(ui);
                    let progress = self.format_progress(entry);
                    parts.push(format!("Indexed {}: {}", label, progress));
                }
            }
        }
        let label = if parts.is_empty() {
            String::new()
        } else {
            parts.join(" • ")
        };
        (label, self.complete)
    }

    pub fn refresh_from_data<I>(&mut self, data: &SearchData, modes: I)
    where
        I: IntoIterator<Item = SearchMode>,
    {
        for mode in modes {
            let len = (mode.behavior().dataset_len)(data);
            self.set_total_for_mode(mode, Some(len));
            self.record_indexed_for_mode(mode, len);
        }
        self.update_completion();
    }

    fn record_indexed_for_mode(&mut self, mode: SearchMode, count: usize) {
        let entry = self.ensure_entry(mode);
        entry.record(count);
    }

    fn set_total_for_mode(&mut self, mode: SearchMode, total: Option<usize>) {
        let entry = self.ensure_entry(mode);
        entry.set_total(total);
    }

    fn update_completion(&mut self) {
        self.complete = !self.entries.is_empty()
            && self.entries.values().all(IndexEntry::is_complete);
    }

    fn format_progress(&self, entry: &IndexEntry) -> ProgressDisplay {
        match entry.total {
            Some(0) => ProgressDisplay::Fixed(0),
            Some(total) if self.complete => ProgressDisplay::Fixed(total),
            Some(total) => ProgressDisplay::Ratio {
                indexed: entry.indexed,
                total,
            },
            None => ProgressDisplay::Fixed(entry.indexed),
        }
    }
}

#[derive(Debug)]
enum ProgressDisplay {
    Fixed(usize),
    Ratio { indexed: usize, total: usize },
}

impl fmt::Display for ProgressDisplay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Fixed(value) => write!(f, "{}", value),
            Self::Ratio { indexed, total } => write!(f, "{}/{}", indexed, total),
        }
    }
}

impl From<&SearchData> for IndexProgress {
    fn from(data: &SearchData) -> Self {
        let mut progress = Self::new(data.facets.len(), data.files.len());
        progress.record_indexed(data.facets.len(), data.files.len());
        progress.mark_complete();
        progress
    }
}

#[cfg(test)]
mod tests {
    use super::IndexProgress;
    use crate::plugins::builtin::{
        FACETS_DEFINITION, FACETS_MODE, FILES_DEFINITION, FILES_MODE,
    };
    use crate::types::{FacetRow, FileRow, SearchData, UiConfig};

    fn ui_config() -> UiConfig {
        UiConfig::for_definitions([&FACETS_DEFINITION, &FILES_DEFINITION])
    }

    #[test]
    fn reports_in_progress_counts() {
        let ui = ui_config();
        let mut progress = IndexProgress::new(10, 20);
        progress.record_indexed(3, 5);
        let (label, complete) = progress.status(&ui, [FACETS_MODE, FILES_MODE]);
        assert_eq!(label, "Indexed Facets: 3/10 • Indexed Files: 5/20");
        assert!(!complete);
    }

    #[test]
    fn collapses_totals_on_completion() {
        let ui = ui_config();
        let mut progress = IndexProgress::new(4, 2);
        progress.record_indexed(4, 2);
        let (label, complete) = progress.status(&ui, [FACETS_MODE, FILES_MODE]);
        assert_eq!(label, "Indexed Facets: 4 • Indexed Files: 2");
        assert!(complete);
    }

    #[test]
    fn ignores_regressions_after_completion() {
        let ui = ui_config();
        let mut progress = IndexProgress::new(4, 2);
        progress.record_indexed(4, 2);
        progress.record_indexed(1, 1);
        let (label, complete) = progress.status(&ui, [FACETS_MODE, FILES_MODE]);
        assert_eq!(label, "Indexed Facets: 4 • Indexed Files: 2");
        assert!(complete);
    }

    #[test]
    fn reports_empty_index() {
        let ui = ui_config();
        let progress = IndexProgress::new(0, 0);
        let (label, complete) = progress.status(&ui, [FACETS_MODE, FILES_MODE]);
        assert_eq!(label, "Indexed Facets: 0 • Indexed Files: 0");
        assert!(complete);
    }

    #[test]
    fn refreshes_from_search_data_snapshot() {
        let ui = ui_config();
        let mut data = SearchData::default();
        data.facets.push(FacetRow::new("tag", 1));
        data.files
            .push(FileRow::new("path.txt", Vec::<String>::new()));

        let mut progress = IndexProgress::new(10, 10);
        progress.refresh_from_data(&data, [FACETS_MODE, FILES_MODE]);

        let (label, complete) = progress.status(&ui, [FACETS_MODE, FILES_MODE]);
        assert_eq!(label, "Indexed Facets: 1 • Indexed Files: 1");
        assert!(complete);
    }

    #[test]
    fn reports_unknown_totals_during_streaming() {
        let ui = ui_config();
        let mut progress = IndexProgress::with_unknown_totals();
        progress.record_indexed(5, 12);

        let (label, complete) = progress.status(&ui, [FACETS_MODE, FILES_MODE]);
        assert_eq!(label, "Indexed Facets: 5 • Indexed Files: 12");
        assert!(!complete);

        progress.set_totals(Some(5), Some(12));
        progress.mark_complete();
        let (label, complete) = progress.status(&ui, [FACETS_MODE, FILES_MODE]);
        assert_eq!(label, "Indexed Facets: 5 • Indexed Files: 12");
        assert!(complete);
    }
}
