use std::{collections::HashMap, path::Path, sync::Arc};

use glam::{Quat, Vec3, Vec4};
use gltf::{Document, buffer::Data, image as gltf_image};
use maple_engine::{
    Scene,
    asset::{Asset, AssetHandle, AssetLibrary, AssetLoader, LoadErr},
    nodes::{Buildable, Builder, Empty},
    scene::{NodeId, SceneAsset},
};
use maple_renderer::{
    core::{
        LazyBuffer, RenderContext,
        texture::{LazyTexture, TextureCreateInfo, TextureFormat, TextureUsage},
    },
    types::Vertex,
};

use crate::{
    components::material::{AlphaMode, MaterialProperties},
    math::AABB,
    nodes::mesh::Mesh3D,
};

/// Conversion result from specular-glossiness to metallic-roughness
struct ConvertedMaterial {
    base_color_factor: Vec4,
    metallic_factor: f32,
    roughness_factor: f32,
}

/// Preprocessed mesh data with all computations done during load
#[derive(Clone)]
struct PreprocessedMesh {
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
    aabb: AABB,
    #[allow(dead_code)]
    // Stored for completeness, accessed via primitive.material() during scene load
    material_index: Option<usize>,
}

type VertexAndBounds = HashMap<(usize, usize), (AABB, LazyBuffer<[Vertex]>)>;

/// Cache for GLTF resources to avoid duplicate GPU allocations
struct GltfCache {
    textures: HashMap<usize, AssetHandle<LazyTexture>>,
    vertex_buffers: VertexAndBounds,
    index_buffers: HashMap<(usize, usize), LazyBuffer<[u32]>>,
    materials: HashMap<usize, MaterialProperties>,
}

impl GltfCache {
    fn new() -> Self {
        Self {
            textures: HashMap::new(),
            vertex_buffers: HashMap::new(),
            index_buffers: HashMap::new(),
            materials: HashMap::new(),
        }
    }
}

/// Unique identifier for a mesh primitive in the GLTF document
#[derive(Hash, Eq, PartialEq, Clone, Copy)]
struct PrimitiveKey {
    mesh_index: usize,
    primitive_index: usize,
}

pub struct GltfScene {
    document: Document,
    /// Preprocessed meshes stored by (mesh_index, primitive_index)
    preprocessed_meshes: HashMap<PrimitiveKey, PreprocessedMesh>,
    /// Cached texture handles registered during load
    texture_handles: HashMap<usize, AssetHandle<LazyTexture>>,
}

impl Asset for GltfScene {
    type Loader = GltfSceneLoader;
}

pub struct GltfSceneLoader;

impl AssetLoader for GltfSceneLoader {
    type Asset = GltfScene;

    fn load(&self, path: &Path, library: &AssetLibrary) -> Result<Arc<Self::Asset>, LoadErr> {
        log::debug!("Loading GLTF from {:?}", path);
        // gltf::import loads document, buffers, and images all at once
        let import_result = gltf::import(path);
        log::debug!("gltf::import returned: {:?}", import_result.is_ok());
        let (document, buffers, images) = import_result.map_err(|e| {
            log::error!("gltf::import failed: {}", e);
            LoadErr::Import(format!("Failed to load GLTF: {}", e))
        })?;

        log::debug!("GLTF import successful, {} images found", images.len());

        // List of extensions we support
        const SUPPORTED_EXTENSIONS: &[&str] = &[
            "KHR_materials_unlit",
            "KHR_materials_pbrSpecularGlossiness",
            "KHR_materials_emissive_strength",
        ];

        // Filter out supported extensions from the used extensions list
        let used_extensions: Vec<&str> = document.extensions_used().collect();
        let unsupported_used: Vec<&str> = used_extensions
            .iter()
            .copied()
            .filter(|ext| !SUPPORTED_EXTENSIONS.contains(ext))
            .collect();

        if !unsupported_used.is_empty() {
            log::debug!(
                "GLTF file uses these unsupported extensions: {:?}",
                unsupported_used
            );
        }

        // Filter out supported extensions from the required extensions list
        let required_extensions: Vec<&str> = document.extensions_required().collect();
        let unsupported_required: Vec<&str> = required_extensions
            .iter()
            .copied()
            .filter(|ext| !SUPPORTED_EXTENSIONS.contains(ext))
            .collect();

        if !unsupported_required.is_empty() {
            return Err(LoadErr::Import(format!(
                "GLTF file requires these unsupported extensions: {:?}",
                unsupported_required
            )));
        }

        // Preprocess all meshes - compute tangents, bitangents, AABB during load
        log::debug!("Preprocessing meshes");
        let preprocessed_meshes = preprocess_all_meshes(&document, &buffers);

        // Preload and register all textures as assets
        log::debug!("Preloading textures");
        let texture_handles = preload_textures(&images, library);
        log::debug!("Textures preloaded: {}", texture_handles.len());

        log::debug!("GLTF loading complete");
        Ok(Arc::new(GltfScene {
            document,
            preprocessed_meshes,
            texture_handles,
        }))
    }
}

