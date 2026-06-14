use std::{any::TypeId, collections::HashMap, sync::Arc};

use maple_engine::{
    asset::{Asset, AssetLoader},
    platform::SendSync,
    prelude::Resource,
};
use maple_renderer::core::{
    DescriptorSet, DescriptorSetLayout, PipelineLayout, RenderDevice, RenderPipeline,
};

pub trait MaterialInstance: SendSync {
    fn layout(&self) -> DescriptorSetLayout;
    fn pipeline(&self, pipeline_layout: PipelineLayout) -> RenderPipeline;
    fn descriptor_set(&self) -> DescriptorSet;
}

pub enum PipelineStage {
    Opaque,
    Transparent,
}

#[derive(Default)]
pub struct MaterialPipelineCache {
    opaque: HashMap<TypeId, RenderPipeline>,
}

impl Resource for MaterialPipelineCache {}

pub struct Material {
    instance: Arc<dyn MaterialInstance>,
}

impl Asset for Material {
    type Loader = MaterialLoader;
}

pub struct PipelineCache {
    opaque: HashMap<TypeId, RenderPipeline>,
}

pub struct MaterialLoader {
    pub device: RenderDevice,
}

impl AssetLoader for MaterialLoader {
    type Asset = Material;
}

impl MaterialLoader {
    pub fn new(device: RenderDevice) -> Self {
        Self { device }
    }
}
