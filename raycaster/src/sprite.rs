use crate::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct Sprite {

    pub x               : f32,
    pub y               : f32,

    pub tile            : Tile,
}

/// A tile
impl Sprite {
    /// Creates a new sprite
    pub fn new(x: f32, y: f32, tile: Tile) -> Self {
        Self {
            x, y,
            tile
        }
    }
}