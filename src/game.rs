use rand::{RngExt, rng};

use crate::{
    board::{Board, Player},
    card::{Card, Group, Range, Special, Unit},
    deck::Cards,
};

pub struct Turn {
    current: Player,
    p1_passed: bool,
    p2_passed: bool,
}

impl Turn {
    const fn new(current: Player) -> Self {
        Self {
            current,
            p1_passed: false,
            p2_passed: false,
        }
    }

    const fn next(&mut self) {
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

    const fn both_passed(&self) -> bool {
        self.p1_passed && self.p2_passed
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
    Muster(Vec<u16>),

    /// Discard strongest non-hero units from the board
    /// If [`Range::ALL`], take into account all ranges from both players
    /// Otherwise discard units only on the opposite side on given [`Range`]
    Scorch(Range),

    /// Result of ability [`crate::card::Ability::Spy`]
    /// Take two cards from deck
    Spy,

    Mardrome,
    CommandersHorn,
    Decoy,
    None,
}

pub struct Game<C: Controller> {
    controller: C,

    turn: Turn,
    board: Board,
    actions: Vec<Action>,

    p1: Cards,
    p2: Cards,
}

impl<C: Controller> Game<C> {
    pub fn new(controller: C, p1: Cards, p2: Cards) -> Self {
        let coin = rng().random_bool(0.5);

        let turn = Turn::new(if coin { Player::P1 } else { Player::P2 });

        Self {
            controller,
            turn,
            board: Board::default(),
            actions: Vec::default(),
            p1,
            p2,
        }
    }

    pub fn start(&mut self) {
        while !self.turn.both_passed() {
            // TODO: display
            let _ = self.board.get_strengths();
            self.next_turn();
        }
    }
}

impl<C: Controller> Game<C> {
    fn next_turn(&mut self) {
        let card = self.pick_card();
        self.actions.push(Action::PlayCard(card));

        self.run_actions();
    }

    fn pick_card(&mut self) -> Card {
        let i = self.controller.select_from_hand();
        self.get_current_player_cards_mut().pick_card(i)
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
                    let range = self.controller.select_range();
                    self.board.put_agile_unit(current, unit, range);
                }
                Action::Medic => {
                    if let Some(card) = self.restore_from_pile() {
                        self.actions.push(Action::PlayCard(card));
                    }
                }
                Action::Muster(ids) => self.play_muster(&ids),
                Action::Scorch(range) => self.board.put_scorch(current, range),
                Action::Spy => self.pick_from_deck(2),
                Action::Mardrome => {
                    let range = self.controller.select_range();
                    self.board.put_row_boost(current, Special::Mardrome, range);
                }
                Action::CommandersHorn => {
                    let range = self.controller.select_range();
                    self.board
                        .put_row_boost(current, Special::CommandersHorn, range);
                }
                Action::Decoy => self.restore_from_board(),
                Action::None => break,
            }
        }

        self.board.update();
        self.turn.next();
    }

    fn restore_from_pile(&mut self) -> Option<Card> {
        let i = self.controller.select_from_pile();
        self.get_current_player_cards_mut().restore_from_pile(i)
    }

    fn restore_from_board(&mut self) {
        if let Some((range, i)) = self.controller.select_from_board() {
            let unit = self.board.remove_unit(self.turn.current, range, i);
            self.get_current_player_cards_mut().add_unit(unit);
        }
    }

    fn play_muster(&mut self, ids: &[u16]) {
        let current = self.turn.current;

        let cards = self.get_current_player_cards_mut().pick_muster(ids);

        for card in cards {
            self.board.put(current, card);
        }
    }

    fn pick_from_deck(&mut self, num: usize) {
        self.get_current_player_cards_mut().pick_from_deck(num);
    }

    const fn get_current_player_cards_mut(&mut self) -> &mut Cards {
        match self.turn.current {
            Player::P1 => &mut self.p1,
            Player::P2 => &mut self.p2,
        }
    }
}

pub trait Controller {
    fn select_from_hand(&self) -> usize;

    fn select_range(&self) -> Range;

    fn select_from_pile(&self) -> usize;

    fn select_from_board(&self) -> Option<(Range, usize)>;
}
