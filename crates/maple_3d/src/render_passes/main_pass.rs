use bytemuck::{Pod, Zeroable};
use maple_engine::{Scene, utils::Debug};
use maple_renderer::{
    core::{
        Buffer, DescriptorBindingType, DescriptorSet, DescriptorSetLayoutDescriptor, RenderContext,
        StageFlags,
    },
    render_graph::{
        graph::{NodeLabel, RenderGraphContext},
        node::{RenderNode, RenderNodeContext, RenderNodeDescriptor, RenderTarget},
    },
};

struct SceneDescriptor {
    pub set: DescriptorSet,
    pub camera_data_buffer: Buffer<Camera3DBufferData>,
}

#[derive(Default, Debug, Pod, Zeroable, Clone, Copy)]
#[repr(C)]
struct SceneData {
    background_color: [f32; 4],
    ambient: f32,
    _padding: [f32; 3],
}

use crate::{
    components::material::MaterialProperties,
    nodes::{
        camera::{Camera3D, Camera3DBufferData},
        mesh::Mesh3D,
    },
};

pub struct Main;
impl NodeLabel for Main {}

#[derive(Default)]
pub struct MainPass {
    scene_data: Option<SceneDescriptor>,
}

impl RenderNode for MainPass {
    fn setup(
        &mut self,
        render_ctx: &RenderContext,
        _graph_ctx: &mut RenderGraphContext,
    ) -> RenderNodeDescriptor {
        // shader
        let shader = render_ctx.create_shader_pair(maple_renderer::core::ShaderPair::Glsl {
            vert: include_str!("../../res/shaders/default/default.vert"),
            frag: include_str!("../../res/shaders/default/default.frag"),
        });

        // layouts
        let material_layout = MaterialProperties::layout(render_ctx).clone();
        let mesh_layout = Mesh3D::layout(render_ctx).clone();
        let scene_layout = render_ctx.create_descriptor_set_layout(DescriptorSetLayoutDescriptor {
            label: Some("scene layout"),
            visibility: StageFlags::VERTEX | StageFlags::FRAGMENT,
            layout: &[
                DescriptorBindingType::UniformBuffer,
                DescriptorBindingType::UniformBuffer,
            ],
        });

        // buffers
        let scene_buffer = render_ctx.create_uniform_buffer(&SceneData::default());
        let camera_buffer = render_ctx.create_uniform_buffer(&Camera3DBufferData::default());

        let scene_set = render_ctx.build_descriptor_set(
            DescriptorSet::builder(&scene_layout)
                .uniform(0, &scene_buffer)
                .uniform(1, &camera_buffer),
        );

        self.scene_data = Some(SceneDescriptor {
            set: scene_set,
            camera_data_buffer: camera_buffer,
        });

        RenderNodeDescriptor {
            shader,
            descriptor_set_layouts: vec![scene_layout, material_layout, mesh_layout],
            target: RenderTarget::Surface,
        }
    }

    fn draw(
        &mut self,
        renderer_ctx: &RenderContext,
        node_ctx: &mut RenderNodeContext,
        _graph_ctx: &mut RenderGraphContext,
        scene: &Scene,
    ) {
        let cameras = scene.collect_items::<Camera3D>();
        let meshes = scene.collect_items::<Mesh3D>();

        let Some(camera) = cameras
            .iter()
            .filter(|c| c.is_active)
            .max_by_key(|c| c.priority)
        else {
            Debug::print_once("no active camera in scene");
            return;
        };

        let Some(scene_data) = &self.scene_data else {
            return;
        };

        renderer_ctx
            .write_buffer(
                &scene_data.camera_data_buffer,
                &camera.get_buffer_data(renderer_ctx.aspect_ratio()),
            )
            .expect("failed to write buffer");

        renderer_ctx
            .render(node_ctx, move |mut fb| {
                fb.bind_descriptor_set(0, &scene_data.set);

                for mesh in &meshes {
                    fb.bind_vertex_buffer(&mesh.get_vertex_buffer(renderer_ctx))
                        .bind_index_buffer(&mesh.get_index_buffer(renderer_ctx))
                        .bind_descriptor_set(1, &mesh.get_material().get_descriptor(renderer_ctx))
                        .bind_descriptor_set(2, &mesh.get_descriptor(renderer_ctx))
                        .draw_indexed();
                }
            })
            .expect("failed to render");
    }
}
