use crate::core::DescriptorSet;

pub trait Global {
    fn get_resource(&self, _key: &str) -> DescriptorSet;
}