fn preload_textures(
    images: &[gltf_image::Data],
    assets: &AssetLibrary,
) -> HashMap<usize, AssetHandle<LazyTexture>> {
    let mut texture_handles = HashMap::new();

    for (image_index, image) in images.iter().enumerate() {
        let (pixels, format) = match image.format {
            gltf::image::Format::R8 => {
                // R8 (grayscale) -> RGBA8: R -> (R, R, R, 255)
                let expanded: Vec<u8> = image.pixels.iter().flat_map(|&r| [r, r, r, 255]).collect();
                (expanded, TextureFormat::RGBA8)
            }
            gltf::image::Format::R8G8 => {
                // RG8 (grayscale+alpha) -> RGBA8: (L, A) -> (L, L, L, A)
                let expanded: Vec<u8> = image
                    .pixels
                    .chunks(2)
                    .flat_map(|la| [la[0], la[0], la[0], la[1]])
                    .collect();
                (expanded, TextureFormat::RGBA8)
            }
            gltf::image::Format::R16 => {
                // R16 -> RGBA16
                let pixels_u16: &[u16] = bytemuck::cast_slice(&image.pixels);
                let expanded: Vec<u16> =
                    pixels_u16.iter().flat_map(|&r| [r, r, r, 65535]).collect();
                (
                    bytemuck::cast_slice(&expanded).to_vec(),
                    TextureFormat::RGBA16,
                )
            }
            gltf::image::Format::R16G16 => {
                // RG16 -> RGBA16
                let pixels_u16: &[u16] = bytemuck::cast_slice(&image.pixels);
                let expanded: Vec<u16> = pixels_u16
                    .chunks(2)
                    .flat_map(|la| [la[0], la[0], la[0], la[1]])
                    .collect();
                (
                    bytemuck::cast_slice(&expanded).to_vec(),
                    TextureFormat::RGBA16,
                )
            }
            gltf::image::Format::R8G8B8 => (image.pixels.clone(), TextureFormat::RGB8),
            gltf::image::Format::R8G8B8A8 => (image.pixels.clone(), TextureFormat::RGBA8),
            gltf::image::Format::R16G16B16 => (image.pixels.clone(), TextureFormat::RGB16),
            gltf::image::Format::R16G16B16A16 => (image.pixels.clone(), TextureFormat::RGBA16),
            gltf::image::Format::R32G32B32FLOAT => (image.pixels.clone(), TextureFormat::RGB16),
            gltf::image::Format::R32G32B32A32FLOAT => {
                (image.pixels.clone(), TextureFormat::RGBA32Float)
            }
        };

        // Only request mipmaps if the format supports compute-based generation
        let supports_mipmaps = matches!(
            format,
            TextureFormat::RGBA8
                | TextureFormat::RGBA16Float
                | TextureFormat::RGBA32Float
                | TextureFormat::RGB8
                | TextureFormat::RGB16
        );

        let mip_level = if supports_mipmaps {
            let max_dimension = image.width.max(image.height) as f32;
            (max_dimension.log2().floor() as u32 + 1).min(10)
        } else {
            1 // No mipmaps for unsupported formats
        };

        let lazy_texture = LazyTexture::new(
            pixels,
            TextureCreateInfo {
                label: None,
                width: image.width,
                height: image.height,
                format,
                usage: TextureUsage::TEXTURE_BINDING | TextureUsage::COPY_DST,
                sample_count: 1,
                mip_level,
            },
        );

        let handle = assets.register(lazy_texture);
        texture_handles.insert(image_index, handle);
    }

    texture_handles
}

