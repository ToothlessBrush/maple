//! the EventReceiver handles systems that are ran on different schedules.
//!
//! for example the you may want move an object if the player presses a key you can define a
//! callback on the Event::Update that checks if that key is pressed then executes a callback the
//! offsets the position.

use crate::Scene;
use crate::context::{FPSManager, GameContext};
use crate::input::InputManager;
use crate::nodes::Node;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex};

/// Events can be emitted by the engine or nodes at various points in the engine process
#[derive(PartialEq, Eq, Clone, Debug, Hash)]
pub enum Event {
    /// this event is emitted before the first frame
    Ready,
    /// this event is emitted every frame
    Update,
    /// custom event that can be emitted and recieved by a node
    Custom(String),
}

type EventFunction = dyn Fn(&mut dyn Node, &mut GameContext) + Send + Sync;
type EventCallbacks = HashMap<Event, Arc<Mutex<EventFunction>>>;

/// Event reciever is responsible for receiving events from the context and running a callback for
/// that specific event
///
/// # Notes
/// Every node has a event receiver
#[derive(Default)]
pub struct EventReceiver {
    callbacks: EventCallbacks,
}

impl Clone for EventReceiver {
    fn clone(&self) -> Self {
        let callbacks = self
            .callbacks
            .iter()
            .map(|(event, callback)| {
                // Clone the callback using `Arc` to handle shared ownership
                let cloned_callback = Arc::clone(callback);
                (event.clone(), cloned_callback)
            })
            .collect();

        EventReceiver { callbacks }
    }
}

impl EventReceiver {
    /// creates a new event receiver
    pub fn new() -> Self {
        EventReceiver {
            callbacks: std::collections::HashMap::new(),
        }
    }

    /// add bahavior `on` a given event
    pub fn on<F, P>(&mut self, event: Event, callback: F)
    where
        F: IntoEventFunction<P>,
    {
        let boxed_callback = callback.into_event_function();

        self.callbacks.insert(
            event,
            Arc::new(Mutex::new(
                move |node: &mut dyn Node, ctx: &mut GameContext| {
                    boxed_callback(node, ctx);
                },
            )),
        );
    }

    /// trigger an event within the event receiver
    pub fn trigger(&mut self, event: Event, target: &mut dyn Node, ctx: &mut GameContext) {
        if let Some(callback) = self.callbacks.get_mut(&event) {
            if let Ok(callback) = callback.lock() {
                callback(target, ctx);
            }
        }
    }
}

pub trait EventParam {
    type Item<'a>;
    fn get_param<'a>(node: &'a mut dyn Node, ctx: &'a mut GameContext) -> Self::Item<'a>;
}

// ==========================
// NodeRef<T> (immutable)
// ==========================
pub struct NodeRef<T> {
    inner: *const T,
    _marker: PhantomData<T>,
}

impl<T> NodeRef<T> {
    #[inline]
    pub fn new(value: &T) -> Self {
        Self {
            inner: value as *const T,
            _marker: PhantomData,
        }
    }
}

impl<T> Deref for NodeRef<T> {
    type Target = T;
    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.inner }
    }
}

// EventParam for NodeRef<T>
impl<T: Node + 'static> EventParam for NodeRef<T> {
    type Item<'a> = NodeRef<T>;

    #[inline]
    fn get_param<'a>(node: &'a mut dyn Node, _ctx: &'a mut GameContext) -> Self::Item<'a> {
        let r: &T = node
            .downcast::<T>()
            .expect("NodeRef<T>: node is not of the expected type");
        NodeRef::new(r)
    }
}

// ==========================
// NodeMut<T> (mutable)
// ==========================
pub struct NodeMut<T> {
    inner: *mut T,
    _marker: PhantomData<T>,
}

impl<T> NodeMut<T> {
    #[inline]
    pub fn new(value: &mut T) -> Self {
        Self {
            inner: value as *mut T,
            _marker: PhantomData,
        }
    }
}

impl<T> Deref for NodeMut<T> {
    type Target = T;
    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.inner }
    }
}

impl<T> DerefMut for NodeMut<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.inner }
    }
}

// EventParam for NodeMut<T>
impl<T: Node + 'static> EventParam for NodeMut<T> {
    type Item<'a> = NodeMut<T>;

    #[inline]
    fn get_param<'a>(node: &'a mut dyn Node, _ctx: &'a mut GameContext) -> Self::Item<'a> {
        let r: &mut T = node
            .downcast_mut::<T>()
            .expect("NodeMut<T>: node is not of the expected type");
        NodeMut::new(r)
    }
}

// =============================================================================
// RES<T> - IMMUTABLE RESOURCE ACCESS
// =============================================================================

pub struct Res<T> {
    inner: *const T,
    _marker: PhantomData<T>,
}

impl<T> Res<T> {
    pub fn new(value: &T) -> Self {
        Self {
            inner: value as *const T,
            _marker: PhantomData,
        }
    }
}

impl<T> Deref for Res<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.inner }
    }
}

// EventParam implementations for Res<T>
impl EventParam for Res<GameContext> {
    type Item<'a> = Res<GameContext>;

    fn get_param<'a>(_node: &'a mut dyn Node, ctx: &'a mut GameContext) -> Self::Item<'a> {
        Res::new(ctx)
    }
}

