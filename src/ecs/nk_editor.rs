use ecs::components_prelude::*;
use ggez::event::Keycode;

use std::collections::HashMap;

use ecs::actions::Actions;

use na::Vector2;

use nuklear::Context as NkCtx;
use nuklear::LayoutFormat;

lazy_static! {
    static ref ACTIONS_WRAPPERS: HashMap<ActionsWrapper, &'static str> = {
        let mut wrappers = HashMap::new();
        wrappers.insert(ActionsWrapper::Empty, "Vide");
        wrappers.insert(ActionsWrapper::PhysicsMove, "Déplacement physique");
        wrappers
    };
}

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
enum ActionsWrapper {
    Empty,
    PhysicsMove,
}

impl ActionsWrapper {
    fn get_action(&self) -> Actions {
        match *self {
            ActionsWrapper::Empty => { Actions::Empty },
            ActionsWrapper::PhysicsMove => { Actions::PhysicsMove(Vector2::new(0., 0.)) },
        }
    }

    fn from_action(action: &Actions) -> Self {
        match *action {
            Actions::Empty => { ActionsWrapper::Empty },
            Actions::Move(_) => { ActionsWrapper::PhysicsMove },
            Actions::PhysicsMove(_) => { ActionsWrapper::PhysicsMove },
            Actions::EntityAction(_, _) => { ActionsWrapper::PhysicsMove },
            Actions::MultipleActions(_) => { ActionsWrapper::PhysicsMove },
        }
    }
}

fn draw_ui_action(mut action: Actions, nk_ctx: &mut NkCtx) -> Actions {
    
  /*  if ui.button(im_str!("action"), (100., 0.)) {
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
*/
    action
}

pub trait NkEditor {
    fn draw_ui(&mut self, nk_ctx: &mut NkCtx);
}

impl NkEditor for RectComponent {
    fn draw_ui(&mut self, nk_ctx: &mut NkCtx) {
        nk_ctx.layout_row_dynamic(30., 1);

        nk_ctx.property_float("x".into(), 0.,&mut self.pos_mut().x, 1000., 1., 10.);

        nk_ctx.property_float("y".into(), 0.,&mut self.pos_mut().y, 1000., 1., 10.);

        let mut w = self.size().x as i32;
        let mut h = self.size().y as i32;

        self.size_mut().x = nk_ctx.propertyi("largeur".into(), 0, w, 1000, 1, 10.) as u32;

        self.size_mut().y = nk_ctx.propertyi("hauteur".into(), 0, h, 1000, 1, 10.) as u32;
    }
}

impl NkEditor for InputComponent {
    fn draw_ui(&mut self, nk_ctx: &mut NkCtx) {
        for (key, jp, action) in self.input_actions.iter_mut() {
            nk_ctx.layout_row_dynamic(30., 2);
            if nk_ctx.button_text(&format!("{}", Keycode::from_i32(*key).unwrap())) {

            }
            nk_ctx.checkbox_text("jp", jp);
            *action = draw_ui_action(action.clone(), nk_ctx);
        }

       /* for (index, (key, jp, action)) in self.input_actions.iter_mut().enumerate() {
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
        }*/
    }
}