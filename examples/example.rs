use std::{default, time::Duration};

use quaturn::context::node_manager::{self};

use quaturn::nodes::model::ModelBuilder;
use quaturn::nodes::point_light::PointLightBuilder;
use quaturn::nodes::{
    model::Primitive, Camera3D, DirectionalLight, Empty, Model, PointLight, UseReadyCallback, UI,
};

use quaturn::components::mesh::MaterialProperties;

use quaturn::components::NodeTransform;

use quaturn::nodes::{NodeBuilder, UseBehaviorCallback};

use quaturn::context::node_manager::{Behavior, Node, NodeManager, Ready, Transformable};
use quaturn::context::GameContext;
use quaturn::renderer::shader::Shader;
use quaturn::utils::color::Color;
use quaturn::Engine;
use quaturn::{egui, glfw, glm};
//use engine::Engine;

const WINDOW_WIDTH: u32 = 1280;
const WINDOW_HEIGHT: u32 = 720;

#[derive(Default, Clone)]
struct CustomNode {
    transform: NodeTransform,
    children: NodeManager,
    pub distance: f32,
}

impl Node for CustomNode {
    fn get_transform(&mut self) -> &mut NodeTransform {
        &mut self.transform
    }

    fn get_children(&mut self) -> &mut NodeManager {
        &mut self.children
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

#[derive(Clone)]
struct Building {
    transform: NodeTransform,
    children: NodeManager,
}

impl Node for Building {
    fn get_transform(&mut self) -> &mut NodeTransform {
        &mut self.transform
    }

    fn get_children(&mut self) -> &mut NodeManager {
        &mut self.children
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
            transform: NodeTransform::default(),
            children: nodes,
        }
    }
}

fn main() {
    let mut engine = Engine::init("Hello Pyramid", WINDOW_WIDTH, WINDOW_HEIGHT);

    engine.set_clear_color(0.5, 0.5, 0.5, 0.5);

    let mut cursor_locked = false;

    let toggle_cursor_lock = |context: &mut GameContext, lock: bool| {
        context.lock_cursor(lock);
    };

    engine.context.nodes.add("building", Building::new());

    // let light = engine
    //     .context
    //     .nodes
    //     .add(
    //         "Point Light",
    //         PointLight::new(NodeTransform::default(), 10, 0.1, 100.0, 1024),
    //     )
    //     .define_behavior(|light, ctx| {
    //         if let Some(camera) = ctx.nodes.get::<Camera3D>("camera") {
    //             let forward = camera.transform.get_forward_vector();

    //             let mut distance = 1.0;

    //             if let Some(node) = light.get_children().get::<CustomNode>("custom") {
    //                 distance = node.distance;
    //             }

    //             light.apply_transform(&mut |t| {
    //                 t.set_position(camera.get_position() + forward * distance);
    //             });
    //         }
    //     });

    engine.context.nodes.add(
        "Point Light",
        NodeBuilder::new(PointLight::new(0.1, 100.0, 1024))
            .with_behavior(|light, ctx| {
                if let Some(camera) = ctx.nodes.get::<Camera3D>("camera") {
                    let forward = camera.transform.get_forward_vector();

                    let mut distance = 1.0;

                    if let Some(node) = light.get_children().get::<CustomNode>("custom") {
                        distance = node.distance;
                    }

                    light.apply_transform(&mut |t| {
                        t.set_position(camera.transform.get_position() + forward * distance);
                    });
                }
            })
            .set_color(Color::from_hex(0xfc1303).into())
            .add_child(
                "light point",
                NodeBuilder::new(Model::new_primitive(Primitive::Sphere))
                    .with_transform(NodeTransform::new(
                        glm::Vec3::default(),
                        glm::Quat::identity(),
                        glm::vec3(0.1, 0.1, 0.1),
                    ))
                    .cast_shadows(false)
                    .has_lighting(false)
                    .build(),
            )
            .add_child("custom", NodeBuilder::new(CustomNode::new()).build())
            .build(),
    );

    // if let Some(light) = engine.context.nodes.get_mut::<PointLight>("Point Light") {
    //     if let Some(model) = light.get_children().get_mut::<Model>("light point") {
    //         println!("{:?}", model.transform);
    //     }
    // }

    // light
    //     .get_children()
    //     .add("light point", Model::new_primitive(Primitive::Sphere))
    //     .apply_transform(&mut |t| {
    //         t.scale(glm::vec3(0.1, 0.1, 0.1));
    //     })
    //     .set_material(
    //         MaterialProperties::default()
    //             .set_base_color_factor(glm::vec4(1.0, 1.0, 1.0, 1.0))
    //             .clone(),
    //     )
    //     .casts_shadows(false)
    //     .has_lighting(false);

    // light.get_children().add("custom", CustomNode::new());

    let camera_pos = glm::vec3(20.0, 20.0, 20.0);

    let camera: &Camera3D = engine
        .context
        .nodes
        .add(
            "camera",
            Camera3D::new(
                0.78539,
                WINDOW_WIDTH as f32 / WINDOW_HEIGHT as f32,
                0.1,
                1000.0,
            ),
        )
        .set_orientation_vector(glm::normalize(&(glm::Vec3::default() - camera_pos)))
        .apply_transform(&mut |t| {
            t.set_position(camera_pos);
        })
        .define_ready(move |camera| {
            //ran before the first frame
            println!("camera ready");
            //camera.set_orientation_vector(glm::Vec3::default() - camera_pos);
        })
        .define_behavior(move |camera, context| {
            // only run when the camera is active
            if cursor_locked {
                camera.take_input(&context.input, context.frame.time_delta.as_secs_f32());
            }

            if context
                .input
                .mouse_button_just_pressed
                .contains(&glfw::MouseButton::Button2)
            {
                cursor_locked = !cursor_locked;
                toggle_cursor_lock(context, cursor_locked);
            }
        });

    // engine.context.nodes.add(
    //     "root",
    //     NodeBuilder::new(Empty::new())
    //         .add_child(
    //             "camera",
    //             NodeBuilder::new(Camera3D::new(1.0, 1.0, 0.1, 100.0)).build(),
    //         )
    //         .build(),
    // );

    // let camera_ptr: *const Camera3D;

    // if let Some(node) = engine.context.nodes.get_mut::<Empty>("root") {
    //     if let Some(camera) = node.get_children().get::<Camera3D>("camera") {
    //         camera_ptr = camera.as_ptr();
    //         engine.context.set_main_camera(camera_ptr);
    //     }
    // }

    // simple game manager example
    engine
        .context
        .nodes
        .add("game manager", Empty::new())
        .define_ready(|_game_manager| {
            //ran before the first frame
            println!("game manager ready");
        })
        .define_behavior(move |_game_manager, context| {
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
        });

    // using default shader
    let shader = engine
        .context
        .nodes
        .add_shader("default", Shader::default());

    shader.bind();
    shader.set_uniform("lightColor", glm::vec4(1.0, 1.0, 1.0, 1.0));

    let selected_node = std::rc::Rc::new(std::cell::RefCell::new(String::new()));

    // ui
    let ui = UI::init(&mut engine.context.window);
    engine
        .context
        .nodes
        .add("debug_panel", ui)
        .define_ui(move |ctx, context| {
            //ui to be drawn every frame
            egui::Window::new("Debug Panel").show(ctx, |ui| {
                for nodes in &mut context.nodes {
                    if ui.button(nodes.0).clicked() {
                        *selected_node.borrow_mut() = nodes.0.clone();
                    };
                }

                ui.horizontal(|ui| {
                    ui.label("FPS: ");
                    ui.label(format!("{:.2}", context.frame.fps));
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

                if let Some(node) = context.nodes.get_mut::<PointLight>("Point Light") {
                    if let Some(child) = node.get_children().get_mut::<CustomNode>("custom") {
                        ui.add(egui::Slider::new(&mut child.distance, 0.0..=20.0));
                    }
                }

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
                    if let Some(node) = context.nodes.get_dyn(&selected_node.borrow()) {
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

    engine.begin();
}
