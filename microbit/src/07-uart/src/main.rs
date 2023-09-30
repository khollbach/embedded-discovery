#![allow(unused_imports)]

#![no_main]
#![no_std]

use cortex_m_rt::entry;
use embedded_hal::serial;
use heapless::Vec;
use rtt_target::{rtt_init_print, rprintln, rprint};
use panic_rtt_target as _;
use core::fmt::{Write, Display, self, Debug};

use microbit::{
    hal::prelude::*,
    hal::uarte,
    hal::uarte::{Baudrate, Parity},
};

mod serial_setup;
use serial_setup::UartePort;

#[entry]
fn main() -> ! {
    rtt_init_print!();
    let board = microbit::Board::take().unwrap();

    let mut serial = {
        let serial = uarte::Uarte::new(
            board.UARTE0,
            board.uart.into(),
            Parity::EXCLUDED,
            Baudrate::BAUD115200,
        );
        UartePort::new(serial)
    };

    // let mut buffer = Vec::<u8, 32>::new();
    // loop {
    //     buffer.clear();

    //     loop {
    //         // We assume that the receiving cannot fail
    //         let byte = nb::block!(serial.read()).unwrap();

    //         if buffer.push(byte).is_err() {
    //             write!(serial, "error: buffer full\r\n").unwrap();
    //             break;
    //         }

    //         if byte == 13 {
    //             for byte in buffer.iter().rev().chain(&[b'\n', b'\r']) {
    //                 nb::block!(serial.write(*byte)).unwrap();
    //             }
    //             break;
    //         }
    //     }
    //     nb::block!(serial.flush()).unwrap();
    // }

    loop {
        // rprintln!("top of loop");

        match read_line(&mut serial) {
            Ok(buf) => {
                for &b in buf.iter().rev() {
                // for &b in buf.as_slice().into_iter().rev() {
                    write!(serial, "{}", b as char).expect("write serial");
                }
                write!(serial, "\r\n").expect("write serial");
            }
            Err(e) => {
                write!(serial, "{e}\r\n").expect("write serial");
            }
        }

        nb::block!(serial.flush()).unwrap();
    }

    // writeln!(serial, "The quick brown fox jumps over the lazy dog.").unwrap();
    // nb::block!(serial.flush()).unwrap();

    // loop {
    //     let b = nb::block!(serial.read()).unwrap();
    //     rprint!("{:?}/{:?}  ", b, b as char);
    //     nb::block!(serial.write(b)).unwrap();
    //     nb::block!(serial.flush()).unwrap();
    // }

    // for &b in b"The quick brown fox jumps over the lazy dog." {
    //     nb::block!(serial.write(b)).unwrap();
    // }
    // nb::block!(serial.flush()).unwrap();

    // nb::block!(serial.write(b'X')).unwrap();
    // nb::block!(serial.flush()).unwrap();

    // loop {}
}

fn read_line<R>(serial: &mut R) -> Result<Vec<u8, 32>, &'static str>
where
    R: serial::Read<u8>,
    <R as serial::Read<u8>>::Error: Debug + Send + Sync,
{
    let mut buf = Vec::<u8, 32>::new();
    loop {
        let b = nb::block!(serial.read()).map_err(|_| "serial read")?;
        if b == b'\r' || b == b'\n' {
            return Ok(buf);
        }
        buf.push(b).map_err(|_| "buf full")?;
    }
}
