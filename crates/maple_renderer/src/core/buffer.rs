use std::marker::PhantomData;

use anyhow::{Result, bail};
use bytemuck::Pod;
use wgpu::{
    BufferUsages, Device, Queue,
    util::{BufferInitDescriptor, DeviceExt},
};

pub struct Buffer<T: ?Sized> {
    pub buffer: wgpu::Buffer,
    len: usize,
    _ty: std::marker::PhantomData<T>,
}

impl<T: Pod> Buffer<[T]> {
    pub fn from_slice(
        device: &Device,
        data: &[T],
        usage: BufferUsages,
        label: &str,
    ) -> Buffer<[T]> {
        let buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some(label),
            contents: bytemuck::cast_slice(data),
            usage,
        });

        Self {
            buffer,
            len: data.len(),
            _ty: PhantomData,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn write(&self, queue: &Queue, data: &[T]) -> Result<()> {
        if !self.buffer.usage().contains(BufferUsages::COPY_DST) {
            bail!("write() requires COPY_DST usage");
        }
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(data));
        Ok(())
    }
}

impl<T: Pod> Buffer<T> {
    pub fn from(device: &Device, data: &T, usage: BufferUsages, label: &str) -> Buffer<T> {
        let buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some(label),
            contents: bytemuck::bytes_of(data),
            usage,
        });

        Self {
            buffer,
            len: 1,
            _ty: PhantomData,
        }
    }

    pub fn write(&self, queue: &Queue, value: &T) -> Result<()> {
        if !self.buffer.usage().contains(BufferUsages::COPY_DST) {
            bail!("write() requires COPY_DST usage");
        }
        queue.write_buffer(&self.buffer, 0, bytemuck::bytes_of(value));
        Ok(())
    }
}
