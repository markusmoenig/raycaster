use crate::prelude::*;

pub struct WorldMap {
    map                 : FxHashMap<(i32, i32), bool>,

    images              : Vec<(Vec<u8>, u32, u32)>,
}

/// The world map
impl WorldMap {
    pub fn new() -> Self {

        Self {
            map         : FxHashMap::default(),
            images      : vec![],
        }
    }

    /// Sets a wall at the given position
    pub fn set_wall(&mut self, x: i32, y: i32) {
        self.map.insert((x, y), true);
    }

    /// Checks if there is a wall at the given position
    pub fn has_wall(&self, x: i32, y: i32) -> bool {
        self.map.get(&(x, y)).is_some()
    }

    /// Adds an image to the list of images
    pub fn add_image(&mut self, data: Vec<u8>, width: u32, height: u32) -> usize {
        let index = self.images.len();
        self.images.push((data, width, height));
        index
    }
}