use rayon::prelude::*;
use std::{collections::HashMap, sync::RwLock, thread};

use crate::{state, DpadPosition, Frame, G29};

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
        [
            (Event::XButtonPressed, Event::XButtonReleased),
            (Event::SquareButtonPressed, Event::SquareButtonReleased),
            (Event::CircleButtonPressed, Event::CircleButtonReleased),
            (Event::TriangleButtonPressed, Event::TriangleButtonReleased),
        ]
        .par_iter()
        .for_each_with(g29.clone(), |g, (pressed, released)| {
            let prev = match pressed {
                Event::XButtonPressed => state::x_button(prev_data),
                Event::SquareButtonPressed => state::square_button(prev_data),
                Event::CircleButtonPressed => state::circle_button(prev_data),
                Event::TriangleButtonPressed => state::triangle_button(prev_data),
                _ => false,
            };

            let new = match pressed {
                Event::XButtonPressed => state::x_button(new_data),
                Event::SquareButtonPressed => state::square_button(new_data),
                Event::CircleButtonPressed => state::circle_button(new_data),
                Event::TriangleButtonPressed => state::triangle_button(new_data),
                _ => false,
            };

            if prev != new {
                if new {
                    self.trigger(*pressed, g);
                } else {
                    self.trigger(*released, g);
                }
            }
        });
    }

    fn trigger_data1_button_events(&self, prev_data: &Frame, new_data: &Frame, g29: &mut G29) {
        [
            (Event::RightShifterPressed, Event::RightShifterReleased),
            (Event::LeftShifterPressed, Event::LeftShifterReleased),
            (Event::R2ButtonPressed, Event::R2ButtonReleased),
            (Event::L2ButtonPressed, Event::L2ButtonReleased),
            (Event::ShareButtonPressed, Event::ShareButtonReleased),
            (Event::OptionsButtonPressed, Event::OptionsButtonReleased),
            (Event::R3ButtonPressed, Event::R3ButtonReleased),
            (Event::L3ButtonPressed, Event::L3ButtonReleased),
        ]
        .par_iter()
        .for_each_with(g29.clone(), |g, (pressed, released)| {
            let prev = match pressed {
                Event::RightShifterPressed => state::right_shifter(prev_data),
                Event::LeftShifterPressed => state::left_shifter(prev_data),
                Event::R2ButtonPressed => state::r2_button(prev_data),
                Event::L2ButtonPressed => state::l2_button(prev_data),
                Event::ShareButtonPressed => state::share_button(prev_data),
                Event::OptionsButtonPressed => state::options_button(prev_data),
                Event::R3ButtonPressed => state::r3_button(prev_data),
                Event::L3ButtonPressed => state::l3_button(prev_data),
                _ => false,
            };

            let new = match pressed {
                Event::RightShifterPressed => state::right_shifter(new_data),
                Event::LeftShifterPressed => state::left_shifter(new_data),
                Event::R2ButtonPressed => state::r2_button(new_data),
                Event::L2ButtonPressed => state::l2_button(new_data),
                Event::ShareButtonPressed => state::share_button(new_data),
                Event::OptionsButtonPressed => state::options_button(new_data),
                Event::R3ButtonPressed => state::r3_button(new_data),
                Event::L3ButtonPressed => state::l3_button(new_data),
                _ => false,
            };

            if prev != new {
                if new {
                    self.trigger(*pressed, g);
                } else {
                    self.trigger(*released, g);
                }
            }
        });
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
        [
            (Event::MinusButtonPressed, Event::MinusButtonReleased),
            (Event::SpinnerRight, Event::SpinnerRight),
            (Event::SpinnerLeft, Event::SpinnerLeft),
            (Event::SpinnerButtonPressed, Event::SpinnerButtonReleased),
            (
                Event::PlaystationButtonPressed,
                Event::PlaystationButtonReleased,
            ),
        ]
        .par_iter()
        .for_each_with(g29.clone(), |g, (pressed, released)| {
            match pressed {
                Event::SpinnerRight => {
                    let prev_spinner_right = state::spinner_right(prev_data);
                    let new_spinner_right = state::spinner_right(new_data);
                    if prev_spinner_right != new_spinner_right && new_spinner_right {
                        self.trigger(Event::SpinnerRight, g);
                    }
                    return;
                }
                Event::SpinnerLeft => {
                    let prev_spinner_left = state::spinner_left(prev_data);
                    let new_spinner_left = state::spinner_left(new_data);
                    if prev_spinner_left != new_spinner_left && new_spinner_left {
                        self.trigger(Event::SpinnerLeft, g);
                    }
                    return;
                }
                _ => {}
            }

            let prev = match pressed {
                Event::MinusButtonPressed => state::minus_button(prev_data),
                Event::SpinnerButtonPressed => state::spinner_button(prev_data),
                Event::PlaystationButtonPressed => state::playstation_button(prev_data),
                _ => false,
            };

            let new = match pressed {
                Event::MinusButtonPressed => state::minus_button(new_data),
                Event::SpinnerButtonPressed => state::spinner_button(new_data),
                Event::PlaystationButtonPressed => state::playstation_button(new_data),
                _ => false,
            };

            if prev != new {
                if new {
                    self.trigger(*pressed, g);
                } else {
                    self.trigger(*released, g);
                }
            }
        });
    }

    fn trigger_steering_events(&self, prev_data: &Frame, new_data: &Frame, g29: &mut G29) {
        [Event::Steering, Event::SteeringFine]
            .par_iter()
            .for_each_with(g29.clone(), |g29, op| {
                let changed = match op {
                    Event::Steering => state::steering(prev_data) != state::steering(new_data),
                    Event::SteeringFine => {
                        state::steering_fine(prev_data) != state::steering_fine(new_data)
                    }
                    _ => false,
                };

                if changed {
                    self.trigger(*op, g29);
                }
            });
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

#[cfg(test)]
mod tests {
    use crate::Frame;

    #[test]
    fn test_different_indices_none() {
        let data1: Frame = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
        let data2: Frame = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];

        let result = crate::events::different_indices(&data1, &data2);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_different_indices_some() {
        let data1: Frame = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
        let data2: Frame = [1, 2, 3, 4, 5, 6, 7, 8, 9, 11, 13, 12];

        let result = crate::events::different_indices(&data1, &data2);
        assert_eq!(result.len(), 2);
        assert_eq!(result, vec![9, 10]);
    }
}
