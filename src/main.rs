mod enums;
mod card;
mod deck;
mod r#move;
mod player;
mod game;
mod knowledge;

use crate::player::Player;
use crate::r#move::Move;
use crate::game::Game;

fn strategy_random_move(_player: &Player, game_state: Game ) -> Move {
    let possible_moves = game_state.possible_moves();
    let mut rng = rand::thread_rng();
    *possible_moves.choose(&mut rng).expect("No possible moves")
}

fn main() {
    let game = Game::new([strategy_random_move, strategy_random_move]);
    game.display_game_state();
}