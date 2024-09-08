extern crate nalgebra_glm as glm;
extern crate stb_image;

use glfw::{Action, Context, Key};

//pub mod egui_backend;
pub mod graphics;
pub mod utils;

use graphics::buffers::{index_buffer, vertex_array, vertex_buffer, vertex_buffer_layout};
use graphics::game_object::{GameObject, Vertex};
use graphics::renderer::{debug_message_callback, Renderer};
use graphics::shader;
use graphics::texture;
use utils::camera::{Camera2D, Camera3D};
use utils::fps_manager::FPSManager;
use utils::input_manager;
use utils::rgb_color::Color;

use std::io::Write;

const MOVE_SPEED: f32 = 1.0; //pixels per second

const WINDOW_WIDTH: u32 = 924;
const WINDOW_HEIGHT: u32 = 580;

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

    // let positions = vec![
    //     Vertex {
    //         position: [-0.5, 0.0, 0.5],
    //         color: [1.0, 1.0, 1.0, 1.0],
    //         tex_coords: [0.0, 0.0],
    //         normals: [0.0, 0.0, 0.0],
    //     },
    //     Vertex {
    //         position: [-0.5, 0.0, -0.5],
    //         color: [1.0, 1.0, 1.0, 1.0],
    //         tex_coords: [5.0, 0.0],
    //         normals: [0.0, 0.0, 0.0],
    //     },
    //     Vertex {
    //         position: [0.5, 0.0, -0.5],
    //         color: [1.0, 1.0, 1.0, 1.0],
    //         tex_coords: [0.0, 0.0],
    //         normals: [0.0, 0.0, 0.0],
    //     },
    //     Vertex {
    //         position: [0.5, 0.0, 0.5],
    //         color: [1.0, 1.0, 1.0, 1.0],
    //         tex_coords: [5.0, 0.0],
    //         normals: [0.0, 0.0, 0.0],
    //     },
    //     Vertex {
    //         position: [0.0, 0.8, 0.0],
    //         color: [1.0, 1.0, 1.0, 1.0],
    //         tex_coords: [2.5, 5.0],
    //         normals: [0.0, 0.0, 0.0],
    //     },
    // ];

    // let indices: [u32; 18] = [2, 1, 0, 3, 2, 0, 0, 1, 4, 1, 2, 4, 2, 3, 4, 3, 0, 4]; //counter clockwise winding for front face

    let positions = vec![
        Vertex {
            position: [-0.5, 0.0, 0.5],
            color: [1.0, 1.0, 1.0, 1.0],
            tex_coords: [0.0, 0.0],
            normals: [0.0, -1.0, 0.0],
        },
        Vertex {
            position: [-0.5, 0.0, -0.5],
            color: [1.0, 1.0, 1.0, 1.0],
            tex_coords: [5.0, 0.0],
            normals: [0.0, -1.0, 0.0],
        },
        Vertex {
            position: [0.5, 0.0, -0.5],
            color: [1.0, 1.0, 1.0, 1.0],
            tex_coords: [5.0, 5.0],
            normals: [0.0, -1.0, 0.0],
        },
        Vertex {
            position: [0.5, 0.0, 0.5],
            color: [1.0, 1.0, 1.0, 1.0],
            tex_coords: [0.0, 5.0],
            normals: [0.0, -1.0, 0.0],
        },
        Vertex {
            position: [-0.5, 0.0, 0.5],
            color: [1.0, 1.0, 1.0, 1.0],
            tex_coords: [0.0, 0.0],
            normals: [-0.8, 0.5, 0.0],
        },
        Vertex {
            position: [-0.5, 0.0, -0.5],
            color: [1.0, 1.0, 1.0, 1.0],
            tex_coords: [5.0, 0.0],
            normals: [-0.8, 0.5, 0.0],
        },
        Vertex {
            position: [0.0, 0.8, 0.0],
            color: [1.0, 1.0, 1.0, 1.0],
            tex_coords: [2.5, 5.0],
            normals: [-0.8, 0.5, 0.0],
        },
        Vertex {
            position: [-0.5, 0.0, -0.5],
            color: [1.0, 1.0, 1.0, 1.0],
            tex_coords: [5.0, 0.0],
            normals: [0.0, 0.5, -0.8],
        },
        Vertex {
            position: [0.5, 0.0, -0.5],
            color: [1.0, 1.0, 1.0, 1.0],
            tex_coords: [0.0, 0.0],
            normals: [0.0, 0.5, -0.8],
        },
        Vertex {
            position: [0.0, 0.8, 0.0],
            color: [1.0, 1.0, 1.0, 1.0],
            tex_coords: [2.5, 5.0],
            normals: [0.0, 0.5, -0.8],
        },
        Vertex {
            position: [0.5, 0.0, -0.5],
            color: [1.0, 1.0, 1.0, 1.0],
            tex_coords: [0.0, 0.0],
            normals: [0.8, 0.5, 0.0],
        },
        Vertex {
            position: [0.5, 0.0, 0.5],
            color: [1.0, 1.0, 1.0, 1.0],
            tex_coords: [5.0, 0.0],
            normals: [0.8, 0.5, 0.0],
        },
        Vertex {
            position: [0.0, 0.8, 0.0],
            color: [1.0, 1.0, 1.0, 1.0],
            tex_coords: [2.5, 5.0],
            normals: [0.8, 0.5, 0.0],
        },
        Vertex {
            position: [0.5, 0.0, 0.5],
            color: [1.0, 1.0, 1.0, 1.0],
            tex_coords: [5.0, 0.0],
            normals: [0.0, 0.5, 0.8],
        },
        Vertex {
            position: [-0.5, 0.0, 0.5],
            color: [1.0, 1.0, 1.0, 1.0],
            tex_coords: [0.0, 0.0],
            normals: [0.0, 0.5, 0.8],
        },
        Vertex {
            position: [0.0, 0.8, 0.0],
            color: [1.0, 1.0, 1.0, 1.0],
            tex_coords: [2.5, 5.0],
            normals: [0.0, 0.5, 0.8],
        },
    ];

    let indices: [u32; 18] = [
        0, 1, 2, // Bottom side
        3, 0, 2, // Bottom side
        4, 6, 5, // Left side
        7, 9, 8, // Non-facing side
        10, 12, 11, // Right side
        13, 15, 14, // Facing side
    ];

    unsafe {
        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
    }

    let mut pyramid: GameObject =
        GameObject::new(positions, indices.to_vec(), "res/textures/brick.png");

    let mut light: GameObject = GameObject::new(
        vec![
            Vertex {
                position: [0.1, 0.1, 0.1],
                color: [1.0, 1.0, 1.0, 1.0],
                tex_coords: [0.0, 0.0],
                normals: [0.0, 0.0, 0.0],
            },
            Vertex {
                position: [0.1, 0.1, -0.1],
                color: [1.0, 1.0, 1.0, 1.0],
                tex_coords: [0.0, 0.0],
                normals: [0.0, 0.0, 0.0],
            },
            Vertex {
                position: [0.1, -0.1, 0.1],
                color: [1.0, 1.0, 1.0, 1.0],
                tex_coords: [0.0, 0.0],
                normals: [0.0, 0.0, 0.0],
            },
            Vertex {
                position: [0.1, -0.1, -0.1],
                color: [1.0, 1.0, 1.0, 1.0],
                tex_coords: [0.0, 0.0],
                normals: [0.0, 0.0, 0.0],
            },
            Vertex {
                position: [-0.1, 0.1, 0.1],
                color: [1.0, 1.0, 1.0, 1.0],
                tex_coords: [0.0, 0.0],
                normals: [0.0, 0.0, 0.0],
            },
            Vertex {
                position: [-0.1, 0.1, -0.1],
                color: [1.0, 1.0, 1.0, 1.0],
                tex_coords: [0.0, 0.0],
                normals: [0.0, 0.0, 0.0],
            },
            Vertex {
                position: [-0.1, -0.1, 0.1],
                color: [1.0, 1.0, 1.0, 1.0],
                tex_coords: [0.0, 0.0],
                normals: [0.0, 0.0, 0.0],
            },
            Vertex {
                position: [-0.1, -0.1, -0.1],
                color: [1.0, 1.0, 1.0, 1.0],
                tex_coords: [0.0, 0.0],
                normals: [0.0, 0.0, 0.0],
            },
        ],
        vec![
            //cube
            0, 2, 1, 1, 2, 3, 4, 5, 6, 5, 7, 6, 0, 1, 4, 1, 5, 4, 2, 6, 3, 6, 7, 3, 0, 4, 2, 4, 6,
            2, 1, 3, 5, 3, 7, 5,
        ],
        "res/textures/blank.png",
    );

    let lightPos: glm::Vec3 = glm::vec3(1.0, 1.0, 1.0);

    light.set_transform(glm::translate(
        &glm::Mat4::identity(),
        &glm::vec3(1.0, 1.0, 1.0),
    ));

    let mut lightShader = shader::Shader::new("res/shaders/light");

    let mut shader = shader::Shader::new("res/shaders/default");
    shader.bind(); // bind for uniforms
    shader.set_uniform4f("lightColor", 1.0, 1.0, 1.0, 1.0);
    shader.set_uniform3f("lightPos", lightPos.x, lightPos.y, lightPos.z);
    shader.unbind();

    // let texture = texture::Texture::new("res/textures/brick.png");
    // texture.bind(0);

    let mut renderer = Renderer::new(Camera3D::new(
        glm::vec3(0.0, 0.0, 2.0),
        0.785398,
        WINDOW_WIDTH as f32 / WINDOW_HEIGHT as f32,
        0.1,
        100.0,
    ));

    let colors = Color::from_hex(0x40739e);
    let black = Color::from_hex(0x000000);
    let mut fps_counter = FPSManager::new();
    //let mut keys_pressed = std::collections::HashSet::new();

    let mut angle = 0.0;

    let mut input_manager = input_manager::InputManager::new(events, glfw);

    // Loop until the user closes the window
    while !window.should_close() {
        // Update the FPS counter
        fps_counter.update(|fps| {
            window.set_title(&format!("Top 10 Windows Ever Made | FPS: {}", fps));
        });

        let cam_pos: glm::Vec3 = renderer.camera.get_position();

        // et uniforms
        shader.bind();
        shader.set_uniform3f("camPos", cam_pos.x, cam_pos.y, cam_pos.z);
        shader.unbind();

        // Render here
        renderer.clear(black.to_tuple());

        renderer.draw_object(&pyramid, &mut shader);

        renderer.draw_object(&light, &mut lightShader);

        pyramid.set_transform(
            glm::translate(&glm::Mat4::identity(), &glm::vec3(0.0, 0.0, 0.0))
                * glm::rotate(&glm::Mat4::identity(), angle, &glm::vec3(0.0, 1.0, 0.0)),
        );
        // angle += 1.0 * fps_counter.time_delta.as_secs_f32();

        if angle >= 360.0 {
            angle = 0.0;
        }

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
