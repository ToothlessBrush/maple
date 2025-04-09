use quaturn::components::Event;
use quaturn::context::scene::Scene;
use quaturn::glfw::PWindow;
use quaturn::math;
use quaturn::nodes::{
    model::Primitive, Camera3D, Camera3DBuilder, Model, ModelBuilder, NodeBuilder,
};

/// get the screen resolution
use std::f32::consts::FRAC_PI_4;

pub struct MainScene;

impl MainScene {
    pub fn build(window: &PWindow) -> Scene {
        let mut scene = Scene::default();

        // add a pyramid node
        scene
            .add(
                "pyramid", // name
                NodeBuilder::<Model>::create_primitive(Primitive::Pyramid) // creates a NodeBuilder for a pyramid Model
                    .on(Event::Update, |model, ctx| {
                        model.transform.rotate_euler_xyz(math::vec3(
                            0.0,
                            90.0 * ctx.frame.time_delta.as_secs_f32(),
                            0.0,
                        ));
                    })
                    .build(), // builds the node
            )
            .expect("failed to add pyramid");

        scene
            .add(
                "camera",
                NodeBuilder::<Camera3D>::create(
                    window.get_size(),
                    FRAC_PI_4, // fov in radians
                )
                .with_position(math::vec3(0.0, 0.0, -10.0)) // offset it back a bit
                .set_orientation_vector(math::vec3(0.0, 0.0, 1.0)) // look forward towards the scene center
                .on(Event::Ready, |_camera, ctx| {
                    ctx.lock_cursor(true);
                })
                .on(Event::Update, |camera, ctx| {
                    camera.free_fly(&ctx.input, ctx.frame.time_delta.as_secs_f32());
                })
                .build(),
            )
            .expect("failed to add camera");

        scene
    }
}
