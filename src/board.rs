use crate::{
    card::{self, Card, Range, Special, Unit},
    game::Action,
    side::{self, Side},
};

#[derive(Clone, Copy, Hash, Eq, PartialEq, Debug)]
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
    fn get_max_row_strength(&self, range: Range) -> Option<u8> {
        [
            self.player1.get_max_row_strength(range),
            self.player2.get_max_row_strength(range),
        ]
        .into_iter()
        .flatten()
        .max()
    }

    pub fn update(&mut self) {
        self.player1.update();
        self.player2.update();
    }
}

impl Board {
    pub fn put(&mut self, player: Player, card: Card) -> Action {
        match card {
            Card::Unit(unit) => self.put_unit(player, unit),
            Card::Special(_, special) => self.put_special(special),
        }
    }

    pub fn put_agile_unit(&mut self, player: Player, unit: Unit, range: Range) {
        assert!(range == Range::MELEE || range == Range::RANGED);

        self.get_current_player_mut(player)
            .put_agile_unit(unit, range);
    }

    pub fn put_row_boost(&mut self, player: Player, boost: Special, range: Range) {
        self.get_current_player_mut(player)
            .put_row_boost(boost, range);
    }

    pub fn put_scorch(&mut self, player: Player, range: Range) {
        self.update();

        // Global scorch
        if range == Range::ALL {
            if let Some(max_strength) = self.get_max_row_strength(range) {
                self.player1.put_scorch(max_strength, Range::ALL);
                self.player2.put_scorch(max_strength, Range::ALL);
            }
        } else {
            // Row target scorch
            let opponent_side = self.get_opponent_player_mut(player);

            let total_row_strength = opponent_side.get_total_row_strength(range);

            // Applies only if total strength of row is >= 10
            if total_row_strength >= 10
                && let Some(max_row_strength) = opponent_side.get_max_row_strength(range)
            {
                opponent_side.put_scorch(max_row_strength, range);
            }
        }
    }

    pub fn remove_unit(&mut self, player: Player, range: Range, i: usize) -> Unit {
        self.get_current_player_mut(player).remove_unit(range, i)
    }
}

impl Board {
    const fn get_current_player_mut(&mut self, player: Player) -> &mut Side {
        match player {
            Player::P1 => &mut self.player1,
            Player::P2 => &mut self.player2,
        }
    }

    const fn get_opponent_player_mut(&mut self, player: Player) -> &mut Side {
        match player {
            Player::P1 => &mut self.player2,
            Player::P2 => &mut self.player1,
        }
    }

    fn put_unit(&mut self, player: Player, unit: Unit) -> Action {
        if unit.range == Range::AGILE {
            return Action::Agile(unit);
        }

        let action = match &unit.ability {
            card::Ability::Medic => Action::Medic,
            card::Ability::Muster(ids) => Action::Muster(ids.clone()),
            card::Ability::Scorch(range) => Action::Scorch(*range),
            card::Ability::Spy => Action::Spy,
            card::Ability::Mardrome
            | card::Ability::CommandersHorn
            | card::Ability::MoraleBoost
            | card::Ability::TightBond(_)
            | card::Ability::Summon(_)
            | card::Ability::Berserker(_)
            | card::Ability::None => Action::None,
        };

        if matches!(action, Action::Spy) {
            self.get_opponent_player_mut(player)
        } else {
            self.get_current_player_mut(player)
        }
        .put_unit(unit);

        action
    }

    fn put_special(&mut self, special: Special) -> Action {
        match special {
            card::Special::CommandersHorn => Action::CommandersHorn,
            card::Special::Decoy => Action::Decoy,
            card::Special::Mardrome => Action::Mardrome,
            card::Special::Scorch => Action::Scorch(Range::ALL),
            card::Special::Weather(weather) => {
                self.player1.put_weather(weather);
                self.player2.put_weather(weather);
                Action::None
            }
        }
    }
}

pub struct Strengths<'a> {
    pub p1: side::Strengths<'a>,
    pub p2: side::Strengths<'a>,
}
