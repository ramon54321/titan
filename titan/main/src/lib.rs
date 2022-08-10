#![feature(generic_associated_types)]
#![feature(type_alias_impl_trait)]

pub use bundle::Bundle;
use query::Query;
use registry::{RegisterArchetype, RegisterComponent, Registry};
use serialization::Serializable;
use storage::Storage;
pub use titan_macros::component;

mod bundle;
mod query;
mod registry;
mod serialization;
mod storage;

pub use query::ResultIter as ResultIteration;

///
/// Type for all enitity identifiers.
///
type EntityId = usize;

///
/// The main identifier for any given component. No two components can have equal `ComponentKind`s.
///
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct ComponentKind(pub String);

///
/// MetaData methods for components. This trait is implemented by the `component` attribute macro.
/// These methods are used by titan internals.
///
pub trait ComponentMeta {
    fn get_component_kind() -> ComponentKind;
}

pub struct ECS {
    pub registry: Registry,
    storage: Storage,
}
impl ECS {
    ///
    /// Register new component.
    ///
    pub fn register_component<T: RegisterComponent>(&mut self) {
        self.registry.register_component::<T>();
    }
    ///
    /// Register new archetype.
    /// Order of component types do not matter.
    ///
    /// Ensure all components types in bundle have been registered before calling this method.
    ///
    pub fn register_archetype<T: RegisterArchetype>(&mut self) {
        self.registry.register_archetype::<T>();
    }
    ///
    /// Spawn bundle of components into new entity.
    ///
    pub fn spawn_bundle<T: Bundle + 'static>(&mut self, bundle: T) {
        self.storage.spawn(bundle)
    }
    ///
    /// Query the storage for all components in archetypes which AT LEAST match the given query
    /// type.
    ///
    pub fn query<'fetch, T: Query<'fetch>>(&'fetch self) -> T::ResultType {
        self.storage.query::<T>()
    }
    ///
    /// Serialize entities to JSON.
    ///
    pub fn serialize(&self) -> String {
        self.storage.serialize(&self.registry)
    }
    ///
    /// Replaces storage with entities from JSON.
    ///
    pub fn deserialize(&mut self, serial: &str) {
        self.storage = Storage::deserialize(serial, &self.registry);
    }
}
impl Default for ECS {
    fn default() -> Self {
        Self {
            registry: Registry::new(),
            storage: Storage::new(),
        }
    }
}
