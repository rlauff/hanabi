use std::fmt;
use crate::card::Card;
use crate::deck::Deck;
use crate::r#move::Move;

#[derive(Clone, Debug)]
pub struct Player {
    pub index: usize,
    pub other_player_index: usize, // reference to the other player by index
    pub hand: Vec<Card>,
    pub hand_knowledge: [Deck; 5], // the possible cards for each card in hand
    pub infered_hand_knowledge: [Deck; 5], // inferred knowledge of what the other player knows about their hand
    pub strategy: fn(&Player) -> Move, // a function that takes &self and returns a Move
}

impl Player {
    pub fn draw(&mut self, deck: &mut Deck) -> Card {
        let new_card = deck.cards.pop().expect("Deck is empty");
        self.hand.push(new_card);
        new_card
    }

    pub fn see(&mut self, card: Card) {
        // Update knowledge about the other player's hand when they draw a card
        for knowledge in &mut self.hand_knowledge {
            if let Some(pos) = knowledge.cards.iter().position(|c| *c == card) {
                knowledge.cards.remove(pos);
            }
        }
    }

    pub fn display_hand(&self) {
        println!("Player {}'s hand: {}", self.index, self);
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
