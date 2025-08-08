use vulkano::{buffer::BufferContents, pipeline::graphics::vertex_input::Vertex as VulkanVertex};

#[repr(C)]
#[derive(Clone, Copy, Debug, BufferContents, VulkanVertex)]
pub struct Vertex {
    #[format(R32G32B32_SFLOAT)]
    pub position: [f32; 3],

    #[format(R32G32B32_SFLOAT)]
    pub normal: [f32; 3],

    #[format(R32G32_SFLOAT)]
    pub tex_uv: [f32; 2],
}

const _: () = {
    let _ = std::mem::size_of::<Vertex>();
};
