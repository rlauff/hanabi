use crate::enums::{Move, MoveResult, Color};
use crate::card::Card;
use crate::strategy::Strategy;
use std::io::{self, Write};
use crate::decksubset::DeckSubset;



pub struct Human { 
    hints_remaining: u8,
    fireworks: [u8; 5],
    my_hand_knowledge: Vec<DeckSubset>,
    partner_hand: Vec<Card>,
    partner_hand_knowledge: Vec<DeckSubset>,
    cards_not_seen: DeckSubset
}

impl Human {
    pub fn new() -> Self {
        Human {
            hints_remaining: 8,
            fireworks: [0; 5],
            my_hand_knowledge: vec![DeckSubset::new_full(); 5],
            partner_hand: Vec::new(),
            partner_hand_knowledge: vec![DeckSubset::new_full(); 5],
            cards_not_seen: DeckSubset::new_full()
        }
    }
}

impl Strategy for Human {
    fn initialize(&mut self, other_player_hand: &Vec<Card>) {
        self.partner_hand = other_player_hand.clone();
        for card in other_player_hand {
            self.cards_not_seen.remove_card(card);
        }
    }

    fn decide_move(&mut self) -> Move {
        // just ask the user for input
        print!("Enter your move: ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let parts: Vec<&str> = input.trim().split_whitespace().collect();
        match parts.as_slice() {
            ["play", index_str] => {
                let index: usize = index_str.parse::<usize>().unwrap()-1;
                Move::Play(index)
            },
            ["discard", index_str] => {
                let index: usize = index_str.parse::<usize>().unwrap()-1;
                Move::Discard(index)
            },
            ["hint", hint_str] => {
                match *hint_str {
                    "red" => Move::HintColor(Color::Red),
                    "green" => Move::HintColor(Color::Green),
                    "blue" => Move::HintColor(Color::Blue),
                    "yellow" => Move::HintColor(Color::Yellow),
                    "white" => Move::HintColor(Color::White),
                    "1" => Move::HintValue(1),
                    "2" => Move::HintValue(2),
                    "3" => Move::HintValue(3),
                    "4" => Move::HintValue(4),
                    "5" => Move::HintValue(5),
                    _ => panic!("Invalid hint"),
                }
            },
            _ => panic!("Invalid move format"),
        }
    }

    fn update_after_own_move(&mut self, mv: &Move, mv_result: &MoveResult, got_new_card: bool) {
        match mv {
            Move::Play(idx) => {
                match mv_result {
                    MoveResult::Play(success, card_played, _) => {
                        if *success {
                            // Update fireworks
                            let color_index = card_played.get_color() as usize;
                            self.fireworks[color_index] += 1;
                        } 
                        // Remove played card knowledge
                        self.my_hand_knowledge.remove(*idx);
                        if got_new_card {
                            self.my_hand_knowledge.push(DeckSubset::new_full());
                        }
                    },
                    _ => ()
                }
            }
            Move::Discard(idx) => {
                // Remove discarded card knowledge
                self.my_hand_knowledge.remove(*idx);
                if got_new_card {
                    self.my_hand_knowledge.push(DeckSubset::new_full());
                }
                if self.hints_remaining < 8 {
                    self.hints_remaining += 1;
                }
            }
            Move::HintColor(color) => {
                self.hints_remaining -= 1;
                // Update partner's hand knowledge based on hint
                match mv_result {
                    MoveResult::Hint(indices) => {
                        for i in indices.iter() {
                            self.partner_hand_knowledge[*i] = self.partner_hand_knowledge[*i].intersect(&DeckSubset::from_color(*color));
                        }
                    },
                    _ => ()
                }
            }
            Move::HintValue(value) => {
                self.hints_remaining -= 1;
                // Update partner's hand knowledge based on hint
                match mv_result {
                    MoveResult::Hint(indices) => {
                        for i in indices.iter() {
                            self.partner_hand_knowledge[*i] = self.partner_hand_knowledge[*i].intersect(&DeckSubset::from_value(*value));
                        }
                    },
                    _ => ()
                }
            }
        }
    }

    fn update_after_other_player_move(&mut self, mv: &Move, mv_result: &MoveResult) {
        match mv {
            Move::Play(idx) => {
                match mv_result {
                    MoveResult::Play(success, card_played, card_drawn) => {
                        self.cards_not_seen.remove_card(card_played); // both see this card
                        if *success {
                            // Update fireworks
                            let color_index = card_played.get_color() as usize;
                            self.fireworks[color_index] += 1;
                        } 
                        // Remove played card knowledge and hand and add new card if drawn
                        self.my_hand_knowledge.remove(*idx);
                        self.partner_hand.remove(*idx);
                        if let Some(card) = card_drawn {
                            self.partner_hand.push(*card);
                            self.my_hand_knowledge.push(DeckSubset::new_full());
                            self.cards_not_seen.remove_card(card);
                        }
                    },
                    _ => ()
                }
            }
            Move::Discard(idx) => {
                match mv_result {
                    MoveResult::Discard(card_discarded, card_drawn) => {
                        self.cards_not_seen.remove_card(card_discarded); // both see this card
                        if self.hints_remaining < 8 {
                            self.hints_remaining += 1;
                        }
                        // Remove played card knowledge and hand and add new card if drawn
                        self.my_hand_knowledge.remove(*idx);
                        self.partner_hand.remove(*idx);
                        if let Some(card) = card_drawn {
                            self.partner_hand.push(*card);
                            self.my_hand_knowledge.push(DeckSubset::new_full());
                            self.cards_not_seen.remove_card(card);
                        }
                    },
                    _ => ()
                }
            }
            Move::HintColor(color) => {
                self.hints_remaining -= 1;
                // Update own's hand knowledge based on hint
                match mv_result {
                    MoveResult::Hint(indices) => {
                        for i in indices.iter() {
                            self.my_hand_knowledge[*i] = self.my_hand_knowledge[*i].intersect(&DeckSubset::from_color(*color));
                        }
                    },
                    _ => ()
                }
            }
            Move::HintValue(value) => {
                self.hints_remaining -= 1;
                // Update own's hand knowledge based on hint
                match mv_result {
                    MoveResult::Hint(indices) => {
                        for i in indices.iter() {
                            self.my_hand_knowledge[*i] = self.my_hand_knowledge[*i].intersect(&DeckSubset::from_value(*value));
                        }
                    },
                    _ => ()
                }
            }
        }
    }
}