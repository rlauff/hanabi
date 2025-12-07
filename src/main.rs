mod enums;
mod card;
mod deck;
mod player;
mod game;
mod decksubset;
mod strategy;
mod strategies;

use std::env;
use crate::game::Game;
use crate::player::Player;
use crate::strategy::Strategy;
use crate::enums::Move;

// Number of games to run in benchmark mode
const GAMES_TO_SIMULATE: u32 = 10000;

fn main() {
    // Registry of strategies. Uses closures to create fresh instances for every game.
    let all_strategies: Vec<(&str, fn() -> Box<dyn Strategy>)> = vec![
        ("Random Only Play", || Box::new(strategies::random_only_play::RandomOnlyPlay::new())),
        ("Gemini", || Box::new(strategies::gemini::Gemini::new())),
    ];

    // --- Argument Parsing ---
    let args: Vec<String> = env::args().collect();
    let mut single_mode = false;
    let mut selected_strategy_inputs: Vec<String> = Vec::new();

    let mut i = 1;
    while i < args.len() {
        let arg = &args[i];
        if arg == "--single" {
            single_mode = true;
        } else if arg.starts_with("--strategies=") {
            // Handle format: --strategies=gemini,random
            let value = arg.trim_start_matches("--strategies=");
            let parts: Vec<String> = value.split(',')
                .map(|s| s.trim().to_lowercase())
                .filter(|s| !s.is_empty())
                .collect();
            selected_strategy_inputs.extend(parts);
        } else if arg == "--strategies" {
            // Handle format: --strategies gemini,random
            if i + 1 < args.len() {
                let value = &args[i+1];
                let parts: Vec<String> = value.split(',')
                    .map(|s| s.trim().to_lowercase())
                    .filter(|s| !s.is_empty())
                    .collect();
                selected_strategy_inputs.extend(parts);
                i += 1; // Skip next arg since we consumed it
            }
        }
        i += 1;
    }

    // --- Strategy Filtering ---
    let strategies_to_run: Vec<_> = all_strategies.into_iter().filter(|(name, _)| {
        if selected_strategy_inputs.is_empty() {
            return true; // If no specific strategies requested, run all
        }

        let name_lower = name.to_lowercase();
        let name_words: Vec<&str> = name_lower.split_whitespace().collect();

        // Check if user input matches the full name OR a specific word in the name
        // e.g. "random" matches "Random Only Play", but "om" does not.
        selected_strategy_inputs.iter().any(|input| {
            name_lower == *input || name_words.contains(&input.as_str())
        })
    }).collect();

    if strategies_to_run.is_empty() {
        println!("No strategies matched your selection: {:?}", selected_strategy_inputs);
        println!("Available strategies:");
        println!(" - Random Only Play");
        println!(" - Gemini");
        return;
    }

    // --- Execution ---
    for (name, create_strategy_fn) in strategies_to_run {
        println!("Running Strategy: {}", name);
        if single_mode {
            run_single_game(name, create_strategy_fn);
        } else {
            run_benchmark(name, create_strategy_fn);
        }
        println!("---------------------------------------");
    }
}

/// Runs 10,000 games and prints statistics
fn run_benchmark(_name: &str, create_strategy_fn: fn() -> Box<dyn Strategy>) {
    println!("Simulating {} games...", GAMES_TO_SIMULATE);

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

    println!("  -> Average Score:     {:.4}", average_score);
    println!("  -> Perfect Games (25): {}", perfect_games);
    println!("  -> Lost Games (0):     {}", zero_score_games);
}

/// Runs a single game and prints step-by-step details
fn run_single_game(_name: &str, create_strategy_fn: fn() -> Box<dyn Strategy>) {
    let p1 = Player::new(create_strategy_fn());
    let p2 = Player::new(create_strategy_fn());
    let mut game = Game::new(p1, p2);
    let mut turn_count = 1;

    // NOTE: This function requires public access to Game fields (players, player_to_move).
    // Ensure `pub` is added to the fields in game.rs if not already present.

    loop {
        // Check for game over condition
        if let Some(final_score) = game.game_over() {
            println!("\nGame Over!");
            println!("Final Score: {}", final_score);
            break;
        }

        println!("\nMove {}:", turn_count);

        // We determine the move manually here for display purposes before applying it.
        // This mirrors the logic in game.advance().
        let player_index = game.player_to_move; 
        let selected_move = game.players[player_index].strategy.decide_move();

        // Print Player 1
        print!("Player 1: {}", game.players[0]);
        if player_index == 0 {
            println!(" -> {}", format_move(&selected_move, &game));
        } else {
            println!();
        }

        // Print Player 2
        print!("Player 2: {}", game.players[1]);
        if player_index == 1 {
            println!(" -> {}", format_move(&selected_move, &game));
        } else {
            println!();
        }

        game.apply_move(selected_move);
        turn_count += 1;
    }
}

fn format_move(mv: &Move, game: &Game) -> String {
    match mv {
        Move::Play(idx) => format!("Play {}", game.players[game.player_to_move].hand[*idx]),
        Move::Discard(idx) => format!("Discard {}", game.players[game.player_to_move].hand[*idx]),
        Move::HintColor(color) => format!("Hint Color {:?}", color),
        Move::HintValue(val) => format!("Hint Value {}", val),
    }
}
