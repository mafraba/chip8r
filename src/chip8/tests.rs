use super::*;

#[test]
fn load_data() {
    let ch8state1 = Chip8State::new();
    let data: &[u8] = &[1,2,3,4];
    let ch8state2 = ch8state1.load(data);

    assert_eq!(ch8state1.ram[0x200], 0);
    assert_eq!(ch8state1.ram[0x201], 0);
    assert_eq!(ch8state1.ram[0x202], 0);
    assert_eq!(ch8state1.ram[0x203], 0);

    assert_eq!(ch8state2.ram[0x199], 0);
    assert_eq!(ch8state2.ram[0x200], 1);
    assert_eq!(ch8state2.ram[0x201], 2);
    assert_eq!(ch8state2.ram[0x202], 3);
    assert_eq!(ch8state2.ram[0x203], 4);
    assert_eq!(ch8state2.ram[0x204], 0);
}

#[test]
fn read_instruction() {
    let mut ch8state = Chip8State::new();
    let data: &[u8] = &[1,2];
    ch8state = ch8state.load(data);
    assert_eq!(ch8state.read_instruction(), Chip8Instruction(0x0102));
}

#[test]
fn nibble_extraction() {
    let op = Chip8Instruction(0xABCD);
    assert_eq!(op.nibble(1), 0x0A);
    assert_eq!(op.nibble(2), 0x0B);
    assert_eq!(op.nibble(3), 0x0C);
    assert_eq!(op.nibble(4), 0x0D);
}

#[test]
fn clear_screen_instruction() {
    let mut ch8state = Chip8State::new();
    // put some garbage on display buffer
    {
        let alien_sprite = &[0xBA, 0x7C, 0xD6, 0xFE, 0x54, 0xAA];
        // put the sprite in 0x300
        (&mut ch8state.ram[0x300..(0x300+alien_sprite.len())]).copy_from_slice(alien_sprite);
        // set I to 0x300
        ch8state.i = 0x300;
        // load instruction to draw + to clear
        ch8state = ch8state.load(&[0xD0, 0x16, 0x00, 0xE0]);
        // run draw
        ch8state = ch8state.exec_instruction();
        assert_eq!(ch8state.display.get_pixel(0,0), true);
    }
    // run cls
    assert_eq!(ch8state.read_instruction(), Chip8Instruction(0x00E0));
    ch8state = ch8state.exec_instruction();
    for c in 0..64 {
        for r in 0..32 {
            assert_eq!(ch8state.display.get_pixel(c,r), false);
        }
    }
}

#[test]
fn read_return_address() {
    let mut ch8state = Chip8State::new();
    // set some ret address
    ch8state.ram[ch8state.sp as usize] = 0x0A;
    ch8state.ram[(ch8state.sp+1) as usize] = 0xBC;
    // check return
    assert_eq!(ch8state.read_return_address(), 0x0ABC);
}

#[test]
fn return_from_subroutine_instruction() {
    let mut ch8state = Chip8State::new();
    let initial_sp = ch8state.sp;
    // set some ret address
    ch8state.ram[ch8state.sp as usize] = 0x0A;
    ch8state.ram[(ch8state.sp+1) as usize] = 0xBC;
    // load return instruction and execute
    ch8state = ch8state.load(&[0x00,0xEE]);
    ch8state = ch8state.exec_instruction();
    // check state
    assert_eq!(ch8state.sp, initial_sp-2);
    assert_eq!(ch8state.pc, 0x0ABC);
}

#[test]
fn jump_instruction() {
    let mut ch8state = Chip8State::new();
    // load return instruction and execute
    ch8state = ch8state.load(&[0x1A,0xBC]);
    ch8state = ch8state.exec_instruction();
    // check state
    assert_eq!(ch8state.pc, 0x0ABC);
}

