use crate::descriptors::SearchPluginDescriptor;

/// Identifies a single tab contributed to the search UI.
#[derive(Clone, Copy)]
pub struct SearchMode {
    descriptor: &'static SearchPluginDescriptor,
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
        std::hash::Hash::hash(&(self.descriptor as *const SearchPluginDescriptor), state);
    }
}

impl SearchMode {
    /// Create a search mode identifier backed by a plugin descriptor.
    #[must_use]
    pub const fn from_descriptor(descriptor: &'static SearchPluginDescriptor) -> Self {
        Self { descriptor }
    }

    /// Return the identifier for this mode.
    #[must_use]
    pub const fn id(self) -> &'static str {
        self.descriptor.id
    }

    /// Access the plugin descriptor backing this mode.
    #[must_use]
    pub const fn descriptor(self) -> &'static SearchPluginDescriptor {
        self.descriptor
    }
}
