use std::{marker::PhantomData, sync::Arc};

use bytemuck::Pod;
use wgpu::{
    BufferUsages, COPY_BUFFER_ALIGNMENT, Device, Queue,
    util::{BufferInitDescriptor, DeviceExt},
};

#[derive(Debug)]
pub struct Buffer<T: ?Sized> {
    pub(crate) buffer: wgpu::Buffer,
    len: usize,
    _ty: std::marker::PhantomData<T>,
}

impl<T: ?Sized> Clone for Buffer<T> {
    fn clone(&self) -> Self {
        Self {
            buffer: self.buffer.clone(),
            len: self.len,
            _ty: PhantomData,
        }
    }
}

impl<T: 'static> GraphResource for Buffer<T> {}

impl<T: Pod> Buffer<[T]> {
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

    pub fn len(&self) -> usize {
        self.len
    }

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

impl<T: Pod> Buffer<T> {
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

use parking_lot::RwLock;

use crate::render_graph::graph::GraphResource;

#[derive(Debug, Clone)]
enum LazyBufferState {
    Pending(Vec<u8>),
    Clean(wgpu::Buffer),
    Dirty(wgpu::Buffer, Vec<u8>),
}

#[derive(Debug)]
pub struct LazyBuffer<T: ?Sized> {
    state: Arc<RwLock<LazyBufferState>>,
    usage: BufferUsages,
    label: Option<&'static str>,
    _ty: PhantomData<T>,
}

impl<T: ?Sized> Clone for LazyBuffer<T> {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            usage: self.usage,
            label: self.label,
            _ty: PhantomData,
        }
    }
}

pub trait LazyBufferable<T: ?Sized> {
    fn get_buffer(&self, device: &Device, queue: &Queue) -> Buffer<T>;
    fn write(&self, new_data: &T);
    fn sync(&self, queue: &Queue);
}

impl<T: Pod> LazyBuffer<T> {
    pub fn new(data: &T, usage: BufferUsages, label: Option<&'static str>) -> LazyBuffer<T> {
        Self {
            state: Arc::new(RwLock::new(LazyBufferState::Pending(bytemuck::bytes_of(data).to_vec()))),
            usage,
            label,
            _ty: PhantomData,
        }
    }
}

impl<T: Pod> LazyBuffer<[T]> {
    pub fn from_slice(
        data: &[T],
        usage: BufferUsages,
        label: Option<&'static str>,
    ) -> LazyBuffer<[T]> {
        Self {
            state: Arc::new(RwLock::new(LazyBufferState::Pending(
                bytemuck::cast_slice(data).to_vec(),
            ))),
            usage,
            label,
            _ty: PhantomData,
        }
    }
}

impl<T: Pod> LazyBufferable<T> for LazyBuffer<T> {
    fn get_buffer(&self, device: &Device, queue: &Queue) -> Buffer<T> {
        // try to read if the buffer is clean
        {
            let read_guard = self.state.read();
            if let LazyBufferState::Clean(buffer) = &*read_guard {
                let len = buffer.size() as usize / std::mem::size_of::<T>();
                return Buffer {
                    buffer: buffer.clone(),
                    len,
                    _ty: PhantomData,
                };
            }
        }

        // if the buffer isnt clean make it clean
        let mut write_guard = self.state.write();
        match &mut *write_guard {
            LazyBufferState::Pending(data) => {
                // Create new buffer
                let buffer = device.create_buffer_init(&BufferInitDescriptor {
                    label: self.label,
                    contents: data,
                    usage: self.usage,
                });
                let result = Buffer {
                    buffer: buffer.clone(),
                    len: 1,
                    _ty: PhantomData,
                };
                *write_guard = LazyBufferState::Clean(buffer);
                result
            }
            LazyBufferState::Clean(buffer) => Buffer {
                buffer: buffer.clone(),
                len: 1,
                _ty: PhantomData,
            },
            LazyBufferState::Dirty(buffer, data) => {
                // Write to existing buffer
                queue.write_buffer(buffer, 0, data);
                let result = Buffer {
                    buffer: buffer.clone(),
                    len: 1,
                    _ty: PhantomData,
                };
                *write_guard = LazyBufferState::Clean(buffer.clone());
                result
            }
        }
    }

