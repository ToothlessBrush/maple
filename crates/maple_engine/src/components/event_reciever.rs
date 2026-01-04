//! the EventReceiver handles systems that are ran on different schedules.
//!
//! for example the you may want move an object if the player presses a key you can define a
//! callback on the Event::Update that checks if that key is pressed then executes a callback the
//! offsets the position.

use crate::context::GameContext;
use crate::nodes::Node;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub trait EventLabel: Any {}

#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
pub struct Ready;
impl EventLabel for Ready {}

#[derive(Clone, Copy, Debug)]
pub struct Update {
    pub dt: f32,
}
impl EventLabel for Update {}

#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
pub struct FixedUpdate;
impl EventLabel for FixedUpdate {}

pub struct EventCtx<'a, E, N> {
    pub node: &'a mut N,
    pub game: &'a mut GameContext,
    pub event: &'a E,
}

type ErasedEventCallback = Box<dyn FnMut(&mut dyn Node, &mut GameContext, &dyn Any) + Send + Sync>;

#[derive(Default)]
pub struct EventReceiver {
    callbacks: HashMap<TypeId, Vec<Arc<Mutex<ErasedEventCallback>>>>,
}

impl Clone for EventReceiver {
    fn clone(&self) -> Self {
        let callbacks = self
            .callbacks
            .iter()
            .map(|(id, cbs)| (*id, cbs.iter().map(Arc::clone).collect()))
            .collect();

        Self { callbacks }
    }
}

impl EventReceiver {
    /// Create a new event receiver
    pub fn new() -> Self {
        Self {
            callbacks: HashMap::new(),
        }
    }

    /// Register a callback for event `E` on node type `N`
    pub fn on<E, N, F>(&mut self, mut f: F)
    where
        E: EventLabel + 'static,
        N: Node + 'static,
        F: for<'a> FnMut(EventCtx<'a, E, N>) + Send + Sync + 'static,
    {
        let event_id = TypeId::of::<E>();

        let callback: ErasedEventCallback = Box::new(
            move |node: &mut dyn Node, game: &mut GameContext, event_data: &dyn Any| {
                // Downcast node
                let node = match node.downcast_mut::<N>() {
                    Some(n) => n,
                    None => return,
                };

                // Downcast event
                let event = match event_data.downcast_ref::<E>() {
                    Some(e) => e,
                    None => return,
                };

                let ctx = EventCtx { node, game, event };

                f(ctx);
            },
        );

        self.callbacks
            .entry(event_id)
            .or_default()
            .push(Arc::new(Mutex::new(callback)));
    }

    /// Trigger an event for a specific node
    pub fn trigger<E: EventLabel>(
        &mut self,
        event: &E,
        target: &mut dyn Node,
        game: &mut GameContext,
    ) {
        let event_id = TypeId::of::<E>();

        if let Some(callbacks) = self.callbacks.get_mut(&event_id) {
            for callback in callbacks {
                if let Ok(mut callback) = callback.lock() {
                    callback(target, game, event as &dyn Any);
                }
            }
        }
    }
}

// helpers
pub fn none<F, E, N>(mut f: F) -> impl for<'a> FnMut(EventCtx<'a, E, N>) + Send + Sync
where
    F: FnMut() + Send + Sync + 'static,
{
    move |_ctx| f()
}

pub fn node<F, E, N>(mut f: F) -> impl for<'a> FnMut(EventCtx<'a, E, N>) + Send + Sync
where
    F: FnMut(&mut N) + Send + Sync + 'static,
{
    move |ctx| f(ctx.node)
}

pub fn event<F, E, N>(mut f: F) -> impl for<'a> FnMut(EventCtx<'a, E, N>) + Send + Sync
where
    F: FnMut(&E) + Send + Sync + 'static,
{
    move |ctx| f(ctx.event)
}

pub fn game<F, E, N>(mut f: F) -> impl for<'a> FnMut(EventCtx<'a, E, N>) + Send + Sync
where
    F: FnMut(&mut GameContext) + Send + Sync + 'static,
{
    move |ctx| f(ctx.game)
}

pub fn node_event<F, E, N>(mut f: F) -> impl for<'a> FnMut(EventCtx<'a, E, N>) + Send + Sync
where
    F: FnMut(&mut N, &E) + Send + Sync + 'static,
{
    move |ctx| f(ctx.node, ctx.event)
}

pub fn node_game<F, E, N>(mut f: F) -> impl for<'a> FnMut(EventCtx<'a, E, N>) + Send + Sync
where
    F: FnMut(&mut N, &mut GameContext) + Send + Sync + 'static,
{
    move |ctx| f(ctx.node, ctx.game)
}

pub fn event_game<F, E, N>(mut f: F) -> impl for<'a> FnMut(EventCtx<'a, E, N>) + Send + Sync
where
    F: FnMut(&E, &mut GameContext) + Send + Sync + 'static,
{
    move |ctx| f(ctx.event, ctx.game)
}

pub fn all<F, E, N>(mut f: F) -> impl for<'a> FnMut(EventCtx<'a, E, N>) + Send + Sync
where
    F: FnMut(&mut N, &E, &mut GameContext) + Send + Sync + 'static,
{
    move |ctx| f(ctx.node, ctx.event, ctx.game)
}
