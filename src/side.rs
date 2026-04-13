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

    /// Returns max unit strength excluding heroes
    pub fn get_max_row_strength(&self, range: Range) -> Option<u8> {
        match range {
            Range::ALL => [
                self.melee.get_max_strength(),
                self.ranged.get_max_strength(),
                self.siege.get_max_strength(),
            ]
            .into_iter()
            .flatten()
            .max(),
            range => self.get_row(range).get_max_strength(),
        }
    }

    /// Returns total unit strength on the given row
    pub fn get_total_row_strength(&self, range: Range) -> u8 {
        self.get_row(range)
            .get_strengths()
            .iter()
            .map(|s| s.get())
            .sum()
    }

    pub fn update(&mut self) {
        self.melee.update();
        self.ranged.update();
        self.siege.update();
    }
}

impl Side {
    pub fn put_unit(&mut self, unit: Unit) {
        self.get_row_mut(unit.range).put_unit(unit);
    }

    pub fn put_agile_unit(&mut self, unit: Unit, range: Range) {
        self.get_row_mut(range).put_unit(unit);
    }

    pub fn put_weather(&mut self, weather: Weather) {
        self.melee.put_weather(weather);
        self.ranged.put_weather(weather);
        self.siege.put_weather(weather);
    }

    pub fn put_row_boost(&mut self, boost: Special, range: Range) {
        self.get_row_mut(range).put_special(boost);
    }

    pub fn put_scorch(&mut self, max_strength: u8, range: Range) {
        match range {
            Range::ALL => {
                self.melee.put_scorch(max_strength);
                self.ranged.put_scorch(max_strength);
                self.siege.put_scorch(max_strength);
            }
            range => self.get_row_mut(range).put_scorch(max_strength),
        }
    }

    pub fn remove_unit(&mut self, range: Range, i: usize) -> Unit {
        self.get_row_mut(range).remove_unit(i)
    }
}

impl Side {
    fn get_row(&self, range: Range) -> &Row {
        match range {
            Range::MELEE => &self.melee,
            Range::RANGED => &self.ranged,
            Range::SIEGE => &self.siege,
            _ => unreachable!(),
        }
    }

    fn get_row_mut(&mut self, range: Range) -> &mut Row {
        match range {
            Range::MELEE => &mut self.melee,
            Range::RANGED => &mut self.ranged,
            Range::SIEGE => &mut self.siege,
            _ => unreachable!(),
        }
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
