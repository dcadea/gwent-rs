#[allow(clippy::wildcard_imports)]
use crate::constants::*;

use std::collections::HashMap;

use rand::{rng, seq::SliceRandom};

use crate::card::{Ability, Card, Range, Special, Strength, Unit, Weather};

enum Faction {
    Monsters,
    Nilfgaard,
    NorthernRealms,
    Skoiatael,
    Skellige,
}

pub struct Cards {
    hand: Vec<Card>,
    deck: Vec<Card>,
    pile: Vec<Card>,
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
        }
    }
}

impl Cards {
    pub const fn is_hand_empty(&self) -> bool {
        self.hand.is_empty()
    }

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

        for i in (0..self.hand.len()).rev() {
            if let Some(Card::Unit(unit)) = self.hand.get(i)
                && ids.contains(&unit.id)
            {
                let card = self.hand.swap_remove(i);
                muster.push(card);
            }
        }

        for i in (0..self.deck.len()).rev() {
            if let Some(Card::Unit(unit)) = self.deck.get(i)
                && ids.contains(&unit.id)
            {
                let card = self.deck.swap_remove(i);
                muster.push(card);
            }
        }

        muster
    }

    pub fn add_unit(&mut self, unit: Unit) {
        self.hand.push(Card::Unit(unit));
    }

    pub fn discard(&mut self, unit: Unit) {
        self.pile.push(Card::Unit(unit));
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
    neutral: HashMap<u16, Card>,
    monsters: HashMap<u16, Card>,
    nilfgaard: HashMap<u16, Card>,
    northern_realms: HashMap<u16, Card>,
    skoiatael: HashMap<u16, Card>,
    skellige: HashMap<u16, Card>,
    special: HashMap<u16, Vec<Card>>,
}

#[rustfmt::skip]
impl Library {
    /// Neutral cards. Each faction can use these.
    fn neutral() -> HashMap<u16, Card> {
        let mut cards = HashMap::new();
        let mut add = |card: Card| { cards.insert(card.id(), card); };

        let bovine_defence = Unit {
            id: BOVINE_DEFENCE,
            strength: Strength::Regular(8),
            name: "Bovine Defence".to_string(),
            ability: Ability::None,
            range: Range::MELEE,
        };

        add(Card::the_hero(AVALLACH, 0, "Avallac'h", Range::MELEE, Ability::Spy));

        add(Card::the_hero(CIRILLA, 15, "Cirilla", Range::MELEE, Ability::Muster(vec![ROACH])));
        add(Card::the_hero(GERALT, 15, "Geralt", Range::MELEE, Ability::Muster(vec![ROACH])));
        add(Card::unit(ROACH, 3, "Roach", Range::MELEE));

        add(Card::hero(TRISS, 7, "Triss", Range::MELEE));
        add(Card::the_hero(YENNEFER, 7, "Yennefer", Range::RANGED, Ability::Medic));
        add(Card::the_unit(COW, 0, "Cow", Range::RANGED, Ability::Summon(Box::new(bovine_defence))));
        add(Card::the_unit(DANDELION, 2, "Dandelion", Range::MELEE, Ability::CommandersHorn));
        add(Card::unit(EMIEL_REGIS, 5, "Emiel Regis", Range::MELEE));

        add(Card::the_unit(GAUNTER_ODIMM, 2, "Gaunter O'Dimm", Range::SIEGE, Ability::Muster(vec![GAUNTER_DARKNESS_1, GAUNTER_DARKNESS_2, GAUNTER_DARKNESS_3])));
        add(Card::the_unit(GAUNTER_DARKNESS_1, 4, "Gaunter O'Dimm Darkness", Range::RANGED, Ability::Muster(vec![GAUNTER_DARKNESS_2, GAUNTER_DARKNESS_3])));
        add(Card::the_unit(GAUNTER_DARKNESS_2, 4, "Gaunter O'Dimm Darkness", Range::RANGED, Ability::Muster(vec![GAUNTER_DARKNESS_1, GAUNTER_DARKNESS_3])));
        add(Card::the_unit(GAUNTER_DARKNESS_3, 4, "Gaunter O'Dimm Darkness", Range::RANGED, Ability::Muster(vec![GAUNTER_DARKNESS_1, GAUNTER_DARKNESS_2])));

        add(Card::the_unit(OLGIERD, 6, "Olgierd", Range::AGILE, Ability::MoraleBoost));
        add(Card::unit(VESEMIR, 6, "Vesemir", Range::MELEE));
        add(Card::the_unit(VILLENTRETENMERTH, 7, "Villentretenmerth", Range::MELEE, Ability::Scorch(Range::MELEE)));
        add(Card::unit(ZOLTAN, 5, "Zoltan", Range::MELEE));

        cards
    }

