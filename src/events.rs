use rayon::prelude::*;
use std::{collections::HashMap, sync::RwLock, thread};

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
    OptionsButtonPressed,
    OptionsButtonReleased,
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

#[derive(Debug)]
pub struct EventMap {
    handlers: HashMap<Event, RwLock<EventHandlers>>,
}

impl Default for EventMap {
    fn default() -> Self {
        Self::new()
    }
}

impl EventMap {
    pub fn new() -> EventMap {
        EventMap {
            handlers: HashMap::new(),
        }
    }

    pub fn insert(&mut self, event: Event, handler: HandlerFn) -> Option<EventHandler> {
        self.handlers
            .entry(event)
            .or_insert_with(|| RwLock::new(EventHandlers::new(event)))
            .write()
            .unwrap()
            .insert(handler)
    }

    pub fn remove(&mut self, event_handler: EventHandler) {
        self.handlers
            .get_mut(&event_handler.event)
            .unwrap()
            .write()
            .unwrap()
            .handlers
            .remove(&event_handler.id);
    }

    fn trigger(&self, event: Event, g29: &mut G29) {
        if let Some(handlers) = self.handlers.get(&event) {
            let handlers = &handlers.read().unwrap().handlers;
            handlers.par_iter().for_each(|(_, handler)| {
                let mut self_1 = g29.clone();
                let ev_clone = *handler; // Clone the event handler
                thread::spawn(move || {
                    (ev_clone.handler)(&mut self_1);
                });
            });
        }
    }

    pub fn trigger_events(&self, prev_data: &Frame, new_data: &Frame, g29: &mut G29) {
        let different_indices = different_indices(prev_data, new_data);

        if different_indices.is_empty() {
            return;
        }

        different_indices.par_iter().for_each(|index| {
            let mut g29 = g29.clone();
            match index {
                0 => {
                    self.trigger_dpad_events(prev_data, new_data, &mut g29);
                    self.trigger_shape_button_events(prev_data, new_data, &mut g29);
                }
                1 => self.trigger_data1_button_events(prev_data, new_data, &mut g29),
                2 => {
                    self.trigger_gear_selector_events(prev_data, new_data, &mut g29);
                    self.trigger_plus_button_events(prev_data, new_data, &mut g29);
                }
                3 => self.trigger_data3_button_events(prev_data, new_data, &mut g29),
                4 | 5 => self.trigger_steering_events(prev_data, new_data, &mut g29),
                6 => self.trigger_throttle_event(prev_data, new_data, &mut g29),
                7 => self.trigger_brake_event(prev_data, new_data, &mut g29),
                8 => self.trigger_clutch_event(prev_data, new_data, &mut g29),
                9 => self.trigger_shifter_x_event(prev_data, new_data, &mut g29),
                10 => self.trigger_shifter_y_event(prev_data, new_data, &mut g29),
                11 => self.trigger_shifter_events(prev_data, new_data, &mut g29),
                _ => {}
            };
        });
    }

    fn trigger_dpad_events(&self, prev_data: &Frame, new_data: &Frame, g29: &mut G29) {
        let prev_dpad = state::dpad(prev_data);
        let new_dpad = state::dpad(new_data);
        if prev_dpad == new_dpad {
            return;
        }

        // which dpad is pressed
        match new_dpad {
            DpadPosition::Up => self.trigger(Event::DpadUpPressed, g29),
            DpadPosition::TopRight => self.trigger(Event::DpadTopRightPressed, g29),
            DpadPosition::Right => self.trigger(Event::DpadRightPressed, g29),
            DpadPosition::BottomRight => self.trigger(Event::DpadBottomRightPressed, g29),
            DpadPosition::Down => self.trigger(Event::DpadBottomPressed, g29),
            DpadPosition::BottomLeft => self.trigger(Event::DpadBottomLeftPressed, g29),
            DpadPosition::Left => self.trigger(Event::DpadLeftPressed, g29),
            DpadPosition::TopLeft => self.trigger(Event::DpadTopLeftPressed, g29),
            _ => {}
        };

        // which dpad is released
        match prev_dpad {
            DpadPosition::Up => self.trigger(Event::DpadUpReleased, g29),
            DpadPosition::TopRight => self.trigger(Event::DpadTopRightReleased, g29),
            DpadPosition::Right => self.trigger(Event::DpadRightReleased, g29),
            DpadPosition::BottomRight => self.trigger(Event::DpadBottomRightReleased, g29),
            DpadPosition::Down => self.trigger(Event::DpadBottomReleased, g29),
            DpadPosition::BottomLeft => self.trigger(Event::DpadBottomLeftReleased, g29),
            DpadPosition::Left => self.trigger(Event::DpadLeftReleased, g29),
            DpadPosition::TopLeft => self.trigger(Event::DpadTopLeftReleased, g29),
            _ => {}
        };
    }

