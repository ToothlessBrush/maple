use glam::Vec3;
use maple_app::{App, Plugin, Running};
use maple_engine::GameContext;

use crate::resource::Physics;

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
        physics.sync_to_rapier(&ctx.scene);
        physics.step();
        physics.sync_to_maple(&ctx.scene);
        physics.dispatch_events(ctx);
    }
}
