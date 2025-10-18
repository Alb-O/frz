mod preview_split;
mod search_tabs;

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;

use crate::extensions::api::descriptors::ExtensionDescriptor;
use crate::extensions::api::error::ExtensionCatalogError;
use crate::extensions::api::registry::ExtensionModule;
use crate::extensions::api::registry::RegisteredModule;
use crate::extensions::api::search::SearchMode;

pub use preview_split::{PreviewSplit, PreviewSplitContext, PreviewSplitStore};
pub use search_tabs::SearchTabStore;

type Cloner = fn(&dyn Any) -> Box<dyn Any + Send + Sync>;
type CleanupHandler = Arc<dyn Fn(&mut dyn Any, SearchMode) + Send + Sync>;

/// Trait implemented by concrete contribution specifications.
trait ContributionSpec: Send + Sync {
    /// Install the contribution into the provided context.
    fn install(
        &self,
        context: &mut ContributionInstallContext<'_>,
    ) -> Result<(), ExtensionCatalogError>;

    /// Clone the contribution specification.
    fn clone_spec(&self) -> Arc<dyn ContributionSpec>;
}

impl<T> ContributionSpec for T
where
    T: ContributionSpecImpl + Clone + 'static,
{
    fn install(
        &self,
        context: &mut ContributionInstallContext<'_>,
    ) -> Result<(), ExtensionCatalogError> {
        <T as ContributionSpecImpl>::install(self, context)
    }

    fn clone_spec(&self) -> Arc<dyn ContributionSpec> {
        Arc::new(self.clone())
    }
}

/// Internal trait implemented for each contribution type.
trait ContributionSpecImpl: Send + Sync {
    fn install(
        &self,
        context: &mut ContributionInstallContext<'_>,
    ) -> Result<(), ExtensionCatalogError>;
}

/// A clonable contribution provided by a package.
#[derive(Clone)]
pub struct Contribution(Arc<dyn ContributionSpec>);

impl Contribution {
    fn new(spec: Arc<dyn ContributionSpec>) -> Self {
        Self(spec)
    }

    /// Create a search tab contribution.
    pub fn search_tab<P>(descriptor: &'static ExtensionDescriptor, module: P) -> Self
    where
        P: ExtensionModule + 'static,
    {
        Self::from_spec(search_tabs::SearchTabContribution::new(descriptor, module))
    }

    /// Create a preview split contribution.
    pub fn preview_split<P>(descriptor: &'static ExtensionDescriptor, preview: P) -> Self
    where
        P: preview_split::PreviewSplit + 'static,
    {
        Self::from_spec(preview_split::PreviewSplitContribution::new(
            descriptor, preview,
        ))
    }

    fn from_spec<T>(spec: T) -> Self
    where
        T: ContributionSpec + 'static,
    {
        Self::new(spec.clone_spec())
    }

    pub(crate) fn install(
        &self,
        context: &mut ContributionInstallContext<'_>,
    ) -> Result<(), ExtensionCatalogError> {
        self.0.install(context)
    }
}

/// Collection of contributions provided by an extension package.
pub trait ExtensionPackage: Send + Sync {
    type Contributions<'a>: IntoIterator<Item = Contribution>
    where
        Self: 'a;

    fn contributions(&self) -> Self::Contributions<'_>;
}

/// Mutable view into the catalog used while installing contributions.
pub struct ContributionInstallContext<'a> {
    search_tabs: &'a mut SearchTabStore,
    registry: &'a mut ContributionRegistry,
}

impl<'a> ContributionInstallContext<'a> {
    pub(crate) fn new(
        search_tabs: &'a mut SearchTabStore,
        registry: &'a mut ContributionRegistry,
    ) -> Self {
        Self {
            search_tabs,
            registry,
        }
    }

    /// Ensure the provided descriptor can be registered.
    pub fn ensure_mode_available(
        &self,
        descriptor: &'static ExtensionDescriptor,
    ) -> Result<(), ExtensionCatalogError> {
        self.search_tabs.ensure_available(descriptor)
    }

