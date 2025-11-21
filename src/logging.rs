/// Lightweight placeholders for logging hooks. The original project piped logs
/// into an in-UI console; the simplified version keeps the call sites intact
/// but drops any runtime logging setup.
pub fn initialize() {}

/// No-op pump for log events.
pub fn pump() {}
