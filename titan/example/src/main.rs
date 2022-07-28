use serde::{Deserialize, Serialize};
use titan::ECS;

fn main() {
    println!("Titan Example!");

    let mut ecs = ECS::default();
    ecs.register_component::<Person>("Person");
    ecs.register_archetype::<(Person,)>();
    ecs.spawn_bundle((Person {
        age: 34,
        height: 175,
    },));

    let ecs_serial = ecs.serialize();
    println!("Serial: {}", ecs_serial);

    let mut ecs_2 = ECS::default();
    ecs_2.register_component::<Person>("Person");
    ecs_2.register_archetype::<(Person,)>();
    ecs_2.deserialize(&ecs_serial);

    let ecs_2_serial = ecs_2.serialize();
    println!("Serial: {}", ecs_2_serial);

    assert_eq!(ecs_serial, ecs_2_serial);
}

#[derive(Serialize, Deserialize)]
struct Person {
    age: u8,
    height: u16,
}
