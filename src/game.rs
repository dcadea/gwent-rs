use crate::{
    board::{Board, Player},
    card::{Card, Group, Range, Unit},
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
    PlayCard(Card),
    Agile(Unit),
    Medic,
    Muster(Group),
    Scorch(Range),
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
                    self.board.put(current, card);
                }
                Action::Agile(unit) => {
                    let range = self.select_agile_range();
                    self.board.put_agile_unit(current, unit, range);
                }
                Action::Medic => {
                    let card = self.restore_from_pile();
                    self.actions.push(Action::PlayCard(card));
                }
                Action::Muster(_) => todo!(),
                Action::Scorch(range) => todo!(),
                Action::Spy => todo!(),
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
}
