use maple::{
    engine::{Node, Scene},
    math::{Quat, Vec3},
    prelude::Resource,
};
use rapier3d::prelude::{
    CCDSolver, ColliderBuilder, ColliderHandle, ColliderSet, DefaultBroadPhase, ImpulseJointSet,
    IntegrationParameters, IslandManager, MultibodyJointSet, NarrowPhase, PhysicsPipeline,
    RigidBodyBuilder, RigidBodyHandle, RigidBodySet,
};

use crate::nodes::RigidBody3D;

pub struct Physics {
    gravity: Vec3,
    integration_parameters: IntegrationParameters,
    physics_pipeline: PhysicsPipeline,
    island_manager: IslandManager,
    broad_phase: DefaultBroadPhase,
    narrow_phase: NarrowPhase,
    impulsive_joint_set: ImpulseJointSet,
    multibody_joint_set: MultibodyJointSet,
    ccd_solver: CCDSolver,
    physics_hooks: (),
    event_handler: (),

    rigid_body_set: RigidBodySet,
    collider_set: ColliderSet,
}

impl Resource for Physics {}

impl Physics {
    /// create the physics resource
    pub fn new(gravity: Vec3) -> Self {
        Self {
            gravity,
            integration_parameters: IntegrationParameters::default(),
            physics_pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: DefaultBroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            impulsive_joint_set: ImpulseJointSet::new(),
            multibody_joint_set: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
            physics_hooks: (),
            event_handler: (),

            rigid_body_set: RigidBodySet::new(),
            collider_set: ColliderSet::new(),
        }
    }

    pub fn add_collidor(
        &mut self,
        parent: &RigidBodyHandle,
        collider: ColliderBuilder,
    ) -> ColliderHandle {
        self.collider_set
            .insert_with_parent(collider, *parent, &mut self.rigid_body_set)
    }

    pub fn add_rigid_body(&mut self, body: RigidBodyBuilder) -> RigidBodyHandle {
        self.rigid_body_set.insert(body)
    }

    /// step in the physics sim should be every 1/60 of a second
    pub fn step(&mut self) {
        self.physics_pipeline.step(
            &self.gravity.into(),
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigid_body_set,
            &mut self.collider_set,
            &mut self.impulsive_joint_set,
            &mut self.multibody_joint_set,
            &mut self.ccd_solver,
            &self.physics_hooks,
            &self.event_handler,
        );
    }

    pub fn sync_to_maple(&self, scene: &mut Scene) {
        scene.for_each(&mut |node: &mut RigidBody3D| {
            let Some(handle) = node.handle else {
                eprintln!("not all nodes added");
                return;
            };

            let body = &self.rigid_body_set[handle];

            // Convert nalgebra types to glam using the convert-glam-030 feature
            node.get_transform().position = (*body.translation()).into();
            node.get_transform().rotation = (*body.rotation()).into();
        });
    }
}
