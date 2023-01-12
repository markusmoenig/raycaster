use crate::prelude::*;
use rand::{thread_rng, Rng};

pub struct WorldMap {
    walls                   : FxHashMap<(i32, i32), Tile>,
    floors                  : FxHashMap<(i32, i32), Tile>,
    ceilings                : FxHashMap<(i32, i32), Tile>,

    images                  : Vec<(Vec<u8>, u32, u32)>,

    pub sprites             : Vec<Sprite>,

    ceiling_tile            : Option<Tile>,
    floor_tile              : Option<Tile>,

    pub fog_color           : [u8;4],
    pub fog_distance        : f32,

    pub lights              : FxHashMap<(i32, i32), Light>,
    pub light_map           : FxHashMap<(i32, i32), f32>
}

/// The world map
impl WorldMap {
    pub fn new() -> Self {

        Self {
            walls           : FxHashMap::default(),
            floors          : FxHashMap::default(),
            ceilings        : FxHashMap::default(),

            images          : vec![],

            sprites         : vec![],

            ceiling_tile    : None,
            floor_tile      : None,

            fog_color       : [0, 0, 0, 255],
            fog_distance    : 6.0,

            lights          : FxHashMap::default(),
            light_map       : FxHashMap::default(),
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
    pub fn set_default_ceiling(&mut self, tile: Tile) {
        self.ceiling_tile = Some(tile);
    }

    /// Gets the ceiling tile
    pub fn get_default_ceiling(&self) -> Option<&Tile> {
        self.ceiling_tile.as_ref()
    }

    /// Sets a ceiling at the given position
    pub fn set_ceiling(&mut self, x: i32, y: i32, tile: Tile) {
        self.ceilings.insert((x, y), tile);
    }

    /// Checks if there is a ceiling at the given position
    pub fn has_ceiling(&self, x: i32, y: i32) -> bool {
        self.ceilings.get(&(x, y)).is_some()
    }

    /// Gets the ceiling at the given position
    pub fn get_ceiling(&self, x: i32, y: i32) -> Option<&Tile> {
        self.ceilings.get(&(x, y))
    }

    /// Sets the floor tile
    pub fn set_default_floor(&mut self, tile: Tile) {
        self.floor_tile = Some(tile);
    }

    /// Gets the floor tile
    pub fn get_default_floor(&self) -> Option<&Tile> {
        self.floor_tile.as_ref()
    }

    /// Sets a floor at the given position
    pub fn set_floor(&mut self, x: i32, y: i32, tile: Tile) {
        self.floors.insert((x, y), tile);
    }

    /// Checks if there is a floor at the given position
    pub fn has_floor(&self, x: i32, y: i32) -> bool {
        self.floors.get(&(x, y)).is_some()
    }

    /// Gets the floor at the given position
    pub fn get_floor(&self, x: i32, y: i32) -> Option<&Tile> {
        self.floors.get(&(x, y))
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

    /// Add a light
    pub fn add_light(&mut self, x: i32, y: i32, intensity: i32) {
        let light = Light::new(intensity);
        self.lights.insert((x, y), light);
    }

    pub fn compute_lighting(&mut self) {
        let mut map : FxHashMap<(i32, i32), f32> = FxHashMap::default();

        let mut rng = thread_rng();

        for (pos, l) in &self.lights {
            map.insert(pos.clone(), 1.0);

            if l.intensity > 0 {
                let mut tl = (pos.0 - 1, pos.1 - 1);
                let mut length = 3;

                let mut d = 1;

                let mut random : f32 = rng.gen();
                random -= 0.5;
                random *= 0.3;

                while d < l.intensity {

                    let i = 1.0 / (d*2) as f32 + random / d as f32;
                    for x in tl.0..tl.0 + length {
                        if let Some(value) = map.get_mut(&(x, tl.1)) {
                            *value += i;
                        } else {
                            map.insert((x, tl.1), i);
                        }

                        if let Some(value) = map.get_mut(&(x, tl.1 + length - 1)) {
                            *value += i;
                        } else {
                            map.insert((x, tl.1 + length - 1), i);
                        }
                    }

                    for y in tl.1+1..tl.1 + length - 1 {
                        if let Some(value) = map.get_mut(&(tl.0, y)) {
                            *value += i;
                        } else {
                            map.insert((tl.0, y), i);
                        }

                        if let Some(value) = map.get_mut(&(tl.0 + length - 1, y)) {
                            *value += i;
                        } else {
                            map.insert((tl.0 + length - 1, y), i);
                        }
                    }

                    d += 1;
                    length += 2;
                    tl.0 -= 1;
                    tl.1 -= 1;
                }
            }
        }

        self.light_map = map;
    }
}