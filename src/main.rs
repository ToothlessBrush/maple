extern crate nalgebra_glm as glm;
extern crate stb_image;

use egui_backend::egui;
use egui_backend::gl;
use egui_backend::glfw;
use egui_gl_glfw as egui_backend;

use glfw::{Action, Context, Key};

//pub mod egui_backend;
pub mod graphics;
pub mod utils;

use graphics::camera::{Camera2D, Camera3D};
use graphics::model::Model;
use graphics::renderer::{debug_message_callback, Renderer};
use graphics::shader;
use graphics::texture::{self, Texture};
use utils::fps_manager::FPSManager;
use utils::input_manager;
use utils::rgb_color::Color;

const WINDOW_WIDTH: u32 = 1920;
const WINDOW_HEIGHT: u32 = 1080;
const PIC_WIDTH: u32 = 320;
const PIC_HEIGHT: u32 = 192;

fn main() {
    use glfw::fail_on_errors;
    let mut glfw = glfw::init(fail_on_errors!()).unwrap();

    glfw.window_hint(glfw::WindowHint::ContextVersion(4, 6));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(
        glfw::OpenGlProfileHint::Core,
    ));
    glfw.window_hint(glfw::WindowHint::DoubleBuffer(true));
    glfw.window_hint(glfw::WindowHint::Resizable(false));

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
    window.set_char_polling(true);
    window.set_key_polling(true);
    window.set_cursor_pos_polling(true);
    window.set_mouse_button_polling(true);
    window.make_current();
    glfw.set_swap_interval(glfw::SwapInterval::Sync(1));

    gl::load_with(|s| window.get_proc_address(s) as *const _);

    let mut cursor_enabled: bool = false;
    window.set_cursor_mode(glfw::CursorMode::Disabled);

    let (width, height) = window.get_framebuffer_size();
    let native_pixels_per_point = window.get_content_scale().0;

    glfw.set_swap_interval(glfw::SwapInterval::Sync(0));

    //init egui
    let mut painter = egui_backend::Painter::new(&mut window);
    let egui_ctx = egui::Context::default();
    //create the egui input state
    let mut egui_input = egui_backend::EguiInputState::new(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::new(0f32, 0f32),
                egui::vec2(width as f32, height as f32) / native_pixels_per_point,
            )),
            ..Default::default()
        },
        native_pixels_per_point,
    );

    let srgba = vec![egui::Color32::BLACK; (PIC_HEIGHT * PIC_WIDTH) as usize];

    let plot_tex_id = painter.new_user_texture(
        (PIC_WIDTH as usize, PIC_HEIGHT as usize),
        &srgba,
        egui::TextureFilter::Linear,
    );
    let mut sine_shift = 0f32;
    let amplitude = 50f32;

    unsafe {
        //gl::Enable(gl::DEBUG_OUTPUT);
        //gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
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

    let camera_pos = glm::vec3(1.0, 1.0, 1.0);

    //create renderer
    let mut renderer = Renderer::new(Camera3D::new(
        camera_pos,
        glm::vec3(0.0, 0.0, 1.0), //glm::normalize(&(glm::vec3(0.0, 0.0, 0.0) - camera_pos)), //look at the origin
        0.785398,
        WINDOW_WIDTH as f32 / WINDOW_HEIGHT as f32,
        0.1,
        1000.0,
    ));

    let colors = Color::from_hex(0x1c1c1c);
    let black = Color::from_hex(0x000000);
    let grey = Color::from_vec3(glm::vec3(0.85, 0.85, 0.90));
    // Create a new FPS counter
    let mut fps_counter = FPSManager::new();
    // Create a new input manager
    let mut input_manager = input_manager::InputManager::new(events, glfw);

    let mut fps_history: Vec<f32> = Vec::new();
    const MAX_HISTORY: usize = 1000;

    // Loop until the user closes the window
    while !window.should_close() {
        input_manager.update(&mut egui_input);

        renderer
            .camera
            .take_input(&input_manager, fps_counter.time_delta.as_secs_f32());

        //update egui
        egui_input.input.time = Some(fps_counter.start_time.elapsed().as_secs_f64());
        egui_ctx.begin_frame(egui_input.input.take());
        egui_input.pixels_per_point = native_pixels_per_point;

        // Update the FPS counter
        let fps = 1.0 / fps_counter.time_delta.as_secs_f32();
        fps_counter.update(|fps| {
            window.set_title(&format!("Top 10 Windows Ever Made | FPS: {}", fps));
        });

        // Render here
        renderer.clear(grey.to_tuple());

        model.draw(&mut shader, &renderer.camera);

        // define ui variables
        let mut camera_pos: (f32, f32, f32) = (
            renderer.camera.get_position().x,
            renderer.camera.get_position().y,
            renderer.camera.get_position().z,
        );

        let mut camera_rot: (f32, f32, f32) = (
            renderer.camera.get_orientation_angles().x,
            renderer.camera.get_orientation_angles().y,
            renderer.camera.get_orientation_angles().z,
        );

        let mut srgba: Vec<egui::Color32> = Vec::new();
        let mut angle = 0f32;

        for y in 0..PIC_HEIGHT {
            for x in 0..PIC_WIDTH {
                srgba.push(egui::Color32::BLACK);
                if y == PIC_HEIGHT - 1 {
                    let y = amplitude * (angle * std::f32::consts::PI / 180f32 + sine_shift).sin();
                    let y = PIC_HEIGHT as f32 / 2f32 - y;
                    srgba[(y as i32 * PIC_WIDTH as i32 + x as i32) as usize] =
                        egui::Color32::YELLOW;
                    angle += 360f32 / PIC_WIDTH as f32;
                }
            }
        }
        sine_shift += 0.1f32;
        painter.update_user_texture_data(&plot_tex_id, &srgba);

        unsafe {
            gl::Disable(gl::DEPTH_TEST);
            gl::Disable(gl::CULL_FACE);
            gl::Disable(gl::BLEND);
        }
        egui::Window::new("").show(&egui_ctx, |ui| {
            //ui.image((plot_tex_id, egui::vec2(PIC_WIDTH as f32, PIC_HEIGHT as f32)));

            ui.label(format!("FPS: {:.0}", fps));

            ui.add(egui::Image::new(egui::load::SizedTexture {
                id: plot_tex_id,
                size: egui::vec2(PIC_WIDTH as f32, PIC_HEIGHT as f32),
            }));
            // Render the FPS graph

            ui.group(|ui| {
                ui.label("Camera");
                ui.group(|ui| {
                    ui.label("Transform");
                    ui.separator();
                    ui.label("Camera Position");
                    ui.horizontal(|ui| {
                        ui.label("X:");
                        ui.add(egui::DragValue::new(&mut camera_pos.0));
                        ui.label("Y:");
                        ui.add(egui::DragValue::new(&mut camera_pos.1));
                        ui.label("Z:");
                        ui.add(egui::DragValue::new(&mut camera_pos.2));
                    });
                    ui.label("Camera Rotation");
                    ui.horizontal(|ui| {
                        ui.label("X:");
                        ui.add(egui::DragValue::new(&mut camera_rot.0));
                        ui.label("Y:");
                        ui.add(egui::DragValue::new(&mut camera_rot.1));
                        ui.label("Z:");
                        ui.add(egui::DragValue::new(&mut camera_rot.2));
                    })
                });
                ui.add(egui::Slider::new(&mut renderer.camera.fov, 0.1..=3.14).text("FOV"));
                ui.add(
                    egui::Slider::new(&mut renderer.camera.look_sensitivity, 0.0..=1.0)
                        .text("Look Sensitivity"),
                );
                ui.add(
                    egui::Slider::new(&mut renderer.camera.move_speed, 0.0..=1000.0)
                        .text("Move Speed"),
                );
                if ui.button("Quit").clicked() {
                    window.set_should_close(true);
                }
            });
        });

        // update changed ui variables
        renderer
            .camera
            .set_position(glm::vec3(camera_pos.0, camera_pos.1, camera_pos.2));
        renderer
            .camera
            .set_orientation_angles(glm::vec3(camera_rot.0, camera_rot.1, camera_rot.2));

        let egui::FullOutput {
            platform_output,
            textures_delta,
            shapes,
            pixels_per_point,
            viewport_output: _,
        } = egui_ctx.end_frame();

        //put copied text into clipboard
        if !platform_output.copied_text.is_empty() {
            egui_backend::copy_to_clipboard(&mut egui_input, platform_output.copied_text);
        }

        let clipped_shapes = egui_ctx.tessellate(shapes, pixels_per_point);
        painter.paint_and_update_textures(1.0, &clipped_shapes, &textures_delta);

        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::Enable(gl::CULL_FACE);
            gl::Enable(gl::BLEND);
        }

        if input_manager.keys.contains(&Key::Escape) {
            window.set_should_close(true);
        }

        if input_manager
            .mouse_button_just_pressed
            .contains(&glfw::MouseButton::Button3)
        {
            window.set_cursor_mode(if cursor_enabled {
                renderer.camera.movement_enabled = true;
                cursor_enabled = false;
                glfw::CursorMode::Disabled
            } else {
                cursor_enabled = true;
                renderer.camera.movement_enabled = false;
                glfw::CursorMode::Normal
            });
        }

        window.swap_buffers();
        //glfw.poll_events();
        //colors.increment(0.01);
    }
}
