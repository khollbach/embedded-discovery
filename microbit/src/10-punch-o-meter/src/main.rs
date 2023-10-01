#![allow(unused_imports)]

#![no_main]
#![no_std]

use core::cmp::max;

use cortex_m::prelude::_embedded_hal_timer_CountDown;
use cortex_m_rt::entry;
use lsm303agr::{Lsm303agr, AccelOutputDataRate, AccelScale};
use microbit::hal::{twim::{self, Frequency}, timer::Timer};
use rtt_target::{rtt_init_print, rprint, rprintln};
use panic_rtt_target as _;

#[entry]
fn main() -> ! {
    rtt_init_print!();
    let board = microbit::Board::take().unwrap();

    let mut timer = Timer::one_shot(board.TIMER0);

    let i2c = twim::Twim::new(board.TWIM0, board.i2c_internal.into(), Frequency::K100);

    let mut sensor = Lsm303agr::new_with_i2c(i2c);
    sensor.set_accel_odr(AccelOutputDataRate::Hz400).unwrap(); // todo
    sensor.set_accel_scale(AccelScale::G16).unwrap();

    // rprint!("...");
    // timer.start(1_000u32); // ns
    // loop {
    //     match timer.wait() {
    //         Ok(()) => break,
    //         Err(nb::Error::WouldBlock) => (),
    //         Err(nb::Error::Other(never)) => void::unreachable(never),
    //     }
    // }
    // rprint!("done.");
    // loop {}


    loop {
        while !sensor.accel_status().unwrap().xyz_new_data {}
        let data = sensor.accel_data().unwrap();
        // rprintln!("{:?}", data);

        if data.x > THRESH {
            let mut max_x = data.x;
            let mut max_y = data.y;
            let mut max_z = data.z;

            rprint!("... ");
            timer.start(1_000_000u32); // (1M ns = 1 second)
            loop {
                match timer.wait() {
                    Ok(()) => break,
                    Err(nb::Error::WouldBlock) => (),
                    Err(nb::Error::Other(never)) => void::unreachable(never),
                }

                while !sensor.accel_status().unwrap().xyz_new_data {}
                let data = sensor.accel_data().unwrap();
                // if data.x < THRESH {
                //     break;
                // }
                
                max_x = max(max_x, data.x);
                max_y = max(max_y, data.y);
                max_z = max(max_z, data.z);
            }
            rprintln!("you punched: {} !  ({} {})", max_x, max_y, max_z);
        }
    }
}

const THRESH: i32 = 2_000;
