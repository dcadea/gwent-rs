use std::ops::{AddAssign, MulAssign};

use bitflags::bitflags;

#[derive(Clone)]
pub enum Card {
    Unit(Unit),
    Special(Special),
}

impl Card {
    pub fn unit(strength: u8, name: impl Into<String>, range: Range) -> Self {
        Self::the_unit(strength, name, range, Ability::None)
    }

    pub fn the_unit(strength: u8, name: impl Into<String>, range: Range, ability: Ability) -> Self {
        Self::Unit(Unit {
            strength: Strength::Regular(strength),
            name: name.into(),
            ability,
            range,
        })
    }

    pub fn hero(strength: u8, name: impl Into<String>, range: Range) -> Self {
        Self::the_hero(strength, name, range, Ability::None)
    }

    pub fn the_hero(strength: u8, name: impl Into<String>, range: Range, ability: Ability) -> Self {
        Self::Unit(Unit {
            strength: Strength::Hero(strength),
            name: name.into(),
            ability,
            range,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Strength {
    Hero(u8),
    Regular(u8),
}

impl Strength {
    pub const fn get(self) -> u8 {
        match self {
            Self::Hero(strength) | Self::Regular(strength) => strength,
        }
    }
    pub fn add_assign(&mut self, rhs: u8) {
        match self {
            Self::Hero(_) => {}
            Self::Regular(strength) => strength.add_assign(rhs),
        }
    }

    pub fn mul_assign(&mut self, rhs: u8) {
        match self {
            Self::Hero(_) => {}
            Self::Regular(strength) => strength.mul_assign(rhs),
        }
    }
}

#[derive(Clone)]
pub struct Unit {
    pub strength: Strength,
    pub name: String,
    pub ability: Ability,
    pub range: Range,
}

pub type Group = u8;

#[derive(Clone, Eq, PartialEq)]
pub enum Ability {
    CommandersHorn,
    Medic,
    MoraleBoost,
    Muster(Group, bool),
    TightBond(Group),
    Scorch(Range),
    Spy,
    Summon,
    Berserker,
    Mardrome(Range),
    None,
}

bitflags! {
    #[derive(Clone, Copy, Eq, PartialEq, Hash)]
    pub struct Range: u8 {
        const MELEE  = 0b001;
        const RANGED = 0b010;
        const SIEGE  = 0b100;
        const AGILE  = Self::MELEE.bits() | Self::RANGED.bits();
        const ALL    = Self::MELEE.bits() | Self::RANGED.bits() | Self::SIEGE.bits();
    }
}

#[derive(Clone, Copy)]
pub enum Special {
    /// Doubles the strength of all [`Card::Unit`] on its row
    CommandersHorn,

    /// Replace one [`Card::Unit`] from the battlefield and return
    /// to player's hand
    Decoy,

    /// Triggers transformation of all [`Ability::Berserker`]
    /// cards on its row
    Mardrome,

    /// Remove all [`Card::Unit`] with the highest strength
    /// from the entire battlefield
    Scorch,

    /// Applies [`Weather`] effect on the entire battlefield
    Weather(Weather),
}

/// Does not affect heroes
#[derive(Clone, Copy, Hash, Eq, PartialEq)]
pub enum Weather {
    /// Sets the strength of all [`Range::MELEE`] units to 1
    BitingFrost,

    /// Sets the strength of all [`Range::RANGED`] units to 1
    ImpenetrableFog,

    /// Sets the strength of all [`Range::SIEGE`] units to 1
    TorrentialRain,

    /// Sets the strength of all [`Range::RANGED`] and [`Range::SIEGE`] units to 1
    SkelligeStorm,

    /// Cancels all weather effects
    #[allow(clippy::enum_variant_names)]
    ClearWeather,
}

impl Weather {
    pub fn affects(self, range: Range) -> bool {
        match self {
            Self::BitingFrost => range == Range::MELEE,
            Self::ImpenetrableFog => range == Range::RANGED,
            Self::TorrentialRain => range == Range::SIEGE,
            Self::SkelligeStorm => (Range::RANGED | Range::SIEGE).intersects(range),
            Self::ClearWeather => false,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::card::{Ability, Card, Range, Strength, Unit};

    impl Unit {
        pub fn new(strength: Strength, ability: Ability, range: Range) -> Self {
            Self {
                strength,
                name: "test_name".to_string(),
                ability,
                range,
            }
        }
    }
}
