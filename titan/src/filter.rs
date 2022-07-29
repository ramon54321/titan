use std::any::TypeId;

use crate::{
    registry::Registry,
    storage::{Archetype, Storage},
};

pub trait ArchetypeFetch {
    type BundleItem<'a>;
    fn fetch<'a>(storage: &'a mut Storage) -> Vec<(usize, Self::BundleItem<'a>)>;
}

impl<A: ComponentFetch + 'static, B: ComponentFetch + 'static> ArchetypeFetch for (A, B) {
    type BundleItem<'a> = (A::ComponentItem<'a>, B::ComponentItem<'a>);
    fn fetch<'a>(storage: &'a mut Storage) -> Vec<(usize, Self::BundleItem<'a>)> {
        // Get archetypes in some way
        //let component_type_id = TypeId::of::<T>();
        //let component_kind = registry.type_id_to_kind(component_type_id);

        // Mock get archetypes
        let archetypes = [storage
            .archetype_by_bundle_kind
            .values_mut()
            .last()
            .unwrap()];

        let mut wrapped_bundles = Vec::new();
        for archetype in archetypes {
            let component_count = archetype.get_entity_count();

            for i in 0..component_count {
                let entity_id = archetype.get_entity_id_at_index_unchecked(i);
                let bundle = (
                    <A as ComponentFetch>::fetch(archetype, i),
                    <B as ComponentFetch>::fetch(archetype, i),
                );
                let wrapped_bundle = (entity_id, bundle);
                wrapped_bundles.push(wrapped_bundle);
            }
        }
        wrapped_bundles
    }
}

pub trait ComponentFetch {
    type ComponentItem<'a>;
    fn fetch<'b>(archetype: &'b Archetype, index: usize) -> Self::ComponentItem<'b>;
}

impl<T: 'static> ComponentFetch for &T {
    type ComponentItem<'a> = &'a T;
    fn fetch<'b>(archetype: &'b Archetype, index: usize) -> Self::ComponentItem<'b> {
        &archetype.get_component_at_index_unchecked::<T>(index)
    }
}

//trait Fetch<'a> {
//type Item;
//fn fetch(registry: &Registry, storage: &mut Storage) -> Vec<Self::Item>;
//}

//impl<'a, T: 'static> Fetch<'a> for &'a T {
//type Item = &'a T;
//fn fetch(registry: &Registry, storage: &mut Storage) -> Vec<Self::Item> {
//// Get archetypes in some way
//let component_type_id = TypeId::of::<T>();
//let component_kind = registry.type_id_to_kind(component_type_id);
//let archetypes = [storage
//.archetype_by_bundle_kind
//.values_mut()
//.last()
//.unwrap()];

//let mut components = Vec::new();
//for archetype in archetypes {
//let component_count = archetype.get_entity_count();
//for i in 0..component_count {
//let component = archetype.get_component_at_index_unchecked::<T>(i);
//components.push(component);
//}
//}

//components
//}
//}
