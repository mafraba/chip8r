use super::*;

#[test]
fn load_data() {
    let mut ch8core = Chip8Core::new();
    assert_eq!(ch8core.ram[0x200], 0);
    assert_eq!(ch8core.ram[0x201], 0);
    assert_eq!(ch8core.ram[0x202], 0);
    assert_eq!(ch8core.ram[0x203], 0);

    let data: &[u8] = &[1,2,3,4];
    ch8core.load(data);
    assert_eq!(ch8core.ram[0x199], 0);
    assert_eq!(ch8core.ram[0x200], 1);
    assert_eq!(ch8core.ram[0x201], 2);
    assert_eq!(ch8core.ram[0x202], 3);
    assert_eq!(ch8core.ram[0x203], 4);
    assert_eq!(ch8core.ram[0x204], 0);
}

#[test]
fn test_read_instruction() {
    let mut ch8core = Chip8Core::new();
    let data: &[u8] = &[1,2];
    ch8core.load(data);
    assert_eq!(ch8core.read_instruction(), 0x0102);
}
