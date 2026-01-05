use glam::Vec3;
use maple_engine::{
    Buildable, Builder, Node, Scene,
    nodes::node_builder::NodePrototype,
    prelude::{EventReceiver, NodeTransform},
};
use rapier3d::prelude::{ActiveEvents, ColliderBuilder, ColliderHandle, Group, InteractionGroups};

/// Collider shape types
#[derive(Clone)]
pub enum ColliderShape {
    /// A ball/sphere with the given radius
    Ball { radius: f32 },
    /// A cuboid/box with half-extents
    Cuboid { hx: f32, hy: f32, hz: f32 },
    /// A capsule (cylinder with hemispherical ends)
    Capsule {
        half_height: f32,
        radius: f32,
        axis: CapsuleAxis,
    },
    /// A cylinder
    Cylinder { half_height: f32, radius: f32 },
    /// A cone
    Cone { half_height: f32, radius: f32 },
    /// A triangle
    Triangle { a: Vec3, b: Vec3, c: Vec3 },
    /// Custom shape (placeholder for more complex shapes)
    Custom,
}

impl Default for ColliderShape {
    fn default() -> Self {
        ColliderShape::Ball { radius: 0.5 }
    }
}

#[derive(Clone, Copy)]
pub enum CapsuleAxis {
    X,
    Y,
    Z,
}

pub struct Collider3D {
    events: EventReceiver,
    children: Scene,
    transform: NodeTransform,

    pub(crate) handle: Option<ColliderHandle>,

    // Configuration
    shape: ColliderShape,
    sensor: bool,
    friction: f32,
    restitution: f32,
    density: f32,
    mass: Option<f32>,
    collision_groups: InteractionGroups,
    solver_groups: InteractionGroups,
    contact_skin: f32,
    enabled: bool,
    active_events: ActiveEvents,
}

impl Node for Collider3D {
    fn get_transform(&mut self) -> &mut NodeTransform {
        &mut self.transform
    }

    fn get_events(&mut self) -> &mut EventReceiver {
        &mut self.events
    }
}

impl Collider3D {
    pub fn new(shape: ColliderShape) -> Self {
        Self {
            events: EventReceiver::default(),
            children: Scene::default(),
            transform: NodeTransform::default(),
            handle: None,

            shape,
            sensor: false,
            friction: 0.5,
            restitution: 0.0,
            density: 1.0,
            mass: None,
            collision_groups: InteractionGroups::all(),
            solver_groups: InteractionGroups::all(),
            contact_skin: 0.0,
            enabled: true,
            active_events: ActiveEvents::empty(),
        }
    }

    pub(crate) fn get_rapier_collidor(&self) -> ColliderBuilder {
        let mut builder = match &self.shape {
            ColliderShape::Ball { radius } => ColliderBuilder::ball(*radius),
            ColliderShape::Cuboid { hx, hy, hz } => ColliderBuilder::cuboid(*hx, *hy, *hz),
            ColliderShape::Capsule {
                half_height,
                radius,
                axis,
            } => match axis {
                CapsuleAxis::X => ColliderBuilder::capsule_x(*half_height, *radius),
                CapsuleAxis::Y => ColliderBuilder::capsule_y(*half_height, *radius),
                CapsuleAxis::Z => ColliderBuilder::capsule_z(*half_height, *radius),
            },
            ColliderShape::Cylinder {
                half_height,
                radius,
            } => ColliderBuilder::cylinder(*half_height, *radius),
            ColliderShape::Cone {
                half_height,
                radius,
            } => ColliderBuilder::cone(*half_height, *radius),
            ColliderShape::Triangle { a, b, c } => {
                ColliderBuilder::triangle((*a).into(), (*b).into(), (*c).into())
            }
            ColliderShape::Custom => {
                // Default to a small ball for custom shapes
                ColliderBuilder::ball(0.5)
            }
        };

        // Note: position and rotation are relative to parent rigid body
        // They are applied as offsets, not absolute transforms

        // Apply all properties
        builder = builder
            .sensor(self.sensor)
            .friction(self.friction)
            .restitution(self.restitution)
            .collision_groups(self.collision_groups)
            .solver_groups(self.solver_groups)
            .enabled(self.enabled)
            .active_events(self.active_events);

        if let Some(mass) = self.mass {
            builder = builder.mass(mass);
        } else {
            builder = builder.density(self.density);
        }

        if self.contact_skin > 0.0 {
            builder = builder.contact_skin(self.contact_skin);
        }

        builder
    }

    pub fn get_handle(&self) -> Option<ColliderHandle> {
        self.handle
    }

    pub fn set_handle(&mut self, handle: ColliderHandle) {
        self.handle = Some(handle);
    }
}

impl Default for Collider3D {
    fn default() -> Self {
        Self::new(ColliderShape::default())
    }
}

impl Buildable for Collider3D {
    type Builder = Collider3DBuilder;

    fn builder() -> Self::Builder {
        Collider3DBuilder {
            proto: NodePrototype::default(),
            shape: ColliderShape::default(),
            sensor: false,
            friction: 0.5,
            restitution: 0.0,
            density: 1.0,
            mass: None,
            collision_groups: InteractionGroups::all(),
            solver_groups: InteractionGroups::all(),
            contact_skin: 0.0,
            enabled: true,
            active_events: ActiveEvents::empty(),
        }
    }
}

pub struct Collider3DBuilder {
    proto: NodePrototype,
    shape: ColliderShape,
    sensor: bool,
    friction: f32,
    restitution: f32,
    density: f32,
    mass: Option<f32>,
    collision_groups: InteractionGroups,
    solver_groups: InteractionGroups,
    contact_skin: f32,
    enabled: bool,
    active_events: ActiveEvents,
}

