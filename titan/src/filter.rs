use std::any::TypeId;

use crate::{
    registry::Registry,
    storage::{Archetype, Storage},
};

trait ArchetypeFetch<'a> {
    type BundleItem;
    fn fetch(storage: &'a mut Storage) -> Vec<(usize, Self::BundleItem)>;
}

impl<'a, A: ComponentFetch<'a> + 'static, B: ComponentFetch<'a> + 'static> ArchetypeFetch<'a>
    for (A, B)
{
    type BundleItem = (A::ComponentItem, B::ComponentItem);
    fn fetch(storage: &'a mut Storage) -> Vec<(usize, Self::BundleItem)> {
        // Get archetypes in some way
        //let component_type_id = TypeId::of::<T>();
        //let component_kind = registry.type_id_to_kind(component_type_id);

        let archetypes = [storage
            .archetype_by_bundle_kind
            .values_mut()
            .last()
            .unwrap()];

        let mut wrapped_bundles = Vec::new();
        for archetype in archetypes {
            let component_count = archetype.get_entity_count();

            for i in 0..component_count {
                //let entity_id = archetype.get_entity_id_at_index_unchecked(i);

                // For each type
                //let component_a = archetype.get_component_at_index_unchecked::<A>(i);
                //let component_b = archetype.get_component_at_index_unchecked::<B>(i);

                //let component_a = <A as ComponentFetch>::fetch(archetype, i);
                //let component_b = <B as ComponentFetch>::fetch(archetype, i);

                let bundle = (
                    <A as ComponentFetch>::fetch(archetype, i),
                    <B as ComponentFetch>::fetch(archetype, i),
                );

                //let bundle = (component_a, component_b);
                //let wrapped_bundle = (entity_id, bundle);
                let wrapped_bundle = (23, bundle);
                wrapped_bundles.push(wrapped_bundle);
            }
        }
        wrapped_bundles
    }
}

trait ComponentFetch<'a> {
    type ComponentItem;
    fn fetch(archetype: &'a mut Archetype, index: usize) -> Self::ComponentItem;
}

impl<'a, T: 'static> ComponentFetch<'a> for &'a T {
    type ComponentItem = &'a T;
    fn fetch(archetype: &'a mut Archetype, index: usize) -> Self::ComponentItem {
        archetype.get_component_at_index_unchecked::<T>(index)
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
