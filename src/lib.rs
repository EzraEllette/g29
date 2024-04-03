#![doc(html_root_url = "https://docs.rs/g29/1.0.0")]
use events::{Event, EventHandler, EventMap, HandlerFn};
use hidapi::{DeviceInfo, HidApi};

use std::{
    env::consts::OS,
    ops::BitOr,
    process::exit,
    sync::{atomic::AtomicBool, Arc, Mutex, RwLock},
    thread::{self, sleep},
    time::Duration,
};

pub mod events;
// pub mod state;
mod state;
// The size of the data frame that the G29 sends
const FRAME_SIZE: usize = 12;

///
/// DpadPosition
///
/// Represents the position of the Dpad on the G29
#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub enum DpadPosition {
    Up,
    TopRight,
    Right,
    BottomRight,
    Down,
    BottomLeft,
    Left,
    TopLeft,
    None,
}

///
/// Event
/// Events that can be triggered by the G29
///

type Frame = [u8; FRAME_SIZE];

///
/// GearSelector
///
/// Represents the gear selected on the G29
///
#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub enum GearSelector {
    Neutral = 0,
    First = 1,
    Second = 2,
    Third = 4,
    Fourth = 8,
    Fifth = 16,
    Sixth = 32,
    Reverse = 64,
}

///
/// Led
///
/// Represents the LED lights on the G29
///
#[repr(u8)]
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Led {
    None = 0x0,
    GreenOne = 0x01,
    GreenTwo = 0x02,
    OrangeOne = 0x04,
    OrangeTwo = 0x08,
    Red = 0x10,
    All = 0x1F,
    Other(u8),
}

impl Led {
    fn as_u8(&self) -> u8 {
        match self {
            Led::None => 0x0,
            Led::GreenOne => 0x01,
            Led::GreenTwo => 0x02,
            Led::OrangeOne => 0x04,
            Led::OrangeTwo => 0x08,
            Led::Red => 0x10,
            Led::All => 0x1F,
            Led::Other(val) => *val,
        }
    }
}

impl BitOr for Led {
    type Output = Led;

    fn bitor(self, other: Self) -> Self::Output {
        match (self, other) {
            (Led::None, _) => other,
            (_, Led::None) => self,
            _ => Led::Other(self.as_u8() | other.as_u8()),
        }
    }
}

static CONNECTED: AtomicBool = AtomicBool::new(false);

///
/// G29
/// Establishes a connection to the Logitech G29 Racing Wheel and provides methods to interact with it.
///
/// # Example
///
/// ```rust
/// use g29::{G29, Options, Led};
/// use std::time::Duration;
/// use std::thread::sleep;
///
///    let options = Options {
///      ..Default::default()
///   };
///
///   let g29 = G29::connect(options);
///
///   g29.set_leds(Led::All);
///
///   sleep(Duration::from_secs(5));
///
///   g29.disconnect();
/// ```
///
#[derive(Debug, Clone)]
pub struct G29 {
    options: Options,
    prepend_write: bool,
    calibrated: bool,
    inner: Arc<RwLock<InnerG29>>,
}

#[derive(Debug)]
struct InnerG29 {
    data: Arc<RwLock<Frame>>,
    reader_handle: Option<thread::JoinHandle<()>>,
    event_handlers: EventMap,
    wheel: Option<Mutex<hidapi::HidDevice>>,
}

