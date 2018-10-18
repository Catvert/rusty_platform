

use std::collections::HashMap;

use imgui::{Ui, ImString, ImStr};

use ggez::event::Keycode;

use ecs::components_prelude::*;
use ecs::actions::Actions;

use na::Vector2;
use ecs::render::SpriteMode;

trait EnumCombo {
    type Enum;

    fn draw_ui_combo(mut self, ui: &Ui) -> Self;
    fn from_wrapper(&self) -> Self::Enum;
    fn to_enum(&self, e: &Self::Enum) -> Self;
}

macro_rules! impl_enum_ui_combo_wrapper {
    ($e:ident, $combo_name:expr; [$ ($wrap_variant:ident => $name:expr; $enum_pattern:pat, $enum_build: expr), *]) => {
        mashup! {
            m["wrapper"] = EnumComboWrapper $e;
            m["wrapper_imstr"] = ENUM_COMBO_WRAPPER_ $e _imstr;
        }

        m! {
            #[derive(Eq, PartialEq, Hash, Clone, Debug)]
            enum "wrapper" {
                 $($wrap_variant),*
            }
        }

        m! {
            use self::"wrapper"::*;
        }

        m! {
            lazy_static! {
                static ref "wrapper_imstr": HashMap<"wrapper", &'static ImStr> = {
                    let mut wrappers = HashMap::new();
                    $(
                        wrappers.insert($wrap_variant, im_str!($name));
                    )*
                    wrappers
                };
            }
        }

        impl EnumCombo for $e {
            m! {
                type Enum = "wrapper";
            }

            fn draw_ui_combo(mut self, ui: &Ui) -> Self {
                m! {
                    let mut pos = "wrapper_imstr".iter().position(|aw| *aw.0 == self.from_wrapper()).unwrap() as i32;
                }

                m! {
                    let names: Vec<&ImStr> = "wrapper_imstr".iter().map(|c| *c.1).collect();
                }

                if ui.combo(im_str!($combo_name), &mut pos, &names, 10) {
                    m! {
                        self = self.to_enum("wrapper_imstr".iter().nth(pos as usize).unwrap().0);
                    }
                }

                self
            }

            fn from_wrapper(&self) -> Self::Enum {
                match *self {
                    $(
                        $enum_pattern => $wrap_variant
                    ),*
                }
            }

            fn to_enum(&self, e: &Self::Enum) -> Self {
                match *e {
                    $(
                        $wrap_variant => $enum_build
                    ),*
                }
            }
        }
    };
}

impl_enum_ui_combo_wrapper!(Actions, "action";
[
    Empty => "Vide"; Actions::Empty, Actions::Empty,
    Move => "Déplacement"; Actions::Move(_), Actions::Move(Vector2::new(0., 0.)),
    PhysicsMove => "Déplacement physique"; Actions::PhysicsMove(_), Actions::PhysicsMove(Vector2::new(0., 0.)),
    PhysicsJump => "Saut physique"; Actions::PhysicsJump(_), Actions::PhysicsJump(0),
    DeleteEntity => "Supprimer l'entité"; Actions::DeleteEntity, Actions::DeleteEntity,
    MultipleActions => "Actions multiple"; Actions::MultipleActions(_), Actions::MultipleActions(vec![]),
    EntityAction => "Actions sur une entité"; Actions::EntityAction(_, _), Actions::EntityAction(None, Box::new(Actions::Empty))
]);

fn draw_ui_action(mut action: Actions, popup_id: &ImStr, ui: &Ui) -> Actions {
    if ui.button(im_str!("action"), (100., 0.)) {
        ui.open_popup(popup_id);
    }

    ui.popup(popup_id, || {
        action = action.clone().draw_ui_combo(ui);

        match action {
            Actions::Empty => {
                ui.text("Vide !");
            }
            Actions::Move(ref mut mv) => {
                let mut x = mv.x as f32;
                let mut y = mv.y as f32;

                if ui.slider_float(im_str!("move x"), &mut x, 0., 100.).build() {
                    mv.x = x as f64;
                }
                if ui.slider_float(im_str!("move y"), &mut y, 0., 100.).build() {
                    mv.y = y as f64;
                }
            }
            Actions::PhysicsMove(ref mut mv) => {
                let mut x = mv.x as f32;
                let mut y = mv.y as f32;

                if ui.slider_float(im_str!("move x"), &mut x, 0., 100.).build() {
                    mv.x = x as f64;
                }
                if ui.slider_float(im_str!("move y"), &mut y, 0., 100.).build() {
                    mv.y = y as f64;
                }
            }
            Actions::PhysicsJump(ref mut height) => {
                let mut height_i32 = *height as i32;
                if ui.slider_int(im_str!("height"), &mut height_i32, 0, 100).build() {
                    *height = height_i32 as u32;
                }
            }
            Actions::DeleteEntity => {}
            Actions::EntityAction(_, _) => {}
            Actions::MultipleActions(_) => {}
        }
    });

    action
}

pub trait ImGuiEditor {
    fn draw_ui(&mut self, ui: &Ui);
}

impl ImGuiEditor for RectComponent {
    fn draw_ui(&mut self, ui: &Ui) {
        let mut x = self.pos_mut().x as f32;
        let mut y = self.pos_mut().y as f32;

        if ui.drag_float(im_str!("x"), &mut x).build() {
            self.pos_mut().x = x as f64;
        }
        if ui.drag_float(im_str!("y"), &mut y).build() {
            self.pos_mut().y = y as f64;
        }

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

impl ImGuiEditor for SpriteComponent {
    fn draw_ui(&mut self, ui: &Ui) {}
}

impl ImGuiEditor for PhysicsComponent {
    fn draw_ui(&mut self, _ui: &Ui) {}
}