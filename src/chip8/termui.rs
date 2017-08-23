use super::*;

use std::io::{Write, stdout, stdin};
use termion::{color, cursor, clear};
use termion::event::*;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

pub fn display_loop(state_rx: ::chan::Receiver<Chip8State>) {
    loop {
        chan_select! {
            state_rx.recv() -> st => {
                display(st.unwrap());
            }
        }
    }
}

fn display(chip8state: Chip8State) {

    let mut stdout = stdout().into_raw_mode().unwrap();

    write!(stdout, "{clear}{goto}{blue}############################## \
                    {green}Pure CHIP-8 joy! \
                    {blue}##############################{color_reset}",
                    // Full screen clear.
                    clear = clear::All,
                    // Goto the cell.
                    goto  = cursor::Goto(2, 2),
                    green = color::Fg(color::Green),
                    blue  = color::Fg(color::Blue),
                    color_reset = color::Fg(color::Reset)).unwrap();

    // Print display
    for row in 0..32 {
        write!(stdout, "{goto}{gray}{row:3}{reset}   ",
                goto = cursor::Goto(2, (row + 4) as u16),
                gray = color::Fg(color::LightBlack),
                reset = color::Fg(color::Reset),
                row = row).unwrap();
        for col in 0..64 {
            if chip8state.get_pixel(col, row) {
                write!(stdout, "█").unwrap();
            } else {
                write!(stdout, " ").unwrap();
            }
        }
    }

    // Print keys states
    write!(stdout, "{goto}{blue}###############{color_reset} ",
                    goto = cursor::Goto(0, 37),
                    blue = color::Fg(color::Blue),
                    color_reset = color::Fg(color::Reset)).unwrap();
    for key in 0..0x10 {
        if chip8state.is_key_down(key) {
            write!(stdout, "{color} {key:X} {reset}",
                    color = color::Bg(color::Red),
                    reset = color::Bg(color::Reset),
                    key = key).unwrap();
        } else {
            write!(stdout, "{color} {key:X} {reset}",
                    color = color::Bg(color::LightBlack),
                    reset = color::Bg(color::Reset),
                    key = key).unwrap();
        }
    }
    write!(stdout, "{blue} ###############{color_reset}",
                    blue = color::Fg(color::Blue),
                    color_reset = color::Fg(color::Reset)).unwrap();

    stdout.flush().unwrap();
}

pub fn listen_for_keys(keys_tx: ::chan::Sender<char>) {
    let stdin = stdin();
    for c in stdin.keys() {
        match c.unwrap() {
            Key::Char(c)   => keys_tx.send(c),
            // Key::Left      => println!("<left>"),
            // Key::Right     => println!("<right>"),
            // Key::Up        => println!("<up>"),
            // Key::Down      => println!("<down>"),
            _              => {},
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    #[ignore]
    #[bench]
    fn display_bench(b: &mut Bencher) {
        let ch8state = Chip8State::new();
        b.iter(|| display(ch8state));
    }
}
