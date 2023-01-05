
#[derive(Clone, Debug, PartialEq)]
pub struct Tile {
    pub color           : Option<[u8;4]>,
    pub texture         : Option<(usize, (usize, usize, usize, usize))>,
    pub frames          : u16
}

/// A tile
impl Tile {

    /// Creates a new tile for a given image rectangle
    pub fn textured(image_id: usize, rect:(usize , usize, usize, usize)) -> Self {
        Self {
            color       : None,
            texture     : Some((image_id, rect)),
            frames      : 1,
        }
    }

    /// Creates a new tile for a given image rectangle which has multiple frames horizontally. The rect is the first frame. Frames is the total number of frames including the first frame.
    pub fn textured_anim(image_id: usize, rect:(usize , usize, usize, usize), frames: u16) -> Self {
        Self {
            color       : None,
            texture     : Some((image_id, rect)),
            frames      : frames,
        }
    }

    /// Creates a new tile for a given color
    pub fn colored(color: [u8;4]) -> Self {

        Self {
            color       : Some(color),
            texture     : None,
            frames      : 1,
        }
    }
}