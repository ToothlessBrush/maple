use crate::game_context::node_manager::{
    Behavior, Node, NodeManager, NodeTransform, Ready,
};
use crate::game_context::GameContext;
use nalgebra_glm as glm;

pub struct Empty {
    pub transform: NodeTransform,
    pub children: NodeManager,

    ready_callback: Option<Box<dyn FnMut(&mut Self)>>,
    behavior_callback: Option<Box<dyn FnMut(&mut Self, &mut GameContext)>>,
}

impl Ready for Empty {
    fn ready(&mut self) {
        if let Some(mut callback) = self.ready_callback.take() {
            callback(self);
            self.ready_callback = Some(callback);
        }
    }
}

impl Behavior for Empty {
    fn behavior(&mut self, context: &mut GameContext) {
        if let Some(mut callback) = self.behavior_callback.take() {
            callback(self, context);
            self.behavior_callback = Some(callback);
        }
    }
}

impl Node for Empty {
    fn get_transform(&mut self) -> &mut NodeTransform {
        &mut self.transform
    }

    fn get_children(&mut self) -> &mut NodeManager {
        &mut self.children
    }

    fn as_ready(&mut self) -> Option<&mut (dyn Ready + 'static)> {
        Some(self)
    }

    fn as_behavior(&mut self) -> Option<&mut (dyn Behavior + 'static)> {
        Some(self)
    }
}

impl Empty {
    pub fn new() -> Self {
        Empty {
            transform: NodeTransform::default(),
            children: NodeManager::new(),

            ready_callback: None,
            behavior_callback: None,
        }
    }

    pub fn define_ready<F>(&mut self, ready_function: F) -> &mut Self
    where
        F: 'static + FnMut(&mut Self),
    {
        self.ready_callback = Some(Box::new(ready_function));
        self
    }

    pub fn define_behavior<F>(&mut self, behavior_function: F) -> &mut Self
    where
        F: 'static + FnMut(&mut Self, &mut GameContext),
    {
        self.behavior_callback = Some(Box::new(behavior_function));
        self
    }
}
