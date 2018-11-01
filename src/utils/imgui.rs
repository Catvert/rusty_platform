use gfx;
use gfx_device_gl;
use ggez::graphics;
use imgui::{
    ImStr,
    Ui,
};
use imgui_gfx_renderer;
use imgui_sys;
use std::{
    ffi::CString,
    os::raw::c_char,
};

fn get_c_char(s: &str) -> *const c_char {
    let s = CString::new(s.as_bytes()).unwrap();
    let p = s.as_ptr();
    std::mem::forget(s);
    p
}

pub trait ImGuiExtensions<'p> {
    fn combo_str(&self, label: &'p ImStr, current_item: &mut i32, items: &'p [&'p str], height_in_items: i32) -> bool;
}

impl<'p> ImGuiExtensions<'p> for Ui<'p> {
    fn combo_str(&self, label: &'p ImStr, current_item: &mut i32, items: &'p [&'p str], height_in_items: i32) -> bool {
        let items_inner: Vec<*const c_char> = items.into_iter().map(|item| get_c_char(item)).collect();
        unsafe {
            imgui_sys::igCombo(
                label.as_ptr(),
                current_item,
                items_inner.as_ptr() as *mut *const c_char,
                items_inner.len() as i32,
                height_in_items,
            )
        }
    }
}

pub trait ToImGuiTex<R: gfx::Resources, F: gfx::Factory<R>> {
    fn to_imgui_tex(&self, factory: &mut F) -> imgui_gfx_renderer::Texture<R>;
}

impl ToImGuiTex<gfx_device_gl::Resources, gfx_device_gl::Factory> for graphics::Image {
    fn to_imgui_tex(&self, factory: &mut gfx_device_gl::Factory) -> (gfx::handle::ShaderResourceView<gfx_device_gl::Resources, [f32; 4]>, gfx::handle::Sampler<gfx_device_gl::Resources>) {
        use gfx::texture::{SamplerInfo, FilterMethod, WrapMode};
        use gfx::memory::Typed;
        use gfx::Factory;

        let sampler = factory.create_sampler(SamplerInfo::new(FilterMethod::Trilinear, WrapMode::Clamp));
        let shader_view = gfx::handle::ShaderResourceView::new(self.texture.clone());
        (shader_view, sampler)
    }
}