
use std::fmt;
use crate::enums::Color;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Card (pub u8);

impl Card {
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
        self.0 % 10
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
