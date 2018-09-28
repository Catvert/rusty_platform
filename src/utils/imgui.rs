use imgui::{ImStr, ImString};
use imgui::Ui;
use imgui_sys;
use std::os::raw::c_char;
use std::ffi::CString;
use std;

fn get_c_char(s: &str) -> *const c_char {
    let s = CString::new(s.as_bytes()).unwrap();
    let p = s.as_ptr();
    std::mem::forget(s);
    p
}

pub trait ImGuiExtensions<'p> {
    fn combo_str(&self,  label: &'p ImStr, current_item: &mut i32, items: &'p [&'p str], height_in_items: i32) -> bool;
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