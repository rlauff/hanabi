use crate::knowledge::Knowledge;
use crate::player::Player;
use crate::deck::Deck;
use crate::r#move::Move;

pub struct Game {
    players: [Player; 2],
    deck: Deck,
    fireworks: [u8; 5], // one for each color
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