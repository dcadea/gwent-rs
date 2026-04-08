use std::ops::{AddAssign, MulAssign};

use bitflags::bitflags;

pub enum Card {
    Unit(Unit),
    Special(Special),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Strength {
    Hero(u8),
    Regular(u8),
}

impl Strength {
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

pub struct Unit {
    pub strength: Strength,
    pub ability: Ability,
    pub range: Range,
}

pub type Group = u8;

#[derive(Eq, PartialEq)]
pub enum Ability {
    CommandersHorn,
    Medic,
    MoraleBoost,
    Muster(Group),
    TightBond(Group),
    Scorch(Range),
    Spy,
    Summon,
    Berserker,
    Mardrome(Range),
    None,
}

bitflags! {
    #[derive(Clone, Copy, Eq, PartialEq)]
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
    /// Doubles the strength of all [`crate::card::Card::Unit`] on its row
    CommandersHorn,

    /// Replace one [`crate::card::Card::Unit`] from the battlefield and return
    /// to player's hand
    Decoy,

    /// Triggers transformation of all [`crate::card::unit::Ability::Berserker`]
    /// cards on its row
    Mardrome,

    /// Remove all [`crate::card::Card::Unit`] with the highest strength
    /// from the entire battlefield
    Scorch,

    /// Applies [`Weather`] effect on the entire battlefield
    Weather(Weather),
}

#[derive(Clone, Copy, Hash, Eq, PartialEq)]
pub enum Weather {
    /// Sets the strength of all [`Melee`] units to 1
    BitingFrost,

    /// Sets the strength of all [`Ranged`] units to 1
    ImpenetrableFog,

    /// Sets the strength of all [`Siege`] units to 1
    TorrentialRain,

    /// Sets the strength of all [`Ranged`] and [`Siege`] units to 1
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
    use crate::card::{Ability, Range, Strength, Unit};

    impl Unit {
        pub fn new(strength: Strength, ability: Ability, range: Range) -> Self {
            Self {
                strength,
                ability,
                range,
            }
        }
    }
}
