use g29::{G29Options, G29};

fn main() {
    let mut g29 = G29::new(G29Options {
        debug: true,
        ..Default::default()
    });
    g29.initialize();

    g29.auto_center_complex(80, 100, 0x03, 0x03, false, 255);
    loop {}
}
