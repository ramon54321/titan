use crate::{
    bundle::{Bundle, BundleKind},
    registry::Registry,
    EntityId,
};
use std::sync::RwLock;
use std::{
    any::{Any, TypeId},
    sync::RwLockReadGuard,
};
use std::{collections::HashMap, sync::RwLockWriteGuard};

pub struct Storage {
    current_entity_id: EntityId,
    pub(crate) archetype_by_bundle_kind: HashMap<BundleKind, Archetype>,
}
impl Storage {
    pub(crate) fn new() -> Self {
        Self {
            current_entity_id: 0,
            archetype_by_bundle_kind: HashMap::new(),
        }
    }
    pub(crate) fn spawn<T: Bundle + 'static>(&mut self, registry: &Registry, bundle: T) {
        self.spawn_with_entity_id(registry, self.current_entity_id, bundle);

        // Increment entity_id for next spawn
        self.current_entity_id = self.current_entity_id + 1;
    }
    pub(crate) fn spawn_with_entity_id<T: Bundle + 'static>(
        &mut self,
        registry: &Registry,
        entity_id: EntityId,
        bundle: T,
    ) {
        let bundle_id = T::get_bundle_id();
        let bundle_kind = registry.bundle_id_to_bundle_kind(bundle_id);

        // Ensure archetype exists
        let archetype = {
            let provisional_archetype = self.archetype_by_bundle_kind.get_mut(&bundle_kind);
            if provisional_archetype.is_some() {
                provisional_archetype.unwrap()
            } else {
                self.archetype_by_bundle_kind
                    .insert(bundle_kind.clone(), Archetype::new());
                self.archetype_by_bundle_kind
                    .get_mut(&bundle_kind.clone())
                    .unwrap()
            }
        };

        // Push bundle into archetype
        bundle.push_into_archetype(entity_id, archetype);

        // Ensure entity_id is safe
        if entity_id >= self.current_entity_id {
            self.current_entity_id = entity_id + 1;
        }
    }
}
pub struct Archetype {
    entity_ids: Vec<EntityId>,
    component_vec_locks_by_type_id: HashMap<TypeId, Box<dyn Any>>,
}
impl Archetype {
    fn new() -> Self {
        Archetype {
            entity_ids: Vec::new(),
            component_vec_locks_by_type_id: HashMap::new(),
        }
    }
    pub fn get_entity_count(&self) -> usize {
        self.entity_ids.len()
    }
    pub(crate) fn push_component<T: 'static>(&mut self, component: T) {
        let type_id = TypeId::of::<T>();
        if !self.component_vec_locks_by_type_id.contains_key(&type_id) {
            self.component_vec_locks_by_type_id
                .insert(type_id.clone(), Box::new(RwLock::new(Vec::<T>::new())));
        }
        let component_vec = self
            .component_vec_locks_by_type_id
            .get_mut(&type_id)
            .unwrap()
            .downcast_mut::<RwLock<Vec<T>>>()
            .expect("Could not downcast component vec to Vec<T>");
        component_vec
            .get_mut()
            .expect("Could not get write access to component vec in order to push new component")
            .push(component);
    }
    pub(crate) fn push_entity_id(&mut self, entity_id: EntityId) {
        self.entity_ids.push(entity_id);
    }
    pub(crate) fn get_entity_id_at_index_unchecked(&self, index: usize) -> EntityId {
        self.entity_ids[index]
    }
    pub(crate) fn get_component_vec_lock<T: 'static>(&self) -> RwLockReadGuard<Vec<T>> {
        let type_id = TypeId::of::<T>();
        self.component_vec_locks_by_type_id
            .get(&type_id)
            .expect("Could not find component vec for given type_id in archetype")
            .downcast_ref::<RwLock<Vec<T>>>()
            .expect("Could not downcast to lock of component vec")
            .try_read()
            .expect("Could not get from component vec lock")
    }
    pub(crate) fn get_component_vec_lock_mut<T: 'static>(&self) -> RwLockWriteGuard<Vec<T>> {
        let type_id = TypeId::of::<T>();
        self.component_vec_locks_by_type_id
            .get(&type_id)
            .expect("Could not find component vec for given type_id in archetype")
            .downcast_ref::<RwLock<Vec<T>>>()
            .expect("Could not downcast to lock of component vec")
            .try_write()
            .expect("Could not get from component vec lock")
    }
    pub(crate) fn get_component_at_index_unchecked_mut<T: 'static>(
        &mut self,
        index: usize,
    ) -> &mut T {
        let type_id = TypeId::of::<T>();
        let raw_component_vec = self
            .component_vec_locks_by_type_id
            .get_mut(&type_id)
            .expect("Could not find component vec for given type_id in archetype")
            .downcast_mut::<RwLock<Vec<T>>>()
            .expect("Could not downcast_mut to lock of component vec")
            .get_mut()
            .expect("Could not get mut from component vec lock");
        &mut raw_component_vec[index]
    }
}
