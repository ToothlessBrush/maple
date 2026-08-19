use glam::Vec3;
use maple_engine::{Node, components::NodeTransform};
use rapier3d::{
    control::KinematicCharacterController,
    dynamics::{RigidBodyBuilder, RigidBodyHandle, RigidBodyType},
    geometry::SharedShape,
};

use crate::nodes::rigid_body::RigidBodyConfiguration;

pub struct CharacterController {
    pub transform: NodeTransform,
    pub(crate) controller: KinematicCharacterController,
    pub(crate) rigid_body: Option<RigidBodyHandle>,
    pub(crate) colliders: Option<SharedShape>,
    pub(crate) is_grounded: bool,

    // Configuration
    pub(crate) config: RigidBodyConfiguration,
}

impl Node for CharacterController {
    fn get_transform(&mut self) -> &mut NodeTransform {
        &mut self.transform
    }
}

impl CharacterController {
    pub(crate) fn to_rapier_body(&self) -> RigidBodyBuilder {
        // Build rigid body from configuration
        let mut builder = match self.config.body_type {
            RigidBodyType::Dynamic => RigidBodyBuilder::dynamic(),
            RigidBodyType::Fixed => RigidBodyBuilder::fixed(),
            RigidBodyType::KinematicPositionBased => RigidBodyBuilder::kinematic_position_based(),
            RigidBodyType::KinematicVelocityBased => RigidBodyBuilder::kinematic_velocity_based(),
        };

        // Apply transform
        let position = Vec3::new(
            self.transform.position.x,
            self.transform.position.y,
            self.transform.position.z,
        );

        builder = builder
            .translation(position)
            .rotation(self.transform.rotation.to_scaled_axis());

        // Apply all configuration
        builder = builder
            .gravity_scale(self.config.gravity_scale)
            .linear_damping(self.config.linear_damping)
            .angular_damping(self.config.angular_damping)
            .locked_axes(self.config.locked_axes)
            .ccd_enabled(self.config.ccd_enabled)
            .can_sleep(self.config.can_sleep)
            .sleeping(self.config.sleeping)
            .dominance_group(self.config.dominance_group)
            .enabled(self.config.enabled);

        if self.config.additional_mass > 0.0 {
            builder = builder.additional_mass(self.config.additional_mass);
        }

        builder
    }
}
