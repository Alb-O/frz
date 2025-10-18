use crate::extensions::api::descriptors::ExtensionDescriptor;

/// Identifies a single tab contributed to the search UI.
#[derive(Clone, Copy)]
pub struct SearchMode {
    descriptor: &'static ExtensionDescriptor,
}

impl std::fmt::Debug for SearchMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("SearchMode").field(&self.id()).finish()
    }
}

impl PartialEq for SearchMode {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.descriptor, other.descriptor)
    }
}

impl Eq for SearchMode {}

impl std::hash::Hash for SearchMode {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::hash::Hash::hash(&(self.descriptor as *const ExtensionDescriptor), state);
    }
}

impl SearchMode {
    /// Create a search mode identifier backed by a extension descriptor.
    #[must_use]
    pub const fn from_descriptor(descriptor: &'static ExtensionDescriptor) -> Self {
        Self { descriptor }
    }

    /// Return the identifier for this mode.
    #[must_use]
    pub const fn id(self) -> &'static str {
        self.descriptor.id
    }

    /// Access the extension descriptor backing this mode.
    #[must_use]
    pub const fn descriptor(self) -> &'static ExtensionDescriptor {
        self.descriptor
    }
}