    fn monsters() -> HashMap<u16, Card> {
        let mut cards = HashMap::new();
        let mut add = |card: Card| { cards.insert(card.id(), card); };

        add(Card::hero(DRAUG, 10, "Draug", Range::MELEE));
        add(Card::hero(IMLERITH, 10, "Imlerith", Range::MELEE));
        add(Card::the_hero(KAYRAN, 8, "Kayran", Range::AGILE, Ability::MoraleBoost));
        add(Card::hero(LESHEN, 10, "Leshen", Range::RANGED));

        add(Card::the_unit(ARACHAS_1, 4, "Arachas", Range::MELEE, Ability::Muster(vec![ARACHAS_2, ARACHAS_3])));
        add(Card::the_unit(ARACHAS_2, 4, "Arachas", Range::MELEE, Ability::Muster(vec![ARACHAS_1, ARACHAS_3])));
        add(Card::the_unit(ARACHAS_3, 4, "Arachas", Range::MELEE, Ability::Muster(vec![ARACHAS_1, ARACHAS_2])));
        add(Card::the_unit(ARACHAS_BEHEMOTH, 6, "Arachas Behemoth", Range::SIEGE, Ability::Muster(vec![ARACHAS_1, ARACHAS_2, ARACHAS_3])));

        add(Card::unit(BOTCHLING, 4, "Botchling", Range::MELEE));
        add(Card::unit(CELAENO_HARPY, 2, "Celaeno Harpy", Range::AGILE));
        add(Card::unit(COCKATRICE, 2, "Cockatrice", Range::RANGED));

        add(Card::the_unit(CRONE_BREWESS, 6, "Crone: Brewess", Range::MELEE, Ability::Muster(vec![CRONE_WEAVESS, CRONE_WHISPESS])));
        add(Card::the_unit(CRONE_WEAVESS, 6, "Crone: Weavess", Range::MELEE, Ability::Muster(vec![CRONE_BREWESS, CRONE_WHISPESS])));
        add(Card::the_unit(CRONE_WHISPESS, 6, "Crone: Whispess", Range::MELEE, Ability::Muster(vec![CRONE_BREWESS, CRONE_WEAVESS])));

        add(Card::unit(EARTH_ELEMENTAL, 6, "Earth Elemental", Range::SIEGE));
        add(Card::unit(ENDREGA, 2, "Endrega", Range::SIEGE));
        add(Card::unit(FIEND, 6, "Fiend", Range::MELEE));
        add(Card::unit(FIRE_ELEMENTAL, 6, "Fire Elemental", Range::SIEGE));
        add(Card::unit(FOGLET, 2, "Foglet", Range::MELEE));
        add(Card::unit(FORKTAIL, 5, "Forktail", Range::MELEE));
        add(Card::unit(FRIGHTENER, 5, "Frightener", Range::MELEE));
        add(Card::unit(GARGOYLE, 2, "Gargoyle", Range::RANGED));

        add(Card::the_unit(GHOUL_1, 1, "Ghoul", Range::MELEE, Ability::Muster(vec![GHOUL_2, GHOUL_3])));
        add(Card::the_unit(GHOUL_2, 1, "Ghoul", Range::MELEE, Ability::Muster(vec![GHOUL_1, GHOUL_3])));
        add(Card::the_unit(GHOUL_3, 1, "Ghoul", Range::MELEE, Ability::Muster(vec![GHOUL_1, GHOUL_2])));

        add(Card::unit(GRAVE_HAG, 5, "Grave Hag", Range::RANGED));
        add(Card::unit(GRIFFIN, 5, "Griffin", Range::MELEE));
        add(Card::unit(HARPY, 2, "Harpy", Range::AGILE));
        add(Card::unit(ICE_GIANT, 5, "Ice Giant", Range::SIEGE));

        add(Card::the_unit(NEKKER_1, 2, "Nekker", Range::MELEE, Ability::Muster(vec![NEKKER_2, NEKKER_3])));
        add(Card::the_unit(NEKKER_2, 2, "Nekker", Range::MELEE, Ability::Muster(vec![NEKKER_1, NEKKER_3])));
        add(Card::the_unit(NEKKER_3, 2, "Nekker", Range::MELEE, Ability::Muster(vec![NEKKER_1, NEKKER_2])));

        add(Card::unit(PLAGUE_MAIDEN, 5, "Plague Maiden", Range::MELEE));
        add(Card::the_unit(TOAD, 7, "Toad", Range::RANGED, Ability::Scorch(Range::RANGED)));

        add(Card::the_unit(VAMP_BRUXA, 4, "Vampire: Bruxa", Range::MELEE, Ability::Muster(vec![VAMP_EKKIMARA, VAMP_FLEDER, VAMP_GARKAIN, VAMP_KATAKAN])));
        add(Card::the_unit(VAMP_EKKIMARA, 4, "Vampire: Ekkimara", Range::MELEE, Ability::Muster(vec![VAMP_BRUXA, VAMP_FLEDER, VAMP_GARKAIN, VAMP_KATAKAN])));
        add(Card::the_unit(VAMP_FLEDER, 4, "Vampire: Fleder", Range::MELEE, Ability::Muster(vec![VAMP_BRUXA, VAMP_EKKIMARA, VAMP_GARKAIN, VAMP_KATAKAN])));
        add(Card::the_unit(VAMP_GARKAIN, 4, "Vampire: Garkain", Range::MELEE, Ability::Muster(vec![VAMP_BRUXA, VAMP_EKKIMARA, VAMP_FLEDER, VAMP_KATAKAN])));
        add(Card::the_unit(VAMP_KATAKAN, 5, "Vampire: Katakan", Range::MELEE, Ability::Muster(vec![VAMP_BRUXA, VAMP_EKKIMARA, VAMP_FLEDER, VAMP_GARKAIN])));

        add(Card::unit(WEREWOLF, 5, "Werewolf", Range::MELEE));
        add(Card::unit(WYVERN, 2, "Wyvern", Range::RANGED));

        cards
    }

