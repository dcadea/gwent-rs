use std::cell::Ref;

use crate::{
    card::{Range, Strength, Unit, Weather},
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
    pub fn get_strengths(&self, range: Range) -> Ref<'_, Vec<Strength>> {
        match range {
            Range::MELEE => &self.melee,
            Range::RANGED => &self.ranged,
            Range::SIEGE => &self.siege,
            _ => unreachable!(),
        }
        .get_strengths()
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
}
