# G29

## Description

Rust crate for using the logitech G29 steering wheel with force feedback.
More Force Feedback options coming soon.

## Features

- Events (coming soon)

## Example

```rust
use g29::{G29, Options};

fn main {
  let g29 = G29::connect(Options::default());

  loop {
    println!("Throttle: {:?}", g29.throttle());
  }
}
```
