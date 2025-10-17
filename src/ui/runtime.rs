use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Result, anyhow};
use ratatui::crossterm::event::{self, Event, KeyEventKind};

use frz_plugin_api::{SearchData, SearchOutcome};

use super::App;

/// Construct an [`App`] for the provided data and run it to completion.
pub fn run(data: SearchData) -> Result<SearchOutcome> {
    let mut app = App::new(data);
    app.run()
}

impl<'a> App<'a> {
    /// Pump the terminal event loop until the user exits with a result.
    pub fn run(&mut self) -> Result<SearchOutcome> {
        let mut terminal = ratatui::init();
        terminal.clear()?;

        self.hydrate_initial_results();

        let (event_tx, event_rx) = mpsc::channel();
        let event_loop_running = Arc::new(AtomicBool::new(true));
        let event_loop_flag = Arc::clone(&event_loop_running);

        let event_thread = thread::spawn(move || -> Result<()> {
            while event_loop_flag.load(Ordering::Relaxed) {
                if event::poll(Duration::from_millis(50))? {
                    let event = event::read()?;
                    if event_tx.send(event).is_err() {
                        break;
                    }
                }
            }
            Ok(())
        });

        let mut pending_events = VecDeque::new();

        let result: Result<SearchOutcome> = 'event_loop: loop {
            self.pump_index_updates();
            self.pump_search_results();
            self.throbber_state.calc_next();

            loop {
                match event_rx.try_recv() {
                    Ok(Event::Resize(_, _)) => {}
                    Ok(event) => pending_events.push_back(event),
                    Err(mpsc::TryRecvError::Empty) => break,
                    Err(mpsc::TryRecvError::Disconnected) => {
                        break 'event_loop Err(anyhow!("input event channel disconnected"));
                    }
                }
            }

            terminal.draw(|frame| self.draw(frame))?;

            let mut maybe_outcome = None;
            while let Some(event) = pending_events.pop_front() {
                match event {
                    Event::Key(key) if key.kind == KeyEventKind::Press => {
                        if let Some(outcome) = self.handle_key(key)? {
                            maybe_outcome = Some(outcome);
                            break;
                        }
                    }
                    Event::Resize(_, _) => {}
                    _ => {}
                }
            }

            if let Some(outcome) = maybe_outcome {
                break Ok(outcome);
            }

            thread::sleep(Duration::from_millis(16));
        };

        ratatui::restore();

        event_loop_running.store(false, Ordering::Relaxed);
        match event_thread.join() {
            Ok(join_result) => join_result?,
            Err(err) => std::panic::resume_unwind(err),
        }

        result
    }

    fn hydrate_initial_results(&mut self) {
        if !self.search.has_issued_query() {
            self.mark_query_dirty();
            self.request_search();
        }

        if let Some(timeout) = self.initial_results_timeout {
            let deadline = Instant::now() + timeout;
            self.initial_results_deadline = Some(deadline);
            while Instant::now() < deadline {
                self.pump_index_updates();
                self.pump_search_results();
                if !self.search.is_in_flight() {
                    break;
                }
                thread::sleep(Duration::from_millis(10));
            }
            self.pump_search_results();
        } else {
            self.initial_results_deadline = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::builtin::files;
    use crate::systems::filesystem::{IndexUpdate, ProgressSnapshot};
    use frz_plugin_api::search::Fs;
    use frz_plugin_api::{AttributeRow, FileRow, SearchData};
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
}
