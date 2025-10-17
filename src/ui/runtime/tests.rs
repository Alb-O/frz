use crate::plugins::api::{AttributeRow, FileRow, SearchData, search::Fs};
use crate::plugins::builtin::files;
use crate::systems::filesystem::{IndexUpdate, ProgressSnapshot};
use crate::ui::App;
use ratatui::{Terminal, backend::TestBackend};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::mpsc;

fn sample_index_update() -> IndexUpdate {
    IndexUpdate {
        files: vec![
            FileRow::filesystem("src/lib.rs", ["src", "*.rs"]),
            FileRow::filesystem("src/main.rs", ["src", "*.rs"]),
            FileRow::filesystem("README.md", ["*.md"]),
        ]
        .into(),
        attributes: vec![
            AttributeRow::new("*.md", 1),
            AttributeRow::new("*.rs", 2),
            AttributeRow::new("src", 2),
        ]
        .into(),
        progress: ProgressSnapshot {
            indexed_attributes: 3,
            indexed_files: 3,
            total_attributes: Some(3),
            total_files: Some(3),
            complete: true,
        },
        reset: true,
        cached_data: None,
    }
}

#[test]
fn initial_files_tab_render_captures_missing_results() {
    let mut app = App::new(SearchData::new());
    app.set_mode(files::mode());
    app.hydrate_initial_results();

    let (tx, rx) = mpsc::channel();
    app.set_index_updates(rx);
    tx.send(sample_index_update()).unwrap();

    app.pump_index_updates();
    app.pump_search_results();

    let mut terminal = Terminal::new(TestBackend::new(80, 20)).unwrap();
    terminal.draw(|frame| app.draw(frame)).unwrap();

    let view = {
        let backend = terminal.backend();
        backend.to_string()
    };
    insta::assert_snapshot!("initial_files_tab_render_captures_missing_results", view);

    assert!(
        app.filtered_len() > 0,
        "expected initial search results to populate without any user input"
    );
}

struct MassiveSyntheticFs {
    total: usize,
}

impl MassiveSyntheticFs {
    fn new(total: usize) -> Self {
        Self { total }
    }
}

struct MassiveIter {
    remaining: usize,
    index: usize,
}

impl Iterator for MassiveIter {
    type Item = io::Result<PathBuf>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.remaining {
            return None;
        }

        let dir = self.index / 512;
        let path = PathBuf::from(format!("dir_{dir:04}/file_{:06}.txt", self.index));
        self.index += 1;
        Some(Ok(path))
    }
}

impl Fs for MassiveSyntheticFs {
    type Iter = MassiveIter;

    fn walk(&self, _root: &Path) -> io::Result<Self::Iter> {
        Ok(MassiveIter {
            remaining: self.total,
            index: 0,
        })
    }
}

#[test]
fn massive_filesystem_initial_load_shows_preview_snapshot() {
    const TOTAL_FILES: usize = 125_000;

    let fs = MassiveSyntheticFs::new(TOTAL_FILES);
    let data = SearchData::from_filesystem_with(&fs, Path::new("/synthetic")).unwrap();
    assert!(
        data.files.len() >= 100_000,
        "expected synthetic filesystem to exceed 100k entries"
    );
    let total_files = data.files.len();
    let total_attributes = data.attributes.len();
    const PREVIEW_SLICE: usize = 512;
    let preview_files: Vec<FileRow> = data.files.iter().take(PREVIEW_SLICE).cloned().collect();
    let preview_attributes = data.attributes.clone();
    drop(data);

    let mut app = App::new(SearchData::new());
    app.set_mode(files::mode());
    app.hydrate_initial_results();

    assert_eq!(
        app.filtered_len(),
        0,
        "no results should be visible before indexing begins"
    );

    let (tx, rx) = mpsc::channel();
    app.set_index_updates(rx);
    tx.send(IndexUpdate {
        files: preview_files.into(),
        attributes: preview_attributes.into(),
        progress: ProgressSnapshot {
            indexed_attributes: total_attributes,
            indexed_files: PREVIEW_SLICE,
            total_attributes: Some(total_attributes),
            total_files: Some(total_files),
            complete: false,
        },
        reset: true,
        cached_data: None,
    })
    .unwrap();

    app.pump_index_updates();
    app.pump_search_results();
    assert!(
        app.filtered_len() > 0,
        "expected preview results to be visible during indexing"
    );

    app.throbber_state.calc_next();

    let mut terminal = Terminal::new(TestBackend::new(100, 30)).unwrap();
    terminal.draw(|frame| app.draw(frame)).unwrap();

    let view = {
        let backend = terminal.backend();
        backend.to_string()
    };

    insta::assert_snapshot!(
        "massive_filesystem_initial_load_shows_preview_snapshot",
        view
    );
}
