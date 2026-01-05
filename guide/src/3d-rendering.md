# 3D Rendering

This guide covers creating 3D visual content in Maple, including meshes, materials, and lighting.

## Basic Meshes

Maple provides several built-in primitive meshes:

```rust
// Cube
Mesh3D::cube()
    .position((0.0, 0.0, 0.0))
    .build()

// Sphere
Mesh3D::sphere()
    .position((0.0, 0.0, 0.0))
    .build()

// Smooth sphere (more subdivisions)
Mesh3D::smooth_sphere()
    .position((0.0, 0.0, 0.0))
    .build()

// Plane
Mesh3D::plane()
    .position((0.0, 0.0, 0.0))
    .build()
```

## Transforming Meshes

Use builder methods to transform meshes:

```rust
Mesh3D::cube()
    .position((x, y, z))           // Set position
    .position(Vec3::new(x, y, z))  // Or use Vec3
    .scale(Vec3::new(2.0, 1.0, 2.0))  // Non-uniform scale
    .scale_factor(2.0)              // Uniform scale
    .rotation(Quat::from_euler(...))  // Set rotation
    .build()
```

## Materials

Materials define how surfaces appear. Use `MaterialProperties` to customize appearance:

```rust
use maple::prelude::*;

Mesh3D::cube()
    .material(
        MaterialProperties::default()
            .with_base_color_factor(Color::RED)
    )
    .build()
```

### Material Properties

```rust
MaterialProperties::default()
    .with_base_color_factor(Color::rgb(1.0, 0.0, 0.0))  // Red
    .with_metallic_factor(0.5)     // Metallic appearance
    .with_roughness_factor(0.3)    // Surface roughness
    .with_emissive_factor(Color::WHITE)  // Glow
```

Available colors include predefined constants:
- `Color::RED`
- `Color::GREEN`
- `Color::BLUE`
- `Color::YELLOW`
- `Color::WHITE`
- `Color::BLACK`
- `Color::GREY`

Or create custom colors:
```rust
Color::rgb(r, g, b)     // Values 0.0 to 1.0
Color::rgba(r, g, b, a) // With alpha
```

## Lighting

Lighting is essential for seeing 3D objects. Maple supports directional lights:

### Directional Light

Directional lights simulate sunlight - parallel rays from a direction:

```rust
scene.add(
    "Sun",
    DirectionalLight::builder()
        .direction(Vec3::new(-1.0, -1.0, -1.0))  // Light direction
        .intensity(1.0)                           // Brightness
        .build(),
);
```

### Light Properties

```rust
DirectionalLight::builder()
    .direction((-1.0, -1.0, -0.5))  // Direction vector
    .intensity(1.0)                  // Light intensity (0.0+)
    .bias(0.0001)                    // Shadow bias (prevents shadow acne)
    .build()
```

The direction vector points **toward** where the light is coming from.

## Camera

The camera defines the viewpoint for rendering:

```rust
scene.add(
    "Camera",
    Camera3D::builder()
        .position((0.0, 5.0, -10.0))
        .orientation_vector(Vec3::new(0.0, -0.5, 1.0))  // Look direction
        .fov(1.57)         // Field of view in radians (1.57 ≈ 90°)
        .far_plane(100.0)  // Far clipping plane
        .build(),
);
```

### Camera Properties

- `position()` - Camera location in 3D space
- `orientation_vector()` - Direction the camera looks
- `fov()` - Field of view in radians
- `far_plane()` - Maximum render distance
- `near_plane()` - Minimum render distance (default: 0.1)

## Complete Example

```rust
impl SceneBuilder for MyScene {
    fn build(&mut self) -> Scene {
        let mut scene = Scene::default();

        // Camera
        scene.add(
            "Camera",
            Camera3D::builder()
                .position((0.0, 5.0, -10.0))
                .orientation_vector(Vec3::new(0.0, -0.5, 1.0))
                .far_plane(100.0)
                .build(),
        );

        // Directional light (sun)
        scene.add(
            "Sun",
            DirectionalLight::builder()
                .direction(Vec3::new(-1.0, -1.0, -0.5))
                .intensity(1.0)
                .build(),
        );

        // Colored cube with custom material
        scene.add(
            "Cube",
            Mesh3D::cube()
                .position((0.0, 1.0, 0.0))
                .material(
                    MaterialProperties::default()
                        .with_base_color_factor(Color::BLUE)
                        .with_metallic_factor(0.5)
                        .with_roughness_factor(0.3)
                )
                .build(),
        );

        // Ground plane
        scene.add(
            "Ground",
            Mesh3D::plane()
                .position((0.0, 0.0, 0.0))
                .scale_factor(10.0)
                .material(
                    MaterialProperties::default()
                        .with_base_color_factor(Color::GREY)
                )
                .build(),
        );

        scene
    }
}
```

## Next Steps

- Add [Behaviors](behavior.md) to create interactive scenes
- Implement [Physics](physics.md) for realistic object interactions
- Check out the [Player Controller Example](examples/player-controller.md)
