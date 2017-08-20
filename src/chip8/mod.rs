#[cfg(test)]
mod tests;
mod display;
mod keyboard;
pub mod termui;

use byteorder::{BigEndian, ByteOrder};
use self::display::Chip8Display;
use self::keyboard::Chip8Keyboard;

// A Chip8 runtime state
#[derive(Copy)]
pub struct Chip8State {
    ram: [u8; 0x1000],              // 4K of memory
    reg: [u8; 16],                  // 16 registers
    i: u16,                         // Memory address register
    sp: u16,                        // Stack pointer
    t_delay: u8,                    // Delay timer
    t_sound: u8,                    // Sound timer
    pc: u16,                        // Program counter
    display: Chip8Display,          // Display model
    keyboard: Chip8Keyboard,        // Keyboard model
    waiting_for_key: Option<u8>,    // If waiting for key press, holds the Vx to put it. 'None' if not waiting
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
            pc: self.pc,
            display: self.display,
            keyboard: self.keyboard,
            waiting_for_key: self.waiting_for_key,
        }
    }
}

// Methods for a Chip8State
impl Chip8State {

    // Create a new Chip8State instance
    pub fn new() -> Chip8State {
        let mut state = Chip8State {
            ram: [0; 0x1000],
            reg: [0; 16],
            i: 0,
            sp: 0xEA0,
            t_delay: 0,
            t_sound: 0,
            pc: 0x200,
            display: Chip8Display::new(),
            keyboard: Chip8Keyboard::new(),
            waiting_for_key: None,
        };
        // load font sprites
        (&mut state.ram[0..FONT_SPRITES.len()]).copy_from_slice(&FONT_SPRITES);
        state
    }

    // Load data onto RAM, return new state
    pub fn load(&self, data: &[u8]) -> Chip8State {
        let mut new_state = *self;
        (&mut new_state.ram[0x200..(0x200+data.len())]).copy_from_slice(data);
        new_state
    }

    pub fn get_pixel(&self, col: u8, row: u8) -> bool {
        self.display.get_pixel(col, row)
    }

    // Read next instruction
    fn read_instruction(&self) -> Chip8Instruction {
        let pointer = &self.ram[(self.pc as usize) ..];
        let word = BigEndian::read_u16(pointer);
        Chip8Instruction(word)
    }

