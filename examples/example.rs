use nalgebra_glm::{left_handed, sin};
use quaturn::context::node_manager::{self};
use quaturn::nodes::camera::Camera3DBuilder;
use quaturn::nodes::model::ModelBuilder;
use quaturn::nodes::point_light::{self, PointLightBuilder};
use quaturn::nodes::{
    model::Primitive, Camera3D, Container, DirectionalLight, Empty, Model, PointLight,
    UseReadyCallback, UI,
};
use std::error::Error;
use std::time::Instant;
use std::{default, time::Duration};

use quaturn::components::mesh::MaterialProperties;

use quaturn::components::NodeTransform;

use quaturn::nodes::{NodeBuilder, UseBehaviorCallback};

use quaturn::context::node_manager::{Node, NodeManager, Transformable};
use quaturn::context::GameContext;
use quaturn::renderer::shader::Shader;
use quaturn::utils::color::Color;
use quaturn::Engine;
use quaturn::{egui, glfw, glm};
use std::f32::consts::{FRAC_PI_4, PI};

use quaturn::components::{Event, EventReceiver};

const RAD_120: f32 = 120.0 * PI / 180.0;
//use engine::Engine;

const WINDOW_WIDTH: u32 = 1280;
const WINDOW_HEIGHT: u32 = 720;

#[derive(Default, Clone)]
struct CustomNode {
    transform: NodeTransform,
    children: NodeManager,
    events: EventReceiver,
    pub distance: f32,
}

impl Node for CustomNode {
    fn get_transform(&mut self) -> &mut NodeTransform {
        &mut self.transform
    }

    fn get_children(&self) -> &NodeManager {
        &self.children
    }

    fn get_children_mut(&mut self) -> &mut NodeManager {
        &mut self.children
    }

    fn get_events(&mut self) -> &mut quaturn::components::EventReceiver {
        &mut self.events
    }
}

impl CustomNode {
    pub fn new() -> Self {
        CustomNode {
            distance: 10.0,
            ..Default::default()
        }
    }
}

#[derive(Clone, Default)]
struct Building {
    transform: NodeTransform,
    children: NodeManager,
    events: EventReceiver,
}

impl Node for Building {
    fn get_transform(&mut self) -> &mut NodeTransform {
        &mut self.transform
    }

    fn get_children(&self) -> &NodeManager {
        &self.children
    }

    fn get_children_mut(&mut self) -> &mut NodeManager {
        &mut self.children
    }

    fn get_events(&mut self) -> &mut quaturn::components::EventReceiver {
        &mut self.events
    }
}

