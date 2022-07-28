use crate::{
    bundle::{Bundle, BundleKind},
    registry::Registry,
    EntityId,
};
use std::any::{Any, TypeId};
use std::collections::HashMap;

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
    component_vecs_by_type_id: HashMap<TypeId, Box<dyn Any>>,
}
impl Archetype {
    fn new() -> Self {
        Archetype {
            entity_ids: Vec::new(),
            component_vecs_by_type_id: HashMap::new(),
        }
    }
    pub fn get_entity_count(&self) -> usize {
        self.entity_ids.len()
    }
    pub(crate) fn push_component<T: 'static>(&mut self, component: T) {
        let type_id = TypeId::of::<T>();
        if !self.component_vecs_by_type_id.contains_key(&type_id) {
            self.component_vecs_by_type_id
                .insert(type_id.clone(), Box::new(Vec::<T>::new()));
        }
        let component_vec = self
            .component_vecs_by_type_id
            .get_mut(&type_id)
            .unwrap()
            .downcast_mut::<Vec<T>>()
            .expect("Could not downcast component vec to Vec<T>");
        component_vec.push(component);
    }
    pub(crate) fn push_entity_id(&mut self, entity_id: EntityId) {
        self.entity_ids.push(entity_id);
    }
    pub(crate) fn get_entity_id_at_index_unchecked(&self, index: usize) -> EntityId {
        self.entity_ids[index]
    }
    pub(crate) fn get_component_at_index_unchecked<T: 'static>(&self, index: usize) -> &T {
        let type_id = TypeId::of::<T>();
        let component_vec = self
            .component_vecs_by_type_id
            .get(&type_id)
            .unwrap()
            .downcast_ref::<Vec<T>>()
            .expect("Could not downcast component vec to Vec<T>");
        &component_vec[index]
    }
}
