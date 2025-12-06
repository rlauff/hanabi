use crate::enums::{Color, Move, MoveResult};
use crate::card::Card;
use crate::strategy::Strategy;
use rand::seq::IndexedRandom;

/// A simple strategy that picks a random valid move.
/// It tracks the game state locally to determine which moves are currently legal.
pub struct RandomStrategy { 
    hints_remaining: u8,
    own_hand: Vec<Card>,
    other_players_hand: Vec<Card>,
}

impl RandomStrategy {
    /// Constructor to create a new instance of the strategy.
    pub fn new() -> Self {
        RandomStrategy {
            hints_remaining: 8, // Standard Hanabi starts with 8 hint tokens
            own_hand: Vec::new(),
            other_players_hand: Vec::new(),
        }
    }

    /// Determines all currently legal moves based on the local state.
    fn possible_moves(&self) -> Vec<Move> {
        let mut moves = Vec::new();

        // 1. Play and Discard moves
        // You can always attempt to play or discard any card currently in your hand.
        for card_index in 0..self.own_hand.len() {
            moves.push(Move::Play(card_index));
            moves.push(Move::Discard(card_index));
        }

        // 2. Hint moves
        // You can only give a hint if there are hint tokens remaining.
        if self.hints_remaining > 0 {
            // Check for valid Color hints
            for color in [Color::Red, Color::Green, Color::Blue, Color::Yellow, Color::White].iter() {
                // You can only hint a color if the other player actually holds a card of that color.
                if self.other_players_hand.iter().any(|&card| card.get_color() == *color) {
                    moves.push(Move::HintColor(*color));
                }
            }

            // Check for valid Value hints
            for value in 1..=5 {
                // You can only hint a value if the other player actually holds a card of that value.
                if self.other_players_hand.iter().any(|&card| card.get_value() == value) {
                    moves.push(Move::HintValue(value));
                }
            }
        }

        moves
    }
}

impl Strategy for RandomStrategy {

    /// Decides the next move by choosing randomly from the list of possible moves.
    fn decide_move(&mut self) -> Move {
        let possible_moves = self.possible_moves();
        let mut rng = rand::rng();
        
        // We must dereference (*) because choose returns a reference (&Move), 
        // but we need to return the Move itself.
        *possible_moves.choose(&mut rng).expect("No possible moves available")
    }

    /// Initializes the strategy at the start of the game with the initial hands.
    fn initialize(&mut self, own_hand: &Vec<Card>, other_player_hand: &Vec<Card>) {
        self.own_hand = own_hand.clone();
        self.other_players_hand = other_player_hand.clone();
        self.hints_remaining = 8; // Reset hints to 8
    }

    /// Updates the local state after the player (self) makes a move.
    fn update_after_own_move(&mut self, mv: &Move, mv_result: &MoveResult) {
        match mv {
            Move::Play(card_index) => {
                // Remove the played card
                self.own_hand.remove(*card_index);
                
                // If a new card was drawn (result contains Some(card)), add it to hand
                if let MoveResult::Play(_, Some(card)) = mv_result {
                    self.own_hand.push(*card);
                }
            }
            Move::Discard(card_index) => {
                // Remove the discarded card
                self.own_hand.remove(*card_index);
                
                // If a new card was drawn, add it to hand
                if let MoveResult::Discard(Some(card)) = mv_result {
                    self.own_hand.push(*card);
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
                if let MoveResult::Play(_, Some(card)) = mv_result {
                    self.other_players_hand.push(*card);
                }
            }
            Move::Discard(card_index) => {
                // Remove the card the other player discarded
                self.other_players_hand.remove(*card_index);
                
                // If they drew a new card, add it to their hand tracker
                if let MoveResult::Discard(Some(card)) = mv_result {
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
}