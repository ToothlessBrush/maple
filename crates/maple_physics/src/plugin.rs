use glam::Vec3;
use maple_app::{App, Plugin, Running};

use crate::resource::Physics;

pub struct Physics3D;

impl Plugin for Physics3D {
    fn init(&self, app: &mut App<Running>) {
        let physics = Physics::new(Vec3 {
            x: 0.0,
            y: -9.81,
            z: 0.0,
        });

        app.context_mut().insert_resource(physics);
    }

    fn fixed_update(&self, app: &mut App<Running>) {
        app.context_mut()
            .with_resource_and_scene(|physics: &mut Physics, scene| {
                physics.sync_to_rapier(scene);
                physics.step();
                physics.sync_to_maple(scene);
            });
    }
}