    fn nilfgaard() -> HashMap<u16, Card> {
        let mut cards = HashMap::new();
        let mut add = |card: Card| { cards.insert(card.id(), card); };

        add(Card::hero(LETHO, 10, "Letto of Gulet", Range::MELEE));
        add(Card::the_hero(MENNO_COEHOORN, 10, "Menno Coehoorn", Range::MELEE, Ability::Medic));
        add(Card::hero(MORVRAN_VOORHIS, 10, "Morvran Voorhis", Range::SIEGE));
        add(Card::hero(TIBOR_EGGEBRACHT, 10, "Tibor Eggebracht", Range::RANGED));
        add(Card::unit(ALBRICH, 2, "Albrich", Range::RANGED));
        add(Card::unit(ASSIRE_VAR_ANAHID, 6, "Assire var Anahid", Range::RANGED));
        add(Card::unit(BLACK_INFANTRY_ARCHER_1, 10, "Black Infrantry Archer", Range::RANGED));
        add(Card::unit(BLACK_INFANTRY_ARCHER_2, 10, "Black Infrantry Archer", Range::RANGED));
        add(Card::unit(CAHIR, 6, "Cahir Mawr Dyffryn", Range::MELEE));
        add(Card::unit(CYNTHIA, 4, "Cynthia", Range::RANGED));
        add(Card::the_unit(ETOLIAN_ARCHERS_1, 1, "Etolian Auxiliary Archers", Range::RANGED, Ability::Medic));
        add(Card::the_unit(ETOLIAN_ARCHERS_2, 1, "Etolian Auxiliary Archers", Range::RANGED, Ability::Medic));
        add(Card::unit(FRINGILLA_VIGO, 6, "Fringilla Vigo", Range::RANGED));
        add(Card::unit(HEAVY_FIRE_SCORPION, 10, "Heavy Fire Scorpion", Range::SIEGE));

        add(Card::the_unit(IMPERA_BRIGADE_1, 3, "Impera Brigade Guard", Range::MELEE, Ability::TightBond(1)));
        add(Card::the_unit(IMPERA_BRIGADE_2, 3, "Impera Brigade Guard", Range::MELEE, Ability::TightBond(1)));
        add(Card::the_unit(IMPERA_BRIGADE_3, 3, "Impera Brigade Guard", Range::MELEE, Ability::TightBond(1)));
        add(Card::the_unit(IMPERA_BRIGADE_4, 3, "Impera Brigade Guard", Range::MELEE, Ability::TightBond(1)));

        add(Card::unit(MORTEISEN, 3, "Morteisen", Range::MELEE));

        add(Card::the_unit(NAUSICAA_RIDER_1, 2, "Nausicaa Cavalry Rider", Range::MELEE, Ability::TightBond(2)));
        add(Card::the_unit(NAUSICAA_RIDER_2, 2, "Nausicaa Cavalry Rider", Range::MELEE, Ability::TightBond(2)));
        add(Card::the_unit(NAUSICAA_RIDER_3, 2, "Nausicaa Cavalry Rider", Range::MELEE, Ability::TightBond(2)));

        add(Card::unit(PUTTKAMMER, 3, "Puttkammer", Range::RANGED));
        add(Card::unit(RAINFARN, 4, "Rainfarn", Range::MELEE));
        add(Card::unit(RENUALD_AEP_MATSEN, 5, "Renuald Aep Matsen", Range::RANGED));
        add(Card::unit(ROTTEN_MANGONEL, 3, "Rotten Mangonel", Range::SIEGE));
        add(Card::the_unit(SHILARD, 7, "Shilard Fitz-Oesterlen", Range::MELEE, Ability::Spy));
        add(Card::unit(SIEGE_ENGINEER, 6, "Siege Engineer", Range::SIEGE));
        add(Card::the_unit(SIEGE_TECHNICIAN, 0, "Siege Technician", Range::SIEGE, Ability::Medic));
        add(Card::the_unit(STEFAN_SKELLEN, 9, "Stefan Skellen", Range::MELEE, Ability::Spy));
        add(Card::unit(SWEERS, 2, "Sweers", Range::RANGED));
        add(Card::unit(VANHEMAR, 4, "Vanhemar", Range::RANGED));
        add(Card::the_unit(VATTIER_DE_RIDEAUX, 4, "Vattier de Rideaux", Range::MELEE, Ability::Spy));
        add(Card::unit(VREEMDE, 2, "Vreemde", Range::MELEE));

        add(Card::the_unit(YOUNG_EMISSARY_1, 5, "Young Emissary", Range::MELEE, Ability::TightBond(3)));
        add(Card::the_unit(YOUNG_EMISSARY_2, 5, "Young Emissary", Range::MELEE, Ability::TightBond(3)));

        add(Card::unit(FIRE_SCORPION, 5, "Fire Scorpion", Range::SIEGE));

        cards
    }