#[test]
fn indexed_jump_instruction() {
    let mut ch8state = Chip8State::new();
    // load return instruction and execute
    ch8state = ch8state.load(&[0xBA,0xB0]);
    ch8state.reg[0] = 0xC;
    ch8state = ch8state.exec_instruction();
    // check state
    assert_eq!(ch8state.pc, 0x0ABC);
}

#[test]
fn call_instruction() {
    let initial_ch8state = Chip8State::new();
    // load return instruction and execute
    let ch8state1 = initial_ch8state.load(&[0x2A,0xBC]);
    let ch8state2 = ch8state1.exec_instruction();
    // check state
    assert_eq!(ch8state2.sp, ch8state1.sp+2, "Incorrect SP register value");
    assert_eq!(ch8state2.read_return_address(), ch8state1.pc+2, "Incorrect return address");
    assert_eq!(ch8state2.pc, 0xABC, "Incorrect PC value");
}

#[test]
fn skip_if_equals_immediate_positive() {
    let mut ch8state = Chip8State::new();
    // load return instruction and execute
    ch8state = ch8state.load(&[0x30,0x12]);
    ch8state.reg[0] = 0x12;
    let pc_pre = ch8state.pc;
    ch8state = ch8state.exec_instruction();
    // check state
    assert_eq!(ch8state.pc, pc_pre+4, "Incorrect PC register value");
}

#[test]
fn skip_if_equals_immediate_negative() {
    let mut ch8state = Chip8State::new();
    // load return instruction and execute
    ch8state = ch8state.load(&[0x30,0x12]);
    // ch8state.reg[0] = 0x00;
    let pc_pre = ch8state.pc;
    ch8state = ch8state.exec_instruction();
    // check state
    assert_eq!(ch8state.pc, pc_pre+2, "Incorrect PC register value");
}

#[test]
fn skip_if_not_equals_immediate_positive() {
    let mut ch8state = Chip8State::new();
    // load return instruction and execute
    ch8state = ch8state.load(&[0x40,0x12]);
    let pc_pre = ch8state.pc;
    ch8state = ch8state.exec_instruction();
    // check state
    assert_eq!(ch8state.pc, pc_pre+4, "Incorrect PC register value");
}

#[test]
fn skip_if_not_equals_immediate_negative() {
    let mut ch8state = Chip8State::new();
    // load return instruction and execute
    ch8state = ch8state.load(&[0x40,0x12]);
    ch8state.reg[0] = 0x12;
    let pc_pre = ch8state.pc;
    ch8state = ch8state.exec_instruction();
    // check state
    assert_eq!(ch8state.pc, pc_pre+2, "Incorrect PC register value");
}

#[test]
fn skip_if_equals_registers_positive() {
    let mut ch8state = Chip8State::new();
    // load return instruction and execute
    ch8state = ch8state.load(&[0x50,0x10]);
    let pc_pre = ch8state.pc;
    ch8state = ch8state.exec_instruction();
    // check state
    assert_eq!(ch8state.pc, pc_pre+4, "Incorrect PC register value");
}

#[test]
fn skip_if_equals_registers_negative() {
    let mut ch8state = Chip8State::new();
    // load return instruction and execute
    ch8state = ch8state.load(&[0x50,0x10]);
    ch8state.reg[0] = 1;
    let pc_pre = ch8state.pc;
    ch8state = ch8state.exec_instruction();
    // check state
    assert_eq!(ch8state.pc, pc_pre+2, "Incorrect PC register value");
}

#[test]
fn skip_if_registers_not_equal_positive() {
    let mut ch8state = Chip8State::new();
    // load return instruction and execute
    ch8state = ch8state.load(&[0x90,0x10]);
    ch8state.reg[0] = 1;
    let pc_pre = ch8state.pc;
    ch8state = ch8state.exec_instruction();
    // check state
    assert_eq!(ch8state.pc, pc_pre+4, "Incorrect PC register value");
}

