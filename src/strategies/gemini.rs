use crate::enums::{Move, MoveResult, Color};
use crate::card::Card;
use crate::strategy::Strategy;
use crate::decksubset::DeckSubset;

/// The Gemini Strategy (v5 - "Tempo & Flow")
/// 
/// Improvements:
/// - **Distance Metric:** Calculates how "far" a card is from being playable.
/// - **Anti-Clog Logic:** Refuses to give "Setup Hints" for cards that are too far in the future (e.g., 4s when stack is 0).
/// - **Panic Discard:** If forced to discard a hinted card, chooses the one furthest from playability.
/// - **Perspective:** Maintains split knowledge (My View vs Public View) from v4.
pub struct Gemini { 
    hints_remaining: u8,
    fireworks: [u8; 5],
    my_hand_knowledge: Vec<DeckSubset>,
    partner_hand: Vec<Card>,
    partner_hand_knowledge: Vec<DeckSubset>,
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

    fn mark_board_change(&mut self, card: &Card) {
        self.my_view_unknowns.remove_card(*card);
        self.public_unknowns.remove_card(*card);
    }

    fn mark_partner_hand(&mut self, card: &Card) {
        self.my_view_unknowns.remove_card(*card);
    }

    fn is_playable(&self, card: &Card) -> bool {
        let color_idx = card.get_color() as usize;
        let val = card.get_value();
        self.fireworks[color_idx] + 1 == val
    }

    fn is_dead(&self, card: &Card) -> bool {
        let color_idx = card.get_color() as usize;
        let val = card.get_value();
        self.fireworks[color_idx] >= val
    }

    /// Calculates "Distance". 
    /// 0 = Playable. 
    /// 1 = Needs 1 card. 
    /// 255 = Dead/Unknown.
    fn get_distance(&self, card: &Card) -> u8 {
        let color_idx = card.get_color() as usize;
        let val = card.get_value();
        let current_stack = self.fireworks[color_idx];

        if current_stack >= val {
            return 255; // Dead
        }
        val - (current_stack + 1)
    }

    fn count_in_discard(&self, card: &Card) -> usize {
        self.discarded_cards.iter()
            .filter(|&c| c.get_color() == card.get_color() && c.get_value() == card.get_value())
            .count()
    }

    fn is_critical(&self, card: &Card) -> bool {
        if self.is_dead(card) { return false; }
        let val = card.get_value();
        if val == 5 { return true; } 
        let copies_in_discard = self.count_in_discard(card);
        let max_copies = match val { 1 => 3, 2 | 3 | 4 => 2, _ => 1 };
        copies_in_discard + 1 >= max_copies
    }

    fn is_slot_certainly_playable(&self, index: usize) -> bool {
        if index >= self.my_hand_knowledge.len() { return false; }
        let possibilities = self.my_hand_knowledge[index].intersect(&self.my_view_unknowns);
        
        let mut any_possible = false;
        for i in 0..50 {
            let c = Card::new(i);
            if possibilities.has_card(c) {
                any_possible = true;
                if !self.is_playable(&c) { return false; }
            }
        }
        any_possible
    }

    fn is_slot_certainly_dead(&self, index: usize) -> bool {
        if index >= self.my_hand_knowledge.len() { return false; }
        let possibilities = self.my_hand_knowledge[index].intersect(&self.my_view_unknowns);
        let mut any_possible = false;
        for i in 0..50 {
            let c = Card::new(i);
            if possibilities.has_card(c) {
                any_possible = true;
                if !self.is_dead(&c) { return false; }
            }
        }
        any_possible
    }

    fn is_slot_hinted(&self, index: usize) -> bool {
        if index >= self.my_hand_knowledge.len() { return false; }
        self.my_hand_knowledge[index].0 != DeckSubset::new_full().0
    }

    fn knowledge_implies_playable(&self, knowledge: &DeckSubset) -> bool {
        let possibilities = knowledge.intersect(&self.public_unknowns);
        let mut count = 0;
        for i in 0..50 {
            let c = Card::new(i);
            if possibilities.has_card(c) {
                count += 1;
                if !self.is_playable(&c) { return false; }
            }
        }
        count > 0
    }

