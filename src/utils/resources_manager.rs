use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

use ggez::Context;
use ggez::graphics::Image;
use ggez::error::GameError;

use std::path::{Path, PathBuf};
use imgui::ImTexture;

pub type RefRM = Rc<RefCell<ResourcesManager>>;

#[derive(Clone)]
pub struct ResourcesManager {
    textures: HashMap<PathBuf, Image>
}

unsafe impl Send for ResourcesManager {}
unsafe impl Sync for ResourcesManager {}

impl ResourcesManager {
    pub fn load_texture(&mut self, ctx: &mut Context, path: &Path) -> Result<(), GameError> {
        if !self.textures.contains_key(path) {
            self.textures.insert(path.to_owned(), Image::new(ctx, path)?);
        }

        Ok(())
    }

    pub fn get_texture(&mut self, path: &Path) -> Option<&Image> {
        self.textures.get(path).map_or(None, |tex| Some(&tex))
    }

    pub fn load_or_get_texture(&mut self, ctx: &mut Context, path: &Path) -> Result<Option<&Image>, GameError> {
        self.load_texture(ctx, path)?;
        Ok(self.get_texture(path))
    }
}

impl Default for ResourcesManager {
    fn default() -> Self {
        ResourcesManager { textures: HashMap::new() }
    }
}
