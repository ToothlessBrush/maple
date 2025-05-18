use crate::{
    nodes::{PointLight, point_light::PointLightBufferData},
    renderer,
};

use super::RenderPass;

/// a render pass responsible for rendering shadow_maps for point lights
pub struct CubeShadowPass;

impl RenderPass for CubeShadowPass {
    fn render(
        &self,
        renderer: &mut crate::renderer::Renderer,
        context: &crate::context::GameContext,
        drawables: &[&dyn crate::nodes::node::Drawable],
        _camera: &crate::nodes::Camera3D,
    ) {
        let lights = context.scene.collect_items::<PointLight>();

        let mut buffer_data: Vec<PointLightBufferData> = Vec::new();

        renderer.shadow_cube_maps.bind_framebuffer();

        for (i, light) in lights.iter().enumerate() {
            renderer.shadow_cube_maps.commit_layer((i * 6) as u32);

            // render the shadow_maps
            let _light_space_matrices =
                light.render_shadow_map(drawables, &mut renderer.shadow_cube_maps, i);

            // assign the ssbo object to refrence this light at that index
            buffer_data.push(light.get_buffered_data(i))
        }

        renderer.shadow_cube_maps.unbind_framebuffer();

        // set the data on the ssbo object
        renderer
            .point_light_buffer
            .set_data(buffer_data.len() as i32, buffer_data.as_slice());

        // bind the depth texture to the shader uniform
        renderer.default_shader.bind();
        renderer.shadow_cube_maps.bind_shadow_map(
            &mut renderer.default_shader,
            "shadowCubeMaps",
            6,
        );
    }
}
