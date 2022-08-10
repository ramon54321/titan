use titan::*;
use titan_macros::component;

#[test]
fn basic() {
    #[component]
    struct Person {
        age: u8,
        height: u16,
    }

    let mut ecs = ECS::default();
    ecs.register_component::<Person>();
    ecs.register_archetype::<(Person,)>();
    ecs.spawn_bundle((Person {
        age: 34,
        height: 175,
    },));

    let ecs_serial = ecs.serialize();
    let mut ecs_2 = ECS::default();
    ecs_2.register_component::<Person>();
    ecs_2.register_archetype::<(Person,)>();
    ecs_2.deserialize(&ecs_serial);
    let ecs_2_serial = ecs_2.serialize();

    assert_eq!(ecs_serial, ecs_2_serial);
}

#[test]
fn bundle_kinds() {
    #[component]
    struct Height(u8);
    #[component]
    struct Age(u8);
    #[component]
    struct Weight(u8);

    let bundle_kind_a = <(Height, Age, Weight)>::get_bundle_kind();
    let bundle_kind_b = <(Age, Weight, Height)>::get_bundle_kind();

    assert_eq!(bundle_kind_a, bundle_kind_b);
}

#[test]
fn filter_archetypes_simple() {
    #[component]
    #[derive(PartialEq)]
    struct Height(u8);
    #[component]
    #[derive(PartialEq)]
    struct Age(u8);
    #[component]
    #[derive(PartialEq)]
    struct Weight(u8);

    let mut ecs = ECS::default();
    ecs.register_component::<Height>();
    ecs.register_component::<Age>();
    ecs.register_component::<Weight>();
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

    assert!(result.contains(&(&Age(20), &Height(180))));
    assert!(result.contains(&(&Age(40), &Height(150))));
    assert!(result.contains(&(&Age(50), &Height(160))));
    assert_eq!(result.len(), 3);
}
