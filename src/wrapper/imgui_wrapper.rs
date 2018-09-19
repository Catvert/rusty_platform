use std::fs::File;
use std::io::Read;

use ggez::Context;
use ggez::event::{MouseButton, Keycode};

use utils::constants;

use imgui::{ImGui, Window, ImGuiCond, ImVec4, FrameSize, Ui, ImFontConfig, FontGlyphRange, ImFont};
use imgui_sys::{igStyleColorsDark, igPushFont, igPopFont};
use imgui_gfx_renderer::{Renderer, Shaders};

use gfx_device_gl;
use gfx_core::{self, Factory};
use gfx_core::handle::RenderTargetView;
use gfx_core::memory::Typed;
use gfx::{CommandBuffer, Encoder};
use ggez::event::Event;
use sdl2::event::WindowEvent;

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
    fn center(mut self, frame_size: FrameSize, size: (f32, f32), size_cond: ImGuiCond, pos_cond: ImGuiCond) -> Self where Self: Sized {
        self // fix pattern warning
    }
}

impl<'ui, 'p> CenteredWindow for Window<'ui, 'p> {
    fn center(mut self, frame_size: FrameSize, size: (f32, f32), size_cond: ImGuiCond, pos_cond: ImGuiCond) -> Self {
        self.size(size, size_cond).position((frame_size.logical_size.0 as f32 / 2. - size.0 / 2., frame_size.logical_size.1 as f32 / 2. - size.1 / 2.), pos_cond)
    }
}

pub enum ImGuiFonts {
    Default,
    Big
}

pub enum ImGuiParseKeycodeModifier {
    Ctrl, Alt, Shift
}

pub enum ImGuiParseKeycode {
    Modifier(ImGuiParseKeycodeModifier),
    Char(char, Option<u8>),
    ImGui(u8),
    None
}

pub struct ImGuiWrapper {
    imgui: ImGui,
    renderer: Renderer<gfx_device_gl::Resources>,
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
        Self { imgui, renderer }
    }

    pub fn process_event(&mut self, event: &Event, ctx: &Context) {
        let mut update_imgui_key = |imgui: &mut ImGui, key: Keycode, pressed: bool| {
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
                self.imgui.set_mouse_down([
                    mouse_btn == MouseButton::Left,
                    mouse_btn == MouseButton::Right,
                    mouse_btn == MouseButton::Middle,
                    false,
                    false
                ]);
            },
            Event::MouseButtonUp { mouse_btn, .. } => {
                self.imgui.set_mouse_down([
                    mouse_btn != MouseButton::Left,
                    mouse_btn != MouseButton::Right,
                    mouse_btn != MouseButton::Middle,
                    false,
                    false
                ]);
            },
            Event::MouseMotion { x, y, .. } => {
                self.imgui.set_mouse_pos(x as f32, y as f32);
            },
            Event::Window { win_event: WindowEvent::Resized(w, h), .. } => {
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

    pub fn render_ui<F: FnMut(&Ui) -> ()>(&mut self, ctx: &mut Context, mut run_ui: F) {
        let logical_size = ctx.gfx_context.window.drawable_size();
        let factory = &mut *ctx.gfx_context.factory;
        let encoder = &mut ctx.gfx_context.encoder;

        self.render_ui_ex(logical_size, factory, encoder, run_ui);
    }

    pub fn render_ui_ex<R: FnMut(&Ui) -> (), F: Factory<gfx_device_gl::Resources>, C: CommandBuffer<gfx_device_gl::Resources>>(&mut self, logical_size: (u32, u32), factory: &mut F, encoder: &mut Encoder<gfx_device_gl::Resources, C>,  mut run_ui: R) {
        let frame_size = FrameSize {
            logical_size: (logical_size.0 as f64, logical_size.1 as f64),
            hidpi_factor: 1.0,
        };

        let ui = self.imgui.frame(frame_size, 1.0 / (constants::DESIRED_FPS as f32));

        run_ui(&ui);

        self.renderer.render(ui, &mut *factory, encoder).expect("Un problème est survenu lors de l'affichage d'ImGui !");
    }
}