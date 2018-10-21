use std::collections::{VecDeque, HashMap};

use specs::prelude::*;

use na::{Point2, Vector2, self};
use num;

use ecs::rect::RectComponent;
use ecs::chunk::ActiveChunkMarker;

use utils::math::Rect;
use utils::constants;

use std::mem;

#[derive(Debug, Clone)]
pub struct JumpData {
    target_height: i32
}

impl JumpData {
    fn new(target_height: i32) -> Self {
        JumpData {
            target_height
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BodyType {
    Static,
    Dynamic {
        apply_gravity: bool,
        #[serde(skip)]
        jump_data: Option<JumpData>,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NextPhysicsStep {
    Move(Vector2<f64>),
    Jump(u32)
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct PhysicsComponent {
    pub body_type: BodyType,
    next_physics_step: VecDeque<NextPhysicsStep>
}

impl PhysicsComponent {
    pub fn new(body_type: BodyType) -> Self {
        PhysicsComponent { body_type, next_physics_step: VecDeque::new() }
    }

    pub fn add_step(&mut self, step: NextPhysicsStep) {
        self.next_physics_step.push_back(step);
    }
}

impl Default for PhysicsComponent {
    fn default() -> Self {
        PhysicsComponent { body_type: BodyType::Static, next_physics_step: VecDeque::new() }
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
        for (ent, physics_comp, _) in (&*entities, &mut physics, &active_chunk).join() {
            if let BodyType::Dynamic { ref apply_gravity, ref mut jump_data } = physics_comp.body_type {
                let next_physics_step = &mut physics_comp.next_physics_step;

                let stop_jump = if let Some(jump_data) = jump_data {
                    next_physics_step.push_back(NextPhysicsStep::Move(Vector2::new(0., -self.gravity.y)));

                    if jump_data.target_height >= rects.get_mut(ent).unwrap().get_rect().pos.y as i32 - self.gravity.y as i32 {
                        true
                    } else {
                        false
                    }
                } else {
                    if *apply_gravity {
                        next_physics_step.push_back(NextPhysicsStep::Move(self.gravity));
                    }

                    false
                };

                if stop_jump {
                    *jump_data = None;
                }

                if !next_physics_step.is_empty() {
                    let other_rects: Vec<Rect> = (&*entities, &mut rects, &active_chunk).join().filter(|(e, _r, _)| { *e != ent }).map(|(_e, r, _)| { r.get_rect().clone() }).collect();
                    let this_rect = rects.get_mut(ent).unwrap().get_rect_mut();

                    while let Some(step) = next_physics_step.pop_front() {
                        match step {
                            NextPhysicsStep::Move(mv) => {
                                Self::move_rec(this_rect, &other_rects, mv.x, mv.y);
                            },
                            NextPhysicsStep::Jump(height) => {
                                if let None = jump_data {
                                    if !Self::check_move_aabb(this_rect, &other_rects, 0., constants::PHYSICS_EPSILON).1 {
                                        *jump_data = Some(JumpData::new(this_rect.pos.y as i32 - height as i32));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}