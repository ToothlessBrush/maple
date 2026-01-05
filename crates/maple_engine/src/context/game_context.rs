use std::{
    any::{Any, TypeId},
    cell::{Ref, RefCell, RefMut},
    collections::HashMap,
};

use winit::event::{DeviceEvent, WindowEvent};

use crate::{components::EventLabel, context::FPSManager, input::InputManager, scene::Scene};

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
        self.get_resource_mut::<InputManager>()
            .handle_device_event(event)
    }

    pub fn window_event(&mut self, event: &WindowEvent) {
        self.get_resource_mut::<InputManager>().handle_event(event);
    }

    pub fn begin_frame(&mut self) {
        self.get_resource_mut::<FPSManager>().update();
    }

    pub fn end_frame(&mut self) {
        self.get_resource_mut::<InputManager>().end_frame();
    }

    pub fn get_resource<R: Resource>(&self) -> Ref<'_, R> {
        let id = TypeId::of::<R>();
        let name = std::any::type_name::<R>();

        let cell = self.resources.get(&id).unwrap_or_else(|| {
            panic!("Resource: {name} not found (did you forget to add its plugin?)")
        });

        cell.try_borrow()
            .unwrap_or_else(|_| panic!("{name} already borrowed mutably"));

        Ref::map(cell.borrow(), |b| {
            b.downcast_ref::<R>().expect("resource type mismatch")
        })
    }

    pub fn get_resource_mut<R: Resource>(&self) -> RefMut<'_, R> {
        let id = TypeId::of::<R>();
        let name = std::any::type_name::<R>();

        let cell = self.resources.get(&id).unwrap_or_else(|| {
            panic!("Resource: {name} not found (did you forget to add its plugin?)")
        });

        cell.try_borrow_mut()
            .unwrap_or_else(|_| panic!("{name} already borrowed"));

        RefMut::map(cell.borrow_mut(), |b| {
            b.downcast_mut::<R>().expect("resource type mismatch")
        })
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
                    log::error!(
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
    pub fn emit<E: EventLabel>(&self, event: E) {
        let nodes = &self.scene;

        nodes.emit(&event, self);
    }
}
