use g29::{Options, G29};

fn main() {
    let mut g29 = G29::connect(Options::default());

    // g29.register_event_handler(g29::Event::Clutch, |g29| {
    //     println!("Clutch: {}", g29.clutch());
    // });

    // println!("{:?}", g29.handlers());
    // g29.handlers();

    loop {}
}
