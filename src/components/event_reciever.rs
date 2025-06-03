//! the EventReceiver handles systems that are ran on different schedules.
//!
//! for example the you may want move an object if the player presses a key you can define a
//! callback on the Event::Update that checks if that key is pressed then executes a callback the
//! offsets the position.

use crate::context::GameContext;
use crate::nodes::Node;
use std::collections::HashMap;
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

type EventCallbacks = HashMap<Event, Arc<Mutex<dyn FnMut(&mut dyn Node, &mut GameContext)>>>;

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
    pub fn on<T: 'static + Node, F>(&mut self, event: Event, mut callback: F)
    where
        F: FnMut(&mut T, &mut GameContext) + 'static,
    {
        self.callbacks.insert(
            event,
            Arc::new(Mutex::new(Box::new(
                // outer callback that downcasts to the inner callback
                move |node: &mut dyn Node, ctx: &mut GameContext| {
                    if let Some(concrete) = node.downcast_mut::<T>() {
                        callback(concrete, ctx);
                    }
                },
            ))),
        );
    }

    /// trigger an event within the event receiver
    pub fn trigger(&mut self, event: Event, target: &mut dyn Node, ctx: &mut GameContext) {
        if let Some(callback) = self.callbacks.get_mut(&event) {
            if let Ok(mut callback) = callback.lock() {
                callback(target, ctx);
            }
        }
    }
}
