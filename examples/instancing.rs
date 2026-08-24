use std::time::Duration;

use maple::prelude::*;
use maple_egui::prelude::*;

fn main() {
    App::new(Config {
        resizeable: false,
        resolution: Some(Resolution {
            width: 1000,
            height: 1000,
        }),
        ..Default::default()
    })
    .add_plugin(Core3D)
    .add_plugin(Physics3D)
    .add_plugin(EguiPlugin)
    .load_scene(MainScene)
    .run();
}

pub struct MainScene;

#[derive(Node)]
struct Positions {
    #[transform]
    pub transform: NodeTransform,
    pub wait_until: Duration,
    pub position: Vec3,
    pub mesh_count: u32,
    pub mesh: AssetHandle<Mesh3D>,
    pub material: AssetHandle<Material>,
    pub hdr: AssetHandle<Texture>,
    pub shell: u32,
    pub spawn_delay: f32,
    pub pending: Vec<(i32, i32, i32)>,
    pub spacing: f32,
}

fn ease_out_back(t: f32) -> f32 {
    let c1 = 1.70158;
    let c3 = c1 + 1.0;
    1.0 + c3 * (t - 1.0).powi(3) + c1 * (t - 1.0).powi(2)
}

fn spiral_indices(n: i32) -> Vec<(i32, i32)> {
    let mut result = Vec::with_capacity((n * n) as usize);
    let (mut top, mut bottom, mut left, mut right) = (0, n - 1, 0, n - 1);

    while top <= bottom && left <= right {
        for col in left..=right {
            result.push((top, col));
        }
        top += 1;
        if top > bottom {
            break;
        }

        for row in top..=bottom {
            result.push((row, right));
        }
        if right == 0 {
            break;
        }
        right -= 1;
        if left > right {
            break;
        }

        for col in (left..=right).rev() {
            result.push((bottom, col));
        }
        if bottom == 0 {
            break;
        }
        bottom -= 1;
        if top > bottom {
            break;
        }

        for row in (top..=bottom).rev() {
            result.push((row, left));
        }
        left += 1;
    }

    result
}

impl SceneBuilder for MainScene {
    fn build(self, assets: &AssetLibrary) -> Scene {
        let scene = Scene::default();

        scene
            .spawn(
                Camera3D::builder()
                    .position((50.0, 50.0, 50.0))
                    .looking_at((15.0, 15.0, 15.0))
                    .far_plane(200.0)
                    .build(),
            )
            .on::<EguiUpdate>(|ctx| {
                let node = ctx.node_ref().transform.position;
                egui::Window::new("fps").show(&ctx, |ui| {
                    ui.label(format!(
                        "fps: {}",
                        ctx.get_resource_mut::<Frame>().avg_fps()
                    ));
                    ui.label(format!(
                        "1% low: {}",
                        ctx.get_resource_mut::<Frame>().low_percent(0.01)
                    ));
                    ui.label(format!("x: {}, y: {}, z: {}", node.x, node.y, node.z))
                });
            })
            .on::<Update>(Camera3D::free_fly(5.0, 0.5))
            .on::<Ready>(|ctx| {
                let mut input: ResMut<Window> = ctx.get_resource_mut();
                input.set_cursor_locked(true);
            });

        let hdr = assets.load("res/kloofendal_48d_partly_cloudy_puresky_4k.hdr");
        scene.spawn(Environment::new(hdr.clone()).with_ibl_strength(0.2));

        scene.spawn(
            DirectionalLight::builder()
                .direction((-1.0, -1.0, -1.0))
                .intensity(4.0)
                .build(),
        );

        let root = scene.spawn(Positions {
            transform: NodeTransform::default(),
            position: Vec3::ZERO,
            wait_until: Duration::ZERO,
            mesh_count: 0,
            material: assets.add::<Material>(Color::CYAN),
            mesh: assets.add::<Mesh3D>(Torus::default()),
            hdr,
            shell: 0,
            spawn_delay: 0.1,
            pending: Vec::new(),
            spacing: 2.5,
        });

        root.on::<Update>(|ctx| {
            let frame: Res<Frame> = ctx.get_resource();
            let mut node = ctx.node_mut();

            if ctx.assets().is_loading(&node.hdr) {
                node.wait_until = frame.elapsed + Duration::from_secs_f32(1.0);
                return;
            }

            let elapsed = frame.elapsed;

            if node.pending.is_empty() {
                if node.shell >= 10 {
                    return;
                }
                let k = node.shell as i32;
                let mut seen = std::collections::HashSet::new();

                // z = k face, spiraling over (x, y)
                for (x, y) in spiral_indices(k + 1) {
                    if seen.insert((x, y, k)) {
                        node.pending.push((x, y, k));
                    }
                }
                // y = k face, spiraling over (x, z)
                for (x, z) in spiral_indices(k + 1) {
                    if seen.insert((x, k, z)) {
                        node.pending.push((x, k, z));
                    }
                }
                // x = k face, spiraling over (y, z)
                for (y, z) in spiral_indices(k + 1) {
                    if seen.insert((k, y, z)) {
                        node.pending.push((k, y, z));
                    }
                }

                let shell_time = 0.2;
                node.spawn_delay = shell_time / node.pending.len() as f32;

                node.shell += 1;
            }

            while elapsed >= node.wait_until {
                let delay = node.spawn_delay;
                node.wait_until += Duration::from_secs_f32(delay);
                if let Some((x, y, z)) = node.pending.pop() {
                    let offset = Vec3::new(x as f32, y as f32, z as f32) * node.spacing;
                    let pos = node.position + offset;
                    let spawned_at = frame.elapsed; // capture "now" before spawning

                    ctx.node_view()
                        .spawn_child(
                            MeshInstance3D::builder()
                                .mesh(node.mesh.clone())
                                .material(node.material.clone())
                                .position(pos)
                                .scale_factor(0.01),
                        )
                        .on::<Update>(move |ctx| {
                            let frame: Res<Frame> = ctx.get_resource();
                            let mut node = ctx.node_mut();

                            let duration = 1.0; // seconds for the pop to finish
                            let t =
                                ((frame.elapsed - spawned_at).as_secs_f32() / duration).min(1.0);

                            let scale = ease_out_back(t);
                            if scale > 0.0 {
                                node.transform.set_scale(Vec3::new(scale, scale, scale));
                            }
                        })
                        .on::<FixedUpdate>(|ctx| {
                            let mut node = ctx.node_mut();
                            let position = node.transform.position;
                            node.transform.rotate_euler_xyz_degrees(position / 100.0);
                        });
                    node.mesh_count += 1;
                }
            }
        })
        .on::<EguiUpdate>(|ctx| {
            egui::Window::new("meshes").show(&ctx, |ui| {
                ui.label(format!("meshes: {}", ctx.node_ref().mesh_count))
            });
        });

        scene
    }
}
