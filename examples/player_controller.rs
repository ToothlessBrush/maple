use maple::prelude::*;

fn main() {
    App::new(Config {
        ..Default::default()
    })
    .add_plugin(Core3D)
    .add_plugin(Physics3D)
    .load_scene(player)
    .load_scene(playground)
    .run();
}

fn player(assets: &AssetLibrary) -> Scene {
    let scene = Scene::default();

    // character controller for handleing movements within the scene
    let controller = scene.spawn(CharacterController::builder().no_snap_to_ground());
    controller.spawn_child(Collider3DBuilder::capsule_y(0.5, 0.5));
    controller.spawn_child(
        MeshInstance3D::builder()
            .mesh(assets.add(Capsule::default()))
            .material(assets.add(Color::RED)),
    );

    // scene camera attached to character
    let camera_handle = controller
        .spawn_child(Camera3D::builder())
        .on(Camera3D::free_look(1.0))
        // orbit
        .on::<Update>(|ctx| {
            let mut node = ctx.node_mut();
            let direction = node.transform.get_forward_vector() * -1.0;

            let position = direction * 10.0;
            node.transform.set_position(position);
        })
        .on::<Ready>(|ctx| {
            ctx.get_resource_mut::<Input>().set_cursor_locked(true);
            println!("Use WASD to move\nUse Space to jump");
        })
        .handle();

    // player movement
    controller.on::<Update>(move |ctx| {
        let mut node = ctx.node_mut();
        let input = ctx.get_resource::<Input>();

        if node.transform.position.y < -10.0 {
            node.transform.position = Vec3::ZERO;
            node.velocity = Vec3::ZERO;
        }

        let camera = ctx.scene().get_ref(camera_handle).unwrap();
        let forward =
            (camera.transform.get_forward_vector() * Vec3::new(1.0, 0.0, 1.0)).normalize();
        let right = (camera.transform.get_right_vector() * Vec3::new(1.0, 0.0, 1.0)).normalize();

        let mut dir = Vec3::default();
        if input.keys.contains(&KeyCode::KeyW) {
            dir += forward * 5.0;
        }
        if input.keys.contains(&KeyCode::KeyS) {
            dir += forward * -5.0;
        }
        if input.keys.contains(&KeyCode::KeyA) {
            dir += right * -5.0;
        }
        if input.keys.contains(&KeyCode::KeyD) {
            dir += right * 5.0;
        }
        if input.keys.contains(&KeyCode::Space) && node.is_grounded() {
            node.velocity.y = 5.0;
        }

        node.velocity.x = dir.x;
        node.velocity.z = dir.z;
    });

    scene
}

fn playground(assets: &AssetLibrary) -> Scene {
    let scene = Scene::default();

    // scene light souce
    scene.spawn(DirectionalLight::builder().direction((1.0, -1.0, -1.0)));

    // ground for player to stand on
    let ground = scene.spawn(RigidBody3DBuilder::fixed().position((0.0, -5.0, 0.0)));
    ground.spawn_child(Collider3DBuilder::cuboid(10.0, 0.5, 10.0));
    ground.spawn_child(
        MeshInstance3D::builder()
            .mesh(assets.add(Cuboid {
                hx: 10.0,
                hy: 0.5,
                hz: 10.0,
            }))
            .material(assets.add(PbrMaterial::default())),
    );

    scene
}
