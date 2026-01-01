use crate::strategy::Strategy;
use crate::card::Card;
use crate::enums::*;
use std::cell::RefCell;
use std::rc::Rc;

// Shared state populated by main.rs before every move
#[derive(Default, Clone)]
pub struct CheatSharedState {
    pub my_hand: Vec<Card>,
    pub partner_hand: Vec<Card>,
    pub deck_cards: Vec<Card>,
    pub fireworks: [u8; 5],
    pub hints_remaining: u8,
}

pub struct Cheater {
    pub shared_state: Rc<RefCell<CheatSharedState>>,
}

impl Cheater {
    pub fn new(shared_state: Rc<RefCell<CheatSharedState>>) -> Self {
        Cheater { shared_state }
    }

    // ------------------------------------------------------------------------
    // Helper Functions
    // ------------------------------------------------------------------------

    fn is_playable(card: &Card, fireworks: &[u8; 5]) -> bool {
        let color_idx = card.get_color() as usize;
        card.get_value() == fireworks[color_idx] + 1
    }

    fn is_dead(card: &Card, fireworks: &[u8; 5]) -> bool {
        let color_idx = card.get_color() as usize;
        card.get_value() <= fireworks[color_idx]
    }

    /// Calculates a "danger score" for discarding a card.
    /// 0 = Dead/Useless (Safe to discard)
    /// 1 = Duplicate in own hand (Safe to discard)
    /// 2 = Copy exists in Deck or Partner Hand (Safe-ish)
    /// 3 = Critical (Last copy in game) - Dangerous
    fn get_discard_score(card: &Card, my_hand: &[Card], partner_hand: &[Card], deck: &[Card], fireworks: &[u8; 5]) -> u8 {
        if Self::is_dead(card, fireworks) {
            return 0;
        }

        // Duplicate in own hand?
        if my_hand.iter().filter(|c| c == &card).count() > 1 {
            return 1;
        }

        // Duplicate elsewhere?
        let in_partner = partner_hand.iter().filter(|c| c == &card).count();
        let in_deck = deck.iter().filter(|c| c == &card).count();
        if in_partner + in_deck > 0 {
            return 2;
        }

        3 // Critical
    }

    /// Finds the best card to discard from a given hand.
    /// Returns (index, score).
    fn find_best_discard(hand: &[Card], partner_hand: &[Card], deck: &[Card], fireworks: &[u8; 5]) -> (usize, u8) {
        let mut best_idx = 0;
        let mut best_score = 4; // Worse than max (3)

        for (i, card) in hand.iter().enumerate() {
            let score = Self::get_discard_score(card, hand, partner_hand, deck, fireworks);
            if score < best_score {
                best_score = score;
                best_idx = i;
            }
        }
        (best_idx, best_score)
    }

    /// Generates a valid hint move to pass the turn.
    fn get_stall_move(partner_hand: &[Card]) -> Move {
        if let Some(c) = partner_hand.first() {
            Move::HintColor(c.get_color())
        } else {
            Move::HintValue(1)
        }
    }
}

impl Strategy for Cheater {
    fn initialize(&mut self, _other_player_hand: &Vec<Card>) {}

    fn decide_move(&mut self) -> Move {
        let state = self.shared_state.borrow();

        // -----------------------------------------------------------
        // 1. IMMEDIATE PLAY (Priority #1)
        // -----------------------------------------------------------
        for (i, card) in state.my_hand.iter().enumerate() {
            if Self::is_playable(card, &state.fireworks) {
                return Move::Play(i);
            }
        }

        // Prepare analysis for next steps
        let (my_discard_idx, my_discard_score) = Self::find_best_discard(
            &state.my_hand,
            &state.partner_hand,
            &state.deck_cards,
            &state.fireworks
        );

        let deck_empty = state.deck_cards.is_empty();

        // -----------------------------------------------------------
        // 2. FORCED DISCARD (0 Hints) - PRIORITY #2
        // -----------------------------------------------------------
        // If we have 0 hints, we CANNOT Hint. We MUST Discard.
        // Even if all cards are critical (score 3), we have no choice.
        if state.hints_remaining == 0 {
            // Edge case: If deck is empty, we cannot discard (in most rules).
            // If deck is empty and 0 hints and no plays => We are soft-locked or lost.
            // We return a discard anyway, as the game engine likely handles the "end of game" checks.
            return Move::Discard(my_discard_idx);
        }

        // -----------------------------------------------------------
        // 3. FORCED HINT (Max Hints or Empty Deck) - PRIORITY #3
        // -----------------------------------------------------------
        // If deck is empty, we can't discard (can't draw). We must Hint.
        if deck_empty {
            return Self::get_stall_move(&state.partner_hand);
        }

        // If hints are full (8), we shouldn't discard (wasteful). We Hint.
        if state.hints_remaining == 8 {
            return Self::get_stall_move(&state.partner_hand);
        }

        // -----------------------------------------------------------
        // 4. STRATEGIC DECISION (Hints > 0 and Hints < 8)
        // -----------------------------------------------------------

        let partner_can_play = state.partner_hand.iter().any(|c| Self::is_playable(c, &state.fireworks));

        // A. Stall if Partner can play
        // Giving a hint costs 0 deck cards. It allows partner to score.
        if partner_can_play {
            return Self::get_stall_move(&state.partner_hand);
        }

        // B. "Pass the Buck" (Who has the safer discard?)
        // Calculate partner's discard score
        let (_, partner_discard_score) = Self::find_best_discard(
            &state.partner_hand,
            &state.my_hand,
            &state.deck_cards,
            &state.fireworks
        );

        // If I have a safe discard (Dead card or Duplicate), just do it.
        // Or if my discard is safer/equal to partner's.
        if my_discard_score <= partner_discard_score {
            // EXCEPTION: If both of us only have Critical cards (score 3),
            // we should NOT discard. We Hint to stall death.
            // We know hints > 0 here because of check #2.
            if my_discard_score == 3 {
                return Self::get_stall_move(&state.partner_hand);
            }

            return Move::Discard(my_discard_idx);
        } else {
            // Partner has a safer discard (e.g. I have score 3, he has 0).
            // I Hint to pass the turn to him.
            return Self::get_stall_move(&state.partner_hand);
        }
    }

    fn update_after_own_move(&mut self, _mv: &Move, _res: &MoveResult, _new: bool) {}
    fn update_after_other_player_move(&mut self, _mv: &Move, _res: &MoveResult) {}
}