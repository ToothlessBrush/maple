use maple::{
    context::scene::Scene,
    math,
    nodes::{Buildable, Builder, Camera3D, DirectionalLight, Model, model::Primitive},
};

pub struct MainScene;

impl MainScene {
    pub fn build() -> Scene {
        let mut scene = Scene::default();

        scene.add(
            "camera",
            Camera3D::builder()
                .position(math::vec3(0.0, 5.0, -10.0))
                .orientation_vector(math::vec3(0.0, -0.5, 1.0))
                .build(),
        );

        scene.add(
            "pyramid",
            Model::builder().add_primitive(Primitive::Pyramid).build(),
        );

        scene.add(
            "light",
            DirectionalLight::builder()
                .direction(math::vec3(-1.0, 1.0, 0.0))
                .build(),
        );

        scene.add(
            "ground",
            Model::builder()
                .add_primitive(Primitive::Plane)
                .position(math::vec3(0.0, -2.0, 0.0))
                .scale_factor(10.0)
                .build(),
        );

        scene
    }
}
