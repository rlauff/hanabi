mod enums;
mod card;
mod deck;
mod r#move;
mod player;
mod game;

use crate::player::Player;
use crate::r#move::Move;
use crate::game::Game;

fn strategy_random_move(_player: &Player) -> Move {
    // Play a random move
    Move::Play(0)
}

fn main() {
    let game = Game::new([strategy_random_move, strategy_random_move]);
    game.display_game_state();
}