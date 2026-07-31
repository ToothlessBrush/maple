//! the EventReceiver handles systems that are ran on different schedules.

use parking_lot::Mutex;

use crate::Scene;
use crate::asset::AssetLibrary;
use crate::context::{GameContext, Res, ResMut, Resource};
use crate::nodes::Node;
use crate::platform::SendSync;
use crate::scene::{NodeHandle, NodeId, NodeMut, NodeRef, NodeView};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;

pub trait Event: Any + SendSync {}

/// ready is an [`Event`] that is ran when the node is added to the scene
#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
pub struct Ready;
impl Event for Ready {}

/// update is an [`Event`] that is ran every frame of the game loop
#[derive(Clone, Copy, Debug)]
pub struct Update {
    pub dt: f32,
}
impl Event for Update {}

/// FixedUpdate is an [`Event`] that is ran at a fixed 60 ticks per second
#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
pub struct FixedUpdate;
impl Event for FixedUpdate {}

/// context for the triggered event
///
/// this derefs the [`Event`] as well as adds a bunch of methods for interacting with the relevent
/// [`Node`], [`GameContext`], and [`Scene`]
pub struct EventCtx<'a, E, N: Node> {
    node: NodeView<'a, N>,
    pub game: &'a GameContext,
    pub event: &'a E,
}

impl<'a, E, N: Node> Deref for EventCtx<'a, E, N> {
    type Target = E;
    fn deref(&self) -> &Self::Target {
        self.event
    }
}

impl<'a, E, N: Node> EventCtx<'a, E, N> {
    /// get a resource from the [`GameContext`]
    pub fn get_resource<T: Resource>(&self) -> Res<T> {
        self.game.get_resource()
    }

    /// get a resource mut from the [`GameContext`]
    pub fn get_resource_mut<T: Resource>(&self) -> ResMut<T> {
        self.game.get_resource_mut()
    }

    /// get the [`AssetLibrary`] for adding and getting game [`crate::asset::Asset`]
    pub fn assets(&self) -> &AssetLibrary {
        &self.game.assets
    }

    /// the [`Scene`] this event is triggered in
    pub fn scene(&self) -> &Scene {
        &self.game.scene
    }

    /// immutible refrence for the [`Node`] this event is triggered on
    pub fn node_ref(&self) -> NodeRef<N> {
        self.node.get_ref()
    }

    /// [`NodeId`]s of this nodes children
    pub fn node_children_ids(&self) -> Vec<NodeId> {
        self.node.children_ids()
    }

    /// children of this node of a certain type
    pub fn node_children<T>(&self) -> Vec<NodeHandle<T>>
    where
        T: Node,
    {
        self.node.children()
    }

    /// first child with the given type T
    pub fn first_child<T>(&self) -> Option<NodeHandle<T>>
    where
        T: Node,
    {
        self.node.children::<T>().first().copied()
    }

    /// nodes parent Id if it exists
    pub fn node_parent_id(&self) -> Option<NodeId> {
        self.node.parent_id()
    }

    /// node parent if it exists and is `T`
    pub fn node_parent<T>(&self) -> Option<NodeHandle<T>>
    where
        T: Node,
    {
        self.node.parent()
    }

    /// immutible node refrence to the parent if it exists and is `T`
    pub fn node_parent_ref<T>(&self) -> Option<NodeRef<T>>
    where
        T: Node,
    {
        self.node_parent::<T>()
            .and_then(|parent| self.scene().get_ref(parent))
    }

    /// mutible refrence to the parent node if it exists and is `T`
    pub fn node_parent_mut<T>(&self) -> Option<NodeMut<T>>
    where
        T: Node,
    {
        self.node_parent::<T>()
            .and_then(|parent| self.scene().get_mut(parent))
    }

    /// mutible refrence for the [`Node`] this event was triggered on
    pub fn node_mut(&self) -> NodeMut<N> {
        self.node.get_mut()
    }

    /// Id of the node this event was triggered for
    pub fn node_id(&self) -> NodeId {
        self.node.id()
    }

    /// View in the scene from the perspective of the node this event was triggered for
    pub fn node_view(&self) -> &'a NodeView<'_, N> {
        &self.node
    }
}

#[cfg(not(target_arch = "wasm32"))]
type ErasedEventCallback = Box<dyn FnMut(&Scene, NodeId, &GameContext, &dyn Any) + Send + Sync>;
#[cfg(target_arch = "wasm32")]
type ErasedEventCallback = Box<dyn FnMut(&Scene, NodeId, &GameContext, &dyn Any)>;

/// stores and triggers node events
#[derive(Default)]
pub struct EventReceiver {
    callbacks: Mutex<HashMap<TypeId, Vec<Arc<Mutex<ErasedEventCallback>>>>>,
}

// impl Clone for EventReceiver {
//     fn clone(&self) -> Self {
//         let callbacks = self
//             .callbacks
//             .lock()
//             .iter()
//             .map(|(id, cbs)| (*id, cbs.iter().map(Arc::clone).collect()))
//             .collect();
//
//         Self { callbacks }
//     }
// }

impl EventReceiver {
    /// Create a new event receiver
    pub fn new() -> Self {
        Self {
            callbacks: Mutex::new(HashMap::new()),
        }
    }

    /// Register a callback for event `E` on node type `N`
    pub fn on<E, N, F>(&self, mut f: F)
    where
        E: Event + 'static,
        N: Node + 'static,
        F: for<'a> FnMut(EventCtx<'a, E, N>) + SendSync + 'static,
    {
        let event_id = TypeId::of::<E>();

        let callback: ErasedEventCallback = Box::new(
            move |scene, node_id, game: &GameContext, event_data: &dyn Any| {
                // Downcast event
                let event = match event_data.downcast_ref::<E>() {
                    Some(e) => e,
                    None => return,
                };

                let Some(handle) = scene.get_view_from_id::<N>(node_id) else {
                    return;
                };

                let ctx = EventCtx {
                    node: handle,
                    game,
                    event,
                };

                f(ctx);
            },
        );

        self.callbacks
            .lock()
            .entry(event_id)
            .or_default()
            .push(Arc::new(Mutex::new(callback)));
    }

    /// Trigger an event for a specific node
    pub fn trigger<E: Event>(&self, event: &E, scene: &Scene, node_id: NodeId, game: &GameContext) {
        let event_id = TypeId::of::<E>();

        if let Some(callbacks) = self.callbacks.lock().get(&event_id) {
            for callback in callbacks {
                let mut callback = callback.lock();
                callback(scene, node_id, game, event as &dyn Any);
            }
        }
    }
}
