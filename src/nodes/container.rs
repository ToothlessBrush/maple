use egui_gl_glfw::egui::Event;

use crate::components::{EventReceiver, NodeTransform};
use crate::context::scene::{Node, Scene};

#[derive(Clone)]
pub struct Container<T> {
    data: T,
    transform: NodeTransform,
    children: Scene,
    events: EventReceiver,
}

impl<T> Container<T> {
    pub fn new(data: T) -> Container<T>
    where
        T: Clone,
    {
        Container {
            data,
            transform: NodeTransform::default(),
            children: Scene::default(),
            events: EventReceiver::default(),
        }
    }

    pub fn get_data(&self) -> &T {
        &self.data
    }

    pub fn get_data_mut(&mut self) -> &mut T {
        &mut self.data
    }
}

impl<T> Node for Container<T>
where
    T: Clone + 'static,
{
    fn get_transform(&mut self) -> &mut NodeTransform {
        &mut self.transform
    }

    fn get_children(&self) -> &Scene {
        &self.children
    }

    fn get_children_mut(&mut self) -> &mut Scene {
        &mut self.children
    }

    fn get_events(&mut self) -> &mut crate::components::EventReceiver {
        &mut self.events
    }
}
