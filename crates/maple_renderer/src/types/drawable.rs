use std::sync::Arc;

use crate::{
    core::{DescriptorSet, buffer::Buffer},
    types::Vertex,
};

pub trait Drawable {
    fn vertex_buffer(&self) -> Buffer<Vertex>;
    fn index_buffer(&self) -> Buffer<u32>;
    fn get_resource(&self, _key: &str) -> Option<DescriptorSet> {
        None
    }
}
