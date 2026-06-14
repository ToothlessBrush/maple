//! Materials are assets that store shaders and describe inputs to a shader

// ```rust
//
// ```

use std::any::Any;

use maple_engine::asset::{Asset, AssetHandle};

pub trait Material: Any {}

pub struct MaterialAsset {
    material: dyn Material,
}
