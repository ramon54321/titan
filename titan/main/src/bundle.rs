use crate::{storage::Archetype, ComponentKind, ComponentMeta, EntityId};
use std::any::TypeId;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct BundleId(u64);
impl BundleId {
    fn from_component_type_ids(type_ids: &[TypeId]) -> Self {
        let mut hasher = DefaultHasher::new();
        type_ids.hash(&mut hasher);
        BundleId(hasher.finish())
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct BundleKind(pub(crate) String);
impl BundleKind {
    pub(crate) fn from_component_kinds(component_kinds: &[ComponentKind]) -> Self {
        let mut name = String::new();
        for component_kind in component_kinds {
            name.push_str(component_kind.0.as_str());
        }
        BundleKind(name)
    }
}

pub trait Bundle {
    fn push_into_archetype(self, entity_id: EntityId, archetype: &mut Archetype);
    fn get_bundle_id() -> BundleId;
}

macro_rules! bundle_impl {
    (
        $(($name:ident, $i:tt)),*
    ) => {
        impl<$($name),*> Bundle for ($($name),*,)
        where
            $($name: 'static + ComponentMeta),* {
            fn push_into_archetype(self, entity_id: EntityId, archetype: &mut Archetype) {
                archetype.push_entity_id(entity_id);
                $(archetype.push_component(self.$i));*
            }
            fn get_bundle_id() -> BundleId {
                let type_ids = [
                    $(TypeId::of::<$name>()),*
                ];
                // TODO: Wrapping is unnessesary as index is never used and is not needed
                let mut wrapped_type_ids = type_ids.into_iter().enumerate().collect::<Vec<(usize, TypeId)>>();
                wrapped_type_ids.sort_by(|a, b| a.1.cmp(&b.1));
                let ordered_type_ids = wrapped_type_ids.into_iter().map(|(_, type_id)| type_id).collect::<Vec<TypeId>>();
                BundleId::from_component_type_ids(&ordered_type_ids)
            }
        }
    };
}

bundle_impl! {(A, 0)}
bundle_impl! {(A, 0), (B, 1)}
bundle_impl! {(A, 0), (B, 1), (C, 2)}
bundle_impl! {(A, 0), (B, 1), (C, 2), (D, 3)}
bundle_impl! {(A, 0), (B, 1), (C, 2), (D, 3), (E, 4)}
bundle_impl! {(A, 0), (B, 1), (C, 2), (D, 3), (E, 4), (F, 5)}
bundle_impl! {(A, 0), (B, 1), (C, 2), (D, 3), (E, 4), (F, 5), (G, 6)}
bundle_impl! {(A, 0), (B, 1), (C, 2), (D, 3), (E, 4), (F, 5), (G, 6), (H, 7)}
