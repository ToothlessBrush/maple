use nalgebra_glm::vec3;
use quaturn::nodes::camera::Camera3DBuilder;
use quaturn::nodes::container::ContainerBuilder;
use quaturn::nodes::empty::EmptyBuilder;
use quaturn::nodes::model::ModelBuilder;
use quaturn::nodes::point_light::PointLightBuilder;
use quaturn::nodes::{model::Primitive, Camera3D, Container, Empty, Model, PointLight};
use std::time::Duration;

use quaturn::nodes::NodeBuilder;

use quaturn::context::scene::{Node, Scene};
use quaturn::renderer::shader::Shader;
use quaturn::utils::color::Color;
use quaturn::{glfw, glm};
use std::f32::consts::{FRAC_PI_4, PI};

use quaturn::components::{Event, EventReceiver, NodeTransform};

use crate::{WINDOW_HEIGHT, WINDOW_WIDTH};
pub struct MainScene;

#[derive(Clone)] // Nodes need Clone trait
struct CustomNode {
    transform: NodeTransform,
    children: Scene,
    events: EventReceiver,

    custom_field: i32,
}

impl Node for CustomNode {
    fn get_events(&mut self) -> &mut EventReceiver {
        &mut self.events
    }
    fn get_children(&self) -> &Scene {
        &self.children
    }
    fn get_transform(&mut self) -> &mut NodeTransform {
        &mut self.transform
    }
    fn get_children_mut(&mut self) -> &mut Scene {
        &mut self.children
    }
}

trait CustomNodeBuilder {
    fn set_custom_field(&mut self, item: i32) -> &mut Self;
}

impl CustomNodeBuilder for NodeBuilder<CustomNode> {
    fn set_custom_field(&mut self, item: i32) -> &mut Self {
        self.node.custom_field = item;
        self
    }
}

impl MainScene {
    pub fn build() -> Scene {
        let mut scene = Scene::default();

        const RAD_120: f32 = 120.0 * PI / 180.0;

        scene
            .add(
                "building",
                NodeBuilder::<Model>::create_gltf("res/models/normalTest.gltf")
                    .with_rotation_euler_xyz(glm::vec3(0.0, 0.0, 0.0))
                    .with_position(glm::vec3(0.0, 0.0, 0.0))
                    //.with_scale(vec3(0.1, 0.1, 0.1))
                    .on(Event::Update, |model, ctx| {
                        //    model.transform.rotate(vec3(0.0, 1.0, 0.0), 1.0);
                    })
                    .build(),
            )
            .expect("failed to add building");

        scene
            .add(
                "plane",
                NodeBuilder::<Model>::create_primitive(Primitive::Plane).build(),
            )
            .expect("failed to add plane");

        // scene
        //     .add(
        //         "model Group",
        //         NodeBuilder::<Empty>::create()
        //             .add_child(
        //                 "cube",
        //                 NodeBuilder::<Model>::create_primitive(Primitive::SmoothSphere)
        //                     // .set_material_base_color(Color::from_8bit_rgb(255, 0, 0).into())
        //                     .with_position(vec3(0.0, 0.0, 0.0))
        //                     // .on(Event::Update, |model, ctx| {
        //                     //     let elapsed = ctx.frame.start_time.elapsed().as_secs_f32();
        //                     //     if (5.0..15.0).contains(&elapsed) {
        //                     //         model.transform.position.y +=
        //                     //             0.5 * ctx.frame.time_delta.as_secs_f32();
        //                     //         return;
        //                     //     } else if elapsed < 5.0 {
        //                     //         return;
        //                     //     }
        //                     //     if let Some(speed) =
        //                     //         model.get_children_mut().get_mut::<Container<f32>>("speed")
        //                     //     {
        //                     //         *speed.get_data_mut() +=
        //                     //             1.0 * ctx.frame.time_delta.as_secs_f32();
        //                     //         let speed_data = *speed.get_data();
        //                     //         model.transform.rotate_euler_xyz(vec3(
        //                     //             3.0 * speed_data,
        //                     //             1.0 * speed_data,
        //                     //             2.0 * speed_data,
        //                     //         ));
        //                     //     }
        //                     // })
        //                     .add_child(
        //                         "speed",
        //                         NodeBuilder::<Container<f32>>::create(0.0f32).build(),
        //                     )
        //                     .build(),
        //             )
        //             .add_child(
        //                 "plane",
        //                 NodeBuilder::<Model>::create_primitive(Primitive::Plane)
        //                     .with_position(vec3(0.0, -1.0, 0.0))
        //                     .with_scale(vec3(10.0, 10.0, 10.0))
        //                     .build(),
        //             )
        //             .build(),
        //     )
        //     .expect("model_group failed");

        let camera_pos = glm::vec3(20.0, 20.0, 20.0);
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
                .set_speed(10.0)
                .set_orientation_vector(glm::Vec3::zeros() - camera_pos)
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

        scene
            .add("bias", NodeBuilder::<Container<f32>>::create(0.0).build())
            .expect("bias node failed");

        // simple game manager example
        scene
            .add(
                "game manager",
                NodeBuilder::<Empty>::create()
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
            )
            .expect("game manager failed");

        scene
    }
}
