use crate::scene::Scene;

pub trait SceneBuilder {
    fn build(&mut self) -> Scene;
}
