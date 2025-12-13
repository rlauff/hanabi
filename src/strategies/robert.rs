use crate::enums::{Move, MoveResult, Color};
use crate::card::Card;
use crate::strategy::Strategy;
use crate::decksubset::DeckSubset;
use std::fs;
use std::str::FromStr;

// robert.rs

// Params struct holding all strategy multipliers/weights
#[derive(Clone, Copy, Debug)]
pub struct Params {
    pub score_play_base: f64,
    pub score_discard_base: f64,
    pub score_hint_base: f64,

    // PLAYING
    pub score_play_exponent_probability: i32,
    pub score_play_by_playability_weight: f64,
    pub score_play_badness_mistake_weight: f64,
    pub score_play_can_play_5_sure: f64,
    pub score_play_make_playable: f64,
    pub score_play_make_playable_weighted_by_partner_knowledge: f64,
    pub score_play_make_discardable: f64,
    pub score_play_make_discardable_weighted_by_partner_knowledge: f64,
    pub score_play_sure: f64,
    pub score_play_focused_hint: f64,

    // DISCARDING
    pub score_discard_exponent_probability: i32,
    pub score_discard_value_of_a_hint: f64,
    pub score_discard_probability_weight: f64,
    pub score_discard_badness_mistake_weight: f64,
    pub score_discard_hints_low_weight: f64,

    // HINTING
    pub score_hint_focused_hint: f64,
    pub score_hint_exponent_information_gain: i32,
    pub score_hint_information_gain: f64,
    pub score_hint_make_playable: f64,
    pub score_hint_make_discardable: f64,

    // SPECIAL PENALTIES
    pub score_badness_discard_only_card_left_of_its_kind: f64,
}

impl Default for Params {
    fn default() -> Self {
        Params {
            score_play_base: 1.0,
            score_discard_base: 1.0,
            score_hint_base: 1.0,

            // PLAYING
            score_play_exponent_probability: 3,
            score_play_by_playability_weight: 20.0,
            score_play_badness_mistake_weight: 100.0,
            score_play_can_play_5_sure: 1000.0,
            score_play_make_playable: 50.0,
            score_play_make_playable_weighted_by_partner_knowledge: 40.0,
            score_play_make_discardable: 2.0,
            score_play_make_discardable_weighted_by_partner_knowledge: 2.0,
            score_play_sure: 100.0,
            score_play_focused_hint: 100.0,

            // DISCARDING
            score_discard_exponent_probability: 2,
            score_discard_value_of_a_hint: 10.0,
            score_discard_probability_weight: 60.0,
            score_discard_badness_mistake_weight: 80.0,
            score_discard_hints_low_weight: 25.0,

            // HINTING
            score_hint_focused_hint: 50.0,
            score_hint_exponent_information_gain: 1,
            score_hint_information_gain: 1.5,
            score_hint_make_playable: 100.0,
            score_hint_make_discardable: 20.0,

            // SPECIAL PENALTIES
            score_badness_discard_only_card_left_of_its_kind: 5000.0,
        }
    }
}

