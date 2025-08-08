use std::{any::Any, sync::Arc};

use crate::backend::vulkan::{VulkanBuffer, VulkanBufferArray};
use anyhow::{anyhow, bail};
use vulkano::buffer::BufferContents;

pub trait GpuBuffer<T>: Any + Send + Sync {
    fn as_any(self) -> Arc<dyn Any + Send + Sync>;
}

pub enum BufferBackend<T: BufferContents> {
    VK(VulkanBuffer<T>),
    VKArray(VulkanBufferArray<T>),
}

pub struct Buffer<T: BufferContents> {
    pub inner: BufferBackend<T>,
}

impl<T: BufferContents> Buffer<T> {}

impl<T: BufferContents> From<VulkanBufferArray<T>> for Buffer<T> {
    fn from(value: VulkanBufferArray<T>) -> Self {
        Self {
            inner: BufferBackend::VKArray(value),
        }
    }
}

impl<T: BufferContents> From<VulkanBuffer<T>> for Buffer<T> {
    fn from(value: VulkanBuffer<T>) -> Self {
        Self {
            inner: BufferBackend::VK(value),
        }
    }
}
