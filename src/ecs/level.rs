use std::fs::{self, File};

use ggez::Context;
use ggez::graphics::{self, BlendMode};
use ggez::graphics::Color;

use na::{Point2};

use utils::resources_manager::RefRM;
use utils::camera::Camera;
use utils::math::Rect;

use specs::{World, DispatcherBuilder, Dispatcher, Join, RunNow};
use specs::saveload::{U64Marker, U64MarkerAllocator};

use ecs::serialization::{SerializeSystem, DeserializeSystem};
use ecs::inputs::InputComponent;
use ecs::physics::{PhysicsComponent};
use ecs::chunk::{ChunkComponent, ActiveChunkMarker, ActiveChunksRect};
use ecs::render::SpriteComponent;
use ecs::rect::RectComponent;
use ecs::actions::*;
use ecs::chunk::ChunkSystem;

use serde;

use utils::serde::ColorDef;
use std::path::{Path, PathBuf};

use utils::constants;

use ron;
use std::io::Write;
use std::io::Read;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Background {
    Texture(String, #[serde(with = "ColorDef")] Color),
    Color(#[serde(with = "ColorDef")] Color)
}

impl Default for Background {
    fn default() -> Self {
        Background::Color((100, 200, 0, 255).into())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LevelConfig {
    pub author: String,
    pub name: String,
    pub background: Background,
    pub dir: PathBuf,
}

impl LevelConfig {
    pub fn load(dir: PathBuf) -> Result<Self, ron::de::Error> {
        assert!(dir.is_dir());

        let config_file = File::open(dir.join(constants::path::LEVEL_CONFIG_FILE.as_path())).map_err(|err| ron::de::Error::from(err))?;
        ron::de::from_reader::<File, Self>(config_file)
    }

    pub fn save(&self) {
        let mut config_file = File::create(self.dir.join(constants::path::LEVEL_CONFIG_FILE.as_path())).expect("Impossible de créer le fichier de configuration du niveau !");
        let content = ron::ser::to_string_pretty(&self, Default::default()).expect("Impossible de sérialiser la configuration du niveau !");
        config_file.write_all(content.as_bytes()).expect("Impossible d'écrire la configuration du niveau dans le fichier !");
    }

    fn world_data_path(&self) -> PathBuf {
        self.dir.join(constants::path::LEVEL_WORLD_DATA_FILE.as_path())
    }
}

pub struct Level<'a, 'b> {
    config: LevelConfig,
    world: World,
    dispatcher: Dispatcher<'a, 'b>,
    chunk_sys: ChunkSystem,
    resources_manager: RefRM,
    blend_mode: Option<BlendMode>
}

impl<'a, 'b> Level<'a, 'b> {
    pub fn load<F: FnMut(DispatcherBuilder<'a, 'b>) -> DispatcherBuilder<'a, 'b>>(config: LevelConfig, resources_manager: RefRM, build_dispatcher: F) -> Self {
        let (world, dispatcher, chunk_sys) = Self::build_default_world(build_dispatcher);

        DeserializeSystem { reader: File::open(&config.world_data_path()).unwrap()  }.run_now(&world.res);

        Level { config, world, dispatcher, chunk_sys, resources_manager, blend_mode: None }
    }

    pub fn new<F: FnMut(DispatcherBuilder<'a, 'b>) -> DispatcherBuilder<'a, 'b>, W: FnMut(&mut World) -> ()>(author: String, name: String, resources_manager: RefRM, build_dispatcher: F, mut populate_world: W) -> Self {
        let (mut world, dispatcher, chunk_sys) = Self::build_default_world(build_dispatcher);

        populate_world(&mut world);

        let dir = constants::path::LEVELS_DIR.join(Path::new(&name));

        let config = LevelConfig {
            author,
            name,
            background: Background::default(),
            dir
        };

       Level { config, world, dispatcher, chunk_sys, resources_manager, blend_mode: None }
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

        let dispatcher_builder = DispatcherBuilder::new();

        let mut dispatcher = build_dispatcher(dispatcher_builder).build();

        dispatcher.setup(&mut world.res);

        let mut chunk_sys = ChunkSystem::new((20, 5), Rect::new(0., 0., 1280, 720));
        chunk_sys.setup(&mut world.res);

        (world, dispatcher, chunk_sys)
    }

    pub fn background_color(&self) -> &Color {
        match self.config.background {
            Background::Texture(_, ref col) => col,
            Background::Color(ref col) => col
        }
    }

    pub fn background_color_mut(&mut self) -> &mut Color {
        match self.config.background {
            Background::Texture(_, ref mut col) => col,
            Background::Color(ref mut col) => col
        }
    }

    pub fn draw(&self, ctx: &mut Context, camera: &Camera) {
        let rects = self.world.write_storage::<RectComponent>();
        let mut sprites = self.world.write_storage::<SpriteComponent>();
        let active_chunk = self.world.write_storage::<ActiveChunkMarker>();
        let active_rect_chunk = self.world.read_resource::<ActiveChunksRect>().get_rect().clone();

        if let Background::Texture(ref _path, ref _col) = self.config.background {

        }

        let rect_in_screen = camera.world_rect_to_screen(&active_rect_chunk);

        graphics::set_color(ctx, (100, 0, 200, 255).into()).unwrap();

        graphics::rectangle(ctx, graphics::DrawMode::Line(1.), rect_in_screen.to_ggez_rect()).unwrap();

        graphics::set_color(ctx, (255, 255, 255, 255).into()).unwrap();

        for (rect, mut spr, _) in (&rects, &mut sprites, &active_chunk).join() {
            spr.draw(ctx, &rect.get_rect(), camera, &self.resources_manager);
        }

    }

    pub fn save(&self) {
        if !self.config.dir.exists() {
            fs::create_dir(&self.config.dir).unwrap();
        }

        SerializeSystem { writer: File::create(&self.config.world_data_path()).unwrap() }.run_now(&self.world.res);
        self.config.save();
    }

    pub fn update(&mut self, _context: &mut Context, camera: &Camera, _dt: f32) {
        self.dispatcher.dispatch(&self.world.res);

        self.chunk_sys.run_now(&self.world.res);

        self.world.write_resource::<ActiveChunksRect>().update_camera(camera);
        self.world.maintain();
    }
}