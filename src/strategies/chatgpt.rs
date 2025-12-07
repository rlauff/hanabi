use crate::enums::{Move, MoveResult, Color};
use crate::card::Card;
use crate::strategy::Strategy;
use crate::decksubset::DeckSubset;

/// ChatGPT strategy inspired by Gemini but slightly simpler.
///
/// Key ideas:
/// - Track per-slot knowledge using `DeckSubset` for own and partner hands.
/// - Track `fireworks` and discarded cards to compute playability/criticality.
/// - Prioritize: play certain cards; give play-enabling hints; save critical partner cards; setup near-future; discard safely.
pub struct ChatGPT {
    hints_remaining: u8,
    fireworks: [u8; 5],
    my_hand_knowledge: Vec<DeckSubset>,
    partner_hand: Vec<Card>,
    partner_hand_knowledge: Vec<DeckSubset>,
    public_unknowns: DeckSubset,
    discarded_cards: Vec<Card>,
    last_hint_value: Option<u8>,
    last_hint_color: Option<Color>,
}

impl ChatGPT {
    pub fn new() -> Self {
        ChatGPT {
            hints_remaining: 8,
            fireworks: [0; 5],
            my_hand_knowledge: Vec::new(),
            partner_hand: Vec::new(),
            partner_hand_knowledge: Vec::new(),
            public_unknowns: DeckSubset::new_full(),
            discarded_cards: Vec::new(),
            last_hint_value: None,
            last_hint_color: None,
        }
    }

    fn is_playable(&self, card: &Card) -> bool {
        let idx = card.get_color() as usize;
        self.fireworks[idx] + 1 == card.get_value()
    }

    fn is_dead(&self, card: &Card) -> bool {
        let idx = card.get_color() as usize;
        self.fireworks[idx] >= card.get_value()
    }

    fn count_in_discard(&self, card: &Card) -> usize {
        self.discarded_cards.iter().filter(|&c| c.get_color() == card.get_color() && c.get_value() == card.get_value()).count()
    }

    fn is_critical(&self, card: &Card) -> bool {
        if self.is_dead(card) { return false; }
        let v = card.get_value();
        if v == 5 { return true; }
        let copies = self.count_in_discard(card);
        let max = match v { 1 => 3, 2 | 3 | 4 => 2, _ => 1 };
        copies + 1 >= max
    }

    fn knowledge_implies_playable(&self, knowledge: &DeckSubset) -> bool {
        let poss = knowledge.intersect(&self.public_unknowns);
        let mut any = false;
        for i in 0..50 {
            let c = Card::new(i);
            if poss.has_card(c) {
                any = true;
                if !self.is_playable(&c) { return false; }
            }
        }
        any
    }

    fn is_slot_certainly_playable(&self, idx: usize) -> bool {
        if idx >= self.my_hand_knowledge.len() { return false; }
        self.knowledge_implies_playable(&self.my_hand_knowledge[idx])
    }

    fn is_slot_certainly_dead(&self, idx: usize) -> bool {
        if idx >= self.my_hand_knowledge.len() { return false; }
        let poss = self.my_hand_knowledge[idx].intersect(&self.public_unknowns);
        let mut any=false;
        for i in 0..50 {
            let c = Card::new(i);
            if poss.has_card(c) {
                any = true;
                if !self.is_dead(&c) { return false; }
            }
        }
        any
    }

    fn calculate_expected_distance(&self, idx: usize) -> f32 {
        if idx >= self.my_hand_knowledge.len() { return 999.0; }
        let poss = self.my_hand_knowledge[idx].intersect(&self.public_unknowns);
        let mut total = 0usize; let mut sum = 0usize;
        for i in 0..50 {
            let c = Card::new(i);
            if poss.has_card(c) {
                total += 1;
                let color_idx = c.get_color() as usize;
                let val = c.get_value();
                if self.fireworks[color_idx] >= val { sum += 10; }
                else { sum += (val - (self.fireworks[color_idx] + 1)) as usize; }
            }
        }
        if total == 0 { return 999.0; }
        (sum as f32) / (total as f32)
    }
}

