use crate::context::node_manager::Node;
use crate::context::GameContext;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(PartialEq, Eq, Clone, Debug, Hash)]
pub enum Event {
    Ready,
    Update,
    Custom(String),
}

pub struct EventReceiver {
    callbacks: HashMap<Event, Arc<Mutex<dyn FnMut(&mut dyn Node, &mut GameContext)>>>,
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

impl Default for EventReceiver {
    fn default() -> Self {
        Self {
            callbacks: std::collections::HashMap::new(),
        }
    }
}

impl EventReceiver {
    pub fn new() -> Self {
        EventReceiver {
            callbacks: std::collections::HashMap::new(),
        }
    }

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

    pub fn trigger(&mut self, event: Event, target: &mut dyn Node, ctx: &mut GameContext) {
        if let Some(callback) = self.callbacks.get_mut(&event) {
            if let Ok(mut callback) = callback.lock() {
                callback(target, ctx);
            }
        }
    }
}

// struct Context {
//     nodes: std::collections::HashMap<String, Box<dyn Node>>,
// }

// impl Context {
//     fn new() -> Self {
//         Self {
//             nodes: HashMap::new(),
//         }
//     }

//     fn add<T>(&mut self, name: &str, node: T)
//     where
//         T: Node + 'static,
//     {
//         self.nodes.insert(name.to_string(), Box::new(node));
//     }

//     fn emit(&mut self, event: Event) {
//         for (_, node) in &mut self.nodes {
//             node.trigger_event(event.clone());
//         }
//     }
// }