    fn northern_realms() -> HashMap<u16, Card> {
        let mut cards = HashMap::new();
        let mut add = |card: Card| { cards.insert(card.id(), card); };

        add(Card::hero(ESTERAD_THYSSEN, 10, "Esterad Thyssen", Range::MELEE));
        add(Card::hero(JOHN_NATALIS, 10, "John Natalis", Range::MELEE));
        add(Card::hero(PHILIPPA_EILHART, 10, "Philippa Eilhart", Range::RANGED));
        add(Card::hero(VERNON_ROCHE, 10, "Vernon Roche", Range::MELEE));
        add(Card::unit(BALLISTA_1, 6, "Ballista", Range::SIEGE));
        add(Card::unit(BALLISTA_2, 6, "Ballista", Range::SIEGE));

        add(Card::the_unit(BLUE_STRIPES_1, 4, "Blue Stripes Commando", Range::MELEE, Ability::TightBond(4)));
        add(Card::the_unit(BLUE_STRIPES_2, 4, "Blue Stripes Commando", Range::MELEE, Ability::TightBond(4)));
        add(Card::the_unit(BLUE_STRIPES_3, 4, "Blue Stripes Commando", Range::MELEE, Ability::TightBond(4)));

        add(Card::the_unit(CATAPULT_1, 8, "Catapult", Range::SIEGE, Ability::TightBond(5)));
        add(Card::the_unit(CATAPULT_2, 8, "Catapult", Range::SIEGE, Ability::TightBond(5)));

        add(Card::the_unit(DRAGON_HUNTER_1, 5, "Dragon Hunter", Range::RANGED, Ability::TightBond(6)));
        add(Card::the_unit(DRAGON_HUNTER_2, 5, "Dragon Hunter", Range::RANGED, Ability::TightBond(6)));
        add(Card::the_unit(DRAGON_HUNTER_3, 5, "Dragon Hunter", Range::RANGED, Ability::TightBond(6)));

        add(Card::unit(DETHMOLD, 6, "Dethmold", Range::RANGED));
        add(Card::the_unit(DUN_BANNER_MEDIC, 5, "Dun Banner Medic", Range::SIEGE, Ability::Medic));
        add(Card::the_unit(SIEGE_EXPERT_1, 1, "Siege Expert", Range::SIEGE, Ability::MoraleBoost));
        add(Card::the_unit(SIEGE_EXPERT_2, 1, "Siege Expert", Range::SIEGE, Ability::MoraleBoost));
        add(Card::the_unit(SIEGE_EXPERT_3, 1, "Siege Expert", Range::SIEGE, Ability::MoraleBoost));
        add(Card::unit(KEIRA_METZ, 5, "Keira Metz", Range::RANGED));

        add(Card::the_unit(POOR_INFANTRY_1, 1, "Poor Fucking Infrantry", Range::MELEE, Ability::TightBond(7)));
        add(Card::the_unit(POOR_INFANTRY_2, 1, "Poor Fucking Infrantry", Range::MELEE, Ability::TightBond(7)));
        add(Card::the_unit(POOR_INFANTRY_3, 1, "Poor Fucking Infrantry", Range::MELEE, Ability::TightBond(7)));

        add(Card::the_unit(PRINCE_STENNIS, 5, "Prince Stennis", Range::MELEE, Ability::Spy));
        add(Card::unit(REDANIAN_SOLDIER_1, 1, "Redanian Foot Soldier", Range::MELEE));
        add(Card::unit(REDANIAN_SOLDIER_2, 1, "Redanian Foot Soldier", Range::MELEE));
        add(Card::unit(SABRINA_GLEVISSIG, 4, "Sabrina Glevissig", Range::RANGED));
        add(Card::unit(SHELDON_SKAGGS, 4, "Sheldon Skaggs", Range::RANGED));
        add(Card::unit(SIEGE_TOWER, 6, "Siege Tower", Range::SIEGE));
        add(Card::unit(SIEGFRIED_OF_DENESLE, 5, "Siegfried of Denesle", Range::MELEE));
        add(Card::the_unit(DIJKSTRA, 4, "Dijkstra", Range::MELEE, Ability::Spy));
        add(Card::unit(SILE_DE_TANSARVILLE, 5, "Sile de Tansarville", Range::RANGED));
        add(Card::the_unit(THALER, 1, "Thaler", Range::SIEGE, Ability::Spy));
        add(Card::unit(TREBUCHET_1, 6, "Trebuchet", Range::SIEGE));
        add(Card::unit(TREBUCHET_2, 6, "Trebuchet", Range::SIEGE));
        add(Card::unit(VES, 5, "Ves", Range::MELEE));
        add(Card::unit(YARPEN_ZIGRIN, 2, "Yarpen Zigrin", Range::MELEE));

        cards
    }

