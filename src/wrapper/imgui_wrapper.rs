use std::fs::File;
use std::io::Read;

use ggez::Context;
use ggez::event::{MouseButton, Keycode};

use imgui::{ImGui, Window, ImGuiCond, ImVec4, FrameSize, Ui, ImFontConfig, FontGlyphRange};
use imgui_sys::{igStyleColorsDark};
use imgui_gfx_renderer::{Renderer, Shaders};

use gfx_device_gl;
use gfx_core::{Factory};
use gfx_core::handle::RenderTargetView;
use gfx_core::memory::Typed;
use gfx::{CommandBuffer, Encoder};
use ggez::event::Event;
use sdl2::event::WindowEvent;
use std::time::Instant;
use scenes::Scene;
use na::Vector2;
use scenes::SceneState;

const IMGUI_TAB: u8 = 0;
const IMGUI_LEFT_ARROW: u8 = 1;
const IMGUI_RIGHT_ARROW: u8 = 2;
const IMGUI_UP_ARROW: u8 = 3;
const IMGUI_DOWN_ARROW: u8 = 3;
const IMGUI_PAGE_UP: u8 = 5;
const IMGUI_PAGE_DOWN: u8 = 6;
const IMGUI_HOME: u8 = 7;
const IMGUI_END: u8 = 8;
const IMGUI_DELETE: u8 = 9;
const IMGUI_BACKSPACE: u8 = 10;
const IMGUI_ENTER: u8 = 11;
const IMGUI_ESCAPE: u8 = 12;
const IMGUI_A: u8 = 13;
const IMGUI_C: u8 = 14;
const IMGUI_V: u8 = 15;
const IMGUI_X: u8 = 16;
const IMGUI_Y: u8 = 17;
const IMGUI_Z: u8 = 18;
const IMGUI_UNDEFINED: u8 = 19;

pub trait CenteredWindow {
    fn center(self, frame_size: FrameSize, size: (f32, f32), size_cond: ImGuiCond, pos_cond: ImGuiCond) -> Self where Self: Sized;
}

impl<'ui, 'p> CenteredWindow for Window<'ui, 'p> {
    fn center(self, frame_size: FrameSize, size: (f32, f32), size_cond: ImGuiCond, pos_cond: ImGuiCond) -> Self {
        self.size(size, size_cond).position((frame_size.logical_size.0 as f32 / 2. - size.0 / 2., frame_size.logical_size.1 as f32 / 2. - size.1 / 2.), pos_cond)
    }
}

#[derive(Copy, Clone, PartialEq, Debug, Default)]
struct MouseState {
    pos: (i32, i32),
    pressed: (bool, bool, bool),
    wheel: f32,
}

pub enum ImGuiFonts {
    Default,
    Big
}

pub struct ImGuiWrapper {
    imgui: ImGui,
    renderer: Renderer<gfx_device_gl::Resources>,
    last_frame: Instant,
    mouse_state: MouseState
}

impl ImGuiWrapper {
    pub fn new(ctx: &mut Context) -> Self {
        let mut imgui = ImGui::init();

        let mut font_file = File::open("resources/fonts/imgui.ttf").expect("La police pour l'interface n'existe pas !");
        let mut font_content = Vec::new();
        font_file.read_to_end(&mut font_content).expect("Impossible de lire la police de l'interface !");

        imgui.fonts().add_font_with_config(&font_content, ImFontConfig::new().size_pixels(20.0), &FontGlyphRange::default());

        unsafe {
            igStyleColorsDark(imgui.style_mut());
        }

        {
            // Fix incorrect colors with sRGB framebuffer
            fn imgui_gamma_to_linear(col: ImVec4) -> ImVec4 {
                let x = col.x.powf(2.2);
                let y = col.y.powf(2.2);
                let z = col.z.powf(2.2);
                let w = 1.0 - (1.0 - col.w).powf(2.2);
                ImVec4::new(x, y, z, w)
            }

            let style = imgui.style_mut();
            style.window_rounding = 10.;
            style.child_rounding = 10.;
            style.frame_rounding = 10.;

            for col in 0..style.colors.len() {
                style.colors[col] = imgui_gamma_to_linear(style.colors[col]);
            }
        }

        let shaders = {
            let version = ctx.gfx_context.device.get_info().shading_language;
            if version.is_embedded {
                if version.major >= 3 {
                    Shaders::GlSlEs300
                } else {
                    Shaders::GlSlEs100
                }
            } else if version.major >= 4 {
                Shaders::GlSl400
            } else if version.major >= 3 {
                Shaders::GlSl130
            } else {
                Shaders::GlSl110
            }
        };

        Self::configure_keys(&mut imgui);

        let renderer = Renderer::init(&mut imgui, &mut *ctx.gfx_context.factory, shaders, RenderTargetView::new(ctx.gfx_context.screen_render_target.clone())).unwrap();
        Self { imgui, renderer, last_frame: Instant::now(), mouse_state: MouseState::default() }
    }

