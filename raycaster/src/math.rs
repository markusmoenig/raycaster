
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug)]
pub struct vec2 {
    pub x            : f32,
    pub y            : f32,
}

/// The world map
impl vec2 {

    /// Inits the vec2 with zeros
    pub fn zero() -> Self {
        Self {
                    x: 0.0,
                    y: 0.0,
        }
    }

    /// Inits the vec2 with the given values
    pub fn new(x: f32, y: f32) -> Self {
        Self {
                    x,
                    y,
        }
    }
}