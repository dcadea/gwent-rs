use std::{cell::Ref, collections::HashMap};

use crate::{
    card::{Range, Special, Strength, Unit, Weather},
    row::Row,
};

pub struct Side {
    melee: Row,
    ranged: Row,
    siege: Row,
}

impl Default for Side {
    fn default() -> Self {
        Self {
            melee: Row::new(Range::MELEE),
            ranged: Row::new(Range::RANGED),
            siege: Row::new(Range::SIEGE),
        }
    }
}

impl Side {
    pub fn get_strengths(&self) -> HashMap<Range, Ref<'_, Vec<Strength>>> {
        let mut rows = HashMap::new();

        rows.insert(Range::MELEE, self.melee.get_strengths());
        rows.insert(Range::RANGED, self.ranged.get_strengths());
        rows.insert(Range::SIEGE, self.siege.get_strengths());

        rows
    }
}

impl Side {
    pub fn put_unit(&mut self, unit: Unit) {
        match unit.range {
            Range::MELEE => &mut self.melee,
            Range::RANGED => &mut self.ranged,
            Range::SIEGE => &mut self.siege,
            _ => unreachable!(),
        }
        .put_unit(unit);
    }

    pub fn put_agile_unit(&mut self, unit: Unit, range: Range) {
        match range {
            Range::MELEE => &mut self.melee,
            Range::RANGED => &mut self.ranged,
            _ => unreachable!(),
        }
        .put_unit(unit);
    }

    pub fn put_weather(&mut self, weather: Weather) {
        self.melee.put_weather(weather);
        self.ranged.put_weather(weather);
        self.siege.put_weather(weather);
    }

    pub fn put_row_boost(&mut self, boost: Special, range: Range) {
        match range {
            Range::MELEE => &mut self.melee,
            Range::RANGED => &mut self.ranged,
            Range::SIEGE => &mut self.siege,
            _ => unreachable!(),
        }
        .put_special(boost);
    }
}
