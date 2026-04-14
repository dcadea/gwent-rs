use std::sync::OnceLock;

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

    pub fn pick_muster(&mut self, ids: &[u16]) -> Vec<Card> {
        let mut muster = Vec::default();

        for i in self.hand.len() - 1..=0 {
            if let Some(Card::Unit(unit)) = self.hand.get(i)
                && ids.contains(&unit.id)
            {
                let card = self.hand.swap_remove(i);
                muster.push(card);
            }
        }

        for i in self.deck.len() - 1..=0 {
            if let Some(Card::Unit(unit)) = self.deck.get(i)
                && ids.contains(&unit.id)
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
            id: 18,
            strength: Strength::Regular(8),
            name: "Bovine Defence".to_string(),
            ability: Ability::None,
            range: Range::MELEE,
        };

        vec![
            Card::the_hero(1, 0, "Avallac'h", Range::MELEE, Ability::Spy),
            Card::the_hero(2, 15, "Cirilla", Range::MELEE, Ability::Muster(vec![4])),
            Card::the_hero(3, 15, "Geralt", Range::MELEE, Ability::Muster(vec![4])),
            Card::unit(4, 3, "Roach", Range::MELEE),
            Card::hero(5, 7, "Triss", Range::MELEE),
            Card::the_hero(6, 7, "Yennefer", Range::RANGED, Ability::Medic),
            Card::the_unit(7, 0, "Cow", Range::RANGED, Ability::Summon(Box::new(bovine_defence))),
            Card::the_unit(8, 2, "Dandelion", Range::MELEE, Ability::CommandersHorn),
            Card::unit(9, 5, "Emiel Regis", Range::MELEE),
            Card::the_unit(10, 2, "Gaunter O'Dimm", Range::SIEGE, Ability::Muster(vec![11, 12, 13])),
            Card::the_unit(11, 4, "Gaunter O'Dimm Darkness", Range::RANGED, Ability::Muster(vec![12, 13])),
            Card::the_unit(12, 4, "Gaunter O'Dimm Darkness", Range::RANGED, Ability::Muster(vec![11, 13])),
            Card::the_unit(13, 4, "Gaunter O'Dimm Darkness", Range::RANGED, Ability::Muster(vec![11, 12])),
            Card::the_unit(14, 6, "Olgierd", Range::AGILE, Ability::MoraleBoost),
            Card::unit(15, 6, "Vesemir", Range::MELEE),
            Card::the_unit(16, 7, "Villentretenmerth", Range::MELEE, Ability::Scorch(Range::MELEE)),
            Card::unit(17, 5, "Zoltan", Range::MELEE),
        ]
    }

    fn monsters() -> Vec<Card> {
        vec![
            Card::hero(19, 10, "Draug", Range::MELEE),
            Card::hero(20, 10, "Imlerith", Range::MELEE),
            Card::the_hero(21, 8, "Kayran", Range::AGILE, Ability::MoraleBoost),
            Card::hero(22, 10, "Leshen", Range::RANGED),
            Card::the_unit(23, 4, "Arachas", Range::MELEE, Ability::Muster(vec![24, 25])),
            Card::the_unit(24, 4, "Arachas", Range::MELEE, Ability::Muster(vec![23, 25])),
            Card::the_unit(25, 4, "Arachas", Range::MELEE, Ability::Muster(vec![23, 24])),
            Card::the_unit(26, 6, "Arachas Behemoth", Range::SIEGE, Ability::Muster(vec![23, 24, 25])),
            Card::unit(27, 4, "Botchling", Range::MELEE),
            Card::unit(28, 2, "Celaeno Harpy", Range::AGILE),
            Card::unit(29, 2, "Cockatrice", Range::RANGED),
            Card::the_unit(30, 6, "Crone: Brewess", Range::MELEE, Ability::Muster(vec![31, 32])),
            Card::the_unit(31, 6, "Crone: Weavess", Range::MELEE, Ability::Muster(vec![30, 32])),
            Card::the_unit(32, 6, "Crone: Whispess", Range::MELEE, Ability::Muster(vec![30, 31])),
            Card::unit(33, 6, "Earth Elemental", Range::SIEGE),
            Card::unit(34, 2, "Endrega", Range::SIEGE),
            Card::unit(35, 6, "Fiend", Range::MELEE),
            Card::unit(36, 6, "Fire Elemental", Range::SIEGE),
            Card::unit(37, 2, "Foglet", Range::MELEE),
            Card::unit(38, 5, "Forktail", Range::MELEE),
            Card::unit(39, 5, "Frightener", Range::MELEE),
            Card::unit(40, 2, "Gargoyle", Range::RANGED),
            Card::the_unit(41, 1, "Ghoul", Range::MELEE, Ability::Muster(vec![42, 43])),
            Card::the_unit(42, 1, "Ghoul", Range::MELEE, Ability::Muster(vec![41, 43])),
            Card::the_unit(43, 1, "Ghoul", Range::MELEE, Ability::Muster(vec![41, 42])),
            Card::unit(44, 5, "Grave Hag", Range::RANGED),
            Card::unit(45, 5, "Griffin", Range::MELEE),
            Card::unit(46, 2, "Harpy", Range::AGILE),
            Card::unit(47, 5, "Ice Giant", Range::SIEGE),
            Card::the_unit(48, 2, "Nekker", Range::MELEE, Ability::Muster(vec![49, 50])),
            Card::the_unit(49, 2, "Nekker", Range::MELEE, Ability::Muster(vec![48, 50])),
            Card::the_unit(50, 2, "Nekker", Range::MELEE, Ability::Muster(vec![48, 49])),
            Card::unit(51, 5, "Plague Maiden", Range::MELEE),
            Card::the_unit(52, 7, "Toad", Range::RANGED, Ability::Scorch(Range::RANGED)),
            Card::the_unit(53, 4, "Vampire: Bruxa", Range::MELEE, Ability::Muster(vec![54, 55, 56, 57])),
            Card::the_unit(54, 4, "Vampire: Ekkimara", Range::MELEE, Ability::Muster(vec![53, 55, 56, 57])),
            Card::the_unit(55, 4, "Vampire: Fleder", Range::MELEE, Ability::Muster(vec![53, 54, 56, 57])),
            Card::the_unit(56, 4, "Vampire: Garkain", Range::MELEE, Ability::Muster(vec![53, 54, 55, 57])),
            Card::the_unit(57, 5, "Vampire: Katakan", Range::MELEE, Ability::Muster(vec![53, 54, 55, 56])),
            Card::unit(58, 5, "Werewolf", Range::MELEE),
            Card::unit(59, 2, "Wyvern", Range::RANGED),
        ]
    }

    fn nilfgaard() -> Vec<Card> {
        vec![
            Card::hero(60, 10, "Letto of Gulet", Range::MELEE),
            Card::the_hero(61, 10, "Menno Coehoorn", Range::MELEE, Ability::Medic),
            Card::hero(62, 10, "Morvran Voorhis", Range::SIEGE),
            Card::hero(63, 10, "Tibor Eggebracht", Range::RANGED),
            Card::unit(64, 2, "Albrich", Range::RANGED),
            Card::unit(65, 6, "Assire var Anahid", Range::RANGED),
            Card::unit(66, 10, "Black Infrantry Archer", Range::RANGED),
            Card::unit(67, 10, "Black Infrantry Archer", Range::RANGED),
            Card::unit(68, 6, "Cahir Mawr Dyffryn", Range::MELEE),
            Card::unit(69, 4, "Cynthia", Range::RANGED),
            Card::the_unit(70, 1, "Etolian Auxiliary Archers", Range::RANGED, Ability::Medic),
            Card::the_unit(71, 1, "Etolian Auxiliary Archers", Range::RANGED, Ability::Medic),
            Card::unit(72, 6, "Fringilla Vigo", Range::RANGED),
            Card::unit(73, 10, "Heavy Fire Scorpion", Range::SIEGE),
            Card::the_unit(74, 3, "Impera Brigade Guard", Range::MELEE, Ability::TightBond(1)),
            Card::the_unit(75, 3, "Impera Brigade Guard", Range::MELEE, Ability::TightBond(1)),
            Card::the_unit(76, 3, "Impera Brigade Guard", Range::MELEE, Ability::TightBond(1)),
            Card::the_unit(77, 3, "Impera Brigade Guard", Range::MELEE, Ability::TightBond(1)),
            Card::unit(78, 3, "Morteisen", Range::MELEE),
            Card::the_unit(79, 2, "Nausicaa Cavalry Rider", Range::MELEE, Ability::TightBond(2)),
            Card::the_unit(80, 2, "Nausicaa Cavalry Rider", Range::MELEE, Ability::TightBond(2)),
            Card::the_unit(81, 2, "Nausicaa Cavalry Rider", Range::MELEE, Ability::TightBond(2)),
            Card::unit(82, 3, "Puttkammer", Range::RANGED),
            Card::unit(83, 4, "Rainfarn", Range::MELEE),
            Card::unit(84, 5, "Renuald Aep Matsen", Range::RANGED),
            Card::unit(85, 3, "Rotten Mangonel", Range::SIEGE),
            Card::the_unit(86, 7, "Shilard Fitz-Oesterlen", Range::MELEE, Ability::Spy),
            Card::unit(87, 6, "Siege Engineer", Range::SIEGE),
            Card::the_unit(88, 0, "Siege Technician", Range::SIEGE, Ability::Medic),
            Card::the_unit(89, 9, "Stefan Skellen", Range::MELEE, Ability::Spy),
            Card::unit(90, 2, "Sweers", Range::RANGED),
            Card::unit(91, 4, "Vanhemar", Range::RANGED),
            Card::the_unit(92, 4, "Vattier de Rideaux", Range::MELEE, Ability::Spy),
            Card::unit(93, 2, "Vreemde", Range::MELEE),
            Card::the_unit(94, 5, "Young Emissary", Range::MELEE, Ability::TightBond(3)),
            Card::the_unit(95, 5, "Young Emissary", Range::MELEE, Ability::TightBond(3)),
            Card::unit(96, 5, "Fire Scorpion", Range::SIEGE),
        ]
    }

    fn northern_realms() -> Vec<Card> {
        vec![
            Card::hero(97, 10, "Esterad Thyssen", Range::MELEE),
            Card::hero(98, 10, "John Natalis", Range::MELEE),
            Card::hero(99, 10, "Philippa Eilhart", Range::RANGED),
            Card::hero(100, 10, "Vernon Roche", Range::MELEE),
            Card::unit(101, 6, "Ballista", Range::SIEGE),
            Card::unit(102, 6, "Ballista", Range::SIEGE),
            Card::the_unit(103, 4, "Blue Stripes Commando", Range::MELEE, Ability::TightBond(4)),
            Card::the_unit(104, 4, "Blue Stripes Commando", Range::MELEE, Ability::TightBond(4)),
            Card::the_unit(105, 4, "Blue Stripes Commando", Range::MELEE, Ability::TightBond(4)),
            Card::the_unit(106, 8, "Catapult", Range::SIEGE, Ability::TightBond(5)),
            Card::the_unit(107, 8, "Catapult", Range::SIEGE, Ability::TightBond(5)),
            Card::the_unit(108, 5, "Dragon Hunter", Range::RANGED, Ability::TightBond(6)),
            Card::the_unit(109, 5, "Dragon Hunter", Range::RANGED, Ability::TightBond(6)),
            Card::the_unit(110, 5, "Dragon Hunter", Range::RANGED, Ability::TightBond(6)),
            Card::unit(111, 6, "Dethmold", Range::RANGED),
            Card::the_unit(112, 5, "Dun Banner Medic", Range::SIEGE, Ability::Medic),
            Card::the_unit(113, 1, "Siege Expert", Range::SIEGE, Ability::MoraleBoost),
            Card::the_unit(114, 1, "Siege Expert", Range::SIEGE, Ability::MoraleBoost),
            Card::the_unit(115, 1, "Siege Expert", Range::SIEGE, Ability::MoraleBoost),
            Card::unit(116, 5, "Keira Metz", Range::RANGED),
            Card::the_unit(117, 1, "Poor Fucking Infrantry", Range::MELEE, Ability::TightBond(7)),
            Card::the_unit(118, 1, "Poor Fucking Infrantry", Range::MELEE, Ability::TightBond(7)),
            Card::the_unit(119, 1, "Poor Fucking Infrantry", Range::MELEE, Ability::TightBond(7)),
            Card::the_unit(120, 5, "Prince Stennis", Range::MELEE, Ability::Spy),
            Card::unit(121, 1, "Redanian Foot Soldier", Range::MELEE),
            Card::unit(122, 1, "Redanian Foot Soldier", Range::MELEE),
            Card::unit(123, 4, "Sabrina Glevissig", Range::RANGED),
            Card::unit(124, 4, "Sheldon Skaggs", Range::RANGED),
            Card::unit(125, 6, "Siege Tower", Range::SIEGE),
            Card::unit(126, 5, "Siegfried of Denesle", Range::MELEE),
            Card::the_unit(127, 4, "Dijkstra", Range::MELEE, Ability::Spy),
            Card::unit(128, 5, "Sile de Tansarville", Range::RANGED),
            Card::the_unit(129, 1, "Thaler", Range::SIEGE, Ability::Spy),
            Card::unit(130, 6, "Trebuchet", Range::SIEGE),
            Card::unit(131, 6, "Trebuchet", Range::SIEGE),
            Card::unit(132, 5, "Ves", Range::MELEE),
            Card::unit(133, 2, "Yarpen Zigrin", Range::MELEE),
        ]
    }

    fn skoiatael() -> Vec<Card> {
        vec![
            Card::hero(134, 10, "Eithne", Range::RANGED),
            Card::hero(135, 10, "Iorveth", Range::RANGED),
            Card::the_hero(136, 10, "Isengrim", Range::MELEE, Ability::MoraleBoost),
            Card::hero(137, 10, "Saesenthessis", Range::RANGED),
            Card::unit(138, 6, "Barclay Els", Range::AGILE),
            Card::unit(139, 3, "Ciaran Aep Easnillien", Range::AGILE),
            Card::unit(140, 6, "Dennis Cranmer", Range::MELEE),
            Card::unit(141, 4, "Dol Blathanna Archer", Range::RANGED),
            Card::unit(142, 6, "Dol Blathanna Scout", Range::AGILE),
            Card::unit(143, 6, "Dol Blathanna Scout", Range::AGILE),
            Card::unit(144, 6, "Dol Blathanna Scout", Range::AGILE),
            Card::the_unit(145, 3, "Dwarven Skirmisher", Range::MELEE, Ability::Muster(vec![146, 147])),
            Card::the_unit(146, 3, "Dwarven Skirmisher", Range::MELEE, Ability::Muster(vec![145, 147])),
            Card::the_unit(147, 3, "Dwarven Skirmisher", Range::MELEE, Ability::Muster(vec![145, 146])),
            Card::the_unit(148, 2, "Elven Skirmisher", Range::RANGED, Ability::Muster(vec![149, 150])),
            Card::the_unit(149, 2, "Elven Skirmisher", Range::RANGED, Ability::Muster(vec![148, 150])),
            Card::the_unit(150, 2, "Elven Skirmisher", Range::RANGED, Ability::Muster(vec![148, 149])),
            Card::unit(151, 6, "Filavandrel Aen Fidhail", Range::AGILE),
            Card::the_unit(152, 0, "Havekar Healer", Range::RANGED, Ability::Medic),
            Card::the_unit(153, 0, "Havekar Healer", Range::RANGED, Ability::Medic),
            Card::the_unit(154, 0, "Havekar Healer", Range::RANGED, Ability::Medic),
            Card::the_unit(155, 5, "Havekar Smuggler", Range::MELEE, Ability::Muster(vec![156, 157])),
            Card::the_unit(156, 5, "Havekar Smuggler", Range::MELEE, Ability::Muster(vec![155, 157])),
            Card::the_unit(157, 5, "Havekar Smuggler", Range::MELEE, Ability::Muster(vec![155, 156])),
            Card::unit(158, 6, "Ida Emean Aep Sivney", Range::RANGED),
            Card::unit(159, 5, "Mahakaman Defender", Range::MELEE),
            Card::unit(160, 5, "Mahakaman Defender", Range::MELEE),
            Card::unit(161, 5, "Mahakaman Defender", Range::MELEE),
            Card::unit(162, 5, "Mahakaman Defender", Range::MELEE),
            Card::unit(163, 5, "Mahakaman Defender", Range::MELEE),
            Card::the_unit(164, 10, "Milva", Range::RANGED, Ability::MoraleBoost),
            Card::unit(165, 1, "Riordain", Range::RANGED),
            Card::the_unit(166, 8, "Schirru", Range::SIEGE, Ability::Scorch(Range::SIEGE)),
            Card::unit(167, 2, "Toruviel", Range::RANGED),
            Card::unit(168, 4, "Vrihedd Brigade Recruit", Range::RANGED),
            Card::unit(169, 5, "Vrihedd Brigade Veteran", Range::AGILE),
            Card::unit(170, 5, "Vrihedd Brigade Veteran", Range::AGILE),
            Card::unit(171, 6, "Yaevinn", Range::AGILE),
        ]
    }

    fn skellige() -> Vec<Card> {
        let vildkaarl = Unit {
            id: 207,
            strength: Strength::Regular(14),
            name: "Vildkaarl".to_string(),
            ability: Ability::MoraleBoost,
            range: Range::MELEE,
        };

        let hemdall = Unit {
            id: 208,
            strength: Strength::Hero(11),
            name: "Hemdall".to_string(),
            ability: Ability::None,
            range: Range::MELEE,
        };

        let young_vildkaarl = Unit {
            id: 209,
            strength: Strength::Regular(8),
            name: "Young Vildkaarl".to_string(),
            ability: Ability::TightBond(11),
            range: Range::RANGED,
        };

        vec![
            Card::the_hero(172, 10, "Cerys", Range::MELEE, Ability::Muster(vec![185, 186, 187])),
            Card::the_hero(173, 8, "Ermion", Range::RANGED, Ability::Mardrome),
            Card::hero(174, 10, "Hjalmar", Range::RANGED),
            Card::the_unit(175, 4, "Berserker", Range::MELEE, Ability::Berserker(Box::new(vildkaarl))),
            Card::the_unit(176, 2, "Birna Bran", Range::MELEE, Ability::Medic),
            Card::unit(177, 6, "Blueboy Lugos", Range::MELEE),
            Card::the_unit(178, 6, "Clan an Craite Warrior", Range::MELEE, Ability::TightBond(8)),
            Card::the_unit(179, 6, "Clan an Craite Warrior", Range::MELEE, Ability::TightBond(8)),
            Card::the_unit(180, 6, "Clan an Craite Warrior", Range::MELEE, Ability::TightBond(8)),
            Card::unit(181, 6, "Clan Brokvar Archer", Range::RANGED),
            Card::unit(182, 6, "Clan Brokvar Archer", Range::RANGED),
            Card::unit(183, 6, "Clan Brokvar Archer", Range::RANGED),
            Card::the_unit(184, 6, "Clan Dimun Pirate", Range::RANGED, Ability::Scorch(Range::ALL)),
            Card::the_unit(185, 4, "Clan Drummond Shieldmaiden", Range::MELEE, Ability::TightBond(9)),
            Card::the_unit(186, 4, "Clan Drummond Shieldmaiden", Range::MELEE, Ability::TightBond(9)),
            Card::the_unit(187, 4, "Clan Drummond Shieldmaiden", Range::MELEE, Ability::TightBond(9)),
            Card::unit(188, 4, "Clan Heymaey Skald", Range::MELEE),
            Card::unit(189, 4, "Clan Tordarroch Armorsmith", Range::MELEE),
            Card::unit(190, 4, "Donar an Hindar", Range::MELEE),
            Card::the_unit(191, 2, "Draig Bon-Dhu", Range::SIEGE, Ability::CommandersHorn),
            Card::unit(192, 4, "Holger Blackhand", Range::SIEGE),
            Card::the_unit(193, 0, "Kambi", Range::MELEE, Ability::Summon(Box::new(hemdall))),
            Card::the_unit(194, 4, "Light Longship", Range::RANGED, Ability::Muster(vec![195, 196])),
            Card::the_unit(195, 4, "Light Longship", Range::RANGED, Ability::Muster(vec![194, 196])),
            Card::the_unit(196, 4, "Light Longship", Range::RANGED, Ability::Muster(vec![194, 195])),
            Card::unit(197, 6, "Madman Lugos", Range::MELEE),
            Card::the_unit(198, 12, "Olaf", Range::AGILE, Ability::MoraleBoost),
            Card::unit(199, 4, "Svanrige", Range::MELEE),
            Card::unit(200, 4, "Udalryk", Range::MELEE),
            Card::the_unit(201, 6, "War Longship", Range::SIEGE, Ability::TightBond(10)),
            Card::the_unit(202, 6, "War Longship", Range::SIEGE, Ability::TightBond(10)),
            Card::the_unit(203, 6, "War Longship", Range::SIEGE, Ability::TightBond(10)),
            Card::the_unit(204, 2, "Young Berserker", Range::RANGED, Ability::Berserker(Box::new(young_vildkaarl.clone()))),
            Card::the_unit(205, 2, "Young Berserker", Range::RANGED, Ability::Berserker(Box::new(young_vildkaarl.clone()))),
            Card::the_unit(206, 2, "Young Berserker", Range::RANGED, Ability::Berserker(Box::new(young_vildkaarl))),
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
