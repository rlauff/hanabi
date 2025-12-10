use crate::enums::{Move, MoveResult, Color};
use crate::card::{self, Card};
use crate::strategy::Strategy;
use crate::decksubset::DeckSubset;

const SCORE_PLAY_EXPONENT_SCORE_PLAYABLE: i32 = 5;
const SCORE_PLAY_BY_PLAYABILITY_WEIGHT: f64 = 10.0;
const SCORE_PLAY_BADNESS_MISTAKE_WEIGHT: f64 = 30.0;
const SCORE_PLAY_CAN_PLAY_5_SURE: f64 = 50.0;
const SCORE_PLAY_MAKE_PLAYABLE: f64 = 5.0;
const SCORE_PLAY_MAKE_PLAYABLE_WEIGHTED_BY_PARTNER_KNOWLEDGE: f64 = 5.0;
const SCORE_PLAY_BADNESS_DISCARD_ONLY_CARD_LEFT_OF_ITS_KIND: f64 = 200.0;

pub struct Robert { 
    hints_remaining: u8,
    mistakes_made: u8,
    fireworks: [u8; 5],
    my_hand_knowledge: Vec<DeckSubset>,
    partner_hand: Vec<Card>,
    partner_hand_knowledge: Vec<DeckSubset>,
    cards_not_seen: DeckSubset
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
            cards_not_seen: DeckSubset::new_full()
        }
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
        card_subset.is_subset(knowledge).then(|| card)

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
                    .intersect(&DeckSubset::from_value(value + 1));
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
    fn partner_probability_playable(&self, idx: usize) -> f64 {
        // divide number of playable cards in knowledge by total number of cards in knowledge
        // intersect with cards not seen to only count cards that could still be in hand
        self.cards_not_seen.intersect(&self.partner_hand_knowledge[idx].intersect(&self.playable_cards())).0.count_ones() as f64 /
        self.cards_not_seen.intersect(&self.partner_hand_knowledge[idx]).0.count_ones() as f64
    }
    fn partner_probability_discardable(&self, idx: usize) -> f64 {
        // divide number of discardable cards in knowledge by total number of cards in knowledge
        // intersect with cards not seen to only count cards that could still be in hand
        self.cards_not_seen.intersect(&self.partner_hand_knowledge[idx].intersect(&&self.discardable_cards())).0.count_ones() as f64 /
        self.cards_not_seen.intersect(&self.partner_hand_knowledge[idx]).0.count_ones() as f64
    }

    // the probability of being the only card left of its kind
    fn probability_only_card_left_of_its_kind(&self, idx: usize) -> f64{
        unimplemented!()
    }

    // score play takes a card index and assigns a score to the move of playing that card
    // higher score means better move
    // Plus points if:
    //  - probability of being playable is high:
    //      Here we take an exponent to give very high weight to cards that are very likely playable, and very low weight to cards that are less likely playable
    //  - making a card in partner's hand playable
    //      we give a base bonus for making a card playable, and an additional bonus weighted by probability of them knowing it is playable
    // Minus points if:
    //  - probability of not being playable is high:
    //      weighted by how bad a mistake would be (more mistakes already made means a mistake is worse)
    fn score_play(&self, idx: usize) -> f64 {
        let mut score = 0.0;

        // give score for probability of being playable
        let probability_playable = self.probability_playable(idx);
        score += probability_playable.powi(SCORE_PLAY_EXPONENT_SCORE_PLAYABLE) * SCORE_PLAY_BY_PLAYABILITY_WEIGHT;

        // remove score for probability of not being playable, weighted seprately by how bad a mistake would be
        // if we can still make mistakes, then we can play riskier
        // +5 so that this factor does not have too much of an impact. Otherwise we might be too risky at the start
        score -= (1.0-probability_playable) * ((self.mistakes_made+5) as f64) * SCORE_PLAY_BADNESS_MISTAKE_WEIGHT;

        // removes score if the card might be the only one of its kind left
        score -= (1.0-probability_playable) * self.probability_only_card_left_of_its_kind(idx) * SCORE_PLAY_BADNESS_DISCARD_ONLY_CARD_LEFT_OF_ITS_KIND;

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
                // we know it is a 5 and we can play it, that a huge bonus:
                score += SCORE_PLAY_CAN_PLAY_5_SURE;
                return score;
            }
            // for each card in partner's hand, check if it would be playable now
            // apply a bonus if it is playable (disregarding wether they know it or not)
            // apply another bonus weighted by probability of them knowing it is playable, but only if it is playable
            for card_idx in 0..self.partner_hand.len() {
                let partner_card = self.partner_hand[card_idx];
                if partner_card.get_color() == color && partner_card.get_value() == playable_value {
                    // card would be playable now
                    score += SCORE_PLAY_MAKE_PLAYABLE; // base bonus for making a card playable
                    let partner_prob_playable = self.partner_probability_playable(card_idx);
                    // bonus weighted by probability of them knowing it is playable
                    score += partner_prob_playable * SCORE_PLAY_MAKE_PLAYABLE_WEIGHTED_BY_PARTNER_KNOWLEDGE;
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
    // Minus points if:
    //  - probability of not being discardable is high:
    //      weighted by how bad a mistake would be (i.e. the probability of being the only card left of its kind is high)
    //  - if the card has a high chance of being playable right now
    fn score_discard(&self, idx: usize) -> f64 {
        unimplemented!()
    }

    // score hint takes a hint move and assigns a score to it
    // higher score means better move
    // Plus points if:
    //  - number of hints is high, so we can afford to give hints
    //  - information gain is high
    //  - cards become playable in partner's hand
    //  - cards become discardable in partner's hand
    // Minus points if:
    //  - number of hints is low, so we need to save hints (this might get removed as it is already implicit)
    fn score_hint(&self, hint: &Move) -> f64 {
        unimplemented!()
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
        Move::Discard(1)
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
                        } else {
                            self.mistakes_made += 1;
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