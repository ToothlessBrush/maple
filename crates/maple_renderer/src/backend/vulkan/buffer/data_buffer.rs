use std::sync::Arc;

use crate::core::buffer::{Buffer, BufferBackend};
use anyhow::anyhow;
use vulkano::buffer::{BufferContents, Subbuffer};

pub(crate) struct VulkanBufferArray<T: BufferContents> {
    pub buffer: Arc<Subbuffer<[T]>>,
}

pub(crate) struct VulkanBuffer<T: BufferContents> {
    pub buffer: Arc<Subbuffer<T>>,
}

impl<T: BufferContents> TryFrom<Buffer<T>> for VulkanBuffer<T> {
    type Error = anyhow::Error;

    fn try_from(value: Buffer<T>) -> Result<Self, Self::Error> {
        if let BufferBackend::VK(b) = value.inner {
            Ok(b)
        } else {
            Err(anyhow!("failed to get vulkan buffer"))
        }
    }
}

impl<T: BufferContents> TryFrom<Buffer<T>> for VulkanBufferArray<T> {
    type Error = anyhow::Error;

    fn try_from(value: Buffer<T>) -> Result<Self, Self::Error> {
        if let BufferBackend::VKArray(b) = value.inner {
            Ok(b)
        } else {
            Err(anyhow!("failed to get vulkan buffer array"))
        }
    }
}
