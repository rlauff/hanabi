use crate::card::Card;
use crate::enums::*;

pub trait Strategy {
    fn decide_move(&mut self) -> Move;

    fn initialize(&mut self, own_hand: &Vec<Card>, other_player_hand: &Vec<Card>);

    fn update_after_own_move(&mut self, mv: &Move, mv_result: &MoveResult);

    fn update_after_other_player_move(&mut self, mv: &Move, mv_result: &MoveResult);
}