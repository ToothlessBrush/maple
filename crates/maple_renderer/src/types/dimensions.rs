//! stores window dimensions

/// height and width of a window in pixles
#[derive(Clone, Copy, Debug)]
pub struct Dimensions {
    /// width the window
    pub width: u32,
    /// height of the window
    pub height: u32,
}

impl Dimensions {
    /// creates a size of 0 by 0
    pub fn zero() -> Self {
        Self {
            width: 0,
            height: 0,
        }
    }
}
