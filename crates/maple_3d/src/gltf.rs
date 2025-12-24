use std::{collections::HashMap, path::Path};

use glam::{Quat, Vec3};
use gltf::{Document, buffer::Data, image as gltf_image};
use maple_engine::{
    Scene,
    nodes::{Buildable, Builder, Empty, Node},
};
use maple_renderer::{
    core::texture::{LazyTexture, TextureCreateInfo, TextureFormat, TextureUsage},
    types::Vertex,
};

use crate::{
    components::material::{AlphaMode, MaterialProperties},
    nodes::mesh::{Mesh3D, Mesh3DBuilder},
};

pub trait GLTFLoader {
    fn load_gltf(file: impl AsRef<Path>) -> Scene;
}

impl GLTFLoader for Scene {
    fn load_gltf(file: impl AsRef<Path>) -> Scene {
        let gltf = gltf::import(file).expect("failed to open GLTF file");

        println!(
            "gltf file declared these unsupported extensions: {:?}",
            gltf.0.extensions_used()
        );
        println!(
            "gltf file requires these unsupported extensions: {:?}",
            gltf.0.extensions_required()
        );

        build_model(gltf)
    }
}

fn build_model(gltf: (Document, Vec<Data>, Vec<gltf_image::Data>)) -> Scene {
    let (doc, buffers, images) = gltf;

    let mut scene = Scene::default();
    let mut texture_cache: HashMap<usize, LazyTexture> = HashMap::new();

    let gltf_scene = doc
        .default_scene()
        .or_else(|| doc.scenes().next())
        .expect("gltf has no scene");

    // Process root nodes (nodes with no parent)
    for node in gltf_scene.nodes() {
        process_node(&node, &mut scene, &buffers, &images, &mut texture_cache);
    }

    scene
}

