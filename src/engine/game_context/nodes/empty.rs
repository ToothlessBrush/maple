use crate::engine::game_context::node_manager::{Behavior, Node, NodeTransform, Ready};
use crate::engine::game_context::GameContext;
use nalgebra_glm as glm;

pub struct Empty {
    tranform: NodeTransform,

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
    type Transform = NodeTransform;

    fn get_model_matrix(&self) -> glm::Mat4 {
        glm::identity()
    }

    fn get_transform(&self) -> &Self::Transform {
        &self.tranform
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn as_ready(&mut self) -> Option<&mut (dyn Ready<Transform = Self::Transform> + 'static)> {
        Some(self)
    }
}

impl Empty {
    pub fn new() -> Self {
        Empty {
            tranform: NodeTransform::default(),

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
