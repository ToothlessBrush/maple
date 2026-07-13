use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use glam::{Quat, Vec3};
use log::error;
use maple_engine::{
    GameContext, Node, Scene,
    prelude::{EventLabel, Resource},
    scene::NodeId,
};
use rapier3d::prelude::{
    ActiveCollisionTypes, ActiveEvents, ActiveHooks, CCDSolver, Collider, ColliderBuilder,
    ColliderHandle, ColliderSet, CollisionEvent, DefaultBroadPhase, EventHandler, ImpulseJointSet,
    IntegrationParameters, IslandManager, MultibodyJointSet, NarrowPhase, PhysicsPipeline,
    RigidBodyBuilder, RigidBodyHandle, RigidBodySet,
};

use crate::nodes::{Collider3D, RigidBody3D};

pub struct ColliderEnter {
    pub other: NodeId,
}
impl EventLabel for ColliderEnter {}

pub struct ColliderExit {
    pub other: NodeId,
}
impl EventLabel for ColliderExit {}

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
    event_handler: PhysicsEventHandler,

    rigid_body_set: RigidBodySet,
    collider_set: ColliderSet,

    // shared between event handler and this
    pending_collision_events: Arc<Mutex<Vec<CollisionEvent>>>,
}

impl Resource for Physics {}

