use crate::card::Card;
use crate::enums::*;

// encoding: tens place = color, units place map: 1 1 1 2 2 3 3 4 4 5

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct DeckSubset (pub u64);

impl DeckSubset {
    pub fn new_full() -> Self {
        DeckSubset((1u64 << 50) - 1) 
    }

    pub fn from_color(color: Color) -> Self {
        match color {
            Color::Red =>       DeckSubset(0b0000000000000000000000000000000000000000000000000000001111111111),    // Cards 0-9
            Color::Green =>     DeckSubset(0b0000000000000000000000000000000000000000000011111111110000000000),  // Cards 10-19
            Color::Blue =>      DeckSubset(0b0000000000000000000000000000000000111111111100000000000000000000),   // Cards 20-29
            Color::Yellow =>    DeckSubset(0b0000000000000000000000001111111111000000000000000000000000000000), // Cards 30-39
            Color::White =>     DeckSubset(0b0000000000000011111111110000000000000000000000000000000000000000),  // Cards 40-49
        }
    }

    pub fn from_color_inverted(color: Color) -> Self {
        // Wir nutzen new_full() als Maske, um sicherzustellen, dass wir im 50-Bit Bereich bleiben
        // und invertieren dann nur die Bits der Farbe.
        let full = Self::new_full().0;
        let col = Self::from_color(color).0;
        DeckSubset((!col) & full)
    }

    pub fn from_value(value: u8) -> Self {
        match value {
            1 =>    DeckSubset(0b0000000000000000000001110000000111000000011100000001110000000111),
            2 =>    DeckSubset(0b0000000000000000000110000000011000000001100000000110000000011000),
            3 =>    DeckSubset(0b0000000000000000011000000001100000000110000000011000000001100000),
            4 =>    DeckSubset(0b0000000000000001100000000110000000011000000001100000000110000000),
            5 =>    DeckSubset(0b0000000000000010000000001000000000100000000010000000001000000000),
            _ => panic!("Invalid value for hint"),
        }
    }

    pub fn from_value_inverted(value: u8) -> Self {
        let full = Self::new_full().0;
        let val = Self::from_value(value).0;
        DeckSubset((!val) & full)
    }

    pub fn from_card(card: Card) -> Self {
        DeckSubset::from_color(card.get_color())
            .intersect(&DeckSubset::from_value(card.get_value()))
    }

    pub fn has_card(&self, card: Card) -> bool {
        (self.0 & (1 << card.0)) != 0
    }

    pub fn remove_card(&mut self, card: Card) {
        self.0 &= !(1 << card.0);
    }

    pub fn add_card(&mut self, card: Card) {
        self.0 |= 1 << card.0;
    }

    pub fn intersect(&self, other: &DeckSubset) -> DeckSubset {
        DeckSubset(self.0 & other.0)
    }
}