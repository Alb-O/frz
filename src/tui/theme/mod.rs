mod builtins;
mod registry;
mod types;

pub use builtins::{LIGHT, SLATE, SOLARIZED, light, slate, solarized};
pub use registry::{by_name, descriptors, names, register_additional, register_definitions};
pub use types::{
    AliasConflict, Theme, ThemeDefinition, ThemeDescriptor, ThemeRegistration,
    ThemeRegistrationReport,
};

impl Default for Theme {
    fn default() -> Self {
        SLATE
    }
}