    pub fn process_event(&mut self, event: &Event, ctx: &Context) {
        let update_imgui_key = |imgui: &mut ImGui, key: Keycode, pressed: bool| {
            let imgui_key = match key {
                Keycode::Tab => IMGUI_TAB,
                Keycode::Left => IMGUI_LEFT_ARROW,
                Keycode::Right => IMGUI_RIGHT_ARROW,
                Keycode::Up => IMGUI_UP_ARROW,
                Keycode::Down => IMGUI_DOWN_ARROW,
                Keycode::PageUp => IMGUI_PAGE_UP,
                Keycode::PageDown => IMGUI_PAGE_DOWN,
                Keycode::Home => IMGUI_HOME,
                Keycode::End => IMGUI_END,
                Keycode::Delete => IMGUI_DELETE,
                Keycode::Backspace => IMGUI_BACKSPACE,
                Keycode::Return => IMGUI_ENTER,
                Keycode::Escape => IMGUI_ESCAPE,
                Keycode::A => IMGUI_A,
                Keycode::C => IMGUI_C,
                Keycode::V => IMGUI_V,
                Keycode::X => IMGUI_X,
                Keycode::Y => IMGUI_Y,
                Keycode::Z => IMGUI_Z,
                Keycode::LCtrl | Keycode::RCtrl => {
                    imgui.set_key_ctrl(pressed);
                    IMGUI_UNDEFINED
                },
                Keycode::LAlt | Keycode::RAlt => {
                    imgui.set_key_alt(pressed);
                    IMGUI_UNDEFINED
                },
                Keycode::LShift | Keycode::RShift => {
                    imgui.set_key_shift(pressed);
                    IMGUI_UNDEFINED
                }
                _ => { IMGUI_UNDEFINED }
            };

            if imgui_key != IMGUI_UNDEFINED {
                imgui.set_key(imgui_key, pressed);
            }
        };

        match *event {
            Event::KeyDown { keycode, .. } => {
                if let Some(key) = keycode {
                    update_imgui_key(&mut self.imgui, key, true);
                }
            },
            Event::KeyUp { keycode, .. } => {
                if let Some(key) = keycode {
                    update_imgui_key(&mut self.imgui, key, false);
                }
            },
            Event::TextInput { ref text, .. } => {
                if let Some(c) = text.chars().nth(0) {
                    self.imgui.add_input_character(c);
                }
            },
            Event::MouseButtonDown {  mouse_btn, .. } => {
                match mouse_btn {
                    MouseButton::Left => {
                        self.mouse_state.pressed.0 = true;
                    },
                    MouseButton::Right => {
                        self.mouse_state.pressed.1 = true;
                    },
                    MouseButton::Middle => {
                        self.mouse_state.pressed.2 = true;
                    },
                    _ => {}
                }
            },
            Event::MouseButtonUp { mouse_btn, .. } => {
                match mouse_btn {
                    MouseButton::Left => {
                        self.mouse_state.pressed.0 = false;
                    },
                    MouseButton::Right => {
                        self.mouse_state.pressed.1 = false;
                    },
                    MouseButton::Middle => {
                        self.mouse_state.pressed.2 = false;
                    },
                    _ => {}
                }
            },
            Event::MouseMotion { x, y, .. } => {
                self.mouse_state.pos = (x, y);
            },
            Event::MouseWheel { y, .. } => {
                self.mouse_state.wheel = y as f32;
            }
            Event::Window { win_event: WindowEvent::Resized(_w, _h), .. } => {
                self.renderer.update_render_target(RenderTargetView::new(ctx.gfx_context.screen_render_target.clone()));
            }
            _ => {}
        }
    }

