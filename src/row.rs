use std::{
    cell::{Cell, Ref, RefCell},
    collections::HashMap,
};

use crate::card::{
    Ability::{CommandersHorn, MoraleBoost, TightBond},
    Group, Range, Special, Strength, Unit,
    Weather::{self, ClearWeather},
};

pub struct Row {
    range: Range,
    units: Vec<Unit>,
    bad_weather: bool,
    commanders_horn: bool,
    mardrome: bool,
    strengths: RefCell<Vec<Strength>>,
    is_dirty: Cell<bool>,
}

impl Row {
    pub fn new(range: Range) -> Self {
        Self {
            range,
            units: Vec::default(),
            bad_weather: false,
            commanders_horn: false,
            mardrome: false,
            strengths: RefCell::default(),
            is_dirty: Cell::default(),
        }
    }
}

impl Row {
    pub fn put_unit(&mut self, unit: Unit) {
        assert!(self.range.intersects(unit.range));
        self.strengths.borrow_mut().push(unit.strength);
        self.units.push(unit);
        self.is_dirty.set(true);
    }

    pub fn put_special(&mut self, special: Special) {
        match special {
            Special::CommandersHorn => self.commanders_horn = true,
            Special::Mardrome => self.mardrome = true,
            _ => unreachable!(),
        }

        self.is_dirty.set(true);
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
            w if self.bad_weather => return,
            w if w.affects(self.range) => self.bad_weather = true,
            _ => return,
        }

        self.is_dirty.set(true);
    }

    fn get_strengths(&self) -> Ref<'_, Vec<Strength>> {
        if self.is_dirty.get() {
            self.recalculate_strengths();
        }

        self.strengths.borrow()
    }

    fn recalculate_strengths(&self) {
        let mut strengths = self.strengths.borrow_mut();

        for (i, unit) in self.units.iter().enumerate() {
            strengths[i] = match unit.strength {
                Strength::Regular(strength) if self.bad_weather => Strength::Regular(1),
                strength => strength,
            };
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

            strengths[i].add_assign(current_boosts);
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
                strengths[i].mul_assign(*bond_count);
            }
        }

        let has_horn_unit = self.units.iter().any(|unit| unit.ability == CommandersHorn);

        if self.commanders_horn || has_horn_unit {
            for (i, unit) in self.units.iter().enumerate() {
                if unit.ability == CommandersHorn && !self.commanders_horn {
                    continue;
                }
                strengths[i].mul_assign(2);
            }
        }

        self.is_dirty.set(false);
    }
}

#[cfg(test)]
mod test {
    use std::panic::AssertUnwindSafe;

    use crate::{
        card::{Ability, Range, Special, Strength, Unit, Weather},
        row::{self, Row},
    };

    #[test]
    fn should_put_regular_unit() {
        let mut row = Row::new(Range::MELEE);
        row.put_unit(Unit::new(Strength::Regular(5), Ability::None, Range::MELEE));

        assert!(vec![Strength::Regular(5)] == *row.get_strengths());
    }

    #[test]
    fn should_put_agile_unit_on_melee_row() {
        let mut row = Row::new(Range::MELEE);
        row.put_unit(Unit::new(Strength::Regular(5), Ability::None, Range::AGILE));

        assert!(vec![Strength::Regular(5)] == *row.get_strengths());
    }

    #[test]
    fn should_put_agile_unit_on_ranged_row() {
        let mut row = Row::new(Range::RANGED);
        row.put_unit(Unit::new(Strength::Regular(5), Ability::None, Range::AGILE));

        assert!(vec![Strength::Regular(5)] == *row.get_strengths());
    }

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

    #[test]
    fn should_apply_morale_boost() {
        let mut row = Row::new(Range::MELEE);
        row.put_unit(Unit::new(Strength::Regular(5), Ability::None, Range::MELEE));
        row.put_unit(Unit::new(
            Strength::Regular(10),
            Ability::MoraleBoost,
            Range::MELEE,
        ));
        row.put_unit(Unit::new(Strength::Hero(7), Ability::None, Range::MELEE));

        assert!(
            vec![
                Strength::Regular(6),
                Strength::Regular(10),
                Strength::Hero(7)
            ] == *row.get_strengths()
        );
    }

    #[test]
    fn should_apply_morale_boost_twice() {
        let mut row = Row::new(Range::MELEE);
        row.put_unit(Unit::new(Strength::Regular(5), Ability::None, Range::MELEE));
        row.put_unit(Unit::new(
            Strength::Regular(10),
            Ability::MoraleBoost,
            Range::MELEE,
        ));
        row.put_unit(Unit::new(Strength::Hero(7), Ability::None, Range::MELEE));
        row.put_unit(Unit::new(
            Strength::Hero(10),
            Ability::MoraleBoost,
            Range::MELEE,
        ));

        assert!(
            vec![
                Strength::Regular(7),
                Strength::Regular(11),
                Strength::Hero(7),
                Strength::Hero(10)
            ] == *row.get_strengths()
        );
    }

