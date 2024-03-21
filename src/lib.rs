use std::{
    env::consts::OS,
    ops::BitOr,
    process::exit,
    sync::{Arc, Mutex, RwLock},
    thread::{self, sleep},
    time::Duration,
};

use data_map::map_data;
use hidapi::{DeviceInfo, HidApi};

pub mod data_map;

#[repr(u8)]
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum G29Led {
    None = 0x0,
    GreenOne = 0x01,
    GreenTwo = 0x02,
    OrangeOne = 0x04,
    OrangeTwo = 0x08,
    Red = 0x10,
    All = 0x1F,
    Other(u8),
}

impl G29Led {
    fn as_u8(&self) -> u8 {
        match self {
            G29Led::None => 0x0,
            G29Led::GreenOne => 0x01,
            G29Led::GreenTwo => 0x02,
            G29Led::OrangeOne => 0x04,
            G29Led::OrangeTwo => 0x08,
            G29Led::Red => 0x10,
            G29Led::All => 0x1F,
            G29Led::Other(val) => *val,
        }
    }
}

impl BitOr for G29Led {
    type Output = G29Led;

    fn bitor(self, other: Self) -> Self::Output {
        match (self, other) {
            (G29Led::None, _) => other,
            (_, G29Led::None) => self,
            _ => G29Led::Other(self.as_u8() | other.as_u8()),
        }
    }
}

#[derive(Debug)]
pub struct G29Options {
    pub debug: bool,
    pub range: u16,
    pub auto_center: [u8; 2],
    pub auto_center_enabled: bool,
}

