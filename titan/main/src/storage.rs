use crate::{
    bundle::{Bundle, BundleKind},
    query::Query,
    ComponentKind, ComponentMeta, EntityId,
};
use std::sync::RwLock;
use std::{any::Any, sync::RwLockReadGuard};
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
    pub(crate) fn spawn<T: Bundle + 'static>(&mut self, bundle: T) {
        self.spawn_with_entity_id(self.current_entity_id, bundle);

        // TODO: Ensure entity_id is incremented correctly
        // Increment entity_id for next spawn
        self.current_entity_id = self.current_entity_id + 1;
    }
    pub(crate) fn spawn_with_entity_id<T: Bundle + 'static>(
        &mut self,
        entity_id: EntityId,
        bundle: T,
    ) {
        let bundle_kind = T::get_bundle_kind();

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
    pub(crate) fn query<'fetch, T: Query<'fetch>>(&'fetch self) -> T::ResultType {
        <T>::query(self)
    }
}
pub struct Archetype {
    entity_ids: Vec<EntityId>,
    component_vec_locks_by_component_kind: HashMap<ComponentKind, Box<dyn Any>>,
}
impl Archetype {
    fn new() -> Self {
        Archetype {
            entity_ids: Vec::new(),
            component_vec_locks_by_component_kind: HashMap::new(),
        }
    }
    pub fn get_entity_count(&self) -> usize {
        self.entity_ids.len()
    }
    pub(crate) fn has_component<T: 'static + ComponentMeta>(&self) -> bool {
        let component_kind = T::get_component_kind();
        self.component_vec_locks_by_component_kind
            .contains_key(&component_kind)
    }
    pub(crate) fn push_component<T: 'static + ComponentMeta>(&mut self, component: T) {
        let component_kind = T::get_component_kind();
        if !self
            .component_vec_locks_by_component_kind
            .contains_key(&component_kind)
        {
            self.component_vec_locks_by_component_kind.insert(
                component_kind.clone(),
                Box::new(RwLock::new(Vec::<T>::new())),
            );
        }
        let component_vec = self
            .component_vec_locks_by_component_kind
            .get_mut(&component_kind)
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
    pub(crate) fn get_component_vec_lock<T: 'static + ComponentMeta>(
        &self,
    ) -> RwLockReadGuard<Vec<T>> {
        let component_kind = T::get_component_kind();
        self.component_vec_locks_by_component_kind
            .get(&component_kind)
            .expect("Could not find component vec for given component_kind in archetype")
            .downcast_ref::<RwLock<Vec<T>>>()
            .expect("Could not downcast to lock of component vec")
            .try_read()
            .expect("Could not get from component vec lock")
    }
    pub(crate) fn get_component_vec_lock_mut<T: 'static + ComponentMeta>(
        &self,
    ) -> RwLockWriteGuard<Vec<T>> {
        let component_kind = T::get_component_kind();
        self.component_vec_locks_by_component_kind
            .get(&component_kind)
            .expect("Could not find component vec for given component_kind in archetype")
            .downcast_ref::<RwLock<Vec<T>>>()
            .expect("Could not downcast to lock of component vec")
            .try_write()
            .expect("Could not get from component vec lock")
    }
}
