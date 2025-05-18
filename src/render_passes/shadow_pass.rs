use crate::{
    nodes::{DirectionalLight, directional_light::DirectionalLightBufferData},
    renderer,
};

use super::RenderPass;

pub struct ShadowPass;

impl RenderPass for ShadowPass {
    fn render(
        &self,
        renderer: &mut crate::renderer::Renderer,
        context: &crate::context::GameContext,
        drawables: &[&dyn crate::nodes::node::Drawable],
        camera: &crate::nodes::Camera3D,
    ) {
        let lights = context.scene.collect_items::<DirectionalLight>();

        let mut offset = 0;
        let mut buffer_data: Vec<DirectionalLightBufferData> = Vec::new();

        // render shadows to lights framebuffer

        renderer.shadow_maps.bind_framebuffer();

        for light in lights {
            renderer
                .shadow_maps
                .commit_layer(offset as u32, light.num_cascades as i32);

            let light_space_matrices = light.render_shadow_map(
                drawables,
                &mut renderer.shadow_maps,
                offset,
                camera.transform.world_space(),
            );

            buffer_data.push(light.get_buffered_data(offset as u32, &light_space_matrices));

            offset += light.num_cascades;
        }

        renderer::depth_map_array::DepthMapArray::unbind_framebuffer();

        // finish shadow rendering

        // finally bind the ssbo and texture to shader
        renderer
            .direct_light_buffer
            .set_data(buffer_data.len() as i32, buffer_data.as_slice());

        renderer.default_shader.bind();
        // bind the shadow texture to texture slot 5
        renderer
            .shadow_maps
            .bind_shadow_map(&mut renderer.default_shader, "shadowMaps", 5);
    }
}
