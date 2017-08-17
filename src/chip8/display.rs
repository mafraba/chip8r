const TOTAL_COLS: usize = 32;
const TOTAL_ROWS: usize = 64;
const TOTAL_PIXELS: usize = TOTAL_ROWS*TOTAL_COLS;

// A 64x32 binary pixels display for CHIP8
struct Chip8Display {
    pixels: [bool; TOTAL_PIXELS]
}

impl Chip8Display {

    fn new() -> Chip8Display {
        Chip8Display{ pixels: [false; TOTAL_PIXELS] }
    }

    fn get_pixel(&self, row: u8, col: u8) -> bool {
        self.pixels[pixel_index(row, col)]
    }

    fn flip_pixel(&mut self, row: u8, col: u8) -> bool {
        let idx = pixel_index(row, col);
        let prev = self.get_pixel(row, col);
        self.pixels[idx] = !prev;
        prev
    }
}

fn pixel_index(row: u8, col: u8) -> usize {
    let r = row as usize;
    let c = col as usize;
    // Don't care about managing out of bounds errors here, just check as preconditions
    if r >= TOTAL_ROWS { panic!("Row index out of bounds") }
    if c >= TOTAL_COLS { panic!("Column index out of bounds") }
    32*r + c
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flip_unset_pixel() {
        let mut d = Chip8Display::new();
        assert_eq!(d.get_pixel(0,0), false);
        let collision = d.flip_pixel(0,0);
        assert_eq!(collision, false);
        assert_eq!(d.get_pixel(0,0), true);
    }

    #[test]
    fn flip_set_pixel() {
        let mut d = Chip8Display::new();
        assert_eq!(d.get_pixel(0,0), false);
        let mut collision = d.flip_pixel(0,0);
        assert_eq!(collision, false);
        assert_eq!(d.get_pixel(0,0), true);
        collision = d.flip_pixel(0,0);
        assert_eq!(collision, true);
        assert_eq!(d.get_pixel(0,0), false);
    }
}
