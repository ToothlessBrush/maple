pub use maple::prelude::*;

fn main() {
    App::default().add_plugin(Core3D).load_scene(scene).run()
}

fn scene(assets: &AssetLibrary) -> Scene {
    let scene = Scene::default();

    scene
        .spawn(Empty::default())
        .on::<FixedUpdate>(|ctx| {
            ctx.node_mut().transform.rotate((0.1, 1.0, 0.1), 0.1);
        })
        .spawn_child(DirectionalLight::builder().direction((1.0, -1.0, -1.0)));

    scene.spawn(
        Camera3D::builder()
            .position((2.0, 2.0, 2.0))
            .looking_at(Vec3::ZERO),
    );

    scene.spawn(
        MeshInstance3D::builder()
            .mesh(assets.add(Plane::default().size((2.0, 2.0))))
            .material(assets.add(Color::WHITE)),
    );

    scene.spawn(
        MeshInstance3D::builder()
            .mesh(assets.add(Cuboid::default().half_extent(0.1)))
            .material(assets.add(Color::BLUE))
            .position((0.0, 0.3, 0.0)),
    );

    scene
}