impl Builder for Collider3DBuilder {
    type Node = Collider3D;

    fn prototype(&mut self) -> &mut NodePrototype {
        &mut self.proto
    }

    fn build(self) -> Self::Node {
        Collider3D {
            transform: self.proto.transform,
            events: self.proto.events,
            children: self.proto.children,
            handle: None,

            shape: self.shape,
            sensor: self.sensor,
            friction: self.friction,
            restitution: self.restitution,
            density: self.density,
            mass: self.mass,
            collision_groups: self.collision_groups,
            solver_groups: self.solver_groups,
            contact_skin: self.contact_skin,
            enabled: self.enabled,
            active_events: ActiveEvents::COLLISION_EVENTS,
        }
    }
}

impl Collider3DBuilder {
    /// Create a ball/sphere collider
    pub fn ball(radius: f32) -> Self {
        Collider3D::builder().shape(ColliderShape::Ball { radius })
    }

    /// Create a cuboid/box collider
    pub fn cuboid(hx: f32, hy: f32, hz: f32) -> Self {
        Collider3D::builder().shape(ColliderShape::Cuboid { hx, hy, hz })
    }

    /// Create a cube collider (all sides equal)
    pub fn cube(half_extent: f32) -> Self {
        Self::cuboid(half_extent, half_extent, half_extent)
    }

    /// Create a capsule along the Y axis
    pub fn capsule_y(half_height: f32, radius: f32) -> Self {
        Collider3D::builder().shape(ColliderShape::Capsule {
            half_height,
            radius,
            axis: CapsuleAxis::Y,
        })
    }

    /// Create a capsule along the X axis
    pub fn capsule_x(half_height: f32, radius: f32) -> Self {
        Collider3D::builder().shape(ColliderShape::Capsule {
            half_height,
            radius,
            axis: CapsuleAxis::X,
        })
    }

    /// Create a capsule along the Z axis
    pub fn capsule_z(half_height: f32, radius: f32) -> Self {
        Collider3D::builder().shape(ColliderShape::Capsule {
            half_height,
            radius,
            axis: CapsuleAxis::Z,
        })
    }

    /// Create a cylinder collider
    pub fn cylinder(half_height: f32, radius: f32) -> Self {
        Collider3D::builder().shape(ColliderShape::Cylinder {
            half_height,
            radius,
        })
    }

    /// Create a cone collider
    pub fn cone(half_height: f32, radius: f32) -> Self {
        Collider3D::builder().shape(ColliderShape::Cone {
            half_height,
            radius,
        })
    }

    /// Create a triangle collider
    pub fn triangle(a: impl Into<Vec3>, b: impl Into<Vec3>, c: impl Into<Vec3>) -> Self {
        Collider3D::builder().shape(ColliderShape::Triangle {
            a: a.into(),
            b: b.into(),
            c: c.into(),
        })
    }

    /// Set the collider shape
    pub fn shape(mut self, shape: ColliderShape) -> Self {
        self.shape = shape;
        self
    }

    /// Make this collider a sensor/trigger (no physics response, only overlap detection)
    pub fn sensor(mut self, sensor: bool) -> Self {
        self.sensor = sensor;
        self
    }

    /// Set the friction coefficient (0.0 = no friction, higher = more friction)
    pub fn friction(mut self, friction: f32) -> Self {
        self.friction = friction;
        self
    }

    /// Set the restitution/bounciness (0.0 = no bounce, 1.0 = perfectly bouncy)
    pub fn restitution(mut self, restitution: f32) -> Self {
        self.restitution = restitution;
        self
    }

    /// Set the density (used to calculate mass if mass is not set directly)
    pub fn density(mut self, density: f32) -> Self {
        self.density = density;
        self.mass = None; // Clear explicit mass
        self
    }

    /// Set the mass directly (overrides density-based calculation)
    pub fn mass(mut self, mass: f32) -> Self {
        self.mass = Some(mass);
        self
    }

    /// Set collision groups (what this collider collides with)
    pub fn collision_groups(mut self, groups: InteractionGroups) -> Self {
        self.collision_groups = groups;
        self
    }

    /// Set solver groups (what this collider interacts with in physics solver)
    pub fn solver_groups(mut self, groups: InteractionGroups) -> Self {
        self.solver_groups = groups;
        self
    }

    /// Set collision memberships and filters manually
    pub fn collision_membership_filter(mut self, memberships: Group, filter: Group) -> Self {
        self.collision_groups = InteractionGroups::new(memberships, filter);
        self
    }

    /// Set contact skin thickness (for performance optimization)
    pub fn contact_skin(mut self, skin: f32) -> Self {
        self.contact_skin = skin;
        self
    }

    /// Set whether the collider is enabled
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Enable specific active events
    pub fn active_events(mut self, events: ActiveEvents) -> Self {
        self.active_events = events;
        self
    }

    /// Enable collision events
    pub fn enable_collision_events(mut self) -> Self {
        self.active_events |= ActiveEvents::COLLISION_EVENTS;
        self
    }

    /// Enable contact force events
    pub fn enable_contact_force_events(mut self) -> Self {
        self.active_events |= ActiveEvents::COLLISION_EVENTS;
        self
    }

    /// Create a trigger zone (sensor) with a ball shape
    pub fn trigger_ball(radius: f32) -> Self {
        Self::ball(radius).sensor(true)
    }

    /// Create a trigger zone (sensor) with a cuboid shape
    pub fn trigger_cuboid(hx: f32, hy: f32, hz: f32) -> Self {
        Self::cuboid(hx, hy, hz).sensor(true)
    }

    /// Create a friction-less collider (ice-like)
    pub fn frictionless(mut self) -> Self {
        self.friction = 0.0;
        self
    }
}
