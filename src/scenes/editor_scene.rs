use specs::{Entity, World, Join, Builder, saveload::MarkedBuilder};

use ggez::Context;
use ggez::graphics::{self, Color};
use ggez::event::{Keycode, MouseButton};

use na::{Point2, Vector2};


use utils::resources_manager::RefRM;
use utils::input_manager::{InputManager, RefInputManager};
use utils::math::Rect;

use ecs::rect::RectComponent;
use ecs::chunk::{ActiveChunkMarker};
use ecs::inputs::{InputSystem, InputComponent};
use ecs::actions::ActionSystem;
use ecs::level::Level;

use scenes::{Scene, SceneState, NextState};
use ecs::render::SpriteComponent;
use ecs::actions::Actions;
use ecs::physics::PhysicsComponent;
use ecs::physics::BodyType;
use specs::saveload::U64Marker;
use utils::sprite::SpriteMode;
use utils::sprite::Sprite;

use std::collections::HashMap;

use specs::LazyUpdate;

use ecs::imgui_editor::ImGuiEditor;
use imgui_sys;
use imgui::Ui;
use imgui::ImStr;
use imgui::ImVec4;
use imgui::EditableColor;

use nfd;
use nfd::Response;
use std::path::Path;

use utils::constants;
use ecs::level::LevelConfig;
use scenes::main_scene::MainScene;
use utils::camera::Camera;

lazy_static! {
    static ref COMPONENTS_WRAPPERS: HashMap<ComponentsWrapper, &'static ImStr> = {
        let mut wrappers = HashMap::new();
        wrappers.insert(ComponentsWrapper::Rect, im_str!("Rectangle"));
        wrappers.insert(ComponentsWrapper::Input, im_str!("Entrée"));
        wrappers.insert(ComponentsWrapper::Physics, im_str!("Physique"));
        wrappers
    };
}

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
enum ComponentsWrapper {
    Rect,
    Input,
    Physics
}

macro_rules! impl_components_wrapper {
    (
         [$ ($wrapper:path => $comp:ident), *]
    ) => {
        impl ComponentsWrapper {
            fn insert(&self, ent: Entity, world: &World) {
                let lazy_update = world.read_resource::<LazyUpdate>();
                match *self {
                    $($wrapper => { lazy_update.insert(ent, $comp::default()) }),*
                }
            }

            fn delete(&self, ent: Entity, world: &World) {
                let lazy_update = world.read_resource::<LazyUpdate>();
                match *self {
                    $($wrapper => { lazy_update.remove::<$comp>(ent) }),*
                }
            }

            fn has_comp(&self, ent: Entity, world: &World) -> bool {
                match *self {
                    $($wrapper => { world.read_storage::<$comp>().contains(ent) }),*
                }
            }

            fn draw_ui(&self, ent: Entity, world: &mut World,  ui: &Ui) {
                match *self {
                    $($wrapper => {
                        if let Some(c) = world.write_storage::<$comp>().get_mut(ent) {
                            c.draw_ui(ui);
                        }
                    }),*
                }
            }
        }
    }
}

impl_components_wrapper!([ComponentsWrapper::Rect => RectComponent, ComponentsWrapper::Input => InputComponent, ComponentsWrapper::Physics => PhysicsComponent]);

struct ImGuiMemoryHelper {
    pub select_entity_view_show_window: bool,
    pub select_entity_view_component_selected: ComponentsWrapper,
    pub select_entity_view_add_component_popup_selected: ComponentsWrapper,
    pub select_entity_view_add_component_popup_show: bool,
}

impl ImGuiMemoryHelper {
    fn new() -> Self {
        ImGuiMemoryHelper {
            select_entity_view_show_window: false,
            select_entity_view_component_selected: ComponentsWrapper::Rect,
            select_entity_view_add_component_popup_selected: ComponentsWrapper::Rect,
            select_entity_view_add_component_popup_show: false,
        }
    }
}

#[derive(Debug, Clone)]
enum EditorMode {
    Default,
    SelectionRectangle(Point2<f64>, Point2<f64>),
    Select(Entity, Option<Vec<Entity>>),
    Copy(Entity, Option<Vec<Entity>>),
}

pub struct EditorScene<'a, 'b> {
    level: Level<'a, 'b>,
    input_manager: RefInputManager,
    resources_manager: RefRM,
    camera: Camera,
    mode: EditorMode,
    imgui_helper: ImGuiMemoryHelper,
    is_ui_hover: bool,
}

