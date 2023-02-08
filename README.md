# RandomId

Provides a lazy iterator that generates IDs in a random order using the FPE algorithm. This is useful for generating IDs that are not sequential, but are unique.

## Usage

```rust
use random_id::RandomIdGenerator;
use rand::prelude::*;

let mut rng = rand::thread_rng();
let mut key = [0u8; 32];
rng.fill(&mut key);

let mut id_generator = RandomIdGenerator::new(key, 1234, 1);

for i in id_generator.take(10) {
   println!("{}", i);
}
```
