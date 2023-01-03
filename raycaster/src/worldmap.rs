use crate::prelude::*;

pub struct WorldMap {
    walls               : FxHashMap<(i32, i32), Tile>,

    images              : Vec<(Vec<u8>, u32, u32)>,

    ceiling_tile        : Option<Tile>,
    floor_tile          : Option<Tile>,
}

/// The world map
impl WorldMap {
    pub fn new() -> Self {

        Self {
            walls       : FxHashMap::default(),
            images      : vec![],

            ceiling_tile: None,
            floor_tile  : None,
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

}