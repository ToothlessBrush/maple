use std::{
    any::{Any, TypeId},
    collections::HashMap,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use winit::event::{DeviceEvent, WindowEvent};

use crate::{
    asset::AssetLibrary,
    components::EventLabel,
    resources::{Frame, Input},
    scene::Scene,
};

pub trait Resource: Any {}

pub struct Res<'a, T: Resource + 'static> {
    lock: RwLockReadGuard<'a, Box<dyn Any + Send + Sync>>,
    _ty: PhantomData<T>,
}

pub struct ResMut<'a, T: Resource + 'static> {
    lock: RwLockWriteGuard<'a, Box<dyn Any + Send + Sync>>,
    _ty: PhantomData<T>,
}

impl<'a, T: Resource + Send + Sync> Deref for Res<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.lock
            .downcast_ref()
            .expect("Res type and Resource type should be the same")
    }
}

impl<'a, T: Resource + Send + Sync> Deref for ResMut<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.lock
            .downcast_ref()
            .expect("ResMut type and Resource type should be the same")
    }
}

impl<'a, T: Resource + Send + Sync> DerefMut for ResMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.lock
            .downcast_mut()
            .expect("ResMut type and Resource type should be the same")
    }
}

/// The main game context, containing all the necessary information for the game to run.
/// This includes the window, the nodes, the frame manager, the input manager, and the shadow distance.
pub struct GameContext {
    /// The node manager of the game.
    pub scene: Scene,

    pub assets: AssetLibrary,

    resources: HashMap<TypeId, RwLock<Box<dyn Any + Send + Sync>>>,
}

impl Default for GameContext {
    fn default() -> Self {
        Self::new()
    }
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
            assets: AssetLibrary::new(),
        }
    }

    pub fn device_event(&mut self, event: &DeviceEvent) {
        self.get_resource_mut::<Input>().handle_device_event(event)
    }

    pub fn window_event(&mut self, event: &WindowEvent) {
        self.get_resource_mut::<Input>().handle_event(event);
    }

    pub fn begin_frame(&mut self) {
        self.scene.poll_async(&self.assets);
        self.get_resource_mut::<Frame>().update();
    }

    pub fn end_frame(&mut self) {
        self.get_resource_mut::<Input>().end_frame();
    }

    pub fn get_resource<R: Resource>(&self) -> Res<'_, R> {
        let id = TypeId::of::<R>();
        let name = std::any::type_name::<R>();

        let cell = self.resources.get(&id).unwrap_or_else(|| {
            panic!("Resource: {name} not found (did you forget to add its plugin?)")
        });

        let lock = cell.read();

        Res {
            lock,
            _ty: PhantomData,
        }
    }

    pub fn get_resource_mut<R: Resource>(&self) -> ResMut<'_, R> {
        let id = TypeId::of::<R>();
        let name = std::any::type_name::<R>();

        let cell = self.resources.get(&id).unwrap_or_else(|| {
            panic!("Resource: {name} not found (did you forget to add its plugin?)")
        });

        let lock = cell.write();

        ResMut {
            lock,
            _ty: PhantomData,
        }
    }
    pub fn insert_resource<R: Resource + Send + Sync>(&mut self, resource: R) {
        let id = TypeId::of::<R>();
        self.resources.insert(id, RwLock::new(Box::new(resource)));
    }

    pub fn pop_ready_queue(&self) {
        self.scene.pop_ready_queue(self);
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
