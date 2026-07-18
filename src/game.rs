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
                Action::Scorch(range) => {
                    for (owner, unit) in self.board.put_scorch(current, range) {
                        self.get_player_cards_mut(owner).discard(unit);
                    }
                }
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

    const fn get_player_cards_mut(&mut self, player: Player) -> &mut Cards {
        match player {
            Player::P1 => &mut self.p1,
            Player::P2 => &mut self.p2,
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
    use std::cell::{Cell, RefCell};
    use std::collections::VecDeque;

    use crate::{
        board::Player,
        card::Range,
        constants::{
            ARACHAS_1, ARACHAS_2, ARACHAS_3, BOTCHLING, CLAN_DIMUN_PIRATE, FIEND, FORKTAIL,
            NEKKER_1, NEKKER_2, NEKKER_3, REDANIAN_SOLDIER_1, SCORCH, SVANRIGE, TRISS,
            VILLENTRETENMERTH, YENNEFER,
        },
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
            0
        }

        fn select_from_board(&self) -> Option<(crate::card::Range, usize)> {
            unimplemented!()
        }
    }

    /// Plays an explicit sequence of hand indices, then passes. Needed when a
    /// test depends on the exact order cards are played.
    struct ScriptedController {
        coin: bool,
        hand: RefCell<VecDeque<usize>>,
    }

    impl ScriptedController {
        fn new(coin: bool, hand: &[usize]) -> Self {
            Self {
                coin,
                hand: RefCell::new(hand.iter().copied().collect()),
            }
        }
    }

    impl Controller for ScriptedController {
        fn toss_coin(&self) -> bool {
            self.coin
        }

        fn select_from_hand(&self) -> Option<usize> {
            self.hand.borrow_mut().pop_front()
        }

        fn select_range(&self) -> Range {
            unimplemented!()
        }

        fn select_from_pile(&self) -> usize {
            0
        }

        fn select_from_board(&self) -> Option<(Range, usize)> {
            unimplemented!()
        }
    }

    /// Asserts that a player's row holds exactly the given card ids (order
    /// independent).
    fn assert_row<C: Controller>(game: &Game<C>, player: Player, range: Range, expected: &[u16]) {
        let mut actual = game.board.get_ids(player, range);
        actual.sort_unstable();

        let mut expected = expected.to_vec();
        expected.sort_unstable();

        assert_eq!(actual, expected);
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

    // --- Card abilities resolve end-to-end through the action queue ---

    #[test]
    fn playing_a_muster_unit_pulls_its_kin_from_the_deck() {
        // P1 holds one Nekker; its two siblings wait in the deck.
        let mut game = Game::new(
            TestController::new(true, 1),
            Cards::monsters(&[NEKKER_1], &[NEKKER_2, NEKKER_3]),
            Cards::northern_realms(&[], &[]),
        );

        game.start();

        // All three Nekkers end up on P1's melee row.
        assert_row(&game, Player::P1, Range::MELEE, &[NEKKER_1, NEKKER_2, NEKKER_3]);
    }

    #[test]
    fn playing_scorch_removes_the_strongest_unit_from_the_board() {
        // P1 plays a lone Botchling (strength 4); P2 answers with Scorch.
        let mut game = Game::new(
            TestController::new(true, 2),
            Cards::monsters(&[BOTCHLING], &[]),
            Cards::northern_realms(&[SCORCH], &[]),
        );

        game.start();

        // Botchling was the strongest unit, so global scorch clears it.
        assert_row(&game, Player::P1, Range::MELEE, &[]);
        assert_row(&game, Player::P2, Range::MELEE, &[]);
    }

    #[test]
    fn a_units_global_scorch_ability_clears_the_strongest_units_on_both_sides() {
        // P1 plays a Fiend (6); P2 answers with Clan Dimun Pirate (6), whose
        // ability scorches the whole battlefield — both 6s go, including itself.
        let mut game = Game::new(
            TestController::new(true, 2),
            Cards::monsters(&[FIEND], &[]),
            Cards::skellige(&[CLAN_DIMUN_PIRATE], &[]),
        );

        game.start();

        assert_row(&game, Player::P1, Range::MELEE, &[]);
        assert_row(&game, Player::P2, Range::RANGED, &[]);
    }

    #[test]
    fn global_scorch_ability_kills_the_caster_when_the_opponent_matches_its_strength() {
        // P1 fields a Fiend (6). P2 plays Svanrige (4) and then Clan Dimun
        // Pirate (6), whose global scorch fires. The board-wide max is 6 — tied
        // between the Fiend and the pirate itself — so both are destroyed while
        // Svanrige (4) survives on the pirate's own side.
        //
        // Call order is P1, P2, P2 (P1 passes with an empty hand before the
        // pirate lands), so every scripted index is 0.
        let mut game = Game::new(
            ScriptedController::new(true, &[0, 0, 0]),
            Cards::monsters(&[FIEND], &[]),
            Cards::skellige(&[SVANRIGE, CLAN_DIMUN_PIRATE], &[]),
        );

        game.start();

        // The Fiend and the pirate both went to strength 6 and were scorched...
        assert_row(&game, Player::P1, Range::MELEE, &[]);
        assert_row(&game, Player::P2, Range::RANGED, &[]);
        // ...but the weaker Svanrige (4) is untouched.
        assert_row(&game, Player::P2, Range::MELEE, &[SVANRIGE]);
    }

    #[test]
    fn a_units_row_scorch_ability_hits_only_the_targeted_opponent_row() {
        // P2 musters three Arachas onto its melee row (4 + 4 + 4 = 12, past the
        // 10-strength threshold). P1 then plays Villentretenmerth, whose melee
        // scorch clears the strongest units on P2's melee row only.
        let mut game = Game::new(
            TestController::new(false, 2),
            Cards::northern_realms(&[VILLENTRETENMERTH], &[]),
            Cards::monsters(&[ARACHAS_1], &[ARACHAS_2, ARACHAS_3]),
        );

        game.start();

        // P2's whole melee row (all the Arachas) is scorched away...
        assert_row(&game, Player::P2, Range::MELEE, &[]);
        // ...while Villentretenmerth survives on P1's own melee row.
        assert_row(&game, Player::P1, Range::MELEE, &[VILLENTRETENMERTH]);
    }

    #[test]
    fn row_scorch_does_nothing_below_the_ten_strength_threshold() {
        // P2's melee row totals 5 + 4 = 9, just under the threshold, so
        // Villentretenmerth's melee scorch must leave both units alone.
        // P1 plays a throwaway Redanian first so Villen resolves only after P2
        // has both units down. Script indices follow the P2, P1, P2, P1 call
        // order (P2 starts on tails).
        let mut game = Game::new(
            ScriptedController::new(false, &[0, 1, 0, 0]),
            Cards::northern_realms(&[REDANIAN_SOLDIER_1, VILLENTRETENMERTH], &[]),
            Cards::monsters(&[FORKTAIL, BOTCHLING], &[]),
        );

        game.start();

        // Under threshold: both units stay put.
        assert_row(&game, Player::P2, Range::MELEE, &[FORKTAIL, BOTCHLING]);
    }

    #[test]
    fn row_scorch_counts_heroes_toward_the_threshold_but_never_removes_them() {
        // P2's melee row is Triss (hero, 7) + Forktail (5) = 12, clearing the
        // threshold. Villentretenmerth's melee scorch removes the strongest
        // *non-hero* (Forktail, 5); the hero survives even though it is
        // stronger.
        let mut game = Game::new(
            ScriptedController::new(false, &[0, 1, 0, 0]),
            Cards::northern_realms(&[REDANIAN_SOLDIER_1, VILLENTRETENMERTH], &[]),
            Cards::monsters(&[TRISS, FORKTAIL], &[]),
        );

        game.start();

        // Only Triss remains — Forktail was scorched, the hero was not.
        assert_row(&game, Player::P2, Range::MELEE, &[TRISS]);
    }

    #[test]
    fn muster_never_pulls_from_the_discard_pile() {
        // P1 must play, in order: Nekker, then Scorch, then Yennefer; P2 passes.
        //   1. Nekker musters its two siblings from the deck -> three Nekkers on
        //      the melee row.
        //   2. Scorch clears the whole row (all strength 2) into P1's pile.
        //   3. Yennefer (Medic) restores one Nekker from the pile and replays
        //      it. That replay re-triggers Muster, which must ignore the two
        //      Nekkers still sitting in the pile.
        //
        // `Cards::monsters` lays the hand out as [neutral, faction, special] =
        // [Yennefer, Nekker, Scorch]. Accounting for `pick_card`'s
        // `swap_remove`, the indices below play Nekker, then Scorch, then
        // Yennefer.
        let mut game = Game::new(
            ScriptedController::new(true, &[1, 1, 0]),
            Cards::monsters(&[YENNEFER, NEKKER_1, SCORCH], &[NEKKER_2, NEKKER_3]),
            Cards::northern_realms(&[], &[]),
        );

        game.start();

        // Exactly one Nekker is back (the medic's), proving Muster did not drag
        // the other two out of the pile.
        assert_eq!(game.board.get_ids(Player::P1, Range::MELEE).len(), 1);
        // Yennefer herself stands on the ranged row.
        assert_row(&game, Player::P1, Range::RANGED, &[YENNEFER]);
    }
}