    fn write(&self, new_data: &T) {
        let mut write_guard = self.state.write();
        let bytes = bytemuck::bytes_of(new_data).to_vec();

        *write_guard = match std::mem::replace(&mut *write_guard, LazyBufferState::Pending(vec![]))
        {
            LazyBufferState::Pending(_) => LazyBufferState::Pending(bytes),
            LazyBufferState::Clean(buffer) => LazyBufferState::Dirty(buffer, bytes),
            LazyBufferState::Dirty(buffer, _) => LazyBufferState::Dirty(buffer, bytes),
        };
    }

    fn sync(&self, queue: &Queue) {
        let mut write_guard = self.state.write();
        if let LazyBufferState::Dirty(buffer, data) = &*write_guard {
            queue.write_buffer(buffer, 0, data);
            *write_guard = LazyBufferState::Clean(buffer.clone());
        }
    }
}

impl<T: Pod> LazyBufferable<[T]> for LazyBuffer<[T]> {
    fn get_buffer(&self, device: &Device, queue: &Queue) -> Buffer<[T]> {
        // First try to read
        {
            let read_guard = self.state.read();
            if let LazyBufferState::Clean(buffer) = &*read_guard {
                let len = buffer.size() as usize / std::mem::size_of::<T>();
                return Buffer {
                    buffer: buffer.clone(),
                    len,
                    _ty: PhantomData,
                };
            }
        }

        // Need to modify state
        let mut write_guard = self.state.write();
        match &mut *write_guard {
            LazyBufferState::Pending(data) => {
                let len = data.len() / std::mem::size_of::<T>();
                let buffer = device.create_buffer_init(&BufferInitDescriptor {
                    label: self.label,
                    contents: data,
                    usage: self.usage,
                });
                let result = Buffer {
                    buffer: buffer.clone(),
                    len,
                    _ty: PhantomData,
                };
                *write_guard = LazyBufferState::Clean(buffer);
                result
            }
            LazyBufferState::Clean(buffer) => {
                let len = buffer.size() as usize / std::mem::size_of::<T>();
                Buffer {
                    buffer: buffer.clone(),
                    len,
                    _ty: PhantomData,
                }
            }
            LazyBufferState::Dirty(buffer, data) => {
                let len = data.len() / std::mem::size_of::<T>();
                queue.write_buffer(buffer, 0, data);
                let result = Buffer {
                    buffer: buffer.clone(),
                    len,
                    _ty: PhantomData,
                };
                *write_guard = LazyBufferState::Clean(buffer.clone());
                result
            }
        }
    }

    fn write(&self, new_data: &[T]) {
        let mut write_guard = self.state.write();
        let bytes = bytemuck::cast_slice(new_data).to_vec();

        *write_guard = match std::mem::replace(&mut *write_guard, LazyBufferState::Pending(vec![]))
        {
            LazyBufferState::Pending(_) => LazyBufferState::Pending(bytes),
            LazyBufferState::Clean(buffer) => LazyBufferState::Dirty(buffer, bytes),
            LazyBufferState::Dirty(buffer, _) => LazyBufferState::Dirty(buffer, bytes),
        };
    }

    fn sync(&self, queue: &Queue) {
        let mut write_guard = self.state.write();
        if let LazyBufferState::Dirty(buffer, data) = &*write_guard {
            queue.write_buffer(buffer, 0, data);
            *write_guard = LazyBufferState::Clean(buffer.clone());
        }
    }
}

impl<T> From<Buffer<[T]>> for LazyBuffer<[T]> {
    fn from(value: Buffer<[T]>) -> Self {
        let usage = value.buffer.usage();
        LazyBuffer {
            state: Arc::new(RwLock::new(LazyBufferState::Clean(value.buffer))),
            usage,
            label: None,
            _ty: PhantomData,
        }
    }
}

impl<T> From<Buffer<T>> for LazyBuffer<T> {
    fn from(value: Buffer<T>) -> Self {
        let usage = value.buffer.usage();
        LazyBuffer {
            state: Arc::new(RwLock::new(LazyBufferState::Clean(value.buffer))),
            usage,
            label: None,
            _ty: PhantomData,
        }
    }
}