    /// Register a search tab implementation.
    pub fn register_search_tab(
        &mut self,
        module: RegisteredModule,
    ) -> Result<(), ExtensionCatalogError> {
        self.search_tabs.ensure_available(module.descriptor())?;
        self.search_tabs.insert(module);
        Ok(())
    }

    /// Access contribution-specific storage, creating it on-demand.
    pub fn storage_mut<T>(&mut self) -> &mut T
    where
        T: Default + Clone + Send + Sync + 'static,
    {
        self.registry.storage_mut::<T>()
    }

    /// Access contribution-specific storage if present.
    pub fn storage<T>(&self) -> Option<&T>
    where
        T: Clone + Send + Sync + 'static,
    {
        self.registry.storage::<T>()
    }

    /// Register cleanup logic invoked when a mode is removed from the catalog.
    pub fn register_cleanup<T, F>(&mut self, cleanup: F)
    where
        T: Default + Clone + Send + Sync + 'static,
        F: Fn(&mut T, SearchMode) + Send + Sync + 'static,
    {
        self.registry.register_cleanup::<T, F>(cleanup);
    }
}

fn clone_value<T>(data: &dyn Any) -> Box<dyn Any + Send + Sync>
where
    T: Clone + Send + Sync + 'static,
{
    let value = data
        .downcast_ref::<T>()
        .expect("contribution store clone type mismatch");
    Box::new(value.clone())
}

#[derive(Default)]
pub struct ContributionRegistry {
    stores: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
    cloners: HashMap<TypeId, Cloner>,
    cleanup: HashMap<TypeId, CleanupHandler>,
}

impl ContributionRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn storage_mut<T>(&mut self) -> &mut T
    where
        T: Default + Clone + Send + Sync + 'static,
    {
        let type_id = TypeId::of::<T>();
        if let std::collections::hash_map::Entry::Vacant(e) = self.stores.entry(type_id) {
            e.insert(Box::new(T::default()));
            self.cloners.insert(type_id, clone_value::<T>);
        }
        self.stores
            .get_mut(&type_id)
            .and_then(|store| store.as_mut().downcast_mut::<T>())
            .expect("contribution store type mismatch")
    }

    pub fn storage<T>(&self) -> Option<&T>
    where
        T: Clone + Send + Sync + 'static,
    {
        let type_id = TypeId::of::<T>();
        if let Some(store) = self.stores.get(&type_id) {
            store.as_ref().downcast_ref::<T>()
        } else {
            None
        }
    }

    pub fn register_cleanup<T, F>(&mut self, cleanup: F)
    where
        T: Default + Clone + Send + Sync + 'static,
        F: Fn(&mut T, SearchMode) + Send + Sync + 'static,
    {
        let type_id = TypeId::of::<T>();
        if let std::collections::hash_map::Entry::Vacant(e) = self.stores.entry(type_id) {
            e.insert(Box::new(T::default()));
            self.cloners.insert(type_id, clone_value::<T>);
        }
        self.cleanup.entry(type_id).or_insert_with(|| {
            Arc::new(move |data: &mut dyn Any, mode: SearchMode| {
                let store = data.downcast_mut::<T>().expect("cleanup type mismatch");
                cleanup(store, mode);
            })
        });
    }

    pub fn remove_mode(&mut self, mode: SearchMode) {
        for (type_id, handler) in self.cleanup.iter() {
            if let Some(store) = self.stores.get_mut(type_id) {
                handler(store.as_mut(), mode);
            }
        }
    }
}

impl Clone for ContributionRegistry {
    fn clone(&self) -> Self {
        let mut stores = HashMap::new();
        for (type_id, store) in &self.stores {
            let cloner = self
                .cloners
                .get(type_id)
                .expect("missing cloner for contribution store");
            stores.insert(*type_id, cloner(store.as_ref()));
        }
        Self {
            stores,
            cloners: self.cloners.clone(),
            cleanup: self.cleanup.clone(),
        }
    }
}