    fn configure_keys(imgui: &mut ImGui) {
        use imgui::ImGuiKey;

        imgui.set_imgui_key(ImGuiKey::Tab, IMGUI_TAB);
        imgui.set_imgui_key(ImGuiKey::LeftArrow, IMGUI_LEFT_ARROW);
        imgui.set_imgui_key(ImGuiKey::RightArrow, IMGUI_RIGHT_ARROW);
        imgui.set_imgui_key(ImGuiKey::UpArrow, IMGUI_UP_ARROW);
        imgui.set_imgui_key(ImGuiKey::DownArrow, IMGUI_DOWN_ARROW);
        imgui.set_imgui_key(ImGuiKey::PageUp, IMGUI_PAGE_UP);
        imgui.set_imgui_key(ImGuiKey::PageDown, IMGUI_PAGE_DOWN);
        imgui.set_imgui_key(ImGuiKey::Home, IMGUI_HOME);
        imgui.set_imgui_key(ImGuiKey::End, IMGUI_END);
        imgui.set_imgui_key(ImGuiKey::Delete, IMGUI_DELETE);
        imgui.set_imgui_key(ImGuiKey::Backspace, IMGUI_BACKSPACE);
        imgui.set_imgui_key(ImGuiKey::Enter, IMGUI_ENTER);
        imgui.set_imgui_key(ImGuiKey::Escape, IMGUI_ESCAPE);
        imgui.set_imgui_key(ImGuiKey::A, IMGUI_A);
        imgui.set_imgui_key(ImGuiKey::C, IMGUI_C);
        imgui.set_imgui_key(ImGuiKey::V, IMGUI_V);
        imgui.set_imgui_key(ImGuiKey::X, IMGUI_X);
        imgui.set_imgui_key(ImGuiKey::Y, IMGUI_Y);
        imgui.set_imgui_key(ImGuiKey::Z, IMGUI_Z);
    }

    pub fn render_scene_ui(&mut self, ctx: &mut Context, scene: &mut Box<dyn Scene>) -> SceneState {
        self.update_mouse();

        let logical_size = ctx.gfx_context.window.drawable_size();

        let frame_size = FrameSize {
            logical_size: (logical_size.0 as f64, logical_size.1 as f64),
            hidpi_factor: 1.0,
        };

        let now = Instant::now();
        let delta = now - self.last_frame;
        let delta_s = delta.as_secs() as f32 + delta.subsec_nanos() as f32 / 1_000_000_000.0;
        self.last_frame = now;

        let ui = self.imgui.frame(frame_size, delta_s);

        let next_scene_state = scene.draw_ui(ctx, Vector2::new(logical_size.0, logical_size.1), &ui);

        let factory = &mut *ctx.gfx_context.factory;
        let encoder = &mut ctx.gfx_context.encoder;

        self.renderer.render(ui, &mut *factory, encoder).expect("Un problème est survenu lors de l'affichage d'ImGui !");

        next_scene_state
    }

    pub fn render_ui_ex<R: FnMut(&Ui) -> (), F: Factory<gfx_device_gl::Resources>, C: CommandBuffer<gfx_device_gl::Resources>>(&mut self, logical_size: (u32, u32), factory: &mut F, encoder: &mut Encoder<gfx_device_gl::Resources, C>,  mut run_ui: R) {
        self.update_mouse();

        let frame_size = FrameSize {
            logical_size: (logical_size.0 as f64, logical_size.1 as f64),
            hidpi_factor: 1.0,
        };

        let now = Instant::now();
        let delta = now - self.last_frame;
        let delta_s = delta.as_secs() as f32 + delta.subsec_nanos() as f32 / 1_000_000_000.0;
        self.last_frame = now;

        let ui = self.imgui.frame(frame_size, delta_s);

        run_ui(&ui);

        self.renderer.render(ui, &mut *factory, encoder).expect("Un problème est survenu lors de l'affichage d'ImGui !");
    }

    fn update_mouse(&mut self) {
        self.imgui.set_mouse_pos(self.mouse_state.pos.0 as f32, self.mouse_state.pos.1 as f32);
        self.imgui.set_mouse_down([
            self.mouse_state.pressed.0,
            self.mouse_state.pressed.1,
            self.mouse_state.pressed.2,
            false,
            false,
        ]);
        self.imgui.set_mouse_wheel(self.mouse_state.wheel);
        self.mouse_state.wheel = 0.0;
    }
}