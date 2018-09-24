//! The simplest possible example that does something.

extern crate ggez;
extern crate gfx;
extern crate gfx_core;
extern crate gfx_device_gl;
extern crate sdl2;
extern crate specs;
#[macro_use]
extern crate specs_derive;
extern crate ron;
extern crate serde;
#[macro_use]
extern crate serde_derive;

extern crate shrev;

extern crate num;
extern crate nalgebra as na;
extern crate ndarray;

extern crate shred;

#[macro_use]
extern crate lazy_static;

extern crate core;

#[macro_use]
extern crate imgui;
extern crate imgui_sys;
extern crate imgui_gfx_renderer;

extern crate nfd;

extern crate walkdir;

mod wrapper;
mod ecs;
mod scenes;
mod utils;
mod game;

use game::Game;

pub fn main() {
    Game::new().run();
}