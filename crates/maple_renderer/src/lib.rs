//! maples renderer which wraps over [`wgpu`]
//!
//! implements the render graph [`render_graph::graph::RenderGraph`], assets like
//! [`core::texture::Texture`], and other conveniences like typed buffers with [`core::buffer::Buffer`]

pub mod core;
pub mod platform;
pub mod render_graph;
pub mod shader_asset;
pub mod texture_asset;
pub mod types;

pub mod prelude {
    pub use crate::core::texture::Texture;
}
