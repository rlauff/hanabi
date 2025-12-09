use crate::enums::{Move, MoveResult, Color};
use crate::card::Card;
use crate::strategy::Strategy;
use crate::decksubset::DeckSubset;

/// The Gemini Strategy (v10 - "The Realist" - Refactored for MoveResult Draw)
pub struct Gemini { 
    hints_remaining: u8,
    fireworks: [u8; 5],
    
    // Knowledge management
    my_hand_knowledge: Vec<DeckSubset>,
    partner_hand: Vec<Card>,
    partner_hand_knowledge: Vec<DeckSubset>,
    
    // Board State tracking
    my_view_unknowns: DeckSubset, 
    public_unknowns: DeckSubset,
    discarded_cards: Vec<Card>,
}

impl Gemini {
    pub fn new() -> Self {
        Gemini {
            hints_remaining: 8,
            fireworks: [0; 5],
            my_hand_knowledge: Vec::new(),
            partner_hand: Vec::new(),
            partner_hand_knowledge: Vec::new(),
            my_view_unknowns: DeckSubset::new_full(),
            public_unknowns: DeckSubset::new_full(),
            discarded_cards: Vec::new(),
        }
    }

    // --- Helpers (Same as before) ---

    fn mark_board_change(&mut self, card: &Card) {
        self.my_view_unknowns.remove_card(card);
        self.public_unknowns.remove_card(card);
    }

    fn mark_partner_hand(&mut self, card: &Card) {
        self.my_view_unknowns.remove_card(card);
    }

    fn is_playable(&self, card: &Card) -> bool {
        let color_idx = card.get_color() as usize;
        let val = card.get_value();
        self.fireworks[color_idx] + 1 == val
    }

    fn count_in_discard(&self, color: Color, value: u8) -> usize {
        self.discarded_cards.iter()
            .filter(|&c| c.get_color() == color && c.get_value() == value)
            .count()
    }

    fn is_useless(&self, card: &Card) -> bool {
        let color_idx = card.get_color() as usize;
        let val = card.get_value();
        let current_stack = self.fireworks[color_idx];
        if current_stack >= val { return true; }

        for req_val in (current_stack + 1)..val {
            let copies_discarded = self.count_in_discard(card.get_color(), req_val);
            let max_copies = match req_val { 1 => 3, 2 | 3 | 4 => 2, 5 => 1, _ => 1 };
            if copies_discarded >= max_copies { return true; }
        }
        false
    }

    fn get_distance(&self, card: &Card) -> u8 {
        if self.is_useless(card) { return 255; }
        let color_idx = card.get_color() as usize;
        let val = card.get_value();
        let current_stack = self.fireworks[color_idx];
        val - (current_stack + 1)
    }

    fn is_card_critical(&self, card: &Card) -> bool {
        if self.is_useless(card) { return false; }
        let val = card.get_value();
        if val == 5 { return true; } 
        let copies_in_discard = self.count_in_discard(card.get_color(), val);
        let max_copies = match val { 1 => 3, 2 | 3 | 4 => 2, _ => 1 };
        copies_in_discard + 1 >= max_copies
    }

    // --- Knowledge Logic (Same as before) ---

    fn is_slot_certainly_playable(&self, index: usize) -> bool {
        if index >= self.my_hand_knowledge.len() { return false; }
        let possibilities = self.my_hand_knowledge[index].intersect(&self.my_view_unknowns);
        if possibilities.0 == 0 { return false; }
        for i in 0..50 {
            let c = &Card::new(i);
            if possibilities.has_card(c) {
                if !self.is_playable(c) { return false; }
            }
        }
        true
    }

    fn is_slot_certainly_useless(&self, index: usize) -> bool {
        if index >= self.my_hand_knowledge.len() { return false; }
        let possibilities = self.my_hand_knowledge[index].intersect(&self.my_view_unknowns);
        if possibilities.0 == 0 { return false; }
        for i in 0..50 {
            let c = &Card::new(i);
            if possibilities.has_card(c) {
                if !self.is_useless(c) { return false; }
            }
        }
        true
    }

    fn is_slot_hinted(&self, index: usize) -> bool {
        if index >= self.my_hand_knowledge.len() { return false; }
        self.my_hand_knowledge[index].0 != DeckSubset::new_full().0
    }

    fn knowledge_implies_playable(&self, knowledge: &DeckSubset) -> bool {
        let possibilities = knowledge.intersect(&self.public_unknowns);
        if possibilities.0 == 0 { return false; }
        let mut possible_count = 0;
        for i in 0..50 {
            let c = &Card::new(i);
            if possibilities.has_card(c) {
                possible_count += 1;
                if !self.is_playable(c) { return false; }
            }
        }
        possible_count > 0
    }

