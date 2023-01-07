pub mod raycaster;
pub mod worldmap;
pub mod math;
pub mod tile;
pub mod sprite;
pub mod light;

pub use crate::worldmap::WorldMap as WorldMap;
pub use crate::tile::Tile as Tile;
pub use crate::sprite::Sprite as Sprite;
pub use crate::light::Light as Light;

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
    pub use crate::sprite::Sprite;
    pub use crate::light::Light;
}