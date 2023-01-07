//use crate::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct Light {

    pub intensity           : i32,
}

/// A tile
impl Light {
    /// Creates a new sprite
    pub fn new(intensity: i32) -> Self {
        Self {
            intensity,
        }
    }
}