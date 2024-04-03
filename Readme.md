# G29

## Description

Rust crate for using the logitech G29 steering wheel with force feedback.
More Force Feedback options coming soon.

Thanks to @nightmode for their NodeJS library that I frequently referenced. [logitech-g29](https://github.com/nightmode/logitech-g29)

## Example

```rust
use g29::{Options, G29};

fn main() {
    let g29 = G29::connect(Options::default());

    g29.register_event_handler(
        g29::events::Event::PlaystationButtonReleased,
        playstation_button_released_handler,
    );

    g29.register_event_handler(g29::events::Event::Throttle, throttle_handler);

    g29.register_event_handler(g29::events::Event::Brake, brake_handler);

    g29.register_event_handler(g29::events::Event::Clutch, clutch_handler);

    while g29.connected() {}
}

fn playstation_button_released_handler(g29: &mut G29) {
    g29.disconnect();
    println!("Playstation button released");
}

fn throttle_handler(g29: &mut G29) {
    println!("Throttle: {}", g29.throttle());
}

fn brake_handler(g29: &mut G29) {
    println!("Brake: {}", g29.brake());
}

fn clutch_handler(g29: &mut G29) {
    println!("Clutch: {}", g29.clutch());
}
```
