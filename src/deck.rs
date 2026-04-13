use rand::{rng, seq::SliceRandom};

use crate::card::{Ability, Card, Group, Range, Special, Strength, Unit, Weather};

pub struct Cards {
    hand: Vec<Card>,
    deck: Vec<Card>,
    pile: Vec<Card>,
    _side: Vec<Card>,
}

impl Cards {
    pub fn new(deck: Deck) -> Self {
        assert!(deck.size() >= 22);

        let mut cards = deck.cards;
        cards.shuffle(&mut rng());

        let (hand, remaining) = cards
            .split_first_chunk::<10>()
            .expect("should be at least 10 cards");

        Self {
            hand: hand.to_vec(),
            deck: remaining.to_vec(),
            pile: Vec::default(),
            _side: Vec::default(),
        }
    }
}

impl Cards {
    pub fn pick_card(&mut self, i: usize) -> Card {
        self.hand.swap_remove(i)
    }

    pub fn restore_from_pile(&mut self, i: usize) -> Option<Card> {
        if let Some(Card::Unit(unit)) = self.pile.get(i)
            && matches!(unit.strength, Strength::Regular(_))
        {
            Some(self.pile.swap_remove(i))
        } else {
            None
        }
    }

    pub fn pick_from_deck(&mut self, num: usize) {
        for _ in 0..num {
            if let Some(card) = self.deck.pop() {
                self.hand.push(card);
            }
        }
    }

    pub fn pick_muster(&mut self, group: Group) -> Vec<Card> {
        let mut muster = Vec::default();

        for i in self.hand.len() - 1..=0 {
            if let Some(Card::Unit(unit)) = self.hand.get(i)
                && let Ability::Muster(mg, false) = unit.ability
                && mg == group
            {
                let card = self.hand.swap_remove(i);
                muster.push(card);
            }
        }

        for i in self.deck.len() - 1..=0 {
            if let Some(Card::Unit(unit)) = self.deck.get(i)
                && let Ability::Muster(mg, false) = unit.ability
                && mg == group
            {
                let card = self.hand.swap_remove(i);
                muster.push(card);
            }
        }

        muster
    }

    pub fn add_unit(&mut self, unit: Unit) {
        self.hand.push(Card::Unit(unit));
    }
}

pub struct Deck {
    cards: Vec<Card>,
}

impl Deck {
    pub const fn new(cards: Vec<Card>) -> Self {
        assert!(cards.len() >= 22);

        Self { cards }
    }

    const fn size(&self) -> usize {
        self.cards.len()
    }
}

pub struct Library {
    neutral: Vec<Card>,
    monsters: Vec<Card>,
    nilfgaard: Vec<Card>,
    northern_realms: Vec<Card>,
    skoiatael: Vec<Card>,
    skellige: Vec<Card>,
    special: Vec<Card>,
}

#[rustfmt::skip]
impl Library {
    /// Neutral cards. Each faction can use these.
    fn neutral() -> Vec<Card> {
        let bovine_defence = Unit {
            strength: Strength::Regular(8),
            name: "Bovine Defence".to_string(),
            ability: Ability::None,
            range: Range::MELEE,
        };

        vec![
            Card::the_hero(0, "Avallac'h", Range::MELEE, Ability::Spy),
            Card::the_hero(15, "Cirilla", Range::MELEE, Ability::Muster(1, true)),
            Card::the_hero(15, "Geralt", Range::MELEE, Ability::Muster(1, true)),
            Card::the_unit(3, "Roach", Range::MELEE, Ability::Muster(1, false)),
            Card::hero(7, "Triss", Range::MELEE),
            Card::the_hero(7, "Yennefer", Range::RANGED, Ability::Medic),
            Card::the_unit(0, "Cow", Range::RANGED, Ability::Summon(Box::new(bovine_defence))),
            Card::the_unit(2, "Dandelion", Range::MELEE, Ability::CommandersHorn),
            Card::unit(5, "Emiel Regis", Range::MELEE),
            Card::the_unit(2, "Gaunter O'Dimm", Range::SIEGE, Ability::Muster(2, true)),
            Card::the_unit(4, "Gaunter O'Dimm Darkness", Range::RANGED, Ability::Muster(2, false)),
            Card::the_unit(4, "Gaunter O'Dimm Darkness", Range::RANGED, Ability::Muster(2, false)),
            Card::the_unit(4, "Gaunter O'Dimm Darkness", Range::RANGED, Ability::Muster(2, false)),
            Card::the_unit(6, "Olgierd", Range::AGILE, Ability::MoraleBoost),
            Card::unit(6, "Vesemir", Range::MELEE),
            Card::the_unit(7, "Villentretenmerth", Range::MELEE, Ability::Scorch(Range::MELEE)),
            Card::unit(5, "Zoltan", Range::MELEE),
        ]
    }

