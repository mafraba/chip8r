use super::*;

use std::io::{self, Write, stdout, stdin};
use termion::{color, cursor, clear, style};
use termion::event::*;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

pub fn display(chip8state: Chip8State) {

    let mut stdout = stdout().into_raw_mode().unwrap();

    write!(stdout, "{clear}{goto}{blue}############################ \
                    {green}Pure CHIP-8 joy! \
                    {blue}############################ {color_reset}",
                    // Full screen clear.
                    clear = clear::All,
                    // Goto the cell.
                    goto  = cursor::Goto(2, 2),
                    green = color::Fg(color::Green),
                    blue  = color::Fg(color::Blue),
                    color_reset = color::Fg(color::Reset));

    for row in 0..32 {
        write!(stdout, "{}{gray}{row:3}{reset}   ",
                goto = cursor::Goto(2, (row + 4) as u16),
                gray = color::Fg(color::LightBlack),
                reset = color::Fg(color::Reset),
                row = row);
        for col in 0..64 {
            if chip8state.get_pixel(col, row) {
                write!(stdout, "█");
            } else {
                write!(stdout, " ");
            }
        }
    }

    stdout.flush().unwrap();
}
