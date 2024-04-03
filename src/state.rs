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
/// if g29.dpad() == DpadPosition::Up {
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
pub fn options_button(data: &[u8; 12]) -> bool {
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

#[cfg(test)]
mod tests {
    fn get_test_state() -> [u8; 12] {
        [
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ]
    }

    #[test]
    fn test_throttle() {
        let mut state = get_test_state();
        state[6] = 0;

        assert_eq!(crate::state::throttle(&state), 0);

        state[6] = 255;
        assert_eq!(crate::state::throttle(&state), 255);

        state[6] = 128;
        assert_eq!(crate::state::throttle(&state), 128);
    }

    #[test]
    fn test_brake() {
        let mut state = get_test_state();
        assert_eq!(crate::state::brake(&state), 0);

        state[7] = 255;
        assert_eq!(crate::state::brake(&state), 255);

        state[7] = 128;
        assert_eq!(crate::state::brake(&state), 128);
    }

    #[test]
    fn test_steering() {
        let mut state = get_test_state();
        assert_eq!(crate::state::steering(&state), 0);

        state[5] = 255;
        assert_eq!(crate::state::steering(&state), 255);

        state[5] = 128;
        assert_eq!(crate::state::steering(&state), 128);
    }

    #[test]
    fn test_steering_fine() {
        let mut state = get_test_state();
        assert_eq!(crate::state::steering_fine(&state), 0);

        state[4] = 255;
        assert_eq!(crate::state::steering_fine(&state), 255);

        state[4] = 128;
        assert_eq!(crate::state::steering_fine(&state), 128);
    }

    #[test]
    fn test_dpad() {
        let mut state = get_test_state();
        state[0] |= 240;
        assert_eq!(crate::state::dpad(&state), crate::DpadPosition::Up);

        state[0] = 240 | 1;
        assert_eq!(crate::state::dpad(&state), crate::DpadPosition::TopRight);

        state[0] = 240 | 2;
        assert_eq!(crate::state::dpad(&state), crate::DpadPosition::Right);

        state[0] = 240 | 3;
        assert_eq!(crate::state::dpad(&state), crate::DpadPosition::BottomRight);

        state[0] = 240 | 4;
        assert_eq!(crate::state::dpad(&state), crate::DpadPosition::Down);

        state[0] = 240 | 5;
        assert_eq!(crate::state::dpad(&state), crate::DpadPosition::BottomLeft);

        state[0] = 240 | 6;
        assert_eq!(crate::state::dpad(&state), crate::DpadPosition::Left);

        state[0] = 240 | 7;
        assert_eq!(crate::state::dpad(&state), crate::DpadPosition::TopLeft);

        state[0] = 240 | 8;
        assert_eq!(crate::state::dpad(&state), crate::DpadPosition::None);
    }

    #[test]
    fn test_x_button() {
        let mut state = get_test_state();
        state[0] |= 240;
        assert!(crate::state::x_button(&state));

        state[0] ^= 16;
        assert!(!crate::state::x_button(&state));
    }

    #[test]
    fn test_square_button() {
        let mut state = get_test_state();
        state[0] |= 240;

        assert!(crate::state::square_button(&state));

        state[0] ^= 32;
        assert!(!crate::state::square_button(&state));
    }

    #[test]
    fn test_circle_button() {
        let mut state = get_test_state();
        state[0] |= 240;

        assert!(crate::state::circle_button(&state));

        state[0] ^= 64;
        assert!(!crate::state::circle_button(&state));
    }

    #[test]
    fn test_triangle_button() {
        let mut state = get_test_state();
        state[0] |= 240;

        assert!(crate::state::triangle_button(&state));

        state[0] ^= 128;
        assert!(!crate::state::triangle_button(&state));
    }

    #[test]
    fn test_right_shifter() {
        let mut state = get_test_state();
        state[1] |= 15;

        assert!(crate::state::right_shifter(&state));

        state[1] ^= 1;
        assert!(!crate::state::right_shifter(&state));
    }

    #[test]
    fn test_left_shifter() {
        let mut state = get_test_state();
        state[1] |= 15;

        assert!(crate::state::left_shifter(&state));

        state[1] ^= 2;
        assert!(!crate::state::left_shifter(&state));
    }

    #[test]
    fn test_r2_button() {
        let mut state = get_test_state();
        state[1] |= 15;

        assert!(crate::state::r2_button(&state));

        state[1] ^= 4;
        assert!(!crate::state::r2_button(&state));
    }

    #[test]
    fn test_l2_button() {
        let mut state = get_test_state();
        state[1] |= 15;

        assert!(crate::state::l2_button(&state));

        state[1] ^= 8;
        assert!(!crate::state::l2_button(&state));
    }

    #[test]
    fn test_share_button() {
        let mut state = get_test_state();
        state[1] |= 240;

        assert!(crate::state::share_button(&state));

        state[1] ^= 16;
        assert!(!crate::state::share_button(&state));
    }

    #[test]
    fn test_options_button() {
        let mut state = get_test_state();
        state[1] |= 240;

        assert!(crate::state::options_button(&state));

        state[1] ^= 32;
        assert!(!crate::state::options_button(&state));
    }

    #[test]
    fn test_r3_button() {
        let mut state = get_test_state();
        state[1] |= 240;

        assert!(crate::state::r3_button(&state));

        state[1] ^= 64;
        assert!(!crate::state::r3_button(&state));
    }

    #[test]
    fn test_l3_button() {
        let mut state = get_test_state();
        state[1] |= 240;

        assert!(crate::state::l3_button(&state));

        state[1] ^= 128;
        assert!(!crate::state::l3_button(&state));
    }

    #[test]
    fn test_gear_selector() {
        let mut state = get_test_state();
        state[2] |= 128;

        assert_eq!(
            crate::state::gear_selector(&state),
            crate::GearSelector::Neutral
        );

        state[2] = 128 | 1;
        assert_eq!(
            crate::state::gear_selector(&state),
            crate::GearSelector::First
        );

        state[2] = 128 | 2;
        assert_eq!(
            crate::state::gear_selector(&state),
            crate::GearSelector::Second
        );

        state[2] = 128 | 4;
        assert_eq!(
            crate::state::gear_selector(&state),
            crate::GearSelector::Third
        );

        state[2] = 128 | 8;
        assert_eq!(
            crate::state::gear_selector(&state),
            crate::GearSelector::Fourth
        );

        state[2] = 128 | 16;
        assert_eq!(
            crate::state::gear_selector(&state),
            crate::GearSelector::Fifth
        );

        state[2] = 128 | 32;
        assert_eq!(
            crate::state::gear_selector(&state),
            crate::GearSelector::Sixth
        );

        state[2] = 128 | 64;
        assert_eq!(
            crate::state::gear_selector(&state),
            crate::GearSelector::Reverse
        );
    }

    #[test]
    fn test_plus_button() {
        let mut state = get_test_state();
        state[2] |= 128;

        assert!(crate::state::plus_button(&state));

        state[2] ^= 128;
        assert!(!crate::state::plus_button(&state));
    }

    #[test]
    fn test_minus_button() {
        let mut state = get_test_state();
        state[3] |= 15;

        assert!(crate::state::minus_button(&state));

        state[3] ^= 1;
        assert!(!crate::state::minus_button(&state));
    }

    #[test]
    fn test_spinner_right() {
        let mut state = get_test_state();
        state[3] |= 15;

        assert!(crate::state::spinner_right(&state));

        state[3] ^= 2;
        assert!(!crate::state::spinner_right(&state));
    }

    #[test]
    fn test_spinner_left() {
        let mut state = get_test_state();
        state[3] |= 15;

        assert!(crate::state::spinner_left(&state));

        state[3] ^= 4;
        assert!(!crate::state::spinner_left(&state));
    }

    #[test]
    fn test_spinner_button() {
        let mut state = get_test_state();
        state[3] |= 15;

        assert!(crate::state::spinner_button(&state));

        state[3] ^= 8;
        assert!(!crate::state::spinner_button(&state));
    }

    #[test]
    fn test_playstation_button() {
        let mut state = get_test_state();
        state[3] |= 240;

        assert!(crate::state::playstation_button(&state));

        state[3] ^= 16;
        assert!(!crate::state::playstation_button(&state));
    }

    #[test]
    fn test_clutch() {
        let mut state = get_test_state();
        assert_eq!(crate::state::clutch(&state), 0);

        state[8] = 255;
        assert_eq!(crate::state::clutch(&state), 255);

        state[8] = 128;
        assert_eq!(crate::state::clutch(&state), 128);
    }

    #[test]
    fn test_shifter_x() {
        let mut state = get_test_state();
        assert_eq!(crate::state::shifter_x(&state), 0);

        state[9] = 255;
        assert_eq!(crate::state::shifter_x(&state), 255);

        state[9] = 128;
        assert_eq!(crate::state::shifter_x(&state), 128);
    }

    #[test]
    fn test_shifter_y() {
        let mut state = get_test_state();
        assert_eq!(crate::state::shifter_y(&state), 0);

        state[10] = 255;
        assert_eq!(crate::state::shifter_y(&state), 255);

        state[10] = 128;
        assert_eq!(crate::state::shifter_y(&state), 128);
    }

    #[test]
    fn test_shifter_pressed() {
        let mut state = get_test_state();
        assert!(!crate::state::shifter_pressed(&state));

        state[11] = 1;
        assert!(crate::state::shifter_pressed(&state));
    }
}
