use crate::enums::{Move, MoveResult, Color};
use crate::card::Card;
use crate::strategy::Strategy;
use crate::decksubset::DeckSubset;

pub struct Gemini { 
    hints_remaining: u8,
    fireworks: [u8; 5],
    
    // Wissen über die eigene Hand
    my_hand_knowledge: Vec<DeckSubset>,
    
    // Was ich in der Hand des Partners sehe
    partner_hand: Vec<Card>,
    
    // Was ich glaube, dass der Partner über seine Hand weiß
    partner_hand_knowledge: Vec<DeckSubset>,

    // Alle Karten, die noch im Spiel sein könnten (nicht abgeworfen, nicht gespielt, nicht beim Partner gesehen)
    // Dies repräsentiert Karten, die im Deck ODER in meiner Hand sein könnten.
    unknown_cards: DeckSubset,

    // Tracking für abgeworfene Karten, um "Save Clues" zu berechnen
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
            unknown_cards: DeckSubset::new_full(),
            discarded_cards: Vec::new(),
        }
    }

    /// Entfernt eine Karte aus dem Pool der Möglichkeiten für MEINE Hand.
    fn mark_seen(&mut self, card: &Card) {
        self.unknown_cards.remove_card(*card);
    }

    /// Ist die Karte exakt jetzt spielbar?
    fn is_playable(&self, card: &Card) -> bool {
        let color_idx = card.get_color() as usize;
        let val = card.get_value();
        self.fireworks[color_idx] + 1 == val
    }

    /// Ist die Karte nutzlos (bereits gespielt)?
    fn is_dead(&self, card: &Card) -> bool {
        let color_idx = card.get_color() as usize;
        let val = card.get_value();
        self.fireworks[color_idx] >= val
    }

    /// Zählt, wie viele Kopien dieser Karte bereits im Müll liegen.
    fn count_in_discard(&self, card: &Card) -> usize {
        self.discarded_cards.iter().filter(|&c| c.get_color() == card.get_color() && c.get_value() == card.get_value()).count()
    }

    /// Prüft, ob eine Karte "kritisch" ist (letzte Chance, sie zu spielen).
    fn is_critical(&self, card: &Card) -> bool {
        if self.is_dead(card) { return false; } // Bereits gespielt -> nicht kritisch
        let val = card.get_value();
        if val == 5 { return true; } // 5er sind immer einzigartig

        let copies_in_discard = self.count_in_discard(card);
        let max_copies = match val {
            1 => 3,
            2 | 3 | 4 => 2,
            _ => 1
        };
        // Wenn alle anderen Kopien weg sind, ist diese kritisch
        copies_in_discard + 1 >= max_copies
    }

    /// Bin ich zu 100% sicher, dass mein Slot 'index' spielbar ist?
    fn is_slot_certainly_playable(&self, index: usize) -> bool {
        if index >= self.my_hand_knowledge.len() { return false; }
        
        let possibilities = self.my_hand_knowledge[index].intersect(&self.unknown_cards);
        let mut any_possible = false;
        
        for i in 0..50 {
            let c = Card::new(i);
            if possibilities.has_card(c) {
                any_possible = true;
                if !self.is_playable(&c) {
                    return false; // Risiko!
                }
            }
        }
        any_possible
    }

    /// Bin ich sicher, dass diese Karte nutzlos ist?
    fn is_slot_certainly_dead(&self, index: usize) -> bool {
        if index >= self.my_hand_knowledge.len() { return false; }
        let possibilities = self.my_hand_knowledge[index].intersect(&self.unknown_cards);
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

    /// Simuliert, ob das gegebene Wissen (subset) ausreicht, um die Karte sicher zu spielen.
    /// Dies nutzt 'unknown_cards' als Approximation für das globale Wissen.
    fn knowledge_implies_playable(&self, knowledge: &DeckSubset) -> bool {
        let possibilities = knowledge.intersect(&self.unknown_cards);
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
}

impl Strategy for Gemini {
    fn initialize(&mut self, other_player_hand: &Vec<Card>) {
        self.hints_remaining = 8;
        self.fireworks = [0; 5];
        self.unknown_cards = DeckSubset::new_full();
        self.discarded_cards.clear();
        self.my_hand_knowledge = vec![DeckSubset::new_full(); 5];
        self.partner_hand = other_player_hand.clone();
        self.partner_hand_knowledge = vec![DeckSubset::new_full(); 5];

        for card in other_player_hand {
            self.mark_seen(card);
        }
    }

    fn decide_move(&mut self) -> Move {
        // 1. PLAY: Eigene sichere Karten spielen (Neueste zuerst/LIFO für bessere Info-Zyklen)
        for i in (0..self.my_hand_knowledge.len()).rev() {
            if self.is_slot_certainly_playable(i) {
                return Move::Play(i);
            }
        }

        // 2. SAVE CLUE: Wenn Partner eine kritische Karte abwerfen würde (Slot 0)
        // Wir tun dies nur, wenn wir Hinweise haben.
        if self.hints_remaining > 0 && !self.partner_hand.is_empty() {
            let chop_idx = 0; // Hanabi Konvention: Älteste Karte wird abgeworfen
            let chop_card = self.partner_hand[chop_idx];

            if self.is_critical(&chop_card) {
                // Partner darf diese Karte nicht verlieren!
                // Prüfen, ob Partner schon Information darüber hat (nicht DeckSubset::full)
                let info_present = self.partner_hand_knowledge[chop_idx].0 != DeckSubset::new_full().0;
                
                // Wenn er noch gar nichts weiß, warnen wir ihn.
                if !info_present {
                    // Bevorzuge Value-Hinweis, da oft klarer bei Einzelkarten (z.B. 5er)
                    // Aber wir müssen sicherstellen, dass der Hinweis "neu" ist.
                    return Move::HintValue(chop_card.get_value());
                }
            }
        }

        // 3. PLAY CLUE: Hinweise geben, die den Partner SOFORT zum Spielen bringen
        if self.hints_remaining > 0 {
            // Wir iterieren über die Karten des Partners, um spielbare zu finden
            // Priorität: Niedrige Werte zuerst (1er vor 2ern)
            for target_val in 1..=5 {
                for (i, card) in self.partner_hand.iter().enumerate() {
                    if card.get_value() != target_val { continue; }
                    if !self.is_playable(card) { continue; }

                    // Wenn der Partner es schon weiß, nichts tun
                    if self.knowledge_implies_playable(&self.partner_hand_knowledge[i]) {
                        continue;
                    }

                    // Simulieren: Welcher Hinweis löst das Problem?
                    
                    // A. Farb-Hinweis
                    let color = card.get_color();
                    let k_after_color = self.partner_hand_knowledge[i].intersect(&DeckSubset::from_color(color));
                    let is_new_color = k_after_color.0 != self.partner_hand_knowledge[i].0;
                    let solves_color = self.knowledge_implies_playable(&k_after_color);

                    if solves_color && is_new_color {
                        return Move::HintColor(color);
                    }

                    // B. Wert-Hinweis
                    let val = card.get_value();
                    let k_after_val = self.partner_hand_knowledge[i].intersect(&DeckSubset::from_value(val));
                    let is_new_val = k_after_val.0 != self.partner_hand_knowledge[i].0;
                    let solves_val = self.knowledge_implies_playable(&k_after_val);

                    if solves_val && is_new_val {
                        return Move::HintValue(val);
                    }
                }
            }

            // 4. SETUP CLUE: Wenn wir viele Hinweise haben (>4) und keinen direkten Play sehen,
            // geben wir Info über wertvolle Karten (z.B. noch nicht gespielte Farben/Werte),
            // damit der Partner sie nicht abwirft oder später spielen kann.
            if self.hints_remaining > 4 {
                for (i, card) in self.partner_hand.iter().enumerate() {
                    // Wir geben Hinweise auf Karten, die nützlich (nicht tot) sind und noch völlig unbekannt
                    if !self.is_dead(card) && self.partner_hand_knowledge[i].0 == DeckSubset::new_full().0 {
                        // Einfachster Heuristik: Value Hinweis
                        return Move::HintValue(card.get_value());
                    }
                }
            }
        }

        // 5. DISCARD: Wenn keine sinnvollen Hinweise oder keine Tokens
        // A. Sicher tote Karten wegwerfen
        if self.hints_remaining < 8 {
            for i in 0..self.my_hand_knowledge.len() {
                if self.is_slot_certainly_dead(i) {
                    return Move::Discard(i);
                }
            }
        }

        // B. Chop (Älteste Karte ohne Hinweise wegwerfen)
        if self.hints_remaining < 8 {
            // Suche die älteste Karte (Index 0 aufwärts), über die wir gar nichts wissen
            for i in 0..self.my_hand_knowledge.len() {
                if self.my_hand_knowledge[i].0 == DeckSubset::new_full().0 {
                    return Move::Discard(i);
                }
            }
            // Fallback: Einfach Index 0 (älteste) wegwerfen, auch wenn wir Hinweise haben (Risiko, aber notwendig)
            return Move::Discard(0);
        }

        // 6. FORCE HINT: Wenn 8 Tokens voll sind, MÜSSEN wir einen Hinweis geben.
        // Wir suchen irgendeinen Hinweis, der "neu" ist (Redundanz vermeiden).
        if !self.partner_hand.is_empty() {
             for (i, card) in self.partner_hand.iter().enumerate() {
                 let val = card.get_value();
                 let k_val = self.partner_hand_knowledge[i].intersect(&DeckSubset::from_value(val));
                 if k_val.0 != self.partner_hand_knowledge[i].0 {
                     return Move::HintValue(val);
                 }
                 
                 let col = card.get_color();
                 let k_col = self.partner_hand_knowledge[i].intersect(&DeckSubset::from_color(col));
                 if k_col.0 != self.partner_hand_knowledge[i].0 {
                     return Move::HintColor(col);
                 }
             }
             // Wenn wir wirklich gar keine neuen Infos geben können (sehr unwahrscheinlich), random fallback
             return Move::HintValue(self.partner_hand[0].get_value());
        }

        Move::Discard(0) 
    }

    fn update_after_own_move(&mut self, mv: &Move, mv_result: &MoveResult, got_new_card: bool) {
        match mv {
            Move::Play(idx) | Move::Discard(idx) => {
                if *idx < self.my_hand_knowledge.len() {
                    self.my_hand_knowledge.remove(*idx);
                }
                if got_new_card {
                    self.my_hand_knowledge.push(DeckSubset::new_full());
                }

                match mv_result {
                    MoveResult::Play(success, card) => {
                        self.mark_seen(card);
                        if *success {
                            self.fireworks[card.get_color() as usize] += 1;
                        } else {
                            self.discarded_cards.push(*card);
                        }
                    },
                    MoveResult::Discard(card) => {
                        self.mark_seen(card);
                        self.discarded_cards.push(*card);
                        if self.hints_remaining < 8 { self.hints_remaining += 1; }
                    },
                    _ => {}
                }
            },
            Move::HintColor(c) => {
                self.hints_remaining -= 1;
                let mut hinted_indices = Vec::new();
                for (i, card) in self.partner_hand.iter().enumerate() {
                     if card.get_color() == *c { hinted_indices.push(i); }
                }
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
                for (i, card) in self.partner_hand.iter().enumerate() {
                     if card.get_value() == *v { hinted_indices.push(i); }
                }
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
                    self.mark_seen(&card);

                    match mv_result {
                        MoveResult::Play(success, _) => {
                            if *success {
                                self.fireworks[card.get_color() as usize] += 1;
                            } else {
                                self.discarded_cards.push(card);
                            }
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
                if let MoveResult::Hint(indices, _) = mv_result {
                    hinted_indices = indices.clone();
                }

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
                if let MoveResult::Hint(indices, _) = mv_result {
                    hinted_indices = indices.clone();
                }

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
        self.mark_seen(card);
        self.partner_hand.push(*card);
        self.partner_hand_knowledge.push(DeckSubset::new_full());
    }
}