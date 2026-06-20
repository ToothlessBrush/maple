use std::{any::TypeId, collections::HashMap, sync::Arc};

use maple_engine::{
    asset::{Asset, AssetLibrary, AssetLoader},
    platform::SendSync,
    prelude::Resource,
};
use maple_renderer::core::{
    DescriptorSet, DescriptorSetLayout, PipelineLayout, RenderContext, RenderDevice, RenderPipeline,
};

pub trait MaterialInstance: SendSync {
    fn stage(&self) -> PipelineStage;
    fn layout(&self, rcx: &RenderContext) -> DescriptorSetLayout;
    fn pipeline(&self, rcx: &RenderContext, pipeline_layout: PipelineLayout) -> RenderPipeline;
    fn descriptor_set(
        &mut self,
        assets: &AssetLibrary,
        rcx: &RenderContext,
        layout: &DescriptorSetLayout,
    ) -> MaterialDescriptorState;
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

impl Material {
    pub fn new<T: MaterialInstance + 'static>(instance: T) -> Self {
        Self {
            instance: Arc::new(instance),
        }
    }
}

/// lets the pass know the current state of the material if it requests it before textures are
/// loading
pub enum MaterialDescriptorState {
    Loading,
    Ready(DescriptorSet),
}

impl Into<Option<DescriptorSet>> for MaterialDescriptorState {
    fn into(self) -> Option<DescriptorSet> {
        match self {
            Self::Loading => None,
            Self::Ready(set) => Some(set),
        }
    }
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
