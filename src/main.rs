#![allow(unused)]
use std::io::{self};

use crate::{
    card::Range,
    deck::Library,
    game::{Controller, Game},
};

mod board;
mod card;
mod constants;
mod deck;
mod game;
mod row;
mod side;

fn main() {
    let _ = Library::default();
    let p1 = todo!();
    let p2 = todo!();
    let mut game = Game::new(ConsoleController {}, p1, p2);

    game.start();
}

struct ConsoleController;

impl Controller for ConsoleController {
    fn select_from_hand(&self) -> usize {
        println!("Select card to play: ");

        let mut input = String::new();

        io::stdin()
            .read_line(&mut input)
            .expect("could not read console input");

        input
            .trim_end()
            .parse::<usize>()
            .expect("could not parse card index")
    }

    fn select_range(&self) -> card::Range {
        println!("Select range (1 - MELEE, 2 - AGILE): ");

        let mut input = String::new();

        io::stdin()
            .read_line(&mut input)
            .expect("could not read console input");

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

        io::stdin()
            .read_line(&mut input)
            .expect("could not read console input");

        input
            .trim_end()
            .parse::<usize>()
            .expect("could not parse card index")
    }

    fn select_from_board(&self) -> Option<(Range, usize)> {
        let range = self.select_range();

        println!("Select card from row: ");

        let mut input = String::new();

        io::stdin()
            .read_line(&mut input)
            .expect("could not read console input");

        let i = input
            .trim_end()
            .parse::<usize>()
            .expect("could not parse card index");

        Some((range, i))
    }
}
