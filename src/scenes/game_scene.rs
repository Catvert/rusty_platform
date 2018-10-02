use ggez::Context;
use ggez::graphics::Color;

use na::Vector2;

use scenes::{Scene, SceneState, NextState};

use ecs::level::Level;
use ecs::inputs::InputSystem;
use ecs::actions::ActionSystem;

use utils::camera::Camera;
use utils::input_manager::RefInputManager;
use utils::resources_manager::RefRM;
use ecs::physics::PhysicsSystem;
use wrapper::imgui_wrapper::CenteredWindow;

use imgui::Ui;
use ggez::event::Keycode;
use imgui::ImGuiCond;
use std::path::PathBuf;
use ecs::level::LevelConfig;
use scenes::main_scene::MainScene;

pub struct GameScene<'a, 'b> {
    level: Level<'a, 'b>,
    input_manager: RefInputManager,
    camera: Camera,
    show_exit_menu: bool
}

impl<'a, 'b> GameScene<'a, 'b> {
    pub fn new(screen_size: Vector2<u32>, resources_manager: RefRM, input_manager: RefInputManager, level_config: LevelConfig) -> Self {
        let level = Level::load(level_config, resources_manager, |builder| {
            builder.with(InputSystem { input_manager: input_manager.clone() }, "input_manager", &[])
                .with(ActionSystem, "action_system", &["input_manager"])
                .with(PhysicsSystem { gravity: Vector2::new(0., 0.) }, "phys_sys", &["action_system"])
        });

        let camera = Camera::new(screen_size,1.);

        GameScene { level, input_manager, camera, show_exit_menu: false }
    }
}

impl<'a, 'b> Scene for GameScene<'a, 'b> {
    fn update(&mut self, ctx: &mut Context, dt: f32) -> SceneState {
        self.level.update(ctx, &self.camera, dt);

        if let Some(true) = self.input_manager.lock().unwrap().is_key_pressed(&Keycode::Escape) {
            self.show_exit_menu = true;
        }

        Ok(NextState::Continue)
    }

    fn draw(&mut self, ctx: &mut Context) -> SceneState {
        self.level.draw( ctx, &self.camera);
        Ok(NextState::Continue)
    }

    fn draw_ui(&mut self, ctx: &mut Context, _window_size: Vector2<u32>, ui: &Ui) -> SceneState {
        let mut next_state = NextState::Continue;

        if self.show_exit_menu {
            ui.window(im_str!("Menu")).title_bar(false).resizable(false).center(ui.frame_size(), (200., 100.), ImGuiCond::Always, ImGuiCond::Always).build(|| {
                if ui.button(im_str!("Reprendre"), (-1., 25.)) {
                    self.show_exit_menu = false;
                }
                if ui.button(im_str!("Recommencer"), (-1., 25.)) {

                }
                if ui.button(im_str!("Quitter"), (-1., 25.)) {
                    next_state = NextState::Replace(Box::new(MainScene::new(ctx, self.input_manager.clone())));
                }
            });
        }

        Ok(next_state)
    }

    fn background_color(&self) -> Color { *self.level.background_color() }
}