use nuklear::*;
use ggez;

use nuklear_backend_gfx::{Drawer, GfxBackend};
use gfx::handle::RenderTargetView;
use gfx::memory::Typed;
use gfx_device_gl;
use ggez::event::MouseButton as GMB;
use na::Vector2;
use sdl2::event::Event;
use ggez::graphics::Image as ggezImage;

use ggez::event::Keycode;
use sdl2::event::WindowEvent;

const MAX_VERTEX_MEMORY: usize = 512 * 1024;
const MAX_ELEMENT_MEMORY: usize = 128 * 1024;
const MAX_COMMANDS_MEMORY: usize = 64 * 1024;

pub enum NkFonts {
    Default, BigFont
}

pub struct NkFontsHolder {
    font_atlas: FontAtlas,
    default: FontID,
    big_font: FontID
}

impl NkFontsHolder {
    pub fn get_font(&self, font: NkFonts) -> &UserFont {
        let font_id = match font {
            NkFonts::Default => { self.default },
            NkFonts::BigFont => { self.big_font },
        };

        self.font_atlas.font(font_id).unwrap().handle()
    }
}

pub struct NuklearWrapper {
    ctx: Context,
    drawer: Drawer<gfx_device_gl::Resources>,
    config: ConvertConfig,
    fonts_holder: NkFontsHolder,
    mouse_pos: (i32, i32)
}

impl NuklearWrapper {
    pub fn new(ggez_ctx: &mut ggez::Context) -> Self {
        let mut allo = Allocator::new_vec();

        let mut drawer = Drawer::new(
            &mut *ggez_ctx.gfx_context.factory,
            RenderTargetView::new(ggez_ctx.gfx_context.screen_render_target.clone()),
            36,
            MAX_VERTEX_MEMORY,
            MAX_ELEMENT_MEMORY,
            Buffer::with_size(&mut allo, MAX_COMMANDS_MEMORY), GfxBackend::OpenGlsl150);

        let mut font_atlas = FontAtlas::new(&mut allo);

        let mut font_cfg = FontConfig::with_size(0.0);
        font_cfg.set_oversample_h(3);
        font_cfg.set_oversample_v(2);
        font_cfg.set_glyph_range(font_cyrillic_glyph_ranges());
        font_cfg.set_ttf(include_bytes!("../../imgui.ttf"));

        font_cfg.set_ttf_data_owned_by_atlas(false);
        font_cfg.set_size(16f32);
        let font_16 = font_atlas.add_font_with_config(&font_cfg).unwrap();

        font_cfg.set_ttf_data_owned_by_atlas(false);
        font_cfg.set_size(32f32);
        let font_32 = font_atlas.add_font_with_config(&font_cfg).unwrap();

        let font_tex = {
            let (b, w, h) = font_atlas.bake(FontAtlasFormat::NK_FONT_ATLAS_RGBA32);
            drawer.add_texture(&mut *ggez_ctx.gfx_context.factory, b, w, h)
        };

        let mut null = DrawNullTexture::default();
        font_atlas.end(font_tex, Some(&mut null));

        let mut ctx = Context::new(&mut allo, font_atlas.font(font_16).unwrap().handle());

        let mut config = ConvertConfig::default();
        config.set_null(null.clone());
        config.set_circle_segment_count(22);
        config.set_curve_segment_count(22);
        config.set_arc_segment_count(22);
        config.set_global_alpha(0.9f32);
        config.set_shape_aa(AntiAliasing::NK_ANTI_ALIASING_ON);
        config.set_line_aa(AntiAliasing::NK_ANTI_ALIASING_ON);

        Self::style_color_custom(&mut ctx);

        NuklearWrapper { ctx, drawer, config, fonts_holder: NkFontsHolder { font_atlas, default: font_16, big_font: font_32 }, mouse_pos: (0, 0) }
    }

    fn style_color_custom(ctx: &mut Context) {
        let style = ctx.style_mut();

        macro_rules! font_white {
            ($style:expr, TEXT) => {
                $style.set_text_normal(color_rgba(255, 255, 255, 255));
                $style.set_text_hover(color_rgba(255, 255, 255, 255));
                $style.set_text_active(color_rgba(0, 0, 0, 255));
            };
            ($style:expr, LABEL) => {
                $style.set_label_normal(color_rgba(255, 255, 255, 255));
                $style.set_label_hover(color_rgba(255, 255, 255, 255));
                $style.set_label_active(color_rgba(0, 0, 0, 255));
            }
        }

        {
            let button_style = style.button_mut();
            button_style.set_normal(StyleItem::color(color_rgba_f(0.10, 0.09, 0.12, 1.00)));
            button_style.set_hover(StyleItem::color(color_rgba_f(0.24, 0.23, 0.29, 1.00)));
            button_style.set_active(StyleItem::color(color_rgba_f(0.56, 0.56, 0.58, 1.00)));

            font_white!(button_style, TEXT);
        }

        {
            let window_style = style.window_mut();
            window_style.set_fixed_background(StyleItem::color(color_rgba_f(0.10, 0.09, 0.12, 1.00)));
        }

        {
            let property_style = style.property_mut();
        }

        {
            let mut button_style = style.button().clone();
            button_style.set_border_color(color_rgba(0, 0, 0, 0));

            let combo_style = style.combo_mut();
            combo_style.set_normal(StyleItem::color(color_rgba_f(0.10, 0.09, 0.12, 1.00)));
            combo_style.set_hover(StyleItem::color(color_rgba_f(0.24, 0.23, 0.29, 1.00)));
            combo_style.set_active(StyleItem::color(color_rgba_f(0.56, 0.56, 0.58, 1.00)));

            combo_style.set_symbol_normal(color_rgba_f(0.10, 0.09, 0.12, 1.00));
            combo_style.set_symbol_hover(color_rgba_f(0.24, 0.23, 0.29, 1.00));
            combo_style.set_symbol_active(color_rgba_f(0.56, 0.56, 0.58, 1.00));

            font_white!(combo_style, LABEL);

            combo_style.set_button(button_style);
        }
    }

