use crate::buffer::GpuBuffer;
use vulkano::buffer::{BufferContents, Subbuffer};

pub(crate) struct VulkanBufferArray<T: BufferContents> {
    pub buffer: Subbuffer<[T]>,
}

pub(crate) struct VulkanBuffer<T: BufferContents> {
    pub buffer: Subbuffer<T>,
}

impl<T: BufferContents> GpuBuffer<T> for VulkanBufferArray<T> {}
impl<T: BufferContents> GpuBuffer<T> for VulkanBuffer<T> {}
