use crate::scene::Scene;

pub trait SceneBuilder {
    fn build(&mut self) -> Scene;
}

impl<T: SceneBuilder> From<T> for Scene {
    fn from(mut builder: T) -> Self {
        builder.build()
    }
}
