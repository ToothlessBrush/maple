pub mod engine;

extern crate nalgebra_glm as glm;

use egui_gl_glfw::egui;
use engine::{
    nodes::{camera::Camera3D, model::Model, ui::UI},
    renderer::shader::Shader,
    utils::{fps_manager, input_manager},
    Engine,
};

const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 600;

fn main() {
    let mut engine = Engine::init("top 10 windows", WINDOW_WIDTH, WINDOW_HEIGHT);

    engine
        .add_model("japan", Model::new("res/scenes/japan/scene.gltf"))
        .rotate_euler_xyz(glm::Vec3::new(0.0, 0.0, -90.0)) // y+ is up
        .define_ready(|model| {
            //ran before the first frame
            println!("model ready");
        })
        .define_behavior(|model, fps_manager, input_manager| {
            //ran every frame
            //println!("model behavior");
        });

    engine
        .add_camera(
            "camera",
            Camera3D::new(
                glm::vec3(10.0, 10.0, 10.0),
                glm::vec3(0.0, 0.0, 1.0),
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
        .define_behavior(|camera, fps_manager, input_manager| {
            //ran every frame
            //println!("camera behavior");
        });

    engine.add_shader("default", Shader::new("res/shaders/default"));

    let ui = UI::init(&mut engine.window);
    engine.add_ui("debug_panel", ui).define_ui(|ctx| {
        //ui to be drawn every frame
        egui::Window::new("Debug Panel").show(ctx, |ui| {
            ui.label("Hello World!");
        });
    });

    engine.begin();
}
