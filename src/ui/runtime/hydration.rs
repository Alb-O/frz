use std::thread;
use std::time::{Duration, Instant};

use crate::ui::App;

impl<'a> App<'a> {
    pub(super) fn hydrate_initial_results(&mut self) {
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
