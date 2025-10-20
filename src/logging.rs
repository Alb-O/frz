use std::env;
use std::sync::OnceLock;

use log::LevelFilter;

static LOGGER: OnceLock<()> = OnceLock::new();

/// Initialise the global logger, enabling runtime log capture for the UI.
///
/// Subsequent calls are no-ops, allowing callers to invoke this during
/// different entry points without worrying about double-initialisation.
pub fn initialize() {
    LOGGER.get_or_init(|| {
        if let Err(error) = tui_logger::init_logger(LevelFilter::Trace) {
            eprintln!("failed to initialise tui-logger: {error}");
            return;
        }

        tui_logger::set_default_level(LevelFilter::Info);

        if !env::var("RUST_LOG")
            .map(|value| value.contains("frz::preview::image"))
            .unwrap_or(false)
        {
            tui_logger::set_level_for_target("frz::preview::image", LevelFilter::Debug);
        }

        // Allow users to opt into traditional RUST_LOG syntax while still using
        // the in-application viewer. Errors are ignored because the filter is
        // optional.
        let _ = tui_logger::set_env_filter_from_env(None);
    });
}

/// Move buffered log events into the main display queue when logging has been
/// initialised.
pub fn pump() {
    if LOGGER.get().is_some() {
        tui_logger::move_events();
    }
}
