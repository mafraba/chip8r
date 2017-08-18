#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_key_state_is_false() {
        let keyboard = Chip8Keyboard::new();
        for i in 0..0xF {
            assert_eq!(keyboard.is_key_pressed(i), false);
        }
    }

    #[test]
    fn key_pressed_event() {
        let mut keyboard = Chip8Keyboard::new();
        keyboard.key_pressed(0);
        assert_eq!(keyboard.is_key_pressed(0), true);
        for i in 1..0xF {
            assert_eq!(keyboard.is_key_pressed(i), false);
        }
    }

    #[test]
    fn key_released_event() {
        let mut keyboard = Chip8Keyboard::new();
        keyboard.key_pressed(0);
        keyboard.key_released(0);
        for i in 0..0xF {
            assert_eq!(keyboard.is_key_pressed(i), false);
        }
    }
}

// Model for a chip8 keyboard state
struct Chip8Keyboard {
    // keys state: true if currently pressed
    keys: [bool; 0xF],
}

impl Chip8Keyboard {
    fn new() -> Chip8Keyboard {
        Chip8Keyboard { keys: [false; 0xF] }
    }

    fn is_key_pressed(&self, key: u8) -> bool {
        self.keys[key as usize]
    }

    fn key_pressed(&mut self, key: u8) {
        self.keys[key as usize] = true;
    }

    fn key_released(&mut self, key: u8) {
        self.keys[key as usize] = false;
    }
}
