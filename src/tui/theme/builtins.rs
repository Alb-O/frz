use super::types::ThemeDefinition;

pub mod light;
pub mod slate;
pub mod solarized;

pub use light::LIGHT;
pub use slate::SLATE;
pub use solarized::SOLARIZED;

pub(super) const BUILT_IN_DEFINITIONS: &[ThemeDefinition] =
    &[light::DEFINITION, slate::DEFINITION, solarized::DEFINITION];
