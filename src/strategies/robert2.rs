use crate::enums::{Move, MoveResult, Color};
use crate::card::Card;
use crate::strategy::Strategy;
use crate::decksubset::DeckSubset;
use std::fs;
use std::str::FromStr;

// robert2.rs


pub struct Robert2 {
    hints_remaining: u8,
    mistakes_made: u8,
    fireworks: [u8; 5],
    number_moves_made: u8,      // the number of moves made by this player before the current one
    my_hand_knowledge: Vec<DeckSubset>,
    partner_hand: Vec<Card>,
    partner_hand_knowledge: Vec<DeckSubset>,
    cards_not_seen: DeckSubset,
    play_next: Vec<usize>,      // holding the results of the focused hints. These cards should be played in this order next
    partner_play_next: Vec<usize>, // holding the results of the focused hints for the partner. These cards should be played in this order next
}

impl Robert2 {
    pub fn new() -> Self {
        Robert2 {
            hints_remaining: 8,
            mistakes_made: 0,
            fireworks: [0; 5],
            number_moves_made: 0,
            my_hand_knowledge: vec![DeckSubset::new_full(); 5],
            partner_hand: Vec::new(),
            partner_hand_knowledge: vec![DeckSubset::new_full(); 5],
            cards_not_seen: DeckSubset::new_full(),
            play_next: Vec::new(),
            partner_play_next: Vec::new(),
        }
    }

    fn all_possible_moves(&self) -> Vec<Move> {
        let mut all_moves: Vec<Move> = Vec::new();
        // play and discard moves
        for i in 0..self.my_hand_knowledge.len() {
            all_moves.push(Move::Play(i));
            all_moves.push(Move::Discard(i));
        }
        // hint moves
        if self.hints_remaining > 0 {
            for value in 1..6 {
                all_moves.push(Move::HintValue(value));
            }
            for color in [Color::Red, Color::Green, Color::Blue, Color::Yellow, Color::White] {
                all_moves.push(Move::HintColor(color));
            }
        }
        all_moves
    }

    fn exact_card_if_known(&self, idx: usize) -> Option<Card> {
        // pick the first 1 in the knowledge bitset to get a potential card
        // then check if this is really the only card in the knowledge
        let knowledge = &self.my_hand_knowledge[idx];
        // find position of first 1
        let first_card_index = knowledge.0.trailing_zeros() as u8;
        // the cards in the decksubset struct are ordered in the same order as Card takes them, so this index is directly usable
        let card = Card::new(first_card_index);
        let card_subset = DeckSubset::from_card_type(&card);
        knowledge.is_subset(&card_subset).then(|| card)

    }

    fn playable_cards(&self) -> DeckSubset {
        let mut playable = DeckSubset::new_empty();
        for (color_index, &top_value) in self.fireworks.iter().enumerate() {
            if top_value < 5 {
                let color = match color_index {
                    0 => Color::Red,
                    1 => Color::Green,
                    2 => Color::Blue,
                    3 => Color::Yellow,
                    4 => Color::White,
                    _ => unreachable!(),
                };
                let next_card_subset = DeckSubset::from_color(color)
                    .intersect(&DeckSubset::from_value(top_value + 1));
                playable = playable.union(&next_card_subset);
            }
        }
        playable
    }

    fn discardable_cards(&self) -> DeckSubset {
        // a card is discardable if fireworks already has it or higher
        let mut discardable = DeckSubset::new_empty();
        for (color_index, &top_value) in self.fireworks.iter().enumerate() {
            for value in 1..=top_value {
                let color = match color_index {
                    0 => Color::Red,
                    1 => Color::Green,
                    2 => Color::Blue,
                    3 => Color::Yellow,
                    4 => Color::White,
                    _ => unreachable!(),
                };
                let next_card_subset = DeckSubset::from_color(color)
                    .intersect(&DeckSubset::from_value(value));
                discardable = discardable.union(&next_card_subset);
            }
        }
        discardable
    }

