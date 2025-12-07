use crate::enums::{Move, MoveResult, Color};
use crate::card::Card;
use crate::strategy::Strategy;

/// Improved ChatGPT strategy: track per-slot known value/color for the partner
/// and avoid repeating identical hints. Prefer hinting 5s (critical) when
/// available and haven't been hinted yet, otherwise hint 1s if they are
/// unhinted and likely playable. Play oldest card when no useful hint.
pub struct ChatGPT {
    hints_remaining: u8,
    other_hand: Vec<Card>,
    other_known_values: Vec<Option<u8>>,
    other_known_colors: Vec<Option<Color>>,
    own_hand_size: usize,
    last_hint: Option<Hint>,
    fireworks: [u8; 5],
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Hint {
    Value(u8),
    Color(Color),
}

impl ChatGPT {
    pub fn new() -> Self {
        ChatGPT {
            hints_remaining: 8,
            other_hand: Vec::new(),
            other_known_values: Vec::new(),
            other_known_colors: Vec::new(),
            own_hand_size: 5,
            last_hint: None,
            fireworks: [0; 5],
        }
    }

    fn ensure_known_lengths(&mut self) {
        while self.other_known_values.len() < self.other_hand.len() {
            self.other_known_values.push(None);
            self.other_known_colors.push(None);
        }
        while self.other_known_values.len() > self.other_hand.len() {
            self.other_known_values.pop();
            self.other_known_colors.pop();
        }
    }

    fn choose_value_hint(&self) -> Option<u8> {
        // Prefer hints that immediately enable partner to play a card (unique playable hints).
        if self.other_hand.is_empty() { return None; }

        // helper: is card playable given known fireworks
        let is_playable = |card: &Card, fireworks: &[u8;5]| {
            let color_idx = card.get_color() as usize;
            let val = card.get_value();
            (fireworks[color_idx] + 1) == val
        };

        // look for playable cards and see if a value hint uniquely identifies one
        for card in &self.other_hand {
            if is_playable(card, &self.fireworks) {
                let v = card.get_value();
                let count_same_value = self.other_hand.iter().filter(|c| c.get_value() == v).count();
                if count_same_value == 1 {
                    return Some(v);
                }
            }
        }

        // fallback: prefer 5 then 1 if there exists at least one slot without that value hinted
        let has_unhinted = |v: u8| {
            for (i, c) in self.other_hand.iter().enumerate() {
                if c.get_value() == v && self.other_known_values.get(i).and_then(|x| *x).is_none() {
                    return true;
                }
            }
            false
        };

        if has_unhinted(5) { return Some(5); }
        if has_unhinted(1) { return Some(1); }
        None
    }
}

impl Strategy for ChatGPT {
    fn decide_move(&mut self) -> Move {
        self.ensure_known_lengths();

        if self.hints_remaining > 0 && !self.other_hand.is_empty() {
            if let Some(val) = self.choose_value_hint() {
                // avoid repeating identical hint
                if self.last_hint != Some(Hint::Value(val)) {
                    // mark hinted slots as having that value
                    for (i, c) in self.other_hand.iter().enumerate() {
                        if c.get_value() == val {
                            self.other_known_values[i] = Some(val);
                        }
                    }
                    self.last_hint = Some(Hint::Value(val));
                    return Move::HintValue(val);
                }
            }
        }

        // No useful hint or can't/shouldn't repeat: play oldest
        if self.own_hand_size > 0 {
            // clear last_hint so we can hint again after playing
            self.last_hint = None;
            return Move::Play(0);
        }

        // fallback
        Move::Discard(0)
    }

    fn initialize(&mut self, other_player_hand: &Vec<Card>) {
        self.hints_remaining = 8;
        self.other_hand = other_player_hand.clone();
        self.own_hand_size = 5;
        self.other_known_values = vec![None; self.other_hand.len()];
        self.other_known_colors = vec![None; self.other_hand.len()];
        self.last_hint = None;
    }

    fn update_after_own_move(&mut self, mv: &Move, mv_result: &MoveResult, got_new_card: bool) {
        match mv {
            Move::Play(_) => {
                if !got_new_card && self.own_hand_size > 0 {
                    self.own_hand_size -= 1;
                }
            }
            Move::Discard(_) => {
                if !got_new_card && self.own_hand_size > 0 {
                    self.own_hand_size -= 1;
                }
                if self.hints_remaining < 8 {
                    self.hints_remaining += 1;
                }
            }
            Move::HintColor(c) => {
                if self.hints_remaining > 0 { self.hints_remaining -= 1; }
                self.last_hint = Some(Hint::Color(*c));
                // when we give a color hint, mark all matching slots
                for (i, card) in self.other_hand.iter().enumerate() {
                    if card.get_color() == *c { self.other_known_colors[i] = Some(*c); }
                }
            }
            Move::HintValue(v) => {
                if self.hints_remaining > 0 { self.hints_remaining -= 1; }
                self.last_hint = Some(Hint::Value(*v));
                for (i, card) in self.other_hand.iter().enumerate() {
                    if card.get_value() == *v { self.other_known_values[i] = Some(*v); }
                }
            }
        }

        // If we played, update fireworks if play was successful
        if let Move::Play(_) = mv {
            if let MoveResult::Play(success, card) = mv_result {
                if *success {
                    let idx = card.get_color() as usize;
                    self.fireworks[idx] += 1;
                }
            }
        }
    }

    fn update_after_other_player_move(&mut self, mv: &Move, mv_result: &MoveResult) {
        match mv {
            Move::Play(idx) | Move::Discard(idx) => {
                if *idx < self.other_hand.len() {
                    self.other_hand.remove(*idx);
                    self.other_known_values.remove(*idx);
                    self.other_known_colors.remove(*idx);
                }

                // If other player drew a card, append it and set unknown knowledge
                match mv_result {
                    MoveResult::Play(success, card) => {
                        self.other_hand.push(*card);
                        self.other_known_values.push(None);
                        self.other_known_colors.push(None);
                        if *success {
                            let idx = card.get_color() as usize;
                            self.fireworks[idx] += 1;
                        }
                    }
                    MoveResult::Discard(card) => {
                        self.other_hand.push(*card);
                        self.other_known_values.push(None);
                        self.other_known_colors.push(None);
                    }
                    _ => {}
                }

                if let Move::Discard(_) = mv {
                    if self.hints_remaining < 8 { self.hints_remaining += 1; }
                }
            }
            Move::HintColor(c) => {
                if self.hints_remaining > 0 { self.hints_remaining -= 1; }
                self.last_hint = Some(Hint::Color(*c));
                // if move result contains hinted indices, update known colors
                if let MoveResult::Hint(indices, _) = mv_result {
                    for &i in indices {
                        if i < self.other_known_colors.len() { self.other_known_colors[i] = Some(*c); }
                    }
                }
            }
            Move::HintValue(v) => {
                if self.hints_remaining > 0 { self.hints_remaining -= 1; }
                self.last_hint = Some(Hint::Value(*v));
                if let MoveResult::Hint(indices, _) = mv_result {
                    for &i in indices {
                        if i < self.other_known_values.len() { self.other_known_values[i] = Some(*v); }
                    }
                }
            }
        }
    }

    fn see(&mut self, card: &Card) {
        self.other_hand.push(*card);
        self.other_known_values.push(None);
        self.other_known_colors.push(None);
    }
}