    fn trigger_shape_button_events(&self, prev_data: &Frame, new_data: &Frame, g29: &mut G29) {
        let prev_x_button = state::x_button(prev_data);
        let new_x_button = state::x_button(new_data);
        if prev_x_button != new_x_button {
            if new_x_button {
                self.trigger(Event::XButtonPressed, g29);
            } else {
                self.trigger(Event::XButtonReleased, g29);
            }
        }

        let prev_square_button = state::square_button(prev_data);
        let new_square_button = state::square_button(new_data);
        if prev_square_button != new_square_button {
            if new_square_button {
                self.trigger(Event::SquareButtonPressed, g29);
            } else {
                self.trigger(Event::SquareButtonReleased, g29);
            }
        }

        let prev_circle_button = state::circle_button(prev_data);
        let new_circle_button = state::circle_button(new_data);
        if prev_circle_button != new_circle_button {
            if new_circle_button {
                self.trigger(Event::CircleButtonPressed, g29);
            } else {
                self.trigger(Event::CircleButtonReleased, g29);
            }
        }

        let prev_triangle_button = state::triangle_button(prev_data);
        let new_triangle_button = state::triangle_button(new_data);
        if prev_triangle_button != new_triangle_button {
            if new_triangle_button {
                self.trigger(Event::TriangleButtonPressed, g29);
            } else {
                self.trigger(Event::TriangleButtonReleased, g29);
            }
        }
    }

    fn trigger_data1_button_events(&self, prev_data: &Frame, new_data: &Frame, g29: &mut G29) {
        let prev_right_shifter = state::right_shifter(prev_data);
        let new_right_shifter = state::right_shifter(new_data);
        if prev_right_shifter != new_right_shifter {
            if new_right_shifter {
                self.trigger(Event::RightShifterPressed, g29);
            } else {
                self.trigger(Event::RightShifterReleased, g29);
            }
        }

        let prev_left_shifter = state::left_shifter(prev_data);
        let new_left_shifter = state::left_shifter(new_data);
        if prev_left_shifter != new_left_shifter {
            if new_left_shifter {
                self.trigger(Event::LeftShifterPressed, g29);
            } else {
                self.trigger(Event::LeftShifterReleased, g29);
            }
        }

        let prev_r2_button = state::r2_button(prev_data);
        let new_r2_button = state::r2_button(new_data);
        if prev_r2_button != new_r2_button {
            if new_r2_button {
                self.trigger(Event::R2ButtonPressed, g29);
            } else {
                self.trigger(Event::R2ButtonReleased, g29);
            }
        }

        let prev_l2_button = state::l2_button(prev_data);
        let new_l2_button = state::l2_button(new_data);
        if prev_l2_button != new_l2_button {
            if new_l2_button {
                self.trigger(Event::L2ButtonPressed, g29);
            } else {
                self.trigger(Event::L2ButtonReleased, g29);
            }
        }

        let prev_share_button = state::share_button(prev_data);
        let new_share_button = state::share_button(new_data);
        if prev_share_button != new_share_button {
            if new_share_button {
                self.trigger(Event::ShareButtonPressed, g29);
            } else {
                self.trigger(Event::ShareButtonReleased, g29);
            }
        }

        let prev_option_button = state::option_button(prev_data);
        let new_option_button = state::option_button(new_data);
        if prev_option_button != new_option_button {
            if new_option_button {
                self.trigger(Event::OptionsButtonPressed, g29);
            } else {
                self.trigger(Event::OptionsButtonReleased, g29);
            }
        }

        let prev_r3_button = state::r3_button(prev_data);
        let new_r3_button = state::r3_button(new_data);
        if prev_r3_button != new_r3_button {
            if new_r3_button {
                self.trigger(Event::R3ButtonPressed, g29);
            } else {
                self.trigger(Event::R3ButtonReleased, g29);
            }
        }

        let prev_l3_button = state::l3_button(prev_data);
        let new_l3_button = state::l3_button(new_data);
        if prev_l3_button != new_l3_button {
            if new_l3_button {
                self.trigger(Event::L3ButtonPressed, g29);
            } else {
                self.trigger(Event::L3ButtonReleased, g29);
            }
        }
    }

    fn trigger_gear_selector_events(&self, prev_data: &Frame, new_data: &Frame, g29: &mut G29) {
        let prev_gear_selector = state::gear_selector(prev_data);
        let new_gear_selector = state::gear_selector(new_data);

        if prev_gear_selector == new_gear_selector {
            return;
        }

        self.trigger(Event::GearChanged, g29);
    }

    fn trigger_plus_button_events(&self, prev_data: &Frame, new_data: &Frame, g29: &mut G29) {
        let prev_plus_button = state::plus_button(prev_data);
        let new_plus_button = state::plus_button(new_data);
        if prev_plus_button == new_plus_button {
            return;
        } else if new_plus_button {
            self.trigger(Event::PlusButtonPressed, g29);
        } else {
            self.trigger(Event::PlusButtonReleased, g29);
        }
    }

