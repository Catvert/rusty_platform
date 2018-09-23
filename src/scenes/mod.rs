use ggez::Context;
use ggez::error::GameError;
use ggez::graphics::Color;

use na::Vector2;

use imgui::Ui;

pub mod main_scene;
pub mod game_scene;
pub mod editor_scene;

pub enum NextState {
    Continue,
    Push(Box<dyn Scene>),
    Pop,
}

pub type SceneState = Result<NextState, GameError>;

pub trait Scene {
    fn update(&mut self, ctx: &mut Context, dt: f32) -> SceneState;
    fn draw(&mut self, ctx: &mut Context) -> SceneState;
    fn draw_ui(&mut self, window_size: Vector2<u32>, ui: &Ui) -> SceneState;
    fn background_color(&self) -> Color;
    fn resize_event(&mut self, _ctx: &mut Context, screen_size: Vector2<u32>) {}
}
