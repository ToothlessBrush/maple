use maple::prelude::*;
use maple_3d::nodes::camera::Camera3D;

pub struct MainScene;

impl SceneBuilder for MainScene {
    fn build(&mut self) -> Scene {
        let mut scene = Scene::default();

        scene.add("Camera", Camera3D::builder().build());

        scene
    }
}