impl Strategy for ChatGPT {
    fn initialize(&mut self, other_player_hand: &Vec<Card>) {
        self.hints_remaining = 8;
        self.fireworks = [0; 5];
        self.public_unknowns = DeckSubset::new_full();
        self.discarded_cards.clear();
        self.my_hand_knowledge = vec![DeckSubset::new_full(); 5];
        self.partner_hand = other_player_hand.clone();
        self.partner_hand_knowledge = vec![DeckSubset::new_full(); 5];
        for c in other_player_hand { self.public_unknowns.remove_card(*c); }
    }

    fn decide_move(&mut self) -> Move {
        // 1. Play certain
        for i in (0..self.my_hand_knowledge.len()).rev() {
            if self.is_slot_certainly_playable(i) { return Move::Play(i); }
        }

        // 2. Save clue: protect critical card in partner's chop (avoid hinting criticals everywhere)
        if self.hints_remaining > 0 && !self.partner_hand.is_empty() {
            let chop_idx = if self.partner_hand.len() == 0 { 0 } else { self.partner_hand.len()-1 };
            let chop = self.partner_hand[chop_idx];
            if self.is_critical(&chop) && (self.last_hint_value != Some(chop.get_value())) {
                return Move::HintValue(chop.get_value());
            }
        }

        // 3. Play-clue: give hints that immediately cause partner to play
        if self.hints_remaining > 0 {
                    for target in 1..=5u8 {
                for (i, card) in self.partner_hand.iter().enumerate() {
                    if card.get_value() != target { continue; }
                    if !self.is_playable(card) { continue; }
                    if self.knowledge_implies_playable(&self.partner_hand_knowledge[i]) { continue; }
                    // color
                    let k_col = self.partner_hand_knowledge[i].intersect(&DeckSubset::from_color(card.get_color()));
                            if k_col.0 != self.partner_hand_knowledge[i].0 && self.knowledge_implies_playable(&k_col) {
                                if Some(card.get_color()) != self.last_hint_color {
                                    return Move::HintColor(card.get_color());
                                }
                            }
                    // value
                    let k_val = self.partner_hand_knowledge[i].intersect(&DeckSubset::from_value(card.get_value()));
                            if k_val.0 != self.partner_hand_knowledge[i].0 && self.knowledge_implies_playable(&k_val) {
                                if Some(card.get_value()) != self.last_hint_value {
                                    return Move::HintValue(card.get_value());
                                }
                            }
                }
            }

            // 4. Setup clues for near future or critical
            if self.hints_remaining > 4 {
                        for (i, card) in self.partner_hand.iter().enumerate() {
                            if self.partner_hand_knowledge[i].0 == DeckSubset::new_full().0 {
                                let dist = if self.fireworks[card.get_color() as usize] >= card.get_value() { 255 } else { card.get_value() - (self.fireworks[card.get_color() as usize] + 1) };
                                if (self.is_critical(card) && i >= self.partner_hand.len().saturating_sub(2)) || dist <= 1 {
                                    if Some(card.get_value()) != self.last_hint_value { return Move::HintValue(card.get_value()); }
                                }
                            }
                        }
            }
        }

        // 5. Discard logic — be conservative: only discard aggressively when hints are low
        if self.hints_remaining <= 4 {
            // A: certain dead
            for i in 0..self.my_hand_knowledge.len() { if self.is_slot_certainly_dead(i) { return Move::Discard(i); } }
            // B: unhinted chop
            for i in 0..self.my_hand_knowledge.len() { if self.my_hand_knowledge[i].0 == DeckSubset::new_full().0 { return Move::Discard(i); } }
            // C: panic: discard furthest
            let mut best_idx = 0usize; let mut best_dist = -1.0f32;
            for i in 0..self.my_hand_knowledge.len() { let d = self.calculate_expected_distance(i); if d > best_dist { best_dist = d; best_idx = i; } }
            return Move::Discard(best_idx);
        }

        // 6. Force hint
        if !self.partner_hand.is_empty() {
                    for (i, card) in self.partner_hand.iter().enumerate() {
                        let k_val = self.partner_hand_knowledge[i].intersect(&DeckSubset::from_value(card.get_value()));
                        if k_val.0 != self.partner_hand_knowledge[i].0 && Some(card.get_value()) != self.last_hint_value { return Move::HintValue(card.get_value()); }
                        let k_col = self.partner_hand_knowledge[i].intersect(&DeckSubset::from_color(card.get_color()));
                        if k_col.0 != self.partner_hand_knowledge[i].0 && Some(card.get_color()) != self.last_hint_color { return Move::HintColor(card.get_color()); }
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
                    MoveResult::Play(success, card) => { if *success { self.fireworks[card.get_color() as usize] += 1; } else { self.discarded_cards.push(*card); } }
                    MoveResult::Discard(card) => { self.discarded_cards.push(*card); if self.hints_remaining < 8 { self.hints_remaining += 1; } }
                    MoveResult::Hint(_, _) => { /* not expected here for play/discard results */ }
                }
            }
            Move::HintColor(c) => {
                self.hints_remaining -= 1;
                self.last_hint_color = Some(*c);
                self.last_hint_value = None;
                let mut hinted = Vec::new();
                for (i, card) in self.partner_hand.iter().enumerate() { if card.get_color() == *c { hinted.push(i); } }
                for i in 0..self.partner_hand_knowledge.len() {
                    if hinted.contains(&i) { self.partner_hand_knowledge[i] = self.partner_hand_knowledge[i].intersect(&DeckSubset::from_color(*c)); }
                    else { self.partner_hand_knowledge[i] = self.partner_hand_knowledge[i].intersect(&DeckSubset::from_color_inverted(*c)); }
                }
            }
            Move::HintValue(v) => {
                self.hints_remaining -= 1;
                self.last_hint_value = Some(*v);
                self.last_hint_color = None;
                let mut hinted = Vec::new();
                for (i, card) in self.partner_hand.iter().enumerate() { if card.get_value() == *v { hinted.push(i); } }
                for i in 0..self.partner_hand_knowledge.len() {
                    if hinted.contains(&i) { self.partner_hand_knowledge[i] = self.partner_hand_knowledge[i].intersect(&DeckSubset::from_value(*v)); }
                    else { self.partner_hand_knowledge[i] = self.partner_hand_knowledge[i].intersect(&DeckSubset::from_value_inverted(*v)); }
                }
            }
            _ => {}
        }
    }

    fn update_after_other_player_move(&mut self, mv: &Move, mv_result: &MoveResult) {
        match mv {
            Move::Play(idx) | Move::Discard(idx) => {
                if *idx < self.partner_hand.len() {
                    let card = self.partner_hand.remove(*idx);
                    self.partner_hand_knowledge.remove(*idx);
                    self.public_unknowns.remove_card(card);
                    match mv_result {
                        MoveResult::Play(success, _) => { if *success { self.fireworks[card.get_color() as usize] += 1; } else { self.discarded_cards.push(card); } }
                        MoveResult::Discard(_) => { self.discarded_cards.push(card); if self.hints_remaining < 8 { self.hints_remaining += 1; } }
                        MoveResult::Hint(_, _) => { /* not expected here */ }
                    }
                }
            }
            Move::HintColor(c) => {
                self.hints_remaining -= 1;
                if let MoveResult::Hint(indices, _) = mv_result {
                    for &i in indices { if i < self.my_hand_knowledge.len() { self.my_hand_knowledge[i] = self.my_hand_knowledge[i].intersect(&DeckSubset::from_color(*c)); } }
                }
            }
            Move::HintValue(v) => {
                self.hints_remaining -= 1;
                if let MoveResult::Hint(indices, _) = mv_result {
                    for &i in indices { if i < self.my_hand_knowledge.len() { self.my_hand_knowledge[i] = self.my_hand_knowledge[i].intersect(&DeckSubset::from_value(*v)); } }
                }
            }
        }
    }

    fn see(&mut self, card: &Card) {
        self.partner_hand.push(*card);
        self.partner_hand_knowledge.push(DeckSubset::new_full());
        self.public_unknowns.remove_card(*card);
    }
}