#[test]
fn skip_if_registers_not_equal_negative() {
    let mut ch8state = Chip8State::new();
    // load return instruction and execute
    ch8state = ch8state.load(&[0x90,0x10]);
    let pc_pre = ch8state.pc;
    ch8state = ch8state.exec_instruction();
    // check state
    assert_eq!(ch8state.pc, pc_pre+2, "Incorrect PC register value");
}

#[test]
fn skip_if_key_pressed_negative_case() {
    let mut ch8state = Chip8State::new();
    // load return instruction and execute
    ch8state = ch8state.load(&[0xE0,0x9E]);
    let pc_pre = ch8state.pc;
    ch8state = ch8state.exec_instruction();
    // check state
    assert_eq!(ch8state.pc, pc_pre+2, "Incorrect PC register value");
}

#[test]
fn skip_if_key_pressed_negative_case_after_release() {
    let mut ch8state = Chip8State::new();
    // load return instruction and execute
    ch8state = ch8state.load(&[0xE0,0x9E]);
    let pc_pre = ch8state.pc;
    ch8state = ch8state.key_down(0);
    ch8state = ch8state.key_up(0);
    ch8state = ch8state.exec_instruction();
    // check state
    assert_eq!(ch8state.pc, pc_pre+2, "Incorrect PC register value");
}

#[test]
fn skip_if_key_pressed_positive_case() {
    let mut ch8state = Chip8State::new();
    // load return instruction and execute
    ch8state = ch8state.load(&[0xE1,0x9E]);
    ch8state.reg[1] = 0xF;
    let pc_pre = ch8state.pc;
    ch8state = ch8state.key_down(0xF);
    ch8state = ch8state.exec_instruction();
    // check state
    assert_eq!(ch8state.pc, pc_pre+4, "Incorrect PC register value");
}

#[test]
fn skip_if_key_not_pressed_when_pressed() {
    let mut ch8state = Chip8State::new();
    // load return instruction and execute
    ch8state = ch8state.load(&[0xE1,0xA1]);
    ch8state.reg[1] = 0xF;
    let pc_pre = ch8state.pc;
    ch8state = ch8state.key_down(0xF);
    ch8state = ch8state.exec_instruction();
    // check state
    assert_eq!(ch8state.pc, pc_pre+2, "Incorrect PC register value");
}

#[test]
fn skip_if_key_not_pressed_when_not_pressed() {
    let mut ch8state = Chip8State::new();
    // load return instruction and execute
    ch8state = ch8state.load(&[0xE1,0xA1]);
    ch8state.reg[1] = 0xF;
    let pc_pre = ch8state.pc;
    ch8state = ch8state.exec_instruction();
    // check state
    assert_eq!(ch8state.pc, pc_pre+4, "Incorrect PC register value");
}

#[test]
fn skip_if_key_not_pressed_when_released() {
    let mut ch8state = Chip8State::new();
    // load return instruction and execute
    ch8state = ch8state.load(&[0xE1,0xA1]);
    let pc_pre = ch8state.pc;
    ch8state = ch8state.key_down(0);
    ch8state = ch8state.key_up(0);
    ch8state = ch8state.exec_instruction();
    // check state
    assert_eq!(ch8state.pc, pc_pre+4, "Incorrect PC register value");
}

#[test]
fn load_immediate() {
    let mut ch8state = Chip8State::new();
    // load return instruction and execute
    ch8state = ch8state.load(&[0x60,0x12]);
    assert_eq!(ch8state.reg[0], 0);
    let pc_pre = ch8state.pc;
    ch8state = ch8state.exec_instruction();
    // check state
    assert_eq!(ch8state.pc, pc_pre+2, "Incorrect program counter");
    assert_eq!(ch8state.reg[0], 0x12, "Incorrect register value");
}

#[test]
fn add_immediate() {
    let mut ch8state = Chip8State::new();
    // load return instruction and execute
    ch8state = ch8state.load(&[0x70,0x12]);
    ch8state.reg[0] = 1;
    let pc_pre = ch8state.pc;
    ch8state = ch8state.exec_instruction();
    // check state
    assert_eq!(ch8state.pc, pc_pre+2, "Incorrect program counter");
    assert_eq!(ch8state.reg[0], 0x13, "Incorrect register value");
}

