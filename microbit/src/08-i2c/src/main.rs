#![allow(unused_imports)]
#![no_main]
#![no_std]

use core::fmt::{Debug, Write};
use cortex_m_rt::entry;
use embedded_hal::serial;
use heapless::Vec;
use lsm303agr::Lsm303agr;
use microbit::hal::uarte::{Baudrate, Parity};
use microbit::hal::{prelude::*, uarte};
use microbit::pac::UARTE0;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};
use core::str;

use microbit::{hal::twim, pac::twim0::frequency::FREQUENCY_A};

const ACCELEROMETER_ADDR: u8 = 0b__001_1001;
const MAGNETOMETER_ADDR: u8 = 0b__001_1110;

const ACCELEROMETER_ID_REG: u8 = 0x0f;
const MAGNETOMETER_ID_REG: u8 = 0x4f;

const ACCELEROMETER_ID: u8 = 0b_0011_0011;
const MAGNETOMETER_ID: u8 = 0b_0100_0000;

mod serial_setup;
use serial_setup::UartePort;

#[entry]
fn main() -> ! {
    rtt_init_print!();
    let board = microbit::Board::take().unwrap();

    let mut i2c = twim::Twim::new(board.TWIM0, board.i2c_internal.into(), FREQUENCY_A::K100);

    // Smoke test
    let mut acc_id = [0u8];
    let mut mag_id = [0u8];
    i2c.write_read(ACCELEROMETER_ADDR, &[ACCELEROMETER_ID_REG], &mut acc_id)
        .unwrap();
    i2c.write_read(MAGNETOMETER_ADDR, &[MAGNETOMETER_ID_REG], &mut mag_id)
        .unwrap();
    assert_eq!(acc_id[0], ACCELEROMETER_ID);
    assert_eq!(mag_id[0], MAGNETOMETER_ID);

    let mut sensor = Lsm303agr::new_with_i2c(i2c).into_mag_continuous().ok().unwrap();
    sensor.init().unwrap();
    sensor
        .set_accel_odr(lsm303agr::AccelOutputDataRate::Hz50)
        .unwrap();
    sensor.set_mag_odr(lsm303agr::MagOutputDataRate::Hz50).unwrap();

    let mut serial = UartePort::new(uarte::Uarte::new(
        board.UARTE0,
        board.uart.into(),
        Parity::EXCLUDED,
        Baudrate::BAUD115200,
    ));

    loop {
        match read_line(&mut serial) {
            Ok(line) => match line.as_slice() {
                b"accelerometer" => {
                    for _ in 0..2 {
                        let mut i = 0;
                        while !sensor.accel_status().unwrap().xyz_new_data {
                            i += 1;
                        }
                        let data = sensor.accel_data().unwrap();
                        rprintln!("{} Acceleration: x {} y {} z {}", i, data.x, data.y, data.z);
                    }
                }
                b"magnetometer" => {
                    for _ in 0..2 {
                        let mut i = 0;
                        while !sensor.mag_status().unwrap().xyz_new_data {
                            i += 1;
                        }
                        let data = sensor.mag_data().unwrap();
                        rprintln!("{} Magnetization(?): x {} y {} z {}", i, data.x, data.y, data.z);
                    }
                }
                _ => {
                    let msg = str::from_utf8(&line).unwrap_or("<non-utf8-data>");
                    write!(serial, "invalid command {msg:?}\r\n").unwrap();
                }
            },
            Err(e) => {
                write!(serial, "{e}\r\n").unwrap();
            }
        }
        nb::block!(serial.flush()).unwrap();
    }
}

fn read_line(serial: &mut UartePort<UARTE0>) -> Result<Vec<u8, 32>, &'static str> {
    write!(serial, "> ").unwrap();
    nb::block!(serial.flush()).unwrap();

    let mut buf = Vec::<u8, 32>::new();
    loop {
        let b = nb::block!(serial.read()).unwrap();

        if b == b'\r' || b == b'\n' {
            write!(serial, "\r\n").unwrap();
            nb::block!(serial.flush()).unwrap();
            return Ok(buf);
        }

        // Echo chars as they're typed.
        write!(serial, "{}", b as char).unwrap();
        nb::block!(serial.flush()).unwrap();

        if let Err(_) = buf.push(b) {
            write!(serial, "\r\n").unwrap();
            nb::block!(serial.flush()).unwrap();
            return Err("buf full");
        }
    }
}
