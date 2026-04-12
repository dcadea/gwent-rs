use std::collections::HashMap;

use crate::card::{
    Ability::{CommandersHorn, MoraleBoost, TightBond},
    Group, Range, Special, Strength, Unit,
    Weather::{self, ClearWeather},
};

pub struct Row {
    range: Range,
    units: Vec<Unit>,
    bad_weather: bool,
    boost: Option<Special>,
    strengths: Vec<Strength>,
    is_dirty: bool,
}

impl Row {
    pub fn new(range: Range) -> Self {
        Self {
            range,
            units: Vec::default(),
            bad_weather: false,
            boost: Option::default(),
            strengths: Vec::default(),
            is_dirty: false,
        }
    }
}

impl Row {
    pub fn put_unit(&mut self, unit: Unit) {
        assert!(self.range.intersects(unit.range));
        self.strengths.push(unit.strength);
        self.units.push(unit);
        self.is_dirty = true;
    }

    pub fn put_special(&mut self, special: Special) {
        if self.boost.is_some() {
            todo!("prevent from putting another boost on same row");
        }

        match &special {
            Special::CommandersHorn | Special::Mardrome => self.boost = Some(special),
            _ => unreachable!(),
        }

        self.is_dirty = true;
    }

    /// Changes `bad_weather` flag based on weather parameter
    ///
    /// If weather is `ClearWeather` and `bad_weather` flag was false, shortcut to not set `is_dirty`
    /// If weather is `ClearWeather` and `bad_weather` flag was true, set `is_dirty` = true
    /// If weather affects current row and `bad_weather` flag was true, shortcut to not set `is_dirty`
    /// If weather affects current row and `bad_weather` flag was false, set `is_dirty` = true
    pub fn put_weather(&mut self, weather: Weather) {
        match weather {
            ClearWeather if !self.bad_weather => return,
            ClearWeather => self.bad_weather = false,
            _ if self.bad_weather => return,
            w if w.affects(self.range) => self.bad_weather = true,
            _ => return,
        }

        self.is_dirty = true;
    }
}

impl Row {
    pub fn get_strengths(&self) -> &[Strength] {
        &self.strengths
    }

    pub fn update(&mut self) {
        if self.is_dirty {
            self.recalculate_strengths();
        }
    }

    fn recalculate_strengths(&mut self) {
        for (i, unit) in self.units.iter().enumerate() {
            self.strengths[i] = match unit.strength {
                Strength::Regular(_) if self.bad_weather => Strength::Regular(1),
                strength => strength,
            };
        }

        let tight_bonds: HashMap<Group, u8> = self
            .units
            .iter()
            .filter(|unit| matches!(unit.ability, TightBond(_)))
            .fold(HashMap::new(), |mut acc, unit| {
                if let TightBond(group) = unit.ability {
                    *acc.entry(group).or_insert(0) += 1;
                }
                acc
            });

        for (i, unit) in self.units.iter().enumerate() {
            if let TightBond(group) = unit.ability
                && let Some(bond_count) = tight_bonds.get(&group)
            {
                self.strengths[i].mul_assign(*bond_count);
            }
        }

        let morale_boosts = u8::try_from(
            self.units
                .iter()
                .filter(|unit| unit.ability == MoraleBoost)
                .count(),
        )
        .unwrap_or(0);

        for (i, unit) in self.units.iter().enumerate() {
            let mut current_boosts = morale_boosts;
            if unit.ability == MoraleBoost {
                current_boosts -= 1;
            }

            self.strengths[i].add_assign(current_boosts);
        }

        let has_horn_unit = self.units.iter().any(|unit| unit.ability == CommandersHorn);

        let commanders_horn = matches!(self.boost, Some(Special::CommandersHorn));
        if commanders_horn || has_horn_unit {
            for (i, unit) in self.units.iter().enumerate() {
                if unit.ability == CommandersHorn && !commanders_horn {
                    continue;
                }
                self.strengths[i].mul_assign(2);
            }
        }

        self.is_dirty = false;
    }
}

#[cfg(test)]
mod test {
    use std::panic::AssertUnwindSafe;

    use crate::{
        card::{Ability, Range, Strength, Unit},
        row::Row,
    };

    #[test]
    fn should_panic_when_unit_is_not_compatible_with_row() {
        for (unit_range, row_range) in [
            (Range::MELEE, Range::RANGED),
            (Range::MELEE, Range::SIEGE),
            (Range::RANGED, Range::MELEE),
            (Range::RANGED, Range::SIEGE),
            (Range::SIEGE, Range::MELEE),
            (Range::SIEGE, Range::RANGED),
            (Range::AGILE, Range::SIEGE),
        ] {
            let mut row = Row::new(row_range);

            let result = std::panic::catch_unwind(AssertUnwindSafe(|| {
                row.put_unit(Unit::new(Strength::Regular(5), Ability::None, unit_range));
            }));

            assert!(result.is_err());
        }
    }
}
