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

    const fn reset(&mut self) {
        self.p1_passed = false;
        self.p2_passed = false;
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

    gems: [u8; 2],

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
            gems: [2, 2],
            p1,
            p2,
        }
    }

    pub fn start(&mut self) {
        loop {
            self.play_round();

            if self.summarize_round() {
                // somebody lost
                break;
            }

            self.end_round();
        }
    }
}

impl<C: Controller> Game<C> {
    pub fn play_round(&mut self) {
        while !self.turn.both_passed() {
            // TODO: display
            let _ = self.board.get_strengths();
            self.next_turn();
        }
    }

    pub fn end_round(&mut self) {
        for (owner, unit) in self.board.clear() {
            self.get_player_cards_mut(owner).discard(unit);
        }

        self.turn.reset();
    }

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

        self.controller
            .select_from_hand()
            .map_or(Action::Pass, |i| {
                Action::PlayCard(self.get_current_player_cards_mut().pick_card(i))
            })
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

    fn summarize_round(&mut self) -> bool {
        let (p1, p2) = (
            self.board.get_total_strength(Player::P1),
            self.board.get_total_strength(Player::P2),
        );

        if p1 <= p2 {
            let gems = self.gems_mut(Player::P1);
            *gems = gems.saturating_sub(1);
        }
        if p2 <= p1 {
            let gems = self.gems_mut(Player::P2);
            *gems = gems.saturating_sub(1);
        }

        self.gems.contains(&0)
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

    const fn gems_mut(&mut self, player: Player) -> &mut u8 {
        match player {
            Player::P1 => &mut self.gems[0],
            Player::P2 => &mut self.gems[1],
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
        card::{Range, Special},
        constants::{
            ARACHAS_1, ARACHAS_2, ARACHAS_3, BARCLAY_ELS, BERSERKER, BIRNA_BRAN, BITING_FROST,
            BLUE_STRIPES_1, BLUE_STRIPES_2, BOTCHLING, CATAPULT_1, CATAPULT_2, CERYS,
            CLAN_DIMUN_PIRATE, CLEAR_WEATHER, COMMANDERS_HORN, DANDELION, DECOY, DRAGON_HUNTER_1,
            DRAGON_HUNTER_2, DRUMMOND_SHIELDMAIDEN_1, DRUMMOND_SHIELDMAIDEN_2,
            DRUMMOND_SHIELDMAIDEN_3, DUN_BANNER_MEDIC, ERMION, ETOLIAN_ARCHERS_1,
            ETOLIAN_ARCHERS_2, FIEND, FORKTAIL, HEMDALL, IDA_EMEAN, IMPENETRABLE_FOG,
            IMPERA_BRIGADE_1, IMPERA_BRIGADE_2, ISENGRIM, KAMBI, KEIRA_METZ, MARDROME, NEKKER_1,
            NEKKER_2, NEKKER_3, OLGIERD, PRINCE_STENNIS, REDANIAN_SOLDIER_1, SCORCH,
            SIEGE_EXPERT_1, SKELLIGE_STORM, SVANRIGE, TORRENTIAL_RAIN, TRISS, VESEMIR, VILDKAARL,
            VILLENTRETENMERTH, YENNEFER, YOUNG_BERSERKER_1, YOUNG_BERSERKER_2, YOUNG_BERSERKER_3,
            YOUNG_VILDKAARL, ZOLTAN,
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
            Range::MELEE
        }

        fn select_from_pile(&self) -> usize {
            0
        }

        fn select_from_board(&self) -> Option<(crate::card::Range, usize)> {
            Some((Range::MELEE, 0))
        }
    }

    /// Plays an explicit sequence of hand indices, then passes. Needed when a
    /// test depends on the exact order cards are played.
    struct ScriptedController {
        coin: bool,
        hand: RefCell<VecDeque<usize>>,
        /// Row chosen for agile units and row boosts.
        range: Range,
        /// Board slot Decoy pulls back to hand.
        decoy_target: Option<(Range, usize)>,
    }

    impl ScriptedController {
        fn new(coin: bool, hand: &[usize]) -> Self {
            Self {
                coin,
                hand: RefCell::new(hand.iter().copied().collect()),
                range: Range::MELEE,
                decoy_target: None,
            }
        }

        fn with_range(mut self, range: Range) -> Self {
            self.range = range;
            self
        }

        fn with_decoy_target(mut self, target: (Range, usize)) -> Self {
            self.decoy_target = Some(target);
            self
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
            self.range
        }

        fn select_from_pile(&self) -> usize {
            0
        }

        fn select_from_board(&self) -> Option<(Range, usize)> {
            self.decoy_target
        }
    }

    /// Returns a player's row as sorted `(card id, strength)` pairs, so both the
    /// units present and their computed strengths can be asserted together.
    fn row_cards<C: Controller>(game: &Game<C>, player: Player, range: Range) -> Vec<(u16, u8)> {
        let ids = game.board.get_ids(player, range);

        let strengths = game.board.get_strengths();
        let side = match player {
            Player::P1 => strengths.p1,
            Player::P2 => strengths.p2,
        };

        let mut pairs: Vec<(u16, u8)> = ids
            .into_iter()
            .zip(side.get(range).iter().map(|s| s.get()))
            .collect();
        pairs.sort_unstable();
        pairs
    }

    /// Asserts a player's row holds exactly the given `(card id, strength)`
    /// pairs (order independent).
    fn assert_cards<C: Controller>(
        game: &Game<C>,
        player: Player,
        range: Range,
        expected: &[(u16, u8)],
    ) {
        let mut expected = expected.to_vec();
        expected.sort_unstable();

        assert_eq!(row_cards(game, player, range), expected);
    }

    /// Asserts a player's hand holds exactly the given card ids (order
    /// independent).
    fn assert_hand(cards: &Cards, expected: &[u16]) {
        let mut actual = cards.hand_ids();
        actual.sort_unstable();

        let mut expected = expected.to_vec();
        expected.sort_unstable();

        assert_eq!(actual, expected);
    }

    /// Asserts the boost special sitting on a player's row.
    fn assert_boost<C: Controller>(
        game: &Game<C>,
        player: Player,
        range: Range,
        expected: Option<Special>,
    ) {
        assert_eq!(game.board.get_boost(player, range), expected);
    }

    /// Asserts a player's discard pile holds exactly the given card ids (order
    /// independent).
    fn assert_pile(cards: &Cards, expected: &[u16]) {
        let mut actual = cards.pile_ids();
        actual.sort_unstable();

        let mut expected = expected.to_vec();
        expected.sort_unstable();

        assert_eq!(actual, expected);
    }

    // --- Turn cursor + pass flags ---

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

        game.play_round();

        assert!(game.turn.both_passed());
    }

    #[test]
    fn game_ends_after_players_exhaust_their_hands() {
        let mut game = Game::new(
            TestController::new(true, 2),
            Cards::monsters(&[BOTCHLING], &[]),
            Cards::northern_realms(&[REDANIAN_SOLDIER_1], &[]),
        );

        game.play_round();

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

        game.play_round();

        // All three Nekkers end up on P1's melee row.
        assert_cards(
            &game,
            Player::P1,
            Range::MELEE,
            &[(NEKKER_1, 2), (NEKKER_2, 2), (NEKKER_3, 2)],
        );
    }

    #[test]
    fn playing_a_muster_unit_pulls_kin_from_both_hand_and_deck() {
        // One sibling stays in hand, the other waits in the deck: playing the
        // first Nekker musters both, so `pick_muster` gathers from the hand as
        // well as the deck.
        let mut game = Game::new(
            TestController::new(true, 1),
            Cards::monsters(&[NEKKER_1, NEKKER_2], &[NEKKER_3]),
            Cards::northern_realms(&[], &[]),
        );

        game.play_round();

        assert_cards(
            &game,
            Player::P1,
            Range::MELEE,
            &[(NEKKER_1, 2), (NEKKER_2, 2), (NEKKER_3, 2)],
        );
    }

    #[test]
    fn playing_scorch_removes_the_strongest_unit_from_the_board() {
        // P1 plays a lone Botchling (strength 4); P2 answers with Scorch.
        let mut game = Game::new(
            TestController::new(true, 2),
            Cards::monsters(&[BOTCHLING], &[]),
            Cards::northern_realms(&[SCORCH], &[]),
        );

        game.play_round();

        // Botchling was the strongest unit, so global scorch clears it.
        assert_cards(&game, Player::P1, Range::MELEE, &[]);
        assert_cards(&game, Player::P2, Range::MELEE, &[]);
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

        game.play_round();

        assert_cards(&game, Player::P1, Range::MELEE, &[]);
        assert_cards(&game, Player::P2, Range::RANGED, &[]);
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

        game.play_round();

        // The Fiend and the pirate both went to strength 6 and were scorched...
        assert_cards(&game, Player::P1, Range::MELEE, &[]);
        assert_cards(&game, Player::P2, Range::RANGED, &[]);
        // ...but the weaker Svanrige (4) is untouched.
        assert_cards(&game, Player::P2, Range::MELEE, &[(SVANRIGE, 4)]);
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

        game.play_round();

        // P2's whole melee row (all the Arachas) is scorched away...
        assert_cards(&game, Player::P2, Range::MELEE, &[]);
        // ...while Villentretenmerth survives on P1's own melee row.
        assert_cards(&game, Player::P1, Range::MELEE, &[(VILLENTRETENMERTH, 7)]);
    }

    #[test]
    fn weather_saps_a_row_below_the_scorch_threshold() {
        // P2 musters three Arachas (4 + 4 + 4 = 12, normally scorchable). P1
        // first plays Biting Frost, which drops the whole melee row to 1 each
        // (total 3), so Villentretenmerth's scorch no longer clears the
        // 10-strength threshold and the row survives. Compare with
        // `a_units_row_scorch_ability_hits_only_the_targeted_opponent_row`,
        // where the same board is scorched without weather.
        //
        // Call order is P2, P1, P1 (P2 passes with an empty hand in between);
        // P1's hand lays out as [Villentretenmerth, Biting Frost].
        let mut game = Game::new(
            ScriptedController::new(false, &[0, 1, 0]),
            Cards::northern_realms(&[BITING_FROST, VILLENTRETENMERTH], &[]),
            Cards::monsters(&[ARACHAS_1], &[ARACHAS_2, ARACHAS_3]),
        );

        game.play_round();

        // Frost kept the Arachas under the threshold, so none are scorched —
        // they survive, each sapped to strength 1 by the weather.
        assert_cards(
            &game,
            Player::P2,
            Range::MELEE,
            &[(ARACHAS_1, 1), (ARACHAS_2, 1), (ARACHAS_3, 1)],
        );
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

        game.play_round();

        // Under threshold: both units stay put.
        assert_cards(
            &game,
            Player::P2,
            Range::MELEE,
            &[(FORKTAIL, 5), (BOTCHLING, 4)],
        );
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

        game.play_round();

        // Only Triss remains — Forktail was scorched, the hero was not.
        assert_cards(&game, Player::P2, Range::MELEE, &[(TRISS, 7)]);
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

        game.play_round();

        // Exactly one Nekker is back (the medic's), proving Muster did not drag
        // the other two out of the pile.
        assert_eq!(game.board.get_ids(Player::P1, Range::MELEE).len(), 1);
        // Yennefer herself stands on the ranged row.
        assert_cards(&game, Player::P1, Range::RANGED, &[(YENNEFER, 7)]);
    }

    #[test]
    fn tight_bond_multiplies_the_weather_reduced_strength() {
        // Step through so the tight bond can be seen before and after weather.
        // P2 (tails) passes first, then P1 plays the two Catapults and rain.
        let mut game = Game::new(
            ScriptedController::new(false, &[0, 1, 0]),
            Cards::northern_realms(&[CATAPULT_1, CATAPULT_2, TORRENTIAL_RAIN], &[]),
            Cards::monsters(&[], &[]),
        );

        game.next_turn(); // P2 passes
        game.next_turn(); // Catapult 1
        game.next_turn(); // Catapult 2
        // Bonded pair before weather: 8 * 2 = 16 each.
        assert_cards(
            &game,
            Player::P1,
            Range::SIEGE,
            &[(CATAPULT_1, 16), (CATAPULT_2, 16)],
        );

        game.next_turn(); // Torrential Rain
        // Weather drops each Catapult to 1, then the tight bond doubles: 1*2 = 2.
        assert_cards(
            &game,
            Player::P1,
            Range::SIEGE,
            &[(CATAPULT_1, 2), (CATAPULT_2, 2)],
        );
    }

    #[test]
    fn tight_bond_weather_and_morale_boost_apply_in_order() {
        // Same two Catapults plus a Siege Expert (morale boost) on the rained-on
        // siege row.
        let mut game = Game::new(
            TestController::new(true, 4),
            Cards::northern_realms(
                &[CATAPULT_1, CATAPULT_2, SIEGE_EXPERT_1, TORRENTIAL_RAIN],
                &[],
            ),
            Cards::monsters(&[], &[]),
        );

        game.play_round();

        // Order is weather -> tight bond -> morale boost:
        //   weather: every unit -> 1
        //   tight bond: bonded Catapults * 2 -> 2 each (the expert isn't bonded)
        //   morale: +1 to every *other* unit -> Catapults 2 + 1 = 3; the expert
        //           grants itself nothing, staying at 1.
        assert_cards(
            &game,
            Player::P1,
            Range::SIEGE,
            &[(CATAPULT_1, 3), (CATAPULT_2, 3), (SIEGE_EXPERT_1, 1)],
        );
    }

    // --- Strength pipeline: morale boost, commander's horn and heroes ---
    //
    // Each test builds a single P1 melee row (P2 stays empty). Card play order
    // never changes the recomputed strengths, so the plays-counter controller
    // (always index 0) suffices; `select_range` resolves agile units and horn
    // to the melee row. Ability strengths (see `deck.rs`): Triss hero 7,
    // Vesemir 6, Olgierd 6 (agile, morale), Isengrim hero 10 (morale), Blue
    // Stripes 4 (tight bond), Dandelion 2 (commander's horn).

    #[test]
    fn multiple_morale_boosts_lift_every_other_unit_but_never_heroes() {
        // Two morale units (Olgierd regular, Isengrim hero) => +2 to non-morale
        // units, +1 to each morale unit (no self-boost). Heroes never gain.
        let mut game = Game::new(
            TestController::new(true, 4),
            Cards::mixed(&[OLGIERD, ISENGRIM, VESEMIR, TRISS], &[]),
            Cards::monsters(&[], &[]),
        );

        game.play_round();

        assert_cards(
            &game,
            Player::P1,
            Range::MELEE,
            &[
                (VESEMIR, 8),   // 6 + 2
                (OLGIERD, 7),   // 6 + 1 (self excluded)
                (ISENGRIM, 10), // hero, unchanged
                (TRISS, 7),     // hero, unchanged
            ],
        );
    }

    #[test]
    fn weather_does_not_affect_heroes() {
        // Step through the turns to assert the row before and after weather.
        // P2 (tails) passes first, then P1 plays Triss, Vesemir, Biting Frost.
        let mut game = Game::new(
            ScriptedController::new(false, &[0, 1, 0]),
            Cards::northern_realms(&[TRISS, VESEMIR, BITING_FROST], &[]),
            Cards::monsters(&[], &[]),
        );

        game.next_turn(); // P2 passes
        game.next_turn(); // Triss
        game.next_turn(); // Vesemir
        // Before weather: both at their base strength.
        assert_cards(&game, Player::P1, Range::MELEE, &[(TRISS, 7), (VESEMIR, 6)]);

        game.next_turn(); // Biting Frost
        // After weather: the regular unit drops to 1, the hero is untouched.
        assert_cards(&game, Player::P1, Range::MELEE, &[(TRISS, 7), (VESEMIR, 1)]);
    }

    #[test]
    fn commanders_horn_unit_doubles_the_row_except_itself() {
        // Dandelion's horn ability doubles every other unit; it never doubles
        // itself, and the hero is immune.
        let mut game = Game::new(
            ScriptedController::new(false, &[0, 1, 0]),
            Cards::northern_realms(&[TRISS, VESEMIR, DANDELION], &[]),
            Cards::monsters(&[], &[]),
        );

        game.next_turn(); // P2 passes
        game.next_turn(); // Triss
        game.next_turn(); // Vesemir
        // Before the horn lands.
        assert_cards(&game, Player::P1, Range::MELEE, &[(TRISS, 7), (VESEMIR, 6)]);

        game.next_turn(); // Dandelion (horn ability)
        assert_cards(
            &game,
            Player::P1,
            Range::MELEE,
            &[(TRISS, 7), (VESEMIR, 12), (DANDELION, 2)],
        );
    }

    #[test]
    fn commanders_horn_special_doubles_the_whole_row() {
        // The horn special doubles every regular unit; the hero is immune.
        let mut game = Game::new(
            ScriptedController::new(false, &[0, 1, 0]),
            Cards::northern_realms(&[TRISS, VESEMIR, COMMANDERS_HORN], &[]),
            Cards::monsters(&[], &[]),
        );

        game.next_turn(); // P2 passes
        game.next_turn(); // Triss
        game.next_turn(); // Vesemir
        // Before the horn lands.
        assert_cards(&game, Player::P1, Range::MELEE, &[(TRISS, 7), (VESEMIR, 6)]);

        game.next_turn(); // Commander's Horn (special)
        assert_cards(
            &game,
            Player::P1,
            Range::MELEE,
            &[(TRISS, 7), (VESEMIR, 12)],
        );
    }

    #[test]
    fn commanders_horn_unit_and_special_double_even_the_horn_unit() {
        // With the horn special present, Dandelion is doubled too.
        let mut game = Game::new(
            TestController::new(true, 4),
            Cards::northern_realms(&[TRISS, VESEMIR, DANDELION, COMMANDERS_HORN], &[]),
            Cards::monsters(&[], &[]),
        );

        game.play_round();

        assert_cards(
            &game,
            Player::P1,
            Range::MELEE,
            &[(TRISS, 7), (VESEMIR, 12), (DANDELION, 4)],
        );
    }

    #[test]
    fn hero_with_a_unit_and_a_morale_boost() {
        // One morale unit => +1 to the other unit; hero unchanged; booster no
        // self-boost.
        let mut game = Game::new(
            TestController::new(true, 3),
            Cards::northern_realms(&[TRISS, VESEMIR, OLGIERD], &[]),
            Cards::monsters(&[], &[]),
        );

        game.play_round();

        assert_cards(
            &game,
            Player::P1,
            Range::MELEE,
            &[(TRISS, 7), (VESEMIR, 7), (OLGIERD, 6)],
        );
    }

    #[test]
    fn hero_with_tight_bond_units_and_a_morale_boost() {
        // Order: tight bond (x2) then morale (+1). Bonded units: 4*2 + 1 = 9.
        let mut game = Game::new(
            TestController::new(true, 4),
            Cards::northern_realms(&[TRISS, BLUE_STRIPES_1, BLUE_STRIPES_2, OLGIERD], &[]),
            Cards::monsters(&[], &[]),
        );

        game.play_round();

        assert_cards(
            &game,
            Player::P1,
            Range::MELEE,
            &[
                (TRISS, 7),
                (BLUE_STRIPES_1, 9),
                (BLUE_STRIPES_2, 9),
                (OLGIERD, 6),
            ],
        );
    }

    #[test]
    fn hero_with_tight_bond_units_and_two_morale_boosts_including_a_hero() {
        // Two morale units (Olgierd + Isengrim). Bonded units: 4*2 + 2 = 10.
        // Olgierd: 6 + 1 = 7. Both heroes stay put.
        let mut game = Game::new(
            TestController::new(true, 5),
            Cards::mixed(
                &[TRISS, BLUE_STRIPES_1, BLUE_STRIPES_2, OLGIERD, ISENGRIM],
                &[],
            ),
            Cards::monsters(&[], &[]),
        );

        game.play_round();

        assert_cards(
            &game,
            Player::P1,
            Range::MELEE,
            &[
                (TRISS, 7),
                (BLUE_STRIPES_1, 10),
                (BLUE_STRIPES_2, 10),
                (OLGIERD, 7),
                (ISENGRIM, 10),
            ],
        );
    }

    #[test]
    fn hero_tight_bond_morale_and_a_horn_unit() {
        // Pre-horn bonded units: 4*2 + 1 = 9; Olgierd 6; Dandelion 2 + 1 = 3.
        // Dandelion's horn doubles every other unit (not itself, no special).
        let mut game = Game::new(
            TestController::new(true, 5),
            Cards::northern_realms(
                &[TRISS, BLUE_STRIPES_1, BLUE_STRIPES_2, OLGIERD, DANDELION],
                &[],
            ),
            Cards::monsters(&[], &[]),
        );

        game.play_round();

        assert_cards(
            &game,
            Player::P1,
            Range::MELEE,
            &[
                (TRISS, 7),
                (BLUE_STRIPES_1, 18),
                (BLUE_STRIPES_2, 18),
                (OLGIERD, 12),
                (DANDELION, 3),
            ],
        );
    }

    #[test]
    fn hero_tight_bond_morale_and_a_horn_special() {
        // Same pre-horn row; the horn special doubles every regular unit.
        let mut game = Game::new(
            TestController::new(true, 5),
            Cards::northern_realms(
                &[
                    TRISS,
                    BLUE_STRIPES_1,
                    BLUE_STRIPES_2,
                    OLGIERD,
                    COMMANDERS_HORN,
                ],
                &[],
            ),
            Cards::monsters(&[], &[]),
        );

        game.play_round();

        assert_cards(
            &game,
            Player::P1,
            Range::MELEE,
            &[
                (TRISS, 7),
                (BLUE_STRIPES_1, 18),
                (BLUE_STRIPES_2, 18),
                (OLGIERD, 12),
            ],
        );
    }

    #[test]
    fn hero_tight_bond_morale_and_both_horn_sources() {
        // Horn unit + horn special: everything (including Dandelion) doubles.
        let mut game = Game::new(
            TestController::new(true, 6),
            Cards::northern_realms(
                &[
                    TRISS,
                    BLUE_STRIPES_1,
                    BLUE_STRIPES_2,
                    OLGIERD,
                    DANDELION,
                    COMMANDERS_HORN,
                ],
                &[],
            ),
            Cards::monsters(&[], &[]),
        );

        game.play_round();

        assert_cards(
            &game,
            Player::P1,
            Range::MELEE,
            &[
                (TRISS, 7),
                (BLUE_STRIPES_1, 18),
                (BLUE_STRIPES_2, 18),
                (OLGIERD, 12),
                (DANDELION, 6),
            ],
        );
    }

    #[test]
    fn hero_tight_bond_two_morales_incl_hero_and_a_horn_unit() {
        // Pre-horn: bonded 4*2 + 2 = 10; Olgierd 7; Dandelion 2 + 2 = 4.
        // Dandelion horn doubles every other unit, not itself.
        let mut game = Game::new(
            TestController::new(true, 6),
            Cards::mixed(
                &[
                    TRISS,
                    BLUE_STRIPES_1,
                    BLUE_STRIPES_2,
                    OLGIERD,
                    ISENGRIM,
                    DANDELION,
                ],
                &[],
            ),
            Cards::monsters(&[], &[]),
        );

        game.play_round();

        assert_cards(
            &game,
            Player::P1,
            Range::MELEE,
            &[
                (TRISS, 7),
                (BLUE_STRIPES_1, 20),
                (BLUE_STRIPES_2, 20),
                (OLGIERD, 14),
                (ISENGRIM, 10),
                (DANDELION, 4),
            ],
        );
    }

    #[test]
    fn hero_tight_bond_two_morales_incl_hero_and_a_horn_special() {
        // Same pre-horn row; the horn special doubles every regular unit.
        let mut game = Game::new(
            TestController::new(true, 6),
            Cards::mixed(
                &[
                    TRISS,
                    BLUE_STRIPES_1,
                    BLUE_STRIPES_2,
                    OLGIERD,
                    ISENGRIM,
                    COMMANDERS_HORN,
                ],
                &[],
            ),
            Cards::monsters(&[], &[]),
        );

        game.play_round();

        assert_cards(
            &game,
            Player::P1,
            Range::MELEE,
            &[
                (TRISS, 7),
                (BLUE_STRIPES_1, 20),
                (BLUE_STRIPES_2, 20),
                (OLGIERD, 14),
                (ISENGRIM, 10),
            ],
        );
    }

    #[test]
    fn hero_tight_bond_two_morales_incl_hero_and_both_horn_sources() {
        // Horn unit + horn special: everything (including Dandelion) doubles.
        let mut game = Game::new(
            TestController::new(true, 7),
            Cards::mixed(
                &[
                    TRISS,
                    BLUE_STRIPES_1,
                    BLUE_STRIPES_2,
                    OLGIERD,
                    ISENGRIM,
                    DANDELION,
                    COMMANDERS_HORN,
                ],
                &[],
            ),
            Cards::monsters(&[], &[]),
        );

        game.play_round();

        assert_cards(
            &game,
            Player::P1,
            Range::MELEE,
            &[
                (TRISS, 7),
                (BLUE_STRIPES_1, 20),
                (BLUE_STRIPES_2, 20),
                (OLGIERD, 14),
                (ISENGRIM, 10),
                (DANDELION, 8),
            ],
        );
    }

    #[test]
    fn weather_tight_bond_morale_and_horn_apply_in_that_order() {
        // Pins the full pipeline on one row: weather -> tight bond -> morale
        // -> horn. Blue Stripes (base 4) walk the whole chain:
        //   weather: 4 -> 1
        //   tight bond (2 units): 1 * 2 = 2
        //   morale (Olgierd): 2 + 1 = 3
        //   horn (special): 3 * 2 = 6
        // Olgierd: 1 (weather) -> 1 (no bond) -> 1 (self, no boost) -> 2 (horn).
        // Triss (hero) ignores every step and stays at 7. Any reordering of the
        // four steps changes these numbers.
        // Stepped so the running strength is asserted after each condition.
        // P2 (tails) passes first, then P1 plays: Triss, Blue Stripes 1, Blue
        // Stripes 2, Olgierd, Commander's Horn, Biting Frost. The scripted
        // indices account for `pick_card`'s `swap_remove` reshuffling the hand
        // (laid out as [Triss, Olgierd, Blue 1, Blue 2, Horn, Frost]).
        let mut game = Game::new(
            ScriptedController::new(false, &[0, 2, 3, 1, 1, 0]),
            Cards::northern_realms(
                &[
                    TRISS,
                    BLUE_STRIPES_1,
                    BLUE_STRIPES_2,
                    OLGIERD,
                    COMMANDERS_HORN,
                    BITING_FROST,
                ],
                &[],
            ),
            Cards::monsters(&[], &[]),
        );

        game.next_turn(); // P2 passes

        game.next_turn(); // Triss (hero, immune to everything that follows)
        assert_cards(&game, Player::P1, Range::MELEE, &[(TRISS, 7)]);

        game.next_turn(); // Blue Stripes 1 -> bond of one, no multiplier yet
        assert_cards(
            &game,
            Player::P1,
            Range::MELEE,
            &[(TRISS, 7), (BLUE_STRIPES_1, 4)],
        );

        game.next_turn(); // Blue Stripes 2 -> tight bond doubles both: 4*2 = 8
        assert_cards(
            &game,
            Player::P1,
            Range::MELEE,
            &[(TRISS, 7), (BLUE_STRIPES_1, 8), (BLUE_STRIPES_2, 8)],
        );

        game.next_turn(); // Olgierd -> morale +1 to the pair; Olgierd at base 6
        assert_cards(
            &game,
            Player::P1,
            Range::MELEE,
            &[
                (TRISS, 7),
                (BLUE_STRIPES_1, 9),
                (BLUE_STRIPES_2, 9),
                (OLGIERD, 6),
            ],
        );

        game.next_turn(); // Commander's Horn -> doubles every regular unit
        assert_cards(
            &game,
            Player::P1,
            Range::MELEE,
            &[
                (TRISS, 7),
                (BLUE_STRIPES_1, 18),
                (BLUE_STRIPES_2, 18),
                (OLGIERD, 12),
            ],
        );

        game.next_turn(); // Biting Frost -> whole chain recomputed under weather
        // weather 4->1, bond *2 = 2, morale +1 = 3, horn *2 = 6 for the pair;
        // Olgierd 1 -> horn 2; Triss (hero) stays 7 throughout.
        assert_cards(
            &game,
            Player::P1,
            Range::MELEE,
            &[
                (TRISS, 7),
                (BLUE_STRIPES_1, 6),
                (BLUE_STRIPES_2, 6),
                (OLGIERD, 2),
            ],
        );
    }

    // --- Decoy returns a unit from the board to the hand ---

    #[test]
    fn decoy_returns_a_unit_from_the_board_to_the_hand() {
        // P1 plays Vesemir, then Decoy targeting it (melee, index 0).
        let mut game = Game::new(
            TestController::new(true, 2),
            Cards::northern_realms(&[VESEMIR, DECOY], &[]),
            Cards::monsters(&[], &[]),
        );

        game.play_round();

        // Vesemir left the board and is back in hand; Decoy itself is consumed.
        assert_cards(&game, Player::P1, Range::MELEE, &[]);
        assert_hand(&game.p1, &[VESEMIR]);
    }

    #[test]
    fn decoy_returns_a_unit_with_an_ability_and_undoes_its_effect() {
        // P1 plays Vesemir, then Dandelion (commander's horn ability), which
        // doubles Vesemir to 12. Decoy then pulls Dandelion (melee, index 1)
        // back to hand, so its horn effect is undone and Vesemir returns to 6.
        let mut game = Game::new(
            ScriptedController::new(true, &[0, 1, 0]).with_decoy_target((Range::MELEE, 1)),
            Cards::northern_realms(&[VESEMIR, DANDELION, DECOY], &[]),
            Cards::monsters(&[], &[]),
        );

        game.play_round();

        // Horn gone: Vesemir back to base strength, Dandelion back in hand.
        assert_cards(&game, Player::P1, Range::MELEE, &[(VESEMIR, 6)]);
        assert_hand(&game.p1, &[DANDELION]);
    }

    #[test]
    fn decoy_undoes_a_morale_boost() {
        // Vesemir (6) is lifted to 7 by Olgierd's morale boost. Decoy pulls
        // Olgierd (melee, index 1) back to hand, so the boost is undone and
        // Vesemir returns to 6.
        let mut game = Game::new(
            ScriptedController::new(true, &[0, 1, 0]).with_decoy_target((Range::MELEE, 1)),
            Cards::northern_realms(&[VESEMIR, OLGIERD, DECOY], &[]),
            Cards::monsters(&[], &[]),
        );

        game.play_round();

        assert_cards(&game, Player::P1, Range::MELEE, &[(VESEMIR, 6)]);
        assert_hand(&game.p1, &[OLGIERD]);
    }

    #[test]
    fn decoy_undoes_a_tight_bond() {
        // Two bonded Blue Stripes are 4 * 2 = 8 each. Decoy pulls the second
        // one (melee, index 1) back to hand, dropping the bond count to 1, so
        // the survivor returns to its base 4.
        let mut game = Game::new(
            ScriptedController::new(true, &[0, 1, 0]).with_decoy_target((Range::MELEE, 1)),
            Cards::northern_realms(&[BLUE_STRIPES_1, BLUE_STRIPES_2, DECOY], &[]),
            Cards::monsters(&[], &[]),
        );

        game.play_round();

        assert_cards(&game, Player::P1, Range::MELEE, &[(BLUE_STRIPES_1, 4)]);
        assert_hand(&game.p1, &[BLUE_STRIPES_2]);
    }

    // --- Clear Weather on the ranged row ---

    #[test]
    fn clear_weather_does_nothing_when_there_is_no_weather() {
        // Keira (ranged, 5) sits under no weather; Clear Weather is a no-op.
        let mut game = Game::new(
            TestController::new(true, 2),
            Cards::northern_realms(&[KEIRA_METZ, CLEAR_WEATHER], &[]),
            Cards::monsters(&[], &[]),
        );

        game.play_round();

        assert_cards(&game, Player::P1, Range::RANGED, &[(KEIRA_METZ, 5)]);
    }

    #[test]
    fn clear_weather_undoes_skellige_storm() {
        // Skellige Storm saps the ranged row to 1; Clear Weather lifts it back.
        // Play order (Keira, Storm, Clear) matters, so use the scripted
        // controller.
        let mut game = Game::new(
            ScriptedController::new(true, &[0, 1, 0]),
            Cards::northern_realms(&[KEIRA_METZ, SKELLIGE_STORM, CLEAR_WEATHER], &[]),
            Cards::monsters(&[], &[]),
        );

        game.play_round();

        assert_cards(&game, Player::P1, Range::RANGED, &[(KEIRA_METZ, 5)]);
    }

    #[test]
    fn skellige_storm_hits_both_ranged_and_siege_tight_bond_rows() {
        // Skellige Storm affects the ranged and siege rows. Bonded pairs on
        // each: weather drops them to 1, then the tight bond doubles: 1 * 2 = 2.
        let mut game = Game::new(
            TestController::new(true, 5),
            Cards::northern_realms(
                &[
                    DRAGON_HUNTER_1,
                    DRAGON_HUNTER_2,
                    CATAPULT_1,
                    CATAPULT_2,
                    SKELLIGE_STORM,
                ],
                &[],
            ),
            Cards::monsters(&[], &[]),
        );

        game.play_round();

        assert_cards(
            &game,
            Player::P1,
            Range::RANGED,
            &[(DRAGON_HUNTER_1, 2), (DRAGON_HUNTER_2, 2)],
        );
        assert_cards(
            &game,
            Player::P1,
            Range::SIEGE,
            &[(CATAPULT_1, 2), (CATAPULT_2, 2)],
        );
    }

    #[test]
    fn the_same_weather_played_twice_does_nothing_extra() {
        // P1 plays Vesemir then Biting Frost; P2 plays a second Biting Frost.
        // The row is already under frost, so the second copy is a no-op — the
        // unit stays at 1 rather than stacking any further.
        let mut game = Game::new(
            TestController::new(true, 3),
            Cards::northern_realms(&[VESEMIR, BITING_FROST], &[]),
            Cards::monsters(&[BITING_FROST], &[]),
        );

        game.next_turn(); // P1 plays Vesemir
        assert_cards(&game, Player::P1, Range::MELEE, &[(VESEMIR, 6)]);

        game.next_turn(); // P2 plays the first Biting Frost -> Vesemir drops to 1
        assert_cards(&game, Player::P1, Range::MELEE, &[(VESEMIR, 1)]);

        game.next_turn(); // P1 plays a second Biting Frost -> already frosted, no change
        assert_cards(&game, Player::P1, Range::MELEE, &[(VESEMIR, 1)]);
    }

    // --- Spy: lands on the opponent, draws two for the caster ---

    #[test]
    fn spy_lands_on_the_opponent_and_draws_two_cards() {
        // P1 plays Prince Stennis (spy). The unit is placed on P2's side and P1
        // draws the two cards waiting in its deck.
        let mut game = Game::new(
            TestController::new(true, 1),
            Cards::northern_realms(&[PRINCE_STENNIS], &[VESEMIR, ZOLTAN]),
            Cards::monsters(&[], &[]),
        );

        game.play_round();

        // The spy sits on the opponent's melee row...
        assert_cards(&game, Player::P2, Range::MELEE, &[(PRINCE_STENNIS, 5)]);
        // ...and P1 drew both deck cards into hand.
        assert_hand(&game.p1, &[VESEMIR, ZOLTAN]);
    }

    // --- Medic: restore a unit from the pile and chain its ability ---

    #[test]
    fn medic_restores_a_unit_from_the_pile() {
        // P1 plays Botchling, scorches it into the pile, then plays Birna Bran
        // (medic), which brings Botchling back to the board.
        // Botchling (monsters) + Birna Bran (skellige) are cross-faction, so
        // this hand needs `mixed`, which also preserves the input play order.
        let mut game = Game::new(
            ScriptedController::new(false, &[0, 1, 0]),
            Cards::mixed(&[BOTCHLING, SCORCH, BIRNA_BRAN], &[]),
            Cards::monsters(&[], &[]),
        );

        game.play_round();

        assert_cards(
            &game,
            Player::P1,
            Range::MELEE,
            &[(BIRNA_BRAN, 2), (BOTCHLING, 4)],
        );
    }

    #[test]
    fn medic_chains_when_it_restores_another_medic() {
        // Two Etolian Archers (medics) are scorched into the pile. Dun Banner
        // Medic restores the first, whose own medic restores the second — so
        // both come back off a single medic play.
        let mut game = Game::new(
            ScriptedController::new(false, &[0, 1, 1, 0]),
            Cards::mixed(
                &[
                    ETOLIAN_ARCHERS_1,
                    ETOLIAN_ARCHERS_2,
                    SCORCH,
                    DUN_BANNER_MEDIC,
                ],
                &[],
            ),
            Cards::monsters(&[], &[]),
        );

        game.play_round();

        assert_cards(
            &game,
            Player::P1,
            Range::RANGED,
            &[(ETOLIAN_ARCHERS_1, 1), (ETOLIAN_ARCHERS_2, 1)],
        );
        assert_cards(&game, Player::P1, Range::SIEGE, &[(DUN_BANNER_MEDIC, 5)]);
    }

    #[test]
    fn medic_restores_a_tight_bond_partner_and_completes_the_bond() {
        // Blue Stripes 2 is scorched into the pile; Blue Stripes 1 is then
        // played alone (bond of one, strength 4). Dun Banner Medic restores
        // Blue Stripes 2, forming the bond so both become 4 * 2 = 8.
        // `northern_realms` lays the hand out as [faction…, special], i.e.
        // [Blue 2, Blue 1, Dun, Scorch]; the script plays Blue 2, Scorch,
        // Blue 1, Dun.
        let mut game = Game::new(
            ScriptedController::new(false, &[0, 0, 1, 0]),
            Cards::northern_realms(
                &[BLUE_STRIPES_2, SCORCH, BLUE_STRIPES_1, DUN_BANNER_MEDIC],
                &[],
            ),
            Cards::monsters(&[], &[]),
        );

        game.next_turn(); // P2 passes
        game.next_turn(); // Blue Stripes 2 alone -> bond of one, strength 4
        assert_cards(&game, Player::P1, Range::MELEE, &[(BLUE_STRIPES_2, 4)]);

        game.next_turn(); // Scorch -> Blue Stripes 2 (4) to pile
        assert_cards(&game, Player::P1, Range::MELEE, &[]);

        game.next_turn(); // Blue Stripes 1 alone -> bond of one, strength 4
        assert_cards(&game, Player::P1, Range::MELEE, &[(BLUE_STRIPES_1, 4)]);

        game.next_turn(); // medic restores Blue Stripes 2 -> bond of two: 4*2 = 8
        assert_cards(
            &game,
            Player::P1,
            Range::MELEE,
            &[(BLUE_STRIPES_1, 8), (BLUE_STRIPES_2, 8)],
        );
        assert_cards(&game, Player::P1, Range::SIEGE, &[(DUN_BANNER_MEDIC, 5)]);
    }

    #[test]
    fn medic_restores_a_row_scorch_unit_which_fires_again() {
        // Villentretenmerth (melee scorch) is scorched into P1's pile, then
        // restored by a medic once P2 has a scorchable row. Call order is P1,
        // P2, P1, P1 (P2 passes with an empty hand before the medic).
        // Hand lays out as [Villentretenmerth, Dun, Scorch]; the P1 plays are
        // Villentretenmerth, Scorch, Dun (all index 0 after each swap_remove),
        // interleaved with P2's Arachas.
        let mut game = Game::new(
            ScriptedController::new(true, &[0, 0, 0, 0]),
            Cards::northern_realms(&[VILLENTRETENMERTH, SCORCH, DUN_BANNER_MEDIC], &[]),
            Cards::monsters(&[ARACHAS_1], &[ARACHAS_2, ARACHAS_3]),
        );

        game.next_turn(); // P1 plays Villentretenmerth (P2 empty -> no scorch yet)
        assert_cards(&game, Player::P1, Range::MELEE, &[(VILLENTRETENMERTH, 7)]);

        game.next_turn(); // P2 musters three Arachas
        assert_cards(
            &game,
            Player::P2,
            Range::MELEE,
            &[(ARACHAS_1, 4), (ARACHAS_2, 4), (ARACHAS_3, 4)],
        );

        game.next_turn(); // P1 scorches: Villentretenmerth (7) is the max -> pile
        assert_cards(&game, Player::P1, Range::MELEE, &[]);
        // Arachas (4) survive that scorch.
        assert_cards(
            &game,
            Player::P2,
            Range::MELEE,
            &[(ARACHAS_1, 4), (ARACHAS_2, 4), (ARACHAS_3, 4)],
        );

        game.next_turn(); // P2 passes (empty hand)
        game.next_turn(); // P1's medic restores Villentretenmerth -> it rescorches
        assert_cards(&game, Player::P2, Range::MELEE, &[]); // Arachas row cleared
        assert_cards(&game, Player::P1, Range::MELEE, &[(VILLENTRETENMERTH, 7)]);
    }

    #[test]
    fn medic_restores_a_global_scorch_unit_which_fires_again() {
        // Clan Dimun Pirate (global scorch) self-scorches into P1's pile on the
        // first play. P2 then fields a Fiend (6). The medic restores the pirate,
        // whose global scorch clears the board-wide max (6) — the Fiend and the
        // pirate both go.
        let mut game = Game::new(
            TestController::new(true, 3),
            Cards::skellige(&[CLAN_DIMUN_PIRATE, BIRNA_BRAN], &[]),
            Cards::monsters(&[FIEND], &[]),
        );

        game.next_turn(); // P1 pirate -> global scorch removes itself (max 6)
        assert_cards(&game, Player::P1, Range::RANGED, &[]);

        game.next_turn(); // P2 plays Fiend
        assert_cards(&game, Player::P2, Range::MELEE, &[(FIEND, 6)]);

        game.next_turn(); // P1 Birna Bran restores the pirate -> rescorch max 6
        assert_cards(&game, Player::P2, Range::MELEE, &[]); // Fiend gone
        assert_cards(&game, Player::P1, Range::RANGED, &[]); // pirate gone again
        assert_cards(&game, Player::P1, Range::MELEE, &[(BIRNA_BRAN, 2)]); // medic stays
    }

    #[test]
    fn medic_restores_a_morale_boost_unit() {
        // Olgierd (morale) is scorched into the pile, then Vesemir is played
        // alone. The medic restores Olgierd, whose morale lifts Vesemir 6 -> 7.
        // Play order: Olgierd, Scorch, Vesemir, Dun Banner Medic.
        let mut game = Game::new(
            ScriptedController::new(false, &[0, 0, 1, 0]),
            Cards::northern_realms(&[OLGIERD, VESEMIR, DUN_BANNER_MEDIC, SCORCH], &[]),
            Cards::monsters(&[], &[]),
        );

        game.next_turn(); // P2 passes
        game.next_turn(); // Olgierd (agile -> melee)
        assert_cards(&game, Player::P1, Range::MELEE, &[(OLGIERD, 6)]);

        game.next_turn(); // Scorch -> Olgierd (6) to pile
        assert_cards(&game, Player::P1, Range::MELEE, &[]);

        game.next_turn(); // Vesemir alone, no boost yet
        assert_cards(&game, Player::P1, Range::MELEE, &[(VESEMIR, 6)]);

        game.next_turn(); // medic restores Olgierd -> morale lifts Vesemir to 7
        assert_cards(
            &game,
            Player::P1,
            Range::MELEE,
            &[(VESEMIR, 7), (OLGIERD, 6)],
        );
        assert_cards(&game, Player::P1, Range::SIEGE, &[(DUN_BANNER_MEDIC, 5)]);
    }

    #[test]
    fn medic_restores_a_spy_which_lands_on_the_opponent() {
        // P2 spies P1 (Prince Stennis lands on P1's melee). P1 scorches it into
        // its own pile, then the medic restores it — replaying it as a spy sends
        // it to P2 and draws two for P1. Call order: P2, P1, (P2 pass), P1.
        let mut game = Game::new(
            ScriptedController::new(false, &[0, 1, 0]),
            Cards::northern_realms(&[SCORCH, DUN_BANNER_MEDIC], &[VESEMIR, ZOLTAN]),
            Cards::northern_realms(&[PRINCE_STENNIS], &[]),
        );

        game.next_turn(); // P2 spy -> lands on P1's melee
        assert_cards(&game, Player::P1, Range::MELEE, &[(PRINCE_STENNIS, 5)]);

        game.next_turn(); // P1 scorches the spy into its own pile
        assert_cards(&game, Player::P1, Range::MELEE, &[]);

        game.next_turn(); // P2 passes (empty hand)
        game.next_turn(); // P1 medic restores the spy -> back to P2, draw two
        assert_cards(&game, Player::P2, Range::MELEE, &[(PRINCE_STENNIS, 5)]);
        assert_cards(&game, Player::P1, Range::SIEGE, &[(DUN_BANNER_MEDIC, 5)]);
        assert_hand(&game.p1, &[VESEMIR, ZOLTAN]);
    }

    // --- Spy interacting with weather and scorch ---

    #[test]
    fn a_spy_on_the_opponent_is_hit_by_weather() {
        // The spy strengthens the opponent, so weather saps it like any unit.
        let mut game = Game::new(
            TestController::new(true, 2),
            Cards::northern_realms(&[PRINCE_STENNIS, BITING_FROST], &[]),
            Cards::monsters(&[], &[]),
        );

        game.next_turn(); // P1 spy -> P2 melee at 5
        assert_cards(&game, Player::P2, Range::MELEE, &[(PRINCE_STENNIS, 5)]);

        game.next_turn(); // P2 passes
        game.next_turn(); // P1 Biting Frost -> spy drops to 1
        assert_cards(&game, Player::P2, Range::MELEE, &[(PRINCE_STENNIS, 1)]);
    }

    #[test]
    fn scorch_kills_the_spy_when_it_is_the_strongest_unit() {
        // The spy (5) is stronger than P1's own Redanian (1); global scorch
        // removes the spy from the opponent's row while the weaker unit stays.
        // Play order: Prince Stennis, Redanian, Scorch.
        let mut game = Game::new(
            ScriptedController::new(true, &[0, 1, 0]),
            Cards::northern_realms(&[PRINCE_STENNIS, REDANIAN_SOLDIER_1, SCORCH], &[]),
            Cards::monsters(&[], &[]),
        );

        game.next_turn(); // P1 spy -> P2 melee at 5
        assert_cards(&game, Player::P2, Range::MELEE, &[(PRINCE_STENNIS, 5)]);

        game.next_turn(); // P2 passes
        game.next_turn(); // P1 Redanian on its own melee
        assert_cards(&game, Player::P1, Range::MELEE, &[(REDANIAN_SOLDIER_1, 1)]);

        game.next_turn(); // P1 scorches: spy (5) is the max and dies
        assert_cards(&game, Player::P2, Range::MELEE, &[]);
        assert_cards(&game, Player::P1, Range::MELEE, &[(REDANIAN_SOLDIER_1, 1)]);
    }

    // --- Cerys musters Drummond Shieldmaidens (tight bond); muster reads only
    // hand + deck, never the pile. Each mustered maiden is 4 * (count on row).

    #[test]
    fn cerys_musters_shieldmaidens_all_from_hand() {
        // Cerys + all three maidens in hand -> bond of three: 4 * 3 = 12 each.
        let mut game = Game::new(
            TestController::new(true, 1),
            Cards::skellige(
                &[
                    CERYS,
                    DRUMMOND_SHIELDMAIDEN_1,
                    DRUMMOND_SHIELDMAIDEN_2,
                    DRUMMOND_SHIELDMAIDEN_3,
                ],
                &[],
            ),
            Cards::monsters(&[], &[]),
        );

        game.play_round();

        assert_cards(
            &game,
            Player::P1,
            Range::MELEE,
            &[
                (CERYS, 10),
                (DRUMMOND_SHIELDMAIDEN_1, 12),
                (DRUMMOND_SHIELDMAIDEN_2, 12),
                (DRUMMOND_SHIELDMAIDEN_3, 12),
            ],
        );
    }

    #[test]
    fn cerys_musters_shieldmaidens_from_hand_and_deck() {
        // One maiden in hand, two in the deck -> still all three: 12 each.
        let mut game = Game::new(
            TestController::new(true, 1),
            Cards::skellige(
                &[CERYS, DRUMMOND_SHIELDMAIDEN_1],
                &[DRUMMOND_SHIELDMAIDEN_2, DRUMMOND_SHIELDMAIDEN_3],
            ),
            Cards::monsters(&[], &[]),
        );

        game.play_round();

        assert_cards(
            &game,
            Player::P1,
            Range::MELEE,
            &[
                (CERYS, 10),
                (DRUMMOND_SHIELDMAIDEN_1, 12),
                (DRUMMOND_SHIELDMAIDEN_2, 12),
                (DRUMMOND_SHIELDMAIDEN_3, 12),
            ],
        );
    }

    #[test]
    fn cerys_musters_shieldmaidens_all_from_deck() {
        // All three maidens wait in the deck -> all three mustered: 12 each.
        let mut game = Game::new(
            TestController::new(true, 1),
            Cards::skellige(
                &[CERYS],
                &[
                    DRUMMOND_SHIELDMAIDEN_1,
                    DRUMMOND_SHIELDMAIDEN_2,
                    DRUMMOND_SHIELDMAIDEN_3,
                ],
            ),
            Cards::monsters(&[], &[]),
        );

        game.play_round();

        assert_cards(
            &game,
            Player::P1,
            Range::MELEE,
            &[
                (CERYS, 10),
                (DRUMMOND_SHIELDMAIDEN_1, 12),
                (DRUMMOND_SHIELDMAIDEN_2, 12),
                (DRUMMOND_SHIELDMAIDEN_3, 12),
            ],
        );
    }

    #[test]
    fn cerys_ignores_the_maiden_sitting_in_the_pile() {
        // Maiden 1 is scorched into the pile; maidens 2 and 3 stay in hand.
        // Cerys musters only the hand copies -> bond of two: 4 * 2 = 8 each.
        // Play order: maiden 1, Scorch, Cerys (maidens 2 & 3 pulled by muster).
        let mut game = Game::new(
            ScriptedController::new(false, &[0, 0, 1]),
            Cards::skellige(
                &[
                    DRUMMOND_SHIELDMAIDEN_1,
                    SCORCH,
                    CERYS,
                    DRUMMOND_SHIELDMAIDEN_2,
                    DRUMMOND_SHIELDMAIDEN_3,
                ],
                &[],
            ),
            Cards::monsters(&[], &[]),
        );

        game.next_turn(); // P2 passes
        game.next_turn(); // Maiden 1 alone -> bond of one, strength 4
        assert_cards(
            &game,
            Player::P1,
            Range::MELEE,
            &[(DRUMMOND_SHIELDMAIDEN_1, 4)],
        );

        game.next_turn(); // Scorch -> Maiden 1 to the pile
        assert_cards(&game, Player::P1, Range::MELEE, &[]);

        game.next_turn(); // Cerys musters maidens 2 & 3 from hand (pile ignored)
        assert_cards(
            &game,
            Player::P1,
            Range::MELEE,
            &[
                (CERYS, 10),
                (DRUMMOND_SHIELDMAIDEN_2, 8),
                (DRUMMOND_SHIELDMAIDEN_3, 8),
            ],
        );
    }

    #[test]
    fn cerys_musters_only_the_deck_maiden_when_two_are_in_the_pile() {
        // Maidens 1 and 2 are scorched into the pile; maiden 3 waits in the
        // deck. Cerys musters only maiden 3 -> bond of one, strength 4.
        // Play order: maiden 1, maiden 2, Scorch, Cerys.
        let mut game = Game::new(
            ScriptedController::new(false, &[0, 1, 0, 0]),
            Cards::skellige(
                &[
                    DRUMMOND_SHIELDMAIDEN_1,
                    DRUMMOND_SHIELDMAIDEN_2,
                    SCORCH,
                    CERYS,
                ],
                &[DRUMMOND_SHIELDMAIDEN_3],
            ),
            Cards::monsters(&[], &[]),
        );

        game.next_turn(); // P2 passes
        game.next_turn(); // Maiden 1
        game.next_turn(); // Maiden 2 -> bond of two: 8 each
        assert_cards(
            &game,
            Player::P1,
            Range::MELEE,
            &[(DRUMMOND_SHIELDMAIDEN_1, 8), (DRUMMOND_SHIELDMAIDEN_2, 8)],
        );

        game.next_turn(); // Scorch -> both maidens to the pile
        assert_cards(&game, Player::P1, Range::MELEE, &[]);

        game.next_turn(); // Cerys musters only maiden 3 from the deck
        assert_cards(
            &game,
            Player::P1,
            Range::MELEE,
            &[(CERYS, 10), (DRUMMOND_SHIELDMAIDEN_3, 4)],
        );
    }

    #[test]
    fn cerys_musters_nothing_when_all_maidens_are_in_the_pile() {
        // All three maidens are scorched into the pile, so Cerys musters none —
        // only Cerys lands. Play order: maidens 1, 2, 3, Scorch, Cerys.
        let mut game = Game::new(
            ScriptedController::new(false, &[0, 1, 2, 0, 0]),
            Cards::skellige(
                &[
                    DRUMMOND_SHIELDMAIDEN_1,
                    DRUMMOND_SHIELDMAIDEN_2,
                    DRUMMOND_SHIELDMAIDEN_3,
                    SCORCH,
                    CERYS,
                ],
                &[],
            ),
            Cards::monsters(&[], &[]),
        );

        game.next_turn(); // P2 passes
        game.next_turn(); // Maiden 1
        game.next_turn(); // Maiden 2
        game.next_turn(); // Maiden 3 -> bond of three: 12 each
        assert_cards(
            &game,
            Player::P1,
            Range::MELEE,
            &[
                (DRUMMOND_SHIELDMAIDEN_1, 12),
                (DRUMMOND_SHIELDMAIDEN_2, 12),
                (DRUMMOND_SHIELDMAIDEN_3, 12),
            ],
        );

        game.next_turn(); // Scorch -> all three to the pile
        assert_cards(&game, Player::P1, Range::MELEE, &[]);

        game.next_turn(); // Cerys musters nothing
        assert_cards(&game, Player::P1, Range::MELEE, &[(CERYS, 10)]);
    }

    #[test]
    fn a_maiden_played_before_cerys_joins_the_mustered_bond() {
        // Maiden 1 is on the board first; Cerys then musters maidens 2 & 3 from
        // the deck, forming a bond of three so all become 12.
        let mut game = Game::new(
            TestController::new(true, 2),
            Cards::skellige(
                &[DRUMMOND_SHIELDMAIDEN_1, CERYS],
                &[DRUMMOND_SHIELDMAIDEN_2, DRUMMOND_SHIELDMAIDEN_3],
            ),
            Cards::monsters(&[], &[]),
        );

        game.next_turn(); // P1 plays Maiden 1 -> bond of one, strength 4
        assert_cards(
            &game,
            Player::P1,
            Range::MELEE,
            &[(DRUMMOND_SHIELDMAIDEN_1, 4)],
        );

        game.next_turn(); // P2 passes
        game.next_turn(); // P1 plays Cerys -> musters 2 & 3 -> bond of three: 12
        assert_cards(
            &game,
            Player::P1,
            Range::MELEE,
            &[
                (CERYS, 10),
                (DRUMMOND_SHIELDMAIDEN_1, 12),
                (DRUMMOND_SHIELDMAIDEN_2, 12),
                (DRUMMOND_SHIELDMAIDEN_3, 12),
            ],
        );
    }

    // --- Summon: leaving the board summons the carried unit (round end,
    // scorch, decoy) ---

    #[test]
    fn a_summon_fires_when_the_round_ends() {
        let mut game = Game::new(
            TestController::new(true, 1),
            Cards::skellige(&[KAMBI], &[]),
            Cards::monsters(&[], &[]),
        );

        game.play_round(); // round 1: P1 plays Kambi -> still a strength-0 summon
        assert_cards(&game, Player::P1, Range::MELEE, &[(KAMBI, 0)]);

        game.end_round(); // Kambi leaves for the pile, Hemdall (11) is summoned
        assert_cards(&game, Player::P1, Range::MELEE, &[(HEMDALL, 11)]);
        assert_pile(&game.p1, &[KAMBI]);
    }

    #[test]
    fn a_summon_fires_when_scorched() {
        // Kambi (strength 0) is the whole board, so global Scorch removes it —
        // sending Kambi to the pile and summoning Hemdall in its place.
        let mut game = Game::new(
            TestController::new(true, 2),
            Cards::skellige(&[KAMBI, SCORCH], &[]),
            Cards::monsters(&[], &[]),
        );

        game.play_round();

        assert_cards(&game, Player::P1, Range::MELEE, &[(HEMDALL, 11)]);
        assert_pile(&game.p1, &[KAMBI]);
    }

    #[test]
    fn a_summon_fires_when_decoyed() {
        // Decoy pulls Kambi back to hand; leaving the board still summons
        // Hemdall.
        let mut game = Game::new(
            TestController::new(true, 2),
            Cards::skellige(&[KAMBI, DECOY], &[]),
            Cards::monsters(&[], &[]),
        );

        game.play_round();

        assert_cards(&game, Player::P1, Range::MELEE, &[(HEMDALL, 11)]);
        assert_hand(&game.p1, &[KAMBI]);
    }

    #[test]
    fn ending_a_round_clears_units_to_their_owners_pile() {
        let mut game = Game::new(
            TestController::new(true, 1),
            Cards::monsters(&[BOTCHLING], &[]),
            Cards::monsters(&[], &[]),
        );

        game.play_round();
        assert_cards(&game, Player::P1, Range::MELEE, &[(BOTCHLING, 4)]);

        game.end_round();
        // Board is empty and Botchling now sits in P1's discard pile.
        assert_cards(&game, Player::P1, Range::MELEE, &[]);
        assert_pile(&game.p1, &[BOTCHLING]);
    }

    #[test]
    fn a_game_ends_when_a_player_runs_out_of_gems() {
        // P1 wins round 1 (Botchling vs empty), costing P2 a gem. Round 2 both
        // boards are empty, so the tie costs both a gem and knocks P2 out.
        let mut game = Game::new(
            TestController::new(true, 1),
            Cards::monsters(&[BOTCHLING], &[]),
            Cards::monsters(&[], &[]),
        );

        game.start();

        assert_eq!(game.gems, [1, 0]); // P2 is out, P1 survives with a gem
    }

    // --- Berserker: transforms into its beast when a Mardroeme hits the row ---

    #[test]
    fn a_mardroeme_special_transforms_a_berserker() {
        let mut game = Game::new(
            TestController::new(true, 2),
            Cards::skellige(&[BERSERKER, MARDROME], &[]),
            Cards::monsters(&[], &[]),
        );

        game.next_turn(); // P1 plays the Berserker (strength 4)
        assert_cards(&game, Player::P1, Range::MELEE, &[(BERSERKER, 4)]);

        game.next_turn(); // P2 passes
        game.next_turn(); // P1 plays Mardroeme -> Berserker becomes Vildkaarl (14)
        assert_cards(&game, Player::P1, Range::MELEE, &[(VILDKAARL, 14)]);
        assert_boost(&game, Player::P1, Range::MELEE, Some(Special::Mardrome));
    }

    #[test]
    fn a_berserker_played_onto_a_mardroemed_row_transforms_at_once() {
        // Mardroeme lands first (row boost, no units yet); the Berserker played
        // afterwards transforms immediately. Play order: Mardroeme, Berserker.
        let mut game = Game::new(
            ScriptedController::new(false, &[1, 0]),
            Cards::skellige(&[BERSERKER, MARDROME], &[]),
            Cards::monsters(&[], &[]),
        );

        game.next_turn(); // P2 passes
        game.next_turn(); // Mardroeme special -> melee row boosted, still empty
        assert_cards(&game, Player::P1, Range::MELEE, &[]);
        assert_boost(&game, Player::P1, Range::MELEE, Some(Special::Mardrome));

        game.next_turn(); // Berserker -> transforms on arrival into Vildkaarl (14)
        assert_cards(&game, Player::P1, Range::MELEE, &[(VILDKAARL, 14)]);
    }

    #[test]
    fn a_transformed_berserker_never_leaves_its_initial_form_in_the_pile() {
        let mut game = Game::new(
            TestController::new(true, 2),
            Cards::skellige(&[BERSERKER, MARDROME], &[]),
            Cards::monsters(&[], &[]),
        );

        game.play_round(); // Berserker + Mardroeme -> Vildkaarl on the board
        assert_cards(&game, Player::P1, Range::MELEE, &[(VILDKAARL, 14)]);

        game.end_round();
        // Only the transformed Vildkaarl reaches the pile — never the Berserker.
        assert_cards(&game, Player::P1, Range::MELEE, &[]);
        assert_pile(&game.p1, &[VILDKAARL]);
    }

    #[test]
    fn ermions_mardroeme_ability_transforms_a_young_berserker() {
        // Ermion carries the Mardroeme ability, so playing him onto the ranged
        // row transforms the Young Berserker there.
        let mut game = Game::new(
            TestController::new(true, 2),
            Cards::skellige(&[YOUNG_BERSERKER_1, ERMION], &[]),
            Cards::monsters(&[], &[]),
        );

        game.next_turn(); // Young Berserker (strength 2)
        assert_cards(&game, Player::P1, Range::RANGED, &[(YOUNG_BERSERKER_1, 2)]);

        game.next_turn(); // P2 passes
        game.next_turn(); // Ermion -> Young Berserker becomes Young Vildkaarl (8)
        assert_cards(
            &game,
            Player::P1,
            Range::RANGED,
            &[(ERMION, 8), (YOUNG_VILDKAARL, 8)],
        );
    }

    #[test]
    fn transformed_young_berserkers_share_a_tight_bond() {
        // Three Young Berserkers become three Young Vildkaarls, which carry a
        // tight bond: 8 * 3 = 24 each.
        let mut game = Game::new(
            TestController::new(true, 4),
            Cards::skellige(
                &[
                    YOUNG_BERSERKER_1,
                    YOUNG_BERSERKER_2,
                    YOUNG_BERSERKER_3,
                    ERMION,
                ],
                &[],
            ),
            Cards::monsters(&[], &[]),
        );

        game.play_round();

        assert_cards(
            &game,
            Player::P1,
            Range::RANGED,
            &[
                (ERMION, 8),
                (YOUNG_VILDKAARL, 24),
                (YOUNG_VILDKAARL, 24),
                (YOUNG_VILDKAARL, 24),
            ],
        );
    }

    #[test]
    fn ermion_and_a_mardroeme_special_together_transform_once() {
        // Both Mardroeme sources hit the ranged row. Ermion already transforms
        // the Young Berserker; adding the Mardroeme special changes nothing —
        // the outcome matches a single source.
        let mut game = Game::new(
            ScriptedController::new(false, &[0, 1, 0]).with_range(Range::RANGED),
            Cards::skellige(&[YOUNG_BERSERKER_1, ERMION, MARDROME], &[]),
            Cards::monsters(&[], &[]),
        );

        game.next_turn(); // P2 passes
        game.next_turn(); // Young Berserker
        game.next_turn(); // Ermion -> transforms it into Young Vildkaarl (8)
        assert_cards(
            &game,
            Player::P1,
            Range::RANGED,
            &[(ERMION, 8), (YOUNG_VILDKAARL, 8)],
        );

        game.next_turn(); // Mardroeme special on the same row -> no further change
        assert_cards(
            &game,
            Player::P1,
            Range::RANGED,
            &[(ERMION, 8), (YOUNG_VILDKAARL, 8)],
        );
    }

    // --- Agile units are placed on the melee or ranged row of the caller's
    // choosing ---

    #[test]
    fn an_agile_unit_can_be_placed_on_the_melee_row() {
        let mut game = Game::new(
            TestController::new(true, 1), // select_range -> MELEE
            Cards::skoiatael(&[BARCLAY_ELS], &[]),
            Cards::monsters(&[], &[]),
        );

        game.play_round();

        assert_cards(&game, Player::P1, Range::MELEE, &[(BARCLAY_ELS, 6)]);
        assert_cards(&game, Player::P1, Range::RANGED, &[]);
    }

    #[test]
    fn an_agile_unit_can_be_placed_on_the_ranged_row() {
        let mut game = Game::new(
            ScriptedController::new(true, &[0]).with_range(Range::RANGED),
            Cards::skoiatael(&[BARCLAY_ELS], &[]),
            Cards::monsters(&[], &[]),
        );

        game.play_round();

        assert_cards(&game, Player::P1, Range::RANGED, &[(BARCLAY_ELS, 6)]);
        assert_cards(&game, Player::P1, Range::MELEE, &[]);
    }

    #[test]
    fn an_agile_units_ability_applies_to_the_row_it_joins() {
        // Olgierd (agile, morale) placed on the ranged row lifts Ida there.
        let mut game = Game::new(
            ScriptedController::new(true, &[0, 0]).with_range(Range::RANGED),
            Cards::skoiatael(&[OLGIERD, IDA_EMEAN], &[]),
            Cards::monsters(&[], &[]),
        );

        game.play_round();

        // Ida 6 + 1 (morale) = 7; Olgierd 6 (no self-boost).
        assert_cards(
            &game,
            Player::P1,
            Range::RANGED,
            &[(IDA_EMEAN, 7), (OLGIERD, 6)],
        );
    }

    #[test]
    fn impenetrable_fog_saps_the_ranged_row() {
        let mut game = Game::new(
            TestController::new(true, 2),
            Cards::northern_realms(&[KEIRA_METZ, IMPENETRABLE_FOG], &[]),
            Cards::monsters(&[], &[]),
        );

        game.next_turn(); // Keira on the ranged row
        assert_cards(&game, Player::P1, Range::RANGED, &[(KEIRA_METZ, 5)]);

        game.next_turn(); // P2 passes
        game.next_turn(); // Impenetrable Fog -> ranged row drops to 1
        assert_cards(&game, Player::P1, Range::RANGED, &[(KEIRA_METZ, 1)]);
    }

    #[test]
    fn nilfgaard_impera_brigade_shares_a_tight_bond() {
        // Two Impera Brigade Guards (Nilfgaard, tight bond): 3 * 2 = 6 each.
        let mut game = Game::new(
            TestController::new(true, 2),
            Cards::nilfgaard(&[IMPERA_BRIGADE_1, IMPERA_BRIGADE_2], &[]),
            Cards::monsters(&[], &[]),
        );

        game.play_round();

        assert_cards(
            &game,
            Player::P1,
            Range::MELEE,
            &[(IMPERA_BRIGADE_1, 6), (IMPERA_BRIGADE_2, 6)],
        );
    }
}
