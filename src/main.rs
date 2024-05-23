extern crate glfw;
use glfw::{Action, Context, Key};

use std::time::{Duration, Instant};

pub mod graphics;

use graphics::buffers::index_buffer;
use graphics::buffers::vertex_array;
use graphics::buffers::{vertex_buffer, vertex_buffer_layout};
use graphics::renderer::debug_message_callback;
use graphics::shader;

fn main() {
    use glfw::fail_on_errors;
    let mut glfw = glfw::init(fail_on_errors!()).unwrap();

    // Create a windowed mode window and its OpenGL context
    let (mut window, events) = glfw
        .create_window(
            800,
            800,
            "Top 10 Windows Ever Made",
            glfw::WindowMode::Windowed,
        )
        .expect("Failed to create GLFW window.");

    // Make the window's context current
    window.make_current();
    window.set_key_polling(true);

    //init gl and load the opengl function pointers

    gl::load_with(|s| window.get_proc_address(s) as *const _);

    unsafe {
        gl::Enable(gl::DEBUG_OUTPUT);
        gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
        gl::DebugMessageCallback(Some(debug_message_callback), std::ptr::null());
    }

    let positions: [f32; 8] = [-0.5, -0.5, 0.5, -0.5, 0.5, 0.5, -0.5, 0.5];

    let indices: [u32; 6] = [0, 1, 2, 2, 3, 0];

    let va = vertex_array::VertexArray::new();
    va.bind();
    let vb = vertex_buffer::VertexBuffer::new(&positions);

    let mut layout = vertex_buffer_layout::VertexBufferLayout::new();
    layout.push::<f32>(2);
    va.add_buffer(&vb, &layout);

    let ib = index_buffer::IndexBuffer::new(&indices);

    let mut shader = shader::Shader::new("res/shaders");
    shader.bind();
    shader.set_uniform4f("u_Color", 0.2, 0.8, 1.0, 1.0);

    va.unbind();
    vb.unbind();
    ib.unbind();
    shader.unbind();

    let mut colors = Color::new(1.0, 0.0, 0.0);

    // Create an FPS counter
    let mut fps_counter = FPSCounter::new();

    // Loop until the user closes the window
    while !window.should_close() {
        // Update the FPS counter
        fps_counter.update(|fps| {
            window.set_title(&format!("Top 10 Windows Ever Made | FPS: {}", fps));
        });

        // Render here
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        //bind shader program
        shader.bind();
        shader.set_uniform4f("u_Color", colors.r, colors.g, colors.b, 1.0);

        //bind vertex array and index buffer
        va.bind();
        ib.bind();

        // Draw the triangles
        unsafe {
            gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, std::ptr::null());
        }

        // Swap front and back buffers
        window.swap_buffers();

        // Poll for and process events
        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            println!("{:?}", event);
            match event {
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    window.set_should_close(true)
                }
                _ => {}
            }
        }

        colors.increment(0.01);
    }
}

struct FPSCounter {
    frame_count: u32,
    last_time: Instant,
}

impl FPSCounter {
    fn new() -> Self {
        FPSCounter {
            frame_count: 0,
            last_time: Instant::now(),
        }
    }

    fn update<T: FnMut(u32)>(&mut self, mut update_fn: T) {
        self.frame_count += 1;
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_time);
        if elapsed >= Duration::from_secs(1) {
            update_fn(self.frame_count);
            self.frame_count = 0;
            self.last_time = now;
        }
    }
}

struct Color {
    r: f32,
    g: f32,
    b: f32,
}

impl Color {
    // Create a new Color
    fn new(r: f32, g: f32, b: f32) -> Self {
        Color { r, g, b }
    }

    // Method to increment the color around the color wheel
    fn increment(&mut self, step: f32) {
        if self.r == 1.0 && self.g < 1.0 && self.b == 0.0 {
            // Red to Yellow (increment green)
            self.g = (self.g + step).min(1.0);
        } else if self.g == 1.0 && self.r > 0.0 && self.b == 0.0 {
            // Yellow to Green (decrement red)
            self.r = (self.r - step).max(0.0);
        } else if self.g == 1.0 && self.b < 1.0 && self.r == 0.0 {
            // Green to Cyan (increment blue)
            self.b = (self.b + step).min(1.0);
        } else if self.b == 1.0 && self.g > 0.0 && self.r == 0.0 {
            // Cyan to Blue (decrement green)
            self.g = (self.g - step).max(0.0);
        } else if self.b == 1.0 && self.r < 1.0 && self.g == 0.0 {
            // Blue to Magenta (increment red)
            self.r = (self.r + step).min(1.0);
        } else if self.r == 1.0 && self.b > 0.0 && self.g == 0.0 {
            // Magenta to Red (decrement blue)
            self.b = (self.b - step).max(0.0);
        }
    }
}
