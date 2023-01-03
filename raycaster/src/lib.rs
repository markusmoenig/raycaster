pub mod raycaster;
pub mod worldmap;
pub mod math;
pub mod tile;

pub use crate::worldmap::WorldMap as WorldMap;
pub use crate::tile::Tile as Tile;

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum WidgetKey {
    Escape,
    Return,
    Delete,
    Up,
    Right,
    Down,
    Left,
    Space,
    Tab
}


pub mod prelude {
    pub use crate::WidgetKey;
    pub use rustc_hash::FxHashMap;
    pub use crate::raycaster::Raycaster;
    pub use crate::worldmap::WorldMap;
    pub use crate::math::vec2;
    pub use crate::tile::Tile;
}