#[test]
fn move_register() {
    let mut ch8state = Chip8State::new();
    // load return instruction and execute
    ch8state = ch8state.load(&[0x81,0x00]);
    ch8state.reg[0] = 1;
    assert_eq!(ch8state.reg[1], 0, "Incorrect register value");
    let pc_pre = ch8state.pc;
    ch8state = ch8state.exec_instruction();
    // check state
    assert_eq!(ch8state.pc, pc_pre+2, "Incorrect program counter");
    assert_eq!(ch8state.reg[0], 1, "Incorrect register value");
    assert_eq!(ch8state.reg[1], 1, "Incorrect register value");
}

#[test]
fn or_register() {
    let mut ch8state = Chip8State::new();
    // load return instruction and execute
    ch8state = ch8state.load(&[0x81,0x01]);
    ch8state.reg[0] = 0x03;
    ch8state.reg[1] = 0x11;
    let pc_pre = ch8state.pc;
    ch8state = ch8state.exec_instruction();
    // check state
    assert_eq!(ch8state.pc, pc_pre+2, "Incorrect program counter");
    assert_eq!(ch8state.reg[0], 0x03, "Incorrect register value");
    assert_eq!(ch8state.reg[1], 0x13, "Incorrect register value");
}

#[test]
fn and_register() {
    let mut ch8state = Chip8State::new();
    // load return instruction and execute
    ch8state = ch8state.load(&[0x81,0x02]);
    ch8state.reg[0] = 0x03;
    ch8state.reg[1] = 0x11;
    let pc_pre = ch8state.pc;
    ch8state = ch8state.exec_instruction();
    // check state
    assert_eq!(ch8state.pc, pc_pre+2, "Incorrect program counter");
    assert_eq!(ch8state.reg[0], 0x03, "Incorrect register value");
    assert_eq!(ch8state.reg[1], 0x01, "Incorrect register value");
}

#[test]
fn xor_register() {
    let mut ch8state = Chip8State::new();
    // load return instruction and execute
    ch8state = ch8state.load(&[0x81,0x03]);
    ch8state.reg[0] = 0x03;
    ch8state.reg[1] = 0x11;
    let pc_pre = ch8state.pc;
    ch8state = ch8state.exec_instruction();
    // check state
    assert_eq!(ch8state.pc, pc_pre+2, "Incorrect program counter");
    assert_eq!(ch8state.reg[0], 0x03, "Incorrect register value");
    assert_eq!(ch8state.reg[1], 0x12, "Incorrect register value");
}

#[test]
fn add_registers_no_carry() {
    let mut ch8state = Chip8State::new();
    // load return instruction and execute
    ch8state = ch8state.load(&[0x81,0x04]);
    ch8state.reg[0] = 0x03;
    ch8state.reg[1] = 0x11;
    let pc_pre = ch8state.pc;
    ch8state = ch8state.exec_instruction();
    // check state
    assert_eq!(ch8state.pc, pc_pre+2, "Incorrect program counter");
    assert_eq!(ch8state.reg[0], 0x03, "Incorrect register value");
    assert_eq!(ch8state.reg[1], 0x14, "Incorrect register value");
    assert_eq!(ch8state.reg[0xF], 0, "Incorrect register value");
}

#[test]
fn add_registers_carry() {
    let mut ch8state = Chip8State::new();
    // load return instruction and execute
    ch8state = ch8state.load(&[0x81,0x04]);
    ch8state.reg[0] = 0xFF;
    ch8state.reg[1] = 0x03;
    let pc_pre = ch8state.pc;
    ch8state = ch8state.exec_instruction();
    // check state
    assert_eq!(ch8state.pc, pc_pre+2, "Incorrect program counter");
    assert_eq!(ch8state.reg[0], 0xFF, "Incorrect register value");
    assert_eq!(ch8state.reg[1], 0x02, "Incorrect register value");
    assert_eq!(ch8state.reg[0xF], 1, "Incorrect register value");
}