    fn skoiatael() -> HashMap<u16, Card> {
        let mut cards = HashMap::new();
        let mut add = |card: Card| { cards.insert(card.id(), card); };

        add(Card::hero(EITHNE, 10, "Eithne", Range::RANGED));
        add(Card::hero(IORVETH, 10, "Iorveth", Range::RANGED));
        add(Card::the_hero(ISENGRIM, 10, "Isengrim", Range::MELEE, Ability::MoraleBoost));
        add(Card::hero(SAESENTHESSIS, 10, "Saesenthessis", Range::RANGED));
        add(Card::unit(BARCLAY_ELS, 6, "Barclay Els", Range::AGILE));
        add(Card::unit(CIARAN, 3, "Ciaran Aep Easnillien", Range::AGILE));
        add(Card::unit(DENNIS_CRANMER, 6, "Dennis Cranmer", Range::MELEE));
        add(Card::unit(DOL_BLATHANNA_ARCHER, 4, "Dol Blathanna Archer", Range::RANGED));
        add(Card::unit(DOL_BLATHANNA_SCOUT_1, 6, "Dol Blathanna Scout", Range::AGILE));
        add(Card::unit(DOL_BLATHANNA_SCOUT_2, 6, "Dol Blathanna Scout", Range::AGILE));
        add(Card::unit(DOL_BLATHANNA_SCOUT_3, 6, "Dol Blathanna Scout", Range::AGILE));

        add(Card::the_unit(DWARVEN_SKIRMISHER_1, 3, "Dwarven Skirmisher", Range::MELEE, Ability::Muster(vec![DWARVEN_SKIRMISHER_2, DWARVEN_SKIRMISHER_3])));
        add(Card::the_unit(DWARVEN_SKIRMISHER_2, 3, "Dwarven Skirmisher", Range::MELEE, Ability::Muster(vec![DWARVEN_SKIRMISHER_1, DWARVEN_SKIRMISHER_3])));
        add(Card::the_unit(DWARVEN_SKIRMISHER_3, 3, "Dwarven Skirmisher", Range::MELEE, Ability::Muster(vec![DWARVEN_SKIRMISHER_1, DWARVEN_SKIRMISHER_2])));

        add(Card::the_unit(ELVEN_SKIRMISHER_1, 2, "Elven Skirmisher", Range::RANGED, Ability::Muster(vec![ELVEN_SKIRMISHER_2, ELVEN_SKIRMISHER_3])));
        add(Card::the_unit(ELVEN_SKIRMISHER_2, 2, "Elven Skirmisher", Range::RANGED, Ability::Muster(vec![ELVEN_SKIRMISHER_1, ELVEN_SKIRMISHER_3])));
        add(Card::the_unit(ELVEN_SKIRMISHER_3, 2, "Elven Skirmisher", Range::RANGED, Ability::Muster(vec![ELVEN_SKIRMISHER_1, ELVEN_SKIRMISHER_2])));

        add(Card::unit(FILAVANDREL, 6, "Filavandrel Aen Fidhail", Range::AGILE));
        add(Card::the_unit(HAVEKAR_HEALER_1, 0, "Havekar Healer", Range::RANGED, Ability::Medic));
        add(Card::the_unit(HAVEKAR_HEALER_2, 0, "Havekar Healer", Range::RANGED, Ability::Medic));
        add(Card::the_unit(HAVEKAR_HEALER_3, 0, "Havekar Healer", Range::RANGED, Ability::Medic));

        add(Card::the_unit(HAVEKAR_SMUGGLER_1, 5, "Havekar Smuggler", Range::MELEE, Ability::Muster(vec![HAVEKAR_SMUGGLER_2, HAVEKAR_SMUGGLER_3])));
        add(Card::the_unit(HAVEKAR_SMUGGLER_2, 5, "Havekar Smuggler", Range::MELEE, Ability::Muster(vec![HAVEKAR_SMUGGLER_1, HAVEKAR_SMUGGLER_3])));
        add(Card::the_unit(HAVEKAR_SMUGGLER_3, 5, "Havekar Smuggler", Range::MELEE, Ability::Muster(vec![HAVEKAR_SMUGGLER_1, HAVEKAR_SMUGGLER_2])));

        add(Card::unit(IDA_EMEAN, 6, "Ida Emean Aep Sivney", Range::RANGED));
        add(Card::unit(MAHAKAMAN_DEFENDER_1, 5, "Mahakaman Defender", Range::MELEE));
        add(Card::unit(MAHAKAMAN_DEFENDER_2, 5, "Mahakaman Defender", Range::MELEE));
        add(Card::unit(MAHAKAMAN_DEFENDER_3, 5, "Mahakaman Defender", Range::MELEE));
        add(Card::unit(MAHAKAMAN_DEFENDER_4, 5, "Mahakaman Defender", Range::MELEE));
        add(Card::unit(MAHAKAMAN_DEFENDER_5, 5, "Mahakaman Defender", Range::MELEE));
        add(Card::the_unit(MILVA, 10, "Milva", Range::RANGED, Ability::MoraleBoost));
        add(Card::unit(RIORDAIN, 1, "Riordain", Range::RANGED));
        add(Card::the_unit(SCHIRRU, 8, "Schirru", Range::SIEGE, Ability::Scorch(Range::SIEGE)));
        add(Card::unit(TORUVIEL, 2, "Toruviel", Range::RANGED));
        add(Card::unit(VRIHEDD_RECRUIT, 4, "Vrihedd Brigade Recruit", Range::RANGED));
        add(Card::unit(VRIHEDD_VETERAN_1, 5, "Vrihedd Brigade Veteran", Range::AGILE));
        add(Card::unit(VRIHEDD_VETERAN_2, 5, "Vrihedd Brigade Veteran", Range::AGILE));
        add(Card::unit(YAEVINN, 6, "Yaevinn", Range::AGILE));

        cards
    }