    fn monsters() -> Vec<Card> {
        vec![
            Card::hero(10, "Draug", Range::MELEE),
            Card::hero(10, "Imlerith", Range::MELEE),
            Card::the_hero(8, "Kayran", Range::AGILE, Ability::MoraleBoost),
            Card::hero(10, "Leshen", Range::RANGED),
            Card::the_unit(4, "Arachas", Range::MELEE, Ability::Muster(3, false)),
            Card::the_unit(4, "Arachas", Range::MELEE, Ability::Muster(3, false)),
            Card::the_unit(4, "Arachas", Range::MELEE, Ability::Muster(3, false)),
            Card::the_unit(6, "Arachas Behemoth", Range::SIEGE, Ability::Muster(3, true)),
            Card::unit(4, "Botchling", Range::MELEE),
            Card::unit(2, "Celaeno Harpy", Range::AGILE),
            Card::unit(2, "Cockatrice", Range::RANGED),
            Card::the_unit(6, "Crone: Brewess", Range::MELEE, Ability::Muster(4, false)),
            Card::the_unit(6, "Crone: Weavess", Range::MELEE, Ability::Muster(4, false)),
            Card::the_unit(6, "Crone: Whispess", Range::MELEE, Ability::Muster(4, false)),
            Card::unit(6, "Earth Elemental", Range::SIEGE),
            Card::unit(2, "Endrega", Range::SIEGE),
            Card::unit(6, "Fiend", Range::MELEE),
            Card::unit(6, "Fire Elemental", Range::SIEGE),
            Card::unit(2, "Foglet", Range::MELEE),
            Card::unit(5, "Forktail", Range::MELEE),
            Card::unit(5, "Frightener", Range::MELEE),
            Card::unit(2, "Gargoyle", Range::RANGED),
            Card::the_unit(1, "Ghoul", Range::MELEE, Ability::Muster(5, false)),
            Card::the_unit(1, "Ghoul", Range::MELEE, Ability::Muster(5, false)),
            Card::the_unit(1, "Ghoul", Range::MELEE, Ability::Muster(5, false)),
            Card::unit(5, "Grave Hag", Range::RANGED),
            Card::unit(5, "Griffin", Range::MELEE),
            Card::unit(2, "Harpy", Range::AGILE),
            Card::unit(5, "Ice Giant", Range::SIEGE),
            Card::the_unit(2, "Nekker", Range::MELEE, Ability::Muster(6, false)),
            Card::the_unit(2, "Nekker", Range::MELEE, Ability::Muster(6, false)),
            Card::the_unit(2, "Nekker", Range::MELEE, Ability::Muster(6, false)),
            Card::unit(5, "Plague Maiden", Range::MELEE),
            Card::the_unit(7, "Toad", Range::RANGED, Ability::Scorch(Range::RANGED)),
            Card::the_unit(4, "Vampire: Bruxa", Range::MELEE, Ability::Muster(7, false)),
            Card::the_unit(4, "Vampire: Ekkimara", Range::MELEE, Ability::Muster(7, false)),
            Card::the_unit(4, "Vampire: Fleder", Range::MELEE, Ability::Muster(7, false)),
            Card::the_unit(4, "Vampire: Garkain", Range::MELEE, Ability::Muster(7, false)),
            Card::the_unit(5, "Vampire: Katakan", Range::MELEE, Ability::Muster(7, false)),
            Card::unit(5, "Werewolf", Range::MELEE),
            Card::unit(2, "Wyvern", Range::RANGED),
        ]
    }

