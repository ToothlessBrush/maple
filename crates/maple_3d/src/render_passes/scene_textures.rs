use maple_engine::Scene;
use maple_renderer::{
    core::{
        RenderContext,
        texture::{Texture, TextureCreateInfo, TextureFormat, TextureUsage},
    },
    render_graph::{
        graph::{NodeLabel, RenderGraphContext},
        node::RenderNode,
    },
};

pub struct SceneTexturesLabel;
impl NodeLabel for SceneTexturesLabel {}

struct SceneTextureSet {
    msaa_color: Texture,
    resolved_color: Texture,
    msaa_normal: Texture,
    resolved_normal: Texture,
    msaa_depth: Texture,
}

impl SceneTextureSet {
    fn create(render_ctx: &RenderContext, dimensions: (u32, u32)) -> Self {
        let surface_format = render_ctx.surface_format();

        let msaa_color = render_ctx.create_texture(TextureCreateInfo {
            label: Some("scene_msaa_color"),
            width: dimensions.0,
            height: dimensions.1,
            format: surface_format,
            usage: TextureUsage::RENDER_ATTACHMENT,
            sample_count: 4,
            mip_level: 1,
        });

        let resolved_color = render_ctx.create_texture(TextureCreateInfo {
            label: Some("scene_resolved_color"),
            width: dimensions.0,
            height: dimensions.1,
            format: surface_format,
            usage: TextureUsage::RENDER_ATTACHMENT | TextureUsage::TEXTURE_BINDING,
            sample_count: 1,
            mip_level: 1,
        });

        let msaa_normal = render_ctx.create_texture(TextureCreateInfo {
            label: Some("scene_msaa_normal"),
            width: dimensions.0,
            height: dimensions.1,
            format: TextureFormat::RGBA8,
            usage: TextureUsage::RENDER_ATTACHMENT,
            sample_count: 4,
            mip_level: 1,
        });

        let resolved_normal = render_ctx.create_texture(TextureCreateInfo {
            label: Some("scene_resolved_normal"),
            width: dimensions.0,
            height: dimensions.1,
            format: TextureFormat::RGBA8,
            usage: TextureUsage::RENDER_ATTACHMENT | TextureUsage::TEXTURE_BINDING,
            sample_count: 1,
            mip_level: 1,
        });

        let msaa_depth = render_ctx.create_texture(TextureCreateInfo {
            label: Some("scene_msaa_depth"),
            width: dimensions.0,
            height: dimensions.1,
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

    fn share_to_graph(&self, graph_ctx: &mut RenderGraphContext) {
        graph_ctx.add_shared_resource("msaa_color_texture", self.msaa_color.clone());
        graph_ctx.add_shared_resource("resolved_color_texture", self.resolved_color.clone());
        graph_ctx.add_shared_resource("msaa_normal_texture", self.msaa_normal.clone());
        graph_ctx.add_shared_resource("resolved_normal_texture", self.resolved_normal.clone());
        graph_ctx.add_shared_resource("main_depth_texture", self.msaa_depth.clone());
    }
}

/// Resource node that creates and manages the main scene render textures
/// This allows other passes to share these textures without creating their own
#[derive(Default)]
pub struct SceneTextures {
    textures: Option<SceneTextureSet>,
}

impl RenderNode for SceneTextures {
    fn setup(&mut self, render_ctx: &RenderContext, graph_ctx: &mut RenderGraphContext) {
        let dimensions = render_ctx.surface_size();
        let textures = SceneTextureSet::create(render_ctx, dimensions);
        textures.share_to_graph(graph_ctx);
        self.textures = Some(textures);
    }

    fn draw(
        &mut self,
        _render_ctx: &RenderContext,
        graph_ctx: &mut RenderGraphContext,
        _scene: &Scene,
    ) {
        // Re-share textures in case they were recreated during resize
        if let Some(textures) = &self.textures {
            textures.share_to_graph(graph_ctx);
        }
    }

    fn resize(&mut self, render_ctx: &RenderContext, dimensions: [u32; 2]) {
        let textures = SceneTextureSet::create(render_ctx, (dimensions[0], dimensions[1]));
        self.textures = Some(textures);
        // Textures will be shared in the next draw() call
    }
}
