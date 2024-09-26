use egui_gl_glfw::glfw;
use glfw::{GlfwReceiver, Key, MouseButton};
use std::collections::HashSet;

pub struct InputManager {
    glfw: glfw::Glfw,
    events: GlfwReceiver<(f64, glfw::WindowEvent)>,
    pub keys: HashSet<Key>,
    pub mouse_buttons: HashSet<MouseButton>,
    pub mouse_position: glm::Vec2,
    pub last_mouse_position: glm::Vec2,
}

impl InputManager {
    pub fn new(events: GlfwReceiver<(f64, glfw::WindowEvent)>, glfw: glfw::Glfw) -> InputManager {
        InputManager {
            glfw: glfw,
            events: events,
            keys: HashSet::new(),
            mouse_buttons: HashSet::new(),
            mouse_position: glm::vec2(0.0, 0.0),
            last_mouse_position: glm::vec2(0.0, 0.0),
        }
    }

    //updates the inputs every frame
    pub fn update(&mut self) {
        self.glfw.poll_events();

        self.last_mouse_position = self.mouse_position;

        for (_, event) in glfw::flush_messages(&self.events) {
            //println!("{:?}", event);
            match event {
                glfw::WindowEvent::Key(key, _, action, _) => {
                    if action == glfw::Action::Press {
                        self.keys.insert(key);
                    } else if action == glfw::Action::Release {
                        self.keys.remove(&key);
                    }
                }

                glfw::WindowEvent::MouseButton(button, action, _) => {
                    if action == glfw::Action::Press {
                        self.mouse_buttons.insert(button);
                    } else if action == glfw::Action::Release {
                        self.mouse_buttons.remove(&button);
                    }
                }
                glfw::WindowEvent::CursorPos(x, y) => {
                    self.mouse_position = glm::vec2(x as f32, y as f32);
                    //println!("Mouse position: {:?}", self.mouse_position);
                }
                _ => {}
            }
        }
    }
}
