use crate::{
    bundle::{Bundle, BundleId, BundleKind},
    storage::{Archetype, Storage},
    ComponentKind, ComponentMeta,
};
use paste::paste;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Map;
use serde_json::Value;
use std::{
    any::{Any, TypeId},
    collections::{HashMap, HashSet},
};

type SerializeFn = Box<dyn Fn(&dyn Any) -> Value>;
type DeserializeFn = Box<dyn Fn(&str) -> Box<dyn Any>>;
type ArchetypeEntitySerializeFn = Box<dyn Fn(usize, &Archetype, &BundleKind) -> Value>;
type ArchetypeEntityDeserializeFn = Box<dyn Fn(&Value, &mut Storage, &Registry)>;

pub struct Registry {
    type_ids: HashSet<TypeId>,
    type_id_to_kind: HashMap<TypeId, ComponentKind>,
    kind_to_serializer: HashMap<ComponentKind, SerializeFn>,
    kind_to_deserializer: HashMap<ComponentKind, DeserializeFn>,
    bundle_ids: HashSet<BundleId>,
    bundle_id_to_bundle_kind: HashMap<BundleId, BundleKind>,
    bundle_kind_to_bundle_id: HashMap<BundleKind, BundleId>,
    bundle_kind_to_archetype_entity_serialize_fn: HashMap<BundleKind, ArchetypeEntitySerializeFn>,
    bundle_kind_to_archetype_entity_deserialize_fn:
        HashMap<BundleKind, ArchetypeEntityDeserializeFn>,
}
impl Registry {
    pub(crate) fn new() -> Self {
        Self {
            type_ids: HashSet::new(),
            type_id_to_kind: HashMap::new(),
            kind_to_serializer: HashMap::new(),
            kind_to_deserializer: HashMap::new(),
            bundle_ids: HashSet::new(),
            bundle_id_to_bundle_kind: HashMap::new(),
            bundle_kind_to_bundle_id: HashMap::new(),
            bundle_kind_to_archetype_entity_serialize_fn: HashMap::new(),
            bundle_kind_to_archetype_entity_deserialize_fn: HashMap::new(),
        }
    }
    pub(crate) fn register_component<T: RegisterComponent>(&mut self, name: &str) {
        T::register(self, name);
    }
    pub(crate) fn register_archetype<T: RegisterArchetype>(&mut self) {
        T::register(self, "");
    }
    pub(crate) fn bundle_id_to_bundle_kind(&self, bundle_id: BundleId) -> BundleKind {
        self.bundle_id_to_bundle_kind
                .get(&bundle_id)
                .expect("Could not get bundle kind given bundle_id; likely caused by not registering the corresponding archetype")
                .clone()
    }
    pub(crate) fn type_id_to_kind(&self, type_id: TypeId) -> ComponentKind {
        self.type_id_to_kind
            .get(&type_id)
            .expect("Could not get kind given type_id")
            .clone()
    }
    pub(crate) fn bundle_kind_to_archetype_entity_serialize_fn(
        &self,
        kind: BundleKind,
    ) -> &ArchetypeEntitySerializeFn {
        self.bundle_kind_to_archetype_entity_serialize_fn
            .get(&kind)
            .expect("Could not get serialize_fn given kind")
    }
    pub(crate) fn bundle_kind_to_archetype_entity_deserialize_fn(
        &self,
        kind: BundleKind,
    ) -> &ArchetypeEntityDeserializeFn {
        self.bundle_kind_to_archetype_entity_deserialize_fn
            .get(&kind)
            .expect("Could not get deserialize_fn given kind")
    }
}

pub trait RegisterComponent {
    fn register(registry: &mut Registry, name: &str);
}

impl<T> RegisterComponent for T
where
    T: Serialize + DeserializeOwned + 'static,
{
    fn register(registry: &mut Registry, name: &str) {
        // Register TypeId and Kind
        let type_id = TypeId::of::<T>();
        let kind = ComponentKind(name.to_string());
        registry.type_ids.insert(type_id);
        registry.type_id_to_kind.insert(type_id, kind.clone());

        // Register SerializeFn
        let kind_serialize_fn = |item: &dyn Any| {
            let item = item
                .downcast_ref::<T>()
                .expect("Could not downcast item to T");
            serde_json::to_value(item).expect("Could not serialize kind to value")
        };
        registry
            .kind_to_serializer
            .insert(kind.clone(), Box::new(kind_serialize_fn));

        // Register DeserializeFn
        let kind_deserialize_fn = |item_serial: &str| {
            Box::new(
                serde_json::from_str::<T>(item_serial)
                    .expect("Could not deserialize str given kind"),
            ) as Box<dyn Any>
        };
        registry
            .kind_to_deserializer
            .insert(kind.clone(), Box::new(kind_deserialize_fn));
    }
}

