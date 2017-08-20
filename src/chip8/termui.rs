use super::*;

use std::io::{self, Write, stdout, stdin};
use termion::{color, cursor, clear, style};
use termion::event::*;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

pub fn display(chip8state: Chip8State) {

    let mut stdout = stdout().into_raw_mode().unwrap();

    write!(stdout, "{clear}{goto}{green}Pure CHIP-8 joy!  {reset}",
             // Full screen clear.
             clear = clear::All,
             // Goto the cell.
             goto  = cursor::Goto(2, 2),
             green   = color::Fg(color::Green),
             reset = color::Fg(color::Reset));
    write!(stdout, "{bold}{blue}:D{reset}",
             bold  = style::Bold,
             blue  = color::Fg(color::Blue),
             reset = style::Reset);

    for row in 0..32 {
        write!(stdout, "{}{:3} ", cursor::Goto(2, (row + 5) as u16), row);
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
