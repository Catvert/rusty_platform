use std::fs::File;
use std::io::Write;
use std::collections::VecDeque;

use ggez::{Context, ContextBuilder};
use ggez::conf::{WindowSetup, NumSamples, WindowMode, FullscreenType};
use ggez::graphics;
use ggez::timer;

use sdl2::event::Event::*;
use sdl2::event::WindowEvent;

use ron;

use utils::input_manager::RefInputManager;

use scenes::SceneState;
use scenes::NextState;
use scenes::main_scene::MainScene;

use utils::constants;

use na::Point2;
use na::Vector2;

use scenes::Scene;
use wrapper::imgui_wrapper::ImGuiWrapper;
use gfx_device_gl;
use sdl2::EventPump;
use imgui::ImGui;


#[derive(Serialize, Deserialize, Debug)]
struct GameConfig {
    window_size: (u32, u32),
    fullscreen_type: FullscreenType,
    borderless: bool,
    vsync: bool
}

impl GameConfig {
    fn load() -> Result<Self, ron::de::Error> {
        let config_file = File::open(constants::path::GAME_CONFIG_FILE.as_path()).map_err(|err| ron::de::Error::from(err))?;
        ron::de::from_reader::<File, Self>(config_file)
    }

    fn save(&self) {
        let mut config_file = File::create(constants::path::GAME_CONFIG_FILE.as_path()).expect("Impossible de créer le fichier de configuration !");
        let content = ron::ser::to_string_pretty(&self, Default::default()).expect("Impossible de sérialiser la configuration !");
        config_file.write_all(content.as_bytes()).expect("Impossible d'écrire la configuration dans le fichier !");
    }
}

impl Default for GameConfig {
    fn default() -> Self {
        GameConfig { window_size: (1280, 720), fullscreen_type: FullscreenType::Off, borderless: false, vsync: true }
    }
}

pub struct Game {
    ctx: Context,
    imgui_wrapper: ImGuiWrapper,
    scenes: VecDeque<Box<dyn Scene>>,
    input_manager: RefInputManager,
    exit: bool,
}

impl Game {
    pub fn new() -> Self {
        let GameConfig { window_size: (width, height), fullscreen_type, borderless, vsync } = GameConfig::load().unwrap_or_else(|err| {
            eprintln!("Le fichier de configuration est inexistant ou corrompu ! Création de la configuration par défaut.. Erreur : {}", err);
            let config = GameConfig::default();
            config.save();
            config
        });

        let window_setup = WindowSetup {
            title: "Rusty-Platform".to_string(),
            icon: "/icon.png".to_string(),
            resizable: true,
            allow_highdpi: false,
            samples: NumSamples::One
        };

        let window_mode = WindowMode {
            width,
            height,
            borderless,
            fullscreen_type,
            vsync,
            min_width: 800,
            min_height: 600,
            max_width: 1920,
            max_height: 1080
        };

        match ContextBuilder::new("platform_finisher", "finch").window_setup(window_setup).window_mode(window_mode).build() {
            Ok(mut context) => {
                let mut imgui_wrapper = ImGuiWrapper::new(&mut context);
                let input_manager = RefInputManager::default();

                let mut scenes: VecDeque<Box<dyn Scene>> = VecDeque::new();

                let mut main_scene = Box::new(MainScene::new(&mut context, input_manager.clone()));

                main_scene.init_ui(&mut context, &mut imgui_wrapper);

                scenes.push_back(main_scene);

                Game { ctx: context, imgui_wrapper, scenes, input_manager, exit: false }
            },
            Err(e) => panic!("Impossible d'initialiser le jeu ! Erreur : {}", e)
        }
    }

