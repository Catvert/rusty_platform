use ggez::Context;
use ggez::graphics::Color;

use na::Vector2;

use scenes::{Scene, SceneState, NextState};

use nuklear::Context as NkCtx;

use ecs::level::Level;
use ecs::inputs::InputSystem;
use ecs::actions::ActionSystem;

use utils::camera::Camera;
use utils::input_manager::RefInputManager;
use utils::resources_manager::RefRM;
use ecs::physics::PhysicsSystem;
use wrapper::nuklear_wrapper::NkFontsHolder;

pub struct GameScene<'a, 'b> {
    level: Level<'a, 'b>,
    camera: Camera
}

impl<'a, 'b> GameScene<'a, 'b> {
    pub fn new(screen_size: Vector2<u32>, resources_manager: RefRM, input_manager: RefInputManager, level_path: String) -> Self {
        let level = Level::load(level_path, resources_manager, |builder| {
            builder.with(InputSystem { input_manager: input_manager.clone() }, "input_manager", &[])
                .with(ActionSystem, "action_system", &["input_manager"])
                .with(PhysicsSystem { gravity: Vector2::new(0., -10.) }, "phys_sys", &["action_system"])
        });

        let camera = Camera::new(screen_size,  1.);

        GameScene { level, camera }
    }
}

impl<'a, 'b> Scene for GameScene<'a, 'b> {
    fn update(&mut self, ctx: &mut Context, dt: f32) -> SceneState {
        self.level.update(ctx, &self.camera, dt);
        Ok(NextState::Continue)
    }

    fn draw(&mut self, ctx: &mut Context) -> SceneState {
        self.level.draw( ctx, &self.camera);
        Ok(NextState::Continue)
    }

    fn draw_ui(&mut self, window_size: Vector2<u32>, nk_ctx: &mut NkCtx, nk_fonts: &NkFontsHolder) -> SceneState {
        Ok(NextState::Continue)
    }

    fn background_color(&self) -> Color { self.level.background_color() }
}