///
/// The options that can be set when connecting to the G29
/// - debug: `bool` - Enable debug mode (default: `false`)
/// - range: `u16` - The range of the wheel (40 - 900) (default: `900`)
/// - auto_center: `[u8; 2]` - The auto center force and turning multiplier (default: `[0x07, 0xff]`)
/// - auto_center_enabled: `bool` - Enable auto centering (default: `true`)
///
/// # Example
///
/// ```rust
/// use g29::Options;
///
/// let options = Options {
///    range: 540,
///    auto_center_enabled: false,
///   ..Default::default()
/// };
/// ```
///
#[derive(Debug, PartialEq, Copy, Clone, Eq, Hash)]
pub struct Options {
    pub debug: bool,
    pub range: u16,
    pub auto_center: [u8; 2],
    pub auto_center_enabled: bool,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            auto_center: [0x07, 0xff],
            debug: false,
            range: 900,
            auto_center_enabled: true,
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

impl G29 {
    ///
    /// Connect to the G29 Racing Wheel
    ///
    pub fn connect(options: Options) -> G29 {
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

        let prepend_write: bool = { matches!(OS, "windows") };

        let mut g29 = G29 {
            options,
            prepend_write,
            calibrated: false,
            inner: Arc::new(RwLock::new(InnerG29 {
                wheel: Some(Mutex::new(wheel)),
                data: Arc::new(RwLock::new([0; FRAME_SIZE])),
                reader_handle: None,
                event_handlers: EventMap::new(),
            })),
        };
        CONNECTED.store(true, std::sync::atomic::Ordering::Release);

        g29.initialize();

        g29
    }

    fn initialize(&mut self) {
        self.inner
            .read()
            .unwrap()
            .wheel
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .set_blocking_mode(false)
            .expect("Failed to set non-blocking mode");

        let mut data = [0u8; FRAME_SIZE];
        let data_size = self
            .inner
            .read()
            .unwrap()
            .wheel
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .read(&mut data)
            .expect("connect -> Error reading from device.");

        self.force_off(0xf3);

        if data_size == FRAME_SIZE || self.calibrated {
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
            let new_wheel = get_wheel_info(&HidApi::new().unwrap())
                .open_device(&HidApi::new().unwrap())
                .unwrap();

            *self
                .inner
                .read()
                .unwrap()
                .wheel
                .as_ref()
                .unwrap()
                .lock()
                .unwrap() = new_wheel;

            self.initialize();
            return;
        }

        self.set_range();
        self.set_auto_center();

        if self.options.debug {
            println!("listen -> Ready to listen for wheel events.");
        }

        // use thread to listen for wheel events and trigger events
        let mut g29_clone = self.clone();
        let local_self = self.inner.clone();
        let thread_handle = thread::spawn(move || {
            while CONNECTED.load(std::sync::atomic::Ordering::Relaxed) {
                let mut new_data = [0u8; FRAME_SIZE];
                match local_self
                    .read()
                    .unwrap()
                    .wheel
                    .as_ref()
                    .unwrap()
                    .lock()
                    .unwrap()
                    .read(&mut new_data)
                {
                    Ok(size_read) if size_read == FRAME_SIZE => {
                        let local_self_write = local_self.read().unwrap();
                        let mut prev_data = local_self_write.data.write().unwrap();

                        if new_data == *prev_data {
                            continue;
                        }

                        local_self_write.event_handlers.trigger_events(
                            &prev_data,
                            &new_data,
                            &mut g29_clone,
                        );

                        *prev_data = new_data;
                    }
                    Ok(_) => {
                        if g29_clone.options.debug {
                            println!("listen -> Incomplete data read from device.");
                        }
                    }
                    Err(e) => {
                        if g29_clone.options.debug {
                            println!("listen -> Error reading from device: {:?}", e);
                        }
                    }
                };
            }
        });
        self.inner.write().unwrap().reader_handle = Some(thread_handle);
    }

    // fn auto_center_complex(
    //     &self,
    //     clockwise_angle: u8,
    //     counter_clockwise_angle: u8,
    //     clockwise_force: u8,
    //     counter_clockwise_force: u8,
    //     reverse: bool,
    //     centering_force: u8,
    // ) {
    //     if !self.options.auto_center_enabled {
    //         return;
    //     }

    //     self.force_off(0xf5);

    //     // auto-center on
    //     self.relay_os(
    //         [
    //             0xfc,
    //             0x01,
    //             clockwise_angle,
    //             counter_clockwise_angle,
    //             clockwise_force | counter_clockwise_force,
    //             reverse as u8,
    //             centering_force,
    //         ],
    //         "set_auto_center_complex",
    //     );
    // }

