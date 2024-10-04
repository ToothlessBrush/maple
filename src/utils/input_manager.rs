use egui_backend::glfw;
use egui_gl_glfw as egui_backend;
use glfw::{GlfwReceiver, Key, MouseButton};
use std::collections::HashSet;

pub struct InputManager {
    glfw: glfw::Glfw,
    events: GlfwReceiver<(f64, glfw::WindowEvent)>,
    pub keys: HashSet<Key>, //this is a set of keys that are currently pressed
    pub key_just_pressed: HashSet<Key>, //this is a set of keys that were just pressed this frame
    pub mouse_buttons: HashSet<MouseButton>, //this is a set of mouse buttons that are currently pressed
    pub mouse_button_just_pressed: HashSet<MouseButton>, //this is a set of mouse buttons that were just pressed this frame
    pub mouse_position: glm::Vec2,                       //the current mouse position
    pub last_mouse_position: glm::Vec2,                  //the mouse position in the last frame
    pub mouse_delta: glm::Vec2, //the change in mouse position since the last frame
}

impl InputManager {
    pub fn new(events: GlfwReceiver<(f64, glfw::WindowEvent)>, glfw: glfw::Glfw) -> InputManager {
        InputManager {
            glfw: glfw,
            events: events,
            keys: HashSet::new(),
            key_just_pressed: HashSet::new(),
            mouse_buttons: HashSet::new(),
            mouse_button_just_pressed: HashSet::new(),
            mouse_position: glm::vec2(0.0, 0.0),
            last_mouse_position: glm::vec2(0.0, 0.0),
            mouse_delta: glm::vec2(0.0, 0.0),
        }
    }

    //updates the inputs every frame
    pub fn update(&mut self, egui_input: &mut egui_backend::EguiInputState) {
        self.glfw.poll_events();

        self.mouse_delta = self.mouse_position - self.last_mouse_position;
        self.last_mouse_position = self.mouse_position;

        self.key_just_pressed.clear(); //clear previous frame's keys
        self.mouse_button_just_pressed.clear(); //clear previous frame's mouse buttons

        for (_, event) in glfw::flush_messages(&self.events) {
            match event {
                glfw::WindowEvent::Key(key, _, action, _) => {
                    if action == glfw::Action::Press {
                        self.keys.insert(key);
                        self.key_just_pressed.insert(key); //add the key to the just pressed set
                    } else if action == glfw::Action::Release {
                        self.keys.remove(&key);
                    }
                }

                glfw::WindowEvent::MouseButton(button, action, _) => {
                    if action == glfw::Action::Press {
                        self.mouse_buttons.insert(button);
                        self.mouse_button_just_pressed.insert(button); //add the button to the just pressed set
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
            egui_backend::handle_event(event, egui_input);
        }
    }
}
