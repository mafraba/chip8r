use super::*;

#[test]
fn load_data() {
    let mut ch8core = Chip8Core::new();
    let initial_state = &*ch8core.state as *const Chip8State;
    assert_eq!(ch8core.state.ram[0x200], 0);
    assert_eq!(ch8core.state.ram[0x201], 0);
    assert_eq!(ch8core.state.ram[0x202], 0);
    assert_eq!(ch8core.state.ram[0x203], 0);

    let data: &[u8] = &[1,2,3,4];
    ch8core.load(data);
    assert_eq!(ch8core.state.ram[0x199], 0);
    assert_eq!(ch8core.state.ram[0x200], 1);
    assert_eq!(ch8core.state.ram[0x201], 2);
    assert_eq!(ch8core.state.ram[0x202], 3);
    assert_eq!(ch8core.state.ram[0x203], 4);
    assert_eq!(ch8core.state.ram[0x204], 0);
    let new_state = &*ch8core.state as *const Chip8State;
    assert!(initial_state != new_state);
}

#[test]
fn read_instruction() {
    let mut ch8core = Chip8Core::new();
    let data: &[u8] = &[1,2];
    ch8core.load(data);
    assert_eq!(ch8core.read_instruction(), Chip8Instruction(0x0102));
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
    let mut ch8core = Chip8Core::new();
    // load a single CLS instruction
    let data: &[u8] = &[0x00,0xE0];
    ch8core.load(data);
    assert_eq!(ch8core.read_instruction(), Chip8Instruction(0x00E0));
    // put some garbage on display buffer
    ch8core.state.ram[0xF00] = 1;
    ch8core.state.ram[0xFFF] = 1;
    let initial_state = &*ch8core.state as *const Chip8State;
    ch8core.exec_instruction();
    // check it was cleared
    for byte in &ch8core.state.ram[0xF00..0xFFF] {
        assert_eq!(*byte, 0);
    }
    let new_state = &*ch8core.state as *const Chip8State;
    assert!(initial_state != new_state);
}

#[test]
fn read_return_address() {
    let mut ch8core = Chip8Core::new();
    // set some ret address
    ch8core.state.ram[ch8core.state.sp as usize] = 0x0A;
    ch8core.state.ram[(ch8core.state.sp+1) as usize] = 0xBC;
    // check return
    assert_eq!(ch8core.read_return_address(), 0x0ABC);
}

#[test]
fn return_from_subroutine_instruction() {
    let mut ch8core = Chip8Core::new();
    let initial_sp = ch8core.state.sp;
    // set some ret address
    ch8core.state.ram[ch8core.state.sp as usize] = 0x0A;
    ch8core.state.ram[(ch8core.state.sp+1) as usize] = 0xBC;
    // load return instruction and execute
    ch8core.load(&[0x00,0xEE]);
    ch8core.exec_instruction();
    // check state
    assert_eq!(ch8core.state.sp, initial_sp-2);
    assert_eq!(ch8core.state.pc, 0x0ABC);
}

#[test]
fn jump_instruction() {
    let mut ch8core = Chip8Core::new();
    // load return instruction and execute
    ch8core.load(&[0x1A,0xBC]);
    ch8core.exec_instruction();
    // check state
    assert_eq!(ch8core.state.pc, 0x0ABC);
}
