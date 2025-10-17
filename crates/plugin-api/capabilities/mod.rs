mod preview_split;
mod search_tabs;

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;

use crate::descriptors::SearchPluginDescriptor;
use crate::error::PluginRegistryError;
use crate::registry::RegisteredPlugin;
use crate::search::SearchMode;

pub use preview_split::{PreviewSplit, PreviewSplitContext, PreviewSplitStore};
pub use search_tabs::SearchTabStore;

type Cloner = fn(&dyn Any) -> Box<dyn Any + Send + Sync>;
type CleanupHandler = Arc<dyn Fn(&mut dyn Any, SearchMode) + Send + Sync>;

/// Trait implemented by concrete capability specifications.
trait CapabilitySpec: Send + Sync {
    /// Install the capability into the provided context.
    fn install(
        &self,
        context: &mut CapabilityInstallContext<'_>,
    ) -> Result<(), PluginRegistryError>;

    /// Clone the capability specification.
    fn clone_spec(&self) -> Arc<dyn CapabilitySpec>;
}

impl<T> CapabilitySpec for T
where
    T: CapabilitySpecImpl + Clone + 'static,
{
    fn install(
        &self,
        context: &mut CapabilityInstallContext<'_>,
    ) -> Result<(), PluginRegistryError> {
        <T as CapabilitySpecImpl>::install(self, context)
    }

    fn clone_spec(&self) -> Arc<dyn CapabilitySpec> {
        Arc::new(self.clone())
    }
}

/// Internal trait implemented for each capability type.
trait CapabilitySpecImpl: Send + Sync {
    fn install(
        &self,
        context: &mut CapabilityInstallContext<'_>,
    ) -> Result<(), PluginRegistryError>;
}

/// A clonable capability contributed by a bundle.
#[derive(Clone)]
pub struct Capability(Arc<dyn CapabilitySpec>);

impl Capability {
    fn new(spec: Arc<dyn CapabilitySpec>) -> Self {
        Self(spec)
    }

    /// Create a search tab capability.
    pub fn search_tab<P>(descriptor: &'static SearchPluginDescriptor, plugin: P) -> Self
    where
        P: crate::registry::SearchPlugin + 'static,
    {
        Self::from_spec(search_tabs::SearchTabCapability::new(descriptor, plugin))
    }

    /// Create a preview split capability.
    pub fn preview_split<P>(descriptor: &'static SearchPluginDescriptor, preview: P) -> Self
    where
        P: preview_split::PreviewSplit + 'static,
    {
        Self::from_spec(preview_split::PreviewSplitCapability::new(
            descriptor, preview,
        ))
    }

    fn from_spec<T>(spec: T) -> Self
    where
        T: CapabilitySpec + 'static,
    {
        Self::new(spec.clone_spec())
    }

    pub(crate) fn install(
        &self,
        context: &mut CapabilityInstallContext<'_>,
    ) -> Result<(), PluginRegistryError> {
        self.0.install(context)
    }
}

/// Collection of capability implementations contributed by a plugin bundle.
pub trait PluginBundle: Send + Sync {
    type Capabilities<'a>: IntoIterator<Item = Capability>
    where
        Self: 'a;

    fn capabilities(&self) -> Self::Capabilities<'_>;
}

/// Mutable view into the registry used while installing capabilities.
pub struct CapabilityInstallContext<'a> {
    search_tabs: &'a mut SearchTabStore,
    registry: &'a mut CapabilityRegistry,
}

impl<'a> CapabilityInstallContext<'a> {
    pub(crate) fn new(
        search_tabs: &'a mut SearchTabStore,
        registry: &'a mut CapabilityRegistry,
    ) -> Self {
        Self {
            search_tabs,
            registry,
        }
    }

    /// Ensure the provided descriptor can be registered.
    pub fn ensure_mode_available(
        &self,
        descriptor: &'static SearchPluginDescriptor,
    ) -> Result<(), PluginRegistryError> {
        self.search_tabs.ensure_available(descriptor)
    }

    /// Register a search tab implementation.
    pub fn register_search_tab(
        &mut self,
        plugin: RegisteredPlugin,
    ) -> Result<(), PluginRegistryError> {
        self.search_tabs.ensure_available(plugin.descriptor())?;
        self.search_tabs.insert(plugin);
        Ok(())
    }

    /// Access a capability-specific store, creating it on-demand.
    pub fn storage_mut<T>(&mut self) -> &mut T
    where
        T: Default + Clone + Send + Sync + 'static,
    {
        self.registry.storage_mut::<T>()
    }

    /// Access a capability-specific store if present.
    pub fn storage<T>(&self) -> Option<&T>
    where
        T: Clone + Send + Sync + 'static,
    {
        self.registry.storage::<T>()
    }

    /// Register cleanup logic invoked when a mode is removed from the registry.
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
        .expect("capability store clone type mismatch");
    Box::new(value.clone())
}

#[derive(Default)]
pub struct CapabilityRegistry {
    stores: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
    cloners: HashMap<TypeId, Cloner>,
    cleanup: HashMap<TypeId, CleanupHandler>,
}

impl CapabilityRegistry {
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
            .expect("capability store type mismatch")
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

impl Clone for CapabilityRegistry {
    fn clone(&self) -> Self {
        let mut stores = HashMap::new();
        for (type_id, store) in &self.stores {
            let cloner = self
                .cloners
                .get(type_id)
                .expect("missing cloner for capability store");
            stores.insert(*type_id, cloner(store.as_ref()));
        }
        Self {
            stores,
            cloners: self.cloners.clone(),
            cleanup: self.cleanup.clone(),
        }
    }
}
