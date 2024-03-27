use crate::{DpadPosition, GearSelector};

pub fn throttle(data: &[u8; 12]) -> u8 {
    data[6]
}

/// Get the brake value.
pub fn brake(data: &[u8; 12]) -> u8 {
    data[7]
}

/// Get the steering value.
pub fn steering(data: &[u8; 12]) -> u8 {
    data[5]
}

/// Get the fine steering value.
pub fn steering_fine(data: &[u8; 12]) -> u8 {
    data[4]
}

/// Get the Dpad position.
/// # Example
/// ```rust
/// if g29.dpad() == DpadPosition::Top {
///    println!("Dpad is at the top");
/// }
/// ````
pub fn dpad(data: &[u8; 12]) -> DpadPosition {
    match data[0] & 15 {
        0 => DpadPosition::Up,
        1 => DpadPosition::TopRight,
        2 => DpadPosition::Right,
        3 => DpadPosition::BottomRight,
        4 => DpadPosition::Down,
        5 => DpadPosition::BottomLeft,
        6 => DpadPosition::Left,
        7 => DpadPosition::TopLeft,
        _ => DpadPosition::None,
    }
}

/// Returns `true` if the x button is pressed.
pub fn x_button(data: &[u8; 12]) -> bool {
    data[0] & 16 == 16
}

/// Returns true if the square button is pressed.
pub fn square_button(data: &[u8; 12]) -> bool {
    data[0] & 32 == 32
}

/// Returns true if the circle button is pressed.
pub fn circle_button(data: &[u8; 12]) -> bool {
    data[0] & 64 == 64
}

/// Returns true if the triangle button is pressed.
pub fn triangle_button(data: &[u8; 12]) -> bool {
    data[0] & 128 == 128
}

/// returns true if the right shifter is pressed.
pub fn right_shifter(data: &[u8; 12]) -> bool {
    data[1] & 1 == 1
}

/// Returns true if the left shifter is pressed.
pub fn left_shifter(data: &[u8; 12]) -> bool {
    data[1] & 2 == 2
}

/// Returns true if the r2 button is pressed.
pub fn r2_button(data: &[u8; 12]) -> bool {
    data[1] & 4 == 4
}

/// Returns true if the l2 button is pressed.
pub fn l2_button(data: &[u8; 12]) -> bool {
    data[1] & 8 == 8
}

/// Returns true if the share button is pressed.
pub fn share_button(data: &[u8; 12]) -> bool {
    data[1] & 16 == 16
}

/// Returns true if the option button is pressed.
pub fn option_button(data: &[u8; 12]) -> bool {
    data[1] & 32 == 32
}

/// Returns true if the r3 button is pressed.
pub fn r3_button(data: &[u8; 12]) -> bool {
    data[1] & 64 == 64
}

/// Returns true if the l3 button is pressed.
pub fn l3_button(data: &[u8; 12]) -> bool {
    data[1] & 128 == 128
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
pub fn gear_selector(data: &[u8; 12]) -> GearSelector {
    match data[2] & 127 {
        0 => GearSelector::Neutral,
        1 => GearSelector::First,
        2 => GearSelector::Second,
        4 => GearSelector::Third,
        8 => GearSelector::Fourth,
        16 => GearSelector::Fifth,
        32 => GearSelector::Sixth,
        64 => GearSelector::Reverse,
        _ => GearSelector::Neutral,
    }
}

/// Returns true if the plus button is pressed.
pub fn plus_button(data: &[u8; 12]) -> bool {
    data[2] & 128 == 128
}

/// Returns true if the minus button is pressed.
pub fn minus_button(data: &[u8; 12]) -> bool {
    data[3] & 1 == 1
}

/// Returns true if the spinner is rotating clockwise.
pub fn spinner_right(data: &[u8; 12]) -> bool {
    data[3] & 2 == 2
}

/// Returns true if the spinner is rotating counter-clockwise.
pub fn spinner_left(data: &[u8; 12]) -> bool {
    data[3] & 4 == 4
}

/// Returns true if the spinner button is pressed.
pub fn spinner_button(data: &[u8; 12]) -> bool {
    data[3] & 8 == 8
}

/// Returns true if the playstation button is pressed.
pub fn playstation_button(data: &[u8; 12]) -> bool {
    data[3] & 16 == 16
}

/// Returns the value of the clutch pedal. (0 - 255)
pub fn clutch(data: &[u8; 12]) -> u8 {
    data[8]
}

/// Returns the value of the shifter x axis.
pub fn shifter_x(data: &[u8; 12]) -> u8 {
    data[9]
}

/// Returns the value of the shifter y axis.
pub fn shifter_y(data: &[u8; 12]) -> u8 {
    data[10]
}

/// Returns true if the shifter is pressed.
pub fn shifter_pressed(data: &[u8; 12]) -> bool {
    data[11] == 1
}
