
use crate::card::Card;
use crate::enums::*;

// encoding: tens place = color, units place map: 1 1 1 2 2 3 3 4 4 5

#[derive(Copy, Clone)]
pub struct Knowledge (pub u64);

impl Knowledge {
    pub fn new_full() -> Self {
        Knowledge(!0)
    }

    pub fn from_color(color: Color) -> Self {
        match color {
            Color::Red =>       Knowledge(0b0000000000000000000000000000000000000000000000000000001111111111),    // Cards 0-9
            Color::Green =>     Knowledge(0b0000000000000000000000000000000000000000000011111111110000000000),  // Cards 10-19
            Color::Blue =>      Knowledge(0b0000000000000000000000000000000000111111111100000000000000000000),   // Cards 20-29
            Color::Yellow =>    Knowledge(0b0000000000000000000000001111111111000000000000000000000000000000), // Cards 30-39
            Color::White =>     Knowledge(0b0000000000000011111111110000000000000000000000000000000000000000),  // Cards 40-49
        }
    }

     pub fn from_value(value: u8) -> Self {
        match value {
            1 =>    Knowledge(0b0000000000000000000001110000000111000000011100000001110000000111),
            2 =>    Knowledge(0b0000000000000000000110000000011000000001100000000110000000011000),
            3 =>    Knowledge(0b0000000000000000011000000001100000000110000000011000000001100000),
            4 =>    Knowledge(0b0000000000000001100000000110000000011000000001100000000110000000),
            5 =>    Knowledge(0b0000000000000010000000001000000000100000000010000000001000000000),
            _ => panic!("Invalid value for hint"), // panic for invalid value, should not happen
        }
    }

    pub fn has_card(&self, card: Card) -> bool {
        (self.0 & (1 << card.0)) & 1 != 0
    }

    pub fn remove_card(&mut self, card: Card) {
        self.0 &= !(1 << card.0);
    }

    pub fn add_card(&mut self, card: Card) {
        self.0 |= 1 << card.0;
    }

    pub fn intersect(&self, other: &Knowledge) -> Knowledge {
        Knowledge(self.0 & other.0)
    }
}
