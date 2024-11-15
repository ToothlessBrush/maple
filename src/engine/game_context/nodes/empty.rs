use crate::engine::game_context::node_manager::Node;
use crate::engine::game_context::GameContext;
use nalgebra_glm as glm;

pub struct Empty {
    ready_callback: Option<Box<dyn FnMut(&mut Self)>>,
    behavior_callback: Option<Box<dyn FnMut(&mut Self, &mut GameContext)>>,
}

impl Node for Empty {
    type Transform = ();

    fn get_model_matrix(&self) -> glm::Mat4 {
        glm::identity()
    }

    fn get_transform(&self) -> &Self::Transform {
        &()
    }

    fn define_ready<F>(&mut self, ready_function: F) -> &mut Self
    where
        F: 'static + FnMut(&mut Self),
    {
        self.ready_callback = Some(Box::new(ready_function));
        self
    }

    fn define_behavior<F>(&mut self, behavior_function: F) -> &mut Self
    where
        F: 'static + FnMut(&mut Self, &mut GameContext),
    {
        self.behavior_callback = Some(Box::new(behavior_function));
        self
    }

    //if the model has a ready function then call it
    fn ready(&mut self) {
        if let Some(mut callback) = self.ready_callback.take() {
            callback(self);
            self.ready_callback = Some(callback);
        }
    }

    //if the model has a behavior function then call it
    fn behavior(&mut self, context: &mut GameContext) {
        if let Some(mut callback) = self.behavior_callback.take() {
            callback(self, context);
            self.behavior_callback = Some(callback);
        }
    }
}
