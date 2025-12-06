mod enums;
mod card;
mod deck;
mod player;
mod game;
mod knowledge;
mod strategy;
mod strategies;

use crate::game::Game;
use crate::player::Player;
use crate::strategy::Strategy;
use crate::strategies::random::RandomStrategy; 
use crate::strategies::random_only_play::RandomOnlyPlay;

const GAMES_TO_SIMULATE: u32 = 1000000;

fn main() {
    // Registry of strategies. Uses closures to create fresh instances for every game.
    let strategies: Vec<(&str, fn() -> Box<dyn Strategy>)> = vec![
        ("Random", || Box::new(RandomStrategy::new())),
        ("Random Only Play", || Box::new(strategies::random_only_play::RandomOnlyPlay::new())),
    ];

    println!("Simulating {} games per strategy...\n", GAMES_TO_SIMULATE);

    for (name, create_strategy_fn) in strategies {
        let mut total_score: u32 = 0;
        let mut perfect_games = 0;
        let mut zero_score_games = 0; // Games lost via 3 mistakes

        for _ in 0..GAMES_TO_SIMULATE {
            let p1 = Player::new(create_strategy_fn());
            let p2 = Player::new(create_strategy_fn());
            let mut game = Game::new(p1, p2);

            // Run game loop until game_over returns a score
            let score = loop {
                if let Some(final_score) = game.game_over() {
                    break final_score;
                }
                game.advance();
            };

            total_score += score as u32;
            
            if score == 25 { perfect_games += 1; }
            if score == 0 { zero_score_games += 1; }
        }

        let average_score = total_score as f64 / GAMES_TO_SIMULATE as f64;

        println!("Strategy: {}", name);
        println!("  -> Average Score:     {:.4}", average_score);
        println!("  -> Perfect Games (25): {}", perfect_games);
        println!("  -> Lost Games (0):     {}", zero_score_games);
        println!("---------------------------------------");
    }
}