/// Preprocess all meshes in the GLTF document
/// This does all the expensive computation upfront during file loading:
/// - Read vertex data from buffers
/// - Compute tangents and bitangents if not present
/// - Compute AABB bounding boxes
fn preprocess_all_meshes(
    document: &Document,
    buffers: &[Data],
) -> HashMap<PrimitiveKey, PreprocessedMesh> {
    let mut preprocessed = HashMap::new();

    for mesh in document.meshes() {
        let mesh_index = mesh.index();

        for (primitive_index, primitive) in mesh.primitives().enumerate() {
            let key = PrimitiveKey {
                mesh_index,
                primitive_index,
            };

            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

            // Read vertex data
            let positions: Vec<[f32; 3]> = reader
                .read_positions()
                .map_or_else(Vec::new, |iter| iter.collect());

            let normals: Vec<[f32; 3]> = reader.read_normals().map_or_else(
                || vec![[0.0, 0.0, 1.0]; positions.len()],
                |iter| iter.collect(),
            );

            let tex_coords: Vec<[f32; 2]> = reader.read_tex_coords(0).map_or_else(
                || vec![[0.0, 0.0]; positions.len()],
                |coords| coords.into_f32().collect(),
            );

            let tangents: Vec<[f32; 4]> = reader
                .read_tangents()
                .map_or_else(Vec::new, |iter| iter.collect());

            // Build vertices with tangents/bitangents
            let mut vertices: Vec<Vertex> = if !tangents.is_empty() {
                positions
                    .into_iter()
                    .enumerate()
                    .map(|(j, pos)| {
                        let tangent_vec3: Vec3 =
                            [tangents[j][0], tangents[j][1], tangents[j][2]].into();
                        let handedness = tangents[j][3];
                        let normal: Vec3 = normals[j].into();

                        let bitangent = normal.cross(tangent_vec3) * handedness;
                        Vertex {
                            position: pos,
                            normal: normal.into(),
                            tex_uv: tex_coords[j],
                            tangent: tangent_vec3.into(),
                            bitangent: bitangent.into(),
                        }
                    })
                    .collect()
            } else {
                positions
                    .into_iter()
                    .enumerate()
                    .map(|(j, pos)| Vertex {
                        position: pos,
                        normal: normals[j],
                        tex_uv: tex_coords[j],
                        tangent: [0.0, 0.0, 0.0],
                        bitangent: [0.0, 0.0, 0.0],
                    })
                    .collect()
            };

            // Read indices
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
            let indices: Vec<u32> = reader
                .read_indices()
                .map_or_else(Vec::new, |iter| iter.into_u32().collect());

            // Calculate tangents if not provided
            if tangents.is_empty() {
                Mesh3D::calculate_tangents(&mut vertices, &indices);
            }

            // Calculate AABB
            let aabb = AABB::from_vertices(&vertices);

            // Get material index
            let material_index = primitive.material().index();

            preprocessed.insert(
                key,
                PreprocessedMesh {
                    vertices,
                    indices,
                    aabb,
                    material_index,
                },
            );
        }
    }

    preprocessed
}

impl SceneAsset for GltfScene {
    fn load(&self, scene: &Scene, parent: Option<NodeId>) {
        let mut cache = GltfCache::new();

        // Load all scenes from the GLTF (usually just one)
        for gltf_scene in self.document.scenes() {
            for node in gltf_scene.nodes() {
                process_node(
                    &node,
                    scene,
                    parent,
                    &self.texture_handles,
                    &self.preprocessed_meshes,
                    &mut cache,
                );
            }
        }
    }
}

