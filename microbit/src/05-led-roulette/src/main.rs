#![deny(unsafe_code)]
#![no_main]
#![no_std]

use cortex_m_rt::entry;
use microbit::{display::blocking::Display, hal::Timer, Board};
use panic_rtt_target as _;
use rtt_target::rtt_init_print;

#[entry]
fn main() -> ! {
    rtt_init_print!();
    let board = Board::take().unwrap();
    let mut timer = Timer::new(board.TIMER0);
    let mut display = Display::new(board.display_pins);

    for (i, j) in border_5x5().cycle() {
        let mut leds = ALL_OFF;
        leds[i][j] = 1;

        display.show(&mut timer, leds, 100);
        display.clear();
    }

    unreachable!()
}

const ALL_OFF: [[u8; 5]; 5] = [
    [0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0],
];

fn border_5x5() -> impl Iterator<Item = (usize, usize)> + Clone {
    border_n_by_n(5)
}

fn border_n_by_n(n: usize) -> impl Iterator<Item = (usize, usize)> + Clone {
    let top = (0..n - 1).map(move |j| (0, j));
    let right = (0..n - 1).map(move |i| (i, n - 1));
    let bot = (1..n).rev().map(move |j| (n - 1, j));
    let left = (1..n).rev().map(move |i| (i, 0));
    top.chain(right).chain(bot).chain(left)
}