    fn set_auto_center(&self) {
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
            .as_ref()
            .expect("relay_os -> Wheel not found")
            .lock()
            .unwrap()
            .write(if self.prepend_write { &new_data } else { &data })
            .unwrap_or_else(|_| {
                panic!(
                    "relay_os -> Error writing to device. Operation: {}",
                    operation
                )
            });
    }

    /// Set auto-center force.
    ///
    /// # Arguments
    /// - `strength` - The strength of the auto-center force (**0x00** to **0x0f**)
    /// - `turning_multiplier` - The rate the effect strength rises as the wheel turns (**0x00** to **0xff**)
    ///
    /// # Example
    ///
    /// ```rust
    /// use g29::{G29, Options};
    ///
    ///   let options = Options {
    ///     ..Default::default()
    ///   };
    ///
    ///
    ///   let mut g29 = G29::connect(options);
    ///
    ///   g29.set_auto_center_force(0x0f, 0xff);
    ///
    ///   loop {}
    /// ```
    ///
    pub fn set_auto_center_force(&mut self, strength: u8, turning_multiplier: u8) {
        self.options.auto_center = [strength, turning_multiplier];

        self.set_auto_center();
    }

    /// Set the LED lights on the G29.
    /// # Arguments
    /// - `leds` - The LED lights to set
    /// # Example
    /// ```rust
    /// use g29::{G29, Options, Led};
    /// use std::time::Duration;
    /// use std::thread::sleep;
    ///
    ///   let options = Options {
    ///     ..Default::default()
    ///   };
    ///
    ///   let g29 = G29::connect(options);
    ///
    ///   loop {
    ///     g29.set_leds(Led::All);
    ///     sleep(Duration::from_secs(1));
    ///     g29.set_leds(Led::Red | Led::GreenOne);
    ///     sleep(Duration::from_secs(1));
    ///   }
    /// ````
    pub fn set_leds(&self, leds: Led) {
        /*
            Set the LED lights on the G29.
        */
        let data = [0xf8, 0x12, leds.as_u8(), 0x00, 0x00, 0x00, 0x01];

        self.relay_os(data, "set_leds");
    }

    /// Set the force feedback on the G29.
    /// # Arguments
    /// - `left` - The strength of the left motor (**0x00** to **0x07**)
    /// - `right` - The strength of the right motor (**0x00** to **0x07**)
    ///
    /// # Example
    /// ```rust
    /// use g29::{G29, Options};
    ///
    ///   let options = Options {
    ///     ..Default::default()
    ///   };
    ///
    ///   let mut g29 = G29::connect(options);
    ///
    ///   g29.force_friction(0x07, 0x07);
    ///
    ///   loop {}
    /// ```
    pub fn force_friction(&self, mut left: u8, mut right: u8) {
        if left | right == 0 {
            self.force_off(2);
            return;
        }

        left *= 7;
        right *= 7;

        self.relay_os(
            [0x21, 0x02, left, 0x00, right, 0x00, 0x00],
            "force_friction",
        );
    }

    /// Get the throttle value.
    ///  255 is depressed, 0 is fully pressed
    pub fn throttle(&self) -> u8 {
        state::throttle(&self.inner.read().unwrap().data.read().unwrap())
    }

    /// Get the brake value.
    ///  255 is depressed, 0 is fully pressed
    pub fn brake(&self) -> u8 {
        state::brake(&self.inner.read().unwrap().data.read().unwrap())
    }

    /// Get the steering value.
    /// 255 is fully right, 0 is fully left
    pub fn steering(&self) -> u8 {
        state::steering(&self.inner.read().unwrap().data.read().unwrap())
    }

    /// Get the fine steering value.
    /// 255 is fully right, 0 is fully left
    pub fn steering_fine(&self) -> u8 {
        state::steering_fine(&self.inner.read().unwrap().data.read().unwrap())
    }

