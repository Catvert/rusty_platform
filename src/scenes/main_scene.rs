use std::sync::Arc;

use ggez::{Context};
use ggez::graphics::{self, Point2, Image, DrawParam, Color};

use na::Vector2;

use nuklear::Context as NkCtx;
use nuklear::Rect;
use nuklear::nuklear_sys::nk_window_flags_NK_WINDOW_DYNAMIC;
use nuklear::{PanelFlags, ShowState};
use nuklear::StyleItem;

use scenes::{Scene, SceneState, NextState};
use scenes::game_scene::GameScene;
use scenes::editor_scene::EditorScene;

use utils::resources_manager::RefRM;
use utils::input_manager::RefInputManager;
use nuklear::Flags;
use wrapper::nuklear_wrapper::NkFontsHolder;
use wrapper::nuklear_wrapper::NkFonts;

pub struct MainScene {
    resources_manager: RefRM,
    input_manager: RefInputManager,
    background: Image,
    show_settings_window: bool,
}

impl MainScene {
    pub fn new(ctx: &mut Context, input_manager: RefInputManager) -> Self {
        let resources_manager = RefRM::default();
        let background = resources_manager.borrow_mut().load_or_get_texture(ctx, "/game/mainmenu.png").unwrap().unwrap().clone();
        MainScene { resources_manager, input_manager, background, show_settings_window: false }
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

        Ok(NextState::Continue)
    }

    fn draw_ui(&mut self, window_size: Vector2<u32>, nk_ctx: &mut NkCtx, nk_fonts: &NkFontsHolder) -> SceneState {
        let mut result = NextState::Continue;

        let background = nk_ctx.style_mut().window_mut().fixed_background();
        nk_ctx.style_mut().window_mut().set_fixed_background(StyleItem::hide());
        if nk_ctx.begin("Menu principal".into(), Rect { x: window_size.x as f32 / 2. - 200. / 2., y: window_size.y as f32 / 2. - 200. / 2., w: 200., h: 200. }, PanelFlags::NK_WINDOW_NO_SCROLLBAR as Flags) {
            nk_ctx.layout_row_dynamic(180. / 4., 1);

            nk_ctx.style_set_font(nk_fonts.get_font(NkFonts::BigFont));

            if nk_ctx.button_text("Jouer") {
                result = NextState::Push(Box::new(GameScene::new(window_size.clone(),self.resources_manager.clone(), self.input_manager.clone(), String::from("test.lvl"))))
            }
            if nk_ctx.button_text("Nouveau") {
                result = NextState::Push(Box::new(EditorScene::new(window_size.clone(),self.resources_manager.clone(), self.input_manager.clone(), String::from("test.lvl"))))
            }
            if nk_ctx.button_text("Options") {
                self.show_settings_window = true;
            }
            if nk_ctx.button_text("Quitter") {
                result = NextState::Pop;
            }

            nk_ctx.style_set_font(nk_fonts.get_font(NkFonts::Default));
        }
        nk_ctx.end();
        nk_ctx.style_mut().window_mut().set_fixed_background(background);

        if self.show_settings_window {
            if nk_ctx.begin("Options".into(), Rect { x: 0., y: window_size.y as f32 / 2. - 200. / 2., w: 200., h: 200. }, PanelFlags::NK_WINDOW_TITLE as Flags | PanelFlags::NK_WINDOW_NO_SCROLLBAR as Flags | PanelFlags::NK_WINDOW_MOVABLE as Flags | PanelFlags::NK_WINDOW_CLOSABLE as Flags) {
                nk_ctx.layout_row_dynamic(180. / 4., 1);
                if nk_ctx.button_text("Quitter") {

                }
            } else {
                self.show_settings_window = false;
            }
            nk_ctx.end();
        }

        Ok(result)
    }

    fn background_color(&self) -> Color { (0, 0, 0, 255).into() }
}
