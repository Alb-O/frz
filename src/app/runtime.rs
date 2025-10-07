use std::time::Duration;

use anyhow::Result;
use ratatui::crossterm::event::{self, Event, KeyEventKind};

use crate::types::{SearchData, SearchOutcome};

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

        let result = loop {
            self.pump_index_updates();
            self.pump_search_results();
            self.throbber_state.calc_next();
            terminal.draw(|frame| self.draw(frame))?;

            if event::poll(Duration::from_millis(50))? {
                match event::read()? {
                    Event::Key(key) if key.kind == KeyEventKind::Press => {
                        if let Some(outcome) = self.handle_key(key)? {
                            break outcome;
                        }
                    }
                    Event::Resize(_, _) => {}
                    _ => {}
                }
            }
        };

        ratatui::restore();
        Ok(result)
    }
}