impl EventParam for Res<FPSManager> {
    type Item<'a> = Res<FPSManager>;

    fn get_param<'a>(_node: &'a mut dyn Node, ctx: &'a mut GameContext) -> Self::Item<'a> {
        Res::new(&ctx.frame)
    }
}

impl EventParam for Res<InputManager> {
    type Item<'a> = Res<InputManager>;

    fn get_param<'a>(_node: &'a mut dyn Node, ctx: &'a mut GameContext) -> Self::Item<'a> {
        Res::new(&ctx.input)
    }
}

impl EventParam for Res<Scene> {
    type Item<'a> = Res<Scene>;

    fn get_param<'a>(_node: &'a mut dyn Node, ctx: &'a mut GameContext) -> Self::Item<'a> {
        Res::new(&ctx.scene)
    }
}

// =============================================================================
// RESMUT<T> - MUTABLE RESOURCE ACCESS
// =============================================================================

pub struct ResMut<T> {
    inner: *mut T,
    _marker: PhantomData<T>,
}

impl<T> ResMut<T> {
    pub fn new(value: &mut T) -> Self {
        Self {
            inner: value as *mut T,
            _marker: PhantomData,
        }
    }
}

impl<T> Deref for ResMut<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.inner }
    }
}

impl<T> DerefMut for ResMut<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.inner }
    }
}

// EventParam implementations for ResMut<T>
impl EventParam for ResMut<GameContext> {
    type Item<'a> = ResMut<GameContext>;

    fn get_param<'a>(_node: &'a mut dyn Node, ctx: &'a mut GameContext) -> Self::Item<'a> {
        ResMut::new(ctx)
    }
}

impl EventParam for ResMut<Scene> {
    type Item<'a> = ResMut<Scene>;

    fn get_param<'a>(_node: &'a mut dyn Node, ctx: &'a mut GameContext) -> Self::Item<'a> {
        ResMut::new(&mut ctx.scene)
    }
}

impl EventParam for ResMut<FPSManager> {
    type Item<'a> = ResMut<FPSManager>;

    fn get_param<'a>(_node: &'a mut dyn Node, ctx: &'a mut GameContext) -> Self::Item<'a> {
        ResMut::new(&mut ctx.frame)
    }
}

impl EventParam for ResMut<InputManager> {
    type Item<'a> = ResMut<InputManager>;

    fn get_param<'a>(_node: &'a mut dyn Node, ctx: &'a mut GameContext) -> Self::Item<'a> {
        ResMut::new(&mut ctx.input)
    }
}

// =============================================================================
// WORKING INTOEVENTFUNCTION - CONCRETE IMPLEMENTATIONS
// =============================================================================

pub trait IntoEventFunction<Params> {
    fn into_event_function(self) -> Box<EventFunction>;
}

// Only implement for the Fn traits - this covers both function items and pointers
impl<F> IntoEventFunction<()> for F
where
    F: Fn() + Send + Sync + 'static,
{
    fn into_event_function(self) -> Box<EventFunction> {
        Box::new(move |_node, _ctx| {
            self();
        })
    }
}

impl<F, P1> IntoEventFunction<(P1,)> for F
where
    F: Fn(P1::Item<'_>) + Send + Sync + 'static,
    P1: EventParam + 'static,
{
    fn into_event_function(self) -> Box<EventFunction> {
        Box::new(move |node, ctx| {
            let node_ptr = node as *mut dyn Node;
            let ctx_ptr = ctx as *mut GameContext;
            let p1 = unsafe { P1::get_param(&mut *node_ptr, &mut *ctx_ptr) };
            self(p1);
        })
    }
}

impl<F, P1, P2> IntoEventFunction<(P1, P2)> for F
where
    F: Fn(P1::Item<'_>, P2::Item<'_>) + Send + Sync + 'static,
    P1: EventParam + 'static,
    P2: EventParam + 'static,
{
    fn into_event_function(self) -> Box<EventFunction> {
        Box::new(move |node, ctx| {
            let node_ptr = node as *mut dyn Node;
            let ctx_ptr = ctx as *mut GameContext;
            let p1 = unsafe { P1::get_param(&mut *node_ptr, &mut *ctx_ptr) };
            let p2 = unsafe { P2::get_param(&mut *node_ptr, &mut *ctx_ptr) };
            self(p1, p2);
        })
    }
}

impl<F, P1, P2, P3> IntoEventFunction<(P1, P2, P3)> for F
where
    F: Fn(P1::Item<'_>, P2::Item<'_>, P3::Item<'_>) + Send + Sync + 'static,
    P1: EventParam + 'static,
    P2: EventParam + 'static,
    P3: EventParam + 'static,
{
    fn into_event_function(self) -> Box<EventFunction> {
        Box::new(move |node, ctx| {
            let node_ptr = node as *mut dyn Node;
            let ctx_ptr = ctx as *mut GameContext;
            let p1 = unsafe { P1::get_param(&mut *node_ptr, &mut *ctx_ptr) };
            let p2 = unsafe { P2::get_param(&mut *node_ptr, &mut *ctx_ptr) };
            let p3 = unsafe { P3::get_param(&mut *node_ptr, &mut *ctx_ptr) };
            self(p1, p2, p3);
        })
    }
}
