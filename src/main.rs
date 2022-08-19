mod player;
mod direction;
mod game;
mod point;
mod command;

use crate::game::Game;
use std::io::stdout;

fn main() {
    Game::new(stdout(), 50, 20, 1, 3).run();
    //            width, height, move, gen
}