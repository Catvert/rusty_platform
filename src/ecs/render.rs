use std::sync::Arc;
use ggez::graphics::Image;

use specs::prelude::*;
use ggez::Context;
use utils::sprite::Sprite;
use utils::resources_manager::RefRM;

use na::Vector2;
use utils::math::Rect;
use utils::camera::Camera;

#[derive(Component, Serialize, Deserialize, Clone, Debug)]
pub struct SpriteComponent {
    sprite: Sprite,
}

impl SpriteComponent {
    pub fn new(sprite: Sprite) -> Self {
        SpriteComponent { sprite }
    }

    pub fn get_or_load(&mut self, ctx: &mut Context, resources_manager: &RefRM) -> Image {
        self.sprite.get_or_load(ctx, resources_manager)
    }

    pub fn draw(&mut self, ctx: &mut Context, rect: &Rect, camera: &Camera, resources_manager: &RefRM) {
        self.sprite.draw(ctx, rect, camera, resources_manager);
    }
}