impl<'a, 'b> EditorScene<'a, 'b> {
    pub fn load_level(screen_size: Vector2<u32>, resources_manager: RefRM, input_manager: RefInputManager, config: LevelConfig) -> Self {
        let level = Level::load(config, resources_manager.clone(), |builder| {
            builder
                .with(InputSystem { input_manager: input_manager.clone() }, "input_manager", &[])
                .with(ActionSystem, "action_system", &["input_manager"])
        });

        let camera = Camera::new(screen_size, Vector2::new(constants::CAMERA_VIEW_SIZE.0, constants::CAMERA_VIEW_SIZE.1), 1.);

        EditorScene { level, input_manager, resources_manager, camera, mode: EditorMode::Default, imgui_helper: ImGuiMemoryHelper::new(), is_ui_hover: false }
    }

    pub fn new_level(screen_size: Vector2<u32>, resources_manager: RefRM, input_manager: RefInputManager, name: String) -> Self {
        let level = Level::new(String::from("finch"), name, resources_manager.clone(), |builder| {
            builder
                .with(InputSystem { input_manager: input_manager.clone() }, "input_manager", &[])
                .with(ActionSystem, "action_system", &["input_manager"])
        }, |world| {
            for x in 0..1 {
                for y in 0..1 {
                    Self::create_entity(world, Point2::new(0. + x as f64 * 100., 0. + y as f64 * 100.), Vector2::new(100, 100), SpriteMode::Stretch, false);
                }
            }

            //Self::create_entity(world, Point2::new(0., 200.), Vector2::new(100, 100), SpriteMode::Stretch, true);
        });

        let camera = Camera::new(screen_size, Vector2::new(constants::CAMERA_VIEW_SIZE.0, constants::CAMERA_VIEW_SIZE.1), 1.);

        EditorScene { level, input_manager, resources_manager, camera, mode: EditorMode::Default, imgui_helper: ImGuiMemoryHelper::new(), is_ui_hover: false }
    }

    fn create_entity(world: &mut World, pos: Point2<f64>, size: Vector2<u32>, mode: SpriteMode, add_input: bool) {
        let mut builder = world.create_entity()
            .with(SpriteComponent::new(Sprite::new(Path::new("/finch_square.jpg").to_owned(), mode)))
            .with(RectComponent::new((pos, size).into()));

        if add_input {
            builder = builder.with(InputComponent::new(vec![
                (Keycode::Q, false, Actions::PhysicsMove([-10., 0.].into())),
                (Keycode::Z, false, Actions::PhysicsMove([0., 10.].into())),
                (Keycode::S, false, Actions::PhysicsMove([0., -10.].into())),
                (Keycode::D, false, Actions::PhysicsMove([10., 0.].into()))
            ]));
        }

        builder.with(PhysicsComponent::new(BodyType::Dynamic, true))
            .marked::<U64Marker>()
            .build();
    }

    pub fn clone_entity(world: &mut World, ent: Entity, new_pos: Point2<f64>) -> Entity {
        use ecs::serialization;

        let copy_ent = serialization::copy_entity(ent, world);

        if let Some(rect) = world.write_storage::<RectComponent>().get_mut(copy_ent) {
            rect.move_to(&new_pos);
        }

        copy_ent
    }

    fn get_entity_under_mouse(&self, input_manager: &InputManager) -> Option<Entity> {
        let entities = self.level.get_world().entities();
        let rect_storage = self.level.get_world().read_storage::<RectComponent>();
        let active_marker = self.level.get_world().read_storage::<ActiveChunkMarker>();
        let mouse_pos = input_manager.get_mouse_pos();

        let mouse_pos_in_world = self.camera.screen_point_to_world(&Point2::new(mouse_pos.x as f64, mouse_pos.y as f64));

        for (ent, rect, _) in (&*entities, &rect_storage, &active_marker).join() {
            if rect.get_rect().contains_pt(&mouse_pos_in_world) {
                return Some(ent);
            }
        }

        None
    }

