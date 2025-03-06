use nalgebra_glm::vec3;
use quaturn::nodes::camera::Camera3DBuilder;
use quaturn::nodes::model::ModelBuilder;
use quaturn::nodes::point_light::PointLightBuilder;
use quaturn::nodes::{Camera3D, Container, Empty, Model, PointLight, model::Primitive};
use std::time::Duration;

use quaturn::nodes::NodeBuilder;

use quaturn::context::scene::{Node, Scene};
use quaturn::renderer::shader::Shader;
use quaturn::utils::color::Color;
use quaturn::{glfw, glm};
use std::f32::consts::{FRAC_PI_4, PI};

use quaturn::components::Event;

use crate::{WINDOW_HEIGHT, WINDOW_WIDTH};
pub struct MainScene;

impl MainScene {
    pub fn build() -> Scene {
        let mut scene = Scene::default();

        const RAD_120: f32 = 120.0 * PI / 180.0;

        scene.add(
            "building",
            NodeBuilder::<Model>::model_gltf("res/models/sponza.glb")
                .with_rotation_euler_xyz(glm::vec3(0.0, 0.0, 0.0))
                .with_scale(vec3(1.0, 1.0, 1.0))
                .on(Event::Update, |model, ctx| {
                    //    model.transform.rotate(vec3(0.0, 1.0, 0.0), 1.0);
                })
                .build(),
        );

        // scene
        //     .add(
        //         "model Group",
        //         NodeBuilder::<Empty>::empty()
        //             .add_child(
        //                 "cube",
        //                 NodeBuilder::<Model>::model_primitive(Primitive::Cube)
        //                     .set_material_base_color(Color::from_8bit_rgb(255, 0, 0).into())
        //                     .with_position(vec3(0.0, 0.0, 0.0))
        //                     .build(),
        //             )
        //             .add_child(
        //                 "plane",
        //                 NodeBuilder::<Model>::model_primitive(Primitive::Plane)
        //                     .with_position(vec3(0.0, -1.0, 0.0))
        //                     .with_scale(vec3(10.0, 10.0, 10.0))
        //                     .build(),
        //             )
        //             .build(),
        //     )
        //     .expect("model_group failed");

        scene
            .add(
                "cube",
                NodeBuilder::<Model>::model_primitive(Primitive::Cube)
                    .set_material_base_color(Color::from_8bit_rgb(255, 0, 0).into())
                    .with_position(vec3(0.0, 0.0, 0.0))
                    .build(),
            )
            .expect("cube failed to build");

        scene
            .add(
                "plane",
                NodeBuilder::<Model>::model_primitive(Primitive::Plane)
                    .with_position(vec3(0.0, -1.0, 0.0))
                    .with_scale(vec3(10.0, 10.0, 10.0))
                    .build(),
            )
            .expect("plane failed to build");

        let camera_pos = glm::vec3(0.0, 0.0, -1.0);
        scene
            .add(
                "camera",
                NodeBuilder::new(Camera3D::new(
                    FRAC_PI_4, // pi/4
                    WINDOW_WIDTH as f32 / WINDOW_HEIGHT as f32,
                    0.1,
                    1000.0,
                ))
                .with_position(camera_pos)
                .set_orientation_vector(glm::vec3(1.0, 0.0, 0.0))
                .on(Event::Update, move |camera, ctx| {
                    //only run when the camera is active
                    let mut cursor_locked = ctx.get_cursor_mode() == glfw::CursorMode::Disabled;

                    if cursor_locked {
                        camera.take_input(&ctx.input, ctx.frame.time_delta.as_secs_f32());
                    }

                    if ctx
                        .input
                        .mouse_button_just_pressed
                        .contains(&glfw::MouseButton::Button2)
                    {
                        cursor_locked = !cursor_locked;
                        ctx.lock_cursor(cursor_locked);
                    }
                })
                // .add_child(
                //     "light",
                //     NodeBuilder::<Container<f32>>::container(10_f32)
                //         .add_child(
                //             "source",
                //             NodeBuilder::<PointLight>::point_light(0.1, 100.0, 1024)
                //                 .set_color(Color::from_8bit_rgb(255, 255, 255).into())
                //                 .with_position(vec3(0.0, 0.0, 10.0))
                //                 .add_child(
                //                     "model",
                //                     NodeBuilder::<Model>::model_primitive(Primitive::Sphere)
                //                         .with_scale(glm::vec3(0.1, 0.1, 0.1))
                //                         .has_lighting(false)
                //                         .cast_shadows(false)
                //                         .build(),
                //                 )
                //                 .build(),
                //         )
                //         .build(),
                // )
                .build(),
            )
            .expect("failed to add model to scene!");

        // engine.context.nodes.add(
        //     "light_group",
        //     NodeBuilder::<Empty>::empty()
        //         .add_child(
        //             "red_light",
        //             NodeBuilder::<PointLight>::point_light(0.1, 100.0, 1024)
        //                 .with_behavior(|light, ctx| {
        //                     let now = Instant::now();
        //                     let elapsed = now.duration_since(ctx.frame.start_time).as_secs_f32();
        //                     light.get_transform().set_position(glm::vec3(
        //                         (elapsed + RAD_120).sin(),
        //                         1.0,
        //                         (elapsed + RAD_120).cos(),
        //                     ));
        //                 })
        //                 .set_color(glm::vec4(1.0, 0.0, 0.0, 1.0))
        //                 .with_position(glm::vec3(1.0, 1.0, -1.0))
        //                 .add_child(
        //                     "model",
        //                     NodeBuilder::<Model>::model_primitive(Primitive::Sphere)
        //                         .with_scale(glm::vec3(0.1, 0.1, 0.1))
        //                         .set_material_base_color(Color::from_8bit_rgb(255, 0, 0).into())
        //                         .has_lighting(false)
        //                         .cast_shadows(false)
        //                         .build(),
        //                 )
        //                 .build(),
        //         )
        //         .add_child(
        //             "green_light",
        //             NodeBuilder::<PointLight>::point_light(0.1, 100.0, 1024)
        //                 .with_behavior(|light, ctx| {
        //                     let now = Instant::now();
        //                     let elapsed = now.duration_since(ctx.frame.start_time).as_secs_f32() * 3.0;
        //                     light.get_transform().set_position(glm::vec3(
        //                         elapsed.sin(),
        //                         1.0,
        //                         elapsed.cos(),
        //                     ));
        //                 })
        //                 .set_color(glm::vec4(0.0, 1.0, 0.0, 1.0))
        //                 .with_position(glm::vec3(-1.0, 1.0, -1.0))
        //                 .add_child(
        //                     "model",
        //                     NodeBuilder::<Model>::model_primitive(Primitive::Sphere)
        //                         .with_scale(glm::vec3(0.1, 0.1, 0.1))
        //                         .set_material_base_color(Color::from_8bit_rgb(0, 255, 0).into())
        //                         .has_lighting(false)
        //                         .cast_shadows(false)
        //                         .build(),
        //                 )
        //                 .build(),
        //         )
        //         .add_child(
        //             "blue_light",
        //             NodeBuilder::<PointLight>::point_light(0.1, 100.0, 1024)
        //                 .with_behavior(|light, ctx| {
        //                     let now = Instant::now();
        //                     let elapsed = now.duration_since(ctx.frame.start_time).as_secs_f32() * 5.0;
        //                     light.get_transform().set_position(glm::vec3(
        //                         (elapsed - RAD_120).sin(),
        //                         1.0,
        //                         (elapsed - RAD_120).cos(),
        //                     ));
        //                 })
        //                 .set_color(glm::vec4(0.0, 0.0, 1.0, 1.0))
        //                 .with_position(glm::vec3(0.0, 1.0, 1.0))
        //                 .add_child(
        //                     "model",
        //                     NodeBuilder::<Model>::model_primitive(Primitive::Sphere)
        //                         .with_scale(glm::vec3(0.1, 0.1, 0.1))
        //                         .set_material_base_color(Color::from_8bit_rgb(0, 0, 255).into())
        //                         .has_lighting(false)
        //                         .cast_shadows(false)
        //                         .build(),
        //                 )
        //                 .build(),
        //         )
        //         .build(),
        // );

        //node_check(container);

        scene.add(
            "bias",
            NodeBuilder::<Container<f32>>::container(0.0).build(),
        );

        // simple game manager example
        scene.add(
            "game manager",
            NodeBuilder::<Empty>::empty()
                .on(Event::Ready, |_empty, _ctx| {
                    println!("game manager ready");
                })
                .on(Event::Update, move |_game_manager, context| {
                    //ran every frame
                    if context.input.keys.contains(&glfw::Key::Escape) {
                        context.window.set_should_close(true);
                    }

                    if context.frame.start_time.elapsed().as_secs_f32()
                        % Duration::from_secs(1).as_secs_f32()
                        == 0.0
                    {
                        let fps = context.frame.fps;

                        context
                            .window
                            .set_title(&format!("Hello Pyramid | fps: {}", fps));
                    }
                })
                .build(),
        );

        // using default shader
        // let shader = scene.add_shader("default", Shader::default());

        // shader.bind();

        scene
    }
}
