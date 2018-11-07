use crate::utils::math::Rect;
use nalgebra::{
    Point2,
    Vector2,
};
use num::clamp;

pub struct Camera {
    position: Point2<f64>,
    screen_size: Vector2<f64>,
    view_size: Vector2<f64>,
    zoom: f64,
}

impl Camera {
    pub fn new(screen_size: Vector2<u32>, view_size: Vector2<u32>, initial_zoom: f64) -> Self {
        Camera {
            position: Point2::new(0., 0.),
            screen_size: Vector2::new(screen_size.x as f64, screen_size.y as f64),
            view_size: Vector2::new(view_size.x as f64, view_size.y as f64),
            zoom: initial_zoom,
        }
    }

    pub fn position(&self) -> Point2<f64> { self.position }

    pub fn world_view(&self) -> Rect {
        let (scale_x, scale_y) = self.get_scale();
        Rect::from(Point2::new(self.position.x * scale_x, self.position.y * scale_y), Vector2::new((self.view_size.x * self.zoom) as u32, (self.view_size.y * self.zoom) as u32))
    }

    pub fn update_screen_size(&mut self, screen_size: Vector2<u32>) {
        self.screen_size = Vector2::new(screen_size.x as f64, screen_size.y as f64);
    }

    pub fn move_by(&mut self, by: Vector2<f64>, bounds: Option<Rect>) {
        self.position.x = if let Some(rect) = bounds {
            clamp(self.position.x + by.x, rect.left(), rect.right())
        } else {
            self.position.x + by.x
        };

        self.position.y = if let Some(rect) = bounds {
            clamp(self.position.y + by.y, rect.top(), rect.bottom())
        } else {
            self.position.y + by.y
        };
    }

    pub fn move_to(&mut self, to: Point2<f64>, bounds: Option<Rect>) {
        self.position = if let Some(rect) = bounds {
            Point2::new(clamp(to.x + self.position.x, rect.left(), rect.right()), clamp(to.y + self.position.y, rect.top(), rect.bottom()))
        } else {
            Point2::new(to.x + self.position.x, to.y + self.position.y)
        };
    }

    pub fn zoom_by(&mut self, by: f64, _bounds: Option<Rect>) {
        self.zoom = clamp(self.zoom + by, 0.2, 100.);
    }

    fn get_scale(&self) -> (f64, f64) {
        (self.view_size.x / self.screen_size.x * self.zoom, self.view_size.y / self.screen_size.y * self.zoom)
    }

    pub fn screen_point_to_world(&self, point: Point2<f64>) -> Point2<f64> {
        let (scale_x, scale_y) = self.get_scale();

        Point2::new(point.x * scale_x + self.position.x * scale_x, point.y * scale_y + self.position.y * scale_y)
    }

    pub fn screen_size_to_world(&self, size: Vector2<f64>) -> Vector2<f64> {
        let (scale_x, scale_y) = self.get_scale();

        Vector2::new(size.x * scale_x, size.y * scale_y)
    }

    pub fn screen_rect_to_world(&self, rect: Rect) -> Rect {
        let pos = self.screen_point_to_world(rect.pos);
        let size = self.screen_size_to_world(Vector2::new(rect.size.x as f64, rect.size.y as f64));
        Rect::from(pos, Vector2::new(size.x as u32, size.y as u32))
    }

    pub fn world_point_to_screen(&self, point: Point2<f64>) -> Point2<f64> {
        let (scale_x, scale_y) = self.get_scale();

        Point2::new(point.x / scale_x - self.position.x, point.y / scale_y - self.position.y)
    }

    pub fn world_size_to_screen(&self, size: Vector2<f64>) -> Vector2<f64> {
        let (scale_x, scale_y) = self.get_scale();

        Vector2::new(size.x / scale_x, size.y / scale_y)
    }

    pub fn world_rect_to_screen(&self, rect: Rect) -> Rect {
        let pos = self.world_point_to_screen(rect.pos);
        let size = self.world_size_to_screen(Vector2::new(rect.size.x as f64, rect.size.y as f64));
        Rect::from(pos, Vector2::new(size.x as u32, size.y as u32))
    }
}