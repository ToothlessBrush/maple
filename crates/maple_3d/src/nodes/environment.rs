use std::path::Path;

use maple_engine::{
    Node, Scene,
    prelude::{EventReceiver, NodeTransform},
};
use maple_renderer::core::{
    RenderContext,
    texture::{LazyTexture, Texture},
};

pub struct Environment {
    pub transform: NodeTransform,
    pub children: Scene,
    pub events: EventReceiver,

    hdri_source: LazyTexture,
    ibl_strength: f32,
}

impl Node for Environment {
    fn get_events(&mut self) -> &mut EventReceiver {
        &mut self.events
    }

    fn get_children(&self) -> &Scene {
        &self.children
    }

    fn get_transform(&mut self) -> &mut NodeTransform {
        &mut self.transform
    }

    fn get_children_mut(&mut self) -> &mut Scene {
        &mut self.children
    }
}

impl Environment {
    pub fn new(hdr: impl AsRef<Path>) -> Self {
        let texture = LazyTexture::new_hdri_from_file(hdr, Some("skybox")).unwrap();
        // most of this is handled by the rendergraph
        Self {
            transform: NodeTransform::default(),
            children: Scene::default(),
            events: EventReceiver::default(),
            hdri_source: texture,
            ibl_strength: 1.0, // Default strength
        }
    }

    pub fn get_hdri_texture(&self, rcx: &RenderContext) -> Texture {
        self.hdri_source.texture(rcx)
    }

    pub fn ibl_strength(&self) -> f32 {
        self.ibl_strength
    }

    pub fn set_ibl_strength(&mut self, strength: f32) {
        self.ibl_strength = strength;
    }

    pub fn with_ibl_strength(mut self, strength: f32) -> Self {
        self.ibl_strength = strength;
        self
    }
}
