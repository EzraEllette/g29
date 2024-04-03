use g29::{Options, G29};

fn main() {
    let g29 = G29::connect(Options::default());

    g29.register_event_handler(g29::events::Event::CircleButtonPressed, |_| {
        println!("Circle button pressed");
    });

    g29.register_event_handler(g29::events::Event::CircleButtonReleased, |_| {
        println!("Circle button released");
    });

    g29.register_event_handler(g29::events::Event::TriangleButtonPressed, |_| {
        println!("Triangle button pressed");
    });

    g29.register_event_handler(g29::events::Event::TriangleButtonReleased, |_| {
        println!("Triangle button released");
    });

    g29.register_event_handler(g29::events::Event::SquareButtonPressed, |_| {
        println!("Square button pressed");
    });

    g29.register_event_handler(g29::events::Event::SquareButtonReleased, |_| {
        println!("Square button released");
    });

    g29.register_event_handler(g29::events::Event::LeftShifterReleased, |g29| {
        g29.disconnect();
    });

    g29.register_event_handler(g29::events::Event::Throttle, |g29| {
        println!("Throttle: {}", g29.throttle());
    });

    while g29.connected() {}
}