    fn nilfgaard() -> Vec<Card> {
        vec![
            Card::hero(10, "Letto of Gulet", Range::MELEE),
            Card::the_hero(10, "Menno Coehoorn", Range::MELEE, Ability::Medic),
            Card::hero(10, "Morvran Voorhis", Range::SIEGE),
            Card::hero(10, "Tibor Eggebracht", Range::RANGED),
            Card::unit(2, "Albrich", Range::RANGED),
            Card::unit(6, "Assire var Anahid", Range::RANGED),
            Card::unit(10, "Black Infrantry Archer", Range::RANGED),
            Card::unit(10, "Black Infrantry Archer", Range::RANGED),
            Card::unit(6, "Cahir Mawr Dyffryn", Range::MELEE),
            Card::unit(4, "Cynthia", Range::RANGED),
            Card::the_unit(1, "Etolian Auxiliary Archers", Range::RANGED, Ability::Medic),
            Card::the_unit(1, "Etolian Auxiliary Archers", Range::RANGED, Ability::Medic),
            Card::unit(6, "Fringilla Vigo", Range::RANGED),
            Card::unit(10, "Heavy Fire Scorpion", Range::SIEGE),
            Card::the_unit(3, "Impera Brigade Guard", Range::MELEE, Ability::TightBond(1)),
            Card::the_unit(3, "Impera Brigade Guard", Range::MELEE, Ability::TightBond(1)),
            Card::the_unit(3, "Impera Brigade Guard", Range::MELEE, Ability::TightBond(1)),
            Card::the_unit(3, "Impera Brigade Guard", Range::MELEE, Ability::TightBond(1)),
            Card::unit(3, "Morteisen", Range::MELEE),
            Card::the_unit(2, "Nausicaa Cavalry Rider", Range::MELEE, Ability::TightBond(2)),
            Card::the_unit(2, "Nausicaa Cavalry Rider", Range::MELEE, Ability::TightBond(2)),
            Card::the_unit(2, "Nausicaa Cavalry Rider", Range::MELEE, Ability::TightBond(2)),
            Card::unit(3, "Puttkammer", Range::RANGED),
            Card::unit(4, "Rainfarn", Range::MELEE),
            Card::unit(5, "Renuald Aep Matsen", Range::RANGED),
            Card::unit(3, "Rotten Mangonel", Range::SIEGE),
            Card::the_unit(7, "Shilard Fitz-Oesterlen", Range::MELEE, Ability::Spy),
            Card::unit(6, "Siege Engineer", Range::SIEGE),
            Card::the_unit(0, "Siege Technician", Range::SIEGE, Ability::Medic),
            Card::the_unit(9, "Stefan Skellen", Range::MELEE, Ability::Spy),
            Card::unit(2, "Sweers", Range::RANGED),
            Card::unit(4, "Vanhemar", Range::RANGED),
            Card::the_unit(4, "Vattier de Rideaux", Range::MELEE, Ability::Spy),
            Card::unit(2, "Vreemde", Range::MELEE),
            Card::the_unit(5, "Young Emissary", Range::MELEE, Ability::TightBond(3)),
            Card::the_unit(5, "Young Emissary", Range::MELEE, Ability::TightBond(3)),
            Card::unit(5, "Fire Scorpion", Range::SIEGE),
        ]
    }

