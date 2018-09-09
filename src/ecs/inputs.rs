use std::collections::HashMap;

use specs::prelude::*;

use ggez::event::Keycode;

use ecs::actions::{Actions, ActionComponent};

use utils::input_manager::{RefInputManager, JustPressed};

#[derive(Component, Serialize, Deserialize, Clone, Default)]
pub struct InputComponent {
    pub input_actions: Vec<(i32, JustPressed, Actions)>
}

impl InputComponent {
    pub fn new(inputs: Vec<(Keycode, JustPressed, Actions)>) -> Self {
        let input_actions = inputs.into_iter().map(|(code, just_pressed, action)| (code as i32, just_pressed, action)).collect();
        InputComponent { input_actions }
    }
}

pub struct InputSystem {
    pub input_manager: RefInputManager
}

impl<'a> System<'a> for InputSystem {
    type SystemData = (Entities<'a>, ReadStorage<'a, InputComponent>, WriteStorage<'a, ActionComponent>);

    fn run(&mut self, (entities, inputs, mut actions): Self::SystemData) {
        for (ent, input) in (&*entities, &inputs).join() {
            for (key, just_pressed, action) in input.input_actions.iter() {
                if let Some(jp) = self.input_manager.lock().unwrap().is_key_pressed(&Keycode::from_i32(*key).expect(&format!("Touche {} non reconnue !", key))) {
                    if jp == just_pressed {
                        let mut inserted = false;
                        {
                            if let Some(a) = actions.get_mut(ent) {
                                a.actions_remaining.push(action.clone());
                                inserted = true;
                            }
                        }

                        if !inserted {
                            actions.insert(ent, ActionComponent { actions_remaining: vec![action.clone()] });
                        }
                    }
                }
            }
        }
    }
}
