use std::collections::{VecDeque, HashMap};

use specs::prelude::*;

use na::{Point2, Vector2, self};
use num;

use ecs::rect::RectComponent;
use ecs::chunk::ActiveChunkMarker;

use utils::math::Rect;
use utils::constants;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BodyType {
    Static, Dynamic
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NextPhysicsStep {
    Move(Vector2<f64>),
    Jump(u32)
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct PhysicsComponent {
    body_type: BodyType,
    apply_gravity: bool,
    next_physics_step: VecDeque<NextPhysicsStep>
}

impl PhysicsComponent {
    pub fn new(body_type: BodyType, apply_gravity: bool) -> Self {
        PhysicsComponent { body_type, apply_gravity, next_physics_step: VecDeque::new() }
    }

    pub fn add_step(&mut self, step: NextPhysicsStep) {
        self.next_physics_step.push_back(step);
    }
}

impl Default for PhysicsComponent {
    fn default() -> Self {
        PhysicsComponent { body_type: BodyType::Static, apply_gravity: false, next_physics_step: VecDeque::new() }
    }
}

pub struct PhysicsSystem {
    pub gravity: Vector2<f64>
}

impl PhysicsSystem {
    // Check if moving the rect in axis x and/or y is possible (AABB)
    fn check_move_aabb(mv_rect: &mut Rect, other_rects: &Vec<Rect>, mv_x: f64, mv_y: f64) -> (bool, bool) {
        let mut can_move_x = true;
        let mut can_move_y = true;

        for other_rect in other_rects.iter() {
            if Rect::from(Point2::new(mv_rect.pos.x + mv_x as f64, mv_rect.pos.y), mv_rect.size).overlaps(other_rect) {
                can_move_x = false;
            }

            if Rect::from(Point2::new(mv_rect.pos.x as f64, mv_rect.pos.y + mv_y as f64), mv_rect.size).overlaps(other_rect) {
                can_move_y = false;
            }
        }

        (can_move_x, can_move_y)
    }

    fn move_rec(mv_rect: &mut Rect, other_rects: &Vec<Rect>, mv_x: f64, mv_y: f64) {
        let (can_move_x, can_move_y) = Self::check_move_aabb(mv_rect, other_rects, mv_x, mv_y);

        if can_move_x {
            mv_rect.move_by(&Vector2::new(mv_x, 0.));
        } else if na::abs(&mv_x) > constants::PHYSICS_EPSILON {
            Self::move_rec(mv_rect, other_rects, mv_x - (constants::PHYSICS_EPSILON * num::signum(mv_x)), 0.);
        }

        if can_move_y {
            mv_rect.move_by(&Vector2::new(0., mv_y));
        } else if na::abs(&mv_y) > constants::PHYSICS_EPSILON {
            Self::move_rec(mv_rect, other_rects, 0., mv_y - (constants::PHYSICS_EPSILON * num::signum(mv_y)));
        }
    }
}

impl<'a> System<'a> for PhysicsSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, PhysicsComponent>,
        WriteStorage<'a, RectComponent>,
        ReadStorage<'a, ActiveChunkMarker>
    );

    fn run(&mut self, (entities, mut physics, mut rects, active_chunk): Self::SystemData) {
        let mut container: HashMap<Entity, VecDeque<NextPhysicsStep>> = HashMap::new();

        for (ent, physics_comp, _) in (&*entities, &mut physics, &active_chunk).join() {
            if physics_comp.apply_gravity {
                physics_comp.next_physics_step.push_back(NextPhysicsStep::Move(self.gravity));
            }

            while let Some(step) = physics_comp.next_physics_step.pop_front() {
                let next_steps = container.entry(ent).or_insert(VecDeque::new());
                next_steps.push_back(step);
            }
        }

        for (entity, steps) in container.iter_mut() {
            let other_rects: Vec<Rect> = (&*entities, &mut rects, &active_chunk).join().filter(|(e, _r, _)| { e != entity }).map(|(_e, r, _)| { r.get_rect().clone() }).collect();
            let this_rect = rects.get_mut(*entity).unwrap().get_rect_mut();

            while let Some(step) = steps.pop_front() {
                match step {
                    NextPhysicsStep::Move(mv) => {
                        Self::move_rec(this_rect, &other_rects, mv.x, mv.y);
                    },
                    NextPhysicsStep::Jump(height) => {
                        println!("{:?}", Self::check_move_aabb(this_rect, &other_rects, 0., -constants::PHYSICS_EPSILON));
                    }
                }
            }
        }
    }
}