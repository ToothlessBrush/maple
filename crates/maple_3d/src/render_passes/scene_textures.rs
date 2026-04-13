use maple_engine::GameContext;
use maple_renderer::{
    core::{
        RenderContext,
        texture::{Texture, TextureCreateInfo, TextureFormat, TextureUsage},
    },
    render_graph::{graph::RenderGraphContext, node::RenderNode},
    types::Dimensions,
};

struct SceneTextureSet {
    msaa_color: Texture,
    resolved_color: Texture,
    msaa_normal: Texture,
    resolved_normal: Texture,
    msaa_depth: Texture,
}

impl SceneTextureSet {
    fn create(rcx: &RenderContext, dimensions: Dimensions) -> Self {
        let msaa_color = rcx.create_texture(TextureCreateInfo {
            label: Some("scene_msaa_color"),
            width: dimensions.width,
            height: dimensions.height,
            format: TextureFormat::RGBA16Float,
            usage: TextureUsage::RENDER_ATTACHMENT,
            sample_count: 4,
            mip_level: 1,
        });

        let resolved_color = rcx.create_texture(TextureCreateInfo {
            label: Some("scene_resolved_color"),
            width: dimensions.width,
            height: dimensions.height,
            format: TextureFormat::RGBA16Float,
            usage: TextureUsage::RENDER_ATTACHMENT | TextureUsage::TEXTURE_BINDING,
            sample_count: 1,
            mip_level: 1,
        });

        let msaa_normal = rcx.create_texture(TextureCreateInfo {
            label: Some("scene_msaa_normal"),
            width: dimensions.width,
            height: dimensions.height,
            format: TextureFormat::RGBA8,
            usage: TextureUsage::RENDER_ATTACHMENT,
            sample_count: 4,
            mip_level: 1,
        });

        let resolved_normal = rcx.create_texture(TextureCreateInfo {
            label: Some("scene_resolved_normal"),
            width: dimensions.width,
            height: dimensions.height,
            format: TextureFormat::RGBA8,
            usage: TextureUsage::RENDER_ATTACHMENT | TextureUsage::TEXTURE_BINDING,
            sample_count: 1,
            mip_level: 1,
        });

        let msaa_depth = rcx.create_texture(TextureCreateInfo {
            label: Some("scene_msaa_depth"),
            width: dimensions.width,
            height: dimensions.height,
            format: TextureFormat::Depth32,
            usage: TextureUsage::RENDER_ATTACHMENT,
            sample_count: 4,
            mip_level: 1,
        });

        Self {
            msaa_color,
            resolved_color,
            msaa_normal,
            resolved_normal,
            msaa_depth,
        }
    }

    fn share_to_graph(&self, gcx: &mut RenderGraphContext) {
        gcx.add_shared_resource("msaa_color_texture", self.msaa_color.clone());
        gcx.add_shared_resource("resolved_color_texture", self.resolved_color.clone());
        gcx.add_shared_resource("msaa_normal_texture", self.msaa_normal.clone());
        gcx.add_shared_resource("resolved_normal_texture", self.resolved_normal.clone());
        gcx.add_shared_resource("main_depth_texture", self.msaa_depth.clone());
    }
}

/// Resource node that creates and manages the main scene render textures
/// This allows other passes to share these textures without creating their own
pub struct SceneTextures {
    textures: SceneTextureSet,
}

impl SceneTextures {
    pub fn setup(rcx: &RenderContext, gcx: &mut RenderGraphContext) -> Self {
        let dimensions = rcx.surface_size();
        let textures = SceneTextureSet::create(rcx, dimensions);
        textures.share_to_graph(gcx);
        Self { textures }
    }
}

impl RenderNode for SceneTextures {
    fn draw(&mut self, _: &RenderContext, gcx: &mut RenderGraphContext, _: &GameContext) {
        // Re-share textures in case they were recreated during resize
        self.textures.share_to_graph(gcx);
    }

    fn resize(&mut self, rcx: &RenderContext, dimensions: Dimensions) {
        let textures = SceneTextureSet::create(rcx, dimensions);
        self.textures = textures;
    }
}
