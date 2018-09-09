use std::sync::Arc;

use ggez::graphics::Image;
use utils::resources_manager::RefRM;
use ggez::Context;
use ggez::graphics;
use ggez::graphics::DrawParam;

use na::{Vector2, Point2};
use utils::camera::Camera;
use utils::math::Rect;
use ggez::graphics::spritebatch::SpriteBatch;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum SpriteMode {
    Stretch,
    Repeat(Vector2<u32>)
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Sprite {
    path: String,
    mode: SpriteMode,
    #[serde(skip)]
    image: Option<Image>
}

impl Sprite {
    pub fn new(path: String, mode: SpriteMode) -> Self {
        Sprite { path, mode, image: None }
    }

    pub fn get_or_load(&mut self, ctx: &mut Context, resources_manager: &RefRM) -> Image {
        if let Some(ref img) = self.image {
            img.clone()
        } else {
            let image = resources_manager.borrow_mut().load_or_get_texture(ctx, &self.path).unwrap().unwrap().clone();
            self.image = Some(image.clone());
            image
        }
    }

    pub fn draw(&mut self, ctx: &mut Context, rect: &Rect, camera: &Camera, resources_manager: &RefRM) {
        let image = self.get_or_load(ctx, resources_manager);

        match self.mode {
            SpriteMode::Stretch => {
                let scale = camera.calculate_dest_scale(Point2::new(rect.size.x as f32 / image.width() as f32, rect.size.y as f32 / image.height() as f32));
                let pos_in_screen = camera.world_to_screen_coords(Point2::new(rect.pos.x, rect.pos.y + rect.size.y as f32));

                graphics::draw_ex(ctx, &image, DrawParam { dest: Point2::new(pos_in_screen.x as f32, pos_in_screen.y as f32), scale: Point2::new(scale.x as f32, scale.y as f32), ..Default::default() }).expect("Erreur lors du dessin!")
            },
            SpriteMode::Repeat(size) => {
                let scale = camera.calculate_dest_scale(Point2::new(size.x as f32 / image.width() as f32, size.y as f32 / image.height() as f32));

                let mut batch = SpriteBatch::new(image);

                let rows: u32 = rect.size.x / size.x;
                let cols: u32 = rect.size.y / size.y;

                for x in 0..rows {
                    for y in 0..cols {
                        let pos_in_screen = camera.world_to_screen_coords(Point2::new(rect.pos.x + (x * size.x) as f32, rect.pos.y + (y * size.y) as f32 + size.y as f32));

                        batch.add(DrawParam {
                            dest: Point2::new(pos_in_screen.x as f32, pos_in_screen.y as f32),
                            scale: Point2::new(scale.x as f32, scale.y as f32),
                            ..Default::default()
                        });
                    }
                }

                graphics::draw_ex(ctx, &batch, Default::default());
            }
        }
    }
}