use super::*;
use test::Bencher;


#[bench]
fn decrease_timers_bench(b: &mut Bencher) {
    let mut ch8state = Chip8State::new();
    ch8state.t_delay = 1;
    ch8state.t_sound = 1;
    b.iter(|| ch8state.decrease_timers());
}

#[bench]
fn instruction_execution_bench(b: &mut Bencher) {
    let mut ch8state = Chip8State::new();
    ch8state = ch8state.load(&[0x80,0x0E]);
    ch8state.reg[0] = 0x83;
    b.iter(|| ch8state.exec_instruction());
}
