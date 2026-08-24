use std::f32::consts::PI;

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

const WALK_SPEED: f32 = 5.0;
const SPRINT_SPEED: f32 = 10.0;
const JUMP_SPEED: f32 = 5.0;

const BOTTOM: f32 = 10.0;

fn player(assets: &AssetLibrary) -> Scene {
    let scene = Scene::default();

    // character controller for handleing movements within the scene
    let controller = scene.spawn(
        CharacterController::builder()
            // just above ground
            .position((0.0, -3.45, 0.0))
            .no_snap_to_ground()
            .slide(true)
            .min_slope_slide_angle_radians(30f32.to_radians())
            .autostep(CharacterAutostep {
                max_height: CharacterLength::Absolute(0.2),
                min_width: CharacterLength::Relative(0.1),
                include_dynamic_bodies: false,
            }),
    );
    controller.spawn_child(Collider3DBuilder::capsule_y(0.5, 0.5));
    controller.spawn_child(
        MeshInstance3D::builder()
            .mesh(assets.add(Capsule::default()))
            .material(assets.add(Color::RED)),
    );

    // scene camera attached to character
    let camera_handle = controller
        .spawn_child(Camera3D::builder().fov(75.0))
        .on(Camera3D::free_look(1.0))
        // orbit
        .on::<Update>(|ctx| {
            let mut node = ctx.node_mut();
            let direction = node.transform.get_forward_vector() * -1.0;

            let position = direction * 5.0;
            node.transform.set_position(position);
        })
        .on::<Ready>(|ctx| {
            ctx.get_resource_mut::<Window>().set_cursor_locked(true);
            println!("Use WASD to move\nUse Space to jump");
        })
        .handle();

    // player movement
    controller.on::<Update>(move |ctx| {
        let mut node = ctx.node_mut();
        let input = ctx.get_resource::<Input>();

        if node.transform.position.y < BOTTOM {
            node.transform.position = Vec3::ZERO;
            node.velocity = Vec3::ZERO;
        }

        let camera = ctx.scene().get_ref(camera_handle).unwrap();
        let forward = camera
            .transform
            .get_forward_vector()
            .with_y(0.0)
            .normalize();
        let right = camera.transform.get_right_vector().with_y(0.0).normalize();

        let move_input = input.get_vector(
            &KeyCode::KeyA,
            &KeyCode::KeyD,
            &KeyCode::KeyS,
            &KeyCode::KeyW,
        );

        let move_speed = if input.keys.contains(&KeyCode::ShiftLeft) {
            SPRINT_SPEED
        } else {
            WALK_SPEED
        };

        let dir = (forward * move_input.y + right * move_input.x).normalize_or_zero() * move_speed;

        if input.keys.contains(&KeyCode::Space) && node.is_grounded() {
            node.velocity.y = JUMP_SPEED;
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

    // skybox and ibl
    let hdr = assets.load("res/kloofendal_48d_partly_cloudy_puresky_4k.hdr");
    scene.spawn(Environment::new(hdr.clone()).with_ibl_strength(0.2));

    // ground for player to stand on
    let ground = scene.spawn(RigidBody3DBuilder::fixed().position((0.0, -5.0, 0.0)));
    ground.spawn_child(Collider3DBuilder::cuboid(20.0, 0.5, 20.0));
    ground.spawn_child(
        MeshInstance3D::builder()
            .mesh(assets.add(Cuboid {
                hx: 20.0,
                hy: 0.5,
                hz: 20.0,
            }))
            .material(assets.add(PbrMaterial::default())),
    );

    let ramp = scene.spawn(
        RigidBody3DBuilder::fixed()
            .position((-12.5, 0.0, 0.0))
            .rotation_euler_xyz_degrees((0.0, 0.0, 50.0)),
    );
    ramp.spawn_child(Collider3DBuilder::cuboid(0.5, 10.0, 1.0));
    ramp.spawn_child(
        MeshInstance3D::builder()
            .mesh(assets.add(Cuboid {
                hx: 0.5,
                hy: 10.0,
                hz: 1.0,
            }))
            .material(assets.add(Color::WHITE)),
    );

    let ramp = scene.spawn(
        RigidBody3DBuilder::fixed()
            .position((-10.0, 0.0, 3.0))
            .rotation_euler_xyz_degrees((0.0, 0.0, 30.0)),
    );
    ramp.spawn_child(Collider3DBuilder::cuboid(0.5, 10.0, 1.0));
    ramp.spawn_child(
        MeshInstance3D::builder()
            .mesh(assets.add(Cuboid::new(0.5, 10.0, 1.0)))
            .material(assets.add(Color::WHITE)),
    );

    // stair archway
    let initial = Vec3::new(5.0, -4.4, 5.0);
    let mesh = assets.add(Cuboid::new(0.2, 0.1, 1.0));
    let material = assets.add(Color::WHITE);
    for x in 0..=40 {
        let height = f32::sin(x as f32 / 40.0 * PI) * 3.0;
        scene
            .spawn(RigidBody3DBuilder::fixed().position((
                initial.x + x as f32 * 0.3,
                initial.y + height,
                0.0,
            )))
            .spawn_child(Collider3DBuilder::cuboid(0.2, 0.1, 1.0))
            .spawn_child(
                MeshInstance3D::builder()
                    .mesh(mesh.clone())
                    .material(material.clone()),
            );
    }

    scene
        .spawn(RigidBody3DBuilder::dynamic().position((-5.0, 0.0, -5.0)))
        .on::<Update>(|ctx| {
            let mut node = ctx.node_mut();
            if node.transform.position.y < BOTTOM {
                node.velocity = Vec3::ZERO;
                node.transform.position = Vec3 {
                    x: -5.0,
                    y: 0.0,
                    z: -5.0,
                }
            }
        })
        .spawn_child(Collider3DBuilder::cube(0.5).mass(1.0))
        .spawn_child(
            MeshInstance3D::builder()
                .mesh(assets.add(Cuboid::default()))
                .material(assets.add(Color::BLUE)),
        );

    scene
        .spawn(RigidBody3DBuilder::dynamic().position((5.0, 0.0, 5.0)))
        .on::<Update>(|ctx| {
            let mut node = ctx.node_mut();
            if node.transform.position.y < BOTTOM {
                node.velocity = Vec3::ZERO;
                node.transform.position = Vec3 {
                    x: 5.0,
                    y: 0.0,
                    z: 5.0,
                }
            }
        })
        .spawn_child(Collider3DBuilder::ball(0.5).mass(1.0).restitution(0.9))
        .spawn_child(
            MeshInstance3D::builder()
                .mesh(assets.add(Sphere::new(0.5)))
                .material(assets.add(PbrMaterial {
                    base_color_factor: Color::BLUE,
                    metallic_factor: 0.9,
                    roughness_factor: 0.05,
                    ..Default::default()
                })),
        );
    scene
}
