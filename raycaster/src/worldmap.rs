use crate::prelude::*;

pub struct WorldMap {
    map             : FxHashMap<(i32, i32), bool>
}

/// The world map
impl WorldMap {
    pub fn new() -> Self {

        Self {
            map     : FxHashMap::default()
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
}