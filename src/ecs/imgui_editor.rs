use imgui::Ui;
use ecs::components_prelude::*;
use ggez::event::Keycode;
use imgui::{ImString, ImStr};

use std::collections::HashMap;

use ecs::actions::Actions;

use na::Vector2;

lazy_static! {
    static ref ACTIONS_WRAPPERS: HashMap<ActionsWrapper, &'static ImStr> = {
        let mut wrappers = HashMap::new();
        wrappers.insert(ActionsWrapper::Empty, im_str!("Vide"));
        wrappers.insert(ActionsWrapper::Move, im_str!("Déplacement"));
        wrappers.insert(ActionsWrapper::PhysicsMove, im_str!("Déplacement physique"));
        wrappers
    };
}

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
enum ActionsWrapper {
    Empty,
    Move,
    PhysicsMove,
}

impl ActionsWrapper {
    fn get_action(&self) -> Actions {
        match *self {
            ActionsWrapper::Empty => { Actions::Empty },
            ActionsWrapper::Move => { Actions::Move(Vector2::new(0., 0.)) },
            ActionsWrapper::PhysicsMove => { Actions::PhysicsMove(Vector2::new(0., 0.)) },
        }
    }

    fn from_action(action: &Actions) -> Self {
        match *action {
            Actions::Empty => { ActionsWrapper::Empty },
            Actions::Move(_) => { ActionsWrapper::Move },
            Actions::PhysicsMove(_) => { ActionsWrapper::PhysicsMove },
            Actions::EntityAction(_, _) => { ActionsWrapper::PhysicsMove },
            Actions::MultipleActions(_) => { ActionsWrapper::PhysicsMove },
        }
    }
}

fn draw_ui_action(mut action: Actions, popup_id: &ImStr, ui: &Ui) -> Actions {
    if ui.button(im_str!("action"), (100., 0.)) {
        ui.open_popup(popup_id);
    }

    ui.popup(popup_id, || {
        let mut pos = ACTIONS_WRAPPERS.iter().position(|aw| *aw.0 == ActionsWrapper::from_action(&action)).unwrap() as i32;
        let names: Vec<&ImStr> = ACTIONS_WRAPPERS.iter().map(|c| *c.1).collect();

        if ui.combo(im_str!("action"), &mut pos, &names, 10) {
            action = ACTIONS_WRAPPERS.iter().nth(pos as usize).unwrap().0.get_action();
        }
        match action {
            Actions::Empty => {
                ui.text("Vide !");
            },
            Actions::Move(ref mut mv) => {
                ui.slider_float(im_str!("move x"), &mut mv.x, 0., 100.).build();
                ui.slider_float(im_str!("move y"), &mut mv.y, 0., 100.).build();
            },
            Actions::PhysicsMove(ref mut mv) => {
                ui.slider_float(im_str!("move x"), &mut mv.x, 0., 100.).build();
                ui.slider_float(im_str!("move y"), &mut mv.y, 0., 100.).build();
            },
            Actions::EntityAction(_, _) => {},
            Actions::MultipleActions(_) => {},
        }
    });

    action
}

pub trait ImGuiEditor {
    fn draw_ui(&mut self, ui: &Ui);
}

impl ImGuiEditor for RectComponent {
    fn draw_ui(&mut self, ui: &Ui) {
        ui.drag_float(im_str!("x"), &mut self.pos_mut().x).build();
        ui.drag_float(im_str!("y"), &mut self.pos_mut().y).build();

        let mut w = self.size().x as i32;
        let mut h = self.size().y as i32;
        if ui.drag_int(im_str!("w"), &mut w).build() {
            self.size_mut().x = w as u32;
        }
        if ui.drag_int(im_str!("h"), &mut h).build() {
            self.size_mut().y = h as u32;
        }
    }
}

impl ImGuiEditor for InputComponent {
    fn draw_ui(&mut self, ui: &Ui) {
        for (index, (key, jp, action)) in self.input_actions.iter_mut().enumerate() {
            ui.with_id(index as i32, || {
                if ui.button(im_str!("{}", Keycode::from_i32(*key).unwrap()), (100., 0.)) {
                    ui.open_popup(im_str!("set key {}", index));
                }

                ui.popup(im_str!("set key {}", index), || {
                    let mut buf = ImString::with_capacity(1);
                    if ui.input_text(im_str!("touche"), &mut buf).build() {
                        let str: &str = buf.as_ref();
                        let c = str.chars().next().unwrap();
                        let key_code = Keycode::from_name(&format!("{}", c)).unwrap();
                        *key = key_code as i32;

                        ui.close_current_popup();
                    }
                });

                ui.same_line(0.);
                ui.checkbox(im_str!("jp"), jp);
                ui.same_line(0.);
                *action = draw_ui_action(action.clone(), im_str!("action {}", index), ui);
            });
        }

        if ui.button(im_str!("Ajouter"), (-1., 0.)) {
            self.input_actions.push((Keycode::A as i32, false, Actions::Empty));
        }
    }
}