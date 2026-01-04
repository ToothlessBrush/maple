use glam::Vec3;
use maple_engine::{
    Buildable, Builder, GameContext, Node, Scene,
    components::Ready,
    nodes::node_builder::NodePrototype,
    prelude::{EventReceiver, NodeTransform},
};
use rapier3d::prelude::{
    LockedAxes, RigidBodyBuilder, RigidBodyHandle, RigidBodyType,
    nalgebra::{UnitQuaternion, Vector3},
};

use crate::{nodes::Collider3D, resource::Physics};

pub struct RigidBody3D {
    events: EventReceiver,
    pub children: Scene,
    pub transform: NodeTransform,

    pub velocity: Vec3,
    pub angular_velocity: Vec3,

    pub(crate) handle: Option<RigidBodyHandle>,

    // Configuration
    body_type: RigidBodyType,
    gravity_scale: f32,
    linear_damping: f32,
    angular_damping: f32,
    locked_axes: LockedAxes,
    ccd_enabled: bool,
    can_sleep: bool,
    sleeping: bool,
    dominance_group: i8,
    additional_mass: f32,
    enabled: bool,
}

impl Node for RigidBody3D {
    fn get_events(&mut self) -> &mut EventReceiver {
        &mut self.events
    }

    fn get_transform(&mut self) -> &mut NodeTransform {
        &mut self.transform
    }

    fn get_children(&self) -> &Scene {
        &self.children
    }

    fn get_children_mut(&mut self) -> &mut Scene {
        &mut self.children
    }
}

impl RigidBody3D {
    fn ready(node: &mut Self, ctx: &mut GameContext) {
        let Some(mut physics) = ctx.get_resource_mut::<Physics>() else {
            log::error!("tried to attach rigid body but didnt find physics plugin");
            return;
        };

        // Build rigid body from configuration
        let mut builder = match node.body_type {
            RigidBodyType::Dynamic => RigidBodyBuilder::dynamic(),
            RigidBodyType::Fixed => RigidBodyBuilder::fixed(),
            RigidBodyType::KinematicPositionBased => RigidBodyBuilder::kinematic_position_based(),
            RigidBodyType::KinematicVelocityBased => RigidBodyBuilder::kinematic_velocity_based(),
        };

        // Apply transform
        let position = Vector3::new(
            node.transform.position.x,
            node.transform.position.y,
            node.transform.position.z,
        );
        let rotation = UnitQuaternion::new_normalize(rapier3d::prelude::nalgebra::Quaternion::new(
            node.transform.rotation.w,
            node.transform.rotation.x,
            node.transform.rotation.y,
            node.transform.rotation.z,
        ));

        builder = builder
            .translation(position)
            .rotation(rotation.scaled_axis());

        // Apply all configuration
        builder = builder
            .gravity_scale(node.gravity_scale)
            .linear_damping(node.linear_damping)
            .angular_damping(node.angular_damping)
            .linvel(node.velocity.into())
            .angvel(node.angular_velocity.into())
            .locked_axes(node.locked_axes)
            .ccd_enabled(node.ccd_enabled)
            .can_sleep(node.can_sleep)
            .sleeping(node.sleeping)
            .dominance_group(node.dominance_group)
            .enabled(node.enabled);

        if node.additional_mass > 0.0 {
            builder = builder.additional_mass(node.additional_mass);
        }

        let handle = physics.add_rigid_body(builder);
        node.handle = Some(handle);

        // Register collider children
        for (_, child) in node.get_children_mut() {
            if let Some(collider) = child.downcast_mut::<Collider3D>() {
                collider.handle =
                    Some(physics.add_collidor(&handle, collider.get_rapier_collidor()));
            }
        }
    }

    pub fn get_handle(&self) -> Option<RigidBodyHandle> {
        self.handle
    }
}

impl Buildable for RigidBody3D {
    type Builder = RigidBody3DBuilder;

    fn builder() -> Self::Builder {
        RigidBody3DBuilder {
            proto: NodePrototype::default(),
            body_type: RigidBodyType::Dynamic,
            gravity_scale: 1.0,
            linear_damping: 0.0,
            angular_damping: 0.0,
            linear_velocity: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
            locked_axes: LockedAxes::empty(),
            ccd_enabled: false,
            can_sleep: true,
            sleeping: false,
            dominance_group: 0,
            additional_mass: 0.0,
            enabled: true,
        }
    }
}

pub struct RigidBody3DBuilder {
    proto: NodePrototype,
    body_type: RigidBodyType,
    gravity_scale: f32,
    linear_damping: f32,
    angular_damping: f32,
    linear_velocity: Vec3,
    angular_velocity: Vec3,
    locked_axes: LockedAxes,
    ccd_enabled: bool,
    can_sleep: bool,
    sleeping: bool,
    dominance_group: i8,
    additional_mass: f32,
    enabled: bool,
}

impl Builder for RigidBody3DBuilder {
    type Node = RigidBody3D;

    fn prototype(&mut self) -> &mut NodePrototype {
        &mut self.proto
    }

