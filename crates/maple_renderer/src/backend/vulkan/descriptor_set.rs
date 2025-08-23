use std::sync::Arc;

use vulkano::descriptor_set::{DescriptorSet, layout::DescriptorSetLayout};

use crate::core::descriptor_set::{DescriptorSetBackend, DescriptorSetLayoutBackend};

#[derive(Clone, Debug)]
pub struct VulkanDescriptorSet {
    pub set: Arc<DescriptorSet>,
}

impl From<crate::core::descriptor_set::DescriptorSet> for VulkanDescriptorSet {
    fn from(value: crate::core::descriptor_set::DescriptorSet) -> Self {
        match value.backend {
            DescriptorSetBackend::Vulkan(vk) => vk,
        }
    }
}

#[derive(Clone, Debug)]
pub struct VulkanDescriptorSetLayout {
    pub layout: Arc<DescriptorSetLayout>,
}

impl From<crate::core::descriptor_set::DescriptorSetLayout> for VulkanDescriptorSetLayout {
    fn from(value: crate::core::descriptor_set::DescriptorSetLayout) -> Self {
        match value.backend {
            DescriptorSetLayoutBackend::Vulkan(vk) => vk,
        }
    }
}
