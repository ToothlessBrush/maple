extern crate quaturn_derive;
use quaturn_derive::add_node_fields;
use quaturn_derive::define_node;
use quaturn_derive::Node;

use std::cell::RefCell;
use std::rc::Rc;

//pub mod engine;
use engine::game_context::nodes::{
    camera::Camera3D,
    directional_light::DirectionalLight,
    model::{Model, Primitive},
    ui::UI,
};
use engine::game_context::GameContext;
use engine::renderer::shader::Shader;
use engine::Engine;
use glfw::Key;
use quaturn::egui::epaint::text::cursor;
use quaturn::egui::text_selection::text_cursor_state::cursor_rect;
use quaturn::engine::game_context::node_manager::{Node, NodeTransform};
use quaturn::engine::game_context::nodes::empty::Empty;
use quaturn::engine::renderer::shader;
use quaturn::{egui, engine, glfw, glm};

//use engine::Engine;

const WINDOW_WIDTH: u32 = 1280;
const WINDOW_HEIGHT: u32 = 720;

fn main() {
    let mut engine = Engine::init("top 10 windows", WINDOW_WIDTH, WINDOW_HEIGHT);

    engine.set_clear_color(0.0, 0.0, 0.0, 1.0);

    let mut cursor_locked = false;

    let toggle_cursor_lock = |context: &mut GameContext, lock: bool| {
        context.lock_cursor(lock);
    };

    // engine
    //     .context
    //     .nodes
    //     .add("model", Model::new_gltf("res/scenes/japan/scene.gltf"))
    //     //.scale(glm::vec3(0.1, 0.1, 0.1))
    //     .define_ready(|_model| {
    //         //ran before the first frame
    //         println!("model ready");
    //     })
    //     .define_behavior(move |model, context| {
    //         //ran every frame
    //         //println!("model behavior");
    //     })
    //     .rotate_euler_xyz(glm::vec3(-90.0, 0.0, 0.0)); // z+ to y+

    engine
        .context
        .nodes
        .add("plane", Model::new_primitive(Primitive::Plane))
        .apply_transform(|t| {
            t.set_scale(glm::vec3(20.0, 20.0, 20.0));
            t.set_position(glm::vec3(0.0, -2.0, 0.0));
        });

    engine
        .context
        .nodes
        .add("cube", Model::new_primitive(Primitive::Teapot))
        .apply_transform(|t| {
            t.set_position(glm::vec3(0.0, 5.0, 0.0));
        })
        .define_behavior(move |cube, context| {
            cube.apply_transform(|t| {
                let time_delta = context.frame.time_delta.as_secs_f32();
                t.rotate_euler_xyz(glm::vec3(
                    time_delta / -2.0 * 50.0,
                    time_delta / 2.0 * 50.0,
                    time_delta * 50.0,
                ));
            });
        });

    engine.context.nodes.add(
        "Direct Light",
        DirectionalLight::new(
            glm::vec3(-1.0, 1.0, 1.0),
            glm::vec3(1.0, 1.0, 1.0),
            1.0,
            50.0,
            4096,
        ),
    );

    let camera_pos = glm::vec3(20.0, 20.0, 20.0);

    engine
        .context
        .nodes
        .add(
            "camera",
            Camera3D::new(
                camera_pos,
                (glm::vec3(0.0, 0.0, 1.0) - camera_pos).normalize(),
                0.78539,
                WINDOW_WIDTH as f32 / WINDOW_HEIGHT as f32,
                0.1,
                1000.0,
            ),
        )
        .define_ready(|_camera| {
            //ran before the first frame
            println!("camera ready");
        })
        .define_behavior(move |camera, context| {
            if cursor_locked {
                camera.take_input(&context.input, context.frame.time_delta.as_secs_f32());
            }

            if context
                .input
                .mouse_button_just_pressed
                .contains(&glfw::MouseButton::Button2)
            {
                cursor_locked = !cursor_locked;
                toggle_cursor_lock(context, cursor_locked);
            }
        });

    engine
        .context
        .nodes
        .add("game manager", Empty::new())
        .define_ready(|_game_manager| {
            //ran before the first frame
            println!("game manager ready");
        })
        .define_behavior(move |_game_manager, context| {
            //ran every frame
            if context.input.keys.contains(&glfw::Key::Escape) {
                context.window.set_should_close(true);
            }
        });

    let shader = engine.context.nodes.add_shader(
        "default",
        Shader::new(
            "res/shaders/default/default.vert",
            "res/shaders/default/default.frag",
            None,
        ),
    );

    shader.bind();
    shader.set_uniform4f("lightColor", 1.0, 1.0, 1.0, 1.0);

    let mut bias = 0.005;
    shader.set_uniform1f("u_bias", bias);

    let ui = UI::init(&mut engine.context.window);
    engine
        .context
        .nodes
        .add("debug_panel", ui)
        .define_ui(move |ctx, context| {
            //engine borrowed here

            //ui to be drawn every frame
            egui::Window::new("Debug Panel").show(ctx, |ui| {
                {
                    if let Some(camera)= context.nodes.get_mut::<Camera3D>("camera") {
                        let (mut camera_pos_x, mut camera_pos_y, mut camera_pos_z) = (
                            camera.get_position().x,
                            camera.get_position().y,
                            camera.get_position().z,
                        );

                        let (mut camera_rotation_x, mut camera_rotation_y, mut camera_rotation_z) = (
                            camera.get_orientation_angles().x,
                            camera.get_orientation_angles().y,
                            camera.get_orientation_angles().z,
                        );
                        ui.label("Hello World!");
                        if ui.button("print").clicked() {
                            println!("Hello World!");
                        }
                        ui.label("Camera Position");
                        ui.horizontal(|ui| {
                            ui.label("X:");
                            ui.add(egui::DragValue::new(&mut camera_pos_x));
                            ui.label("Y:");
                            ui.add(egui::DragValue::new(&mut camera_pos_y));
                            ui.label("Z:");
                            ui.add(egui::DragValue::new(&mut camera_pos_z));
                        });
                        ui.label("Camera Rotation");
                        ui.horizontal(|ui| {
                            ui.label("X:");
                            ui.add(egui::DragValue::new(&mut camera_rotation_x));
                            ui.label("Y:");
                            ui.add(egui::DragValue::new(&mut camera_rotation_y));
                            ui.label("Z:");
                            ui.add(egui::DragValue::new(&mut camera_rotation_z));
                        });
                        ui.add(
                            egui::Slider::new(&mut camera.move_speed, 0.0..=1000.0).text("Move Speed"),
                        );
                        //reassign camera position and rotation from ui
                        // camera.set_position(glm::vec3(camera_pos_x, camera_pos_y, camera_pos_z));
                        // camera.set_orientation_angles(glm::vec3(
                        //     camera_rotation_x,
                        //     camera_rotation_y,
                        //     camera_rotation_z,
                        // ));
                    }
                }
                {
                    //extract camera info
                    if let Some(light) = context.nodes.get_mut::<DirectionalLight>("Direct Light") {
                        let mut shadow_distance = light.get_far_plane();
                        ui.add(
                            egui::Slider::new(&mut shadow_distance, 0.0..=1000.0)
                                .text("Shadow Distance"),
                        );
                        light.set_far_plane(shadow_distance);
                    }
                }
                {
                    ui.add(egui::Slider::new(&mut bias, 0.0..=0.01).text("Shadow Bias"));
                    context
                        .nodes
                        .shaders
                        .get_mut(&context.nodes.active_shader)
                        .unwrap()
                        .set_uniform1f("u_bias", bias);
                }
            });
        });

    engine.begin();
}
