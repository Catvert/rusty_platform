use ggez::Context;
use ggez::error::GameError;
use ggez::graphics::Color;

use nuklear::Context as NkCtx;

use na::Vector2;
use wrapper::nuklear_wrapper::NkFontsHolder;

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
    fn draw_ui(&mut self, window_size: Vector2<u32>, nk_ctx: &mut NkCtx, nk_fonts: &NkFontsHolder) -> SceneState;
    fn background_color(&self) -> Color;
    fn resize_event(&mut self, _ctx: &mut Context, screen_size: Vector2<u32>) {}
}