    fn get_entities_in_rect(&self, in_rect: &Rect) -> Vec<Entity> {
        let entities = self.level.get_world().entities();
        let rect_storage = self.level.get_world().read_storage::<RectComponent>();
        let active_marker = self.level.get_world().read_storage::<ActiveChunkMarker>();

        let mut overlaps_entities = vec![];

        for (ent, rect, _) in (&*entities, &rect_storage, &active_marker).join() {
            if rect.get_rect().overlaps(in_rect) {
                overlaps_entities.push(ent);
            }
        }

        overlaps_entities
    }

    fn update_camera(&mut self) {
        if !self.is_ui_hover {
            let input_manager = self.input_manager.lock().unwrap();

            let chunks_bounds = Some(self.level.get_chunk_sys().get_bounds_chunks());

            if let Some(_jp) = input_manager.is_key_pressed(&Keycode::Left) {
                self.camera.move_by(&Vector2::new(-10., 0.), chunks_bounds);
            }

            if let Some(_jp) = input_manager.is_key_pressed(&Keycode::Right) {
                self.camera.move_by(&Vector2::new(10., 0.), chunks_bounds);
            }

            if let Some(_jp) = input_manager.is_key_pressed(&Keycode::Up) {
                self.camera.move_by(&Vector2::new(0., 10.), chunks_bounds);
            }

            if let Some(_jp) = input_manager.is_key_pressed(&Keycode::Down) {
                self.camera.move_by(&Vector2::new(0., -10.), chunks_bounds);
            }

            if let Some(_jp) = input_manager.is_key_pressed(&Keycode::P) {
                self.camera.zoom_by(0.01, chunks_bounds);
            }

            if let Some(_jp) = input_manager.is_key_pressed(&Keycode::M) {
                self.camera.zoom_by(-0.01, chunks_bounds);
            }
        }
    }
}

