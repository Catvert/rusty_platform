use crate::{
    ecs::{
        actions::ActionComponent,
        chunk::{
            ActiveChunkMarker,
            ActiveChunksRect,
            ChunkComponent,
            ChunkSystem,
        },
        inputs::InputComponent,
        physics::PhysicsComponent,
        rect::RectComponent,
        render::SpriteComponent,
    },
    utils::math::Rect,
};
use specs::{
    Builder,
    Entity,
    Join,
    LazyUpdate,
    saveload::{Marker, MarkerAllocator, U64Marker, U64MarkerAllocator},
    System,
    World,
};

pub mod level;
pub mod actions;
pub mod physics;
pub mod inputs;
pub mod serialization;
pub mod chunk;
pub mod render;
pub mod rect;
pub mod imgui_editor;
pub mod loading;

pub fn create_default_world() -> (World, ChunkSystem) {
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

    let mut chunk_sys = ChunkSystem::new((20, 5), Rect::new(0., 0., 1280, 720));
    chunk_sys.setup(&mut world.res);
    (world, chunk_sys)
}

pub fn copy_world(copy_world: &World) -> (World, ChunkSystem) {
    let copy_entities = copy_world.entities();

    let (mut world, chunk_sys) = create_default_world();

    let copy_world_u64_marker = copy_world.read_storage::<U64Marker>();

    for ent in copy_entities.join() {
        let new_ent = {
            let mut new_ent = world.create_entity();

            macro_rules! add_copy_comp {
                ($comp:ty) => {
                    match copy_world.read_storage::<$comp>().get(ent) {
                        Some(c) => {
                            new_ent = new_ent.with(c.clone());
                        },
                        None => {}
                    };
                };
            }

            add_copy_comp!(RectComponent);
            add_copy_comp!(SpriteComponent);
            add_copy_comp!(PhysicsComponent);
            add_copy_comp!(InputComponent);

            new_ent.build()
        };

        // TODO pas top
        let mut u64_marker_allocator = world.write_resource::<U64MarkerAllocator>();

        u64_marker_allocator.allocate(new_ent, Some(copy_world_u64_marker.get(ent).unwrap().id()));
    }

    (world, chunk_sys)
}

pub fn copy_entity(entity: Entity, world: &mut World) -> Entity {
    let copy_ent = world.entities().create();

    {
        let lazy_updater = world.read_resource::<LazyUpdate>();

        macro_rules! add_copy_comp {
            ($comp:ty) => {
                match world.read_storage::<$comp>().get(entity) {
                    Some(c) => lazy_updater.insert(copy_ent.clone(), c.clone()),
                    None => {}
                };
            };
        }

        add_copy_comp!(RectComponent);
        add_copy_comp!(SpriteComponent);
        add_copy_comp!(PhysicsComponent);
        add_copy_comp!(InputComponent);

        let mut alloc = world.write_resource::<<U64Marker as Marker>::Allocator>();
        alloc.mark(copy_ent, &mut world.write_storage::<U64Marker>());
    }

    world.maintain();

    copy_ent
}