    fn trigger_data3_button_events(&self, prev_data: &Frame, new_data: &Frame, g29: &mut G29) {
        /*
           minus_button
           spinner_right
           spinner_left
           spinner_button
           playstation_button
        */

        let prev_minus_button = state::minus_button(prev_data);
        let new_minus_button = state::minus_button(new_data);
        if prev_minus_button != new_minus_button {
            if new_minus_button {
                self.trigger(Event::MinusButtonPressed, g29);
            } else {
                self.trigger(Event::MinusButtonReleased, g29);
            }
        }

        let prev_spinner_right = state::spinner_right(prev_data);
        let new_spinner_right = state::spinner_right(new_data);
        if prev_spinner_right != new_spinner_right && new_spinner_right {
            self.trigger(Event::SpinnerRight, g29);
        }

        let prev_spinner_left = state::spinner_left(prev_data);
        let new_spinner_left = state::spinner_left(new_data);
        if prev_spinner_left != new_spinner_left && new_spinner_left {
            self.trigger(Event::SpinnerLeft, g29);
        }

        let prev_spinner_button = state::spinner_button(prev_data);
        let new_spinner_button = state::spinner_button(new_data);
        if prev_spinner_button != new_spinner_button {
            if new_spinner_button {
                self.trigger(Event::SpinnerButtonPressed, g29);
            } else {
                self.trigger(Event::SpinnerButtonReleased, g29);
            }
        }

        let prev_playstation_button = state::playstation_button(prev_data);
        let new_playstation_button = state::playstation_button(new_data);
        if prev_playstation_button != new_playstation_button {
            if new_playstation_button {
                self.trigger(Event::PlaystationButtonPressed, g29);
            } else {
                self.trigger(Event::PlaystationButtonReleased, g29);
            }
        }
    }

    fn trigger_steering_events(&self, prev_data: &Frame, new_data: &Frame, g29: &mut G29) {
        let prev_steering_fine = state::steering_fine(prev_data);
        let new_steering_fine = state::steering_fine(new_data);
        if prev_steering_fine != new_steering_fine {
            self.trigger(Event::SteeringFine, g29);
        }

        let prev_steering = state::steering(prev_data);
        let new_steering = state::steering(new_data);
        if prev_steering != new_steering {
            self.trigger(Event::Steering, g29);
        }
    }

    fn trigger_throttle_event(&self, prev_data: &Frame, new_data: &Frame, g29: &mut G29) {
        let prev_throttle = state::throttle(prev_data);
        let new_throttle = state::throttle(new_data);

        if prev_throttle != new_throttle {
            self.trigger(Event::Throttle, g29);
        }
    }

    fn trigger_brake_event(&self, prev_data: &Frame, new_data: &Frame, g29: &mut G29) {
        let prev_brake = state::brake(prev_data);
        let new_brake = state::brake(new_data);

        if prev_brake != new_brake {
            self.trigger(Event::Brake, g29);
        }
    }

    fn trigger_clutch_event(&self, prev_data: &Frame, new_data: &Frame, g29: &mut G29) {
        let prev_clutch = state::clutch(prev_data);
        let new_clutch = state::clutch(new_data);

        if prev_clutch != new_clutch {
            self.trigger(Event::Clutch, g29);
        }
    }

    fn trigger_shifter_x_event(&self, prev_data: &Frame, new_data: &Frame, g29: &mut G29) {
        let prev_shifter_x = state::shifter_x(prev_data);
        let new_shifter_x = state::shifter_x(new_data);

        if prev_shifter_x != new_shifter_x {
            self.trigger(Event::ShifterX, g29);
        }
    }

    fn trigger_shifter_y_event(&self, prev_data: &Frame, new_data: &Frame, g29: &mut G29) {
        let prev_shifter_y = state::shifter_y(prev_data);
        let new_shifter_y = state::shifter_y(new_data);

        if prev_shifter_y != new_shifter_y {
            self.trigger(Event::ShifterY, g29);
        }
    }

    fn trigger_shifter_events(&self, prev_data: &Frame, new_data: &Frame, g29: &mut G29) {
        let prev_shifter_pressed = state::shifter_pressed(prev_data);
        let new_shifter_pressed = state::shifter_pressed(new_data);

        if prev_shifter_pressed != new_shifter_pressed {
            if new_shifter_pressed {
                self.trigger(Event::ShifterPressed, g29);
            } else {
                self.trigger(Event::ShifterReleased, g29);
            }
        }
    }
}

fn different_indices(data1: &Frame, data2: &Frame) -> Vec<usize> {
    data1
        .iter()
        .enumerate()
        .filter_map(|(i, &x)| if x != data2[i] { Some(i) } else { None })
        .collect()
}