    /// Get the Dpad position.
    /// # Example
    /// ```rust
    /// if g29.dpad() == DpadPosition::Top {
    ///    println!("Dpad is at the top");
    /// }
    /// ````
    pub fn dpad(&self) -> DpadPosition {
        state::dpad(&self.inner.read().unwrap().data.read().unwrap())
    }

    /// Returns `true` if the x button is pressed.
    pub fn x_button(&self) -> bool {
        state::x_button(&self.inner.read().unwrap().data.read().unwrap())
    }

    /// Returns true if the square button is pressed.
    pub fn square_button(&self) -> bool {
        state::square_button(&self.inner.read().unwrap().data.read().unwrap())
    }

    /// Returns true if the circle button is pressed.
    pub fn circle_button(&self) -> bool {
        state::circle_button(&self.inner.read().unwrap().data.read().unwrap())
    }

    /// Returns true if the triangle button is pressed.
    pub fn triangle_button(&self) -> bool {
        state::triangle_button(&self.inner.read().unwrap().data.read().unwrap())
    }

    /// returns true if the right shifter is pressed.
    pub fn right_shifter(&self) -> bool {
        state::right_shifter(&self.inner.read().unwrap().data.read().unwrap())
    }

    /// Returns true if the left shifter is pressed.
    pub fn left_shifter(&self) -> bool {
        state::left_shifter(&self.inner.read().unwrap().data.read().unwrap())
    }

    /// Returns true if the r2 button is pressed.
    pub fn r2_button(&self) -> bool {
        state::r2_button(&self.inner.read().unwrap().data.read().unwrap())
    }

    /// Returns true if the l2 button is pressed.
    pub fn l2_button(&self) -> bool {
        state::l2_button(&self.inner.read().unwrap().data.read().unwrap())
    }

    /// Returns true if the share button is pressed.
    pub fn share_button(&self) -> bool {
        state::share_button(&self.inner.read().unwrap().data.read().unwrap())
    }

    /// Returns true if the option button is pressed.
    pub fn option_button(&self) -> bool {
        state::options_button(&self.inner.read().unwrap().data.read().unwrap())
    }

    /// Returns true if the r3 button is pressed.
    pub fn r3_button(&self) -> bool {
        state::r3_button(&self.inner.read().unwrap().data.read().unwrap())
    }

    /// Returns true if the l3 button is pressed.
    pub fn l3_button(&self) -> bool {
        state::l3_button(&self.inner.read().unwrap().data.read().unwrap())
    }

    /// Get the gear selector position.
    ///
    /// # Example
    ///
    /// ```rust
    /// if g29.gear_selector() == GearSelector::First {
    ///    println!("Gear is in first");
    /// }
    /// ```
    ///
    pub fn gear_selector(&self) -> GearSelector {
        state::gear_selector(&self.inner.read().unwrap().data.read().unwrap())
    }

    /// Returns true if the plus button is pressed.
    pub fn plus_button(&self) -> bool {
        state::plus_button(&self.inner.read().unwrap().data.read().unwrap())
    }

    /// Returns true if the minus button is pressed.
    pub fn minus_button(&self) -> bool {
        state::minus_button(&self.inner.read().unwrap().data.read().unwrap())
    }

    /// Returns true if the spinner is rotating clockwise.
    pub fn spinner_right(&self) -> bool {
        state::spinner_right(&self.inner.read().unwrap().data.read().unwrap())
    }

    /// Returns true if the spinner is rotating counter-clockwise.
    pub fn spinner_left(&self) -> bool {
        state::spinner_left(&self.inner.read().unwrap().data.read().unwrap())
    }

    /// Returns true if the spinner button is pressed.
    pub fn spinner_button(&self) -> bool {
        state::spinner_button(&self.inner.read().unwrap().data.read().unwrap())
    }

    /// Returns true if the playstation button is pressed.
    pub fn playstation_button(&self) -> bool {
        state::playstation_button(&self.inner.read().unwrap().data.read().unwrap())
    }

