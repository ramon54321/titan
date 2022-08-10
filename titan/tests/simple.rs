use std::any::TypeId;

use serde::{Deserialize, Serialize};
use titan::*;
use titan_macros::component;

#[test]
fn basic() {
    #[derive(Debug, Serialize, Deserialize)]
    struct Person {
        age: u8,
        height: u16,
    }

    let mut ecs = ECS::default();
    ecs.register_component::<Person>("Person");
    ecs.register_archetype::<(Person,)>();
    ecs.spawn_bundle((Person {
        age: 34,
        height: 175,
    },));

    let ecs_serial = ecs.serialize();
    let mut ecs_2 = ECS::default();
    ecs_2.register_component::<Person>("Person");
    ecs_2.register_archetype::<(Person,)>();
    ecs_2.deserialize(&ecs_serial);
    let ecs_2_serial = ecs_2.serialize();

    assert_eq!(ecs_serial, ecs_2_serial);
}

#[test]
fn master() {
    use serde::{Deserialize, Serialize};

    #[component]
    struct Age(u8);
    #[component]
    struct Name(String);
    #[component]
    struct Person {
        height: u16,
        weight: u16,
    }

    let mut registry = Registry::new();
    registry.register_component::<Age>(&"Age");
    registry.register_component::<Name>(&"Name");
    registry.register_component::<Person>(&"Person");
    registry.register_archetype::<(Name, Age)>();

    let mut storage = Storage::new();
    storage.spawn(&registry, (Age(23), Name("Jeff".to_string())));
    storage.spawn(&registry, (Age(19), Name("Julia".to_string())));
    storage.spawn(&registry, (Name("Bob".to_string()), Age(29)));

    for (age, name) in storage.query::<(&mut Age, &Name)>().result_iter() {
        if name.0 == "Julia" {
            age.0 = 34;
        }
    }

    let storage_serial = storage.serialize(&registry);
    println!("{}", storage_serial);

    let new_storage = Storage::deserialize(&storage_serial, &registry);
    let new_storage_serial = new_storage.serialize(&registry);
    println!("{}", new_storage_serial);

    for (age, name) in new_storage.query::<(&Age, &Name)>().result_iter() {
        println!("{} is {} years old.", name.0, age.0);
    }
}

#[test]
fn multi_archetype() {
    #[derive(Debug, Serialize, Deserialize)]
    struct Height(u8);
    #[derive(Debug, Serialize, Deserialize)]
    struct Age(u8);
    #[derive(Debug, Serialize, Deserialize)]
    struct Weight(u8);

    println!("{:?}", TypeId::of::<Age>());
    println!("{:?}", TypeId::of::<Height>());
    println!("{:?}", TypeId::of::<Weight>());

    let mut ecs = ECS::default();
    ecs.register_component::<Height>("Height");
    ecs.register_component::<Age>("Age");
    ecs.register_component::<Weight>("Weight");
    ecs.register_archetype::<(Age,)>();
    ecs.register_archetype::<(Height, Age)>();
    ecs.register_archetype::<(Height, Age, Weight)>();
    ecs.spawn_bundle((Age(10),));
    ecs.spawn_bundle((Age(20), Height(180)));
    ecs.spawn_bundle((Age(30),));
    ecs.spawn_bundle((Age(40), Height(150)));
    ecs.spawn_bundle((Age(50), Height(160), Weight(80)));

    let mut result = ecs.query::<(&Age, &Height)>();
    let result: Vec<_> = result.result_iter().collect();
    println!("Result: {:?}", result);
}

#[test]
fn fetch_single() {
    use crate::registry::Registry;
    use serde::{Deserialize, Serialize};
    #[derive(Debug, Serialize, Deserialize)]
    struct Age(u8);
    #[derive(Debug, Serialize, Deserialize)]
    struct Name(String);
    #[derive(Debug, Serialize, Deserialize)]
    struct Person {
        height: u16,
        weight: u16,
    }

    let mut registry = Registry::new();
    registry.register_component::<Age>(&"Age");
    registry.register_component::<Name>(&"Name");
    registry.register_component::<Person>(&"Person");
    registry.register_archetype::<(Name, Age)>();

    let mut storage = Storage::new();
    storage.spawn(&registry, (Age(23), Name("Jeff".to_string())));
    storage.spawn(&registry, (Age(19), Name("Julia".to_string())));
    storage.spawn(&registry, (Name("Bob".to_string()), Age(29)));

    for (age, name) in <(&mut Age, &Name)>::query(&storage).result_iter() {
        println!("{:?}", (&age, name));
        age.0 = age.0 + 1;
    }
    for (age, name) in <(&mut Age, &Name)>::query(&storage).result_iter() {
        println!("{:?}", (age, name));
    }
    for (age, name) in storage.query::<(&mut Age, &Name)>().result_iter() {
        println!("{:?}", (age, name));
    }
}

#[test]
fn can_register_simple() {
    use serde::Deserialize;
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