    fn northern_realms() -> Vec<Card> {
        vec![
            Card::hero(10, "Esterad Thyssen", Range::MELEE),
            Card::hero(10, "John Natalis", Range::MELEE),
            Card::hero(10, "Philippa Eilhart", Range::RANGED),
            Card::hero(10, "Vernon Roche", Range::MELEE),
            Card::unit(6, "Ballista", Range::SIEGE),
            Card::unit(6, "Ballista", Range::SIEGE),
            Card::the_unit(4, "Blue Stripes Commando", Range::MELEE, Ability::TightBond(4)),
            Card::the_unit(4, "Blue Stripes Commando", Range::MELEE, Ability::TightBond(4)),
            Card::the_unit(4, "Blue Stripes Commando", Range::MELEE, Ability::TightBond(4)),
            Card::the_unit(8, "Catapult", Range::SIEGE, Ability::TightBond(5)),
            Card::the_unit(8, "Catapult", Range::SIEGE, Ability::TightBond(5)),
            Card::the_unit(5, "Dragon Hunter", Range::RANGED, Ability::TightBond(6)),
            Card::the_unit(5, "Dragon Hunter", Range::RANGED, Ability::TightBond(6)),
            Card::the_unit(5, "Dragon Hunter", Range::RANGED, Ability::TightBond(6)),
            Card::unit(6, "Dethmold", Range::RANGED),
            Card::the_unit(5, "Dun Banner Medic", Range::SIEGE, Ability::Medic),
            Card::the_unit(1, "Siege Expert", Range::SIEGE, Ability::MoraleBoost),
            Card::the_unit(1, "Siege Expert", Range::SIEGE, Ability::MoraleBoost),
            Card::the_unit(1, "Siege Expert", Range::SIEGE, Ability::MoraleBoost),
            Card::unit(5, "Keira Metz", Range::RANGED),
            Card::the_unit(1, "Poor Fucking Infrantry", Range::MELEE, Ability::TightBond(7)),
            Card::the_unit(1, "Poor Fucking Infrantry", Range::MELEE, Ability::TightBond(7)),
            Card::the_unit(1, "Poor Fucking Infrantry", Range::MELEE, Ability::TightBond(7)),
            Card::the_unit(5, "Prince Stennis", Range::MELEE, Ability::Spy),
            Card::unit(1, "Redanian Foot Soldier", Range::MELEE),
            Card::unit(1, "Redanian Foot Soldier", Range::MELEE),
            Card::unit(4, "Sabrina Glevissig", Range::RANGED),
            Card::unit(4, "Sheldon Skaggs", Range::RANGED),
            Card::unit(6, "Siege Tower", Range::SIEGE),
            Card::unit(5, "Siegfried of Denesle", Range::MELEE),
            Card::the_unit(4, "Dijkstra", Range::MELEE, Ability::Spy),
            Card::unit(5, "Sile de Tansarville", Range::RANGED),
            Card::the_unit(1, "Thaler", Range::SIEGE, Ability::Spy),
            Card::unit(6, "Trebuchet", Range::SIEGE),
            Card::unit(6, "Trebuchet", Range::SIEGE),
            Card::unit(5, "Ves", Range::MELEE),
            Card::unit(2, "Yarpen Zigrin", Range::MELEE),
        ]
    }

