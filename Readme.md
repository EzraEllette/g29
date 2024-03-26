# G29

## Description

Rust crate for using the logitech G29 steering wheel with force feedback.
More Force Feedback options coming soon.

Currently the `events` feature is a bit slow in my opinion, so I'm working on optimization.

Thanks to @nightmode for their NodeJS library that I frequently referenced. [logitech-g29](https://github.com/nightmode/logitech-g29)

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

### Buy me a coffee