    // the probability of a card being playable/discardable based on knowledge
    fn probability_playable(&self, idx: usize) -> f64 {
        // divide number of playable cards in knowledge by total number of cards in knowledge
        // intersect with cards not seen to only count cards that could still be in hand
        self.cards_not_seen.intersect(&self.my_hand_knowledge[idx].intersect(&self.playable_cards())).0.count_ones() as f64 /
            self.cards_not_seen.intersect(&self.my_hand_knowledge[idx]).0.count_ones() as f64
    }
    fn probability_discardable(&self, idx: usize) -> f64 {
        // divide number of discardable cards in knowledge by total number of cards in knowledge
        // intersect with cards not seen to only count cards that could still be in hand
        self.cards_not_seen.intersect(&self.my_hand_knowledge[idx].intersect(&&self.discardable_cards())).0.count_ones() as f64 /
            self.cards_not_seen.intersect(&self.my_hand_knowledge[idx]).0.count_ones() as f64
    }

    // the probability of a card being playable/discardable based on knowledge from partners perspective
    fn partner_probability_playable(&self, idx: usize, hint: Option<Move>) -> f64 {
        // if we pass a hint, then we want to know the probability after this hint is given, so we intersect with it
        let hint_subset = if let Some(h) = hint {
            match h {
                Move::HintColor(color) => { DeckSubset::from_color(color) },
                Move::HintValue(value) => { DeckSubset::from_value(value) },
                _ => unreachable!()
            }
        } else {
            DeckSubset::new_full()
        };
        // divide number of playable cards in knowledge by total number of cards in knowledge
        // intersect with cards not seen to only count cards that could still be in hand
        hint_subset.intersect(&self.cards_not_seen.intersect(&self.partner_hand_knowledge[idx].intersect(&self.playable_cards()))).0.count_ones() as f64 /
            hint_subset.intersect(&self.cards_not_seen.intersect(&self.partner_hand_knowledge[idx])).0.count_ones() as f64
    }

    fn partner_probability_discardable(&self, idx: usize, hint: Option<Move>) -> f64 {
        // if we pass a hint, then we want to know the probability after this hint is given, so we intersect with it
        let hint_subset = if let Some(h) = hint {
            match h {
                Move::HintColor(color) => { DeckSubset::from_color(color) },
                Move::HintValue(value) => { DeckSubset::from_value(value) },
                _ => unreachable!()
            }
        } else {
            DeckSubset::new_full()
        };
        // divide number of discardable cards in knowledge by total number of cards in knowledge
        // intersect with cards not seen to only count cards that could still be in hand
        hint_subset.intersect(&self.cards_not_seen.intersect(&self.partner_hand_knowledge[idx].intersect(&&self.discardable_cards()))).0.count_ones() as f64 /
            hint_subset.intersect(&self.cards_not_seen.intersect(&self.partner_hand_knowledge[idx])).0.count_ones() as f64
    }

    // the probability of being the only card left of its kind
    fn probability_only_card_left_of_its_kind(&self, idx: usize) -> f64{
        let mut number_only_card_left = 0;
        for value in 0..4 {
            for color_idx in  0..4 {
                let card_subset = DeckSubset::from_card_type(&Card::from_value_color_idx(value, color_idx));
                if card_subset.intersect(&self.my_hand_knowledge[idx]).intersect(&self.cards_not_seen).0.count_ones() == 1 {
                    number_only_card_left += 1;
                }
            }
        }
        number_only_card_left as f64 / self.my_hand_knowledge[idx].intersect(&self.cards_not_seen).0.count_ones() as f64
    }

    fn number_of_cards_excluded_by_color_hint(&self, color: Color) -> [u8; 5] {
        let mut number_of_cards_excluded_array = [0u8; 5];
        for i in 0..self.partner_hand_knowledge.len() {
            if self.partner_hand[i].get_color() == color {
                // intersect the subset of all cards that could be in this hand position by the set of cards which do not have this color
                // this is the number of cards that has been excluded by this hint for this card
                let number_of_cards_excluded = self.cards_not_seen
                    .intersect(&self.partner_hand_knowledge[i])
                    .intersect(&DeckSubset::from_color_inverted(color)).0.count_ones();
                number_of_cards_excluded_array[i] = number_of_cards_excluded as u8;
            } else {
                // in this case, the partner learns that this card is not of this color, i.e. all cards of this color are excluded
                let number_of_cards_excluded = self.cards_not_seen
                    .intersect(&self.partner_hand_knowledge[i])
                    .intersect(&DeckSubset::from_color(color)).0.count_ones();
                number_of_cards_excluded_array[i] = number_of_cards_excluded as u8;
            }
        }
        number_of_cards_excluded_array
    }

