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
    pub fn get_strengths(&self) -> Strengths<'_> {
        Strengths {
            melee: self.melee.get_strengths(),
            ranged: self.ranged.get_strengths(),
            siege: self.siege.get_strengths(),
        }
    }

    pub fn update(&mut self) {
        self.melee.update();
        self.ranged.update();
        self.siege.update();
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

pub struct Strengths<'a> {
    pub melee: &'a [Strength],
    pub ranged: &'a [Strength],
    pub siege: &'a [Strength],
}

#[cfg(test)]
impl Strengths<'_> {
    pub const fn get(&self, range: Range) -> &[Strength] {
        match range {
            Range::MELEE => self.melee,
            Range::RANGED => self.ranged,
            Range::SIEGE => self.siege,
            _ => unreachable!(),
        }
    }
}
