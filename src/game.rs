use crate::knowledge::{self, Knowledge};
use crate::player::Player;
use crate::deck::Deck;
use crate::r#move::Move;
use crate::enums::Color;

pub struct Game {
    players: [Player; 2],
    deck: Deck,
    fireworks: [u8; 5],
    hints_remaining: u8,
    mistakes_made: u8,
    player_to_move: usize,
}

impl Game {
    pub fn new(player_strategies: [fn(&Player) -> Move; 2]) -> Self {
        let mut deck = Deck::new_full_deck();
        deck.shuffle();

        let mut players = [
            Player {
                hand: Vec::new(),
                hand_knowledge: [ Knowledge::new_full(); 5],
                infered_hand_knowledge: [ Knowledge::new_full(); 5],
                strategy: player_strategies[0],
            },
            Player {
                hand: Vec::new(),
                hand_knowledge: [ Knowledge::new_full(); 5],
                infered_hand_knowledge: [ Knowledge::new_full(); 5],
                strategy: player_strategies[1],
            },
        ];

        let mut game = Game {
            players,
            deck,
            fireworks: [0; 5],
            hints_remaining: 8,
            mistakes_made: 0,
            player_to_move: 0,
        };

        // Deal initial hands: pop from deck into each player's hand and notify the other player
        for player_index in 0..2 {
            for _ in 0..5 {
                let other_player_index = if player_index == 0 { 1 } else { 0 };
                let new_card = game.deck.cards.pop().expect("Deck is empty");
                game.players[player_index].hand.push(new_card);
                game.players[other_player_index].see(new_card);
            }
        }
        game
    }

    pub fn possible_moves(&self) -> Vec<Move> {
        let mut moves = Vec::new();
        let current_player = &self.players[self.player_to_move];

        // Play and Discard moves
        for card_index in 0..current_player.hand.len() {
            moves.push(Move::Play(card_index));
            moves.push(Move::Discard(card_index));
        }

        // Hint moves
        if self.hints_remaining > 0 {
            let other_player_index = if self.player_to_move == 0 { 1 } else { 0 };
            let other_player = &self.players[other_player_index];

            // Hint colors
            for color in [Color::Red, Color::Green, Color::Blue, Color::Yellow, Color::White].iter() {
                if other_player.hand.iter().any(|&card| card.get_color() == *color) {
                    moves.push(Move::HintColor(*color));
                }
            }

            // Hint values
            for value in 1..=5u8 {
                if other_player.hand.iter().any(|&card| card.get_value() == value) {
                    moves.push(Move::HintValue(value));
                }
            }
        }

        moves
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
        let card = self.players[self.player_to_move].hand[card_index];
        // both players see the played card
        for player in &mut self.players {
            player.see(card);
            player.other_player_sees(card);
        }

        let card_color_index = card.get_color() as usize;
        let card_value = card.get_value();
        if self.fireworks[card_color_index] + 1 == card_value {
            // Successful play
            self.fireworks[card_color_index] += 1;
        } else {
            // Failed play
            self.mistakes_made += 1;
        }
        self.players[self.player_to_move].hand.remove(card_index);
        // Draw a new card if possible
        if let Some(new_card) = self.deck.cards.pop() {
            self.players[self.player_to_move].hand.push(new_card);
            let other_player_index = if self.player_to_move == 0 { 1 } else { 0 };
            self.players[other_player_index].see(new_card);
        }
    }

    fn discard(&mut self, card_index: usize) {
        let card = self.players[self.player_to_move].hand.remove(card_index);
        // both players see the discarded card
        for player in &mut self.players {
            player.see(card);
            player.other_player_sees(card);
        }

        if self.hints_remaining < 8 {
            self.hints_remaining += 1;
        }
        // Draw a new card if possible
        if let Some(new_card) = self.deck.cards.pop() {
            self.players[self.player_to_move].hand.push(new_card);
            let other_player_index = if self.player_to_move == 0 { 1 } else { 0 };
            self.players[other_player_index].see(new_card);
        }
    }

    fn give_hint_color(&mut self, color: Color) {
        // generate the knowledge hint
        let knowledge_hint = Knowledge::from_color(color);
        let other_player_index = if self.player_to_move == 0 { 1 } else { 0 };
        let other_player = &mut self.players[other_player_index];
        other_player.get_hint(&knowledge_hint, 
            &other_player.hand.iter().enumerate()
                .filter_map(|(i, &card)| if card.get_color() == color { Some(i) } else { None })
                .collect::<Vec<usize>>()
        );
        if self.hints_remaining > 0 {
            self.hints_remaining -= 1;
        }
    }

    fn give_hint_value(&mut self, value: u8) {
        // generate the knowledge hint
        let knowledge_hint = Knowledge::from_value(value);
        let other_player_index = if self.player_to_move == 0 { 1 } else { 0 };
        let other_player = &mut self.players[other_player_index];
        other_player.get_hint(&knowledge_hint, 
            &other_player.hand.iter().enumerate()
                .filter_map(|(i, &card)| if card.get_value() == value { Some(i) } else { None })
                .collect::<Vec<usize>>()
        );
        if self.hints_remaining > 0 {
            self.hints_remaining -= 1;
        }
    }

    pub fn display_game_state(&self) {
        println!("Fireworks: {:?}", self.fireworks);
        println!("Hints remaining: {}", self.hints_remaining);
        println!("Mistakes made: {}", self.mistakes_made);
        for (i, player) in self.players.iter().enumerate() {
            println!("Player {}'s hand:", i);
            player.display_hand();
        }
    }
}