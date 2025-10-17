mod plugin;
mod registered_plugin;
mod store;

pub use plugin::SearchPlugin;
pub use registered_plugin::RegisteredPlugin;
pub use store::SearchPluginRegistry;

#[cfg(test)]
mod tests;
