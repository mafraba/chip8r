#![feature(slice_patterns)]

extern crate byteorder;     // to read instructions as big_endian u16 words
#[macro_use] extern crate chan;          // for easier timers
extern crate clap;          // to manage command line options and arguments
extern crate rand;          // to generate random numbers
extern crate termion;       // to display screen on terminal

mod chip8;

use clap::{Arg, App};
use chip8::Chip8State;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::thread;

fn main() {
    // Define command line arguments.
    let matches = App::new("CHIP-8 interpreter")
        .version("0.1.0")
        .author("Manuel Franco <mafraba@gmail.com>")
        .about("Run CHIP-8 programs in all their glory")
        .arg(Arg::with_name("file.ch8")
                 .short("i")
                 .long("input")
                 .takes_value(true)
                 .required(true)
                 .help("Input CHIP-8 program"))
        .get_matches();

    // Get input file
    let input_file = matches.value_of("file.ch8").unwrap();

    // Read it
    let mut ch8_file = match File::open(input_file) {
        Err(why) => panic!("couldn't open input file '{}': {}", input_file, why.description()),
        Ok(file) => file,
    };
    let mut ch8_buffer = Vec::new();
    ch8_file.read_to_end(&mut ch8_buffer).expect("Failed to read ROM");

    // Create runtime
    let mut ch8state = Chip8State::new();

    // Load file to memory
    ch8state = ch8state.load(&ch8_buffer);

    // use std::{thread, time};
    // let interval = time::Duration::from_millis(1000/120); // ~ 60Hz
    // thread::sleep(interval);
    //
    // for i in 0..(10*60) {
    //     ch8state = ch8state.exec_instruction();
    //     if i % 5 == 0 { display(ch8state); }
    //     thread::sleep(interval);
    // }

    let tick = chan::tick_ms(10); // ~ 100Hz
    let timers_decrease = chan::tick_ms(1000/60); // ~ 60Hz
    let display_refresh = chan::tick_ms(1000/15); // ~ 25Hz
    let (tx_keys, rx_keys) = chan::sync(20);
    thread::spawn(|| chip8::termui::listen_for_keys(tx_keys));
    loop {
        chan_select! {
            tick.recv() => {
                ch8state = ch8state.exec_instruction();
            },
            timers_decrease.recv() => {
                ch8state = ch8state.decrease_timers();
            },
            display_refresh.recv() => {
                chip8::termui::display(ch8state);
            },
            rx_keys.recv() -> key => {
                // if hexadecimal digit
                match key.unwrap().to_digit(16) {
                    Some(digit) => {
                        let d = digit as u8;
                        // since no key-release events are received, need to simulate it with a new press
                        if ch8state.is_key_down(d) {
                            ch8state = ch8state.key_up(d);
                        } else {
                            ch8state = ch8state.key_down(d);
                        }
                    },
                    None => {}
                }
            }
        }
    }
}
