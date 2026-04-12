use rand::{rng, seq::SliceRandom};

use crate::card::{Ability, Card, Group, Strength, Unit};

pub struct Cards {
    hand: Vec<Card>,
    deck: Vec<Card>,
    pile: Vec<Card>,
    _side: Vec<Card>,
}

impl Cards {
    pub fn new(deck: Deck) -> Self {
        assert!(deck.size() >= 22);

        let mut cards = deck.cards;
        cards.shuffle(&mut rng());

        let (hand, remaining) = cards
            .split_first_chunk::<10>()
            .expect("should be at least 10 cards");

        Self {
            hand: hand.to_vec(),
            deck: remaining.to_vec(),
            pile: Vec::default(),
            _side: Vec::default(),
        }
    }
}

impl Cards {
    pub fn pick_card(&mut self, i: usize) -> Card {
        self.hand.swap_remove(i)
    }

    pub fn restore_from_pile(&mut self, i: usize) -> Option<Card> {
        if let Some(Card::Unit(unit)) = self.pile.get(i)
            && matches!(unit.strength, Strength::Regular(_))
        {
            Some(self.pile.swap_remove(i))
        } else {
            None
        }
    }

    pub fn pick_from_deck(&mut self, num: usize) {
        for _ in 0..num {
            if let Some(card) = self.deck.pop() {
                self.hand.push(card);
            }
        }
    }

    pub fn pick_muster(&mut self, group: Group) -> Vec<Card> {
        let mut muster = Vec::default();

        for i in self.hand.len() - 1..=0 {
            if let Some(Card::Unit(unit)) = self.hand.get(i)
                && unit.ability == Ability::Muster(group)
            {
                let card = self.hand.swap_remove(i);
                muster.push(card);
            }
        }

        for i in self.deck.len() - 1..=0 {
            if let Some(Card::Unit(unit)) = self.deck.get(i)
                && unit.ability == Ability::Muster(group)
            {
                let card = self.hand.swap_remove(i);
                muster.push(card);
            }
        }

        muster
    }

    pub fn add_unit(&mut self, unit: Unit) {
        self.hand.push(Card::Unit(unit));
    }
}

pub struct Deck {
    cards: Vec<Card>,
}

impl Deck {
    pub const fn new(cards: Vec<Card>) -> Self {
        assert!(cards.len() >= 22);

        Self { cards }
    }

    const fn size(&self) -> usize {
        self.cards.len()
    }
}

// TODO
// struct Library {
//     monsters: Vec<Card>,
//     nilfgaard: Vec<Card>,
//     northern_realms: Vec<Card>,
//     skoiatael: Vec<Card>,
//     skellige: Vec<Card>,
//     neutral: Vec<Card>,
// }
