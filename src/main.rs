#![allow(unused)]
use std::io::{self};

use crate::{
    card::{Card, Range, Strength, Unit},
    deck::{Cards, Deck},
    game::{Controller, Game},
};

mod board;
mod card;
mod deck;
mod game;
mod row;
mod side;

fn main() {
    let p1 = test_cards();
    let p2 = test_cards();
    let mut game = Game::new(ConsoleController {}, p1, p2);

    game.start();
}

struct ConsoleController;

impl Controller for ConsoleController {
    fn select_from_hand(&self) -> usize {
        println!("Select card to play: ");

        let mut input = String::new();

        io::stdin().read_line(&mut input).unwrap();

        input.trim_end().parse::<usize>().unwrap()
    }

    fn select_range(&self) -> card::Range {
        println!("Select range (1 - MELEE, 2 - AGILE): ");

        let mut input = String::new();

        io::stdin().read_line(&mut input).unwrap();

        match input.trim_end() {
            "1" => Range::MELEE,
            "2" => Range::RANGED,
            "3" => Range::SIEGE,
            _ => panic!("Invalid range"),
        }
    }

    fn select_from_pile(&self) -> usize {
        println!("Select card to restore: ");

        let mut input = String::new();

        io::stdin().read_line(&mut input).unwrap();

        input.trim_end().parse::<usize>().unwrap()
    }
}

fn test_cards() -> Cards {
    Cards::new(Deck::new(vec![
        Card::Unit(Unit {
            strength: Strength::Regular(5),
            ability: card::Ability::None,
            range: Range::AGILE,
        }),
        Card::Unit(Unit {
            strength: Strength::Regular(5),
            ability: card::Ability::None,
            range: Range::AGILE,
        }),
        Card::Unit(Unit {
            strength: Strength::Regular(5),
            ability: card::Ability::None,
            range: Range::AGILE,
        }),
        Card::Unit(Unit {
            strength: Strength::Regular(5),
            ability: card::Ability::None,
            range: Range::AGILE,
        }),
        Card::Unit(Unit {
            strength: Strength::Regular(5),
            ability: card::Ability::None,
            range: Range::AGILE,
        }),
        Card::Unit(Unit {
            strength: Strength::Regular(5),
            ability: card::Ability::None,
            range: Range::AGILE,
        }),
        Card::Unit(Unit {
            strength: Strength::Regular(5),
            ability: card::Ability::None,
            range: Range::AGILE,
        }),
        Card::Unit(Unit {
            strength: Strength::Regular(5),
            ability: card::Ability::None,
            range: Range::AGILE,
        }),
        Card::Unit(Unit {
            strength: Strength::Regular(5),
            ability: card::Ability::None,
            range: Range::AGILE,
        }),
        Card::Unit(Unit {
            strength: Strength::Regular(5),
            ability: card::Ability::None,
            range: Range::AGILE,
        }),
        Card::Unit(Unit {
            strength: Strength::Regular(5),
            ability: card::Ability::None,
            range: Range::AGILE,
        }),
        Card::Unit(Unit {
            strength: Strength::Regular(5),
            ability: card::Ability::None,
            range: Range::AGILE,
        }),
        Card::Unit(Unit {
            strength: Strength::Regular(5),
            ability: card::Ability::None,
            range: Range::AGILE,
        }),
        Card::Unit(Unit {
            strength: Strength::Regular(5),
            ability: card::Ability::None,
            range: Range::AGILE,
        }),
        Card::Unit(Unit {
            strength: Strength::Regular(5),
            ability: card::Ability::None,
            range: Range::AGILE,
        }),
        Card::Unit(Unit {
            strength: Strength::Regular(5),
            ability: card::Ability::None,
            range: Range::AGILE,
        }),
        Card::Unit(Unit {
            strength: Strength::Regular(5),
            ability: card::Ability::None,
            range: Range::AGILE,
        }),
        Card::Unit(Unit {
            strength: Strength::Regular(5),
            ability: card::Ability::None,
            range: Range::AGILE,
        }),
        Card::Unit(Unit {
            strength: Strength::Regular(5),
            ability: card::Ability::None,
            range: Range::AGILE,
        }),
        Card::Unit(Unit {
            strength: Strength::Regular(5),
            ability: card::Ability::None,
            range: Range::AGILE,
        }),
        Card::Unit(Unit {
            strength: Strength::Regular(5),
            ability: card::Ability::None,
            range: Range::AGILE,
        }),
        Card::Unit(Unit {
            strength: Strength::Regular(5),
            ability: card::Ability::None,
            range: Range::AGILE,
        }),
    ]))
}