    /// Returns the value of the clutch pedal.
    /// 255 is depressed, 0 is fully pressed
    pub fn clutch(&self) -> u8 {
        state::clutch(&self.inner.read().unwrap().data.read().unwrap())
    }

    /// Returns the value of the shifter x axis.
    pub fn shifter_x(&self) -> u8 {
        state::shifter_x(&self.inner.read().unwrap().data.read().unwrap())
    }

    /// Returns the value of the shifter y axis.
    pub fn shifter_y(&self) -> u8 {
        state::shifter_y(&self.inner.read().unwrap().data.read().unwrap())
    }

    /// Returns true if the shifter is pressed.
    pub fn shifter_pressed(&self) -> bool {
        state::shifter_pressed(&self.inner.read().unwrap().data.read().unwrap())
    }

    /// Disconnect from the G29.
    /// # Example
    /// ```rust
    /// use g29::{G29, Options};
    /// use std::time::Duration;
    /// use std::thread::sleep;
    ///
    ///   let options = Options {
    ///     ..Default::default()
    ///   };
    ///
    ///   let mut g29 = G29::connect(options);
    ///   
    ///   sleep(Duration::from_secs(5));
    ///   
    ///   g29.disconnect();
    /// ```
    pub fn disconnect(&mut self) {
        if !self.connected() {
            return;
        }

        self.force_off(0xf3);
        self.set_leds(Led::None);
        self.force_friction(0, 0);
        self.options.auto_center = [0x00, 0x00];
        self.set_auto_center();

        // set connected to false

        CONNECTED.store(false, std::sync::atomic::Ordering::Release);
        self.inner.write().unwrap().wheel = None;
        // join all threads
        if let Some(handle) = self.inner.write().unwrap().reader_handle.take() {
            handle.join().unwrap();
        }
    }

    pub fn connected(&self) -> bool {
        CONNECTED.load(std::sync::atomic::Ordering::Relaxed)
            && self.inner.read().unwrap().wheel.is_some()
    }

    ///
    /// Register an event handler for a specific event.
    /// # Arguments
    /// - `event` - The event to register the handler for
    /// - `handler` - The handler function
    /// # Example
    /// ```rust
    /// use g29::{G29, Options, Event, EventHandler};
    /// use std::time::Duration;
    /// use std::thread::sleep;
    ///
    ///  let options = Options {
    ///   ..Default::default()
    /// };
    ///
    /// let g29 = G29::connect(options);
    ///
    /// let handler: EventHandler = g29.register_event_handler(Event::Steering, |g29| {
    ///    println!("Steering: {}", g29.steering());
    /// });
    ///
    /// sleep(Duration::from_secs(5));
    ///
    /// g29.unregister_event_handler(handler);
    ///
    /// g29.disconnect();
    /// ```
    pub fn register_event_handler(&self, event: Event, handler: HandlerFn) -> Option<EventHandler> {
        self.inner
            .write()
            .unwrap()
            .event_handlers
            .insert(event, handler)
    }

    ///
    /// Unregister an event handler for a specific event.
    /// # Arguments
    /// - `event_handler` - The event handler to unregister
    /// # Example
    /// ```rust
    /// use g29::{G29, Options, Event, EventHandler};
    /// use std::time::Duration;
    /// use std::thread::sleep;
    ///
    ///  let options = Options {
    ///   ..Default::default()
    /// };
    ///
    /// let g29 = G29::connect(options);
    ///
    /// let handler: EventHandler = g29.register_event_handler(Event::Steering, |g29| {
    ///    println!("Steering: {}", g29.steering());
    /// });
    ///
    /// sleep(Duration::from_secs(5));
    ///
    /// g29.unregister_event_handler(handler);
    ///
    /// g29.disconnect();
    /// ```
    pub fn unregister_event_handler(&mut self, event_handler: EventHandler) {
        self.inner
            .write()
            .unwrap()
            .event_handlers
            .remove(event_handler);
    }
}
