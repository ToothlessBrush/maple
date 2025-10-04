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

#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
pub struct Update;
impl EventLabel for Update {}

type EventCallback = Arc<Mutex<dyn FnMut(&mut dyn Node, &mut GameContext) + Send + Sync>>;
type EventCallbacks = HashMap<TypeId, EventCallback>;

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
                let cloned_callback = Arc::clone(callback);
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
    pub fn on<E, T, F, Marker>(&mut self, _event: E, callback: F)
    where
        E: EventLabel,
        T: Node + 'static,
        F: IntoEventCallback<T, Marker> + 'static,
    {
        let mut typed_callback = callback.into_callback();
        let event = TypeId::of::<E>();

        self.callbacks.insert(
            event,
            Arc::new(Mutex::new(Box::new(
                move |node: &mut dyn Node, ctx: &mut GameContext| {
                    if let Some(concrete) = node.downcast_mut::<T>() {
                        typed_callback(concrete, ctx);
                    }
                },
            ))),
        );
    }

    /// trigger an event within the event receiver
    pub fn trigger<E: EventLabel>(
        &mut self,
        _event: &E,
        target: &mut dyn Node,
        ctx: &mut GameContext,
    ) {
        let event = TypeId::of::<E>();

        if let Some(callback) = self.callbacks.get_mut(&event) {
            if let Ok(mut callback) = callback.lock() {
                callback(target, ctx);
            }
        }
    }
}

type EventCallbackBox<T> = Box<dyn FnMut(&mut T, &mut GameContext) + Send + Sync>;

pub trait IntoEventCallback<T: Node, Marker> {
    fn into_callback(self) -> EventCallbackBox<T>;
}

impl<T, F> IntoEventCallback<T, fn()> for F
where
    T: Node + 'static,
    F: FnMut() + Send + Sync + 'static,
{
    fn into_callback(mut self) -> EventCallbackBox<T> {
        Box::new(move |_, _| self())
    }
}

impl<T, F> IntoEventCallback<T, fn(&mut GameContext)> for F
where
    T: Node + 'static,
    F: FnMut(&mut GameContext) + Send + Sync + 'static,
{
    fn into_callback(mut self) -> EventCallbackBox<T> {
        Box::new(move |_, ctx| self(ctx))
    }
}

impl<T, F> IntoEventCallback<T, fn(&GameContext)> for F
where
    T: Node + 'static,
    F: FnMut(&GameContext) + Send + Sync + 'static,
{
    fn into_callback(mut self) -> EventCallbackBox<T> {
        Box::new(move |_, ctx| self(ctx))
    }
}

impl<T, F> IntoEventCallback<T, fn(&mut T)> for F
where
    T: Node + 'static,
    F: FnMut(&mut T) + Send + Sync + 'static,
{
    fn into_callback(mut self) -> EventCallbackBox<T> {
        Box::new(move |node, _| self(node))
    }
}

impl<T, F> IntoEventCallback<T, fn(&T)> for F
where
    T: Node + 'static,
    F: FnMut(&T) + Send + Sync + 'static,
{
    fn into_callback(mut self) -> EventCallbackBox<T> {
        Box::new(move |node, _| self(node))
    }
}

impl<T, F> IntoEventCallback<T, fn(&mut T, &mut GameContext)> for F
where
    T: Node + 'static,
    F: FnMut(&mut T, &mut GameContext) + Send + Sync + 'static,
{
    fn into_callback(mut self) -> EventCallbackBox<T> {
        Box::new(move |node, ctx| self(node, ctx))
    }
}

impl<T, F> IntoEventCallback<T, fn(&T, &GameContext)> for F
where
    T: Node + 'static,
    F: FnMut(&T, &GameContext) + Send + Sync + 'static,
{
    fn into_callback(mut self) -> EventCallbackBox<T> {
        Box::new(move |node, ctx| self(node, ctx))
    }
}

impl<T, F> IntoEventCallback<T, fn(&mut GameContext, &mut T)> for F
where
    T: Node + 'static,
    F: FnMut(&mut GameContext, &mut T) + Send + Sync + 'static,
{
    fn into_callback(mut self) -> EventCallbackBox<T> {
        Box::new(move |node, ctx| self(ctx, node))
    }
}

impl<T, F> IntoEventCallback<T, fn(&GameContext, &T)> for F
where
    T: Node + 'static,
    F: FnMut(&GameContext, &T) + Send + Sync + 'static,
{
    fn into_callback(mut self) -> EventCallbackBox<T> {
        Box::new(move |node, ctx| self(ctx, node))
    }
}
