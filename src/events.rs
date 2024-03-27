use rayon::prelude::*;
use std::collections::HashMap;

use crate::{state, DpadPosition, Frame, GearSelector, G29};

pub type HandlerFn = fn(g29: &mut G29);

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

#[derive(Debug, Copy, Clone)]
pub struct EventHandler {
    pub id: usize,
    pub event: Event,
    pub handler: HandlerFn,
}

#[derive(Debug)]
pub struct EventHandlers {
    pub event: Event,
    pub next_id: usize,
    pub handlers: HashMap<usize, EventHandler>,
}

impl EventHandlers {
    pub fn new(event: Event) -> EventHandlers {
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

fn different_indices(data1: &Frame, data2: &Frame) -> Vec<usize> {
    data1
        .iter()
        .enumerate()
        .filter_map(|(i, &x)| if x != data2[i] { Some(i) } else { None })
        .collect()
}

pub fn events(prev_data: &Frame, new_data: &Frame) -> Vec<Event> {
    let different_indices = different_indices(prev_data, new_data);

    if different_indices.is_empty() {
        return vec![];
    }

    let events_to_trigger = different_indices.par_iter().flat_map(|index| {
        let mut events_to_trigger = vec![];
        match index {
            0 => {
                events_to_trigger.extend(dpad_events(prev_data, new_data));
                events_to_trigger.extend(shape_button_events(prev_data, new_data));
            }
            1 => events_to_trigger.extend(data1_button_events(prev_data, new_data)),
            2 => {
                events_to_trigger.extend(gear_selector_events(prev_data, new_data));
                events_to_trigger.extend(plus_button_events(prev_data, new_data));
            }
            3 => events_to_trigger.extend(data3_button_events(prev_data, new_data)),
            4 | 5 => events_to_trigger.extend(steering_events(prev_data, new_data)),
            6 => events_to_trigger.extend(throttle_event(prev_data, new_data)),
            7 => events_to_trigger.extend(brake_event(prev_data, new_data)),
            8 => events_to_trigger.extend(clutch_event(prev_data, new_data)),
            9 => events_to_trigger.extend(shifter_x_event(prev_data, new_data)),
            10 => events_to_trigger.extend(shifter_y_event(prev_data, new_data)),
            11 => events_to_trigger.extend(shifter_events(prev_data, new_data)),
            _ => {}
        };
        events_to_trigger
    });

    events_to_trigger.collect()
}

pub fn dpad_events(prev_data: &Frame, new_data: &Frame) -> Vec<Event> {
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

pub fn shape_button_events(prev_data: &Frame, new_data: &Frame) -> Vec<Event> {
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

pub fn data1_button_events(prev_data: &Frame, new_data: &Frame) -> Vec<Event> {
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

pub fn gear_selector_events(prev_data: &Frame, new_data: &Frame) -> Vec<Event> {
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

pub fn plus_button_events(prev_data: &Frame, new_data: &Frame) -> Vec<Event> {
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

pub fn data3_button_events(prev_data: &Frame, new_data: &Frame) -> Vec<Event> {
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

pub fn steering_events(prev_data: &Frame, new_data: &Frame) -> Vec<Event> {
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

pub fn throttle_event(prev_data: &Frame, new_data: &Frame) -> Vec<Event> {
    let prev_throttle = state::throttle(prev_data);
    let new_throttle = state::throttle(new_data);

    if prev_throttle == new_throttle {
        vec![]
    } else {
        vec![Event::Throttle]
    }
}

pub fn brake_event(prev_data: &Frame, new_data: &Frame) -> Vec<Event> {
    let prev_brake = state::brake(prev_data);
    let new_brake = state::brake(new_data);

    if prev_brake == new_brake {
        vec![]
    } else {
        vec![Event::Brake]
    }
}

pub fn clutch_event(prev_data: &Frame, new_data: &Frame) -> Vec<Event> {
    let prev_clutch = state::clutch(prev_data);
    let new_clutch = state::clutch(new_data);

    if prev_clutch == new_clutch {
        vec![]
    } else {
        vec![Event::Clutch]
    }
}

pub fn shifter_x_event(prev_data: &Frame, new_data: &Frame) -> Vec<Event> {
    let prev_shifter_x = state::shifter_x(prev_data);
    let new_shifter_x = state::shifter_x(new_data);

    if prev_shifter_x == new_shifter_x {
        vec![]
    } else {
        vec![Event::ShifterX]
    }
}

pub fn shifter_y_event(prev_data: &Frame, new_data: &Frame) -> Vec<Event> {
    let prev_shifter_y = state::shifter_y(prev_data);
    let new_shifter_y = state::shifter_y(new_data);

    if prev_shifter_y == new_shifter_y {
        vec![]
    } else {
        vec![Event::ShifterY]
    }
}

pub fn shifter_events(prev_data: &Frame, new_data: &Frame) -> Vec<Event> {
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
