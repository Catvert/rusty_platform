use crate::{
    ecs::{
        self,
        chunk::{
            ActiveChunksRect,
            ChunkSystem,
        },
        loading::LoadingResourcesSystem,
        render::{
            RenderSystem,
            SpriteImage
        },
        serialization::{
            DeserializeSystem,
            SerializeSystem,
        },
    },
    utils::{
        camera::Camera,
        constants,
        resources_manager::ResourcesManager,
        serde::ColorDef,
    },
};
use ggez::{
    Context,
    graphics::{
        self,
        BlendMode,
        Color,
    },
};
use ron;
use serde::{
    Deserialize,
    Serialize,
};
use specs::{
    Dispatcher,
    DispatcherBuilder,
    RunNow,
    World,
};
use std::{
    fs::{
        self,
        File,
    },
    io::Write,
    path::{
        Path,
        PathBuf,
    },
};
use specs::Entity;
use specs::saveload::U64Marker;
use specs::saveload::U64MarkerAllocator;
use specs::saveload::MarkerAllocator;
use specs::saveload::Marker;
use crate::ecs::rect::RectComponent;
use nalgebra::Point2;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Background {
    Texture(SpriteImage, #[serde(with = "ColorDef")] Color),
    Color(#[serde(with = "ColorDef")] Color),
}

impl Default for Background {
    fn default() -> Self {
        Background::Color((100, 200, 0, 255).into())
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct FollowEntity(pub Option<U64Marker>);

impl Default for FollowEntity {
    fn default() -> Self {
        FollowEntity(None)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LevelConfig {
    pub author: String,
    pub name: String,
    pub background: Background,
    pub dir: PathBuf,
    pub initial_follow_entity: FollowEntity
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
    resources_manager: ResourcesManager,
    blend_mode: Option<BlendMode>,
}

pub fn clone<'l, 'l2, F: FnMut(DispatcherBuilder<'l, 'l2>) -> DispatcherBuilder<'l, 'l2>>(level: &Level, mut build_dispatcher: F) -> Level<'l, 'l2> {
    let (mut world, chunk_sys) = ecs::copy_world(&level.world);

    let dispatcher_builder = DispatcherBuilder::new();

    let mut dispatcher = build_dispatcher(dispatcher_builder).build();

    dispatcher.setup(&mut world.res);

    Level {
        config: level.config.clone(),
        world,
        dispatcher,
        chunk_sys,
        resources_manager: level.resources_manager.clone(),
        blend_mode: level.blend_mode.clone(),
    }
}

impl<'a, 'b> Level<'a, 'b> {
    pub fn load<F: FnMut(DispatcherBuilder<'a, 'b>) -> DispatcherBuilder<'a, 'b>>(ctx: &mut Context, config: LevelConfig, resources_manager: Option<ResourcesManager>, build_dispatcher: F) -> Self {
        let (mut world, dispatcher, chunk_sys) = Self::build_default_world(build_dispatcher);

        world.write_resource::<FollowEntity>().0 = config.initial_follow_entity.0.clone();

        let mut resources_manager = resources_manager.unwrap_or_default();

        DeserializeSystem { reader: File::open(&config.world_data_path()).unwrap() }.run_now(&world.res);

        LoadingResourcesSystem { ctx, resources_manager: &mut resources_manager }.run_now(&world.res);

        Level { config, world, dispatcher, chunk_sys, resources_manager, blend_mode: None }
    }

    pub fn new<F: FnMut(DispatcherBuilder<'a, 'b>) -> DispatcherBuilder<'a, 'b>, W: FnMut(&mut World) -> ()>(ctx: &mut Context, author: String, name: String, build_dispatcher: F, mut populate_world: W) -> Self {
        let (mut world, dispatcher, chunk_sys) = Self::build_default_world(build_dispatcher);

        let mut resources_manager = ResourcesManager::default();

        populate_world(&mut world);

        LoadingResourcesSystem { ctx, resources_manager: &mut resources_manager }.run_now(&world.res);

        let dir = constants::path::LEVELS_DIR.join(Path::new(&name));

        let config = LevelConfig {
            author,
            name,
            background: Background::default(),
            dir,
            initial_follow_entity: FollowEntity::default()
        };

        Level { config, world, dispatcher, chunk_sys, resources_manager, blend_mode: None }
    }

    pub fn get_world(&self) -> &World { &self.world }
    pub fn get_world_mut(&mut self) -> &mut World { &mut self.world }

    pub fn get_chunk_sys(&self) -> &ChunkSystem { &self.chunk_sys }

    fn build_default_world<F: FnMut(DispatcherBuilder<'a, 'b>) -> DispatcherBuilder<'a, 'b>>(mut build_dispatcher: F) -> (World, Dispatcher<'a, 'b>, ChunkSystem) {
        let (mut world, chunk_sys) = ecs::create_default_world();

        let dispatcher_builder = DispatcherBuilder::new();

        let mut dispatcher = build_dispatcher(dispatcher_builder).build();

        dispatcher.setup(&mut world.res);

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
        let active_rect_chunk = self.world.read_resource::<ActiveChunksRect>().get_rect().clone();

        if let Background::Texture(ref _path, ref _col) = self.config.background {}

        let rect_in_screen = camera.world_rect_to_screen(active_rect_chunk);

        graphics::set_color(ctx, (100, 0, 200, 255).into()).unwrap();

        graphics::rectangle(ctx, graphics::DrawMode::Line(1.), rect_in_screen.to_ggez_rect()).unwrap();

        graphics::set_color(ctx, (255, 255, 255, 255).into()).unwrap();

        RenderSystem { ctx, camera }.run_now(&self.world.res);
    }

    pub fn save(&mut self) {
        if !self.config.dir.exists() {
            fs::create_dir(&self.config.dir).unwrap();
        }

        SerializeSystem { writer: File::create(&self.config.world_data_path()).unwrap() }.run_now(&self.world.res);

        self.config.initial_follow_entity = self.world.read_resource::<FollowEntity>().clone();

        self.config.save();
    }

    pub fn set_follow_camera(&self, entity: Option<Entity>) {
        *self.world.write_resource::<FollowEntity>() = FollowEntity(entity.map(|e| (*self.world.read_storage::<U64Marker>().get(e).unwrap())));
    }

    pub fn update_follow_camera(&self, camera: &mut Camera) {
        if let Some(entity) = self.world.read_resource::<FollowEntity>().0 {
            if let Some(entity) = self.world.read_resource::<U64MarkerAllocator>().retrieve_entity_internal(entity.id()) {
                if let Some(rect) = self.world.read_storage::<RectComponent>().get(entity) {
                    let pos = camera.world_point_to_screen(rect.pos());

                    camera.move_to(pos, None);
                }
            }
        }
    }

    pub fn update(&mut self, _context: &mut Context, camera: &Camera, _dt: f32) {
        self.dispatcher.dispatch(&self.world.res);

        self.chunk_sys.run_now(&self.world.res);

        self.world.write_resource::<ActiveChunksRect>().update_camera(camera);
        self.world.maintain();
    }
}