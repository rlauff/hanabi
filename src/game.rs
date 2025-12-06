use rand::rand_core::le;

use crate::knowledge::{self, Knowledge};
use crate::player::Player;
use crate::deck::Deck;
use crate::enums::Color;
use crate::{card, enums::*};

pub struct Game {
    players: [Player; 2],
    deck: Deck,
    fireworks: [u8; 5],
    hints_remaining: u8,
    mistakes_made: u8,
    player_to_move: usize,
}

impl Game {
    pub fn new(player1: Player, player2: Player) -> Self {
        let mut deck = Deck::new_full_deck();
        deck.shuffle();

        let mut players = [player1, player2];

        let mut game = Game {
            players,
            deck,
            fireworks: [0; 5],
            hints_remaining: 8,
            mistakes_made: 0,
            player_to_move: 0,
        };

        // Deal initial hands
        let mut player0_hand = Vec::new();
        let mut player1_hand = Vec::new();
        for _ in 0..5 {
            player0_hand.push(game.players[0].draw(&mut game.deck));
            player1_hand.push(game.players[1].draw(&mut game.deck));
        }

        // initialize players stretegy with other player's hand
        game.players[0].strategy.initialize(&player1_hand);
        game.players[1].strategy.initialize(&player0_hand);

        game
    }

    pub fn advance(&mut self) {
        let player_index = self.player_to_move;
        let selected_move = self.players[player_index].strategy.decide_move();
        self.apply_move(selected_move);
    }

    pub fn apply_move(&mut self, mv: Move) {
        match mv {
            Move::Play(card_index) => self.play(card_index),
            Move::Discard(card_index) => self.discard(card_index),
            Move::HintColor(color) => self.give_hint_color(color),
            Move::HintValue(value) => self.give_hint_value(value),
        }
        self.player_to_move = if self.player_to_move == 0 { 1 } else { 0 };
    }

    fn play(&mut self, card_index: usize) {
        // println!("Player {} plays card {} at index {}", self.player_to_move, self.players[self.player_to_move].hand[card_index], card_index);
        let card = self.players[self.player_to_move].hand[card_index];
        let card_color_index = card.get_color() as usize;
        let card_value = card.get_value();

        self.players[self.player_to_move].hand.remove(card_index);
        // Draw a new card if possible
        let got_new_card: bool;
        if let Some(new_card) = self.deck.cards.pop() {
            self.players[self.player_to_move].hand.push(new_card);
            let other_player_index = if self.player_to_move == 0 { 1 } else { 0 };
            self.players[other_player_index].strategy.see(&new_card);
            got_new_card = true;
        } else {
            got_new_card = false;
        }

        if self.fireworks[card_color_index] + 1 == card_value {
            // Successful play
            self.fireworks[card_color_index] += 1;
            self.players[self.player_to_move].strategy.update_after_own_move(&Move::Play(card_index), &MoveResult::Play(true, card), got_new_card);
            let other_player_index = if self.player_to_move == 0 { 1 } else { 0 };
            self.players[other_player_index].strategy.update_after_other_player_move(&Move::Play(card_index), &MoveResult::Play(true, card));
        } else {
            // Failed play
            self.mistakes_made += 1;
            self.players[self.player_to_move].strategy.update_after_own_move(&Move::Play(card_index), &MoveResult::Play(false, card), got_new_card);
            let other_player_index = if self.player_to_move == 0 { 1 } else { 0 };
            self.players[other_player_index].strategy.update_after_other_player_move(&Move::Play(card_index), &MoveResult::Play(false, card));
        }
    }

    fn discard(&mut self, card_index: usize) {
        let card = self.players[self.player_to_move].hand.remove(card_index);
        if self.hints_remaining < 8 {
            self.hints_remaining += 1;
        }
         // Draw a new card if possible
        let got_new_card: bool;
        if let Some(new_card) = self.deck.cards.pop() {
            self.players[self.player_to_move].hand.push(new_card);
            let other_player_index = if self.player_to_move == 0 { 1 } else { 0 };
            self.players[other_player_index].strategy.see(&new_card);
            got_new_card = true;
        } else {
            got_new_card = false;
        }

        self.players[self.player_to_move].strategy.update_after_own_move(&Move::Discard(card_index), &MoveResult::Discard(card), got_new_card);
        let other_player_index = if self.player_to_move == 0 { 1 } else { 0 };
        self.players[other_player_index].strategy.update_after_other_player_move(&Move::Discard(card_index), &MoveResult::Discard(card));
    }

    fn give_hint_color(&mut self, color: Color) {
        if self.hints_remaining == 0 {
            panic!("No hints remaining");
        }
        self.hints_remaining -= 1;
        let other_player_index = if self.player_to_move == 0 { 1 } else { 0 };
        let other_player = &self.players[other_player_index];
        let mut hinted_indices = other_player.hand.iter().enumerate()
            .filter(|(_, card)| card.get_color() == color)
            .map(|(index, _)| index)
            .collect::<Vec<usize>>();

        let knowledge_updates = hinted_indices.iter().map(|x| Knowledge::from_color(color)).collect::<Vec<Knowledge>>();

        self.players[self.player_to_move].strategy.update_after_own_move(&Move::HintColor(color), &MoveResult::Hint(hinted_indices.clone(), knowledge_updates.clone()), false);
        self.players[other_player_index].strategy.update_after_other_player_move(&Move::HintColor(color), &MoveResult::Hint(hinted_indices, knowledge_updates));
    }

    fn give_hint_value(&mut self, value: u8) {
        if self.hints_remaining == 0 {
            panic!("No hints remaining");
        }
        self.hints_remaining -= 1;
        let other_player_index = if self.player_to_move == 0 { 1 } else { 0 };
        let other_player = &self.players[other_player_index];
        let mut hinted_indices = other_player.hand.iter().enumerate()
            .filter(|(_, card)| card.get_value() == value)
            .map(|(index, _)| index)
            .collect::<Vec<usize>>();

        let knowledge_updates = hinted_indices.iter().map(|x| Knowledge::from_value(value)).collect::<Vec<Knowledge>>();

        self.players[self.player_to_move].strategy.update_after_own_move(&Move::HintValue(value), &MoveResult::Hint(hinted_indices.clone(), knowledge_updates.clone()), false);
        self.players[other_player_index].strategy.update_after_other_player_move(&Move::HintValue(value), &MoveResult::Hint(hinted_indices, knowledge_updates));
    }

    // pub fn display_game_state(&self) {
    //     println!("Fireworks: {:?}", self.fireworks);
    //     println!("Hints remaining: {}", self.hints_remaining);
    //     println!("Mistakes made: {}", self.mistakes_made);
    //     for (i, player) in self.players.iter().enumerate() {
    //         println!("Player {}'s hand:", i);
    //         player.display_hand();
    //     }
    // }

    pub fn game_over(&self) -> Option<u8> {
        if self.mistakes_made >= 3 || self.fireworks.iter().all(|&f| f == 5) || (self.deck.cards.is_empty() && self.players.iter().all(|p| p.hand.len() == 4)) {
            let score: u8 = self.fireworks.iter().sum();
            Some(score)
        } else {
            None
        }
    }
}