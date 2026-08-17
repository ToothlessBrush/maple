use maple_engine::{Node, prelude::NodeTransform};

/// listening position for recieving audio from [`super::AudioSource`] and provides spatial audio
///
/// only one listener can be active at once
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