    /// Calculates the expected "Distance" of a card in my hand.
    /// Used for "Panic Discarding" - discard the card that is likely furthest away.
    fn calculate_expected_distance(&self, index: usize) -> f32 {
        if index >= self.my_hand_knowledge.len() { return 999.0; }
        let possibilities = self.my_hand_knowledge[index].intersect(&self.my_view_unknowns);
        
        let mut total = 0;
        let mut dist_sum = 0;

        for i in 0..50 {
            let c = Card::new(i);
            if possibilities.has_card(c) {
                total += 1;
                let d = self.get_distance(&c);
                if d == 255 { dist_sum += 10; } // Penalize dead cards heavily
                else { dist_sum += d as usize; }
            }
        }
        if total == 0 { return 999.0; }
        (dist_sum as f32) / (total as f32)
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
        // 1. PLAY (Certainty)
        for i in (0..self.my_hand_knowledge.len()).rev() {
            if self.is_slot_certainly_playable(i) { return Move::Play(i); }
        }

        // 2. SAVE CLUE (Critical ONLY)
        if self.hints_remaining > 0 && !self.partner_hand.is_empty() {
            let mut likely_discard_idx = 0;
            let mut found_chop = false;
            // Find oldest unhinted
            for i in 0..self.partner_hand.len() {
                if self.partner_hand_knowledge[i].0 == DeckSubset::new_full().0 {
                    likely_discard_idx = i;
                    found_chop = true;
                    break; 
                }
            }
            // If all hinted, partner might discard index 0
            if !found_chop { likely_discard_idx = 0; }

            let chop_card = self.partner_hand[likely_discard_idx];
            if self.is_critical(&chop_card) {
                // If it's already hinted, we assume safe (unless it was a color hint that didn't reveal value)
                // But generally, if it's the chop, we should warn.
                if found_chop { return Move::HintValue(chop_card.get_value()); }
            }
        }

        // 3. PLAY CLUE (Immediate Progress)
        if self.hints_remaining > 0 {
            for target_val in 1..=5 {
                for (i, card) in self.partner_hand.iter().enumerate() {
                    if card.get_value() != target_val { continue; }
                    if !self.is_playable(card) { continue; }

                    if self.knowledge_implies_playable(&self.partner_hand_knowledge[i]) { continue; }

                    // Color Hint
                    let color = card.get_color();
                    let k_col = self.partner_hand_knowledge[i].intersect(&DeckSubset::from_color(color));
                    if k_col.0 != self.partner_hand_knowledge[i].0 && self.knowledge_implies_playable(&k_col) {
                        return Move::HintColor(color);
                    }

                    // Value Hint
                    let val = card.get_value();
                    let k_val = self.partner_hand_knowledge[i].intersect(&DeckSubset::from_value(val));
                    if k_val.0 != self.partner_hand_knowledge[i].0 && self.knowledge_implies_playable(&k_val) {
                        return Move::HintValue(val);
                    }
                }
            }
            
            // 4. SETUP CLUE (Tempo Adjusted)
            // Only hint if card is NEAR FUTURE (Distance <= 1) OR Critical.
            // Do NOT hint far future cards (Distance > 1), they clog the hand.
            if self.hints_remaining > 4 { // Increased threshold to conserve clues
                 for (i, card) in self.partner_hand.iter().enumerate() {
                     if self.partner_hand_knowledge[i].0 == DeckSubset::new_full().0 { // Only if unhinted
                         let dist = self.get_distance(card);
                         
                         // Hint if Critical (always) OR Distance is low (0 or 1)
                         if self.is_critical(card) || (dist <= 1) {
                             return Move::HintValue(card.get_value());
                         }
                     }
                 }
            }
        }

        // 5. DISCARD
        if self.hints_remaining < 8 {
            // A. Certain Trash
            for i in 0..self.my_hand_knowledge.len() {
                if self.is_slot_certainly_dead(i) { return Move::Discard(i); }
            }

            // B. Unhinted (The Chop)
            for i in 0..self.my_hand_knowledge.len() {
                if !self.is_slot_hinted(i) { return Move::Discard(i); }
            }
            
            // C. PANIC DISCARD (Hand Clogging Fix)
            // All cards are hinted. Discard the one furthest from being playable.
            // (Previous version randomized this or picked least likely to be trash, which kept 4s)
            let mut best_discard_idx = 0;
            let mut highest_distance = -1.0;

            for i in 0..self.my_hand_knowledge.len() {
                let dist = self.calculate_expected_distance(i);
                // We want to discard HIGH distance cards (far future)
                if dist > highest_distance {
                    highest_distance = dist;
                    best_discard_idx = i;
                }
            }
            return Move::Discard(best_discard_idx);
        }

        // 6. FORCE HINT
        if !self.partner_hand.is_empty() {
             // Find *any* new info
             for (i, card) in self.partner_hand.iter().enumerate() {
                 let val = card.get_value();
                 let k_val = self.partner_hand_knowledge[i].intersect(&DeckSubset::from_value(val));
                 if k_val.0 != self.partner_hand_knowledge[i].0 { return Move::HintValue(val); }
                 
                 let col = card.get_color();
                 let k_col = self.partner_hand_knowledge[i].intersect(&DeckSubset::from_color(col));
                 if k_col.0 != self.partner_hand_knowledge[i].0 { return Move::HintColor(col); }
             }
             return Move::HintValue(self.partner_hand[self.partner_hand.len()-1].get_value());
        }

        Move::Discard(0) 
    }

