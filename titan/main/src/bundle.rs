use crate::{storage::Archetype, ComponentKind, ComponentMeta, EntityId};
use paste::paste;
use std::hash::Hash;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct BundleKind(pub(crate) String);
impl BundleKind {
    pub(crate) fn from_component_kinds(component_kinds: &[ComponentKind]) -> Self {
        let mut name = String::new();
        let mut component_kinds = component_kinds.to_vec();
        component_kinds.sort_by_key(|component_kind| component_kind.0.to_lowercase());
        for component_kind in component_kinds {
            name.push_str(component_kind.0.as_str());
        }
        BundleKind(name)
    }
}

pub trait Bundle {
    fn push_into_archetype(self, entity_id: EntityId, archetype: &mut Archetype);
    fn get_bundle_kind() -> BundleKind;
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
            #[allow(non_snake_case)]
            fn get_bundle_kind() -> BundleKind {
                $(let paste!{[<kind_ $name>]} = <$name>::get_component_kind();)*
                let bundle_kind = BundleKind::from_component_kinds(&[$(paste!{[<kind_ $name>]}),*]);
                bundle_kind
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
