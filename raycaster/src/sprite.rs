use crate::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct Sprite {

    pub x               : f32,
    pub y               : f32,

    pub tile            : Tile,

    /// Shrinks the sprite
    pub shrink          : i32,

    /// Moves the sprite up and down
    pub move_y          : f32,

    /// Distance from the player, used for sorting the sprites
    /// Only used internally
    pub distance        : f32,
}

/// A tile
impl Sprite {
    /// Creates a new sprite
    pub fn new(x: f32, y: f32, tile: Tile) -> Self {
        Self {
            x, y,
            tile,
            shrink      : 1,
            move_y      : 0.0,
            distance    : 0.0,
        }
    }
}