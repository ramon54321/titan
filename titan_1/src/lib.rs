use registry::Registry;
use serde::{Deserialize, Serialize};
use storage::{Archetype, Storage};

#[test]
fn master() {
    use serialization::Serializable;

    #[derive(Serialize, Deserialize)]
    struct Age(u8);
    #[derive(Serialize, Deserialize)]
    struct Name(String);

    let mut registry = Registry::new();
    registry.register_component::<Age>(&"Age");
    registry.register_component::<Name>(&"Name");
    registry.register_archetype::<(Age, Name)>(&"AgeName");

    let mut storage = Storage::new();
    storage.spawn(&registry, (Age(23), Name("Jeff".to_string())));
    storage.spawn(&registry, (Age(19), Name("Julia".to_string())));

    let storage_serial = storage.serialize(&registry);
    println!("{}", storage_serial);
}

mod serialization {
    use crate::{registry::Registry, storage::Storage};

    pub trait Serializable<T> {
        fn serialize(&self, registry: &Registry) -> String;
        fn deserialize(serial: &str) -> T;
    }

    impl Serializable<Storage> for Storage {
        fn serialize(&self, registry: &Registry) -> String {
            let mut entity_values = Vec::new();
            for (bundle_kind, archetype) in self.archetype_by_bundle_kind.iter() {
                let archetype_entity_serialize_fn =
                    registry.bundle_kind_to_archetype_entity_serialize_fn(bundle_kind.clone());

                // Serialize each entity in archetype
                for i in 0..archetype.get_entity_count() {
                    let entity_value = (archetype_entity_serialize_fn)(i, &archetype);
                    entity_values.push(entity_value);
                }
            }
            let entities_string = serde_json::to_string(&entity_values).unwrap();
            entities_string
        }
        fn deserialize(serial: &str) -> Storage {
            Storage::new()
        }
    }
} /* serialization */

type EntityId = usize;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct BundleKind(String);
impl BundleKind {
    fn from_component_kinds(component_kinds: &[ComponentKind]) -> Self {
        let mut name = String::new();
        for component_kind in component_kinds {
            name.push_str(component_kind.0.as_str());
        }
        BundleKind(name)
    }
}

pub trait Bundle {
    fn push_into_archetype(self, entity_id: EntityId, archetype: &mut Archetype);
}

macro_rules! bundle_impl {
    (
        $(($name:ident, $i:tt)),*
    ) => {
        impl<$($name),*> Bundle for ($($name),*,)
        where
            $($name: 'static),* {
            fn push_into_archetype(self, entity_id: EntityId, archetype: &mut Archetype) {
                archetype.push_entity_id(entity_id);
                $(archetype.push_component(self.$i));*
            }
        }
    };
}

bundle_impl! {(A, 0)}
bundle_impl! {(A, 0), (B, 1)}
bundle_impl! {(A, 0), (B, 1), (C, 2)}
bundle_impl! {(A, 0), (B, 1), (C, 2), (D, 3)}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct ComponentKind(String);

mod storage {
    use crate::registry::Registry;
    use crate::{Bundle, BundleKind, ComponentKind, EntityId};
    use std::any::{Any, TypeId};
    use std::collections::HashMap;

    pub struct Storage {
        current_entity_id: EntityId,
        pub(crate) archetype_by_bundle_kind: HashMap<BundleKind, Archetype>,
    }
    impl Storage {
        pub fn new() -> Self {
            Self {
                current_entity_id: 0,
                archetype_by_bundle_kind: HashMap::new(),
            }
        }
        pub fn spawn<T: Bundle + 'static>(&mut self, registry: &Registry, bundle: T) {
            let bundle_type_id = TypeId::of::<T>();
            let bundle_kind = registry.bundle_type_id_to_kind(bundle_type_id);

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
            bundle.push_into_archetype(self.current_entity_id, archetype);

            // Increment entity_id for next spawn
            self.current_entity_id = self.current_entity_id + 1;
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
} /* storage */

mod registry {
    use crate::storage::Archetype;
    use crate::BundleKind;
    use crate::ComponentKind;
    use serde::{de::DeserializeOwned, Deserialize, Serialize};
    use serde_json::Map;
    use serde_json::Value;
    use std::{
        any::{Any, TypeId},
        collections::{HashMap, HashSet},
    };

    type SerializeFn = Box<dyn Fn(&dyn Any) -> Value>;
    type DeserializeFn = Box<dyn Fn(&str) -> Box<dyn Any>>;
    type ArchetypeEntitySerializeFn = Box<dyn Fn(usize, &Archetype) -> Value>;

