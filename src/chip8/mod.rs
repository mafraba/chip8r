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
            &[1,n1,n2,n3] => self.jump_to(u16_from_nibbles(0, n1, n2, n3)),
            // 2nnn: Call subroutine at 'nnn'
            &[2,n1,n2,n3] => self.call_subroutine(u16_from_nibbles(0, n1, n2, n3)),
            // 3xkk: Skip next instruction if Vx == kk
            &[3,x,k1,k2] => self.skip_if_equals_immediate(x, u8_from_nibbles(k1, k2)),
            // 4xkk: Skip next instruction if Vx != kk
            &[4,x,k1,k2] => self.skip_if_not_equals_immediate(x, u8_from_nibbles(k1, k2)),
            // 5xy0: Skip next instruction if Vx = Vy
            &[5,x,y,0] => self.skip_if_equals_registers(x, y),
            // 6xkk: Put the value kk into register Vx
            &[6,x,k1,k2] => self.load_immediate(x, u8_from_nibbles(k1, k2)),
            // 7xkk: Add the value kk to register Vx
            &[7,x,k1,k2] => self.add_immediate(x, u8_from_nibbles(k1, k2)),
            // 8xy0: Store the value of register Vy in register Vx
            &[8,x,y,0] => self.move_register(x, y),
            // 8xy1: Performs a bitwise OR on the values of Vx and Vy, then stores the result in Vx
            &[8,x,y,1] => self.or_registers(x, y),
            // 8xy2: Performs a bitwise AND on the values of Vx and Vy, then stores the result in Vx
            &[8,x,y,2] => self.and_registers(x, y),
            // 8xy3: Performs a bitwise XOR on the values of Vx and Vy, then stores the result in Vx
            &[8,x,y,3] => self.xor_registers(x, y),
            // 8xy4: Set Vx = Vx + Vy, set VF = carry
            &[8,x,y,4] => self.add_registers(x, y),
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

    fn skip_if_equals_immediate(&self, reg_index: u8, value: u8) -> Chip8State {
        let mut new_state = *self;
        let reg_value = self.reg[reg_index as usize];
        // Increment PC, once more if values equal
        new_state.pc += 2;
        if reg_value == value {
            new_state.pc += 2;
        }
        new_state
    }

    fn skip_if_not_equals_immediate(&self, reg_index: u8, value: u8) -> Chip8State {
        let mut new_state = *self;
        let reg_value = self.reg[reg_index as usize];
        // Increment PC, once more if values not equal
        new_state.pc += 2;
        if reg_value != value {
            new_state.pc += 2;
        }
        new_state
    }

    fn skip_if_equals_registers(&self, reg1: u8, reg2: u8) -> Chip8State {
        let mut new_state = *self;
        let reg1_value = self.reg[reg1 as usize];
        let reg2_value = self.reg[reg2 as usize];
        // Increment PC, once more if values equal
        new_state.pc += 2;
        if reg1_value == reg2_value {
            new_state.pc += 2;
        }
        new_state
    }

    fn load_immediate(&self, reg_index: u8, value: u8) -> Chip8State {
        let mut new_state = *self;
        new_state.reg[reg_index as usize] = value;
        new_state.pc += 2;
        new_state
    }

    // TODO: overflow case ?
    fn add_immediate(&self, reg_index: u8, value: u8) -> Chip8State {
        let mut new_state = *self;
        let current = new_state.reg[reg_index as usize];
        new_state.reg[reg_index as usize] = current + value;
        new_state.pc += 2;
        new_state
    }

    fn move_register(&self, reg1: u8, reg2: u8) -> Chip8State {
        let mut new_state = *self;
        new_state.reg[reg1 as usize] = new_state.reg[reg2 as usize];
        new_state.pc += 2;
        new_state
    }

    fn or_registers(&self, reg1: u8, reg2: u8) -> Chip8State {
        let mut new_state = *self;
        new_state.reg[reg1 as usize] |= new_state.reg[reg2 as usize];
        new_state.pc += 2;
        new_state
    }

    fn and_registers(&self, reg1: u8, reg2: u8) -> Chip8State {
        let mut new_state = *self;
        new_state.reg[reg1 as usize] &= new_state.reg[reg2 as usize];
        new_state.pc += 2;
        new_state
    }

    fn xor_registers(&self, reg1: u8, reg2: u8) -> Chip8State {
        let mut new_state = *self;
        new_state.reg[reg1 as usize] ^= new_state.reg[reg2 as usize];
        new_state.pc += 2;
        new_state
    }

    // The values of Vx and Vy are added together. If the result is greater than 8 bits (>255)
    // VF is set to 1, otherwise 0. Only the lowest 8 bits of the result are kept, and stored in Vx
    fn add_registers(&self, reg1: u8, reg2: u8) -> Chip8State {
        let mut new_state = *self;
        let mut r = new_state.reg[reg1 as usize] as u16;
        r += new_state.reg[reg2 as usize] as u16;
        // take just lowest byte
        new_state.reg[reg1 as usize] = (r & 0xFF) as u8;
        // signal carry
        if r & 0xFF00 != 0 {
            new_state.reg[0xF] = 1;
        } else {
            new_state.reg[0xF] = 0;
        }
        new_state.pc += 2;
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

fn u16_from_nibbles(n0: u8, n1: u8, n2: u8, n3: u8) -> u16 {
    (((n0 as u16) << 12) | ((n1 as u16) << 8) | ((n2 as u16) << 4) | n3 as u16)
}

fn u8_from_nibbles(n0: u8, n1: u8) -> u8 {
    (n0 << 4) | n1
}
