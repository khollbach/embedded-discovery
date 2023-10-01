#![allow(unused_imports)]
#![no_main]
#![no_std]

use calibration::Calibration;
use cortex_m::interrupt;
use cortex_m_rt::entry;
use lsm303agr::Measurement;
use microbit::display::blocking::{Display as BlockingDisplay};
use microbit::display::nonblocking::{Display, BitImage, GreyscaleImage};
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

mod calibration;
use crate::calibration::calc_calibration;
use crate::calibration::calibrated_measurement;

use microbit::hal::Timer;

use microbit::{hal::twim, pac::twim0::frequency::FREQUENCY_A};

use lsm303agr::{AccelOutputDataRate, Lsm303agr, MagOutputDataRate};

// You'll find this useful ;-)
use core::f32::consts::PI;
use libm::{atan2f, sqrtf};

const CALIBRATION: Calibration = Calibration {
    center: Measurement {
        x: -24728,
        y: 32424,
        z: 86592,
    },
    scale: Measurement {
        x: 1289,
        y: 1309,
        z: 1348,
    },
    radius: 42624,
};

#[entry]
fn main() -> ! {
    rtt_init_print!();
    let board = microbit::Board::take().unwrap();

    let i2c = twim::Twim::new(board.TWIM0, board.i2c_internal.into(), FREQUENCY_A::K100);

    let mut sensor = Lsm303agr::new_with_i2c(i2c);
    sensor.init().unwrap();
    sensor.set_mag_odr(MagOutputDataRate::Hz10).unwrap();
    sensor.set_accel_odr(AccelOutputDataRate::Hz10).unwrap();
    let mut sensor = sensor.into_mag_continuous().ok().unwrap();

    // let mut display = Display::new(board.TIMER0, board.display_pins);
    // let calibration = calc_calibration(&mut sensor, &mut display, &mut timer);
    // rprintln!("Calibration done, entering busy loop");

    let mut timer = Timer::new(board.TIMER0);
    let mut display = BlockingDisplay::new(board.display_pins);

    let calibration = CALIBRATION;
    rprintln!("Using default calibration: {:?}", calibration);

    loop {
        while !sensor.mag_status().unwrap().xyz_new_data {}
        let mut data = sensor.mag_data().unwrap();
        data = calibrated_measurement(data, &calibration);

        // rprintln!("x: {}, y: {}, z: {}", data.x, data.y, data.z);

        let x = data.x as f32;
        let y = data.y as f32;
        let z = data.z as f32;
        let magnitude = sqrtf(x * x + y * y + z * z);
        rprintln!("{} nT, {} mG", magnitude, magnitude/100.);

        // let dir = match (data.x > 0, data.y > 0) {
        //     (true, true) => Direction::NorthEast,
        //     (false, true) => Direction::NorthWest,
        //     (false, false) => Direction::SouthWest,
        //     (true, false) => Direction::SouthEast,
        // };

        // // If I'm facing a given direction, which way is north, relative to my
        // // current direction?
        // let arrow = match dir {
        //     Direction::NorthEast => UP_LEFT,
        //     Direction::NorthWest => UP_RIGHT,
        //     Direction::SouthWest => DOWN_RIGHT,
        //     Direction::SouthEast => DOWN_LEFT,
        // };
        // // let arrow = arrow.map(|row| row.map(|x| if x != 0 { 9 } else { 0 }));
        
        // let theta = atan2f(data.y as f32, data.x as f32);
        // let dir = match theta {
        //     _ if theta > PI / 2. => Direction::NorthWest,
        //     _ if theta < -PI / 2. => Direction::SouthWest,
        //     _ if theta >= 0. => Direction::NorthEast,
        //     _ => Direction::SouthEast,
        // };

        // // If I'm facing a given direction, which way is north, relative to my
        // // current direction?
        // let arrow = match dir {
        //     Direction::NorthEast => UP_LEFT,
        //     Direction::NorthWest => UP_RIGHT,
        //     Direction::SouthWest => DOWN_RIGHT,
        //     Direction::SouthEast => DOWN_LEFT,
        // };

        let theta = atan2f(data.y as f32, data.x as f32);
        let dir = match theta {
            _ if theta > 7./8. * PI => Direction::West,
            _ if theta > 5./8. * PI => Direction::NorthWest,
            _ if theta > 3./8. * PI => Direction::North,
            _ if theta > 1./8. * PI => Direction::NorthEast,

            _ if theta < -7./8. * PI => Direction::West,
            _ if theta < -5./8. * PI => Direction::SouthWest,
            _ if theta < -3./8. * PI => Direction::South,
            _ if theta < -1./8. * PI => Direction::SouthEast,

            _ => Direction::East,
        };

        // If I'm facing a given direction, which way is north, relative to my
        // current direction?
        let arrow = match dir {
            Direction::NorthEast => UP_LEFT,
            Direction::NorthWest => UP_RIGHT,
            Direction::SouthWest => DOWN_RIGHT,
            Direction::SouthEast => DOWN_LEFT,

            Direction::North => UP,
            Direction::South => DOWN,
            Direction::East => LEFT,
            Direction::West => RIGHT,
        };

        display.show(&mut timer, arrow, 100 /* ms */);
        // interrupt::free(|_| display.show(&BitImage::new(&arrow)));
    }
}

