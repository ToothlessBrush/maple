use maple::context::scene::Scene;
use maple::glfw::PWindow;
use maple::math;
use maple::nodes::node_builder::{Buildable, Builder};
use maple::nodes::{
    Camera3D, Camera3DBuilder, DirectionalLight, DirectionalLightBuilder, Model, ModelBuilder,
    NodeBuilder, model::Primitive,
};
use maple::nodes::{PointLight, PointLightBuilder};
use maple::utils::color;
use std::path::Path;

/// get the screen resolution
use std::f32::consts::FRAC_PI_4;

pub struct MainScene;

impl MainScene {
    pub fn build(window: &PWindow) -> Scene {
        let mut scene = Scene::default();

        //         add a pyramid node
        // scene
        //     .add(
        //         "pyramid", // name
        //         // creates a NodeBuilder for a pyramid Model
        //         NodeBuilder::<Model>::create_primitive(Primitive::Pyramid)
        //             // make it spin to demonstrate udate behavior
        //             .on(maple::components::Event::Update, |model, ctx| {
        //                 model.transform.rotate_euler_xyz(math::vec3(
        //                     0.0,
        //                     90.0 * ctx.frame.time_delta.as_secs_f32(),
        //                     0.0,
        //                 ));
        //             })
        //             .build(),
        //     )
        //     .expect("failed to add pyramid");

        // // add a ground to demonstrate shadows
        // scene
        //     .add(
        //         "ground",
        //         NodeBuilder::<Model>::create_primitive(Primitive::Plane)
        //             .with_position(math::vec3(0.0, -2.0, 0.0))
        //             .with_scale_factor(10.0)
        //             .build(),
        //     )
        //     .expect("faile to build ground");

        scene
            .add(
                "model",
                Model::builder()
                    .load_gltf(Path::new("res/models/Untitled.gltf"))
                    .rotation_euler_xyz(math::vec3(0.0, 0.0, -90.0))
                    .build(),
            )
            .expect("failed to load model");

        // scene
        //     .add(
        //         "point light",
        //         NodeBuilder::<PointLight>::create(0.0, 10.0)
        //             .with_position(math::vec3(0.0, 3.0, 0.0))
        //             .set_intensity(10.0)
        //             .build(),
        //     )
        //     .expect("failed to create pointlight");

        scene
            .add(
                "camera",
                Camera3D::builder()
                    // offset it back a bit
                    .position(math::vec3(1.0, 1.0, -10.0))
                    // look forward towards the scene center and slightly downward
                    .orientation_vector(math::vec3(0.0, -0.2, 1.0))
                    .on(maple::components::Event::Ready, |camera, ctx| {
                        ctx.lock_cursor(true);
                    })
                    .on(maple::components::Event::Update, |camera, ctx| {
                        camera.free_fly(&ctx.input, ctx.frame.time_delta_f32);
                    })
                    .build(),
            )
            .expect("failed to add camera");

        // add a sun to demonstrate light
        scene
            .add("sun", DirectionalLight::builder().build())
            .expect("failed to add Light");

        scene
    }
}
