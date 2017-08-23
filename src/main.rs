#![feature(slice_patterns)]
#![feature(test)]

extern crate byteorder;     // to read instructions as big_endian u16 words
#[macro_use] extern crate chan;          // for easier timers
extern crate clap;          // to manage command line options and arguments
extern crate rand;          // to generate random numbers
extern crate termion;       // to display screen on terminal
extern crate test;

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
        .arg(Arg::with_name("clock_speed")
                .short("s")
                .long("speed")
                .takes_value(true)
                .default_value("500")
                .validator(validate_clock_speed)
                .help("Clock speed (in hertzs)"))
        .get_matches();

    // Get input file
    let input_file = matches.value_of("file.ch8").unwrap();
    let clock_speed_str = matches.value_of("clock_speed").unwrap();
    let clock_speed = clock_speed_str.parse::<u32>().unwrap();

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

    // Create timer to execution of instruction
    let tick = chan::tick_ms(1000/clock_speed);
    // Create timer thread for decreasing delay and sound timers
    let timers_decrease = chan::tick_ms(1000/60); // ~ 60Hz
    // Create timer thread for refreshing display
    let display_refresh_timer = chan::tick_ms(1000/15); // ~ 15Hz
    // Create display thread so we can just send the state and continue this main thread
    let (tx_display, rx_display) = chan::async();
    thread::spawn(|| chip8::termui::display_loop(rx_display));
    // Create thread to read keyboard events
    let (tx_keys, rx_keys) = chan::async();
    thread::spawn(|| chip8::termui::listen_for_keys(tx_keys));

    loop {
        chan_select! {
            tick.recv() => {
                ch8state = ch8state.exec_instruction();
            },
            timers_decrease.recv() => {
                ch8state = ch8state.decrease_timers();
            },
            display_refresh_timer.recv() => {
                tx_display.send(ch8state);
            },
            rx_keys.recv() -> key => {
                // if hexadecimal digit
                match key.unwrap().to_digit(16) {
                    Some(digit) => {
                        let d = digit as u8;
                        // since no key-release events are received,
                        // need to simulate it with a new press
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

fn validate_clock_speed(clock_speed_str: String) -> Result<(), String> {
    let clock_speed = clock_speed_str.parse::<u32>();
    match clock_speed {
        Ok(s) => {
            if s>0 && s<=1000 {
                Ok(())
            } else {
                Err(String::from("the clock speed must be lower than 1000Hz"))
            }
        },
        _ => Err(format!("not an integer value: {}", clock_speed_str))
    }
}