impl Params {
    // tries to load values from a file, falls back to default if file not found or parsing fails
    pub fn load_from_file_or_default(filename: &str) -> Self {
        let mut params = Params::default();
        
        if let Ok(content) = fs::read_to_string(filename) {
            // println!("Loading params from {}", filename);
            for line in content.lines() {
                let parts: Vec<&str> = line.split('=').map(|s| s.trim()).collect();
                if parts.len() == 2 {
                    let key = parts[0];
                    let val_str = parts[1];
                    
                    // Helper macro to update fields to avoid repetition
                    macro_rules! update_f64 {
                        ($field:ident) => {
                            if key == stringify!($field) {
                                if let Ok(v) = f64::from_str(val_str) { params.$field = v; }
                            }
                        };
                    }
                    macro_rules! update_i32 {
                        ($field:ident) => {
                            if key == stringify!($field) {
                                if let Ok(v) = i32::from_str(val_str) { params.$field = v; }
                            }
                        };
                    }

                    update_f64!(score_play_base);
                    update_f64!(score_discard_base);
                    update_f64!(score_hint_base);

                    update_i32!(score_play_exponent_probability);
                    update_f64!(score_play_by_playability_weight);
                    update_f64!(score_play_badness_mistake_weight);
                    update_f64!(score_play_can_play_5_sure);
                    update_f64!(score_play_make_playable);
                    update_f64!(score_play_make_playable_weighted_by_partner_knowledge);
                    update_f64!(score_play_make_discardable);
                    update_f64!(score_play_make_discardable_weighted_by_partner_knowledge);
                    update_f64!(score_play_sure);
                    update_f64!(score_play_focused_hint);

                    update_i32!(score_discard_exponent_probability);
                    update_f64!(score_discard_value_of_a_hint);
                    update_f64!(score_discard_probability_weight);
                    update_f64!(score_discard_badness_mistake_weight);
                    update_f64!(score_discard_hints_low_weight);

                    update_f64!(score_hint_focused_hint);
                    update_i32!(score_hint_exponent_information_gain);
                    update_f64!(score_hint_information_gain);
                    update_f64!(score_hint_make_playable);
                    update_f64!(score_hint_make_discardable);

                    update_f64!(score_badness_discard_only_card_left_of_its_kind);
                }
            }
        } else {
            // println!("Could not read params file {}, using defaults.", filename);
        }
        params
    }
}

pub struct Robert { 
    hints_remaining: u8,
    mistakes_made: u8,
    fireworks: [u8; 5],
    my_hand_knowledge: Vec<DeckSubset>,
    partner_hand: Vec<Card>,
    partner_hand_knowledge: Vec<DeckSubset>,
    cards_not_seen: DeckSubset,
    focused_hint: Option<usize>, // potentially the index to the card that was hinted directly
    params: Params, // holds the strategy parameters
}

impl Robert {
    pub fn new() -> Self {
        Robert {
            hints_remaining: 8,
            mistakes_made: 0,
            fireworks: [0; 5],
            my_hand_knowledge: vec![DeckSubset::new_full(); 5],
            partner_hand: Vec::new(),
            partner_hand_knowledge: vec![DeckSubset::new_full(); 5],
            cards_not_seen: DeckSubset::new_full(),
            focused_hint: None,
            params: Params::load_from_file_or_default("robert_params.txt")
        }
    }
    
