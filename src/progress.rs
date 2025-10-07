use std::fmt;

use crate::types::SearchData;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct IndexCounts {
    facets: usize,
    files: usize,
}

impl IndexCounts {
    const fn new(facets: usize, files: usize) -> Self {
        Self { facets, files }
    }

    fn update_max(&mut self, facets: usize, files: usize) {
        self.facets = self.facets.max(facets);
        self.files = self.files.max(files);
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct IndexTotals {
    facets: Option<usize>,
    files: Option<usize>,
}

impl IndexTotals {
    const fn new(facets: Option<usize>, files: Option<usize>) -> Self {
        Self { facets, files }
    }

    fn set(&mut self, facets: Option<usize>, files: Option<usize>, indexed: &IndexCounts) {
        self.facets = facets.map(|total| total.max(indexed.facets));
        self.files = files.map(|total| total.max(indexed.files));
    }
}

/// Tracks how many items have been indexed relative to the total expected counts.
///
/// The UI can query this tracker to decide whether to render the throbber and to
/// format a progress label that remains stable once indexing has completed.
#[derive(Debug, Clone)]
pub struct IndexProgress {
    totals: IndexTotals,
    indexed: IndexCounts,
    complete: bool,
}

impl IndexProgress {
    /// Create a new tracker with the provided total counts.
    #[must_use]
    pub const fn new(total_facets: usize, total_files: usize) -> Self {
        Self::with_totals(Some(total_facets), Some(total_files))
    }

    /// Create a tracker where the totals are unknown and will be supplied later.
    #[must_use]
    pub const fn with_unknown_totals() -> Self {
        Self::with_totals(None, None)
    }

    const fn with_totals(total_facets: Option<usize>, total_files: Option<usize>) -> Self {
        let complete = matches!(total_facets, Some(0)) && matches!(total_files, Some(0));
        Self {
            totals: IndexTotals::new(total_facets, total_files),
            indexed: IndexCounts::new(0, 0),
            complete,
        }
    }

    /// Update the expected total counts.
    ///
    /// The totals never shrink below the number of items that have already been
    /// indexed to ensure the completion state remains truthful if totals are
    /// adjusted after progress updates have been recorded.
    pub fn set_totals(&mut self, total_facets: Option<usize>, total_files: Option<usize>) {
        self.totals.set(total_facets, total_files, &self.indexed);
        self.update_completion();
    }

    /// Record the number of indexed facets and files.
    ///
    /// The stored counts only ever increase, so callers can safely report
    /// transient values (for example filtered result counts) without affecting
    /// the eventual completion snapshot once the full index is available.
    pub fn record_indexed(&mut self, facets: usize, files: usize) {
        self.indexed.update_max(facets, files);
        self.update_completion();
    }

    /// Mark indexing as complete regardless of the recorded totals.
    pub fn mark_complete(&mut self) {
        self.complete = true;
    }

    /// Return a formatted status label and a completion flag suitable for the UI.
    #[must_use]
    pub fn status(&self, facet_label: &str, file_label: &str) -> (String, bool) {
        let facet_progress = self.format_progress(self.indexed.facets, self.totals.facets);
        let file_progress = self.format_progress(self.indexed.files, self.totals.files);
        let progress = format!(
            "Indexed {}: {} • Indexed {}: {}",
            facet_label, facet_progress, file_label, file_progress,
        );
        (progress, self.complete)
    }

    fn format_progress(&self, indexed: usize, total: Option<usize>) -> ProgressDisplay {
        match total {
            Some(0) => ProgressDisplay::Fixed(0),
            Some(total) if self.complete => ProgressDisplay::Fixed(total),
            Some(total) => ProgressDisplay::Ratio { indexed, total },
            None => ProgressDisplay::Unknown { indexed },
        }
    }

    fn update_completion(&mut self) {
        let facets_complete = match self.totals.facets {
            Some(total) => total == 0 || self.indexed.facets >= total,
            None => false,
        };
        let files_complete = match self.totals.files {
            Some(total) => total == 0 || self.indexed.files >= total,
            None => false,
        };

        if facets_complete && files_complete {
            self.complete = true;
        }
    }

    /// Reconcile the tracked counts with the provided search data snapshot.
    pub fn refresh_from_data(&mut self, data: &SearchData) {
        let total_facets = data.facets.len();
        let total_files = data.files.len();
        self.set_totals(Some(total_facets), Some(total_files));
        self.record_indexed(total_facets, total_files);
        self.mark_complete();
    }
}

#[derive(Debug)]
enum ProgressDisplay {
    Fixed(usize),
    Ratio { indexed: usize, total: usize },
    Unknown { indexed: usize },
}

impl fmt::Display for ProgressDisplay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Fixed(value) => write!(f, "{}", value),
            Self::Ratio { indexed, total } => write!(f, "{}/{}", indexed, total),
            Self::Unknown { indexed } => write!(f, "{}/?", indexed),
        }
    }
}