// enum Direction {
//     NorthEast,
//     NorthWest,
//     SouthEast,
//     SouthWest,
// }

enum Direction {
    North,
    East,
    South,
    West,
    NorthEast,
    NorthWest,
    SouthEast,
    SouthWest,
}

const UP_LEFT: [[u8; 5]; 5] = [
    [1, 1, 1, 1, 0],
    [1, 1, 0, 0, 0],
    [1, 0, 1, 0, 0],
    [1, 0, 0, 1, 0],
    [0, 0, 0, 0, 1],
];

const UP_RIGHT: [[u8; 5]; 5] = [
    [0, 1, 1, 1, 1],
    [0, 0, 0, 1, 1],
    [0, 0, 1, 0, 1],
    [0, 1, 0, 0, 1],
    [1, 0, 0, 0, 0],
];

const DOWN_LEFT: [[u8; 5]; 5] = [
    [0, 0, 0, 0, 1],
    [1, 0, 0, 1, 0],
    [1, 0, 1, 0, 0],
    [1, 1, 0, 0, 0],
    [1, 1, 1, 1, 0],
];

const DOWN_RIGHT: [[u8; 5]; 5] = [
    [1, 0, 0, 0, 0],
    [0, 1, 0, 0, 1],
    [0, 0, 1, 0, 1],
    [0, 0, 0, 1, 1],
    [0, 1, 1, 1, 1],
];

const UP: [[u8; 5]; 5] = [
    [0, 0, 1, 0, 0],
    [0, 1, 1, 1, 0],
    [1, 0, 1, 0, 1],
    [0, 0, 1, 0, 0],
    [0, 0, 1, 0, 0],
];


const DOWN: [[u8; 5]; 5] = [
    [0, 0, 1, 0, 0],
    [0, 0, 1, 0, 0],
    [1, 0, 1, 0, 1],
    [0, 1, 1, 1, 0],
    [0, 0, 1, 0, 0],
];

const LEFT: [[u8; 5]; 5] = [
    [0, 0, 1, 0, 0],
    [0, 1, 0, 0, 0],
    [1, 1, 1, 1, 1],
    [0, 1, 0, 0, 0],
    [0, 0, 1, 0, 0],
];

const RIGHT: [[u8; 5]; 5] = [
    [0, 0, 1, 0, 0],
    [0, 0, 0, 1, 0],
    [1, 1, 1, 1, 1],
    [0, 0, 0, 1, 0],
    [0, 0, 1, 0, 0],
];