pub trait RegisterArchetype {
    fn register(registry: &mut Registry, name: &str);
}

macro_rules! register_archetype_impl {
        (
            $($name:ident),*
        ) => {
            impl<$($name),*> RegisterArchetype for ($($name),*,)
            where
                $(
                    $name: Serialize + DeserializeOwned + ComponentMeta + 'static
                 ),*,
            {
                #[allow(non_snake_case)]
                fn register(registry: &mut Registry, _name: &str) {
                    // Register TypeId and Kind
                    $(
                        let paste!{[<type_id_ $name>]} = TypeId::of::<$name>();
                     )*
                    $(
                        let paste!{[<kind_ $name>]} = paste!{registry.type_id_to_kind([<type_id_ $name>])};
                     )*
                    let bundle_id = <(
                        $($name),*,
                                                      )>::get_bundle_id();
                    let bundle_kind = BundleKind::from_component_kinds(&[
                                                                       $(
                                                                           paste!{[<kind_ $name>]}
                                                                        ),*
                    ]);
                    registry.bundle_ids.insert(bundle_id.clone());
                    registry
                        .bundle_id_to_bundle_kind
                        .insert(bundle_id.clone(), bundle_kind.clone());
                    registry
                        .bundle_kind_to_bundle_id
                        .insert(bundle_kind.clone(), bundle_id.clone());

                    // Register SerializeFn
                    let archetype_entity_serialize_fn =
                        |entity_index: usize, archetype: &Archetype, bundle_kind: &BundleKind| {
                            let entity_id =
                                archetype.get_entity_id_at_index_unchecked(entity_index);

                            // Serialize each component
                            $(
                                let paste!{[<component_ $name>]} = &archetype.get_component_vec_lock::<$name>()[entity_index];
                                let paste!{[<component_ $name _value>]} =
                                    serde_json::to_value(paste!{[<component_ $name>]}).unwrap();
                             )*

                            // Build entity object
                            let mut entity_object = Map::new();
                            entity_object.insert(
                                "bundle_kind".to_string(),
                                Value::from(bundle_kind.0.clone()),
                            );
                            entity_object.insert("entity_id".to_string(), Value::from(entity_id));

                            $(
                                entity_object.insert(stringify!{$name}.to_string(),
                                    paste!{[<component_ $name _value>]});
                             )*

                            Value::from(entity_object)
                        };
                    registry
                        .bundle_kind_to_archetype_entity_serialize_fn
                        .insert(bundle_kind.clone(), Box::new(archetype_entity_serialize_fn));

                    // Register DeserializeFn
                    let archetype_entity_deserialize_fn =
                        |entity_value: &Value, storage: &mut Storage, registry: &Registry| {
                            let entity_object = entity_value
                                .as_object()
                                .expect("Could not parse JSON value as object");
                            let entity_id = entity_object
                                .get(&"entity_id".to_string())
                                .expect("Could not get JSON entity_id")
                                .as_u64()
                                .expect("Could not parse JSON value as u64");

                            $(
                                let paste!{[<component_ $name _value>]} = entity_object
                                    .get(&stringify!{$name}.to_string())
                                    .expect("Could not get JSON component");
                                let paste!{[<component_ $name>]} =
                                    serde_json::from_value::<$name>(paste!{[<component_ $name _value>]}.clone())
                                        .expect("Could not parse JSON value as component");
                             )*

                                let bundle = (
                                    $(
                                        paste!{[<component_ $name>]}
                                     ),*,
                                             );

                            storage.spawn_with_entity_id(registry, entity_id as usize, bundle);
                        };
                    registry
                        .bundle_kind_to_archetype_entity_deserialize_fn
                        .insert(
                            bundle_kind.clone(),
                            Box::new(archetype_entity_deserialize_fn),
                        );
                }
            }
        };
    }

register_archetype_impl! { A }
register_archetype_impl! { A, B }
register_archetype_impl! { A, B, C }
register_archetype_impl! { A, B, C, D }
register_archetype_impl! { A, B, C, D, E }
register_archetype_impl! { A, B, C, D, E, F }
register_archetype_impl! { A, B, C, D, E, F, G }
register_archetype_impl! { A, B, C, D, E, F, G, H }
