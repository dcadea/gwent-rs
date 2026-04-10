use std::thread::current;

use crate::{
    board::{Board, Player},
    card::{Card, Group, Range, Unit},
    deck::Cards,
};

pub struct Turn {
    current: Player,
    p1_passed: bool,
    p2_passed: bool,
}

impl Turn {
    pub const fn next(&mut self) {
        match (self.p1_passed, self.p2_passed) {
            (true, true) => {}
            (true, false) => self.current = Player::P2,
            (false, true) => self.current = Player::P1,
            (false, false) => {
                self.current = match self.current {
                    Player::P1 => Player::P2,
                    Player::P2 => Player::P1,
                };
            }
        }
    }
}

pub enum Action {
    /// Just play a card
    PlayCard(Card),

    /// Only happens when [`Range::AGILE`] is played
    /// Select a row when agile unit card is placed
    Agile(Unit),

    /// Result of ability [`crate::card::Ability::Medic`]
    /// Return a card from discard pile
    Medic,

    /// Find all [`crate::card::Ability::Muster`] cards of a kind in both hand and deck
    /// and play immediately
    Muster(Group),

    /// Discard strongest non-hero units from the board
    /// If [`Range::ALL`], take into account all ranges from both players
    /// Otherwise discard units only on the opposite side on given [`Range`]
    Scorch(Range),

    /// Result of ability [`crate::card::Ability::Spy`]
    /// Take two cards from deck
    Spy,

    Berserker,
    Mardrome(Range),
    CommandersHorn,
    Decoy,
    None,
}

pub struct Game {
    turn: Turn,
    board: Board,
    actions: Vec<Action>,

    p1: Cards,
    p2: Cards,
}

impl Game {
    fn play_card(&mut self, i: usize) {
        let card: Card = todo!("select card from hand");

        self.actions.push(Action::PlayCard(card));

        self.run_actions();
    }

    fn run_actions(&mut self) {
        let current = self.turn.current;

        while let Some(action) = self.actions.pop() {
            match action {
                Action::PlayCard(card) => {
                    let action = self.board.put(current, card);
                    self.actions.push(action);
                }
                Action::Agile(unit) => {
                    let range = self.select_agile_range();
                    self.board.put_agile_unit(current, unit, range);
                }
                Action::Medic => {
                    let card = self.restore_from_pile();
                    self.actions.push(Action::PlayCard(card));
                }
                Action::Muster(group) => self.play_muster(group),
                Action::Scorch(range) => todo!(),
                Action::Spy => self.pick_from_deck(2),
                Action::Berserker => todo!(),
                Action::Mardrome(range) => todo!(),
                Action::CommandersHorn => todo!(),
                Action::Decoy => todo!(),
                Action::None => break,
            }
        }
    }

    fn select_agile_range(&self) -> Range {
        todo!("user selects MELEE or RANGED range")
    }

    fn restore_from_pile(&self) -> Card {
        todo!("user selects a card from pile")
    }

    fn play_muster(&mut self, group: Group) {
        let current = self.turn.current;
        let cards = match current {
            Player::P1 => self.p1.pick_muster(group),
            Player::P2 => self.p2.pick_muster(group),
        };

        cards.into_iter().for_each(|card| {
            self.board.put(current, card);
        });
    }

    fn pick_from_deck(&mut self, num: usize) {
        match self.turn.current {
            Player::P1 => self.p1.pick_from_deck(num),
            Player::P2 => self.p2.pick_from_deck(num),
        }
    }
}
