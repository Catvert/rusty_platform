use na::{self, Point2, Vector2};
use ggez::graphics;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Rect {
    pub pos: Point2<f32>,
    pub size: Vector2<u32>
}

impl Rect {
    pub fn left(&self) -> f32 {
        self.pos.x
    }

    pub fn right(&self) -> f32 {
        self.pos.x + self.size.x as f32
    }

    pub fn top(&self) -> f32 {
        self.pos.y
    }

    pub fn bottom(&self) -> f32 {
        self.pos.y + self.size.y as f32
    }

    pub fn move_by(&mut self, by: &Vector2<f32>) {
        self.pos += by;
    }
    pub fn move_to(&mut self, to: &Point2<f32>) {
        self.pos = *to;
    }
    pub fn resize_by(&mut self, by: &Vector2<u32>) { self.size += by; }
    pub fn resize_to(&mut self, to: &Vector2<u32>) {
        self.size = *to;
    }

    pub fn contains(&self, point: Point2<f32>) -> bool {
        point.x >= self.left() && point.x <= self.right() && point.y <= self.bottom()
            && point.y >= self.top()
    }

    pub fn overlaps(&self, other: &Rect) -> bool {
        self.left() <= other.right() && self.right() >= other.left() && self.top() <= other.bottom()
            && self.bottom() >= other.top()
    }

    pub fn to_ggez_rect(&self) -> graphics::Rect {
        graphics::Rect::new(self.pos.x, self.pos.y, self.size.x as f32, self.size.y as f32)
    }
}

impl Rect {
    pub fn new(x: f32, y: f32, w: u32, h: u32) -> Self {
        Rect { pos: Point2::new(x, y), size: Vector2::new(w, h) }
    }

    pub fn from(pos: Point2<f32>, size: Vector2<u32>) -> Self {
        Rect { pos, size }
    }

    pub fn from_points(p1: &Point2<i32>, p2: &Point2<i32>) -> Self {
        let x = na::min(p1.x, p2.x);
        let y = na::min(p1.y, p2.y);
        let w = na::abs(&(p1.x - p2.x)) as u32;
        let h = na::abs(&(p1.y - p2.y)) as u32;

        Rect::new(x as f32, y as f32, w, h)
    }
}

impl From<(f32, f32, u32, u32)> for Rect {
    fn from((x, y, w, h): (f32, f32, u32, u32)) -> Self {
        Rect::new(x, y, w, h)
    }
}

impl From<(Point2<f32>, Vector2<u32>)> for Rect {
    fn from((pos, size): (Point2<f32>, Vector2<u32>)) -> Self {
        Rect::new(pos.x, pos.y, size.x, size.y)
    }
}