    fn update_after_own_move(&mut self, mv: &Move, mv_result: &MoveResult, got_new_card: bool) {
        match mv {
            Move::Play(idx) | Move::Discard(idx) => {
                if *idx < self.my_hand_knowledge.len() { self.my_hand_knowledge.remove(*idx); }
                if got_new_card { self.my_hand_knowledge.push(DeckSubset::new_full()); }
                match mv_result {
                    MoveResult::Play(success, card) => {
                        self.mark_board_change(card);
                        if *success { self.fireworks[card.get_color() as usize] += 1; } 
                        else { self.discarded_cards.push(*card); }
                    },
                    MoveResult::Discard(card) => {
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
        match mv {
            Move::Play(idx) | Move::Discard(idx) => {
                if *idx < self.partner_hand.len() {
                    let card = self.partner_hand.remove(*idx);
                    self.partner_hand_knowledge.remove(*idx);
                    self.mark_board_change(&card);
                    match mv_result {
                        MoveResult::Play(success, _) => {
                            if *success { self.fireworks[card.get_color() as usize] += 1; } 
                            else { self.discarded_cards.push(card); }
                        },
                        MoveResult::Discard(_) => {
                            self.discarded_cards.push(card);
                            if self.hints_remaining < 8 { self.hints_remaining += 1; }
                        },
                        _ => {}
                    }
                }
            },
            Move::HintColor(c) => {
                self.hints_remaining -= 1;
                let mut hinted_indices = Vec::new();
                if let MoveResult::Hint(indices, _) = mv_result { hinted_indices = indices.clone(); }
                for (i, subset) in self.my_hand_knowledge.iter_mut().enumerate() {
                    if hinted_indices.contains(&i) {
                        *subset = subset.intersect(&DeckSubset::from_color(*c));
                    } else {
                        *subset = subset.intersect(&DeckSubset::from_color_inverted(*c));
                    }
                }
            },
            Move::HintValue(v) => {
                self.hints_remaining -= 1;
                let mut hinted_indices = Vec::new();
                if let MoveResult::Hint(indices, _) = mv_result { hinted_indices = indices.clone(); }
                for (i, subset) in self.my_hand_knowledge.iter_mut().enumerate() {
                    if hinted_indices.contains(&i) {
                        *subset = subset.intersect(&DeckSubset::from_value(*v));
                    } else {
                        *subset = subset.intersect(&DeckSubset::from_value_inverted(*v));
                    }
                }
            }
        }
    }

    fn see(&mut self, card: &Card) {
        self.mark_partner_hand(card);
        self.partner_hand.push(*card);
        self.partner_hand_knowledge.push(DeckSubset::new_full());
    }
}