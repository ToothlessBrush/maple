use crate::vulkan::{VulkanBuffer, VulkanBufferArray};
use vulkano::buffer::BufferContents;

pub trait GpuBuffer<T> {}

pub struct Buffer<T> {
    inner: Box<dyn GpuBuffer<T>>,
}

impl<T> Buffer<T> {}

impl<T: 'static + Send + Sync + BufferContents> From<VulkanBufferArray<T>> for Buffer<T> {
    fn from(value: VulkanBufferArray<T>) -> Self {
        Self {
            inner: Box::new(value),
        }
    }
}

impl<T: 'static + Send + Sync + BufferContents> From<VulkanBuffer<T>> for Buffer<T> {
    fn from(value: VulkanBuffer<T>) -> Self {
        Self {
            inner: Box::new(value),
        }
    }
}