/// Convert specular-glossiness workflow to metallic-roughness workflow
/// Based on the Khronos reference implementation
fn convert_specular_glossiness_to_metallic_roughness(
    diffuse_factor: Vec4,
    specular_factor: Vec3,
    glossiness_factor: f32,
) -> ConvertedMaterial {
    // Constants for the conversion
    const EPSILON: f32 = 1e-6;
    const DIELECTRIC_SPECULAR: f32 = 0.04;

    // Convert glossiness to roughness
    let roughness_factor = 1.0 - glossiness_factor;

    // Calculate perceived brightness of diffuse and specular
    let diffuse_perceived = perceive_brightness(Vec3::new(
        diffuse_factor.x,
        diffuse_factor.y,
        diffuse_factor.z,
    ));
    let specular_perceived = perceive_brightness(specular_factor);

    // Solve for metallic factor
    let metallic_factor = if specular_perceived < DIELECTRIC_SPECULAR {
        0.0
    } else {
        let a = DIELECTRIC_SPECULAR;
        let b = diffuse_perceived * (1.0 - DIELECTRIC_SPECULAR) / (1.0 - a) + specular_perceived
            - 2.0 * DIELECTRIC_SPECULAR;
        let c = DIELECTRIC_SPECULAR - specular_perceived;
        let d = (b * b - 4.0 * a * c).max(0.0);
        (-b + d.sqrt()) / (2.0 * a).max(EPSILON)
    };

    let metallic_factor = metallic_factor.clamp(0.0, 1.0);

    // Compute base color from diffuse and specular
    let base_color_from_diffuse = diffuse_factor
        * (1.0 - DIELECTRIC_SPECULAR)
        * (1.0 / (1.0 - DIELECTRIC_SPECULAR).max(EPSILON));
    let base_color_from_specular =
        Vec4::new(specular_factor.x, specular_factor.y, specular_factor.z, 1.0)
            - Vec4::splat(DIELECTRIC_SPECULAR * (1.0 - metallic_factor))
                * (1.0 / (1.0 - DIELECTRIC_SPECULAR).max(EPSILON));
    let base_color_from_specular = base_color_from_specular * (1.0 / metallic_factor.max(EPSILON));

    // Lerp between the two based on metallic factor
    let base_color_factor = base_color_from_diffuse * (1.0 - metallic_factor)
        + base_color_from_specular * metallic_factor;

    ConvertedMaterial {
        base_color_factor: Vec4::new(
            base_color_factor.x,
            base_color_factor.y,
            base_color_factor.z,
            diffuse_factor.w, // Preserve alpha from diffuse
        ),
        metallic_factor,
        roughness_factor,
    }
}

/// Calculate perceived brightness using luminance formula
fn perceive_brightness(color: Vec3) -> f32 {
    // Standard luminance coefficients (Rec. 709)
    0.299 * color.x + 0.587 * color.y + 0.114 * color.z
}

