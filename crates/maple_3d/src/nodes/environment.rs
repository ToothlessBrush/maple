use std::path::Path;

use maple_engine::{
    Node, Scene,
    prelude::{EventReceiver, NodeTransform},
};
use maple_renderer::core::{
    RenderContext,
    texture::{LazyTexture, Texture, TextureCube},
};

pub struct Environment {
    pub transform: NodeTransform,
    pub children: Scene,
    pub events: EventReceiver,

    hdri_source: LazyTexture,
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

        Self {
            transform: NodeTransform::default(),
            children: Scene::default(),
            events: EventReceiver::default(),
            hdri_source: texture,
        }
    }

    pub fn get_hdri_texture(&self, rcx: &RenderContext) -> Texture {
        self.hdri_source.texture(rcx)
    }
}
