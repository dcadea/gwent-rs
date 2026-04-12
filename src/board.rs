use crate::{
    card::{self, Card, Range, Special, Unit},
    game::Action,
    side::{self, Side},
};

#[derive(Clone, Copy, Hash, Eq, PartialEq)]
pub enum Player {
    P1,
    P2,
}

#[derive(Default)]
pub struct Board {
    player1: Side,
    player2: Side,
}

impl Board {
    pub fn get_strengths(&self) -> Strengths<'_> {
        Strengths {
            p1: self.player1.get_strengths(),
            p2: self.player2.get_strengths(),
        }
    }

    /// Returns max unit strength excluding heroes
    fn get_max_strength(&self, range: Range) -> Option<u8> {
        [
            self.player1.get_max_strength(range),
            self.player2.get_max_strength(range),
        ]
        .into_iter()
        .flatten()
        .max()
    }

    pub fn recalculate_strengths(&mut self) {
        self.player1.recalculate_strengths();
        self.player2.recalculate_strengths();
    }
}

impl Board {
    pub fn put(&mut self, player: Player, card: Card) -> Action {
        match card {
            Card::Unit(unit) => {
                if unit.range == Range::AGILE {
                    return Action::Agile(unit);
                }

                let action = match unit.ability {
                    card::Ability::Medic => Action::Medic,
                    card::Ability::Muster(group) => Action::Muster(group),
                    card::Ability::Scorch(range) => Action::Scorch(range),
                    card::Ability::Spy => Action::Spy,
                    card::Ability::Berserker => Action::Berserker,
                    card::Ability::Mardrome(range) => Action::Mardrome(range),
                    _ => Action::None,
                };

                match player {
                    Player::P1 => {
                        if matches!(action, Action::Spy) {
                            &mut self.player2
                        } else {
                            &mut self.player1
                        }
                    }
                    Player::P2 => {
                        if matches!(action, Action::Spy) {
                            &mut self.player1
                        } else {
                            &mut self.player2
                        }
                    }
                }
                .put_unit(unit);

                action
            }
            Card::Special(special) => match special {
                card::Special::CommandersHorn => Action::CommandersHorn,
                card::Special::Decoy => Action::Decoy,
                card::Special::Mardrome => Action::Mardrome(Range::ALL),
                card::Special::Scorch => Action::Scorch(Range::ALL),
                card::Special::Weather(weather) => {
                    self.player1.put_weather(weather);
                    self.player2.put_weather(weather);
                    Action::None
                }
            },
        }
    }

    pub fn put_agile_unit(&mut self, player: Player, unit: Unit, range: Range) {
        assert!(range == Range::MELEE || range == Range::RANGED);
        match player {
            Player::P1 => &mut self.player1,
            Player::P2 => &mut self.player2,
        }
        .put_agile_unit(unit, range);
    }

    pub fn put_row_boost(&mut self, player: Player, boost: Special, range: Range) {
        match player {
            Player::P1 => &mut self.player1,
            Player::P2 => &mut self.player2,
        }
        .put_row_boost(boost, range);
    }

    pub fn put_scorch(&mut self, player: Player, range: Range) {
        self.recalculate_strengths();

        // Global scorch
        if range == Range::ALL {
            if let Some(max_strength) = self.get_max_strength(range) {
                self.player1.put_scorch(max_strength, Range::ALL);
                self.player2.put_scorch(max_strength, Range::ALL);
            }
        } else {
            // Row target scorch
            let total_row_strength = match player {
                Player::P1 => &self.player2,
                Player::P2 => &self.player1,
            }
            .get_total_strength(range);

            // Applies only if total strength of row is >= 10
            if total_row_strength >= 10 {
                match player {
                    Player::P1 => {
                        if let Some(max_row_strength) = self.player2.get_max_strength(range) {
                            self.player2.put_scorch(max_row_strength, range);
                        }
                    }
                    Player::P2 => {
                        if let Some(max_row_strength) = self.player1.get_max_strength(range) {
                            self.player1.put_scorch(max_row_strength, range);
                        }
                    }
                }
            }
        }
    }
}

pub struct Strengths<'a> {
    pub p1: side::Strengths<'a>,
    pub p2: side::Strengths<'a>,
}

