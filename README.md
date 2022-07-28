<div align="center">
  <span><img src="https://upload.wikimedia.org/wikipedia/commons/thumb/d/d5/Rust_programming_language_black_logo.svg/1920px-Rust_programming_language_black_logo.svg.png" width="100"></span>
</div>

## Titan: Minimal, Explicit and Simplistic ECS in Rust

Titan aims to be a minimal yet useful [Entity Component System](https://en.wikipedia.org/wiki/Entity_component_system) library, supporting full serialization and deserialization of stored data. The library is entirely written in Rust.

Other ECSs tend to be heavy and overly complicated. The core tenets of an ECS is that bundles of components can be spawned and removed, and that bundles containing specific sets of components can be iterated over quickly. Titan uses an `archetype` layout for component data, so adding and removing of components are slow in comparison to direct component vecs. However, this allows very fast component iteration, which is arguably the most important consideration for an ECS. 

### Example

```rust
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
```

### Development

- [x] Manual Registation of Components and Archetypes
- [x] Spawn component Bundles
- [x] Serialize and Deserialize storage
- [ ] Iterate Archetypes
- [ ] Iterate based on Filter 

