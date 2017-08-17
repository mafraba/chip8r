const TOTAL_COLS: usize = 64;
const TOTAL_ROWS: usize = 32;
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

    // Draw a byte (as part of a sprite)
    fn draw_byte(&mut self, row: u8, col: u8, byte: u8) -> bool {
        let mut collision = false;
        let bits: [bool; 8] = [
            (byte & 0x80) != 0,
            (byte & 0x40) != 0,
            (byte & 0x20) != 0,
            (byte & 0x10) != 0,
            (byte & 0x08) != 0,
            (byte & 0x04) != 0,
            (byte & 0x02) != 0,
            (byte & 0x01) != 0,
        ];

        for (i, b) in bits.iter().enumerate() {
            if *b {
                let col_idx = col + i as u8;
                collision |= self.flip_pixel(row, col_idx);
            }
        }
        collision
    }
}

fn pixel_index(row: u8, col: u8) -> usize {
    let r = row as usize;
    let c = col as usize;
    // Don't care about managing out of bounds errors here, just check as preconditions
    if r >= TOTAL_ROWS { panic!("Row index out of bounds: {}", r) }
    if c >= TOTAL_COLS { panic!("Column index out of bounds: {}", c) }
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

    #[test]
    fn draw_byte_no_collision() {
        let mut d = Chip8Display::new();
        let collision = d.draw_byte(0, 0, 0xAA);
        assert_eq!(collision, false);
        for c in 0..64 {
            for r in 0..32 {
                if c < 8 && c % 2 == 0 && r == 0 {
                    assert_eq!(d.get_pixel(r,c), true);
                } else {
                    assert_eq!(d.get_pixel(r,c), false);
                }
            }
        }
    }

    #[test]
    fn draw_byte_with_collision() {
        let mut d = Chip8Display::new();
        d.flip_pixel(0,0);
        let collision = d.draw_byte(0, 0, 0xAA);
        assert_eq!(collision, true);
        for c in 0..64 {
            for r in 0..32 {
                if c > 0 && c < 8 && c % 2 == 0 && r == 0 {
                    assert_eq!(d.get_pixel(r,c), true);
                } else {
                    assert_eq!(d.get_pixel(r,c), false);
                }
            }
        }
    }
}