    fn skellige() -> HashMap<u16, Card> {
        let mut cards = HashMap::new();
        let mut add = |card: Card| { cards.insert(card.id(), card); };

        let vildkaarl = Unit {
            id: VILDKAARL,
            strength: Strength::Regular(14),
            name: "Vildkaarl".to_string(),
            ability: Ability::MoraleBoost,
            range: Range::MELEE,
        };

        let hemdall = Unit {
            id: HEMDALL,
            strength: Strength::Hero(11),
            name: "Hemdall".to_string(),
            ability: Ability::None,
            range: Range::MELEE,
        };

        let young_vildkaarl = Unit {
            id: YOUNG_VILDKAARL,
            strength: Strength::Regular(8),
            name: "Young Vildkaarl".to_string(),
            ability: Ability::TightBond(11),
            range: Range::RANGED,
        };

        add(Card::the_hero(CERYS, 10, "Cerys", Range::MELEE, Ability::Muster(vec![DRUMMOND_SHIELDMAIDEN_1, DRUMMOND_SHIELDMAIDEN_2, DRUMMOND_SHIELDMAIDEN_3])));
        add(Card::the_hero(ERMION, 8, "Ermion", Range::RANGED, Ability::Mardrome));
        add(Card::hero(HJALMAR, 10, "Hjalmar", Range::RANGED));
        add(Card::the_unit(BERSERKER, 4, "Berserker", Range::MELEE, Ability::Berserker(Box::new(vildkaarl))));
        add(Card::the_unit(BIRNA_BRAN, 2, "Birna Bran", Range::MELEE, Ability::Medic));
        add(Card::unit(BLUEBOY_LUGOS, 6, "Blueboy Lugos", Range::MELEE));

        add(Card::the_unit(CLAN_AN_CRAITE_1, 6, "Clan an Craite Warrior", Range::MELEE, Ability::TightBond(8)));
        add(Card::the_unit(CLAN_AN_CRAITE_2, 6, "Clan an Craite Warrior", Range::MELEE, Ability::TightBond(8)));
        add(Card::the_unit(CLAN_AN_CRAITE_3, 6, "Clan an Craite Warrior", Range::MELEE, Ability::TightBond(8)));

        add(Card::unit(CLAN_BROKVAR_1, 6, "Clan Brokvar Archer", Range::RANGED));
        add(Card::unit(CLAN_BROKVAR_2, 6, "Clan Brokvar Archer", Range::RANGED));
        add(Card::unit(CLAN_BROKVAR_3, 6, "Clan Brokvar Archer", Range::RANGED));
        add(Card::the_unit(CLAN_DIMUN_PIRATE, 6, "Clan Dimun Pirate", Range::RANGED, Ability::Scorch(Range::ALL)));

        add(Card::the_unit(DRUMMOND_SHIELDMAIDEN_1, 4, "Clan Drummond Shieldmaiden", Range::MELEE, Ability::TightBond(9)));
        add(Card::the_unit(DRUMMOND_SHIELDMAIDEN_2, 4, "Clan Drummond Shieldmaiden", Range::MELEE, Ability::TightBond(9)));
        add(Card::the_unit(DRUMMOND_SHIELDMAIDEN_3, 4, "Clan Drummond Shieldmaiden", Range::MELEE, Ability::TightBond(9)));

        add(Card::unit(CLAN_HEYMAEY_SKALD, 4, "Clan Heymaey Skald", Range::MELEE));
        add(Card::unit(CLAN_TORDARROCH_ARMORSMITH, 4, "Clan Tordarroch Armorsmith", Range::MELEE));
        add(Card::unit(DONAR_AN_HINDAR, 4, "Donar an Hindar", Range::MELEE));
        add(Card::the_unit(DRAIG_BON_DHU, 2, "Draig Bon-Dhu", Range::SIEGE, Ability::CommandersHorn));
        add(Card::unit(MADMAN_LUGOS, 6, "Madman Lugos", Range::MELEE));
        add(Card::unit(HOLGER_BLACKHAND, 4, "Holger Blackhand", Range::SIEGE));
        add(Card::the_unit(KAMBI, 0, "Kambi", Range::MELEE, Ability::Summon(Box::new(hemdall))));

        add(Card::the_unit(LIGHT_LONGSHIP_1, 4, "Light Longship", Range::RANGED, Ability::Muster(vec![LIGHT_LONGSHIP_2, LIGHT_LONGSHIP_3])));
        add(Card::the_unit(LIGHT_LONGSHIP_2, 4, "Light Longship", Range::RANGED, Ability::Muster(vec![LIGHT_LONGSHIP_1, LIGHT_LONGSHIP_3])));
        add(Card::the_unit(LIGHT_LONGSHIP_3, 4, "Light Longship", Range::RANGED, Ability::Muster(vec![LIGHT_LONGSHIP_1, LIGHT_LONGSHIP_2])));

        add(Card::the_unit(OLAF, 12, "Olaf", Range::AGILE, Ability::MoraleBoost));
        add(Card::unit(SVANRIGE, 4, "Svanrige", Range::MELEE));
        add(Card::unit(UDALRYK, 4, "Udalryk", Range::MELEE));

        add(Card::the_unit(WAR_LONGSHIP_1, 6, "War Longship", Range::SIEGE, Ability::TightBond(10)));
        add(Card::the_unit(WAR_LONGSHIP_2, 6, "War Longship", Range::SIEGE, Ability::TightBond(10)));
        add(Card::the_unit(WAR_LONGSHIP_3, 6, "War Longship", Range::SIEGE, Ability::TightBond(10)));

        add(Card::the_unit(YOUNG_BERSERKER_1, 2, "Young Berserker", Range::RANGED, Ability::Berserker(Box::new(young_vildkaarl.clone()))));
        add(Card::the_unit(YOUNG_BERSERKER_2, 2, "Young Berserker", Range::RANGED, Ability::Berserker(Box::new(young_vildkaarl.clone()))));
        add(Card::the_unit(YOUNG_BERSERKER_3, 2, "Young Berserker", Range::RANGED, Ability::Berserker(Box::new(young_vildkaarl))));

        cards
    }