#[cfg(test)]
mod test {
    use crate::{
        board::{Board, Player::P1},
        card::{Ability, Card, Range, Special, Strength, Weather},
    };

    #[test]
    fn should_put_regular_unit() {
        let mut board = Board::default();

        board.put(P1, Card::unit(5, Range::MELEE));
        board.recalculate_strengths();

        let row = board.get_strengths().p1.melee;

        assert_eq!(row, vec![Strength::Regular(5)]);
    }

    #[test]
    fn should_put_agile_unit_on_melee_row() {
        // TODO:
        // let mut board = Board::default();
        // board.put(
        //     P1,
        //     Card::Unit(Unit::new(Strength::Regular(5), Ability::None, Range::AGILE)),
        // );

        // assert!(vec![Strength::Regular(5)] == *board.get_strengths(P1, Range::MELEE));
    }

    #[test]
    fn should_put_agile_unit_on_ranged_row() {
        // TODO:
        // let mut board = Board::default();
        // board.put(
        //     P1,
        //     Card::Unit(Unit::new(Strength::Regular(5), Ability::None, Range::AGILE)),
        // );

        // assert!(vec![Strength::Regular(5)] == *board.get_strengths(P1, Range::MELEE));
    }

    #[test]
    fn should_apply_morale_boost_twice() {
        let cards = [
            Card::unit(5, Range::MELEE),
            Card::the_unit(10, Range::MELEE, Ability::MoraleBoost),
            Card::hero(7, Range::MELEE),
            Card::the_hero(10, Range::MELEE, Ability::MoraleBoost),
        ];

        let mut board = Board::default();
        for card in cards {
            board.put(P1, card);
        }
        board.recalculate_strengths();

        let row = board.get_strengths().p1.melee;

        assert_eq!(
            row,
            vec![
                Strength::Regular(7),
                Strength::Regular(11),
                Strength::Hero(7),
                Strength::Hero(10)
            ]
        );
    }

    #[test]
    fn should_apply_commanders_horn() {
        // TODO
        // let mut board = Board::default();
        // board.put(
        //     P1,
        //     Card::Unit(Unit::new(Strength::Regular(5), Ability::None, Range::MELEE)),
        // );
        // board.put(P1, Card::Special(Special::CommandersHorn));
        // board.put(
        //     P1,
        //     Card::Unit(Unit::new(Strength::Hero(7), Ability::None, Range::MELEE)),
        // );

        // assert!(vec![Strength::Regular(10), Strength::Hero(7)] == *row.get_strengths());
    }

    #[test]
    fn should_apply_unit_commanders_horn() {
        let cards = [
            Card::unit(5, Range::MELEE),
            Card::the_unit(2, Range::MELEE, Ability::CommandersHorn),
        ];

        let mut board = Board::default();
        for card in cards {
            board.put(P1, card);
        }
        board.recalculate_strengths();

        let row = board.get_strengths().p1.melee;

        assert_eq!(row, vec![Strength::Regular(10), Strength::Regular(2)]);
    }

    #[test]
    fn should_apply_unit_and_special_commanders_horns() {
        // TODO
        // let mut row = Row::new(Range::MELEE);
        // row.put_unit(Unit::new(Strength::Regular(5), Ability::None, Range::MELEE));
        // row.put_unit(Unit::new(
        //     Strength::Regular(2),
        //     Ability::CommandersHorn,
        //     Range::MELEE,
        // ));
        // row.put_special(Special::CommandersHorn);

        // assert!(vec![Strength::Regular(10), Strength::Regular(4)] == *row.get_strengths());
    }

    #[test]
    fn should_apply_tight_bond() {
        let cards = [
            Card::the_unit(4, Range::MELEE, Ability::TightBond(1)),
            Card::the_unit(5, Range::MELEE, Ability::TightBond(2)),
            Card::hero(7, Range::MELEE),
            Card::the_unit(4, Range::MELEE, Ability::TightBond(1)),
        ];

        let mut board = Board::default();
        for card in cards {
            board.put(P1, card);
        }
        board.recalculate_strengths();

        let row = board.get_strengths().p1.melee;

        assert_eq!(
            row,
            vec![
                Strength::Regular(8),
                Strength::Regular(5),
                Strength::Hero(7),
                Strength::Regular(8)
            ]
        );
    }