impl From<&SearchData> for IndexProgress {
    fn from(data: &SearchData) -> Self {
        let total_facets = data.facets.len();
        let total_files = data.files.len();
        let mut progress = IndexProgress::new(total_facets, total_files);
        progress.record_indexed(total_facets, total_files);
        progress.mark_complete();
        progress
    }
}

#[cfg(test)]
mod tests {
    use super::IndexProgress;
    use crate::types::{FacetRow, FileRow, SearchData};

    #[test]
    fn reports_in_progress_counts() {
        let mut progress = IndexProgress::new(10, 20);
        progress.record_indexed(3, 5);
        let (label, complete) = progress.status("Facets", "Files");
        assert_eq!(label, "Indexed Facets: 3/10 • Indexed Files: 5/20");
        assert!(!complete);
    }

    #[test]
    fn collapses_totals_on_completion() {
        let mut progress = IndexProgress::new(4, 2);
        progress.record_indexed(4, 2);
        let (label, complete) = progress.status("Facets", "Files");
        assert_eq!(label, "Indexed Facets: 4 • Indexed Files: 2");
        assert!(complete);
    }

    #[test]
    fn ignores_regressions_after_completion() {
        let mut progress = IndexProgress::new(4, 2);
        progress.record_indexed(4, 2);
        progress.record_indexed(1, 1);
        let (label, complete) = progress.status("Facets", "Files");
        assert_eq!(label, "Indexed Facets: 4 • Indexed Files: 2");
        assert!(complete);
    }

    #[test]
    fn reports_empty_index() {
        let progress = IndexProgress::new(0, 0);
        let (label, complete) = progress.status("Facets", "Files");
        assert_eq!(label, "Indexed Facets: 0 • Indexed Files: 0");
        assert!(complete);
    }

    #[test]
    fn refreshes_from_search_data_snapshot() {
        let mut data = SearchData::default();
        data.facets.push(FacetRow::new("tag", 1));
        data.files
            .push(FileRow::new("path.txt", Vec::<String>::new()));

        let mut progress = IndexProgress::new(10, 10);
        progress.refresh_from_data(&data);

        let (label, complete) = progress.status("Facets", "Files");
        assert_eq!(label, "Indexed Facets: 1 • Indexed Files: 1");
        assert!(complete);
    }

    #[test]
    fn reports_unknown_totals_during_streaming() {
        let mut progress = IndexProgress::with_unknown_totals();
        progress.record_indexed(5, 12);

        let (label, complete) = progress.status("Facets", "Files");
        assert_eq!(label, "Indexed Facets: 5/? • Indexed Files: 12/?");
        assert!(!complete);

        progress.set_totals(Some(5), Some(12));
        progress.mark_complete();
        let (label, complete) = progress.status("Facets", "Files");
        assert_eq!(label, "Indexed Facets: 5 • Indexed Files: 12");
        assert!(complete);
    }
}
