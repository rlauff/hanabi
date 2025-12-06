use crate::r#move::Move;
use crate::card::Card;
use crate::player::Player;


pub trait Strategy {
    fn decide_move(&mut self) -> Move;

    fn update_after_own_move(&mut self, mv: &Move, card_revealed: Option<Card>);

    fn update_after_other_player_move(&mut self, mv: &Move, card_revealed: Option<Card>);
}