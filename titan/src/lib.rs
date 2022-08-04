#![feature(generic_associated_types)]
#![feature(type_alias_impl_trait)]

use bundle::Bundle;
use query::Query;
use registry::{RegisterArchetype, RegisterComponent, Registry};
use serialization::Serializable;
use storage::Storage;

mod bundle;
mod query;
mod registry;
mod serialization;
mod storage;

pub use query::ResultIter as ResultIteration;

type EntityId = usize;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub(crate) struct ComponentKind(String);

pub struct ECS {
    registry: Registry,
    storage: Storage,
}
impl ECS {
    ///
    /// Register new component.
    ///
    pub fn register_component<T: RegisterComponent>(&mut self, name: &str) {
        self.registry.register_component::<T>(name);
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
        self.storage.spawn(&self.registry, bundle)
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

#[test]
fn master() {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    struct Age(u8);
    #[derive(Debug, Serialize, Deserialize)]
    struct Name(String);
    #[derive(Debug, Serialize, Deserialize)]
    struct Person {
        height: u16,
        weight: u16,
    }

    let mut registry = Registry::new();
    registry.register_component::<Age>(&"Age");
    registry.register_component::<Name>(&"Name");
    registry.register_component::<Person>(&"Person");
    registry.register_archetype::<(Name, Age)>();

    let mut storage = Storage::new();
    storage.spawn(&registry, (Age(23), Name("Jeff".to_string())));
    storage.spawn(&registry, (Age(19), Name("Julia".to_string())));
    storage.spawn(&registry, (Name("Bob".to_string()), Age(29)));

    for (age, name) in storage.query::<(&mut Age, &Name)>().iter() {
        if name.0 == "Julia" {
            age.0 = 34;
        }
    }

    let storage_serial = storage.serialize(&registry);
    println!("{}", storage_serial);

    let new_storage = Storage::deserialize(&storage_serial, &registry);
    let new_storage_serial = new_storage.serialize(&registry);
    println!("{}", new_storage_serial);

    for (age, name) in new_storage.query::<(&Age, &Name)>().iter() {
        println!("{} is {} years old.", name.0, age.0);
    }
}
