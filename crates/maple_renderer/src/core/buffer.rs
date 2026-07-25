//! buffers store data on the gpu

use std::marker::PhantomData;

use bytemuck::Pod;
use wgpu::{
    BufferUsages, COPY_BUFFER_ALIGNMENT, Device, Queue,
    util::{BufferInitDescriptor, DeviceExt},
};

/// a typed handle to a data buffer that lives on the gpu
#[derive(Debug)]
pub struct Buffer<T: ?Sized + SendSync> {
    pub(crate) buffer: wgpu::Buffer,
    len: usize,
    _ty: std::marker::PhantomData<T>,
}

impl<T: ?Sized + SendSync> Clone for Buffer<T> {
    fn clone(&self) -> Self {
        Self {
            buffer: self.buffer.clone(),
            len: self.len,
            _ty: PhantomData,
        }
    }
}

impl<T: 'static + SendSync> GraphResource for Buffer<T> {}

impl<T: Pod + SendSync> Buffer<[T]> {
    pub(crate) fn from_slice(
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
    pub(crate) fn from_size(
        device: &Device,
        len: usize,
        usage: BufferUsages,
        label: &str,
    ) -> Buffer<[T]> {
        let elem = size_of::<T>() as u64;
        let mut size = elem * (len as u64);

        // if the aligment is off then add padding
        if size.is_multiple_of(COPY_BUFFER_ALIGNMENT) {
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

    /// length of buffer array
    pub fn len(&self) -> usize {
        self.len
    }

    /// if the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub(crate) fn write(&self, queue: &Queue, data: &[T]) {
        assert!(
            self.buffer.usage().contains(BufferUsages::COPY_DST),
            "write() requires COPY_DST usage"
        );
        assert!(
            data.len() <= self.len(),
            "tried to write to a buffer with smaller size"
        );

        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(data));
    }
}

impl<T: Pod + SendSync> Buffer<T> {
    pub(crate) fn from(device: &Device, data: &T, usage: BufferUsages, label: &str) -> Buffer<T> {
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

    /// Creates an empty buffer for a single T
    pub(crate) fn empty(device: &Device, usage: BufferUsages, label: &str) -> Buffer<T> {
        let mut size = size_of::<T>() as u64;
        // Ensure proper alignment for copy operations
        if size.is_multiple_of(COPY_BUFFER_ALIGNMENT) {
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
            len: 1,
            _ty: PhantomData,
        }
    }

    pub(crate) fn write(&self, queue: &Queue, value: &T) {
        assert!(
            self.buffer.usage().contains(BufferUsages::COPY_DST),
            "write() requires COPY_DST usage"
        );

        queue.write_buffer(&self.buffer, 0, bytemuck::bytes_of(value));
    }
}

use crate::{platform::SendSync, render_graph::graph::GraphResource};
