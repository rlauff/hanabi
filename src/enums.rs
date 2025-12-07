use crate::card::Card;
use crate::knowledge::DeckSubset;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Red,
    Green,
    Blue,
    Yellow,
    White,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Move {
    Play(usize),
    Discard(usize),
    HintColor(Color),
    HintValue(u8),
}

pub enum MoveResult{
    Play(bool, Card),
    Discard(Card),
    Hint(Vec<usize>, Vec<DeckSubset>), //indices of cards hinted, knowledge updates for each card in other player's hand
}