impl Building {
    pub fn new() -> Building {
        let mut building_model = Model::new_gltf("res/models/light_test.glb");
        building_model.apply_transform(&mut |t| {
            t.rotate_euler_xyz(glm::vec3(0.0, 0.0, 0.0));
        });
        let mut nodes = NodeManager::new();
        nodes.add("building model", building_model);
        Building {
            children: nodes,
            ..Default::default()
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut engine = Engine::init("Hello Pyramid", WINDOW_WIDTH, WINDOW_HEIGHT);

    engine.set_clear_color(0.0, 0.0, 0.0, 1.0);

    let mut cursor_locked = false;

    let toggle_cursor_lock = |context: &mut GameContext, lock: bool| {
        context.lock_cursor(lock);
    };

    engine.context.nodes.add(
        "building",
        NodeBuilder::new(Model::new_gltf("res/models/japan/scene.gltf"))
            .with_rotation_euler_xyz(glm::vec3(-90.0, 0.0, 0.0))
            .build(),
    )?;

    let camera_pos = glm::vec3(20.0, 20.0, 20.0);

    engine.context.nodes.add(
        "camera",
        NodeBuilder::new(Camera3D::new(
            FRAC_PI_4, // pi/4
            WINDOW_WIDTH as f32 / WINDOW_HEIGHT as f32,
            0.1,
            1000.0,
        ))
        .with_position(camera_pos)
        .set_orientation_vector(glm::vec3(0.0, 0.0, 0.0) - camera_pos)
        .on(Event::Update, move |camera, ctx| {
            //only run when the camera is active
            if cursor_locked {
                camera.take_input(&ctx.input, ctx.frame.time_delta.as_secs_f32());
            }

            if ctx
                .input
                .mouse_button_just_pressed
                .contains(&glfw::MouseButton::Button2)
            {
                cursor_locked = !cursor_locked;
                toggle_cursor_lock(ctx, cursor_locked);
            }
        })
        .add_child(
            "light",
            NodeBuilder::<Container<f32>>::container(10_f32)
                .add_child(
                    "source",
                    NodeBuilder::<PointLight>::point_light(0.1, 100.0, 1024)
                        .set_color(Color::from_8bit_rgb(255, 255, 255).into())
                        .on(Event::Update, |light, ctx| {
                            if let Some(camera) = ctx.nodes.get_mut::<Camera3D>("camera") {
                                let position = camera.transform.get_forward_vector();
                                if let Some(node) =
                                    camera.get_children().get::<Container<f32>>("light")
                                {
                                    let distance = *node.get_data();
                                    light.get_transform().set_position(position * distance);
                                }
                            }
                        })
                        .add_child(
                            "model",
                            NodeBuilder::<Model>::model_primitive(Primitive::Sphere)
                                .with_scale(glm::vec3(0.1, 0.1, 0.1))
                                .has_lighting(false)
                                .cast_shadows(false)
                                .build(),
                        )
                        .build(),
                )
                .build(),
        )
        .build(),
    )?;

    // engine.context.nodes.add(
    //     "light_group",
    //     NodeBuilder::<Empty>::empty()
    //         .add_child(
    //             "red_light",
    //             NodeBuilder::<PointLight>::point_light(0.1, 100.0, 1024)
    //                 .with_behavior(|light, ctx| {
    //                     let now = Instant::now();
    //                     let elapsed = now.duration_since(ctx.frame.start_time).as_secs_f32();
    //                     light.get_transform().set_position(glm::vec3(
    //                         (elapsed + RAD_120).sin(),
    //                         1.0,
    //                         (elapsed + RAD_120).cos(),
    //                     ));
    //                 })
    //                 .set_color(glm::vec4(1.0, 0.0, 0.0, 1.0))
    //                 .with_position(glm::vec3(1.0, 1.0, -1.0))
    //                 .add_child(
    //                     "model",
    //                     NodeBuilder::<Model>::model_primitive(Primitive::Sphere)
    //                         .with_scale(glm::vec3(0.1, 0.1, 0.1))
    //                         .set_material_base_color(Color::from_8bit_rgb(255, 0, 0).into())
    //                         .has_lighting(false)
    //                         .cast_shadows(false)
    //                         .build(),
    //                 )
    //                 .build(),
    //         )
    //         .add_child(
    //             "green_light",
    //             NodeBuilder::<PointLight>::point_light(0.1, 100.0, 1024)
    //                 .with_behavior(|light, ctx| {
    //                     let now = Instant::now();
    //                     let elapsed = now.duration_since(ctx.frame.start_time).as_secs_f32() * 3.0;
    //                     light.get_transform().set_position(glm::vec3(
    //                         elapsed.sin(),
    //                         1.0,
    //                         elapsed.cos(),
    //                     ));
    //                 })
    //                 .set_color(glm::vec4(0.0, 1.0, 0.0, 1.0))
    //                 .with_position(glm::vec3(-1.0, 1.0, -1.0))
    //                 .add_child(
    //                     "model",
    //                     NodeBuilder::<Model>::model_primitive(Primitive::Sphere)
    //                         .with_scale(glm::vec3(0.1, 0.1, 0.1))
    //                         .set_material_base_color(Color::from_8bit_rgb(0, 255, 0).into())
    //                         .has_lighting(false)
    //                         .cast_shadows(false)
    //                         .build(),
    //                 )
    //                 .build(),
    //         )
    //         .add_child(
    //             "blue_light",
    //             NodeBuilder::<PointLight>::point_light(0.1, 100.0, 1024)
    //                 .with_behavior(|light, ctx| {
    //                     let now = Instant::now();
    //                     let elapsed = now.duration_since(ctx.frame.start_time).as_secs_f32() * 5.0;
    //                     light.get_transform().set_position(glm::vec3(
    //                         (elapsed - RAD_120).sin(),
    //                         1.0,
    //                         (elapsed - RAD_120).cos(),
    //                     ));
    //                 })
    //                 .set_color(glm::vec4(0.0, 0.0, 1.0, 1.0))
    //                 .with_position(glm::vec3(0.0, 1.0, 1.0))
    //                 .add_child(
    //                     "model",
    //                     NodeBuilder::<Model>::model_primitive(Primitive::Sphere)
    //                         .with_scale(glm::vec3(0.1, 0.1, 0.1))
    //                         .set_material_base_color(Color::from_8bit_rgb(0, 0, 255).into())
    //                         .has_lighting(false)
    //                         .cast_shadows(false)
    //                         .build(),
    //                 )
    //                 .build(),
    //         )
    //         .build(),
    // );

    //node_check(container);

    engine.context.nodes.add(
        "bias",
        NodeBuilder::<Container<f32>>::container(0.0).build(),
    )?;

    // simple game manager example
    engine.context.nodes.add(
        "game manager",
        NodeBuilder::<Empty>::empty()
            .on(Event::Ready, |_empty, _ctx| {
                println!("game manager ready");
            })
            .on(Event::Update, move |_game_manager, context| {
                //ran every frame
                if context.input.keys.contains(&glfw::Key::Escape) {
                    context.window.set_should_close(true);
                }

                if context.frame.start_time.elapsed().as_secs_f32()
                    % Duration::from_secs(1).as_secs_f32()
                    == 0.0
                {
                    let fps = context.frame.fps;

                    context
                        .window
                        .set_title(&format!("Hello Pyramid | fps: {}", fps));
                }
            })
            .build(),
    )?;

    // using default shader
    let shader = engine
        .context
        .nodes
        .add_shader("default", Shader::default());

    shader.bind();

    let selected_node = std::rc::Rc::new(std::cell::RefCell::new(String::new()));

    // ui
    let ui = UI::init(&mut engine.context.window);
    engine
        .context
        .nodes
        .add("debug_panel", ui)?
        .define_ui(move |ctx, context| {
            //ui to be drawn every frame
            egui::Window::new("Debug Panel").show(ctx, |ui| {
                for nodes in &mut context.nodes {
                    if ui.button(nodes.0).clicked() {
                        *selected_node.borrow_mut() = nodes.0.clone();
                    };
                }

                ui.label(format!(
                    "{:.2}",
                    context.frame.time_delta.as_secs_f32() * 1000.0
                ));

                if let Some(container) = context.nodes.get_mut::<Container<f64>>("bias") {
                    ui.add(egui::Slider::new(container.get_data_mut(), 0.0..=1.0));
                    let bias_value = *container.get_data() as f32; // Copy the value before dropping the borrow

                    // Now that we've extracted bias_value, the mutable borrow on container is gone
                    if let Some(shader) = context.nodes.get_shader_mut("default") {
                        // Mutably borrow container again now that shader is borrowed
                        shader.bind();
                        shader.set_uniform("u_bias", bias_value);
                    }
                }

                ui.horizontal(|ui| {
                    ui.label("FPS: ");
                    ui.label(format!("{:.2}", context.frame.fps));
                });

                if let Some(light) = context.nodes.get_mut::<PointLight>("camera/light/source") {
                    ui.add(egui::Slider::new(light.get_intensity_mut(), 0.0..=10.0));
                }

                ui.horizontal(|ui| {
                    if let Some(light) = context.nodes.get_mut::<PointLight>("second_light") {
                        let color = light.get_color_mut();
                        ui.add(
                            egui::DragValue::new(&mut color.x)
                                .range(0.0..=1.0)
                                .speed(0.01),
                        );
                        ui.add(
                            egui::DragValue::new(&mut color.y)
                                .range(0.0..=1.0)
                                .speed(0.01),
                        );
                        ui.add(
                            egui::DragValue::new(&mut color.z)
                                .range(0.0..=1.0)
                                .speed(0.01),
                        );
                    }
                });

                // if let Some(node) = context.nodes.get_mut::<CustomNode>("custom") {
                //     let mut transparency = node.transparent;
                //     if let Some(node2) = node.get_children().get_mut::<Model>("childmodel") {
                //         ui.add(
                //             egui::Slider::new(&mut transparency, 0.0..=1.0).text("Transparency"),
                //         );
                //         node2.set_material({
                //             let mut material = MaterialProperties::default();
                //             material.set_base_color_factor(glm::vec4(1.0, 0.0, 0.0, transparency));
                //             material.set_alpha_mode(
                //                 quaturn::context::node_manager::nodes::mesh::AlphaMode::Blend,
                //             );
                //             material.set_double_sided(false);
                //             material
                //         });
                //         node.transparent = transparency;
                //     }
                // }

                // if let Some(model) = context.nodes.get_mut::<CustomNode>("custom") {
                //     if let Some(child) = model.children.get_mut::<Model>("childmodel") {
                //         let mut model_pos = child.get_transform().get_position();
                //         ui.label("Model Position");
                //         ui.horizontal(|ui| {
                //             ui.label("X:");
                //             ui.add(egui::DragValue::new(&mut model_pos.x));
                //             ui.label("Y:");
                //             ui.add(egui::DragValue::new(&mut model_pos.y));
                //             ui.label("Z:");
                //             ui.add(egui::DragValue::new(&mut model_pos.z));
                //         });
                //         child.apply_transform(&mut |t| {
                //             t.set_position(*model_pos);
                //         });
                //     }
                // }

                // if let Some(node) = context.nodes.get_mut::<Camera3D>("camera") {
                //     if let Some(child) = node.get_children_mut().get_mut::<CustomNode>("light") {
                //         ui.add(egui::Slider::new(&mut child.distance, 0.0..=20.0));
                //     }
                // }

                if let Some(camera) = context.nodes.get_mut::<Camera3D>("camera") {
                    let (mut camera_pos_x, mut camera_pos_y, mut camera_pos_z) = (
                        camera.transform.get_position().x,
                        camera.transform.get_position().y,
                        camera.transform.get_position().z,
                    );

                    let (mut camera_rotation_x, mut camera_rotation_y, mut camera_rotation_z) = (
                        camera.get_orientation_angles().x,
                        camera.get_orientation_angles().y,
                        camera.get_orientation_angles().z,
                    );
                    ui.label("Camera Position");
                    ui.horizontal(|ui| {
                        ui.label("X:");
                        ui.add(egui::DragValue::new(&mut camera_pos_x));
                        ui.label("Y:");
                        ui.add(egui::DragValue::new(&mut camera_pos_y));
                        ui.label("Z:");
                        ui.add(egui::DragValue::new(&mut camera_pos_z));
                    });
                    ui.label("Camera Rotation");
                    ui.horizontal(|ui| {
                        ui.label("X:");
                        ui.add(egui::DragValue::new(&mut camera_rotation_x));
                        ui.label("Y:");
                        ui.add(egui::DragValue::new(&mut camera_rotation_y));
                        ui.label("Z:");
                        ui.add(egui::DragValue::new(&mut camera_rotation_z));
                    });
                    ui.add(
                        egui::Slider::new(&mut camera.move_speed, 0.0..=1000.0).text("Move Speed"),
                    );
                    //reassign camera position and rotation from ui
                    // camera.set_position(glm::vec3(camera_pos_x, camera_pos_y, camera_pos_z));
                    // camera.set_orientation_angles(glm::vec3(
                    //     camera_rotation_x,
                    //     camera_rotation_y,
                    //     camera_rotation_z,
                    // ));
                }

                {
                    //extract camera info
                    if let Some(light) = context.nodes.get_mut::<DirectionalLight>("Direct Light") {
                        let mut shadow_distance = light.get_far_plane();
                        ui.add(
                            egui::Slider::new(&mut shadow_distance, 0.0..=1000.0)
                                .text("Shadow Distance"),
                        );
                        light.set_far_plane(shadow_distance);
                    }
                }
                // {
                //     ui.add(egui::Slider::new(&mut bias, 0.0..=0.01).text("Shadow Bias"));
                //     context
                //         .nodes
                //         .shaders
                //         .get_mut(&context.nodes.active_shader)
                //         .unwrap()
                //         .set_uniform1f("u_bias", bias);
                // }
            });

            if !selected_node.borrow().is_empty() {
                egui::Window::new(selected_node.borrow().as_str()).show(ctx, |ui| {
                    ui.label(&*selected_node.borrow());
                    if let Some(node) = context.nodes.get_dyn_mut(&selected_node.borrow()) {
                        ui.horizontal(|ui| {
                            let drag_speed = 0.1;

                            let mut position = node.get_transform().get_position().clone();
                            let initial_position = position.clone();
                            ui.label("Position:");

                            ui.add(egui::DragValue::new(&mut position.x).speed(drag_speed));
                            ui.add(egui::DragValue::new(&mut position.y).speed(drag_speed));
                            ui.add(egui::DragValue::new(&mut position.z).speed(drag_speed));
                            if initial_position != position {
                                node.apply_transform(&mut |t| {
                                    let delta_position = position - initial_position;
                                    t.translate(delta_position);
                                });
                            }
                        });

                        ui.horizontal(|ui| {
                            ui.label("Rotation:");
                            let mut rotation = node.get_transform().get_rotation_euler_xyz();
                            let initial_rotation = rotation.clone();
                            ui.add(egui::DragValue::new(&mut rotation.x));
                            ui.add(egui::DragValue::new(&mut rotation.y));
                            ui.add(egui::DragValue::new(&mut rotation.z));
                            if initial_rotation != rotation {
                                node.apply_transform(&mut |t| {
                                    let delta_rotation = rotation - initial_rotation;
                                    t.rotate_euler_xyz(delta_rotation);
                                });
                            }
                        });

                        ui.horizontal(|ui| {
                            ui.label("Scale:");
                            let mut scale = node.get_transform().get_scale().clone();
                            let initial_scale = scale.clone();

                            ui.add(egui::DragValue::new(&mut scale.x));
                            ui.add(egui::DragValue::new(&mut scale.y));
                            ui.add(egui::DragValue::new(&mut scale.z));
                            if initial_scale != scale {
                                node.apply_transform(&mut |t| {
                                    t.set_scale(scale);
                                });
                            }
                        });
                    }

                    if ui.button("deselect").clicked() {
                        *selected_node.borrow_mut() = String::new();
                    }
                });
            }
        });

    MainScene::new();

    engine.begin()
}

use std::collections::HashMap;

struct Scene {
    nodes: HashMap<String, Box<dyn Node>>,
}

struct MainScene;

impl MainScene {
    fn new() -> Scene {
        Scene {
            nodes: HashMap::new(),
        }
    }
}

// vs...

mod main_scene {
    use super::Scene;
    use std::collections::HashMap;

    pub fn new() -> Scene {
        Scene {
            nodes: HashMap::new(),
        }
    }
}
