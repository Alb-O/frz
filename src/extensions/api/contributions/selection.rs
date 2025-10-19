use std::collections::HashMap;
use std::sync::Arc;

use crate::extensions::api::error::ExtensionCatalogError;
use crate::extensions::api::search::{AttributeRow, FileRow, SearchData, SearchMode};

use super::{ContributionInstallContext, ContributionSpecImpl, ScopedContribution};

/// Resource reference resolved from the current preview selection.
pub enum PreviewResource<'a> {
    /// Selected file row.
    File(&'a FileRow),
    /// Selected attribute row.
    Attribute(&'a AttributeRow),
}

/// Strategy capable of converting the UI selection into a preview resource.
pub trait SelectionResolver: Send + Sync {
    /// Resolve the current selection into a [`PreviewResource`].
    fn resolve<'a>(
        &self,
        data: &'a SearchData,
        filtered: &'a [usize],
        selected: Option<usize>,
    ) -> Option<PreviewResource<'a>>;
}

/// Storage for selection resolvers registered by extensions.
#[derive(Clone, Default)]
pub struct SelectionResolverStore {
    resolvers: HashMap<SearchMode, Arc<dyn SelectionResolver>>,
}

impl SelectionResolverStore {
    /// Register a selection resolver for the provided mode.
    pub fn register(
        &mut self,
        mode: SearchMode,
        resolver: Arc<dyn SelectionResolver>,
    ) -> Result<(), ExtensionCatalogError> {
        if self.resolvers.contains_key(&mode) {
            return Err(ExtensionCatalogError::contribution_conflict(
                "selection resolver",
                mode,
            ));
        }
        self.resolvers.insert(mode, resolver);
        Ok(())
    }

    /// Retrieve the resolver registered for the provided mode, if any.
    #[must_use]
    pub fn get(&self, mode: SearchMode) -> Option<Arc<dyn SelectionResolver>> {
        self.resolvers.get(&mode).cloned()
    }

    /// Remove the resolver registered for the provided mode.
    pub fn remove(&mut self, mode: SearchMode) {
        self.resolvers.remove(&mode);
    }
}

impl ScopedContribution for SelectionResolverStore {
    type Output = Arc<dyn SelectionResolver>;

    fn resolve(&self, mode: SearchMode) -> Option<Self::Output> {
        self.get(mode)
    }
}

/// Contribution describing a selection resolver implementation.
#[derive(Clone)]
pub struct SelectionResolverContribution {
    descriptor: &'static crate::extensions::api::descriptors::ExtensionDescriptor,
    resolver: Arc<dyn SelectionResolver>,
}

impl SelectionResolverContribution {
    pub fn new<R>(
        descriptor: &'static crate::extensions::api::descriptors::ExtensionDescriptor,
        resolver: R,
    ) -> Self
    where
        R: SelectionResolver + 'static,
    {
        let resolver: Arc<dyn SelectionResolver> = Arc::new(resolver);
        Self {
            descriptor,
            resolver,
        }
    }
}

impl ContributionSpecImpl for SelectionResolverContribution {
    fn install(
        &self,
        context: &mut ContributionInstallContext<'_>,
    ) -> Result<(), ExtensionCatalogError> {
        let mode = SearchMode::from_descriptor(self.descriptor);
        let store = context.storage_mut::<SelectionResolverStore>();
        store.register(mode, Arc::clone(&self.resolver))?;
        context.register_cleanup::<SelectionResolverStore, _>(SelectionResolverStore::remove);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::extensions::builtin::files;

    struct FileResolver;

    impl SelectionResolver for FileResolver {
        fn resolve<'a>(
            &self,
            data: &'a SearchData,
            filtered: &'a [usize],
            selected: Option<usize>,
        ) -> Option<PreviewResource<'a>> {
            let index = selected?;
            let row_index = filtered.get(index).copied()?;
            data.files.get(row_index).map(PreviewResource::File)
        }
    }

    #[test]
    fn store_registers_and_resolves() {
        let mut store = SelectionResolverStore::default();
        let mode = files::mode();
        store
            .register(mode, Arc::new(FileResolver))
            .expect("register resolver");
        let resolver = store.get(mode).expect("resolver present");
        let data = SearchData::new().with_files(vec![FileRow::new("lib.rs", Vec::<String>::new())]);
        let filtered = vec![0];
        match resolver.resolve(&data, &filtered, Some(0)) {
            Some(PreviewResource::File(row)) => assert_eq!(row.path, "lib.rs"),
            _ => panic!("expected file resource"),
        }

        store.remove(mode);
        assert!(store.get(mode).is_none());
    }
}
