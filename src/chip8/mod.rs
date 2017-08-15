#[cfg(test)]
mod tests;

use byteorder::{BigEndian, ByteOrder};


// Implementation of a CHIP-8 runtime
pub struct Chip8Core {
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
    pub fn new() -> Chip8Core {
        Default::default()
    }

    // Load data onto RAM
    pub fn load(&mut self, data: &[u8]) {
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
            // 1nnn: Jump to 'nnn' address
            &[1,a1,a2,a3] => self.jump_to(((a1 as u16) << 8) | ((a2 as u16) << 4) | (a3 as u16)),
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

    fn jump_to(&mut self, addr: u16) {
        self.pc = addr;
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
