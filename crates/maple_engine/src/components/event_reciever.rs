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

#[derive(Default)]
pub struct EventReceiver {
    callbacks: EventCallbacks,
}

// Update the callback type to use &dyn Any
type EventCallback = Box<dyn FnMut(&mut dyn Node, &mut GameContext, &dyn Any) + Send + Sync>;
type EventCallbacks = HashMap<TypeId, Vec<Arc<Mutex<EventCallback>>>>;

// Updated callback box to use &dyn Any
type EventCallbackBox<N> = Box<dyn FnMut(&mut N, &mut GameContext, &dyn Any) + Send + Sync>;

pub trait IntoEventCallback<N: Node, E: EventLabel, Marker> {
    fn into_callback(self) -> EventCallbackBox<N>;
}

impl Clone for EventReceiver {
    fn clone(&self) -> Self {
        let callbacks = self
            .callbacks
            .iter()
            .map(|(event, callbacks)| {
                let cloned_callback = callbacks
                    .iter()
                    .map(|callback| Arc::clone(callback))
                    .collect();
                (*event, cloned_callback)
            })
            .collect();
        EventReceiver { callbacks }
    }
}

impl EventReceiver {
    /// creates a new event receiver
    pub fn new() -> Self {
        EventReceiver {
            callbacks: HashMap::new(),
        }
    }

