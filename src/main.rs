use g29::events::Event;
use g29::{Options, G29};

fn main() {
    let options = Options::default();
    let g29 = G29::connect(options);

    g29.register_event_handler(Event::OptionButtonReleased, |g29| {
        g29.disconnect();
    });

    g29.register_event_handler(Event::Clutch, |g29| {
        println!("Clutch {}", g29.clutch());
    });

    g29.register_event_handler(Event::Brake, |g29| {
        println!("Brake {}", g29.brake());
    });

    g29.register_event_handler(Event::Throttle, |g29| {
        println!("Throttle {}", g29.throttle());
    });

    g29.register_event_handler(Event::Steering, |g29| {
        println!("Steering {}", g29.steering());
    });

    while g29.connected() {}
}