#[test]
fn sub_registers_borrow() {
    let mut ch8state = Chip8State::new();
    // load return instruction and execute
    ch8state = ch8state.load(&[0x81,0x05]);
    ch8state.reg[0] = 1;
    ch8state.reg[1] = 0;
    let pc_pre = ch8state.pc;
    ch8state = ch8state.exec_instruction();
    // check state
    assert_eq!(ch8state.pc, pc_pre+2, "Incorrect program counter");
    assert_eq!(ch8state.reg[0], 1, "Incorrect register value");
    assert_eq!(ch8state.reg[1], 0xFF, "Incorrect register value");
    assert_eq!(ch8state.reg[0xF], 0, "Incorrect register value");
}

#[test]
fn sub_registers_no_borrow() {
    let mut ch8state = Chip8State::new();
    // load return instruction and execute
    ch8state = ch8state.load(&[0x81,0x05]);
    ch8state.reg[0] = 1;
    ch8state.reg[1] = 3;
    let pc_pre = ch8state.pc;
    ch8state = ch8state.exec_instruction();
    // check state
    assert_eq!(ch8state.pc, pc_pre+2, "Incorrect program counter");
    assert_eq!(ch8state.reg[0], 1, "Incorrect register value");
    assert_eq!(ch8state.reg[1], 2, "Incorrect register value");
    assert_eq!(ch8state.reg[0xF], 1, "Incorrect register value");
}

#[test]
fn shr_register_no_carry() {
    let mut ch8state = Chip8State::new();
    // load return instruction and execute
    ch8state = ch8state.load(&[0x80,0x06]);
    ch8state.reg[0] = 2;
    let pc_pre = ch8state.pc;
    ch8state = ch8state.exec_instruction();
    // check state
    assert_eq!(ch8state.pc, pc_pre+2, "Incorrect program counter");
    assert_eq!(ch8state.reg[0], 1, "Incorrect register value");
    assert_eq!(ch8state.reg[0xF], 0, "Incorrect register value");
}

#[test]
fn shr_register_carry() {
    let mut ch8state = Chip8State::new();
    // load return instruction and execute
    ch8state = ch8state.load(&[0x80,0x06]);
    ch8state.reg[0] = 3;
    let pc_pre = ch8state.pc;
    ch8state = ch8state.exec_instruction();
    // check state
    assert_eq!(ch8state.pc, pc_pre+2, "Incorrect program counter");
    assert_eq!(ch8state.reg[0], 1, "Incorrect register value");
    assert_eq!(ch8state.reg[0xF], 1, "Incorrect register value");
}

#[test]
fn subn_registers_borrow() {
    let mut ch8state = Chip8State::new();
    // load return instruction and execute
    ch8state = ch8state.load(&[0x80,0x17]);
    ch8state.reg[0] = 1;
    ch8state.reg[1] = 0;
    let pc_pre = ch8state.pc;
    ch8state = ch8state.exec_instruction();
    // check state
    assert_eq!(ch8state.pc, pc_pre+2, "Incorrect program counter");
    assert_eq!(ch8state.reg[1], 0, "Incorrect register value");
    assert_eq!(ch8state.reg[0], 0xFF, "Incorrect register value");
    assert_eq!(ch8state.reg[0xF], 0, "Incorrect register value");
}

#[test]
fn subn_registers_no_borrow() {
    let mut ch8state = Chip8State::new();
    // load return instruction and execute
    ch8state = ch8state.load(&[0x80,0x17]);
    ch8state.reg[0] = 1;
    ch8state.reg[1] = 2;
    let pc_pre = ch8state.pc;
    ch8state = ch8state.exec_instruction();
    // check state
    assert_eq!(ch8state.pc, pc_pre+2, "Incorrect program counter");
    assert_eq!(ch8state.reg[0], 1, "Incorrect register value");
    assert_eq!(ch8state.reg[1], 2, "Incorrect register value");
    assert_eq!(ch8state.reg[0xF], 1, "Incorrect register value");
}

