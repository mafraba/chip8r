#[cfg(test)]
mod tests;

extern crate clap;

use clap::{Arg, App};
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
    let mut ch8core = Chip8Core::new();

    // Load file to memory
    ch8core.load(&ch8_buffer);

    println!("To be continued ... :)");
}

// Implementation of a CHIP-8 runtime
struct Chip8Core {
    ram: [u8; 0x1000],  // 4K of memory
    reg: [u8; 16],      // 16 registers
    i: u16,             // Memory address register
    sp: u16,            // Stack pointer
    t_delay: u8,        // Delay timer
    t_sound: u8,        // Sound timer
    pc: u16             // Program counter
}

// Default values for a Chip8Core fileds
impl Default for Chip8Core {
    fn default() -> Chip8Core {
        Chip8Core {
            ram: [0; 0x1000],
            reg: [0; 16],
            i: 0,
            sp: 0xEA0,
            t_delay: 0,
            t_sound: 0,
            pc: 0x200
        }
    }
}

// Methods for a Chip8Core
impl Chip8Core {
    // Create a new brand Chip8Core instance
    fn new() -> Chip8Core {
        Default::default()
    }

    // Load data onto RAM
    fn load(&mut self, data: &[u8]) {
        (&mut self.ram[0x200..(0x200+data.len())]).copy_from_slice(data);
    }
}
