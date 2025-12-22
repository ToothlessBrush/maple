use std::{
    any::{Any, TypeId},
    cell::{Ref, RefCell, RefMut},
    collections::HashMap,
};

use winit::event::{DeviceEvent, WindowEvent};

use crate::{
    components::event_reciever::EventLabel, context::FPSManager, input::InputManager, scene::Scene,
};

pub trait Resource: Any {}

/// The main game context, containing all the necessary information for the game to run.
/// This includes the window, the nodes, the frame manager, the input manager, and the shadow distance.
#[derive(Default)]
pub struct GameContext {
    /// The node manager of the game.
    pub scene: Scene,

    resources: HashMap<TypeId, RefCell<Box<dyn Any>>>,
}

impl GameContext {
    /// Creates a new game context with the given events, glfw, and window.
    ///
    /// # Arguments
    /// - `events` - The input events of the game.
    /// - `glfw` - The glfw context of the game.
    /// - `window` - The window of the game.
    ///
    /// # Returns
    /// The new game context.
    pub fn new() -> GameContext {
        GameContext {
            scene: Scene::new(),
            resources: HashMap::new(),
        }
    }

    pub fn device_event(&mut self, event: &DeviceEvent) {
        if let Some(mut input) = self.get_resource_mut::<InputManager>() {
            input.handle_device_event(event);
        }
    }

    pub fn window_event(&mut self, event: &WindowEvent) {
        if let Some(mut input) = self.get_resource_mut::<InputManager>() {
            input.handle_event(event);
        }
    }

    pub fn begin_frame(&mut self) {
        if let Some(mut frame) = self.get_resource_mut::<FPSManager>() {
            frame.update();
        }
    }

    pub fn end_frame(&mut self) {
        if let Some(mut input) = self.get_resource_mut::<InputManager>() {
            input.end_frame();
        }
    }

    pub fn get_resource<R: Resource>(&self) -> Option<Ref<'_, R>> {
        let id = TypeId::of::<R>();
        let cell = self.resources.get(&id)?;

        match cell.try_borrow() {
            Ok(borrowed) => Some(Ref::map(borrowed, |b| {
                b.downcast_ref::<R>()
                    .expect("Resource type mismatch - this should never happen")
            })),
            Err(_) => {
                eprintln!(
                    "Failed to borrow resource {} (already borrowed mutably)",
                    std::any::type_name::<R>()
                );
                None
            }
        }
    }

    pub fn get_resource_mut<R: Resource>(&self) -> Option<RefMut<'_, R>> {
        let id = TypeId::of::<R>();
        let cell = self.resources.get(&id)?;

        match cell.try_borrow_mut() {
            Ok(borrowed) => Some(RefMut::map(borrowed, |b| {
                b.downcast_mut::<R>()
                    .expect("Resource type mismatch - this should never happen")
            })),
            Err(_) => {
                eprintln!(
                    "Failed to mutably borrow resource {} (already borrowed)",
                    std::any::type_name::<R>()
                );
                None
            }
        }
    }

    pub fn insert_resource<R: Resource>(&mut self, resource: R) {
        let id = TypeId::of::<R>();
        self.resources.insert(id, RefCell::new(Box::new(resource)));
    }

    pub fn with_resource_and_scene<R: Resource, F>(&mut self, mut f: F)
    where
        F: FnMut(&mut R, &mut Scene),
    {
        let id = TypeId::of::<R>();
        if let Some(cell) = self.resources.get(&id) {
            match cell.try_borrow_mut() {
                Ok(mut borrowed) => {
                    if let Some(resource) = borrowed.downcast_mut::<R>() {
                        f(resource, &mut self.scene);
                    }
                }
                Err(_) => {
                    eprintln!(
                        "Failed to mutably borrow resource {} in with_resource_and_scene (already borrowed)",
                        std::any::type_name::<R>()
                    );
                }
            }
        }
    }

    /// emits an event to the currently loaded nodes in the context
    ///
    /// # Arguments
    /// - `event` - which event to emit
    ///
    /// # example
    /// ```rust
    /// use maple::{
    ///     Engine, config::EngineConfig,
    ///     components::Event,
    /// };
    ///
    /// # use std::error::Error;
    ///
    /// let mut engine = Engine::init(EngineConfig::default())?;
    ///
    /// // emit some custom event such as a damage event
    /// engine.context.emit(Event::Custom("damage".to_string()));
    /// # Ok::<(), Box<dyn Error>>(())
    /// ```
    pub fn emit<E: EventLabel>(&mut self, event: E) {
        let nodes = &mut self.scene as *mut Scene;

        // we need to pass self when we are borrowing self.nodes and idk another solution

        unsafe { (*nodes).emit(&event, self) }
    }

    // /// set the main camea of the engine
    // ///
    // /// TODO remove raw pointer
    // pub fn set_main_camera(&mut self, camera: *const Camera3D) {
    //     let mut search_path = Vec::<String>::new();

    //     // Iterate through the nodes and try to find the camera path.
    //     for node in &mut self.scene {
    //         if let Some(path) = Self::traverse_nodes(node, Vec::new(), camera) {
    //             search_path = path;
    //             break; // Exit once the camera is found
    //         }
    //     }

    //     if search_path.is_empty() {
    //         println!("no matching result");
    //     } else {
    //         println!("camera found at path: {:?}", search_path);
    //         self.active_camera_path = search_path;
    //     }
    // }

    /// time since last frame. this is really useful if you want smooth movement
    ///
    /// by multiplying somthing that is frame dependant such as a transform it will move at a
    /// consistant speed even if the frame rate is different
    ///
    /// # example
    /// ```rust
    /// use maple::{
    ///     components::Event,
    ///     math,
    ///     nodes::{Builder, Buildable, Empty}
    /// };
    ///
    /// Empty::builder()
    ///     .on(Event::Update, |node, ctx| {
    ///         node.transform.rotate_euler_xyz(math::vec3(0.0, 90.0 * ctx.time_delta(), 0.0));
    ///     })
    ///     .build();
    /// ```
    pub fn time_delta(&self) -> f32 {
        if let Some(frame) = self.get_resource::<FPSManager>() {
            frame.time_delta_f32
        } else {
            eprintln!("couldnt get time delta from fps manager");
            0.0
        }
    }

    // fn traverse_nodes(
    //     node: (&String, &mut Box<dyn Node>),
    //     parent_path: Vec<String>,
    //     camera: *const Camera3D,
    // ) -> Option<Vec<String>> {
    //     let mut current_path = parent_path.clone();
    //     current_path.push(node.0.clone());

    //     // Check if the current node is the camera we're searching for
    //     if let Some(current_camera) = node.1.as_any().downcast_ref::<Camera3D>() {
    //         if std::ptr::eq(current_camera, camera) {
    //             return Some(current_path); // Return the path if camera matches
    //         }
    //     }

    //     // Recursively check each child node
    //     for child in node.1.get_children_mut() {
    //         if let Some(path) = Self::traverse_nodes(child, current_path.clone(), camera) {
    //             return Some(path); // Return path if camera is found in child
    //         }
    //     }

    //     None // Return None if the camera is not found in this node or its children
    // }
}
