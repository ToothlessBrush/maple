use maple_engine::{Node, prelude::NodeTransform};

#[derive(Default, Clone)]
pub struct AudioListener {
    pub transform: NodeTransform,
    pub priority: i32,
}

impl Node for AudioListener {
    fn get_transform(&mut self) -> &mut NodeTransform {
        &mut self.transform
    }
}
