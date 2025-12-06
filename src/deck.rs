use crate::card::Card;
use crate::enums::*;

use std::fmt;
use rand::seq::SliceRandom;
use rand::thread_rng;

pub struct Deck {
    pub cards: Vec<Card>,
}

impl Deck {
    pub fn new_full_deck() -> Self {
        Deck {
            cards: (0..=49)
            .map(|i| Card::new(i as u8))
            .collect::<Vec<Card>>() 
        }
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