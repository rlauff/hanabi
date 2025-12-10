use std::fmt;
use crate::card::Card;
use crate::deck::Deck;
use crate::strategy::Strategy;

pub struct Player {
    pub hand: Vec<Card>,
    pub strategy: Box<dyn Strategy>,
}

impl Player {
    pub fn new(strategy: Box<dyn Strategy>) -> Self {
        Player {
            hand: Vec::new(),
            strategy,
        }
    }

    pub fn draw(&mut self, deck: &mut Deck) -> Card {
        let new_card = deck.cards.pop().expect("Deck is empty");
        self.hand.push(new_card);
        new_card
    }
}

impl fmt::Display for Player {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for card in &self.hand {
            write!(f, "{} ", card)?;
        }
        Ok(())
    }
}
