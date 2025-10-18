mod plugin;
mod registered_plugin;
mod store;

pub use plugin::FrzPlugin;
pub use registered_plugin::RegisteredPlugin;
pub use store::FrzPluginRegistry;

#[cfg(test)]
mod tests;