    pub fn new_with_params(params: Params) -> Self {
        Robert {
            hints_remaining: 8,
            mistakes_made: 0,
            fireworks: [0; 5],
            my_hand_knowledge: vec![DeckSubset::new_full(); 5],
            partner_hand: Vec::new(),
            partner_hand_knowledge: vec![DeckSubset::new_full(); 5],
            cards_not_seen: DeckSubset::new_full(),
            focused_hint: None,
            params,
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

    // score play takes a card index and assigns a score to the move of playing that card
    // higher score means better move
    //  - if cannot play for sure and only one life left: score 0
    // Plus points if:
    //  - play the focused hint
    //  - probability of being playable is high:
    //      Here we take an exponent to give very high weight to cards that are very likely playable, and very low weight to cards that are less likely playable
    //  - add some extra points if we are 100% sure
    //  - making a card in partner's hand playable
    //      we give a base bonus for making a card playable, and an additional bonus weighted by probability of them knowing it is playable
    //  - making a card in partner's hand discardable
    //      again with a base bonus and more if they also know about it
    // Minus points if:
    //  - probability of not being playable is high:
    //      weighted by how bad a mistake would be (more mistakes already made means a mistake is worse)
    fn score_play(&mut self, idx: usize) -> f64 {
        let mut score = 0.0;

        // play the focused hint card:
        if let Some(i) = self.focused_hint && idx == i{
            score += self.params.score_play_focused_hint;
        }

        // give score for probability of being playable
        let probability_playable = self.probability_playable(idx);
        if probability_playable < 1.0-10e-15 && self.mistakes_made == 2 { return 0.0 } // do not lose the game
        score += probability_playable.powi(self.params.score_play_exponent_probability) * self.params.score_play_by_playability_weight;

        // extra points if we are sure
        if probability_playable > 1.0 - 10e-15 { 
            score += self.params.score_play_sure;
         } 

        // remove score for probability of not being playable, weighted seprately by how bad a mistake would be
        // if we can still make mistakes, then we can play riskier
        // +5 so that this factor does not have too much of an impact. Otherwise we might be too risky at the start
        score -= (1.0-probability_playable) * ((self.mistakes_made+5) as f64) * self.params.score_play_badness_mistake_weight;

        // removes score if the card might be the only one of its kind left
        score -= (1.0-probability_playable) * self.probability_only_card_left_of_its_kind(idx) * self.params.score_badness_discard_only_card_left_of_its_kind;

        // give a bonus if it makes a card in partner's hand playable
        // weighted by probability of that card being playable from their perspective
        // only works if we know what the card is exactly

        if let Some(card) = self.exact_card_if_known(idx) {
            let color = card.get_color();
            let color_index = color as usize;
            let value = card.get_value();
            // first check if the card is even playable
            if value != self.fireworks[color_index] + 1 {
                return score; // no bonus if card is not playable
            }
            // the value of the new card that would now be playable
            let playable_value = self.fireworks[color_index] + 1;
            if playable_value == 6 {
                // we know it is a 5 and we can play it, that a huge bonus
                // we dont need to check if this makes a card in partners hand playable, because it is a 5
                score += self.params.score_play_can_play_5_sure;
                return score;
            }
            // for each card in partner's hand, check if it would be playable now
            // apply a bonus if it is playable (disregarding wether they know it or not)
            // apply another bonus weighted by probability of them knowing it is playable, but only if it is playable
            for card_idx in 0..self.partner_hand.len() {
                let partner_card = self.partner_hand[card_idx];
                let partner_card_color = partner_card.get_color();
                let partner_card_value = partner_card.get_value();
                if partner_card_color == color && partner_card_value == playable_value {
                    // card would be playable now
                    score += self.params.score_play_make_playable; // base bonus for making a card playable
                    // temporarily add this card to he fireworks so the probability function works
                    // might change later to just pass the fireworks to probability function, but this way the data stays in place
                    self.fireworks[color_index] += 1;
                    let partner_prob_playable = self.partner_probability_playable(card_idx, None);
                    self.fireworks[color_index] -= 1;
                    // bonus weighted by probability of them knowing it is playable
                    score += partner_prob_playable * self.params.score_play_make_playable_weighted_by_partner_knowledge;
                }
                if partner_card_color == color && partner_card_value < playable_value {
                    // this card can now be discarded
                    score += self.params.score_play_make_discardable;
                    // temporarily add this card to he fireworks so the probability function works
                    // might change later to just pass the fireworks to probability function, but this way the data stays in place
                    self.fireworks[color_index] += 1;
                    let partner_prob_playable = self.partner_probability_discardable(card_idx, None);
                    self.fireworks[color_index] -= 1;
                    // bonus weighted by probability of them knowing it is discardable
                    score += partner_prob_playable * self.params.score_play_make_discardable_weighted_by_partner_knowledge;
                }
            }
        }

        score
    }

    // score discard takes a card index and assigns a score to the move of discarding that card
    // higher score means better move
    // Plus points if:
    //  - probability of being discardable is high:
    //      Here we take an exponent to give very high weight to cards that are very likely discardable, and very low weight to cards that are less likely discardable
    //  - number of hints is low, so we need new hints
    //  - the hint is focused one one card and that card is playable
    // Minus points if:
    //  - probability of not being discardable is high:
    //  - the card might be the only one left of its kind ( and is not played yet )
    fn score_discard(&self, idx: usize) -> f64 {
        let mut score: f64 = 0.0;

        // give score for probability of being discardable
        let probability_discardable = self.probability_discardable(idx);
        score += probability_discardable.powi(self.params.score_discard_exponent_probability) * self.params.score_discard_probability_weight;

        // give score if hints are low
        score += (8-self.hints_remaining) as f64 * self.params.score_discard_hints_low_weight;

        // remove score for probability of not being discardable
        score -= (1.0-probability_discardable) * self.params.score_discard_badness_mistake_weight;

        // removes score if the card might be the only one of its kind left
        score -= (1.0-probability_discardable) * self.probability_only_card_left_of_its_kind(idx) * self.params.score_badness_discard_only_card_left_of_its_kind;

        if score<0. { 0. } else { score }
    }

    // score hint takes a hint move and assigns a score to it
    // higher score means better move
    // Plus points if:
    //  - number of hints is high, so we can afford to give hints ( implemented as a weight multiplied at the end )
    //  - information gain is high ( cards excluded from the DeckSubsets, weighted by an exponent because going from 1 to 2 possible cards is more valueable than from 20 to 5 )
    //  - giving a focused hint to a playable card
    //  - cards become playable in partner's hand
    //  - cards become discardable in partner's hand
    // TODO: Maybe it would be better to look at the difference between probabilities before and after hint instead of the number of cardss excluded
    fn score_hint(&self, hint: &Move) -> f64 {

        let cards_affected_indices: Vec<usize> = match hint {
            Move::HintColor(color) => (0..self.partner_hand.len())
                .filter(|x| self.partner_hand[*x].get_color() == *color)
                .collect(),
            Move::HintValue(value) => (0..self.partner_hand.len())
                .filter(|x| self.partner_hand[*x].get_value() == *value)
                .collect(),
            _ => unreachable!(),
        };

        if cards_affected_indices.is_empty() {
            return -1000.0; 
        }

        let mut score = 0.0;
        let information_gained_array = match hint {
            Move::HintColor(color) => { self.number_of_cards_excluded_by_color_hint(*color) },
            Move::HintValue(value) => { self.number_of_cards_excluded_by_value_hint(*value) },
            _ => unreachable!()
        };

        for i in 0..self.partner_hand_knowledge.len() {
            score += (1.0 + (information_gained_array[i] as f64 / self.partner_hand_knowledge[i].0.count_ones() as f64)  
                                * self.params.score_hint_information_gain).powi(self.params.score_hint_exponent_information_gain) - 1.0;
        }

        // Focused Hint Logic
        if cards_affected_indices.len() == 1 {
            let idx = cards_affected_indices[0];
            let card_affected = self.partner_hand[idx];
            let card_affected_color = card_affected.get_color();
            let card_affected_value = card_affected.get_value();
            
            if card_affected_value == self.fireworks[card_affected_color as usize] + 1 {
                // Only add score if partner knows about it
                if self.partner_probability_playable(idx, None) < 0.99 {
                    score += self.params.score_hint_focused_hint;
                }
            } else if card_affected_value > self.fireworks[card_affected_color as usize] + 1 {
                 // Bad hint
                score -= self.params.score_hint_focused_hint;
            }
        }

        // Look if cards become playable or discardable
        for i in 0..self.partner_hand_knowledge.len() {
            // Check if becoming playable
            // Wichtig: Wir prüfen, ob die Karte VORHER noch nicht sicher spielbar war
            if self.partner_probability_playable(i, Some(*hint)) > 0.99 && self.partner_probability_playable(i, None) < 0.99 {
                score += self.params.score_hint_make_playable;
            }
            
            // Check if becoming discardable
            if self.partner_probability_discardable(i, Some(*hint)) > 0.99 && self.partner_probability_discardable(i, None) < 0.99 {
                score += self.params.score_hint_make_discardable;
            }
        }
        
        score
    }

    // entry point for the score functions
    fn score_move(&mut self, mv: &Move) -> f64 {
        let score = match mv {
            Move::Play(idx) => self.score_play(*idx) * self.params.score_play_base,
            Move::Discard(idx) => self.score_discard(*idx) * self.params.score_discard_base,
            Move::HintColor(_) | Move::HintValue(_) => self.score_hint(mv) * self.params.score_hint_base,
        };
        // println!("{:?}: {}", mv, score);
        score
    }
}

impl Strategy for Robert {
    fn initialize(&mut self, other_player_hand: &Vec<Card>) {
        self.partner_hand = other_player_hand.clone();
        for card in other_player_hand {
            self.cards_not_seen.remove_card(card);
        }
    }

    fn decide_move(&mut self) -> Move {
        let all_moves = self.all_possible_moves();

        // we find the max score move by interpreting the f64 as a bit vector.
        // If the sign bit is 0, the number is positive and we flip that bit
        // Otherwise, we flip all bits to reverse the 2's complement

        *all_moves
            .iter()
            .max_by_key(|&m| { let b = self.score_move(m).to_bits() as i64; b ^ (b >> 63 & i64::MAX) })
            .expect("There must be at least one move")
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