    fn special() -> HashMap<u16, Vec<Card>> {
        let mut cards = HashMap::new();

        let mut add_specials = |id: u16, special: Special, count: usize| {
            let copies = (0..count)
                .map(|_| Card::Special(id, special))
                .collect::<Vec<_>>();
            cards.insert(id, copies);
        };

        add_specials(BITING_FROST, Special::Weather(Weather::BitingFrost), 3);
        add_specials(CLEAR_WEATHER, Special::Weather(Weather::ClearWeather), 3);
        add_specials(IMPENETRABLE_FOG, Special::Weather(Weather::ImpenetrableFog), 3);
        add_specials(SKELLIGE_STORM, Special::Weather(Weather::SkelligeStorm), 3);
        add_specials(TORRENTIAL_RAIN, Special::Weather(Weather::TorrentialRain), 3);

        add_specials(COMMANDERS_HORN, Special::CommandersHorn, 3);
        add_specials(DECOY, Special::Decoy, 3);
        add_specials(SCORCH, Special::Scorch, 3);
        add_specials(MARDROME, Special::Mardrome, 3);

        cards
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

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use crate::{
        card::Card,
        deck::{Cards, Faction, Library},
    };

    impl Cards {
        pub fn hand_ids(&self) -> Vec<u16> {
            self.hand.iter().map(Card::id).collect()
        }

        pub fn pile_ids(&self) -> Vec<u16> {
            self.pile.iter().map(Card::id).collect()
        }

        pub fn monsters(hand: &[u16], deck: &[u16]) -> Self {
            let mut lib = Library::default();

            let hand = get_from_lib(hand, &mut lib, Faction::Monsters);
            let deck = get_from_lib(deck, &mut lib, Faction::Monsters);

            Self {
                hand,
                deck,
                pile: Vec::default(),
            }
        }

        pub fn northern_realms(hand: &[u16], deck: &[u16]) -> Self {
            let mut lib = Library::default();

            let hand = get_from_lib(hand, &mut lib, Faction::NorthernRealms);
            let deck = get_from_lib(deck, &mut lib, Faction::NorthernRealms);

            Self {
                hand,
                deck,
                pile: Vec::default(),
            }
        }

        pub fn skellige(hand: &[u16], deck: &[u16]) -> Self {
            let mut lib = Library::default();

            let hand = get_from_lib(hand, &mut lib, Faction::Skellige);
            let deck = get_from_lib(deck, &mut lib, Faction::Skellige);

            Self {
                hand,
                deck,
                pile: Vec::default(),
            }
        }

        /// Builds a hand/deck from card ids regardless of faction, so a single
        /// row can mix neutral, faction and special cards freely.
        pub fn mixed(hand: &[u16], deck: &[u16]) -> Self {
            let mut lib = Library::default();

            Self {
                hand: take_any(hand, &mut lib),
                deck: take_any(deck, &mut lib),
                pile: Vec::default(),
            }
        }
    }

    fn take_any(ids: &[u16], lib: &mut Library) -> Vec<Card> {
        ids.iter()
            .filter_map(|id| {
                lib.neutral
                    .remove(id)
                    .or_else(|| lib.monsters.remove(id))
                    .or_else(|| lib.nilfgaard.remove(id))
                    .or_else(|| lib.northern_realms.remove(id))
                    .or_else(|| lib.skoiatael.remove(id))
                    .or_else(|| lib.skellige.remove(id))
                    .or_else(|| lib.special.get_mut(id).and_then(Vec::pop))
            })
            .collect()
    }

    fn get_from_lib(ids: &[u16], lib: &mut Library, f: Faction) -> Vec<Card> {
        let mut collected: Vec<Card> = ids.iter().filter_map(|id| lib.neutral.remove(id)).collect();

        let mut faction_cards: Vec<Card> = ids
            .iter()
            .filter_map(|id| lib.get_mut(&f).remove(id))
            .collect();

        collected.append(&mut faction_cards);

        let mut special_cards: Vec<Card> = ids
            .iter()
            .filter_map(|id| lib.special.get_mut(id).and_then(Vec::pop))
            .collect();

        collected.append(&mut special_cards);

        collected
    }

    impl Library {
        fn get_mut(&mut self, f: &Faction) -> &mut HashMap<u16, Card> {
            match f {
                Faction::Monsters => &mut self.monsters,
                Faction::Nilfgaard => &mut self.nilfgaard,
                Faction::NorthernRealms => &mut self.northern_realms,
                Faction::Skoiatael => &mut self.skoiatael,
                Faction::Skellige => &mut self.skellige,
            }
        }
    }
}