    /*pub fn load_icon(&mut self, ggez_ctx: &ggez::Context, image: ggezImage) -> Image {
        let (_, _, w, h) = image.get_dimensions();
        image.
        let mut hnd = self.drawer.add_texture(&mut *ggez_ctx.gfx_context.factory, &image, w, h);

        Image::with_id(hnd.id().unwrap())
    }*/

    pub fn get(&mut self) -> (&mut Context, &NkFontsHolder) { (&mut self.ctx, &self.fonts_holder) }

    pub fn process_event(&mut self, event: &Event, ggez_ctx: &ggez::Context) {
        match *event {
            Event::KeyDown { keycode, .. } => {
                if let Some(keycode) = keycode {
                    let key = match keycode {
                        Keycode::Backspace => Key::NK_KEY_BACKSPACE,
                        Keycode::Delete => Key::NK_KEY_DEL,
                        Keycode::Up => Key::NK_KEY_UP,
                        Keycode::Down => Key::NK_KEY_DOWN,
                        Keycode::Left => Key::NK_KEY_LEFT,
                        Keycode::Right => Key::NK_KEY_RIGHT,
                        _ => Key::NK_KEY_NONE
                    };

                    self.ctx.input_key(key, true);
                }
            },
            Event::KeyUp { keycode, .. } => {
                if let Some(keycode) = keycode {
                    let key = match keycode {
                        Keycode::Backspace => Key::NK_KEY_BACKSPACE,
                        Keycode::Delete => Key::NK_KEY_DEL,
                        Keycode::Up => Key::NK_KEY_UP,
                        Keycode::Down => Key::NK_KEY_DOWN,
                        Keycode::Left => Key::NK_KEY_LEFT,
                        Keycode::Right => Key::NK_KEY_RIGHT,
                        _ => Key::NK_KEY_NONE
                    };

                    self.ctx.input_key(key, false);
                }
            },
            Event::TextInput { ref text, .. } => {
                if let Some(c) = text.chars().nth(0) {
                    self.ctx.input_unicode(c);
                }
            },
            Event::MouseMotion {
                mousestate,
                x,
                y, xrel, yrel, .. } => {
                self.mouse_pos.0 = x;
                self.mouse_pos.1 = y;

                self.ctx.input_motion(x, y);
            },
            Event::MouseButtonDown {  mouse_btn, x, y, .. } => {
                let button = match mouse_btn {
                    GMB::Left => { Button::NK_BUTTON_LEFT },
                    GMB::Middle => { Button::NK_BUTTON_MIDDLE },
                    GMB::Right => { Button::NK_BUTTON_RIGHT },
                    _ => Button::NK_BUTTON_MAX
                };
                self.ctx.input_button(button, x, y, true);
            },
            Event::MouseButtonUp { mouse_btn, x, y, .. } => {
                let button = match mouse_btn {
                    GMB::Left => { Button::NK_BUTTON_LEFT },
                    GMB::Middle => { Button::NK_BUTTON_MIDDLE },
                    GMB::Right => { Button::NK_BUTTON_RIGHT },
                    _ => Button::NK_BUTTON_MAX
                };
                self.ctx.input_button(button, x, y, false);
            },
            Event::MouseWheel { .. } => {},
            Event::Window { win_event: WindowEvent::Resized(w, h), .. } => {
                self.drawer.col = Some(RenderTargetView::new(ggez_ctx.gfx_context.screen_render_target.clone()));
            }
            _ => {}
        }
    }

    pub fn render(&mut self, ggez_ctx: &mut ggez::Context, window_size: Vector2<u32>) {
        self.drawer.draw(&mut self.ctx, &mut self.config, &mut ggez_ctx.gfx_context.encoder, &mut *ggez_ctx.gfx_context.factory, window_size.x, window_size.y, Vec2 { x: 1., y: 1.});
        self.ctx.clear();
    }
}