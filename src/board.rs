use std::collections::HashSet;

use crate::{
    card::{self, Card, Range, Unit, Weather},
    game::Action,
    side::Side,
};

#[derive(Clone, Copy)]
pub enum Player {
    P1,
    P2,
}

#[derive(Default)]
pub struct Board {
    player1: Side,
    player2: Side,
    weather: HashSet<Weather>,
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

                match (&action, player) {
                    (Action::Spy, Player::P1) | (_, Player::P2) => &mut self.player2,
                    (Action::Spy, Player::P2) | (_, Player::P1) => &mut self.player1,
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
}
