use ggez::graphics::Color;

use serde;

#[derive(Serialize, Deserialize)]
#[serde(remote="Color")]
pub struct ColorDef {
    /// Red component
    pub r: f32,
    /// Green component
    pub g: f32,
    /// Blue component
    pub b: f32,
    /// Alpha component
    pub a: f32,
}