    fn calculate_discard_score(&self, index: usize) -> i32 {
        if index >= self.my_hand_knowledge.len() { return -9999; }
        if self.is_slot_certainly_useless(index) { return 1000; }

        let possibilities = self.my_hand_knowledge[index].intersect(&self.my_view_unknowns);
        let mut total_count = 0;
        let mut critical_count = 0;
        let mut dist_accum = 0;

        for i in 0..50 {
            let c = &Card::new(i);
            if possibilities.has_card(c) {
                total_count += 1;
                if self.is_card_critical(&c) { critical_count += 1; }
                let d = self.get_distance(&c);
                if d == 255 { dist_accum += 20; } else { dist_accum += d as usize; }
            }
        }

        if total_count == 0 { return 0; }
        if self.is_slot_hinted(index) { return -1000; }

        let mut score = 100;
        let critical_prob = critical_count as f32 / total_count as f32;
        score -= (critical_prob * 5000.0) as i32;
        score += dist_accum as i32 / total_count as i32;
        score
    }
}

impl Strategy for Gemini {
    fn initialize(&mut self, other_player_hand: &Vec<Card>) {
        self.hints_remaining = 8;
        self.fireworks = [0; 5];
        self.my_view_unknowns = DeckSubset::new_full();
        self.public_unknowns = DeckSubset::new_full();
        self.discarded_cards.clear();
        self.my_hand_knowledge = vec![DeckSubset::new_full(); 5];
        self.partner_hand = other_player_hand.clone();
        self.partner_hand_knowledge = vec![DeckSubset::new_full(); 5];

        for card in other_player_hand {
            self.mark_partner_hand(card);
        }
    }

    fn decide_move(&mut self) -> Move {
        // 1. PLAY
        for i in (0..self.my_hand_knowledge.len()).rev() {
            if self.is_slot_certainly_playable(i) { return Move::Play(i); }
        }
        // 2. CHOP
        let mut partner_discard_idx = 0;
        let mut found_chop = false;
        for i in 0..self.partner_hand.len() {
            if self.partner_hand_knowledge[i].0 == DeckSubset::new_full().0 {
                partner_discard_idx = i;
                found_chop = true;
                break; 
            }
        }
        if !found_chop { partner_discard_idx = 0; } 

        // 3. SAVE
        if self.hints_remaining > 0 && !self.partner_hand.is_empty() {
            let card_at_risk = self.partner_hand[partner_discard_idx];
            if self.is_card_critical(&card_at_risk) {
                return Move::HintValue(card_at_risk.get_value());
            }
        }

        // 4. PLAY CLUE
        if self.hints_remaining > 0 {
            for target_val in 1..=5 {
                for (i, card) in self.partner_hand.iter().enumerate() {
                    if card.get_value() != target_val { continue; }
                    if self.is_useless(card) { continue; } 
                    if !self.is_playable(card) { continue; }
                    if self.knowledge_implies_playable(&self.partner_hand_knowledge[i]) { continue; }

                    let color = card.get_color();
                    let k_col = self.partner_hand_knowledge[i].intersect(&DeckSubset::from_color(color));
                    if k_col.0 != self.partner_hand_knowledge[i].0 && self.knowledge_implies_playable(&k_col) {
                        return Move::HintColor(color);
                    }
                    let val = card.get_value();
                    let k_val = self.partner_hand_knowledge[i].intersect(&DeckSubset::from_value(val));
                    if k_val.0 != self.partner_hand_knowledge[i].0 && self.knowledge_implies_playable(&k_val) {
                        return Move::HintValue(val);
                    }
                }
            }
            // 5. SETUP CLUE
            if self.hints_remaining > 1 {
                 for (i, card) in self.partner_hand.iter().enumerate() {
                     if self.partner_hand_knowledge[i].0 == DeckSubset::new_full().0 { 
                         if self.is_useless(card) { continue; }
                         let dist = self.get_distance(card);
                         if dist <= 1 || card.get_value() == 5 {
                             return Move::HintValue(card.get_value());
                         }
                     }
                 }
            }
        }
        // 6. DISCARD
        if self.hints_remaining < 8 {
            let mut best_discard_idx = 0;
            let mut max_score = i32::MIN;
            for i in 0..self.my_hand_knowledge.len() {
                let score = self.calculate_discard_score(i);
                if score > max_score {
                    max_score = score;
                    best_discard_idx = i;
                }
            }
            return Move::Discard(best_discard_idx);
        }
        // 7. FORCE HINT
        if !self.partner_hand.is_empty() {
             let last_idx = self.partner_hand.len() - 1;
             return Move::HintValue(self.partner_hand[last_idx].get_value());
        }
        Move::Discard(0) 
    }

