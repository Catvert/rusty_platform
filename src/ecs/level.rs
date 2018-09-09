use std::fs::File;

use ggez::Context;
use ggez::graphics::{self, DrawParam, BlendMode};
use ggez::graphics::spritebatch::SpriteBatch;
use ggez::event::Keycode;
use ggez::graphics::Color;

use na::{Point2, Vector2};

use utils::resources_manager::RefRM;
use utils::camera::Camera;
use utils::math::Rect;

use specs::{World, DispatcherBuilder, Dispatcher, Join, Builder, RunNow};
use specs::saveload::{U64Marker, U64MarkerAllocator, MarkedBuilder};

use ecs::serialization::{SerializeSystem, DeserializeSystem};
use ecs::inputs::InputComponent;
use ecs::physics::{PhysicsComponent, BodyType};
use ecs::chunk::{ChunkComponent, ActiveChunkMarker, ActiveChunksRect};
use ecs::render::SpriteComponent;
use ecs::rect::RectComponent;
use ecs::actions::*;
use ecs::chunk::ChunkSystem;
use utils::sprite::SpriteMode;

pub enum Background {
    Texture(String, Color),
    Color(Color)
}

pub struct Level<'a, 'b> {
    level_path: String,
    world: World,
    dispatcher: Dispatcher<'a, 'b>,
    chunk_sys: ChunkSystem,
    resources_manager: RefRM,
    background: Background,
    blend_mode: Option<BlendMode>
}

impl<'a, 'b> Level<'a, 'b> {
    pub fn load<F: FnMut(DispatcherBuilder<'a, 'b>) -> DispatcherBuilder<'a, 'b>>(level_path: String, resources_manager: RefRM, mut build_dispatcher: F) -> Self {
        let (mut world, dispatcher, chunk_sys) = Self::build_default_world(build_dispatcher);

        DeserializeSystem { reader: File::open(&level_path).unwrap()  }.run_now(&world.res);

        Level { level_path, world, dispatcher, chunk_sys, resources_manager, background: Background::Color((100, 200, 0, 255).into()), blend_mode: None }
    }

    pub fn new<F: FnMut(DispatcherBuilder<'a, 'b>) -> DispatcherBuilder<'a, 'b>, W: FnMut(&mut World) -> ()>(level_path: String, resources_manager: RefRM, mut build_dispatcher: F, mut populate_world: W) -> Self {
        let (mut world, dispatcher, chunk_sys) = Self::build_default_world(build_dispatcher);

        populate_world(&mut world);

       Level { level_path, world, dispatcher, chunk_sys, resources_manager, background: Background::Color((100, 200, 0, 255).into()), blend_mode: None }
    }

    pub fn get_world(&self) -> &World { &self.world }
    pub fn get_world_mut(&mut self) -> &mut World { &mut self.world }

    pub fn get_chunk_sys(&self) -> &ChunkSystem { &self.chunk_sys }

    fn build_default_world<F: FnMut(DispatcherBuilder<'a, 'b>) -> DispatcherBuilder<'a, 'b>>(mut build_dispatcher: F) -> (World, Dispatcher<'a, 'b>, ChunkSystem) {
        let mut world = World::new();
        world.register::<RectComponent>();
        world.register::<SpriteComponent>();
        world.register::<InputComponent>();
        world.register::<ActionComponent>();
        world.register::<PhysicsComponent>();
        world.register::<ChunkComponent>();
        world.register::<ActiveChunkMarker>();
        world.register::<U64Marker>();

        world.add_resource(U64MarkerAllocator::new());
        world.add_resource(ActiveChunksRect::new(Rect::new(0., 0., 1000, 1000), 1.5));

        let mut dispatcher_builder = DispatcherBuilder::new();

        let mut dispatcher = build_dispatcher(dispatcher_builder).build();

        dispatcher.setup(&mut world.res);

        let mut chunk_sys = ChunkSystem::new((20, 5), Rect::new(0., 0., 1280, 720));
        chunk_sys.setup(&mut world.res);

        (world, dispatcher, chunk_sys)
    }

    pub fn background_color(&self) -> Color {
        match self.background {
            Background::Texture(ref path, col) => col.clone(),
            Background::Color(col) => col.clone()
        }
    }

    pub fn draw(&self, ctx: &mut Context, camera: &Camera) {
        let rects = self.world.write_storage::<RectComponent>();
        let mut sprites = self.world.write_storage::<SpriteComponent>();
        let active_chunk = self.world.write_storage::<ActiveChunkMarker>();
        let active_rect_chunk = self.world.read_resource::<ActiveChunksRect>().get_rect().clone();

        if let Background::Texture(ref path, ref col) = self.background {

        }

        let x = camera.world_to_screen_coords(active_rect_chunk.pos);
        let size = camera.calculate_dest_scale(Point2::new(active_rect_chunk.size.x as f32, active_rect_chunk.size.y as f32));

        graphics::set_color(ctx, (100, 0, 200, 255).into());

        graphics::rectangle(ctx, graphics::DrawMode::Line(1.), graphics::Rect::new(x.x as f32, x.y as f32, size.x, -size.y));

        graphics::set_color(ctx, (255, 255, 255, 255).into());

        for (rect, mut spr, _) in (&rects, &mut sprites, &active_chunk).join() {
            use std::sync::Arc;
            use ggez::graphics::Image;

            spr.draw(ctx, &rect.get_rect(), camera, &self.resources_manager);
        }

    }

    pub fn save(&self) {
        SerializeSystem { writer: File::create(&self.level_path).unwrap() }.run_now(&self.world.res);
    }

    pub fn update(&mut self, context: &mut Context, camera: &Camera, dt: f32) {
        self.dispatcher.dispatch(&self.world.res);
        self.chunk_sys.run_now(&self.world.res);

        self.world.write_resource::<ActiveChunksRect>().update_camera(camera);
        self.world.maintain();
    }
}