    // Execute next instruction, returning a new state
    pub fn exec_instruction(&self) -> Chip8State {
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
            &[5,x,y,0] => self.skip_if_registers_equal(x, y),
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
            // 8xy5: Set Vx = Vx - Vy, set VF = (Vx>Vy)
            &[8,x,y,5] => self.sub_registers(x, y),
            // 8xy6: Set Vx = Vx SHR 1
            &[8,x,_,6] => self.shr_register(x),
            // 8xy7: Set Vx = Vy - Vx, set VF = (Vy>Vx)
            &[8,x,y,7] => self.subn_registers(x, y),
            // 8xyE: Set Vx = Vx SHL 1
            &[8,x,_,0xE] => self.shl_register(x),
            // 9xy0: Skip next instruction if Vx != Vy
            &[9,x,y,0] => self.skip_if_registers_not_equal(x, y),
            // Annn: Set I = nnn
            &[0xA,n1,n2,n3] => self.set_i(u16_from_nibbles(0, n1, n2, n3)),
            // Bnnn: Jump to location nnn + V0
            &[0xB,n1,n2,n3] => self.indexed_jump(u16_from_nibbles(0, n1, n2, n3)),
            // Cxkk: Set Vx = random byte AND kk
            &[0xC,x,k1,k2] => self.masked_random(x, u8_from_nibbles(k1, k2)),
            // Dxyn: Display n-byte sprite starting at memory location I at (Vx, Vy)
            &[0xD,x,y,n] => self.draw_sprite(x, y, n),
            // Ex9E: Skip next instruction if key with the value of Vx is pressed
            &[0xE,x,9,0xE] => self.skip_if_key_down(x),
            // ExA1: Skip next instruction if key with the value of Vx is NOT pressed
            &[0xE,x,0xA,0x1] => self.skip_if_key_up(x),
            // Fx07: Skip next instruction if key with the value of Vx is NOT pressed
            &[0xF,x,0,7] => self.move_delay_timer_value_to_register(x),
            // Fx0A: Wait for a key press, store the value of the key in Vx
            &[0xF,x,0,0xA] => self.wait_for_key(x),
            // Fx15: Set delay timer = Vx
            &[0xF,x,1,5] => self.set_delay_timer(x),
            // Fx18: Set delay timer = Vx
            &[0xF,x,1,8] => self.set_sound_timer(x),
            // Fx1E: Set I = I + Vx
            &[0xF,x,1,0xE] => self.add_register_to_i(x),
            // Fx29: Set I = location of sprite for digit Vx
            &[0xF,x,2,9] => self.set_sprite_location(x),
            // Fx33: Store BCD representation of Vx in memory locations I, I+1, and I+2
            &[0xF,x,3,3] => self.binary_coded_decimal_conversion(x),
            // Fx55: Store registers V0 through Vx in memory starting at location I
            &[0xF,x,5,5] => self.dump_registers_up_to(x),
            // Fx65: Read registers V0 through Vx from memory starting at location I
            &[0xF,x,6,5] => self.load_registers_up_to(x),
            // Panic if unknown
            _ => panic!("Unknown instruction: {:?}", op)
        }
    }

    fn clear_screen(&self) -> Chip8State {
        let mut new_state = *self;
        new_state.display = display::Chip8Display::new();
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

    fn indexed_jump(&self, addr: u16) -> Chip8State {
        let mut new_state = *self;
        new_state.pc = addr + new_state.reg[0] as u16;
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

    fn skip_if_registers_equal(&self, vx: u8, vy: u8) -> Chip8State {
        let mut new_state = *self;
        let vx_value = self.reg[vx as usize];
        let vy_value = self.reg[vy as usize];
        // Increment PC, once more if values equal
        new_state.pc += 2;
        if vx_value == vy_value {
            new_state.pc += 2;
        }
        new_state
    }

    fn skip_if_registers_not_equal(&self, vx: u8, vy: u8) -> Chip8State {
        let mut new_state = *self;
        let vx_value = self.reg[vx as usize];
        let vy_value = self.reg[vy as usize];
        // Increment PC, once more if values not equal
        new_state.pc += 2;
        if vx_value != vy_value {
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

    fn move_register(&self, vx: u8, vy: u8) -> Chip8State {
        let mut new_state = *self;
        new_state.reg[vx as usize] = new_state.reg[vy as usize];
        new_state.pc += 2;
        new_state
    }

    fn or_registers(&self, vx: u8, vy: u8) -> Chip8State {
        let mut new_state = *self;
        new_state.reg[vx as usize] |= new_state.reg[vy as usize];
        new_state.pc += 2;
        new_state
    }

    fn and_registers(&self, vx: u8, vy: u8) -> Chip8State {
        let mut new_state = *self;
        new_state.reg[vx as usize] &= new_state.reg[vy as usize];
        new_state.pc += 2;
        new_state
    }

    fn xor_registers(&self, vx: u8, vy: u8) -> Chip8State {
        let mut new_state = *self;
        new_state.reg[vx as usize] ^= new_state.reg[vy as usize];
        new_state.pc += 2;
        new_state
    }

    // The values of Vx and Vy are added together. If the result is greater than 8 bits (>255)
    // VF is set to 1, otherwise 0. Only the lowest 8 bits of the result are kept, and stored in Vx
    fn add_registers(&self, vx: u8, vy: u8) -> Chip8State {
        let mut new_state = *self;
        let r1 = new_state.reg[vx as usize];
        let r2 = new_state.reg[vy as usize];
        let (sum, carry) = r1.overflowing_add(r2);
        new_state.reg[vx as usize] = sum;
        new_state.reg[0xF] = carry as u8;
        new_state.pc += 2;
        new_state
    }

    // If Vx > Vy, then VF is set to 1, otherwise 0. Then Vy is subtracted from Vx,
    // and the results stored in Vx
    fn sub_registers(&self, vx: u8, vy: u8) -> Chip8State {
        let mut new_state = *self;
        let x = new_state.reg[vx as usize];
        let y = new_state.reg[vy as usize];
        new_state.reg[0xF] = (x > y) as u8;
        let (sub, _) = x.overflowing_sub(y);
        new_state.reg[vx as usize] = sub;
        new_state.pc += 2;
        new_state
    }

    // If Vy > Vx, then VF is set to 1, otherwise 0. Then Vx is subtracted from Vy,
    // and the results stored in Vx
    fn subn_registers(&self, vx: u8, vy: u8) -> Chip8State {
        let mut new_state = *self;
        let x = new_state.reg[vx as usize];
        let y = new_state.reg[vy as usize];
        new_state.reg[0xF] = (y > x) as u8;
        let (sub, _) = y.overflowing_sub(x);
        new_state.reg[vx as usize] = sub;
        new_state.pc += 2;
        new_state
    }

    // VF is set to the least-significant bit of Vx. Then Vx is divided by 2.
    fn shr_register(&self, vx: u8) -> Chip8State {
        let mut new_state = *self;
        let x = new_state.reg[vx as usize];
        new_state.reg[0xF] = x & 1;
        new_state.reg[vx as usize] = x >> 1;
        new_state.pc += 2;
        new_state
    }

    // VF is set to the most-significant bit of Vx. Then Vx is multiplied by 2.
    fn shl_register(&self, vx: u8) -> Chip8State {
        let mut new_state = *self;
        let x = new_state.reg[vx as usize];
        new_state.reg[0xF] = x >> 7;
        new_state.reg[vx as usize] = x << 1;
        new_state.pc += 2;
        new_state
    }

    fn set_i(&self, n: u16) -> Chip8State {
        let mut new_state = *self;
        new_state.i = n;
        new_state.pc += 2;
        new_state
    }

    fn masked_random(&self, vx: u8, mask: u8) -> Chip8State {
        let mut new_state = *self;
        let random = ::rand::random::<u8>();
        new_state.reg[vx as usize] = random & mask;
        new_state.pc += 2;
        new_state
    }

    fn draw_sprite(&self, x: u8, y: u8, n: u8) -> Chip8State {
        let mut new_state = *self;
        // get coordinates
        let col = new_state.reg[x as usize];
        let row = new_state.reg[y as usize];
        // load sprite
        let sprite_begin = new_state.i as usize;
        let sprite_end = (new_state.i+(n as u16)) as usize;
        let sprite = &new_state.ram[sprite_begin..sprite_end];
        // draw
        let collision = new_state.display.draw_sprite(col, row, sprite);
        // set vF
        new_state.reg[0xF] = collision as u8;
        // increase PC
        new_state.pc += 2;
        new_state
    }

    fn skip_if_key_down(&self, x: u8) -> Chip8State {
        let mut new_state = *self;
        let key = new_state.reg[x as usize];
        new_state.pc += 2;
        if new_state.is_key_down(key) {
            new_state.pc += 2;
        }
        new_state
    }

    fn skip_if_key_up(&self, x: u8) -> Chip8State {
        let mut new_state = *self;
        let key = new_state.reg[x as usize];
        new_state.pc += 2;
        if !new_state.is_key_down(key) {
            new_state.pc += 2;
        }
        new_state
    }

    fn is_key_down(&self, key: u8) -> bool {
        self.keyboard.is_key_pressed(key)
    }

    // handler for 'key pressed' events
    fn key_down(&mut self, key: u8) -> Chip8State {
        let mut new_state = *self;
        new_state.keyboard.key_pressed(key);
        match new_state.waiting_for_key {
            Some(x) => {
                new_state.reg[x as usize] = key;
                new_state.pc += 2;
                new_state.waiting_for_key = None;
            }
            _ => {}
        }
        new_state
    }

    // handler for 'key released' events
    fn key_up(&mut self, key: u8) -> Chip8State {
        let mut new_state = *self;
        new_state.keyboard.key_released(key);
        new_state
    }

    fn move_delay_timer_value_to_register(&self, x: u8) -> Chip8State {
        let mut new_state = *self;
        new_state.reg[x as usize] = new_state.t_delay;
        new_state.pc += 2;
        new_state
    }

    fn wait_for_key(&self, x: u8) -> Chip8State {
        let mut new_state = *self;
        new_state.waiting_for_key = Some(x);
        new_state
    }

    fn set_delay_timer(&self, x: u8) -> Chip8State {
        let mut new_state = *self;
        new_state.t_delay = new_state.reg[x as usize];
        new_state.pc += 2;
        new_state
    }

    fn set_sound_timer(&self, x: u8) -> Self {
        let mut new_state = *self;
        new_state.t_sound = new_state.reg[x as usize];
        new_state.pc += 2;
        new_state
    }

    fn add_register_to_i(&self, x: u8) -> Self {
        let mut new_state = *self;
        let vx = new_state.reg[x as usize];
        new_state.i += vx as u16;
        new_state.pc += 2;
        new_state
    }

    fn set_sprite_location(&self, x: u8) -> Self {
        let mut new_state = *self;
        let vx = new_state.reg[x as usize];
        new_state.i = (vx as u16) * 5;
        new_state.pc += 2;
        new_state
    }

    fn binary_coded_decimal_conversion(&self, x: u8) -> Self {
        let mut new_state = *self;
        let target_address = new_state.i as usize;
        let vx = new_state.reg[x as usize];
        let units = vx % 10;
        let tens = (vx / 10) % 10;
        let hundreds = (vx / 100) % 10;
        new_state.ram[target_address] = hundreds;
        new_state.ram[target_address+1] = tens;
        new_state.ram[target_address+2] = units;
        new_state.pc += 2;
        new_state
    }

    fn dump_registers_up_to(&self, x: u8) -> Self {
        let mut new_state = *self;
        {
            let target_address = new_state.i as usize;
            let target_ram_range = target_address..(1 + target_address + x as usize);
            let mut target_ram_slice = &mut new_state.ram[target_ram_range];
            let regs_slice = &new_state.reg[0..(1 + x as usize)];
            target_ram_slice.copy_from_slice(regs_slice);
        }
        new_state.pc += 2;
        new_state
    }

    fn load_registers_up_to(&self, x: u8) -> Self {
        let mut new_state = *self;
        {
            let source_address = new_state.i as usize;
            let source_ram_range = source_address..(1 + source_address + x as usize);
            let source_ram_slice = &new_state.ram[source_ram_range];
            let regs_slice = &mut new_state.reg[0..(1 + x as usize)];
            regs_slice.copy_from_slice(source_ram_slice);
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

const FONT_SPRITES: [u8; 80] = [
    // "0"	Binary	Hex
    0xF0,    0x90,    0x90,    0x90,    0xF0,
    // "1"	Binary	Hex
    0x20,    0x60,    0x20,    0x20,    0x70,
    // "2"	Binary	Hex
    0xF0,    0x10,    0xF0,    0x80,    0xF0,
    // "3"	Binary	Hex
    0xF0,    0x10,    0xF0,    0x10,    0xF0,
    // "4"	Binary	Hex
    0x90,    0x90,    0xF0,    0x10,    0x10,
    // "5"	Binary	Hex
    0xF0,    0x80,    0xF0,    0x10,    0xF0,
    // "6"	Binary	Hex
    0xF0,    0x80,    0xF0,    0x90,    0xF0,
    // "7"	Binary	Hex
    0xF0,    0x10,    0x20,    0x40,    0x40,
    // "8"	Binary	Hex
    0xF0,    0x90,    0xF0,    0x90,    0xF0,
    // "9"	Binary	Hex
    0xF0,    0x90,    0xF0,    0x10,    0xF0,
    // "A"	Binary	Hex
    0xF0,    0x90,    0xF0,    0x90,    0x90,
    // "B"	Binary	Hex
    0xE0,    0x90,    0xE0,    0x90,    0xE0,
    // "C"	Binary	Hex
    0xF0,    0x80,    0x80,    0x80,    0xF0,
    // "D"	Binary	Hex
    0xE0,    0x90,    0x90,    0x90,    0xE0,
    // "E"	Binary	Hex
    0xF0,    0x80,    0xF0,    0x80,    0xF0,
    // "F"	Binary	Hex
    0xF0,    0x80,    0xF0,    0x80,    0x80,
];