    #[test]
    fn should_apply_commanders_horn() {
        let mut row = Row::new(Range::MELEE);
        row.put_unit(Unit::new(Strength::Regular(5), Ability::None, Range::MELEE));
        row.put_special(Special::CommandersHorn);
        row.put_unit(Unit::new(Strength::Hero(7), Ability::None, Range::MELEE));

        assert!(vec![Strength::Regular(10), Strength::Hero(7)] == *row.get_strengths());
    }

    #[test]
    fn should_apply_unit_commanders_horn() {
        let mut row = Row::new(Range::MELEE);
        row.put_unit(Unit::new(Strength::Regular(5), Ability::None, Range::MELEE));
        row.put_unit(Unit::new(
            Strength::Regular(2),
            Ability::CommandersHorn,
            Range::MELEE,
        ));

        assert!(vec![Strength::Regular(10), Strength::Regular(2)] == *row.get_strengths());
    }

    #[test]
    fn should_apply_unit_and_special_commanders_horns() {
        let mut row = Row::new(Range::MELEE);
        row.put_unit(Unit::new(Strength::Regular(5), Ability::None, Range::MELEE));
        row.put_unit(Unit::new(
            Strength::Regular(2),
            Ability::CommandersHorn,
            Range::MELEE,
        ));
        row.put_special(Special::CommandersHorn);

        assert!(vec![Strength::Regular(10), Strength::Regular(4)] == *row.get_strengths());
    }

    #[test]
    fn should_apply_tight_bond() {
        let mut row = Row::new(Range::MELEE);
        row.put_unit(Unit::new(
            Strength::Regular(4),
            Ability::TightBond(1),
            Range::MELEE,
        ));
        row.put_unit(Unit::new(
            Strength::Regular(5),
            Ability::TightBond(2),
            Range::MELEE,
        ));
        row.put_unit(Unit::new(Strength::Hero(7), Ability::None, Range::MELEE));
        row.put_unit(Unit::new(
            Strength::Regular(4),
            Ability::TightBond(1),
            Range::MELEE,
        ));

        assert!(
            vec![
                Strength::Regular(8),
                Strength::Regular(5),
                Strength::Hero(7),
                Strength::Regular(8)
            ] == *row.get_strengths()
        );
    }

    #[test]
    fn should_apply_tight_bond_and_morale_boost() {
        let mut row = Row::new(Range::MELEE);
        row.put_unit(Unit::new(
            Strength::Regular(4),
            Ability::TightBond(1),
            Range::MELEE,
        ));
        row.put_unit(Unit::new(
            Strength::Regular(5),
            Ability::TightBond(2),
            Range::MELEE,
        ));
        row.put_unit(Unit::new(Strength::Hero(7), Ability::None, Range::MELEE));
        row.put_unit(Unit::new(
            Strength::Regular(4),
            Ability::TightBond(1),
            Range::MELEE,
        ));
        row.put_unit(Unit::new(
            Strength::Regular(10),
            Ability::MoraleBoost,
            Range::MELEE,
        ));

        assert!(
            vec![
                Strength::Regular(10),
                Strength::Regular(6),
                Strength::Hero(7),
                Strength::Regular(10),
                Strength::Regular(10)
            ] == *row.get_strengths()
        );
    }