    fn skoiatael() -> Vec<Card> {
        vec![
            Card::hero(10, "Eithne", Range::RANGED),
            Card::hero(10, "Iorveth", Range::RANGED),
            Card::the_hero(10, "Isengrim", Range::MELEE, Ability::MoraleBoost),
            Card::hero(10, "Saesenthessis", Range::RANGED),
            Card::unit(6, "Barclay Els", Range::AGILE),
            Card::unit(3, "Ciaran Aep Easnillien", Range::AGILE),
            Card::unit(6, "Dennis Cranmer", Range::MELEE),
            Card::unit(4, "Dol Blathanna Archer", Range::RANGED),
            Card::unit(6, "Dol Blathanna Scout", Range::AGILE),
            Card::unit(6, "Dol Blathanna Scout", Range::AGILE),
            Card::unit(6, "Dol Blathanna Scout", Range::AGILE),
            Card::the_unit(3, "Dwarven Skirmisher", Range::MELEE, Ability::Muster(8, false)),
            Card::the_unit(3, "Dwarven Skirmisher", Range::MELEE, Ability::Muster(8, false)),
            Card::the_unit(3, "Dwarven Skirmisher", Range::MELEE, Ability::Muster(8, false)),
            Card::the_unit(2, "Elven Skirmisher", Range::RANGED, Ability::Muster(9, false)),
            Card::the_unit(2, "Elven Skirmisher", Range::RANGED, Ability::Muster(9, false)),
            Card::the_unit(2, "Elven Skirmisher", Range::RANGED, Ability::Muster(9, false)),
            Card::unit(6, "Filavandrel Aen Fidhail", Range::AGILE),
            Card::the_unit(0, "Havekar Healer", Range::RANGED, Ability::Medic),
            Card::the_unit(0, "Havekar Healer", Range::RANGED, Ability::Medic),
            Card::the_unit(0, "Havekar Healer", Range::RANGED, Ability::Medic),
            Card::the_unit(5, "Havekar Smuggler", Range::MELEE, Ability::Muster(10, false)),
            Card::the_unit(5, "Havekar Smuggler", Range::MELEE, Ability::Muster(10, false)),
            Card::the_unit(5, "Havekar Smuggler", Range::MELEE, Ability::Muster(10, false)),
            Card::unit(6, "Ida Emean Aep Sivney", Range::RANGED),
            Card::unit(5, "Mahakaman Defender", Range::MELEE),
            Card::unit(5, "Mahakaman Defender", Range::MELEE),
            Card::unit(5, "Mahakaman Defender", Range::MELEE),
            Card::unit(5, "Mahakaman Defender", Range::MELEE),
            Card::unit(5, "Mahakaman Defender", Range::MELEE),
            Card::the_unit(10, "Milva", Range::RANGED, Ability::MoraleBoost),
            Card::unit(1, "Riordain", Range::RANGED),
            Card::the_unit(8, "Schirru", Range::SIEGE, Ability::Scorch(Range::SIEGE)),
            Card::unit(2, "Toruviel", Range::RANGED),
            Card::unit(4, "Vrihedd Brigade Recruit", Range::RANGED),
            Card::unit(5, "Vrihedd Brigade Veteran", Range::AGILE),
            Card::unit(5, "Vrihedd Brigade Veteran", Range::AGILE),
            Card::unit(6, "Yaevinn", Range::AGILE),
        ]
    }

