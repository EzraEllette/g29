use hidapi::{DeviceInfo, HidApi};
#[cfg(feature = "events")]
use rayon::prelude::*;
use std::{
    collections::HashMap,
    env::consts::OS,
    ops::BitOr,
    process::exit,
    sync::{Arc, Mutex, RwLock},
    thread::{self, sleep},
    time::Duration,
    vec,
};

type HandlerFn = fn(g29: &mut G29);

pub mod state;

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
#[cfg(feature = "events")]
#[derive(Debug, PartialEq, Copy, Clone, Eq, Hash)]
pub enum Event {
    /// Steering wheel is turned
    Steering,
    /// Steering wheel is turned finely
    SteeringFine,
    /// Throttle changed
    Throttle,
    /// Brake changed
    Brake,
    /// Clutch changed
    Clutch,
    DpadUpPressed,
    DpadUpReleased,
    DpadTopRightPressed,
    DpadTopRightReleased,
    DpadRightPressed,
    DpadRightReleased,
    DpadBottomRightPressed,
    DpadBottomRightReleased,
    DpadBottomPressed,
    DpadBottomReleased,
    DpadBottomLeftPressed,
    DpadBottomLeftReleased,
    DpadLeftPressed,
    DpadLeftReleased,
    DpadTopLeftPressed,
    DpadTopLeftReleased,
    XButtonPressed,
    XButtonReleased,
    SquareButtonPressed,
    SquareButtonReleased,
    CircleButtonPressed,
    CircleButtonReleased,
    TriangleButtonPressed,
    TriangleButtonReleased,
    RightShifterPressed,
    RightShifterReleased,
    LeftShifterPressed,
    LeftShifterReleased,
    R2ButtonPressed,
    R2ButtonReleased,
    L2ButtonPressed,
    L2ButtonReleased,
    ShareButtonPressed,
    ShareButtonReleased,
    OptionButtonPressed,
    OptionButtonReleased,
    R3ButtonPressed,
    R3ButtonReleased,
    L3ButtonPressed,
    L3ButtonReleased,
    PlusButtonPressed,
    PlusButtonReleased,
    MinusButtonPressed,
    MinusButtonReleased,
    // Spinner is rotating right
    SpinnerRight,
    // Spinner is rotating left
    SpinnerLeft,
    SpinnerButtonPressed,
    SpinnerButtonReleased,
    PlaystationButtonPressed,
    PlaystationButtonReleased,
    /// Shifter X axis changed
    ShifterX,
    /// Shifter Y axis changed
    ShifterY,
    ShifterPressed,
    ShifterReleased,
    /// Gear selector changed
    GearChanged,
}

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

#[cfg(feature = "events")]
#[derive(Debug, Copy, Clone)]
pub struct EventHandler {
    pub id: usize,
    pub event: Event,
    handler: HandlerFn,
}

#[cfg(feature = "events")]
#[derive(Debug)]
struct EventHandlers {
    pub event: Event,
    pub next_id: usize,
    handlers: HashMap<usize, EventHandler>,
}

#[cfg(feature = "events")]
impl EventHandlers {
    fn new(event: Event) -> EventHandlers {
        EventHandlers {
            event,
            next_id: 0,
            handlers: HashMap::new(),
        }
    }

    pub fn insert(&mut self, handler: HandlerFn) -> Option<EventHandler> {
        let id = self.next_id;
        self.next_id += 1;

        let event_handler = EventHandler {
            id,
            event: self.event,
            handler,
        };

        self.handlers.insert(id, event_handler);

        Some(event_handler)
    }
}

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
    inner: Arc<RwLock<InnerG29>>,
    calibrated: bool,
    connected: bool,
}

#[derive(Debug)]
struct InnerG29 {
    data: Arc<RwLock<Frame>>,
    wheel: Mutex<hidapi::HidDevice>,
    reader_handle: Option<thread::JoinHandle<()>>,
    #[cfg(feature = "events")]
    event_handlers: RwLock<HashMap<Event, EventHandlers>>,
}

///
/// Options
/// The options that can be set when connecting to the G29
/// - debug: `bool` - Enable debug mode (default: false)
/// - range: `u16` - The range of the wheel (40 - 900) (default: 900)
/// - auto_center: `[u8; 2]` - The auto center force and turning multiplier (default: [0x07, 0xff])
/// - auto_center_enabled: `bool` - Enable auto centering (default: true)
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