/// Recursively process a gltf node and its children
fn process_node(
    node: &gltf::Node,
    parent_scene: &mut Scene,
    buffers: &[Data],
    images: &[gltf_image::Data],
    texture_cache: &mut HashMap<usize, LazyTexture>,
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

    let empty_ref = parent_scene.add(node_name, empty_node);

    // If this node has a mesh, create Mesh3D nodes for each primitive
    if let Some(mesh) = node.mesh() {
        for (i, primitive) in mesh.primitives().enumerate() {
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

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

            let indices: Vec<u32> = reader
                .read_indices()
                .map_or_else(Vec::new, |iter| iter.into_u32().collect());

            // Build vertices
            let mut vertices: Vec<Vertex> = if !tangents.is_empty() {
                positions
                    .into_iter()
                    .enumerate()
                    .map(|(i, pos)| {
                        let tangent_vec3: Vec3 =
                            [tangents[i][0], tangents[i][1], tangents[i][2]].into();
                        let handedness = tangents[i][3];
                        let normal: Vec3 = normals[i].into();

                        let bitangent = normal.cross(tangent_vec3) * handedness;
                        Vertex {
                            position: pos,
                            normal: normal.into(),
                            tex_uv: tex_coords[i],
                            tangent: tangent_vec3.into(),
                            bitangent: bitangent.into(),
                        }
                    })
                    .collect()
            } else {
                // No tangents provided, calculate them
                positions
                    .into_iter()
                    .enumerate()
                    .map(|(i, pos)| Vertex {
                        position: pos,
                        normal: normals[i],
                        tex_uv: tex_coords[i],
                        tangent: [0.0, 0.0, 0.0],
                        bitangent: [0.0, 0.0, 0.0],
                    })
                    .collect()
            };

            // Calculate tangents if not provided
            if tangents.is_empty() {
                Mesh3D::calculate_tangents(&mut vertices, &indices);
            }

            // Load textures
            let base_color_texture = load_texture(
                &primitive,
                |m| {
                    m.pbr_metallic_roughness()
                        .base_color_texture()
                        .map(|t| t.texture().source().index())
                },
                texture_cache,
                images,
            );

            let metallic_roughness_texture = load_texture(
                &primitive,
                |m| {
                    m.pbr_metallic_roughness()
                        .metallic_roughness_texture()
                        .map(|t| t.texture().source().index())
                },
                texture_cache,
                images,
            );

            let normal_texture = load_texture(
                &primitive,
                |m| m.normal_texture().map(|t| t.texture().source().index()),
                texture_cache,
                images,
            );

            let occlusion_texture = load_texture(
                &primitive,
                |m| m.occlusion_texture().map(|f| f.texture().source().index()),
                texture_cache,
                images,
            );

            let emissive_texture = load_texture(
                &primitive,
                |m| m.emissive_texture().map(|t| t.texture().source().index()),
                texture_cache,
                images,
            );

            // Build material
            let gltf_alpha_mode = match primitive.material().alpha_mode() {
                gltf::material::AlphaMode::Opaque => AlphaMode::Opaque,
                gltf::material::AlphaMode::Mask => AlphaMode::Mask,
                gltf::material::AlphaMode::Blend => AlphaMode::Blend,
            };

            let mut material = MaterialProperties::default()
                .with_base_color_factor(
                    primitive
                        .material()
                        .pbr_metallic_roughness()
                        .base_color_factor(),
                )
                .with_metallic_factor(
                    primitive
                        .material()
                        .pbr_metallic_roughness()
                        .metallic_factor(),
                )
                .with_roughness_factor(
                    primitive
                        .material()
                        .pbr_metallic_roughness()
                        .roughness_factor(),
                )
                .with_emissive_factor(Vec3::from_slice(
                    primitive.material().emissive_factor().as_slice(),
                ))
                .with_double_sided(primitive.material().double_sided())
                .with_alpha_mode(gltf_alpha_mode)
                .with_alpha_cutoff(primitive.material().alpha_cutoff().unwrap_or(0.5));

            if let Some(normal_scale) = primitive.material().normal_texture() {
                material = material.with_normal_scale(normal_scale.scale());
            }

            if let Some(ao_strength) = primitive.material().occlusion_texture() {
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

            // Create Mesh3D with material using builder pattern
            let mesh_3d = Mesh3DBuilder::new(vertices, indices)
                .material(material)
                .build();

            let primitive_name = format!("primitive_{}", i);
            empty_ref.get_children_mut().add(&primitive_name, mesh_3d);
        }
    }

    // Recursively process children
    for child_node in node.children() {
        process_node(
            &child_node,
            empty_ref.get_children_mut(),
            buffers,
            images,
            texture_cache,
        );
    }
}

fn load_texture<'a>(
    primitive: &gltf::Primitive<'a>,
    index_fn: impl Fn(&gltf::Material<'a>) -> Option<usize>,
    texture_cache: &mut HashMap<usize, LazyTexture>,
    images: &[gltf_image::Data],
) -> Option<LazyTexture> {
    if let Some(image_index) = index_fn(&primitive.material()) {
        let lazy_texture = texture_cache
            .entry(image_index)
            .or_insert_with(|| {
                let image = &images[image_index];

                let format = match image.format {
                    gltf::image::Format::R8 => TextureFormat::R8,
                    gltf::image::Format::R8G8 => TextureFormat::RG8,
                    gltf::image::Format::R8G8B8 => TextureFormat::RGB8,
                    gltf::image::Format::R8G8B8A8 => TextureFormat::RGBA8,
                    gltf::image::Format::R16 => TextureFormat::R16,
                    gltf::image::Format::R16G16 => TextureFormat::RG16,
                    gltf::image::Format::R16G16B16 => TextureFormat::RGB16,
                    gltf::image::Format::R16G16B16A16 => TextureFormat::RGBA16,
                    gltf::image::Format::R32G32B32FLOAT => TextureFormat::RGB16,
                    gltf::image::Format::R32G32B32A32FLOAT => TextureFormat::RGBA32Float,
                };

                LazyTexture::new(
                    image.pixels.clone(),
                    TextureCreateInfo {
                        label: None,
                        width: image.width,
                        height: image.height,
                        format,
                        usage: TextureUsage::TEXTURE_BINDING | TextureUsage::COPY_DST,
                        sample_count: 1,
                    },
                )
            })
            .clone();
        return Some(lazy_texture);
    }
    None
}