    #[test]
    fn should_apply_tight_bond_and_morale_boost() {
        let cards = [
            Card::the_unit(4, Range::MELEE, Ability::TightBond(1)),
            Card::the_unit(5, Range::MELEE, Ability::TightBond(2)),
            Card::hero(7, Range::MELEE),
            Card::the_unit(4, Range::MELEE, Ability::TightBond(1)),
            Card::the_unit(10, Range::MELEE, Ability::MoraleBoost),
        ];

        let mut board = Board::default();
        for card in cards {
            board.put(P1, card);
        }
        board.recalculate_strengths();

        let row = board.get_strengths().p1.melee;

        assert_eq!(
            row,
            vec![
                Strength::Regular(9),
                Strength::Regular(6),
                Strength::Hero(7),
                Strength::Regular(9),
                Strength::Regular(10)
            ]
        );
    }

    #[test]
    fn should_apply_moral_boost_tight_bond_and_commanders_horn() {
        let cards = [
            Card::unit(5, Range::MELEE),
            Card::hero(7, Range::MELEE),
            Card::the_unit(6, Range::MELEE, Ability::MoraleBoost),
            Card::the_hero(10, Range::MELEE, Ability::MoraleBoost),
            Card::the_unit(5, Range::MELEE, Ability::TightBond(2)),
            Card::the_unit(5, Range::MELEE, Ability::TightBond(2)),
            Card::the_unit(4, Range::MELEE, Ability::TightBond(1)),
            Card::the_unit(4, Range::MELEE, Ability::TightBond(1)),
            Card::the_unit(4, Range::MELEE, Ability::TightBond(1)),
            Card::the_unit(2, Range::MELEE, Ability::CommandersHorn),
        ];

        // TODO
        // row.put_special(Special::CommandersHorn);

        let mut board = Board::default();
        for card in cards {
            board.put(P1, card);
        }
        board.recalculate_strengths();

        let row = board.get_strengths().p1.melee;

        assert_eq!(
            row,
            vec![
                Strength::Regular(14),
                Strength::Hero(7),
                Strength::Regular(14),
                Strength::Hero(10),
                Strength::Regular(24),
                Strength::Regular(24),
                Strength::Regular(28),
                Strength::Regular(28),
                Strength::Regular(28),
                Strength::Regular(4),
            ]
        );
    }

    #[test]
    fn should_affect_units_by_weather() {
        for (range, weather) in [
            (Range::MELEE, Weather::BitingFrost),
            (Range::RANGED, Weather::ImpenetrableFog),
            (Range::RANGED, Weather::SkelligeStorm),
            (Range::SIEGE, Weather::TorrentialRain),
            (Range::SIEGE, Weather::SkelligeStorm),
        ] {
            let cards = [
                Card::unit(5, range),
                Card::hero(10, range),
                Card::Special(Special::Weather(weather)),
            ];

            let mut board = Board::default();
            for card in cards {
                board.put(P1, card);
            }
            board.recalculate_strengths();

            let strengths = board.get_strengths();
            let row = strengths.p1.get(range);

            assert_eq!(row, vec![Strength::Regular(1), Strength::Hero(10)]);
        }
    }

    #[test]
    fn should_affect_units_by_weather_only_once() {
        for (range, weather) in [
            (Range::MELEE, Weather::BitingFrost),
            (Range::RANGED, Weather::ImpenetrableFog),
            (Range::RANGED, Weather::SkelligeStorm),
            (Range::SIEGE, Weather::TorrentialRain),
            (Range::SIEGE, Weather::SkelligeStorm),
        ] {
            let cards = [
                Card::unit(5, range),
                Card::hero(10, range),
                Card::Special(Special::Weather(weather)),
                Card::Special(Special::Weather(weather)),
            ];

            let mut board = Board::default();
            for card in cards {
                board.put(P1, card);
            }
            board.recalculate_strengths();

            let strengths = board.get_strengths();
            let row = strengths.p1.get(range);

            assert_eq!(row, vec![Strength::Regular(1), Strength::Hero(10)]);
        }
    }

