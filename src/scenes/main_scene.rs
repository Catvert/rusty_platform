use crate::{
    ecs::level::LevelConfig,
    scenes::{
        editor_scene::EditorScene,
        game_scene::GameScene,
        NextState,
        Scene,
        SceneState,
    },
    utils::{
        constants,
        imgui::ImGuiExtensions,
        input_manager::RefInputManager,
        resources_manager::RefRM,
    },
    wrapper::imgui_wrapper::CenteredWindow,
};
use ggez::{
    Context,
    graphics::{
        self,
        Color,
        DrawParam,
        Image,
        Point2,
    },
};
use imgui::{
    im_str,
    ImGuiCol,
    ImGuiCond,
    Ui,
};
use nalgebra::Vector2;
use std::ffi::OsStr;
use walkdir::WalkDir;

pub struct MainScene {
    resources_manager: RefRM,
    input_manager: RefInputManager,
    background: Image,
    logo: Image,
    levels: Vec<LevelConfig>,
    show_levels_window: bool,
    levels_window_select_level: i32,
    show_settings_window: bool,
}

impl MainScene {
    pub fn new(ctx: &mut Context, input_manager: RefInputManager) -> Self {
        let resources_manager = RefRM::default();
        let background = resources_manager.borrow_mut().load_or_get_texture(ctx, constants::path::MAIN_MENU_BACKGROUND_FILE.as_path()).unwrap().unwrap().clone();
        let logo = resources_manager.borrow_mut().load_or_get_texture(ctx, constants::path::MAIN_MENU_LOGO_FILE.as_path()).unwrap().unwrap().clone();

        let levels: Vec<LevelConfig> = WalkDir::new(constants::path::LEVELS_DIR.as_path()).into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| { e.file_name() == OsStr::new(constants::path::LEVEL_CONFIG_FILE.as_path()) })
            .filter_map(|config_file| LevelConfig::load(config_file.path().parent().unwrap().to_owned()).ok())
            .collect();

        MainScene { resources_manager, input_manager, background, logo, levels, show_levels_window: false, levels_window_select_level: 0, show_settings_window: false }
    }
}

impl Scene for MainScene {
    fn update(&mut self, _ctx: &mut Context, _dt: f32) -> SceneState {
        Ok(NextState::Continue)
    }

    fn draw(&mut self, ctx: &mut Context) -> SceneState {
        let window_size = ctx.gfx_context.window.drawable_size();

        graphics::draw_ex(ctx, &self.background, DrawParam {
            dest: Point2::new(0.0, 0.0),
            scale: Point2::new(window_size.0 as f32 / self.background.width() as f32, window_size.1 as f32 / self.background.height() as f32),
            ..Default::default()
        })?;

        let logo_size = Vector2::new(600., 125.);

        graphics::draw_ex(ctx, &self.logo, DrawParam {
            dest: Point2::new(window_size.0 as f32 / 2. - logo_size.x / 2., 50.),
            scale: Point2::new(logo_size.x / self.logo.width() as f32, logo_size.y / self.logo.height() as f32),
            ..Default::default()
        })?;

        Ok(NextState::Continue)
    }

    fn draw_ui(&mut self, ctx: &mut Context, ui: &Ui) -> SceneState {
        let mut result = NextState::Continue;

        ui.with_color_vars(&[(ImGuiCol::WindowBg, (0., 0., 0., 0.))], || {
            ui.window(im_str!("Menu principal")).title_bar(false).movable(false).resizable(false).center(ui.frame_size(), (150., 200.), ImGuiCond::Always, ImGuiCond::Always).build(|| {
                if ui.button(im_str!("Jouer"), (-1., 0.)) {
                    self.show_levels_window = true;
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
            let MainScene { ref mut show_levels_window, ref levels, ref mut levels_window_select_level, ref resources_manager, ref input_manager, .. } = self;

            ui.window(im_str!("Sélection d'un niveau")).opened(show_levels_window).build(|| {
                ui.combo_str(im_str!("niveau"), levels_window_select_level, levels.iter().map(|l| l.name.as_str()).collect::<Vec<_>>().as_slice(), 30);

                if ui.button(im_str!("Jouer"), (-1., 0.)) {
                    if let Some(config) = levels.iter().nth(*levels_window_select_level as usize) {
                        result = NextState::Replace(Box::new(GameScene::new(ctx,input_manager.clone(), config.clone())));
                    }
                }

                if ui.button(im_str!("Éditer"), (-1., 0.)) {
                    if let Some(config) = levels.iter().nth(*levels_window_select_level as usize) {
                        result = NextState::Replace(Box::new(EditorScene::load_level(ctx,None, input_manager.clone(), config.clone())));
                    }
                }

                if ui.button(im_str!("Nouveau"), (-1., 0.)) {
                    result = NextState::Replace(Box::new(EditorScene::new_level(ctx, input_manager.clone(), String::from("test"))))
                }
            });
        }

        if self.show_settings_window {
            ui.window(im_str!("Options")).opened(&mut self.show_settings_window).resizable(false).center(ui.frame_size(), (150., 200.), ImGuiCond::Always, ImGuiCond::Once).build(|| {});
        }

        Ok(result)
    }

    fn background_color(&self) -> Color { (0, 0, 0, 255).into() }
}
