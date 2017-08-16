#![feature(slice_patterns)]

extern crate byteorder;     // to read instructions as big_endian u16 words
extern crate clap;          // to manage command line options and arguments

mod chip8;

use clap::{Arg, App};
use chip8::Chip8State;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;

fn main() {
    // Define command line arguments.
    let matches = App::new("CHIP-8 dissassembler")
        .version("0.1.0")
        .author("Manuel Franco <mafraba@gmail.com>")
        .about("Reads CHIP-8 programs and prints them")
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

    println!("To be continued ... :)");
}
