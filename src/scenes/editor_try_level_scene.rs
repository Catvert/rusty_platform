use crate::{
    ecs::{
        actions::ActionSystem,
        inputs::InputSystem,
        level::Level,
        physics::PhysicsSystem,
    },
    scenes::{
        NextState,
        Scene,
        SceneState,
    },
    utils::{
        camera::Camera,
        constants,
        input_manager::RefInputManager,
    },
};
use ggez::{
    Context,
    event::Keycode,
    graphics::Color,
};
use imgui::Ui;
use nalgebra::Vector2;

pub struct EditorTryLevelScene<'a, 'b> {
    level: Level<'a, 'b>,
    input_manager: RefInputManager,
    camera: Camera,
}

impl<'a, 'b> EditorTryLevelScene<'a, 'b> {
    pub fn new(screen_size: Vector2<u32>, input_manager: RefInputManager, editor_level: &Level) -> Self {
        let level = crate::ecs::level::clone(editor_level, |builder| {
            builder.with(InputSystem { input_manager: input_manager.clone() }, "input_manager", &[])
                .with(ActionSystem, "action_system", &["input_manager"])
                .with(PhysicsSystem { gravity: Vector2::new(0., 9.81) }, "phys_sys", &["action_system"])
        });

        EditorTryLevelScene {
            level,
            input_manager,
            camera: Camera::new(screen_size, Vector2::new(constants::CAMERA_VIEW_SIZE.0, constants::CAMERA_VIEW_SIZE.1), 1.),
        }
    }
}

impl<'a, 'b> Scene for EditorTryLevelScene<'a, 'b> {
    fn update(&mut self, ctx: &mut Context, dt: f32) -> SceneState {
        let mut next_state = NextState::Continue;

        self.level.update(ctx, &self.camera, dt);

        if let Some(true) = self.input_manager.lock().unwrap().is_key_pressed(&Keycode::Escape) {
            next_state = NextState::Pop;
        }

        Ok(next_state)
    }

    fn draw(&mut self, ctx: &mut Context) -> SceneState {
        self.level.draw(ctx, &self.camera);
        Ok(NextState::Continue)
    }

    fn draw_ui(&mut self, ctx: &mut Context, _window_size: Vector2<u32>, ui: &Ui) -> SceneState {
        let mut next_state = NextState::Continue;

        Ok(next_state)
    }

    fn background_color(&self) -> Color { *self.level.background_color() }

    fn resize_event(&mut self, _ctx: &mut Context, screen_size: Vector2<u32>) {
        self.camera.update_screen_size(screen_size);
    }
}