use std::{collections::HashMap, path::Path};

use glam::{Quat, Vec3, Vec4};
use gltf::{Document, buffer::Data, image as gltf_image};
use maple_engine::{
    Scene,
    asset::{Asset, AssetHandle, AssetLibrary, AssetLoader, FileLoader, LoadErr},
    nodes::{Buildable, Builder, Empty},
    scene::{InstancableScene, InstanceId, NodeId, SceneAsset},
};
use maple_renderer::core::{
    RenderDevice, RenderQueue,
    mipmap_generator::MipmapGenerator,
    texture::{Texture, TextureCreateInfo, TextureFormat, TextureUsage},
};

use crate::{
    assets::{
        material::AlphaMode,
        materials::pbr_material::PbrMaterial,
        mesh::{Mesh3D, Mesh3DLoader},
    },
    math::Vertex,
    nodes::mesh_instance::MeshInstance3D,
    prelude::Material,
};

/// Conversion result from specular-glossiness to metallic-roughness
struct ConvertedMaterial {
    base_color_factor: Vec4,
    metallic_factor: f32,
    roughness_factor: f32,
}

/// Unique identifier for a mesh primitive in the GLTF document
#[derive(Hash, Eq, PartialEq, Clone, Copy)]
pub struct PrimitiveKey {
    mesh_index: usize,
    primitive_index: usize,
}

pub struct GltfScene {
    /// Preprocessed meshes
    preprocessed_meshes: HashMap<PrimitiveKey, AssetHandle<Mesh3D>>,
    /// gltf textures
    texture_handles: HashMap<usize, AssetHandle<Texture>>,
    /// preprocessed materials
    material_handles: HashMap<usize, AssetHandle<Material>>,
    material_names: HashMap<String, usize>,

    scene: InstancableScene,
}

impl Asset for GltfScene {
    type Loader = GltfSceneLoader;
}

impl GltfScene {
    pub fn get_mesh(&self, key: PrimitiveKey) -> Option<AssetHandle<Mesh3D>> {
        self.preprocessed_meshes.get(&key).cloned()
    }

    pub fn get_texture(&self, key: usize) -> Option<AssetHandle<Texture>> {
        self.texture_handles.get(&key).cloned()
    }

    pub fn get_material(&self, key: usize) -> Option<AssetHandle<Material>> {
        self.material_handles.get(&key).cloned()
    }

    pub fn get_material_by_name(&self, name: &str) -> Option<AssetHandle<Material>> {
        let Some(id) = self.material_names.get(name) else {
            return None;
        };

        self.material_handles.get(id).cloned()
    }
}

pub struct GltfSceneLoader {
    pub(crate) device: RenderDevice,
    pub(crate) queue: RenderQueue,
    pub(crate) mipmap_generator: MipmapGenerator,
}

impl GltfSceneLoader {
    pub fn new(
        device: RenderDevice,
        queue: RenderQueue,
        mipmap_generator: MipmapGenerator,
    ) -> Self {
        Self {
            device,
            queue,
            mipmap_generator,
        }
    }
}

impl AssetLoader for GltfSceneLoader {
    type Asset = GltfScene;
}

impl FileLoader for GltfSceneLoader {
    fn load_path(&self, path: &Path, library: &AssetLibrary) -> Result<Self::Asset, LoadErr> {
        log::info!("Loading GLTF from {:?}", path);
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
        let preprocessed_meshes = preprocess_meshes(&library, self, &document, &buffers);

        // Preload and register all textures as assets
        log::debug!("Preloading textures");
        let texture_handles = preload_textures(
            &self.device,
            &self.queue,
            &self.mipmap_generator,
            &images,
            library,
        );
        log::debug!("Textures preloaded: {}", texture_handles.len());

        log::debug!("Preloading Materials");
        let (material_handles, material_names) =
            preprocess_materials(&library, &texture_handles, &document);
        log::debug!("materials preloaded: {}", material_handles.len());

        log::info!("Finished loading GLTF from {:?}", path);

        let scene = InstancableScene::new();

        // Load all scenes from the GLTF (usually just one)
        for gltf_scene in document.scenes() {
            for node in gltf_scene.nodes() {
                process_node(
                    self,
                    &node,
                    &scene,
                    None,
                    &texture_handles,
                    &material_handles,
                    &preprocessed_meshes,
                );
            }
        }

        Ok(GltfScene {
            preprocessed_meshes,
            texture_handles,
            material_handles,
            scene,
            material_names,
        })
    }
}