#[cfg(feature = "events")]
fn different_indices(data1: &Frame, data2: &Frame) -> Vec<usize> {
    data1
        .iter()
        .enumerate()
        .filter_map(|(i, &x)| if x != data2[i] { Some(i) } else { None })
        .collect()
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
            connected: true,
            inner: Arc::new(RwLock::new(InnerG29 {
                wheel: Mutex::new(wheel),
                data: Arc::new(RwLock::new([0; FRAME_SIZE])),
                reader_handle: None,
                #[cfg(feature = "events")]
                event_handlers: RwLock::new(HashMap::new()),
            })),
        };

        g29.initialize();

        g29
    }

    fn initialize(&mut self) {
        self.inner
            .read()
            .unwrap()
            .wheel
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

            *self.inner.read().unwrap().wheel.lock().unwrap() = new_wheel;

            self.initialize();
            return;
        }

        self.set_range();
        self.set_auto_center();

        if self.options.debug {
            println!("listen -> Ready to listen for wheel events.");
        }

        // let options = self.options.clone();
        // let prepend_write = self.prepend_write;
        // let calibrated = self.calibrated;
        // use thread to listen for wheel events and trigger events
        let self_1 = self.clone();
        let local_self = self.inner.clone();
        let thread_handle = thread::spawn(move || loop {
            let mut new_data = [0u8; FRAME_SIZE];
            let size_read = local_self
                .read()
                .unwrap()
                .wheel
                .lock()
                .unwrap()
                .read(&mut new_data)
                .expect("listen -> Error reading from device.");
            if size_read == FRAME_SIZE {
                let local_self_write = local_self.read().unwrap();
                let mut prev_data = local_self_write.data.write().unwrap();

                if new_data == *prev_data {
                    continue;
                }

                #[cfg(feature = "events")]
                {
                    let events = local_self_write.events(&prev_data, &new_data);

                    events.par_iter().for_each(|event| {
                        local_self_write
                            .event_handlers
                            .read()
                            .unwrap()
                            .get(event)
                            .map(|event_handlers| {
                                event_handlers.handlers.values().for_each(|event_handler| {
                                    let ev_clone = *event_handler;
                                    let mut self_1 = self_1.clone();
                                    thread::spawn(move || {
                                        (ev_clone.handler)(&mut self_1);
                                    });
                                });

                                Some(())
                            });
                    });
                }
                *prev_data = new_data;
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
    pub fn set_leds(&mut self, leds: Led) {
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
    ///   g29.force_feedback(0x07, 0x07);
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
    /// 0 is
    pub fn throttle(&self) -> u8 {
        state::throttle(&self.inner.read().unwrap().data.read().unwrap())
    }

    /// Get the brake value.
    pub fn brake(&self) -> u8 {
        state::brake(&self.inner.read().unwrap().data.read().unwrap())
    }

    /// Get the steering value.
    pub fn steering(&self) -> u8 {
        state::steering(&self.inner.read().unwrap().data.read().unwrap())
    }

    /// Get the fine steering value.
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
        state::option_button(&self.inner.read().unwrap().data.read().unwrap())
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

    /// Returns the value of the clutch pedal. (0 - 255)
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
        self.force_off(0xf3);
        self.set_leds(Led::None);
        self.force_friction(0, 0);
        self.options.auto_center = [0x00, 0x00];
        self.set_auto_center();
        let mut inner = self.inner.write().unwrap();

        inner
            .reader_handle
            .take()
            .and_then(|handle| handle.join().ok());

        inner.reader_handle = None;
        self.connected = false;
    }

    pub fn connected(&self) -> bool {
        self.connected
    }

    #[cfg(feature = "events")]
    pub fn register_event_handler(&self, event: Event, handler: HandlerFn) -> Option<EventHandler> {
        self.inner
            .write()
            .unwrap()
            .event_handlers
            .write()
            .unwrap()
            .entry(event)
            .or_insert_with(|| EventHandlers::new(event))
            .insert(handler)
    }

    #[cfg(feature = "events")]
    pub fn unregister_event_handler(&mut self, event_handler: EventHandler) {
        self.inner
            .write()
            .unwrap()
            .event_handlers
            .write()
            .unwrap()
            .get_mut(&event_handler.event)
            .map(|handlers| {
                handlers.handlers.remove(&event_handler.id);
                Some(handlers)
            });
    }
}

impl Drop for G29 {
    fn drop(&mut self) {
        self.disconnect();
    }
}

#[cfg(feature = "events")]
impl InnerG29 {
    fn events(&self, prev_data: &Frame, new_data: &Frame) -> Vec<Event> {
        let different_indices = different_indices(prev_data, new_data);

        if different_indices.is_empty() {
            return vec![];
        }

        let events_to_trigger = different_indices.par_iter().flat_map(|index| {
            let mut events_to_trigger = vec![];
            match index {
                0 => {
                    events_to_trigger.extend(self.dpad_events(prev_data, new_data));
                    events_to_trigger.extend(self.shape_button_events(prev_data, new_data));
                }
                1 => events_to_trigger.extend(self.data1_button_events(prev_data, new_data)),
                2 => {
                    events_to_trigger.extend(self.gear_selector_events(prev_data, new_data));
                    events_to_trigger.extend(self.plus_button_events(prev_data, new_data));
                }
                3 => events_to_trigger.extend(self.data3_button_events(prev_data, new_data)),
                4 | 5 => events_to_trigger.extend(self.steering_events(prev_data, new_data)),
                6 => events_to_trigger.extend(self.throttle_event(prev_data, new_data)),
                7 => events_to_trigger.extend(self.brake_event(prev_data, new_data)),
                8 => events_to_trigger.extend(self.clutch_event(prev_data, new_data)),
                9 => events_to_trigger.extend(self.shifter_x_event(prev_data, new_data)),
                10 => events_to_trigger.extend(self.shifter_y_event(prev_data, new_data)),
                11 => events_to_trigger.extend(self.shifter_events(prev_data, new_data)),
                _ => {}
            };
            events_to_trigger
        });

        events_to_trigger.collect()
    }

    fn dpad_events(&self, prev_data: &Frame, new_data: &Frame) -> Vec<Event> {
        let prev_dpad = state::dpad(prev_data);
        let new_dpad = state::dpad(new_data);
        if prev_dpad == new_dpad {
            return vec![];
        }

        let mut events = vec![];

        // which dpad is pressed
        match new_dpad {
            DpadPosition::Up => events.push(Event::DpadUpPressed),
            DpadPosition::TopRight => events.push(Event::DpadTopRightPressed),
            DpadPosition::Right => events.push(Event::DpadRightPressed),
            DpadPosition::BottomRight => events.push(Event::DpadBottomRightPressed),
            DpadPosition::Down => events.push(Event::DpadBottomPressed),
            DpadPosition::BottomLeft => events.push(Event::DpadBottomLeftPressed),
            DpadPosition::Left => events.push(Event::DpadLeftPressed),
            DpadPosition::TopLeft => events.push(Event::DpadTopLeftPressed),
            _ => {}
        };

        // which dpad is released
        match prev_dpad {
            DpadPosition::Up => events.push(Event::DpadUpReleased),
            DpadPosition::TopRight => events.push(Event::DpadTopRightReleased),
            DpadPosition::Right => events.push(Event::DpadRightReleased),
            DpadPosition::BottomRight => events.push(Event::DpadBottomRightReleased),
            DpadPosition::Down => events.push(Event::DpadBottomReleased),
            DpadPosition::BottomLeft => events.push(Event::DpadBottomLeftReleased),
            DpadPosition::Left => events.push(Event::DpadLeftReleased),
            DpadPosition::TopLeft => events.push(Event::DpadTopLeftReleased),
            _ => {}
        };

        events
    }

    fn shape_button_events(&self, prev_data: &Frame, new_data: &Frame) -> Vec<Event> {
        let mut events = vec![];

        let prev_x_button = state::x_button(prev_data);
        let new_x_button = state::x_button(new_data);
        if prev_x_button != new_x_button {
            if new_x_button {
                events.push(Event::XButtonPressed);
            } else {
                events.push(Event::XButtonReleased);
            }
        }

        let prev_square_button = state::square_button(prev_data);
        let new_square_button = state::square_button(new_data);
        if prev_square_button != new_square_button {
            if new_square_button {
                events.push(Event::SquareButtonPressed);
            } else {
                events.push(Event::SquareButtonReleased);
            }
        }

        let prev_circle_button = state::circle_button(prev_data);
        let new_circle_button = state::circle_button(new_data);
        if prev_circle_button != new_circle_button {
            if new_circle_button {
                events.push(Event::CircleButtonPressed);
            } else {
                events.push(Event::CircleButtonReleased);
            }
        }

        let prev_triangle_button = state::triangle_button(prev_data);
        let new_triangle_button = state::triangle_button(new_data);
        if prev_triangle_button != new_triangle_button {
            if new_triangle_button {
                events.push(Event::TriangleButtonPressed);
            } else {
                events.push(Event::TriangleButtonReleased);
            }
        }

        events
    }

    fn data1_button_events(&self, prev_data: &Frame, new_data: &Frame) -> Vec<Event> {
        let mut events = vec![];

        let prev_right_shifter = state::right_shifter(prev_data);
        let new_right_shifter = state::right_shifter(new_data);
        if prev_right_shifter != new_right_shifter {
            if new_right_shifter {
                events.push(Event::RightShifterPressed);
            } else {
                events.push(Event::RightShifterReleased);
            }
        }

        let prev_left_shifter = state::left_shifter(prev_data);
        let new_left_shifter = state::left_shifter(new_data);
        if prev_left_shifter != new_left_shifter {
            if new_left_shifter {
                events.push(Event::LeftShifterPressed);
            } else {
                events.push(Event::LeftShifterReleased);
            }
        }

        let prev_r2_button = state::r2_button(prev_data);
        let new_r2_button = state::r2_button(new_data);
        if prev_r2_button != new_r2_button {
            if new_r2_button {
                events.push(Event::R2ButtonPressed);
            } else {
                events.push(Event::R2ButtonReleased);
            }
        }

        let prev_l2_button = state::l2_button(prev_data);
        let new_l2_button = state::l2_button(new_data);
        if prev_l2_button != new_l2_button {
            if new_l2_button {
                events.push(Event::L2ButtonPressed);
            } else {
                events.push(Event::L2ButtonReleased);
            }
        }

        let prev_share_button = state::share_button(prev_data);
        let new_share_button = state::share_button(new_data);
        if prev_share_button != new_share_button {
            if new_share_button {
                events.push(Event::ShareButtonPressed);
            } else {
                events.push(Event::ShareButtonReleased);
            }
        }

        let prev_option_button = state::option_button(prev_data);
        let new_option_button = state::option_button(new_data);
        if prev_option_button != new_option_button {
            if new_option_button {
                events.push(Event::OptionButtonPressed);
            } else {
                events.push(Event::OptionButtonReleased);
            }
        }

        let prev_r3_button = state::r3_button(prev_data);
        let new_r3_button = state::r3_button(new_data);
        if prev_r3_button != new_r3_button {
            if new_r3_button {
                events.push(Event::R3ButtonPressed);
            } else {
                events.push(Event::R3ButtonReleased);
            }
        }

        let prev_l3_button = state::l3_button(prev_data);
        let new_l3_button = state::l3_button(new_data);
        if prev_l3_button != new_l3_button {
            if new_l3_button {
                events.push(Event::L3ButtonPressed);
            } else {
                events.push(Event::L3ButtonReleased);
            }
        }

        events
    }

    fn gear_selector_events(&self, prev_data: &Frame, new_data: &Frame) -> Vec<Event> {
        let prev_gear_selector = state::gear_selector(prev_data);
        let new_gear_selector = state::gear_selector(new_data);

        if prev_gear_selector == new_gear_selector {
            return vec![];
        }

        let mut events = vec![];

        match new_gear_selector {
            GearSelector::Neutral => events.push(Event::GearChanged),
            GearSelector::First => events.push(Event::GearChanged),
            GearSelector::Second => events.push(Event::GearChanged),
            GearSelector::Third => events.push(Event::GearChanged),
            GearSelector::Fourth => events.push(Event::GearChanged),
            GearSelector::Fifth => events.push(Event::GearChanged),
            GearSelector::Sixth => events.push(Event::GearChanged),
            GearSelector::Reverse => events.push(Event::GearChanged),
        }

        events
    }

    fn plus_button_events(&self, prev_data: &Frame, new_data: &Frame) -> Vec<Event> {
        let prev_plus_button = state::plus_button(prev_data);
        let new_plus_button = state::plus_button(new_data);
        if prev_plus_button == new_plus_button {
            vec![]
        } else if new_plus_button {
            vec![Event::PlusButtonPressed]
        } else {
            vec![Event::PlusButtonReleased]
        }
    }

    fn data3_button_events(&self, prev_data: &Frame, new_data: &Frame) -> Vec<Event> {
        /*
           minus_button
           spinner_right
           spinner_left
           spinner_button
           playstation_button
        */

        let mut events = vec![];

        let prev_minus_button = state::minus_button(prev_data);
        let new_minus_button = state::minus_button(new_data);
        if prev_minus_button != new_minus_button {
            if new_minus_button {
                events.push(Event::MinusButtonPressed);
            } else {
                events.push(Event::MinusButtonReleased);
            }
        }

        let prev_spinner_right = state::spinner_right(prev_data);
        let new_spinner_right = state::spinner_right(new_data);
        if prev_spinner_right != new_spinner_right && new_spinner_right {
            events.push(Event::SpinnerRight);
        }

        let prev_spinner_left = state::spinner_left(prev_data);
        let new_spinner_left = state::spinner_left(new_data);
        if prev_spinner_left != new_spinner_left && new_spinner_left {
            events.push(Event::SpinnerLeft);
        }

        let prev_spinner_button = state::spinner_button(prev_data);
        let new_spinner_button = state::spinner_button(new_data);
        if prev_spinner_button != new_spinner_button {
            if new_spinner_button {
                events.push(Event::SpinnerButtonPressed);
            } else {
                events.push(Event::SpinnerButtonReleased);
            }
        }

        let prev_playstation_button = state::playstation_button(prev_data);
        let new_playstation_button = state::playstation_button(new_data);
        if prev_playstation_button != new_playstation_button {
            if new_playstation_button {
                events.push(Event::PlaystationButtonPressed);
            } else {
                events.push(Event::PlaystationButtonReleased);
            }
        }

        events
    }

    fn steering_events(&self, prev_data: &Frame, new_data: &Frame) -> Vec<Event> {
        let mut events = vec![];

        let prev_steering = state::steering(prev_data);
        let new_steering = state::steering(new_data);
        if prev_steering != new_steering {
            events.push(Event::Steering);
        }

        let prev_steering_fine = state::steering_fine(prev_data);
        let new_steering_fine = state::steering_fine(new_data);
        if prev_steering_fine != new_steering_fine {
            events.push(Event::SteeringFine);
        }

        events
    }

    fn throttle_event(&self, prev_data: &Frame, new_data: &Frame) -> Vec<Event> {
        let prev_throttle = state::throttle(prev_data);
        let new_throttle = state::throttle(new_data);

        if prev_throttle == new_throttle {
            vec![]
        } else {
            vec![Event::Throttle]
        }
    }

    fn brake_event(&self, prev_data: &Frame, new_data: &Frame) -> Vec<Event> {
        let prev_brake = state::brake(prev_data);
        let new_brake = state::brake(new_data);

        if prev_brake == new_brake {
            vec![]
        } else {
            vec![Event::Brake]
        }
    }

    fn clutch_event(&self, prev_data: &Frame, new_data: &Frame) -> Vec<Event> {
        let prev_clutch = state::clutch(prev_data);
        let new_clutch = state::clutch(new_data);

        if prev_clutch == new_clutch {
            vec![]
        } else {
            vec![Event::Clutch]
        }
    }

    fn shifter_x_event(&self, prev_data: &Frame, new_data: &Frame) -> Vec<Event> {
        let prev_shifter_x = state::shifter_x(prev_data);
        let new_shifter_x = state::shifter_x(new_data);

        if prev_shifter_x == new_shifter_x {
            vec![]
        } else {
            vec![Event::ShifterX]
        }
    }

    fn shifter_y_event(&self, prev_data: &Frame, new_data: &Frame) -> Vec<Event> {
        let prev_shifter_y = state::shifter_y(prev_data);
        let new_shifter_y = state::shifter_y(new_data);

        if prev_shifter_y == new_shifter_y {
            vec![]
        } else {
            vec![Event::ShifterY]
        }
    }

    fn shifter_events(&self, prev_data: &Frame, new_data: &Frame) -> Vec<Event> {
        let prev_shifter_pressed = state::shifter_pressed(prev_data);
        let new_shifter_pressed = state::shifter_pressed(new_data);

        if prev_shifter_pressed == new_shifter_pressed {
            vec![]
        } else if new_shifter_pressed {
            vec![Event::ShifterPressed]
        } else {
            vec![Event::ShifterReleased]
        }
    }
}