impl<'a, 'b> Scene for EditorScene<'a, 'b> {
    fn update(&mut self, ctx: &mut Context, dt: f32) -> SceneState {
        {
            let input_manager = self.input_manager.lock().unwrap();

            let mouse_pos = input_manager.get_mouse_pos();
            let mouse_in_world = self.camera.screen_point_to_world(&Point2::new(mouse_pos.x as f64, mouse_pos.y as f64));
            let delta_mouse = input_manager.get_delta_mouse();
            let delta_mouse_in_world = self.camera.screen_size_to_world(&Vector2::new(delta_mouse.x as f64, delta_mouse.y as f64));

            if let Some(jp) = input_manager.is_mouse_pressed(&MouseButton::Left) {
                let next_mode = match self.mode.clone() {
                    EditorMode::Default => {
                        if !self.is_ui_hover {
                            if *jp {
                                if let Some(ent) = self.get_entity_under_mouse(&input_manager) {
                                    Some(EditorMode::Select(ent, None))
                                } else {
                                    None
                                }
                            } else {
                                let start_point = Point2::new(mouse_pos.x as f64, mouse_pos.y as f64);
                                Some(EditorMode::SelectionRectangle(start_point, start_point))
                            }
                        } else {
                            None
                        }
                    }
                    EditorMode::SelectionRectangle(p1, _p2) => {
                        Some(EditorMode::SelectionRectangle(p1, Point2::new(mouse_pos.x as f64, mouse_pos.y as f64)))
                    }
                    EditorMode::Select(entity, other_entities) => {
                        let mut next_mode = None;

                        if !self.is_ui_hover {
                            if !*jp {
                                let mut rect_storage = self.level.get_world().write_storage::<RectComponent>();

                                macro_rules! move_ent {
                                    ($ent:expr) => {
                                      if let Some(rect) = rect_storage.get_mut($ent) {
                                        let mut move_x = delta_mouse_in_world.x;
                                        let mut move_y = delta_mouse_in_world.y;

                                        let bounds = self.level.get_chunk_sys().get_bounds_chunks();

                                        if !(bounds.left() <= rect.pos().x + move_x && bounds.right() >= rect.pos().x + move_x + rect.size().x as f64) {
                                            move_x = 0.;
                                        }

                                        if !(bounds.top() <= rect.pos().y + move_y && bounds.bottom() >= rect.pos().y + move_y + rect.size().y as f64) {
                                            move_y = 0.;
                                        }

                                        rect.move_by(&Vector2::new(move_x, move_y));
                                    }
                                    };
                                }

                                move_ent!(entity);

                                if let Some(other_entities) = other_entities {
                                    for ent in other_entities.iter() {
                                        move_ent!(*ent);
                                    }
                                }
                            } else {
                                if let Some(ent) = self.get_entity_under_mouse(&input_manager) {
                                    let mut other_entities = other_entities;

                                    if let Some(ref mut other_entities) = other_entities {
                                        if other_entities.contains(&ent) { // ent under mouse already in group
                                            other_entities.retain(|e| *e != ent); // delete ent under mouse from group -> select entity
                                            if entity != ent {
                                                other_entities.push(entity); // swap select entity to group
                                            }
                                        } else {
                                            if entity != ent {
                                                other_entities.clear(); // an other entity outside the group is selected
                                            }
                                        }
                                    }

                                    next_mode = Some(EditorMode::Select(ent, other_entities))
                                } else {
                                    next_mode = Some(EditorMode::Default);
                                }
                            }
                        }

                        next_mode
                    }
                    EditorMode::Copy(entity, other_entities) => {
                        if *jp {
                            let (select_rect, other_ent_pos) = {
                                let rect_storage = self.level.get_world().read_storage::<RectComponent>();
                                let select_rect = rect_storage.get(entity).map_or(Rect::new(0., 0., 0, 0), |rect| rect.get_rect().clone());
                                let mut other_ent_pos = vec![];

                                if let Some(other_entities) = other_entities {
                                    for other_entity in other_entities.iter() {
                                        other_ent_pos.push((other_entity.clone(), rect_storage.get(*other_entity).map_or(Point2::new(0., 0.), |rect| rect.get_rect().pos.clone())));
                                    }
                                }

                                (select_rect, other_ent_pos)
                            };

                            let select_rect_placed = Rect::from(Point2::new(mouse_in_world.x - select_rect.size.x as f64 / 2., mouse_in_world.y - select_rect.size.y as f64 / 2.), select_rect.size);
                            Self::clone_entity(self.level.get_world_mut(), entity, Point2::new(select_rect_placed.pos.x, select_rect_placed.pos.y));

                            for (other_ent, pos) in other_ent_pos.iter() {
                                let delta_x = select_rect.pos.x - pos.x;
                                let delta_y = select_rect.pos.y - pos.y;

                                Self::clone_entity(self.level.get_world_mut(), *other_ent, Point2::new(select_rect_placed.pos.x - delta_x, select_rect_placed.pos.y - delta_y));
                            }
                        }

                        None
                    }
                };

                if let Some(mode) = next_mode {
                    self.mode = mode;
                }
            } else { // Not pressed left cursor
                // Check end selection
                if let EditorMode::SelectionRectangle(p1, p2) = self.mode {
                    let p1 = Point2::new(p1.x as i32, p1.y as i32);
                    let p2 = Point2::new(p2.x as i32, p2.y as i32);

                    let entities = self.get_entities_in_rect(&self.camera.screen_rect_to_world(&Rect::from_points(&p1, &p2)));

                    if entities.is_empty() {
                        self.mode = EditorMode::Default;
                    } else {
                        let select_ent = *entities.first().unwrap();
                        let other_entities = Vec::from(&entities[1..]);
                        let other_entities = if other_entities.is_empty() { None } else { Some(other_entities) };
                        self.mode = EditorMode::Select(select_ent, other_entities);

                    }
                }
            }

            if let Some(true) = input_manager.is_mouse_pressed(&MouseButton::Right) {
                match self.mode {
                    EditorMode::Default => {}
                    EditorMode::Select(select_ent, _) => {
                        if let Some(ent) = self.get_entity_under_mouse(&input_manager) {
                            if select_ent == ent {
                                self.imgui_helper.select_entity_view_show_window = true;
                            }
                        }
                    }
                    EditorMode::Copy(_, _) => {
                        self.mode = EditorMode::Default;
                    }
                    EditorMode::SelectionRectangle(_, _) => {}
                }
            }

            if !self.is_ui_hover {
                if let Some(true) = input_manager.is_key_pressed(&Keycode::Backspace) {
                    if let EditorMode::Select(entity, other_entities) = self.mode.clone() {
                        macro_rules! delete_ent {
                            ($ent:expr) => {
                                self.level.get_world_mut().delete_entity($ent).expect(&format!("Impossible de supprimer l'entité {:?}", $ent));
                            }
                        }

                        delete_ent!(entity);

                        if let Some(other_entities) = other_entities {
                            for other_ent in other_entities.iter() {
                                delete_ent!(*other_ent);
                            }
                        }
                    }

                    self.mode = EditorMode::Default;
                }

                if let Some(true) = input_manager.is_key_pressed(&Keycode::C) {
                    if let Some(mode) = if let EditorMode::Select(ref entity, ref other_entities) = self.mode {
                        Some(EditorMode::Copy(entity.clone(), other_entities.clone()))
                    } else { None } {
                        self.mode = mode;
                    }
                }
            }
        }

        self.update_camera();

        self.level.update(ctx, &self.camera, dt);
        Ok(NextState::Continue)
    }