#[test]
fn shl_register_no_carry() {
    let mut ch8state = Chip8State::new();
    // load instruction and execute
    ch8state = ch8state.load(&[0x80,0x0E]);
    ch8state.reg[0] = 2;
    let pc_pre = ch8state.pc;
    ch8state = ch8state.exec_instruction();
    // check state
    assert_eq!(ch8state.pc, pc_pre+2, "Incorrect program counter");
    assert_eq!(ch8state.reg[0], 4, "Incorrect register value");
    assert_eq!(ch8state.reg[0xF], 0, "Incorrect register value");
}

#[test]
fn shl_register_carry() {
    let mut ch8state = Chip8State::new();
    // load instruction and execute
    ch8state = ch8state.load(&[0x80,0x0E]);
    ch8state.reg[0] = 0x83;
    let pc_pre = ch8state.pc;
    ch8state = ch8state.exec_instruction();
    // check state
    assert_eq!(ch8state.pc, pc_pre+2, "Incorrect program counter");
    assert_eq!(ch8state.reg[0], 6, "Incorrect register value");
    assert_eq!(ch8state.reg[0xF], 1, "Incorrect register value");
}

#[test]
fn set_i() {
    let mut ch8state = Chip8State::new();
    // load instruction and execute
    ch8state = ch8state.load(&[0xA1,0x23]);
    assert_eq!(ch8state.i, 0);
    let pc_pre = ch8state.pc;
    ch8state = ch8state.exec_instruction();
    // check state
    assert_eq!(ch8state.pc, pc_pre+2, "Incorrect program counter");
    assert_eq!(ch8state.i, 0x123, "Incorrect register value");
}

#[test]
fn masked_random() {
    let mut ch8state = Chip8State::new();
    // load instruction and execute
    ch8state = ch8state.load(&[0xC0,0x0F]);
    ch8state.reg[0] = 0xFF;
    let pc_pre = ch8state.pc;
    ch8state = ch8state.exec_instruction();
    // check state
    assert_eq!(ch8state.pc, pc_pre+2, "Incorrect program counter");
    assert!(ch8state.reg[0] <= 0xF, "Incorrect register value");
}

#[test]
fn draw_sprite_no_collision() {
    let mut ch8state = Chip8State::new();
    // Sprite map   Binary      Hex
    // X.XXX.X.     0b10111010  $BA
    // .XXXXX..     0b01111100  $7C
    // XX.X.XX.     0b11010110  $D6
    // XXXXXXX.     0b11111110  $FE
    // .X.X.X..     0b01010100  $54
    // X.X.X.X.     0b10101010  $AA
    let alien_sprite = &[0xBA, 0x7C, 0xD6, 0xFE, 0x54, 0xAA];
    // put the sprite in 0x300
    (&mut ch8state.ram[0x300..(0x300+alien_sprite.len())]).copy_from_slice(alien_sprite);
    // set I to 0x300
    ch8state.i = 0x300;
    // set coordinates to 0,0
    ch8state.reg[0] = 0;
    ch8state.reg[1] = 0;
    // load instruction
    let drw_instruction = &[0xD0,0x16]; // 6-bytes sprite, draw at (V0,V1)
    ch8state = ch8state.load(drw_instruction);
    // run it
    let pc_pre = ch8state.pc;
    ch8state = ch8state.exec_instruction();
    assert_eq!(ch8state.pc, pc_pre+2, "Incorrect program counter");
    // check just some pixels, e.g. a sprite diagonal
    assert_eq!(ch8state.display.get_pixel(0,0), true, "Incorrect register value");
    assert_eq!(ch8state.display.get_pixel(1,1), true, "Incorrect register value");
    assert_eq!(ch8state.display.get_pixel(2,2), false, "Incorrect register value");
    assert_eq!(ch8state.display.get_pixel(3,3), true, "Incorrect register value");
    assert_eq!(ch8state.display.get_pixel(4,4), false, "Incorrect register value");
    assert_eq!(ch8state.display.get_pixel(5,5), false, "Incorrect register value");
    // check VF for collision indicator
    assert_eq!(ch8state.reg[0xF], 0, "Incorrect collision detection");
}

