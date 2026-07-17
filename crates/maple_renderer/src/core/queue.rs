use std::sync::Arc;

use bytemuck::Pod;
use maple_engine::platform::SendSync;
use wgpu::Queue;

use crate::core::{Buffer, texture::Texture};

#[derive(Clone, Debug)]
pub struct RenderQueue {
    pub(crate) queue: Arc<Queue>,
}

impl RenderQueue {
    pub fn write_buffer<T: Pod + SendSync + Sized>(&self, buffer: &Buffer<T>, value: &T) {
        buffer.write(&self.queue, value)
    }

    pub fn write_buffer_slice<T: Pod + SendSync>(&self, buffer: &Buffer<[T]>, data: &[T]) {
        buffer.write(&self.queue, data)
    }

    pub fn write_texture(&self, texture: &Texture, data: &[u8]) {
        texture.write(&self.queue, data)
    }

    pub fn write_texture_reigon(
        &self,
        texture: &Texture,
        data: &[u8],
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) {
        texture.write_region(&self.queue, x, y, width, height, data);
    }
}
