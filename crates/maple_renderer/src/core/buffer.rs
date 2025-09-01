use std::marker::PhantomData;

use anyhow::{Result, bail};
use bytemuck::Pod;
use wgpu::{
    BufferUsages, COPY_BUFFER_ALIGNMENT, Device, Queue,
    util::{BufferInitDescriptor, DeviceExt},
};

#[derive(Debug, Clone)]
pub struct Buffer<T: ?Sized> {
    pub(crate) buffer: wgpu::Buffer,
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

    /// creates a buffer from an array size (NOT BYTE SIZE)
    pub fn from_size(device: &Device, len: usize, usage: BufferUsages, label: &str) -> Buffer<[T]> {
        let elem = size_of::<T>() as u64;
        let mut size = elem * (len as u64);

        // if the aligment is off then add padding
        if size % COPY_BUFFER_ALIGNMENT != 0 {
            size += COPY_BUFFER_ALIGNMENT - (size % COPY_BUFFER_ALIGNMENT);
        }

        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size,
            usage,
            mapped_at_creation: false,
        });

        Self {
            buffer,
            len,
            _ty: PhantomData,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn write(&self, queue: &Queue, data: &[T]) -> Result<()> {
        if !self.buffer.usage().contains(BufferUsages::COPY_DST) {
            bail!("write() requires COPY_DST usage");
        }
        if data.len() > self.len() {
            bail!(
                "write exceeds capacity: tried to write {} to a buffer of size {}",
                data.len(),
                self.len()
            );
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
