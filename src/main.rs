use rayon::prelude::*;
use std::rc::Rc;
use std::cell::RefCell;
use crate::strategies::cheater::{Cheater, CheatSharedState};

mod enums;
mod card;
mod deck;
mod player;
mod game;
mod decksubset;
mod strategy;
mod strategies;
mod evolve_robert;

use std::env;
use crate::game::Game;
use crate::player::Player;
use crate::strategy::Strategy;
use crate::enums::Move;

// Number of games to run in benchmark mode
const GAMES_TO_SIMULATE: u32 = 10000;

type StrategyFactory = fn() -> Box<dyn Strategy>;

fn main() {

    // Registry of strategies.
    let all_strategies: Vec<(&str, StrategyFactory)> = vec![
        ("Gemini", || Box::new(strategies::gemini::Gemini::new())),
        ("ChatGPT", || Box::new(strategies::chatgpt::ChatGPT::new())),
        ("Robert", || Box::new(strategies::robert::Robert::new())),
        ("Human", || Box::new(strategies::human::Human::new())),
    ];

    // --- Argument Parsing ---
    let args: Vec<String> = env::args().collect();

    // Check for evolution mode
    if args.contains(&"evolve-robert".to_string()) {
        evolve_robert::run_evolution();
        return;
    }

    // Check for Cheater simulation mode
    if args.contains(&"--cheater".to_string()) {
        run_cheater_benchmark();
        return;
    }

    // Find selected strategies based on args
    let mut selected_strategies: Vec<(&str, StrategyFactory)> = Vec::new();

    // We look for strategy names in the arguments preserving order (optional, but good for P1 vs P2)
    // If we iterate through args, we can pick them up.
    // Alternatively, just iterate the registry and check containment to allow unordered args.
    // The prompt implies "two strategy names in any order".
    // Let's filter the args to find valid strategy names.

    for arg in &args {
        if let Some(pair) = all_strategies.iter().find(|(name, _)| *name == arg) {
            selected_strategies.push(*pair);
        }
    }

    // Default fallback if not enough args provided
    if selected_strategies.len() < 2 {
        println!("Not enough strategies specified. Usage: cargo run -- <Strat1> <Strat2> [--single]");
        println!("Available strategies: {:?}", all_strategies.iter().map(|(n, _)| n).collect::<Vec<_>>());
        // For safety, just exit or default to something safe if you prefer
        return;
    }

    // Take the first two found
    let (p1_name, p1_factory) = selected_strategies[0];
    let (p2_name, p2_factory) = selected_strategies[1];

    let mut single_mode = args.contains(&"--single".to_string());

    // Force single mode if Human is involved
    if p1_name == "Human" || p2_name == "Human" && !single_mode {
        single_mode = true;
        println!("Human player detected: Forcing single game mode.");
    }

    // --- Execution ---
    println!("Matchup: P1 [{}] vs P2 [{}]", p1_name, p2_name);

    if single_mode {
        run_single_game(p1_name, p1_factory, p2_name, p2_factory);
    } else {
        run_benchmark(p1_factory, p2_factory);
    }
}

fn run_single_game_bench(strat1: StrategyFactory, strat2: StrategyFactory) -> u8 {
    let p1 = Player::new(strat1());
    let p2 = Player::new(strat2());
    let mut game = Game::new(p1, p2);

    // Run game loop until game_over returns a score
    loop {
        if let Some(final_score) = game.game_over() {
            return final_score;
        }
        game.advance();
    };
}

/// Runs GAMES_TO_SIMULATE games and prints statistics
fn run_benchmark(p1_factory: StrategyFactory, p2_factory: StrategyFactory) {
    println!("Simulating {} games...", GAMES_TO_SIMULATE);

    let scores: Vec<u8> = (0..GAMES_TO_SIMULATE)
                    .into_par_iter()
                    .map(|_| run_single_game_bench(p1_factory, p2_factory))
                    .collect();

    let mut total_score: u32 = 0;
    let mut perfect_games = 0;
    let mut zero_score_games = 0;

    for score in scores.iter() {
        total_score += *score as u32;
        if *score == 25 {
            perfect_games += 1;
        }
        if *score == 0 {
            zero_score_games += 1;
        }
    }
    let average_score = total_score as f64 / GAMES_TO_SIMULATE as f64;
    println!("  -> Average Score:     {:.4}", average_score);
    println!("  -> Perfect Games (25): {}", perfect_games);
    println!("  -> Lost Games (0):     {}", zero_score_games);
}

