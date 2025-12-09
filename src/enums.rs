use crate::card::Card;
use crate::decksubset::DeckSubset;

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
    Play(bool, Card, Option<Card>), // success, played card, new card if drawn
    Discard(Card, Option<Card>), // discarded card, new card if drawn
    Hint(Vec<usize>), //indices of cards hinted, knowledge updates for each card in other player's hand
}