    fn number_of_cards_excluded_by_value_hint(&self, value: u8) -> [u8; 5] {
        let mut number_of_cards_excluded_array = [0u8; 5];
        for i in 0..self.partner_hand_knowledge.len() {
            if self.partner_hand[i].get_value() == value {
                // intersect the subset of all cards that could be in this hand position by the set of cards which do not have this value
                // this is the number of cards that has been excluded by this hint for this card
                let number_of_cards_excluded = self.cards_not_seen
                    .intersect(&self.partner_hand_knowledge[i])
                    .intersect(&&DeckSubset::from_value_inverted(value)).0.count_ones();
                number_of_cards_excluded_array[i] = number_of_cards_excluded as u8;
            } else {
                // in this case, the partner learns that this card is not of this value, i.e. all cards of this value are excluded
                let number_of_cards_excluded = self.cards_not_seen
                    .intersect(&self.partner_hand_knowledge[i])
                    .intersect(&&DeckSubset::from_value(value)).0.count_ones();
                number_of_cards_excluded_array[i] = number_of_cards_excluded as u8;
            }
        }
        number_of_cards_excluded_array
    }
}

impl Strategy for Robert2 {
    fn initialize(&mut self, other_player_hand: &Vec<Card>) {
        self.partner_hand = other_player_hand.clone();
        for card in other_player_hand {
            self.cards_not_seen.remove_card(card);
        }
    }

    fn decide_move(&mut self) -> Move {
       unimplemented!()
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
                        } else {
                            self.mistakes_made += 1;
                        }
                        // Remove played card knowledge
                        self.my_hand_knowledge.remove(*idx);
                        if got_new_card {
                            self.my_hand_knowledge.push(DeckSubset::new_full());
                        }
                    },
                    _ => ()
                }
                // if we played the focused hint, then its None now
                if let Some(i) = self.focused_hint && i == *idx {
                    self.focused_hint = None;
                }
                // if we played a card left of the focused hint, then we must shift it
                if let Some(i) = self.focused_hint && i > *idx {
                    self.focused_hint = Some(i-1);
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
                // if we discarded the focused hint, then its None now
                if let Some(i) = self.focused_hint && i == *idx {
                    self.focused_hint = None;
                }
                // if we discarded a card left of the focused hint, then we must shift it
                if let Some(i) = self.focused_hint && i > *idx {
                    self.focused_hint = Some(i-1);
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
                        } else {
                            self.mistakes_made += 1;
                        }
                        // Remove played card knowledge and hand and add new card if drawn
                        self.partner_hand_knowledge.remove(*idx);
                        self.partner_hand.remove(*idx);
                        if let Some(card) = card_drawn {
                            self.partner_hand.push(*card);
                            self.partner_hand_knowledge.push(DeckSubset::new_full());
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
                        self.partner_hand_knowledge.remove(*idx);
                        self.partner_hand.remove(*idx);
                        if let Some(card) = card_drawn {
                            self.partner_hand.push(*card);
                            self.partner_hand_knowledge.push(DeckSubset::new_full());
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
                        // update the cards the hint was about
                        for i in indices.iter() {
                            self.my_hand_knowledge[*i] = self.my_hand_knowledge[*i].intersect(&DeckSubset::from_color(*color));
                        }
                        // update the other cards
                        for i in (0..self.my_hand_knowledge.len()).filter(|x| !indices.contains(x)) {
                            self.my_hand_knowledge[i] = self.my_hand_knowledge[i].intersect(&DeckSubset::from_color_inverted(*color));
                        }
                        // if the hint is only about one card, then it is a focused hint
                        if indices.len() == 1 {
                            self.focused_hint = Some(indices[0]);
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
                        // update the cards the hint was about
                        for i in indices.iter() {
                            self.my_hand_knowledge[*i] = self.my_hand_knowledge[*i].intersect(&DeckSubset::from_value(*value));
                        }
                        // update the other cards
                        for i in (0..self.my_hand_knowledge.len()).filter(|x| !indices.contains(x)) {
                            self.my_hand_knowledge[i] = self.my_hand_knowledge[i].intersect(&DeckSubset::from_value_inverted(*value));
                        }
                        // if the hint is only about one card, then it is a focused hint
                        if indices.len() == 1 {
                            self.focused_hint = Some(indices[0]);
                        }
                    },
                    _ => ()
                }
            }
        }
    }
}