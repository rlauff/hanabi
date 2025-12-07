use std::fmt;
use crate::card::Card;
use crate::deck::Deck;
use crate::knowledge::DeckSubset;
use crate::strategy::Strategy;
use crate::enums::*;

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

    // pub fn see(&mut self, card: Card) {
    //     // Update knowledge about the other player's hand when they draw a card
    //     for knowledge in &mut self.hand_knowledge {
    //         knowledge.remove_card(card);
    //     }
    // }

    // pub fn other_player_sees(&mut self, card: Card) {
    //     // Update inferred knowledge about what the other player knows about their hand
    //     for knowledge in &mut self.infered_hand_knowledge {
    //         knowledge.remove_card(card);
    //     }
    // }

    // pub fn get_hint(&mut self, hint: &Knowledge, card_indices: &[usize]) {
    //     for &index in card_indices {
    //         self.hand_knowledge[index] = self.hand_knowledge[index].intersect(hint);
    //     }
    // }

    pub fn display_hand(&self) {
        println!("{}", self);
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
