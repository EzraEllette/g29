use g29::{G29Options, G29};

fn main() {
    let mut g29 = G29::new(G29Options {
        debug: true,
        ..Default::default()
    });
    g29.initialize();

    loop {
        std::thread::sleep(std::time::Duration::from_millis(100));
        println!(
            " Throttle: {} Brake: {} Steering: {}",
            g29.throttle(),
            g29.brake(),
            g29.steering()
        );
    }
}
