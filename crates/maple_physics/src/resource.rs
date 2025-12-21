use maple::{
    engine::{Node, Scene},
    math::Vec3,
    prelude::Resource,
};
use rapier3d::prelude::{
    CCDSolver, ColliderBuilder, ColliderHandle, ColliderSet, DefaultBroadPhase, ImpulseJointSet,
    IntegrationParameters, IslandManager, MultibodyJointSet, NarrowPhase, PhysicsPipeline,
    RigidBodyBuilder, RigidBodyHandle, RigidBodySet, nalgebra::UnitQuaternion,
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

    pub fn set_gravity(&mut self, gravity: Vec3) {
        self.gravity = gravity
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

    pub fn sync_to_rapier(&mut self, scene: &Scene) {
        scene.for_each_ref(&mut |node: &RigidBody3D| {
            let Some(handle) = node.handle else {
                eprint!("node not added");
                return;
            };

            let body = &mut self.rigid_body_set[handle];

            // Check if position changed (only update if different to avoid resetting velocity)
            let rapier_pos: Vec3 = (*body.translation()).into();
            if (node.transform.position - rapier_pos).length_squared() > 1e-6 {
                body.set_translation(node.transform.position.into(), true);
            }

            // Check if rotation changed (only update if different to avoid resetting angular velocity)
            let rapier_rot: maple::math::Quat = (*body.rotation()).into();
            // Compare quaternions using dot product (close to 1.0 or -1.0 means same rotation)
            let dot = node.transform.rotation.dot(rapier_rot).abs();
            if dot < 0.9999 {
                // If not nearly identical
                let rotation =
                    UnitQuaternion::new_normalize(rapier3d::prelude::nalgebra::Quaternion::new(
                        node.transform.rotation.w,
                        node.transform.rotation.x,
                        node.transform.rotation.y,
                        node.transform.rotation.z,
                    ));
                body.set_rotation(rotation, true);
            }

            // Always update velocity (user can freely modify)
            body.set_linvel(node.velocity.into(), true);

            // Always update angular velocity (user can freely modify)
            body.set_angvel(node.angular_velocity.into(), true);
        });
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
            node.velocity = (*body.linvel()).into();
            node.angular_velocity = (*body.angvel()).into();
        });
    }
}
