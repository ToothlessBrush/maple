use bytemuck::{Pod, Zeroable};
use glam::Mat4;
use maple_engine::{Scene, utils::Debug};
use maple_renderer::{
    core::{
        Buffer, DepthCompare, DescriptorBindingType, DescriptorSet, DescriptorSetLayoutDescriptor,
        RenderContext, StageFlags, texture::TextureUsage,
    },
    render_graph::{
        graph::{NodeLabel, RenderGraphContext},
        node::{DepthTarget, RenderNode, RenderNodeContext, RenderNodeDescriptor, RenderTarget},
    },
};

struct SceneDescriptor {
    pub scene_set: DescriptorSet,
    pub camera_data_buffer: Buffer<Camera3DBufferData>,

    pub light_set: DescriptorSet,
    pub direct_light_buffer: Buffer<DirectionalLightBuffer>,
    pub point_light_buffer: Buffer<PointLightBuffer>,
}

#[derive(Default, Debug, Pod, Zeroable, Clone, Copy)]
#[repr(C)]
struct SceneData {
    background_color: [f32; 4],
    ambient: f32,
    _padding: [f32; 3],
}

impl SceneData {
    pub fn ambient(mut self, ambient: f32) -> Self {
        self.ambient = ambient;
        self
    }
}

use crate::{
    components::material::MaterialProperties,
    nodes::{
        camera::{Camera3D, Camera3DBufferData},
        directional_light::{DirectionalLight, DirectionalLightBuffer, DirectionalLightBufferData},
        mesh::Mesh3D,
        point_light::{PointLight, PointLightBuffer, PointLightBufferData},
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
        let shader = render_ctx.create_shader_pair(maple_renderer::core::ShaderPair::Wgsl {
            vert: include_str!("../../res/shaders/default/default.vert.wgsl"),
            frag: include_str!("../../res/shaders/default/default.frag.wgsl"),
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
        let scene_buffer = render_ctx.create_uniform_buffer(&SceneData::default().ambient(0.1));
        let camera_buffer = render_ctx.create_uniform_buffer(&Camera3DBufferData::default());

        let scene_set = render_ctx.build_descriptor_set(
            DescriptorSet::builder(&scene_layout)
                .uniform(0, &scene_buffer)
                .uniform(1, &camera_buffer),
        );

        let direct_light_buffer =
            render_ctx.create_empty_storage_buffer::<DirectionalLightBuffer>();

        let point_light_buffer = render_ctx.create_empty_storage_buffer::<PointLightBuffer>();

        let light_layout = render_ctx.create_descriptor_set_layout(DescriptorSetLayoutDescriptor {
            label: Some("light layout"),
            visibility: StageFlags::FRAGMENT,
            layout: &[
                DescriptorBindingType::Storage { read_only: true },
                DescriptorBindingType::Storage { read_only: true },
            ],
        });

        let light_set = render_ctx.build_descriptor_set(
            DescriptorSet::builder(&light_layout)
                .storage(0, &direct_light_buffer)
                .storage(1, &point_light_buffer),
        );

        self.scene_data = Some(SceneDescriptor {
            scene_set,
            camera_data_buffer: camera_buffer,
            light_set,
            direct_light_buffer,
            point_light_buffer,
        });

        RenderNodeDescriptor {
            shader,
            descriptor_set_layouts: vec![scene_layout, material_layout, mesh_layout, light_layout],
            target: vec![RenderTarget::Surface],
            depth: DepthTarget::Auto {
                compare_function: DepthCompare::Less,
            },
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
        let direct_lights = scene.collect_items::<DirectionalLight>();
        let point_lights = scene.collect_items::<PointLight>();

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

        let direct_light_buffer = DirectionalLightBuffer::from_lights(
            &direct_lights
                .iter()
                .map(|light| light.get_buffered_data(0, &[Mat4::IDENTITY]))
                .collect::<Vec<_>>(),
        );

        renderer_ctx.write_buffer(&scene_data.direct_light_buffer, &direct_light_buffer);

        let point_light_buffers = PointLightBuffer::from_lights(
            &point_lights
                .iter()
                .map(|light| light.get_buffered_data(0))
                .collect::<Vec<_>>(),
        );

        renderer_ctx.write_buffer(&scene_data.point_light_buffer, &point_light_buffers);

        renderer_ctx.write_buffer(
            &scene_data.camera_data_buffer,
            &camera.get_buffer_data(renderer_ctx.aspect_ratio()),
        );
        renderer_ctx
            .render(node_ctx, move |mut fb| {
                fb.bind_descriptor_set(0, &scene_data.scene_set)
                    .bind_descriptor_set(3, &scene_data.light_set);

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