    fn build(self) -> Self::Node {
        let mut body = RigidBody3D {
            transform: self.proto.transform,
            events: self.proto.events,
            children: self.proto.children,
            handle: None,

            velocity: self.linear_velocity,
            angular_velocity: self.angular_velocity,

            body_type: self.body_type,
            gravity_scale: self.gravity_scale,
            linear_damping: self.linear_damping,
            angular_damping: self.angular_damping,
            locked_axes: self.locked_axes,
            ccd_enabled: self.ccd_enabled,
            can_sleep: self.can_sleep,
            sleeping: self.sleeping,
            dominance_group: self.dominance_group,
            additional_mass: self.additional_mass,
            enabled: self.enabled,
        };

        body.events.on::<Ready, _, _>(RigidBody3D::ready);
        body
    }
}

impl RigidBody3DBuilder {
    /// Create a dynamic rigid body (affected by forces and gravity)
    pub fn dynamic() -> Self {
        RigidBody3D::builder().body_type(RigidBodyType::Dynamic)
    }

    /// Create a static rigid body (immovable, infinite mass)
    pub fn fixed() -> Self {
        RigidBody3D::builder().body_type(RigidBodyType::Fixed)
    }

    /// Create a kinematic body controlled by position
    pub fn kinematic_position_based() -> Self {
        RigidBody3D::builder().body_type(RigidBodyType::KinematicPositionBased)
    }

    /// Create a kinematic body controlled by velocity
    pub fn kinematic_velocity_based() -> Self {
        RigidBody3D::builder().body_type(RigidBodyType::KinematicVelocityBased)
    }

    /// Set the rigid body type
    pub fn body_type(mut self, body_type: RigidBodyType) -> Self {
        self.body_type = body_type;
        self
    }

    /// Set the gravity scale (1.0 = normal gravity, 0.0 = no gravity)
    pub fn gravity_scale(mut self, scale: f32) -> Self {
        self.gravity_scale = scale;
        self
    }

    /// Set linear damping (resistance to linear motion)
    pub fn linear_damping(mut self, damping: f32) -> Self {
        self.linear_damping = damping;
        self
    }

    /// Set angular damping (resistance to rotation)
    pub fn angular_damping(mut self, damping: f32) -> Self {
        self.angular_damping = damping;
        self
    }

    /// Set initial linear velocity
    pub fn linear_velocity(mut self, velocity: impl Into<Vec3>) -> Self {
        self.linear_velocity = velocity.into();
        self
    }

    /// Set initial angular velocity (axis-angle representation)
    pub fn angular_velocity(mut self, velocity: impl Into<Vec3>) -> Self {
        self.angular_velocity = velocity.into();
        self
    }

    /// Lock translation on specific axes
    pub fn lock_translations(mut self) -> Self {
        self.locked_axes |= LockedAxes::TRANSLATION_LOCKED;
        self
    }

    /// Lock rotation on specific axes
    pub fn lock_rotations(mut self) -> Self {
        self.locked_axes |= LockedAxes::ROTATION_LOCKED;
        self
    }

    /// Lock specific axes
    pub fn locked_axes(mut self, axes: LockedAxes) -> Self {
        self.locked_axes = axes;
        self
    }

    /// Lock translation on X axis
    pub fn lock_translation_x(mut self) -> Self {
        self.locked_axes |= LockedAxes::TRANSLATION_LOCKED_X;
        self
    }

    /// Lock translation on Y axis
    pub fn lock_translation_y(mut self) -> Self {
        self.locked_axes |= LockedAxes::TRANSLATION_LOCKED_Y;
        self
    }

    /// Lock translation on Z axis
    pub fn lock_translation_z(mut self) -> Self {
        self.locked_axes |= LockedAxes::TRANSLATION_LOCKED_Z;
        self
    }

    /// Lock rotation on X axis
    pub fn lock_rotation_x(mut self) -> Self {
        self.locked_axes |= LockedAxes::ROTATION_LOCKED_X;
        self
    }

    /// Lock rotation on Y axis
    pub fn lock_rotation_y(mut self) -> Self {
        self.locked_axes |= LockedAxes::ROTATION_LOCKED_Y;
        self
    }

    /// Lock rotation on Z axis
    pub fn lock_rotation_z(mut self) -> Self {
        self.locked_axes |= LockedAxes::ROTATION_LOCKED_Z;
        self
    }

    /// Enable Continuous Collision Detection
    pub fn ccd_enabled(mut self, enabled: bool) -> Self {
        self.ccd_enabled = enabled;
        self
    }

    /// Allow the rigid body to sleep when inactive
    pub fn can_sleep(mut self, can_sleep: bool) -> Self {
        self.can_sleep = can_sleep;
        self
    }

    /// Start the rigid body in a sleeping state
    pub fn sleeping(mut self, sleeping: bool) -> Self {
        self.sleeping = sleeping;
        self
    }

    /// Set dominance group (higher values dominate lower values in constraints)
    pub fn dominance_group(mut self, group: i8) -> Self {
        self.dominance_group = group;
        self
    }

    /// Add additional mass to the rigid body
    pub fn additional_mass(mut self, mass: f32) -> Self {
        self.additional_mass = mass;
        self
    }

    /// Set whether the rigid body is enabled
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}
