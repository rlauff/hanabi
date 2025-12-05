use crate::card::Card;
use crate::enums::Color;
use std::fmt;
use rand::seq::SliceRandom;
use rand::thread_rng;

#[derive(Copy, Clone, Debug)]
pub struct Deck {
    pub cards: Vec<Card>,
}

impl Deck {
    pub fn new_full_deck() -> Self {
        let mut cards = Vec::new();
        let amounts = [3, 2, 2, 2, 1]; // Amounts of cards for values 1 to 5

        let colors = [Color::Red, Color::Green, Color::Blue, Color::Yellow, Color::White];
        for (ci, _color) in colors.iter().enumerate() {
            for value in 1..=5u8 {
                for _ in 0..amounts[(value - 1) as usize] {
                    let encoded = (ci as u8) * 10 + value;
                    cards.push(Card(encoded));
                }
            }
        }

        Deck { cards }
    }

    pub fn shuffle(&mut self) {
        let mut rng = thread_rng();
        self.cards.shuffle(&mut rng);
    }
}

impl fmt::Display for Deck {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for card in &self.cards {
            write!(f, "{} ", card)?;
        }
        Ok(())
    }
}