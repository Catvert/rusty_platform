use specs::prelude::*;

use na::{Point2, Vector2};

use utils::math::Rect;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RectComponent {
    rect: Rect
}

impl Default for RectComponent {
    fn default() -> Self {
        RectComponent::new(Rect::new(0., 0., 0, 0))
    }
}

impl Component for RectComponent {
    type Storage = FlaggedStorage<Self, VecStorage<Self>>;
}

impl RectComponent {
    pub fn new(rect: Rect) -> Self {
        RectComponent { rect }
    }

    pub fn get_rect(&self) -> &Rect { &self.rect }

    pub fn get_rect_mut(&mut self) -> &mut Rect { &mut self.rect }

    pub fn pos(&self) -> &Point2<f32> {
        &self.rect.pos
    }

    pub fn pos_mut(&mut self) -> &mut Point2<f32> {
        &mut self.rect.pos
    }

    pub fn size(&self) -> &Vector2<u32> {
        &self.rect.size
    }

    pub fn size_mut(&mut self) -> &mut Vector2<u32> {
        &mut self.rect.size
    }

    pub fn move_by(&mut self, by: &Vector2<f32>) {
        self.rect.move_by(by);
    }

    pub fn move_to(&mut self, to: &Point2<f32>) {
        self.rect.move_to(to);
    }
}
