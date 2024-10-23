pub mod engine;

extern crate nalgebra_glm as glm;

use egui_gl_glfw::egui;
use engine::game_context::nodes::{camera::Camera3D, model::Model, ui::UI};
use engine::renderer::shader::Shader;
use engine::Engine;

const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 600;

fn main() {
    let mut engine = Engine::init("top 10 windows", WINDOW_WIDTH, WINDOW_HEIGHT);

    engine.set_clear_color(1.0, 1.0, 1.0, 1.0);

    engine
        .context
        .nodes
        .add_model("model", Model::new("res/scenes/japan/scene.gltf"))
        .rotate_euler_xyz(glm::Vec3::new(-90.0, 0.0, 0.0)) // y+ is up
        .define_ready(|model| {
            //ran before the first frame
            println!("model ready");
        })
        .define_behavior(|model, context| {
            //ran every frame
            //println!("model behavior");
        });

    engine
        .context
        .nodes
        .add_camera(
            "camera",
            Camera3D::new(
                glm::vec3(20.0, 20.0, 20.0),
                (glm::vec3(0.0, 0.0, 0.0) - glm::vec3(20.0, 20.0, 20.0)).normalize(),
                0.78539,
                WINDOW_WIDTH as f32 / WINDOW_HEIGHT as f32,
                0.1,
                1000.0,
            ),
        )
        .define_ready(|camera| {
            //ran before the first frame
            println!("camera ready");
        })
        .define_behavior(|camera, context| {
            camera.take_input(&context.input, context.frame.time_delta.as_secs_f32());
        });

    engine
        .context
        .nodes
        .add_shader("default", Shader::new("res/shaders/default"));

    let ui = UI::init(&mut engine.window);
    engine
        .context
        .nodes
        .add_ui("debug_panel", ui)
        .define_ui(move |ctx, context| {
            //engine borrowed here

            let camera: &mut Camera3D = context.nodes.get_camera("camera");
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

            //ui to be drawn every frame
            egui::Window::new("Debug Panel").show(ctx, |ui| {
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
            });
        });

    engine.begin(); //also borrowed here
}
