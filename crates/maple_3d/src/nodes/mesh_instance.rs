use std::marker::PhantomData;

use bytemuck::{Pod, Zeroable};
use maple_engine::{
    Buildable, Builder, Node,
    asset::{AssetHandle, AssetLibrary},
    nodes::node_builder::NodePrototype,
    prelude::NodeTransform,
};

use crate::{
    assets::mesh::Mesh3D,
    prelude::{Material, MaterialInstance, MaterialInstanceMut, MaterialInstanceRef},
};

#[derive(Debug, Default, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct Mesh3DUniformBufferData {
    pub model: [[f32; 4]; 4],
    pub normal_matrix: [[f32; 4]; 4],
}

#[allow(unused_imports, reason = "used in doc")]
use maple_engine::prelude::Scene;

/// represents a mesh in a [`Scene`]
///
/// groups asset handles for a [`Mesh3D`] and [`Material`] for the scene hierarchy for which
/// the render pipeline can fetch and render from.
///
/// instances with the same mesh and material handle IDs will be batched automatically and rendered in one draw call
///
/// # Example
/// ```no_run
/// # use maple_engine::prelude::*;
/// # use maple_3d::prelude::*;
/// # let scene = Scene::default();
/// # let assets = AssetLibrary::default();
/// scene.spawn(MeshInstance3D::builder()
///     .mesh(assets.add(Cuboid::default()))
///     .material(assets.add(PbrMaterial {
///         base_color_factor: Color::BLUE,
///         ..Default::default()
///     }))
/// );
/// ```
#[derive(Clone, Default)]
pub struct MeshInstance3D {
    /// Transform of the node
    pub transform: NodeTransform,

    /// reference to the mesh asset used by this instance
    pub mesh: Option<AssetHandle<Mesh3D>>,

    /// reference to the material this mesh uses
    ///
    /// **Meshes with no material will not be rendered**
    pub material: Option<AssetHandle<Material>>,
}

impl MeshInstance3D {
    pub fn get_uniform(&self) -> Mesh3DUniformBufferData {
        let model = self.transform.world_space().matrix.to_cols_array_2d();
        let normal_matrix = self
            .transform
            .world_space()
            .matrix
            .inverse()
            .transpose()
            .to_cols_array_2d();

        Mesh3DUniformBufferData {
            model,
            normal_matrix,
        }
    }

    pub fn get_material<T: MaterialInstance>(
        &self,
        assets: &AssetLibrary,
    ) -> Option<MaterialInstanceRef<T>> {
        let handle = self.material.as_ref()?;
        let material = assets.get(handle)?; // Option<AssetRef<Material>>

        // verify the type matches now, so callers get None instead of a later panic
        material.get_instance::<T>()?;

        Some(MaterialInstanceRef {
            material,
            _ty: PhantomData,
        })
    }

    pub fn get_material_mut<T: MaterialInstance>(
        &self,
        assets: &AssetLibrary,
    ) -> Option<MaterialInstanceMut<T>> {
        let handle = self.material.as_ref()?;
        let mut material = assets.get_mut(handle)?; // Option<AssetRef<Material>>

        // verify the type matches now, so callers get None instead of a later panic
        material.get_instance_mut::<T>()?;

        Some(MaterialInstanceMut {
            material,
            _ty: PhantomData,
        })
    }
}

impl Node for MeshInstance3D {
    fn get_transform(&mut self) -> &mut NodeTransform {
        &mut self.transform
    }
}

#[derive(Default)]
pub struct MeshInstance3DBuilder {
    prototype: NodePrototype,
    mesh: Option<AssetHandle<Mesh3D>>,
    material: Option<AssetHandle<Material>>,
}

impl Buildable for MeshInstance3D {
    type Builder = MeshInstance3DBuilder;
    fn builder() -> Self::Builder {
        MeshInstance3DBuilder::default()
    }
}

impl Builder for MeshInstance3DBuilder {
    type Node = MeshInstance3D;
    fn prototype(&mut self) -> &mut NodePrototype {
        &mut self.prototype
    }

    fn build(self) -> Self::Node {
        Self::Node {
            transform: self.prototype.transform,
            mesh: self.mesh,
            material: self.material,
        }
    }
}

impl MeshInstance3DBuilder {
    pub fn mesh(mut self, mesh: AssetHandle<Mesh3D>) -> Self {
        self.mesh = Some(mesh);
        self
    }

    pub fn material(mut self, material: AssetHandle<Material>) -> Self {
        self.material = Some(material);
        self
    }
}
