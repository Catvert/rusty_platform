use ggez::{Context};
use ggez::graphics::{self, Point2, Image, DrawParam, Color};

use na::Vector2;

use scenes::{Scene, SceneState, NextState};
use scenes::game_scene::GameScene;
use scenes::editor_scene::EditorScene;

use utils::resources_manager::RefRM;
use utils::input_manager::RefInputManager;
use imgui::Ui;
use wrapper::imgui_wrapper::CenteredWindow;
use imgui::ImGuiCol;
use imgui::ImGuiCond;

use std::fs::File;

use walkdir::{WalkDir, DirEntry};

pub struct MainScene {
    resources_manager: RefRM,
    input_manager: RefInputManager,
    background: Image,
    logo: Image,
    levels: Vec<DirEntry>,
    show_levels_window: bool,
    show_settings_window: bool,
}

impl MainScene {
    pub fn new(ctx: &mut Context, input_manager: RefInputManager) -> Self {
        let resources_manager = RefRM::default();
        let background = resources_manager.borrow_mut().load_or_get_texture(ctx, "/game/mainmenu.png").unwrap().unwrap().clone();
        let logo = resources_manager.borrow_mut().load_or_get_texture(ctx, "/game/logo.png").unwrap().unwrap().clone();

        let levels =  WalkDir::new("resources/levels").into_iter()
            .filter_map(|e| e.ok())
            .filter(|e|  e.file_name().to_str()
                .map(|e| e.ends_with("data.pclvl"))
                .unwrap_or(false))
            .collect();

        MainScene { resources_manager, input_manager, background, logo, levels, show_levels_window: false, show_settings_window: false }
    }
}

impl Scene for MainScene {
    fn update(&mut self, _ctx: &mut Context, dt: f32) -> SceneState {
        Ok(NextState::Continue)
    }

    fn draw(&mut self, ctx: &mut Context) -> SceneState {
        let screen_size = (ctx.conf.window_mode.width, ctx.conf.window_mode.height);

        graphics::draw_ex(ctx, &self.background, DrawParam {
            dest: Point2::new(0.0, 0.0),
            scale: Point2::new(screen_size.0 as f32 / self.background.width() as f32, screen_size.1 as f32 / self.background.height() as f32),
            ..Default::default()
        })?;

        let logo_size = Vector2::new(600., 125.);

        graphics::draw_ex(ctx, &self.logo, DrawParam {
            dest: Point2::new(screen_size.0 as f32 / 2. - logo_size.x / 2., 50.),
            scale: Point2::new(logo_size.x / self.logo.width() as f32, logo_size.y / self.logo.height() as f32),
            ..Default::default()
        })?;

        Ok(NextState::Continue)
    }

    fn draw_ui(&mut self, window_size: Vector2<u32>, ui: &Ui) -> SceneState {
        let mut result = NextState::Continue;

        ui.with_color_vars(&[(ImGuiCol::WindowBg, (0., 0., 0., 0.))], || {
            ui.window(im_str!("Menu principal")).title_bar(false).movable(false).resizable(false).center(ui.frame_size(), (150., 200.), ImGuiCond::Always, ImGuiCond::Always).build(|| {
                if ui.button(im_str!("Jouer"), (-1., 0.)) {
                    result = NextState::Push(Box::new(GameScene::new(window_size.clone(),self.resources_manager.clone(), self.input_manager.clone(), String::from("test.lvl"))))
                }

                if ui.button(im_str!("Nouveau"), (-1., 0.)) {
                    result = NextState::Push(Box::new(EditorScene::new(window_size.clone(),self.resources_manager.clone(), self.input_manager.clone(), String::from("test.lvl"))))
                }

                if ui.button(im_str!("Options"), (-1., 0.)) {
                    self.show_settings_window = true;
                }

                if ui.button(im_str!("Quitter"), (-1., 0.)) {
                    result = NextState::Pop;
                }
            });
        });

        if self.show_levels_window {
            ui.window(im_str!("Sélection d'un niveau")).opened(&mut self.show_levels_window).build(|| {

            });
        }

        if self.show_settings_window {
            ui.window(im_str!("Options")).opened(&mut self.show_settings_window).resizable(false).center(ui.frame_size(), (150., 200.), ImGuiCond::Always, ImGuiCond::Once).build(||{

            });
        }

        Ok(result)
    }

    fn background_color(&self) -> Color { (0, 0, 0, 255).into() }
}