/// Recursively process a gltf node and its children
fn process_node(
    node: &gltf::Node,
    scene: &Scene,
    parent: Option<NodeId>,
    texture_handles: &HashMap<usize, AssetHandle<LazyTexture>>,
    preprocessed_meshes: &HashMap<PrimitiveKey, PreprocessedMesh>,
    cache: &mut GltfCache,
) {
    let (translation, rotation, scale) = node.transform().decomposed();

    let translation: Vec3 = translation.into();
    let rotation: Quat = Quat::from_array(rotation);
    let scale: Vec3 = scale.into();

    let node_name = node.name().unwrap_or("unnamed_node");

    // Create an Empty node for this gltf node to hold the transform
    let empty_node = Empty::builder()
        .position(translation)
        .rotation(rotation)
        .scale(scale)
        .build();

    // Add to scene with parent
    let empty_handle = match parent {
        Some(parent_id) => scene.spawn_as_child(node_name, empty_node, parent_id),
        None => scene.spawn(node_name, empty_node),
    };

    // If this node has a mesh, create Mesh3D nodes for each primitive
    if let Some(mesh) = node.mesh() {
        let mesh_index = mesh.index();

        for (primitive_index, primitive) in mesh.primitives().enumerate() {
            let key = PrimitiveKey {
                mesh_index,
                primitive_index,
            };

            // Get preprocessed mesh data (already has tangents, bitangents, AABB computed)
            let preprocessed = preprocessed_meshes
                .get(&key)
                .expect("Mesh should have been preprocessed during load");

            // Check if we have cached vertex buffer for this mesh
            let (aabb, vertex_buffer) = if let Some(cached_buffer) =
                cache.vertex_buffers.get(&(mesh_index, primitive_index))
            {
                cached_buffer.clone()
            } else {
                let aabb = preprocessed.aabb;
                let vbuffer = RenderContext::create_vertex_buffer_lazy(&preprocessed.vertices);
                cache
                    .vertex_buffers
                    .insert((mesh_index, primitive_index), (aabb, vbuffer.clone()));
                (aabb, vbuffer)
            };

            // Check if we have cached index buffer for this mesh
            let index_buffer = if let Some(cached_buffer) =
                cache.index_buffers.get(&(mesh_index, primitive_index))
            {
                cached_buffer.clone()
            } else {
                let ibuffer = RenderContext::create_index_buffer_lazy(&preprocessed.indices);
                cache
                    .index_buffers
                    .insert((mesh_index, primitive_index), ibuffer.clone());
                ibuffer
            };

            // Check material
            let material_model = primitive.material();
            let material_index = material_model.index();

            let material = if let Some(material_idx) = material_index {
                if let Some(cached_material) = cache.materials.get(&material_idx) {
                    cached_material.clone()
                } else {
                    let built_material = build_material(
                        &material_model,
                        &primitive,
                        &mut cache.textures,
                        texture_handles,
                    );
                    cache.materials.insert(material_idx, built_material.clone());
                    built_material
                }
            } else {
                build_material(
                    &material_model,
                    &primitive,
                    &mut cache.textures,
                    texture_handles,
                )
            };

            let mesh_3d = Mesh3D::from_buffers(vertex_buffer, index_buffer, material, aabb);

            let primitive_name = format!("primitive_{}", primitive_index);
            // Add mesh as child of the empty node
            empty_handle.spawn_child(&primitive_name, mesh_3d);
        }
    }

    // Recursively process children - pass this node's ID as parent
    for child_node in node.children() {
        process_node(
            &child_node,
            scene,
            Some(empty_handle.id()),
            texture_handles,
            preprocessed_meshes,
            cache,
        );
    }
}
/// Build a material from a GLTF material
fn build_material<'a>(
    material_model: &gltf::Material<'a>,
    primitive: &gltf::Primitive<'a>,
    texture_cache: &mut HashMap<usize, AssetHandle<LazyTexture>>,
    texture_handles: &HashMap<usize, AssetHandle<LazyTexture>>,
) -> MaterialProperties {
    let use_specular_glossiness = material_model.pbr_specular_glossiness().is_some();

    // Load textures and factors based on workflow
    let (
        base_color_factor,
        metallic_factor,
        roughness_factor,
        base_color_texture,
        metallic_roughness_texture,
    ) = if use_specular_glossiness {
        // SPECULAR-GLOSSINESS WORKFLOW
        let pbr_sg = material_model.pbr_specular_glossiness().unwrap();

        // Convert factors from specular-glossiness to metallic-roughness
        let diffuse_factor = Vec4::from_slice(&pbr_sg.diffuse_factor());
        let specular_factor = Vec3::from_slice(&pbr_sg.specular_factor());
        let glossiness_factor = pbr_sg.glossiness_factor();

        let converted = convert_specular_glossiness_to_metallic_roughness(
            diffuse_factor,
            specular_factor,
            glossiness_factor,
        );

        // Load diffuse texture (maps to base color)
        let base_color_tex = load_texture(
            primitive,
            |m| {
                m.pbr_specular_glossiness()
                    .and_then(|sg| sg.diffuse_texture())
                    .map(|t| t.texture().source().index())
            },
            texture_cache,
            texture_handles,
        );

        // Load specular-glossiness texture
        let metallic_roughness_tex = load_texture(
            primitive,
            |m| {
                m.pbr_specular_glossiness()
                    .and_then(|sg| sg.specular_glossiness_texture())
                    .map(|t| t.texture().source().index())
            },
            texture_cache,
            texture_handles,
        );

        (
            converted.base_color_factor,
            converted.metallic_factor,
            converted.roughness_factor,
            base_color_tex,
            metallic_roughness_tex,
        )
    } else {
        // METALLIC-ROUGHNESS WORKFLOW (default)
        let pbr_mr = material_model.pbr_metallic_roughness();

        let base_color_tex = load_texture(
            primitive,
            |m| {
                m.pbr_metallic_roughness()
                    .base_color_texture()
                    .map(|t| t.texture().source().index())
            },
            texture_cache,
            texture_handles,
        );

        let metallic_roughness_tex = load_texture(
            primitive,
            |m| {
                m.pbr_metallic_roughness()
                    .metallic_roughness_texture()
                    .map(|t| t.texture().source().index())
            },
            texture_cache,
            texture_handles,
        );

        (
            Vec4::from_slice(&pbr_mr.base_color_factor()),
            pbr_mr.metallic_factor(),
            pbr_mr.roughness_factor(),
            base_color_tex,
            metallic_roughness_tex,
        )
    };

    // Load common textures (same for both workflows)
    let normal_texture = load_texture(
        primitive,
        |m| m.normal_texture().map(|t| t.texture().source().index()),
        texture_cache,
        texture_handles,
    );

    let occlusion_texture = load_texture(
        primitive,
        |m| m.occlusion_texture().map(|f| f.texture().source().index()),
        texture_cache,
        texture_handles,
    );

    let emissive_texture = load_texture(
        primitive,
        |m| m.emissive_texture().map(|t| t.texture().source().index()),
        texture_cache,
        texture_handles,
    );

    // Build material
    let gltf_alpha_mode = match material_model.alpha_mode() {
        gltf::material::AlphaMode::Opaque => AlphaMode::Opaque,
        gltf::material::AlphaMode::Mask => AlphaMode::Mask,
        gltf::material::AlphaMode::Blend => AlphaMode::Blend,
    };

    // Check for unlit extension
    let is_unlit = material_model.unlit();

    let mut material = MaterialProperties::default()
        .with_base_color_factor(base_color_factor)
        .with_metallic_factor(metallic_factor)
        .with_roughness_factor(roughness_factor)
        .with_emissive_factor({
            let emissive = Vec3::from_slice(material_model.emissive_factor().as_slice());
            let strength = material_model.emissive_strength().unwrap_or(1.0);
            emissive * strength
        })
        .with_double_sided(material_model.double_sided())
        .with_alpha_mode(gltf_alpha_mode)
        .with_alpha_cutoff(material_model.alpha_cutoff().unwrap_or(0.5))
        .with_unlit(is_unlit);

    if let Some(normal_scale) = material_model.normal_texture() {
        material = material.with_normal_scale(normal_scale.scale());
    }

    if let Some(ao_strength) = material_model.occlusion_texture() {
        material = material.with_ambient_occlusion_strength(ao_strength.strength());
    }

    if let Some(tex) = base_color_texture {
        material = material.with_base_color_texture(tex);
    }

    if let Some(tex) = metallic_roughness_texture {
        material = material.with_metallic_roughness_texture(tex);
    }

    if let Some(tex) = normal_texture {
        material = material.with_normal_texture(tex);
    }

    if let Some(tex) = occlusion_texture {
        material = material.with_occlusion_texture(tex);
    }

    if let Some(tex) = emissive_texture {
        material = material.with_emissive_texture(tex);
    }

    material
}

fn load_texture<'a>(
    primitive: &gltf::Primitive<'a>,
    index_fn: impl Fn(&gltf::Material<'a>) -> Option<usize>,
    texture_cache: &mut HashMap<usize, AssetHandle<LazyTexture>>,
    texture_handles: &HashMap<usize, AssetHandle<LazyTexture>>,
) -> Option<AssetHandle<LazyTexture>> {
    if let Some(image_index) = index_fn(&primitive.material()) {
        // Check cache first, otherwise get from preloaded handles
        let handle = texture_cache
            .entry(image_index)
            .or_insert_with(|| {
                texture_handles
                    .get(&image_index)
                    .cloned()
                    .expect("Texture should have been preloaded")
            })
            .clone();
        return Some(handle);
    }
    None
}
