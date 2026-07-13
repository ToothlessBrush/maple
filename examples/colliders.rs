use maple::physics::{Group, InteractionGroups, InteractionTestMode};
pub use maple::prelude::*;

fn main() {
    App::default()
        .add_plugin(Core3D)
        .add_plugin(Physics3D)
        .load_scene(scene)
        .run()
}

fn scene(assets: &AssetLibrary) -> Scene {
    let scene = Scene::default();

    scene.spawn(
        DirectionalLight::builder()
            .direction((1.0, -1.0, -1.0))
            .color((1.0, 0.85, 0.6)) // warmish color
            .intensity(1.0),
    );

    scene.spawn(
        Camera3D::builder()
            .position((10.0, 10.0, 10.0))
            .looking_at(Vec3::ZERO),
    );

    // make a cube that moves within group 1
    scene
        .spawn(
            Collider3DBuilder::cube(0.5)
                .position((5.0, 0.0, 0.0))
                .collision_groups(InteractionGroups::new(
                    Group::GROUP_1,
                    Group::all(),
                    InteractionTestMode::And,
                )),
        )
        .on::<FixedUpdate>(|ctx| {
            ctx.node_mut().transform.position += Vec3::new(-0.1, 0.0, 0.0);
        })
        .on::<ColliderEnter>(|ctx| println!("I just hit {:?}", ctx.event.other))
        .spawn_child(
            MeshInstance3D::builder()
                .mesh(assets.add(Cuboid::default()))
                .material(assets.add(Color::RED)),
        );

    // make a cube that moves with group 2
    scene
        .spawn(
            Collider3DBuilder::cube(0.5)
                .position((15.0, 0.0, 0.0))
                .collision_groups(InteractionGroups::new(
                    Group::GROUP_2,
                    Group::all(),
                    InteractionTestMode::And,
                )),
        )
        .on::<FixedUpdate>(|ctx| {
            ctx.node_mut().transform.position += Vec3::new(-0.1, 0.0, 0.0);
        })
        .on::<ColliderEnter>(|ctx| println!("I just hit {:?}", ctx.event.other))
        .spawn_child(
            MeshInstance3D::builder()
                .mesh(assets.add(Cuboid::default()))
                .material(assets.add(Color::BLUE)),
        );

    // this cube will only detect collisions from group 2
    scene
        .spawn(
            Collider3DBuilder::cube(0.5)
                .position((0.0, 0.0, 0.0))
                .collision_groups(InteractionGroups::new(
                    Group::GROUP_2,
                    Group::GROUP_2,
                    InteractionTestMode::And,
                )),
        )
        .on::<ColliderEnter>(|ctx| println!("I was hit by {:?}", ctx.event.other))
        .spawn_child(
            MeshInstance3D::builder()
                .mesh(assets.add(Cuboid::default()))
                .material(assets.add(Color::BLUE)),
        );

    // this cube will detect collisions from everything
    scene
        .spawn(
            Collider3DBuilder::cube(0.5)
                .collision_groups(InteractionGroups::new(
                    Group::all(),
                    Group::all(),
                    InteractionTestMode::And,
                ))
                .position((-5.0, 0.0, 0.0)),
        )
        .on::<ColliderEnter>(|ctx| println!("I was hit by {:?}", ctx.event.other))
        .spawn_child(
            MeshInstance3D::builder()
                .mesh(assets.add(Cuboid::default()))
                .material(assets.add(Color::MAGENTA)),
        );

    scene
}
