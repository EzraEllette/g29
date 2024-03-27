use g29::{Options, G29};

fn main() {
    let g29 = G29::connect(Options::default());

    g29.register_event_handler(g29::Event::OptionButtonReleased, |g29| g29.disconnect());

    // println!("{:?}", g29.handlers());
    // g29.handlers();

    while g29.connected() {
        println!(
            " Throttle: {} | Brake: {} | Clutch: {}",
            g29.throttle(),
            g29.brake(),
            g29.clutch()
        );
    }
}