    fn draw(&mut self, ctx: &mut Context) -> SceneState {
        let mouse_pos = self.input_manager.lock().unwrap().get_mouse_pos();
        let mouse_in_world = self.camera.screen_point_to_world(&Point2::new(mouse_pos.x as f64, mouse_pos.y as f64));

        self.level.draw(ctx, &self.camera);

        match self.mode.clone() {
            EditorMode::Default => {}
            EditorMode::SelectionRectangle(p1, p2) => {
                let p1 = Point2::new(p1.x as i32, p1.y as i32);
                let p2 = Point2::new(p2.x as i32, p2.y as i32);

                graphics::set_color(ctx, Color::from_rgba(100, 0, 0, 100))?;

                graphics::rectangle(ctx, graphics::DrawMode::Fill, Rect::from_points(&p1, &p2).to_ggez_rect())?;

                graphics::set_color(ctx, Color::from_rgba(255, 255, 255, 255))?;
            }
            EditorMode::Select(entity, other_entities) => {
                macro_rules! draw_ent_rect {
                    ($ent:expr) => {
                        if let Some(rect) = self.level.get_world().read_storage::<RectComponent>().get($ent) {
                            graphics::rectangle(ctx, graphics::DrawMode::Line(2.0), self.camera.world_rect_to_screen(rect.get_rect()).to_ggez_rect())?;
                        }
                    };
                }

                let old_color = graphics::get_color(ctx);
                graphics::set_color(ctx, (255, 180, 40).into())?;

                draw_ent_rect!(entity);

                graphics::set_color(ctx, old_color)?;

                if let Some(other_entities) = other_entities {
                    for other_ent in other_entities.iter() {
                        draw_ent_rect!(*other_ent);
                    }
                }
            }
            EditorMode::Copy(entity, other_entities) => {
                let default_color = graphics::get_color(ctx);
                graphics::set_color(ctx, [default_color.r, default_color.g, default_color.b, 0.5].into())?;

                let rect_storage = self.level.get_world().read_storage::<RectComponent>();
                let mut sprite_storage = self.level.get_world().write_storage::<SpriteComponent>();
                if let Some(select_rect) = rect_storage.get(entity) {
                    let size = select_rect.get_rect().size;
                    let select_rect_placed = Rect::from(Point2::new(mouse_in_world.x - size.x as f64 / 2., mouse_in_world.y - size.y as f64 / 2.), size);

                    if let Some(spr) = sprite_storage.get_mut(entity) {
                        spr.draw(ctx, &select_rect_placed, &self.camera, &self.resources_manager);
                    }

                    if let Some(other_entities) = other_entities {
                        for other_ent in other_entities.iter() {
                            if let Some(other_rect) = rect_storage.get(*other_ent) {
                                if let Some(spr) = sprite_storage.get_mut(*other_ent) {
                                    let delta_x = select_rect.get_rect().pos.x - other_rect.get_rect().pos.x;
                                    let delta_y = select_rect.get_rect().pos.y - other_rect.get_rect().pos.y;

                                    let rect = Rect::from(Point2::new(select_rect_placed.pos.x - delta_x, select_rect_placed.pos.y - delta_y), other_rect.get_rect().size);

                                    spr.draw(ctx, &rect, &self.camera, &self.resources_manager);
                                }
                            }
                        }
                    }
                }

                graphics::set_color(ctx, default_color)?;
            }
        }


        Ok(NextState::Continue)
    }

