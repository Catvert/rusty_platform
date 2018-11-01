use crate::{
    ecs::render::SpriteComponent,
    utils::resources_manager::ResourcesManager,
};
use ggez::Context;
use specs::{
    Join,
    System,
    WriteStorage,
};

pub struct LoadingResourcesSystem<'a> {
    pub ctx: &'a mut Context,
    pub resources_manager: &'a mut ResourcesManager,
}

impl<'a> System<'a> for LoadingResourcesSystem<'a> {
    type SystemData = WriteStorage<'a, SpriteComponent>;

    fn run(&mut self, mut sprite: Self::SystemData) {
        for spr in (&mut sprite).join() {
            if let Some(ref mut spr_image) = spr.image {
                if !spr_image.is_loaded() {
                    spr_image.load_after_deserialization(|path| {
                        self.resources_manager.load_or_get_texture(self.ctx, path).unwrap().unwrap().clone()
                    });
                }
            }
        }
    }
}