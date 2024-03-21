use std::{thread::sleep, time::Duration};

use g29::{G29Led, G29Options, G29};

fn main() {
    let mut g29 = G29::new(G29Options {
        debug: true,
        ..Default::default()
    });
    g29.initialize();

    loop {
        g29.set_leds(G29Led::All);
        sleep(Duration::from_millis(100));
        g29.set_leds(G29Led::Red | G29Led::GreenOne);
        sleep(Duration::from_millis(100));
    }
}
