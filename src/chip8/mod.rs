#[cfg(test)]
mod tests;

use byteorder::{BigEndian, ByteOrder};

// Implementation of a CHIP-8 runtime, consisting basically in a state and a set of operations
pub struct Chip8Core {
    state: Box<Chip8State>
}

// A Chip8 runtime state
#[derive(Copy)]
struct Chip8State {
    ram: [u8; 0x1000],  // 4K of memory
    reg: [u8; 16],      // 16 registers
    i: u16,             // Memory address register
    sp: u16,            // Stack pointer
    t_delay: u8,        // Delay timer
    t_sound: u8,        // Sound timer
    pc: u16             // Program counter
}

// Manual implementation is required since arrays only implement clone for sizes < 32
impl Clone for Chip8State {
    fn clone(&self) -> Self {
        Chip8State{
            ram: self.ram,
            reg: self.reg,
            i: self.i,
            sp: self.sp,
            t_delay: self.t_delay,
            t_sound: self.t_sound,
            pc: self.pc
        }
    }
}

// Methods for a Chip8Core
impl Chip8Core {

    // Create a new Chip8Core instance
    pub fn new() -> Chip8Core {
        Chip8Core{
            state: Box::new(Chip8State {
                ram: [0; 0x1000],
                reg: [0; 16],
                i: 0,
                sp: 0xEA0,
                t_delay: 0,
                t_sound: 0,
                pc: 0x200
            })
        }
    }

    // Load data onto RAM
    pub fn load(&mut self, data: &[u8]) {
        let mut new_state = *self.state;
        (&mut new_state.ram[0x200..(0x200+data.len())]).copy_from_slice(data);
        self.state = Box::new(new_state);
    }

    // Read next instruction
    fn read_instruction(&self) -> Chip8Instruction {
        let pointer = &self.state.ram[(self.state.pc as usize) ..];
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
        let mut new_state = *self.state;
        (&mut new_state.ram[0xF00..0xFFF]).copy_from_slice(&[0;0xFF]);
        new_state.pc += 2;
        self.state = Box::new(new_state);
    }

    fn return_from_subroutine(&mut self) {
        let ret_addr = self.read_return_address();
        let mut new_state = *self.state;
        new_state.pc = ret_addr;
        new_state.sp -= 2;
        self.state = Box::new(new_state);
    }

    // Read the return address the SP points to
    fn read_return_address(&self) -> u16 {
        BigEndian::read_u16(&self.state.ram[(self.state.sp as usize)..])
    }

    fn jump_to(&mut self, addr: u16) {
        let mut new_state = *self.state;
        new_state.pc = addr;
        self.state = Box::new(new_state);
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
