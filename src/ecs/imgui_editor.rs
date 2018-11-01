use crate::ecs::{
    actions::Actions,
    inputs::InputComponent,
    physics::{
        BodyType,
        PhysicsComponent,
    },
    rect::RectComponent,
    render::{
        SpriteComponent,
        SpriteMode,
    },
};
use ggez::event::Keycode;
use imgui::{
    ImStr,
    ImString,
    Ui,
};
use imgui::im_str;
use lazy_static::lazy_static;
use nalgebra::Vector2;
use std::{
    collections::HashMap,
    num::NonZeroU32,
};

trait EnumCombo where Self: Sized {
    type Wrapper;

    fn draw_ui_combo(&mut self, ui: &Ui) -> Option<Self>;
    fn to_wrapper(&self) -> Self::Wrapper;
    fn to_enum(&self, w: &Self::Wrapper) -> Self;
}

macro_rules! impl_enum_ui_combo_wrapper {
    ($wrapper: ident, $wrapper_imstr: ident, $e:ident, $combo_name:expr; [$ ($wrap_variant:ident => $name:expr; $enum_pattern:pat, $enum_build: expr), *]) => {
        #[derive(Eq, PartialEq, Hash, Clone, Debug)]
        enum $wrapper {
            $($wrap_variant),*
        }

       use self::$wrapper::*;
            lazy_static! {
                static ref $wrapper_imstr: HashMap<$wrapper, &'static ImStr> = {
                    let mut wrappers = HashMap::new();
                    $(
                        wrappers.insert($wrap_variant, im_str!($name));
                    )*
                    wrappers
                };
            }


        impl EnumCombo for $e {
            type Wrapper = $wrapper;

            fn draw_ui_combo(&mut self, ui: &Ui) -> Option<Self> {
                let mut pos = $wrapper_imstr.iter().position(|aw| *aw.0 == self.to_wrapper()).unwrap() as i32;

                let names: Vec<&ImStr> =  $wrapper_imstr.iter().map(|c| *c.1).collect();

                if ui.combo(im_str!($combo_name), &mut pos, &names, 10) {

                        return Some(self.to_enum($wrapper_imstr.iter().nth(pos as usize).unwrap().0));

                }

                None
            }

            fn to_wrapper(&self) -> Self::Wrapper {
                match *self {
                    $(
                        $enum_pattern => $wrap_variant
                    ),*
                }
            }

            fn to_enum(&self, w: &Self::Wrapper) -> Self {
                match *w {
                    $(
                        $wrap_variant => $enum_build
                    ),*
                }
            }
        }
    };
}


impl_enum_ui_combo_wrapper!(ActionsWrapperMod, ActionsWrapperImStr, Actions, "action";
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
        if let Some(a) = action.draw_ui_combo(ui) {
            action = a;
        }

        match action {
            Actions::Empty => {
                ui.text("Vide !");
            }
            Actions::Move(ref mut mv) => {
                let mut x = mv.x as f32;
                let mut y = mv.y as f32;

                if ui.slider_float(im_str!("move x"), &mut x, -100., 100.).build() {
                    mv.x = x as f64;
                }
                if ui.slider_float(im_str!("move y"), &mut y, -100., 100.).build() {
                    mv.y = y as f64;
                }
            }
            Actions::PhysicsMove(ref mut mv) => {
                let mut x = mv.x as f32;
                let mut y = mv.y as f32;

                if ui.slider_float(im_str!("move x"), &mut x, -100., 100.).build() {
                    mv.x = x as f64;
                }
                if ui.slider_float(im_str!("move y"), &mut y, -100., 100.).build() {
                    mv.y = y as f64;
                }
            }
            Actions::PhysicsJump(ref mut height) => {
                let mut height_i32 = *height as i32;
                if ui.slider_int(im_str!("height"), &mut height_i32, 0, 1000).build() {
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

impl_enum_ui_combo_wrapper!(SpriteModeWrapper, SpriteModeWrapperImStr, SpriteMode, "mode"; [
    Stretch => "Remplir"; SpriteMode::Stretch, SpriteMode::Stretch,
    Repeat => "Répéter"; SpriteMode::Repeat { .. }, SpriteMode::Repeat { x: NonZeroU32::new(1).unwrap(), y: NonZeroU32::new(1).unwrap() }
]);

impl ImGuiEditor for SpriteComponent {
    fn draw_ui(&mut self, ui: &Ui) {
        if let Some(m) = self.mode.draw_ui_combo(ui) {
            self.mode = m;
        }

        match self.mode {
            SpriteMode::Stretch => {}
            SpriteMode::Repeat { ref mut x, ref mut y } => {
                let mut x_i32 = x.get() as i32;
                let mut y_i32 = y.get() as i32;

                if ui.drag_int(im_str!("x"), &mut x_i32).build() {
                    if x_i32 > 0 {
                        *x = NonZeroU32::new(x_i32 as u32).unwrap();
                    }
                }
                if ui.drag_int(im_str!("y"), &mut y_i32).build() {
                    if y_i32 > 0 {
                        *y = NonZeroU32::new(y_i32 as u32).unwrap();
                    }
                }
            }
        }
    }
}


impl_enum_ui_combo_wrapper!(BodyTypeWrapper, BodyTypeWrapperImStr, BodyType, "type"; [
    Static => "Statique"; BodyType::Static, BodyType::Static,
    Dynamic => "Dynamique"; BodyType::Dynamic { .. }, BodyType::Dynamic { apply_gravity: false, jump_data: None }
]);

impl ImGuiEditor for PhysicsComponent {
    fn draw_ui(&mut self, ui: &Ui) {
        if let Some(b_t) = self.body_type.draw_ui_combo(ui) {
            self.body_type = b_t;
        }

        match self.body_type {
            BodyType::Static => {}
            BodyType::Dynamic { ref mut apply_gravity, ref mut jump_data } => {
                ui.checkbox(im_str!("Appliquer la gravité"), apply_gravity);
            }
        }
    }
}