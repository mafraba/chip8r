#![feature(slice_patterns)]

#[cfg(test)]
mod tests;

extern crate byteorder;     // to read instructions as big_endian u16 words
extern crate clap;          // to manage command line options and arguments

use byteorder::{BigEndian, ByteOrder};
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
// XXX: will probably end up refactoring this into an immutable 'Chip8State' entity
//      wrapped in here, so each state modification leads to a new state but each of them is immutable
impl Chip8Core {

    // Create a new Chip8Core instance
    fn new() -> Chip8Core {
        Default::default()
    }

    // Load data onto RAM
    fn load(&mut self, data: &[u8]) {
        (&mut self.ram[0x200..(0x200+data.len())]).copy_from_slice(data);
    }

    // Read next instruction
    fn read_instruction(&self) -> Chip8Instruction {
        let pointer = &self.ram[(self.pc as usize) ..];
        let word = BigEndian::read_u16(pointer);
        Chip8Instruction(word)
    }

    // Execute next instruction
    fn exec_instruction(&mut self) {
        let op = self.read_instruction();
        let nibbles = [op.nibble(1), op.nibble(2), op.nibble(3), op.nibble(4)];
        match &nibbles {
            // 00E0: Clear screen
            &[0,0,0xE,0] => self.clear_screen(),
            // 00EE: Return from subroutine
            &[0,0,0xE,0xE] => self.return_from_subroutine(),
            //
            _ => {}
        }
    }

    fn clear_screen(&mut self) {
        (&mut self.ram[0xF00..0xFFF]).copy_from_slice(&[0;0xFF]);
        self.pc += 2;
    }

    fn return_from_subroutine(&mut self) {
        let ret_addr = self.read_return_address();
        self.pc = ret_addr;
        self.sp -= 2;
    }

    // Read the return address the SP points to
    fn read_return_address(&self) -> u16 {
        BigEndian::read_u16(&self.ram[(self.sp as usize)..])
    }
}

// Chip8 instructions modeled as 16-bit unsigned integers
#[derive(Debug)]
#[derive(PartialEq)]
struct Chip8Instruction(u16);

impl Chip8Instruction {
    // Extract selected nibble (1-4)
    fn nibble(&self, n: u8) -> u8 {
        assert!(n > 0 && n <= 4, "nibble out of range 1-4");
        let nibbles_to_shift = 4 - n;
        let bits_to_shift = 4 * nibbles_to_shift;
        ((self.0 >> bits_to_shift) & 0xF) as u8
    }
}
