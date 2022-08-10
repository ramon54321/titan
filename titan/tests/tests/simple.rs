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
fn multi_archetype() {
    #[component]
    struct Height(u8);
    #[component]
    struct Age(u8);
    #[component]
    struct Weight(u8);

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
