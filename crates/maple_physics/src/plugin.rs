//! plugin used for physics

use glam::Vec3;
use maple_app::{App, Plugin, Running};
use maple_engine::{context::Res, resources::Frame};

use crate::resource::Physics;

/// manages physics within the scene
///
/// runs the physics simulator with rapier and syncs transforms
pub struct Physics3D;

impl Plugin for Physics3D {
    fn ready(&self, app: &mut App<Running>) {
        let physics = Physics::new(Vec3 {
            x: 0.0,
            y: -9.81,
            z: 0.0,
        });

        app.context_mut().insert_resource(physics);
    }

    fn fixed_update(&self, app: &mut App<Running>) {
        let ctx = app.context_mut();

        let mut physics = ctx.get_resource_mut::<Physics>();
        physics.initialize_character_controllers(&ctx.scene);
        physics.initialize_bodies(&ctx.scene);
        physics.sync_to_rapier(&ctx.scene);
        physics.step();
        physics.move_character_controller(&ctx.scene, 1.0 / 60.0);
        physics.sync_to_maple(&ctx.scene);
        physics.dispatch_events(ctx);
    }
}