    #[test]
    fn should_not_affect_units_by_weather() {
        for (range, weather) in [
            (Range::MELEE, Weather::ImpenetrableFog),
            (Range::MELEE, Weather::SkelligeStorm),
            (Range::MELEE, Weather::TorrentialRain),
            (Range::RANGED, Weather::BitingFrost),
            (Range::SIEGE, Weather::BitingFrost),
        ] {
            let cards = [
                Card::unit(5, range),
                Card::hero(10, range),
                Card::Special(Special::Weather(weather)),
            ];

            let mut board = Board::default();
            for card in cards {
                board.put(P1, card);
            }
            board.recalculate_strengths();

            let strengths = board.get_strengths();
            let row = strengths.p1.get(range);

            assert_eq!(row, vec![Strength::Regular(5), Strength::Hero(10)]);
        }
    }

    #[test]
    fn should_not_affect_units_by_clear_weather() {
        for range in [Range::MELEE, Range::RANGED, Range::SIEGE] {
            let cards = [
                Card::unit(5, range),
                Card::hero(10, range),
                Card::Special(Special::Weather(Weather::ClearWeather)),
            ];

            let mut board = Board::default();
            for card in cards {
                board.put(P1, card);
            }
            board.recalculate_strengths();

            let strengths = board.get_strengths();
            let row = strengths.p1.get(range);

            assert_eq!(row, vec![Strength::Regular(5), Strength::Hero(10)]);
        }
    }

    #[test]
    fn should_restore_units_strength_when_weather_is_cleared() {
        for (range, weather) in [
            (Range::MELEE, Weather::BitingFrost),
            (Range::RANGED, Weather::ImpenetrableFog),
            (Range::RANGED, Weather::SkelligeStorm),
            (Range::SIEGE, Weather::TorrentialRain),
            (Range::SIEGE, Weather::SkelligeStorm),
        ] {
            let cards = [
                Card::unit(5, range),
                Card::hero(10, range),
                Card::Special(Special::Weather(weather)),
            ];

            let mut board = Board::default();
            for card in cards {
                board.put(P1, card);
            }
            board.recalculate_strengths();

            let strengths = board.get_strengths();
            let row = strengths.p1.get(range);

            assert_eq!(row, vec![Strength::Regular(1), Strength::Hero(10)]);

            board.put(P1, Card::Special(Special::Weather(Weather::ClearWeather)));
            board.recalculate_strengths();

            let strengths = board.get_strengths();
            let row = strengths.p1.get(range);

            assert_eq!(row, vec![Strength::Regular(5), Strength::Hero(10)]);
        }
    }

    #[test]
    fn should_apply_weather_when_moral_boost_tight_bond_and_commanders_horn() {
        let cards = [
            Card::Special(Special::Weather(Weather::BitingFrost)),
            Card::unit(5, Range::MELEE),
            Card::hero(7, Range::MELEE),
            Card::the_unit(6, Range::MELEE, Ability::MoraleBoost),
            Card::the_hero(10, Range::MELEE, Ability::MoraleBoost),
            Card::the_unit(5, Range::MELEE, Ability::TightBond(2)),
            Card::the_unit(5, Range::MELEE, Ability::TightBond(2)),
            Card::the_unit(4, Range::MELEE, Ability::TightBond(1)),
            Card::the_unit(4, Range::MELEE, Ability::TightBond(1)),
            Card::the_unit(4, Range::MELEE, Ability::TightBond(1)),
            Card::the_unit(2, Range::MELEE, Ability::CommandersHorn),
        ];

        // TODO:
        // row.put_special(Special::CommandersHorn);

        let mut board = Board::default();
        for card in cards {
            board.put(P1, card);
        }
        board.recalculate_strengths();

        let row = board.get_strengths().p1.melee;

        assert_eq!(
            row,
            vec![
                Strength::Regular(6),
                Strength::Hero(7),
                Strength::Regular(4),
                Strength::Hero(10),
                Strength::Regular(8),
                Strength::Regular(8),
                Strength::Regular(10),
                Strength::Regular(10),
                Strength::Regular(10),
                Strength::Regular(3),
            ]
        );
    }
}