    fn skellige() -> Vec<Card> {
        let vildkaarl = Unit {
            strength: Strength::Regular(14),
            name: "Vildkaarl".to_string(),
            ability: Ability::MoraleBoost,
            range: Range::MELEE,
        };

        let hemdall = Unit {
            strength: Strength::Hero(11),
            name: "Hemdall".to_string(),
            ability: Ability::None,
            range: Range::MELEE,
        };

        let young_vildkaarl = Unit {
            strength: Strength::Regular(8),
            name: "Young Vildkaarl".to_string(),
            ability: Ability::TightBond(11),
            range: Range::RANGED,
        };

        vec![
            Card::the_hero(10, "Cerys", Range::MELEE, Ability::Muster(11, false)), // FIXME
            Card::the_hero(8, "Ermion", Range::RANGED, Ability::Mardrome),
            Card::hero(10, "Hjalmar", Range::RANGED),
            Card::the_unit(4, "Berserker", Range::MELEE, Ability::Berserker(Box::new(vildkaarl))),
            Card::the_unit(2, "Birna Bran", Range::MELEE, Ability::Medic),
            Card::unit(6, "Blueboy Lugos", Range::MELEE),
            Card::the_unit(6, "Clan an Craite Warrior", Range::MELEE, Ability::TightBond(8)),
            Card::the_unit(6, "Clan an Craite Warrior", Range::MELEE, Ability::TightBond(8)),
            Card::the_unit(6, "Clan an Craite Warrior", Range::MELEE, Ability::TightBond(8)),
            Card::unit(6, "Clan Brokvar Archer", Range::RANGED),
            Card::unit(6, "Clan Brokvar Archer", Range::RANGED),
            Card::unit(6, "Clan Brokvar Archer", Range::RANGED),
            Card::the_unit(6, "Clan Dimun Pirate", Range::RANGED, Ability::Scorch(Range::ALL)),
            Card::the_unit(4, "Clan Drummond Shieldmaiden", Range::MELEE, Ability::TightBond(9)),
            Card::the_unit(4, "Clan Drummond Shieldmaiden", Range::MELEE, Ability::TightBond(9)),
            Card::the_unit(4, "Clan Drummond Shieldmaiden", Range::MELEE, Ability::TightBond(9)),
            Card::unit(4, "Clan Heymaey Skald", Range::MELEE),
            Card::unit(4, "Clan Tordarroch Armorsmith", Range::MELEE),
            Card::unit(4, "Donar an Hindar", Range::MELEE),
            Card::the_unit(2, "Draig Bon-Dhu", Range::SIEGE, Ability::CommandersHorn),
            Card::unit(4, "Holger Blackhand", Range::SIEGE),
            Card::the_unit(0, "Kambi", Range::MELEE, Ability::Summon(Box::new(hemdall))),
            Card::the_unit(4, "Light Longship", Range::RANGED, Ability::Muster(12, false)),
            Card::the_unit(4, "Light Longship", Range::RANGED, Ability::Muster(12, false)),
            Card::the_unit(4, "Light Longship", Range::RANGED, Ability::Muster(12, false)),
            Card::unit(6, "Madman Lugos", Range::MELEE),
            Card::the_unit(12, "Olaf", Range::AGILE, Ability::MoraleBoost),
            Card::unit(4, "Svanrige", Range::MELEE),
            Card::unit(4, "Udalryk", Range::MELEE),
            Card::the_unit(6, "War Longship", Range::SIEGE, Ability::TightBond(10)),
            Card::the_unit(6, "War Longship", Range::SIEGE, Ability::TightBond(10)),
            Card::the_unit(6, "War Longship", Range::SIEGE, Ability::TightBond(10)),
            Card::the_unit(2, "Young Berserker", Range::RANGED, Ability::Berserker(Box::new(young_vildkaarl.clone()))),
            Card::the_unit(2, "Young Berserker", Range::RANGED, Ability::Berserker(Box::new(young_vildkaarl.clone()))),
            Card::the_unit(2, "Young Berserker", Range::RANGED, Ability::Berserker(Box::new(young_vildkaarl))),
        ]
    }

    fn special() -> Vec<Card> {
        vec![
            Card::Special(Special::Weather(Weather::BitingFrost)),
            Card::Special(Special::Weather(Weather::BitingFrost)),
            Card::Special(Special::Weather(Weather::BitingFrost)),
            Card::Special(Special::Weather(Weather::ClearWeather)),
            Card::Special(Special::Weather(Weather::ClearWeather)),
            Card::Special(Special::Weather(Weather::ClearWeather)),
            Card::Special(Special::Weather(Weather::ImpenetrableFog)),
            Card::Special(Special::Weather(Weather::ImpenetrableFog)),
            Card::Special(Special::Weather(Weather::ImpenetrableFog)),
            Card::Special(Special::Weather(Weather::SkelligeStorm)),
            Card::Special(Special::Weather(Weather::SkelligeStorm)),
            Card::Special(Special::Weather(Weather::SkelligeStorm)),
            Card::Special(Special::Weather(Weather::TorrentialRain)),
            Card::Special(Special::Weather(Weather::TorrentialRain)),
            Card::Special(Special::Weather(Weather::TorrentialRain)),
            Card::Special(Special::CommandersHorn),
            Card::Special(Special::CommandersHorn),
            Card::Special(Special::CommandersHorn),
            Card::Special(Special::Decoy),
            Card::Special(Special::Decoy),
            Card::Special(Special::Decoy),
            Card::Special(Special::Scorch),
            Card::Special(Special::Scorch),
            Card::Special(Special::Scorch),
        ]
    }
}

impl Default for Library {
    fn default() -> Self {
        Self {
            neutral: Self::neutral(),
            monsters: Self::monsters(),
            nilfgaard: Self::nilfgaard(),
            northern_realms: Self::northern_realms(),
            skoiatael: Self::skoiatael(),
            skellige: Self::skellige(),
            special: Self::special(),
        }
    }
}
