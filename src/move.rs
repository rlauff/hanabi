use crate::enums::Color;

pub enum Move {
    Play(usize),
    Discard(usize),
    HintColor(Color),
    HintValue(u8),
}