    /// add behavior `on` a given event
    pub fn on<E, N, Marker>(&mut self, callback: impl IntoEventCallback<N, E, Marker> + 'static)
    where
        E: EventLabel,
        N: Node + 'static,
    {
        let mut typed_callback = callback.into_callback();
        let event = TypeId::of::<E>();

        let wrapped_callback = Arc::new(Mutex::new(Box::new(
            move |node: &mut dyn Node, ctx: &mut GameContext, event_data: &dyn Any| {
                if let Some(concrete) = node.downcast_mut::<N>() {
                    typed_callback(concrete, ctx, event_data);
                }
            },
        ) as EventCallback));

        self.callbacks
            .entry(event)
            .or_default()
            .push(wrapped_callback);
    }

    /// trigger an event within the event receiver
    pub fn trigger<E: EventLabel>(
        &mut self,
        event: &E,
        target: &mut dyn Node,
        ctx: &mut GameContext,
    ) {
        let event_id = TypeId::of::<E>();
        if let Some(callbacks) = self.callbacks.get_mut(&event_id) {
            for callback in callbacks.iter_mut() {
                if let Ok(mut callback) = callback.lock() {
                    callback(target, ctx, event as &dyn Any);
                }
            }
        }
    }
}

// hell
impl<N, E, F> IntoEventCallback<N, E, fn()> for F
where
    N: Node + 'static,
    E: EventLabel,
    F: FnMut() + Send + Sync + 'static,
{
    fn into_callback(mut self) -> EventCallbackBox<N> {
        Box::new(move |_, _, _| self())
    }
}

impl<N, E, F> IntoEventCallback<N, E, fn(&mut GameContext)> for F
where
    N: Node + 'static,
    E: EventLabel,
    F: FnMut(&mut GameContext) + Send + Sync + 'static,
{
    fn into_callback(mut self) -> EventCallbackBox<N> {
        Box::new(move |_, ctx, _| self(ctx))
    }
}

impl<N, E, F> IntoEventCallback<N, E, fn(&GameContext)> for F
where
    N: Node + 'static,
    E: EventLabel,
    F: FnMut(&GameContext) + Send + Sync + 'static,
{
    fn into_callback(mut self) -> EventCallbackBox<N> {
        Box::new(move |_, ctx, _| self(ctx))
    }
}

impl<N, E, F> IntoEventCallback<N, E, fn(&mut N)> for F
where
    N: Node + 'static,
    E: EventLabel,
    F: FnMut(&mut N) + Send + Sync + 'static,
{
    fn into_callback(mut self) -> EventCallbackBox<N> {
        Box::new(move |node, _, _| self(node))
    }
}

impl<N, E, F> IntoEventCallback<N, E, fn(&N)> for F
where
    N: Node + 'static,
    E: EventLabel,
    F: FnMut(&N) + Send + Sync + 'static,
{
    fn into_callback(mut self) -> EventCallbackBox<N> {
        Box::new(move |node, _, _| self(node))
    }
}

impl<N, E, F> IntoEventCallback<N, E, fn(&mut N, &mut GameContext)> for F
where
    N: Node + 'static,
    E: EventLabel,
    F: FnMut(&mut N, &mut GameContext) + Send + Sync + 'static,
{
    fn into_callback(mut self) -> EventCallbackBox<N> {
        Box::new(move |node, ctx, _| self(node, ctx))
    }
}

impl<N, E, F> IntoEventCallback<N, E, fn(&N, &GameContext)> for F
where
    N: Node + 'static,
    E: EventLabel,
    F: FnMut(&N, &GameContext) + Send + Sync + 'static,
{
    fn into_callback(mut self) -> EventCallbackBox<N> {
        Box::new(move |node, ctx, _| self(node, ctx))
    }
}

impl<N, E, F> IntoEventCallback<N, E, fn(&mut GameContext, &mut N)> for F
where
    N: Node + 'static,
    E: EventLabel,
    F: FnMut(&mut GameContext, &mut N) + Send + Sync + 'static,
{
    fn into_callback(mut self) -> EventCallbackBox<N> {
        Box::new(move |node, ctx, _| self(ctx, node))
    }
}

impl<N, E, F> IntoEventCallback<N, E, fn(&GameContext, &N)> for F
where
    N: Node + 'static,
    E: EventLabel,
    F: FnMut(&GameContext, &N) + Send + Sync + 'static,
{
    fn into_callback(mut self) -> EventCallbackBox<N> {
        Box::new(move |node, ctx, _| self(ctx, node))
    }
}

impl<N, E, F> IntoEventCallback<N, E, fn(&mut N, &GameContext)> for F
where
    N: Node + 'static,
    E: EventLabel,
    F: FnMut(&mut N, &GameContext) + Send + Sync + 'static,
{
    fn into_callback(mut self) -> EventCallbackBox<N> {
        Box::new(move |node, ctx, _| self(node, ctx))
    }
}

impl<N, E, F> IntoEventCallback<N, E, fn(&N, &mut GameContext)> for F
where
    N: Node + 'static,
    E: EventLabel,
    F: FnMut(&N, &mut GameContext) + Send + Sync + 'static,
{
    fn into_callback(mut self) -> EventCallbackBox<N> {
        Box::new(move |node, ctx, _| self(node, ctx))
    }
}

impl<N, E, F> IntoEventCallback<N, E, fn(&GameContext, &mut N)> for F
where
    N: Node + 'static,
    E: EventLabel,
    F: FnMut(&GameContext, &mut N) + Send + Sync + 'static,
{
    fn into_callback(mut self) -> EventCallbackBox<N> {
        Box::new(move |node, ctx, _| self(ctx, node))
    }
}

impl<N, E, F> IntoEventCallback<N, E, fn(&mut GameContext, &N)> for F
where
    N: Node + 'static,
    E: EventLabel,
    F: FnMut(&mut GameContext, &N) + Send + Sync + 'static,
{
    fn into_callback(mut self) -> EventCallbackBox<N> {
        Box::new(move |node, ctx, _| self(ctx, node))
    }
}

// Event + Node (both orders, mutable and immutable)
impl<N, E, F> IntoEventCallback<N, E, fn(&E, &mut N)> for F
where
    N: Node + 'static,
    E: EventLabel,
    F: FnMut(&E, &mut N) + Send + Sync + 'static,
{
    fn into_callback(mut self) -> EventCallbackBox<N> {
        Box::new(move |node, _, event_data| {
            if let Some(event) = event_data.downcast_ref::<E>() {
                self(event, node)
            }
        })
    }
}

impl<N, E, F> IntoEventCallback<N, E, fn(&mut N, &E)> for F
where
    N: Node + 'static,
    E: EventLabel,
    F: FnMut(&mut N, &E) + Send + Sync + 'static,
{
    fn into_callback(mut self) -> EventCallbackBox<N> {
        Box::new(move |node, _, event_data| {
            if let Some(event) = event_data.downcast_ref::<E>() {
                self(node, event)
            }
        })
    }
}

impl<N, E, F> IntoEventCallback<N, E, fn(&E, &N)> for F
where
    N: Node + 'static,
    E: EventLabel,
    F: FnMut(&E, &N) + Send + Sync + 'static,
{
    fn into_callback(mut self) -> EventCallbackBox<N> {
        Box::new(move |node, _, event_data| {
            if let Some(event) = event_data.downcast_ref::<E>() {
                self(event, node)
            }
        })
    }
}

// All three parameters - Event first
impl<N, E, F> IntoEventCallback<N, E, fn(&E, &mut N, &mut GameContext)> for F
where
    N: Node + 'static,
    E: EventLabel,
    F: FnMut(&E, &mut N, &mut GameContext) + Send + Sync + 'static,
{
    fn into_callback(mut self) -> EventCallbackBox<N> {
        Box::new(move |node, ctx, event_data| {
            if let Some(event) = event_data.downcast_ref::<E>() {
                self(event, node, ctx)
            }
        })
    }
}

impl<N, E, F> IntoEventCallback<N, E, fn(&E, &mut GameContext, &mut N)> for F
where
    N: Node + 'static,
    E: EventLabel,
    F: FnMut(&E, &mut GameContext, &mut N) + Send + Sync + 'static,
{
    fn into_callback(mut self) -> EventCallbackBox<N> {
        Box::new(move |node, ctx, event_data| {
            if let Some(event) = event_data.downcast_ref::<E>() {
                self(event, ctx, node)
            }
        })
    }
}

impl<N, E, F> IntoEventCallback<N, E, fn(&E, &N, &mut GameContext)> for F
where
    N: Node + 'static,
    E: EventLabel,
    F: FnMut(&E, &N, &mut GameContext) + Send + Sync + 'static,
{
    fn into_callback(mut self) -> EventCallbackBox<N> {
        Box::new(move |node, ctx, event_data| {
            if let Some(event) = event_data.downcast_ref::<E>() {
                self(event, node, ctx)
            }
        })
    }
}

impl<N, E, F> IntoEventCallback<N, E, fn(&E, &mut GameContext, &N)> for F
where
    N: Node + 'static,
    E: EventLabel,
    F: FnMut(&E, &mut GameContext, &N) + Send + Sync + 'static,
{
    fn into_callback(mut self) -> EventCallbackBox<N> {
        Box::new(move |node, ctx, event_data| {
            if let Some(event) = event_data.downcast_ref::<E>() {
                self(event, ctx, node)
            }
        })
    }
}

impl<N, E, F> IntoEventCallback<N, E, fn(&E, &N, &GameContext)> for F
where
    N: Node + 'static,
    E: EventLabel,
    F: FnMut(&E, &N, &GameContext) + Send + Sync + 'static,
{
    fn into_callback(mut self) -> EventCallbackBox<N> {
        Box::new(move |node, ctx, event_data| {
            if let Some(event) = event_data.downcast_ref::<E>() {
                self(event, node, ctx)
            }
        })
    }
}

impl<N, E, F> IntoEventCallback<N, E, fn(&E, &GameContext, &N)> for F
where
    N: Node + 'static,
    E: EventLabel,
    F: FnMut(&E, &GameContext, &N) + Send + Sync + 'static,
{
    fn into_callback(mut self) -> EventCallbackBox<N> {
        Box::new(move |node, ctx, event_data| {
            if let Some(event) = event_data.downcast_ref::<E>() {
                self(event, ctx, node)
            }
        })
    }
}

// All three parameters - Node first
impl<N, E, F> IntoEventCallback<N, E, fn(&mut N, &E, &mut GameContext)> for F
where
    N: Node + 'static,
    E: EventLabel,
    F: FnMut(&mut N, &E, &mut GameContext) + Send + Sync + 'static,
{
    fn into_callback(mut self) -> EventCallbackBox<N> {
        Box::new(move |node, ctx, event_data| {
            if let Some(event) = event_data.downcast_ref::<E>() {
                self(node, event, ctx)
            }
        })
    }
}

impl<N, E, F> IntoEventCallback<N, E, fn(&mut N, &mut GameContext, &E)> for F
where
    N: Node + 'static,
    E: EventLabel,
    F: FnMut(&mut N, &mut GameContext, &E) + Send + Sync + 'static,
{
    fn into_callback(mut self) -> EventCallbackBox<N> {
        Box::new(move |node, ctx, event_data| {
            if let Some(event) = event_data.downcast_ref::<E>() {
                self(node, ctx, event)
            }
        })
    }
}

// All three parameters - Context first
impl<N, E, F> IntoEventCallback<N, E, fn(&mut GameContext, &E, &mut N)> for F
where
    N: Node + 'static,
    E: EventLabel,
    F: FnMut(&mut GameContext, &E, &mut N) + Send + Sync + 'static,
{
    fn into_callback(mut self) -> EventCallbackBox<N> {
        Box::new(move |node, ctx, event_data| {
            if let Some(event) = event_data.downcast_ref::<E>() {
                self(ctx, event, node)
            }
        })
    }
}

impl<N, E, F> IntoEventCallback<N, E, fn(&mut GameContext, &mut N, &E)> for F
where
    N: Node + 'static,
    E: EventLabel,
    F: FnMut(&mut GameContext, &mut N, &E) + Send + Sync + 'static,
{
    fn into_callback(mut self) -> EventCallbackBox<N> {
        Box::new(move |node, ctx, event_data| {
            if let Some(event) = event_data.downcast_ref::<E>() {
                self(ctx, node, event)
            }
        })
    }
}

impl<N, E, F> IntoEventCallback<N, E, fn(&GameContext, &E, &mut N)> for F
where
    N: Node + 'static,
    E: EventLabel,
    F: FnMut(&GameContext, &E, &mut N) + Send + Sync + 'static,
{
    fn into_callback(mut self) -> EventCallbackBox<N> {
        Box::new(move |node, ctx, event_data| {
            if let Some(event) = event_data.downcast_ref::<E>() {
                self(ctx, event, node)
            }
        })
    }
}

impl<N, E, F> IntoEventCallback<N, E, fn(&GameContext, &mut N, &E)> for F
where
    N: Node + 'static,
    E: EventLabel,
    F: FnMut(&GameContext, &mut N, &E) + Send + Sync + 'static,
{
    fn into_callback(mut self) -> EventCallbackBox<N> {
        Box::new(move |node, ctx, event_data| {
            if let Some(event) = event_data.downcast_ref::<E>() {
                self(ctx, node, event)
            }
        })
    }
}

impl<N, E, F> IntoEventCallback<N, E, fn(&GameContext, &E, &N)> for F
where
    N: Node + 'static,
    E: EventLabel,
    F: FnMut(&GameContext, &E, &N) + Send + Sync + 'static,
{
    fn into_callback(mut self) -> EventCallbackBox<N> {
        Box::new(move |node, ctx, event_data| {
            if let Some(event) = event_data.downcast_ref::<E>() {
                self(ctx, event, node)
            }
        })
    }
}

// Marker types to distinguish different parameter patterns
pub struct EventOnly;
pub struct NodeOnly;

// Just event
impl<N, E, F> IntoEventCallback<N, E, (EventOnly, fn(&E))> for F
where
    N: Node + 'static,
    E: EventLabel,
    F: FnMut(&E) + Send + Sync + 'static,
{
    fn into_callback(mut self) -> EventCallbackBox<N> {
        Box::new(move |_, _, event_data| {
            if let Some(event) = event_data.downcast_ref::<E>() {
                self(event)
            }
        })
    }
}

// Just node (immutable)
impl<N, E, F> IntoEventCallback<N, E, (NodeOnly, fn(&N))> for F
where
    N: Node + 'static,
    E: EventLabel,
    F: FnMut(&N) + Send + Sync + 'static,
{
    fn into_callback(mut self) -> EventCallbackBox<N> {
        Box::new(move |node, _, _| self(node))
    }
}
