extern crate nalgebra_glm as glm;
extern crate stb_image;

use glfw::{Action, Context, Key};

//pub mod egui_backend;
pub mod graphics;
pub mod utils;

use graphics::buffers::{
    index_buffer, vertex_array, vertex_buffer, vertex_buffer::Vertex, vertex_buffer_layout,
};
use graphics::camera::{Camera2D, Camera3D};
use graphics::mesh::Mesh;
use graphics::model::Model;
use graphics::renderer::{debug_message_callback, Renderer};
use graphics::shader;
use graphics::texture::{self, Texture};
use utils::fps_manager::FPSManager;
use utils::input_manager;
use utils::rgb_color::Color;

use std::io::Write;

const WINDOW_WIDTH: u32 = 1920;
const WINDOW_HEIGHT: u32 = 1080;

fn main() {
    use glfw::fail_on_errors;
    let mut glfw = glfw::init(fail_on_errors!()).unwrap();

    glfw.window_hint(glfw::WindowHint::Samples(Some(4)));

    //create window with gl context
    let (mut window, events) = glfw
        .create_window(
            WINDOW_WIDTH,
            WINDOW_HEIGHT,
            "Top 10 Windows Ever Made",
            glfw::WindowMode::Windowed,
        )
        .expect("Failed to create GLFW window.");

    //window.make_current();

    //input polling
    window.set_key_polling(true);
    window.set_cursor_pos_polling(true);
    window.set_mouse_button_polling(true);

    window.set_cursor_mode(glfw::CursorMode::Disabled);

    //init gl and load the opengl function pointers
    gl::load_with(|s| window.get_proc_address(s) as *const _);

    unsafe {
        gl::Enable(gl::DEBUG_OUTPUT);
        gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
        gl::DebugMessageCallback(Some(debug_message_callback), std::ptr::null());
    }

    unsafe {
        gl::Enable(gl::DEPTH_TEST);
        gl::DepthFunc(gl::LESS);

        gl::Enable(gl::CULL_FACE);
        gl::CullFace(gl::BACK);
        gl::FrontFace(gl::CCW);
    }

    unsafe {
        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
    }

    let mut lightPos: glm::Vec3 = glm::vec3(0.0, 1.0, 0.0);
    let mut light_model: glm::Mat4 = glm::identity();
    light_model = glm::translate(&light_model, &lightPos);

    //light.set_transform(glm::translate(&glm::Mat4::identity(), &lightPos));

    // let mut lightShader = shader::Shader::new("res/shaders/light");
    // lightShader.bind();
    // lightShader.set_uniform_mat4f("u_Model", &light_model);
    // lightShader.unbind();

    //build the shader and set lighting info
    let mut shader = shader::Shader::new("res/shaders/default");
    shader.bind(); // bind for uniforms
    shader.set_uniform4f("lightColor", 1.0, 1.0, 1.0, 1.0);
    shader.set_uniform3f("lightPos", lightPos.x, lightPos.y, lightPos.z);
    shader.unbind();

    //load the model
    let mut model = Model::new("res/scenes/japan/scene.gltf");
    model.rotate(glm::vec3(1.0, 0.0, 0.0), -90.0); //convert from gltf to opengl coordinate system (y+ is up) idk why its different it the same company

    //create renderer
    let mut renderer = Renderer::new(Camera3D::new(
        glm::vec3(0.0, 0.0, 2.0),
        0.785398,
        WINDOW_WIDTH as f32 / WINDOW_HEIGHT as f32,
        0.1,
        1000.0,
    ));

    let colors = Color::from_hex(0x1c1c1c);
    let black = Color::from_hex(0x000000);

    // Create a new FPS counter
    let mut fps_counter = FPSManager::new();
    // Create a new input manager
    let mut input_manager = input_manager::InputManager::new(events, glfw);

    // Loop until the user closes the window
    while !window.should_close() {
        // Update the FPS counter
        fps_counter.update(|fps| {
            window.set_title(&format!("Top 10 Windows Ever Made | FPS: {}", fps));
        });

        // Render here
        renderer.clear(colors.to_tuple());

        model.draw(&mut shader, &renderer.camera);
        // model.rotate(
        //     glm::vec3(0.0, 1.0, 0.0),
        //     45.0 * fps_counter.time_delta.as_secs_f32(),
        // ); //rotate the model

        input_manager.update();

        if input_manager.keys.contains(&Key::Escape) {
            window.set_should_close(true);
        }

        renderer
            .camera
            .take_input(&input_manager, fps_counter.time_delta.as_secs_f32());

        window.swap_buffers();
        //glfw.poll_events();

        //colors.increment(0.01);
    }
}