    #[test]
    fn should_apply_moral_boost_tight_bond_and_commanders_horn() {
        let mut row = Row::new(Range::MELEE);

        row.put_unit(Unit::new(Strength::Regular(5), Ability::None, Range::MELEE));
        row.put_unit(Unit::new(Strength::Hero(7), Ability::None, Range::MELEE));
        row.put_unit(Unit::new(
            Strength::Regular(6),
            Ability::MoraleBoost,
            Range::MELEE,
        ));
        row.put_unit(Unit::new(
            Strength::Hero(10),
            Ability::MoraleBoost,
            Range::MELEE,
        ));
        row.put_unit(Unit::new(
            Strength::Regular(5),
            Ability::TightBond(2),
            Range::MELEE,
        ));
        row.put_unit(Unit::new(
            Strength::Regular(5),
            Ability::TightBond(2),
            Range::MELEE,
        ));
        row.put_unit(Unit::new(
            Strength::Regular(4),
            Ability::TightBond(1),
            Range::MELEE,
        ));
        row.put_unit(Unit::new(
            Strength::Regular(4),
            Ability::TightBond(1),
            Range::MELEE,
        ));
        row.put_unit(Unit::new(
            Strength::Regular(4),
            Ability::TightBond(1),
            Range::MELEE,
        ));
        row.put_unit(Unit::new(
            Strength::Regular(2),
            Ability::CommandersHorn,
            Range::MELEE,
        ));
        row.put_special(Special::CommandersHorn);

        assert!(
            vec![
                Strength::Regular(14),
                Strength::Hero(7),
                Strength::Regular(14),
                Strength::Hero(10),
                Strength::Regular(28),
                Strength::Regular(28),
                Strength::Regular(36),
                Strength::Regular(36),
                Strength::Regular(36),
                Strength::Regular(8),
            ] == *row.get_strengths()
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
            let mut row = Row::new(range);
            row.put_unit(Unit::new(Strength::Regular(5), Ability::None, range));
            row.put_unit(Unit::new(Strength::Hero(10), Ability::None, range));
            row.put_weather(weather);

            assert!(vec![Strength::Regular(1), Strength::Hero(10)] == *row.get_strengths());
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
            let mut row = Row::new(range);
            row.put_unit(Unit::new(Strength::Regular(5), Ability::None, range));
            row.put_unit(Unit::new(Strength::Hero(10), Ability::None, range));

            row.put_weather(weather);
            assert!(vec![Strength::Regular(1), Strength::Hero(10)] == *row.get_strengths());

            row.put_weather(weather);
            assert!(vec![Strength::Regular(1), Strength::Hero(10)] == *row.get_strengths());

            row.put_weather(weather);
            assert!(vec![Strength::Regular(1), Strength::Hero(10)] == *row.get_strengths());
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
            let mut row = Row::new(range);
            row.put_unit(Unit::new(Strength::Regular(5), Ability::None, range));
            row.put_unit(Unit::new(Strength::Hero(10), Ability::None, range));
            row.put_weather(weather);

            assert!(vec![Strength::Regular(5), Strength::Hero(10)] == *row.get_strengths());
        }
    }

    #[test]
    fn should_not_affect_units_by_clear_weather() {
        for range in [Range::MELEE, Range::RANGED, Range::SIEGE] {
            let mut row = Row::new(range);
            row.put_unit(Unit::new(Strength::Regular(5), Ability::None, range));
            row.put_unit(Unit::new(Strength::Hero(10), Ability::None, range));
            row.put_weather(Weather::ClearWeather);

            assert!(vec![Strength::Regular(5), Strength::Hero(10)] == *row.get_strengths());
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
            let mut row = Row::new(range);
            row.put_unit(Unit::new(Strength::Regular(5), Ability::None, range));
            row.put_unit(Unit::new(Strength::Hero(10), Ability::None, range));
            row.put_weather(weather);

            assert!(vec![Strength::Regular(1), Strength::Hero(10)] == *row.get_strengths());

            row.put_weather(Weather::ClearWeather);

            assert!(vec![Strength::Regular(5), Strength::Hero(10)] == *row.get_strengths());
        }
    }

    #[test]
    fn should_apply_weather_when_moral_boost_tight_bond_and_commanders_horn() {
        let mut row = Row::new(Range::MELEE);

        row.put_weather(Weather::BitingFrost);
        row.put_unit(Unit::new(Strength::Regular(5), Ability::None, Range::MELEE));
        row.put_unit(Unit::new(Strength::Hero(7), Ability::None, Range::MELEE));
        row.put_unit(Unit::new(
            Strength::Regular(6),
            Ability::MoraleBoost,
            Range::MELEE,
        ));
        row.put_unit(Unit::new(
            Strength::Hero(10),
            Ability::MoraleBoost,
            Range::MELEE,
        ));
        row.put_unit(Unit::new(
            Strength::Regular(5),
            Ability::TightBond(2),
            Range::MELEE,
        ));
        row.put_unit(Unit::new(
            Strength::Regular(5),
            Ability::TightBond(2),
            Range::MELEE,
        ));
        row.put_unit(Unit::new(
            Strength::Regular(4),
            Ability::TightBond(1),
            Range::MELEE,
        ));
        row.put_unit(Unit::new(
            Strength::Regular(4),
            Ability::TightBond(1),
            Range::MELEE,
        ));
        row.put_unit(Unit::new(
            Strength::Regular(4),
            Ability::TightBond(1),
            Range::MELEE,
        ));
        row.put_unit(Unit::new(
            Strength::Regular(2),
            Ability::CommandersHorn,
            Range::MELEE,
        ));
        row.put_special(Special::CommandersHorn);

        assert!(
            vec![
                Strength::Regular(6),
                Strength::Hero(7),
                Strength::Regular(4),
                Strength::Hero(10),
                Strength::Regular(12),
                Strength::Regular(12),
                Strength::Regular(18),
                Strength::Regular(18),
                Strength::Regular(18),
                Strength::Regular(6),
            ] == *row.get_strengths()
        );
    }
}