/// Runs a single game and prints step-by-step details
fn run_single_game(p1_name: &str, p1_factory: StrategyFactory, p2_name: &str, p2_factory: StrategyFactory) {
    let p1 = Player::new(p1_factory());
    let p2 = Player::new(p2_factory());
    let mut game = Game::new(p1, p2);
    let mut turn_count = 1;

    let p1_is_human = p1_name == "Human";
    let p2_is_human = p2_name == "Human";

    loop {
        // Check for game over condition
        if let Some(final_score) = game.game_over() {
            println!("\nGame Over!");
            println!("Final Score: {}", final_score);
            break;
        }

        println!("\n---------------------------------------");
        println!("Move {}:", turn_count);

        // We determine the move manually here for display purposes before applying it.
        let player_index = game.player_to_move;

        // Before asking for the move, print the game state from the perspective of an observer,
        // BUT hide hands if necessary.

        // Print Player 1
        print!("Player 1 ({}): ", p1_name);
        if p1_is_human {
             println!("[HIDDEN HAND]");
        } else {
             println!("{}", game.players[0]);
        }

        // Print Player 2
        print!("Player 2 ({}): ", p2_name);
        if p2_is_human && false{
             println!("[HIDDEN HAND]");
        } else {
             println!("{}", game.players[1]);
        }

        println!("Fireworks: \x1b[31m{}\x1b[0m, \x1b[32m{}\x1b[0m, \x1b[34m{}\x1b[0m, \x1b[33m{}\x1b[0m, \x1b[37m{}\x1b[0m", game.fireworks[0], game.fireworks[1], game.fireworks[2], game.fireworks[3], game.fireworks[4]);

        let selected_move = game.players[player_index].strategy.decide_move();

        // Print the move chosen
        let current_player_name = if player_index == 0 { p1_name } else { p2_name };
        println!("{} plays -> {}", current_player_name, format_move(&selected_move, &game));

        game.apply_move(selected_move);
        turn_count += 1;
    }
}

fn format_move(mv: &Move, game: &Game) -> String {
    let player_idx = game.player_to_move;
    match mv {
        Move::Play(idx) => {
            // Zeige Karte, die gespielt wird
            format!("Play index {} ({})", idx+1, game.players[player_idx].hand[*idx])
        },
        Move::Discard(idx) => {
            // Zeige Karte, die abgeworfen wird
            format!("Discard index {} ({})", idx+1, game.players[player_idx].hand[*idx])
        },
        Move::HintColor(color) => {
            // Berechne die betroffenen Indizes beim ANDEREN Spieler
            let target_idx = if player_idx == 0 { 1 } else { 0 };
            let indices: Vec<usize> = game.players[target_idx].hand.iter().enumerate()
                .filter(|(_, card)| card.get_color() == *color)
                .map(|(i, _)| i)
                .collect();
            format!("Hint Color {:?} -> Indices {:?}", color, indices.iter().map(|x| x+1).collect::<Vec<_>>())
        },
        Move::HintValue(val) => {
            // Berechne die betroffenen Indizes beim ANDEREN Spieler
            let target_idx = if player_idx == 0 { 1 } else { 0 };
            let indices: Vec<usize> = game.players[target_idx].hand.iter().enumerate()
                .filter(|(_, card)| card.get_value() == *val)
                .map(|(i, _)| i)
                .collect();
            format!("Hint Value {} -> Indices {:?}", val, indices.iter().map(|x| x+1).collect::<Vec<_>>())
        },
    }
}

fn run_cheater_benchmark() {
    println!("Simulating {} games (Cheater vs Cheater)...", GAMES_TO_SIMULATE);

    // paralell with rayon
    let results: Vec<u8> = (0..GAMES_TO_SIMULATE)
        .into_par_iter()
        .map(|_| {
            // Setup Shared States
            let cheat_state_p1 = Rc::new(RefCell::new(CheatSharedState::default()));
            let cheat_state_p2 = Rc::new(RefCell::new(CheatSharedState::default()));

            // Setup strategies
            let strat1 = Box::new(Cheater::new(cheat_state_p1.clone()));
            let strat2 = Box::new(Cheater::new(cheat_state_p2.clone()));

            // setup game
            let p1 = Player::new(strat1);
            let p2 = Player::new(strat2);
            let mut game = Game::new(p1, p2);

            // game loop with injection of the illegal information
            loop {
                if let Some(score) = game.game_over() {
                    return score;
                }
                {
                    let mut s1 = cheat_state_p1.borrow_mut();
                    s1.my_hand = game.players[0].hand.clone();
                    s1.partner_hand = game.players[1].hand.clone();
                    s1.hints_remaining = game.hints_remaining.clone();
                    s1.fireworks = game.fireworks.clone();
                    s1.deck_cards = game.deck.cards.clone();
                }
                {
                    let mut s2 = cheat_state_p2.borrow_mut();
                    s2.my_hand = game.players[1].hand.clone();
                    s2.partner_hand = game.players[0].hand.clone();
                    s2.hints_remaining = game.hints_remaining.clone();
                    s2.fireworks = game.fireworks.clone();
                    s2.deck_cards = game.deck.cards.clone();
                }

                game.advance();
            }
        })
        .collect();

    // Results summary
    let total_score: u32 = results.iter().map(|&s| s as u32).sum();
    let avg_score = total_score as f64 / GAMES_TO_SIMULATE as f64;
    let perfect_games = results.iter().filter(|&&s| s == 25).count();
    let perfect_percent = (perfect_games as f64 / GAMES_TO_SIMULATE as f64) * 100.0;

    println!("Average Score: {:.4}", avg_score);
    println!("Perfect Games: {} ({:.2}%)", perfect_games, perfect_percent);
}