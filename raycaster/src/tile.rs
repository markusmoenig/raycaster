
#[derive(Clone, Debug, PartialEq)]
pub struct Tile {
    pub color           : Option<[u8;4]>,
    pub texture         : Option<(usize, (usize, usize, usize, usize))>,
}

/// A tile
impl Tile {

    /// Creates a new tile for a given image rectangle
    pub fn textured(image_id: usize, rect:(usize , usize, usize, usize)) -> Self {
        Self {
            color       : None,
            texture     : Some((image_id, rect)),
        }
    }

    /// Creates a new tile for a given color
    pub fn colored(color: [u8;4]) -> Self {

        Self {
            color       : Some(color),
            texture     : None,
        }
    }
}