    fn handle_scene_state(result: SceneState, ctx: &mut Context, scenes: &mut VecDeque<Box<dyn Scene>>, imgui_wrapper: &mut ImGuiWrapper, exit: &mut bool) {
        if match result {
            Ok(state) => {
                match state {
                    NextState::Continue => { false },
                    NextState::Push(mut scene) => {
                        scene.init_ui(ctx, imgui_wrapper);
                        scenes.push_front(scene);
                        false
                    },
                    NextState::Replace(mut scene) => {
                        scene.init_ui(ctx, imgui_wrapper);

                        scenes.pop_front();
                        scenes.push_front(scene);
                        false
                    }
                    NextState::Pop => {
                        scenes.pop_front();
                        scenes.is_empty()
                    },
                }
            },
            Err(e) => {
                panic!("Erreur : {}", e);
            }
        } {
            *exit = true;
        }
    }

    fn process_events(&mut self, event_pump: &mut EventPump) {
        let mut input_manager = self.input_manager.lock().unwrap();

        for event in event_pump.poll_iter() {
            self.ctx.process_event(&event);
            self.imgui_wrapper.process_event(&event, &self.ctx);

            match event {
                Quit { .. } => {
                    self.exit = true;
                }
                KeyDown {
                    keycode,
                    ..
                } => {
                    if let Some(key) = keycode {
                        input_manager.update_key(key, true);
                    }
                }
                KeyUp {
                    keycode,
                    ..
                } => {
                    if let Some(key) = keycode {
                        input_manager.update_key(key, false);
                    }
                }
                MouseButtonDown {
                    mouse_btn, ..
                } => {
                    input_manager.update_mouse(mouse_btn, true);
                },
                MouseButtonUp {
                    mouse_btn, ..
                } => {
                    input_manager.update_mouse(mouse_btn, false);
                },
                MouseMotion {
                    x,
                    y,
                    ..
                } => {
                    input_manager.update_mouse_pos(Point2::new(x, y));
                },
                MouseWheel { x: _, y: _, .. } => {

                },
                ControllerButtonDown { button: _, which: _, .. } => {}
                ControllerButtonUp { button: _, which: _, .. } => {}
                ControllerAxisMotion {
                    axis: _, value: _, which: _, ..
                } => {},
                Window {
                    win_event: WindowEvent::FocusGained,
                    ..
                } => {},
                Window {
                    win_event: WindowEvent::FocusLost,
                    ..
                } => {},
                Window {
                    win_event: WindowEvent::Resized(w, h),
                    ..
                } => {
                    self.scenes.front_mut().unwrap().resize_event(&mut self.ctx, Vector2::new(w as u32, h as u32));

                    let new_rect = graphics::Rect::new(
                        0.0,
                        0.0,
                        w as f32,
                        h as f32,
                    );
                    graphics::set_screen_coordinates(&mut self.ctx, new_rect).unwrap();
                },
                _ => {}
            }
        }
    }

    pub fn run(&mut self) {
        let mut event_pump = self.ctx.sdl_context.event_pump().expect("Impossible d'obtenir le gestionnaire d'événements !");

        while !self.exit {
            self.ctx.timer_context.tick();

            self.process_events(&mut event_pump);

            let Game { ref mut ctx, ref mut scenes, ref mut imgui_wrapper, ref mut exit, .. } = self;

            {
                while timer::check_update_time(ctx, constants::DESIRED_FPS) {
                    let dt = 1.0 / (constants::DESIRED_FPS as f32);

                    let result = scenes.front_mut().unwrap().update(ctx, dt);
                    Self::handle_scene_state(result, ctx,scenes, imgui_wrapper, exit);

                    self.input_manager.lock().unwrap().update();
                }
            }

            graphics::set_background_color(ctx, scenes.front_mut().unwrap().background_color());

            graphics::clear(ctx);

            {
                let result = scenes.front_mut().unwrap().draw(ctx);
                Self::handle_scene_state(result, ctx, scenes, imgui_wrapper, exit);
            }

            {
                let window_size = ctx.gfx_context.window.drawable_size();
                let window_size = Vector2::new(window_size.0, window_size.1);

                let result = imgui_wrapper.render_scene_ui(ctx,scenes.front_mut().unwrap());
                Self::handle_scene_state(result, ctx, scenes, imgui_wrapper, exit);
            }

            graphics::present(ctx);
        }
    }
}

