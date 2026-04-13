#[derive(Clone, Copy, Debug)]
pub struct Dimensions {
    pub width: u32,
    pub height: u32,
}

impl Dimensions {
    pub fn zero() -> Self {
        Self {
            width: 0,
            height: 0,
        }
    }
}
