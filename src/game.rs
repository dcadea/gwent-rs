use crate::{
    board::{Board, Player},
    card::{Card, Range, Special, Unit},
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

    const fn pass(&mut self) {
        match self.current {
            Player::P1 => self.p1_passed = true,
            Player::P2 => self.p2_passed = true,
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

    /// The current player ends their turn
    Pass,

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
        let coin = controller.toss_coin();

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
        let action = self.pick_action();
        self.actions.push(action);

        self.run_actions();
    }

    fn pick_action(&mut self) -> Action {
        // A player with no cards left is forced to pass, otherwise they choose
        // whether to play a card or pass (`select_from_hand` returns `None`).
        // TODO: don't auto-pass on an empty hand once leader ability evaluation
        // is implemented — a leader ability can still be used with no cards.
        if self.get_current_player_cards().is_hand_empty() {
            return Action::Pass;
        }

        match self.controller.select_from_hand() {
            Some(i) => Action::PlayCard(self.get_current_player_cards_mut().pick_card(i)),
            None => Action::Pass,
        }
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
                Action::Pass => self.turn.pass(),
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

    const fn get_current_player_cards(&self) -> &Cards {
        match self.turn.current {
            Player::P1 => &self.p1,
            Player::P2 => &self.p2,
        }
    }

    const fn get_current_player_cards_mut(&mut self) -> &mut Cards {
        match self.turn.current {
            Player::P1 => &mut self.p1,
            Player::P2 => &mut self.p2,
        }
    }
}

pub trait Controller {
    fn toss_coin(&self) -> bool;

    /// Returns the index of the card to play from hand, or `None` to pass.
    fn select_from_hand(&self) -> Option<usize>;

    fn select_range(&self) -> Range;

    fn select_from_pile(&self) -> usize;

    fn select_from_board(&self) -> Option<(Range, usize)>;
}

#[cfg(test)]
mod test {
    use std::cell::Cell;

    use crate::{
        board::Player,
        constants::{BOTCHLING, REDANIAN_SOLDIER_1},
        deck::Cards,
        game::{Controller, Game, Turn},
    };

    struct TestController {
        coin: bool,
        /// Number of cards to play (always from index 0) before passing.
        plays: Cell<usize>,
    }

    impl TestController {
        const fn new(coin: bool, plays: usize) -> Self {
            Self {
                coin,
                plays: Cell::new(plays),
            }
        }

        const fn with_coin(coin: bool) -> Self {
            Self::new(coin, 0)
        }
    }

    impl Controller for TestController {
        fn toss_coin(&self) -> bool {
            self.coin
        }

        fn select_from_hand(&self) -> Option<usize> {
            let plays = self.plays.get();
            if plays == 0 {
                return None;
            }
            self.plays.set(plays - 1);
            Some(0)
        }

        fn select_range(&self) -> crate::card::Range {
            unimplemented!()
        }

        fn select_from_pile(&self) -> usize {
            unimplemented!()
        }

        fn select_from_board(&self) -> Option<(crate::card::Range, usize)> {
            unimplemented!()
        }
    }

    // --- Turn state machine ---

    #[test]
    fn new_turn_starts_with_given_player_and_no_passes() {
        let turn = Turn::new(Player::P1);

        assert_eq!(turn.current, Player::P1);
        assert!(!turn.p1_passed);
        assert!(!turn.p2_passed);
        assert!(!turn.both_passed());
    }

    #[test]
    fn next_alternates_while_nobody_has_passed() {
        let mut turn = Turn::new(Player::P1);

        turn.next();
        assert_eq!(turn.current, Player::P2);

        turn.next();
        assert_eq!(turn.current, Player::P1);
    }

    #[test]
    fn next_sticks_to_p2_after_p1_passed() {
        let mut turn = Turn::new(Player::P1);
        turn.pass();

        turn.next();
        assert_eq!(turn.current, Player::P2);

        turn.next();
        assert_eq!(turn.current, Player::P2);
    }

    #[test]
    fn next_sticks_to_p1_after_p2_passed() {
        let mut turn = Turn::new(Player::P2);
        turn.pass();

        turn.next();
        assert_eq!(turn.current, Player::P1);

        turn.next();
        assert_eq!(turn.current, Player::P1);
    }

    #[test]
    fn next_keeps_current_once_both_passed() {
        let mut turn = Turn::new(Player::P1);
        turn.pass();
        turn.next();
        turn.pass();

        assert!(turn.both_passed());

        turn.next();
        assert_eq!(turn.current, Player::P2);
    }

    #[test]
    fn pass_marks_only_the_current_player() {
        let mut turn = Turn::new(Player::P1);
        turn.pass();
        assert!(turn.p1_passed);
        assert!(!turn.p2_passed);

        turn.next();
        turn.pass();
        assert!(turn.both_passed());
    }

    // --- Coin toss decides the starting player ---

    #[test]
    fn heads_makes_p1_start() {
        let game = Game::new(
            TestController::with_coin(true),
            Cards::monsters(&[], &[]),
            Cards::northern_realms(&[], &[]),
        );

        assert_eq!(game.turn.current, Player::P1);
    }

    #[test]
    fn tails_makes_p2_start() {
        let game = Game::new(
            TestController::with_coin(false),
            Cards::monsters(&[], &[]),
            Cards::northern_realms(&[], &[]),
        );

        assert_eq!(game.turn.current, Player::P2);
    }

    // --- The game loop terminates once both players pass ---

    #[test]
    fn game_ends_when_both_players_pass_immediately() {
        let mut game = Game::new(
            TestController::with_coin(true),
            Cards::monsters(&[], &[]),
            Cards::northern_realms(&[], &[]),
        );

        game.start();

        assert!(game.turn.both_passed());
    }

    #[test]
    fn game_ends_after_players_exhaust_their_hands() {
        let mut game = Game::new(
            TestController::new(true, 2),
            Cards::monsters(&[BOTCHLING], &[]),
            Cards::northern_realms(&[REDANIAN_SOLDIER_1], &[]),
        );

        game.start();

        assert!(game.turn.both_passed());
    }
}
