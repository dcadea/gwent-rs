use std::collections::HashMap;

use crate::card::{
    Ability::{Berserker, CommandersHorn, Mardrome, MoraleBoost, Summon, TightBond},
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

    /// Removes every non-hero unit whose strength equals `max_strength` and
    /// returns them so the caller can move them to the discard pile.
    pub fn put_scorch(&mut self, max_strength: u8) -> Vec<Unit> {
        self.update();

        let mut discarded = Vec::new();

        for i in (0..self.units.len()).rev() {
            if let Strength::Regular(strength) = self.strengths[i]
                && strength == max_strength
            {
                self.strengths.swap_remove(i);
                discarded.push(self.units.swap_remove(i));
            }
        }

        for unit in &discarded {
            self.spawn_summon(unit);
        }

        if !discarded.is_empty() {
            self.is_dirty = true;
        }

        discarded
    }

    pub fn remove_unit(&mut self, i: usize) -> Unit {
        self.strengths.swap_remove(i);
        let unit = self.units.swap_remove(i);
        self.spawn_summon(&unit);
        self.is_dirty = true;
        unit
    }

    pub fn clear(&mut self) -> Vec<Unit> {
        let removed = std::mem::take(&mut self.units);
        self.strengths.clear();
        self.bad_weather = false;
        self.boost = None;

        for unit in &removed {
            self.spawn_summon(unit);
        }

        self.is_dirty = true;

        removed
    }

    fn spawn_summon(&mut self, removed: &Unit) {
        if let Summon(target) = &removed.ability {
            let summoned = *target.clone();
            self.strengths.push(summoned.strength);
            self.units.push(summoned);
            self.is_dirty = true;
        }
    }
}

impl Row {
    pub fn get_strengths(&self) -> &[Strength] {
        &self.strengths
    }

    /// Returns max unit strength excluding heroes
    pub fn get_max_strength(&self) -> Option<u8> {
        self.strengths
            .iter()
            .filter(|s| matches!(s, Strength::Regular(_)))
            .map(|s| s.get())
            .max()
    }

    pub fn update(&mut self) {
        if !self.is_dirty {
            return;
        }

        self.transform_berserkers();
        self.apply_weather();
        self.apply_tight_bonds();
        self.apply_morale_boost();
        self.apply_commanders_horn();

        self.is_dirty = false;
    }
}

impl Row {
    fn transform_berserkers(&mut self) {
        let has_mardrome_unit = self
            .units
            .iter()
            .any(|unit| matches!(unit.ability, Mardrome));

        let mardrome = matches!(self.boost, Some(Special::Mardrome));
        if mardrome || has_mardrome_unit {
            let berserkers = self
                .units
                .iter()
                .enumerate()
                .filter_map(|(i, unit)| {
                    if let Berserker(b) = &unit.ability {
                        Some((i, *b.clone()))
                    } else {
                        None
                    }
                })
                .collect::<Vec<(usize, Unit)>>();

            for (i, berserker) in berserkers {
                self.units[i] = berserker;
            }
        }
    }

    fn apply_weather(&mut self) {
        for (i, unit) in self.units.iter().enumerate() {
            self.strengths[i] = match unit.strength {
                Strength::Regular(_) if self.bad_weather => Strength::Regular(1),
                strength => strength,
            };
        }
    }

    fn apply_tight_bonds(&mut self) {
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
    }

    fn apply_morale_boost(&mut self) {
        let morale_boosts = u8::try_from(
            self.units
                .iter()
                .filter(|unit| matches!(unit.ability, MoraleBoost))
                .count(),
        )
        .unwrap_or(0);

        for (i, unit) in self.units.iter().enumerate() {
            let mut current_boosts = morale_boosts;
            if matches!(unit.ability, MoraleBoost) {
                current_boosts -= 1;
            }

            self.strengths[i].add_assign(current_boosts);
        }
    }

    fn apply_commanders_horn(&mut self) {
        let has_horn_unit = self
            .units
            .iter()
            .any(|unit| matches!(unit.ability, CommandersHorn));

        let commanders_horn = matches!(self.boost, Some(Special::CommandersHorn));
        if commanders_horn || has_horn_unit {
            for (i, unit) in self.units.iter().enumerate() {
                if matches!(unit.ability, CommandersHorn) && !commanders_horn {
                    continue;
                }
                self.strengths[i].mul_assign(2);
            }
        }
    }
}

#[cfg(test)]
impl Row {
    pub fn get_ids(&self) -> Vec<u16> {
        self.units.iter().map(|unit| unit.id).collect()
    }

    pub const fn get_boost(&self) -> Option<Special> {
        self.boost
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
