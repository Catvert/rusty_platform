use nalgebra::{
    self,
    Point2,
    Vector2,
};
use ggez::graphics;
use serde::{
    Serialize,
    Deserialize,
};

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Rect {
    pub pos: Point2<f64>,
    pub size: Vector2<u32>,
}

impl Rect {
    pub fn left(&self) -> f64 {
        self.pos.x
    }

    pub fn right(&self) -> f64 {
        self.pos.x + self.size.x as f64
    }

    pub fn top(&self) -> f64 {
        self.pos.y
    }

    pub fn bottom(&self) -> f64 {
        self.pos.y + self.size.y as f64
    }

    pub fn move_by(&mut self, by: Vector2<f64>) {
        self.pos += by;
    }
    pub fn move_to(&mut self, to: Point2<f64>) {
        self.pos = to;
    }
    pub fn resize_by(&mut self, by: Vector2<u32>) { self.size += by; }
    pub fn resize_to(&mut self, to: Vector2<u32>) {
        self.size = to;
    }

    pub fn contains_pt(&self, point: Point2<f64>) -> bool {
        point.x >= self.left() && point.x <= self.right() && point.y <= self.bottom()
            && point.y >= self.top()
    }

    pub fn contains_rect(&self, rect: Rect) -> bool {
        self.left() <= rect.left() && self.right() >= rect.right() && self.top() <= rect.top() && self.bottom() >= rect.bottom()
    }

    pub fn overlaps(&self, other: Rect) -> bool {
        self.left() <= other.right() && self.right() >= other.left() && self.top() <= other.bottom()
            && self.bottom() >= other.top()
    }

    pub fn to_ggez_rect(&self) -> graphics::Rect {
        graphics::Rect::new(self.pos.x as f32, self.pos.y as f32, self.size.x as f32, self.size.y as f32)
    }
}

impl Rect {
    pub fn new(x: f64, y: f64, w: u32, h: u32) -> Self {
        Rect { pos: Point2::new(x, y), size: Vector2::new(w, h) }
    }

    pub fn from(pos: Point2<f64>, size: Vector2<u32>) -> Self {
        Rect { pos, size }
    }

    pub fn from_points(p1: Point2<i32>, p2: Point2<i32>) -> Self {
        let x = nalgebra::min(p1.x, p2.x);
        let y = nalgebra::min(p1.y, p2.y);
        let w = nalgebra::abs(&(p1.x - p2.x)) as u32;
        let h = nalgebra::abs(&(p1.y - p2.y)) as u32;

        Rect::new(x as f64, y as f64, w, h)
    }
}

impl From<(f64, f64, u32, u32)> for Rect {
    fn from((x, y, w, h): (f64, f64, u32, u32)) -> Self {
        Rect::new(x, y, w, h)
    }
}

impl From<(Point2<f64>, Vector2<u32>)> for Rect {
    fn from((pos, size): (Point2<f64>, Vector2<u32>)) -> Self {
        Rect::new(pos.x, pos.y, size.x, size.y)
    }
}