    fn draw_ui(&mut self, ctx: &mut Context, _window_size: Vector2<u32>, ui: &Ui) -> SceneState {
        let mut next_state = NextState::Continue;

        ui.main_menu_bar(|| {
            ui.menu(im_str!("Fichier")).build(|| {
                if ui.menu_item(im_str!("Sauvegarder et quitter")).build() {
                    self.level.save();
                    next_state = NextState::Replace(Box::new(MainScene::new(ctx, self.input_manager.clone())));
                }

                if ui.menu_item(im_str!("Ajouter une ressources ..")).build() {
                    let result = nfd::open_file_dialog(Some("jpg"), None).unwrap();
                    match result {
                        Response::Okay(file) => {
                            println!("{:?}", file);
                        },
                        Response::OkayMultiple(_) => {

                        },
                        Response::Cancel => {

                        },
                    }

                }
            });

            ui.menu(im_str!("Niveau")).build(|| {
                if ui.collapsing_header(im_str!("Arrière plan")).build() {
                    let color = self.level.background_color_mut();

                    let mut editable_color = [color.r, color.g, color.b];
                    if ui.color_edit(im_str!("background color"), EditableColor::Float3(&mut editable_color)).build() {
                        color.r = editable_color[0];
                        color.g = editable_color[1];
                        color.b = editable_color[2];
                    }
                }
            });

            ui.text(&format!("{:?}", self.mode));
        });

        if self.imgui_helper.select_entity_view_show_window {
            if let EditorMode::Select(entity, _) = self.mode.clone() {
                let mut opened = self.imgui_helper.select_entity_view_show_window;
                ui.window(im_str!("Entité {}", entity.id())).opened(&mut opened).always_auto_resize(true).build(|| {
                    {
                        let av_comps: Vec<(ComponentsWrapper, &'static ImStr)> = COMPONENTS_WRAPPERS.iter().filter(|c| c.0.has_comp(entity, self.level.get_world())).map(|c| (c.0.clone(), *c.1)).collect();

                        let mut pos = av_comps.iter().position(|c| c.0 == self.imgui_helper.select_entity_view_component_selected).map_or(-1, |pos| pos as i32);

                        let names: Vec<&ImStr> = av_comps.iter().map(|c| c.1).collect();

                        if ui.combo(im_str!("component"), &mut pos, &names[..], 10) {
                            let comp = av_comps.iter().nth(pos as usize).unwrap().0.clone();
                            self.imgui_helper.select_entity_view_component_selected = comp;
                        }

                        ui.same_line(0.);

                        if ui.button(im_str!("Supprimer"), (100., 0.)) {
                            if let Some(comp) = av_comps.iter().nth(pos as usize) {
                                comp.0.delete(entity, self.level.get_world());
                            }
                        }
                    }

                    self.imgui_helper.select_entity_view_component_selected.draw_ui(entity, self.level.get_world_mut(), ui);

                    if ui.button(im_str!("Ajouter un composant"), (-1., 0.)) {
                        ui.open_popup(im_str!("add_comp"));
                    }

                    ui.popup(im_str!("add_comp"), || {
                        let missing_comps: Vec<(ComponentsWrapper, &'static ImStr)> = COMPONENTS_WRAPPERS.iter().filter(|c| !c.0.has_comp(entity, self.level.get_world())).map(|c| (c.0.clone(), *c.1)).collect();

                        let mut pos = missing_comps.iter().position(|c| c.0 == self.imgui_helper.select_entity_view_add_component_popup_selected).map_or(-1, |pos| pos as i32);

                        let names: Vec<&ImStr> = missing_comps.iter().map(|c| c.1).collect();

                        if ui.combo(im_str!("component"), &mut pos, &names[..], 10) {
                            let comp = missing_comps.iter().nth(pos as usize).unwrap().0.clone();
                            self.imgui_helper.select_entity_view_add_component_popup_selected = comp;
                        }

                        if ui.button(im_str!("Ajouter"), (100., 0.)) {
                            if let Some(comp) = missing_comps.iter().nth(pos as usize) {
                                comp.0.insert(entity, self.level.get_world());
                                ui.close_current_popup();
                            }
                        }
                    })
                });

                self.imgui_helper.select_entity_view_show_window = opened;
            }
        }

        self.is_ui_hover = unsafe { imgui_sys::igIsAnyWindowHovered() || imgui_sys::igIsAnyItemHovered() || imgui_sys::igIsAnyItemActive() };

        Ok(next_state)
    }

    fn background_color(&self) -> Color { *self.level.background_color() }

    fn resize_event(&mut self, _ctx: &mut Context, screen_size: Vector2<u32>) {
        self.camera.update_screen_size(&screen_size);
    }
}