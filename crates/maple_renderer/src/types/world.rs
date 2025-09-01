use crate::types::{Drawable, globals::Global};

/// stores renderable scene data
#[derive(Default)]
pub struct World<'a> {
    pub drawable: &'a [&'a dyn Drawable],
    pub globals: &'a [&'a dyn Global],
}