fn preload_textures(
    device: &RenderDevice,
    queue: &RenderQueue,
    mipmap_generator: &MipmapGenerator,
    images: &[gltf_image::Data],
    assets: &AssetLibrary,
) -> HashMap<usize, AssetHandle<Texture>> {
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

        let texture = device.create_texture(TextureCreateInfo {
            label: None,
            width: image.width,
            height: image.height,
            format,
            usage: TextureUsage::TEXTURE_BINDING | TextureUsage::COPY_DST,
            sample_count: 1,
            mip_level,
        });
        queue.write_texture(&texture, &pixels);
        mipmap_generator.generate_mipmaps(&texture);

        let handle = assets.register(texture);

        texture_handles.insert(image_index, handle);
    }

    texture_handles
}

/// Preprocess all meshes in the GLTF document
/// This does all the expensive computation upfront during file loading:
/// - Read vertex data from buffers
/// - Compute tangents and bitangents if not present
/// - Compute AABB bounding boxes
fn preprocess_meshes(
    assets: &AssetLibrary,
    loader: &GltfSceneLoader,
    document: &Document,
    buffers: &[Data],
) -> HashMap<PrimitiveKey, AssetHandle<Mesh3D>> {
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
                Mesh3DLoader::calculate_tangents(&mut vertices, &indices);
            }

            preprocessed.insert(
                key,
                assets.add(Mesh3D::new(&loader.device, &vertices, &indices)),
            );
        }
    }

    preprocessed
}

fn preprocess_materials(
    assets: &AssetLibrary,
    texture_handles: &HashMap<usize, AssetHandle<Texture>>,
    document: &Document,
) -> (
    HashMap<usize, AssetHandle<Material>>,
    HashMap<String, usize>,
) {
    let mut materials = HashMap::new();
    let mut material_names = HashMap::new();
    for material_model in document.materials() {
        let Some(material_idx) = material_model.index() else {
            continue;
        };

        if let Some(name) = material_model.name() {
            material_names.insert(name.to_string(), material_idx);
        }

        materials.insert(
            material_idx,
            build_material(assets, &material_model, texture_handles),
        );
    }
    (materials, material_names)
}