    pub struct Registry {
        type_ids: HashSet<TypeId>,
        type_id_to_kind: HashMap<TypeId, ComponentKind>,
        kind_to_type_id: HashMap<ComponentKind, TypeId>,
        kind_to_serializer: HashMap<ComponentKind, SerializeFn>,
        kind_to_deserializer: HashMap<ComponentKind, DeserializeFn>,
        bundle_type_ids: HashSet<TypeId>,
        bundle_type_id_to_kind: HashMap<TypeId, BundleKind>,
        bundle_kind_to_type_id: HashMap<BundleKind, TypeId>,
        bundle_kind_to_archetype_entity_serialize_fn:
            HashMap<BundleKind, ArchetypeEntitySerializeFn>,
    }
    impl Registry {
        pub fn new() -> Self {
            Self {
                type_ids: HashSet::new(),
                type_id_to_kind: HashMap::new(),
                kind_to_type_id: HashMap::new(),
                kind_to_serializer: HashMap::new(),
                kind_to_deserializer: HashMap::new(),
                bundle_type_ids: HashSet::new(),
                bundle_type_id_to_kind: HashMap::new(),
                bundle_kind_to_type_id: HashMap::new(),
                bundle_kind_to_archetype_entity_serialize_fn: HashMap::new(),
            }
        }
        pub fn register_component<T: RegisterComponent>(&mut self, name: &str) {
            T::register(self, name);
        }
        pub fn register_archetype<T: RegisterArchetype>(&mut self, name: &str) {
            T::register(self, name);
        }
        pub(crate) fn bundle_type_id_to_kind(&self, type_id: TypeId) -> BundleKind {
            self.bundle_type_id_to_kind
                .get(&type_id)
                .expect("Could not get bundle kind given type_id")
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
        //pub(crate) fn kind_to_deserializer(&self, kind: ComponentKind) -> &DeserializeFn {
        //self.kind_to_deserializer.get(&kind).expect("Could not get deserialize_fn given kind")
        //}
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
            registry.kind_to_type_id.insert(kind.clone(), type_id);

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

    impl<A, B> RegisterArchetype for (A, B)
    where
        A: Serialize + DeserializeOwned + 'static,
        B: Serialize + DeserializeOwned + 'static,
    {
        #[allow(non_snake_case)]
        fn register(registry: &mut Registry, name: &str) {
            // Register TypeId and Kind
            let type_id_A = TypeId::of::<A>();
            let type_id_B = TypeId::of::<B>();
            let kind_A = registry.type_id_to_kind(type_id_A);
            let kind_B = registry.type_id_to_kind(type_id_B);
            let bundle_type_id = TypeId::of::<(A, B)>();
            let bundle_kind = BundleKind::from_component_kinds(&[kind_A, kind_B]);
            registry.bundle_type_ids.insert(bundle_type_id);
            registry
                .bundle_type_id_to_kind
                .insert(bundle_type_id, bundle_kind.clone());
            registry
                .bundle_kind_to_type_id
                .insert(bundle_kind.clone(), bundle_type_id);

            // Register SerializeFn
            let archetype_entity_serialize_fn = |entity_index: usize, archetype: &Archetype| {
                let entity_id = archetype.get_entity_id_at_index_unchecked(entity_index);

                // Serialize each component
                let component_a = archetype.get_component_at_index_unchecked::<A>(entity_index);
                let component_a_value = serde_json::to_value(component_a).unwrap();
                let component_b = archetype.get_component_at_index_unchecked::<B>(entity_index);
                let component_b_value = serde_json::to_value(component_b).unwrap();

                // Build entity object
                let mut entity_object = Map::new();
                entity_object.insert("entity_id".to_string(), Value::from(entity_id));
                entity_object.insert("A".to_string(), component_a_value);
                entity_object.insert("B".to_string(), component_b_value);

                Value::from(entity_object)
            };
            registry
                .bundle_kind_to_archetype_entity_serialize_fn
                .insert(bundle_kind.clone(), Box::new(archetype_entity_serialize_fn));

            //let kind_serialize_fn = |item: &dyn Any| {
            //let item = item
            //.downcast_ref::<T>()
            //.expect("Could not downcast item to T");
            //serde_json::to_value(item).expect("Could not serialize kind to value")
            //};
            //registry
            //.kind_to_serializer
            //.insert(kind.clone(), Box::new(kind_serialize_fn));
        }
    }

    #[test]
    fn can_register_simple() {
        #[derive(Serialize, Deserialize)]
        struct Name(String);

        let mut registry = Registry::new();
        registry.register_component::<Name>(&"Name");

        assert_eq!(registry.type_ids.len(), 1);
        assert_eq!(registry.type_id_to_kind.len(), 1);

        let component_type_id = TypeId::of::<Name>();
        assert_eq!(
            *registry.type_id_to_kind.get(&component_type_id).unwrap(),
            ComponentKind("Name".to_string()),
        );

        assert_eq!(registry.kind_to_serializer.len(), 1);
        assert_eq!(registry.kind_to_deserializer.len(), 1);
    }
} /* registry */