#[test]
fn draw_sprite_with_collision() {
    let mut ch8state = Chip8State::new();
    let alien_sprite = &[0xBA, 0x7C, 0xD6, 0xFE, 0x54, 0xAA];
    // put the sprite in 0x300
    (&mut ch8state.ram[0x300..(0x300+alien_sprite.len())]).copy_from_slice(alien_sprite);
    // set I to 0x300
    ch8state.i = 0x300;
    // set coordinates to 0,0
    ch8state.reg[0] = 0;
    ch8state.reg[1] = 0;
    ch8state = ch8state.load(&[0xD0,0x16,0xD0,0x16]);
    // run it
    let pc_pre = ch8state.pc;
    ch8state = ch8state.exec_instruction();
    assert_eq!(ch8state.pc, pc_pre+2, "Incorrect program counter");
    assert_eq!(ch8state.reg[0xF], 0, "Incorrect collision detection");
    ch8state = ch8state.exec_instruction();
    assert_eq!(ch8state.pc, pc_pre+4, "Incorrect program counter");
    assert_eq!(ch8state.reg[0xF], 1, "Incorrect collision detection");
}

#[test]
fn move_delay_timer_value_to_register() {
    let mut ch8state = Chip8State::new();
    ch8state.t_delay = 123;
    ch8state = ch8state.load(&[0xF2,0x07]);
    let pc_pre = ch8state.pc;
    ch8state = ch8state.exec_instruction();
    assert_eq!(ch8state.pc, pc_pre+2, "Incorrect program counter");
    assert_eq!(ch8state.reg[2], 123);
}

#[test]
fn wait_for_key() {
    let mut ch8state = Chip8State::new();
    ch8state = ch8state.load(&[0xF3,0x0A]);
    let pc_pre = ch8state.pc;
    ch8state = ch8state.exec_instruction();
    assert_eq!(ch8state.pc, pc_pre, "Incorrect program counter");
    assert_eq!(ch8state.reg[3], 0);
    assert_eq!(ch8state.waiting_for_key, Some(3));
    ch8state = ch8state.exec_instruction();
    assert_eq!(ch8state.pc, pc_pre, "Incorrect program counter");
    assert_eq!(ch8state.reg[3], 0);
    assert_eq!(ch8state.waiting_for_key, Some(3));
}

#[test]
fn key_arrives_while_waiting_for_key() {
    let mut ch8state = Chip8State::new();
    ch8state = ch8state.load(&[0xF7,0x0A]);
    let pc_pre = ch8state.pc;
    ch8state = ch8state.exec_instruction();
    assert_eq!(ch8state.pc, pc_pre, "Incorrect program counter");
    assert_eq!(ch8state.reg[7], 0);
    assert_eq!(ch8state.waiting_for_key, Some(7));
    ch8state = ch8state.key_down(0xC);
    assert_eq!(ch8state.pc, pc_pre + 2, "Incorrect program counter");
    assert_eq!(ch8state.reg[7], 0xC);
    assert_eq!(ch8state.waiting_for_key, None);
}

#[test]
fn set_delay_timer() {
    let mut ch8state = Chip8State::new();
    ch8state = ch8state.load(&[0xF9,0x15]);
    ch8state.reg[9] = 60;
    let pc_pre = ch8state.pc;
    ch8state = ch8state.exec_instruction();
    assert_eq!(ch8state.pc, pc_pre+2, "Incorrect program counter");
    assert_eq!(ch8state.t_delay, 60);
}
