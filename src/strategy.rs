use crate::card::Card;
use crate::enums::*;

pub trait Strategy {
    fn initialize(&mut self, other_player_hand: &Vec<Card>);

    fn decide_move(&mut self) -> Move;

    fn update_after_own_move(&mut self, mv: &Move, mv_result: &MoveResult, got_new_card: bool);

    fn update_after_other_player_move(&mut self, mv: &Move, mv_result: &MoveResult);
}