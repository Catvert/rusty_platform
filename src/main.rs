use crate::game::Game;

mod wrapper;
mod ecs;
mod scenes;
mod utils;
mod game;

pub fn main() {
    Game::new().run();
}