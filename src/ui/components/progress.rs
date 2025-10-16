use std::collections::HashMap;

use crate::types::SearchData;

#[derive(Default, Clone, Debug)]
struct ProgressEntry {
    indexed: usize,
    total: Option<usize>,
}

impl ProgressEntry {
    fn set_total(&mut self, total: Option<usize>) {
        if let (Some(total), Some(current_total)) = (total, self.total)
            && total < current_total
        {
            self.total = Some(current_total);
            return;
        }
        if let Some(total) = total
            && total < self.indexed
        {
            self.total = Some(self.indexed);
            return;
        }
        self.total = total;
    }

    fn record(&mut self, count: usize) {
        if count > self.indexed {
            self.indexed = count;
        }
    }

    fn is_complete(&self) -> bool {
        match self.total {
            Some(0) => true,
            Some(total) => self.indexed >= total,
            None => false,
        }
    }

    fn format(&self) -> ProgressDisplay {
        match self.total {
            Some(0) => ProgressDisplay::Fixed(0),
            Some(total) if self.is_complete() => ProgressDisplay::Fixed(total),
            Some(total) => ProgressDisplay::Ratio {
                indexed: self.indexed,
                total,
            },
            None => ProgressDisplay::Fixed(self.indexed),
        }
    }
}

/// Tracks how many items have been indexed relative to the total expected counts.
#[derive(Debug, Clone, Default)]
pub struct IndexProgress {
    entries: HashMap<&'static str, ProgressEntry>,
    order: Vec<&'static str>,
    complete: bool,
}

impl IndexProgress {
    /// Create a progress tracker without any datasets registered.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a tracker where totals are unknown and will be supplied later.
    #[must_use]
    pub fn with_unknown_totals() -> Self {
        Self::new()
    }

    /// Ensure a dataset is tracked by the progress monitor.
    pub fn register_dataset(&mut self, key: &'static str) {
        if !self.entries.contains_key(key) {
            self.entries.insert(key, ProgressEntry::default());
            self.order.push(key);
        }
    }

    /// Record indexed counts for one or more datasets.
    pub fn record_indexed(&mut self, updates: &[(&'static str, usize)]) {
        for (key, count) in updates {
            self.register_dataset(key);
            if let Some(entry) = self.entries.get_mut(key) {
                entry.record(*count);
            }
        }
        self.update_completion();
    }

    /// Update total counts for one or more datasets.
    pub fn set_totals(&mut self, totals: &[(&'static str, Option<usize>)]) {
        for (key, total) in totals {
            self.register_dataset(key);
            if let Some(entry) = self.entries.get_mut(key) {
                entry.set_total(*total);
            }
        }
        self.update_completion();
    }

    /// Mark indexing as complete regardless of recorded totals.
    pub fn mark_complete(&mut self) {
        self.complete = true;
    }

    /// Return a formatted status label and a completion flag suitable for the UI.
    #[must_use]
    pub fn status(&self, labels: &[(&str, String)]) -> (String, bool) {
        let mut segments = Vec::new();
        for key in &self.order {
            let entry = match self.entries.get(key) {
                Some(entry) => entry,
                None => continue,
            };
            let label = labels
                .iter()
                .find_map(|(id, label)| {
                    if *id == *key {
                        Some(label.as_str())
                    } else {
                        None
                    }
                })
                .unwrap_or(*key);
            segments.push(format!("Indexed {}: {}", label, entry.format()));
        }
        let status = segments.join(" • ");
        (status, self.complete)
    }

    fn update_completion(&mut self) {
        if self.entries.is_empty() {
            self.complete = false;
            return;
        }
        self.complete = self.entries.values().all(ProgressEntry::is_complete);
    }

    /// Reconcile the tracked counts with the provided search data snapshot.
    pub fn refresh_from_data(
        &mut self,
        data: &SearchData,
        datasets: impl IntoIterator<Item = (&'static str, usize)>,
    ) {
        let mut totals = Vec::new();
        let mut indexed = Vec::new();
        for (key, count) in datasets {
            totals.push((key, Some(count)));
            indexed.push((key, count));
        }
        self.set_totals(&totals);
        self.record_indexed(&indexed);
        self.mark_complete();
        if self.entries.is_empty() {
            // Fallback for datasets not explicitly registered.
            self.register_dataset("facets");
            self.register_dataset("files");
            self.record_indexed(&[("facets", data.facets.len()), ("files", data.files.len())]);
            self.set_totals(&[
                ("facets", Some(data.facets.len())),
                ("files", Some(data.files.len())),
            ]);
            self.mark_complete();
        }
    }
}

#[derive(Debug)]
enum ProgressDisplay {
    Fixed(usize),
    Ratio { indexed: usize, total: usize },
}

impl std::fmt::Display for ProgressDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Fixed(value) => write!(f, "{}", value),
            Self::Ratio { indexed, total } => write!(f, "{}/{}", indexed, total),
        }
    }
}