impl Default for G29Options {
    fn default() -> Self {
        G29Options {
            auto_center: [0x07, 0xff],
            debug: false,
            range: 900,
            auto_center_enabled: true,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Memory {
    wheel: Wheel,
    shifter: Shifter,
    pedals: Pedals,
}

#[derive(Debug, Clone, Copy)]
pub struct Wheel {
    turn: u8,
    shift_left: u8,
    shift_right: u8,
    dpad: u8,
    button_x: u8,
    button_square: u8,
    button_triangle: u8,
    button_circle: u8,
    button_l2: u8,
    button_r2: u8,
    button_l3: u8,
    button_r3: u8,
    button_plus: u8,
    button_minus: u8,
    spinner: u8,
    button_spinner: u8,
    button_share: u8,
    button_option: u8,
    button_playstation: u8,
}

#[derive(Debug, Clone, Copy)]
pub struct Shifter {
    gear: u8,
}

#[derive(Debug, Clone, Copy)]
pub struct Pedals {
    gas: u8,
    brake: u8,
    clutch: u8,
}

impl Default for Memory {
    fn default() -> Self {
        Memory {
            wheel: Wheel::default(),
            shifter: Shifter::default(),
            pedals: Pedals::default(),
        }
    }
}

impl Default for Wheel {
    fn default() -> Self {
        Wheel {
            turn: 50_u8,
            shift_left: 0,
            shift_right: 0,
            dpad: 0,
            button_x: 0,
            button_square: 0,
            button_triangle: 0,
            button_circle: 0,
            button_l2: 0,
            button_r2: 0,
            button_l3: 0,
            button_r3: 0,
            button_plus: 0,
            button_minus: 0,
            spinner: 0,
            button_spinner: 0,
            button_share: 0,
            button_option: 0,
            button_playstation: 0,
        }
    }
}

impl Default for Shifter {
    fn default() -> Self {
        Shifter { gear: 0 }
    }
}

impl Default for Pedals {
    fn default() -> Self {
        Pedals {
            gas: 0,
            brake: 0,
            clutch: 0,
        }
    }
}

fn is_logitech_g29(device: &DeviceInfo) -> bool {
    device.vendor_id() == 1133
        && (device.product_string().unwrap() == "G29 Driving Force Racing Wheel"
            || device.product_id() == 49743)
        && (device.interface_number() == 0 || device.usage_page() == 1)
}

fn get_wheel_info(api: &HidApi) -> DeviceInfo {
    let list = api.device_list();

    list.into_iter()
        .find(|device| is_logitech_g29(device))
        .expect("No wheel found")
        .clone()
}

pub struct G29 {
    options: G29Options,
    prepend_write: bool,
    inner: Arc<RwLock<InnerG29>>,
    calibrated: bool,
}

struct InnerG29 {
    memory: Arc<RwLock<Memory>>,
    wheel: Mutex<hidapi::HidDevice>,
    prev_memory: Arc<RwLock<Memory>>,
    prev_data: Arc<RwLock<[u8; 12]>>,
    prev_led: Arc<RwLock<G29Led>>,
    reader_handle: Option<thread::JoinHandle<()>>,
}

impl G29 {
    pub fn new(options: G29Options) -> G29 {
        if options.debug {
            println!("userOptions -> {:?}", options);
        }
        // get wheel
        let api = HidApi::new().expect("Failed to initialize HID API");
        let wheel_info = get_wheel_info(&api);

        if wheel_info.path().is_empty() {
            if options.debug {
                println!("findWheel -> Oops, could not find a G29 Wheel. Is it plugged in?");
                exit(1);
            }
        } else if options.debug {
            println!("findWheel -> Found G29 Wheel at {:?}", wheel_info.path());
        }

        let wheel = wheel_info.open_device(&api).expect("Failed to open device");
        wheel
            .set_blocking_mode(false)
            .expect("Failed to set non-blocking mode");

        let prepend_write: bool = {
            match OS {
                "windows" => true,
                _ => false,
            }
        };

        G29 {
            options,
            prepend_write,
            calibrated: false,
            inner: Arc::new(RwLock::new(InnerG29 {
                memory: Arc::new(RwLock::new(Memory::default())),
                wheel: Mutex::new(wheel),
                prev_memory: Arc::new(RwLock::new(Memory::default())),
                prev_data: Arc::new(RwLock::new([0; 12])),
                prev_led: Arc::new(RwLock::new(G29Led::None)),
                reader_handle: None,
            })),
        }
    }

    pub fn throttle(&self) -> u8 {
        self.inner.read().unwrap().memory.read().unwrap().pedals.gas
    }

    pub fn brake(&self) -> u8 {
        self.inner
            .read()
            .unwrap()
            .memory
            .read()
            .unwrap()
            .pedals
            .brake
    }

    pub fn steering(&self) -> u8 {
        self.inner.read().unwrap().memory.read().unwrap().wheel.turn
    }

    pub fn steering_angle(&self) -> f32 {
        let percent = self.inner.read().unwrap().memory.read().unwrap().wheel.turn;

        // convert to degrees based on range option
        let degrees = (percent as f32 * self.options.range as f32) / 100_f32;

        if self.options.debug {
            println!("get_wheel_angle -> {}", degrees);
        }

        degrees
    }

    pub fn initialize(&mut self) {
        self.inner
            .write()
            .unwrap()
            .wheel
            .lock()
            .unwrap()
            .set_blocking_mode(false)
            .expect("Failed to set non-blocking mode");

        let mut data = [0 as u8; 12];
        let data_size = self
            .inner
            .read()
            .unwrap()
            .wheel
            .lock()
            .unwrap()
            .read(&mut data)
            .expect("connect -> Error reading from device.");

        self.force_off(0xf3);

        if data_size == 12 || self.calibrated {
            if self.options.debug {
                println!("connect -> Wheel already in high precision mode.");
            }
            self.listen(true);
        } else {
            if self.options.debug {
                println!("connect -> Initializing Wheel.");
            }

            if !self.calibrated {
                self.calibrate_wheel();
                self.calibrated = true;
            }

            self.listen(false);
        }
    }

    fn calibrate_wheel(&mut self) {
        // G29 Wheel init from - https://github.com/torvalds/linux/blob/master/drivers/hid/hid-lg4ff.c
        self.relay_os([0xf8, 0x0a, 0x00, 0x00, 0x00, 0x00, 0x00], "init_1");
        self.relay_os([0xf8, 0x09, 0x05, 0x01, 0x01, 0x00, 0x00], "init_2");

        sleep(Duration::from_secs(8));
    }

    fn listen(&mut self, ready: bool) {
        if !ready {
            let new_wheel = Mutex::new(
                get_wheel_info(&HidApi::new().unwrap())
                    .open_device(&HidApi::new().unwrap())
                    .unwrap(),
            );

            self.inner.write().unwrap().wheel = new_wheel;

            self.initialize();
            return;
        }

        self.set_range();
        self.auto_center();

        if self.options.debug {
            println!("listen -> Ready to listen for wheel events.");
        }

        // use thread to listen for wheel events and trigger events
        let local_self = self.inner.clone();
        let thread_handle = thread::spawn(move || loop {
            let buf = &mut [0 as u8; 12];
            let size_read = local_self
                .read()
                .unwrap()
                .wheel
                .lock()
                .unwrap()
                .read(buf)
                .expect("listen -> Error reading from device.");
            if size_read > 0 {
                let local_self_read = local_self.read().unwrap();
                let mut prev_data = local_self_read.prev_data.write().unwrap();
                if *prev_data != *buf {
                    let dif_positions = data_map::diff_positions(*prev_data, *buf);

                    *prev_data = *buf;
                    let mut memory = local_self_read.memory.write().unwrap();
                    let mut prev_memory = local_self_read.prev_memory.write().unwrap();

                    *prev_memory = *memory;
                    map_data(dif_positions, *buf, &mut memory);
                }
            }
        });

        self.inner.write().unwrap().reader_handle = Some(thread_handle);
    }

    fn auto_center(&self) {
        /*
            Set wheel autocentering based on existing options.
        */
        if self.options.auto_center_enabled {
            // auto-center on
            self.relay_os([0x14, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00], "auto_center on");
            self.relay_os(
                [
                    0xfe,
                    0x0d,
                    self.options.auto_center[0],
                    self.options.auto_center[0],
                    self.options.auto_center[1],
                    0x00,
                    0x00,
                ],
                "set_auto_center_force",
            );
        } else {
            // auto-center off
            self.relay_os(
                [0xf5, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
                "auto_center off",
            );
        }
    }

    fn set_range(&mut self) {
        /*
            Set wheel range.
        */
        if self.options.range < 40 {
            self.options.range = 40;
        }

        if self.options.range > 900 {
            self.options.range = 900;
        }

        let range1 = self.options.range & 0x00ff;
        let range2 = (self.options.range & 0xff00) >> 8;

        self.relay_os(
            [0xf8, 0x81, range1 as u8, range2 as u8, 0x00, 0x00, 0x00],
            "set_range",
        );
    }

    fn force_off(&self, slot: u8) {
        // turn off effects (except for auto-center)
        self.relay_os([slot, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00], "force_off");
    }

    fn relay_os(&self, data: [u8; 7], operation: &str) {
        /*
        Relay low level commands directly to the hardware after applying OS specific tweaks, if needed.
        @param  {Object}  data  Array of data to write. For example: [0xf8, 0x12, 0x1f, 0x00, 0x00, 0x00, 0x01]
        */

        let mut new_data: [u8; 8] = [0; 8];

        if self.prepend_write {
            // data.unshift(0x00)
            new_data = [
                0x00, data[0], data[1], data[2], data[3], data[4], data[5], data[6],
            ];
        }

        self.inner
            .read()
            .unwrap()
            .wheel
            .lock()
            .unwrap()
            .write(if self.prepend_write { &new_data } else { &data })
            .expect(
                format!(
                    "relay_os -> Error writing to device. Operation: {}",
                    operation
                )
                .as_str(),
            );
    }

    pub fn set_auto_center_force(&mut self, strength: u8, turning_multiplier: u8) {
        /*
            Set auto-center force.
        */
        // byte 3-4 is effect strength, 0x00 to 0x0f

        // byte 5 is the rate the effect strength rises as the wheel turns, 0x00 to 0xff
        self.options.auto_center = [strength, turning_multiplier];

        self.auto_center();
    }

    pub fn set_leds(&mut self, leds: G29Led) {
        /*
            Set the LED lights on the G29.
        */
        if leds != *self.inner.read().unwrap().prev_led.read().unwrap() {
            let data = [0xf8, 0x12, leds.as_u8(), 0x00, 0x00, 0x00, 0x01];

            self.relay_os(data, "set_leds");
            *self.inner.write().unwrap().prev_led.write().unwrap() = leds;
        }
    }

    pub fn force_friction(&self, mut left: u8, mut right: u8) {
        if left | right == 0 {
            self.force_off(2);
            return;
        }

        // sending manual relay() commands to the hardware seems to reveal a 0x00 through 0x07 range
        // 0x07 is the strongest friction and then 0x08 is no friction
        // friction ramps up again from 0x08 to 0x0F
        left = left * 7;
        right = right * 7;

        // the first "number" is for left rotation, the second for right rotation
        self.relay_os(
            [0x21, 0x02, left, 0x00, right, 0x00, 0x00],
            "force_friction",
        );
    } // forceFriction
}

impl Drop for G29 {
    fn drop(&mut self) {
        self.force_off(0xf3);
        self.set_leds(G29Led::None);
        self.force_friction(0, 0);
        self.options.auto_center = [0x00, 0x00];
        self.auto_center();
        let mut inner = self.inner.write().unwrap();

        inner
            .reader_handle
            .take()
            .and_then(|handle| handle.join().ok());
    }
}
