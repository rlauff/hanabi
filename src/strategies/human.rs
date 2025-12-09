use crate::enums::{Move, MoveResult, Color};
use crate::card::Card;
use crate::strategy::Strategy;
use crate::decksubset::DeckSubset;
use std::io::{self, Write};

pub struct Human { 
   
}

impl Strategy for Human {
    fn initialize(&mut self, other_player_hand: &Vec<Card>) {
        
    }

    fn decide_move(&mut self) -> Move {
        // just ask the user for input
        print!("Enter your move (e.g., 'play 0', 'discard 1', 'hint color 2 red'): ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let parts: Vec<&str> = input.trim().split_whitespace().collect();
        match parts.as_slice() {
            ["play", index_str] => {
                let index: usize = index_str.parse().unwrap();
                Move::Play(index)
            },
            ["discard", index_str] => {
                let index: usize = index_str.parse().unwrap();
                Move::Discard(index)
            },
            ["hint", "color", index_str, color_str] => {
                let index: usize = index_str.parse().unwrap();
                let color = match color_str.to_lowercase().as_str() {
                    "red" => Color::Red,
                    "green" => Color::Green,
                    "blue" => Color::Blue,
                    "yellow" => Color::Yellow,
                    "white" => Color::White,
                    _ => panic!("Invalid color"),
                };
                Move::HintColor(color)
            },
            _ => panic!("Invalid move format"),
        }
    }

    fn update_after_own_move(&mut self, mv: &Move, mv_result: &MoveResult, got_new_card: bool) {
        
    }

    fn update_after_other_player_move(&mut self, mv: &Move, mv_result: &MoveResult) {
        
    }
}