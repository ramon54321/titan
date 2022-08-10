use titan::ResultIteration;
use titan::ECS;
use titan_macros::component;

fn main() {
    // Create main ECS instance.
    let mut ecs = ECS::default();

    // Register each individual component.
    ecs.register_component::<Id>();
    ecs.register_component::<Position>();
    ecs.register_component::<Person>();

    // Register each archetype which is a unique component combination.
    ecs.register_archetype::<(Id, Position)>();
    ecs.register_archetype::<(Id, Position, Person)>();

    // Spawn entities, 2 of archetype (Id, Position, Person) and 1 of archetype (Id, Position).
    // Note, order of components in the bundle does not matter.
    ecs.spawn_bundle((
        Person {
            age: 34,
            height: 175,
            weight: 75,
        },
        Id(23),
        Position { x: -34.5, y: 88.1 },
    ));
    ecs.spawn_bundle((
        Id(91),
        Position { x: 7.1, y: 12.9 },
        Person {
            age: 24,
            height: 160,
            weight: 68,
        },
    ));
    ecs.spawn_bundle((Position { x: 103.4, y: -71.7 }, Id(55)));

    // Query for specific archetypes. & asks for read permission, whereas &mut requests write
    // permission. Note, once again the order does not matter.
    for (id, person, position) in ecs
        .query::<(&Id, &mut Person, &mut Position)>()
        .result_iter()
    {
        println!(
            "{:?} is at {:?} and has a height of {}",
            id, position, person.height
        );

        // We can also mutate components which were requested with &mut.
        position.x = 0.0;
    }

    // Serialize the component storage into JSON.
    let ecs_serial = ecs.serialize();

    // Create a new ECS into which we will deserialize the previous ECS.
    let mut ecs_2 = ECS::default();

    // Once again we register the components into the new ECS. This would usually be done on
    // startup. Since you won't have more than 1 ECS instance, you do not need to register
    // components multiple times.
    ecs_2.register_component::<Id>();
    ecs_2.register_component::<Position>();
    ecs_2.register_component::<Person>();
    ecs_2.register_archetype::<(Id, Position)>();
    ecs_2.register_archetype::<(Id, Position, Person)>();

    // Deserialize the 'saved' ECS into the new ECS instance.
    ecs_2.deserialize(&ecs_serial);

    // Query new ECS and print deserialized components that match query.
    for (id, person, position) in ecs
        .query::<(&Id, &mut Person, &mut Position)>()
        .result_iter()
    {
        println!(
            "{:?} is at {:?} and has a height of {}",
            id, position, person.height
        );
    }
}

#[component]
struct Id(u32);

#[component]
struct Position {
    x: f32,
    y: f32,
}

#[component]
struct Person {
    age: u8,
    height: u16,
    weight: u16,
}
