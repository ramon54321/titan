use crate::{bundle::BundleKind, registry::Registry, storage::Storage};
use serde_json::Value;

pub trait Serializable<T> {
    fn serialize(&self, registry: &Registry) -> String;
    fn deserialize(serial: &str, registry: &Registry) -> T;
}

impl Serializable<Storage> for Storage {
    fn serialize(&self, registry: &Registry) -> String {
        let mut entity_values = Vec::new();
        for (bundle_kind, archetype) in self.archetype_by_bundle_kind.iter() {
            let archetype_entity_serialize_fn =
                registry.bundle_kind_to_archetype_entity_serialize_fn(bundle_kind.clone());

            // Serialize each entity in archetype
            for i in 0..archetype.get_entity_count() {
                let entity_value = (archetype_entity_serialize_fn)(i, &archetype, bundle_kind);
                entity_values.push(entity_value);
            }
        }
        let entities_string = serde_json::to_string(&entity_values).unwrap();
        entities_string
    }
    fn deserialize(serial: &str, registry: &Registry) -> Storage {
        let entity_values = serde_json::from_str::<Value>(serial)
            .expect("Could not deserialize entities from JSON");
        let entity_values = entity_values
            .as_array()
            .expect("Could not parse JSON value as array");
        let mut storage = Storage::new();
        for entity_value in entity_values {
            let entity_object = entity_value
                .as_object()
                .expect("Could not parse JSON value as object");
            let bundle_kind_string = entity_object
                .get(&"bundle_kind".to_string())
                .expect("Could not get bundle_kind on JSON value")
                .as_str()
                .expect("Could not parse JSON bundle_kind as str")
                .to_string();
            let bundle_kind = BundleKind(bundle_kind_string);
            let archetype_entity_deserialize_fn =
                registry.bundle_kind_to_archetype_entity_deserialize_fn(bundle_kind);
            (archetype_entity_deserialize_fn)(entity_value, &mut storage);
        }
        storage
    }
}
