use std::path::Path;

use image::ImageError;
use maple_engine::asset::{Asset, AssetLibrary, AssetLoader, FileLoader, LoadErr};

use crate::core::{RenderDevice, RenderQueue, texture::Texture};

/// Texture asset that can be loaded through the asset system
/// Supports HDR, EXR, PNG, JPG, and other image formats
impl Asset for Texture {
    type Loader = TextureAssetLoader;
}

/// Loader for texture assets
/// Automatically detects HDR formats (.hdr, .exr) and loads them appropriately
pub struct TextureAssetLoader {
    device: RenderDevice,
    queue: RenderQueue,
}

impl TextureAssetLoader {
    pub fn new(device: RenderDevice, queue: RenderQueue) -> Self {
        Self { device, queue }
    }
}

impl AssetLoader for TextureAssetLoader {
    type Asset = Texture;
}

impl FileLoader for TextureAssetLoader {
    fn load_path(&self, path: &Path, _library: &AssetLibrary) -> Result<Self::Asset, LoadErr> {
        // Check file extension to determine if it's HDR
        let extension = path
            .extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_lowercase());

        let texture = match extension.as_deref() {
            Some("hdr") | Some("exr") => {
                log::info!("loading hdri texture from: {:?}", path);
                // Load as HDR texture with RGBA32Float format
                let texture =
                    Texture::new_hdri_from_file(&self.device.device, &self.queue.queue, path, None)
                        .map_err(|e: ImageError| {
                            LoadErr::Import(format!("Failed to load HDR texture: {}", e))
                        })?;
                log::info!("Finished loading hdri Texture: {:?}", path);
                texture
            }
            Some("png") | Some("jpg") | Some("jpeg") | Some("bmp") | Some("tga") | Some("webp") => {
                log::info!("loading texture from: {:?}", path);
                // Load as standard texture
                let texture =
                    Texture::from_file(&self.device.device, &self.queue.queue, path, None)
                        .map_err(|e: ImageError| {
                            LoadErr::Import(format!("Failed to load texture: {}", e))
                        })?;
                log::info!("Finished loading Texture: {:?}", path);
                texture
            }
            _ => {
                return Err(LoadErr::Import(format!(
                    "Unsupported texture format: {:?}",
                    extension
                )));
            }
        };

        Ok(texture)
    }
}
