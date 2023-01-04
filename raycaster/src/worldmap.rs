use crate::prelude::*;

pub struct WorldMap {
    walls                   : FxHashMap<(i32, i32), Tile>,

    images                  : Vec<(Vec<u8>, u32, u32)>,

    pub sprites             : Vec<Sprite>,

    ceiling_tile            : Option<Tile>,
    floor_tile              : Option<Tile>,

    pub fog_color           : [u8;4],
    pub fog_distance        : f32,
}

/// The world map
impl WorldMap {
    pub fn new() -> Self {

        Self {
            walls           : FxHashMap::default(),
            images          : vec![],

            sprites         : vec![],

            ceiling_tile    : None,
            floor_tile      : None,

            fog_color       : [0, 0, 0, 255],
            fog_distance    : 6.0,
        }
    }

    /// Sets a wall at the given position
    pub fn set_wall(&mut self, x: i32, y: i32, tile: Tile) {
        self.walls.insert((x, y), tile);
    }

    /// Checks if there is a wall at the given position
    pub fn has_wall(&self, x: i32, y: i32) -> bool {
        self.walls.get(&(x, y)).is_some()
    }

    /// Gets the wall at the given position
    pub fn get_wall(&self, x: i32, y: i32) -> Option<&Tile> {
        self.walls.get(&(x, y))
    }

    /// Sets the ceiling tile
    pub fn set_ceiling_tile(&mut self, tile: Tile) {
        self.ceiling_tile = Some(tile);
    }

    /// Gets the ceiling tile
    pub fn get_ceiling_tile(&self) -> Option<&Tile> {
        self.ceiling_tile.as_ref()
    }

    /// Sets the floor tile
    pub fn set_floor_tile(&mut self, tile: Tile) {
        self.floor_tile = Some(tile);
    }

    /// Gets the floor tile
    pub fn get_floor_tile(&self) -> Option<&Tile> {
        self.floor_tile.as_ref()
    }

    /// Adds an image to the list of images
    pub fn add_image(&mut self, data: Vec<u8>, width: u32, height: u32) -> usize {
        let index = self.images.len();
        self.images.push((data, width, height));
        index
    }

    /// Gets the image at the given index
    pub fn get_image(&self, index: usize) -> Option<&(Vec<u8>, u32, u32)> {
        self.images.get(index)
    }

    /// Adds a sprite to the list of sprites
    pub fn add_sprite(&mut self, sprite: Sprite) {
        self.sprites.push(sprite);
    }

    /// Set the fog color and distance
    pub fn set_fog(&mut self, color: [u8; 4], distance: f32) {
        self.fog_color = color;
        self.fog_distance = distance;
    }

}