impl SceneAsset for GltfScene {
    fn load(&self, scene: &Scene, parent: Option<NodeId>) {
        match parent {
            Some(node) => scene.merge_as_child(self.scene.instance(), node),
            None => scene.merge(self.scene.instance()),
        };
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
    loader: &GltfSceneLoader,
    node: &gltf::Node,
    scene: &InstancableScene,
    parent: Option<InstanceId>,
    texture_handles: &HashMap<usize, AssetHandle<Texture>>,
    material_handles: &HashMap<usize, AssetHandle<Material>>,
    preprocessed_meshes: &HashMap<PrimitiveKey, AssetHandle<Mesh3D>>,
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

            // Get preprocessed mesh data
            let mesh_3d = preprocessed_meshes
                .get(&key)
                .expect("Mesh should have been preprocessed during load");

            // Check material
            let material_model = primitive.material();
            let Some(material_index) = material_model.index() else {
                continue;
            };

            let material = material_handles
                .get(&material_index)
                .expect("material should have been preloaded");

            let mesh_instance = MeshInstance3D::builder()
                .mesh(mesh_3d.clone())
                .material(material.clone())
                .build();

            let primitive_name = format!("primitive_{}", primitive_index);
            // Add mesh as child of the empty node
            scene.spawn_as_child(&primitive_name, mesh_instance, empty_handle);
        }
    }

    // Recursively process children - pass this node's ID as parent
    for child_node in node.children() {
        process_node(
            loader,
            &child_node,
            scene,
            Some(empty_handle),
            texture_handles,
            material_handles,
            preprocessed_meshes,
        );
    }
}
/// Build a material from a GLTF material
fn build_material<'a>(
    assets: &AssetLibrary,
    material_model: &gltf::Material<'a>,
    texture_handles: &HashMap<usize, AssetHandle<Texture>>,
) -> AssetHandle<Material> {
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
            material_model,
            |m| {
                m.pbr_specular_glossiness()
                    .and_then(|sg| sg.diffuse_texture())
                    .map(|t| t.texture().source().index())
            },
            texture_handles,
        );

        // Load specular-glossiness texture
        let metallic_roughness_tex = load_texture(
            material_model,
            |m| {
                m.pbr_specular_glossiness()
                    .and_then(|sg| sg.specular_glossiness_texture())
                    .map(|t| t.texture().source().index())
            },
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
            material_model,
            |m| {
                m.pbr_metallic_roughness()
                    .base_color_texture()
                    .map(|t| t.texture().source().index())
            },
            texture_handles,
        );

        let metallic_roughness_tex = load_texture(
            material_model,
            |m| {
                m.pbr_metallic_roughness()
                    .metallic_roughness_texture()
                    .map(|t| t.texture().source().index())
            },
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
        material_model,
        |m| m.normal_texture().map(|t| t.texture().source().index()),
        texture_handles,
    );

    let occlusion_texture = load_texture(
        material_model,
        |m| m.occlusion_texture().map(|f| f.texture().source().index()),
        texture_handles,
    );

    let emissive_texture = load_texture(
        material_model,
        |m| m.emissive_texture().map(|t| t.texture().source().index()),
        texture_handles,
    );

    // Build material
    let gltf_alpha_mode = match material_model.alpha_mode() {
        gltf::material::AlphaMode::Opaque => AlphaMode::Opaque,
        gltf::material::AlphaMode::Mask => AlphaMode::Mask,
        gltf::material::AlphaMode::Blend => AlphaMode::Blend,
    };

    // Check for unlit extension
    let _is_unlit = material_model.unlit();

    let mut material = PbrMaterial {
        base_color_factor: base_color_factor.into(),
        base_color_texture,
        metallic_factor,
        roughness_factor,
        metallic_roughness_texture,
        emissive_factor: {
            let emissive = Vec3::from_slice(material_model.emissive_factor().as_slice());
            let strength = material_model.emissive_strength().unwrap_or(1.0);
            (emissive * strength).into()
        },
        emissive_texture,
        normal_texture,
        occlusion_texture,
        double_sided: material_model.double_sided(),
        alpha_mode: gltf_alpha_mode,
        alpha_cutoff: material_model.alpha_cutoff().unwrap_or(0.5),
        ..Default::default()
    };

    if let Some(normal_scale) = material_model.normal_texture() {
        material.normal_scale = normal_scale.scale();
    }

    if let Some(ao_strength) = material_model.occlusion_texture() {
        material.ambient_occlusion_strength = ao_strength.strength();
    }

    assets.add(material)
}

fn load_texture<'a>(
    material_model: &gltf::Material<'a>,
    index_fn: impl Fn(&gltf::Material<'a>) -> Option<usize>,
    texture_handles: &HashMap<usize, AssetHandle<Texture>>,
) -> Option<AssetHandle<Texture>> {
    if let Some(image_index) = index_fn(material_model) {
        // Check cache first, otherwise get from preloaded handles
        let handle = texture_handles
            .get(&image_index)
            .expect("texture should be preloaded");
        return Some(handle.clone());
    }
    None
}
