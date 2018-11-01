use ggez::event::{
    Keycode,
    MouseButton,
};
use nalgebra::Point2;
use std::{
    collections::HashMap,
    sync::{
        Arc,
        Mutex,
    },
};

pub type JustPressed = bool;

pub type RefInputManager = Arc<Mutex<InputManager>>;

unsafe impl Send for InputManager {}

unsafe impl Sync for InputManager {}

pub struct InputManager {
    pressed_keys: HashMap<Keycode, JustPressed>,
    pressed_mouse: HashMap<MouseButton, JustPressed>,
    mouse_pos: Point2<i32>,
    last_mouse_pos: Point2<i32>,
}

impl Default for InputManager {
    fn default() -> Self {
        InputManager::new()
    }
}

impl InputManager {
    pub fn new() -> Self {
        InputManager {
            pressed_keys: HashMap::new(),
            pressed_mouse: HashMap::new(),
            mouse_pos: Point2::new(0, 0),
            last_mouse_pos: Point2::new(0, 0),
        }
    }

    pub fn get_mouse_pos(&self) -> Point2<i32> {
        self.mouse_pos
    }

    pub fn get_delta_mouse(&self) -> Point2<i32> {
        return Point2::new(self.mouse_pos.x - self.last_mouse_pos.x, self.mouse_pos.y - self.last_mouse_pos.y);
    }

    pub fn is_key_pressed(&self, key: &Keycode) -> Option<&JustPressed> {
        self.pressed_keys.get(key)
    }

    pub fn is_mouse_pressed(&self, button: &MouseButton) -> Option<&JustPressed> {
        self.pressed_mouse.get(button)
    }

    pub fn update(&mut self) {
        for (_, just_pressed) in self.pressed_keys.iter_mut() {
            *just_pressed = false;
        }

        for (_, just_pressed) in self.pressed_mouse.iter_mut() {
            *just_pressed = false;
        }

        self.last_mouse_pos = self.mouse_pos;
    }

    pub fn update_mouse_pos(&mut self, pos: Point2<i32>) {
        self.mouse_pos = pos;
    }

    pub fn update_key(&mut self, key: Keycode, pressed: bool) {
        if pressed {
            let jp = *self.pressed_keys.get(&key).unwrap_or(&true);
            self.pressed_keys.insert(key, jp);
        } else {
            self.pressed_keys.remove(&key);
        }
    }

    pub fn update_mouse(&mut self, button: MouseButton, pressed: bool) {
        if pressed {
            let jp = *self.pressed_mouse.get(&button).unwrap_or(&true);
            self.pressed_mouse.insert(button, jp);
        } else {
            self.pressed_mouse.remove(&button);
        }
    }
}