use std::fs::File;
use std::io::Write;
use std::collections::VecDeque;

use ggez::{Context, ContextBuilder};
use ggez::conf::{WindowSetup, NumSamples, WindowMode, FullscreenType};
use ggez::graphics;
use ggez::timer;

use gfx_core::handle::RenderTargetView;
use gfx_core::memory::Typed;

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

use wrapper::nuklear_wrapper::NuklearWrapper;
use scenes::Scene;
use ggez::graphics::Image;
use wrapper::imgui_wrapper::ImGuiWrapper;


const GAME_CONFIG_PATH: &'static str = "resources/config.ron";

#[derive(Serialize, Deserialize, Debug)]
struct GameConfig {
    window_size: (u32, u32),
    fullscreen_type: FullscreenType,
    borderless: bool,
    vsync: bool
}

impl GameConfig {
    fn load() -> Self {
        let config_file = File::open(GAME_CONFIG_PATH);

        let default_and_save = || {
            let config = GameConfig::default();
            config.save();
            config
        };

        match config_file {
            Ok(file) => {
                let mut content = String::new();
                ron::de::from_reader(file).unwrap_or_else(|err| {
                    println!("Le fichier de configuration est corrompu ! Création d'une nouvelle configuration..");
                    default_and_save()
                })
            }
            Err(e) => {
                println!("Le fichier de configuration est absent ! Création d'une nouvelle configuration..");
                default_and_save()
            }
        }
    }

    fn save(&self) {
        let mut config_file = File::create(GAME_CONFIG_PATH).expect("Impossible de créer le fichier de configuration !");
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
    context: Context,
    imgui_wrapper: ImGuiWrapper,
    scenes: VecDeque<Box<dyn Scene>>,
    input_manager: RefInputManager,
    exit: bool,
}

impl Game {
    pub fn new() -> Self {
        let GameConfig { window_size: (width, height), fullscreen_type, borderless, vsync } = GameConfig::load();

        let window_setup = WindowSetup {
            title: "Platform Finisher".to_string(),
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
                let imgui_wrapper = ImGuiWrapper::new(&mut context);
                let input_manager = RefInputManager::default();

                let mut scenes: VecDeque<Box<dyn Scene>> = VecDeque::new();
                scenes.push_back(Box::new(MainScene::new(&mut context, input_manager.clone())));

                Game { context, imgui_wrapper, scenes, input_manager, exit: false }
            },
            Err(e) => panic!("Impossible d'initialiser le jeu ! Erreur : {}", e)
        }
    }

    fn handle_scene_state(result: SceneState, scenes: &mut VecDeque<Box<dyn Scene>>, exit: &mut bool) {
        if match result {
            Ok(state) => {
                match state {
                    NextState::Continue => { false },
                    NextState::Push(scene) => {
                        scenes.push_front(scene);
                        false
                    },
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

    pub fn run(&mut self) {
        let mut event_pump = self.context.sdl_context.event_pump().expect("Impossible d'obtenir le gestionnaire d'événements !");

        while !self.exit {
            self.context.timer_context.tick();

            for event in event_pump.poll_iter() {
                self.context.process_event(&event);
                self.imgui_wrapper.process_event(&event, &self.context);
                match event {
                    Quit { .. } => {
                        self.exit = true;
                    }
                    KeyDown {
                        keycode,
                        keymod,
                        repeat,
                        ..
                    } => {
                        if let Some(key) = keycode {
                            self.input_manager.lock().unwrap().update_key(key, true);
                        }
                    }
                    KeyUp {
                        keycode,
                        keymod,
                        repeat,
                        ..
                    } => {
                        if let Some(key) = keycode {
                            self.input_manager.lock().unwrap().update_key(key, false);
                        }
                    }
                    MouseButtonDown {
                        mouse_btn, x, y, ..
                    } => {
                        self.input_manager.lock().unwrap().update_mouse(mouse_btn, true);
                    },
                    MouseButtonUp {
                        mouse_btn, x, y, ..
                    } => {
                        self.input_manager.lock().unwrap().update_mouse(mouse_btn, false);
                    },
                    MouseMotion {
                        mousestate,
                        x,
                        y,
                        xrel,
                        yrel,
                        ..
                    } => {
                        self.input_manager.lock().unwrap().update_mouse_pos(Point2::new(x, y));
                    },
                    MouseWheel { x, y, .. } => {},
                    ControllerButtonDown { button, which, .. } => {}
                    ControllerButtonUp { button, which, .. } => {}
                    ControllerAxisMotion {
                        axis, value, which, ..
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
                        self.scenes.front_mut().unwrap().resize_event(&mut self.context, Vector2::new(w as u32, h as u32));
                    },
                    _ => {}
                }
            }

            let Game { ref mut context, ref mut scenes, ref mut imgui_wrapper, ref mut exit, .. } = self;

            {
                while timer::check_update_time(context, constants::DESIRED_FPS) {
                    let dt = 1.0 / (constants::DESIRED_FPS as f32);

                    let result = scenes.front_mut().unwrap().update(context, dt);
                    Self::handle_scene_state(result, scenes, exit);

                    self.input_manager.lock().unwrap().update();
                }
            }

            if timer::get_ticks(context) % 100 == 0 {
                println!("Delta frame time: {:?} ", timer::get_delta(context));
                println!("Average FPS: {}", timer::get_fps(context));
            }

            graphics::set_background_color(context, scenes.front_mut().unwrap().background_color());

            graphics::clear(context);

            {
                let result = scenes.front_mut().unwrap().draw(context);
                Self::handle_scene_state(result, scenes, exit);
            }

            {
                let window_size = context.gfx_context.window.drawable_size();
                let window_size = Vector2::new(window_size.0, window_size.1);

                imgui_wrapper.render_ui(context, move |ui| {
                    let result = scenes.front_mut().unwrap().draw_ui( window_size, ui);
                    Self::handle_scene_state(result, scenes, exit)
                });
            }

            graphics::present(context);
        }
    }
}

