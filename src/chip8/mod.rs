#[cfg(test)]
mod tests;

use byteorder::{BigEndian, ByteOrder};

// A Chip8 runtime state
#[derive(Copy)]
pub struct Chip8State {
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

// Methods for a Chip8State
impl Chip8State {

    // Create a new Chip8State instance
    pub fn new() -> Chip8State {
        Chip8State {
            ram: [0; 0x1000],
            reg: [0; 16],
            i: 0,
            sp: 0xEA0,
            t_delay: 0,
            t_sound: 0,
            pc: 0x200
        }
    }

    // Load data onto RAM, return new state
    pub fn load(&self, data: &[u8]) -> Chip8State {
        let mut new_state = *self;
        (&mut new_state.ram[0x200..(0x200+data.len())]).copy_from_slice(data);
        new_state
    }

    // Read next instruction
    fn read_instruction(&self) -> Chip8Instruction {
        let pointer = &self.ram[(self.pc as usize) ..];
        let word = BigEndian::read_u16(pointer);
        Chip8Instruction(word)
    }

    // Execute next instruction, returning a new state
    fn exec_instruction(&self) -> Chip8State {
        let op = self.read_instruction();
        let nibbles = [op.nibble(1), op.nibble(2), op.nibble(3), op.nibble(4)];
        match &nibbles {
            // 00E0: Clear screen
            &[0,0,0xE,0] => self.clear_screen(),
            // 00EE: Return from subroutine
            &[0,0,0xE,0xE] => self.return_from_subroutine(),
            // 1nnn: Jump to 'nnn' address
            &[1,n1,n2,n3] => self.jump_to(compose_address(n1,n2,n3)),
            // 2nnn: Call subroutine at 'nnn'
            &[2,n1,n2,n3] => self.call_subroutine(compose_address(n1,n2,n3)),
            // Panic if unknown
            _ => panic!("Unknown instruction: {:?}", op)
        }
    }

    fn clear_screen(&self) -> Chip8State {
        let mut new_state = *self;
        (&mut new_state.ram[0xF00..0xFFF]).copy_from_slice(&[0;0xFF]);
        new_state.pc += 2;
        new_state
    }

    fn return_from_subroutine(&self) -> Chip8State {
        let ret_addr = self.read_return_address();
        let mut new_state = *self;
        new_state.pc = ret_addr;
        new_state.sp -= 2;
        new_state
    }

    // Read the return address the SP points to
    fn read_return_address(&self) -> u16 {
        BigEndian::read_u16(&self.ram[(self.sp as usize)..])
    }

    fn jump_to(&self, addr: u16) -> Chip8State {
        let mut new_state = *self;
        new_state.pc = addr;
        new_state
    }

    fn call_subroutine(&self, subroutine_addr: u16) -> Chip8State {
        let mut new_state = *self;
        // stack return address
        let return_address = self.pc + 2;
        new_state.sp += 2;
        new_state.ram[new_state.sp as usize] = (return_address >> 8) as u8;
        new_state.ram[(new_state.sp + 1) as usize] = (return_address & 0xFF) as u8;
        // set PC
        new_state.pc = subroutine_addr;
        new_state
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

fn compose_address(n1: u8, n2: u8, n3: u8) -> u16 {
    (((n1 as u16) << 8) | ((n2 as u16) << 4) | n3 as u16)
}
