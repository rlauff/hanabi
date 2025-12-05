
use crate::card::Card;

// encoding: tens place = color, units place map: 1 1 1 2 2 3 3 4 4 5

#[derive(Copy, Clone)]
pub struct Knowledge (pub u64);

impl Knowledge {
    pub fn new_full() -> Self {
        Knowledge(!0)
    }

    pub fn has_card(&self, card: Card) -> bool {
        (self.0 & (1 << card.0)) & 1 != 0
    }

    pub fn remove_card(&mut self, card: Card) {
        self.0 &= !(1 << card.0);
    }

    pub fn add_card(&mut self, card: Card) {
        self.0 |= 1 << card.0;
    }
}
