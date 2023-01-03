use crate::prelude::*;

pub struct WorldMap {
    walls               : FxHashMap<(i32, i32), Tile>,

    images              : Vec<(Vec<u8>, u32, u32)>,
}

/// The world map
impl WorldMap {
    pub fn new() -> Self {

        Self {
            walls       : FxHashMap::default(),
            images      : vec![],
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