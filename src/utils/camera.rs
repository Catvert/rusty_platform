use ggez::{self, GameResult};

use na::{Vector2, Point2};

use na::{self, clamp};
use utils::constants;
use utils::math::Rect;
use ggez::graphics;

pub struct Camera {
    screen_size: Vector2<f64>,
    view_size: Vector2<f64>,
    view_center: Point2<f64>,
    zoom: f64
}

impl Camera {
    pub fn new(screen_size: Vector2<u32>, initial_zoom: f64) -> Self {
        Camera {
            screen_size: Vector2::new(screen_size.x as f64, screen_size.y as f64),
            view_size: Vector2::new(constants::CAMERA_VIEW_SIZE.0 as f64, constants::CAMERA_VIEW_SIZE.1 as f64),
            view_center: Point2::new(constants::CAMERA_VIEW_SIZE.0 as f64 / 2., constants::CAMERA_VIEW_SIZE.1 as f64 / 2.),
            zoom: initial_zoom
        }
    }

    pub fn set_screen_size(&mut self, size: &Vector2<u32>) {
        self.screen_size = Vector2::new(size.x as f64, size.y as f64);
    }

    pub fn move_by(&mut self, by: &Vector2<f64>, bounds: Option<&Rect>) {
        self.view_center.x = if let Some(rect) = bounds {
            clamp(self.view_center.x + by.x, rect.left() + self.view_size.x as f64 / 2., rect.right() - self.view_size.x / 2.)
        } else {
            self.view_center.x + by.x
        };

        self.view_center.y = if let Some(rect) = bounds {
            clamp(self.view_center.y + by.y, rect.top() + self.view_size.y as f64 / 2., rect.bottom() - self.view_size.y / 2.)
        } else {
            self.view_center.y + by.y
        };
    }

    pub fn move_to(&mut self, to: &Point2<f64>, bounds: Option<&Rect>) {
        self.view_center = if let Some(rect) = bounds {
            Point2::new(clamp(to.x, rect.left() + self.view_size.x / 2., rect.right() - self.view_size.x / 2.), clamp(to.y, rect.top() + self.view_size.y / 2., rect.bottom() - self.view_size.y / 2.))
        } else {
            *to
        };
    }

    pub fn zoom_by(&mut self, by: f64, _bounds: Option<&Rect>) {
        self.zoom = clamp(self.zoom + by, 0.2, 100.);
    }

    pub fn world_rect_to_screen(&self, rect: &Rect) -> Rect {
        let pos_in_screen = self.world_to_screen_coords(rect.pos);
        let size =  self.calculate_dest_scale(Point2::new(rect.size.x as f64, rect.size.y as f64));
        Rect::from(Point2::new(pos_in_screen.x as f64, pos_in_screen.y as f64 - size.y), Vector2::new(size.x as u32, size.y as u32))
    }

    pub fn world_to_screen_coords(&self, from: Point2<f64>) -> na::Point2<i32> {
        let pixels_per_unit = self.screen_size.component_div(&self.view_size) * self.zoom;
        let view_offset = from - self.view_center;
        let view_scale = view_offset.component_mul(&pixels_per_unit);

        let x = (*view_scale).x + (*self.screen_size).x / 2.0;
        let y = (*self.screen_size).y - ((*view_scale).y + (*self.screen_size).y / 2.0);
        na::Point2::new(x as i32, y as i32)
    }

    pub fn screen_to_world_coords(&self, from: na::Point2<i32>) -> Point2<f64> {
        let (sx, sy) = (from.x, from.y);
        let sx = sx as f64;
        let sy = sy as f64;
        let flipped_x = sx - ((*self.screen_size).x / 2.0);
        let flipped_y = -sy + (*self.screen_size).y / 2.0;
        let screen_coords = Vector2::new(flipped_x, flipped_y);
        let units_per_pixel = self.view_size.component_div(&self.screen_size);
        let view_scale = screen_coords.component_mul(&units_per_pixel);
        let view_offset = self.view_center + view_scale;

        view_offset
    }

    pub fn view_size(&self) -> Vector2<f64> { Vector2::new(self.view_size.x, self.view_size.y) }

    pub fn location_center(&self) -> &Point2<f64> {
        &self.view_center
    }
    pub fn location_zero(&self) -> Point2<f64> {
        let center = self.location_center();
        Point2::new(center.x - self.view_size.x / 2., center.y - self.view_size.y / 2.)
    }

    fn calculate_dest_point(&self, location: Point2<f64>) -> Point2<f64> {
        let point = self.world_to_screen_coords(location);
        Point2::new(point.x as f64, point.y as f64)
    }

    pub fn calculate_dest_scale(&self, scale: Point2<f64>) -> Point2<f64> {
        Point2::new(self.zoom * scale.x * (self.screen_size.x / self.view_size.x) as f64, self.zoom * scale.y * (self.screen_size.y / self.view_size.y) as f64)
    }
}