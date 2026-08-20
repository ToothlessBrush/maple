use std::f32::consts::FRAC_PI_4;

use glam::Vec3;
use maple_engine::{
    Buildable, Builder, Node, components::NodeTransform, nodes::node_builder::NodePrototype,
};
use rapier3d::{
    control::KinematicCharacterController,
    dynamics::{RigidBodyBuilder, RigidBodyHandle},
    geometry::SharedShape,
};

pub use rapier3d::control::CharacterAutostep;
pub use rapier3d::control::CharacterLength;

pub struct CharacterControllerConfiguration {
    pub gravity_scale: f32,
    pub enabled: bool,
    pub slide: bool,
    pub offset: CharacterLength,
    pub up: Vec3,
    pub max_slope_climb_angle: f32,
    pub min_slope_slide_angle: f32,
    pub autostep: Option<CharacterAutostep>,
    pub snap_to_ground: Option<CharacterLength>,
}

pub struct CharacterController {
    pub transform: NodeTransform,
    pub(crate) controller: KinematicCharacterController,
    pub(crate) rigid_body: Option<RigidBodyHandle>,
    pub(crate) colliders: Option<SharedShape>,
    pub(crate) is_grounded: bool,

    pub velocity: Vec3,

    // Configuration
    pub(crate) config: CharacterControllerConfiguration,
}

impl Node for CharacterController {
    fn get_transform(&mut self) -> &mut NodeTransform {
        &mut self.transform
    }
}

impl CharacterController {
    pub(crate) fn to_rapier_body(&self) -> RigidBodyBuilder {
        // Build rigid body from configuration
        let mut builder = RigidBodyBuilder::kinematic_position_based();
        // Apply transform
        let position = Vec3::new(
            self.transform.position.x,
            self.transform.position.y,
            self.transform.position.z,
        );

        builder = builder
            .translation(position)
            .rotation(self.transform.rotation.to_scaled_axis())
            .enabled(self.config.enabled);

        builder
    }

    pub fn is_grounded(&self) -> bool {
        self.is_grounded
    }
}

impl Buildable for CharacterController {
    type Builder = CharacterControllerBuilder;

    fn builder() -> Self::Builder {
        CharacterControllerBuilder {
            proto: NodePrototype::default(),
            gravity_scale: 1.0,
            enabled: true,
            autostep: None,
            offset: CharacterLength::Relative(0.01),
            up: Vec3::Y,
            slide: true,
            snap_to_ground: Some(CharacterLength::Relative(0.2)),
            max_slope_climb_angle: FRAC_PI_4,
            min_slope_slide_angle: FRAC_PI_4,
            normal_nudge_factor: 1.0e-4,
        }
    }
}

/// used to build [`CharacterController`]
pub struct CharacterControllerBuilder {
    proto: NodePrototype,
    gravity_scale: f32,
    enabled: bool,
    offset: CharacterLength,
    up: Vec3,
    slide: bool,
    max_slope_climb_angle: f32,
    min_slope_slide_angle: f32,
    autostep: Option<CharacterAutostep>,
    snap_to_ground: Option<CharacterLength>,
    normal_nudge_factor: f32,
}

impl Builder for CharacterControllerBuilder {
    type Node = CharacterController;

    fn prototype(&mut self) -> &mut NodePrototype {
        &mut self.proto
    }

    fn build(self) -> Self::Node {
        CharacterController {
            transform: self.proto.transform,
            rigid_body: None,
            controller: KinematicCharacterController {
                snap_to_ground: self.snap_to_ground,
                autostep: self.autostep,
                min_slope_slide_angle: self.min_slope_slide_angle,
                max_slope_climb_angle: self.max_slope_climb_angle,
                up: self.up,
                offset: self.offset,
                slide: self.slide,
                normal_nudge_factor: self.normal_nudge_factor,
            },
            colliders: None,
            is_grounded: false,
            velocity: Vec3::ZERO,

            config: CharacterControllerConfiguration {
                gravity_scale: self.gravity_scale,
                enabled: self.enabled,
                offset: self.offset,
                autostep: self.autostep,
                slide: self.slide,
                max_slope_climb_angle: self.max_slope_climb_angle,
                min_slope_slide_angle: self.min_slope_slide_angle,
                snap_to_ground: self.snap_to_ground,
                up: self.up,
            },
        }
    }
}

impl CharacterControllerBuilder {
    /// how much gravity affects this controller
    pub fn gravity_scale(mut self, scale: f32) -> Self {
        self.gravity_scale = scale;
        self
    }

    /// Set whether the character controller is enabled
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Set the collider offset maintained against obstacles
    pub fn offset(mut self, offset: CharacterLength) -> Self {
        self.offset = offset;
        self
    }

    /// Set the up direction used for slope/ground checks
    pub fn up(mut self, up: impl Into<Vec3>) -> Self {
        self.up = up.into();
        self
    }

    /// Set whether the controller slides along obstacles it can't climb
    pub fn slide(mut self, slide: bool) -> Self {
        self.slide = slide;
        self
    }

    /// Set the maximum slope angle the controller can climb
    pub fn max_slope_climb_angle(mut self, angle: f32) -> Self {
        self.max_slope_climb_angle = angle;
        self
    }

    /// Set the minimum slope angle at which the controller slides off
    pub fn min_slope_slide_angle(mut self, angle: f32) -> Self {
        self.min_slope_slide_angle = angle;
        self
    }

    /// Enable automatic step-climbing behavior
    pub fn autostep(mut self, autostep: CharacterAutostep) -> Self {
        self.autostep = Some(autostep);
        self
    }

    /// Disable automatic step-climbing behavior
    pub fn no_autostep(mut self) -> Self {
        self.autostep = None;
        self
    }

    /// Set snapping to ground within the given distance
    pub fn snap_to_ground(mut self, snap: CharacterLength) -> Self {
        self.snap_to_ground = Some(snap);
        self
    }

    /// Disable snapping to ground
    pub fn no_snap_to_ground(mut self) -> Self {
        self.snap_to_ground = None;
        self
    }

    /// Set the normal nudge factor used to prevent getting stuck on edges
    pub fn normal_nudge_factor(mut self, factor: f32) -> Self {
        self.normal_nudge_factor = factor;
        self
    }
}