    fn update_after_own_move(&mut self, mv: &Move, mv_result: &MoveResult, got_new_card: bool) {
        match mv {
            Move::Play(idx) | Move::Discard(idx) => {
                if *idx < self.my_hand_knowledge.len() { self.my_hand_knowledge.remove(*idx); }
                if got_new_card { self.my_hand_knowledge.push(DeckSubset::new_full()); }
                match mv_result {
                    MoveResult::Play(success, card, _) => { // Note: The Option<Card> is None for own move
                        self.mark_board_change(card);
                        if *success { self.fireworks[card.get_color() as usize] += 1; } 
                        else { self.discarded_cards.push(*card); }
                    },
                    MoveResult::Discard(card, _) => {
                        self.mark_board_change(card);
                        self.discarded_cards.push(*card);
                        if self.hints_remaining < 8 { self.hints_remaining += 1; }
                    },
                    _ => {}
                }
            },
            Move::HintColor(c) => {
                self.hints_remaining -= 1;
                let mut hinted_indices = Vec::new();
                for (i, card) in self.partner_hand.iter().enumerate() { if card.get_color() == *c { hinted_indices.push(i); } }
                for i in 0..self.partner_hand_knowledge.len() {
                    if hinted_indices.contains(&i) {
                        self.partner_hand_knowledge[i] = self.partner_hand_knowledge[i].intersect(&DeckSubset::from_color(*c));
                    } else {
                        self.partner_hand_knowledge[i] = self.partner_hand_knowledge[i].intersect(&DeckSubset::from_color_inverted(*c));
                    }
                }
            },
            Move::HintValue(v) => {
                self.hints_remaining -= 1;
                let mut hinted_indices = Vec::new();
                for (i, card) in self.partner_hand.iter().enumerate() { if card.get_value() == *v { hinted_indices.push(i); } }
                for i in 0..self.partner_hand_knowledge.len() {
                    if hinted_indices.contains(&i) {
                        self.partner_hand_knowledge[i] = self.partner_hand_knowledge[i].intersect(&DeckSubset::from_value(*v));
                    } else {
                        self.partner_hand_knowledge[i] = self.partner_hand_knowledge[i].intersect(&DeckSubset::from_value_inverted(*v));
                    }
                }
            }
        }
    }

    fn update_after_other_player_move(&mut self, mv: &Move, mv_result: &MoveResult) {
        // 1. Handle Play/Discard (Card leaving hand)
        let drawn_card_opt = match mv {
            Move::Play(idx) | Move::Discard(idx) => {
                if *idx < self.partner_hand.len() {
                    // Update internal tracker
                    let card = self.partner_hand.remove(*idx);
                    self.partner_hand_knowledge.remove(*idx);
                    self.mark_board_change(&card);

                    // Update Board State
                    match mv_result {
                        MoveResult::Play(success, _, drawn) => {
                            if *success { self.fireworks[card.get_color() as usize] += 1; } 
                            else { self.discarded_cards.push(card); }
                            drawn
                        },
                        MoveResult::Discard(_, drawn) => {
                            self.discarded_cards.push(card);
                            if self.hints_remaining < 8 { self.hints_remaining += 1; }
                            drawn
                        },
                        _ => &None // Should not happen for Play/Discard
                    }
                } else {
                    &None
                }
            },
            Move::HintColor(c) => {
                self.hints_remaining -= 1;
                let mut hinted_indices = Vec::new();
                if let MoveResult::Hint(indices) = mv_result { hinted_indices = indices.clone(); }
                for (i, subset) in self.my_hand_knowledge.iter_mut().enumerate() {
                    if hinted_indices.contains(&i) {
                        *subset = subset.intersect(&DeckSubset::from_color(*c));
                    } else {
                        *subset = subset.intersect(&DeckSubset::from_color_inverted(*c));
                    }
                }
                &None
            },
            Move::HintValue(v) => {
                self.hints_remaining -= 1;
                let mut hinted_indices = Vec::new();
                if let MoveResult::Hint(indices) = mv_result { hinted_indices = indices.clone(); }
                for (i, subset) in self.my_hand_knowledge.iter_mut().enumerate() {
                    if hinted_indices.contains(&i) {
                        *subset = subset.intersect(&DeckSubset::from_value(*v));
                    } else {
                        *subset = subset.intersect(&DeckSubset::from_value_inverted(*v));
                    }
                }
                &None
            }
        };

        // 2. Handle Draw (New card entering hand) - Replaces `see()`
        if let Some(new_card) = drawn_card_opt {
            self.mark_partner_hand(new_card);
            self.partner_hand.push(*new_card);
            self.partner_hand_knowledge.push(DeckSubset::new_full());
        }
    }
}