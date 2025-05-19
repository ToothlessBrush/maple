use maple::context::scene::Scene;
use maple::nodes::ui::UIBuilder;
use maple::nodes::{
    Camera3D, Container, ContainerBuilder, DirectionalLight, Model, NodeBuilder, PointLight,
    UI,
};
use maple::{egui, glfw, math};

pub struct UIScene;

impl UIScene {
    pub fn build(window: &glfw::PWindow) -> Scene {
        let mut scene = Scene::default();

        scene
            .add(
                "debug_panel",
                NodeBuilder::<UI>::create(window)
                    .add_child(
                        "selectedNode",
                        NodeBuilder::<Container<Option<String>>>::create(None).build(),
                    )
                    .build(),
            )
            .expect("failed to create ui")
            .define_ui(move |ctx, context| {
                //ui to be drawn every frame
                egui::Window::new("Debug Panel").show(ctx, |ui| {
                    
                ui.label("Nodes");
                ui.group(|ui| {
                    let node_names: Vec<String> = context.scene.get_all().keys().cloned().collect();
                    for name in &node_names {
                        if ui.button(name).clicked() {
                            if let Some(selected_node) = context.scene.get_mut::<Container<Option<String>>>("debug_panel/selectedNode") {
                                *selected_node.get_item_mut() = Some(name.clone());
                                println!("{}", name);
                            }
                        }
                    }
                });

                // {
                //      let Some(selected) = context.scene.get::<Container<Option<String>>>("debug_panel/selectedNode").map(|n| n.get_item()) else { return };
                //      if let Some(selected_node) = selected.clone() {
                //          if let Some(node) = context.scene.get_dyn_mut(&selected_node) {
                //              ui.group(|ui| {
                //                  ui.label(&selected_node);
                //                  let transform = node.get_transform();
                //                  ui.label("transform");
                //                  ui.horizontal(|ui| {
                //                      ui.add(egui::DragValue::new(&mut transform.position.x));
                //                      ui.add(egui::DragValue::new(&mut transform.position.y));
                //                      ui.add(egui::DragValue::new(&mut transform.position.z));
                //                  });
                //                  ui.label("scale");
                //                  ui.horizontal(|ui| {
                //                      ui.add(egui::DragValue::new(&mut transform.scale.x));
                //                      ui.add(egui::DragValue::new(&mut transform.scale.y));
                //                      ui.add(egui::DragValue::new(&mut transform.scale.z));
                //                  });
                //                  ui.group(|ui| {
                //                      let children: Vec<String> = node.get_children().get_all().keys().cloned().collect();

                //                      ui.label("children");
                //                      for name in children {
                //                          if ui.button(name.clone()).clicked() {
                //                              selected = &Some(format!("{}/{}", selected_node, name));
                //                              println!("{:?}", selected);
                //                          }
                //                      }
                //                      

                //                  })
                //              });
                //          }
                //      }
                //      if let Some(selected_node) = context.scene.get_mut::<Container<Option<String>>>("debug_panel/selectedNode") {
                //          *selected_node.get_item_mut() = selected.clone();
                //      }

                // }

                ui.label(format!(
                    "{:.2}",
                    context.frame.time_delta.as_secs_f32() * 1000.0
                ));

                if let Some(group) = context.scene.get_mut::<Model>("building") {
                    let mut scale = group.transform.scale.x;
                    ui.add(egui::Slider::new(&mut scale, 0.1..=100.0));
                    group.transform.set_scale(math::vec3(scale, scale, scale));
                }

                if let Some(light) = context.scene.get_mut::<DirectionalLight>("direct_light") {
                    ui.label("direct light direction");
                    let mut direction: math::Vec3 = light.direction;
                    ui.horizontal(|ui| {
                        ui.add(egui::DragValue::new(&mut direction.x).speed(0.01));
                        ui.add(egui::DragValue::new(&mut direction.y).speed(0.01));
                        ui.add(egui::DragValue::new(&mut direction.z).speed(0.01));
                    });
                    direction = math::normalize(&direction);
                    light.direction = direction;
                };

                if let Some(container) = context.scene.get_mut::<Container<f32>>("bias") {
                    ui.add(egui::Slider::new(container.get_item_mut(), 0.0..=1.0));
                    let bias_value = *container.get_item(); // Copy the value before dropping the borrow

                    // Now that we've extracted bias_value, the mutable borrow on container is gone
                    if let Some(shader) = context.scene.get_shader_mut("default") {
                        // Mutably borrow container again now that shader is borrowed
                        shader.bind();
                        shader.set_uniform("u_bias", bias_value);
                    }
                }

                ui.horizontal(|ui| {
                    ui.label("FPS: ");
                    ui.label(format!("{:.2}", context.frame.fps));
                });

                //ui.horizontal(add_contents)

                // if let Some(light) = context.scene.get_mut::<PointLight>("camera/light/source") {
                //     ui.add(egui::Slider::new(light.get_intensity_mut(), 0.0..=10.0));
                // }

                ui.horizontal(|ui| {
                    if let Some(light) = context.scene.get_mut::<PointLight>("second_light") {
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

                if let Some(camera) = context.scene.get_mut::<Camera3D>("camera") {
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
                        egui::Slider::new(&mut camera.move_speed, 0.0..=100.0).drag_value_speed(0.5).text("Move Speed"),
                    );
                    //reassign camera position and rotation from ui
                    // camera.set_position(math::vec3(camera_pos_x, camera_pos_y, camera_pos_z));
                    // camera.set_orientation_angles(math::vec3(
                    //     camera_rotation_x,
                    //     camera_rotation_y,
                    //     camera_rotation_z,
                    // ));
                }

                {
                    ui.label("bias offset");
                    ui.add(egui::Slider::new(&mut context.scene_state.bias_offset, 0.0..=0.005).drag_value_speed(0.000001));
                    ui.label("bias factor");
                    ui.add(egui::Slider::new(&mut context.scene_state.bias_factor, 0.0..=0.005).drag_value_speed(0.000001));
                }

                {
                    //extract camera info
                    if let Some(light) = context.scene.get_mut::<DirectionalLight>("Direct Light") {
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
            });

        scene
    }
}
