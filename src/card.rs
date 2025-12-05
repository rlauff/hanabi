
use std::fmt;
use crate::enums::Color;

// encoding: tens place = color, units place map: 1 1 1 2 2 3 3 4 4 5

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Card (pub u8);

impl Card {
    pub fn new(encoded: u8) -> Self {
        Card(encoded)
    }

    pub fn get_color(&self) -> Color {
        match self.0 / 10 {
            0 => Color::Red,
            1 => Color::Green,
            2 => Color::Blue,
            3 => Color::Yellow,
            4 => Color::White,
            _ => panic!("Invalid card color"), // panic for invalid color, should not happen
        }
    }

    pub fn get_value(&self) -> u8 {
        match self.0 % 10 {
            0..=2 => 1,
            3..=4 => 2,
            5..=6 => 3,
            7..=8 => 4,
            9 => 5,
            _ => panic!("Invalid card value"), // panic for invalid value, should not happen
        }
    }
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (start, end) = match self.get_color() {
            Color::Red => ("\x1b[31m", "\x1b[0m"),
            Color::Green => ("\x1b[32m", "\x1b[0m"),
            Color::Blue => ("\x1b[34m", "\x1b[0m"),
            Color::Yellow => ("\x1b[33m", "\x1b[0m"),
            Color::White => ("\x1b[37m", "\x1b[0m"),
        };

        write!(f, "{}[{}]{}", start, self.get_value(), end)
    }
}
