use specs::prelude::*;
use specs::saveload::{U64Marker, U64MarkerAllocator, MarkerAllocator, Marker};

use na::Vector2;

use ecs::rect::RectComponent;
use ecs::physics::{PhysicsComponent, NextPhysicsStep};

#[derive(Component)]
pub struct ActionComponent {
    pub actions_remaining: Vec<Actions>
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum Actions {
    Empty,
    Move(Vector2<f64>),
    PhysicsMove(Vector2<f64>),
    PhysicsJump(u32),
    DeleteEntity,
    EntityAction(Option<U64Marker>, Box<Actions>),
    MultipleActions(Vec<Actions>),
}

pub struct ActionSystem;

impl<'a> System<'a> for ActionSystem {
    type SystemData = (
        Entities<'a>,
        Read<'a, U64MarkerAllocator>,
        WriteStorage<'a, RectComponent>,
        WriteStorage<'a, ActionComponent>,
        WriteStorage<'a, PhysicsComponent>,
    );

    fn run(&mut self, (entities, u64_marker_allocator, mut rects, mut actions, mut physics): Self::SystemData) {
        fn perform_action<'a>(
            entity: Entity,
            action: &Actions,
            entities: &Entities<'a>,
            u64_marker_allocator: &U64MarkerAllocator,
            rect_storage: &mut WriteStorage<'a, RectComponent>,
            phys_storage: &mut WriteStorage<'a, PhysicsComponent>) {
            match action {
                Actions::Move(mv) => {
                    if let Some(rect) = rect_storage.get_mut(entity) {
                        rect.pos_mut().x += mv.x;
                        rect.pos_mut().y += mv.y;
                    }
                }
                Actions::PhysicsMove(mv) => {
                    if let Some(phys) = phys_storage.get_mut(entity) {
                        phys.add_step(NextPhysicsStep::Move(*mv))
                    }
                }
                Actions::PhysicsJump(height) => {
                    if let Some(phys) = phys_storage.get_mut(entity) {
                        phys.add_step(NextPhysicsStep::Jump(*height))
                    }
                }
                Actions::DeleteEntity => {
                    entities.delete(entity).unwrap();
                }
                Actions::EntityAction(u64_marker, action) => {
                    if let Some(u64_marker) = u64_marker {
                        if let Some(ent) = u64_marker_allocator.retrieve_entity_internal(u64_marker.id()) {
                            perform_action(ent, action, entities, u64_marker_allocator, rect_storage, phys_storage);
                        }
                    }
                }
                Actions::MultipleActions(actions) => {
                    for action in actions.iter() {
                        perform_action(entity, action, entities, u64_marker_allocator, rect_storage, phys_storage);
                    }
                }
                _ => {}
            }
        }

        for (ent, action) in (&*entities, &mut actions).join() {
            for action in action.actions_remaining.iter() {
                perform_action(ent, action, &entities, &u64_marker_allocator, &mut rects, &mut physics);
            }

            action.actions_remaining.clear();
        }
    }
}