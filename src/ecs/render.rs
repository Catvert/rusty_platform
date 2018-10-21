use ggez::graphics::Image;

use specs::prelude::*;
use ggez::Context;
use utils::resources_manager::ResourcesManager;

use utils::math::Rect;
use utils::camera::Camera;
use ecs::rect::RectComponent;
use ecs::chunk::ActiveChunkMarker;
use std::path::PathBuf;
use ggez::graphics::{self, DrawParam};
use ggez::graphics::spritebatch::SpriteBatch;
use std::num::NonZeroU32;
use na::{Vector2, Point2};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum SpriteMode {
    Stretch,
    Repeat { x: NonZeroU32, y: NonZeroU32 }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SpriteImage {
    path: PathBuf,
    #[serde(skip)]
    image: Option<Image>
}

impl SpriteImage {
    pub fn is_loaded(&self) -> bool {
        self.image.is_some()
    }

    pub fn new_unloaded(path: PathBuf) -> Self {
        SpriteImage {
            path,
            image: None
        }
    }

    pub fn load(path: PathBuf, ctx: &mut Context, resources_manager: &mut ResourcesManager) -> Self {
        let image =  resources_manager.load_or_get_texture(ctx, &path).unwrap().cloned();
        SpriteImage {
            path,
            image
        }
    }

    pub fn load_after_deserialization<F: FnOnce(&PathBuf) -> Image>(&mut self, f: F) {
        debug_assert!(self.image.is_none());
        self.image = Some(f(&self.path));
    }

    pub fn update(&mut self, path: PathBuf, ctx: &mut Context, resources_manager: &mut ResourcesManager) {
        self.image = resources_manager.load_or_get_texture(ctx, &path).unwrap().cloned();
        self.path = path;
    }
}

#[derive(Component, Serialize, Deserialize, Clone, Debug)]
pub struct SpriteComponent {
    pub image: Option<SpriteImage>,
    pub mode: SpriteMode
}

impl SpriteComponent {
    pub fn new(image: Option<SpriteImage>, mode: SpriteMode) -> Self {
        SpriteComponent { image, mode }
    }
}

impl Default for SpriteComponent {
    fn default() -> Self {
        SpriteComponent {
            image: None,
            mode: SpriteMode::Stretch
        }
    }
}

pub struct RenderSystem<'a> {
    pub ctx: &'a mut Context,
    pub camera: &'a Camera,
}

pub fn draw_sprite(ctx: &mut Context, camera: &Camera, spr_image: &SpriteImage, rect: &Rect, mode: &SpriteMode) {
    let image = spr_image.image.as_ref().expect("Sprite non chargé finch !");

    match mode {
        SpriteMode::Stretch => {
            let scale = camera.world_size_to_screen(&Vector2::new(rect.size.x as f64 / image.width() as f64, rect.size.y as f64 / image.height() as f64));
            let pos_in_screen = camera.world_point_to_screen(&rect.pos);

            graphics::draw_ex(ctx, image, DrawParam { dest: Point2::new(pos_in_screen.x as f32, pos_in_screen.y as f32), scale: Point2::new(scale.x as f32, scale.y as f32), ..Default::default() }).unwrap();
        },
        SpriteMode::Repeat { x, y } => {
            let scale = camera.world_size_to_screen(&Vector2::new(rect.size.x as f64 / x.get() as f64 / image.width() as f64, rect.size.y as f64 / y.get() as f64 / image.height() as f64));

            let mut batch = SpriteBatch::new(image.clone());

            for x in 0..x.get() {
                for y in 0..y.get()  {
                    let pos_in_screen = camera.world_point_to_screen(&Point2::new(rect.pos.x, rect.pos.y + y as f64 * scale.y));

                    batch.add(DrawParam {
                        dest: Point2::new(pos_in_screen.x as f32 + x as f32 * scale.x as f32, pos_in_screen.y as f32),
                        scale: Point2::new(scale.x as f32, scale.y as f32),
                        ..Default::default()
                    });
                }
            }

            graphics::draw_ex(ctx, &batch, Default::default()).unwrap();
        }
        /*SpriteMode::RepeatSize(size) => {
            let scale = camera.world_size_to_screen(&Vector2::new(size.x.get() as f64 / image.width() as f64, size.y.get() as f64 / image.height() as f64));

            let mut batch = SpriteBatch::new(image.clone());

            let rows: u32 = rect.size.x / size.x.get();
            let cols: u32 = rect.size.y / size.y.get();

            for x in 0..rows {
                for y in 0..cols {
                    let pos_in_screen = camera.world_point_to_screen(&Point2::new(rect.pos.x + (x * size.x.get()) as f64, rect.pos.y + (y * size.y.get()) as f64));

                    batch.add(DrawParam {
                        dest: Point2::new(pos_in_screen.x as f32, pos_in_screen.y as f32),
                        scale: Point2::new(scale.x as f32, scale.y as f32),
                        ..Default::default()
                    });
                }
            }

            graphics::draw_ex(ctx, &batch, Default::default()).unwrap();
        }*/
    }
}

impl<'a> System<'a> for RenderSystem<'a> {
    type SystemData = (
        ReadStorage<'a, RectComponent>,
        ReadStorage<'a, SpriteComponent>,
        ReadStorage<'a, ActiveChunkMarker>
    );

    fn run(&mut self, (rects, sprites, active_chunks): Self::SystemData) {
        for (rect, spr, _) in (&rects, &sprites, &active_chunks).join() {
            if let Some(ref spr_image) = spr.image {
                draw_sprite(self.ctx, self.camera, spr_image, rect.get_rect(), &spr.mode);
            }
        }
    }
}