use nalgebra_glm as glm; // Importing the nalgebra_glm crate for mathematical operations

use egui_backend::glfw;
use egui_gl_glfw as egui_backend;
use glfw::{GlfwReceiver, Key, MouseButton};
use std::collections::HashSet;

pub struct InputManager {
    glfw: glfw::Glfw,
    event_receiver: GlfwReceiver<(f64, glfw::WindowEvent)>,
    pub events: Vec<(f64, glfw::WindowEvent)>, //store events for the current frame
    pub keys: HashSet<Key>,                    //this is a set of keys that are currently pressed
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
            event_receiver: events,
            events: Vec::new(), //initialize with a default event
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
    pub fn update(&mut self) {
        self.glfw.poll_events();

        self.mouse_delta = self.mouse_position - self.last_mouse_position;
        self.last_mouse_position = self.mouse_position;

        self.key_just_pressed.clear(); //clear previous frame's keys
        self.mouse_button_just_pressed.clear(); //clear previous frame's mouse buttons

        self.events.clear(); //clear previous frame's events
        self.events = glfw::flush_messages(&self.event_receiver).collect();

        for (_, event) in self.events.iter() {
            match event {
                glfw::WindowEvent::Key(key, _, action, _) => {
                    if *action == glfw::Action::Press {
                        self.keys.insert(*key);
                        self.key_just_pressed.insert(*key); //add the key to the just pressed set
                    } else if *action == glfw::Action::Release {
                        self.keys.remove(key);
                    }
                }

                glfw::WindowEvent::MouseButton(button, action, _) => {
                    if *action == glfw::Action::Press {
                        self.mouse_buttons.insert(*button);
                        self.mouse_button_just_pressed.insert(*button); //add the button to the just pressed set
                    } else if *action == glfw::Action::Release {
                        self.mouse_buttons.remove(button);
                    }
                }
                glfw::WindowEvent::CursorPos(x, y) => {
                    self.mouse_position = glm::vec2(*x as f32, *y as f32);
                    //println!("Mouse position: {:?}", self.mouse_position);
                }
                _ => {}
            }
        }
    }
}
