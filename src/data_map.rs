//----------
// Data Map
//----------
/*
Details on each item of the read buffer provided by node-hid for the Logitech G29.

    Zero
        Wheel - Dpad
            0 = Top
            1 = Top Right
            2 = Right
            3 = Bottom Right
            4 = Bottom
            5 = Bottom Left
            6 = Left
            7 = Top Left
            8 = Dpad in Neutral Position

        Wheel - Symbol Buttons
           16 = X
           32 = Square
           64 = Circle
          128 = Triangle

    One
        Wheel - Shifter Pedals
            1 = Right Shifter
            2 = Left Shifter

        Wheel - Buttons
            4 = R2 Button
            8 = L2 Button
           16 = Share Button
           32 = Option Button
           64 = R3 Button
          128 = L3 Button

    Two
        Shifter - Gear Selector
             0 = Neutral
             1 = 1st Gear
             2 = 2nd Gear
             4 = 3rd Gear
             8 = 4th Gear
            16 = 5th Gear
            32 = 6th Gear
            64 = Reverse Gear

        Wheel
           128 = Plus Button

    Three
        Wheel - Spinner and Buttons
            1 = Minus Button
            2 = Spinner Right
            4 = Spinner Left
            8 = Spinner Button
           16 = PlayStation Button

    Four
        Wheel - Wheel Turn (fine movement)
            0-255

            0 is far left
            255 is far right

    Five
        Wheel - Wheel Turn
            0-255

            0 is far left
            255 is far right

    Six
        Pedals - Gas
            0-255

            0 is full gas
            255 is no pressure

    Seven
        Pedals - Brake
            0-255

            0 is full brake
            255 is no pressure

    Eight
        Pedals - Clutch
            0-255

            0 is full clutch
            255 is no pressure

    Nine
        Shifter
            X Coordinates (not used)

    Ten
        Shifter
            Y Coordinates (not used)

    Eleven
        Shifter
            Contains data on whether or not the gear selector is pressed down into the unit.
            If pressed down, the user is probably preparing to go into reverse. (not used)
*/

use crate::Memory;

//-----------
// Functions
//-----------
pub fn map_data(data_diff_positions: Vec<usize>, data: [u8; 12], memory: &mut Memory) {
    /*
    Figure out what has changed since the last event and call relevant functions to translate those changes to a memory object.
    @param   {Object}  dataDiffPositions  An array.
    @param   {Buffer}  data               Buffer data from a node-hid event.
    @param   {Object}  memory             Memory object to modify.
    @return  {Object}  memory             Modified memory object.
    */

    for i in data_diff_positions.iter() {
        match i {
            0 => {
                wheel_dpad(data, memory);
                wheel_buttons_symbols(data, memory);
            }
            1 => {
                wheel_shift_pedals(data, memory);
                wheel_buttons(data, memory);
            }
            2 => {
                shifter_gear(data, memory);
                wheel_button_plus(data, memory);
            }
            3 => wheel_spinner_and_buttons(data, memory),
            4 | 5 => wheel_turn(data, memory),
            6 => pedals_gas(data, memory),
            7 => pedals_brake(data, memory),
            8 => pedals_clutch(data, memory),
            11 => {
                shifter_gear(data, memory) // for reverse
            }
            _ => {}
        }
    }
} // dataMap

fn reduce_number_from_to(mut num: u8, mut to: u8) -> u8 {
    /*
      Reduce a number by 128, 64, 32, etc... without going lower than a second number.
    */
    to = to * 2;

    let mut y = 128;

    while y > 1 {
        if num < to {
            break;
        }

        if num - y > 0 {
            num = num - y
        }

        y /= 2;
    }

    num
} // reduceNumberFromTo

//------------------
// Functions: Wheel
//------------------
fn wheel_button_plus(data: [u8; 12], memory: &mut Memory) {
    let d = data[2];

    memory.wheel.button_plus = if d & 128 != 0 { 1 } else { 0 };
} // wheelButtonPlus

fn wheel_buttons(data: [u8; 12], memory: &mut Memory) {
    let d = data[1];

    memory.wheel.button_r2 = if d & 4 != 0 { 1 } else { 0 };

    memory.wheel.button_l2 = if d & 8 != 0 { 1 } else { 0 };

    memory.wheel.button_share = if d & 16 != 0 { 1 } else { 0 };

    memory.wheel.button_option = if d & 32 != 0 { 1 } else { 0 };

    memory.wheel.button_r3 = if d & 64 != 0 { 1 } else { 0 };

    memory.wheel.button_l3 = if d & 128 != 0 { 1 } else { 0 };
} // wheelButtons

