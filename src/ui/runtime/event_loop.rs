use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc};
use std::thread;
use std::time::Duration;

use anyhow::{Result, anyhow};
use ratatui::crossterm::event::{self, Event, KeyEventKind};

use crate::extensions::api::{SearchData, SearchOutcome};
use crate::ui::App;

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
}
