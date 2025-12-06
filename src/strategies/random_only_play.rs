use crate::enums::{Color, Move, MoveResult};
use crate::card::Card;
use crate::strategy::Strategy;
use rand::seq::IndexedRandom;

/// A simple strategy that picks a random valid move.
/// It tracks the game state locally to determine which moves are currently legal.
pub struct RandomOnlyPlay { 
    hints_remaining: u8,
    own_hand_size: u8,
    other_players_hand: Vec<Card>,
}

impl RandomOnlyPlay {
    /// Constructor to create a new instance of the strategy.
    pub fn new() -> Self {
        RandomOnlyPlay {
            hints_remaining: 8, // Standard Hanabi starts with 8 hint tokens
            own_hand_size: 5,
            other_players_hand: Vec::new(),
        }
    }

    /// Determines all currently legal moves based on the local state.
    fn possible_moves(&self) -> Vec<Move> {
        let mut moves = Vec::new();

        // 1. Play and Discard moves
        // You can always attempt to play or discard any card currently in your hand.
        for card_index in 0..self.own_hand_size {
            moves.push(Move::Play(card_index as usize));
        }
        moves
    }
}

impl Strategy for RandomOnlyPlay {

    /// Decides the next move by choosing randomly from the list of possible moves.
    fn decide_move(&mut self) -> Move {
        let possible_moves = self.possible_moves();
        let mut rng = rand::rng();
        
        // We must dereference (*) because choose returns a reference (&Move), 
        // but we need to return the Move itself.
        *possible_moves.choose(&mut rng).expect("No possible moves available")
    }

    /// Initializes the strategy at the start of the game with the initial hands.
    fn initialize(&mut self, other_player_hand: &Vec<Card>) {
        self.own_hand_size = 5;
        self.other_players_hand = other_player_hand.clone();
        self.hints_remaining = 8; // Reset hints to 8
    }

    /// Updates the local state after the player (self) makes a move.
    fn update_after_own_move(&mut self, mv: &Move, mv_result: &MoveResult ,got_new_card: bool) {
        match mv {
            Move::Play(card_index) => {
                if !got_new_card {
                    // If no new card was drawn, just decrease hand size
                    self.own_hand_size -= 1;
                }
            }
            Move::Discard(card_index) => {
                if !got_new_card {
                    // If no new card was drawn, just decrease hand size
                    self.own_hand_size -= 1;
                }
                
                // Discarding regains a hint token, up to a max of 8
                if self.hints_remaining < 8 {
                    self.hints_remaining += 1;
                }
            }
            // Giving a hint consumes a hint token
            Move::HintColor(_) | Move::HintValue(_) => {
                self.hints_remaining -= 1;
            }
        }
    }

    /// Updates the local state after the other player makes a move.
    fn update_after_other_player_move(&mut self, mv: &Move, mv_result: &MoveResult) {
        match mv {
            Move::Play(card_index) => {
                // Remove the card the other player played
                self.other_players_hand.remove(*card_index);
                
                // If they drew a new card, add it to their hand tracker
                if let MoveResult::Play(_, card) = mv_result {
                    self.other_players_hand.push(*card);
                }
            }
            Move::Discard(card_index) => {
                // Remove the card the other player discarded
                self.other_players_hand.remove(*card_index);
                
                // If they drew a new card, add it to their hand tracker
                if let MoveResult::Discard(card) = mv_result {
                    self.other_players_hand.push(*card);
                }
                
                // Discarding regains a hint token
                if self.hints_remaining < 8 {
                    self.hints_remaining += 1;
                }
            }
            // Giving a hint consumes a hint token
            Move::HintColor(_) | Move::HintValue(_) => {
                self.hints_remaining -= 1;
            }
        }
    }

    fn see(&mut self, _card: &Card) {
        // This strategy does not utilize information about seen cards.
    }
}