fn wheel_buttons_symbols(data: [u8; 12], memory: &mut Memory) {
    let d = data[0];

    memory.wheel.button_x = if d & 16 != 0 { 1 } else { 0 };
    memory.wheel.button_square = if d & 32 != 0 { 1 } else { 0 };
    memory.wheel.button_circle = if d & 64 != 0 { 1 } else { 0 };
    memory.wheel.button_triangle = if d & 128 != 0 { 1 } else { 0 };
} // wheelButtonsSymbols

fn wheel_dpad(data: [u8; 12], memory: &mut Memory) {
    let dpad = reduce_number_from_to(data[0], 8);

    match dpad {
        8 => memory.wheel.dpad = 0,
        7 => memory.wheel.dpad = 8,
        6 => memory.wheel.dpad = 7,
        5 => memory.wheel.dpad = 6,
        4 => memory.wheel.dpad = 5,
        3 => memory.wheel.dpad = 4,
        2 => memory.wheel.dpad = 3,
        1 => memory.wheel.dpad = 2,
        0 => memory.wheel.dpad = 1,
        _ => {}
    }
} // wheelDpad

fn wheel_shift_pedals(data: [u8; 12], memory: &mut Memory) {
    let d = data[1];

    memory.wheel.shift_right = d & 1;

    memory.wheel.shift_left = if d & 2 != 0 { 1 } else { 0 };
} // wheelShiftPedals

fn wheel_spinner_and_buttons(data: [u8; 12], memory: &mut Memory) {
    let d = data[3];

    memory.wheel.button_minus = d & 1;

    if d & 2 != 0 {
        memory.wheel.spinner = 2 // right
    } else if d & 4 != 0 {
        memory.wheel.spinner = 1 // left
    } else {
        memory.wheel.spinner = 0
    }

    memory.wheel.button_spinner = if d & 8 != 0 { 1 } else { 0 };

    memory.wheel.button_playstation = if d & 16 != 0 { 1 } else { 0 };
} // wheelSpinnerAndButtons

fn wheel_turn(data: [u8; 12], memory: &mut Memory) {
    let wheel = data[5]; // between 0 and 255

    memory.wheel.turn = wheel;
} // wheelTurn

//-------------------
// Functions: Pedals
//-------------------
fn pedals_brake(data: [u8; 12], memory: &mut Memory) {
    memory.pedals.brake = pedal_to_percent(data[7]);
} // pedalsBrake

fn pedals_clutch(data: [u8; 12], memory: &mut Memory) {
    memory.pedals.clutch = pedal_to_percent(data[8]);
} // pedalsClutch

fn pedals_gas(data: [u8; 12], memory: &mut Memory) {
    memory.pedals.gas = pedal_to_percent(data[6]);
} // pedalsGas

fn pedal_to_percent(num: u8) -> u8 {
    /*
      Converts a number from 0-255 to a number from 0-100.
    */
    let percent = 100 - ((num as f32 / 255.0) * 100.0) as u8;
    percent
} // pedalToPercent

//--------------------
// Functions: Shifter
//--------------------
fn shifter_gear(data: [u8; 12], memory: &mut Memory) {
    let stick = reduce_number_from_to(data[2], 64);

    match stick {
        0 => memory.shifter.gear = 0,
        1 => memory.shifter.gear = 1,
        2 => memory.shifter.gear = 2,
        4 => memory.shifter.gear = 3,
        8 => memory.shifter.gear = 4,
        16 => memory.shifter.gear = 5,
        32 => memory.shifter.gear = 6,
        64 => memory.shifter.gear = 7, // reverse
        _ => {}
    }
} // shifterGear

pub fn diff_positions(prev: [u8; 12], current: [u8; 12]) -> Vec<usize> {
    /*
    Compare two arrays and return an array of the positions that have changed.
    @param   {Buffer}  prev     Buffer data from a node-hid event.
    @param   {Buffer}  current  Buffer data from a node-hid event.
    @return  {Array}   diff     Array of positions that have changed.
    */
    let mut diff = vec![];

    for i in 0..12 {
        if prev[i] != current[i] {
            diff.push(i);
        }
    }

    diff
} // diffPositions
