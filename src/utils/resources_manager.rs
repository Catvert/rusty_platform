use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

use ggez::Context;
use ggez::graphics::Image;
use ggez::error::GameError;

pub type RefRM = Rc<RefCell<ResourcesManager>>;

pub struct ResourcesManager {
    textures: HashMap<String, Image>
}

unsafe impl Send for ResourcesManager {}
unsafe impl Sync for ResourcesManager {}

impl ResourcesManager {
    pub fn load_texture(&mut self, ctx: &mut Context, path: &str) -> Result<(), GameError> {
        let string = String::from(path);
        if !self.textures.contains_key(&string) {
            self.textures.insert(string, Image::new(ctx, path)?);
        }

        Ok(())
    }

    pub fn get_texture(&mut self, path: &str) -> Option<&Image> {
        self.textures.get(&String::from(path)).map_or(None, |tex| Some(&tex))
    }

    pub fn load_or_get_texture(&mut self, ctx: &mut Context, path: &str) -> Result<Option<&Image>, GameError> {
        self.load_texture(ctx, path)?;
        Ok(self.get_texture(path))
    }
}

impl Default for ResourcesManager {
    fn default() -> Self {
        ResourcesManager { textures: HashMap::new() }
    }
}