impl Physics {
    /// create the physics resource
    pub fn new(gravity: Vec3) -> Self {
        let events = Arc::new(Mutex::new(Vec::new()));

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
            event_handler: PhysicsEventHandler {
                events: events.clone(),
            },

            rigid_body_set: RigidBodySet::new(),
            collider_set: ColliderSet::new(),
            pending_collision_events: events.clone(),
        }
    }

    pub fn set_gravity(&mut self, gravity: Vec3) {
        self.gravity = gravity
    }

    pub fn add_collidor_with_parent(
        &mut self,
        parent: &RigidBodyHandle,
        collider: ColliderBuilder,
    ) -> ColliderHandle {
        self.collider_set
            .insert_with_parent(collider, *parent, &mut self.rigid_body_set)
    }

    pub fn add_free_collidor(&mut self, collider: Collider) -> ColliderHandle {
        self.collider_set.insert(collider)
    }

    pub fn add_rigid_body(&mut self, body: RigidBodyBuilder) -> RigidBodyHandle {
        self.rigid_body_set.insert(body)
    }

    /// Initialize any RigidBody3D nodes that haven't been added to the physics world yet
    pub fn initialize_bodies(&mut self, scene: &Scene) {
        use rapier3d::prelude::{RigidBodyBuilder, RigidBodyType};

        scene.for_each_with_id(&mut |node_id, node: &mut RigidBody3D| {
            // Skip if already initialized
            if node.handle.is_some() {
                return;
            }

            // Build rigid body from configuration
            let mut builder = match node.body_type {
                RigidBodyType::Dynamic => RigidBodyBuilder::dynamic(),
                RigidBodyType::Fixed => RigidBodyBuilder::fixed(),
                RigidBodyType::KinematicPositionBased => {
                    RigidBodyBuilder::kinematic_position_based()
                }
                RigidBodyType::KinematicVelocityBased => {
                    RigidBodyBuilder::kinematic_velocity_based()
                }
            };

            // Apply transform
            let position = Vec3::new(
                node.transform.position.x,
                node.transform.position.y,
                node.transform.position.z,
            );

            builder = builder
                .translation(position)
                .rotation(node.transform.rotation.to_scaled_axis());

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

            let handle = self.add_rigid_body(builder);
            node.handle = Some(handle);

            // Find and attach all Collider3D children
            let children = scene.children_ids(node_id);
            for child_id in children {
                if let Some(child) = scene.get::<Collider3D>(child_id) {
                    let mut child_node = child.write();
                    let collider_handle = child_node.get_rapier_collidor();
                    child_node.handle =
                        Some(self.add_collidor_with_parent(&handle, collider_handle));
                }
            }
        });

        scene.for_each(&mut |node: &mut Collider3D| {
            if node.handle.is_some() {
                // already registered
                return;
            }

            let handle = node
                .get_rapier_collidor()
                .translation(node.transform.position)
                .rotation(node.transform.rotation.to_scaled_axis())
                .active_collision_types(
                    ActiveCollisionTypes::default() | ActiveCollisionTypes::FIXED_FIXED,
                )
                .build();

            node.handle = Some(self.add_free_collidor(handle))
        });
    }

    pub fn sync_to_rapier(&mut self, scene: &Scene) {
        scene.for_each_ref(&mut |node: &RigidBody3D| {
            let Some(handle) = node.handle else {
                error!("node not added");
                return;
            };

            let body = &mut self.rigid_body_set[handle];

            // Check if position changed (only update if different to avoid resetting velocity)
            let rapier_pos: Vec3 = body.translation();
            if (node.transform.position - rapier_pos).length_squared() > 1e-6 {
                body.set_translation(node.transform.position.into(), true);
            }

            // Check if rotation changed (only update if different to avoid resetting angular velocity)
            let rapier_rot: Quat = (*body.rotation()).into();
            // Compare quaternions using dot product (close to 1.0 or -1.0 means same rotation)
            let dot = node.transform.rotation.dot(rapier_rot).abs();
            if dot < 0.9999 {
                // If not nearly identical
                body.set_rotation(node.transform.rotation, true);
            }

            // Always update velocity (user can freely modify)
            body.set_linvel(node.velocity.into(), true);

            // Always update angular velocity (user can freely modify)
            body.set_angvel(node.angular_velocity.into(), true);
        });

        scene.for_each_ref(&mut |node: &Collider3D| {
            let Some(handle) = node.handle else {
                return;
            };

            // check if collider is parented to a rigid body if so it was already updated
            if self.collider_set[handle].parent().is_some() {
                return;
            }

            if let Some(collider) = self.collider_set.get_mut(handle) {
                let rapier_pos = collider.translation();
                if (node.transform.position - rapier_pos).length_squared() > 1e-6 {
                    collider.set_translation(node.transform.position);
                }

                let rapier_rot = collider.rotation();
                let dot = node.transform.rotation.dot(rapier_rot).abs();
                if dot < 0.9999 {
                    collider.set_rotation(node.transform.rotation);
                }
            }
        });
    }

    /// step in the physics sim should be every 1/60 of a second
    pub fn step(&mut self) {
        self.physics_pipeline.step(
            self.gravity,
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

    pub fn sync_to_maple(&self, scene: &Scene) {
        scene.for_each(&mut |node: &mut RigidBody3D| {
            let Some(handle) = node.handle else {
                log::error!("not all nodes added");
                return;
            };

            let body = &self.rigid_body_set[handle];

            // Convert nalgebra types to glam using the convert-glam-030 feature
            node.get_transform().position = body.translation();
            node.get_transform().rotation = *body.rotation();
            node.velocity = body.linvel();
            node.angular_velocity = body.angvel();
        });
    }

    pub fn dispatch_events(&mut self, ctx: &GameContext) {
        // take events since they will be cleared anyway
        let events: Vec<CollisionEvent> = {
            let mut events = self.pending_collision_events.lock().unwrap();
            std::mem::take(&mut *events)
        };

        if events.is_empty() {
            return;
        }

        let scene = &ctx.scene;

        // map collider handle to node id
        let handle_map: HashMap<ColliderHandle, NodeId> = {
            let mut map = HashMap::new();
            scene.for_each_with_id(&mut |id, node: &mut Collider3D| {
                if let Some(handle) = node.handle {
                    map.insert(handle, id);
                }
            });
            map
        };

        for event in events {
            let (h1, h2, is_enter) = match event {
                CollisionEvent::Started(h1, h2, _) => (h1, h2, true),
                CollisionEvent::Stopped(h1, h2, _) => (h1, h2, false),
            };

            let node1 = handle_map.get(&h1).copied();
            let node2 = handle_map.get(&h2).copied();

            if let (Some(id1), Some(id2)) = (node1, node2) {
                if is_enter {
                    scene.emit_to(id1, &ColliderEnter { other: id2 }, ctx);
                    scene.emit_to(id2, &ColliderEnter { other: id1 }, ctx);
                } else {
                    scene.emit_to(id1, &ColliderExit { other: id2 }, ctx);
                    scene.emit_to(id2, &ColliderExit { other: id1 }, ctx);
                }
            }
        }
    }
}

pub struct PhysicsEventHandler {
    events: Arc<Mutex<Vec<CollisionEvent>>>,
}

impl EventHandler for PhysicsEventHandler {
    fn handle_collision_event(
        &self,
        _bodies: &RigidBodySet,
        _colliders: &ColliderSet,
        event: rapier3d::prelude::CollisionEvent,
        _contact_pair: Option<&rapier3d::prelude::ContactPair>,
    ) {
        self.events.lock().unwrap().push(event);
    }

    fn handle_contact_force_event(
        &self,
        _dt: f32,
        _bodies: &RigidBodySet,
        _colliders: &ColliderSet,
        _contact_pair: &rapier3d::prelude::ContactPair,
        _total_force_magnitude: f32,
    ) {
    }
}
