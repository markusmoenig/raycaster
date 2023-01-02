pub mod raycaster;
pub mod worldmap;
pub mod math;

pub use crate::worldmap::WorldMap as WorldMap;

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
}